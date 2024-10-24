
use actix_http::Payload;
use futures::future::LocalBoxFuture;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use utoipa:: ToSchema;
use uuid::Uuid;
use actix_web::{web, FromRequest, HttpRequest};
use serde_json::Value;
use crate::{errors::GenericError, websocket::WebSocketActionType};




#[derive(Debug, Deserialize, Serialize)]
pub struct JWTClaims {
    pub sub: String,
    pub exp: usize,
}


#[derive(Serialize, Debug, ToSchema)]
pub struct GenericResponse<D> {
    pub status: bool,
    pub customer_message: String,
    pub code: String,
    pub data: Option<D>,
}

impl<D> GenericResponse<D> {
    pub fn success(message: &str, data: Option<D>) -> Self {
        Self {
            status: true,
            customer_message: String::from(message),
            code: String::from("200"),
            data,
        }
    }

    pub fn error(message: &str, code: &str, data: Option<D>) -> Self {
        Self {
            status: false,
            customer_message: String::from(message),
            code: String::from(code),
            data,
        }
    }
}




#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub workers: usize
}



#[derive(Debug, Deserialize, Clone)]
pub struct Jwt {
    pub secret: SecretString,
    pub expiry: i64,
}



#[derive(Debug, Deserialize, Clone)]
pub struct SecretSetting {
    pub jwt: Jwt,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub secret: SecretSetting,
}




#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WebSocketParam {
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub device_id: String,
}

pub trait WSKeyTrait {
    fn get_ws_key(&self) -> String;
}

impl WSKeyTrait for WebSocketParam {
    fn get_ws_key(&self) -> String {
        format!("{}#{}#{}", self.user_id, self.business_id, self.device_id)
    }
}


#[derive(Deserialize, Debug, Serialize, ToSchema)]
pub struct WSRequest {
    #[schema(value_type = String)]
    pub user_id: Option<Uuid>,
    #[schema(value_type = String)]
    pub business_id: Option<Uuid>,
    pub device_id: Option<String>,
    pub action_type: WebSocketActionType,
    pub data: Value
    
}

impl WSKeyTrait for WSRequest {
    fn get_ws_key(&self) -> String {
        format!(
            "{}#{}#{}",
            self.user_id.map_or("NA".to_string(), |id| id.to_string()),
            self.business_id
                .map_or("NA".to_string(), |id| id.to_string()),
            self.device_id.clone().unwrap_or("NA".to_string())
        )
    }
}

impl FromRequest for WSRequest {
    type Error = GenericError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(GenericError::ValidationError(e.to_string())),
            }
        })
    }
}