//! P8c: **a face escreve o próprio nome com as letras dele?** As faces de símbolos instaladas
//! mapeiam o ASCII para outros desenhos (o dropdown de fontes mostrava o nome em dingbats); a régua
//! é a da própria face — o cmap e o nome do glifo — nunca uma lista de nomes de família.

use ph2d_pixel_text::carimbo::Fonte;
use ph2d_pixel_text::outline::covers;
use ph2d_pixel_text::{BUILTIN, FontChoice, TextStyle};

/// `(ficheiro, nome da família, escreve-o?)` — as faces desta máquina (ausente = salta).
const CASOS: [(&str, &str, bool); 5] = [
    ("/usr/share/fonts/gsfonts/D050000L.otf", "D050000L", false),
    (
        "/usr/share/fonts/gsfonts/StandardSymbolsPS.otf",
        "Standard Symbols PS",
        false,
    ),
    ("/usr/share/fonts/TTF/DejaVuSans.ttf", "DejaVu Sans", true),
    (
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "Noto Sans",
        true,
    ),
    (
        "/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf",
        "Noto Sans Symbols",
        true,
    ),
];

#[test]
fn a_symbol_face_does_not_cover_its_own_name() {
    let mut medidos = 0;
    for (caminho, nome, quer) in CASOS {
        let Ok(dados) = std::fs::read(caminho) else {
            continue;
        };
        medidos += 1;
        assert_eq!(covers(&dados, 0, nome), quer, "{caminho}: «{nome}»");
    }
    if std::path::Path::new(CASOS[0].0).exists() {
        assert!(medidos >= 1);
    }
}

#[test]
fn every_builtin_pixel_font_covers_its_own_name() {
    for b in BUILTIN {
        let f = Fonte::new(TextStyle::new(FontChoice::Builtin(b.0.to_owned()), 1));
        assert!(f.covers(b.0), "{}", b.0);
    }
}
