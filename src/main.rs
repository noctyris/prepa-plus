use serde::{Deserialize, Serialize};
use webpki_roots::TLS_SERVER_ROOTS;
use anyhow::{Context, Result};
use rustls::RootCertStore;
use dioxus::prelude::*;

use crate::parse::Semaine;
mod parse;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Creds {
    username: String,
    password: String,
}

const BASE: &str = "https://cpgedupuydelome.prepas-plus.fr";
const LOGIN_URL: &str = "https://cpgedupuydelome.prepas-plus.fr/account/login/";

fn main() {
    launch(app);
}

fn app() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut semaines = use_signal(|| None::<Vec<Semaine>>);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    let data = use_resource(move || {
        let creds = load_creds();
        async move {
            match creds {
                Some(c) => get_notes(&c.username, &c.password).await.ok(),
                None => None,
            }
        }
    });

    use_effect(move || {
        if let Some(Some(s)) = data() {
            semaines.set(Some(s));
        }
    });

    rsx! {
        div { class: "container",
            img { src: asset!("icons/128x128.png"), class: "logo" }
            h1 { 
                "Prépa+"
            }
            if semaines().is_none() {
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
                            Ok(s) => {
                                semaines.set(Some(s));
                                let _ = save_creds(&Creds {
                                    username: username.read().clone(),
                                    password: password.read().clone(),
                                });
                            },
                            Err(e) => error.set(Some(e.to_string())),
                        }
                        loading.set(false);
                    },
                    if loading() { "Chargement..." } else { "Voir mes notes" }
                }
                if let Some(e) = error() {
                    p { class: "error", "{e}" }
                }
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

#[cfg(target_os = "android")]
fn cookies_path() -> std::path::PathBuf {
    use jni::objects::JObject;

    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.expect("JavaVM");
    let mut env = vm.attach_current_thread().expect("JNIEnv");
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };

    let file = env
        .call_method(&activity, "getFilesDir", "()Ljava/io/File;", &[])
        .expect("getFilesDir")
        .l()
        .expect("JObject");
    let path = env
        .call_method(&file, "getAbsolutePath", "()Ljava/lang/String;", &[])
        .expect("getAbsolutePath")
        .l()
        .expect("JString");
    let s = env.get_string((&path).into()).expect("str");
//    let mut p = std::path::PathBuf::from(s.to_string_lossy().to_string());
    let p = std::path::PathBuf::from(s.to_string_lossy().to_string());
    p
}

#[cfg(not(target_os = "android"))]
fn cookies_path() -> std::path::PathBuf {
    let mut p = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    p.push("prepa-plus");
    p
}

fn data_dir() -> std::path::PathBuf {
    cookies_path().parent().unwrap().to_path_buf()
}

fn load_creds() -> Option<Creds> {
    let bytes = std::fs::read(data_dir().join("creds.json")).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn save_creds(c: &Creds) -> Result<()> {
    std::fs::create_dir_all(data_dir())?;
    let json = serde_json::to_vec(c)?;
    std::fs::write(data_dir().join("creds.json"), json)?;
    Ok(())
}

async fn get_notes(username: &str, password: &str) -> Result<Vec<Semaine>> {
    let mut roots = RootCertStore::empty();
    roots.extend(TLS_SERVER_ROOTS.iter().cloned());

    let tls = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .tls_backend_preconfigured(tls)
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
    parse::parse_notes(&notes_page)
}
