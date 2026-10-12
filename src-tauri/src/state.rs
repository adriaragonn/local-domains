use std::sync::Mutex;

use crate::{hosts::HostsManager, models::Domain, repositories::DomainRepository};

pub struct AppState {
    pub domains: Mutex<Vec<Domain>>,
    pub repository: DomainRepository,
    pub hosts_manager: HostsManager,
}

impl AppState {
    pub fn new(repository: DomainRepository, domains: Vec<Domain>) -> Self {
        Self {
            domains: Mutex::new(domains),
            repository,
            hosts_manager: HostsManager::new(),
        }
    }
}
