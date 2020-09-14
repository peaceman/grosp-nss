use serde::Deserialize;

use super::SettingsError;

#[derive(Debug)]
pub struct Tls {
    pub server_cert_file: String,
    pub server_key_file: String,
    pub ca_cert_file: String,
}

impl Tls {
    pub fn new(mut sources: Vec<PartialTls>) -> Result<Self, SettingsError> {
        let merged: PartialTls = sources
            .iter_mut()
            .fold(Default::default(), |acc, x| PartialTls {
                server_cert_file: acc.server_cert_file.or_else(|| x.server_cert_file.take()),
                server_key_file: acc.server_key_file.or_else(|| x.server_key_file.take()),
                ca_cert_file: acc.ca_cert_file.or_else(|| x.ca_cert_file.take()),
            });

        Ok(Tls {
            server_cert_file: merged
                .server_cert_file
                .ok_or_else(|| SettingsError::MissingValue("tls.server_cert_file".into()))?,
            server_key_file: merged
                .server_key_file
                .ok_or_else(|| SettingsError::MissingValue("tls.server_key_file".into()))?,
            ca_cert_file: merged
                .ca_cert_file
                .ok_or_else(|| SettingsError::MissingValue("tls.ca_cert_file".into()))?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialTls {
    pub server_cert_file: Option<String>,
    pub server_key_file: Option<String>,
    pub ca_cert_file: Option<String>,
}

impl Default for PartialTls {
    fn default() -> Self {
        PartialTls {
            server_cert_file: None,
            server_key_file: None,
            ca_cert_file: None,
        }
    }
}
