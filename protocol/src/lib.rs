use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "server", derive(Serialize))]
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SessionState {
    pub users: HashMap<String, UserState>,
    pub admin: Option<String>,
}

#[cfg_attr(feature = "server", derive(Serialize))]
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserState {
    pub name: Option<String>,
    pub points: Option<String>,
    pub is_spectator: bool,
    #[cfg(feature = "server")]
    #[serde(skip)]
    pub kicked: bool,
}

#[cfg_attr(feature = "server", derive(Deserialize))]
#[cfg_attr(feature = "client", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
#[serde(tag = "tag", content = "content")]
pub enum ClientMessage {
    NameChange(String),
    SetPoints(String),
    ResetPoints,
    Whoami,
    ClaimSession,
    KickUser(String),
    SetSpectator(bool),
}

#[cfg_attr(feature = "server", derive(Serialize))]
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Debug, Clone, PartialEq)]
#[serde(tag = "tag", content = "content")]
pub enum ServerMessage {
    State(SessionState),
    Whoami(String),
    Error(String),
    KeepAlive,
}
