use dioxus::prelude::*;

mod parse;
mod fetch;
use parse::Semaine;

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
        let creds = fetch::load_creds();
        async move {
            match creds {
                Some(c) => fetch::get_notes(&c.username, &c.password).await.ok(),
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
                        match fetch::get_notes(&username.read(), &password.read()).await {
                            Ok(s) => {
                                semaines.set(Some(s));
                                let _ = fetch::save_creds(&fetch::Creds {
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
