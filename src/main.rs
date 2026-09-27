use anyhow::{Context, Result};
use scraper::{Html, Selector};
use dioxus::prelude::*;
use serde::Serialize;

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

fn main() {
    launch(app);
}

fn app() -> Element {
    let mut username = use_signal(|| "HJAMIER".to_string());
    let mut password = use_signal(String::new);
    let mut semaines = use_signal(|| None::<Vec<Semaine>>);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    rsx! {
        div { class: "container",
            h1 { "Prépa+" }
            input {
                value: "{username}",
                oninput: move |e| username.set(e.value()),
                placeholder: "Identifiant",
            }
            input {
                r#type: "password",
                value: "{password}",
                oninput: move |e| password.set(e.value()),
                placeholder: "Mot de passe",
            }
            button {
                disabled: loading(),
                onclick: move |_| async move {
                    loading.set(true);
                    error.set(None);
                    match get_notes(&username.read(), &password.read()).await {
                        Ok(s) => semaines.set(Some(s)),
                        Err(e) => error.set(Some(e.to_string())),
                    }
                    loading.set(false);
                },
                if loading() { "Chargement..." } else { "Voir mes notes" }
            }
            if let Some(err) = error() {
                p { class: "error", "{err}" }
            }
            if let Some(semaines) = semaines() {
                for sem in semaines {
                    div { class: "card",
                        h3 { "Semaine {sem.numero:?} — {sem.debut} → {sem.fin}" }
                        for n in &sem.notes {
                            div { class: "row",
                                span { class: "matiere", "{n.matiere}" }
                                span { class: "note", "{n.note:?}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn parse_fr(s: &str) -> Option<f32> {
    s.trim().replace(',', ".").parse().ok()
}

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

async fn get_notes(username: &str, password: &str) -> Result<Vec<Semaine>> {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()?;

    let page = client.get(LOGIN_URL).send().await?.text().await?;

    let re = regex::Regex::new(r#"name="csrfmiddlewaretoken" value="([^"]+)""#)?;
    let csrf = re
        .captures(&page)
        .context("Token CSRF introuvable")?
        .get(1)
        .context("Token CSRF introuvable")?
        .as_str()
        .to_string();

    let params = [
        ("csrfmiddlewaretoken", csrf.as_str()),
        ("login_view-current_step", "auth"),
        ("auth-username", username),
        ("auth-password", password),
    ];
    let r = client
        .post(LOGIN_URL)
        .header("Referer", LOGIN_URL)
        .form(&params)
        .send()
        .await?;
    if r.url().as_str().contains("account/login") {
        anyhow::bail!("Échec de connexion");
    }

    let notes_page = client
        .get(format!("{BASE}/colles/mes_notes"))
        .send()
        .await?
        .text()
        .await?;
    parse_notes(&notes_page)
}
