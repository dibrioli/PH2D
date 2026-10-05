//! **As seções VIVAS do Inspector — a tabela, e só ela.**
//!
//! ⚠️ **Irmão de [`super::menus`] por CAP de LOC** (2026-08-21): aquele arquivo estava a 673 e a
//! tabela levava-o a 739 contra um teto de 700. *Cortar para o irmão é a cura; alargar a
//! allowlist não é* — os ratchets só descem.
//!
//! A responsabilidade também separa limpo: `menus.rs` é sobre o que o botão-direito ABRE; isto é
//! sobre o que uma seção viva É, e as quatro faces dela (dobra · pega de arrasto · despacho da pega ·
//! menu de botão direito) que passaram meses a discordar por serem enumeradas em quatro sítios.

use super::*;

/// **A TABELA ÚNICA das seções vivas do Inspector: `(cabeçalho, pega de arrasto)`.**
///
/// ⚠️ A 2.ª coluna foi o PONTO DE COR até 2026-09-29: o dono retirou o círculo e o lugar dele
/// virou a pega que reordena a secção — a mesma tabela, a mesma posição, outro controlo.
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
///
/// ⚠️ `48` → `47` em 2026-10-05: a secção LIVE MESH saiu com o 3D (ADR-0179).
pub const LIVE_SECTIONS: [(NodeId, NodeId); 46] = [
    (INSP_LIVE_NAME_SECTION, INSP_LIVE_NAME_GRIP),
    (INSP_LIVE_VISIBILITY_SECTION, INSP_LIVE_VISIBILITY_GRIP),
    (INSP_LIVE_TRANSFORM_SECTION, INSP_LIVE_TRANSFORM_GRIP),
    (INSP_LIVE_RENDER_SECTION, INSP_LIVE_RENDER_GRIP),
    (INSP_LIVE_COLOR_SECTION, INSP_LIVE_COLOR_GRIP),
    (INSP_LIVE_SHEET_SECTION, INSP_LIVE_SHEET_GRIP),
    (INSP_LIVE_ORDERING_SECTION, INSP_LIVE_ORDERING_GRIP),
    (INSP_LIVE_SAMPLING_SECTION, INSP_LIVE_SAMPLING_GRIP),
    (INSP_LIVE_SLICE_SECTION, INSP_LIVE_SLICE_GRIP),
    (INSP_LIVE_ANCHOR_SECTION, INSP_LIVE_ANCHOR_GRIP),
    (INSP_LIVE_ANIM_SECTION, INSP_LIVE_ANIM_GRIP),
    (INSP_LIVE_BLEND_SECTION, INSP_LIVE_BLEND_GRIP),
    (INSP_LIVE_PHYSICS_SECTION, INSP_LIVE_PHYSICS_GRIP),
    (INSP_LIVE_JOINT_SECTION, INSP_LIVE_JOINT_GRIP),
    (INSP_LIVE_WHEEL_SECTION, INSP_LIVE_WHEEL_GRIP),
    (INSP_LIVE_PLAYER_SECTION, INSP_LIVE_PLAYER_GRIP),
    // ⭐ A secção TIMERS (TOP-20 #2, W3) — a 17.ª, e a primeira da família LÓGICA.
    (INSP_LIVE_TIMER_SECTION, INSP_LIVE_TIMER_GRIP),
    // ⭐ A secção SIGNAL ACTIONS (TOP-20 #5, W3) — a 18.ª, e a segunda da família LÓGICA.
    (INSP_LIVE_ACTION_SECTION, INSP_LIVE_ACTION_GRIP),
    // ⭐ A secção AUDIO (TOP-20 #4, W3) — a 19.ª, e a primeira da família ÁUDIO.
    (INSP_LIVE_AUDIO_SECTION, INSP_LIVE_AUDIO_GRIP),
    // ⭐ A secção CAMERA (TOP-20 #7, W3) — a 20.ª, e a primeira da família CÂMERA.
    (INSP_LIVE_CAMERA_SECTION, INSP_LIVE_CAMERA_GRIP),
    // ⭐ A secção TAGS (TOP-20 #9, W3) — a 21.ª, e a primeira da família IDENTIDADE que é opcional.
    //
    // ⚠️ **No FIM da tabela, e pintada com as outras opcionais**, embora o descritor a ponha na
    // família `Identity`: uma secção que só existe para quem TEM o componente (ADR-0166) nasce no
    // grupo das opcionais, como as quatro acima — e acrescentar a meio renumeraria a lista posicional
    // das notas, que é a armadilha que a §5 9-Slice já pagou.
    (INSP_LIVE_TAGS_SECTION, INSP_LIVE_TAGS_GRIP),
    // ⛔⛔ **As três seguintes entraram TARDE, e o atraso é o achado** (ver
    // [`super::INSP_LIVE_FACTORY_GRIP`]): as duas da fábrica shiparam **fora** desta tabela em
    // 2026-09-14, logo com o chevron a prometer uma dobra que não podia acontecer. Quem o viu foi
    // a wave seguinte, ao ir escrever a mesma linha.
    //
    // ⭐ A 22.ª e a 23.ª — FACTORY e LIFECYCLE (TOP-20 #11 e #12).
    (INSP_LIVE_FACTORY_SECTION, INSP_LIVE_FACTORY_GRIP),
    (INSP_LIVE_LIFECYCLE_SECTION, INSP_LIVE_LIFECYCLE_GRIP),
    // ⭐ A 24.ª — TOP-DOWN PLAYER (TOP-20 #13), a primeira da família MOVIMENTO que é opcional.
    (INSP_LIVE_TOPDOWN_SECTION, INSP_LIVE_TOPDOWN_GRIP),
    // ⭐ A 25.ª — PROJECTILE MOTION (TOP-20 #14), a segunda da família MOVIMENTO.
    (INSP_LIVE_PROJECTILE_SECTION, INSP_LIVE_PROJECTILE_GRIP),
    // ⭐⭐ RAY SENSOR (suplente #21) — o objecto que OLHA. ⚠️ **Entrar aqui não é arrumação:** quem
    // falta nesta tabela não é `mark_collapsible_section`ado nem é `is_section_header_id`, logo o
    // cabeçalho pinta o chevron e **a dobra não pode acontecer** — o defeito que a FACTORY e a
    // LIFECYCLE shiparam em 2026-09-14 e que a wave seguinte apanhou ao vir escrever esta linha.
    (INSP_LIVE_RAY_SECTION, INSP_LIVE_RAY_GRIP),
    // ⭐ A 26.ª — STATE MACHINE (TOP-20 #15), o cérebro autorável. ⚠️ Entrou **no mesmo commit** que
    // a secção, que é exactamente o que o censo `architecture_every_live_section_is_in_the_table`
    // existe para garantir desde que a fábrica shipou fora desta tabela.
    (INSP_LIVE_SM_SECTION, INSP_LIVE_SM_GRIP),
    // ⭐ A 27.ª — SCRIPT (TOP-20 #16), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_SCRIPT_SECTION, INSP_LIVE_SCRIPT_GRIP),
    // ⭐ A 28.ª — PARTICLES (TOP-20 #18), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_PARTICLES_SECTION, INSP_LIVE_PARTICLES_GRIP),
    // ⭐ A 29.ª — HUD (TOP-20 #20), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_HUD_SECTION, INSP_LIVE_HUD_GRIP),
    // ⭐ A 30.ª — SEQUENCE (TOP-20 #19), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_SEQ_SECTION, INSP_LIVE_SEQ_GRIP),
    (INSP_LIVE_WATCH_SECTION, INSP_LIVE_WATCH_GRIP),
    // ⭐ A 32.ª — GATILHO (suplente #24), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_TRIGGER_SECTION, INSP_LIVE_TRIGGER_GRIP),
    // ⭐ A 33.ª — TWEEN (suplente #22), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_TWEEN_SECTION, INSP_LIVE_TWEEN_GRIP),
    // ⭐ A 34.ª — PATH FOLLOW (suplente #23), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_PATHFOLLOW_SECTION, INSP_LIVE_PATHFOLLOW_GRIP),
    // ⭐ A 35.ª e a 36.ª — CAMERA SHAKE e SHAKE EMITTER (suplente #25), no mesmo commit que as
    // secções, pela lei do censo acima. ⚠️ **Duas e não uma:** elas moram em objectos DIFERENTES
    // (a câmera e quem explode), logo nunca aparecem juntas no mesmo Inspector.
    (INSP_LIVE_SHAKE_SECTION, INSP_LIVE_SHAKE_GRIP),
    (INSP_LIVE_EMITTER_SECTION, INSP_LIVE_EMITTER_GRIP),
    // ⭐ A 37.ª — WEAPON (a ARMA do jogador), no mesmo commit que a secção, pela lei do censo
    // `architecture_every_live_section_is_in_the_table`.
    (INSP_LIVE_WEAPON_SECTION, INSP_LIVE_WEAPON_GRIP),
    // ⭐ A 39.ª — PARALLAX (plano 24), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_PARALLAX_SECTION, INSP_LIVE_PARALLAX_GRIP),
    // ⭐ A 40.ª e a 41.ª — HEALTH e DAMAGE (plano 28, W3), no mesmo commit que as secções, pela lei
    // do censo acima. ⚠️ **Duas e não uma:** um inimigo que também magoa mostra as duas, e uma bala
    // só a segunda.
    (INSP_LIVE_HEALTH_SECTION, INSP_LIVE_HEALTH_GRIP),
    (INSP_LIVE_DAMAGE_SECTION, INSP_LIVE_DAMAGE_GRIP),
    // ⭐ A 42.ª — HEALTH BAR (plano 28, W4), no mesmo commit que a secção, pela lei do censo acima.
    (INSP_LIVE_HEALTH_BAR_SECTION, INSP_LIVE_HEALTH_BAR_GRIP),
    // ⭐ A 44.ª e a 45.ª — NAV REGION e NAV AGENT (plano 30, W4), no mesmo commit que as secções.
    (INSP_LIVE_NAV_REGION_SECTION, INSP_LIVE_NAV_REGION_GRIP),
    (INSP_LIVE_NAV_AGENT_SECTION, INSP_LIVE_NAV_AGENT_GRIP),
    // ⭐ A 46.ª e a 47.ª — NAV COST AREA e NAV LINK (plano 30, W7), no mesmo commit que as secções.
    (
        INSP_LIVE_NAV_COST_AREA_SECTION,
        INSP_LIVE_NAV_COST_AREA_GRIP,
    ),
    (INSP_LIVE_NAV_LINK_SECTION, INSP_LIVE_NAV_LINK_GRIP),
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

/// Só as pegas de arrasto — a outra projeção da mesma tabela.
pub const LIVE_SECTION_GRIP_IDS: [NodeId; LIVE_SECTIONS.len()] = project_section_grip_ids();

const fn project_section_grip_ids() -> [NodeId; LIVE_SECTIONS.len()] {
    let mut out = [NodeId(0); LIVE_SECTIONS.len()];
    let mut i = 0;
    while i < LIVE_SECTIONS.len() {
        out[i] = LIVE_SECTIONS[i].1;
        i += 1;
    }
    out
}

/// ⭐ **A pega de arrasto de uma secção** — a leitura da tabela na direcção cabeçalho → pega.
#[must_use]
pub fn grip_of(section: NodeId) -> Option<NodeId> {
    LIVE_SECTIONS
        .iter()
        .find(|(s, _)| *s == section)
        .map(|(_, g)| *g)
}

/// ⭐ **A secção de uma pega** — a leitura na outra direcção, a do despacho do arrasto.
#[must_use]
pub fn section_of_grip(grip: NodeId) -> Option<NodeId> {
    LIVE_SECTIONS
        .iter()
        .find(|(_, g)| *g == grip)
        .map(|(s, _)| *s)
}

/// O sal que deriva a pega de uma secção que não está na [`LIVE_SECTIONS`]. Um `NodeId` é o hash
/// de um nome, e um XOR por uma constante ímpar fixa é uma bijecção: a pega de uma secção nunca é
/// outra secção do mesmo painel por acaso, e não há uma segunda tabela a escrever à mão.
const SAL_DA_PEGA: u64 = 0x5EC7_10A1_6219_9E37;

/// ⭐⭐ **A pega de QUALQUER secção** (2026-09-30, ordem do dono: *«siga com os outros painéis»*):
/// a da tabela do Inspector quando ela a declara, senão a derivada pelo [`SAL_DA_PEGA`].
///
/// ⚠️ **A volta (pega → secção) de uma pega derivada NÃO se faz por aqui**: ela lê-se no hit-index
/// do quadro ([`crate::interaction::HitIndex::section_of_grip`]), que só conhece as pegas que um
/// painel de facto PINTOU — inverter o XOR aceitaria qualquer id como pega de alguma coisa.
#[must_use]
pub fn grip_de(section: NodeId) -> NodeId {
    grip_of(section).unwrap_or(NodeId(section.0 ^ SAL_DA_PEGA))
}
