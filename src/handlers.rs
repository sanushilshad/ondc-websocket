use crate::errors::GenericError;
use crate::pulsar_client::{AppState, MessageData};
use crate::schemas::{GenericResponse, ProcessType, WSKeyTrait, WSRequest, WebSocketParam};
use crate::websocket::{MessageToClient, Server, WebSocketSession};
use actix::Addr;
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
#[utoipa::path(get, path = "/", tag = "Health Check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Running Server")
}

#[utoipa::path(
    get,
    path = "/websocket",
    tag = "WebSocket",
    description = "For Order flow the WebSocket should only send the business_id, for Product search all the three paramters are required.",
    summary = "Connect WebSocket API",
    params(
        ("device_id" = Option<String>, Query, description = "Device Id"),
        ("user_id" = Option<String>, Query, description = "User Id"),
        ("business_id" = String, Query, description = "Business Id"),
    )
)]
#[tracing::instrument(name = "Commence web socket", skip(stream), fields())]
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
    post,
    path = "/send",
    tag = "WebSocket",
    description = "Send WebSocket",
    summary = "Send WebSocket API",

    params(
        ("Authorization" = String, Header, description = "JWT token"),
    ),
    request_body(content = WSRequest, description = "Request Body"),
    responses(
        (status=200, description= "Web Socket response", body=GenericResponse),
    ),


)]
#[tracing::instrument(name = "send_web_socket", skip(pulsar_client))]
pub async fn send_web_socket(
    req: WSRequest,
    websocket_srv: web::Data<Addr<Server>>,
    pulsar_client: web::Data<AppState>,
) -> Result<web::Json<GenericResponse>, GenericError> {
    let ws_json = serde_json::to_value(&req.data).unwrap();
    let ws_key = &req.get_ws_key();
    let msg = MessageToClient::new(req.action_type, ws_json, Some(ws_key.to_string()));
    let mut producer = pulsar_client.producer.lock().await;
    // let a = ProducerMessage {};
    if req.process_type.is_none() {
        producer
            .send_non_blocking(MessageData {
                partition_key: ws_key.to_string(),
                data: serde_json::to_string(&msg).unwrap(),
            })
            .await
            .map_err(|e| GenericError::UnexpectedError(e.into()))?;
    } else if req.process_type == Some(ProcessType::Immediate) {
        websocket_srv.do_send(msg);
    }

    Ok(web::Json(GenericResponse::success(
        "Successfully send Web Socket Notification",
    )))
}
