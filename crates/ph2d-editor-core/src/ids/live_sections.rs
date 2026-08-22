//! **As seções VIVAS do Inspector — a tabela, e só ela.**
//!
//! ⚠️ **Irmão de [`super::menus`] por CAP de LOC** (2026-08-21): aquele arquivo estava a 673 e a
//! tabela levava-o a 739 contra um teto de 700. *Cortar para o irmão é a cura; alargar a
//! allowlist não é* — os ratchets só descem.
//!
//! A responsabilidade também separa limpo: `menus.rs` é sobre o que o botão-direito ABRE; isto é
//! sobre o que uma seção viva É, e as quatro faces dela (dobra · ponto de cor · despacho do ponto ·
//! menu de contorno) que passaram meses a discordar por serem enumeradas em quatro sítios.

use super::*;

/// **A TABELA ÚNICA das seções vivas do Inspector: `(cabeçalho, ponto de cor)`.**
///
/// Name · Visibility · Transform · Render · Color & Tint · Sprite Sheet · Ordering · Sampling ·
/// **9-Slice** · **Sockets/Anchors** · Material & Blend · Physics Body · Physics Joint · Pulley Wheel · Platform Player.
///
/// # Por que UMA tabela de PARES, e não duas listas
///
/// ⚠️ Cada seção viva precisa de aparecer em **quatro** sítios que ninguém liga entre si: o
/// registo de dobra (`mark_collapsible_section`), o registo do ponto de cor (`register(_, Plain)`),
/// o braço de despacho do ponto, e o menu de botão-direito. Enquanto isso foi **enumerado** em
/// cada um deles, apodreceu: medido em 2026-08-21, três cabeçalhos (Ordering · Sampling · Blend)
/// pintavam o chevron e **não dobravam**, e sete pontos de cor estavam mortos — enquanto a nota que
/// já denunciava a podridão (`ph2d-panel-inspector/src/event.rs`) dizia **três**.
///
/// A cura que aquela nota nomeou é esta: *uma tabela `(seção, cor)` que o `pre_populate` e o braço
/// leem*. Uma seção nova entra aqui **uma vez** e nasce viva nos quatro sítios.
///
/// ⚠️ `finish_section` lê `store.section_outline_color(<id da seção>)` para TODA seção viva, por
/// isso uma seção ausente daqui tem um contorno que o passe de pintura está pronto a desenhar e
/// gesto nenhum que o possa definir.
pub const LIVE_SECTIONS: [(NodeId, NodeId); 15] = [
    (INSP_LIVE_NAME_SECTION, INSP_LIVE_NAME_COLOR),
    (INSP_LIVE_VISIBILITY_SECTION, INSP_LIVE_VISIBILITY_COLOR),
    (INSP_LIVE_TRANSFORM_SECTION, INSP_LIVE_TRANSFORM_COLOR),
    (INSP_LIVE_RENDER_SECTION, INSP_LIVE_RENDER_COLOR),
    (INSP_LIVE_COLOR_SECTION, INSP_LIVE_COLOR_COLOR),
    (INSP_LIVE_SHEET_SECTION, INSP_LIVE_SHEET_COLOR),
    (INSP_LIVE_ORDERING_SECTION, INSP_LIVE_ORDERING_COLOR),
    (INSP_LIVE_SAMPLING_SECTION, INSP_LIVE_SAMPLING_COLOR),
    (INSP_LIVE_SLICE_SECTION, INSP_LIVE_SLICE_COLOR),
    (INSP_LIVE_ANCHOR_SECTION, INSP_LIVE_ANCHOR_COLOR),
    (INSP_LIVE_BLEND_SECTION, INSP_LIVE_BLEND_COLOR),
    (INSP_LIVE_PHYSICS_SECTION, INSP_LIVE_PHYSICS_COLOR),
    (INSP_LIVE_JOINT_SECTION, INSP_LIVE_JOINT_COLOR),
    (INSP_LIVE_WHEEL_SECTION, INSP_LIVE_WHEEL_COLOR),
    (INSP_LIVE_PLAYER_SECTION, INSP_LIVE_PLAYER_COLOR),
];

/// Só os cabeçalhos — **projeção** de [`LIVE_SECTIONS`], nunca uma segunda lista.
///
/// ⚠️ Era uma lista à mão, e é por isso que existia a hipótese de as duas discordarem. Derivá-la
/// custa um `const fn` e apaga a classe inteira: *não se escolhe um desempate melhor, não se tem
/// empate.*
pub const LIVE_SECTION_IDS: [NodeId; LIVE_SECTIONS.len()] = project_section_ids();

const fn project_section_ids() -> [NodeId; LIVE_SECTIONS.len()] {
    let mut out = [NodeId(0); LIVE_SECTIONS.len()];
    let mut i = 0;
    while i < LIVE_SECTIONS.len() {
        out[i] = LIVE_SECTIONS[i].0;
        i += 1;
    }
    out
}

/// Só os pontos de cor — a outra projeção da mesma tabela.
pub const LIVE_SECTION_COLOR_IDS: [NodeId; LIVE_SECTIONS.len()] = project_section_color_ids();

const fn project_section_color_ids() -> [NodeId; LIVE_SECTIONS.len()] {
    let mut out = [NodeId(0); LIVE_SECTIONS.len()];
    let mut i = 0;
    while i < LIVE_SECTIONS.len() {
        out[i] = LIVE_SECTIONS[i].1;
        i += 1;
    }
    out
}
