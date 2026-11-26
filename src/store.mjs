import {lstat,mkdir,readFile,writeFile,rename,realpath,rmdir} from 'node:fs/promises';
import {join,isAbsolute} from 'node:path';
import {randomUUID} from 'node:crypto';
import {validateProfile} from './core.mjs';
async function checked(path,{directory=false,optional=false}={}) {
  try { const s=await lstat(path); if(s.isSymbolicLink() || (directory?!s.isDirectory():!s.isFile())) throw new Error(`Refusing unsafe state path: ${path}`); return true; }
  catch(e) { if(e.code==='ENOENT'&&optional)return false; throw e; }
}
async function stateDir(project,create=false) {
  if(typeof project!=='string'||!isAbsolute(project)) throw new Error('An explicit absolute project directory is required.');
  const root=await realpath(project); await checked(root,{directory:true});
  const state=join(root,'.stepwise');
  if(!await checked(state,{directory:true,optional:true})) { if(!create)return null; await mkdir(state,{mode:0o700}); }
  return state;
}
async function readJson(dir,file,fallback) {
  const path=join(dir,file); if(!await checked(path,{optional:true}))return fallback;
  return JSON.parse(await readFile(path,'utf8'));
}
async function writeJson(dir,file,value) {
  const dest=join(dir,file); await checked(dest,{optional:true});
  const tmp=join(dir,`.${randomUUID()}.tmp`);
  await writeFile(tmp,JSON.stringify(value,null,2)+'\n',{mode:0o600,flag:'wx'}); await rename(tmp,dest);
}
export async function loadProject(project) {
  const dir=await stateDir(project); if(!dir)return {profile:null,decisions:[],recipes:[]};
  const profile=await readJson(dir,'profile.json',null);
  return {profile:profile?validateProfile(profile):null,decisions:await readJson(dir,'decisions.json',[]),recipes:await readJson(dir,'recipes.json',[])};
}
export async function initProject(project,profile) {
  const validated=validateProfile(profile),dir=await stateDir(project,true);
  if(await checked(join(dir,'profile.json'),{optional:true}))throw new Error('Project already initialized. Edit .stepwise/profile.json deliberately to update it.');
  await writeJson(dir,'profile.json',validated);return validated;
}
export async function saveRecord(project,type,record) {
  if(!['decision','recipe'].includes(type))throw new Error('Record type must be decision or recipe.');
  if(!record||typeof record.title!=='string'||!record.title.trim()||typeof record.body!=='string'||!record.body.trim()||record.body.length>20000||record.title.length>200)throw new Error('A title (1–200 characters) and body (1–20000 characters) are required.');
  const dir=await stateDir(project); if(!dir)throw new Error('Initialize the project before saving records.');
  const lock=join(dir,'.write-lock');
  try {await mkdir(lock,{mode:0o700});} catch(e){if(e.code==='EEXIST')throw new Error('Project state is busy. Retry after the other write completes. If a process crashed, inspect and remove .stepwise/.write-lock.');throw e;}
  try {
    const file=type==='decision'?'decisions.json':'recipes.json';
    const records=await readJson(dir,file,[]);if(!Array.isArray(records))throw new Error('Invalid record store.');
    const saved={id:randomUUID(),createdAt:new Date().toISOString(),title:record.title,body:record.body};
    records.push(saved);await writeJson(dir,file,records);return saved;
  } finally {await rmdir(lock);}
}
