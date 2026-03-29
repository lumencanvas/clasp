//! Tests for the journal query REST API.
//!
//! Gated behind `#[cfg(feature = "journal")]` since the journal module is optional.
//! Run with: cargo test --features journal

#[cfg(feature = "journal")]
mod journal_api_tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use clasp_core::security::{CpskValidator, Scope, TokenInfo};
    use clasp_core::{SignalType, Value};
    use clasp_journal::entry::JournalEntry;
    use clasp_journal::SqliteJournal;
    #[allow(unused_imports)]
    use clasp_journal::Journal;
    use clasp_relay::journal_api::{journal_router, JournalApiState};
    use http_body_util::BodyExt;
    use serde_json::Value as JsonValue;
    use std::sync::Arc;
    use tower::ServiceExt;

    struct TestHarness {
        state: Arc<JournalApiState>,
        admin_token: String,
        read_only_token: String,
    }

    impl TestHarness {
        fn new() -> Self {
            let journal = Arc::new(SqliteJournal::in_memory().unwrap());
            let validator = Arc::new(CpskValidator::new());

            let admin_token = CpskValidator::generate_token();
            let read_only_token = CpskValidator::generate_token();

            let admin_scopes = vec![Scope::parse("admin:/**").unwrap()];
            validator.register(
                admin_token.clone(),
                TokenInfo::new(admin_token.clone(), admin_scopes),
            );

            let read_scopes = vec![Scope::parse("read:/**").unwrap()];
            validator.register(
                read_only_token.clone(),
                TokenInfo::new(read_only_token.clone(), read_scopes),
            );

            Self {
                state: Arc::new(JournalApiState { journal, validator }),
                admin_token,
                read_only_token,
            }
        }

        fn app(&self) -> axum::Router {
            journal_router(self.state.clone())
        }

        fn admin_get(&self, uri: &str) -> Request<Body> {
            Request::builder()
                .method("GET")
                .uri(uri)
                .header("Authorization", format!("Bearer {}", self.admin_token))
                .body(Body::empty())
                .unwrap()
        }
    }

    async fn response_json(resp: axum::response::Response) -> JsonValue {
        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&body_bytes).unwrap_or(serde_json::json!({}))
    }

    // -- Auth tests --

    #[tokio::test]
    async fn missing_auth_header_returns_401() {
        let h = TestHarness::new();
        let app = h.app();

        let req = Request::builder()
            .method("GET")
            .uri("/api/journal/latest")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_token_returns_401() {
        let h = TestHarness::new();
        let app = h.app();

        let req = Request::builder()
            .method("GET")
            .uri("/api/journal/latest")
            .header("Authorization", "Bearer bogus-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn non_admin_token_returns_403() {
        let h = TestHarness::new();
        let app = h.app();

        let req = Request::builder()
            .method("GET")
            .uri("/api/journal/latest")
            .header(
                "Authorization",
                format!("Bearer {}", h.read_only_token),
            )
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn malformed_auth_header_returns_401() {
        let h = TestHarness::new();
        let app = h.app();

        // No "Bearer " prefix
        let req = Request::builder()
            .method("GET")
            .uri("/api/journal/latest")
            .header("Authorization", &h.admin_token)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // -- /api/journal/latest --

    #[tokio::test]
    async fn latest_seq_returns_zero_for_empty_journal() {
        let h = TestHarness::new();
        let app = h.app();

        let resp = app
            .oneshot(h.admin_get("/api/journal/latest"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = response_json(resp).await;
        assert_eq!(json["seq"], 0);
    }

    #[tokio::test]
    async fn latest_seq_increments_after_append() {
        let h = TestHarness::new();

        let entry1 = JournalEntry::from_set(
            "/mixer/fader1".to_string(),
            Value::Float(0.75),
            1,
            "test-author".to_string(),
            1000,
        );
        let entry2 = JournalEntry::from_set(
            "/mixer/fader2".to_string(),
            Value::Float(0.5),
            1,
            "test-author".to_string(),
            2000,
        );
        h.state.journal.append(entry1).await.unwrap();
        h.state.journal.append(entry2).await.unwrap();

        let app = h.app();
        let resp = app
            .oneshot(h.admin_get("/api/journal/latest"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = response_json(resp).await;
        assert_eq!(json["seq"], 2);
    }

    // -- /api/journal/query --

    #[tokio::test]
    async fn query_empty_journal_returns_empty_array() {
        let h = TestHarness::new();
        let app = h.app();

        let resp = app
            .oneshot(h.admin_get("/api/journal/query"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = response_json(resp).await;
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn query_returns_matching_entries() {
        let h = TestHarness::new();

        h.state
            .journal
            .append(JournalEntry::from_set(
                "/mixer/fader1".to_string(),
                Value::Float(0.75),
                1,
                "alice".to_string(),
                1000,
            ))
            .await
            .unwrap();
        h.state
            .journal
            .append(JournalEntry::from_set(
                "/lights/spot1".to_string(),
                Value::Float(1.0),
                1,
                "bob".to_string(),
                2000,
            ))
            .await
            .unwrap();
        h.state
            .journal
            .append(JournalEntry::from_set(
                "/mixer/fader2".to_string(),
                Value::Float(0.5),
                1,
                "alice".to_string(),
                3000,
            ))
            .await
            .unwrap();

        let app = h.app();
        let resp = app
            .oneshot(h.admin_get("/api/journal/query?pattern=/mixer/**"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = response_json(resp).await;
        let entries = json.as_array().unwrap();
        assert_eq!(entries.len(), 2, "Should match two /mixer/** entries");
        assert_eq!(entries[0]["address"], "/mixer/fader1");
        assert_eq!(entries[1]["address"], "/mixer/fader2");
    }

    #[tokio::test]
    async fn query_with_limit() {
        let h = TestHarness::new();

        for i in 0..5 {
            h.state
                .journal
                .append(JournalEntry::from_set(
                    format!("/test/p{}", i),
                    Value::Int(i),
                    1,
                    "author".to_string(),
                    (i as u64) * 1000,
                ))
                .await
                .unwrap();
        }

        let app = h.app();
        let resp = app
            .oneshot(h.admin_get("/api/journal/query?limit=3"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = response_json(resp).await;
        let entries = json.as_array().unwrap();
        assert_eq!(entries.len(), 3);
    }

    // -- /api/journal/since --

    #[tokio::test]
    async fn since_returns_entries_after_seq() {
        let h = TestHarness::new();

        for i in 0..5 {
            h.state
                .journal
                .append(JournalEntry::from_set(
                    format!("/param/{}", i),
                    Value::Int(i),
                    1,
                    "author".to_string(),
                    (i as u64) * 1000,
                ))
                .await
                .unwrap();
        }

        let app = h.app();
        // seq=2 means "entries after seq 2", so entries 3,4,5
        let resp = app
            .oneshot(h.admin_get("/api/journal/since?seq=2"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = response_json(resp).await;
        let entries = json.as_array().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0]["seq"], 3);
        assert_eq!(entries[1]["seq"], 4);
        assert_eq!(entries[2]["seq"], 5);
    }

    #[tokio::test]
    async fn since_with_limit() {
        let h = TestHarness::new();

        for i in 0..10 {
            h.state
                .journal
                .append(JournalEntry::from_set(
                    format!("/p/{}", i),
                    Value::Int(i),
                    1,
                    "author".to_string(),
                    (i as u64) * 1000,
                ))
                .await
                .unwrap();
        }

        let app = h.app();
        let resp = app
            .oneshot(h.admin_get("/api/journal/since?seq=5&limit=2"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = response_json(resp).await;
        let entries = json.as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["seq"], 6);
        assert_eq!(entries[1]["seq"], 7);
    }

    // -- /api/journal/snapshot --

    #[tokio::test]
    async fn snapshot_returns_empty_for_fresh_journal() {
        let h = TestHarness::new();
        let app = h.app();

        let resp = app
            .oneshot(h.admin_get("/api/journal/snapshot"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = response_json(resp).await;
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 0);
    }

    // -- /api/journal/query with type filter --

    #[tokio::test]
    async fn query_with_type_filter() {
        let h = TestHarness::new();

        // Append a SET (param) and a PUBLISH (event)
        h.state
            .journal
            .append(JournalEntry::from_set(
                "/mixer/fader1".to_string(),
                Value::Float(0.75),
                1,
                "alice".to_string(),
                1000,
            ))
            .await
            .unwrap();
        h.state
            .journal
            .append(JournalEntry::from_publish(
                "/events/button".to_string(),
                SignalType::Event,
                Value::Bool(true),
                "bob".to_string(),
                2000,
            ))
            .await
            .unwrap();

        let app = h.app();

        // Filter to only events
        let resp = app
            .oneshot(h.admin_get("/api/journal/query?types=event"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = response_json(resp).await;
        let entries = json.as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["address"], "/events/button");
    }
}
