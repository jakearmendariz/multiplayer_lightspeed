use log::info;

use actix_files::Files;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

mod message;
mod server;
mod session;
mod lightspeed;

use session::WsChatSession;

#[macro_use]
extern crate lazy_static;

async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    ws::start(WsChatSession::default(), &req, stream)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let addr = "0.0.0.0:8080";

    let srv = HttpServer::new(move || {
        App::new()
            .service(web::resource("/ws/").to(chat_route))
            .service(Files::new("/", "./static/").index_file("lightspeed.html"))
    })
    .bind(&addr)?;

    info!("Starting http server: {}", &addr);

    srv.run().await
}
