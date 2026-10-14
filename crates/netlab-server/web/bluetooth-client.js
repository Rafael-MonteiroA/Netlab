'use strict';
const bt={socket:null,address:null,online:false,manual:true,retry:null,attempt:0,timer:null,pending:new Map(),rows:new Map(),unread:new Set(),seen:new Map(),records:new Map(),peers:new Map(),lastSeen:0,offset:0,count:0,readReceipts:true,storageHealthy:false};
let btQrBlob=null,btIO=Promise.resolve(),btLoadTicket=0,btSelectTicket=0,btSending=false;
function btSave(task){const result=btIO.then(task);btIO=result.catch(()=>{});return result;}
function btStatus(text){$('bt-status').textContent=text;}
function btHideQr(){$('bt-qr-panel').hidden=true;if(btQrBlob)URL.revokeObjectURL(btQrBlob);btQrBlob=null;$('bt-invite-qr').removeAttribute('src');}
function btStop(){btHideQr();bt.manual=true;clearTimeout(bt.retry);bt.retry=null;clearInterval(bt.timer);const old=bt.socket;bt.socket=null;old?.close();bt.online=false;btStatus('Desconectado. Histórico e fila permanecem protegidos neste Windows.');}
function btSend(value){if(!bt.online||bt.socket?.readyState!==WebSocket.OPEN)return false;bt.socket.send(JSON.stringify(value));return true;}
function btFlush(){for(const v of bt.pending.values())if(!btSend(v))break;}
function btError(error){bt.storageHealthy=false;btStatus(error.message||'Histórico indisponível. Nenhuma confirmação foi enviada.');}
function btPeer(address){return bt.peers.get(address)||{address,name:address,draft:'',blocked:false};}
function btPeerList(){const select=$('bt-history-select');select.replaceChildren(new Option('Escolha uma conversa salva',''));for(const p of bt.peers.values())select.add(new Option(p.name+(p.blocked?' (bloqueado)':''),p.address));select.value=bt.address||'';}
async function btHistoryPage(){if(!bt.address)return;const address=bt.address,ticket=++btLoadTicket;const page=await vault.list('bt-history:'+address,bt.offset);if(ticket!==btLoadTicket||bt.address!==address)return;bt.rows.clear();bt.records.clear();$('bt-messages').replaceChildren();for(const row of page.records.reverse()){bt.records.set(row.key,row.value);btRow(row.value,row.value.mine);}bt.count=page.count;$('bt-history-count').textContent=`${page.count} mensagens salvas`;$('bt-older').disabled=bt.offset+200>=page.count;$('bt-newer').disabled=bt.offset===0;}
async function btSelect(address){
 if(!address)return;const ticket=++btSelectTicket;btStop();await btIO;
 const queued=await vault.all('bt-outbox:'+address);if(ticket!==btSelectTicket)return;
 bt.address=address;bt.offset=0;bt.pending.clear();bt.unread.clear();bt.seen.clear();
 const peer=btPeer(address);bt.peers.set(address,peer);for(const row of queued)bt.pending.set(row.key,row.value);
 await btSave(()=>vault.put('bt-settings','last',{address}));if(ticket!==btSelectTicket)return;
 $('bt-message').value=peer.draft||'';$('bt-block').textContent=peer.blocked?'Liberar aparelho':'Bloquear aparelho';await btHistoryPage();btPeerList();
}
async function btRestore(){try{
 for(const row of await vault.all('bt-peers'))bt.peers.set(row.key,row.value);
 const settings=await vault.list('bt-settings'),saved=new Map(settings.records.map(r=>[r.key,r.value]));bt.readReceipts=saved.get('privacy')?.readReceipts!==false;$('bt-read-receipts').checked=bt.readReceipts;
 const old=JSON.parse(sessionStorage.getItem('netlab.bt.pending')||'null');
 if(old?.address&&/^([A-F0-9]{2}:){5}[A-F0-9]{2}$/.test(old.address)){
  const peer=btPeer(old.address);bt.peers.set(old.address,peer);await vault.put('bt-peers',old.address,peer);
  for(const frame of (old.pending||[]).slice(0,50)){const row={...frame,mine:true,time:frame.localTime||Date.now(),status:'queued'};await vault.batch([vaultPut('bt-history:'+old.address,frame.id,row),vaultPut('bt-outbox:'+old.address,frame.id,frame)]);}
  sessionStorage.removeItem('netlab.bt.pending');if(!saved.get('last'))saved.set('last',{address:old.address});
 }
 bt.storageHealthy=true;btPeerList();if(saved.get('last')?.address)await btSelect(saved.get('last').address);
}catch(error){btError(error);}}
const btReady=btRestore();
async function btDevices(){
 $('bt-refresh').disabled=true;
 try{const response=await fetch('/api/bluetooth/devices',{signal:AbortSignal.timeout(10000)});if(!response.ok)throw Error('Abra localhost no próprio PC Windows para controlar Bluetooth.');const data=await response.json(),selected=$('bt-devices').value;$('bt-devices').replaceChildren();for(const d of data.devices){const p={...btPeer(d.address),name:d.name};bt.peers.set(d.address,p);$('bt-devices').add(new Option(`${d.name}${p.blocked?' (bloqueado)':''} · ${d.address}`,d.address));}if([...$('bt-devices').options].some(o=>o.value===selected))$('bt-devices').value=selected;btStatus(data.error||'Selecione o celular pareado e conecte por QR Code ou manualmente.');}catch(error){btStatus(error.message);}finally{$('bt-refresh').disabled=false;}
}
function btRetry(){if(bt.manual||bt.retry||!bt.address)return;const delay=Math.min(30000,1000*2**Math.min(bt.attempt++,5));btStatus(`Conexão interrompida. Tentando novamente em ${delay/1000} s; a fila está salva.`);bt.retry=setTimeout(()=>{bt.retry=null;btConnect(bt.address);},delay);}
async function btConnect(address){
 await btReady;if(!address){btStatus('Selecione um aparelho pareado.');return;}if(!bt.storageHealthy){btStatus('Resolva o armazenamento protegido antes de conectar; reabra o aplicativo.');return;}
 if(btPeer(address).blocked){btStatus('Aparelho bloqueado. Libere-o antes de conectar.');return;}
 try{if(bt.address!==address)await btSelect(address);if(bt.address!==address)return;await btSave(()=>vault.put('bt-peers',address,btPeer(address)));btPeerList();}catch(error){btError(error);return;}
 clearTimeout(bt.retry);bt.retry=null;bt.manual=false;const old=bt.socket,ws=new WebSocket(`${location.protocol==='https:'?'wss:':'ws:'}//${location.host}/ws/bluetooth`);bt.socket=ws;old?.close();bt.online=false;btStatus('Conectando por Bluetooth…');
 const timeout=setTimeout(()=>{if(bt.socket===ws&&!bt.online)ws.close();},25000);let incoming=Promise.resolve();ws.onopen=()=>ws.send(JSON.stringify({address}));
 ws.onmessage=e=>{incoming=incoming.then(async()=>{
  if(bt.socket!==ws)return;let d;try{d=JSON.parse(e.data);}catch{return;}bt.lastSeen=Date.now();
  if(d.type==='connected'){clearTimeout(timeout);bt.online=true;bt.attempt=0;btStatus('Conectado por Bluetooth. Histórico salvo neste Windows.');btSend({type:'hello',name:'NetLab Windows'});btFlush();clearInterval(bt.timer);bt.timer=setInterval(()=>{if(Date.now()-bt.lastSeen>45000){ws.close();return;}btSend({type:'ping',id:'heartbeat'});btFlush();},15000);}
  else if(d.type==='chat'){
   const split=d.id.lastIndexOf('-'),prefix=d.id.slice(0,split),seq=Number(d.id.slice(split+1));if(!/^[a-f0-9]{32}$/.test(prefix)||!Number.isSafeInteger(seq)||seq<=0)throw Error('Identificador de mensagem inválido.');
   const markers=await vault.list('bt-seen:'+address+':'+prefix),oldSeq=Math.max(Number(stored(`netlab.bt.seen.${address}.${prefix}`,'0')),markers.records[0]?.value.seq||0);
   if(seq>oldSeq){const row={...d,mine:false,time:Date.now(),status:'received'};await btSave(()=>vault.batch([vaultPut('bt-history:'+address,d.id,row),vaultPut('bt-seen:'+address+':'+prefix,'last',{seq})]));if(bt.address===address){bt.records.set(d.id,row);bt.count++;$('bt-history-count').textContent=`${bt.count} mensagens salvas`;if(bt.offset===0)btRow(row,false);bt.unread.add(d.id);}}
   if(bt.socket===ws){btSend({type:'ack',id:d.id,status:bt.seen.get(d.id)==='read'?'read':'received'});btMarkRead();}
  }else if(d.type==='ack'){
   if(!['received','read'].includes(d.status))return;let row=bt.records.get(d.id);if(!row)row=(await vault.get('bt-history:'+address,d.id)).record?.value;if(!row&&bt.pending.has(d.id))row={...bt.pending.get(d.id),mine:true,time:bt.pending.get(d.id).localTime,status:'queued'};
   const ops=[vaultDelete('bt-outbox:'+address,d.id)];if(row){row={...row,status:row.status==='read'?'read':d.status};ops.push(vaultPut('bt-history:'+address,d.id,row));}
   await btSave(()=>vault.batch(ops));if(bt.address===address){bt.pending.delete(d.id);if(row){bt.records.set(d.id,row);const element=bt.rows.get(d.id)?.querySelector('.chat-delivery');if(element)element.textContent=btDelivery(row.status);}}
  }else if(d.type==='ping')btSend({type:'pong',id:d.id});else if(d.type==='error')btStatus(d.message);
 }).catch(error=>{if(bt.socket===ws){bt.manual=true;ws.close();}btError(error);});};
 ws.onclose=()=>{clearTimeout(timeout);if(bt.socket!==ws)return;bt.online=false;clearInterval(bt.timer);btRetry();};ws.onerror=()=>btStatus('Falha ao conectar. Verifique Bluetooth, pareamento e se outra aba está conectada.');
}
async function btStartQr(){
 await btReady;const address=$('bt-devices').value;if(!address){btStatus('Primeiro pareie o celular, atualize a lista e selecione-o.');return;}if(btPeer(address).blocked){btStatus('Libere o aparelho bloqueado antes de gerar um convite.');return;}$('bt-qr-start').disabled=true;
 try{const response=await fetch('/api/bluetooth/qr',{signal:AbortSignal.timeout(10000)}),data=await response.json();if(!response.ok)throw Error(data.error||'QR Code indisponível');await btConnect(address);if(!bt.socket||bt.address!==address)return;btHideQr();btQrBlob=URL.createObjectURL(new Blob([data.svg],{type:'image/svg+xml'}));$('bt-invite-qr').src=btQrBlob;$('bt-qr-target').textContent=`PC: ${data.name} · Celular: ${$('bt-devices').selectedOptions[0].textContent}`;$('bt-qr-panel').hidden=false;}catch(error){btStatus(error.message);}finally{$('bt-qr-start').disabled=false;}
}
function btDelivery(status){return {read:'Exibida no outro aparelho',received:'Recebida',queued:'Na fila salva — aguardando confirmação'}[status]||'Salva';}
function btRow(d,mine){if(bt.rows.has(d.id))return;const row=element('div',undefined,`chat-item${mine?' mine':''}`),date=d.time?new Date(d.time).toLocaleString('pt-BR'):'';row.append(element('div',`${mine?'Você':d.name||'Android'} · ${date}`,'chat-meta'),element('div',d.text,'chat-text'));if(mine)row.append(element('div',btDelivery(d.status),'chat-delivery'));bt.rows.set(d.id,row);const log=$('bt-messages'),close=log.scrollHeight-log.scrollTop-log.clientHeight<100;log.append(row);while(log.children.length>200){bt.rows.delete(bt.rows.keys().next().value);log.firstChild.remove();}if(close||mine)log.scrollTop=log.scrollHeight;}
function btMarkRead(){if(!bt.readReceipts||document.hidden||$('page-bluetooth').hidden)return;for(const id of bt.unread)if(btSend({type:'ack',id,status:'read'})){bt.seen.set(id,'read');bt.unread.delete(id);}}
async function btDraft(){if(!bt.address)return;const address=bt.address,peer={...btPeer(address),draft:$('bt-message').value};bt.peers.set(address,peer);await btSave(()=>vault.put('bt-peers',address,peer));}
$('bt-message').addEventListener('input',()=>btDraft().catch(btError));
$('bt-form').onsubmit=async e=>{
 e.preventDefault();if(btSending)return;await btReady;if(btSending)return;const raw=$('bt-message').value,text=raw.trim(),address=bt.address;if(!address||!text)return;if(btPeer(address).blocked){btStatus('Aparelho bloqueado.');return;}if(bt.pending.size>=50){btStatus('Aguarde o envio das 50 mensagens pendentes.');return;}
 btSending=true;
 const id=[...crypto.getRandomValues(new Uint8Array(16))].map(n=>n.toString(16).padStart(2,'0')).join('')+'-1',frame={type:'chat',id,name:'NetLab Windows',text,localTime:Date.now()},row={...frame,mine:true,time:frame.localTime,status:'queued'};
 try{await btSave(()=>vault.batch([vaultPut('bt-history:'+address,id,row),vaultPut('bt-outbox:'+address,id,frame)]));if(bt.address===address){bt.pending.set(id,frame);bt.records.set(id,row);bt.count++;$('bt-history-count').textContent=`${bt.count} mensagens salvas`;if(bt.offset) {bt.offset=0;await btHistoryPage();}else btRow(row,true);btSend(frame);if($('bt-message').value===raw){$('bt-message').value='';await btDraft();}}}catch(error){btError(error);}finally{btSending=false;}
};
$('bt-refresh').onclick=btDevices;$('bt-connect').onclick=()=>{btHideQr();btConnect($('bt-devices').value);};$('bt-qr-start').onclick=btStartQr;$('bt-disconnect').onclick=btStop;
$('bt-history-select').onchange=()=>{const address=$('bt-history-select').value;if(address)btSelect(address).catch(btError);};
$('bt-older').onclick=()=>{bt.offset+=200;btHistoryPage().catch(btError);};$('bt-newer').onclick=()=>{bt.offset=Math.max(0,bt.offset-200);btHistoryPage().catch(btError);};
$('bt-read-receipts').onchange=()=>{bt.readReceipts=$('bt-read-receipts').checked;btSave(()=>vault.put('bt-settings','privacy',{readReceipts:bt.readReceipts})).catch(btError);};
$('bt-block').onclick=async()=>{const address=bt.address||$('bt-devices').value;if(!address)return;const peer={...btPeer(address),blocked:!btPeer(address).blocked};try{await btSave(()=>vault.put('bt-peers',address,peer));bt.peers.set(address,peer);if(peer.blocked)btStop();$('bt-block').textContent=peer.blocked?'Liberar aparelho':'Bloquear aparelho';btPeerList();btStatus(peer.blocked?'Bloqueado neste NetLab. Para remover o pareamento, use as configurações do Windows.':'Aparelho liberado.');}catch(error){btError(error);}};
$('bt-clear').onclick=async()=>{const address=bt.address;if(!address||!confirm('Apagar conversa, rascunho e fila neste PC? Não pode ser desfeito e não apaga a cópia do destinatário.'))return;btStop();try{await btIO;const peer={...btPeer(address),draft:''};await vault.batch([{action:'clear',namespace:'bt-history:'+address},{action:'clear',namespace:'bt-outbox:'+address},vaultPut('bt-peers',address,peer)]);bt.peers.set(address,peer);bt.pending.clear();$('bt-message').value='';bt.offset=0;await btHistoryPage();}catch(error){btError(error);}};
$('bt-export').onclick=async()=>{const address=bt.address;if(!address||!confirm('Exportar os textos desta conversa em arquivo sem criptografia?'))return;try{await btIO;const rows=await vault.all('bt-history:'+address);downloadText('NetLab-conversa.txt',rows.reverse().map(r=>`${new Date(r.value.time).toLocaleString('pt-BR')} | ${r.value.mine?'Você':r.value.name||'Android'}: ${r.value.text}`).join('\n'));}catch(error){btError(error);}};
document.addEventListener('visibilitychange',btMarkRead);document.querySelector('[data-page="bluetooth"]').addEventListener('click',btMarkRead);
