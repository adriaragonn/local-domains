import './styles.css';
import {
  getDomains,
  toggleDomain,
  createDomain,
  updateDomain,
  deleteDomain,
  enableAll,
  disableAll,
  toggleGroup,
  exportDomains,
  importDomains,
  flushDns,
  helperStatus,
  validateHostname,
  validateIp,
} from './services/domainService';
import { Domain } from './types/Domain';

let domains: Domain[] = [];
let search = '';
let groupFilter = 'all';
let helperOk = true;

const app = document.querySelector<HTMLDivElement>('#app');

if (!app) {
  throw new Error('App container not found');
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function groups(): string[] {
  const set = new Set<string>();

  for (const d of domains) {
    if (d.group.trim()) set.add(d.group.trim());
  }

  return [...set].sort();
}

function filtered(): Domain[] {
  const q = search.trim().toLowerCase();

  return domains.filter((d) => {
    if (groupFilter !== 'all' && d.group !== groupFilter) return false;
    if (!q) return true;

    return (
      d.hostname.toLowerCase().includes(q) ||
      d.ip.includes(q) ||
      d.group.toLowerCase().includes(q)
    );
  });
}

function toast(message: string, type: 'ok' | 'error' = 'ok') {
  const container = document.querySelector('#toasts');
  if (!container) return;

  const el = document.createElement('div');
  el.className = `toast ${type}`;
  el.textContent = message;
  container.appendChild(el);

  setTimeout(() => el.remove(), 3500);
}

function handleError(error: unknown) {
  const message = typeof error === 'string' ? error : String(error);
  console.error(error);
  toast(message, 'error');
}

async function refresh(newDomains: Domain[]) {
  domains = newDomains;
  renderList();
}

function renderShell() {
  app!.innerHTML = `
  <main class="container">
    <div id="helper-banner"></div>

    <header class="header">
      <div>
        <h1>Local Domains</h1>
        <p>Gestiona tus dominios locales sin tocar el hosts a mano</p>
      </div>
      <button id="btn-add" class="btn primary">+ Añadir</button>
    </header>

    <section class="toolbar">
      <input id="search" type="search" placeholder="Buscar hostname, IP o proyecto…" value="${escapeHtml(search)}" />
      <select id="group-filter"></select>
    </section>

    <section class="toolbar actions">
      <button id="btn-enable-all" class="btn small">Activar todos</button>
      <button id="btn-disable-all" class="btn small">Desactivar todos</button>
      <button id="btn-flush" class="btn small">Limpiar DNS</button>
      <button id="btn-export" class="btn small">Exportar</button>
      <button id="btn-import" class="btn small">Importar</button>
      <input id="import-file" type="file" accept="application/json,.json" hidden />
    </section>

    <section id="domains" class="domains"></section>
    <div id="modal-root"></div>
    <div id="toasts"></div>
  </main>`;

  const searchInput = document.querySelector<HTMLInputElement>('#search')!;
  searchInput.addEventListener('input', () => {
    search = searchInput.value;
    renderList();
  });

  document.querySelector('#btn-add')!.addEventListener('click', () => showForm());
  document.querySelector('#btn-enable-all')!.addEventListener('click', async () => {
    try {
      await refresh(await enableAll());
      toast('Todos los dominios activados');
    } catch (e) { handleError(e); }
  });
  document.querySelector('#btn-disable-all')!.addEventListener('click', async () => {
    try {
      await refresh(await disableAll());
      toast('Todos los dominios desactivados');
    } catch (e) { handleError(e); }
  });
  document.querySelector('#btn-flush')!.addEventListener('click', async () => {
    try {
      await flushDns();
      toast('Caché DNS limpiada');
    } catch (e) { handleError(e); }
  });
  document.querySelector('#btn-export')!.addEventListener('click', async () => {
    try {
      const json = await exportDomains();
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'local-domains.json';
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) { handleError(e); }
  });

  const fileInput = document.querySelector<HTMLInputElement>('#import-file')!;
  document.querySelector('#btn-import')!.addEventListener('click', () => fileInput.click());
  fileInput.addEventListener('change', async () => {
    const file = fileInput.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      await refresh(await importDomains(text));
      toast('Importación completada (todo importado como desactivado)');
    } catch (e) { handleError(e); }
    finally { fileInput.value = ''; }
  });

  renderGroupFilter();
  renderList();
  checkHelper();
}

function renderGroupFilter() {
  const select = document.querySelector<HTMLSelectElement>('#group-filter');
  if (!select) return;

  const all = groups();
  select.innerHTML =
    `<option value="all">Todos los proyectos</option>` +
    all.map((g) => `<option value="${escapeHtml(g)}" ${g === groupFilter ? 'selected' : ''}>${escapeHtml(g)}</option>`).join('');

  select.onchange = () => {
    groupFilter = select.value;
    renderList();
  };
}

function renderHelperBanner() {
  const banner = document.querySelector('#helper-banner');
  if (!banner) return;

  banner.innerHTML = helperOk ? '' : `
    <div class="banner warn">
      <strong>Helper no disponible.</strong>
      Ejecuta <code>hosts-helper.exe</code> como administrador para poder escribir en el hosts.
    </div>`;
}

async function checkHelper() {
  try {
    await helperStatus();
    helperOk = true;
  } catch {
    helperOk = false;
  }
  renderHelperBanner();
}

function renderList() {
  renderGroupFilter();
  const container = document.querySelector<HTMLDivElement>('#domains');
  if (!container) return;

  const list = filtered();

  if (domains.length === 0) {
    container.innerHTML = `
      <div class="empty">
        <h2>Sin dominios todavía</h2>
        <p>Crea el primero: ej. <code>mi-app.test</code> → <code>127.0.0.1</code>. Se guardará en tu hosts con un bloque propio y copia de seguridad.</p>
      </div>`;
    return;
  }

  if (list.length === 0) {
    container.innerHTML = `<div class="empty"><p>Sin resultados para esa búsqueda.</p></div>`;
    return;
  }

  // Agrupar por proyecto.
  const byGroup = new Map<string, Domain[]>();
  for (const d of list) {
    const key = d.group.trim() || 'Sin proyecto';
    if (!byGroup.has(key)) byGroup.set(key, []);
    byGroup.get(key)!.push(d);
  }

  container.innerHTML = [...byGroup.entries()]
    .map(([group, items]) => {
      const allOn = items.every((d) => d.enabled);
      const realGroup = group === 'Sin proyecto' ? '' : group;
      return `
      <section class="group">
        <header class="group-header">
          <strong>${escapeHtml(group)}</strong>
          <span class="muted">${items.filter((d) => d.enabled).length}/${items.length} activos</span>
          ${realGroup ? `
            <button class="btn tiny" data-group-on="${escapeHtml(realGroup)}">Activar proyecto</button>
            <button class="btn tiny" data-group-off="${escapeHtml(realGroup)}">Desactivar</button>
          ` : allOn
            ? `<button class="btn tiny" data-group-off="">Desactivar</button>`
            : ''}
        </header>
        ${items.map((domain) => `
        <article class="domain">
          <div class="domain-info">
            <div class="domain-name">
              <span class="status ${domain.enabled ? 'enabled' : 'disabled'}"></span>
              ${escapeHtml(domain.hostname)}
              ${domain.group ? `<span class="badge">${escapeHtml(domain.group)}</span>` : ''}
            </div>
            <div class="domain-ip">${escapeHtml(domain.ip)}</div>
          </div>
          <div class="domain-actions">
            <button class="icon-btn" title="Abrir http://${escapeHtml(domain.hostname)}" data-open="${domain.id}">↗</button>
            <button class="icon-btn" title="Copiar hostname" data-copy="${domain.id}">⧉</button>
            <button class="edit" data-edit="${domain.id}">Editar</button>
            <button class="delete" data-delete="${domain.id}">Eliminar</button>
            <button class="toggle ${domain.enabled ? 'active' : ''}" data-toggle="${domain.id}">
              ${domain.enabled ? 'ON' : 'OFF'}
            </button>
          </div>
        </article>`).join('')}
      </section>`;
    })
    .join('');

  container.querySelectorAll('[data-toggle]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const id = (btn as HTMLElement).dataset.toggle!;
      try { await refresh(await toggleDomain(id)); }
      catch (e) { handleError(e); }
    });
  });

  container.querySelectorAll('[data-edit]').forEach((btn) => {
    btn.addEventListener('click', () => {
      const id = (btn as HTMLElement).dataset.edit!;
      const domain = domains.find((d) => d.id === id);
      if (domain) showForm(domain);
    });
  });

  container.querySelectorAll('[data-delete]').forEach((btn) => {
    btn.addEventListener('click', () => {
      const id = (btn as HTMLElement).dataset.delete!;
      const domain = domains.find((d) => d.id === id);
      if (domain) showConfirm(domain);
    });
  });

  container.querySelectorAll('[data-copy]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const id = (btn as HTMLElement).dataset.copy!;
      const domain = domains.find((d) => d.id === id);
      if (!domain) return;
      try {
        await navigator.clipboard.writeText(domain.hostname);
        toast('Hostname copiado');
      } catch { toast(domain.hostname); }
    });
  });

  container.querySelectorAll('[data-open]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const id = (btn as HTMLElement).dataset.open!;
      const domain = domains.find((d) => d.id === id);
      if (!domain) return;
      const url = `http://${domain.hostname}`;
      try {
        const { openUrl } = await import('@tauri-apps/plugin-opener');
        await openUrl(url);
      } catch {
        window.open(url, '_blank');
      }
    });
  });

  container.querySelectorAll('[data-group-on]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const group = (btn as HTMLElement).dataset.groupOn!;
      try {
        await refresh(await toggleGroup(group, true));
        toast(`Proyecto "${group}" activado`);
      } catch (e) { handleError(e); }
    });
  });

  container.querySelectorAll('[data-group-off]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const group = (btn as HTMLElement).dataset.groupOff!;
      try {
        if (group) {
          await refresh(await toggleGroup(group, false));
          toast(`Proyecto "${group}" desactivado`);
        } else {
          await refresh(await disableAll());
        }
      } catch (e) { handleError(e); }
    });
  });
}

function showForm(domain?: Domain) {
  const root = document.querySelector('#modal-root')!;
  const isEdit = !!domain;

  root.innerHTML = `
    <div class="overlay">
      <form id="domain-form" class="modal">
        <h2>${isEdit ? 'Editar dominio' : 'Nuevo dominio'}</h2>
        <label>Hostname
          <input id="f-hostname" type="text" placeholder="mi-app.test" value="${escapeHtml(domain?.hostname ?? '')}" required />
          <small class="err" id="e-hostname"></small>
        </label>
        <label>IP
          <input id="f-ip" type="text" placeholder="127.0.0.1" value="${escapeHtml(domain?.ip ?? '127.0.0.1')}" required />
          <small class="err" id="e-ip"></small>
        </label>
        <label>Proyecto (opcional)
          <input id="f-group" type="text" placeholder="tienda, cliente-x…" value="${escapeHtml(domain?.group ?? '')}" list="group-list" />
          <datalist id="group-list">${groups().map((g) => `<option value="${escapeHtml(g)}">`).join('')}</datalist>
        </label>
        <div class="form-actions">
          <button type="button" id="f-cancel">Cancelar</button>
          <button type="submit" class="btn primary">Guardar</button>
        </div>
      </form>
    </div>`;

  document.querySelector('#f-cancel')!.addEventListener('click', () => { root.innerHTML = ''; });

  document.querySelector<HTMLFormElement>('#domain-form')!.addEventListener('submit', async (e) => {
    e.preventDefault();
    const hostname = document.querySelector<HTMLInputElement>('#f-hostname')!.value;
    const ip = document.querySelector<HTMLInputElement>('#f-ip')!.value;
    const group = document.querySelector<HTMLInputElement>('#f-group')!.value;

    const hErr = validateHostname(hostname);
    const iErr = validateIp(ip);
    document.querySelector('#e-hostname')!.textContent = hErr ?? '';
    document.querySelector('#e-ip')!.textContent = iErr ?? '';
    if (hErr || iErr) return;

    try {
      if (isEdit) {
        await refresh(await updateDomain(domain!.id, hostname.trim(), ip.trim(), group.trim()));
        toast('Dominio actualizado');
      } else {
        await refresh(await createDomain(hostname.trim(), ip.trim(), group.trim()));
        toast('Dominio creado (desactivado por defecto)');
      }
      root.innerHTML = '';
    } catch (err) { handleError(err); }
  });
}

function showConfirm(domain: Domain) {
  const root = document.querySelector('#modal-root')!;
  root.innerHTML = `
    <div class="overlay">
      <div class="modal">
        <h2>Eliminar ${escapeHtml(domain.hostname)}</h2>
        <p>Se quitará del hosts ${domain.enabled ? '(ahora mismo está activo)' : ''} y de tu lista. Esta acción no se puede deshacer.</p>
        <div class="form-actions">
          <button id="c-cancel">Cancelar</button>
          <button id="c-ok" class="btn danger">Eliminar</button>
        </div>
      </div>
    </div>`;

  document.querySelector('#c-cancel')!.addEventListener('click', () => { root.innerHTML = ''; });
  document.querySelector('#c-ok')!.addEventListener('click', async () => {
    try {
      await refresh(await deleteDomain(domain.id));
      root.innerHTML = '';
      toast('Dominio eliminado');
    } catch (e) { handleError(e); }
  });
}

async function init() {
  renderShell();
  try {
    domains = await getDomains();
    renderList();
  } catch (error) {
    handleError(error);
  }
}

init().catch(handleError);
