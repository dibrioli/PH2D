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
pub const LIVE_SECTIONS: [(NodeId, NodeId); 32] = [
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
    (INSP_LIVE_ANIM_SECTION, INSP_LIVE_ANIM_COLOR),
    (INSP_LIVE_BLEND_SECTION, INSP_LIVE_BLEND_COLOR),
    (INSP_LIVE_PHYSICS_SECTION, INSP_LIVE_PHYSICS_COLOR),
    (INSP_LIVE_JOINT_SECTION, INSP_LIVE_JOINT_COLOR),
    (INSP_LIVE_WHEEL_SECTION, INSP_LIVE_WHEEL_COLOR),
    (INSP_LIVE_PLAYER_SECTION, INSP_LIVE_PLAYER_COLOR),
    // ⭐ A secção TIMERS (TOP-20 #2, W3) — a 17.ª, e a primeira da família LÓGICA.
    (INSP_LIVE_TIMER_SECTION, INSP_LIVE_TIMER_COLOR),
    // ⭐ A secção SIGNAL ACTIONS (TOP-20 #5, W3) — a 18.ª, e a segunda da família LÓGICA.
    (INSP_LIVE_ACTION_SECTION, INSP_LIVE_ACTION_COLOR),
    // ⭐ A secção AUDIO (TOP-20 #4, W3) — a 19.ª, e a primeira da família ÁUDIO.
    (INSP_LIVE_AUDIO_SECTION, INSP_LIVE_AUDIO_COLOR),
    // ⭐ A secção CAMERA (TOP-20 #7, W3) — a 20.ª, e a primeira da família CÂMERA.
    (INSP_LIVE_CAMERA_SECTION, INSP_LIVE_CAMERA_COLOR),
    // ⭐ A secção TAGS (TOP-20 #9, W3) — a 21.ª, e a primeira da família IDENTIDADE que é opcional.
    //
    // ⚠️ **No FIM da tabela, e pintada com as outras opcionais**, embora o descritor a ponha na
    // família `Identity`: uma secção que só existe para quem TEM o componente (ADR-0166) nasce no
    // grupo das opcionais, como as quatro acima — e acrescentar a meio renumeraria a lista posicional
    // das notas, que é a armadilha que a §5 9-Slice já pagou.
    (INSP_LIVE_TAGS_SECTION, INSP_LIVE_TAGS_COLOR),
    // ⛔⛔ **As três seguintes entraram TARDE, e o atraso é o achado** (ver
    // [`super::INSP_LIVE_FACTORY_COLOR`]): as duas da fábrica shiparam **fora** desta tabela em
    // 2026-09-14, logo com o chevron a prometer uma dobra que não podia acontecer. Quem o viu foi
    // a wave seguinte, ao ir escrever a mesma linha.
    //
    // ⭐ A 22.ª e a 23.ª — FACTORY e LIFECYCLE (TOP-20 #11 e #12).
    (INSP_LIVE_FACTORY_SECTION, INSP_LIVE_FACTORY_COLOR),
    (INSP_LIVE_LIFECYCLE_SECTION, INSP_LIVE_LIFECYCLE_COLOR),
    // ⭐ A 24.ª — TOP-DOWN PLAYER (TOP-20 #13), a primeira da família MOVIMENTO que é opcional.
    (INSP_LIVE_TOPDOWN_SECTION, INSP_LIVE_TOPDOWN_COLOR),
    // ⭐ A 25.ª — PROJECTILE MOTION (TOP-20 #14), a segunda da família MOVIMENTO.
    (INSP_LIVE_PROJECTILE_SECTION, INSP_LIVE_PROJECTILE_COLOR),
    // ⭐ A 26.ª — STATE MACHINE (TOP-20 #15), o cérebro autorável. ⚠️ Entrou **no mesmo commit** que
    // a secção, que é exactamente o que o censo `architecture_every_live_section_is_in_the_table`
    // existe para garantir desde que a fábrica shipou fora desta tabela.
    (INSP_LIVE_SM_SECTION, INSP_LIVE_SM_COLOR),
    // ⭐ A 27.ª — SCRIPT (TOP-20 #16), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_SCRIPT_SECTION, INSP_LIVE_SCRIPT_COLOR),
    // ⭐ A 28.ª — PARTICLES (TOP-20 #18), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_PARTICLES_SECTION, INSP_LIVE_PARTICLES_COLOR),
    // ⭐ A 29.ª — HUD (TOP-20 #20), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_HUD_SECTION, INSP_LIVE_HUD_COLOR),
    // ⭐ A 30.ª — SEQUENCE (TOP-20 #19), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_SEQ_SECTION, INSP_LIVE_SEQ_COLOR),
    (INSP_LIVE_WATCH_SECTION, INSP_LIVE_WATCH_COLOR),
    // ⭐ A 32.ª — GATILHO (suplente #24), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_TRIGGER_SECTION, INSP_LIVE_TRIGGER_COLOR),
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
