//! **O CHIP DE CHROME TEM UMA PORTA SÓ PARA O EIXO DO HOVER.**
//!
//! # A classe de defeito
//!
//! O rail de ferramentas e os chips da barra de topo desenham **a mesma coisa**: um quadrado com
//! fundo, borda e um glifo, que reage ao ponteiro. O `cluster_painter` diz isso por escrito —
//! *"Matriz EXATA do rail (tool_rail.rs:248-280)"* — e copiou a matriz de TOKENS fielmente.
//!
//! ⚠️ **Mas o eixo não é a matriz.** O rail mistura `Border → BorderEmph` por `t` (e o tint do
//! glifo com ele); a cópia resolve a borda pelo **estado duro**. Resultado medido: no chip da barra
//! de topo o **ícone amacia** (ele passa pelo `paint_icon_button`, que recebe o par) e a **moldura
//! à volta dele SALTA** — dentro do mesmo chip, no mesmo quadro.
//!
//! ⚠️ **Copiar a matriz e não copiar o eixo é invisível a toda a suíte:** os dois pintores ficam
//! verdes sozinhos, os tokens batem certo num screenshot parado, e a divergência só existe **em
//! movimento**.
//!
//! # A lei
//!
//! *Quem pinta um chip de chrome pede a cor ao [`ph2d_editor_core::widget::chip_axis_color`], e o
//! `t` ao [`ph2d_editor_core::widget::chip_axis_t`].*
//!
//! As duas nasceram privadas dentro do `widget/tool_rail/paint.rs` — e é essa privacidade que
//! deixou a cópia nascer sem elas. Promovidas ao [`ph2d_editor_core::widget::button_surface`],
//! elas são a porta dos dois.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).expect("fonte legível")
}

/// Os dois pintores de chip de chrome deste app.
const CHIP_PAINTERS: &[&str] = &[
    "src/widget/tool_rail/paint.rs",
    "src/screens/hero/topbar/cluster_painter.rs",
];

/// ⛔ **NENHUM PINTOR DE CHIP RESOLVE A BORDA PELO ESTADO DURO.**
///
/// A agulha é `resolve(border` — a forma exacta em que a cor da moldura salta o eixo. Ela é
/// precisa: o token duro continua a existir (ele é o `hard` que a porta recebe como recuo), e
/// proibir o NOME apanharia a própria cura.
#[test]
fn no_chrome_chip_resolves_its_border_from_the_hard_state() {
    let offenders: Vec<String> = CHIP_PAINTERS
        .iter()
        .filter(|rel| read(rel).contains("resolve(border"))
        .map(|rel| (*rel).to_string())
        .collect();
    assert!(
        offenders.is_empty(),
        "estes pintores de chip resolvem a moldura pelo estado DURO — o glifo deles amacia e a \
         borda salta, dentro do mesmo chip. A porta é \
         `chip_axis_color(t, Border, BorderEmph, hard, theme)`:\n  {}",
        offenders.join("\n  ")
    );
}

/// ⛔ **E OS DOIS PEDEM O `t` À MESMA PORTA.**
///
/// ⚠️ É a metade que impede a divergência de voltar por outro caminho: um pintor que misturasse a
/// borda mas calculasse o próprio *"este estado é uma quantidade?"* podia acender um chip
/// **activo** — e um chip activo não é meio-activo. A guarda tem um dono.
#[test]
fn both_chrome_chip_painters_ask_the_same_door_for_the_axis() {
    let missing: Vec<String> = CHIP_PAINTERS
        .iter()
        .filter(|rel| !read(rel).contains("chip_axis_t("))
        .map(|rel| (*rel).to_string())
        .collect();
    assert!(
        missing.is_empty(),
        "estes pintores de chip não pedem o `t` à porta comum — a guarda do eixo (*um chip ACTIVO \
         não é meio-activo*) passa a ter duas respostas:\n  {}",
        missing.join("\n  ")
    );
}
