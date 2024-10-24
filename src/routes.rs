
use crate::handlers::{health_check, send_web_socket, web_socket};
use crate::middlewares::RequireAuth;
use crate::openapi::ApiDoc;
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn routes(cfg: &mut web::ServiceConfig) {
    let openapi = ApiDoc::openapi();
    cfg
        .route("/", web::get().to(health_check))
        .route("/websocket", web::get().to(web_socket))
        .route("/send", web::post().to(send_web_socket).wrap(RequireAuth))
        .service(SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", openapi.clone()));
}
