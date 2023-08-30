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
#[derive(Clone, Debug)]
pub struct ApiCommandBuilder<A> {
    command: ApiCommand<A>,
}

impl<A> ApiCommandBuilder<A> {
    pub fn command(mut self, command: String) -> Self {
        self.command.command = command;
        self
    }

    pub fn stream_session_id(mut self, stream_session_id: Option<String>) -> Self {
        self.command.stream_session_id = stream_session_id;
        self
    }

    pub fn custom_tag(mut self, custom_tag: Option<String>) -> Self {
        self.command.custom_tag = custom_tag;
        self
    }

    pub fn arguments(mut self, arguments: Option<A>) -> Self {
        self.command.arguments = arguments;
        self
    }

    /// Get built command
    pub fn build(self) -> ApiCommand<A> {
        self.command
    }
}


impl<A> Default for ApiCommandBuilder<A> {
    fn default() -> Self {
        Self {
            command: ApiCommand {
                command: "".to_owned(),
                custom_tag: None,
                stream_session_id: None,
                arguments: None,
            }
        }
    }
}
