use wmux::socket::protocol::{Request, Response};

#[test]
fn parse_valid_request() {
    let json = r#"{"id":"abc","method":"system.ping","params":{}}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    assert_eq!(req.id, "abc");
    assert_eq!(req.method, "system.ping");
}

#[test]
fn parse_request_without_params() {
    let json = r#"{"id":"abc","method":"system.ping"}"#;
    let req: Request = serde_json::from_str(json).unwrap();
    assert_eq!(req.method, "system.ping");
    assert!(req.params.is_none() || req.params.as_ref().unwrap().is_null());
}

#[test]
fn serialize_success_response() {
    let resp = Response::success("abc".into(), serde_json::json!({"pong": true}));
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains(r#""ok":true"#));
    assert!(json.contains(r#""pong":true"#));
}

#[test]
fn serialize_error_response() {
    let resp = Response::error("abc".into(), "not_found", "Workspace not found");
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains(r#""ok":false"#));
    assert!(json.contains("not_found"));
}
