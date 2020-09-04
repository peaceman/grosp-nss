pub mod bandwidth;

use serde::Deserialize;

use super::SettingsError;
use bandwidth::{Bandwidth, PartialBandwidth};

#[derive(Debug)]
pub struct NodeStats {
    pub bandwidth: Bandwidth,
}

impl NodeStats {
    pub fn new(mut sources: Vec<PartialNodeStats>) -> Result<Self, SettingsError> {
        let bandwidth_sources = sources
            .iter_mut()
            .map(|s| s.bandwidth.take())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect();

        Ok(NodeStats {
            bandwidth: Bandwidth::new(bandwidth_sources)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialNodeStats {
    pub bandwidth: Option<PartialBandwidth>,
}

impl Default for PartialNodeStats {
    fn default() -> Self {
        PartialNodeStats { bandwidth: None }
    }
}
