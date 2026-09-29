use scraper::{Html, Selector};
use serde::Serialize;
use anyhow::Result;

#[derive(Debug, Serialize, Clone)]
pub struct Semaine {
    pub numero: Option<u8>,
    pub debut:  String,
    pub fin:    String,
    pub notes:  Vec<Note>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Note {
    pub matiere:    String,
    pub note:       Option<f32>,
    pub professeur: String,
    pub rang:       Rang,
    pub moyenne:    Option<f32>,
    pub ecart_type: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Rang {
    pub rang:  Option<u16>,
    pub total: Option<u16>,
}

fn parse_fr(s: &str) -> Option<f32> {
    s.trim().replace(",", ".").parse().ok()
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

pub fn parse_notes(html: &str) -> Result<Vec<Semaine>> {
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
