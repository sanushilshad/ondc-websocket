use actix::Addr;
use actix_web::{ web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;



use crate::errors::GenericError;
use crate::schemas::{GenericResponse, WSRequest, WSKeyTrait, WebSocketParam};
use crate:: websocket::{MessageToClient, Server, WebSocketSession};




#[utoipa::path(
    get,
    path = "/",
    tag = "Health Check",
)]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Running Server")
}

#[utoipa::path(
    post,
    path = "/websocket",
    tag = "Connect WebSocket",

    params(
        ("device_id" = String, Query, description = "Device Id"),
        ("user_id" = String, Query, description = "User Id"),
        ("business_id" = String, Query, description = "Business Id"),
    )
)]

#[tracing::instrument(
    name = "Commence web socket",
    skip(stream),
    fields(
    )
    )]
pub async fn web_socket(
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<WebSocketParam>,
    server_addr: web::Data<Addr<Server>>,
) -> Result<HttpResponse, Error> {
    let web_socket_key = query.get_ws_key();
    let res = ws::start(
        WebSocketSession::new(web_socket_key, server_addr.get_ref().clone()),
        &req,
        stream,
    )?;
    Ok(res)
}


#[utoipa::path(
    get,
    path = "/send",
    tag = "Send WebSocket",

    params(
        ("Authorization" = String, Header, description = "JWT token"),
    )
)]
#[tracing::instrument(name = "send_web_socket")]
pub async fn send_web_socket(
    req: WSRequest,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<GenericResponse<()>>, GenericError>{
    let ws_json = serde_json::to_value(&req.data).unwrap();
    let ws_key = &req.get_ws_key();
    let msg = MessageToClient::new(
        req.action_type,
        ws_json,
        Some(ws_key.to_string()),
    );
    websocket_srv.do_send(msg);
    Ok(web::Json(GenericResponse::success(
        "Successfully send Web Socket Notification",
        Some(()),
       
    )))
    
}



