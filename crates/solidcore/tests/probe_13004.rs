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

#[test]
fn probe_13004_ab() {
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    for fixes in [UpstreamFixes::default(), UpstreamFixes::all()] {
        for n in ["cyl", "arm"] {
            for mode in ["none", "refresh", "shells"] {
                let mut k = Kernel::default();
                k.fixes = fixes;
                let t = read(&mut k, "torso");
                let mut x = read(&mut k, n);
                solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut x, &mirror, true).unwrap();
                match mode {
                    "refresh" => x.refresh_cached_topology(),
                    "shells" => {
                        let ids: Vec<_> = x.shells().ids().collect();
                        for sid in ids {
                            let mut faces = Vec::new();
                            for fu in x.shell(sid).unwrap().get_faceuses().unwrap() {
                                if let Some(f) = x.faceuse(fu).unwrap().face {
                                    if !faces.contains(&f) { faces.push(f); }
                                }
                            }
                            x.shell_mut(sid).unwrap().faces = faces;
                        }
                    }
                    _ => {}
                }
                let all = fixes == UpstreamFixes::all();
                match boolean_union(&mut k, t, x) {
                    Ok(mut u) => eprintln!("ab all={all} {n} {mode}: ok, solid {:?}, volume {:?}", is_manifold_solid(&u), volume(&k, &mut u)),
                    Err(e) => eprintln!("ab all={all} {n} {mode}: ERR {e}"),
                }
            }
        }
    }
}

#[test]
fn probe_13004_cache() {
    use solidcore::query::brep::BrepQuery;
    use solidcore::topology::brep::p1::SetRegionVoidType;
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    for n in ["cyl", "arm"] {
        for mode in ["reload", "reload+infinite", "orig+infinite"] {
            let mut k = Kernel::default();
            k.fixes = UpstreamFixes::all();
            let t = read(&mut k, "torso");
            let mut x = read(&mut k, n);
            if mode != "orig+infinite" {
                solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut x, &mirror, true).unwrap();
                let data = bytes(&k, &x);
                x = reread(&mut k, data);
            }
            if mode.ends_with("+infinite") {
                let previous = x.editing_enabled();
                x.set_editing_enabled(false);
                let query = BrepQuery::of_brep(&x);
                let r = solidcore::topology::brep::p5::find_and_set_infinite_region(&mut k, &mut x, &query, None, SetRegionVoidType::InfiniteRegionSwap, None);
                x.set_editing_enabled(previous);
                eprintln!("cache {n} {mode}: infinite {:?}", r.map(|_| ()));
            }
            match boolean_union(&mut k, t, x) {
                Ok(mut u) => eprintln!("cache {n} {mode}: ok, solid {:?}, volume {:?}", is_manifold_solid(&u), volume(&k, &mut u)),
                Err(e) => eprintln!("cache {n} {mode}: ERR {e}"),
            }
        }
    }
}

/// Moves the UV trim of edge use `from` (by arena position) onto `to`.
fn move_trim(b: &mut Brep, from: usize, to: usize) {
    let ids: Vec<_> = b.edgeuses().ids().collect();
    let c = b.edgeuse_mut(ids[from]).unwrap().uv_trim_curve.take();
    assert!(c.is_some(), "no trim on {from}");
    assert!(b.edgeuse(ids[to]).unwrap().uv_trim_curve.is_none());
    b.edgeuse_mut(ids[to]).unwrap().uv_trim_curve = c;
}

#[test]
fn probe_13004_move() {
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let moves: [(&str, &[(usize, usize)]); 5] = [("none", &[]), ("all", &[(0, 1), (6, 7), (10, 11)]), ("seam", &[(0, 1)]), ("e2", &[(6, 7)]), ("cap", &[(10, 11)])];
    for (what, m) in moves {
        let mut k = Kernel::default();
        k.fixes = UpstreamFixes::all();
        let t = read(&mut k, "torso");
        let mut x = read(&mut k, "cyl");
        solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut x, &mirror, true).unwrap();
        let data = bytes(&k, &x);
        let mut x = reread(&mut k, data);
        for &(a, b) in m {
            move_trim(&mut x, a, b);
        }
        match boolean_union(&mut k, t, x) {
            Ok(mut u) => eprintln!("move {what}: ok, solid {:?}, volume {:?}", is_manifold_solid(&u), volume(&k, &mut u)),
            Err(e) => eprintln!("move {what}: ERR {e}"),
        }
    }
}

fn copy_edge_curves(to: &mut Brep, from: &Brep) {
    let a: Vec<_> = to.edges().ids().collect();
    let b: Vec<_> = from.edges().ids().collect();
    for (ea, eb) in a.into_iter().zip(b) {
        to.edge_mut(ea).unwrap().curve = from.edge(eb).unwrap().curve.clone();
    }
}

fn copy_trims(to: &mut Brep, from: &Brep) {
    let a: Vec<_> = to.edgeuses().ids().collect();
    let b: Vec<_> = from.edgeuses().ids().collect();
    for (ea, eb) in a.into_iter().zip(b) {
        to.edgeuse_mut(ea).unwrap().uv_trim_curve = from.edgeuse(eb).unwrap().uv_trim_curve.clone();
    }
}

#[test]
fn probe_13004_transplant() {
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    for what in ["mem", "mem<-reload curves", "mem<-reload trims", "mem<-reload both", "reload", "reload<-mem curves", "reload<-mem trims", "reload<-mem both"] {
        let mut k = Kernel::default();
        k.fixes = UpstreamFixes::all();
        let t = read(&mut k, "torso");
        let mut mem = read(&mut k, "cyl");
        solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut mem, &mirror, true).unwrap();
        let data = bytes(&k, &mem);
        let mut reload = reread(&mut k, data);
        let x = if what.starts_with("mem") {
            if what.contains("curves") || what.contains("both") { copy_edge_curves(&mut mem, &reload); }
            if what.contains("trims") || what.contains("both") { copy_trims(&mut mem, &reload); }
            mem
        } else {
            if what.contains("curves") || what.contains("both") { copy_edge_curves(&mut reload, &mem); }
            if what.contains("trims") || what.contains("both") { copy_trims(&mut reload, &mem); }
            reload
        };
        match boolean_union(&mut k, t, x) {
            Ok(mut u) => eprintln!("tp {what}: ok, solid {:?}, volume {:?}", is_manifold_solid(&u), volume(&k, &mut u)),
            Err(e) => eprintln!("tp {what}: ERR {e}"),
        }
    }
}

#[test]
fn probe_13004_seam() {
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    for what in ["seam only", "twice", "twice arm", "orig"] {
        let mut k = Kernel::default();
        k.fixes = UpstreamFixes::all();
        let t = read(&mut k, "torso");
        let n = if what == "twice arm" { "arm" } else { "cyl" };
        let mut x = read(&mut k, n);
        match what {
            "seam only" => {
                solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut x, &mirror, true).unwrap();
                let data = bytes(&k, &x);
                let reload = reread(&mut k, data);
                let e0 = x.edges().ids().next().unwrap();
                let r0 = reload.edges().ids().next().unwrap();
                x.edge_mut(e0).unwrap().curve = reload.edge(r0).unwrap().curve.clone();
            }
            "twice" | "twice arm" => {
                solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut x, &mirror, true).unwrap();
                solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut x, &mirror, true).unwrap();
                for e in x.edges().ids().collect::<Vec<_>>() {
                    let kind = format!("{:?}", x.edge(e).unwrap().curve.as_ref().map(|c| c.kind()));
                    eprintln!("seam {what}: edge kind {}", &kind[..kind.len().min(40)]);
                }
            }
            _ => {}
        }
        match boolean_union(&mut k, t, x) {
            Ok(mut u) => eprintln!("seam {what}: ok, solid {:?}, volume {:?}", is_manifold_solid(&u), volume(&k, &mut u)),
            Err(e) => eprintln!("seam {what}: ERR {e}"),
        }
    }
}

fn summary(b: &Brep) -> String {
    use solidcore::topology::brep::p1 as bl;
    let mut pts: Vec<String> = bl::vertices(b, None).unwrap().into_iter().map(|v| {
        let p = b.vertex(v).unwrap().point;
        format!("({:.4},{:.4},{:.4})", p.x, p.y, p.z)
    }).collect();
    pts.sort();
    format!("faces {} edges {} vertices {} {:?}", bl::faces(b, None).unwrap().len(), bl::edges(b, None).unwrap().len(), pts.len(), pts)
}

#[test]
fn probe_13004_imprint() {
    use solidcore::boolean::api_brep::boolean;
    use solidcore::geom::a_object::BooleanOperation;
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    for seam in ["line", "bspline"] {
        for cyl_first in [true, false] {
            let mut k = Kernel::default();
            k.fixes = UpstreamFixes::all();
            let t = read(&mut k, "torso");
            let mut c = read(&mut k, "cyl");
            if seam == "bspline" {
                solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut c, &mirror, true).unwrap();
                solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut c, &mirror, true).unwrap();
            }
            let (a, b) = if cyl_first { (c, t) } else { (t, c) };
            match boolean(&mut k, a, b, BooleanOperation::Imprint) {
                Ok(o) => eprintln!("imprint {seam} cyl_first={cyl_first}: {}", summary(&o.result)),
                Err(e) => eprintln!("imprint {seam} cyl_first={cyl_first}: ERR {e}"),
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Hd(u8);
impl solidcore::geom::solution_array::SolutionObject for Hd {
    fn kind(&self) -> solidcore::geom::solution_array::ObjectKind {
        if self.0 == 0 { solidcore::geom::solution_array::ObjectKind::Curve } else { solidcore::geom::solution_array::ObjectKind::Surface }
    }
}

#[test]
fn probe_13004_cs() {
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let mut k = Kernel::default();
    k.fixes = UpstreamFixes::all();
    let t = read(&mut k, "torso");
    let line_cyl = read(&mut k, "cyl");
    let mut bs_cyl = read(&mut k, "cyl");
    solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut bs_cyl, &mirror, true).unwrap();
    solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut bs_cyl, &mirror, true).unwrap();
    for (what, c) in [("line", &line_cyl), ("bspline", &bs_cyl)] {
        let e0 = c.edges().ids().next().unwrap();
        let edge = c.edge(e0).unwrap();
        let curve = edge.curve.as_ref().unwrap();
        eprintln!("cs {what}: interval {:?} is_line(1e-12) {:?} is_line(1e-9) {:?}", edge.interval, curve.is_line(&k, 10, 1e-12).map(|l| l.is_some()), curve.is_line(&k, 10, 1e-9).map(|l| l.is_some()));
        for f in t.faces().ids() {
            let surf = t.face(f).unwrap().surface.as_ref().unwrap();
            let dom = surf.natural_uv_domain_if_built();
            match surf.global_curve_intersect(&k, [Hd(0), Hd(1)], &dom, curve, &edge.interval, 1e-5) {
                Ok(sols) => {
                    let v: Vec<String> = sols.as_slice().iter().map(|s| format!("{:?} t={:?}", s.solution_type, s.start.param(0))).collect();
                    eprintln!("cs {what} face {f:?}: {} sols {:?}", sols.len(), v);
                }
                Err(e) => eprintln!("cs {what} face {f:?}: ERR {e}"),
            }
        }
    }
}

#[test]
fn probe_13004_bbox() {
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let mut k = Kernel::default();
    k.fixes = UpstreamFixes::all();
    let mut line_cyl = read(&mut k, "cyl");
    let mut bs_cyl = read(&mut k, "cyl");
    solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut bs_cyl, &mirror, true).unwrap();
    solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut bs_cyl, &mirror, true).unwrap();
    for (what, c) in [("line", &mut line_cyl), ("bspline", &mut bs_cyl)] {
        let bb = calculate_bounding_box(&k, &mut *c, BoundingBoxOptions::default());
        eprintln!("bbox {what}: body {:?}", bb.map(|b| (b.min(), b.max())));
        eprintln!("bbox {what}: tolerance {}", c.tolerance());
    }
}

#[test]
fn probe_13004_cc() {
    use solidcore::boolean::api_brep::boolean;
    use solidcore::curve::curve::p8::CurveIntersectOptions;
    use solidcore::geom::a_object::BooleanOperation;
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let mut k = Kernel::default();
    k.fixes = UpstreamFixes::all();
    let t = read(&mut k, "torso");
    let c = read(&mut k, "cyl");
    let imprinted = boolean(&mut k, t, c, BooleanOperation::Imprint).unwrap().result;
    let line_cyl = read(&mut k, "cyl");
    let mut bs_cyl = read(&mut k, "cyl");
    solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut bs_cyl, &mirror, true).unwrap();
    solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut bs_cyl, &mirror, true).unwrap();
    for (what, cyl) in [("line", &line_cyl), ("bspline", &bs_cyl)] {
        let e0 = cyl.edges().ids().next().unwrap();
        let seam = cyl.edge(e0).unwrap();
        let sc = seam.curve.as_ref().unwrap();
        for e in imprinted.edges().ids() {
            let edge = imprinted.edge(e).unwrap();
            let Some(ec) = edge.curve.as_ref() else { continue };
            let opts: CurveIntersectOptions<Hd> = CurveIntersectOptions::default();
            match sc.global_curve_intersect(&k, &seam.interval, ec, &edge.interval, 1e-5, &opts) {
                Ok(sols) if sols.len() > 0 => {
                    let v: Vec<String> = sols.as_slice().iter().map(|s| format!("{:?} t={:?}", s.solution_type, s.start.param(0))).collect();
                    eprintln!("cc {what} vs edge {e:?} ({:?}): {v:?}", std::mem::discriminant(ec.kind()));
                }
                Ok(_) => {}
                Err(err) => eprintln!("cc {what} vs edge {e:?}: ERR {err}"),
            }
        }
        eprintln!("cc {what}: done");
    }
}

#[test]
fn probe_13004_fi() {
    use solidcore::topology::face::p3::{face_intersect_breps, FaceBreps, FaceIntersectOptions};
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    for what in ["line", "bspline"] {
        let mut k = Kernel::default();
        k.fixes = UpstreamFixes::all();
        let mut t = read(&mut k, "torso");
        let mut c = read(&mut k, "cyl");
        if what == "bspline" {
            solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut c, &mirror, true).unwrap();
            solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut c, &mirror, true).unwrap();
        }
        let cf = c.faces().ids().next().unwrap();
        let tf = t.faces().ids().nth(1).unwrap();
        let mut opts = FaceIntersectOptions::default();
        opts.want_this_uv_curves = true;
        opts.want_points = true;
        opts.want_this_classes = true;
        opts.want_other_classes = true;
        match face_intersect_breps(&k, FaceBreps::Two(&mut c, &mut t), cf, tf, 1e-5, 1.0, &opts) {
            Ok(r) => {
                eprintln!("fi {what}: curves {} points {:?}", r.curves_3d.len(), r.points_3d);
                for (i, cv) in r.curves_3d.iter().enumerate() {
                    let iv = cv.natural_interval();
                    let a = cv.evaluate_point(&k, iv.min());
                    let b = cv.evaluate_point(&k, iv.max());
                    eprintln!("fi {what}: curve {i} {:?} {:?} -> {:?}", iv, a, b);
                }
                std::fs::write(format!("B:/Claude/ct/opus-d-smb/fi_{what}.txt"), format!("{:#?}", r.this_classes)).unwrap();
            }
            Err(e) => eprintln!("fi {what}: ERR {e}"),
        }
    }
}

#[test]
fn probe_13004_ana() {
    use solidcore::curve::curve::p8::CurveIntersectOptions;
    use solidcore::topology::face::p3::{face_intersect_breps, FaceBreps, FaceIntersectOptions};
    let mirror = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let mut k = Kernel::default();
    k.fixes = UpstreamFixes::all();
    let mut t = read(&mut k, "torso");
    let mut c = read(&mut k, "cyl");
    let line_cyl = read(&mut k, "cyl");
    solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut c, &mirror, true).unwrap();
    solidcore::topology::brep::mirror::transform_matrix(&mut k, &mut c, &mirror, true).unwrap();
    let cf = c.faces().ids().next().unwrap();
    let tf = t.faces().ids().nth(1).unwrap();
    let r = face_intersect_breps(&k, FaceBreps::Two(&mut c, &mut t), cf, tf, 1e-5, 1.0, &FaceIntersectOptions::default()).unwrap();
    let ell = &r.curves_3d[0];
    for (what, cyl) in [("line", &line_cyl), ("bspline", &c)] {
        let e0 = cyl.edges().ids().next().unwrap();
        let seam = cyl.edge(e0).unwrap();
        let sc = seam.curve.as_ref().unwrap();
        let nat = sc.natural_interval();
        eprintln!("ana {what}: natural {:?} edge interval {:?} P(0.6575) {:?}", nat, seam.interval, sc.evaluate_point(&k, 0.6575051020337));
        if let Ok(Some(a)) = sc.create_analytic_curve(&k, &nat) {
            eprintln!("ana {what}: analytic natural {:?} A(0.6575) {:?}", a.natural_interval(), a.evaluate_point(&k, 0.6575051020337));
        }
        let opts: CurveIntersectOptions<Hd> = CurveIntersectOptions::default();
        let f = ell.global_curve_intersect(&k, &ell.natural_interval(), sc, &seam.interval, 2e-5, &opts).map(|s| s.len());
        let b = sc.global_curve_intersect(&k, &seam.interval, ell, &ell.natural_interval(), 2e-5, &opts).map(|s| s.len());
        eprintln!("ana {what}: ellipse x seam {f:?}; seam x ellipse {b:?}");
    }
}
