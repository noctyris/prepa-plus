use anyhow::Context;
use std::env;

//const BASE: &str = "https://cpgedupuydelome.prepas-plus.fr";
//const LOGIN_URL: &str = "https://cpgedupuydelome.prepas-plus.fr/account/login/";

fn main() {
    let username = "HJAMIER";
    let password = env::var("PREPA_PW").context("Définis PREPA_PW=...");
    match password {
        Ok(val) => println!("{}: {:?}", "PREPA_PW", val),
        Err(e) => println!("Error {}: {}", "PREPA_PW", e),
    }
}

