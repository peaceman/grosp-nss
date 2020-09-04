use std::time::Duration;
use std::rc::Rc;

use serde::Deserialize;

use crate::settings::SettingsError;

#[derive(Debug)]
pub struct Bandwidth {
    pub tx_file: String,
    pub rx_file: String,
    pub update_interval: Duration,
}

impl Bandwidth {
    pub fn new(sources: Vec<PartialBandwidth>) -> Result<Self, SettingsError> {
        let merged: PartialBandwidth = sources
            .iter()
            .fold(Default::default(), |acc, x| PartialBandwidth {
                tx_file: acc.tx_file.or_else(|| x.tx_file.clone()),
                rx_file: acc.rx_file.or_else(|| x.rx_file.clone()),
                update_interval: acc.update_interval.or(x.update_interval),
            });

        let mut tx_file = merged.tx_file.ok_or_else(|| SettingsError::MissingValue("bandwidth.tx_file".into()))?;
        let mut rx_file = merged.rx_file.ok_or_else(|| SettingsError::MissingValue("bandwidth.rx_file".into()))?;

        Rc::make_mut(&mut tx_file);
        Rc::make_mut(&mut rx_file);

        let tx_file = Rc::try_unwrap(tx_file).unwrap();
        let rx_file = Rc::try_unwrap(rx_file).unwrap();

        Ok(Bandwidth {
            tx_file,
            rx_file,
            update_interval: merged.update_interval.ok_or_else(|| SettingsError::MissingValue("bandwidth.update_interval".into()))?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialBandwidth {
    pub tx_file: Option<Rc<String>>,
    pub rx_file: Option<Rc<String>>,

    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub update_interval: Option<Duration>,
}

impl Default for PartialBandwidth {
    fn default() -> Self {
        PartialBandwidth {
            tx_file: None,
            rx_file: None,
            update_interval: None,
        }
    }
}
