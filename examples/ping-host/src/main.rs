use std::time::Duration;

use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("Pinging the host service to check liveness.");

    loop {
        match ping().await {
            Ok(status) => println!("{status}"),
            Err(err) => println!("Error while pinging status: {:?}", err),
        }

        sleep(Duration::from_secs(5)).await
    }
}

async fn ping() -> Result<String, reqwest::Error> {
    let status = reqwest::get("http://host.containers.internal:8000/status")
        .await?
        .text()
        .await?;
    Ok(status)
}
