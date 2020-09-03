fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(&["proto/nodestats/node_stats.proto"], &["proto/nodestats"])?;

    Ok(())
}
