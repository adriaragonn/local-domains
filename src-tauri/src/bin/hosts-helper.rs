use local_domains_manager_lib::hosts::HostsManager;

use std::{
    io::{BufRead, BufReader, Write},
    sync::mpsc,
    time::Duration,
};

use interprocess::{
    local_socket::{
        prelude::*,
        GenericNamespaced,
        ListenerNonblockingMode,
        ListenerOptions,
    },
    os::windows::{
        local_socket::ListenerOptionsExt,
        security_descriptor::SecurityDescriptor,
    },
};

use widestring::U16CString;

use windows_service::{
    define_windows_service,
    service::{
        ServiceControl,
        ServiceControlAccept,
        ServiceExitCode,
        ServiceState,
        ServiceStatus,
        ServiceType,
    },
    service_control_handler::{
        self,
        ServiceControlHandlerResult,
    },
    service_dispatcher,
};

const PIPE_NAME: &str = "local-domains-manager";
const SERVICE_NAME: &str = "LocalDomainsHelper";

define_windows_service!(ffi_service_main, service_main);

fn create_security_descriptor(
) -> Result<SecurityDescriptor, Box<dyn std::error::Error>> {
    let sddl = U16CString::from_str("D:P(A;;GA;;;AU)")?;

    let descriptor =
        SecurityDescriptor::deserialize(sddl.as_ucstr())?;

    Ok(descriptor)
}

fn create_listener(
) -> Result<
    interprocess::local_socket::Listener,
    Box<dyn std::error::Error>,
> {
    let name = PIPE_NAME
        .to_ns_name::<GenericNamespaced>()?;

    let security_descriptor =
        create_security_descriptor()?;

    let listener = ListenerOptions::new()
        .name(name)
        .nonblocking(ListenerNonblockingMode::Accept)
        .security_descriptor(security_descriptor)
        .create_sync()?;

    Ok(listener)
}

fn run_helper(
    stop_receiver: Option<mpsc::Receiver<()>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = create_listener()?;

    println!("Hosts Helper running");

    loop {
        // -------------------------------------------------
        // Comprobar si Windows nos ha pedido detenernos
        // -------------------------------------------------
        if let Some(receiver) = &stop_receiver {
            match receiver.try_recv() {
                Ok(_) => {
                    println!("Stopping Hosts Helper");
                    break;
                }

                Err(mpsc::TryRecvError::Disconnected) => {
                    println!("Stop channel disconnected");
                    break;
                }

                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        // -------------------------------------------------
        // Esperar conexiones
        //
        // El listener es NON-BLOCKING, por lo que podemos
        // comprobar periódicamente la señal de STOP.
        // -------------------------------------------------
        let connection = match listener.incoming().next() {
            Some(Ok(connection)) => connection,

            Some(Err(error))
                if error.kind() == std::io::ErrorKind::WouldBlock =>
            {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }

            Some(Err(error)) => {
                eprintln!("Connection error: {}", error);
                continue;
            }

            None => {
                break;
            }
        };

        let mut reader = BufReader::new(connection);

        let mut command = String::new();

        if reader.read_line(&mut command).is_err() {
            continue;
        }

        let command = command.trim();

        println!("Received: {}", command);

        let response = match command {
            // -------------------------------------------------
            // PING
            // -------------------------------------------------
            "PING" => {
                "OK|PONG\n".to_string()
            }

            // -------------------------------------------------
            // FLUSH - limpia la caché DNS del sistema
            // -------------------------------------------------
            "FLUSH" => match flush_dns() {
                Ok(_) => "OK|FLUSHED\n".to_string(),
                Err(error) => format!("ERROR|{}\n", error),
            },

            // -------------------------------------------------
            // QUIT
            // -------------------------------------------------
            "QUIT" => {
                "OK|BYE\n".to_string()
            }

            // -------------------------------------------------
            // ENABLE
            // ENABLE|127.0.0.1|mi-local.local.com
            // -------------------------------------------------
            _ if command.starts_with("ENABLE|") => {
                let parts: Vec<&str> =
                    command.split('|').collect();

                if parts.len() != 3 {
                    "ERROR|INVALID_ENABLE_COMMAND\n".to_string()
                } else {
                    let ip = parts[1].trim();
                    let hostname =
                        parts[2].trim().to_lowercase();

                    if ip.parse::<std::net::IpAddr>().is_err() {
                        "ERROR|IP inválida\n".to_string()
                    } else if hostname.is_empty()
                        || !hostname.contains('.')
                    {
                        "ERROR|Hostname inválido\n".to_string()
                    } else {
                        let manager =
                            HostsManager::new();

                        match manager.enable(ip, &hostname) {
                            Ok(_) => {
                                "OK|ENABLED\n".to_string()
                            }

                            Err(error) => {
                                format!("ERROR|{}\n", error)
                            }
                        }
                    }
                }
            }

            // -------------------------------------------------
            // DISABLE
            // DISABLE|mi-local.local.com
            // -------------------------------------------------
            _ if command.starts_with("DISABLE|") => {
                let parts: Vec<&str> =
                    command.split('|').collect();

                if parts.len() != 2 {
                    "ERROR|INVALID_DISABLE_COMMAND\n".to_string()
                } else {
                    let hostname = parts[1];

                    let manager =
                        HostsManager::new();

                    match manager.disable_domain(hostname) {
                        Ok(_) => {
                            "OK|DISABLED\n".to_string()
                        }

                        Err(error) => {
                            format!("ERROR|{}\n", error)
                        }
                    }
                }
            }

            // -------------------------------------------------
            // UNKNOWN
            // -------------------------------------------------
            _ => {
                "ERROR|UNKNOWN_COMMAND\n".to_string()
            }
        };

        if let Err(error) =
            reader.get_mut().write_all(response.as_bytes())
        {
            eprintln!("Response error: {}", error);
        }

        let _ = reader.get_mut().flush();

        if command == "QUIT" {
            break;
        }
    }

    Ok(())
}

// ---------------------------------------------------------
// Limpieza de caché DNS por plataforma
// ---------------------------------------------------------

fn flush_dns() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("ipconfig")
            .arg("/flushdns")
            .output()
            .map_err(|error| {
                format!("No se pudo limpiar la DNS: {}", error)
            })
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    Err("ipconfig /flushdns falló".to_string())
                }
            })
    }

    #[cfg(target_os = "macos")]
    {
        let lookup = std::process::Command::new("dscacheutil")
            .arg("-flushcache")
            .output()
            .map_err(|error| {
                format!("No se pudo limpiar la DNS: {}", error)
            })?;

        let mdns = std::process::Command::new("killall")
            .arg("-HUP")
            .arg("mDNSResponder")
            .output()
            .map_err(|error| {
                format!("No se pudo limpiar la DNS: {}", error)
            })?;

        if lookup.status.success() && mdns.status.success() {
            Ok(())
        } else {
            Err("No se pudo limpiar la DNS en macOS".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Intenta systemd-resolve, si no existe no falla duro.
        let result = std::process::Command::new("systemd-resolve")
            .arg("--flush-caches")
            .output();

        match result {
            Ok(output) if output.status.success() => Ok(()),
            _ => Err(
                "No se pudo limpiar la DNS en Linux (systemd-resolve no disponible)"
                    .to_string(),
            ),
        }
    }

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    )))]
    {
        Err("Plataforma no soportada para flush DNS".to_string())
    }
}

// ---------------------------------------------------------
// Windows Service
// ---------------------------------------------------------

fn service_main(_arguments: Vec<std::ffi::OsString>) {
    let (stop_sender, stop_receiver) =
        mpsc::channel();

    let event_handler =
        move |event| -> ServiceControlHandlerResult {
            match event {
                ServiceControl::Stop
                | ServiceControl::Shutdown => {
                    println!("Service stop requested");

                    let _ = stop_sender.send(());

                    ServiceControlHandlerResult::NoError
                }

                _ => {
                    ServiceControlHandlerResult::NoError
                }
            }
        };

    let status_handle =
        match service_control_handler::register(
            SERVICE_NAME,
            event_handler,
        ) {
            Ok(handle) => handle,

            Err(error) => {
                eprintln!(
                    "Failed to register service handler: {}",
                    error
                );

                return;
            }
        };

    // -----------------------------------------------------
    // RUNNING
    // -----------------------------------------------------

    let status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,

        current_state: ServiceState::Running,

        controls_accepted:
            ServiceControlAccept::STOP
                | ServiceControlAccept::SHUTDOWN,

        exit_code:
            ServiceExitCode::Win32(0),

        checkpoint: 0,

        wait_hint: Duration::default(),

        process_id: None,
    };

    if let Err(error) =
        status_handle.set_service_status(status)
    {
        eprintln!(
            "Failed to set service status: {}",
            error
        );

        return;
    }

    println!("LocalDomainsHelper service started");

    // -----------------------------------------------------
    // MAIN LOOP
    // -----------------------------------------------------

    if let Err(error) =
        run_helper(Some(stop_receiver))
    {
        eprintln!(
            "Helper error: {}",
            error
        );
    }

    // -----------------------------------------------------
    // STOPPED
    // -----------------------------------------------------

    let status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,

        current_state: ServiceState::Stopped,

        controls_accepted:
            ServiceControlAccept::empty(),

        exit_code:
            ServiceExitCode::Win32(0),

        checkpoint: 0,

        wait_hint: Duration::default(),

        process_id: None,
    };

    let _ =
        status_handle.set_service_status(status);

    println!("LocalDomainsHelper service stopped");
}

// ---------------------------------------------------------
// Console mode
//
// Esto permite seguir ejecutando:
//
// hosts-helper.exe
//
// directamente desde PowerShell durante desarrollo.
// ---------------------------------------------------------

fn run_console()
    -> Result<(), Box<dyn std::error::Error>>
{
    run_helper(None)
}

// ---------------------------------------------------------
// MAIN
// ---------------------------------------------------------

fn main()
    -> Result<(), Box<dyn std::error::Error>>
{
    // Cuando Windows inicia el ejecutable como servicio,
    // service_dispatcher conecta nuestro proceso con
    // el Service Control Manager.
    //
    // Si no estamos siendo ejecutados como servicio,
    // ejecutamos el helper en modo consola.

    match service_dispatcher::start(
        SERVICE_NAME,
        ffi_service_main,
    ) {
        Ok(_) => Ok(()),

        Err(_) => {
            run_console()
        }
    }
}