use serde::{Deserialize, Serialize};
use tenda_playground_platform::ProtocolMessage;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JsonProtocolMessage {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "unlock")]
    Unlock,
    #[serde(rename = "output")]
    Output { payload: String },
    #[serde(rename = "result")]
    Result { value_type: String, value: String },
    #[serde(rename = "error")]
    Error { payload: Vec<String> },
}

impl From<ProtocolMessage> for JsonProtocolMessage {
    fn from(message: ProtocolMessage) -> Self {
        use ProtocolMessage::*;

        match message {
            Ready => JsonProtocolMessage::Ready,
            Unlock => JsonProtocolMessage::Unlock,
            Output(output) => JsonProtocolMessage::Output { payload: output },
            Result(value_type, value) => JsonProtocolMessage::Result {
                value_type: value_type.to_string(),
                value,
            },
            Error(message) => JsonProtocolMessage::Error { payload: message },
        }
    }
}

impl std::fmt::Display for JsonProtocolMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json_string = serde_json::to_string(self).unwrap();
        write!(f, "{}", json_string)
    }
}
