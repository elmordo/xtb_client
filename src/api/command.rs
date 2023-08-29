use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCommand<A> {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<A>,
}
