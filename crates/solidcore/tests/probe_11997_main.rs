#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::print_stderr)]
//! probe for solidcraft#11997, not for a PR
use solidcore::boolean::api_brep::boolean_union;
use solidcore::geom::attribute::{AttributeStore, ObjectKey};
use solidcore::kernel::{KernelOptions, UpstreamFixes};
use solidcore::mass::integrator::properties::{
    brep_properties, MassPropertyBehavior, PropertyAttributes,
};
use solidcore::query::brep::{calculate_bounding_box, BoundingBoxOptions};
use solidcore::topology::api_files::read_brep_from_file;
use solidcore::topology::brep::mirror::transform_matrix;
use solidcore::topology::brep::p10::TopologyEnv;
use solidcore::topology::brep::p2::is_manifold_solid;
use solidcore::topology::topology::NotifyRecords;
use solidcore::{Brep, Kernel};

fn read(k: &mut Kernel, bytes: &[u8]) -> Brep {
    let fresh = Brep::new();
    let mut store = AttributeStore::new();
    let mut records = NotifyRecords::new();
    let key_of = |e| fresh.owner_identity(e);
    let mut env = TopologyEnv {
        all_attributes: &[],
        attributes: &mut store,
        key_of: &key_of,
        records: &mut records,
        next_key: 1 << 50,
    };
    let mut b = read_brep_from_file(k, Some(bytes.to_vec()), false, false, &mut env).unwrap();
    b.refresh_cached_topology();
    b
}

fn volume(k: &Kernel, b: &mut Brep) -> f64 {
    let bbox = calculate_bounding_box(k, &mut *b, BoundingBoxOptions::default()).unwrap();
    let store = AttributeStore::default();
    let key_of = |_| ObjectKey(0);
    brep_properties(
        k,
        b,
        1e-9,
        bbox.mid(),
        MassPropertyBehavior::default(),
        &PropertyAttributes {
            store: &store,
            key_of: &key_of,
        },
    )
    .unwrap()
    .volume
}


#[test]
fn probe_11997_main() {
    let dir = std::path::Path::new("B:/Claude/ct/opus-d/smb/11997");
    for mask in ["mask_imported", "mask_raised"] {
        for rib in ["rib1", "rib3", "rib7"] {
            for (fx, fixes) in [("default", UpstreamFixes::default()), ("all", UpstreamFixes::all())] {
                let mut k = Kernel::new(&KernelOptions::default());
                k.fixes = fixes;
                let m = read(&mut k, &std::fs::read(dir.join(format!("{mask}.smb"))).unwrap());
                let r = read(&mut k, &std::fs::read(dir.join(format!("{rib}.smb"))).unwrap());
                match boolean_union(&mut k, m, r) {
                    Ok(mut u) => { let s = is_manifold_solid(&u).unwrap(); eprintln!("{mask} + {rib} [{fx}]: solid {s} vol {}", volume(&k, &mut u)); }
                    Err(e) => eprintln!("{mask} + {rib} [{fx}]: ERR {e}"),
                }
            }
        }
    }
}

fn fails(names: &[&str], mask: &str, rib: &str) -> bool {
    let dir = std::path::Path::new("B:/Claude/ct/opus-d/smb/11997");
    let mut k = Kernel::new(&KernelOptions::default());
    let mut f = UpstreamFixes::default();
    for n in names { *f.flag_mut(n).unwrap() = true; }
    k.fixes = f;
    let m = read(&mut k, &std::fs::read(dir.join(format!("{mask}.smb"))).unwrap());
    let r = read(&mut k, &std::fs::read(dir.join(format!("{rib}.smb"))).unwrap());
    match boolean_union(&mut k, m, r) {
        Ok(u) => !is_manifold_solid(&u).unwrap(),
        Err(e) => { eprintln!("  err {e}"); true }
    }
}

#[test]
fn probe_11997_bisect() {
    let all: Vec<&str> = UpstreamFixes::default().flags().filter(|(_, on)| !on).map(|(n, _)| n).collect();
    eprintln!("{} non-default flags", all.len());
    for (mask, rib) in [("mask_raised", "rib3"), ("mask_imported", "rib7")] {
        let mut set = all.clone();
        assert!(fails(&set, mask, rib));
        while set.len() > 1 {
            let (a, b) = set.split_at(set.len() / 2);
            if fails(a, mask, rib) { set = a.to_vec(); }
            else if fails(b, mask, rib) { set = b.to_vec(); }
            else { eprintln!("{mask}+{rib}: needs flags from both halves of {set:?}"); break; }
        }
        eprintln!("{mask}+{rib}: culprit set {set:?}");
    }
}

#[test]
fn probe_11997_one() {
    let mut f = UpstreamFixes::default();
    *f.flag_mut("brep_edit_statuses_pass_on").unwrap() = true;
    let dir = std::path::Path::new("B:/Claude/ct/opus-d/smb/11997");
    let mut k = Kernel::new(&KernelOptions::default());
    k.fixes = f;
    let m = read(&mut k, &std::fs::read(dir.join("mask_imported.smb")).unwrap());
    let r = read(&mut k, &std::fs::read(dir.join("rib7.smb")).unwrap());
    eprintln!("result {:?}", boolean_union(&mut k, m, r).map(|_| ()).err());
}

#[test]
fn probe_11997_faces() {
    let dir = std::path::Path::new("B:/Claude/ct/opus-d/smb/11997");
    let mut k = Kernel::new(&KernelOptions::default());
    for n in ["mask_imported", "rib7"] {
        let b = read(&mut k, &std::fs::read(dir.join(format!("{n}.smb"))).unwrap());
        for f in solidcore::topology::brep::p1::faces(&b, None).unwrap() {
            let s = solidcore::topology::face::p2::surface(&b, f).unwrap().map(|s| s.type_string()).unwrap_or("none");
            let vs = solidcore::topology::face::p4::vertices(&b, f, None, false, None).unwrap();
            let pts: Vec<_> = vs.iter().map(|v| { let p = solidcore::topology::vertex::point(&b, *v).unwrap(); (p.x * 1000.0).round() / 1000.0 }).collect();
            eprintln!("{n} {f:?} {s} xs {pts:?}");
        }
    }
}
