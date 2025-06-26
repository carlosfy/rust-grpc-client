use crate::audio::AudioChunk;
use futures_util::stream::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::metadata; // For .next()

use crate::audio::{AudioMessage, AudioMetadata, carlos_client::CarlosClient};

pub mod audio {
    tonic::include_proto!("audio");
}

const CHUNK_SIZE: usize = 1024 * 64;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // -- Connect to the server --
    let mut client = CarlosClient::connect("http://[::1]:50051").await?;

    // Preapare the audio data stream
    let (tx, rx) = mpsc::channel(4);

    tokio::spawn(async move {
        let mut reader = hound::WavReader::open("sample.wav").expect("Failed to open WAV file");
        let spec = reader.spec();

        println!("WAV file spec: {:?}", spec);

        // 1. Send the metadata message first
        let metadata = AudioMetadata {
            sample_rate: spec.sample_rate,
            channels: spec.channels as i32,
            bit_depth: spec.bits_per_sample as i32,
        };

        tx.send(AudioMessage {
            message_type: Some(audio::audio_message::MessageType::Metadata(metadata)),
        })
        .await
        .unwrap();
        println!("Sent Metadata.");

        // 2. Read samples and send them in chunks
        let samples_16bit: Vec<i16> = reader.samples().filter_map(Result::ok).collect();

        // Convert i16 samples to a byte vector (u8)
        let pcm_data: Vec<u8> = samples_16bit
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect();

        for chunk_bytes in pcm_data.chunks(CHUNK_SIZE) {
            let chunk = AudioChunk {
                pcm_data: chunk_bytes.to_vec(),
            };

            tx.send(AudioMessage {
                message_type: Some(audio::audio_message::MessageType::Chunk(chunk)),
            })
            .await
            .unwrap();
            println!("Sent chunk of size {} bytes.", chunk_bytes.len());

            // Add a small delay to simulate real-time streaming
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        println!("Finished sending all audio chunks.");
    });

    // --- Make the RPC call and handle the response stream ---
    let request = Request::new(ReceiverStream::new(rx));
    let mut response_stream = client.process_audio(request).await?.into_inner();

    // Listen for incoming messages from the server
    while let Some(received) = response_stream.next().await {
        match received {
            Ok(text_result) => {
                println!("[SERVER RESPONSE]: {}", text_result.text_chunk);
            }
            Err(status) => {
                eprintln!("ERROR from server: {}", status);
                break;
            }
        }
    }

    print!("hi");
    Ok(())
}
