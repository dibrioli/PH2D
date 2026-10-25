//! Ficheiros de paleta: ida e volta em cada formato que se escreve, e paridade com os leitores do
//! MVP (`mvp_color.json`) — fora dos dois defeitos MEDIDOS dele, que aqui são lei ao contrário.

use ph2d_pixel_io::palette_file::{PaletteError, PaletteFormat, read, read_as, write};
use serde_json::Value;

fn colors() -> Vec<[u8; 4]> {
    vec![
        [0, 0, 0, 255],
        [255, 128, 7, 255],
        [10, 20, 30, 255],
        [255, 255, 255, 255],
    ]
}

#[test]
fn every_writable_format_round_trips_the_colors() {
    for (fmt, ext) in PaletteFormat::WRITABLE {
        let bytes = write(fmt, "Teste", &colors(), Some(2));
        let back = read(&bytes, Some(ext)).expect(ext);
        assert_eq!(back.colors, colors(), "{ext}");
        if fmt == PaletteFormat::Act {
            assert_eq!(back.transparent, Some(2), ".act guarda o transparente");
        }
    }
    let riff = write(PaletteFormat::RiffPal, "x", &colors(), None);
    assert_eq!(
        read(&riff, None).expect("riff").colors,
        colors(),
        "RIFF pelo cabeçalho"
    );
}

#[test]
fn gpl_keeps_alpha_with_the_rgba_channels_header() {
    let c = vec![[1, 2, 3, 128], [4, 5, 6, 255]];
    let bytes = write(PaletteFormat::Gpl, "a", &c, None);
    assert!(String::from_utf8_lossy(&bytes).contains("Channels: RGBA"));
    assert_eq!(read(&bytes, Some("gpl")).expect("gpl").colors, c);
}

#[test]
fn empty_and_colourless_files_fail_loudly() {
    assert_eq!(read(b"  \n", None), Err(PaletteError::Empty));
    assert_eq!(read(b"nada aqui", None), Err(PaletteError::NoColors));
    assert_eq!(read(b"{isso", None), Err(PaletteError::Json));
    assert_eq!(
        read_as(b"nao e gpl", PaletteFormat::Gpl),
        Err(PaletteError::NotThisFormat(PaletteFormat::Gpl))
    );
}

/// O `.pal` JASC do Paint Shop Pro lê-se pelo cabeçalho. ⛔ O MVP farejava-o como lista de hex e
/// tirava `#225555` do `255` — o vetor dele está na fixtura como o defeito que NÃO se copia.
#[test]
fn jasc_pal_is_read_by_its_header_not_as_a_hex_list() {
    let text = b"JASC-PAL\r\n0100\r\n2\r\n255 0 0\r\n0 255 0\r\n";
    let p = read(text, None).expect("jasc");
    assert_eq!(p.colors, vec![[255, 0, 0, 255], [0, 255, 0, 255]]);
}

#[test]
fn readers_match_the_mvp_on_gpl_lospec_and_hex() {
    let f: Value = serde_json::from_str(include_str!(
        "../../ph2d-pixel-core/tests/fixtures/mvp_color.json"
    ))
    .expect("json");
    let cases = f["palette"]["parse"].as_array().expect("parse");
    assert!(cases.len() >= 13, "a fixtura encolheu");
    let mut checked = 0;
    for c in cases {
        let text = c["text"].as_str().expect("t");
        let ok = &c["palette"]["ok"];
        // Os defeitos medidos do MVP: tokens numéricos de 3 dígitos lidos como `#rgb`.
        let mvp_defect = text.starts_with("JASC") || text.contains("300 0 0");
        if ok.is_null() || mvp_defect {
            continue;
        }
        let want: Vec<[u8; 4]> = ok["colors"]
            .as_array()
            .expect("c")
            .iter()
            .map(|v| {
                let a = v.as_array().expect("rgb");
                [0, 1, 2]
                    .map(|i| a[i].as_u64().expect("u") as u8)
                    .into_iter()
                    .chain([255])
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("4")
            })
            .collect();
        let got = read(text.as_bytes(), None).expect(text);
        assert_eq!(got.colors, want, "{text:?}");
        checked += 1;
    }
    assert!(checked >= 6, "só {checked} casos comparados");
}
