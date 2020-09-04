use serde::Deserialize;
use std::net::{SocketAddr};

use super::SettingsError;

#[derive(Debug)]
pub struct Http {
    pub socket: SocketAddr,
}

impl Http {
    pub fn new(sources: Vec<PartialHttp>) -> Result<Self, SettingsError> {
        let merged: PartialHttp = sources
            .iter()
            .fold(Default::default(), |acc, x| PartialHttp {
                socket: acc.socket.or(x.socket),
            });

        Ok(Http {
            socket: merged
                .socket
                .ok_or_else(|| SettingsError::MissingValue("http.socket".to_string()))?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialHttp {
    pub socket: Option<SocketAddr>,
}

impl Default for PartialHttp {
    fn default() -> Self {
        PartialHttp { socket: None }
    }
}
