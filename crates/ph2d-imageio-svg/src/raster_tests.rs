use super::*;

const DISC: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50" viewBox="0 0 10 5"><circle cx="5" cy="2.5" r="2.5" fill="#ff0000"/></svg>"##;

/// O tamanho natural é o do `<svg>`; sem `width`/`height`, o do `viewBox`.
#[test]
fn the_natural_size_is_the_svg_box() {
    assert_eq!(natural_size(&parse(DISC).unwrap()), [100, 50]);
    let vb = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30.2 7"/>"#;
    assert_eq!(natural_size(&parse(vb).unwrap()), [31, 7]);
}

/// ⭐ Nítido a qualquer tamanho: o mesmo SVG a 100 e a 4000 px tem a borda do disco à MESMA fracção
/// da imagem, com UMA faixa de transição de ~1 px (não ampliada 40×).
#[test]
fn it_rasterizes_at_the_asked_size_not_upscaled() {
    let tree = parse(DISC).unwrap();
    for w in [100u32, 4000] {
        let h = w / 2;
        let px = rasterize(&tree, [w, h]).unwrap();
        assert_eq!(px.len(), (w * h * 4) as usize);
        // A linha do meio: vermelho opaco no centro, transparente nos cantos, e só ~2 píxeis de
        // alfa parcial (as duas bordas).
        let row = &px[(h / 2 * w * 4) as usize..][..(w * 4) as usize];
        let mid = &row[(w / 2 * 4) as usize..][..4];
        assert_eq!(mid, [255, 0, 0, 255], "{w}");
        assert_eq!(row[3], 0, "{w}");
        let partial = row.chunks(4).filter(|p| p[3] != 0 && p[3] != 255).count();
        assert!(partial <= 4, "{w}: {partial} píxeis de borda");
    }
}

/// O alfa sai RETO: um vermelho a meia opacidade é vermelho puro com alfa ~128.
#[test]
fn the_alpha_comes_out_straight() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4" fill="#ff0000" fill-opacity="0.5"/></svg>"##;
    let px = rasterize(&parse(svg).unwrap(), [4, 4]).unwrap();
    assert_eq!(px[0], 255);
    assert!((127..=129).contains(&px[3]), "{}", px[3]);
}

/// Um `href` para o disco não se lê (a defesa da porta de importação).
#[test]
fn a_filesystem_href_is_not_read() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="4" height="4"><image width="4" height="4" xlink:href="/etc/hostname"/></svg>"#;
    let px = rasterize(&parse(svg).unwrap(), [4, 4]).unwrap();
    assert!(px.chunks(4).all(|p| p[3] == 0));
}

#[test]
fn the_texts_are_counted() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><text>a</text><g><text>b</text></g></svg>"#;
    assert_eq!(dropped_texts(svg), 2);
    assert_eq!(dropped_texts(DISC), 0);
}

/// Um `.svgz` vira o texto dele; o que não é gzip passa igual; uma bomba acima do tecto não entra.
#[test]
fn a_svgz_becomes_its_text_up_to_the_ceiling() {
    use std::io::Write as _;
    let gz = |b: &[u8]| {
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        e.write_all(b).unwrap();
        e.finish().unwrap()
    };
    assert_eq!(&*plain(&gz(DISC)).unwrap(), DISC);
    assert_eq!(&*plain(DISC).unwrap(), DISC);
    let bomb = gz(&vec![b' '; MAX_ARCHIVE_TEXT_BYTES as usize + 1]);
    assert!(plain(&bomb).is_none());
}

/// ⭐ Auditoria W15 — um SVG ANINHADO num `data:` em gzip passa pelo mesmo tecto (o usvg descomprimia-o
/// sem limite): a bomba não entra e o resto do SVG desenha-se; um `.svgz` de topo acima do tecto é
/// recusado com a frase da defesa.
#[test]
fn a_nested_gzip_data_uri_is_bounded() {
    use base64::Engine as _;
    use std::io::Write as _;
    let mut inner = br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4"/>"#.to_vec();
    inner.resize(MAX_ARCHIVE_TEXT_BYTES as usize + 16, b' ');
    inner.extend_from_slice(b"</svg>");
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    e.write_all(&inner).unwrap();
    let gz = e.finish().unwrap();
    let b64 = base64::engine::general_purpose::STANDARD.encode(&gz);
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="4" height="4"><rect width="2" height="4" fill="#ff0000"/><image width="4" height="4" xlink:href="data:image/svg+xml;base64,{b64}"/></svg>"##
    );
    let px = rasterize(&parse(svg.as_bytes()).unwrap(), [4, 4]).unwrap();
    assert_eq!(&px[..4], [255, 0, 0, 255], "o resto desenha-se");
    assert_eq!(px[3 * 4 + 3], 0, "a bomba não entrou");
    assert!(matches!(parse(&gz), Err(Error::Decode(m)) if m.contains("MAX_ARCHIVE_TEXT_BYTES")));
}
