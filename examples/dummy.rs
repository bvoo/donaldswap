use obws::Client;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::connect("localhost", 4455, Some("password")).await?;
    client.scenes().set_current_program_scene("My Scene").await?;
    Ok(())
}
