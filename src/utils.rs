

use chrono::{Duration, Utc};
use config::{ConfigError, Environment};


use crate::{errors::CustomJWTTokenError, schemas::{ JWTClaims, Settings}};
use secrecy::{ExposeSecret, SecretString};
use jsonwebtoken::{
    decode, encode, Algorithm as JWTAlgorithm, DecodingKey, EncodingKey, Header, Validation,
};



#[tracing::instrument(name = "Decode JWT token")]
pub fn decode_token<T: Into<String> + std::fmt::Debug>(
    token: T,
    secret: &SecretString,
) -> Result<String, CustomJWTTokenError> {
    let decoding_key = DecodingKey::from_secret(secret.expose_secret().as_bytes());
    let decoded = decode::<JWTClaims>(
        &token.into(),
        &decoding_key,
        &Validation::new(JWTAlgorithm::HS256),
    );
    match decoded {
        Ok(token) => Ok(token.claims.sub.to_string()),
        Err(e) => {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    Err(CustomJWTTokenError::Expired)
                }
                _ => Err(CustomJWTTokenError::Invalid("Invalid Token".to_string())),
            }
        }
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}


pub fn get_configuration() -> Result<Settings, ConfigError> {
    let builder = config::Config::builder()
        .add_source(Environment::default().separator("__"))
        .add_source(
            Environment::with_prefix("LIST")
                .try_parsing(true)
                .separator("__")
                .keep_prefix(false)
                .list_separator(","),
        )
        .build()?;
    builder.try_deserialize::<Settings>()
}





#[tracing::instrument(name = "Generate JWT token for user")]
pub fn generate_jwt_token_for_user(
    user_id: &str,
    expiry_time: i64,
    secret: &SecretString,
) -> Result<SecretString, anyhow::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expiry_time))
        .expect("valid timestamp")
        .timestamp() as usize;
    let claims: JWTClaims = JWTClaims {
        sub: user_id.to_owned(),
        exp: expiration,
    };
    let header = Header::new(JWTAlgorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.expose_secret().as_bytes());
    let token: String = encode(&header, &claims, &encoding_key).expect("Failed to generate token");
    Ok(SecretString::new(token.into()))
}


