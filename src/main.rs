use anyhow::{Context, Result};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use dotenv::dotenv;
use std::env;

const BASE: &str = "https://cpgedupuydelome.prepas-plus.fr";
const LOGIN_URL: &str = "https://cpgedupuydelome.prepas-plus.fr/account/login/";

#[derive(Debug, Serialize, Clone)]
struct Semaine {
    numero: Option<u8>,
    debut:  String,
    fin:    String,
    notes:  Vec<Note>,
}

#[derive(Debug, Serialize, Clone)]
struct Note {
    matiere:    String,
    note:       Option<f32>,
    professeur: String,
    rang:       Rang,
    moyenne:    Option<f32>,
    ecart_type: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
struct Rang {
    rang:  Option<u16>,
    total: Option<u16>,
}

/// "13,27" -> 13.27 (virgule décimale française)
fn parse_fr(s: &str) -> Option<f32> {
    s.trim().replace(',', ".").parse().ok()
}

/// "S1: 14/09-18/09" -> (1, "14/09", "18/09")
fn parse_semaine(s: &str) -> (Option<u8>, String, String) {
    let (tag, dates) = match s.split_once(':') {
        Some((t, d)) => (t.trim().trim_start_matches('S').to_string(), d.trim()),
        None => (String::new(), s.trim()),
    };
    let (debut, fin) = match dates.split_once('-') {
        Some((d, f)) => (d.trim().to_string(), f.trim().to_string()),
        None => (dates.to_string(), String::new()),
    };
    (tag.parse::<u8>().ok(), debut, fin)
}

/// "Philippe Eric; Rg:13/46; Moy:13,27; ET:1,94" -> (prof, rang, moyenne, écart-type)
fn parse_detail(d: &str) -> (String, Rang, Option<f32>, Option<f32>) {
    let mut prof = String::new();
    let mut rang = Rang { rang: None, total: None };
    let mut moyenne = None;
    let mut et = None;

    for part in d.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("Rg:") {
            let mut it = rest.splitn(2, '/');
            rang.rang = it.next().and_then(|s| s.trim().parse().ok());
            rang.total = it.next().and_then(|s| s.trim().parse().ok());
        } else if let Some(v) = part.strip_prefix("Moy:") {
            moyenne = parse_fr(v);
        } else if let Some(v) = part.strip_prefix("ET:") {
            et = parse_fr(v);
        } else if !part.is_empty() {
            prof = part.to_string();
        }
    }
    (prof, rang, moyenne, et)
}

fn get_notes() -> Result<Vec<Semaine>> {
    // Read dotfile
    dotenv().ok();

    // Get username and password
    let username = "HJAMIER";
    let password = env::var("PREPA_PW").context("Définis PREPA_PW=...")?;

    // Build web session to stay connected
    let client = Client::builder()
        .cookie_store(true)
        .build()?;

    // GET login page
    let page = client.get(LOGIN_URL).send()?.text()?;

    // Extract CSRF
    let re = regex::Regex::new(r#"name="csrfmiddlewaretoken" value="([^"]+)""#)?;
    let csrf = re
        .captures(&page)
        .context("Token CSRF introuvable")?
        .get(1)
        .map(|m| m.as_str().to_string())
        .context("Token CSRF introuvable")?;

    // POST login
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

    // GET mes_notes + parse
    let notes_page = client
        .get(format!("{BASE}/colles/mes_notes"))
        .send()?
        .text()?;
    parse_notes(&notes_page)
}

fn parse_notes(html: &str) -> Result<Vec<Semaine>> {
    let doc = Html::parse_document(html);
    let table_sel = Selector::parse("table").unwrap();
    let th_sel = Selector::parse("th").unwrap();
    let row_sel = Selector::parse("tr").unwrap();
    let cell_sel = Selector::parse("th, td").unwrap();
    let span_sel = Selector::parse("span").unwrap();

    let mut semaines: Vec<Semaine> = Vec::new();

    for table in doc.select(&table_sel) {
        let Some(th) = table.select(&th_sel).next() else { continue };
        if th.text().collect::<String>().trim() != "Semaine" { continue; }

        let rows: Vec<_> = table.select(&row_sel).collect();
        let headers: Vec<String> = rows[0]
            .select(&th_sel)
            .map(|h| h.text().collect::<String>().trim().to_string())
            .collect();

        for row in rows.iter().skip(1) {
            let cells: Vec<_> = row.select(&cell_sel).collect();
            if cells.is_empty() { continue; }
            let raw = cells[0].text().collect::<String>().trim().to_string();
            let (numero, debut, fin) = parse_semaine(&raw);

            let mut notes = Vec::new();
            for (matiere, cell) in headers[1..].iter().zip(cells[1..].iter()) {
                let Some(span) = cell.select(&span_sel).next() else { continue };
                let note_txt = span.text().collect::<String>().trim().to_string();
                let detail = span.value().attr("title").unwrap_or("").to_string();
                let (professeur, rang, moyenne, ecart_type) = parse_detail(&detail);

                notes.push(Note {
                    matiere: matiere.clone(),
                    note: parse_fr(&note_txt),
                    professeur,
                    rang,
                    moyenne,
                    ecart_type,
                });
            }
            semaines.push(Semaine { numero, debut, fin, notes });
        }
    }
    Ok(semaines)
}

fn main() -> Result<()> {
    let semaines = get_notes()?;
    println!("{}", serde_json::to_string_pretty(&semaines)?);
    Ok(())
}
