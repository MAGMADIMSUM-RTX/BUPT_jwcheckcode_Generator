mod models;
mod database;
mod qr_parser;
mod time_utils;
mod handlers;
mod server;

use server::start_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    start_server().await
}
