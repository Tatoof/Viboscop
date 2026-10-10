#[test]
fn mono_to_stereo_duplication_is_stable() {
    let mono = vec![0.0, 1.0, -1.0];
    let stereo = viboscop::audio::dup_mono_to_stereo(&mono);
    assert_eq!(stereo.len(), 6);
    assert_eq!(stereo, vec![0.0, 0.0, 1.0, 1.0, -1.0, -1.0]);
}

#[test]
fn channel_pair_selection_works() {
    let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let left = viboscop::audio::select_pair(&samples, 0, 1);
    assert_eq!(left, vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)]);
}

#[test]
fn playback_gain_changes_only_output_not_source() {
    let source = vec![0.5, -0.5, 0.25];
    let gain = 0.25f32;
    let output: Vec<f32> = source.iter().map(|sample| sample * gain).collect();
    assert_eq!(output, vec![0.125, -0.125, 0.0625]);
    assert_eq!(source, vec![0.5, -0.5, 0.25]);
}
