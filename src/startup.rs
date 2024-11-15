use crate::middlewares::SaveRequestResponse;
use crate::pulsar_client::AppState;
use crate::routes::routes;
use crate::schemas::Settings;
use crate::websocket;
use actix::Actor;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_actix_web::TracingLogger;
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
        let server = run(listener, configuration).await?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

async fn run(listener: TcpListener, configuration: Settings) -> Result<Server, anyhow::Error> {
    let secret_obj = web::Data::new(configuration.secret);
    let workers = configuration.application.workers;
    let application_obj = web::Data::new(configuration.application);
    let pulsar = configuration.pulsar.client().await?;
    let producer = pulsar.get_producer().await;
    let consumer = pulsar
        .get_consumer("test_consumer".to_owned(), "test_subscription".to_owned())
        .await;
    let pulsar_prod = web::Data::new(AppState {
        producer: Mutex::new(producer),
    });
    // let pulsar_prod = web::Data::new(producer);

    let ws_server = web::Data::new(websocket::Server::new().start());

    pulsar.start_consumer(consumer, ws_server.clone()).await;
    let server = HttpServer::new(move || {
        App::new()
            //.app_data(web::JsonConfig::default().limit(1024 * 1024 * 50))
            .wrap(SaveRequestResponse)
            .wrap(TracingLogger::default())
            .app_data(secret_obj.clone())
            .app_data(application_obj.clone())
            .app_data(ws_server.clone())
            .app_data(pulsar_prod.clone())
            // .app_data(pulsar_consumer.clone())
            .configure(routes)
    })
    .workers(workers)
    .listen(listener)?
    .run();

    Ok(server)
}
