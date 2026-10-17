use cargo_elicit::{
    model::*,
    scheduler::{self, Limits},
    store::Store,
};
use std::collections::{BTreeMap, BTreeSet};
fn target(id: &str, kind: Kind) -> Target {
    Target {
        id: id.into(),
        name: id.into(),
        kind,
        visibility: Visibility::Public,
        file: "src/lib.rs".into(),
        line: 1,
        column: 0,
        aliases: vec![],
        spec_hash: format!("spec-{id}"),
        model_base_hash: String::new(),
        model_inputs: BTreeMap::new(),
        model_bindings: None,
        body_hash: "body".into(),
        original_body: "body".into(),
        source_hash: "source".into(),
        specified: true,
        blocker: None,
        stale: vec![],
        dependencies: BTreeMap::new(),
        calls: BTreeSet::new(),
        unresolved_calls: vec![],
        required_units: if kind.proof() {
            vec!["contract".into()]
        } else {
            vec![]
        },
        notes: vec![],
        removed: false,
        implementation_changed: false,
        closed_primitive: true,
        trusted: false,
    }
}
fn state() -> State {
    let mut s = State::new(Tool::Kani, Scope::default());
    s.input = "current".into();
    for id in ["a", "b", "c"] {
        s.targets.insert(id.into(), target(id, Kind::Function));
    }
    s
}
fn proof(id: &str) -> Verification {
    Verification {
        id: format!("proof-{id}"),
        target: id.into(),
        units: vec!["contract".into()],
        obligations: vec!["postcondition".into()],
        scope: Coverage::FullContract,
        assumptions: vec![],
        restrictions: vec![],
        input: "current".into(),
        tool: Tool::Kani,
        tool_version: "0.68.0".into(),
        trusted_base: vec!["Kani/CBMC".into()],
        reusable: true,
        outcome: Outcome::Success,
        summary: "".into(),
        diagnostic: None,
        elapsed_ms: 0,
    }
}
#[test]
fn full_coverage_only() {
    let mut s = state();
    s.results.push(proof("a"));
    assert!(s.verified("a"));
    for out in [
        Outcome::Skipped,
        Outcome::UnknownScope,
        Outcome::Unresolved,
        Outcome::Timeout,
        Outcome::Interrupted,
        Outcome::EnvironmentError,
        Outcome::InputChanged,
    ] {
        s.results[0].outcome = out;
        assert!(!s.verified("a"));
    }
    s.results[0].outcome = Outcome::Success;
    s.results[0].scope = Coverage::Limited;
    assert!(!s.verified("a"));
    s.results[0].scope = Coverage::FullContract;
    s.results[0].obligations.clear();
    assert!(!s.verified("a"));
}
#[test]
fn assumptions_and_restrictions_are_not_infrastructure() {
    let mut s = state();
    s.results.push(proof("a"));
    s.results[0].assumptions.push("unproved helper".into());
    assert!(!s.verified("a"));
    s.results[0].assumptions.clear();
    s.results[0]
        .restrictions
        .push("u8 instantiation of generic API".into());
    assert!(!s.verified("a"));
    s.results[0].restrictions.clear();
    s.results[0].trusted_base.clear();
    assert!(!s.verified("a"));
}
#[test]
fn every_required_unit_needed() {
    let mut s = state();
    s.results.push(proof("a"));
    s.targets
        .get_mut("a")
        .unwrap()
        .required_units
        .push("invariant_preservation".into());
    assert!(!s.verified("a"));
    let mut r = proof("a");
    r.units = vec!["invariant_preservation".into()];
    s.results.push(r);
    assert!(s.verified("a"));
}
#[test]
fn types_have_no_independent_verified() {
    let mut s = state();
    s.targets.get_mut("a").unwrap().kind = Kind::Struct;
    s.results.push(proof("a"));
    assert!(!s.verified("a"));
}
#[test]
fn direct_dependency_propagation_stops_at_unchanged_spec() {
    let mut s = state();
    let ah = s.targets["a"].spec_hash.clone();
    let bh = s.targets["b"].spec_hash.clone();
    s.targets
        .get_mut("b")
        .unwrap()
        .dependencies
        .insert("a".into(), ah);
    s.targets
        .get_mut("c")
        .unwrap()
        .dependencies
        .insert("b".into(), bh);
    s.results.extend([proof("b"), proof("c")]);
    let mut fresh = s.targets.values().cloned().collect::<Vec<_>>();
    fresh.iter_mut().find(|t| t.id == "a").unwrap().spec_hash = "changed".into();
    s.reconcile(fresh, vec![], "current".into());
    assert!(!s.targets["b"].stale.is_empty());
    assert!(s.targets["c"].stale.is_empty());
    assert!(!s.verified("c"));
    let ah = s.targets["a"].spec_hash.clone();
    s.targets.get_mut("a").unwrap().stale.clear();
    let b = s.targets.get_mut("b").unwrap();
    b.dependencies.insert("a".into(), ah);
    b.stale.clear();
    assert!(s.verified("c"));
}
#[test]
fn undeclared_inputs_invalidate_proof_but_not_regenerate_specs() {
    let mut s = state();
    s.results.push(proof("a"));
    let fresh = s.targets.values().cloned().collect();
    s.reconcile(fresh, vec![], "different-package-input".into());
    assert!(!s.verified("a"));
    assert!(s.targets["a"].ready());
    assert!(scheduler::next(&s, false, &Limits::default()).is_none());
}
#[test]
fn notes_do_not_invalidate_proof() {
    let mut s = state();
    s.results.push(proof("a"));
    s.targets.get_mut("a").unwrap().notes.push("memo".into());
    let fresh = s.targets.values().cloned().collect();
    s.reconcile(fresh, vec![], "current".into());
    assert!(s.verified("a"));
}
#[test]
fn cycles_cannot_discharge_each_other() {
    let mut s = state();
    let ah = s.targets["a"].spec_hash.clone();
    let bh = s.targets["b"].spec_hash.clone();
    s.targets
        .get_mut("a")
        .unwrap()
        .dependencies
        .insert("b".into(), bh);
    s.targets
        .get_mut("b")
        .unwrap()
        .dependencies
        .insert("a".into(), ah);
    s.results.extend([proof("a"), proof("b")]);
    assert!(!s.verified("a"));
    assert!(!s.verified("b"));
}
#[test]
fn worker_body_changes_cannot_complete() {
    let mut s = state();
    s.run.active = Some(Active {
        task: Task {
            work: None,
            kind: TaskKind::Specify,
            targets: vec!["a".into()],
        },
        input: "current".into(),
        started: 0,
    });
    let mut fresh = s.targets.values().cloned().collect::<Vec<_>>();
    fresh.iter_mut().find(|t| t.id == "a").unwrap().body_hash = "edited".into();
    s.reconcile(fresh, vec![], "edited".into());
    assert!(s.targets["a"].implementation_changed);
    s.run.active = None;
    let fresh = s.targets.values().cloned().collect();
    s.reconcile(fresh, vec![], "edited".into());
    assert!(s.targets["a"].implementation_changed);
}
#[test]
fn cannot_shrink_denominator_by_reclassifying() {
    let mut s = state();
    let mut fresh = s.targets.values().cloned().collect::<Vec<_>>();
    fresh[0].visibility = Visibility::Auxiliary;
    s.reconcile(fresh, vec![], "current".into());
    assert_eq!(s.targets["a"].visibility, Visibility::Public);
    assert!(s.targets["a"].implementation_changed);
    assert!(!s.complete(false));
}
#[test]
fn deleted_definitions_retain_history() {
    let mut s = state();
    s.results.push(proof("a"));
    s.reconcile(vec![], vec![], "new".into());
    assert!(s.targets["a"].removed);
    assert_eq!(s.results.len(), 1);
    assert!(!s.verified("a"));
}
#[test]
fn verification_not_starved_by_unspecified_targets() {
    let mut s = state();
    s.targets.get_mut("b").unwrap().specified = false;
    s.targets.get_mut("c").unwrap().specified = false;
    let t = scheduler::next(&s, true, &Limits::default()).unwrap();
    assert_eq!(t.kind, TaskKind::Verify);
    assert_eq!(t.targets, vec!["a"]);
}
#[test]
fn finite_attempts_survive_changing_helpers() {
    let mut s = state();
    let t = Task {
        work: None,
        kind: TaskKind::Verify,
        targets: vec!["a".into()],
    };
    s.run.attempts.insert(
        t.key(),
        Attempt {
            total: 3,
            last_input: "old".into(),
            failed: true,
            ..Attempt::default()
        },
    );
    assert!(!scheduler::eligible(&s, &t, &Limits::default()));
    s.run.attempts.get_mut(&t.key()).unwrap().total = 1;
    s.run.attempts.get_mut(&t.key()).unwrap().last_input = "current".into();
    assert!(!scheduler::eligible(&s, &t, &Limits::default()));
    s.input = "changed".into();
    assert!(scheduler::eligible(&s, &t, &Limits::default()));
}
#[test]
fn atomic_ledger_revision_and_lock_conflicts() {
    let d = tempfile::tempdir().unwrap();
    let store = Store {
        root: d.path().to_path_buf(),
    };
    std::fs::write(
        d.path().join("elicit.toml"),
        toml::to_string(&Config::new(Tool::Kani)).unwrap(),
    )
    .unwrap();
    let mut s = state();
    store.save(&mut s).unwrap();
    let revision = s.revision;
    let lock = store.lock("state").unwrap();
    assert!(store.lock("state").is_err());
    drop(lock);
    store
        .transaction(Some(revision), |s| {
            s.run.last_result = "preserved".into();
            Ok(())
        })
        .unwrap();
    assert!(store.transaction(Some(revision), |_| Ok(())).is_err());
    assert_eq!(store.load().unwrap().run.last_result, "preserved");
    std::fs::create_dir_all(d.path().join(".elicit/cache")).unwrap();
    std::fs::remove_dir_all(d.path().join(".elicit/cache")).unwrap();
    assert_eq!(store.load().unwrap().results.len(), 0);
}
#[test]
fn callee_first_and_scc_order() {
    let mut s = state();
    for t in s.targets.values_mut() {
        t.specified = false;
    }
    s.targets.get_mut("a").unwrap().calls.insert("b".into());
    s.targets.get_mut("b").unwrap().calls.insert("c".into());
    let g = scheduler::groups(&s);
    assert_eq!(g, vec![vec!["c"], vec!["b"], vec!["a"]]);
    s.targets.get_mut("c").unwrap().calls.insert("b".into());
    let g = scheduler::groups(&s);
    assert_eq!(g[0], vec!["b", "c"]);
}

#[test]
fn equal_rank_components_use_id_order() {
    let mut s = state();
    s.targets.remove("c");
    for id in ["y", "z"] {
        s.targets.insert(id.into(), target(id, Kind::Function));
    }
    s.targets.get_mut("a").unwrap().calls.insert("z".into());
    s.targets.get_mut("b").unwrap().calls.insert("y".into());
    assert_eq!(
        scheduler::groups(&s),
        vec![vec!["y"], vec!["z"], vec!["a"], vec!["b"]]
    );
}

#[test]
fn state_transaction_waits_for_concurrent_status_reconciliation() {
    let d = tempfile::tempdir().unwrap();
    let store = Store {
        root: d.path().into(),
    };
    std::fs::write(
        d.path().join("elicit.toml"),
        toml::to_string(&Config::new(Tool::Kani)).unwrap(),
    )
    .unwrap();
    let mut s = state();
    store.save(&mut s).unwrap();
    let held = store.lock("state").unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));
        drop(held);
    });
    store
        .transaction(Some(s.revision), |s| {
            s.run.last_result = "serialized".into();
            Ok(())
        })
        .unwrap();
    release.join().unwrap();
    assert_eq!(store.load().unwrap().run.last_result, "serialized");
}

#[test]
fn unsupported_traits_remain_visible_without_scheduling_retries() {
    let mut s = state();
    s.tool = Tool::Aeneas;
    s.targets.clear();
    let mut t = target("trait", Kind::TraitImpl);
    t.specified = false;
    t.stale.push("unsupported mapping".into());
    s.targets.insert(t.id.clone(), t);
    assert!(scheduler::next(&s, true, &Limits::default()).is_none());
    s.input = "unrelated proof update".into();
    assert!(scheduler::next(&s, true, &Limits::default()).is_none());
    assert_eq!(s.targets.len(), 1);
    assert!(!s.complete(true));
}

#[test]
fn issue_blocks_until_semantic_context_changes() {
    let mut s = state();
    s.context_input = "rust-and-tools".into();
    let input = s.target_input("a");
    s.targets.get_mut("a").unwrap().blocker = Some(Blocker {
        reason: "counterexample".into(),
        input,
    });
    let fresh = s.targets.values().cloned().collect();
    s.reconcile(fresh, vec![], "unrelated-proof-edit".into());
    assert!(!s.schedulable("a"));
    s.context_input = "changed-rust-or-toolchain".into();
    let fresh = s.targets.values().cloned().collect();
    s.reconcile(fresh, vec![], "new".into());
    assert!(s.schedulable("a"));
}

#[test]
fn recheck_failure_is_not_reopened_by_other_proof_edits() {
    let mut s = state();
    s.context_input = "stable-context".into();
    let task = Task {
        work: None,
        kind: TaskKind::Recheck,
        targets: vec!["a".into()],
    };
    s.run.attempts.insert(
        task.key(),
        Attempt {
            total: 1,
            failed: true,
            last_input: scheduler::task_input(&s, &task),
            ..Attempt::default()
        },
    );
    s.input = "changed-unrelated-proof-module".into();
    assert!(!scheduler::eligible(&s, &task, &Limits::default()));
    s.targets.get_mut("a").unwrap().spec_hash = "actually-changed-contract".into();
    assert!(scheduler::eligible(&s, &task, &Limits::default()));
}

#[test]
fn additional_diagnostics_and_regressions_are_not_forward_progress() {
    let before = state();
    let mut after = before.clone();
    after
        .targets
        .get_mut("a")
        .unwrap()
        .stale
        .push("new diagnostic".into());
    assert!(!scheduler::advanced(&before, &after));
    let mut fixed = after.clone();
    fixed.targets.get_mut("a").unwrap().stale.clear();
    assert!(scheduler::advanced(&after, &fixed));
}

#[test]
fn type_model_review_survives_extension_but_not_changed_definitions() {
    let mut s = State::new(Tool::Aeneas, Scope::default());
    let mut original = target("Type", Kind::Struct);
    original.model_base_hash = "type declaration".into();
    original
        .model_inputs
        .insert("Model.invariant".into(), "first meaning".into());
    original.bind_models();
    s.targets.insert(original.id.clone(), original.clone());
    let mut extended = original.clone();
    extended
        .model_inputs
        .insert("Model.unrelated".into(), "new model".into());
    extended.bind_models();
    s.reconcile(vec![extended.clone()], vec![], "new proof input".into());
    assert!(s.targets["Type"].ready());
    assert_eq!(s.targets["Type"].spec_hash, original.spec_hash);
    assert_eq!(s.targets["Type"].model_bindings.as_ref().unwrap().len(), 1);
    extended
        .model_inputs
        .insert("Model.invariant".into(), "changed meaning".into());
    extended.bind_models();
    s.reconcile(vec![extended], vec![], "changed semantic input".into());
    assert!(!s.targets["Type"].ready());
    assert!(
        s.targets["Type"]
            .stale
            .contains(&"specification changed".into())
    );
}

#[test]
fn fallback_model_stamps_continue_tracking_changes_after_a_bound_name_disappears() {
    let mut t = target("Type", Kind::Struct);
    t.model_base_hash = "type".into();
    t.model_inputs
        .insert("decl:Model.invariant".into(), "meaning".into());
    t.bind_models();
    t.model_inputs = BTreeMap::from([("@source".into(), "first unsupported model source".into())]);
    let first = t.model_hash();
    t.model_inputs
        .insert("@source".into(), "changed unsupported model source".into());
    assert_ne!(first, t.model_hash());
}

fn lemma_step(goal: &str) -> cargo_elicit::plan::ProofStep {
    cargo_elicit::plan::ProofStep {
        id: "loop_bound".into(),
        goal: goal.into(),
        kind: cargo_elicit::plan::StepKind::Lemma,
        target: None,
        reason: "Needed to establish the caller's overflow precondition".into(),
        needs: vec![],
        statement: Some("True".into()),
        theorem: Some("Lemmas.bound".into()),
    }
}
#[test]
fn selected_goals_do_not_require_unrelated_apis_but_never_hide_package_inventory() {
    use cargo_elicit::plan::*;
    let mut s = state();
    s.goal = VerificationGoal::SelectedApis(vec!["a".into()]);
    s.processing_input = s.input.clone();
    s.results.push(proof("a"));
    assert!(s.complete(true));
    assert_eq!(s.targets.len(), 3);
    s.targets.get_mut("b").unwrap().removed = true;
    assert!(!s.complete(true), "selection cannot hide removed APIs");
    s.targets.get_mut("b").unwrap().removed = false;
    s.goal = VerificationGoal::WholePackage;
    assert!(!s.complete(true));
}
#[test]
fn plans_reject_cycles_unknown_dependencies_and_foreign_goals() {
    use cargo_elicit::plan::*;
    let mut s = state();
    s.tool = Tool::Aeneas;
    s.goal = VerificationGoal::SelectedApis(vec!["a".into()]);
    let mut p = lemma_step("a");
    p.needs = vec![p.id.clone()];
    assert!(install(&mut s, vec![p]).is_err());
    let mut p = lemma_step("a");
    p.needs = vec!["missing".into()];
    assert!(install(&mut s, vec![p]).is_err());
    assert!(install(&mut s, vec![lemma_step("b")]).is_err());
}
#[test]
fn checked_lemma_is_progress_but_never_api_completion_and_edits_invalidate_it() {
    use cargo_elicit::plan::*;
    let mut s = state();
    s.tool = Tool::Aeneas;
    s.goal = VerificationGoal::SelectedApis(vec!["a".into()]);
    let p = lemma_step("a");
    install(&mut s, vec![p.clone()]).unwrap();
    let before = s.clone();
    assert!(!checked(&s, &p));
    s.proof_plan.evidence.insert(
        p.id.clone(),
        LemmaEvidence {
            input: s.input.clone(),
            step_hash: step_hash(&p),
            outcome: Outcome::Success,
            cache: "cache".into(),
        },
    );
    assert!(checked(&s, &p));
    assert!(scheduler::advanced(&before, &s));
    assert!(!s.verified("a"));
    s.input = "edited".into();
    assert!(!checked(&s, &p));
}
#[test]
fn planner_inspects_goals_before_helpers_and_returns_to_caller() {
    use cargo_elicit::plan::*;
    let mut s = state();
    s.tool = Tool::Aeneas;
    s.goal = VerificationGoal::SelectedApis(vec!["c".into()]);
    s.targets.get_mut("c").unwrap().calls.insert("a".into());
    let t = scheduler::next(&s, true, &Limits::default()).unwrap();
    assert_eq!(t.kind, TaskKind::Plan);
    assert_eq!(t.targets, vec!["c"]);
    let p = ProofStep {
        id: "helper".into(),
        goal: "c".into(),
        kind: StepKind::Contract,
        target: Some("a".into()),
        reason: "Required by caller c".into(),
        needs: vec![],
        statement: None,
        theorem: None,
    };
    install(&mut s, vec![p]).unwrap();
    assert_eq!(
        scheduler::next(&s, true, &Limits::default())
            .unwrap()
            .targets,
        vec!["a"]
    );
    s.results.push(proof("a"));
    let t = scheduler::next(&s, true, &Limits::default()).unwrap();
    assert_eq!(t.kind, TaskKind::Integrate);
    assert_eq!(t.targets, vec!["c"]);
}
#[test]
fn plan_edits_cannot_reuse_a_different_lemma_and_selectors_reject_ambiguity() {
    use cargo_elicit::plan::*;
    let mut s = state();
    s.tool = Tool::Aeneas;
    let p = lemma_step("a");
    install(&mut s, vec![p.clone()]).unwrap();
    s.proof_plan.evidence.insert(
        p.id.clone(),
        LemmaEvidence {
            input: s.input.clone(),
            step_hash: step_hash(&p),
            outcome: Outcome::Success,
            cache: "cache".into(),
        },
    );
    let mut changed = p;
    changed.statement = Some("False".into());
    install(&mut s, vec![changed]).unwrap();
    assert!(s.proof_plan.evidence.is_empty());
    s.targets.get_mut("b").unwrap().name = "a".into();
    assert!(resolve(&s, &["a".into()]).is_err());
}

#[test]
fn plan_retry_keys_ignore_checker_cache_paths_and_explanatory_text() {
    use cargo_elicit::plan::*;
    let mut s = state();
    s.tool = Tool::Aeneas;
    let p = lemma_step("a");
    install(&mut s, vec![p.clone()]).unwrap();
    s.proof_plan.evidence.insert(
        p.id.clone(),
        LemmaEvidence {
            input: s.input.clone(),
            step_hash: step_hash(&p),
            outcome: Outcome::Unresolved,
            cache: "cache1".into(),
        },
    );
    let task = Task {
        kind: TaskKind::ProveLemma,
        targets: vec!["a".into()],
        work: Some(p.id.clone()),
    };
    let key = scheduler::task_input(&s, &task);
    s.proof_plan.evidence.get_mut(&p.id).unwrap().cache = "cache2".into();
    s.proof_plan.steps[0].reason =
        "Additional diagnostics do not change the mathematical task".into();
    assert_eq!(key, scheduler::task_input(&s, &task));
}
#[test]
fn failed_prerequisite_returns_to_bounded_replanning() {
    use cargo_elicit::plan::*;
    let mut s = state();
    s.tool = Tool::Aeneas;
    s.goal = VerificationGoal::SelectedApis(vec!["a".into()]);
    let p = lemma_step("a");
    install(&mut s, vec![p.clone()]).unwrap();
    let t = scheduler::next(&s, true, &Limits::default()).unwrap();
    assert_eq!(t.kind, TaskKind::ProveLemma);
    s.proof_plan.evidence.insert(
        p.id.clone(),
        LemmaEvidence {
            input: s.input.clone(),
            step_hash: step_hash(&p),
            outcome: Outcome::Unresolved,
            cache: "cache".into(),
        },
    );
    s.run.attempts.insert(
        t.key(),
        Attempt {
            failed: true,
            last_input: scheduler::task_input(&s, &t),
            total: 1,
            ..Default::default()
        },
    );
    let repair = scheduler::next(&s, true, &Limits::default()).unwrap();
    assert_eq!(repair.kind, TaskKind::Plan);
    assert_eq!(repair.work.as_deref(), Some("repair"));
    s.run.attempts.insert(
        repair.key(),
        Attempt {
            failed: true,
            last_input: scheduler::task_input(&s, &repair),
            total: 1,
            ..Default::default()
        },
    );
    assert!(scheduler::next(&s, true, &Limits::default()).is_none());
}

#[test]
fn root_dependency_uses_its_own_prerequisites_and_rejects_delegation_cycles() {
    use cargo_elicit::plan::*;
    let mut s = state();
    s.tool = Tool::Aeneas;
    s.goal = VerificationGoal::SelectedApis(vec!["c".into(), "a".into()]);
    let delegate = ProofStep {
        id: "need_a".into(),
        goal: "c".into(),
        kind: StepKind::Contract,
        target: Some("a".into()),
        reason: "Caller c uses selected API a".into(),
        needs: vec![],
        statement: None,
        theorem: None,
    };
    let lemma = lemma_step("a");
    install(&mut s, vec![delegate.clone(), lemma]).unwrap();
    let t = scheduler::next(&s, true, &Limits::default()).unwrap();
    assert_eq!(t.kind, TaskKind::ProveLemma);
    assert_eq!(t.work.as_deref(), Some("loop_bound"));
    let back = ProofStep {
        id: "need_c".into(),
        goal: "a".into(),
        kind: StepKind::Contract,
        target: Some("c".into()),
        reason: "Would create circular root completion".into(),
        needs: vec![],
        statement: None,
        theorem: None,
    };
    assert!(install(&mut s, vec![delegate, back]).is_err());
}

#[test]
fn saved_lemma_is_checked_before_reauthoring_and_new_input_reopens_checkpoint() {
    use cargo_elicit::plan::*;
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("elicit-proofs/aeneas");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Lemmas.lean"), "import ElicitContracts\n").unwrap();
    let mut s = state();
    s.tool = Tool::Aeneas;
    s.scope.manifest_path = root
        .path()
        .join("Cargo.toml")
        .to_string_lossy()
        .into_owned();
    install(&mut s, vec![lemma_step("a")]).unwrap();
    assert!(checkpoint_needed(&s));
    s.proof_plan.last_checkpoint = checkpoint_key(&s);
    assert!(!checkpoint_needed(&s));
    s.input = "new proof input".into();
    assert!(checkpoint_needed(&s));
}
