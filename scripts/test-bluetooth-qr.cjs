const {readFileSync}=require('node:fs');
const vm=require('node:vm');
const assert=require('node:assert/strict');
const path=require('node:path');
const source=readFileSync(path.join(__dirname,'../crates/netlab-server/web/bluetooth-client.js'),'utf8');
function harness(ok=true){
 const nodes=new Map(),sockets=[],requests=[];
 const $=id=>{if(!nodes.has(id))nodes.set(id,{value:'',hidden:true,disabled:false,textContent:'',src:'',selectedOptions:[{textContent:'S21 FE'}],replaceChildren(){},add(){},removeAttribute(name){delete this[name];},addEventListener(){}});return nodes.get(id);};
 $('bt-devices').value='AA:BB:CC:DD:EE:FF';
 class WebSocket{static OPEN=1;constructor(url){this.url=url;this.sent=[];sockets.push(this);}send(text){this.sent.push(JSON.parse(text));}close(){}}
 const context={$,WebSocket,Option:class{constructor(text,value){this.textContent=text;this.value=value;}},vault:{all:async()=>[],list:async()=>({records:[],count:0}),put:async()=>{}},document:{addEventListener(){},querySelector:()=>$('tab')},location:{protocol:'http:',host:'localhost:8787'},
  URL:{createObjectURL:()=> 'blob:qr',revokeObjectURL(){}},Blob,AbortSignal,
  fetch:async url=>{requests.push(url);return{ok,json:async()=>ok?{name:'PC de teste',svg:'<svg></svg>'}:{error:'Bluetooth indisponível'}};},
  sessionStorage:{getItem:()=>null,setItem(){}},setTimeout:()=>1,clearTimeout(){},setInterval:()=>1,clearInterval(){}};
 vm.runInNewContext(source,context);
 return{$,sockets,requests};
}
(async()=>{
 let h=harness();await h.$('bt-qr-start').onclick();
 assert.equal(h.requests[0],'/api/bluetooth/qr');assert.equal(h.$('bt-qr-panel').hidden,false);assert.equal(h.$('bt-invite-qr').src,'blob:qr');
 assert.equal(h.sockets.length,1);h.sockets[0].onopen();assert.equal(h.sockets[0].sent[0].address,'AA:BB:CC:DD:EE:FF');
 h.$('bt-disconnect').onclick();assert.equal(h.$('bt-qr-panel').hidden,true);
 h=harness(false);await h.$('bt-qr-start').onclick();assert.equal(h.sockets.length,0);assert.equal(h.$('bt-status').textContent,'Bluetooth indisponível');assert.equal(h.$('bt-qr-start').disabled,false);
 h=harness();h.$('bt-devices').value='';await h.$('bt-qr-start').onclick();assert.equal(h.requests.length,0);assert.equal(h.sockets.length,0);
 console.log('PASS: QR starts connection to selected phone; cancel hides QR; unavailable Bluetooth and empty selection do not connect');
})().catch(error=>{console.error(error);process.exitCode=1;});
