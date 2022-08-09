use std::{fmt::Write, rc::Rc};

use futures::prelude::*;
use gloo_net::websocket::{futures::WebSocket, Message as WebSocketMessage};
use yew::prelude::*;

use super::*;
use planc_protocol::{ClientMessage, ServerMessage, SessionState};

#[derive(Debug, PartialEq, Properties)]
pub struct SessionProps {
    pub id: String,
}

#[function_component(Session)]
pub fn session(props: &SessionProps) -> Html {
    let context = use_context::<Rc<AppContext>>().unwrap();
    let history = use_history().unwrap();
    let name = if let Some(name) = context.name().clone() {
        name
    } else {
        history.push(Route::Home);
        return html! {};
    };
    let websocket_uri = {
        let mut websocket_uri = String::new();
        let location = web_sys::window().unwrap().location();
        match location.protocol().unwrap().as_ref() {
            "http:" => websocket_uri += "ws://",
            "https:" => websocket_uri += "wss://",
            _ => panic!("Unknown protocol in location"),
        }
        websocket_uri += &location.hostname().unwrap();
        if let Ok(port) = location.port() {
            write!(websocket_uri, ":{}", port).unwrap();
        }
        websocket_uri += "/api/";
        websocket_uri += &props.id;
        websocket_uri
    };
    let remote_state = use_state(SessionState::default);
    let remote_uid = use_state(|| Some(String::default()));
    let remote_error = use_state(|| None);
    let sender = {
        let remote_state = remote_state.clone();
        let remote_uid = remote_uid.clone();
        let remote_error = remote_error.clone();
        use_state(move || {
            let websocket = WebSocket::open(&websocket_uri).expect("Error opening connection");
            let (mut sink, mut stream) = websocket.split();
            wasm_bindgen_futures::spawn_local(async move {
                // Handle received messages
                while let Some(raw_message) = stream.next().await {
                    let text = match raw_message {
                        Ok(WebSocketMessage::Text(text)) => text,
                        Ok(_) => {
                            log::warn!("Invalid message received");
                            continue;
                        }
                        Err(err) => {
                            log::error!("Error received message: {}", err);
                            continue;
                        }
                    };
                    let message = match serde_json::from_str(&text) {
                        Ok(message) => message,
                        Err(err) => {
                            log::error!("Error decoding received message: {}", err);
                            continue;
                        }
                    };
                    match message {
                        ServerMessage::State(state) => remote_state.set(state),
                        ServerMessage::Whoami(uid) => remote_uid.set(Some(uid)),
                        ServerMessage::Error(error) => remote_error.set(Some(error)),
                        ServerMessage::KeepAlive => {}
                    }
                }
            });
            let (sender, mut receiver) = futures::channel::mpsc::unbounded();
            wasm_bindgen_futures::spawn_local(async move {
                // Send messages
                while let Some(message) = receiver.next().await {
                    let text = serde_json::to_string(&message).unwrap();
                    let raw_message = WebSocketMessage::Text(text);
                    if let Err(err) = sink.send(raw_message).await {
                        log::error!("Error sending message: {}", err);
                    }
                }
            });
            {
                let mut sender = sender.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    // Send whoami request
                    sender.send(ClientMessage::Whoami).await.unwrap();
                    // Send name change request
                    sender.send(ClientMessage::NameChange(name)).await.unwrap();
                });
            }
            sender
        })
    };
    let is_admin = matches!((&*remote_uid, &remote_state.admin), (Some(uid), Some(admin_uid)) if uid == admin_uid);
    html! {
        <>
            <Participants
                users={remote_state.users.clone()}
                is_admin={is_admin}
                on_kick={
                    let sender = sender.clone();
                    Callback::from(move |user_id| {
                        sender.unbounded_send(ClientMessage::KickUser(user_id)).ok();
                    })
                }
            />
            <Cards on_click={
                let sender = sender.clone();
                Callback::from(move |card: &'static str| {
                    sender.unbounded_send(ClientMessage::SetPoints(card.to_string())).ok();
                })
            } />
            if remote_state.admin.is_none() {
                <button onclick={
                    let sender = sender.clone();
                    Callback::from(move |_| {
                        sender.unbounded_send(ClientMessage::ClaimSession).ok();
                    })
                }>{"Claim Session"}</button>
            }
        </>
    }
}
