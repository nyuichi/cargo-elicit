use crate::{
    model::Tool,
    process,
    project::{self, Config, Obligation, PackageConfig, VerificationConfig},
    store::Store,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(
    version,
    disable_help_subcommand = true,
    about = "Track native verification command results and input freshness. Never starts an AI."
)]
struct Cli {
    /// Expand saved details instead of the bounded summary.
    #[arg(long, global = true)]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Create a project and explicitly discover its original Rust API inventory.
    Init {
        #[arg(long, value_enum, default_value = "aeneas")]
        tool: Tool,
        #[arg(long)]
        package: Option<String>,
        #[arg(long, value_delimiter = ',')]
        features: Vec<String>,
        #[arg(long)]
        all_features: bool,
        #[arg(long)]
        no_default_features: bool,
        #[arg(long)]
        target: Option<String>,
        #[arg(long = "api")]
        apis: Vec<String>,
        /// Native verification argv; place after all other init options.
        #[arg(long, num_args = 1.., allow_hyphen_values = true)]
        command: Vec<String>,
        /// Working directory for the native command, relative to the project.
        #[arg(long)]
        directory: Option<String>,
    },
    /// Observe inputs and show progress, recovery facts or focused details. Never runs proofs.
    Status {
        /// Obligation/API selector; with --plan, one or more dependency roots.
        ids: Vec<String>,
        /// Read the saved snapshot immediately, without tools or input observation.
        #[arg(long)]
        saved: bool,
        #[arg(long)]
        json: bool,
        /// Include completed goals in the summary.
        #[arg(long, conflicts_with_all = ["plan", "inventory"])]
        all: bool,
        /// Show dependency-first SCCs and shared callees.
        #[arg(long, conflicts_with = "inventory")]
        plan: bool,
        /// Show the complete saved API inventory for this observation.
        #[arg(long, conflicts_with = "ids")]
        inventory: bool,
        /// Expand dependency and source details.
        #[arg(long, requires = "plan")]
        details: bool,
    },
    /// Run native verification commands and save input-bound results.
    Verify {
        ids: Vec<String>,
        /// Run the configured command again instead of reusing a saved result.
        #[arg(long)]
        fresh: bool,
        #[arg(long, default_value_t = 600)]
        seconds: u64,
        #[arg(long)]
        json: bool,
    },
}
pub fn entry() -> i32 {
    let mut args = std::env::args().collect::<Vec<_>>();
    if args.get(1).is_some_and(|a| a == "elicit") {
        args.remove(1);
    }
    let cli = Cli::parse_from(args);
    let action = match &cli.command {
        Commands::Init { .. } => "init",
        Commands::Status { .. } => "status",
        Commands::Verify { .. } => "verify",
    };
    let record_failures = !matches!(&cli.command, Commands::Status { saved: true, .. });
    // Even read-only commands install no signal handler and need no runtime/tool setup.
    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            if record_failures {
                if let Ok(root) = std::env::current_dir() {
                    if let Ok(store) = Store::discover(&root) {
                        let _ = store.history(
                            "operation_failed",
                            serde_json::json!({"operation":action,"reason":format!("{e:#}")}),
                        );
                    }
                }
            }
            eprintln!("error: {e:#}");
            if process::interrupted() { 130 } else { 2 }
        }
    }
}
fn print(snapshot: &project::Snapshot, json: bool, verbose: bool) -> Result<()> {
    print_mode(snapshot, json, verbose, None)
}
fn print_mode(
    snapshot: &project::Snapshot,
    json: bool,
    verbose: bool,
    operation: Option<(&str, &[String], usize)>,
) -> Result<()> {
    let mut view = project::view(snapshot);
    {
        let all = json || verbose;
        let goals = view["goals"].as_array().unwrap();
        let completed = |g: &&serde_json::Value| crate::resume::complete(g);
        let done = goals.iter().filter(completed).count();
        let shown: Vec<_> = goals
            .iter()
            .filter(|g| all || !completed(g))
            .cloned()
            .collect();
        view["summary"] = serde_json::json!({"total":goals.len(),"completed":done,
            "remaining":goals.len()-done,"hidden_completed":if all {0}else{done},
            "completion_criterion":"successful native command at saved snapshot; no newer failure"});
        view["goals"] = shown.into();
        view["filter"] = if all { "all" } else { "remaining" }.into();
        let plan = crate::plan::build(snapshot, &[])?;
        view["candidate_sccs"] = plan["components"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["candidate"] == true)
            .map(|r| {
                serde_json::json!({"id":r["id"],"label":({
                    let id = r["id"].as_str().unwrap();
                    match snapshot.inventory.iter().find(|t| t.id == id) {
                        Some(t) if snapshot.inventory.iter().filter(|p|p.name==t.name).count()==1 => t.name.clone(),
                        _ => id.to_owned()
                    }
                }),
                "members":r["members"],"selected_root_count":r["selected_root_count"],
                "call_resolution_incomplete":r["call_resolution_incomplete"]})
            })
            .collect::<Vec<_>>()
            .into();
    }
    if let Some((kind, ids, reused_count)) = operation {
        let evidence_kind = match kind {
            "verify" => "run",
            "verify-fresh" => "rerun",
            other => other,
        };
        let results: Vec<_> = project::select(snapshot, ids)?.into_iter().map(|g| {
            let e=project::current_evidence(snapshot,g);
            serde_json::json!({"id":g.id,"accepted":e.is_some_and(|e|e.accepted),
                "reason":g.limitation.as_deref().or_else(||e.map(|e|e.reason.as_str())).unwrap_or("no current evidence"),"receipt":e.map(|e|&e.receipt),
                "phase":e.map(|e|&e.phase),"diagnostic":e.and_then(|e|e.result.as_ref()).and_then(|r|r.diagnostic.as_ref())})
        }).collect();
        let passed = results.iter().filter(|r| r["accepted"] == true).count();
        view["operation"] = serde_json::json!({"kind":kind,"requested":results.len(),"passed":passed,"unresolved":results.len()-passed,"results":results});
        if matches!(kind, "verify" | "verify-fresh") {
            view["operation"]["kind"] = "verify".into();
            view["operation"]["evidence_kind"] = evidence_kind.into();
            view["operation"]["fresh"] = (kind == "verify-fresh").into();
            view["operation"]["reused_count"] = reused_count.into();
            view["operation"]["reused_evidence"] = (reused_count == results.len()).into();
        }
    }
    if !json && !verbose {
        println!("{}", crate::ui::status(&view));
        return Ok(());
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&view)?);
        return Ok(());
    }
    println!("{}", serde_json::to_string_pretty(&view)?);
    Ok(())
}

fn run(cli: Cli) -> Result<i32> {
    let root = std::env::current_dir()?;
    let verbose = cli.verbose;
    if let Commands::Init {
        tool,
        package,
        features,
        all_features,
        no_default_features,
        target,
        apis,
        command,
        directory,
    } = cli.command
    {
        process::install_interrupt()?;
        let config = Config {
            schema: 3,
            verification: VerificationConfig {
                tool,
                command,
                directory,
                inputs: Vec::new(),
            },
            package: PackageConfig {
                name: package,
                features,
                all_features,
                no_default_features,
                target,
            },
            obligations: apis
                .into_iter()
                .map(|id| Obligation {
                    target: Some(id.clone()),
                    id,
                    statement: None,
                    theorem: None,
                    description: String::new(),
                    command: Vec::new(),
                    directory: None,
                })
                .collect(),
        };
        let snapshot = project::init(&root, config)?;
        print(&snapshot, false, verbose)?;
        return Ok(0);
    }
    let store = Store::discover(&root)?;
    match cli.command {
        Commands::Status {
            ids,
            saved,
            json,
            all,
            plan,
            inventory,
            details,
        } => {
            if !plan && ids.len() > 1 {
                anyhow::bail!(
                    "status accepts one API selector; use --plan for multiple dependency roots"
                );
            }
            let snapshot = if saved {
                project::load(&store)?
            } else {
                process::install_interrupt()?;
                let _lock = store.lock("project")?;
                project::refresh(&store)?
            };
            let observation = if saved {
                "saved_snapshot"
            } else {
                "refreshed_inputs_without_proof_execution"
            };
            let currentness = if saved {
                "unknown; --saved did not inspect verification inputs"
            } else {
                "observed_at_snapshot; not a new proof or a guarantee against later edits"
            };
            let mut value = if plan {
                crate::plan::build(&snapshot, &ids)?
            } else if inventory {
                serde_json::json!({"schema":3,"snapshot":snapshot.input,"observed_at_unix_ms":snapshot.observed_at,"inventory":snapshot.inventory})
            } else if let Some(id) = ids.first() {
                let mut detail = project::show(&snapshot, id)?;
                if let Some(history) = detail["evidence"].as_array_mut() {
                    for e in history {
                        e.as_object_mut().unwrap().remove("input_files");
                    }
                }
                detail
            } else {
                let mut recovery = crate::resume::view(&store, &snapshot, !saved)?;
                let goals = recovery["goals"].as_array_mut().unwrap();
                if !all {
                    goals.retain(|g| !crate::resume::complete(g));
                }
                recovery["filter"] = if all { "all" } else { "remaining" }.into();
                recovery["summary"]["hidden_completed"] = if all {
                    0.into()
                } else {
                    recovery["summary"]["completed"].clone()
                };
                recovery
            };
            if saved && value["progress"].is_object() {
                value["progress"]["source_status"] = "unknown".into();
                value["progress"]["environment_status"] = "unknown".into();
            }
            value["observation"] = observation.into();
            value["workspace_currentness"] = currentness.into();
            if json || (verbose && !plan) {
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else if plan {
                println!("Observation: {observation}; {currentness}");
                println!(
                    "{}",
                    if details || verbose {
                        crate::plan::render(&value)
                    } else {
                        crate::ui::plan(&value)
                    }
                );
            } else if inventory {
                println!("Inventory — {observation}; {currentness}");
                for t in &snapshot.inventory {
                    println!("{}  {:?}  {}:{}", t.id, t.kind, t.file, t.line);
                }
            } else if !ids.is_empty() {
                let mut recovery = crate::resume::view(&store, &snapshot, !saved)?;
                let goal_id = value["goal"]["id"].clone();
                recovery["goals"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|g| g["id"] == goal_id);
                recovery["filter"] = "all".into();
                println!("{}", crate::resume::render(&recovery));
            } else {
                print!("{}", crate::resume::render(&value));
            }
        }
        Commands::Verify {
            ids,
            fresh,
            seconds,
            json,
        } => {
            process::install_interrupt()?;
            let (s, code, reused_count) = project::check(&store, &ids, seconds, fresh)?;
            print_mode(
                &s,
                json,
                verbose,
                Some((
                    if fresh { "verify-fresh" } else { "verify" },
                    &ids,
                    reused_count,
                )),
            )?;
            return Ok(code);
        }
        Commands::Init { .. } => unreachable!(),
    }
    Ok(0)
}
