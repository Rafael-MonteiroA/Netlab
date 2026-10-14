'use strict';
function svgNode(tag, attributes, text) { const node = document.createElementNS('http://www.w3.org/2000/svg', tag); Object.entries(attributes).forEach(([key, value]) => node.setAttribute(key, value)); if (text !== undefined) node.textContent = text; return node; }
function drawMap(id, peers, center, subtitle) {
  const svg = $(id); svg.replaceChildren(); const cx = 350, cy = 178;
  peers.forEach((peer, index) => {
    const angle = (index / peers.length) * Math.PI * 2 - Math.PI / 2;
    const x = cx + Math.cos(angle) * 240, y = cy + Math.sin(angle) * 116;
    svg.append(svgNode('line', { x1: cx, y1: cy, x2: x, y2: y, class: `wire${peer.enabled ? '' : ' off'}` }));
    const group = svgNode('g', {}); group.append(svgNode('title', {}, `${peer.name} — ${peer.detail}`), svgNode('rect', { x: x - 29, y: y - 24, width: 58, height: 48, rx: 12, class: 'map-node' }), svgNode('text', { x, y: y + 8, 'text-anchor': 'middle', class: 'map-icon' }, '▣'), svgNode('text', { x, y: y + 42, 'text-anchor': 'middle', class: 'map-title' }, peer.name.length > 17 ? peer.name.slice(0, 16) + '…' : peer.name), svgNode('text', { x, y: y + 58, 'text-anchor': 'middle', class: 'map-subtitle' }, peer.detail)); svg.append(group);
  });
  svg.append(svgNode('circle', { cx, cy, r: 40, class: 'map-center' }), svgNode('text', { x: cx, y: cy + 9, 'text-anchor': 'middle', class: 'map-core-icon' }, '↔'), svgNode('text', { x: cx, y: cy + 62, 'text-anchor': 'middle', class: 'map-title' }, center), svgNode('text', { x: cx, y: cy + 80, 'text-anchor': 'middle', class: 'map-subtitle' }, subtitle));
  if (!peers.length) svg.append(svgNode('text', { x: cx, y: 323, 'text-anchor': 'middle', class: 'map-empty' }, 'Crie uma rede para começar a conectar.'));
}

const defaults = [{ name: 'PC1', ip: '10.0.0.1', latency_ms: 5, enabled: true }, { name: 'PC2', ip: '10.0.0.2', latency_ms: 7, enabled: true }];
let hosts = structuredClone(defaults);
function validateHosts(input) {
  if (!Array.isArray(input) || input.length < 2 || input.length > 16) throw new Error('O cenário deve ter entre 2 e 16 hosts.');
  const names = new Set(), ips = new Set();
  return input.map(host => {
    if (!host || typeof host.name !== 'string' || !host.name.trim() || host.name.length > 40 || typeof host.ip !== 'string' || !/^\d{1,3}(\.\d{1,3}){3}$/.test(host.ip) || host.ip.split('.').some(n => Number(n) > 255) || !Number.isInteger(host.latency_ms) || host.latency_ms < 0 || host.latency_ms > 1000 || typeof host.enabled !== 'boolean') throw new Error('Host inválido: confira nome, IPv4, latência (0–1000 ms) e estado do link.');
    if (names.has(host.name.trim()) || ips.has(host.ip)) throw new Error('Nomes e endereços IP não podem se repetir.');
    names.add(host.name.trim()); ips.add(host.ip);
    return { name: host.name.trim(), ip: host.ip, latency_ms: host.latency_ms, enabled: host.enabled };
  });
}
try { hosts = validateHosts(JSON.parse(stored('netlab.hosts', 'null'))); } catch { /* Use the working default topology. */ }
function invalidateSimulation() { $('sim-summary').replaceChildren(element('strong', 'Topologia atualizada.'), element('p', 'Execute novamente para obter os resultados desta configuração.')); $('event-count').textContent = 'Aguardando execução'; $('packet-list').replaceChildren(element('div', 'Execute um ping para inspecionar os pacotes.', 'empty-small')); }
function saveHosts() { save('netlab.hosts', JSON.stringify(hosts)); drawMap('sim-map', hosts.map(h => ({ name: h.name, detail: `${h.ip} · ${h.enabled ? h.latency_ms + ' ms' : 'desligado'}`, enabled: h.enabled })), 'SW1', `${hosts.length} portas · switch virtual`); }
function renderHosts() {
  $('hosts-table').replaceChildren();
  hosts.forEach((host, index) => {
    const row = element('tr');
    [['name', 'text'], ['ip', 'text'], ['latency_ms', 'number']].forEach(([key, type]) => {
      const cell = element('td'), input = element('input'); input.type = type; input.value = host[key]; input.setAttribute('aria-label', `${key === 'name' ? 'Nome' : key === 'ip' ? 'IPv4' : 'Latência'} do host ${index + 1}`);
      if (type === 'number') { input.min = '0'; input.max = '1000'; } else input.maxLength = key === 'name' ? 40 : 15;
      input.onchange = () => { host[key] = type === 'number' ? Number(input.value) : input.value; invalidateSimulation(); saveHosts(); renderHostSelects(); };
      cell.append(input); row.append(cell);
    });
    const toggleCell = element('td'), toggle = element('input'); toggle.type = 'checkbox'; toggle.checked = host.enabled; toggle.setAttribute('aria-label', `Ativar link do host ${index + 1}`); toggle.onchange = () => { host.enabled = toggle.checked; invalidateSimulation(); saveHosts(); }; toggleCell.append(toggle);
    const removeCell = element('td'), remove = element('button', '×'); remove.disabled = hosts.length <= 2; remove.setAttribute('aria-label', `Remover host ${index + 1}`); remove.onclick = () => { hosts.splice(index, 1); invalidateSimulation(); renderHosts(); }; removeCell.append(remove);
    row.append(toggleCell, removeCell); $('hosts-table').append(row);
  });
  $('add-host').disabled = hosts.length >= 16; renderHostSelects(); saveHosts();
}
function renderHostSelects() {
  for (const id of ['source', 'target']) { const previous = $(id).value; $(id).replaceChildren(); hosts.forEach((h, i) => $(id).add(new Option(`${h.name} · ${h.ip}`, String(i)))); $(id).value = previous && Number(previous) < hosts.length ? previous : id === 'target' ? '1' : '0'; }
}
$('add-host').onclick = () => { if (hosts.length >= 16) return; let n = 1; while (hosts.some(h => h.name === `PC${n}` || h.ip === `10.0.0.${n}`)) n++; hosts.push({ name: `PC${n}`, ip: `10.0.0.${n}`, latency_ms: 5, enabled: true }); invalidateSimulation(); renderHosts(); };
$('simulate-form').onsubmit = async event => {
  event.preventDefault(); notice('simulation-error'); $('run-ping').disabled = true;
  try {
    const scenario = { hosts: validateHosts(hosts), source: Number($('source').value), target: Number($('target').value), count: Number($('ping-count').value) };
    const response = await fetch('/api/simulate', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(scenario), signal: AbortSignal.timeout(10000) });
    const result = await response.json(); if (!response.ok) throw new Error(result.error || 'Não foi possível executar. Confira os parâmetros.');
    const stats = element('div', undefined, 'sim-stats'); [[result.sent, 'enviados'], [result.received, 'recebidos'], [result.lost, 'perdidos'], [result.average_rtt === null ? '—' : `${result.average_rtt} ms`, 'RTT virtual médio']].forEach(([value, label]) => { const stat = element('div'); stat.append(element('strong', value), element('span', label)); stats.append(stat); });
    $('sim-summary').replaceChildren(stats); $('event-count').textContent = `${result.packets.length} pacotes · ${result.events.length} eventos`; $('packet-list').replaceChildren();
    result.packets.forEach(packet => { const detail = element('details'), summary = element('summary', `#${packet.id}  ${packet.source} → ${packet.target}  /  ${packet.status}`); detail.append(summary); packet.trace.forEach(trace => { const line = element('div', undefined, 'trace'); line.append(element('time', `${trace.time_ms} ms`), document.createTextNode(trace.detail)); detail.append(line); }); $('packet-list').append(detail); });
    $('sim-map').querySelectorAll('.wire:not(.off)').forEach(line => { line.classList.remove('pulse'); requestAnimationFrame(() => line.classList.add('pulse')); });
  } catch (error) { notice('simulation-error', error.message); }
  finally { $('run-ping').disabled = false; }
};
$('export-scenario').onclick = () => {
  try { const blob = new Blob([JSON.stringify({ format: 'netlab-star-v1', hosts: validateHosts(hosts) }, null, 2)], { type: 'application/json' }); const url = URL.createObjectURL(blob), a = element('a'); a.href = url; a.download = 'netlab-rede.json'; a.click(); setTimeout(() => URL.revokeObjectURL(url), 1000); }
  catch (error) { notice('simulation-error', error.message); }
};
$('import-scenario').onchange = async event => {
  try { const file = event.target.files[0]; if (!file) return; if (file.size > 32768) throw new Error('O cenário deve ter no máximo 32 KB.'); const parsed = JSON.parse(await file.text()); if (parsed.format !== 'netlab-star-v1') throw new Error('Formato incompatível. Use um JSON exportado por este laboratório.'); hosts = validateHosts(parsed.hosts); invalidateSimulation(); renderHosts(); notice('simulation-error'); toast('Topologia importada.'); }
  catch (error) { notice('simulation-error', error.message); } finally { event.target.value = ''; }
};
renderPeers(); renderHosts();

// Diagnostics use an explicit allowlist; never include errors or peer/session data.
$('support-generate').onclick=async()=>{await btReady;$('support-report').value=JSON.stringify({application:'NetLab',version:'0.7.0',reportFormat:1,bluetoothConnected:bt.online,bluetoothPending:bt.pending.size,protectedStorageAvailable:bt.storageHealthy,readReceipts:bt.readReceipts,networkConnected:state.socket?.readyState===WebSocket.OPEN},null,2);$('support-status').textContent='Relatório gerado localmente. Revise o texto antes de compartilhar.';};
$('support-copy').onclick=async()=>{try{if(!$('support-report').value)return;await navigator.clipboard.writeText($('support-report').value);$('support-status').textContent='Relatório copiado.';}catch{$('support-status').textContent='Selecione o texto e copie com Ctrl+C.';}};
$('support-download').onclick=()=>{if($('support-report').value)downloadText('NetLab-diagnostico.txt',$('support-report').value);};
$('large-text').checked=stored('netlab.largeText','false')==='true';
function applyTextSize(){document.documentElement.classList.toggle('large-text',$('large-text').checked);save('netlab.largeText',String($('large-text').checked));}
$('large-text').onchange=applyTextSize;applyTextSize();
