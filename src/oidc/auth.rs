use super::user;
use axum::async_trait;
use axum::extract::FromRequest;
use axum::http::StatusCode;
use axum::BoxError;
use serde_json::json;
use std::sync::Arc;

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
        let auth_header = req
            .headers()
            .and_then(|headers| headers.get(axum::http::header::AUTHORIZATION))
            .and_then(|value| value.to_str().ok());

        match auth_header {
            Some(header) => {
                let state: &Arc<crate::State> = &*req.extensions().unwrap().get().unwrap();
                let oidc_client = &state.oidc_client;

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
                return Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({"error": "missing auth header"})),
                ))
            }
        }
    }
}
