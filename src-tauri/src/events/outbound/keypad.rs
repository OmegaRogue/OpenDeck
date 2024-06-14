use std::time::Duration;

use super::{send_to_plugin, GenericInstancePayload};

use crate::shared::{ActionContext, Context};
use crate::store::profiles::{acquire_locks_mut, get_slot_mut, save_profile};

use serde::Serialize;

#[derive(Serialize)]
struct KeyEvent {
	event: &'static str,
	action: String,
	context: ActionContext,
	device: String,
	payload: GenericInstancePayload,
}

pub async fn key_down(device: &str, key: u8) -> Result<(), anyhow::Error> {
	let mut locks = acquire_locks_mut().await;
	let selected_profile = locks.device_stores.get_selected_profile(device)?;
	let context = Context {
		device: device.to_owned(),
		profile: selected_profile.to_owned(),
		controller: "Keypad".to_owned(),
		position: key,
	};
	let slot = get_slot_mut(&context, &mut locks).await?;
	let slot_r = slot.as_mut().unwrap();
	match slot_r.action.uuid.as_str() {
		"com.amansprojects.starterpack.multiaction" => {
			for instance in slot_r.multi.iter_mut() {
				if instance.action.uuid == "com.amansprojects.starterpack.delay" {
					tokio::time::sleep(Duration::from_millis(instance.settings.get("delay").unwrap().as_u64().unwrap())).await;
					continue
				}
				send_to_plugin(
					&instance.action.plugin,
					&KeyEvent {
						event: "keyDown",
						action: instance.action.uuid.clone(),
						context: instance.context.clone(),
						device: instance.context.device.clone(),
						payload: GenericInstancePayload::new(instance, true),
					},
				)
					.await?;
				tokio::time::sleep(Duration::from_millis(100)).await;
				if instance.states.len() == 2 && !instance.action.disable_automatic_states {
					instance.current_state = (instance.current_state + 1) % (instance.states.len() as u16);
				}
				send_to_plugin(
					&instance.action.plugin,
					&KeyEvent {
						event: "keyUp",
						action: instance.action.uuid.clone(),
						context: instance.context.clone(),
						device: instance.context.device.clone(),
						payload: GenericInstancePayload::new(instance, true),
					},
				)
					.await?;
				tokio::time::sleep(Duration::from_millis(100)).await;
			}
			save_profile(device, &mut locks).await?;
			let _ = crate::events::frontend::update_state(crate::APP_HANDLE.get().unwrap(), context, &mut locks).await;
		}
		"com.amansprojects.starterpack.multiactionSwitch" => {
			for instance in slot_r.multi[slot_r.current_state as usize].multi.iter_mut() {
				if instance.action.uuid == "com.amansprojects.starterpack.delay" {
					tokio::time::sleep(Duration::from_millis(instance.settings.get("delay").unwrap().as_u64().unwrap())).await;
					continue
				}
				send_to_plugin(
					&instance.action.plugin,
					&KeyEvent {
						event: "keyDown",
						action: instance.action.uuid.clone(),
						context: instance.context.clone(),
						device: instance.context.device.clone(),
						payload: GenericInstancePayload::new(instance, true),
					},
				)
					.await?;
				tokio::time::sleep(Duration::from_millis(100)).await;
				if instance.states.len() == 2 && !instance.action.disable_automatic_states {
					instance.current_state = (instance.current_state + 1) % (instance.states.len() as u16);
				}
				send_to_plugin(
					&instance.action.plugin,
					&KeyEvent {
						event: "keyUp",
						action: instance.action.uuid.clone(),
						context: instance.context.clone(),
						device: instance.context.device.clone(),
						payload: GenericInstancePayload::new(instance, true),
					},
				)
					.await?;
				tokio::time::sleep(Duration::from_millis(100)).await;
			}
			slot_r.current_state = (slot_r.current_state + 1) % (slot_r.states.len() as u16);
			save_profile(device, &mut locks).await?;
			let _ = crate::events::frontend::update_state(crate::APP_HANDLE.get().unwrap(), context, &mut locks).await;
		},
		_ => {
			send_to_plugin(
				&slot_r.action.plugin,
				&KeyEvent {
					event: "keyDown",
					action: slot_r.action.uuid.clone(),
					context: slot_r.context.clone(),
					device: slot_r.context.device.clone(),
					payload: GenericInstancePayload::new(slot_r, false),
				},
			)
				.await?;
		},
	}

	Ok(())
}

pub async fn key_up(device: &str, key: u8) -> Result<(), anyhow::Error> {
	let mut locks = acquire_locks_mut().await;
	let selected_profile = locks.device_stores.get_selected_profile(device)?;
	let context = Context {
		device: device.to_owned(),
		profile: selected_profile.to_owned(),
		controller: "Keypad".to_owned(),
		position: key,
	};

	let slot = get_slot_mut(&context, &mut locks).await?;
	let instance = slot.as_mut().unwrap();

	if instance.states.len() == 2 && !instance.action.disable_automatic_states {
		instance.current_state = (instance.current_state + 1) % (instance.states.len() as u16);
	}

	send_to_plugin(
		&instance.action.plugin,
		&KeyEvent {
			event: "keyUp",
			action: instance.action.uuid.clone(),
			context: instance.context.clone(),
			device: instance.context.device.clone(),
			payload: GenericInstancePayload::new(instance, false),
		},
	)
	.await?;

	save_profile(device, &mut locks).await?;
	let _ = crate::events::frontend::update_state(crate::APP_HANDLE.get().unwrap(), context, &mut locks).await;

	Ok(())
}
