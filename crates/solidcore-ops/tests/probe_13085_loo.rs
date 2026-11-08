//! PROBE (not for the PR): which of SolidCraft's fixes makes the 13085 cut at 1e-9 / 36 deg fail.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
use solidcore_ops::boolean::{boolean, BooleanOp};
use solidcore_ops::{session, Body};

fn load(name: &str) -> Body {
    let bytes = std::fs::read(format!("B:/Claude/boolean-13085/smb/{name}")).unwrap();
    Body::from_smb(bytes, false).unwrap()
}

fn cut(tool: &str) -> String {
    let a = load("cup.smb");
    let b = load(tool);
    match boolean(a, b, BooleanOp::Subtract) {
        Ok(Some(r)) => format!("ok faces {}", r.counts().unwrap()[0]),
        Ok(None) => "ok empty".to_string(),
        Err(e) => format!("ERR {e}"),
    }
}

#[test]
fn probe_leave_one_out() {
    let names: Vec<String> = std::fs::read_to_string("probe_fixes.txt")
        .unwrap()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let all: Vec<&str> = names
        .iter()
        .map(String::as_str)
        .filter(|n| {
            let known = session::set_fixes([*n]).is_ok();
            if !known {
                eprintln!("LOO unknown flag dropped: {n}");
            }
            known
        })
        .collect();
    eprintln!("LOO {} flags kept", all.len());
    for tool in ["tool36_exact.smb", "tool36_1e-9.smb"] {
        session::set_fixes([]).unwrap();
        let base_default = cut(tool);
        session::set_fixes(all.iter().copied()).unwrap();
        let base_all = cut(tool);
        eprintln!("LOO {tool}: defaults -> {base_default}; solidcraft set -> {base_all}");
        for (i, n) in all.iter().enumerate() {
            let minus: Vec<&str> = all.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, x)| *x).collect();
            session::set_fixes(minus).unwrap();
            let r = cut(tool);
            if r.starts_with("ok") != base_all.starts_with("ok") {
                eprintln!("LOO {tool}: set minus {n} -> {r}");
            }
            session::set_fixes([*n]).unwrap();
            let r = cut(tool);
            if r.starts_with("ok") != base_default.starts_with("ok") {
                eprintln!("LOO {tool}: defaults plus {n} -> {r}");
            }
        }
        eprintln!("LOO {tool}: done");
    }
}
