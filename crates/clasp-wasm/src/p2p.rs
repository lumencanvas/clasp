//! WebRTC P2P support for WASM/Browser
//!
//! This module provides WebRTC peer-to-peer connectivity using the browser's
//! RTCPeerConnection API via web-sys bindings.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    MessageEvent, RtcConfiguration, RtcDataChannel, RtcDataChannelInit, RtcDataChannelState,
    RtcIceCandidate, RtcIceCandidateInit, RtcIceServer, RtcPeerConnection,
    RtcPeerConnectionIceEvent, RtcSdpType, RtcSessionDescriptionInit,
};

use clasp_core::{signal_address, P2P_ANNOUNCE, P2P_SIGNAL_PREFIX};

/// P2P connection state
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmP2PState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
    Closed,
}

/// A single WebRTC peer connection
#[wasm_bindgen]
pub struct WasmP2PConnection {
    peer_session_id: String,
    correlation_id: String,
    pc: RtcPeerConnection,
    reliable_channel: Rc<RefCell<Option<RtcDataChannel>>>,
    unreliable_channel: Rc<RefCell<Option<RtcDataChannel>>>,
    state: Rc<RefCell<WasmP2PState>>,
    pending_candidates: Rc<RefCell<Vec<String>>>,
    on_message: Rc<RefCell<Option<js_sys::Function>>>,
    on_state_change: Rc<RefCell<Option<js_sys::Function>>>,
}

#[wasm_bindgen]
impl WasmP2PConnection {
    /// Create a new P2P connection (as offerer/initiator)
    #[wasm_bindgen(constructor)]
    pub fn new(
        peer_session_id: &str,
        correlation_id: &str,
        ice_servers: Option<js_sys::Array>,
    ) -> Result<WasmP2PConnection, JsValue> {
        let config = create_rtc_config(ice_servers)?;
        let pc = RtcPeerConnection::new_with_configuration(&config)?;

        let connection = WasmP2PConnection {
            peer_session_id: peer_session_id.to_string(),
            correlation_id: correlation_id.to_string(),
            pc,
            reliable_channel: Rc::new(RefCell::new(None)),
            unreliable_channel: Rc::new(RefCell::new(None)),
            state: Rc::new(RefCell::new(WasmP2PState::Disconnected)),
            pending_candidates: Rc::new(RefCell::new(Vec::new())),
            on_message: Rc::new(RefCell::new(None)),
            on_state_change: Rc::new(RefCell::new(None)),
        };

        Ok(connection)
    }

    /// Get the peer session ID
    #[wasm_bindgen(getter)]
    pub fn peer_session_id(&self) -> String {
        self.peer_session_id.clone()
    }

    /// Get the correlation ID
    #[wasm_bindgen(getter)]
    pub fn correlation_id(&self) -> String {
        self.correlation_id.clone()
    }

    /// Get the current connection state
    #[wasm_bindgen(getter)]
    pub fn state(&self) -> WasmP2PState {
        *self.state.borrow()
    }

    /// Set the message callback
    pub fn set_on_message(&self, callback: js_sys::Function) {
        *self.on_message.borrow_mut() = Some(callback);
    }

    /// Set the state change callback
    pub fn set_on_state_change(&self, callback: js_sys::Function) {
        *self.on_state_change.borrow_mut() = Some(callback);
    }

    /// Create an SDP offer (as initiator)
    pub async fn create_offer(&self) -> Result<String, JsValue> {
        *self.state.borrow_mut() = WasmP2PState::Connecting;

        // Create data channels
        self.create_data_channels()?;

        // Create and set local offer
        let offer = wasm_bindgen_futures::JsFuture::from(self.pc.create_offer()).await?;
        let offer_obj: RtcSessionDescriptionInit = offer.unchecked_into();

        wasm_bindgen_futures::JsFuture::from(self.pc.set_local_description(&offer_obj)).await?;

        // Extract SDP string
        let sdp = self
            .pc
            .local_description()
            .ok_or_else(|| JsValue::from_str("No local description"))?
            .sdp();

        Ok(sdp)
    }

    /// Handle a received SDP offer and create an answer (as answerer)
    pub async fn create_answer(&self, remote_sdp: &str) -> Result<String, JsValue> {
        *self.state.borrow_mut() = WasmP2PState::Connecting;

        // Set up data channel handler for incoming channels
        self.setup_incoming_channel_handler();

        // Set remote description (the offer)
        let offer_init = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        offer_init.set_sdp(remote_sdp);
        wasm_bindgen_futures::JsFuture::from(self.pc.set_remote_description(&offer_init)).await?;

        // Process pending ICE candidates
        self.process_pending_candidates().await?;

        // Create and set local answer
        let answer = wasm_bindgen_futures::JsFuture::from(self.pc.create_answer()).await?;
        let answer_obj: RtcSessionDescriptionInit = answer.unchecked_into();

        wasm_bindgen_futures::JsFuture::from(self.pc.set_local_description(&answer_obj)).await?;

        // Extract SDP string
        let sdp = self
            .pc
            .local_description()
            .ok_or_else(|| JsValue::from_str("No local description"))?
            .sdp();

        Ok(sdp)
    }

    /// Set the remote SDP answer (as offerer)
    pub async fn set_remote_answer(&self, remote_sdp: &str) -> Result<(), JsValue> {
        let answer_init = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
        answer_init.set_sdp(remote_sdp);
        wasm_bindgen_futures::JsFuture::from(self.pc.set_remote_description(&answer_init)).await?;

        // Process pending ICE candidates
        self.process_pending_candidates().await?;

        Ok(())
    }

    /// Add an ICE candidate from the remote peer
    pub async fn add_ice_candidate(&self, candidate_json: &str) -> Result<(), JsValue> {
        // If remote description not set yet, queue the candidate
        if self.pc.remote_description().is_none() {
            self.pending_candidates
                .borrow_mut()
                .push(candidate_json.to_string());
            return Ok(());
        }

        let candidate_obj = js_sys::JSON::parse(candidate_json)?;
        let candidate_init = RtcIceCandidateInit::from(candidate_obj);
        let candidate = RtcIceCandidate::new(&candidate_init)?;
        wasm_bindgen_futures::JsFuture::from(
            self.pc
                .add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate)),
        )
        .await?;

        Ok(())
    }

    /// Send data over the reliable channel
    pub fn send_reliable(&self, data: &[u8]) -> Result<(), JsValue> {
        if let Some(channel) = self.reliable_channel.borrow().as_ref() {
            if channel.ready_state() == RtcDataChannelState::Open {
                channel.send_with_u8_array(data)?;
            } else {
                return Err(JsValue::from_str("Channel not open"));
            }
        } else {
            return Err(JsValue::from_str("No reliable channel"));
        }
        Ok(())
    }

    /// Send data over the unreliable channel
    pub fn send_unreliable(&self, data: &[u8]) -> Result<(), JsValue> {
        if let Some(channel) = self.unreliable_channel.borrow().as_ref() {
            if channel.ready_state() == RtcDataChannelState::Open {
                channel.send_with_u8_array(data)?;
            } else {
                return Err(JsValue::from_str("Channel not open"));
            }
        } else {
            return Err(JsValue::from_str("No unreliable channel"));
        }
        Ok(())
    }

    /// Close the connection
    pub fn close(&self) {
        *self.state.borrow_mut() = WasmP2PState::Closed;

        if let Some(channel) = self.reliable_channel.borrow().as_ref() {
            channel.close();
        }
        if let Some(channel) = self.unreliable_channel.borrow().as_ref() {
            channel.close();
        }
        self.pc.close();
    }

    /// Check if connection is established
    #[wasm_bindgen(getter)]
    pub fn connected(&self) -> bool {
        *self.state.borrow() == WasmP2PState::Connected
    }

    // =========================================================================
    // Internal methods
    // =========================================================================

    fn create_data_channels(&self) -> Result<(), JsValue> {
        // Create reliable channel
        let reliable_init = RtcDataChannelInit::new();
        let reliable = self
            .pc
            .create_data_channel_with_data_channel_dict("clasp-reliable", &reliable_init);
        self.setup_channel_handlers(&reliable, true);
        *self.reliable_channel.borrow_mut() = Some(reliable);

        // Create unreliable channel
        let unreliable_init = RtcDataChannelInit::new();
        unreliable_init.set_ordered(false);
        unreliable_init.set_max_retransmits(0);
        let unreliable = self
            .pc
            .create_data_channel_with_data_channel_dict("clasp", &unreliable_init);
        self.setup_channel_handlers(&unreliable, false);
        *self.unreliable_channel.borrow_mut() = Some(unreliable);

        Ok(())
    }

    fn setup_incoming_channel_handler(&self) {
        let reliable_channel = self.reliable_channel.clone();
        let unreliable_channel = self.unreliable_channel.clone();
        let on_message = self.on_message.clone();
        let state = self.state.clone();
        let on_state_change = self.on_state_change.clone();

        let ondatachannel = Closure::wrap(Box::new(move |event: web_sys::RtcDataChannelEvent| {
            let channel = event.channel();
            let label = channel.label();

            // Set up handlers
            let on_message_clone = on_message.clone();
            let reliable = label == "clasp-reliable";

            let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Some(callback) = on_message_clone.borrow().as_ref() {
                    let _ = callback.call2(
                        &JsValue::NULL,
                        &event.data(),
                        &JsValue::from_bool(reliable),
                    );
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            channel.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();

            // Check for connection completion
            let state_clone = state.clone();
            let on_state_change_clone = on_state_change.clone();
            let onopen = Closure::wrap(Box::new(move |_: JsValue| {
                *state_clone.borrow_mut() = WasmP2PState::Connected;
                if let Some(callback) = on_state_change_clone.borrow().as_ref() {
                    let _ = callback.call1(&JsValue::NULL, &JsValue::from_str("connected"));
                }
            }) as Box<dyn FnMut(JsValue)>);
            channel.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            onopen.forget();

            if label == "clasp-reliable" {
                *reliable_channel.borrow_mut() = Some(channel);
            } else if label == "clasp" {
                *unreliable_channel.borrow_mut() = Some(channel);
            }
        })
            as Box<dyn FnMut(web_sys::RtcDataChannelEvent)>);

        self.pc
            .set_ondatachannel(Some(ondatachannel.as_ref().unchecked_ref()));
        ondatachannel.forget();
    }

    fn setup_channel_handlers(&self, channel: &RtcDataChannel, reliable: bool) {
        let on_message = self.on_message.clone();
        let state = self.state.clone();
        let on_state_change = self.on_state_change.clone();

        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Some(callback) = on_message.borrow().as_ref() {
                let _ =
                    callback.call2(&JsValue::NULL, &event.data(), &JsValue::from_bool(reliable));
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        channel.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let state_clone = state.clone();
        let on_state_change_clone = on_state_change.clone();
        let onopen = Closure::wrap(Box::new(move |_: JsValue| {
            *state_clone.borrow_mut() = WasmP2PState::Connected;
            if let Some(callback) = on_state_change_clone.borrow().as_ref() {
                let _ = callback.call1(&JsValue::NULL, &JsValue::from_str("connected"));
            }
        }) as Box<dyn FnMut(JsValue)>);
        channel.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        let state_clone = state.clone();
        let on_state_change_clone = on_state_change.clone();
        let onclose = Closure::wrap(Box::new(move |_: JsValue| {
            *state_clone.borrow_mut() = WasmP2PState::Closed;
            if let Some(callback) = on_state_change_clone.borrow().as_ref() {
                let _ = callback.call1(&JsValue::NULL, &JsValue::from_str("closed"));
            }
        }) as Box<dyn FnMut(JsValue)>);
        channel.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();
    }

    async fn process_pending_candidates(&self) -> Result<(), JsValue> {
        let candidates: Vec<String> = self.pending_candidates.borrow_mut().drain(..).collect();
        for candidate_json in candidates {
            let candidate_obj = js_sys::JSON::parse(&candidate_json)?;
            let candidate_init = RtcIceCandidateInit::from(candidate_obj);
            let candidate = RtcIceCandidate::new(&candidate_init)?;
            wasm_bindgen_futures::JsFuture::from(
                self.pc
                    .add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate)),
            )
            .await?;
        }
        Ok(())
    }
}

/// P2P connection manager for WASM
#[wasm_bindgen]
pub struct WasmP2PManager {
    session_id: Rc<RefCell<Option<String>>>,
    connections: Rc<RefCell<HashMap<String, WasmP2PConnection>>>,
    known_peers: Rc<RefCell<HashMap<String, Vec<String>>>>,
    ice_servers: Option<js_sys::Array>,
    on_peer_announced: Rc<RefCell<Option<js_sys::Function>>>,
    on_connection_state: Rc<RefCell<Option<js_sys::Function>>>,
    signal_callback: Rc<RefCell<Option<js_sys::Function>>>,
}

#[wasm_bindgen]
impl WasmP2PManager {
    /// Create a new P2P manager
    #[wasm_bindgen(constructor)]
    pub fn new(ice_servers: Option<js_sys::Array>) -> WasmP2PManager {
        WasmP2PManager {
            session_id: Rc::new(RefCell::new(None)),
            connections: Rc::new(RefCell::new(HashMap::new())),
            known_peers: Rc::new(RefCell::new(HashMap::new())),
            ice_servers,
            on_peer_announced: Rc::new(RefCell::new(None)),
            on_connection_state: Rc::new(RefCell::new(None)),
            signal_callback: Rc::new(RefCell::new(None)),
        }
    }

    /// Set the session ID
    pub fn set_session_id(&self, session_id: String) {
        *self.session_id.borrow_mut() = Some(session_id);
    }

    /// Get the session ID
    #[wasm_bindgen(getter)]
    pub fn session_id(&self) -> Option<String> {
        self.session_id.borrow().clone()
    }

    /// Set callback for sending signaling messages
    /// Callback receives (address: string, payload: object)
    pub fn set_signal_callback(&self, callback: js_sys::Function) {
        *self.signal_callback.borrow_mut() = Some(callback);
    }

    /// Set callback for peer announcements
    /// Callback receives (session_id: string, features: string[])
    pub fn set_on_peer_announced(&self, callback: js_sys::Function) {
        *self.on_peer_announced.borrow_mut() = Some(callback);
    }

    /// Set callback for connection state changes
    /// Callback receives (peer_session_id: string, state: string)
    pub fn set_on_connection_state(&self, callback: js_sys::Function) {
        *self.on_connection_state.borrow_mut() = Some(callback);
    }

    /// Announce P2P capability
    pub fn announce(&self) -> Result<(), JsValue> {
        let session_id = self
            .session_id
            .borrow()
            .clone()
            .ok_or_else(|| JsValue::from_str("Not connected"))?;

        let announce = serde_json::json!({
            "session_id": session_id,
            "p2p_capable": true,
            "features": ["webrtc", "reliable", "unreliable"]
        });

        self.send_signal(P2P_ANNOUNCE, &announce)?;
        Ok(())
    }

    /// Handle incoming P2P announce
    pub fn handle_announce(&self, payload: &JsValue) -> Result<(), JsValue> {
        let json: serde_json::Value = serde_wasm_bindgen::from_value(payload.clone())?;

        if let (Some(session_id), Some(features)) = (
            json.get("session_id").and_then(|v| v.as_str()),
            json.get("features").and_then(|v| v.as_array()),
        ) {
            // Don't track ourselves
            if Some(session_id.to_string()) == *self.session_id.borrow() {
                return Ok(());
            }

            let features: Vec<String> = features
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            self.known_peers
                .borrow_mut()
                .insert(session_id.to_string(), features.clone());

            // Notify callback
            if let Some(callback) = self.on_peer_announced.borrow().as_ref() {
                let features_array = js_sys::Array::new();
                for f in &features {
                    features_array.push(&JsValue::from_str(f));
                }
                let _ = callback.call2(
                    &JsValue::NULL,
                    &JsValue::from_str(session_id),
                    &features_array,
                );
            }
        }

        Ok(())
    }

    /// Initiate a P2P connection to a peer
    pub async fn connect_to_peer(&self, peer_session_id: &str) -> Result<(), JsValue> {
        let our_session_id = self
            .session_id
            .borrow()
            .clone()
            .ok_or_else(|| JsValue::from_str("Not connected"))?;

        // Generate correlation ID
        let correlation_id = format!(
            "{}-{}-{}",
            our_session_id,
            peer_session_id,
            js_sys::Math::random()
        );

        // Create connection
        let connection =
            WasmP2PConnection::new(peer_session_id, &correlation_id, self.ice_servers.clone())?;

        // Set up ICE candidate handler
        self.setup_ice_handler(&connection, peer_session_id)?;

        // Create offer
        let sdp = connection.create_offer().await?;

        // Store connection
        self.connections
            .borrow_mut()
            .insert(peer_session_id.to_string(), connection);

        // Send offer via signaling
        let signal = serde_json::json!({
            "type": "offer",
            "from": our_session_id,
            "sdp": sdp,
            "correlation_id": correlation_id
        });

        let address = signal_address(peer_session_id);
        self.send_signal(&address, &signal)?;

        Ok(())
    }

    /// Handle incoming P2P signal
    pub async fn handle_signal(&self, address: &str, payload: &JsValue) -> Result<(), JsValue> {
        if !address.starts_with(P2P_SIGNAL_PREFIX) {
            return Ok(());
        }

        let json: serde_json::Value = serde_wasm_bindgen::from_value(payload.clone())?;
        let signal_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match signal_type {
            "offer" => self.handle_offer(&json).await,
            "answer" => self.handle_answer(&json).await,
            "ice_candidate" => self.handle_ice_candidate(&json).await,
            "connected" => self.handle_connected(&json),
            "disconnected" => self.handle_disconnected(&json),
            _ => Ok(()),
        }
    }

    /// Get list of known P2P-capable peers
    pub fn known_peers(&self) -> js_sys::Array {
        let array = js_sys::Array::new();
        for key in self.known_peers.borrow().keys() {
            array.push(&JsValue::from_str(key));
        }
        array
    }

    /// Check if connected to a peer via P2P
    pub fn is_peer_connected(&self, peer_session_id: &str) -> bool {
        self.connections
            .borrow()
            .get(peer_session_id)
            .map(|c| c.connected())
            .unwrap_or(false)
    }

    /// Disconnect from a peer
    pub fn disconnect_peer(&self, peer_session_id: &str) {
        if let Some(connection) = self.connections.borrow_mut().remove(peer_session_id) {
            connection.close();

            // Send disconnect signal
            if let Some(our_session_id) = self.session_id.borrow().as_ref() {
                let signal = serde_json::json!({
                    "type": "disconnected",
                    "from": our_session_id,
                    "correlation_id": connection.correlation_id(),
                    "reason": "User requested disconnect"
                });
                let address = signal_address(peer_session_id);
                let _ = self.send_signal(&address, &signal);
            }
        }
    }

    // =========================================================================
    // Internal signal handlers
    // =========================================================================

    async fn handle_offer(&self, json: &serde_json::Value) -> Result<(), JsValue> {
        let from = json
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsValue::from_str("Missing 'from' field"))?;
        let sdp = json
            .get("sdp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsValue::from_str("Missing 'sdp' field"))?;
        let correlation_id = json
            .get("correlation_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsValue::from_str("Missing 'correlation_id' field"))?;

        let our_session_id = self
            .session_id
            .borrow()
            .clone()
            .ok_or_else(|| JsValue::from_str("Not connected"))?;

        // Create connection as answerer
        let connection = WasmP2PConnection::new(from, correlation_id, self.ice_servers.clone())?;

        // Set up ICE candidate handler
        self.setup_ice_handler(&connection, from)?;

        // Create answer
        let answer_sdp = connection.create_answer(sdp).await?;

        // Store connection
        self.connections
            .borrow_mut()
            .insert(from.to_string(), connection);

        // Send answer
        let signal = serde_json::json!({
            "type": "answer",
            "from": our_session_id,
            "sdp": answer_sdp,
            "correlation_id": correlation_id
        });

        let address = signal_address(from);
        self.send_signal(&address, &signal)?;

        Ok(())
    }

    async fn handle_answer(&self, json: &serde_json::Value) -> Result<(), JsValue> {
        let from = json
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsValue::from_str("Missing 'from' field"))?;
        let sdp = json
            .get("sdp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsValue::from_str("Missing 'sdp' field"))?;

        if let Some(connection) = self.connections.borrow().get(from) {
            connection.set_remote_answer(sdp).await?;
        }

        Ok(())
    }

    async fn handle_ice_candidate(&self, json: &serde_json::Value) -> Result<(), JsValue> {
        let from = json
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsValue::from_str("Missing 'from' field"))?;
        let candidate = json
            .get("candidate")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsValue::from_str("Missing 'candidate' field"))?;

        if let Some(connection) = self.connections.borrow().get(from) {
            connection.add_ice_candidate(candidate).await?;
        }

        Ok(())
    }

    fn handle_connected(&self, json: &serde_json::Value) -> Result<(), JsValue> {
        let from = json.get("from").and_then(|v| v.as_str()).unwrap_or("");

        // Notify callback
        if let Some(callback) = self.on_connection_state.borrow().as_ref() {
            let _ = callback.call2(
                &JsValue::NULL,
                &JsValue::from_str(from),
                &JsValue::from_str("connected"),
            );
        }

        Ok(())
    }

    fn handle_disconnected(&self, json: &serde_json::Value) -> Result<(), JsValue> {
        let from = json.get("from").and_then(|v| v.as_str()).unwrap_or("");

        // Remove connection
        if let Some(connection) = self.connections.borrow_mut().remove(from) {
            connection.close();
        }

        // Notify callback
        if let Some(callback) = self.on_connection_state.borrow().as_ref() {
            let _ = callback.call2(
                &JsValue::NULL,
                &JsValue::from_str(from),
                &JsValue::from_str("disconnected"),
            );
        }

        Ok(())
    }

    fn setup_ice_handler(
        &self,
        connection: &WasmP2PConnection,
        peer_session_id: &str,
    ) -> Result<(), JsValue> {
        let signal_callback = self.signal_callback.clone();
        let session_id = self.session_id.clone();
        let correlation_id = connection.correlation_id.clone();
        let peer_session_id = peer_session_id.to_string();

        let onicecandidate = Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
            if let Some(candidate) = event.candidate() {
                if let (Some(callback), Some(our_session_id)) = (
                    signal_callback.borrow().as_ref(),
                    session_id.borrow().as_ref(),
                ) {
                    if let Ok(candidate_json) = js_sys::JSON::stringify(&candidate) {
                        let candidate_str = candidate_json.as_string().unwrap_or_default();
                        let signal = serde_json::json!({
                            "type": "ice_candidate",
                            "from": our_session_id,
                            "candidate": candidate_str,
                            "correlation_id": correlation_id
                        });

                        let address = signal_address(&peer_session_id);
                        if let Ok(payload) = serde_wasm_bindgen::to_value(&signal) {
                            let _ = callback.call2(
                                &JsValue::NULL,
                                &JsValue::from_str(&address),
                                &payload,
                            );
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(RtcPeerConnectionIceEvent)>);

        connection
            .pc
            .set_onicecandidate(Some(onicecandidate.as_ref().unchecked_ref()));
        onicecandidate.forget();

        Ok(())
    }

    fn send_signal(&self, address: &str, payload: &serde_json::Value) -> Result<(), JsValue> {
        if let Some(callback) = self.signal_callback.borrow().as_ref() {
            let payload_js = serde_wasm_bindgen::to_value(payload)?;
            callback.call2(&JsValue::NULL, &JsValue::from_str(address), &payload_js)?;
        }
        Ok(())
    }
}

// =========================================================================
// Helper functions
// =========================================================================

fn create_rtc_config(ice_servers: Option<js_sys::Array>) -> Result<RtcConfiguration, JsValue> {
    let config = RtcConfiguration::new();

    let servers = js_sys::Array::new();

    if let Some(user_servers) = ice_servers {
        for server in user_servers.iter() {
            servers.push(&server);
        }
    } else {
        // Default to Google STUN servers
        let default_server = RtcIceServer::new();
        let urls = js_sys::Array::new();
        urls.push(&JsValue::from_str("stun:stun.l.google.com:19302"));
        urls.push(&JsValue::from_str("stun:stun1.l.google.com:19302"));
        default_server.set_urls(&urls);
        servers.push(&default_server);
    }

    config.set_ice_servers(&servers);
    Ok(config)
}
