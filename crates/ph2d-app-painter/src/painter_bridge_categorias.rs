//! **As CATEGORIAS do navegador de presets** (dono 22/10, depois do smoke: um título por pasta
//! «motor · meio · grupo» lia-se como «nomes de pincéis apagados»). Como o Procreate, o Krita e o
//! Rebelle: um título por CATEGORIA, numa ordem fixa. A categoria vem da pasta do MEIO (o 2.º nível
//! de `<motor>/<meio>/<grupo>/<nome>`) e funde os motores — o Ink do Digital e o do Wet Painter são
//! um só «Ink». Pasta que a tabela não conhece (a do artista) → «Other», no fim.

use ph2d_i18n::{Idioma, tr_em};

/// `(chave do rótulo, pastas de meio que a categoria junta)` — a ORDEM do navegador. Pasta de
/// fábrica nova = uma entrada aqui (`cada_pasta_de_meio_da_fabrica_tem_categoria`).
pub(crate) const CATEGORIAS: &[(&str, &[&str])] = &[
    ("app.painter.presets.category.basic", &["Basic"]),
    ("app.painter.presets.category.pencil", &["Pencil"]),
    ("app.painter.presets.category.ink", &["Ink"]),
    ("app.painter.presets.category.marker", &["Marker"]),
    ("app.painter.presets.category.pastel", &["Pastel"]),
    ("app.painter.presets.category.oil", &["Oil", "ExpressOil"]),
    ("app.painter.presets.category.watercolor", &["Watercolor"]),
    ("app.painter.presets.category.airbrush", &["Airbrush"]),
    (
        "app.painter.presets.category.texture",
        &["Texture", "Stamp"],
    ),
    (
        "app.painter.presets.category.blend_erase",
        &["Blend", "Eraser"],
    ),
    ("app.painter.presets.category.effects", &["Effects"]),
];

const OUTRA: &str = "app.painter.presets.category.other";

/// O índice da categoria de uma pasta de meio; `CATEGORIAS.len()` = «Other».
pub(crate) fn categoria_da_pasta(meio: &str) -> usize {
    CATEGORIAS
        .iter()
        .position(|(_, pastas)| pastas.contains(&meio))
        .unwrap_or(CATEGORIAS.len())
}

/// O rótulo da categoria `i` (HR-15: pela tabela de texto).
pub(crate) fn rotulo_da_categoria(idioma: Idioma, i: usize) -> &'static str {
    tr_em(idioma, CATEGORIAS.get(i).map_or(OUTRA, |(k, _)| k))
}
