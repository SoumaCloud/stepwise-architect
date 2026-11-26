use crate::{Result, text};
use serde_json::{Value, json};

fn number(v: Option<&Value>, label: &str) -> Result<f64> {
    let n = v
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{label} must be numeric."))?;
    if !n.is_finite() || n < 0.0 {
        return Err(format!("{label} must be finite and nonnegative."));
    }
    Ok(n)
}
fn total(s: &Value) -> Result<Value> {
    let basis = text(s, "basis", 20)?;
    if !["forecast", "observed"].contains(&basis) {
        return Err("basis must be forecast or observed.".into());
    }
    let items = s["items"].as_array().ok_or("items must be an array.")?;
    if items.is_empty() || items.len() > 100 {
        return Err("Provide 1 to 100 cost items.".into());
    }
    let mut output = Vec::new();
    let mut sum = 0.0;
    for item in items {
        text(item, "name", 1000)?;
        let n = match text(item, "kind", 20)? {
            "fixed" => number(item.get("monthly"), "monthly")?,
            "metered" => {
                let per = number(item.get("per"), "per")?;
                if per == 0.0 {
                    return Err("Rate denominator per must be greater than zero.".into());
                }
                number(item.get("quantity"), "quantity")? / per * number(item.get("rate"), "rate")?
            }
            _ => return Err("Cost kind must be fixed or metered.".into()),
        };
        sum += n;
        if !sum.is_finite() {
            return Err("Cost total overflowed. Check rates and quantities.".into());
        }
        let mut item = item.clone();
        item["monthlyCost"] = json!(n);
        output.push(item);
    }
    Ok(json!({"basis":basis,"items":output,"totalMonthly":sum}))
}
pub fn compare(input: &Value) -> Result<Value> {
    let currency = text(input, "currency", 3)?;
    if currency.len() != 3 || !currency.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err("Supply a three-letter uppercase currency code.".into());
    }
    let workload = text(input, "workload", 8000)?;
    if input
        .get("qualityAcceptable")
        .is_some_and(|v| !v.is_boolean())
    {
        return Err("qualityAcceptable must be boolean when supplied.".into());
    }
    let current = total(&input["current"])?;
    let proposed = total(&input["proposed"])?;
    let switching = input
        .get("switchingCost")
        .map(|v| number(Some(v), "switchingCost"))
        .transpose()?
        .unwrap_or(0.0);
    let saving =
        current["totalMonthly"].as_f64().unwrap() - proposed["totalMonthly"].as_f64().unwrap();
    let payback = if saving > 0.0 {
        Some(switching / saving)
    } else {
        None
    };
    let net = saving * 12.0 - switching;
    if !net.is_finite() || payback.is_some_and(|p| !p.is_finite()) {
        return Err("Comparison overflowed. Check rates and quantities.".into());
    }
    let observed = current["basis"] == "observed"
        && proposed["basis"] == "observed"
        && input["qualityAcceptable"] == true;
    Ok(
        json!({"currency":currency,"workload":workload,"current":current,"proposed":proposed,"monthlySaving":saving,"switchingCost":switching,
        "paybackMonths":payback,"netFirstYear":net,"qualityAcceptable":input.get("qualityAcceptable"),
        "status":if observed {"Observed comparison with user-reported acceptable quality"}else{"Estimate requiring representative cost and quality measurements"},
        "assumptions":["User-supplied rates and quantities; no live vendor pricing fetched.","Both scenarios must cover the same monthly workload and service quality.","Include model calls, retries, evaluations, seats, hosting, database, storage, egress and maintenance where applicable.","Free/open-source software may still have infrastructure, API and labour costs."]}),
    )
}
