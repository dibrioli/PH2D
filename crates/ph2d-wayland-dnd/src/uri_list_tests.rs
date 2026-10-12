use super::*;

fn paths(list: &UriList) -> Vec<&str> {
    list.paths.iter().map(|p| p.to_str().unwrap()).collect()
}

#[test]
fn both_line_breaks_give_the_same_files() {
    let crlf = parse(b"file:///home/a.png\r\nfile:///home/b.png\r\n");
    let lf = parse(b"file:///home/a.png\nfile:///home/b.png\n");
    assert_eq!(paths(&crlf), ["/home/a.png", "/home/b.png"]);
    assert_eq!(crlf, lf);
}

#[test]
fn a_last_line_without_a_break_still_counts() {
    assert_eq!(
        paths(&parse(b"file:///a.png\nfile:///b.png")),
        ["/a.png", "/b.png"]
    );
}

#[test]
fn comments_and_blank_lines_are_skipped_and_never_counted_as_refused() {
    let list = parse(b"# from Dolphin\r\n\r\nfile:///a.png\r\n#file:///not.png\r\n");
    assert_eq!(paths(&list), ["/a.png"]);
    assert_eq!(list.not_files, 0);
}

#[test]
fn percent_escapes_decode_to_the_bytes_of_the_name() {
    let list = parse(b"file:///home/enio/Imagens/fo%20to%20%C3%A1gua.png\n");
    assert_eq!(paths(&list), ["/home/enio/Imagens/fo to água.png"]);
}

#[cfg(unix)]
#[test]
fn a_name_that_is_not_utf8_survives_byte_for_byte() {
    use std::os::unix::ffi::OsStrExt;
    let list = parse(b"file:///tmp/%FF.png\n");
    assert_eq!(list.paths[0].as_os_str().as_bytes(), b"/tmp/\xFF.png");
}

#[test]
fn the_host_is_dropped() {
    let list = parse(b"file://localhost/a.png\nfile://estudio/b.png\nFILE:///c.png\nfile:/d.png\n");
    assert_eq!(paths(&list), ["/a.png", "/b.png", "/c.png", "/d.png"]);
}

#[test]
fn what_is_not_a_local_file_is_counted_as_refused() {
    let list =
        parse(b"https://example.org/a.png\nfile://host-only\nfile:relative.png\nfile:///ok.png\n");
    assert_eq!(paths(&list), ["/ok.png"]);
    assert_eq!(list.not_files, 3);
}

#[test]
fn a_trailing_nul_is_not_part_of_the_name() {
    assert_eq!(paths(&parse(b"file:///a.png\r\n\0")), ["/a.png"]);
    assert_eq!(paths(&parse(b"file:///a.png\0")), ["/a.png"]);
}

/// W18 — o link de um vídeo largado do navegador: não é ficheiro (conta), e o endereço vem na lista dos
/// links da web; um `mailto:` não.
#[test]
fn web_links_are_kept_beside_the_count() {
    let l = parse(b"https://www.youtube.com/watch?v=aqz-KE-bpKQ\r\nmailto:a@b.c\r\nfile:///tmp/a.png\r\n");
    assert_eq!(l.not_files, 2);
    assert_eq!(l.links, vec!["https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_owned()]);
    assert_eq!(l.paths.len(), 1);
}
