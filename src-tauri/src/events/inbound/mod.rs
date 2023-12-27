mod misc;
mod settings;

use crate::shared::ActionContext;

use serde::Deserialize;
use log::warn;

#[derive(Deserialize)]
pub struct RegisterEvent {
	pub uuid: String
}

#[derive(Deserialize)]
pub struct ContextEvent<C = ActionContext> {
	pub context: C
}

#[derive(Deserialize)]
pub struct PayloadEvent<T> {
	pub payload: T
}

#[derive(Deserialize)]
pub struct ContextAndPayloadEvent<T, C = ActionContext> {
	pub context: C,
	pub payload: T
}

#[derive(Deserialize)]
#[serde(tag = "event")]
#[serde(rename_all = "camelCase")]
pub enum InboundEventType {
	OpenUrl(PayloadEvent<misc::OpenUrlEvent>),
	SetSettings(ContextAndPayloadEvent<serde_json::Value>),
	GetSettings(ContextEvent),
	SetGlobalSettings(ContextAndPayloadEvent<serde_json::Value, String>),
	GetGlobalSettings(ContextEvent<String>)
}

pub async fn process_incoming_message(data: tokio_tungstenite::tungstenite::Message) -> Result<(), tokio_tungstenite::tungstenite::Error> {
	if let tokio_tungstenite::tungstenite::Message::Text(text) = data {
		let decoded: InboundEventType = match serde_json::from_str(&text) {
			Ok(event) => event,
			Err(_) => return Ok(())
		};

		if let Err(error) = match decoded {
			InboundEventType::OpenUrl(event) => misc::open_url(event).await,
			InboundEventType::SetSettings(event) => settings::set_settings(event).await,
			InboundEventType::GetSettings(event) => settings::get_settings(event).await,
			InboundEventType::SetGlobalSettings(event) => settings::set_global_settings(event).await,
			InboundEventType::GetGlobalSettings(event) => settings::get_global_settings(event).await
		} {
			warn!("Failed to process incoming event from plugin: {}\n\tCaused by: {}", error, error.root_cause())
		}
	}

	Ok(())
}