export const isNative=typeof window!=='undefined'&&typeof window.NetLabNative?.postMessage==='function';
export const initialState={phase:'idle',status:'Pronto para conectar',peerName:'',peerAddress:'',running:false,connected:false,canSend:false,pending:0,messages:[],devices:[],bluetooth:'unknown',deviceName:'Meu Android',notice:'',sendResult:null};
export function command(type,payload={}){if(!isNative)return false;window.NetLabNative.postMessage(JSON.stringify({type,...payload}));return true;}
export function subscribe(listener){const handler=e=>listener({...initialState,...e.detail});window.addEventListener('netlab:state',handler);command('refresh');return()=>window.removeEventListener('netlab:state',handler);}
export function deliveryLabel(status){return {queued:'Na fila',received:'Recebida',read:'Exibida'}[status]||'Na fila';}
export function clockLabel(value){return value?new Intl.DateTimeFormat('pt-BR',{hour:'2-digit',minute:'2-digit'}).format(new Date(value)):'—';}
