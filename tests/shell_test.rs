use wmux::terminal::shell::detect_shell;

#[test]
fn cli_flag_takes_precedence() {
    let result = detect_shell(Some("C:\\custom\\shell.exe".into()));
    assert_eq!(result, "C:\\custom\\shell.exe");
}

#[test]
fn falls_back_to_powershell() {
    let result = detect_shell(None);
    assert!(!result.is_empty());
}
