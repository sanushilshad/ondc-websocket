
use actix::Actor;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use crate::schemas:: Settings;
use crate::routes::routes;
use crate::websocket;
pub struct Application {
    port: u16,
    server: Server,
}
impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {

        let address = format!(
            "{}:{}",
            &configuration.application.host, &configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
        
            configuration,
        )
        .await?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}


async fn run(
    listener: TcpListener,
    configuration: Settings,
) -> Result<Server, anyhow::Error> {
    let secret_obj = web::Data::new(configuration.secret);
    let workers = configuration.application.workers;
    let application_obj = web::Data::new(configuration.application);
    // let _secret_key = Key::from(hmac_secret.expose_secret().as_bytes())
    let ws_server = web::Data::new(websocket::Server::new().start());
    let server = HttpServer::new(move || {
        App::new()
            //.app_data(web::JsonConfig::default().limit(1024 * 1024 * 50))
            .wrap(TracingLogger::default())
            .app_data(secret_obj.clone())
            .app_data(application_obj.clone())
            .app_data(ws_server.clone())
            .configure(routes)
    })
    .workers(workers)
    .listen(listener)?
    .run();

    Ok(server)
}
