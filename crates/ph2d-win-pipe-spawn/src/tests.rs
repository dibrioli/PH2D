use super::command_line;

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn line(program: &str, args: &[&str]) -> String {
    let args: Vec<Vec<u16>> = args.iter().map(|a| w(a)).collect();
    let mut l = command_line(&w(program), &args);
    assert_eq!(l.pop(), Some(0), "termina em NUL");
    String::from_utf16(&l).expect("UTF-16")
}

/// As regras do `CommandLineToArgvW`: o que passa limpo, o que vai entre aspas, as `\` que dobram.
#[test]
fn the_command_line_quotes_by_the_crt_rules() {
    assert_eq!(
        line(
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            &["--headless", "--remote-debugging-io-pipes=1234,5678"]
        ),
        r#""C:\Program Files\Google\Chrome\Application\chrome.exe" --headless --remote-debugging-io-pipes=1234,5678"#
    );
    // Um perfil num utilizador com espaço: entre aspas, as `\` do meio ficam simples.
    assert_eq!(
        line(
            "a.exe",
            &[r"--user-data-dir=C:\Users\Ana Rita\AppData\Local\Temp\ph2d-cdp-1-2"]
        ),
        r#""a.exe" "--user-data-dir=C:\Users\Ana Rita\AppData\Local\Temp\ph2d-cdp-1-2""#
    );
    // `\` antes do fecho das aspas dobra; `\` antes de `"` dobra e escapa a aspa.
    assert_eq!(line("a.exe", &[r"C:\x y\"]), r#""a.exe" "C:\x y\\""#);
    assert_eq!(line("a.exe", &[r#"a\"b"#]), r#""a.exe" "a\\\"b""#);
    assert_eq!(line("a.exe", &[""]), r#""a.exe" """#);
    assert_eq!(
        line("a.exe", &[r"C:\sem\espaco"]),
        r#""a.exe" C:\sem\espaco"#
    );
}
