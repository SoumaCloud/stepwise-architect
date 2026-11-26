import {readFile,access} from 'node:fs/promises';
import {resolve,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
import {catalog,stages} from '../src/core.mjs';
const root=dirname(dirname(fileURLToPath(import.meta.url)));
const files=['plugin.json','mcp.json','.claude-plugin/plugin.json','.claude-plugin/marketplace.json','.codex-plugin/plugin.json','.agents/plugins/marketplace.json','gemini-extension.json','package.json'];
const manifests={};for(const file of files)manifests[file]=JSON.parse(await readFile(resolve(root,file),'utf8'));
for(const file of ['README.md','LICENSE','Cargo.toml','Cargo.lock','rust/main.rs','skills/project-adviser/SKILL.md','skills/project-adviser/scripts/stepwise.sh','skills/project-adviser/references/command-guide.md','GEMINI.md','commands/stepwise.toml','bin/stepwise.mjs','docs/INSTALL.md'])await access(resolve(root,file));
const ids=new Set();for(const tool of catalog.tools){assert.ok(!ids.has(tool.id),'Duplicate tool');ids.add(tool.id);assert.equal(new URL(tool.url).protocol,'https:');assert.ok(tool.stages.every(s=>stages.includes(s)));for(const k of ['runsLocal','requiresSubscription','externalData'])assert.ok([true,false,null].includes(tool[k]),`${tool.id} ${k}`);}
assert.equal(manifests['plugin.json'].name,manifests['gemini-extension.json'].name);
assert.equal(manifests['mcp.json'].mcpServers.stepwise.type,'stdio');
assert.ok(manifests['mcp.json'].mcpServers.stepwise.args[0].startsWith('${PLUGIN_ROOT}/'));
assert.ok(manifests['gemini-extension.json'].mcpServers.stepwise.args[0].startsWith('${extensionPath}/'));
const claude=JSON.parse(await readFile(resolve(root,'.mcp.json'),'utf8'));assert.deepEqual(claude.mcpServers,{},'Claude uses the Rust skill directly without auto-starting Node.');
console.log(`Package checks passed: Rust Claude skill, ${ids.size} tools, ${stages.length} workflows, optional Node MCP layouts. Host runtime checks are separate.`);
