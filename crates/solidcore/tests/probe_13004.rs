#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
use solidcore::boolean::api_brep::boolean_union;
use solidcore::geom::attribute::{AttributeStore, ObjectKey};
use solidcore::geom::object::ObjectRef;
use solidcore::kernel::UpstreamFixes;
use solidcore::mass::integrator::properties::{brep_properties, MassPropertyBehavior, PropertyAttributes};
use solidcore::query::brep::{calculate_bounding_box, BoundingBoxOptions};
use solidcore::topology::brep::p2::is_manifold_solid;
use solidcore::topology::api_files::{read_brep_from_file, write_brep_to_file};
use solidcore::topology::brep_data_write::FromBrepEnv;
use solidcore::topology::{EdgeId, EdgeuseId, FaceId};
use solidcore::topology::brep::p10::TopologyEnv;
use solidcore::topology::topology::NotifyRecords;
use solidcore::{Brep, EntityRef, Kernel};

fn caller_named(e: EntityRef) -> ObjectRef {
    let r = solidcore::topology::brep::p3::topology_ref(0, e);
    ObjectRef { type_code: r.type_code, key: ObjectKey(r.type_code.unsigned_abs() * 1_000_000 + (r.key.0 & 0xffff_ffff)) }
}

fn volume(k: &Kernel, b: &mut Brep) -> Option<f64> {
    let bbox = calculate_bounding_box(k, &mut *b, BoundingBoxOptions::default()).unwrap();
    let store = AttributeStore::default();
    let key_of = |_| ObjectKey(0);
    brep_properties(k, b, 1e-6, bbox.mid(), MassPropertyBehavior::default(), &PropertyAttributes { store: &store, key_of: &key_of }).ok().map(|p| p.volume)
}

fn read(k: &mut Kernel, name: &str) -> Brep {
    let bytes = std::fs::read(format!("B:/Claude/ct/opus-d-smb/{name}.smb")).unwrap();
    let mut attributes = AttributeStore::new();
    let mut records = NotifyRecords::new();
    let mut env = TopologyEnv { all_attributes: &[], attributes: &mut attributes, key_of: &caller_named, records: &mut records, next_key: 1 << 50 };
    let mut b = read_brep_from_file(k, Some(bytes), false, false, &mut env).unwrap();
    b.refresh_cached_topology();
    b
}

#[test]
fn probe_13004() {
    let mut only = UpstreamFixes::default();
    only.boolean_union_refuses_solid_leak = true;
    for (fixes, label) in [(UpstreamFixes::default(), "default"), (only, "leak-guard"), (UpstreamFixes::all(), "all")] {
        for (a, b) in [("torso", "arm"), ("torso", "arm_mirrored"), ("torso_mirrored", "arm"), ("torso", "cyl"), ("torso", "cyl_mirrored"), ("torso", "cyl_direct")] {
            if !std::path::Path::new(&format!("B:/Claude/ct/opus-d-smb/{b}.smb")).exists() { continue; }
            let mut k = Kernel::default();
            k.fixes = fixes;
            let x = read(&mut k, a);
            let y = read(&mut k, b);
            match boolean_union(&mut k, x, y) {
                Ok(mut u) => {
                    let faces = solidcore::topology::brep::p1::faces(&u, None).unwrap().len();
                    let solid = is_manifold_solid(&u);
                    eprintln!("{label} {a}+{b}: ok, faces {faces}, manifold solid {solid:?}, volume {:?}", volume(&k, &mut u));
                }
                Err(e) => eprintln!("{label} {a}+{b}: ERR {e}"),
            }
        }
    }
}

#[test]
fn probe_13004_live() {
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let mut only = UpstreamFixes::default();
    only.boolean_union_refuses_solid_leak = true;
    for (fixes, label) in [(UpstreamFixes::default(), "default"), (only, "leak-guard"), (UpstreamFixes::all(), "all")] {
        for n in ["cyl", "arm"] {
            let mut k = Kernel::default();
            k.fixes = fixes;
            let t = read(&mut k, "torso");
            let mut x = read(&mut k, n);
            solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut x, &mirror, true).unwrap();
            match boolean_union(&mut k, t, x) {
                Ok(mut u) => {
                    let solid = is_manifold_solid(&u);
                    eprintln!("live {label} torso+mirror({n}): ok, manifold solid {solid:?}, volume {:?}", volume(&k, &mut u));
                }
                Err(e) => eprintln!("live {label} torso+mirror({n}): ERR {e}"),
            }
        }
    }
}

fn bytes(k: &Kernel, b: &Brep) -> Vec<u8> {
    let store = AttributeStore::new();
    let records = NotifyRecords::new();
    let env = FromBrepEnv {
        attributes: &store,
        key_of: &caller_named,
        records: &records,
        curve_key: &|_: EdgeId| ObjectKey(0),
        uv_curve_key: &|_: EdgeuseId| ObjectKey(0),
        surface_key: &|_: FaceId| ObjectKey(0),
    };
    write_brep_to_file(k, Some(b), &env, false).unwrap()
}

fn reread(k: &mut Kernel, data: Vec<u8>) -> Brep {
    let mut attributes = AttributeStore::new();
    let mut records = NotifyRecords::new();
    let mut env = TopologyEnv { all_attributes: &[], attributes: &mut attributes, key_of: &caller_named, records: &mut records, next_key: 1 << 50 };
    let mut b = read_brep_from_file(k, Some(data), false, false, &mut env).unwrap();
    b.refresh_cached_topology();
    b
}

#[test]
fn probe_13004_dump() {
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let mut k = Kernel::default();
    k.fixes = UpstreamFixes::all();
    let mut x = read(&mut k, "cyl");
    solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut x, &mirror, true).unwrap();
    let data = bytes(&k, &x);
    let y = reread(&mut k, data);
    std::fs::write("B:/Claude/ct/opus-d-smb/mem.txt", format!("{x:#?}")).unwrap();
    std::fs::write("B:/Claude/ct/opus-d-smb/reload.txt", format!("{y:#?}")).unwrap();
    for (what, b) in [("mem", x), ("reload", y)] {
        let t = read(&mut k, "torso");
        match boolean_union(&mut k, t, b) {
            Ok(mut u) => eprintln!("dump {what}: ok, volume {:?}", volume(&k, &mut u)),
            Err(e) => eprintln!("dump {what}: ERR {e}"),
        }
    }
}
