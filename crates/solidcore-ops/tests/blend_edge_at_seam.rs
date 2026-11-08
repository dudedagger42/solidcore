//! A round on an edge that ends on the seam of a periodic face builds as it does away from it.
//! SolidCraft #12001: a 6.75 mm block, a bore of radius 0.762 through it and a 90° V-groove
//! along x across the top, through the bore's axis (x = -7.2, y = 0). The groove's valley edge
//! runs from the bore's wall to the block's side on either side of the bore; a 0.2 mm round on
//! the half that ends where the bore's circle starts (its seam, at +x) failed with status 1001,
//! the other half built. The record is SolidCraft's body, its bore extruded from a circle
//! starting at angle 0.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // test

mod common;

use common::{edge_at, volume};
use solidcore_ops::{blend, Blend, Body, Corners};
use solidcore_record::brep::Brep as Record;

/// The valley's height: the groove is 0.762 deep below the top at z = 5.8.
const VALLEY_Z: f64 = 5.038;

fn grooved() -> Body {
    let record: Record =
        serde_json::from_str(include_str!("data/groove_valley_at_bore_seam.json")).unwrap();
    Body::from_record(&record, true).unwrap().0
}

/// The volume a 0.2 round adds on the valley's half through `x` (the round is made on a fresh
/// copy of the body, in place).
fn round_adds(x: f64) -> f64 {
    let body = grooved();
    let before = volume(&body);
    let e = edge_at(&body, [x, 0.0, VALLEY_Z]);
    blend(&body, Blend::Round, &[e], &[0.2], &[], Corners::Filled)
        .unwrap_or_else(|err| panic!("the round on the valley through x {x}: {err:?}"));
    volume(&body) - before
}

#[test]
fn a_round_on_a_valley_edge_ending_on_a_bores_seam_builds_as_its_mirror_image() {
    // The halves are mirror images in x = -7.2 but for where the bore's circle starts.
    let away = round_adds(-8.7);
    let at_seam = round_adds(-5.7);
    // A 90° concave valley's round of r adds (1 - pi/4) r^2 = 0.0086 mm² of section.
    assert!(away > 0.0, "the round away from the seam adds {away}");
    assert!(
        (at_seam - away).abs() < 1e-6 * away.max(1.0),
        "the round ending on the seam adds {at_seam}, its mirror image {away}"
    );
}
