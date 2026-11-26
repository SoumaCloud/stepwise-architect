function nonnegative(v,label) { if(typeof v!=='number'||!Number.isFinite(v)||v<0) throw new Error(`${label} must be a finite nonnegative number.`); return v; }
function total(s,label) {
  if(!s||!Array.isArray(s.items)||!s.items.length) throw new Error(`${label}.items must contain at least one cost item.`);
  if(!['forecast','observed'].includes(s.basis)) throw new Error(`${label}.basis must be forecast or observed.`);
  const items=s.items.map((x,i)=>{
    if(typeof x.name!=='string'||!x.name.trim()) throw new Error(`${label} item ${i} needs a name.`);
    if(x.kind==='fixed') return {...x,monthlyCost:nonnegative(x.monthly,`${x.name}.monthly`)};
    if(x.kind!=='metered') throw new Error('Cost kind must be fixed or metered.');
    const quantity=nonnegative(x.quantity,`${x.name}.quantity`),rate=nonnegative(x.rate,`${x.name}.rate`),per=nonnegative(x.per,`${x.name}.per`);
    if(!per) throw new Error('Rate denominator per must be greater than zero.');
    return {...x,monthlyCost:quantity/per*rate};
  });
  const totalMonthly=items.reduce((n,x)=>n+x.monthlyCost,0);
  if(!Number.isFinite(totalMonthly))throw new Error('Cost total overflowed. Check rates and quantities.');
  return {basis:s.basis,items,totalMonthly};
}
export function compareCosts(input) {
  if(!input||typeof input.currency!=='string'||!/^[A-Z]{3}$/.test(input.currency)) throw new Error('Supply a three-letter uppercase currency code. All rates must use that currency.');
  if(typeof input.workload!=='string'||!input.workload.trim()) throw new Error('Describe the identical monthly workload being compared.');
  if(input.qualityAcceptable!==undefined&&typeof input.qualityAcceptable!=='boolean') throw new Error('qualityAcceptable must be boolean when supplied.');
  const current=total(input.current,'current'),proposed=total(input.proposed,'proposed');
  const switchingCost=nonnegative(input.switchingCost??0,'switchingCost');
  const saving=current.totalMonthly-proposed.totalMonthly;
  if(!Number.isFinite(saving*12-switchingCost)|| (saving>0&&!Number.isFinite(switchingCost/saving)))throw new Error('Comparison overflowed. Check rates and quantities.');
  const verified=current.basis==='observed'&&proposed.basis==='observed'&&input.qualityAcceptable===true;
  return {currency:input.currency,workload:input.workload,current,proposed,monthlySaving:saving,switchingCost,
    paybackMonths:saving>0?switchingCost/saving:null,netFirstYear:saving*12-switchingCost,
    status:verified?'Observed comparison with user-reported acceptable quality':'Estimate requiring representative cost and quality measurements',
    qualityAcceptable:input.qualityAcceptable??null,
    assumptions:['User-supplied rates and quantities; no live vendor pricing fetched.','Both scenarios must cover the same monthly workload and service quality.','Include model calls, retries, evaluations, seats, hosting, database, storage, egress and maintenance where applicable.','Free/open-source software may still have infrastructure, API and labour costs.']};
}
