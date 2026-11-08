//! PROBE (not for the PR): the 13085 cut at 1e-9 / 36 deg from .smb operands, the failing routine.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use solidcore::boolean::api_brep::boolean;
use solidcore::geom::a_object::BooleanOperation;
use solidcore::geom::attribute::{AttributeStore, ObjectKey};
use solidcore::geom::object::ObjectRef;
use solidcore::kernel::Kernel;
use solidcore::topology::api_files::read_brep_from_file;
use solidcore::topology::brep::p10::TopologyEnv;
use solidcore::topology::topology::NotifyRecords;
use solidcore::topology::Brep;
use solidcore::EntityRef;

fn key_of(e: EntityRef) -> ObjectRef {
    let (type_code, kind, i) = match e {
        EntityRef::Brep(x) => (16011, 0u64, x.index()),
        EntityRef::Region(x) => (16012, 1, x.index()),
        EntityRef::Shell(x) => (16013, 2, x.index()),
        EntityRef::Faceuse(x) => (16022, 3, x.index()),
        EntityRef::Face(x) => (16002, 4, x.index()),
        EntityRef::Loopuse(x) => (16004, 5, x.index()),
        EntityRef::Loop(x) => (16003, 6, x.index()),
        EntityRef::Edgeuse(x) => (16005, 7, x.index()),
        EntityRef::Edge(x) => (16016, 8, x.index()),
        EntityRef::Vertexuse(x) => (16006, 9, x.index()),
        EntityRef::Vertex(x) => (16021, 10, x.index()),
    };
    ObjectRef { type_code, key: ObjectKey(((kind + 1) << 32) | u64::from(i)) }
}

fn load(k: &mut Kernel, path: &str) -> Brep {
    let data = std::fs::read(path).unwrap();
    let mut attributes = AttributeStore::default();
    let mut records = NotifyRecords::new();
    let mut env = TopologyEnv { all_attributes: &[], attributes: &mut attributes, key_of: &key_of, records: &mut records, next_key: 1 << 50 };
    let mut b = read_brep_from_file(k, Some(data), false, false, &mut env).unwrap();
    b.refresh_cached_topology();
    b
}

fn solidcraft_kernel() -> Kernel {
    let mut k = Kernel::default();
    k.fixes.analytic_copy_keeps_parameterization = true;
    k.fixes.analytic_corner_failure_is_false = true;
    k.fixes.analytic_inversion_answers_apart = true;
    k.fixes.approximate_surface_bezier_starter = true;
    k.fixes.approximate_surface_checks_every_row = true;
    k.fixes.ascii_doubles_round_trip = true;
    k.fixes.bend_volume_bbox_contains_wide_angle_and_inf_u = true;
    k.fixes.bend_volume_trim_param_box_across_period = true;
    k.fixes.best_fit_line_largest_scatter_eigenvector = true;
    k.fixes.boolean2d_plane_covers_far_faces = true;
    k.fixes.boolean_cleanup_removes_slits = true;
    k.fixes.boolean_coincident_strip_drops_crossing = true;
    k.fixes.boolean_lists_virtual_surface_copies = true;
    k.fixes.boolean_new_face_closure_both_sides = true;
    k.fixes.boolean_outer_shell_first = true;
    k.fixes.boolean_skips_faces_apart = true;
    k.fixes.boolean_splits_definition_across_split = true;
    k.fixes.boolean_temporary_transform_statuses = true;
    k.fixes.boolean_union_refuses_solid_leak = true;
    k.fixes.boolean_vertex_by_best_crossing = true;
    k.fixes.boolean_wire_edge_keeps_faceuse_shell = true;
    k.fixes.box_face_failures_propagate = true;
    k.fixes.brep_area_between_solid_and_void = true;
    k.fixes.brep_edit_statuses_pass_on = true;
    k.fixes.brep_ray_classifies_current_trims = true;
    k.fixes.bspline_check_min_radius = true;
    k.fixes.bspline_simple_offset_matches_parameterization = true;
    k.fixes.bspline_trim_keeps_rational_end_spans = true;
    k.fixes.bspline_trim_preserves_uv_spans = true;
    k.fixes.check_loop_projected_seam_ends = true;
    k.fixes.check_loop_vertex_tolerance = true;
    k.fixes.check_seam_crossing_unclosed = true;
    k.fixes.circle_offset_antiparallel_normal = true;
    k.fixes.circle_silhouette_perspective_center = true;
    k.fixes.circumscribe_axis_map_by_basis = true;
    k.fixes.classify_edge_interval = true;
    k.fixes.close_loop_relates_only_coincident_edges = true;
    k.fixes.closed_smooth_pole_tangent_limit = true;
    k.fixes.composite_curve_bbox_segment_intervals = true;
    k.fixes.cone_create_canonical_preserves_inside_out_and_checks_height = true;
    k.fixes.cone_iso_circle_at_axial_height = true;
    k.fixes.cone_parallel_line_domains = true;
    k.fixes.cone_parallel_line_midpoint = true;
    k.fixes.cone_unbounded_parameterized_as_nurb = true;
    k.fixes.conic_evaluate_uses_computed_second_deriv = true;
    k.fixes.corner_blend_coons_patch_tangent_to_sides = true;
    k.fixes.corner_blend_stays_inside_neighbours = true;
    k.fixes.corner_patch_face_side_twist = true;
    k.fixes.cover_refuses_folded_continuation = true;
    k.fixes.cover_wraps_across_closed_seam = true;
    k.fixes.cpoly_remove_poly_face_check_member = true;
    k.fixes.crv_in_volume_mults_by_break_index = true;
    k.fixes.curve_bounded_surface_read_uv_type_and_implicit_outer = true;
    k.fixes.curve_check_zero_speed_tangent_end = true;
    k.fixes.curve_contacts_at_curve_ends = true;
    k.fixes.curve_loop_normal_from_area = true;
    k.fixes.curve_loop_start_after_previous_end = true;
    k.fixes.curve_max_distance_single_sample_midpoint = true;
    k.fixes.curve_property_gs_branch_bounds = true;
    k.fixes.curve_sweep_caps_at_sweep_ends = true;
    k.fixes.curve_sweep_closed_path_joins_ends = true;
    k.fixes.curve_tight_box_interval_knots = true;
    k.fixes.cylinder_plane_section_trims_exact = true;
    k.fixes.cylinder_plane_tangent_within_tolerance = true;
    k.fixes.cylinder_rebuild_mean_radius = true;
    k.fixes.database_io_byte_swaps_read_boolean_and_u16 = true;
    k.fixes.defeature_refuses_open_base = true;
    k.fixes.degen_param_tiebreak_uses_actual_param = true;
    k.fixes.degenerate_proto_edge_in_one_vertex = true;
    k.fixes.delete_vertex_joins_seam_iso_curve = true;
    k.fixes.drop_curve_v_domain_degeneracy = true;
    k.fixes.dupin_bottom_hyperbola_axis = true;
    k.fixes.edge_check_orders_tangent_faces = true;
    k.fixes.edge_reversal_keeps_failed_edge = true;
    k.fixes.edgeuse_convexity_propagates_step_off_failure = true;
    k.fixes.ellipse_adjust_step_interval_keeps_turns = true;
    k.fixes.ellipse_convert_to_2d_uses_both_axes = true;
    k.fixes.ellipse_open_arc_keeps_end_angle = true;
    k.fixes.ellipse_step_interval_checks = true;
    k.fixes.ellipse_trim_seam_lines_span_cone_height = true;
    k.fixes.extent3d_contains_both_segment_endpoints = true;
    k.fixes.extrusion_origin_scaled_zero = true;
    k.fixes.extrusion_step_uses_clamped_uv = true;
    k.fixes.face_points_span_domain = true;
    k.fixes.face_shrink_bound_on_knot = true;
    k.fixes.face_swap_uv_transposes_trims = true;
    k.fixes.face_tight_box_searches_trim = true;
    k.fixes.face_trim_curves_kept_where_exact = true;
    k.fixes.fillet_3x2_mixed_concave_side_edge = true;
    k.fixes.fillet_3x2_tangent_side_edge_roles = true;
    k.fixes.fillet_blend_curve_revolved_on_arc = true;
    k.fixes.fillet_chamfer_cone_on_arc = true;
    k.fixes.fillet_chamfer_cone_stored_as_cone = true;
    k.fixes.fillet_chamfer_corner = true;
    k.fixes.fillet_cleanup_keeps_target_faces = true;
    k.fixes.fillet_cleanup_removes_input_slits = true;
    k.fixes.fillet_cliff_overflow = true;
    k.fixes.fillet_corner_ball_center_from_axes = true;
    k.fixes.fillet_corner_ball_patch = true;
    k.fixes.fillet_corner_face_curve_halves = true;
    k.fixes.fillet_corner_per_vertex_fan = true;
    k.fixes.fillet_corner_sphere_from_four_arcs = true;
    k.fixes.fillet_corner_statuses = true;
    k.fixes.fillet_corner_variable_radius_rail_on_opposite_fillet = true;
    k.fixes.fillet_edge_curve_domain_statuses = true;
    k.fixes.fillet_exact_arc_sections = true;
    k.fixes.fillet_exact_arc_spans = true;
    k.fixes.fillet_exact_round_classes = true;
    k.fixes.fillet_gn_section_generator = true;
    k.fixes.fillet_half_degree_crease = true;
    k.fixes.fillet_knife_edge_rounds = true;
    k.fixes.fillet_mitre_concave_face_corners = true;
    k.fixes.fillet_mitre_gap_patch = true;
    k.fixes.fillet_nx1_end_rib_global_merge = true;
    k.fixes.fillet_nx1_opposite_continuation_side_edge = true;
    k.fixes.fillet_nx1_tangent_corner_edge = true;
    k.fixes.fillet_nx1_tangent_side_edge = true;
    k.fixes.fillet_overflow_crease_along_edge = true;
    k.fixes.fillet_overflow_trim_at_creases = true;
    k.fixes.fillet_patch_keeps_fillet_edges = true;
    k.fixes.fillet_point_step_per_point = true;
    k.fixes.fillet_rail_end_on_continued_surface = true;
    k.fixes.fillet_rail_rim_on_face = true;
    k.fixes.fillet_rail_trim_at_near_knot = true;
    k.fixes.fillet_rails_meet_at_one_end_point = true;
    k.fixes.fillet_refuse_ball_not_fitting = true;
    k.fixes.fillet_refuse_crossing_rails = true;
    k.fixes.fillet_refuse_face_consuming_blends = true;
    k.fixes.fillet_refuse_mixed_corner_crossing_rails = true;
    k.fixes.fillet_refuse_non_manifold_result = true;
    k.fixes.fillet_refuse_overlapping_bands = true;
    k.fixes.fillet_refuse_rails_leaving_faces = true;
    k.fixes.fillet_removal_takes_ring_fillets = true;
    k.fixes.fillet_retry_radius_as_first = true;
    k.fixes.fillet_round_as_definition = true;
    k.fixes.fillet_round_ends_at_tangent_continuation = true;
    k.fixes.fillet_run_on_bspline_trims = true;
    k.fixes.fillet_run_on_past_far_end_only = true;
    k.fixes.fillet_section_end_trim_at_creases = true;
    k.fixes.fillet_setback_beside_curved_faces = true;
    k.fixes.fillet_split_edge_keeps_near_half = true;
    k.fixes.fillet_statuses_pass_on = true;
    k.fixes.fillet_surface_plane_curve_on_plane_equation = true;
    k.fixes.fillet_tangent_point_one_sided_step = true;
    k.fixes.fillet_torus_center_on_axis = true;
    k.fixes.fillet_torus_rail_on_axis = true;
    k.fixes.fillet_torus_span_from_start_section = true;
    k.fixes.fillet_trace_from_tangent_point_toward_next = true;
    k.fixes.fillet_trace_reaches_rails = true;
    k.fixes.fillet_trace_rejects_short_step = true;
    k.fixes.fillet_trim_inside_by_best_dot = true;
    k.fixes.fillet_trim_status = true;
    k.fixes.fillet_variable_span_follows_fillet = true;
    k.fixes.fillet_vertex_error_diagnostics = true;
    k.fixes.flat_corners_test_bits = true;
    k.fixes.found_torus_sphere_keep_tested_nurb = true;
    k.fixes.gap_samples_non_bspline_v_stride = true;
    k.fixes.gap_samples_use_trimmed_spans = true;
    k.fixes.geometric_g2_continuity = true;
    k.fixes.global_solver_identical_prefers_larger_when_maximizing = true;
    k.fixes.glue_vertices_joins_every_region = true;
    k.fixes.good_drop_result_distances_in_order = true;
    k.fixes.heal_stage3_failures_propagate = true;
    k.fixes.imprint_extends_edge_to_proto_vertex = true;
    k.fixes.imprint_extension_keeps_analytic_curve = true;
    k.fixes.imprint_reuses_coincident_face_edge = true;
    k.fixes.imprint_skips_collapsed_definition = true;
    k.fixes.imprint_trims_edge_to_proto_vertex = true;
    k.fixes.integrator_propagates_failure = true;
    k.fixes.internal_pole_row_through_knot = true;
    k.fixes.is_iso_curve_at_propagates_canonical_failure = true;
    k.fixes.is_point_on_curve_propagates_eval_failure = true;
    k.fixes.is_topological_vertex_rejects_cusps_and_tangent_errors = true;
    k.fixes.iso_curve_sliver_domain_is_degenerate = true;
    k.fixes.iso_curve_uv_line_both_params = true;
    k.fixes.join_touching_loops = true;
    k.fixes.keep_exact_pcurves = true;
    k.fixes.keep_operand_uv_trims = true;
    k.fixes.knot_vector_valid_requires_span = true;
    k.fixes.linear_sweep_arc_is_cylinder = true;
    k.fixes.linear_sweep_arc_to_cylinder_zero = true;
    k.fixes.local_operation_box_holds_new_surfaces = true;
    k.fixes.local_operation_closes_walls_to_nothing = true;
    k.fixes.local_operation_refuses_loose_tangent_edges = true;
    k.fixes.local_operation_step_walls = true;
    k.fixes.local_solve1d_unbounded_newton_step = true;
    k.fixes.local_surface_intersect_guards_max_boundary_retry = true;
    k.fixes.loop_orientation_floor_from_loop_box = true;
    k.fixes.loop_use_reports_open_gaps = true;
    k.fixes.loops_go_on_round_the_vertex = true;
    k.fixes.mark_lock_pair_advances_and_locks_second_context = true;
    k.fixes.mirror_curve_keeps_euclidean_control_points = true;
    k.fixes.new_edge_vertex_tolerance_covers_gap = true;
    k.fixes.nurb_cylinder_inside_out_frame = true;
    k.fixes.offset_analytic_mate_edges = true;
    k.fixes.offset_clip_right_side_out = true;
    k.fixes.offset_closed_curve_trim_arc = true;
    k.fixes.offset_closes_loop_gaps = true;
    k.fixes.offset_collapsed_rounds_mated = true;
    k.fixes.offset_edges_keep_source_tolerance = true;
    k.fixes.offset_excess_faces_after_merge = true;
    k.fixes.offset_extended_edges_mated = true;
    k.fixes.offset_extension_as_itself = true;
    k.fixes.offset_extension_from_base = true;
    k.fixes.offset_four_face_vertex_split = true;
    k.fixes.offset_keep_cavity_regions = true;
    k.fixes.offset_missing_surface_fails = true;
    k.fixes.offset_open_edge_second_trim_point = true;
    k.fixes.offset_point_vertex_caps = true;
    k.fixes.offset_resolve_crossing_loops = true;
    k.fixes.offset_splits_corner_vertices = true;
    k.fixes.offset_stitch_at_body_tolerance = true;
    k.fixes.offset_swap_uv_extended_distance = true;
    k.fixes.offset_tangent_gap_statuses = true;
    k.fixes.offset_updates_result_tolerances = true;
    k.fixes.offset_walls_touch_refused = true;
    k.fixes.oscillating_skips_failed_guesses = true;
    k.fixes.periodic_extent_add_empty_interval = true;
    k.fixes.perspective_sphere_offsets = true;
    k.fixes.planar_path_profile_in_path_frame = true;
    k.fixes.planar_path_profile_placement_statuses = true;
    k.fixes.planar_path_sweep_statuses = true;
    k.fixes.planar_path_uncapped_open_profiles = true;
    k.fixes.planar_section_face_tolerance = true;
    k.fixes.plane_first_torus_section_exact = true;
    k.fixes.plane_recognition_origin_after_axes = true;
    k.fixes.plane_reparameterize_moves_origin = true;
    k.fixes.point_class_gap_3d_propagates_no_drop_error = true;
    k.fixes.point_on_start_point_squared_tolerance = true;
    k.fixes.point_set_tight_pbox_projects_points = true;
    k.fixes.point_solve_keeps_triangle_guess = true;
    k.fixes.polar_box_region_to_arc_transposes_domain = true;
    k.fixes.precise_properties_body_length_scale = true;
    k.fixes.primitive_path_curve_statuses = true;
    k.fixes.projected_curve_transform_when_owned = true;
    k.fixes.proto_edge_self_loop_on_vertex_degenerate = true;
    k.fixes.proto_edge_split_upper_keeps_the_old_max = true;
    k.fixes.proto_edge_splits_at_captured_vertices = true;
    k.fixes.proto_vertex_miss_is_false = true;
    k.fixes.query_statuses_pass_on = true;
    k.fixes.refine_cc_range_ends = true;
    k.fixes.remove_slit_loops_where_solids_touch = true;
    k.fixes.revolution_frame_from_profile_origin = true;
    k.fixes.revolution_offset_axis_both_sides = true;
    k.fixes.revolution_round_offset_collapses = true;
    k.fixes.revolve_analytic_profile_faces = true;
    k.fixes.revolve_arc_centre_near_axis_is_sphere = true;
    k.fixes.revolve_arc_touching_axis_is_horn_torus = true;
    k.fixes.revolve_holed_profile_void = true;
    k.fixes.revolve_nested_shells_as_cavities = true;
    k.fixes.revolve_splits_curves_on_axis = true;
    k.fixes.seam_point_solutions_are_one = true;
    k.fixes.sheet_sweep_joins_profile_wires = true;
    k.fixes.shell_edges_on_analytics_exact = true;
    k.fixes.shell_empty_cavity_fails = true;
    k.fixes.shell_inner_shell_of_statuses = true;
    k.fixes.shell_one_construction = true;
    k.fixes.simple_coincidence_crossing_is_point = true;
    k.fixes.slice_coincident_faces_sheet_side = true;
    k.fixes.sphere_from_arcs_common_centre = true;
    k.fixes.ssi_alignment_statuses = true;
    k.fixes.ssi_low_angle_start_on_probation = true;
    k.fixes.ssi_pole_point_keeps_its_tangent = true;
    k.fixes.ssi_proto_topology_statuses_pass_on = true;
    k.fixes.ssi_singular_start_inside_direction = true;
    k.fixes.ssi_split_curves_at_singular_points = true;
    k.fixes.ssi_start_loop_trace_budget = true;
    k.fixes.ssi_tangent_contact_points = true;
    k.fixes.ssi_tangent_curve_tracer = true;
    k.fixes.ssi_trace_tangent_curves_once = true;
    k.fixes.stitch_solid_nests_cavities = true;
    k.fixes.stop_tracking_checks_own_callback = true;
    k.fixes.surface_continuity_checks_last_v_knot = true;
    k.fixes.surface_parallel_to_vector_checks = true;
    k.fixes.surface_point_solver_propagates_bbox_failure = true;
    k.fixes.swap_uv_keeps_analytic_definition = true;
    k.fixes.sweep_frame_on_planar_path_normal = true;
    k.fixes.sweep_statuses_pass_on = true;
    k.fixes.taper_pivot_nearest_corner = true;
    k.fixes.taper_tangent_corner_planes = true;
    k.fixes.taper_wall_closes_at_apex = true;
    k.fixes.temporary_radius_setup_errors = true;
    k.fixes.thicken_planar_side_walls = true;
    k.fixes.topological_vertex_has_only_edge_uses = true;
    k.fixes.topology_failed_evaluations_propagate = true;
    k.fixes.torus_check_reads_v_interval = true;
    k.fixes.torus_check_spindle_apex = true;
    k.fixes.torus_iso_circle_axis_from_midpoint = true;
    k.fixes.torus_keeps_inside_out = true;
    k.fixes.torus_major_radius_from_origin = true;
    k.fixes.torus_signed_gen_curve_interval = true;
    k.fixes.transform_aligned_bbox_image = true;
    k.fixes.transform_model_size_from_body_box = true;
    k.fixes.transform_scales_each_tolerance = true;
    k.fixes.translation_sweep_regularize = true;
    k.fixes.trim_boundary_seam_wrap = true;
    k.fixes.twist_simple_duw_sign = true;
    k.fixes.uv_trim_checks_on_surface = true;
    k.fixes.uv_trim_corner_end_points_only = true;
    k.fixes.uv_trim_joins_drop_pieces = true;
    k.fixes.uv_trim_reversal_on_surface = true;
    k.fixes.validate_tolerances_fixes_reported_gaps = true;
    k.fixes.validation_does_not_make_uv_trims = true;
    k.fixes.volume_discontinuity_checks_vw_unaligned = true;
    k.fixes.volume_lift_third_derivative_terms = true;
    k.fixes.volume_representational_end_continuities = true;
    k.fixes.imprint_fits_edge_end_to_final_vertex = true;
    k
}

#[test]
fn probe_failing_routine() {
    for (tag, tool) in [("exact", "tool36_exact.smb"), ("1e-9", "tool36_1e-9.smb")] {
        let mut k = solidcraft_kernel();
        let a = load(&mut k, "B:/Claude/boolean-13085/smb/cup.smb");
        let b = load(&mut k, &format!("B:/Claude/boolean-13085/smb/{tool}"));
        match boolean(&mut k, a, b, BooleanOperation::Difference) {
            Ok(o) => eprintln!("PROBE [{tag}] ok: faces {}", o.result.face_list().len()),
            Err(e) => eprintln!("PROBE [{tag}] ERR {e:?}\n  display: {e}"),
        }
    }
}

/// PROBE: the default-mode operands (defaults + the three imprint flags) at 1e-8 / 36 deg.
#[test]
fn probe_default_mode_routine() {
    let mut k = Kernel::default();
    k.fixes.imprint_trims_edge_to_proto_vertex = true;
    k.fixes.imprint_extends_edge_to_proto_vertex = true;
    k.fixes.imprint_fits_edge_end_to_final_vertex = true;
    let a = load(&mut k, "B:/Claude/boolean-13085/smb/default1e-8_cup.smb");
    let b = load(&mut k, "B:/Claude/boolean-13085/smb/default1e-8_tool36.smb");
    match boolean(&mut k, a, b, BooleanOperation::Difference) {
        Ok(o) => eprintln!("PROBE [default 1e-8] ok: faces {}", o.result.face_list().len()),
        Err(e) => eprintln!("PROBE [default 1e-8] ERR {e:?}\n  display: {e}"),
    }
}

/// PROBE: the default-mode cup's vertex and edge tolerances (the tangent-point vertices).
#[test]
fn probe_cup_tolerances() {
    let mut k = Kernel::default();
    let b = load(&mut k, "B:/Claude/boolean-13085/smb/default1e-8_cup.smb");
    eprintln!("TOL brep tolerance {:e}", b.tolerance());
    for &v in b.vertex_list() {
        let d = b.vertex(v).unwrap();
        eprintln!("TOL vertex {v:?} at ({:.6}, {:.6}, {:.3}) tol {:e}", d.point.x, d.point.y, d.point.z, d.tolerance);
    }
    for &e in b.edge_list() {
        let d = b.edge(e).unwrap();
        if d.tolerance > 0.0 {
            eprintln!("TOL edge {e:?} tol {:e}", d.tolerance);
        }
    }
}
