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
