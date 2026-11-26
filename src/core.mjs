import { readFileSync } from 'node:fs';

export const catalog = JSON.parse(readFileSync(new URL('../data/catalog.json', import.meta.url)));
export const workflows = JSON.parse(readFileSync(new URL('../data/workflows.json', import.meta.url)));
export const stages = Object.keys(workflows);
const terms = s => [...new Set(String(s).toLowerCase().match(/[a-z0-9]+/g) || [])];
const stageTerms = {
  research: ['research','competitor','market','scrape','crawl'], validation: ['validate','validation','interview'],
  planning: ['plan','planning','roadmap'], architecture: ['architecture','architect','stack'],
  design: ['design','ui','visual','generic','website'], coding: ['code','coding','implement'],
  review: ['review','coderabbit','pr'], agents: ['agent','agents','multi'],
  database: ['database','postgres','sql','connection'], testing: ['test','testing','eval'],
  automation: ['automation','automate','workflow'], deployment: ['deploy','deployment','hosting'],
  marketing: ['marketing','launch','conversion'], operations: ['monitor','operations','observability'],
  cost: ['cost','budget','tokens','saving']
};
export function inferStage(task) {
  const tokens = terms(task);
  const best=Object.entries(stageTerms).map(([stage,tags])=>({stage,n:tokens.filter(t=>tags.includes(t)).length})).sort((a,b)=>b.n-a.n)[0];
  return best.n ? best.stage : 'planning';
}
export function validateProfile(p) {
  if (!p || typeof p !== 'object' || Array.isArray(p)) throw new Error('Project profile must be an object.');
  for (const k of ['name','description']) if (typeof p[k] !== 'string' || !p[k].trim() || p[k].length > 4000) throw new Error(`${k} must be a nonempty string of at most 4000 characters.`);
  for (const k of ['existingTools','requirements']) if (p[k] !== undefined && (!Array.isArray(p[k]) || p[k].some(v=>typeof v!=='string') || p[k].length>100)) throw new Error(`${k} must be an array of strings, at most 100 items.`);
  if (p.constraints !== undefined && (!p.constraints || typeof p.constraints!=='object' || Array.isArray(p.constraints))) throw new Error('constraints must be an object.');
  const c = p.constraints || {};
  for (const k of ['localOnly','noNewSubscription','sensitiveData']) if (c[k] !== undefined && typeof c[k] !== 'boolean') throw new Error(`${k} must be boolean.`);
  if (c.monthlyBudget !== undefined && (!Number.isFinite(c.monthlyBudget) || c.monthlyBudget<0)) throw new Error('monthlyBudget must be a nonnegative number.');
  return {name:p.name.trim(),description:p.description.trim(),existingTools:p.existingTools||[],requirements:p.requirements||[],constraints:{...c}};
}
export function advise({task,stage,profile,asOf=new Date().toISOString().slice(0,10)}={}) {
  if (typeof task!=='string' || !task.trim() || task.length>8000) throw new Error('A task of 1 to 8000 characters is required.');
  stage = stage || inferStage(task);
  if (!stages.includes(stage)) throw new Error(`Unknown stage. Choose ${stages.join(', ')}.`);
  const p = profile ? validateProfile(profile) : null;
  const c = p?.constraints||{};
  const tokens = terms(task);
  const existing = (p?.existingTools||[]).map(x=>x.toLowerCase());
  const rejected=[], candidates=[];
  for (const item of catalog.tools) {
    if (!item.stages.includes(stage)) continue;
    const inUse = existing.includes(item.id) || existing.includes(item.name.toLowerCase()) || item.existingByDefault===true;
    const reasons=[];
    if(c.localOnly && item.runsLocal===false) reasons.push('Requires a hosted service; conflicts with localOnly.');
    if(c.noNewSubscription && item.requiresSubscription===true && !inUse) reasons.push('Requires a new subscription.');
    if(c.sensitiveData && item.externalData===true) reasons.push('External data processing needs explicit project approval before use.');
    if(reasons.length) { rejected.push({id:item.id,name:item.name,reasons}); continue; }
    const matchedTerms = tokens.filter(t=>item.tags.includes(t));
    const checks=[];
    if(c.localOnly && item.runsLocal===null) checks.push('Verify local execution; unspecified in catalog.');
    if(c.noNewSubscription && item.requiresSubscription===null && !inUse) checks.push('Verify a suitable plan without a new subscription.');
    if(c.sensitiveData && item.externalData===null) checks.push('Verify data flow for the chosen deployment and configuration.');
    if(p?.requirements.length) checks.push('Validate every project-specific requirement; this catalog does not certify compliance.');
    candidates.push({...item,inUse,matchedTerms,checks,rank:matchedTerms.length*3+(inUse?2:0)});
  }
  candidates.sort((a,b)=>b.rank-a.rank||a.id.localeCompare(b.id));
  const freshnessDays=Math.floor((Date.parse(asOf)-Date.parse(catalog.reviewedOn))/86400000);
  if (!Number.isFinite(freshnessDays) || freshnessDays<0) throw new Error('asOf must be a valid date on or after the catalog review date.');
  return {version:'0.1.0-alpha',task,stage,project:p?.name||null,workflow:workflows[stage],
    recommendations:candidates.slice(0,3).map(({rank,...t})=>({...t,evidence:'Curated documentation candidate, not a comparative benchmark.',pricing:'Unknown. Check the linked official page before a financial decision.'})),
    rejected,contextNeeded:p?['Actual workload and bill','Quality baseline','Any unrecorded security or residency requirements']:['Project description','Existing tools','Budget and workload','Security and data restrictions'],
    catalog:{reviewedOn:catalog.reviewedOn,ageDays:freshnessDays,stale:freshnessDays>30,notice:catalog.notice},
    budget:c.monthlyBudget===undefined?'No project budget supplied.':`Budget ${c.monthlyBudget} ${c.currency||'currency unspecified'} per month. Candidate affordability has not been verified.`,
    nextAction:'Verify shortlisted tools against current official documentation, then test one change on a representative task. Save the decision and observed result.'};
}
export function researchPlan({question,profile}) {
  if(typeof question!=='string'||!question.trim()||question.length>8000) throw new Error('A research question of 1 to 8000 characters is required.');
  if(profile) validateProfile(profile);
  const subject = profile?.description || question;
  return {status:'Research plan only. No live research was performed by this local tool.',question,queries:[
    `"${subject}" software alternatives pricing`, `${subject} site:reddit.com problem`, `${subject} site:producthunt.com`, `${subject} site:github.com`, `${subject} site:x.com`],
    evidenceFields:['URL','Access date','Product and claimed capability','User context','Recent concrete pain','Price and limits','Evidence type','Uncertainty'],
    method:['Search official product pages and repositories first.','Use accessible public Reddit, Product Hunt, X and forums to find examples, respecting site access rules.','Deduplicate cross-posts; distinguish founder promotion from customer experience.','Map direct competitors, partial substitutes and manual workflows. Do not infer demand from mentions alone.','Interview target builders and test one measurable offer.'],
    deliverable:'A source-linked competitor comparison, repeated pain examples, uncertain claims, and one validation experiment.'};
}
