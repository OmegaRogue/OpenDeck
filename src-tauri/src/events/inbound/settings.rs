pub async fn set_settings(event: super::ContextAndPayloadEvent<serde_json::Value>) -> Result<(), anyhow::Error> {
	let (
		app,
		mut device_stores,
		devices,
		mut profile_stores
	) = crate::store::profiles::lock_mutexes().await;

	let selected_profile = &device_stores.get_device_store(&event.context.device, app.as_ref().unwrap())?.value.selected_profile;
	let device = devices.get(&event.context.device).unwrap();
	let store = profile_stores.get_profile_store(device, selected_profile, app.as_ref().unwrap())?;
	let profile = &mut store.value;

	let instance = match event.context.controller.as_str() {
		"Encoder" => profile.sliders[event.context.position as usize].as_mut(),
		_ => profile.keys[event.context.position as usize].as_mut()
	};

	if let Some(instance) = instance {
		instance.settings = event.payload;
		store.save()?;
	}

	Ok(())
}

pub async fn get_settings(event: super::ContextEvent) -> Result<(), anyhow::Error> {
	if let Some(instance) = crate::store::profiles::get_instance(
		&event.context.device,
		event.context.position,
		&event.context.controller
	).await? {
		crate::events::outbound::settings::did_receive_settings(event.context, &instance).await?;
	}

	Ok(())
}

pub async fn set_global_settings(event: super::ContextAndPayloadEvent<serde_json::Value, String>) -> Result<(), anyhow::Error> {
	let app = crate::APP_HANDLE.lock().await;
	let app = app.as_ref().unwrap();

	let settings_dir = app.path_resolver().app_config_dir().unwrap().join("settings/");
	std::fs::create_dir_all(&settings_dir)?;

	let path = settings_dir.join(event.context + ".json");
	std::fs::write(path, event.payload.to_string())?;
	Ok(())
}

pub async fn get_global_settings(event: super::ContextEvent<String>) -> Result<(), anyhow::Error> {
	crate::events::outbound::settings::did_receive_global_settings(&event.context).await
}