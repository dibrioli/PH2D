//! **As cenas de smoke da CONFERÊNCIA DOS NÓS** (doc 89) — só as DECLARAÇÕES dos
//! módulos, uma família num arquivo só.
//!
//! ⚠️ **Este arquivo existe por um teto de LOC, e o corte é por FAMÍLIA e não por
//! data:** o `motion_state.rs` passou os 600 do HR-18 ao ganhar a 40ª cena, e um split
//! por *"as declarações novas"* envelheceria na semana seguinte. Estas respondem todas
//! à mesma pergunta — *que documento cada célula da conferência encena* — e é isso que
//! as mantém juntas.
//!
//! ⚠️ **Os caminhos NÃO mudam.** O pai reexporta este módulo com `pub(crate) use`, então
//! `motion_state::conferencia_demos_*` continua a resolver como sempre resolveu — um
//! split que renomeasse 40 caminhos seria um diff de centenas de linhas para ganhar uma.

/// As cenas da CONFERÊNCIA DOS NÓS (doc 89) — as quatro metades que só o olho
/// julga, `PH2D_GPU_COOK_DEMO=32..35`.
#[path = "motion_state_conferencia_demos.rs"]
pub(crate) mod conferencia_demos;
/// A cena da ARITMETICA (`=41`) — o grupo A da conferencia (cinco nos irmaos do
/// dominio de VALOR), irmao pelo mesmo teto de LOC.
#[path = "motion_state_conferencia_demos_arith.rs"]
pub(crate) mod conferencia_demos_arith;

/// A cena do AUDIO (`=40`), irmao pelo mesmo motivo — e porque ela escreve a
/// propria fixture em disco (nao ha asset de audio no repo).
#[path = "motion_state_conferencia_demos_audio.rs"]
pub(crate) mod conferencia_demos_audio;
/// A cena da DIRECAO (`=38`) mora num irmao: o pai bate no teto de LOC da shell.
#[path = "motion_state_conferencia_demos_direction.rs"]
pub(crate) mod conferencia_demos_direction;
#[path = "motion_state_conferencia_demos_stats.rs"]
pub(crate) mod conferencia_demos_stats;
#[path = "motion_state_conferencia_demos_table_seed.rs"]
pub(crate) mod conferencia_demos_table_seed;

// O grupo E — a comparação e o nome que não resolve (cena `=45`).
// ⚠️ `pub(crate)` porque o gate do badge da cena mora no `render_loop` (ver o doc
// do `build_compare_demo_document`).
#[path = "motion_state_conferencia_demos_compare.rs"]
pub(crate) mod conferencia_demos_compare;
// O grupo F — o ENVELOPE: que forma tem uma coisa que acende e apaga (cena `=46`).
#[path = "motion_state_conferencia_demos_envelope.rs"]
pub(crate) mod conferencia_demos_envelope;

#[path = "motion_state_conferencia_demos_velocity.rs"]
pub(crate) mod conferencia_demos_velocity;

#[path = "motion_state_conferencia_demos_collide.rs"]
pub(crate) mod conferencia_demos_collide;
#[path = "motion_state_conferencia_demos_pin.rs"]
pub(crate) mod conferencia_demos_pin;
#[path = "motion_state_conferencia_demos_proximity.rs"]
pub(crate) mod conferencia_demos_proximity;
#[path = "motion_state_conferencia_demos_rate.rs"]
pub(crate) mod conferencia_demos_rate;

#[path = "motion_state_conferencia_demos_octave.rs"]
pub(crate) mod conferencia_demos_octave;

#[path = "motion_state_conferencia_demos_shape.rs"]
pub(crate) mod conferencia_demos_shape;

#[path = "motion_state_conferencia_demos_axes.rs"]
pub(crate) mod conferencia_demos_axes;
#[path = "motion_state_conferencia_demos_clock.rs"]
pub(crate) mod conferencia_demos_clock;
#[path = "motion_state_conferencia_demos_column.rs"]
pub(crate) mod conferencia_demos_column;
#[path = "motion_state_conferencia_demos_field_space.rs"]
pub(crate) mod conferencia_demos_field_space;
#[path = "motion_state_conferencia_demos_join.rs"]
pub(crate) mod conferencia_demos_join;
#[path = "motion_state_conferencia_demos_sortkey.rs"]
pub(crate) mod conferencia_demos_sortkey;

#[path = "motion_state_conferencia_demos_space.rs"]
pub(crate) mod conferencia_demos_space;
#[path = "motion_state_conferencia_demos_substep.rs"]
pub(crate) mod conferencia_demos_substep;
#[path = "motion_state_conferencia_demos_taper.rs"]
pub(crate) mod conferencia_demos_taper;

#[path = "motion_state_conferencia_demos_cursor.rs"]
pub(crate) mod conferencia_demos_cursor;

/// A cena do QUE O EFEITO NAO SABIA FAZER (`=84`) — as curas NOVAS da folha 11: o modo da
/// sombra e a lente do rgb_split. ⚠️ Irmã da `=70` e não a mesma cena: aquela mostra a família
/// `fx.*` inteira, esta mostra o par ANTES/DEPOIS de dois controles que não existiam.
#[path = "motion_state_conferencia_demos_fx_modes.rs"]
pub(crate) mod conferencia_demos_fx_modes;

/// A cena do CAMPO QUE ERA UM NUMERO (`=83`) — as duas portas lidas por `.first()` e a
/// altura da onda que so' sabia engordar. ⚠️ Oraculo: a figura VARIA ao longo de si mesma.
#[path = "motion_state_conferencia_demos_campo.rs"]
pub(crate) mod conferencia_demos_campo;
#[path = "motion_state_conferencia_demos_color.rs"]
pub(crate) mod conferencia_demos_color;
/// A cena da FAIXA (`=79`) — onde a saída de um animador cai, e a armadilha da
/// polaridade que a entrega curou (folha 06).
#[path = "motion_state_conferencia_demos_faixa.rs"]
pub(crate) mod conferencia_demos_faixa;
#[path = "motion_state_conferencia_demos_field.rs"]
pub(crate) mod conferencia_demos_field;
#[path = "motion_state_conferencia_demos_force.rs"]
pub(crate) mod conferencia_demos_force;
#[path = "motion_state_conferencia_demos_fx.rs"]
pub(crate) mod conferencia_demos_fx;
/// A cena da CURA DOS KNOBS MORTOS (`=82`) — doc 90. ⚠️ O oráculo dela é CONTAR
/// LINHAS: cada célula desenha o mesmo controle no mínimo e no máximo ao mesmo
/// tempo, e onde ele é mudo as duas cópias coincidem.
#[path = "motion_state_conferencia_demos_gates.rs"]
pub(crate) mod conferencia_demos_gates;
#[path = "motion_state_conferencia_demos_goal.rs"]
pub(crate) mod conferencia_demos_goal;
/// A cena dos KNOBS (`=78`) — os nove controles apendados ao domínio de valor
/// (folha 15), cada um com o nó sem ele desenhado ao lado.
#[path = "motion_state_conferencia_demos_knobs.rs"]
pub(crate) mod conferencia_demos_knobs;
/// A cena do OPERADOR (`=77`) — o *Echo Operator* do rastro e o *Strobe Operator* do
/// flash, que a folha 07 dizia serem um conserto só.
#[path = "motion_state_conferencia_demos_operator.rs"]
pub(crate) mod conferencia_demos_operator;
/// A cena do METRÓNOMO (`=80`) — a régua, a fase por-linha, a janela e a
/// referência por-elemento (folha 12, que fechou por inteiro).
#[path = "motion_state_conferencia_demos_pulso.rs"]
pub(crate) mod conferencia_demos_pulso;
#[path = "motion_state_conferencia_demos_rank.rs"]
pub(crate) mod conferencia_demos_rank;
#[path = "motion_state_conferencia_demos_sim.rs"]
pub(crate) mod conferencia_demos_sim;
/// A cena do ESTILO (`=76`) — a borda, o Trim e o tracejado do `source.shape` (folha 14).
#[path = "motion_state_conferencia_demos_style.rs"]
pub(crate) mod conferencia_demos_style;
/// A cena da UTILIDADE (`=81`) — o vocabulário do mixer, a ordem rodada e o
/// ponto polar (folha 08). ⚠️ Ela desenha FIGURAS, não perfis.
#[path = "motion_state_conferencia_demos_util.rs"]
pub(crate) mod conferencia_demos_util;

#[path = "motion_state_conferencia_demos_drizzle.rs"]
pub(crate) mod conferencia_demos_drizzle;

#[path = "motion_state_conferencia_demos_deform.rs"]
pub(crate) mod conferencia_demos_deform;
#[path = "motion_state_conferencia_demos_text.rs"]
pub(crate) mod conferencia_demos_text;
#[path = "motion_state_conferencia_demos_time.rs"]
pub(crate) mod conferencia_demos_time;
#[path = "motion_state_conferencia_demos_transform.rs"]
pub(crate) mod conferencia_demos_transform;
#[path = "motion_state_conferencia_demos_wave.rs"]
pub(crate) mod conferencia_demos_wave;
#[path = "motion_state_conferencia_demos_weight.rs"]
pub(crate) mod conferencia_demos_weight;
