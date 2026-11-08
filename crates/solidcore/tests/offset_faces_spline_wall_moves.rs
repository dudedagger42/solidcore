//! Offset faces on a B-spline wall (solidcraft#10884, #13103 via #8287): a prism whose one wall
//! is the extrusion of an interpolating spline, that wall moved 1.6 along its normal by the
//! offset faces executive. The move is made (the body grows by 1.6 times the wall's area), and
//! it is not refused as `NoEffect`: the wall is one face, so after the local operation its UV
//! domain is the extended offset surface's box, and nine points over that box lie far off the
//! face; the "moved no face" test must sample the face's own region.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // test helpers

use solidcore::curve::curve::Curve;
use solidcore::geom::attribute::{AttributeStore, ObjectKey};
use solidcore::mass::integrator::properties::{brep_volume, PropertyAttributes};
use solidcore::math::Vec3;
use solidcore::offset::offset_faces::OffsetFacesExecutive;
use solidcore::primitive::api_curves::{create_curve, create_line_segment};
use solidcore::sweep::api_brep::create_taper_extrude;
use solidcore::topology::brep::p2::is_manifold_solid;
use solidcore::topology::face::p2 as face_p2;
use solidcore::topology::FaceId;
use solidcore::{Brep, Kernel};

const H: f64 = 5.0;
const D: f64 = 1.6;

fn volume(k: &Kernel, b: &mut Brep) -> f64 {
    let store = AttributeStore::default();
    let attrs = PropertyAttributes {
        store: &store,
        key_of: &|_| ObjectKey(0),
    };
    brep_volume(k, b, 1e-11, &attrs).unwrap()
}

/// The prism: the region under the spline through (0, 0), (10, 5), (20, 0), (30, 5) down to
/// y = -10, extruded `H` along z (the extrusion runs toward -z).
fn prism(k: &mut Kernel) -> Brep {
    let p = |x: f64, y: f64| Vec3::new(x, y, 0.0);
    let spline = create_curve(&[p(0.0, 0.0), p(10.0, 5.0), p(20.0, 0.0), p(30.0, 5.0)]).unwrap();
    let curves = [
        Curve::from(spline),
        Curve::from(create_line_segment(p(30.0, 5.0), p(30.0, -10.0)).unwrap()),
        Curve::from(create_line_segment(p(30.0, -10.0), p(0.0, -10.0)).unwrap()),
        Curve::from(create_line_segment(p(0.0, -10.0), p(0.0, 0.0)).unwrap()),
    ];
    let refs: Vec<Option<&Curve>> = curves.iter().map(Some).collect();
    create_taper_extrude(k, &refs, H, 0.0, 3, false).unwrap()
}

/// The face whose surface centre is at `at` within 1e-6 (the wall's or a cap's).
fn face_at(k: &Kernel, b: &Brep, at: impl Fn(Vec3) -> bool) -> FaceId {
    b.face_list()
        .iter()
        .copied()
        .find(|&f| {
            let (Ok(Some(s)), Ok(d)) = (face_p2::surface(b, f), face_p2::uv_domain(b, f)) else {
                return false;
            };
            s.evaluate_point(k, d.evaluate(0.5, 0.5)).is_ok_and(&at)
        })
        .expect("the face")
}

/// The spline wall's area, from the curve's length (an extrusion): the curve sampled finely.
fn wall_area(k: &Kernel) -> f64 {
    let p = |x: f64, y: f64| Vec3::new(x, y, 0.0);
    let c = Curve::from(
        create_curve(&[p(0.0, 0.0), p(10.0, 5.0), p(20.0, 0.0), p(30.0, 5.0)]).unwrap(),
    );
    let ivl = c.natural_interval();
    let n = 4000;
    let mut len = 0.0;
    let mut prev = c.evaluate_point(k, ivl.min()).unwrap();
    for i in 1..=n {
        let t = ivl.min() + (ivl.max() - ivl.min()) * f64::from(i) / f64::from(n);
        let q = c.evaluate_point(k, t).unwrap();
        len += (q - prev).length();
        prev = q;
    }
    len * H
}

#[test]
fn the_spline_wall_moves_and_is_not_refused() {
    let mut k = Kernel::default();
    let mut b = prism(&mut k);
    let before = volume(&k, &mut b);
    let wall = face_at(&k, &b, |c| {
        c.x > 0.5 && c.x < 29.5 && c.y > -1.0 && c.z.abs() > 1.0 && c.z.abs() < H - 1.0
    });
    let mut exec = OffsetFacesExecutive::new();
    exec.offset_faces(&mut k, &mut b, &[wall], D)
        .unwrap_or_else(|e| panic!("refused {:?}: {e}", exec.refusal()));
    assert!(is_manifold_solid(&b).unwrap(), "not a solid after the move");
    let grew = volume(&k, &mut b) - before;
    // The wall's offset by D sweeps D times the wall's area plus (D^2 / 2) H times the curve's
    // signed turning, which this S-shaped curve's two bends nearly cancel: within two percent.
    let want = D * wall_area(&k);
    assert!(
        (grew - want).abs() < 0.02 * want,
        "the body grew by {grew}, expected about {want}"
    );
}

/// A planar wall of the same prism, moved as before: the control.
#[test]
fn a_planar_wall_of_the_same_prism_moves() {
    let mut k = Kernel::default();
    let mut b = prism(&mut k);
    let before = volume(&k, &mut b);
    let wall = face_at(&k, &b, |c| (c.y + 10.0).abs() < 1e-6 && c.z.abs() > 1.0);
    let mut exec = OffsetFacesExecutive::new();
    exec.offset_faces(&mut k, &mut b, &[wall], D)
        .unwrap_or_else(|e| panic!("refused {:?}: {e}", exec.refusal()));
    assert!(is_manifold_solid(&b).unwrap());
    let grew = volume(&k, &mut b) - before;
    assert!(
        (grew - D * 30.0 * H).abs() < 1e-6,
        "the body grew by {grew}"
    );
}
