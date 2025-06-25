use crate::audio::audio_processor_client::AudioProcessorClient;

pub mod audio {
    tonic::include_proto!("audio");
}

const CHUNK_SIZE: usize = 1024 * 64;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("hi");
    Ok(())
}
