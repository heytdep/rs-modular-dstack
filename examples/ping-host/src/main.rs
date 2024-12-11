use std::time::Duration;

use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let host_address = std::env::var("HOST").unwrap_or("host.containers.internal:8000".into());
    println!("Pinging the host service to check liveness.");

    loop {
        match ping(&host_address).await {
            Ok(status) => println!("{status}"),
            Err(err) => println!("Error while pinging status: {:?}", err),
        }

        sleep(Duration::from_secs(5)).await
    }
}

async fn ping(host_address: &str) -> Result<String, reqwest::Error> {
    let status = reqwest::get(format!("http://{}/status", host_address))
        .await?
        .text()
        .await?;
    Ok(status)
}
