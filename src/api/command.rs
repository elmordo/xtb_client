use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCommand<A> {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<A>,
}


impl<A> ApiCommand<A> {
    pub fn builder() -> ApiCommandBuilder<A> {
        ApiCommandBuilder::default()
    }
}


/// Build the command
#[derive(Clone, Debug, Default)]
pub struct ApiCommandBuilder<A> {
    command: ApiCommand<A>,
}

impl<A> ApiCommandBuilder<A> {
    pub fn command(mut self, command: String) -> Self {
        self.command.command = command;
        self
    }

    pub fn stream_session_id(mut self, stream_session_id: String) -> Self {
        self.command.stream_session_id = Some(stream_session_id);
        self
    }

    pub fn custom_tag(mut self, custom_tag: String) -> Self {
        self.command.custom_tag = Some(custom_tag);
        self
    }

    pub fn arguments(mut self, arguments: A) -> Self {
        self.command.arguments = Some(arguments);
        self
    }

    /// Get built command
    pub fn build(self) -> ApiCommand<A> {
        self.command
    }
}
