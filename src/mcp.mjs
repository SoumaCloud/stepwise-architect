import {McpServer} from '@modelcontextprotocol/sdk/server/mcp.js';
import {StdioServerTransport} from '@modelcontextprotocol/sdk/server/stdio.js';
import {z} from 'zod';
import {advise,researchPlan,catalog,stages} from './core.mjs';
import {compareCosts} from './costs.mjs';
import {loadProject,initProject,saveRecord} from './store.mjs';

const textResult = value => ({content:[{type:'text',text:JSON.stringify(value,null,2)}]});
const profile=z.object({name:z.string().min(1).max(4000),description:z.string().min(1).max(4000),existingTools:z.array(z.string()).max(100).optional(),requirements:z.array(z.string()).max(100).optional(),constraints:z.object({localOnly:z.boolean().optional(),noNewSubscription:z.boolean().optional(),sensitiveData:z.boolean().optional(),monthlyBudget:z.number().nonnegative().optional(),currency:z.string().optional()}).optional()});
const project=z.string().min(1).describe('Explicit absolute path to the user-approved project directory.');
export async function startMcp() {
  const server=new McpServer({name:'stepwise-architect',version:'0.1.0'});
  function register(name,description,inputSchema,readOnly,fn) {
    server.registerTool(name,{description,inputSchema,annotations:{readOnlyHint:readOnly,destructiveHint:false,idempotentHint:readOnly,openWorldHint:false}},async args=>{
      try {return textResult(await fn(args));} catch(e){return {isError:true,content:[{type:'text',text:e.message}]};}
    });
  }
  register('advise','Recommend contextual tools and a next-step workflow. Curated candidates only; no live price or compliance verification.',{task:z.string().min(1).max(8000),stage:z.enum(stages).optional(),project:project.optional()},true,async({project,...args})=>advise({...args,profile:project?(await loadProject(project)).profile:undefined}));
  register('research_plan','Prepare market research queries and evidence fields. This does not perform a web search.',{question:z.string().min(1).max(8000),project:project.optional()},true,async({question,project})=>researchPlan({question,profile:project?(await loadProject(project)).profile:undefined}));
  const item=z.discriminatedUnion('kind',[z.object({kind:z.literal('fixed'),name:z.string().min(1),monthly:z.number().nonnegative()}),z.object({kind:z.literal('metered'),name:z.string().min(1),quantity:z.number().nonnegative(),rate:z.number().nonnegative(),per:z.number().positive()})]);
  const scenario=z.object({basis:z.enum(['forecast','observed']),items:z.array(item).min(1).max(100)});
  register('compare_costs','Compare identical monthly workloads with user-supplied rates. Include infrastructure, calls, retries and labour. No prices fetched.',{currency:z.string().regex(/^[A-Z]{3}$/),workload:z.string().min(1),current:scenario,proposed:scenario,switchingCost:z.number().nonnegative().optional(),qualityAcceptable:z.boolean().optional()},true,compareCosts);
  register('get_project','Read only the project profile, saved decisions and recipes in .stepwise. Does not scan source code or secret files.',{project},true,({project})=>loadProject(project));
  register('init_project','Write a new project profile to .stepwise/profile.json. Never overwrites an existing profile.',{project,profile},false,({project,profile})=>initProject(project,profile));
  register('save_decision','Save an explicit architecture or tool decision locally. Store rationale and evidence URLs; do not store credentials.',{project,title:z.string().min(1).max(200),body:z.string().min(1).max(20000)},false,({project,...record})=>saveRecord(project,'decision',record));
  register('save_recipe','Save a reusable prompt or workflow locally. Do not include secrets.',{project,title:z.string().min(1).max(200),body:z.string().min(1).max(20000)},false,({project,...record})=>saveRecord(project,'recipe',record));
  register('get_catalog','Return curated candidate tools and documented limitations. Not a live marketplace scan.',{},true,()=>catalog);
  await server.connect(new StdioServerTransport());
}
