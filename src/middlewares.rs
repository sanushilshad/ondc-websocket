use crate::errors::GenericError;
use crate::schemas::SecretSetting;
use crate::utils::decode_token;
use actix_http::header::UPGRADE;
use actix_http::{h1, Payload};
use actix_web::body::{self, BoxBody};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{http, web, Error};
use futures::future::LocalBoxFuture;
use std::cell::RefCell;
use std::future::{ready, Ready};
use std::rc::Rc;
use tracing::instrument;

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token = req
            .cookie("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        let jwt_secret = &req
            .app_data::<web::Data<SecretSetting>>()
            .unwrap()
            .jwt
            .secret;

        if token.is_none() {
            let error_message = "Authorization header is missing".to_string();
            let (request, _pl) = req.into_parts();
            let json_error = GenericError::ValidationError(error_message);
            return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
        }

        let _ = match decode_token(token.unwrap(), jwt_secret) {
            Ok(id) => id,
            Err(e) => {
                return Box::pin(async move {
                    let (request, _pl) = req.into_parts();
                    Ok(ServiceResponse::from_err(
                        GenericError::InvalidJWT(e.to_string()),
                        request,
                    ))
                });
            }
        };

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

/// Middleware factory for requiring authentication.
pub struct RequireAuth;

impl<S> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub fn bytes_to_payload(buf: web::Bytes) -> Payload {
    let (_, mut pl) = h1::Payload::create(true);
    pl.unread_data(buf);
    Payload::from(pl)
}

pub struct ReadReqResMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S> Service<ServiceRequest> for ReadReqResMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    #[instrument(skip(self), name = "Request Response Payload", fields(path = %req.path()))]
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        //
        let is_websocket = req.headers().contains_key(UPGRADE)
            && req.headers().get(UPGRADE).unwrap() == "websocket";
        let is_on_search = req.path().ends_with("on_search");
        let is_non_json_req_res =
            req.path().contains("/docs/") || req.path().contains("/api-docs/");
        if is_websocket || is_non_json_req_res {
            Box::pin(async move {
                let fut = svc.call(req).await?;
                Ok(fut)
            })
        } else {
            Box::pin(async move {
                if !is_on_search {
                    let request_str: String = req.extract::<String>().await?;
                    tracing::info!({%request_str}, "HTTP Response");
                    req.set_payload(bytes_to_payload(web::Bytes::from(request_str)));
                }
                let fut = svc.call(req).await?;

                let (req, res) = fut.into_parts();
                let (res, body) = res.into_parts();
                let body_bytes = body::to_bytes(body).await.ok().unwrap();
                let response_str = match std::str::from_utf8(&body_bytes) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        tracing::error!("Error decoding response body");
                        String::from("")
                    }
                };
                tracing::info!({%response_str}, "HTTP Response");
                let res = res.set_body(BoxBody::new(response_str));
                let res = ServiceResponse::new(req, res);
                Ok(res)
            })
        }
    }
}

pub struct SaveRequestResponse;

impl<S> Transform<S, ServiceRequest> for SaveRequestResponse
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = ReadReqResMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ReadReqResMiddleware {
            service: Rc::new(RefCell::new(service)),
        }))
    }
}
