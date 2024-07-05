use std::io;

use pikacloud_backend::server;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    println!("Hello, world!");
    server::server().await
}
