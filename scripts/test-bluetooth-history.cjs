const {readFileSync}=require('node:fs');
const vm=require('node:vm'),assert=require('node:assert/strict'),path=require('node:path');
const source=readFileSync(path.join(__dirname,'../crates/netlab-server/web/bluetooth-client.js'),'utf8');
const pause=()=>new Promise(r=>setImmediate(r));
function harness(db=new Map()){
 class Node {constructor(){this.children=[];this.value='';this.hidden=false;this.textContent='';this.scrollHeight=0;this.scrollTop=0;this.clientHeight=100;this.options=[];}append(...nodes){this.children.push(...nodes);}add(node){this.options.push(node);}replaceChildren(){this.children=[];this.options=[];}removeAttribute(){}addEventListener(){}querySelector(c){return this.children.find(n=>'.'+n.className===c);}get firstChild(){return{remove:()=>this.children.shift()};}}
 const nodes=new Map(),$=id=>{if(!nodes.has(id))nodes.set(id,new Node());return nodes.get(id);};
 const ns=name=>{if(!db.has(name))db.set(name,new Map());return db.get(name);},copy=v=>structuredClone(v);
 let fail=false;const vault={async all(n){return [...ns(n)].map(([key,value])=>({key,value:copy(value)})).reverse();},async list(n,offset=0){const records=await this.all(n);return {records:records.slice(offset,offset+200),count:records.length};},async get(n,k){return{record:ns(n).has(k)?{key:k,value:copy(ns(n).get(k))}:null};},async put(n,k,v){if(fail)throw Error('Storage unavailable');ns(n).set(k,copy(v));},async batch(ops){if(fail)throw Error('Storage unavailable');for(const o of ops){if(o.action==='put')await this.put(o.namespace,o.key,o.value);else if(o.action==='delete')ns(o.namespace).delete(o.key);else if(o.action==='clear')ns(o.namespace).clear();}}};
 const sockets=[];class WebSocket{static OPEN=1;constructor(){this.sent=[];this.readyState=1;sockets.push(this);}send(s){this.sent.push(JSON.parse(s));}close(){this.readyState=3;}}
 const context={$,vault,WebSocket,Option:class{constructor(text,value){this.textContent=text;this.value=value;}},element:(tag,text,className)=>Object.assign(new Node(),{textContent:text,className}),vaultPut:(namespace,key,value)=>({action:'put',namespace,key,value}),vaultDelete:(namespace,key)=>({action:'delete',namespace,key}),crypto:require('node:crypto').webcrypto,document:{hidden:false,addEventListener(){},querySelector:()=>$('tab')},location:{protocol:'http:',host:'localhost:8787'},URL:{revokeObjectURL(){}},stored:(k,f)=>f,sessionStorage:{getItem:()=>null},setTimeout:()=>1,clearTimeout(){},setInterval:()=>1,clearInterval(){},confirm:()=>true};
 vm.createContext(context);vm.runInContext(source,context);
 return{$,sockets,db,fail:()=>{fail=true;},run:s=>vm.runInContext(s,context),async settle(){for(let i=0;i<12;i++)await pause();await vm.runInContext('btIO',context);}};
}
(async()=>{
 const address='AA:BB:CC:DD:EE:FF';let h=harness();await h.run('btReady');await h.run(`btSelect('${address}')`);
 h.$('bt-message').value='Saved text';await h.$('bt-form').onsubmit({preventDefault(){}});assert.equal(h.db.get('bt-outbox:'+address).size,1);
 h.$('bt-message').value='Unsent draft';await h.run('btDraft()');
 h=harness(h.db);await h.run('btReady');assert.equal(h.$('bt-message').value,'Unsent draft');assert.equal(h.run('bt.pending.size'),1);assert.equal(h.$('bt-messages').children.length,1);
 await h.run(`btConnect('${address}')`);const ws=h.sockets[0];ws.onmessage({data:JSON.stringify({type:'connected'})});await h.settle();assert(ws.sent.some(f=>f.type==='chat'&&f.text==='Saved text'));
 const queued=ws.sent.find(f=>f.type==='chat');ws.onmessage({data:JSON.stringify({type:'ack',id:queued.id,status:'read'})});await h.settle();assert.equal(h.db.get('bt-outbox:'+address).size,0);
 const frame={type:'chat',id:'a'.repeat(32)+'-1',text:'Received text',name:'Peer'};ws.onmessage({data:JSON.stringify(frame)});await h.settle();assert.equal(h.db.get('bt-history:'+address).size,2);assert(ws.sent.some(f=>f.type==='ack'&&f.id===frame.id));
 await h.$('bt-clear').onclick();assert.equal(h.db.get('bt-history:'+address).size,0);
 await h.run(`btConnect('${address}')`);const ws2=h.sockets.at(-1);ws2.onmessage({data:'{"type":"connected"}'});await h.settle();ws2.onmessage({data:JSON.stringify(frame)});await h.settle();assert.equal(h.db.get('bt-history:'+address).size,0,'deleted message must not reappear');
 h.fail();const next={...frame,id:'a'.repeat(32)+'-2'};ws2.onmessage({data:JSON.stringify(next)});await h.settle();assert(!ws2.sent.some(f=>f.type==='ack'&&f.id===next.id),'failed save must never acknowledge');
 console.log('PASS: queue and draft survive reopening; retry and receipt persist; deletion retains deduplication; failed save never acknowledges');
})().catch(e=>{console.error(e);process.exitCode=1;});
