import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const cli=fileURLToPath(new URL('../bin/stepwise.mjs',import.meta.url));
test('CLI runs a dependency-free demo and validates options',()=>{const r=spawnSync(process.execPath,[cli,'demo'],{encoding:'utf8'});assert.equal(r.status,0,r.stderr);assert.equal(JSON.parse(r.stdout).stage,'review');const bad=spawnSync(process.execPath,[cli,'advise','--task','review','--bogus','yes'],{encoding:'utf8'});assert.equal(bad.status,1);assert.match(bad.stderr,/Unsupported option/);});
