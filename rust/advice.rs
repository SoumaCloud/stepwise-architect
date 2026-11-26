use crate::{Result, text};
use chrono::{NaiveDate, Utc};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub const STAGES: &[&str] = &[
    "research",
    "validation",
    "planning",
    "architecture",
    "design",
    "coding",
    "review",
    "agents",
    "database",
    "testing",
    "automation",
    "deployment",
    "marketing",
    "operations",
    "cost",
];

pub fn catalog() -> Value {
    serde_json::from_str(include_str!("../data/catalog.json"))
        .expect("bundled catalog is valid JSON")
}
pub fn workflows() -> Value {
    serde_json::from_str(include_str!("../data/workflows.json"))
        .expect("bundled workflows are valid JSON")
}

fn terms(s: &str) -> BTreeSet<String> {
    s.to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn infer_stage(task: &str) -> &'static str {
    let groups: &[(&str, &[&str])] = &[
        (
            "research",
            &["research", "competitor", "market", "scrape", "crawl"],
        ),
        ("validation", &["validate", "validation", "interview"]),
        ("planning", &["plan", "planning", "roadmap"]),
        ("architecture", &["architecture", "architect", "stack"]),
        ("design", &["design", "ui", "visual", "generic", "website"]),
        ("coding", &["code", "coding", "implement"]),
        ("review", &["review", "coderabbit", "pr"]),
        ("agents", &["agent", "agents", "multi"]),
        ("database", &["database", "postgres", "sql", "connection"]),
        ("testing", &["test", "testing", "eval"]),
        ("automation", &["automation", "automate", "workflow"]),
        ("deployment", &["deploy", "deployment", "hosting"]),
        ("marketing", &["marketing", "launch", "conversion"]),
        ("operations", &["monitor", "operations", "observability"]),
        ("cost", &["cost", "budget", "tokens", "saving"]),
    ];
    let tokens = terms(task);
    let mut best = ("planning", 0);
    for (stage, tags) in groups {
        let n = tags.iter().filter(|tag| tokens.contains(**tag)).count();
        if n > best.1 {
            best = (stage, n);
        }
    }
    best.0
}

pub fn validate_profile(p: &Value) -> Result<Value> {
    if !p.is_object() {
        return Err("Project profile must be an object.".into());
    }
    let name = text(p, "name", 4000)?.trim();
    let description = text(p, "description", 4000)?.trim();
    for key in ["existingTools", "requirements"] {
        if let Some(v) = p.get(key) {
            let values = v
                .as_array()
                .ok_or_else(|| format!("{key} must be an array."))?;
            if values.len() > 100
                || values
                    .iter()
                    .any(|s| s.as_str().is_none_or(|s| s.chars().count() > 4000))
            {
                return Err(format!(
                    "{key} must have at most 100 strings of at most 4000 characters."
                ));
            }
        }
    }
    let constraints = p.get("constraints").cloned().unwrap_or(json!({}));
    if !constraints.is_object() {
        return Err("constraints must be an object.".into());
    }
    for key in ["localOnly", "noNewSubscription", "sensitiveData"] {
        if constraints.get(key).is_some_and(|v| !v.is_boolean()) {
            return Err(format!("{key} must be boolean."));
        }
    }
    if let Some(budget) = constraints.get("monthlyBudget") {
        let n = budget.as_f64().ok_or("monthlyBudget must be numeric.")?;
        if !n.is_finite() || n < 0.0 {
            return Err("monthlyBudget must be finite and nonnegative.".into());
        }
    }
    if let Some(currency) = constraints.get("currency") {
        let code = currency.as_str().ok_or("currency must be a string.")?;
        if code.len() != 3 || !code.bytes().all(|c| c.is_ascii_uppercase()) {
            return Err("currency must be a three-letter uppercase code.".into());
        }
    }
    Ok(
        json!({"name":name,"description":description,"existingTools":p.get("existingTools").cloned().unwrap_or(json!([])),"requirements":p.get("requirements").cloned().unwrap_or(json!([])),"constraints":constraints}),
    )
}

pub fn validate_catalog(c: &Value) -> Result<()> {
    if c["schemaVersion"] != 1 {
        return Err("Unsupported catalog schemaVersion.".into());
    }
    NaiveDate::parse_from_str(text(c, "reviewedOn", 10)?, "%Y-%m-%d")
        .map_err(|_| "Invalid catalog reviewedOn date.")?;
    let tools = c["tools"]
        .as_array()
        .ok_or("Catalog tools must be an array.")?;
    let mut ids = BTreeSet::new();
    for tool in tools {
        for key in ["id", "name", "url", "why", "tradeoff", "setup"] {
            text(tool, key, 20000)?;
        }
        if !ids.insert(text(tool, "id", 200)?) {
            return Err("Duplicate catalog id.".into());
        }
        if !text(tool, "url", 2000)?.starts_with("https://") {
            return Err("Catalog source URL must use HTTPS.".into());
        }
        for key in ["runsLocal", "requiresSubscription", "externalData"] {
            if !tool.get(key).is_some_and(|v| v.is_null() || v.is_boolean()) {
                return Err(format!("{key} must be boolean or null."));
            }
        }
        for key in ["tags", "stages"] {
            let list = tool[key]
                .as_array()
                .ok_or_else(|| format!("{key} must be an array."))?;
            if list.iter().any(|v| v.as_str().is_none()) {
                return Err(format!("{key} must contain strings."));
            }
        }
        if tool["stages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| !STAGES.contains(&v.as_str().unwrap()))
        {
            return Err("Unknown catalog stage.".into());
        }
    }
    Ok(())
}

pub fn advise(
    task: &str,
    stage: Option<&str>,
    profile: Option<&Value>,
    source: &Value,
    as_of: Option<NaiveDate>,
) -> Result<Value> {
    text(&json!({"task":task}), "task", 8000)?;
    validate_catalog(source)?;
    let stage = stage.unwrap_or_else(|| infer_stage(task));
    if !STAGES.contains(&stage) {
        return Err(format!("Unknown stage. Choose {}.", STAGES.join(", ")));
    }
    let p = profile.map(validate_profile).transpose()?;
    let constraints = p.as_ref().map(|p| &p["constraints"]);
    let flag = |key: &str| constraints.is_some_and(|c| c[key] == true);
    let in_use: BTreeSet<_> = p
        .as_ref()
        .and_then(|p| p["existingTools"].as_array())
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_lowercase)
        .collect();
    let tokens = terms(task);
    let mut candidates = Vec::new();
    let mut rejected = Vec::new();
    for tool in source["tools"].as_array().unwrap() {
        if !tool["stages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == stage)
        {
            continue;
        }
        let existing = in_use.contains(&tool["id"].as_str().unwrap().to_lowercase())
            || in_use.contains(&tool["name"].as_str().unwrap().to_lowercase())
            || tool["existingByDefault"] == true;
        let mut reasons = Vec::new();
        if flag("localOnly") && tool["runsLocal"] == false {
            reasons.push("Requires a hosted service; conflicts with localOnly.");
        }
        if flag("noNewSubscription") && tool["requiresSubscription"] == true && !existing {
            reasons.push("Requires a new subscription.");
        }
        if flag("sensitiveData") && tool["externalData"] == true {
            reasons.push("External data processing needs explicit project approval before use.");
        }
        if !reasons.is_empty() {
            rejected.push(json!({"id":tool["id"],"name":tool["name"],"reasons":reasons}));
            continue;
        }
        let matches: Vec<_> = tokens
            .iter()
            .filter(|s| {
                tool["tags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v.as_str() == Some(s.as_str()))
            })
            .cloned()
            .collect();
        let mut checks = Vec::new();
        if flag("localOnly") && tool["runsLocal"].is_null() {
            checks.push("Verify local execution; unspecified in catalog.");
        }
        if flag("noNewSubscription") && tool["requiresSubscription"].is_null() && !existing {
            checks.push("Verify a suitable plan without a new subscription.");
        }
        if flag("sensitiveData") && tool["externalData"].is_null() {
            checks.push("Verify data flow for the chosen deployment and configuration.");
        }
        if p.as_ref()
            .is_some_and(|p| !p["requirements"].as_array().unwrap().is_empty())
        {
            checks.push("Validate every project-specific requirement; this catalog does not certify compliance.");
        }
        let rank = matches.len() * 3 + if existing { 2 } else { 0 };
        let mut result = tool.clone();
        result["inUse"] = json!(existing);
        result["matchedTerms"] = json!(matches);
        result["checks"] = json!(checks);
        result["evidence"] = json!("Curated documentation candidate, not a comparative benchmark.");
        result["pricing"] =
            json!("Unknown. Check the linked official page before a financial decision.");
        candidates.push((rank, result));
    }
    candidates.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1["id"].as_str().cmp(&b.1["id"].as_str()))
    });
    let date =
        NaiveDate::parse_from_str(source["reviewedOn"].as_str().unwrap(), "%Y-%m-%d").unwrap();
    let age = (as_of.unwrap_or_else(|| Utc::now().date_naive()) - date).num_days();
    if age < 0 {
        return Err("asOf must be on or after the catalog review date.".into());
    }
    let budget = constraints
        .and_then(|c| c.get("monthlyBudget"))
        .map(|n| {
            format!(
                "Budget {n} {} per month. Candidate affordability has not been verified.",
                constraints
                    .and_then(|c| c["currency"].as_str())
                    .unwrap_or("currency unspecified")
            )
        })
        .unwrap_or_else(|| "No project budget supplied.".into());
    Ok(
        json!({"version":"0.2.0-rust-alpha","task":task,"stage":stage,"project":p.as_ref().map(|p|p["name"].clone()),"workflow":workflows()[stage],
        "recommendations":candidates.into_iter().take(3).map(|(_,v)|v).collect::<Vec<_>>(),"rejected":rejected,
        "contextNeeded":if p.is_some(){vec!["Actual workload and bill","Quality baseline","Any unrecorded security or residency requirements"]}else{vec!["Project description","Existing tools","Budget and workload","Security and data restrictions"]},
        "catalog":{"reviewedOn":source["reviewedOn"],"ageDays":age,"stale":age>30,"notice":source["notice"]},"budget":budget,
        "nextAction":"Verify shortlisted tools against current official documentation, then test one change on a representative task. Save the decision and observed result."}),
    )
}

pub fn research(question: &str, profile: Option<&Value>) -> Result<Value> {
    text(&json!({"question":question}), "question", 8000)?;
    let p = profile.map(validate_profile).transpose()?;
    let subject = p
        .as_ref()
        .and_then(|p| p["description"].as_str())
        .unwrap_or(question);
    Ok(
        json!({"status":"Research plan only. No live research was performed by this local tool.","question":question,
        "queries":[format!("\"{subject}\" software alternatives pricing"),format!("{subject} site:reddit.com problem"),format!("{subject} site:producthunt.com"),format!("{subject} site:github.com"),format!("{subject} site:x.com")],
        "evidenceFields":["URL","Access date","Product and claimed capability","User context","Recent concrete pain","Price and limits","Evidence type","Uncertainty"],
        "method":["Search official product pages and repositories first.","Use accessible public Reddit, Product Hunt, X and forums, respecting site access rules.","Deduplicate cross-posts and distinguish promotion from customer experience.","Do not infer demand from mentions alone. Interview target builders and test one measurable offer."],
        "deliverable":"A source-linked competitor comparison, repeated pain examples, uncertain claims, and one validation experiment."}),
    )
}
