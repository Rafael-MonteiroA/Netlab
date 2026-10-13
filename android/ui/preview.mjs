import http from 'node:http';
import {readFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
const root=fileURLToPath(new URL('../app/src/main/assets/ui/',import.meta.url));
http.createServer(async(req,res)=>{const path=new URL(req.url,'http://localhost').pathname;const file=path==='/'?'index.html':path.slice(1);if(!['index.html','app.js','style.css'].includes(file)){res.writeHead(404).end();return;}try{res.writeHead(200,{'Content-Type':file.endsWith('.js')?'text/javascript; charset=utf-8':file.endsWith('.css')?'text/css; charset=utf-8':'text/html; charset=utf-8','Cache-Control':'no-store'});res.end(await readFile(root+file));}catch{res.writeHead(404).end('Execute npm run build');}}).listen(8789,'127.0.0.1',()=>console.log('Prévia visual: http://localhost:8789 (sem rádio Bluetooth)'));
