use anyhow::Context;
use dotenv::dotenv;
use std::env;

//const BASE: &str = "https://cpgedupuydelome.prepas-plus.fr";
//const LOGIN_URL: &str = "https://cpgedupuydelome.prepas-plus.fr/account/login/";

fn main() {
    dotenv().ok();
    let _username = "HJAMIER";
    let password = env::var("PREPA_PW").context("Définis PREPA_PW=...");
}

