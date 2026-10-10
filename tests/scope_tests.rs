#[test]
fn center_mapping_keeps_zero_at_origin() {
    let center = viboscop::scope::ScopeGeometry::new(10, 6, 1.0, 1.0);
    let point = center.map(0.0, 0.0);
    assert_eq!(point, (5, 3));
}

#[test]
fn vertical_inversion_is_applied() {
    let center = viboscop::scope::ScopeGeometry::new(10, 6, 1.0, 1.0);
    let point = center.map(0.0, 1.0);
    assert_eq!(point, (5, 2));
}

#[test]
fn degenerate_dimensions_are_safe() {
    let center = viboscop::scope::ScopeGeometry::new(0, 0, 1.0, 1.0);
    let point = center.map(1.0, -1.0);
    assert_eq!(point, (0, 0));
}

#[test]
fn trace_history_tracks_real_samples() {
    let mut history = viboscop::scope::TraceHistory::new(8);
    history.push((0.25, 0.5));
    history.push((-0.25, -0.5));
    let snapshot = history.snapshot();
    assert_eq!(snapshot.len(), 2);
    assert_eq!(snapshot[0], (0.25, 0.5));
    assert_eq!(snapshot[1], (-0.25, -0.5));
}

#[test]
fn render_scope_for_known_signal_is_not_blank() {
    let samples = [(0.8, 0.8), (-0.8, -0.8), (0.2, -0.2)];
    let rendered = viboscop::scope::render_scope(&samples, 20, 10, 4.0, 4.0);
    assert!(rendered.contains('●') || rendered.contains('█'));
    assert!(rendered.contains('─') || rendered.contains('│') || rendered.contains('┼'));
}
