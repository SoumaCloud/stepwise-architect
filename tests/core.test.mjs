import test from 'node:test';
import assert from 'node:assert/strict';
import {advise,researchPlan,inferStage,validateProfile} from '../src/core.mjs';
import {compareCosts} from '../src/costs.mjs';
import {readFile} from 'node:fs/promises';
const p={name:'SEO SaaS',description:'Weekly reports',existingTools:['github-actions'],constraints:{noNewSubscription:true}};
const scenario=JSON.parse(await readFile(new URL('../examples/costs.json',import.meta.url)));
test('review recommendation fits the current task and preserves existing tools',()=>{
  const r=advise({task:'code review security',stage:'review',profile:p});
  assert.equal(r.stage,'review');assert.equal(r.recommendations[0].id,'semgrep');assert.ok(r.recommendations.some(x=>x.id==='existing-assistant'));assert.ok(r.recommendations.every(x=>x.pricing.startsWith('Unknown')));
});
test('sensitive and local requirements exclude hosted processing',()=>{
  const r=advise({task:'database postgres',stage:'database',profile:{...p,constraints:{localOnly:true,sensitiveData:true}}});
  assert.ok(r.rejected.some(x=>x.id==='neon'));assert.ok(r.recommendations.some(x=>x.id==='postgres'));
});
test('unknown configuration is a verification requirement',()=>{
  const r=advise({task:'database backend',stage:'database',profile:{...p,constraints:{sensitiveData:true,noNewSubscription:true}}});
  const s=r.recommendations.find(x=>x.id==='supabase');assert.ok(s.checks.some(x=>x.includes('data flow')));assert.ok(s.checks.some(x=>x.includes('subscription')));
});
test('stale catalog is surfaced and invalid inputs fail',()=>{
  assert.equal(advise({task:'review',asOf:'2027-01-01'}).catalog.stale,true);
  assert.throws(()=>advise({task:'x',stage:'nonexistent'}));assert.throws(()=>advise({task:''}));assert.throws(()=>validateProfile({...p,constraints:{localOnly:'yes'}}));
});
test('task inference and research remain honest about live evidence',()=>{
  assert.equal(inferStage('database connection'),'database');const r=researchPlan({question:'Is this market crowded?',profile:p});assert.match(r.status,/No live research/);assert.equal(r.queries.length,5);
});
test('forecast costs include metered units switching costs and payback',()=>{
  const r=compareCosts(scenario);assert.equal(r.current.totalMonthly,160);assert.equal(r.proposed.totalMonthly,115);assert.equal(r.monthlySaving,45);assert.equal(r.paybackMonths,60/45);assert.equal(r.netFirstYear,480);assert.match(r.status,/Estimate/);
});
test('zero or negative savings never produces positive payback',()=>{
  const s=structuredClone(scenario);s.proposed=s.current;assert.equal(compareCosts(s).paybackMonths,null);
  s.proposed={basis:'forecast',items:[{kind:'fixed',name:'expensive',monthly:200}]};assert.equal(compareCosts(s).monthlySaving,-40);assert.equal(compareCosts(s).paybackMonths,null);
});
test('missing rates negative inputs and invalid currency fail instead of becoming zero',()=>{
  for(const change of [s=>delete s.current.items[1].rate,s=>s.current.items[1].per=0,s=>s.switchingCost=-1,s=>s.currency='usd',s=>s.qualityAcceptable='yes']){
    const s=structuredClone(scenario);change(s);assert.throws(()=>compareCosts(s));
  }
});
test('observed claim requires both observed workloads and quality evidence',()=>{
  const s=structuredClone(scenario);s.current.basis=s.proposed.basis='observed';assert.match(compareCosts(s).status,/Estimate/);s.qualityAcceptable=true;assert.match(compareCosts(s).status,/user-reported/);
});
test('overflowing costs fail instead of serializing an invalid result',()=>{const s=structuredClone(scenario);s.current.items[1].quantity=Number.MAX_VALUE;s.current.items[1].rate=Number.MAX_VALUE;assert.throws(()=>compareCosts(s),/overflowed/);});
