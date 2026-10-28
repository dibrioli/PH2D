//! `.msk` (Select ▸ Load/Save MSK file) contra o LibreSprite 1.3 corrido como oráculo
//! (`tests/fixtures/oracle_msk.json`, gerada por `docs/PixelArt/ferramentas/oraculo_msk.py`; o
//! formato e os desvios em `docs/PixelArt/ferramentas/oraculo_msk.md`).
//!
//! `clear_rows` é a selecção que o app TINHA (Edit ▸ Clear sobre a imagem opaca): prova
//! independente do ficheiro.

use ph2d_pixel_core::Rect;
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_io::msk::{LEGACY_LEN, MaskFile, MskError, read_msk, write_msk};
use serde_json::Value;

/// O caso em que o Clear do ORÁCULO erra: carrega certo (a regravação dá os bits de entrada) mas
/// não apaga nada quando a máscara não-retangular começa em x/y negativos. (O legado lido 6 bytes
/// adiante é o outro defeito: tem teste próprio.)
const ORACLE_DEFECTS: [&str; 1] = ["r_negative_origin"];

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/oracle_msk.json")).expect("fixtura")
}

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
        .collect()
}

fn cases() -> Vec<Value> {
    fixture()["cases"].as_array().expect("cases").clone()
}

fn name(c: &Value) -> &str {
    c["name"].as_str().unwrap()
}

fn is_defect(c: &Value) -> bool {
    ORACLE_DEFECTS.contains(&name(c))
}

fn canvas(c: &Value) -> (u32, u32) {
    let v = &c["canvas"];
    (v[0].as_u64().unwrap() as u32, v[1].as_u64().unwrap() as u32)
}

/// A selecção que o oráculo tinha na tela, das linhas do Clear.
fn cleared(c: &Value) -> Mask {
    let (w, h) = canvas(c);
    let mut m = Mask::empty(w, h);
    for (y, row) in c["clear_rows"].as_array().unwrap().iter().enumerate() {
        for (x, ch) in row.as_str().unwrap().bytes().enumerate() {
            m.set(x as i32, y as i32, ch == b'1');
        }
    }
    m
}

fn oracle_msk(c: &Value) -> Option<Vec<u8>> {
    c["oracle_msk_hex"].as_str().map(hex)
}

/// Cada ficheiro que o oráculo GRAVOU lê-se na selecção que ele tinha na tela.
#[test]
fn msk_reading_every_oracle_file_gives_the_oracle_selection() {
    let mut n = 0;
    for c in cases() {
        let Some(bytes) = oracle_msk(&c) else {
            continue;
        };
        if is_defect(&c) {
            continue;
        }
        let (w, h) = canvas(&c);
        let file = read_msk(&bytes).unwrap_or_else(|e| panic!("{}: {e:?}", name(&c)));
        assert_eq!(file.to_selection(w, h), cleared(&c), "{}", name(&c));
        n += 1;
    }
    assert!(n >= 25, "a fixtura encolheu: {n}");
}

/// Escrita byte a byte: a selecção da tela, cortada à caixa justa, dá o ficheiro do oráculo; e
/// todo ficheiro do oráculo (também os que saem da tela) regrava-se igual.
#[test]
fn msk_writing_is_byte_exact_against_every_oracle_file() {
    let mut from_canvas = 0;
    for c in cases() {
        let Some(bytes) = oracle_msk(&c) else {
            continue;
        };
        let file = read_msk(&bytes).expect("lê");
        assert_eq!(write_msk(&file), bytes, "regravar {}", name(&c));
        let (w, h) = canvas(&c);
        if !is_defect(&c) && file.bounds().intersect(Rect::new(0, 0, w, h)) == file.bounds() {
            let sel = MaskFile::from_selection(&cleared(&c)).expect("há selecção");
            assert_eq!(write_msk(&sel), bytes, "da tela {}", name(&c));
            from_canvas += 1;
        }
    }
    assert!(from_canvas >= 20, "{from_canvas}");
}

/// Os ficheiros que FABRICÁMOS: o que o oráculo carrega dá a mesma selecção e a mesma regravação;
/// o que ele recusa («Error loading .msk file») ou onde ele fica PARADO é erro aqui.
#[test]
fn msk_crafted_inputs_load_like_the_oracle() {
    let mut refused = 0;
    for c in cases().into_iter().filter(|c| c["direction"] == "read") {
        let input = hex(c["input_msk_hex"].as_str().unwrap());
        let n = name(&c);
        if c["load"] != "loaded" {
            assert!(
                read_msk(&input).is_err(),
                "o oráculo deu {} em {n}",
                c["load"]
            );
            refused += 1;
            continue;
        }
        if n.starts_with("r_legacy") {
            continue;
        }
        let file = read_msk(&input).unwrap_or_else(|e| panic!("{n}: {e:?}"));
        let (w, h) = canvas(&c);
        match oracle_msk(&c) {
            Some(resaved) => {
                if !is_defect(&c) {
                    assert_eq!(file.to_selection(w, h), cleared(&c), "{n}");
                }
                let tight = MaskFile::from_selection(&file.mask).map(|m| MaskFile {
                    x: file.x + m.x,
                    y: file.y + m.y,
                    mask: m.mask,
                });
                assert_eq!(tight.map(|t| write_msk(&t)), Some(resaved), "regravado {n}");
            }
            // Sem selecção o Save do oráculo dá erro (e o Clear apaga a tela toda).
            None => assert!(file.mask.is_empty(), "{n}: sem regravação só se vazio"),
        }
    }
    assert!(refused >= 8, "{refused}");
}

/// O legado de 8000 bytes lê-se da ORIGEM; o oráculo lê-o 6 bytes adiante e enche o fim de `1`s
/// (o defeito medido, que a fixtura guarda e que aqui se prova ser só esse deslocamento).
#[test]
fn msk_legacy_8000_reads_from_the_origin_not_six_bytes_ahead() {
    let mut n = 0;
    for c in cases()
        .into_iter()
        .filter(|c| name(c).starts_with("r_legacy_8000"))
    {
        if c["load"] != "loaded" || c["input_msk_hex"].as_str().unwrap().len() != 2 * LEGACY_LEN {
            continue;
        }
        n += 1;
        let input = hex(c["input_msk_hex"].as_str().unwrap());
        let file = read_msk(&input).expect("legado");
        assert_eq!((file.x, file.y), (0, 0));
        let ours = write_msk(&file);
        assert_eq!(
            &ours[70..],
            &input[..],
            "{}: os bits são os do ficheiro",
            name(&c)
        );
        let oracle = oracle_msk(&c).expect("regravou");
        assert_eq!(
            &oracle[70..70 + LEGACY_LEN - 6],
            &input[6..],
            "{}",
            name(&c)
        );
        assert!(oracle[70 + LEGACY_LEN - 6..].iter().all(|b| *b == 0xFF));
    }
    assert_eq!(n, 3, "os três legados de 8000 bytes da fixtura");
}

#[test]
fn msk_round_trip_keeps_offset_holes_and_odd_widths() {
    let mut m = Mask::empty(13, 7);
    for (x, y) in [(0, 0), (12, 0), (5, 3), (6, 3), (12, 6), (0, 6)] {
        m.set(x, y, true);
    }
    let f = MaskFile {
        x: -4,
        y: 9,
        mask: m,
    };
    assert_eq!(read_msk(&write_msk(&f)), Ok(f.clone()));
    let canvas = f.to_selection(20, 20);
    assert_eq!(canvas.count(), 4, "as duas da coluna 0 caem fora da tela");
}

#[test]
fn msk_empty_selection_has_no_file_and_short_input_fails() {
    assert_eq!(MaskFile::from_selection(&Mask::empty(8, 8)), None);
    assert_eq!(read_msk(&[]), Err(MskError::Truncated));
    assert_eq!(
        read_msk(&vec![0; LEGACY_LEN]).map(|f| f.mask.is_empty()),
        Ok(true)
    );
}
