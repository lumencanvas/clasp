//! GET message handler -- returns current state values.
//!
//! Looks up a single address in the router state and returns a SNAPSHOT
//! containing the current value, revision, and writer. Respects scope checks
//! and snapshot filtering.

use clasp_core::{codec, Action, ErrorMessage, Message, SecurityMode};
use tracing::warn;

use super::{HandlerContext, MessageResult};

pub(crate) async fn handle(
    get: &clasp_core::GetMessage,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let session = ctx.session.as_ref()?;

    if ctx.security_mode == SecurityMode::Authenticated
        && !session.has_scope(Action::Read, &get.address)
    {
        warn!(
            "Session {} denied GET to {} - insufficient scope",
            session.id, get.address
        );
        let error = Message::Error(ErrorMessage {
            code: 301,
            message: "Insufficient scope for read operation".to_string(),
            address: Some(get.address.clone()),
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    if let Some(param_state) = ctx.state.get_state(&get.address) {
        let mut params = vec![clasp_core::ParamValue {
            address: get.address.clone(),
            value: param_state.value,
            revision: param_state.revision,
            writer: Some(param_state.writer),
            timestamp: Some(param_state.timestamp),
        }];

        if let Some(ref filter) = ctx.snapshot_filter {
            params = filter.filter_snapshot(params, session, ctx.state);
        }

        if params.is_empty() {
            return Some(MessageResult::None);
        }

        let snapshot = Message::Snapshot(clasp_core::SnapshotMessage { params });
        let bytes = codec::encode(&snapshot).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    Some(MessageResult::None)
}
