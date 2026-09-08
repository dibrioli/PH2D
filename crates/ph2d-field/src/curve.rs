//! ⭐⭐ **AS DUAS CURVAS COM ESPESSURA** (W136) — a Bezier quadrática e a onda em anel.
//!
//! # ⛔⛔ A cerca da ONDA tem UMA candidata, e as outras duas foram descartadas ANTES de existirem
//!
//! A primeira redacção deste ficheiro ia ter um tecto de espessura com **três** candidatas — o
//! espaço até ao eixo e os **dois raios de curvatura** (na crista e no vale da onda), pela ideia de
//! que uma faixa mais grossa que o raio de curvatura **dobra**.
//!
//! ⛔ **Ela não dobra, e a razão é a definição:** a peça é `{ (ρ, φ) : |ρ − R(φ)| ≤ t }`, e isso é,
//! para cada `φ`, um **intervalo em `ρ`**. Um conjunto definido ponto a ponto assim nunca se
//! auto-intersecta — o que dobraria era o *offset* da curva, que é outra coisa. ⇒ as duas
//! candidatas de curvatura **decidiriam zero células**, que é exactamente a candidata morta que a
//! [`crate::knot_cord_ceiling`] teve de descobrir por um censo depois de a shipar.
//!
//! ⚠️ **O que a faixa radial custa está DECLARADO:** ela mede-se ao longo do raio, não perpendicular
//! à curva, logo no flanco de um lóbulo ela parece mais grossa. É o mesmo carácter da onda do
//! `Document` (W123), e é o que a torna exprimível sem uma segunda distância.

/// O mínimo de lóbulos — com `1` a onda é um anel excêntrico, que já é uma forma.
pub const MIN_WAVE_LOBES: u32 = 1;

/// **O máximo de lóbulos.** ⚠️ O recurso é a **MARCHA**: o divisor da onda cresce com `n`, o campo
/// fica mais conservador e o passo encurta. ⛔ Não é a árvore — um lóbulo a mais não acrescenta um
/// nó.
///
/// ⚠️ **MEDIDO** (`the_price_of_the_curves`, 07/09, `load 3,95`, uma peça a 640×360; esfera `2,5`,
/// tubo `2,0`):
///
/// | lóbulos | 1 | 2 | 4 | 8 | **12** | 16 | 24 | 32 | 48 |
/// |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
/// | ms | 3,4 | 4,2 | 5,2 | 6,9 | **8,5** | 10,4 | 12,7 | 15,5 | 18,7 |
///
/// ⭐ **O critério é o mesmo da rosca — *duas peças ainda cabem num quadro*** (`16,7 ms`): a curva
/// cruza os `8,35` a **`11,8` lóbulos**, que arredonda para `12`. ⛔ E a `48` uma peça **sozinha**
/// passa o quadro (`18,7`), que é a parede que este tecto existe para não deixar alcançar.
pub const MAX_WAVE_LOBES: u32 = 12;

/// ⚠️ **A fracção do vão que a espessura nunca ocupa toda.** Com `t = radius − amplitude` o furo
/// fecha no eixo e o `ρ_min` do divisor vai a zero.
pub const WAVE_THICKNESS_MARGIN: f32 = 0.90;

/// ⭐ **ATÉ ONDE A ESPESSURA DA ONDA PODE IR** — o vão entre o vale da onda e o eixo, com margem.
///
/// ⭐ **É a MESMA função que o painel, a validação e a porta de escrita usam** — um painel que
/// calculasse o próprio tecto ofereceria o que o documento recusa.
#[must_use]
pub fn wave_thickness_ceiling(radius: f32, amplitude: f32) -> f32 {
    (WAVE_THICKNESS_MARGIN * (radius - amplitude)).max(0.0)
}
