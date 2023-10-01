pub struct Config {
    pub(crate) cinegestion_login: String,
    pub(crate) cinegestion_password: String,
}

impl Config {
    pub fn load() -> Config {
        Config {
            cinegestion_login: std::env::var("CINEGESTION_LOGIN").expect("Miss CINEGESTION_LOGIN"),
            cinegestion_password: std::env::var("CINEGESTION_PASSWORD").expect("Miss CINEGESTION_PASSWORD"),
        }
    }
}