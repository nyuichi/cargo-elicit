use crate::{model::*, process, store::Store};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, IsTerminal, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

pub fn instructions(tool: Tool, required: bool, exe: &str) -> String {
    format!(
        r#"You are the single cargo-elicit verification worker. Work in the original source tree.
Verifier: {tool}; verification required: {required}. Use the user's Codex model, reasoning, auth and permissions.
Write meaningful contracts and proof obligations needed for the declared goal; do not author unrelated type invariants or trait laws first. Do not weaken a contract merely to obtain a proof. If a specification is wrong, report the reason with agent issue before revising it.
Never change runtime behavior, move APIs out of the denominator, copy a function to another project for proof, or hide assumptions as trusted infrastructure. Existing uncommitted work must be preserved. Never git commit/reset/clean. Do not start parallel/subagents.
Creusot uses creusot_std contracts, invariants, logic/ghost/lemmas. Trusted is acceptable only for specification mode, never evidence of a proof. Kani uses cfg_attr(kani,kani::requires/ensures), arbitrary input proof_for_contract harnesses calling the original function. Put models and law/preservation harnesses in the original sources; report unsupported invariants/laws instead of pretending coverage.
Aeneas uses Lean 4. Trait implementation targets are unsupported and remain visible as blocked; do not restore, resubmit or repeatedly investigate their contracts. Prove explicit original method contracts instead; any dictionary law can remain a local support theorem. Type invariants have no generated function-contract constant: state their predicates directly, not as nonexistent ElicitContracts names. Keep persistent files under elicit-proofs/aeneas in the selected package. contracts.toml has schema_version=1 and [contracts."TARGET_ID"] entries with requires (optional Lean predicate), ensures (required Lean predicate), theorem (Lean name). agent show ID reports the generated contract name. Specs.lean optionally defines logical models; Proofs.lean imports ElicitContracts and proves that exact generated proposition. Generated definitions are in ElicitGenerated.lean in the wrapper cache; never edit them. Contracts quantify every argument of the actual translated Lean function, including self, slices, generic parameters and trait dictionaries. ensures receives those arguments followed by the translated output; mutable-reference outputs may carry updated state instead of Rust unit. Use agent show for contract names and inspect the freshly generated signature. Reuse the recorded diagnostic cache for original model inspection. Keep scratch compiler/proof objects under .elicit/cache, not among persistent proof sources. Structs, methods and local helper calls are supported when the translated body and its transitive proof dependencies are checked; trait-implementation laws still require explicit method obligations. Do not use sorry, project axioms or native_decide as proof evidence. In specification mode Proofs.lean need not exist. Unsupported mappings must remain issues.
Goal-directed verification: read the selected goal and proof_plan in each task. Do not drain every helper or type first. A Plan turn must inspect/attempt the goal's translated body, state the goal contract, and identify actual missing invariants or lemmas. Keep the plan focused on completing that goal; follow the user's stated requirements. You may submit the goal contract before its proof and need not submit unrelated type invariants. Then install the plan with {exe} agent plan --file PATH --revision N. JSON format: {{"steps":[{{"id":"unique_id","goal":"ROOT_TARGET_ID","kind":"contract|lemma|integrate","target":"TARGET_ID or null","reason":"why this is needed at the caller","needs":["step_id"],"statement":"Lean proposition or null","theorem":"Lean theorem name or null"}}]}}. Preserve existing steps for other goals when updating. Contract steps prepare an original helper/API contract and proof; integrate steps target the root API and return to its body; lemma steps have target=null and name an exact proposition and theorem in Lemmas.lean. Plans can be revised after new proof failures; no plan or declared dependency is proof evidence. Do not invent prerequisites just to populate a plan. A directly proved goal needs no extra steps: install an integration step for it. Work on one original caller at a time; unvisited API roots do not need a planning turn before that caller can progress. An initial Integrate turn is a real attempt to prove the original goal within the supplied worker budget, not merely an invitation to list helpers. Installing a plan is not a useful proof checkpoint by itself: continue the body attempt until the goal is proved, a concrete missing proof obligation is identified, or a substantive reusable lemma is saved. For each prerequisite describe the actual unsolved caller obligation it discharges; do not enumerate every helper spec speculatively. Return to the original body after each necessary checkpoint. Short Plan turns are for repairing a blocked plan; preserve a concise plan before their supplied turn_seconds deadline. Final proofs must preserve the user's arbitrary-input contract.
Aeneas checkpoint files: Lemmas.lean imports ElicitContracts and contains independently compilable checked lemmas. Proofs.lean may import Lemmas. Keep unfinished experiments in .elicit/cache; never put sorry or failing attempts in checkpoint modules. Add lemma statements/theorem names to the plan. Finish a turn after a useful checkpoint so the scheduler can kernel-check it. For long tasks, save completed lemmas and update the plan instead of losing them in one uncompleted API task. API ledger evidence remains separate. Use ElicitAdapter.contract_to_spec to convert a checked existential execution contract to an Aeneas WP specification, and generate a caller-facing theorem with `elicit_spec rule_name from contract_theorem` after the helper theorem, before its callers. The pinned Aeneas uses @[step] for step* registration (not the mvcgen @[spec] attribute); the axiom audit checks these dependencies too.
Use the ledger CLI: {exe} agent show [ID]; {exe} agent submit ID --revision N --note REASON [--depends ID ...] [--unit UNIT ...]; {exe} agent depend ID DEPENDENCY --revision N; {exe} agent issue ID --revision N --reason REASON. Fetch the current revision with agent show after source edits. Submit runs syntax/type processing through the verifier and rejects changing inputs. Do not edit state.toml directly.
You may run verifier experiments, but only cargo-elicit's official wrapper records proof evidence. agent verify is available outside active turns; during an active turn finish and let the scheduler run the wrapper while edits are stopped.
Do not claim verified yourself. Preserve partial results and limitations. For a blocked task leave one concise agent issue. It blocks automatic retries until relevant semantic inputs change or an explicit run --retry; adding more diagnostic text does not unblock it. Submit every target separately with its current revision; a completed turn without accepted work is not success.
"#
    )
}
pub fn autonomous_instructions(exe: &str) -> String {
    format!(
        r#"You are the single verification author for the user's entire declared goal set. Work autonomously in the original source tree, choosing the proof order, supporting lemmas and local module layout. Follow the user's requirements. Do not change runtime behavior, weaken arbitrary-input contracts, remove goals, copy runtime functions into an easier crate, or use subagents. Preserve existing work; never git commit/reset/clean.
The backend is pinned Charon/Aeneas with Lean 4. Persistent sources live in elicit-proofs/aeneas. contracts.toml uses schema_version=1 and [contracts."TARGET_ID"] entries: requires (optional predicate), ensures (required predicate), theorem (Lean name). Specs.lean optionally defines models. Proofs.lean imports ElicitContracts and may import other tracked local modules. Lemmas and completed API proofs may live in separate local .lean modules. Inspect the fresh translated signatures in the diagnostic cache. Contracts quantify all original translated arguments and require successful execution. Generated ElicitGenerated/ElicitAdapter/ElicitContracts modules are never editable. Only propext, Classical.choice and Quot.sound are permitted axioms; no sorry/admit, project axioms or native_decide. Use ElicitAdapter.spec_to_contract or contract_to_spec to bridge standard Aeneas total-WP proofs and generated contracts; elicit_spec can register a checked contract for step* callers.
Do not install a plan or submit every API separately just for bookkeeping. Continue through useful lemmas and caller proofs in the same turn. Save completed work periodically; keep unfinished experiments and compiler objects under .elicit/cache. Before returning, record your explicit semantic review in one operation: {exe} agent stage --all --note "why these contracts match the user's goal" (or list exact target IDs instead of --all). For whole-package work, include reviewed type target IDs explicitly; --all selects declared contracts only. This stages approval identities; it does not claim a proof. You may continue editing proof terms after staging, but changes to contracts/models/runtime require restaging. The stopped-worker collector checks the stable files, accepts matching reviewed specifications and records only actual kernel evidence. agent show [ID] supplies IDs, generated contract names and diagnostics when needed. Unsupported traits remain visible; prove explicit methods without pretending whole-crate coverage.
Return when the requested goal set is complete, a concrete blocker needs attention, or the supplied overall authoring budget expires. Do not finish merely because one lemma was saved. A plan is optional scratch guidance, not required progress. Report limitations honestly; only the collector can record verified evidence. Do not edit state.toml. User interruption and command approvals must be respected.
"#
    )
}
#[derive(Debug)]
pub struct WorkerDeadline;
impl std::fmt::Display for WorkerDeadline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Codex task deadline exceeded")
    }
}
impl std::error::Error for WorkerDeadline {}
pub struct Codex {
    child: Child,
    input: ChildStdin,
    rx: Receiver<Value>,
    seq: u64,
    pub thread: String,
    turn: Option<String>,
    pub usage: (u64, u64, u64),
}
impl Drop for Codex {
    fn drop(&mut self) {
        if let Some(t) = self.turn.clone() {
            let _=self.send(json!({"id":999999,"method":"turn/interrupt","params":{"threadId":self.thread,"turnId":t}}));
        }
        process::kill_group(self.child.id());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Codex {
    pub fn connect(store: &Store, s: &State, required: bool) -> Result<Self> {
        Self::connect_mode(store, s, required, false)
    }
    pub fn connect_autonomous(store: &Store, s: &State, required: bool) -> Result<Self> {
        Self::connect_mode(store, s, required, true)
    }
    fn connect_mode(store: &Store, s: &State, required: bool, autonomous: bool) -> Result<Self> {
        let mut cmd = Command::new("codex");
        cmd.arg("app-server").current_dir(&store.root);
        process::group(&mut cmd);
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("cannot start codex app-server; install and authenticate Codex first")?;
        let input = child.stdin.take().unwrap();
        let out = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(out).lines() {
                let Ok(line) = line else {
                    break;
                };
                if line.len() > 16 * 1024 * 1024 {
                    break;
                }
                if let Ok(v) = serde_json::from_str(&line) {
                    if tx.send(v).is_err() {
                        break;
                    }
                }
            }
        });
        let mut c = Self {
            child,
            input,
            rx,
            seq: 0,
            thread: String::new(),
            turn: None,
            usage: (0, 0, 0),
        };
        c.request("initialize",json!({"clientInfo":{"name":"cargo-elicit","title":"cargo-elicit","version":env!("CARGO_PKG_VERSION")},"capabilities":{}}),Duration::from_secs(30))?;
        c.send(json!({"method":"initialized","params":{}}))?;
        let exe = std::env::current_exe()?.to_string_lossy().into_owned();
        let params = json!({"cwd":store.root,"developerInstructions":if autonomous { autonomous_instructions(&exe) } else { instructions(s.tool,required,&exe) }});
        let result = if let Some(id) = &s.run.thread {
            let mut p = params.clone();
            p["threadId"] = json!(id);
            c.request("thread/resume", p, Duration::from_secs(60))
                .or_else(|_| c.request("thread/start", params.clone(), Duration::from_secs(60)))?
        } else {
            c.request("thread/start", params, Duration::from_secs(60))?
        };
        c.thread = result["thread"]["id"]
            .as_str()
            .context("app-server thread ID missing")?
            .into();
        c.usage = s.run.usage_for(&c.thread);
        Ok(c)
    }
    fn send(&mut self, v: Value) -> Result<()> {
        serde_json::to_writer(&mut self.input, &v)?;
        self.input.write_all(b"\n")?;
        self.input.flush()?;
        Ok(())
    }
    fn next(&mut self, deadline: Instant) -> Result<Value> {
        loop {
            if process::interrupted() || Instant::now() >= deadline {
                if let Some(id) = self.turn.clone() {
                    self.seq += 1;
                    let _=self.send(json!({"id":self.seq,"method":"turn/interrupt","params":{"threadId":self.thread,"turnId":id}}));
                }
                if process::interrupted() {
                    bail!("interrupted");
                }
                return Err(WorkerDeadline.into());
            }
            match self.rx.recv_timeout(Duration::from_millis(100)) {
                Ok(v) => {
                    if v.get("id").is_some() && v.get("method").is_some() {
                        self.server_request(&v, deadline)?;
                        continue;
                    }
                    if v["method"] == "thread/tokenUsage/updated" {
                        let u = &v["params"]["tokenUsage"]["total"];
                        self.usage = (
                            u["inputTokens"].as_u64().unwrap_or(0),
                            u["cachedInputTokens"].as_u64().unwrap_or(0),
                            u["outputTokens"].as_u64().unwrap_or(0),
                        );
                    }
                    return Ok(v);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => bail!("Codex app-server disconnected"),
            }
        }
    }
    fn request(&mut self, method: &str, params: Value, timeout: Duration) -> Result<Value> {
        self.seq += 1;
        let id = self.seq;
        self.send(json!({"id":id,"method":method,"params":params}))?;
        let deadline = Instant::now() + timeout;
        loop {
            let v = self.next(deadline)?;
            if v["id"] == id {
                if let Some(e) = v.get("error") {
                    bail!("app-server {method}: {e}");
                }
                return Ok(v["result"].clone());
            }
        }
    }
    fn server_request(&mut self, v: &Value, deadline: Instant) -> Result<()> {
        let method = v["method"].as_str().unwrap_or("");
        if !matches!(
            method,
            "item/commandExecution/requestApproval" | "item/fileChange/requestApproval"
        ) {
            self.send(json!({"id":v["id"],"error":{"code":-32601,"message":"cargo-elicit does not support this interactive request"}}))?;
            bail!("unsupported app-server request {method}; no permission was granted");
        }
        if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
            self.send(json!({"id":v["id"],"result":{"decision":"decline"}}))?;
            bail!(
                "approval required in unattended execution: {method}; resume in an interactive terminal"
            );
        }
        eprintln!(
            "Codex approval request ({method}):\n{}\nAllow this request? [y/N]",
            serde_json::to_string_pretty(&v["params"])?
        );
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line);
            let _ = tx.send(line);
        });
        let answer = loop {
            if process::interrupted() || Instant::now() >= deadline {
                bail!("approval interrupted or deadline exceeded");
            }
            if let Ok(l) = rx.recv_timeout(Duration::from_millis(100)) {
                break l;
            }
        };
        let decision = if answer.trim().eq_ignore_ascii_case("y") {
            "accept"
        } else {
            "decline"
        };
        self.send(json!({"id":v["id"],"result":{"decision":decision}}))?;
        if decision == "decline" {
            bail!("approval declined");
        }
        Ok(())
    }
    pub fn run(
        &mut self,
        s: &State,
        task: &Task,
        timeout: Duration,
        remaining_seconds: u64,
    ) -> Result<String> {
        let _ = s; // Only task context changes between turns; shared instructions belong to the thread.
        let details:Vec<_>=task.targets.iter().filter_map(|id|s.targets.get(id)).map(|t|json!({"id":t.id,"file":t.file,"line":t.line,"kind":t.kind,"dependencies":t.dependencies,"stale":t.stale,"notes":t.notes,"required_units":t.required_units,"specification_ready":t.ready(),"currently_verified":s.verified(&t.id),"aeneas_contract":(s.tool==Tool::Aeneas && t.kind.proof()).then(||crate::aeneas::contract_name(&t.id)),"verification_results":s.results.iter().filter(|r| r.target == t.id).collect::<Vec<_>>()})).collect();
        let text = if task.kind == TaskKind::Author {
            serde_json::to_string(&json!({"task":task.kind,"goal":s.goal,"targets":details,
                "remaining_seconds":remaining_seconds,"turn_seconds":timeout.as_secs(),
                "completion":"Work autonomously on the entire user goal set. Save partial work without returning after each lemma. Return when the goal is complete, a concrete blocker needs attention, or the overall authoring budget expires. Stage reviewed contracts once with agent stage; the stopped-worker collector performs formal checks and records evidence."}))?
        } else {
            serde_json::to_string(
                &json!({"task":task.kind,"targets":details,"work":task.work,"goal":s.goal,"proof_plan":s.proof_plan,"remaining_seconds":remaining_seconds,"turn_seconds":timeout.as_secs(),"input_revision":s.revision,"input_fingerprint":s.input,"completion":"Plan: inspect the goal body, attempt its proof, and install concrete missing prerequisites. Other work: prepare the assigned contract/lemma, reconnect it to the caller, then finish for kernel checking. API completion always requires its full contract."}),
            )?
        };
        let start = Instant::now();
        let response = self.request(
            "turn/start",
            json!({"threadId":self.thread,"input":[{"type":"text","text":text}]}),
            timeout,
        )?;
        self.turn = response["turn"]["id"].as_str().map(str::to_string);
        if self.turn.is_none() {
            bail!("app-server turn ID missing");
        }
        loop {
            let v = self.next(start + timeout)?;
            if v["method"] == "turn/completed"
                && v["params"]["threadId"] == self.thread
                && v["params"]["turn"]["id"].as_str() == self.turn.as_deref()
            {
                self.turn = None;
                let status = v["params"]["turn"]["status"].as_str().unwrap_or("unknown");
                if status != "completed" {
                    bail!("Codex turn ended with status {status}");
                }
                return Ok(status.into());
            }
        }
    }
}
