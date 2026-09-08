use super::AppId;
use crate::app_provider::AppProvider;
use serde_json::json;
use std::collections::HashSet;

#[test]
fn unsupported_identities_survive_loading_lookup_and_saving() {
    let provider = crate::mock_app_provider();
    for stored in [
        json!({"namespace": "android-activity", "value": "opaque/profile/component"}),
        json!({"namespace": "future-scheme", "value": " Mixed/Case:☃ \\value"}),
        json!({"namespace": "freedesktop", "value": "unsupported/value"}),
        json!({"namespace": "", "value": ""}),
    ] {
        let id: AppId = serde_json::from_value(stored.clone()).unwrap();
        assert!(provider.entry(&id).is_none());
        assert_eq!(serde_json::to_value(&id).unwrap(), stored);
    }
}

#[test]
fn namespaces_distinguish_lookup_and_collection_keys() {
    let provider = crate::mock_app_provider();
    let mock = AppId::from_parts("mock", "notes");
    let foreign = AppId::from_parts("freedesktop", "notes");
    assert_eq!(provider.entry(&mock).unwrap().id, mock);
    assert!(provider.entry(&foreign).is_none());
    assert_eq!(HashSet::from([mock, foreign]).len(), 2);
}
