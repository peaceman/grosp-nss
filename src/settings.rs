mod error;
mod http;

use error::*;
use http::*;

use std::fs::File;
use std::io::Read;

use serde::Deserialize;

#[derive(Debug)]
pub struct Settings {
    pub http: Http,
}

impl Settings {
    pub fn from_file(file_path: &str) -> Result<Self, SettingsError> {
        let reader = File::open(file_path).map_err(|e| SettingsError::FileParse {
            path: Some(file_path.to_string()),
            cause: Box::new(e),
        })?;

        Settings::from_reader(reader)
    }

    pub fn from_reader<T: Read>(reader: T) -> Result<Self, SettingsError> {
        let file_settings: PartialSettings =
            serde_yaml::from_reader(reader).map_err(|e| SettingsError::FileParse {
                path: None,
                cause: Box::new(e),
            })?;

        Settings::merge(vec![file_settings, Default::default()])
    }

    pub fn merge(mut sources: Vec<PartialSettings>) -> Result<Self, SettingsError> {
        let http_sources = sources
            .iter_mut()
            .map(|s| s.http.take())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect();

        Ok(Settings {
            http: Http::new(http_sources)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialSettings {
    http: Option<PartialHttp>,
}

impl Default for PartialSettings {
    fn default() -> Self {
        PartialSettings {
            http: Some(Default::default()),
        }
    }
}
