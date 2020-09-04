use std::fs;
use std::path::PathBuf;

use log::warn;

pub trait CounterSource: Send + Sync {
    fn get_rx(&self) -> u64;
    fn get_tx(&self) -> u64;
}

pub struct FileCounterSource {
    rx_file: PathBuf,
    tx_file: PathBuf,
}

impl FileCounterSource {
    fn new<T: Into<PathBuf>>(rx_file: T, tx_file: T) -> Self {
        FileCounterSource {
            rx_file: rx_file.into(),
            tx_file: tx_file.into(),
        }
    }
}

impl CounterSource for FileCounterSource {
    fn get_rx(&self) -> u64 {
        fs::read_to_string(&self.rx_file)
            .map_err(anyhow::Error::new)
            .and_then(|file_content| file_content.parse().map_err(anyhow::Error::new))
            .unwrap_or_else(|e| {
                warn!("Failed to get rx value from file: {:?}", e);

                0
            })
    }

    fn get_tx(&self) -> u64 {
        fs::read_to_string(&self.tx_file)
            .map_err(anyhow::Error::new)
            .and_then(|file_content| file_content.parse().map_err(anyhow::Error::new))
            .unwrap_or_else(|e| {
                warn!("Failed to get rx value from file: {:?}", e);

                0
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::error::Error;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use rand::{thread_rng, Rng};

    struct TestFile {
        file_path: Box<Path>,
    }

    impl TestFile {
        fn new(content: &str) -> Result<TestFile, Box<dyn Error>> {
            let file_path = TestFile::gen_file_path();
            let mut file = File::create(&file_path)?;
            file.write_all(content.as_bytes())?;

            Ok(TestFile {
                file_path: file_path.into_boxed_path(),
            })
        }

        fn gen_file_path() -> PathBuf {
            let filename: String = thread_rng()
                .sample_iter(rand::distributions::Alphanumeric)
                .take(8)
                .collect();

            let mut path = env::temp_dir();
            path.set_file_name(filename);

            path
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            if let Err(e) = fs::remove_file(&self.file_path) {
                eprintln!(
                    "Failed to remove tmp file: {}\n{}",
                    self.file_path.to_string_lossy(),
                    e
                );
            }
        }
    }

    #[test]
    fn test_non_existent_files() {
        let source = FileCounterSource::new("invalid", "invalid");

        assert_eq!(0, source.get_rx());
        assert_eq!(0, source.get_tx());
    }

    #[test]
    fn test_regular() {
        let rx_file = TestFile::new("23").unwrap();
        let tx_file = TestFile::new("5").unwrap();

        let source = FileCounterSource::new(rx_file.file_path.as_ref(), tx_file.file_path.as_ref());

        assert_eq!(23, source.get_rx());
        assert_eq!(5, source.get_tx());
    }

    #[test]
    fn test_non_integer() {
        let rx_file = TestFile::new("foobar").unwrap();
        let tx_file = TestFile::new("23.5").unwrap();

        let source = FileCounterSource::new(rx_file.file_path.as_ref(), tx_file.file_path.as_ref());

        assert_eq!(0, source.get_rx());
        assert_eq!(0, source.get_tx());
    }

    #[test]
    fn test_file_content_change() {
        let rx_file = TestFile::new("23").unwrap();
        let tx_file = TestFile::new("23").unwrap();

        let source = FileCounterSource::new(rx_file.file_path.as_ref(), tx_file.file_path.as_ref());

        assert_eq!(23, source.get_rx());
        assert_eq!(23, source.get_tx());

        fs::write(rx_file.file_path.as_ref(), "42").unwrap();
        fs::write(tx_file.file_path.as_ref(), "33").unwrap();

        assert_eq!(42, source.get_rx());
        assert_eq!(33, source.get_tx());
    }
}
