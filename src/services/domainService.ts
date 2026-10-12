import { invoke } from '@tauri-apps/api/core';
import { Domain } from '../types/Domain';

export async function getDomains(): Promise<Domain[]> {
  return await invoke<Domain[]>('get_domains');
}

export async function toggleDomain(id: string): Promise<Domain[]> {
  return await invoke<Domain[]>('toggle_domain', { id });
}

export async function createDomain(
  hostname: string,
  ip: string,
  group: string,
): Promise<Domain[]> {
  return await invoke<Domain[]>('create_domain', {
    hostname,
    ip,
    group,
  });
}

export async function updateDomain(
  id: string,
  hostname: string,
  ip: string,
  group: string,
): Promise<Domain[]> {
  return await invoke<Domain[]>('update_domain', {
    id,
    hostname,
    ip,
    group,
  });
}

export async function deleteDomain(id: string): Promise<Domain[]> {
  return await invoke<Domain[]>('delete_domain', { id });
}

export async function enableAll(): Promise<Domain[]> {
  return await invoke<Domain[]>('enable_all');
}

export async function disableAll(): Promise<Domain[]> {
  return await invoke<Domain[]>('disable_all');
}

export async function toggleGroup(
  group: string,
  enabled: boolean,
): Promise<Domain[]> {
  return await invoke<Domain[]>('toggle_group', { group, enabled });
}

export async function exportDomains(): Promise<string> {
  return await invoke<string>('export_domains');
}

export async function importDomains(json: string): Promise<Domain[]> {
  return await invoke<Domain[]>('import_domains', { json });
}

export async function flushDns(): Promise<string> {
  return await invoke<string>('flush_dns');
}

export async function helperStatus(): Promise<string> {
  return await invoke<string>('helper_status');
}

export function validateHostname(hostname: string): string | null {
  const value = hostname.trim().toLowerCase();

  if (!value) return 'El hostname no puede estar vacío';
  if (!value.includes('.')) return 'Debe contener un punto (ej. mi-app.test)';
  if (!/^[a-z0-9._-]+$/.test(value)) return 'Solo letras, números, puntos y guiones';
  if (value.includes('..')) return 'Formato inválido';

  return null;
}

export function validateIp(ip: string): string | null {
  const value = ip.trim();

  if (!value) return 'La IP no puede estar vacía';

  const ipv4 = /^(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}$/;
  const ipv6 = /^[0-9a-fA-F:]+$/.test(value) && value.includes(':');

  if (!ipv4.test(value) && !ipv6) return 'IP inválida (ej. 127.0.0.1)';

  return null;
}
