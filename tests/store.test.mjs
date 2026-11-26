import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,rm,mkdir,symlink,readFile,readdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {loadProject,initProject,saveRecord} from '../src/store.mjs';
async function fixture(t){const root=await mkdtemp(join(tmpdir(),'stepwise-test-'));t.after(()=>rm(root,{recursive:true,force:true}));return root;}
const profile={name:'Example',description:'An indie SaaS',constraints:{sensitiveData:true}};
test('read is side-effect free and init never overwrites',async t=>{const root=await fixture(t);assert.equal((await loadProject(root)).profile,null);assert.deepEqual(await readdir(root),[]);await initProject(root,profile);await assert.rejects(initProject(root,profile),/already initialized/);assert.equal((await loadProject(root)).profile.name,'Example');});
test('decisions and recipes persist separately',async t=>{const root=await fixture(t);await initProject(root,profile);await saveRecord(root,'decision',{title:'Database',body:'Use Postgres. Revisit backup needs before launch.'});await saveRecord(root,'recipe',{title:'UI review',body:'Inspect the actual page at two widths.'});const state=await loadProject(root);assert.equal(state.decisions.length,1);assert.equal(state.recipes.length,1);assert.ok(state.decisions[0].id);});
test('symlinked state directories are refused',async t=>{const root=await fixture(t),outside=await fixture(t);await symlink(outside,join(root,'.stepwise'));await assert.rejects(initProject(root,profile),/unsafe/);assert.deepEqual(await readdir(outside),[]);});
test('symlinked state files and unsafe record types are refused',async t=>{const root=await fixture(t),outside=await fixture(t);await initProject(root,profile);await symlink(join(outside,'target.json'),join(root,'.stepwise','decisions.json'));await assert.rejects(saveRecord(root,'decision',{title:'x',body:'y'}),/unsafe/);await assert.rejects(saveRecord(root,'../escape',{title:'x',body:'y'}),/Record type/);});
test('busy writes fail without dropping existing records',async t=>{const root=await fixture(t);await initProject(root,profile);await saveRecord(root,'decision',{title:'First',body:'Saved'});await mkdir(join(root,'.stepwise','.write-lock'));await assert.rejects(saveRecord(root,'decision',{title:'Second',body:'Busy'}),/busy/);const saved=JSON.parse(await readFile(join(root,'.stepwise','decisions.json')));assert.equal(saved.length,1);});
