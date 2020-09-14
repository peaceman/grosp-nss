pub mod tls;

use serde::Deserialize;
use std::net::SocketAddr;

use super::SettingsError;
use tls::{Tls, PartialTls};

#[derive(Debug)]
pub struct Http {
    pub socket: SocketAddr,
    pub tls: Tls,
}

impl Http {
    pub fn new(mut sources: Vec<PartialHttp>) -> Result<Self, SettingsError> {
        let socket: Option<SocketAddr> = sources
            .iter_mut()
            .map(|s| s.socket)
            .fold(Default::default(), |acc, x| acc.or(x));

        let tls_sources = sources
            .iter_mut()
            .map(|s| s.tls.take())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect();

        Ok(Http {
            socket: socket
                .ok_or_else(|| SettingsError::MissingValue("http.socket".to_string()))?,
            tls: Tls::new(tls_sources)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialHttp {
    pub socket: Option<SocketAddr>,
    pub tls: Option<PartialTls>,
}

impl Default for PartialHttp {
    fn default() -> Self {
        PartialHttp { socket: None, tls: None }
    }
}
