use config::{ConfigError, Config};
use serde::Deserialize;

lazy_static::lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().unwrap();
}

#[derive(Debug, Deserialize, Clone)]
pub struct Finnhub {
    pub api_key: String,
    test_key: String,
}

const CONFIG_FILE_PATH: &str = "./config/Postulacion.toml";

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub finnhub: Finnhub,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let settings = Config::builder()
        .add_source(config::File::with_name(CONFIG_FILE_PATH))
        .build()?;

        settings.try_deserialize()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let settings = Settings::new().unwrap();
        assert_eq!(settings.finnhub.test_key, "test_key_value");
    }
}
