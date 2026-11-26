#!/usr/bin/env node
import {readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {advise,researchPlan,catalog,stages} from '../src/core.mjs';
import {compareCosts} from '../src/costs.mjs';
import {loadProject,initProject,saveRecord} from '../src/store.mjs';

const help=`Stepwise Architect 0.1.0 alpha
Free local project-aware tool adviser. Node 22+.

  stepwise demo
  stepwise init --project /absolute/project --file profile.json
  stepwise advise --project /absolute/project --stage review --task "Review this PR cheaply"
  stepwise research --project /absolute/project --task "Validate my product idea"
  stepwise compare-costs --file costs.json
  stepwise status --project /absolute/project
  stepwise remember --project /absolute/project --file decision.json
  stepwise recipe --project /absolute/project --file recipe.json
  stepwise catalog
  stepwise mcp

All results are JSON. No separate model API key, telemetry, web requests or background jobs.
Stages: ${stages.join(', ')}
Only init, remember and recipe write project state. Native assistants handle reasoning and live research.
`;
async function main(){
  const [cmd,...rest]=process.argv.slice(2);const args=Object.create(null);
  for(let i=0;i<rest.length;i+=2){if(!rest[i].startsWith('--')||!rest[i+1]||rest[i+1].startsWith('--'))throw new Error('Use --name value arguments.');args[rest[i].slice(2)]=rest[i+1];}
  const allowed={demo:[],init:['project','file'],advise:['project','task','stage'],research:['project','task'],'compare-costs':['file'],status:['project'],remember:['project','file'],recipe:['project','file'],catalog:[],mcp:[],help:[]};
  if(!cmd||cmd==='--help'||cmd==='help'){console.log(help);return;}
  if(!allowed[cmd])throw new Error(`Unknown command ${cmd}. Run stepwise help.`);
  for(const key of Object.keys(args))if(!allowed[cmd].includes(key))throw new Error(`Unsupported option --${key} for ${cmd}.`);
  if(cmd==='mcp'){const{startMcp}=await import('../src/mcp.mjs');await startMcp();return;}
  const project=()=>{if(!args.project)throw new Error('--project is required.');return resolve(args.project);};
  const json=async()=>{if(!args.file)throw new Error('--file is required.');return JSON.parse(await readFile(resolve(args.file),'utf8'));};
  let result;
  switch(cmd){
    case 'demo':result=advise({task:'Review code without adding another subscription',stage:'review',profile:{name:'Example indie SaaS',description:'A small SEO reporting app',existingTools:['existing-assistant','github-actions'],constraints:{noNewSubscription:true,monthlyBudget:100,currency:'USD'}}});break;
    case 'init':result=await initProject(project(),await json());break;
    case 'advise':result=advise({task:args.task,stage:args.stage,profile:args.project?(await loadProject(project())).profile:undefined});break;
    case 'research':result=researchPlan({question:args.task,profile:args.project?(await loadProject(project())).profile:undefined});break;
    case 'compare-costs':result=compareCosts(await json());break;
    case 'status':result=await loadProject(project());break;
    case 'remember':result=await saveRecord(project(),'decision',await json());break;
    case 'recipe':result=await saveRecord(project(),'recipe',await json());break;
    case 'catalog':result=catalog;break;
  }
  console.log(JSON.stringify(result,null,2));
}
main().catch(e=>{console.error(`stepwise: ${e.message}`);process.exitCode=1;});
