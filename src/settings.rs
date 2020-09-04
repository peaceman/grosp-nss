mod error;
mod http;
mod node_stats;

use error::*;
use http::*;
use node_stats::*;
use node_stats::bandwidth::*;

use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use serde::Deserialize;

#[derive(Debug)]
pub struct Settings {
    pub http: Http,
    pub node_stats: NodeStats,
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

        let node_stats_sources = sources
            .iter_mut()
            .map(|s| s.node_stats.take())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect();

        Ok(Settings {
            http: Http::new(http_sources)?,
            node_stats: NodeStats::new(node_stats_sources)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialSettings {
    http: Option<PartialHttp>,
    node_stats: Option<PartialNodeStats>,
}

impl Default for PartialSettings {
    fn default() -> Self {
        PartialSettings {
            http: Some(PartialHttp {
                socket: Some(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 2351)),
            }),
            node_stats: Some(PartialNodeStats {
                bandwidth: Some(PartialBandwidth {
                    tx_file: None,
                    rx_file: None,
                    update_interval: Some(Duration::from_secs(5)),
                })
            }),
        }
    }
}
