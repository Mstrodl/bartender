use super::client::OIDCClient;
use super::user;
use axum::async_trait;
use axum::extract::FromRequest;
use axum::http::StatusCode;
use axum::BoxError;
use serde_json::json;
use std::env;

pub struct OIDCAuth(pub user::OIDCUser);

#[async_trait]
impl<B> FromRequest<B> for OIDCAuth
where
    B: axum::body::HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = (axum::http::StatusCode, axum::Json<serde_json::Value>);

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        // Grab the "Authorization" header from the request
        let auth_header = req
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .map(|h| h.to_str().unwrap());

        match auth_header {
            Some(header) => {
                // Get the OIDClient from the request global state
                let oidc_client: &OIDCClient = &*req.extensions().get().unwrap();

                match oidc_client.validate_token(header).await {
                    Ok(user) => {
                        return Ok(Self(user));
                    }
                    Err(_) => {
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            axum::Json(json!({"error": "token invalid or expired"})),
                        ))
                    }
                }
            }
            None => {
                // If there's no "Authorization" header, get the "X-Auth-Token" header
                let machine_secret = req
                    .headers()
                    .get("X-Auth-Token")
                    .map(|value| value.to_str().unwrap());

                match machine_secret {
                    Some(secret) => {
                        if secret == env::var("MACHINE_SECRET").unwrap() {
                            Ok(Self(user::OIDCUser {
                                name: Some(String::from("Drink Machine")),
                                preferred_username: String::from("drink_machine"),
                                groups: Box::new([String::from("drink")]),
                                drink_balance: Some(0),
                            }))
                        } else {
                            return Err((
                                StatusCode::UNAUTHORIZED,
                                axum::Json(json!({"error": "invalid machine secret"})),
                            ));
                        }
                    }
                    // There's no known auth header at all
                    None => {
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            axum::Json(json!({"error": "missing auth header"})),
                        ))
                    }
                }
            }
        }
    }
}
