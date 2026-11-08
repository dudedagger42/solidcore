//! PROBE (not for the PR): the cost of the self-touch glue pre-pass on a large body.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
use solidcore_ops::boolean::{boolean, BooleanOp};
use solidcore_ops::sweep::{extrude, Curve, DraftCorner};
use solidcore_ops::{session, Body};

const SIDE: usize = 400;

/// A prism over a polygon of 4 * SIDE segments on a square's rim (no trigonometry).
fn prism() -> Body {
    let mut pts = Vec::with_capacity(4 * SIDE);
    for i in 0..SIDE {
        let t = i as f64 / SIDE as f64 * 100.0;
        pts.push([t, 0.0, 0.0]);
    }
    for i in 0..SIDE {
        let t = i as f64 / SIDE as f64 * 100.0;
        pts.push([100.0, t, 0.0]);
    }
    for i in 0..SIDE {
        let t = i as f64 / SIDE as f64 * 100.0;
        pts.push([100.0 - t, 100.0, 0.0]);
    }
    for i in 0..SIDE {
        let t = i as f64 / SIDE as f64 * 100.0;
        pts.push([0.0, 100.0 - t, 0.0]);
    }
    let n = pts.len();
    let curves: Vec<Curve> = (0..n)
        .map(|i| Curve::Line {
            start: pts[i],
            end: pts[(i + 1) % n],
        })
        .collect();
    extrude(
        &curves,
        [0.0, 0.0, 1.0],
        10.0,
        0.0,
        DraftCorner::Curvature,
        true,
    )
    .unwrap()
}

fn small_box() -> Body {
    let c = [
        Curve::Line {
            start: [40.0, 40.0, 5.0],
            end: [60.0, 40.0, 5.0],
        },
        Curve::Line {
            start: [60.0, 40.0, 5.0],
            end: [60.0, 60.0, 5.0],
        },
        Curve::Line {
            start: [60.0, 60.0, 5.0],
            end: [40.0, 60.0, 5.0],
        },
        Curve::Line {
            start: [40.0, 60.0, 5.0],
            end: [40.0, 40.0, 5.0],
        },
    ];
    extrude(&c, [0.0, 0.0, 1.0], 20.0, 0.0, DraftCorner::Curvature, true).unwrap()
}

#[test]
fn probe_timing() {
    let p = prism();
    eprintln!("TIMING prism counts {:?}", p.counts().unwrap());
    let sets: [(&str, Vec<&str>); 3] = [
        ("flag off", vec![]),
        ("flag on", vec!["boolean_glues_self_touching_edges"]),
        ("flag off again", vec![]),
    ];
    for (tag, fixes) in sets {
        session::set_fixes(fixes.iter().copied()).unwrap();
        let mut best = f64::MAX;
        for _ in 0..3 {
            let a = prism();
            let b = small_box();
            let t0 = std::time::Instant::now();
            let r = boolean(a, b, BooleanOp::Union).unwrap().unwrap();
            let dt = t0.elapsed().as_secs_f64();
            best = best.min(dt);
            eprintln!(
                "TIMING {tag}: union {dt:.3} s, result counts {:?}",
                r.counts().unwrap()
            );
        }
        eprintln!("TIMING {tag}: best {best:.3} s");
    }
}
