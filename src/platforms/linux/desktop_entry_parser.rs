use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use crate::app_entry::{LaunchArg, LaunchArgPart, LaunchCommand};

pub(super) struct DesktopEntry {
    values: HashMap<String, String>,
}

impl DesktopEntry {
    pub(super) fn parse(contents: &str) -> Option<Self> {
        let mut values = HashMap::new();
        let mut in_desktop_entry = false;
        let mut saw_desktop_entry = false;

        for line in contents.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry = line == "[Desktop Entry]";
                if in_desktop_entry {
                    saw_desktop_entry = true;
                }
                continue;
            }

            if !in_desktop_entry {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                values.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        saw_desktop_entry.then_some(Self { values })
    }

    pub(super) fn raw(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub(super) fn string(&self, key: &str) -> Option<String> {
        self.localized_raw(key).map(unescape_desktop_value)
    }

    pub(super) fn boolean(&self, key: &str) -> bool {
        match self.raw(key) {
            Some(value) => value.eq_ignore_ascii_case("true"),
            None => false,
        }
    }

    pub(super) fn string_list(&self, key: &str) -> Option<Vec<String>> {
        self.raw(key).map(split_desktop_string_list)
    }

    pub(super) fn launch_command(
        &self,
        desktop_file: &Path,
        requires_terminal: bool,
    ) -> Option<LaunchCommand> {
        parse_launch_command(
            self.raw("Exec")?,
            desktop_file,
            self.raw("Icon"),
            requires_terminal,
        )
    }

    fn localized_raw(&self, key: &str) -> Option<&str> {
        for localized_key in localized_key_candidates(key) {
            if let Some(value) = self.raw(&localized_key) {
                return Some(value);
            }
        }

        self.raw(key)
    }
}

fn localized_key_candidates(key: &str) -> Vec<String> {
    current_locale_suffixes()
        .into_iter()
        .map(|locale| format!("{key}[{locale}]"))
        .collect()
}

fn current_locale_suffixes() -> Vec<String> {
    let mut suffixes = Vec::new();

    if let Some(language) = env::var("LANGUAGE").ok().filter(|value| !value.is_empty()) {
        for locale in language.split(':').filter(|locale| !locale.is_empty()) {
            suffixes.extend(locale_suffixes(locale));
        }
    }

    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Some(locale) = env::var(key).ok().filter(|value| !value.is_empty()) {
            suffixes.extend(locale_suffixes(&locale));
            break;
        }
    }

    dedupe_strings(suffixes)
}

fn locale_suffixes(locale: &str) -> Vec<String> {
    let (locale, modifier) = locale
        .split_once('@')
        .map(|(locale, modifier)| (locale, Some(modifier)))
        .unwrap_or((locale, None));
    let locale = locale
        .split_once('.')
        .map(|(locale, _)| locale)
        .unwrap_or(locale)
        .replace('-', "_");

    if locale.is_empty() || locale == "C" || locale == "POSIX" {
        return Vec::new();
    }

    let language = locale
        .split_once('_')
        .map(|(language, _)| language)
        .unwrap_or(&locale);
    let mut suffixes = Vec::new();

    if let Some(modifier) = modifier.filter(|modifier| !modifier.is_empty()) {
        suffixes.push(format!("{locale}@{modifier}"));
    }

    suffixes.push(locale.clone());

    if language != locale {
        if let Some(modifier) = modifier.filter(|modifier| !modifier.is_empty()) {
            suffixes.push(format!("{language}@{modifier}"));
        }

        suffixes.push(language.to_string());
    }

    dedupe_strings(suffixes)
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    values.into_iter().fold(Vec::new(), |mut deduped, value| {
        if !deduped.contains(&value) {
            deduped.push(value);
        }

        deduped
    })
}

fn parse_launch_command(
    exec: &str,
    desktop_file: &Path,
    icon: Option<&str>,
    requires_terminal: bool,
) -> Option<LaunchCommand> {
    let exec = unescape_desktop_value(exec);
    let mut tokens = split_exec_command(&exec)?;
    let executable = tokens
        .first()
        .and_then(|token| resolve_executable_token(&token.value))
        .map(PathBuf::from)?;
    let args = tokens
        .drain(1..)
        .map(|token| launch_arg_from_exec_token(token, desktop_file, icon))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();

    Some(LaunchCommand {
        executable,
        args,
        requires_terminal,
    })
}

fn launch_arg_from_exec_token(
    token: ExecToken,
    desktop_file: &Path,
    icon: Option<&str>,
) -> Option<Vec<LaunchArg>> {
    match token.value.as_str() {
        "%f" => Some(vec![LaunchArg::File]),
        "%F" => Some(vec![LaunchArg::Files]),
        "%u" => Some(vec![LaunchArg::Url]),
        "%U" => Some(vec![LaunchArg::Urls]),
        "%i" => Some(
            icon.filter(|value| !value.is_empty())
                .map(|value| vec![LaunchArg::Icon(unescape_desktop_value(value))])
                .unwrap_or_default(),
        ),
        "%c" => Some(vec![LaunchArg::AppName]),
        "%k" => Some(vec![LaunchArg::DesktopFile(desktop_file.to_path_buf())]),
        _ => launch_arg_template_from_exec_token(&token, desktop_file),
    }
}

fn launch_arg_template_from_exec_token(
    token: &ExecToken,
    desktop_file: &Path,
) -> Option<Vec<LaunchArg>> {
    let parts = resolve_token_parts(&token.value, desktop_file)?;

    if parts.is_empty() {
        return Some(Vec::new());
    }

    if parts.len() == 1
        && let LaunchArgPart::Literal(value) = &parts[0]
    {
        return Some(vec![LaunchArg::Literal(value.clone())]);
    }

    if token.was_quoted {
        return None;
    }

    Some(vec![LaunchArg::Template(parts)])
}

fn resolve_token_parts(token: &str, desktop_file: &Path) -> Option<Vec<LaunchArgPart>> {
    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut chars = token.chars();

    while let Some(ch) = chars.next() {
        if ch != '%' {
            literal.push(ch);
            continue;
        }

        match chars.next() {
            Some('%') => literal.push('%'),
            Some('d' | 'D' | 'n' | 'N' | 'v' | 'm') => {}
            Some('f') => {
                push_literal_part(&mut parts, &mut literal);
                parts.push(LaunchArgPart::File);
            }
            Some('u') => {
                push_literal_part(&mut parts, &mut literal);
                parts.push(LaunchArgPart::Url);
            }
            Some('c') => {
                push_literal_part(&mut parts, &mut literal);
                parts.push(LaunchArgPart::AppName);
            }
            Some('k') => {
                push_literal_part(&mut parts, &mut literal);
                parts.push(LaunchArgPart::DesktopFile(desktop_file.to_path_buf()));
            }
            Some('F' | 'U' | 'i') => return None,
            Some(_) => return None,
            None => literal.push('%'),
        }
    }

    push_literal_part(&mut parts, &mut literal);
    Some(parts)
}

fn push_literal_part(parts: &mut Vec<LaunchArgPart>, literal: &mut String) {
    if !literal.is_empty() {
        parts.push(LaunchArgPart::Literal(std::mem::take(literal)));
    }
}

fn split_exec_command(exec: &str) -> Option<Vec<ExecToken>> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut in_quotes = false;
    let mut token_was_quoted = false;
    let mut escaped = false;

    for ch in exec.chars() {
        if escaped {
            token.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => {
                in_quotes = !in_quotes;
                token_was_quoted = true;
            }
            ch if ch.is_whitespace() && !in_quotes => {
                if !token.is_empty() || token_was_quoted {
                    tokens.push(ExecToken {
                        value: std::mem::take(&mut token),
                        was_quoted: token_was_quoted,
                    });
                    token_was_quoted = false;
                }
            }
            ch => token.push(ch),
        }
    }

    if escaped {
        token.push('\\');
    }

    if in_quotes {
        return None;
    }

    if !token.is_empty() || token_was_quoted {
        tokens.push(ExecToken {
            value: token,
            was_quoted: token_was_quoted,
        });
    }

    Some(tokens)
}

fn resolve_executable_token(token: &str) -> Option<String> {
    let parts = resolve_token_parts(token, Path::new(""))?;

    if parts.is_empty() {
        return None;
    }

    parts
        .into_iter()
        .map(|part| match part {
            LaunchArgPart::Literal(value) => Some(value),
            _ => None,
        })
        .collect()
}

fn split_desktop_string_list(value: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut item = String::new();
    let mut chars = value.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some(';') => item.push(';'),
                Some('s') => item.push(' '),
                Some('n') => item.push('\n'),
                Some('t') => item.push('\t'),
                Some('r') => item.push('\r'),
                Some('\\') => item.push('\\'),
                Some(other) => item.push(other),
                None => item.push('\\'),
            }
            continue;
        }

        if ch == ';' {
            items.push(std::mem::take(&mut item));
        } else {
            item.push(ch);
        }
    }

    if !item.is_empty() {
        items.push(item);
    }

    items
}

fn unescape_desktop_value(value: &str) -> String {
    let mut unescaped = String::new();
    let mut chars = value.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            unescaped.push(ch);
            continue;
        }

        match chars.next() {
            Some('s') => unescaped.push(' '),
            Some('n') => unescaped.push('\n'),
            Some('t') => unescaped.push('\t'),
            Some('r') => unescaped.push('\r'),
            Some('\\') => unescaped.push('\\'),
            Some(other) => unescaped.push(other),
            None => unescaped.push('\\'),
        }
    }

    unescaped
}

struct ExecToken {
    value: String,
    was_quoted: bool,
}

#[cfg(test)]
mod tests {
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
        let launch =
            parse_launch_command("app old=%d", Path::new("app.desktop"), None, false).unwrap();

        assert_eq!(launch.args, vec![LaunchArg::Literal("old=".to_string())]);
    }

    #[test]
    fn unescapes_values_before_splitting_exec_command() {
        let launch =
            parse_launch_command(r#"app "two\swords""#, Path::new("app.desktop"), None, false)
                .unwrap();

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
}
