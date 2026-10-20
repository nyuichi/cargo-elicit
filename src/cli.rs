use crate::{
    model::Tool,
    process,
    project::{self, Config, Obligation, PackageConfig, VerificationConfig},
    store::Store,
};
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
#[derive(Parser)]
#[command(
    version,
    about = "Inspect obligations, check proofs, and preserve reproducible verification evidence. Never starts an AI."
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
    },
    /// Create the reserved operational-notes directory and print its path.
    Notes {
        #[arg(long)]
        json: bool,
    },
    /// Observe current Cargo scope, sources and tool inputs; never runs Lean proofs.
    Refresh {
        #[arg(long)]
        json: bool,
    },
    /// Read the last saved snapshot only. Does not inspect current files or start tools.
    Status {
        #[arg(long)]
        json: bool,
        /// Include obligations with accepted fresh audit and semantic review.
        #[arg(long)]
        all: bool,
    },
    /// List saved obligation statuses or the complete saved API inventory.
    List {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        inventory: bool,
    },
    /// Explain an obligation, its evidence, review and next actions from saved data.
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Inspect a dependency-first SCC plan from saved data; never starts tools.
    Plan {
        /// Show contracts, accepted dependencies and full graph details.
        #[arg(long)]
        details: bool,
        /// Obligation IDs or inventory IDs/names; defaults to declared API goals.
        ids: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Check current inputs; reuse authenticated unchanged Lean module products.
    Check {
        ids: Vec<String>,
        #[arg(long, default_value_t = 600)]
        seconds: u64,
        #[arg(long)]
        json: bool,
    },
    /// Independently rebuild and audit current inputs in a fresh isolated workspace.
    Audit {
        ids: Vec<String>,
        #[arg(long, default_value_t = 600)]
        seconds: u64,
        #[arg(long)]
        json: bool,
    },
    /// Record semantic judgment separately from machine proof. No review gates checks.
    Review {
        ids: Vec<String>,
        #[arg(long, value_enum, default_value = "accepted")]
        verdict: Verdict,
        #[arg(long)]
        note: String,
        #[arg(long)]
        json: bool,
    },
}
#[derive(Clone, Copy, ValueEnum)]
enum Verdict {
    Accepted,
    NeedsWork,
}
pub fn entry() -> i32 {
    let mut args = std::env::args().collect::<Vec<_>>();
    if args.get(1).is_some_and(|a| a == "elicit") {
        args.remove(1);
    }
    let cli = Cli::parse_from(args);
    // Even read-only commands install no signal handler and need no runtime/tool setup.
    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            if process::interrupted() { 130 } else { 2 }
        }
    }
}
fn print(snapshot: &project::Snapshot, json: bool, verbose: bool) -> Result<()> {
    print_mode(snapshot, json, None, verbose, None)
}
fn print_mode(
    snapshot: &project::Snapshot,
    json: bool,
    status_all: Option<bool>,
    verbose: bool,
    operation: Option<(&str, &[String])>,
) -> Result<()> {
    let mut view = project::view(snapshot);
    {
        let all = status_all.unwrap_or(false);
        let goals = view["goals"].as_array().unwrap();
        let completed = |g: &&serde_json::Value| {
            g["audit"] == "accepted" && g["review"] == "accepted" && g["check"] != "failed"
        };
        let done = goals.iter().filter(completed).count();
        let shown: Vec<_> = goals
            .iter()
            .filter(|g| all || !completed(g))
            .cloned()
            .collect();
        view["summary"] = serde_json::json!({"total":goals.len(),"completed":done,
            "remaining":goals.len()-done,"hidden_completed":if all {0}else{done},
            "completion_criterion":"accepted fresh audit and accepted semantic review at saved snapshot; no current failed check"});
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
    if let Some((kind, ids)) = operation {
        let results: Vec<_> = project::select(snapshot, ids)?.into_iter().map(|g| {
            let e=project::evidence_status(snapshot,g,kind);
            serde_json::json!({"id":g.id,"accepted":e.is_some_and(|e|e.accepted),
                "reason":e.map(|e|e.reason.as_str()).unwrap_or("no current evidence"),"receipt":e.map(|e|&e.receipt)})
        }).collect();
        let passed = results.iter().filter(|r| r["accepted"] == true).count();
        view["operation"] = serde_json::json!({"kind":kind,"requested":results.len(),"passed":passed,"unresolved":results.len()-passed,"results":results});
    }
    if !json && !verbose {
        println!("{}", crate::ui::status(&view));
        return Ok(());
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&view)?);
        return Ok(());
    }
    println!(
        "Saved snapshot {} ({})",
        &snapshot.input[..12.min(snapshot.input.len())],
        snapshot.observed_at
    );
    println!(
        "Workspace currentness was not checked by this read. Use refresh, check or audit to observe inputs."
    );
    println!(
        "{} obligations; audited at snapshot: {}. This is not a whole-package claim.",
        snapshot.goals.len(),
        view["declared_obligations_audited_at_snapshot"]
    );
    if status_all.is_some() {
        println!(
            "{} remaining; {} completed ({} hidden). Completion = fresh audit + semantic review. Use status --all to include completed items.",
            view["summary"]["remaining"],
            view["summary"]["completed"],
            view["summary"]["hidden_completed"]
        );
        let candidates = view["candidate_sccs"].as_array().unwrap();
        if !candidates.is_empty() {
            println!("Candidate SCCs (known call dependencies only; inspect plan for boundaries):");
            for c in candidates.iter().take(5) {
                println!(
                    "  {} (shared by {} selected roots){}",
                    c["id"].as_str().unwrap(),
                    c["selected_root_count"],
                    if c["call_resolution_incomplete"] == true {
                        "; unresolved calls"
                    } else {
                        ""
                    }
                );
            }
            if candidates.len() > 5 {
                println!("  ... {} more in plan", candidates.len() - 5);
            }
        }
    }
    for goal in view["goals"].as_array().unwrap() {
        println!(
            "{}  audit={}  check={}  review={}",
            goal["id"].as_str().unwrap(),
            goal["audit"].as_str().unwrap(),
            goal["check"].as_str().unwrap(),
            goal["review"].as_str().unwrap()
        );
        let validity = &goal["audit_validity"];
        if let Some(receipt) = validity["receipt"].as_str() {
            println!(
                "  audit {}: {} (saved observation)",
                validity["state"].as_str().unwrap(),
                receipt
            );
        }
        if validity["state"] == "stale" {
            if validity["specification_changed"] == true {
                println!("  audit invalidated: formal specification binding changed");
            }
            for change in validity["input_changes"].as_array().unwrap().iter().take(5) {
                println!(
                    "  audit invalidated: {} {}",
                    change["change"].as_str().unwrap(),
                    change["path"].as_str().unwrap()
                );
            }
            let count = validity["input_changes"].as_array().unwrap().len();
            if count > 5 {
                println!("  ... {} more input changes in status --json", count - 5);
            }
            if validity["change_details_available"] != true {
                println!(
                    "  detailed changes unavailable in this older evidence record; rerun audit for current evidence"
                );
            }
        }
        if let Some(evidence) = goal.get("last_evidence").filter(|e| e["accepted"] == false) {
            println!(
                "  {}: {}",
                evidence["phase"].as_str().unwrap_or("check"),
                evidence["reason"].as_str().unwrap_or("unresolved")
            );
            println!("  receipt: {}", evidence["receipt"].as_str().unwrap_or(""));
        }
    }
    println!("Use plan [API ...] for dependency-first SCCs, shared callees and evidence status.");
    for issue in &snapshot.limitations {
        println!("inventory limitation: {issue}");
    }
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
    } = cli.command
    {
        process::install_interrupt()?;
        let config = Config {
            schema: 2,
            verification: VerificationConfig { tool },
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
                })
                .collect(),
        };
        let snapshot = project::init(&root, config)?;
        print(&snapshot, false, verbose)?;
        return Ok(0);
    }
    let store = Store::discover(&root)?;
    match cli.command {
        Commands::Notes { json } => {
            process::install_interrupt()?;
            let _lock = store.lock("project")?;
            let path = project::notes(&store)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"path":path,"excluded_from_verification_inputs":true,"policy":"operational records only; never read from build/proof code; archive separately"})
                );
            } else {
                println!(
                    "{}\nOperational records only. Never read from build/proof code. Archive separately (.elicit is usually Git-ignored).",
                    path.display()
                );
            }
        }
        Commands::Status { json, all } => {
            print_mode(&project::load(&store)?, json, Some(all), verbose, None)?
        }
        Commands::List { json, inventory } => {
            let snapshot = project::load(&store)?;
            if inventory {
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &serde_json::json!({"schema":2,"observation":"saved_snapshot","workspace_currentness":"not_observed_by_this_read; run refresh","observed_at_unix_ms":snapshot.observed_at,"inventory":snapshot.inventory})
                        )?
                    );
                } else {
                    println!(
                        "Saved inventory at {}; current workspace not inspected.",
                        snapshot.observed_at
                    );
                    for t in snapshot.inventory {
                        println!("{}  {:?}  {}:{}", t.id, t.kind, t.file, t.line);
                    }
                }
            } else {
                print_mode(&snapshot, json, Some(true), verbose, None)?;
            }
        }
        Commands::Show { id, json } => {
            let value = project::show(&project::load(&store)?, &id)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else {
                println!(
                    "{} — saved snapshot at {}",
                    value["goal"]["id"].as_str().unwrap_or("obligation"),
                    value["observed_at_unix_ms"]
                );
                println!("Current workspace was not inspected; run refresh to observe changes.");
                println!("{}", serde_json::to_string_pretty(&value["goal"])?);
                if let Some(name) = value["aeneas_contract"].as_str() {
                    println!("Generated API proposition: {name}");
                }
                if let Some(evidence) = value["evidence"].as_array().and_then(|v| v.last()) {
                    println!(
                        "Latest evidence: {} — {}",
                        evidence["phase"], evidence["reason"]
                    );
                    println!("Receipt: {}", evidence["receipt"]);
                }
                println!("{}", value["next"].as_str().unwrap_or(""));
            }
        }
        Commands::Plan { ids, json, details } => {
            let value = crate::plan::build(&project::load(&store)?, &ids)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else {
                println!(
                    "{}",
                    if details || verbose {
                        crate::plan::render(&value)
                    } else {
                        crate::ui::plan(&value)
                    }
                );
            }
        }
        Commands::Refresh { json } => {
            process::install_interrupt()?;
            let _lock = store.lock("project")?;
            print(&project::refresh(&store)?, json, verbose)?;
        }
        Commands::Check { ids, seconds, json } => {
            process::install_interrupt()?;
            let (s, code) = project::check(&store, &ids, seconds, false)?;
            print_mode(&s, json, None, verbose, Some(("check", &ids)))?;
            return Ok(code);
        }
        Commands::Audit { ids, seconds, json } => {
            process::install_interrupt()?;
            let (s, code) = project::check(&store, &ids, seconds, true)?;
            print_mode(&s, json, None, verbose, Some(("audit", &ids)))?;
            return Ok(code);
        }
        Commands::Review {
            ids,
            verdict,
            note,
            json,
        } => {
            process::install_interrupt()?;
            let verdict = match verdict {
                Verdict::Accepted => "accepted",
                Verdict::NeedsWork => "needs_work",
            };
            print(
                &project::review(&store, &ids, verdict, &note)?,
                json,
                verbose,
            )?;
        }
        Commands::Init { .. } => unreachable!(),
    }
    Ok(0)
}
