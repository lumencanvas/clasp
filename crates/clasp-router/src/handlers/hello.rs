//! HELLO message handler -- authenticates clients and creates sessions.

use clasp_core::{codec, ErrorMessage, Message, SecurityMode, ValidationResult};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::session::Session;
use super::{send_chunked_snapshot, HandlerContext, MessageResult};

pub(crate) async fn handle(
    hello: &clasp_core::HelloMessage,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let (authenticated, subject, scopes) = match ctx.security_mode {
        SecurityMode::Open => (false, None, Vec::new()),
        SecurityMode::Authenticated => {
            let token = match &hello.token {
                Some(t) => t,
                None => {
                    warn!("Connection rejected: no token provided in authenticated mode");
                    #[cfg(feature = "metrics")]
                    metrics::counter!("clasp_errors_total", "code" => "300").increment(1);
                    let error = Message::Error(ErrorMessage {
                        code: 300,
                        message: "Authentication required".to_string(),
                        address: None,
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&error).ok()?;
                    let _ = ctx.sender.send(bytes).await;
                    return Some(MessageResult::Disconnect);
                }
            };

            let validator = match ctx.token_validator {
                Some(v) => v,
                None => {
                    error!("Authenticated mode but no token validator configured");
                    #[cfg(feature = "metrics")]
                    metrics::counter!("clasp_errors_total", "code" => "500").increment(1);
                    let error = Message::Error(ErrorMessage {
                        code: 500,
                        message: "Server misconfiguration".to_string(),
                        address: None,
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&error).ok()?;
                    let _ = ctx.sender.send(bytes).await;
                    return Some(MessageResult::Disconnect);
                }
            };

            match validator.validate(token) {
                ValidationResult::Valid(info) => {
                    info!(
                        "Token validated for subject: {:?}, scopes: {}",
                        info.subject,
                        info.scopes.len()
                    );
                    (true, info.subject, info.scopes)
                }
                ValidationResult::Expired => {
                    warn!("Connection rejected: token expired");
                    #[cfg(feature = "metrics")]
                    metrics::counter!("clasp_errors_total", "code" => "302").increment(1);
                    let error = Message::Error(ErrorMessage {
                        code: 302,
                        message: "Token has expired".to_string(),
                        address: None,
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&error).ok()?;
                    let _ = ctx.sender.send(bytes).await;
                    return Some(MessageResult::Disconnect);
                }
                ValidationResult::Invalid(reason) => {
                    warn!("Connection rejected: invalid token - {}", reason);
                    #[cfg(feature = "metrics")]
                    metrics::counter!("clasp_errors_total", "code" => "300").increment(1);
                    let error = Message::Error(ErrorMessage {
                        code: 300,
                        message: format!("Invalid token: {}", reason),
                        address: None,
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&error).ok()?;
                    let _ = ctx.sender.send(bytes).await;
                    return Some(MessageResult::Disconnect);
                }
                ValidationResult::NotMyToken => {
                    warn!("Connection rejected: unrecognized token format");
                    #[cfg(feature = "metrics")]
                    metrics::counter!("clasp_errors_total", "code" => "300").increment(1);
                    let error = Message::Error(ErrorMessage {
                        code: 300,
                        message: "Unrecognized token format".to_string(),
                        address: None,
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&error).ok()?;
                    let _ = ctx.sender.send(bytes).await;
                    return Some(MessageResult::Disconnect);
                }
            }
        }
    };

    let mut new_session =
        Session::new(ctx.sender.clone(), hello.name.clone(), hello.features.clone());

    if authenticated {
        new_session.set_authenticated(
            hello.token.clone().unwrap_or_default(),
            subject,
            scopes,
        );
    }

    let new_session = Arc::new(new_session);
    let session_id = new_session.id.clone();
    ctx.sessions.insert(session_id.clone(), new_session.clone());

    info!(
        "Session created: {} ({}) authenticated={}",
        hello.name, session_id, new_session.authenticated
    );

    #[cfg(feature = "federation")]
    if new_session.is_federation_peer() {
        info!(
            "Federation peer detected: {} ({})",
            hello.name, session_id
        );
    }

    let welcome = new_session.welcome_message(&ctx.config.name, &ctx.config.features);
    let response = codec::encode(&welcome).ok()?;
    let _ = ctx.sender.send(response).await;

    let mut full_snapshot = ctx.state.full_snapshot();
    if let Some(ref filter) = ctx.snapshot_filter {
        full_snapshot.params =
            filter.filter_snapshot(full_snapshot.params, &new_session, ctx.state);
    }
    send_chunked_snapshot(ctx.sender, full_snapshot).await;

    Some(MessageResult::NewSession(new_session))
}
