//! Dono, 05/11: **as 12 fontes do Text, e só elas.** Sem cinza nenhum, sem a perna do «g» cortada,
//! e cada tamanho é o desenho nativo com as células ampliadas.

use ph2d_pixel_text::carimbo::{Fonte, font_list, recorta_a_tinta};
use ph2d_pixel_text::{BUILTIN, FontChoice, TextMask, TextStyle, builtin_line_height, rasterize};

const AMOSTRA: &str = "Hamburgefonts 0123";

fn mascara(nome: &str, px: u32) -> TextMask {
    let estilo = TextStyle::new(FontChoice::Builtin(nome.to_owned()), px);
    rasterize(AMOSTRA, &estilo).expect("a fonte abre")
}

#[test]
fn the_font_list_is_exactly_the_twelve() {
    let l = font_list();
    assert_eq!(l.len(), 12);
    let mut sem_repetir = l.clone();
    sem_repetir.dedup();
    assert_eq!(l, sem_repetir);
}

#[test]
fn every_font_has_no_grey_cell_at_its_native_size() {
    for (i, b) in BUILTIN.iter().enumerate() {
        let px = builtin_line_height(i).expect("nativa");
        let m = mascara(b.0, px);
        assert!(m.w > 0 && m.h > 0, "{}: sem tinta", b.0);
        assert!(m.missing.is_empty(), "{}: faltam {:?}", b.0, m.missing);
        assert!(
            m.cover.iter().all(|&c| c == 0 || c == u8::MAX),
            "{}: cinza a {px} px",
            b.0
        );
    }
}

#[test]
fn twice_the_native_size_is_every_cell_doubled() {
    for (i, b) in BUILTIN.iter().enumerate() {
        let px = builtin_line_height(i).expect("nativa");
        let (um, dois) = (
            recorta_a_tinta(&mascara(b.0, px)),
            recorta_a_tinta(&mascara(b.0, px * 2)),
        );
        assert_eq!(
            (um.0 * 2, um.1 * 2),
            (dois.0, dois.1),
            "{}: a tinta não dobra",
            b.0
        );
        for y in 0..dois.1 {
            for x in 0..dois.0 {
                let quer = um.2[((y / 2) * um.0 + x / 2) as usize];
                let tem = dois.2[(y * dois.0 + x) as usize];
                assert_eq!(tem, quer, "{}: célula ({x},{y}) a {} px", b.0, px * 2);
            }
        }
    }
}

/// Foto de mesa, 05/11: o corte da descida do Aseprite comia a última linha do «g» na Micro 5.
#[test]
fn the_tail_of_a_descender_is_never_cut() {
    for (i, b) in BUILTIN.iter().enumerate() {
        // O corte só existe no contorno, e o «g» destas duas assenta na linha de base, por desenho.
        if b.3 == 0 || ["Press Start 2P", "Silkscreen"].contains(&b.0) {
            continue;
        }
        let px = builtin_line_height(i).expect("nativa");
        let estilo = TextStyle::new(FontChoice::Builtin(b.0.to_owned()), px);
        let m = rasterize("g", &estilo).expect("abre");
        let fundo = (0..m.h)
            .rev()
            .find(|&y| (0..m.w).any(|x| m.cover[(y * m.w + x) as usize] != 0))
            .expect("tinta");
        assert!(
            i32::try_from(fundo).expect("altura") >= m.baseline,
            "{}: o «g» acaba na linha {fundo}, a base é {}",
            b.0,
            m.baseline
        );
    }
}

#[test]
fn every_font_places_and_measures_like_the_text_tool_needs() {
    for (i, b) in BUILTIN.iter().enumerate() {
        let px = builtin_line_height(i).expect("nativa");
        let f = Fonte::pick(u16::try_from(i).expect("12"), px);
        let (ascent, linha) = f.metrics().expect("métricas");
        assert!(ascent > 0 && linha >= ascent, "{}", b.0);
        let (m, _) = f.place("Ag", (0, 0)).expect("coloca");
        assert!(m.cover.contains(&u8::MAX), "{}: tinta", b.0);
    }
}
