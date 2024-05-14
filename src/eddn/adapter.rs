use bus::Bus;
use std::io;
use std::io::Read;
use std::time::Duration;

use flate2::read::ZlibDecoder;
use json::JsonValue;
use log::error;

pub struct EddnAdapter {
    pub bus_writer: Bus<JsonValue>,
}

impl EddnAdapter {
    pub async fn subscribe_to_eddn(mut self) {
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        subscriber
            .connect(std::env::var("ZEROMQ_URL").unwrap().as_str())
            .expect("Failed to connect to ZeroMQ Server");
        subscriber
            .set_subscribe(b"")
            .expect("Failed to subscribe to channel");

        tokio::time::sleep(Duration::from_millis(1)).await;

        loop {
            let data = subscriber.recv_bytes(0).expect("Failed receiving update");

            let message = decode_reader(data).unwrap();

            if message == "END" {
                break;
            }

            match json::parse(message.as_str()) {
                Ok(json) => {
                    let result = self.bus_writer.try_broadcast(json["message"].clone());
                    match result {
                        Ok(_) => {}
                        Err(_) => {
                            error!("Channel is full");
                        }
                    }
                }
                Err(error) => {
                    error!("Error parsing json: {}", error);
                }
            }
        }
    }
}

// Uncompresses a Zlib Encoded vector of bytes and returns a string or error
// Here &[u8] implements Read
fn decode_reader(bytes: Vec<u8>) -> io::Result<String> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
}
