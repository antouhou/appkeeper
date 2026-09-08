use super::*;

#[test]
fn parses_exec_field_codes_into_launch_args() {
    let launch = parse_launch_command(
        "firefox --new-window %u %i %c %k %%",
        Path::new("/usr/share/applications/firefox.desktop"),
        Some("firefox"),
        false,
    )
    .expect("exec command should parse");

    assert_eq!(launch.executable, PathBuf::from("firefox"));
    assert_eq!(
        launch.args,
        vec![
            LaunchArg::Literal("--new-window".to_string()),
            LaunchArg::Url,
            LaunchArg::Icon("firefox".to_string()),
            LaunchArg::AppName,
            LaunchArg::DesktopFile(PathBuf::from("/usr/share/applications/firefox.desktop")),
            LaunchArg::Literal("%".to_string()),
        ]
    );
    assert!(!launch.requires_terminal);
}

#[test]
fn parses_compound_single_argument_field_codes() {
    let launch = parse_launch_command(
        "app --open=%u name=%c",
        Path::new("/usr/share/applications/app.desktop"),
        None,
        false,
    )
    .expect("exec command should parse");

    assert_eq!(
        launch.args,
        vec![
            LaunchArg::Template(vec![
                LaunchArgPart::Literal("--open=".to_string()),
                LaunchArgPart::Url,
            ]),
            LaunchArg::Template(vec![
                LaunchArgPart::Literal("name=".to_string()),
                LaunchArgPart::AppName,
            ]),
        ]
    );
}

#[test]
fn rejects_unknown_field_codes() {
    assert!(parse_launch_command("app %x", Path::new("app.desktop"), None, false).is_none());
}

#[test]
fn rejects_standalone_only_codes_inside_literals() {
    assert!(
        parse_launch_command("app --files=%F", Path::new("app.desktop"), None, false).is_none()
    );
    assert!(
        parse_launch_command(
            "app --icon=%i",
            Path::new("app.desktop"),
            Some("app"),
            false
        )
        .is_none()
    );
}

#[test]
fn removes_deprecated_field_codes() {
    let launch = parse_launch_command("app old=%d", Path::new("app.desktop"), None, false).unwrap();

    assert_eq!(launch.args, vec![LaunchArg::Literal("old=".to_string())]);
}

#[test]
fn unescapes_values_before_splitting_exec_command() {
    let launch =
        parse_launch_command(r#"app "two\swords""#, Path::new("app.desktop"), None, false).unwrap();

    assert_eq!(
        launch.args,
        vec![LaunchArg::Literal("two words".to_string())]
    );
}

#[test]
fn splits_exec_command_with_quotes_and_escapes() {
    let tokens = split_exec_command(r#"app "two words" escaped\ space"#).unwrap();

    assert_eq!(
        tokens
            .into_iter()
            .map(|token| token.value)
            .collect::<Vec<_>>(),
        vec!["app", "two words", "escaped space"]
    );
}

#[test]
fn splits_string_lists_with_escaped_semicolons() {
    assert_eq!(
        split_desktop_string_list(r#"Utility;Semi\;Colon;"#),
        vec!["Utility", "Semi;Colon"]
    );
}

#[test]
fn builds_locale_suffix_fallbacks() {
    assert_eq!(
        locale_suffixes("pt_BR.UTF-8@latin"),
        vec!["pt_BR@latin", "pt_BR", "pt@latin", "pt"]
    );
}
