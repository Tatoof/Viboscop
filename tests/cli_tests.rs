#[test]
fn cli_accepts_theme_and_path() {
    let args = viboscop::cli::parse_args(["viboscop", "--theme", "neon", "song.flac"])
        .expect("valid arguments should parse");
    assert_eq!(args.theme, viboscop::cli::Theme::Neon);
    assert_eq!(args.path.to_string_lossy(), "song.flac");
}

#[test]
fn cli_rejects_invalid_theme() {
    let err = viboscop::cli::parse_args(["viboscop", "--theme", "bad", "song.flac"]);
    assert!(err.is_err());
}
