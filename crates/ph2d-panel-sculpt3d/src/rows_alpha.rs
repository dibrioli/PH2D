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

use crate::state::Sculpt3dUi;

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
