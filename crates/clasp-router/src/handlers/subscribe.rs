//! SUBSCRIBE / UNSUBSCRIBE message handlers.

use clasp_core::{codec, ErrorMessage, Message, SecurityMode};
use tracing::{debug, warn};

use crate::subscription::Subscription;
use super::{send_chunked_snapshot, HandlerContext, MessageResult};

pub(crate) async fn handle_subscribe(
    sub: &clasp_core::SubscribeMessage,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let session = ctx.session.as_ref()?;

    let current_subs = session.subscriptions().len();
    let max_subs = 1000;
    if current_subs >= max_subs {
        warn!(
            "Session {} subscription limit reached ({}/{})",
            session.id, current_subs, max_subs
        );
        #[cfg(feature = "metrics")]
        metrics::counter!("clasp_errors_total", "code" => "429").increment(1);
        let error = Message::Error(ErrorMessage {
            code: 429,
            message: format!("Subscription limit reached (max {})", max_subs),
            address: Some(sub.pattern.clone()),
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    if ctx.security_mode == SecurityMode::Authenticated
        && !session.has_strict_read_scope(&sub.pattern)
    {
        warn!(
            "Session {} denied SUBSCRIBE to {} - insufficient scope",
            session.id, sub.pattern
        );
        let error = Message::Error(ErrorMessage {
            code: 301,
            message: "Insufficient scope for subscription".to_string(),
            address: Some(sub.pattern.clone()),
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    match Subscription::new(
        sub.id,
        session.id.clone(),
        &sub.pattern,
        sub.types.clone(),
        sub.options.clone().unwrap_or_default(),
    ) {
        Ok(subscription) => {
            ctx.subscriptions.add(subscription);
            session.add_subscription(sub.id);
            #[cfg(feature = "metrics")]
            metrics::gauge!("clasp_subscriptions_active").increment(1.0);
            debug!("Session {} subscribed to {}", session.id, sub.pattern);

            let mut snapshot = ctx.state.snapshot(&sub.pattern);
            if let Some(ref filter) = ctx.snapshot_filter {
                snapshot.params = filter.filter_snapshot(snapshot.params, session, ctx.state);
            }
            if !snapshot.params.is_empty() {
                send_chunked_snapshot(ctx.sender, snapshot).await;
            }
        }
        Err(e) => {
            warn!("Invalid subscription pattern: {}", e);
            let error = Message::Error(ErrorMessage {
                code: 202,
                message: e.to_string(),
                address: Some(sub.pattern.clone()),
                correlation_id: None,
            });
            let bytes = codec::encode(&error).ok()?;
            return Some(MessageResult::Send(bytes));
        }
    }

    Some(MessageResult::None)
}

pub(crate) async fn handle_unsubscribe(
    unsub: &clasp_core::UnsubscribeMessage,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let session = ctx.session.as_ref()?;
    ctx.subscriptions.remove(&session.id, unsub.id);
    session.remove_subscription(unsub.id);
    #[cfg(feature = "metrics")]
    metrics::gauge!("clasp_subscriptions_active").decrement(1.0);
    Some(MessageResult::None)
}
