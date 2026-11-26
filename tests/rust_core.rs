use chrono::NaiveDate;
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use stepwise_architect::{advice, costs, store};

fn profile() -> Value {
    json!({"name":"SEO SaaS","description":"Weekly business reports","existingTools":["github-actions"],"constraints":{"noNewSubscription":true}})
}
fn date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 26).unwrap()
}
fn advice_for(p: Option<&Value>, task: &str, stage: &str) -> Value {
    advice::advise(task, Some(stage), p, &advice::catalog(), Some(date())).unwrap()
}
fn scenario() -> Value {
    serde_json::from_str(include_str!("../examples/costs.json")).unwrap()
}

#[test]
fn relevant_review_and_existing_tools() {
    let r = advice_for(Some(&profile()), "code review security", "review");
    assert_eq!(r["recommendations"][0]["id"], "semgrep");
    assert!(
        r["recommendations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["id"] == "existing-assistant")
    );
    assert!(
        r["recommendations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|x| x["pricing"].as_str().unwrap().starts_with("Unknown"))
    );
}
#[test]
fn local_sensitive_profile_excludes_cloud_database() {
    let mut p = profile();
    p["constraints"] = json!({"localOnly":true,"sensitiveData":true});
    let r = advice_for(Some(&p), "postgres database", "database");
    assert!(
        r["rejected"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["id"] == "neon")
    );
    assert!(
        r["recommendations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["id"] == "postgres")
    );
}
#[test]
fn unknown_data_handling_requires_verification() {
    let mut p = profile();
    p["constraints"]["sensitiveData"] = json!(true);
    let r = advice_for(Some(&p), "security review", "review");
    let scanner = r["recommendations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == "semgrep")
        .unwrap();
    assert!(
        scanner["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("data flow"))
    );
}
#[test]
fn stale_and_invalid_dates_are_explicit() {
    let c = advice::catalog();
    let r = advice::advise(
        "review",
        None,
        None,
        &c,
        Some(NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()),
    )
    .unwrap();
    assert_eq!(r["catalog"]["stale"], true);
    assert!(
        advice::advise(
            "review",
            None,
            None,
            &c,
            Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        )
        .is_err()
    );
}
#[test]
fn profile_and_task_validation() {
    let mut p = profile();
    p["constraints"]["localOnly"] = json!("yes");
    assert!(advice::validate_profile(&p).is_err());
    p = profile();
    p["constraints"]["monthlyBudget"] = json!(-1);
    assert!(advice::validate_profile(&p).is_err());
    assert!(advice::advise("", None, None, &advice::catalog(), Some(date())).is_err());
    assert!(advice::advise("x", Some("invalid"), None, &advice::catalog(), Some(date())).is_err());
}
#[test]
fn updated_catalog_is_validated_and_changes_suggestions() {
    let mut c = advice::catalog();
    c["tools"]
        .as_array_mut()
        .unwrap()
        .retain(|v| v["id"] != "semgrep");
    let r = advice::advise(
        "code review security",
        Some("review"),
        None,
        &c,
        Some(date()),
    )
    .unwrap();
    assert!(
        r["recommendations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["id"] != "semgrep")
    );
    c["tools"][0]["externalData"] = json!("unknown");
    assert!(advice::validate_catalog(&c).is_err());
}
#[test]
fn research_is_a_plan_and_stages_are_inferred() {
    assert_eq!(advice::infer_stage("database connection"), "database");
    assert_eq!(advice::infer_stage("something unspecified"), "planning");
    let r = advice::research("Validate demand", Some(&profile())).unwrap();
    assert_eq!(r["queries"].as_array().unwrap().len(), 5);
    assert!(r["status"].as_str().unwrap().contains("No live research"));
}
#[test]
fn cost_totals_payback_and_first_year() {
    let r = costs::compare(&scenario()).unwrap();
    assert_eq!(r["current"]["totalMonthly"].as_f64(), Some(160.0));
    assert_eq!(r["proposed"]["totalMonthly"].as_f64(), Some(115.0));
    assert_eq!(r["monthlySaving"].as_f64(), Some(45.0));
    assert_eq!(r["paybackMonths"].as_f64(), Some(60.0 / 45.0));
    assert_eq!(r["netFirstYear"].as_f64(), Some(480.0));
}
#[test]
fn zero_and_negative_savings_have_no_payback() {
    let mut s = scenario();
    s["proposed"] = s["current"].clone();
    assert!(costs::compare(&s).unwrap()["paybackMonths"].is_null());
    s["proposed"] =
        json!({"basis":"forecast","items":[{"kind":"fixed","name":"Expensive","monthly":200}]});
    let r = costs::compare(&s).unwrap();
    assert_eq!(r["monthlySaving"].as_f64(), Some(-40.0));
    assert!(r["paybackMonths"].is_null());
}
#[test]
fn invalid_costs_do_not_default_to_free() {
    for key in ["rate", "quantity", "per"] {
        let mut s = scenario();
        s["current"]["items"][1]
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(costs::compare(&s).is_err());
    }
    let mut s = scenario();
    s["current"]["items"][1]["per"] = json!(0);
    assert!(costs::compare(&s).is_err());
    s = scenario();
    s["switchingCost"] = json!(-1);
    assert!(costs::compare(&s).is_err());
    s = scenario();
    s["qualityAcceptable"] = json!("yes");
    assert!(costs::compare(&s).is_err());
    s = scenario();
    s["currency"] = json!("usd");
    assert!(costs::compare(&s).is_err());
}
#[test]
fn cost_overflow_is_an_error() {
    let mut s = scenario();
    s["current"]["items"][1]["quantity"] = json!(f64::MAX);
    s["current"]["items"][1]["rate"] = json!(f64::MAX);
    assert!(costs::compare(&s).is_err());
}
#[test]
fn measured_claim_needs_observed_costs_and_quality() {
    let mut s = scenario();
    s["current"]["basis"] = json!("observed");
    s["proposed"]["basis"] = json!("observed");
    assert!(
        costs::compare(&s).unwrap()["status"]
            .as_str()
            .unwrap()
            .starts_with("Estimate")
    );
    s["qualityAcceptable"] = json!(true);
    assert!(
        costs::compare(&s).unwrap()["status"]
            .as_str()
            .unwrap()
            .starts_with("Observed")
    );
}
#[test]
fn reads_do_not_create_state_and_init_never_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    assert!(store::load(dir.path()).unwrap()["profile"].is_null());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    store::init(dir.path(), &profile()).unwrap();
    assert!(store::init(dir.path(), &profile()).is_err());
    assert!(store::load(Path::new(".")).is_err());
}
#[test]
fn profile_changes_affect_advice_and_preserve_saved_decisions() {
    let dir = tempfile::tempdir().unwrap();
    let p = profile();
    store::init(dir.path(), &p).unwrap();
    store::save(
        dir.path(),
        "decision",
        &json!({"title":"Database","body":"Trial Postgres; revisit at launch."}),
    )
    .unwrap();
    let mut changed = p;
    changed["constraints"] = json!({"localOnly":true,"sensitiveData":true});
    store::update_profile(dir.path(), &changed).unwrap();
    let state = store::load(dir.path()).unwrap();
    assert_eq!(state["decisions"].as_array().unwrap().len(), 1);
    let r = advice_for(Some(&state["profile"]), "database backend", "database");
    assert!(
        r["rejected"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["id"] == "neon")
    );
    store::save(
        dir.path(),
        "recipe",
        &json!({"title":"UI","body":"Inspect the actual page."}),
    )
    .unwrap();
    assert_eq!(
        store::load(dir.path()).unwrap()["recipes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn contention_does_not_drop_records() {
    let dir = tempfile::tempdir().unwrap();
    store::init(dir.path(), &profile()).unwrap();
    let record = json!({"title":"First","body":"Preserve this"});
    store::save(dir.path(), "decision", &record).unwrap();
    fs::create_dir(dir.path().join(".stepwise/.write-lock")).unwrap();
    assert!(
        store::save(dir.path(), "decision", &record)
            .unwrap_err()
            .contains("busy")
    );
    assert_eq!(
        store::load(dir.path()).unwrap()["decisions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
#[cfg(unix)]
#[test]
fn symlinked_state_and_files_are_refused() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    symlink(outside.path(), root.path().join(".stepwise")).unwrap();
    assert!(
        store::init(root.path(), &profile())
            .unwrap_err()
            .contains("unsafe")
    );
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    let root = tempfile::tempdir().unwrap();
    store::init(root.path(), &profile()).unwrap();
    symlink(
        outside.path().join("escape.json"),
        root.path().join(".stepwise/decisions.json"),
    )
    .unwrap();
    assert!(
        store::save(root.path(), "decision", &json!({"title":"x","body":"y"}))
            .unwrap_err()
            .contains("unsafe")
    );
    assert!(store::save(root.path(), "../escape", &json!({"title":"x","body":"y"})).is_err());
}
#[test]
fn cli_rejects_unknown_options_and_runs_without_node() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_stepwise"))
        .arg("demo")
        .env("PATH", dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["stage"], "review");
    let output = Command::new(env!("CARGO_BIN_EXE_stepwise"))
        .args(["advise", "--task", "review", "--bogus", "yes"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}
#[test]
fn cli_accepts_stdin_json_and_project_paths_with_spaces() {
    let root = tempfile::tempdir().unwrap();
    let product = root.path().join("my product");
    fs::create_dir(&product).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_stepwise"))
        .args(["init", "--project"])
        .arg(&product)
        .args(["--file", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(profile().to_string().as_bytes())
        .unwrap();
    assert!(child.wait_with_output().unwrap().status.success());
    assert_eq!(
        store::load(&product).unwrap()["profile"]["name"],
        "SEO SaaS"
    );
}
