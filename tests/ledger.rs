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
        body_hash: "body".into(),
        original_body: "body".into(),
        source_hash: "source".into(),
        specified: true,
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
