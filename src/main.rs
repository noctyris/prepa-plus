use anyhow::{Context, Result};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use dotenv::dotenv;
use std::env;

const BASE: &str = "https://cpgedupuydelome.prepas-plus.fr";
const LOGIN_URL: &str = "https://cpgedupuydelome.prepas-plus.fr/account/login/";

fn main() -> Result<()> {
    dotenv().ok();
    let username = "HJAMIER";
    // Temporaire : mot de passe via variable d'environnement
    let password = env::var("PREPA_PW").context("Définis PREPA_PW=...")?;

    // reqwest::blocking est plus simple pour un CLI
    let client = Client::builder()
        .cookie_store(true)  // équivalent de requests.Session()
        .build()?;

    // 1. GET login page
    let page = client.get(LOGIN_URL).send()?.text()?;
    // 2. Extraire le CSRF
    let re = regex::Regex::new(r#"name="csrfmiddlewaretoken" value="([^"]+)""#)?;
    let csrf = re
        .captures(&page)
        .context("Token CSRF introuvable")?
        .get(1)
        .map(|m| m.as_str().to_string())
        .context("Token CSRF introuvable")?;

    // 3. POST login
    let params = [
        ("csrfmiddlewaretoken", csrf.as_str()),
        ("login_view-current_step", "auth"),
        ("auth-username", username),
        ("auth-password", password.as_str()),
    ];
    let r = client
        .post(LOGIN_URL)
        .header("Referer", LOGIN_URL)
        .form(&params)
        .send()?;
    if r.url().as_str().contains("account/login") {
        println!("{}", r.status());
        anyhow::bail!("Échec de connexion");
    }

    // 4. GET mes_notes + parser
    let notes_page = client
        .get(format!("{BASE}/colles/mes_notes"))
        .send()?
        .text()?;
    parse_notes(&notes_page);
    Ok(())
}

fn parse_notes(html: &str) {
    let doc = Html::parse_document(html);
    // table dont le premier th == "Semaine"
    let table_sel = Selector::parse("table").unwrap();
    let th_sel = Selector::parse("th").unwrap();
    let row_sel = Selector::parse("tr").unwrap();
    let cell_sel = Selector::parse("th, td").unwrap();
    let span_sel = Selector::parse("span").unwrap();

    for table in doc.select(&table_sel) {
        let first_th = table.select(&th_sel).next();
        let Some(th) = first_th else { continue };
        if th.text().collect::<String>().trim() != "Semaine" { continue; }

        let rows: Vec<_> = table.select(&row_sel).collect();
        let headers: Vec<String> = rows[0]
            .select(&th_sel)
            .map(|h| h.text().collect::<String>().trim().to_string())
            .collect();

        for row in rows.iter().skip(1) {
            let cells: Vec<_> = row.select(&cell_sel).collect();
            if cells.is_empty() { continue; }
            let semaine = cells[0].text().collect::<String>().trim().to_string();
            println!("=== {semaine} ===");
            for (matiere, cell) in headers[1..].iter().zip(cells[1..].iter()) {
                let Some(span) = cell.select(&span_sel).next() else { continue };
                let note = span.text().collect::<String>().trim().to_string();
                let details = span.value().attr("title").unwrap_or("");
                println!("  {matiere}: {note} ({details})");
            }
        }
    }
}
