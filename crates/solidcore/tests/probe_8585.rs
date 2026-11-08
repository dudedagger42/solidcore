//! kernel-f probe for SolidCraft #8585 (not for upstream): a 0.2 thick plate with k x k
//! square holes joined onto a box whose top it lies on; time vs k.
use solidcore::boolean::api_brep::{boolean_difference, boolean_union};
use solidcore::kernel::UpstreamFixes;
use solidcore::primitive::api_primitives::create_box;
use solidcore::topology::brep::p1::faces;
use solidcore::{Kernel, Point3};
use std::time::Instant;

#[test]
#[ignore = "probe"]
fn probe_8585() -> Result<(), Box<dyn std::error::Error>> {
    let ks: Vec<usize> = std::env::var("PROBE_KS").ok().map(|v| v.split(',').map(|a| a.parse().unwrap()).collect()).unwrap_or(vec![5, 10, 14, 20]);
    for overlap in [false, true] {
        for &k in &ks {
            let mut kernel = Kernel::default();
            kernel.fixes = UpstreamFixes::all();
            let mut plate = create_box(&mut kernel, Point3::new(-25.0, -25.0, 10.0), 50.0, 50.0, 0.2)?;
            let pitch = 48.0 / k as f64;
            let t = Instant::now();
            kernel.work_counters.reset();
            for i in 0..k {
                for j in 0..k {
                    let c = Point3::new(-24.0 + (i as f64 + 0.2) * pitch, -24.0 + (j as f64 + 0.2) * pitch, 9.0);
                    let tool = create_box(&mut kernel, c, pitch * 0.6, pitch * 0.6, 3.0)?;
                    plate = boolean_difference(&mut kernel, plate, tool)?;
                }
            }
            let cut_s = t.elapsed().as_secs_f64();
            let cut_pairs = kernel.work_counters.face_pairs();
            let cut_loops = kernel.work_counters.loops_classified();
            #[cfg(feature = "profile")]
            let _ = kernel.take_profile();
            let h = if overlap { 10.1 } else { 10.0 };
            let base = create_box(&mut kernel, Point3::new(-30.0, -30.0, 0.0), 60.0, 60.0, h)?;
            kernel.work_counters.reset();
            let t = Instant::now();
            let u = boolean_union(&mut kernel, base, plate)?;
            let join_s = t.elapsed().as_secs_f64();
            println!(
                "{{\"overlap\":{overlap},\"holes\":{},\"cut_s\":{cut_s:.3},\"cut_pairs\":{cut_pairs},\"cut_loops\":{cut_loops},\"join_s\":{join_s:.3},\"join_pairs\":{},\"join_loops\":{},\"faces\":{}}}",
                k * k,
                kernel.work_counters.face_pairs(),
                kernel.work_counters.loops_classified(),
                faces(&u, None)?.len()
            );
            #[cfg(feature = "profile")]
            {
                let p = kernel.take_profile();
                let mut s: Vec<_> = p.summary.iter().collect();
                s.sort_by_key(|n| std::cmp::Reverse(n.self_ns.unwrap_or(0)));
                for n in s.iter().take(12) {
                    println!("   {:>10.1} ms self {:>10.1} total x{} {}", n.self_ns.unwrap_or(0) as f64 / 1e6, n.total_ns.unwrap_or(0) as f64 / 1e6, n.count, n.name);
                }
            }
        }
    }
    Ok(())
}
