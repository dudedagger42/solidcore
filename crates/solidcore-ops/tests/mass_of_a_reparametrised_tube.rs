//! A tube measures the same whatever parameters its faces' splines run over.
//! SolidCraft #12760: a round tube (radii 21.2 and 18.6, 5071.94 long, its axis in the XZ
//! plane) measured 14 % over its volume after a STEP export and import. The two records are
//! SolidCraft's body as built (its splines run over arc length from 0) and as read back from
//! its STEP file (the same surfaces, the splines over other parameter ranges: 3..7 round the
//! cylinders, 4..8 along the end faces' circles).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // test

use solidcore_ops::Body;
use solidcore_record::brep::Brep as Record;

const BUILT: &str = include_str!("data/tube_built.json");
const READ_BACK: &str = include_str!("data/tube_after_step_round_trip.json");

/// The tube's length: the outer cylinder's spline runs 0..LENGTH along its axis.
const LENGTH: f64 = 5071.94122980056;

fn body(json: &str) -> Body {
    let record: Record = serde_json::from_str(json).unwrap();
    Body::from_record(&record, true).unwrap().0
}

#[test]
fn a_tube_read_back_from_step_measures_as_built() {
    let exact = std::f64::consts::PI * (21.2_f64.powi(2) - 18.6_f64.powi(2)) * LENGTH;
    for (name, json) in [("built", BUILT), ("read back", READ_BACK)] {
        let m = body(json).mass_properties(1e-6).unwrap();
        assert!(
            (m.volume - exact).abs() < 1e-5 * exact,
            "{name}: volume {} against {exact}",
            m.volume
        );
    }
}

/// Probe (not for the PR): each read-back face swapped for the built one.
#[test]
#[ignore = "probe"]
fn probe_swap_faces() {
    let built: serde_json::Value = serde_json::from_str(BUILT).unwrap();
    let back: serde_json::Value = serde_json::from_str(READ_BACK).unwrap();
    let n = back["faces"].as_array().unwrap().len();
    for i in 0..n {
        let mut v = back.clone();
        v["faces"][i] = built["faces"][i].clone();
        let r: Record = serde_json::from_value(v).unwrap();
        let m = Body::from_record(&r, true).unwrap().0.mass_properties(1e-6).unwrap();
        println!("PROBE face {i} from built: volume {} area {}", m.volume, m.area);
        let mut w = built.clone();
        w["faces"][i] = back["faces"][i].clone();
        let r: Record = serde_json::from_value(w).unwrap();
        let m = Body::from_record(&r, true).unwrap().0.mass_properties(1e-6).unwrap();
        println!("PROBE face {i} from read-back: volume {} area {}", m.volume, m.area);
    }
}
