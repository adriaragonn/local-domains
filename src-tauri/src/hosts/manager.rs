use std::{fs, path::PathBuf};

use crate::models::Domain;

const START_MARKER: &str = "# LOCAL-DOMAINS-MANAGER";
const END_MARKER: &str = "# END-LOCAL-DOMAINS-MANAGER";

pub struct HostsManager {
    hosts_path: PathBuf,
}

impl HostsManager {
    pub fn new() -> Self {
        Self {
            hosts_path: default_hosts_path(),
        }
    }

    pub fn read(&self) -> Result<String, String> {
        fs::read_to_string(&self.hosts_path)
            .map_err(|error| {
                format!("No se pudo leer el archivo hosts: {}", error)
            })
    }

    pub fn write(&self, content: &str) -> Result<(), String> {
        self.backup()?;

        fs::write(&self.hosts_path, content)
            .map_err(|error| {
                format!("No se pudo escribir el archivo hosts: {}", error)
            })
    }

    /// Guarda una copia de seguridad antes de cada escritura.
    /// No falla la operación si el backup falla: lo devuelve como
    /// advertencia registrada por el llamador si lo necesita.
    fn backup(&self) -> Result<(), String> {
        let original = fs::read(&self.hosts_path).map_err(|error| {
            format!("No se pudo leer el archivo hosts: {}", error)
        })?;

        let backup_path = self.hosts_path.with_extension("ldm.bak");

        fs::write(&backup_path, original).map_err(|error| {
            format!("No se pudo crear la copia de seguridad: {}", error)
        })?;

        Ok(())
    }

    pub fn enable_domain(
        &self,
        domain: &Domain,
    ) -> Result<(), String> {
        self.enable(&domain.ip, &domain.hostname)
    }

    pub fn disable_domain(
        &self,
        hostname: &str,
    ) -> Result<(), String> {
        self.disable(hostname)
    }

    pub fn enable(
        &self,
        ip: &str,
        hostname: &str,
    ) -> Result<(), String> {
        let content = self.read()?;

        let updated = self.add_or_update_domain(
            &content,
            ip,
            hostname,
        );

        self.write(&updated)
    }

    pub fn disable(
        &self,
        hostname: &str,
    ) -> Result<(), String> {
        let content = self.read()?;

        let updated = self.remove_domain(
            &content,
            hostname,
        );

        self.write(&updated)
    }

    fn add_or_update_domain(
        &self,
        content: &str,
        ip: &str,
        hostname: &str,
    ) -> String {
        let mut lines: Vec<String> =
            content.lines().map(String::from).collect();

        let domain_line = format!("{} {}", ip, hostname);

        let start_index = lines
            .iter()
            .position(|line| line.trim() == START_MARKER);

        let end_index = lines
            .iter()
            .position(|line| line.trim() == END_MARKER);

        match (start_index, end_index) {
            (Some(start), Some(end)) if start < end => {
                let existing_index = lines[start + 1..end]
                    .iter()
                    .position(|line| {
                        Self::extract_hostname(line)
                            .is_some_and(|value| value == hostname)
                    })
                    .map(|index| start + 1 + index);

                match existing_index {
                    Some(index) => {
                        lines[index] = domain_line;
                    }

                    None => {
                        lines.insert(end, domain_line);
                    }
                }
            }

            _ => {
                if !lines.is_empty()
                    && !lines.last().unwrap().is_empty()
                {
                    lines.push(String::new());
                }

                lines.push(START_MARKER.to_string());
                lines.push(domain_line);
                lines.push(END_MARKER.to_string());
            }
        }

        Self::normalize_line_endings(lines)
    }

    fn remove_domain(
        &self,
        content: &str,
        hostname: &str,
    ) -> String {
        let mut lines: Vec<String> =
            content.lines().map(String::from).collect();

        let start_index = lines
            .iter()
            .position(|line| line.trim() == START_MARKER);

        let end_index = lines
            .iter()
            .position(|line| line.trim() == END_MARKER);

        let (Some(start), Some(end)) = (start_index, end_index) else {
            return content.to_string();
        };

        if start >= end {
            return content.to_string();
        }

        // Buscar únicamente el dominio que queremos eliminar.
        let domain_index = lines[start + 1..end]
            .iter()
            .position(|line| {
                Self::extract_hostname(line)
                    .is_some_and(|value| value == hostname)
            })
            .map(|index| start + 1 + index);

        if let Some(index) = domain_index {
            lines.remove(index);
        } else {
            return content.to_string();
        }

        // Comprobar si todavía queda algún dominio gestionado.
        let new_end = lines
            .iter()
            .position(|line| line.trim() == END_MARKER);

        if let Some(new_end) = new_end {
            let remaining_domains = lines
                .iter()
                .skip(start + 1)
                .take(new_end - start - 1)
                .any(|line| Self::extract_hostname(line).is_some());

            if !remaining_domains {
                lines.drain(start..=new_end);
            }
        }

        Self::normalize_line_endings(lines)
    }

    fn extract_hostname(line: &str) -> Option<&str> {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            return None;
        }

        let mut parts = line.split_whitespace();

        parts.next()?;
        parts.next()
    }

    fn normalize_line_endings(
        lines: Vec<String>,
    ) -> String {
        if lines.is_empty() {
            return String::new();
        }

        format!("{}\r\n", lines.join("\r\n"))
    }
}

fn default_hosts_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(
            r"C:\Windows\System32\drivers\etc\hosts",
        )
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/etc/hosts")
    }
}