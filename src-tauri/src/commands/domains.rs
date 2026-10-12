use tauri::State;

use crate::{
    helper::client::HelperClient,
    models::{
        domain::{
            normalize_group,
            validate_hostname,
            validate_ip,
        },
        Domain,
    },
    state::AppState,
};

fn persist(
    state: &State<'_, AppState>,
    domains: &[Domain],
) -> Result<(), String> {
    state.repository.save(domains)
}

#[tauri::command]
pub fn get_domains(state: State<'_, AppState>) -> Result<Vec<Domain>, String> {
    let domains = state
        .domains
        .lock()
        .map_err(|_| "No se pudo acceder al estado".to_string())?;

    Ok(domains.clone())
}

#[tauri::command]
pub fn toggle_domain(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Domain>, String> {
    let mut domains = state
        .domains
        .lock()
        .map_err(|_| "No se pudo acceder al estado".to_string())?;

    let domain = domains
        .iter_mut()
        .find(|domain| domain.id == id)
        .ok_or_else(|| format!("Dominio no encontrado: {}", id))?;

    if domain.enabled {
        HelperClient::disable(&domain.hostname)?;

        domain.enabled = false;
    } else {
        HelperClient::enable(&domain.ip, &domain.hostname)?;

        domain.enabled = true;
    }

    persist(&state, &domains)?;

    Ok(domains.clone())
}

#[tauri::command]
pub fn create_domain(
    hostname: String,
    ip: String,
    group: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Domain>, String> {
    let hostname = validate_hostname(&hostname)?;
    let ip = validate_ip(&ip)?;
    let group = normalize_group(group.as_deref().unwrap_or(""));

    let mut domains = state
        .domains
        .lock()
        .map_err(|_| "No se pudo acceder al estado".to_string())?;

    if domains.iter().any(|domain| domain.hostname == hostname) {
        return Err("Ya existe un dominio con ese hostname".to_string());
    }

    let domain = Domain {
        id: uuid::Uuid::new_v4().to_string(),
        hostname,
        ip,
        enabled: false,
        group,
    };

    domains.push(domain);

    persist(&state, &domains)?;

    Ok(domains.clone())
}

#[tauri::command]
pub fn update_domain(
    id: String,
    hostname: String,
    ip: String,
    group: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Domain>, String> {
    let hostname = validate_hostname(&hostname)?;
    let ip = validate_ip(&ip)?;
    let group = normalize_group(group.as_deref().unwrap_or(""));

    let mut domains = state
        .domains
        .lock()
        .map_err(|_| "No se pudo acceder al estado".to_string())?;

    if domains
        .iter()
        .any(|domain| domain.id != id && domain.hostname == hostname)
    {
        return Err("Ya existe un dominio con ese hostname".to_string());
    }

    let old = domains
        .iter()
        .find(|domain| domain.id == id)
        .cloned()
        .ok_or_else(|| format!("Dominio no encontrado: {}", id))?;

    // Si estaba activo, reescribimos la entrada del hosts.
    if old.enabled {
        HelperClient::disable(&old.hostname)?;
        HelperClient::enable(&ip, &hostname)?;
    }

    let domain = domains
        .iter_mut()
        .find(|domain| domain.id == id)
        .ok_or_else(|| format!("Dominio no encontrado: {}", id))?;

    domain.hostname = hostname;
    domain.ip = ip;
    domain.group = group;

    persist(&state, &domains)?;

    Ok(domains.clone())
}

#[tauri::command]
pub fn delete_domain(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Domain>, String> {
    let mut domains = state
        .domains
        .lock()
        .map_err(|_| "No se pudo acceder al estado".to_string())?;

    let domain = domains
        .iter()
        .find(|domain| domain.id == id)
        .cloned()
        .ok_or_else(|| format!("Dominio no encontrado: {}", id))?;

    if domain.enabled {
        HelperClient::disable(&domain.hostname)?;
    }

    domains.retain(|domain| domain.id != id);

    persist(&state, &domains)?;

    Ok(domains.clone())
}

#[tauri::command]
pub fn enable_all(state: State<'_, AppState>) -> Result<Vec<Domain>, String> {
    let mut domains = state
        .domains
        .lock()
        .map_err(|_| "No se pudo acceder al estado".to_string())?;

    for domain in domains.iter_mut().filter(|d| !d.enabled) {
        HelperClient::enable(&domain.ip, &domain.hostname)?;
        domain.enabled = true;
    }

    persist(&state, &domains)?;

    Ok(domains.clone())
}

#[tauri::command]
pub fn disable_all(state: State<'_, AppState>) -> Result<Vec<Domain>, String> {
    let mut domains = state
        .domains
        .lock()
        .map_err(|_| "No se pudo acceder al estado".to_string())?;

    for domain in domains.iter_mut().filter(|d| d.enabled) {
        HelperClient::disable(&domain.hostname)?;
        domain.enabled = false;
    }

    persist(&state, &domains)?;

    Ok(domains.clone())
}

#[tauri::command]
pub fn toggle_group(
    group: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Domain>, String> {
    let group = normalize_group(&group);

    let mut domains = state
        .domains
        .lock()
        .map_err(|_| "No se pudo acceder al estado".to_string())?;

    for domain in domains
        .iter_mut()
        .filter(|d| d.group == group && d.enabled != enabled)
    {
        if enabled {
            HelperClient::enable(&domain.ip, &domain.hostname)?;
        } else {
            HelperClient::disable(&domain.hostname)?;
        }

        domain.enabled = enabled;
    }

    persist(&state, &domains)?;

    Ok(domains.clone())
}

#[tauri::command]
pub fn export_domains(state: State<'_, AppState>) -> Result<String, String> {
    let domains = state
        .domains
        .lock()
        .map_err(|_| "No se pudo acceder al estado".to_string())?;

    serde_json::to_string_pretty(&domains.clone())
        .map_err(|error| format!("No se pudo exportar: {}", error))
}

#[tauri::command]
pub fn import_domains(
    json: String,
    state: State<'_, AppState>,
) -> Result<Vec<Domain>, String> {
    let imported: Vec<Domain> = serde_json::from_str(&json)
        .map_err(|_| "Archivo de importación inválido".to_string())?;

    if imported.len() > 500 {
        return Err("Demasiados dominios (máx. 500)".to_string());
    }

    let mut validated: Vec<Domain> = Vec::with_capacity(imported.len());

    for item in imported {
        let hostname = validate_hostname(&item.hostname)?;
        let ip = validate_ip(&item.ip)?;

        validated.push(Domain {
            id: uuid::Uuid::new_v4().to_string(),
            hostname,
            ip,
            enabled: false,
            group: normalize_group(&item.group),
        });
    }

    // Sin duplicados por hostname.
    let mut seen = std::collections::HashSet::new();

    for domain in &validated {
        if !seen.insert(domain.hostname.clone()) {
            return Err(format!(
                "Hostname duplicado en la importación: {}",
                domain.hostname
            ));
        }
    }

    let mut domains = state
        .domains
        .lock()
        .map_err(|_| "No se pudo acceder al estado".to_string())?;

    *domains = validated.clone();

    persist(&state, &domains)?;

    Ok(domains.clone())
}

#[tauri::command]
pub fn flush_dns() -> Result<String, String> {
    HelperClient::flush_dns()
}

#[tauri::command]
pub fn helper_status() -> Result<String, String> {
    HelperClient::ping()
}

#[tauri::command]
pub fn get_hosts_content(state: State<'_, AppState>) -> Result<String, String> {
    state.hosts_manager.read()
}
