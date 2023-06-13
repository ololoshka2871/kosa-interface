use std::time::Duration;

use kosa_interface::Kosa;


#[tokio::main(flavor = "current_thread")]
async fn main() -> tokio_serial::Result<()> {
    let mut kosa = Kosa::new("COM5");

    loop {
        match kosa.get_measurement(Duration::from_millis(50)).await {
            Ok(res) => println!("Result: {}", res),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
