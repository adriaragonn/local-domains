use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub hostname: String,
    pub ip: String,
    pub enabled: bool,
    #[serde(default)]
    pub group: String,
}

impl Domain {
    pub fn normalized_group(&self) -> String {
        self.group.trim().to_string()
    }
}

pub fn validate_hostname(hostname: &str) -> Result<String, String> {
    let hostname = hostname.trim().to_lowercase();

    if hostname.is_empty() {
        return Err("El hostname no puede estar vacío".to_string());
    }

    if hostname.len() > 253 {
        return Err("El hostname es demasiado largo".to_string());
    }

    let valid = hostname.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || c == '.'
            || c == '-'
            || c == '_'
    });

    if !valid {
        return Err(
            "Hostname inválido: usa solo letras, números, puntos y guiones"
                .to_string(),
        );
    }

    if !hostname.contains('.') {
        return Err(
            "Hostname inválido: debe contener al menos un punto (ej. mi-app.test)"
                .to_string(),
        );
    }

    if hostname.starts_with('.')
        || hostname.starts_with('-')
        || hostname.ends_with('.')
        || hostname.ends_with('-')
    {
        return Err("Hostname inválido: formato incorrecto".to_string());
    }

    if hostname.contains("..") {
        return Err("Hostname inválido: no puede contener '..'".to_string());
    }

    Ok(hostname)
}

pub fn validate_ip(ip: &str) -> Result<String, String> {
    let ip = ip.trim().to_string();

    if ip.is_empty() {
        return Err("La IP no puede estar vacía".to_string());
    }

    // Acepta IPv4 e IPv6 usando el parser estándar.
    if ip.parse::<std::net::IpAddr>().is_err() {
        return Err(
            "IP inválida: usa una IPv4 (127.0.0.1) o IPv6 válida"
                .to_string(),
        );
    }

    Ok(ip)
}

pub fn normalize_group(group: &str) -> String {
    group.trim().to_string()
}
