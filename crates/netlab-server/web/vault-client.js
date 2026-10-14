'use strict';
// No message content in localStorage. Windows encrypts each record with user DPAPI.
const vault={
 async request(body){const r=await fetch('/api/vault',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});if(!r.ok)throw Error('Não foi possível salvar o histórico protegido. Confira o servidor e o espaço livre; seu rascunho foi mantido.');return r.json();},
 async list(namespace,offset=0){const r=await fetch(`/api/vault?namespace=${encodeURIComponent(namespace)}&offset=${offset}`);if(!r.ok)throw Error('Histórico protegido disponível somente no Windows por localhost.');return r.json();},
 get(namespace,key){return this.request({action:'get',namespace,key});},
 put(namespace,key,value){return this.request({action:'put',namespace,key,value});},
 remove(namespace,key){return this.request({action:'delete',namespace,key});},
 clear(namespace){return this.request({action:'clear',namespace});},
 batch(value){return this.request({action:'batch',value});},
 async all(namespace){let records=[];for(let offset=0;;offset+=200){const page=await this.list(namespace,offset);records.push(...page.records);if(records.length>=page.count)break;}return records;}
};
function vaultPut(namespace,key,value){return {action:'put',namespace,key,value};}
function vaultDelete(namespace,key){return {action:'delete',namespace,key};}
function downloadText(name,text){const url=URL.createObjectURL(new Blob([text],{type:'text/plain;charset=utf-8'}));const a=document.createElement('a');a.href=url;a.download=name;document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);}
