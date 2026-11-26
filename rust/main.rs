use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use std::path::PathBuf;
use stepwise_architect::{Result, advice, costs, parse_reader, read_json, store};

#[derive(Parser)]
#[command(
    version,
    about = "Local Rust commands for the Stepwise Architect Claude Code skill. No model API key, telemetry or web requests."
)]
struct Cli {
    /// Explicit updated catalog JSON. Otherwise uses the catalog bundled at build time.
    #[arg(long, global = true)]
    catalog: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// See a review recommendation without writing any files.
    Demo,
    /// Initialize a profile without replacing existing context.
    Init {
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        file: PathBuf,
    },
    /// Replace the complete project profile after an authorized context change.
    UpdateProfile {
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        file: PathBuf,
    },
    /// Recommend candidates for the current task and explicit project constraints.
    Advise {
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long)]
        task: String,
        #[arg(long)]
        stage: Option<String>,
        #[arg(long)]
        as_of: Option<String>,
    },
    /// Prepare queries and evidence fields. Does not perform live web research.
    Research {
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long)]
        task: String,
    },
    /// Compare the same workload using user-supplied rates and quality evidence.
    CompareCosts {
        #[arg(long)]
        file: PathBuf,
    },
    /// Read the profile, decisions and recipes without modifying state.
    Status {
        #[arg(long)]
        project: PathBuf,
    },
    /// Save a decision with rationale and evidence.
    Remember {
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        file: PathBuf,
    },
    /// Save a reusable prompt or workflow.
    Recipe {
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        file: PathBuf,
    },
    /// Show the curated candidate catalog and its limitations.
    Catalog,
}
fn input(path: &std::path::Path) -> Result<Value> {
    if path == std::path::Path::new("-") {
        parse_reader(std::io::stdin().lock())
    } else {
        read_json(path)
    }
}
fn profile(project: Option<&std::path::Path>) -> Result<Option<Value>> {
    let value = project.map(store::load).transpose()?;
    Ok(value.and_then(|v| v.get("profile").filter(|p| !p.is_null()).cloned()))
}
fn run(cli: Cli) -> Result<Value> {
    let source = cli
        .catalog
        .as_ref()
        .map(|p| read_json(p))
        .transpose()?
        .unwrap_or_else(advice::catalog);
    advice::validate_catalog(&source)?;
    match cli.command {
        Command::Demo => advice::advise(
            "Review code without adding another subscription",
            Some("review"),
            Some(
                &json!({"name":"Example indie SaaS","description":"A small SEO reporting app","existingTools":["existing-assistant","github-actions"],"constraints":{"noNewSubscription":true,"monthlyBudget":100,"currency":"USD"}}),
            ),
            &source,
            None,
        ),
        Command::Init { project, file } => store::init(&project, &input(&file)?),
        Command::UpdateProfile { project, file } => store::update_profile(&project, &input(&file)?),
        Command::Advise {
            project,
            task,
            stage,
            as_of,
        } => {
            let date = as_of
                .map(|s| {
                    chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                        .map_err(|_| "asOf must be a valid YYYY-MM-DD date.".to_string())
                })
                .transpose()?;
            advice::advise(
                &task,
                stage.as_deref(),
                profile(project.as_deref())?.as_ref(),
                &source,
                date,
            )
        }
        Command::Research { project, task } => {
            advice::research(&task, profile(project.as_deref())?.as_ref())
        }
        Command::CompareCosts { file } => costs::compare(&input(&file)?),
        Command::Status { project } => store::load(&project),
        Command::Remember { project, file } => store::save(&project, "decision", &input(&file)?),
        Command::Recipe { project, file } => store::save(&project, "recipe", &input(&file)?),
        Command::Catalog => Ok(source),
    }
}
fn main() {
    match run(Cli::parse())
        .and_then(|v| serde_json::to_string_pretty(&v).map_err(|e| e.to_string()))
    {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("stepwise: {error}");
            std::process::exit(1);
        }
    }
}
