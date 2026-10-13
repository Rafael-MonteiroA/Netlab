'use strict';
const bt={socket:null,address:null,online:false,manual:true,retry:null,attempt:0,timer:null,pending:new Map(),rows:new Map(),unread:new Set(),seen:new Map(),lastSeen:0};
function btId(){let id=stored('netlab.bt.installation','');if(!/^[a-f0-9]{32}$/.test(id)){id=[...crypto.getRandomValues(new Uint8Array(16))].map(n=>n.toString(16).padStart(2,'0')).join('');save('netlab.bt.installation',id);}let seq=Number(stored('netlab.bt.sequence','0'))+1;save('netlab.bt.sequence',String(seq));return `${id}-${seq}`;}
function btStatus(text){$('bt-status').textContent=text;}
function btPersist(){try{sessionStorage.setItem('netlab.bt.pending',JSON.stringify({address:bt.address,pending:[...bt.pending.values()]}));}catch{}}
function btSend(value){if(!bt.online||bt.socket?.readyState!==WebSocket.OPEN)return false;bt.socket.send(JSON.stringify(value));return true;}
function btFlush(){for(const v of bt.pending.values())if(!btSend(v))break;}
function btRetry(){if(bt.manual||bt.retry||!bt.address)return;const delay=Math.min(30000,1000*2**Math.min(bt.attempt++,5));btStatus(`Bluetooth desconectado. Nova tentativa em ${delay/1000} s. Mantenha “Receber conexão” aberto no Android.`);bt.retry=setTimeout(()=>{bt.retry=null;btConnect(bt.address);},delay);}
async function btDevices(){
 $('bt-refresh').disabled=true;btStatus('Verificando o adaptador e os aparelhos pareados…');
 try{const response=await fetch('/api/bluetooth/devices',{signal:AbortSignal.timeout(10000)});if(!response.ok)throw Error('Abra http://localhost:8787 no próprio computador Windows para controlar o Bluetooth.');const data=await response.json();$('bt-devices').replaceChildren();for(const d of data.devices)$('bt-devices').add(new Option(`${d.name} · ${d.address}`,d.address));btStatus(data.error||`${data.devices.length} aparelho(s) pareado(s). No Android, abra NetLab Bluetooth e toque em Receber conexão.`);}
 catch(e){btStatus(e.message);}finally{$('bt-refresh').disabled=false;}
}
function btConnect(address){
 if(!address){btStatus('Atualize e selecione um Android pareado.');return;}
 clearTimeout(bt.retry);bt.retry=null;bt.manual=false;
 if(bt.address!==address){bt.pending.clear();bt.rows.clear();bt.unread.clear();bt.seen.clear();$('bt-messages').replaceChildren();try{const saved=JSON.parse(sessionStorage.getItem('netlab.bt.pending'));if(saved?.address===address)for(const v of saved.pending.slice(0,50))bt.pending.set(v.id,v);}catch{}}
 bt.address=address;for(const v of bt.pending.values())btRow(v,true);const old=bt.socket;const ws=new WebSocket(`${location.protocol==='https:'?'wss:':'ws:'}//${location.host}/ws/bluetooth`);bt.socket=ws;old?.close();bt.online=false;btStatus('Conectando ao Android por Bluetooth…');
 const timeout=setTimeout(()=>{if(!bt.online)ws.close();},25000);
 ws.onopen=()=>ws.send(JSON.stringify({address}));
 ws.onmessage=e=>{if(bt.socket!==ws)return;let d;try{d=JSON.parse(e.data);}catch{return;}bt.lastSeen=Date.now();
  if(d.type==='connected'){clearTimeout(timeout);bt.online=true;bt.attempt=0;btStatus('Conectado por Bluetooth. Wi-Fi, internet e dados móveis não são necessários.');btSend({type:'hello',name:'NetLab Windows'});btFlush();clearInterval(bt.timer);bt.timer=setInterval(()=>{if(Date.now()-bt.lastSeen>45000){ws.close();return;}btSend({type:'ping',id:'heartbeat'});btFlush();},15000);}
  else if(d.type==='chat'){
   const split=d.id.lastIndexOf('-'),prefix=d.id.slice(0,split),seq=Number(d.id.slice(split+1));
   if(!/^[a-f0-9]{32}$/.test(prefix)||!Number.isSafeInteger(seq)||seq<=0){btStatus('Mensagem com identificador incompatível.');return;}
   const key=`netlab.bt.seen.${address}.${prefix}`,last=Number(stored(key,'0'));
   if(seq>last){save(key,String(seq));btRow(d,false);bt.unread.add(d.id);}
   btSend({type:'ack',id:d.id,status:bt.seen.get(d.id)==='read'?'read':'received'});btMarkRead();
  }else if(d.type==='ack'){const row=bt.rows.get(d.id);if(row){const status=row.querySelector('.chat-delivery');if(d.status==='read')status.textContent='Exibida no outro aparelho';else if(status.textContent!=='Exibida no outro aparelho')status.textContent='Recebida';}bt.pending.delete(d.id);btPersist();}
  else if(d.type==='ping')btSend({type:'pong',id:d.id});
  else if(d.type==='error')btStatus(d.message);
 };
 ws.onclose=()=>{clearTimeout(timeout);if(bt.socket!==ws)return;bt.online=false;clearInterval(bt.timer);btRetry();};
 ws.onerror=()=>btStatus('Não foi possível abrir o Bluetooth. Verifique o adaptador, o pareamento e se outra aba está usando a conexão.');
}
function btRow(d,mine){if(bt.rows.has(d.id))return;const row=element('div',undefined,`chat-item${mine?' mine':''}`);row.append(element('div',mine?'Você':(d.name||'Android'),'chat-meta'),element('div',d.text,'chat-text'));if(mine)row.append(element('div','Na fila / aguardando confirmação','chat-delivery'));bt.rows.set(d.id,row);$('bt-messages').append(row);while($('bt-messages').children.length>256)$('bt-messages').firstChild.remove();if(bt.rows.size>256)bt.rows.delete(bt.rows.keys().next().value);$('bt-messages').scrollTop=$('bt-messages').scrollHeight;}
function btMarkRead(){if(document.hidden||$('page-bluetooth').hidden)return;for(const id of bt.unread){if(btSend({type:'ack',id,status:'read'})){bt.seen.set(id,'read');bt.unread.delete(id);}}if(bt.seen.size>256)bt.seen.delete(bt.seen.keys().next().value);}
$('bt-refresh').onclick=btDevices;$('bt-connect').onclick=()=>btConnect($('bt-devices').value);
$('bt-disconnect').onclick=()=>{bt.manual=true;clearTimeout(bt.retry);bt.retry=null;clearInterval(bt.timer);bt.socket?.close();bt.socket=null;bt.online=false;btStatus('Bluetooth desconectado. A fila fica nesta aba até reconectar.');};
$('bt-form').onsubmit=e=>{e.preventDefault();const text=$('bt-message').value.trim();if(!bt.address){btStatus('Selecione um aparelho e conecte primeiro.');return;}if(!text)return;if(bt.pending.size>=50){btStatus('Limite de 50 mensagens pendentes.');return;}const v={type:'chat',id:btId(),name:'NetLab Windows',text};bt.pending.set(v.id,v);btPersist();btRow(v,true);btSend(v);$('bt-message').value='';};
document.addEventListener('visibilitychange',btMarkRead);document.querySelector('[data-page="bluetooth"]').addEventListener('click',btMarkRead);
