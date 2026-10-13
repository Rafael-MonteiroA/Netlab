import fs from 'node:fs';
import path from 'node:path';
import {createRequire} from 'node:module';
const require=createRequire(import.meta.url);
export function thirdPartyNotices(){
 const seen=new Set(),blocks=[];
 function visit(name){
  let dir=path.dirname(require.resolve(name));
  while(dir!==path.dirname(dir)){
   const manifest=path.join(dir,'package.json');
   if(fs.existsSync(manifest)&&JSON.parse(fs.readFileSync(manifest,'utf8')).name===name)break;
   dir=path.dirname(dir);
  }
  if(seen.has(dir))return;seen.add(dir);
  const metadata=JSON.parse(fs.readFileSync(path.join(dir,'package.json'),'utf8'));
  const licenses=fs.readdirSync(dir).filter(f=>/^licen[cs]e(?:\..*)?$/i.test(f));
  blocks.push(`${name} ${metadata.version}\nLicense: ${metadata.license||'See package license'}\n${licenses.map(f=>fs.readFileSync(path.join(dir,f),'utf8')).join('\n')}`);
  for(const dependency of Object.keys(metadata.dependencies||{}))visit(dependency);
 }
 const pkg=JSON.parse(fs.readFileSync(new URL('package.json',import.meta.url),'utf8'));
 for(const name of Object.keys(pkg.dependencies))visit(name);
 return 'NETLAB ANDROID — THIRD-PARTY SOFTWARE NOTICES\n\n'+blocks.join('\n\n'+'='.repeat(72)+'\n\n');
}
