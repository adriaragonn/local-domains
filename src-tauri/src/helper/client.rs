use std::io::{BufRead, BufReader, Write};

use interprocess::local_socket::{
    prelude::*,
    GenericNamespaced,
    Stream,
};

const PIPE_NAME: &str = "local-domains-manager";

pub struct HelperClient;

impl HelperClient {
    fn send_command(command: &str) -> Result<String, String> {
        let name = PIPE_NAME
            .to_ns_name::<GenericNamespaced>()
            .map_err(|error| error.to_string())?;

        let mut connection =
            Stream::connect(name).map_err(|_| {
                "No se pudo conectar con el helper elevado. Ejecuta hosts-helper.exe como administrador primero.".to_string()
            })?;

        connection
            .write_all(format!("{}\n", command).as_bytes())
            .map_err(|error| error.to_string())?;

        connection
            .flush()
            .map_err(|error| error.to_string())?;

        let mut reader = BufReader::new(connection);

        let mut response = String::new();

        reader
            .read_line(&mut response)
            .map_err(|error| error.to_string())?;

        let response = response.trim().to_string();

        if response.starts_with("OK|") {
            Ok(response
                .strip_prefix("OK|")
                .unwrap_or("")
                .to_string())
        } else if response.starts_with("ERROR|") {
            Err(response
                .strip_prefix("ERROR|")
                .unwrap_or("Helper error")
                .to_string())
        } else if response.is_empty() {
            Err("El helper no respondió".to_string())
        } else {
            // Compatibilidad con respuestas antiguas sin prefijo.
            Ok(response)
        }
    }

    pub fn ping() -> Result<String, String> {
        Self::send_command("PING")
    }

    pub fn enable(
        ip: &str,
        hostname: &str,
    ) -> Result<String, String> {
        Self::send_command(
            &format!("ENABLE|{}|{}", ip, hostname)
        )
    }

    pub fn disable(
        hostname: &str,
    ) -> Result<String, String> {
        Self::send_command(
            &format!("DISABLE|{}", hostname)
        )
    }

    pub fn flush_dns() -> Result<String, String> {
        Self::send_command("FLUSH")
    }
}
