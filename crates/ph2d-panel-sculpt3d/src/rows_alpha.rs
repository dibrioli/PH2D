//! **AS PERGUNTAS QUE O PADRÃO FAZ** — irmão (`#[path]`) do [`super`], cortado
//! por ASSUNTO.
//!
//! A tabela do `rows.rs` diz **o que um knob É** (rótulo, faixa, passo, quem lê
//! e quem escreve). Aqui vivem as duas outras perguntas, as duas do PADRÃO:
//!
//! * **quando uma pista dele APARECE** — [`stamp_alpha`] e [`directional_alpha`],
//!   que são a lei anti-controle-morto desta seção;
//! * **como um valor de pista ATRAVESSA para o motor** — [`degrees`], a fronteira
//!   `f32` → `u16`, e o teto que o motor declara.
//!
//! ⚠️ **O `always` NÃO veio junto, e a ausência é o corte:** ele é o predicado
//! partilhado pelas três tabelas (pincel, sombreamento, topologia) e não fala do
//! padrão — trazê-lo para cá o poria num arquivo cujo nome mente sobre ele.
//!
//! ⚠️ **O gate de LOC foi o gatilho, não a razão.** O `rows.rs` cruzou os 600 do
//! `architecture_panel_loc_cap` quando a demão ganhou a row de altura, e o que
//! saiu foi a metade com fronteira própria — não a última coisa que alguém
//! escreveu. ⚠️ E ele ficou **latente**: aquele gate mora em
//! `ph2d-editor-core/tests/`, então um fechamento por `cargo test -p
//! ph2d-panel-sculpt3d` não o alcança (a família estrutural que esta casa já
//! registrou várias vezes).

use ph2d_editor_core::ids;

use super::types::{Place, Row};
use crate::state::{Sculpt3dUi, UiLevel};

/// **Só com um CARIMBO armado** — as duas pistas de colocação.
///
/// ⚠️ **A pergunta é `is_image`, e não `is_directional`, e a diferença é o que
/// separa um carimbo de um campo:** os três procedurais direcionais apontam para
/// um lado e são HOMOGÊNEOS ao longo dele — um campo infinito não tem posição,
/// só fase, e uma fase é outro controle (uma semente) que este módulo não tem.
/// Oferecer o deslocamento ali seriam duas pistas que o Strata ignora por
/// completo e que o Scratches e o Weave leem como um número sem significado.
///
/// ⚠️ E a neutralidade dos outros não depende desta função: quem a garante é o
/// `Brush::alpha_frame`, que ZERA o deslocamento sem uma imagem armada. Esta
/// decide o que APARECE; aquele decide o que o motor recebe — e é por isso que
/// esconder a row aqui não pode deixar um valor autorado agindo em silêncio.
pub(super) fn stamp_alpha(u: &Sculpt3dUi) -> bool {
    u.brush
        .alpha
        .as_ref()
        .is_some_and(ph2d_sculpt3d::Alpha::is_image)
}

/// **Só com um padrão DIRECIONAL armado.**
///
/// ⚠️ A pergunta é feita à porta do MOTOR ([`ph2d_sculpt3d::Alpha::is_directional`]),
/// nunca a uma lista de nomes aqui: sob um dos seis isotrópicos o eixo não move
/// um bit — há gate provando —, e duas pistas que desenham e não fazem nada são
/// o controle morto que esta casa varre a cada wave. É a mesma lei do
/// `Plane Offset` e das duas pistas de lâmpada sob um matcap.
pub(super) fn directional_alpha(u: &Sculpt3dUi) -> bool {
    u.brush
        .alpha
        .as_ref()
        .is_some_and(ph2d_sculpt3d::Alpha::is_directional)
}

/// Um valor de pista → graus inteiros.
///
/// ⚠️ **A pista é `f32` e o ângulo é `u16`**, e a conversão mora AQUI, na
/// fronteira, e não no motor: o rotor deste app anda de grau em grau, então um
/// ângulo fracionário não teria como ser resolvido sem um segundo caminho. É a
/// mesma travessia que o painel já faz para as duas pistas de lâmpada.
/// ⚠️ **ARREDONDA, e o gate de costura pegou o truncamento na hora.** A row
/// mostra zero casas, então `134,625` é lido como **135** no readout; truncando,
/// o padrão iria para 134 e o número na tela discordaria do eixo que o pincel
/// usa — a doença de *seed ≠ sample* que este repo já pagou em quatro módulos.
/// A tolerância de `0,5` do gate `each_row_owns_exactly_one_field` é literalmente
/// o arredondamento que ele espera encontrar aqui.
///
/// ⚠️ **`safe_clamp` e não `.clamp`**, e o `arch_safe_clamp_only` foi quem cobrou:
/// o teto `f32::from(u16::MAX)` **não é um literal**, e o `.clamp` da `std`
/// **panica** com bounds trocados e devolve o valor original com `NaN`. Aqui um
/// `NaN` cairia no `as u16`, que é comportamento definido mas absurdo (zero) — a
/// peneira tem de vir antes.
pub(super) fn degrees(v: f32) -> u16 {
    ph2d_editor_core::math::safe_clamp(v.round(), 0.0, f32::from(u16::MAX)) as u16
}

/// O zênite do eixo, lido do dono dele.
pub(super) const MAX_AXIS_ELEV_F32: f32 = ph2d_sculpt3d::MAX_AXIS_ELEV_DEG as f32; // CLAMP-OK: teto do motor

// ── ⭐ AS SEIS PISTAS DO PADRÃO ─────────────────────────────────────────────
//
// ⚠️ **Elas vieram do `rows.rs` em 2026-09-08, e o corte é o MESMO que criou
// este ficheiro:** as perguntas que estas fileiras fazem e os números que elas
// lêem já viviam aqui, e só as fileiras estavam do outro lado. *Metade de um
// assunto em dois ficheiros lê-se como dois assuntos.*
//
// ⚠️ **O gatilho foi de novo o `architecture_panel_loc_cap`** — o `rows.rs`
// cruzou os `600` (`604`) ao ganhar as duas fileiras do report de 08/09 — e de
// novo o que saiu foi a metade com fronteira própria, não a última escrita.

pub(super) const ALPHA_SCALE: Row = Row {
    label: "panel.sculpt3d.alpha_scale",
    slider: ids::SCULPT3D_ALPHA_SCALE,
    chip: ids::SCULPT3D_ALPHA_SCALE_NUM,
    // ⚠️ Os dois extremos são do MOTOR, não escolhidos aqui: eles saem da lei
    // das dez arestas (`ph2d_sculpt3d::DEFAULT_ALPHA_SCALE`), e um literal
    // nesta tabela seria a segunda cópia deles.
    min: ph2d_sculpt3d::MIN_ALPHA_SCALE,
    max: ph2d_sculpt3d::MAX_ALPHA_SCALE,
    step: 0.01, // LITERAL-PX-OK: passo em unidades de objeto, não métrica de layout
    decimals: 2,
    get: |u| u.brush.alpha_scale,
    set: |u, v| u.brush.alpha_scale = v,
    // ⚠️ **A row some sem padrão armado**, e não é cosmético: o número é o
    // tamanho de uma feature que não existe. É o mesmo mecanismo das duas
    // pistas de lâmpada sob um matcap — uma row condicional é PULADA, nunca
    // pintada apagada, porque um controle que desenha e não responde mente.
    //
    // ⚠️ **E ela some também sob um CARIMBO**, porque a pergunta muda de
    // régua: um estêncil é medido na TELA, não no modelo, e quem responde
    // por ele é a row seguinte. Reusar este número com duas unidades faria
    // ele trocar de significado em silêncio ao trocar de padrão.
    show: |u| u.brush.alpha.is_some() && !stamp_alpha(u),
    level: UiLevel::Basic,
    place: Place::AfterAlpha,
};

pub(super) const STAMP_SCALE: Row = Row {
    label: "panel.sculpt3d.stamp_scale",
    slider: ids::SCULPT3D_STAMP_SCALE,
    chip: ids::SCULPT3D_STAMP_SCALE_NUM,
    // ⚠️ **A faixa é em FRAÇÃO DA ALTURA DA TELA**, e por isso ela não fala
    // do modelo: `1,0` é um ladrilho ocupando a tela inteira e `0,02` são
    // cinquenta atravessando-a. Um estêncil não sabe o tamanho da peça — é
    // justamente essa independência que o artista pediu —, então herdar a
    // pista de `Pattern Size` (unidades de OBJETO, semeada pela densidade da
    // malha) seria herdar a régua errada.
    min: 0.02,  // LITERAL-PX-OK: fração da altura da vista, não métrica de layout
    max: 1.0,   // LITERAL-PX-OK: idem
    step: 0.01, // LITERAL-PX-OK: idem
    decimals: 2,
    get: |u| u.brush.alpha_stencil_scale,
    set: |u, v| u.brush.alpha_stencil_scale = v,
    show: stamp_alpha,
    level: UiLevel::Basic,
    place: Place::AfterAlpha,
};

pub(super) const ALPHA_OFF_X: Row = Row {
    label: "panel.sculpt3d.alpha_off_x",
    slider: ids::SCULPT3D_ALPHA_OFF_X,
    chip: ids::SCULPT3D_ALPHA_OFF_X_NUM,
    // ⚠️ **A faixa é SIMÉTRICA e mede um LADO do modelo.** Uma primitiva
    // nasce cabendo na esfera unitária (span 2), então ±1 leva o carimbo de
    // uma ponta à outra; e o zero tem de cair no MEIO da pista, porque
    // *nenhum deslocamento* é o estado neutro e não um extremo.
    min: -1.0,
    max: 1.0,
    step: 0.01, // LITERAL-PX-OK: passo em unidades de objeto, não métrica de layout
    decimals: 2,
    get: |u| u.brush.alpha_offset[0],
    set: |u, v| u.brush.alpha_offset[0] = v,
    show: stamp_alpha,
    level: UiLevel::Basic,
    place: Place::AfterAlpha,
};

pub(super) const ALPHA_OFF_Y: Row = Row {
    label: "panel.sculpt3d.alpha_off_y",
    slider: ids::SCULPT3D_ALPHA_OFF_Y,
    chip: ids::SCULPT3D_ALPHA_OFF_Y_NUM,
    min: -1.0,
    max: 1.0,
    step: 0.01, // LITERAL-PX-OK: passo em unidades de objeto, não métrica de layout
    decimals: 2,
    get: |u| u.brush.alpha_offset[1],
    set: |u, v| u.brush.alpha_offset[1] = v,
    show: stamp_alpha,
    level: UiLevel::Basic,
    place: Place::AfterAlpha,
};

pub(super) const ALPHA_AZ: Row = Row {
    label: "panel.sculpt3d.alpha_az",
    slider: ids::SCULPT3D_ALPHA_AZ,
    chip: ids::SCULPT3D_ALPHA_AZ_NUM,
    min: 0.0,
    // 359 e não 360 — os dois extremos seriam o MESMO azimute, e uma pista
    // cujas duas pontas significam a mesma coisa tem um degrau invisível. É
    // a mesma régua do `light_az`, e de propósito: um artista que aprendeu a
    // apontar a luz não devia reaprender a apontar o padrão.
    max: 359.0, // LITERAL-PX-OK: graus de azimute, nao metrica de design
    step: 5.0,  // LITERAL-PX-OK: passo em graus
    decimals: 0,
    get: |u| f32::from(u.brush.alpha_az_deg),
    set: |u, v| u.brush.alpha_az_deg = degrees(v),
    show: directional_alpha,
    level: UiLevel::Basic,
    place: Place::AfterAlpha,
};

pub(super) const ALPHA_ELEV: Row = Row {
    label: "panel.sculpt3d.alpha_elev",
    slider: ids::SCULPT3D_ALPHA_ELEV,
    chip: ids::SCULPT3D_ALPHA_ELEV_NUM,
    // ⚠️ **Sem o piso que a LÂMPADA tem.** Lá o `MIN_ELEV_DEG` existe porque
    // uma luz rasante degenera a resposta plana; um EIXO não degenera em
    // lugar nenhum — o frame é ortonormal por identidade em qualquer
    // elevação. Copiar o piso do vizinho seria um limite herdado por
    // analogia, que é o que esta casa varre a cada wave.
    min: 0.0,
    // ⚠️ O zênite vem do MOTOR, não é escolhido aqui: acima dele o eixo
    // desceria do outro lado e o azimute já cobre esse hemisfério — dois
    // caminhos para a mesma direção.
    max: MAX_AXIS_ELEV_F32,
    step: 5.0, // LITERAL-PX-OK: passo em graus
    decimals: 0,
    get: |u| f32::from(u.brush.alpha_elev_deg),
    set: |u, v| u.brush.alpha_elev_deg = degrees(v),
    // ⚠️ **Ela some sob um CARIMBO, e é o modo inteiro numa linha:** o eixo
    // de um estêncil é a VISTA, por definição. Um controle que o inclinasse
    // tiraria o carimbo da frente — exatamente o que este modo existe para
    // impedir —, então ele não é oferecido em vez de ser oferecido e
    // ignorado.
    show: |u| directional_alpha(u) && !stamp_alpha(u),
    level: UiLevel::Basic,
    place: Place::AfterAlpha,
};
