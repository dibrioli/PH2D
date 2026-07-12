//! O catálogo de **apresentação** do conector — o par de `shapes` para a linha que gruda
//! em duas formas.
//!
//! Três campos, e a mesma mecânica dos parâmetros de forma: um [`FieldDesc`] descreve
//! rótulo, faixa, passo e unidade; o painel itera e desenha; a shell clampa e aplica. A
//! seção Connector é irmã da de parâmetros de forma — de propósito.
//!
//! # Por que a DERIVAÇÃO AUTOMÁTICA mora aqui, e não só no cozimento
//!
//! `jetty` e `spread` são `Option<f32>` no [`VecConnector`]: `None` = automático (o jetty
//! acompanha o tamanho das caixas; o spread, o índice paralelo), `Some(v)` = o usuário
//! fixou no painel. O campo do painel, porém, tem de mostrar o valor **EFETIVO** — o
//! automático, enquanto ninguém fixou nada. Um campo que mostrasse `0` (ou vazio) faria o
//! slider SALTAR no primeiro toque, longe da linha que o olho vê.
//!
//! Logo há dois leitores do mesmo número: quem **coze** a rota e quem a **exibe**. Se cada
//! um tiver a sua cópia da fórmula, o painel mente no dia em que alguém mexer numa delas —
//! é a lição `feedback_derived_coordinate_seed_must_match_sample` (o tempo remapeado quebrou
//! três vezes assim). Por isso [`auto_jetty`] + [`SPREAD_STEP`] vivem AQUI, num lugar que
//! painel e shell (e o cozimento) importam: **semente e amostra saem da mesma função**.
//!
//! [`VecConnector`]: (o componente ECS, `ph2d_ecs::VecConnector` — esta crate não depende
//! da ECS: recebe as meias-extensões das caixas, que é tudo de que a fórmula precisa)

use crate::shapes::{FieldDesc, FieldUnit};

/// O **jetty automático**, como fração da menor meia-extensão da MAIOR das duas caixas.
/// Relativo, e não em unidades fixas: no PH2D uma forma tem ~1-2 unidades de mundo, mas
/// nada impede uma de ter 50 — um jetty absoluto ficaria invisível numa e gigante na outra.
pub const JETTY_K: f64 = 0.35;
/// Piso do jetty automático (duas pontas soltas não têm caixa de onde derivá-lo).
pub const JETTY_MIN: f64 = 0.08;
/// Teto do jetty automático.
pub const JETTY_MAX: f64 = 1.0;

/// O passo do **spread automático**: o afastamento por `parallel_index`. Dois conectores no
/// MESMO par de formas não podem se sobrepor — o segundo sumiria exatamente por baixo do
/// primeiro, e o usuário juraria que ele não foi criado.
pub const SPREAD_STEP: f64 = 0.35;

/// **O jetty automático das duas caixas** — a fórmula que o cozimento usa quando
/// `VecConnector::jetty` é `None`, e que o painel exibe como valor efetivo.
///
/// Recebe as **meias-extensões** `(hw, hh)` de cada ponta em MUNDO (uma ponta solta é um
/// ponto: `(0, 0)` — o piso do clamp cuida dela).
#[must_use]
pub fn auto_jetty(a_half: (f64, f64), b_half: (f64, f64)) -> f64 {
    let smallest = |(hw, hh): (f64, f64)| hw.min(hh);
    (JETTY_K * smallest(a_half).max(smallest(b_half))).clamp(JETTY_MIN, JETTY_MAX)
}

/// Os nomes das rotas, na ordem do discriminante de `ph2d_ecs::RouteKind`
/// (`Straight = 0`, `Orthogonal = 1`) — o botão de escolha CICLA por eles.
pub const ROUTE_NAMES: &[&str] = &["Straight", "Orthogonal"];

/// **Route** — como a linha é desenhada. Uma escolha, não um número: "Route: 1" não diz
/// nada; "Route: Orthogonal" diz tudo (a mesma regra do ponto de vista dos sólidos).
pub const ROUTE: FieldDesc = FieldDesc {
    label: "Route",
    min: 0.0,
    max: 1.0,
    step: 1.0,
    unit: FieldUnit::Choice(ROUTE_NAMES),
};

/// **Jetty** — o quanto a linha avança reta antes de poder dobrar (o que a faz "sair para
/// cima" antes de virar, em vez de dobrar colada na caixa). Em unidades de MUNDO: o teto
/// (2.0) é o dobro do teto do automático, então dá para exagerar de propósito; o piso é
/// pequeno mas não-zero (jetty zero seria "dobre em cima da forma").
pub const JETTY: FieldDesc = FieldDesc {
    label: "Jetty",
    min: 0.02,
    max: 2.0,
    step: 0.01,
    unit: FieldUnit::Ratio,
};

/// **Spread** — o deslocamento que separa dois conectores no mesmo par de caixas. **Com
/// sinal**: o feixe automático alterna os lados da face (`0, +1, −1, …`), então fixar um
/// valor negativo é tão útil quanto um positivo.
pub const SPREAD: FieldDesc = FieldDesc {
    label: "Spread",
    min: -2.0,
    max: 2.0,
    step: 0.01,
    unit: FieldUnit::Ratio,
};

/// **Corner** — o raio das QUINAS DO PERCURSO (as dobras do cotovelo), em unidades de
/// mundo. `0` = afiado: o default do fluxograma clássico, e a resposta certa na maioria
/// dos diagramas — por isso não é `Option` como o jetty e o spread (não existe um "raio
/// automático" que faça sentido; zero já é ele).
///
/// Não confundir com o arredondamento da SETA (`StrokeSpec::marker_round`): um alisa a
/// dobra do caminho, o outro alisa as quinas da própria ponta.
pub const CORNER: FieldDesc = FieldDesc {
    label: "Corner",
    min: 0.0,
    max: 1.0,
    step: 0.01,
    unit: FieldUnit::Ratio,
};

/// Clampa um valor autorado à faixa do seu campo — a mesma porta para o painel (que
/// desenha) e para a shell (que aplica). Delega ao catálogo: um clamp só, para campos de
/// forma, de conector e de ponta (senão as três faixas divergiriam).
#[must_use]
pub fn clamp_to(f: &FieldDesc, v: f64) -> f64 {
    crate::shapes::clamp_to(f, v)
}

/// A **próxima** rota (o clique cicla). Espelho de `shapes::next_choice`, mas sobre o
/// discriminante do `RouteKind` — o valor continua um `f64` na fronteira do painel.
#[must_use]
pub fn next_route(current: f64) -> f64 {
    let n = ROUTE_NAMES.len().max(1) as f64;
    (current.round() + 1.0).rem_euclid(n)
}

/// O rótulo da rota corrente (o que o botão mostra).
#[must_use]
pub fn route_label(current: f64) -> &'static str {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o indice e clampado a [0, n-1] antes do cast"
    )]
    let i = current.round().clamp(0.0, (ROUTE_NAMES.len() - 1) as f64) as usize;
    ROUTE_NAMES.get(i).copied().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O jetty automático acompanha o TAMANHO** — é a razão de ele ser `Option` e não um
    /// `f32` com default. Uma forma minúscula e uma enorme não podem receber o mesmo avanço.
    #[test]
    fn the_automatic_jetty_follows_the_size_of_the_bigger_box() {
        let small = auto_jetty((0.5, 0.5), (0.5, 0.5));
        let big = auto_jetty((0.5, 0.5), (2.0, 2.0));
        assert!(
            big > small,
            "a caixa MAIOR e que dita o jetty: {big} deveria passar de {small}"
        );
        // A maior manda: a ordem das pontas não muda nada.
        assert!((auto_jetty((2.0, 2.0), (0.5, 0.5)) - big).abs() < 1e-12);
    }

    /// Duas pontas SOLTAS não têm caixa de onde derivar nada — o piso salva a rota (jetty
    /// zero seria "dobre exatamente em cima do ponto").
    #[test]
    fn two_free_ends_fall_back_to_the_floor_instead_of_zero() {
        assert!((auto_jetty((0.0, 0.0), (0.0, 0.0)) - JETTY_MIN).abs() < 1e-12);
        // E uma caixa gigante satura no teto (senão a linha daria a volta no mundo).
        assert!((auto_jetty((99.0, 99.0), (0.0, 0.0)) - JETTY_MAX).abs() < 1e-12);
    }

    /// A menor meia-extensão manda dentro de UMA caixa: uma forma larga e chata avança
    /// pouco (senão o jetty sairia maior que a própria altura dela).
    #[test]
    fn a_flat_wide_box_is_measured_by_its_short_side() {
        let flat = auto_jetty((10.0, 0.5), (0.0, 0.0));
        assert!(
            (flat - JETTY_K * 0.5).abs() < 1e-12,
            "jetty = k * min(hw, hh)"
        );
    }

    /// O botão de rota CICLA e volta ao começo — e o rótulo nunca sai vazio (um botão em
    /// branco é um botão que ninguém clica).
    #[test]
    fn the_route_button_cycles_and_always_has_a_label() {
        assert!((next_route(0.0) - 1.0).abs() < f64::EPSILON);
        assert!(
            next_route(1.0).abs() < f64::EPSILON,
            "de Orthogonal volta a Straight"
        );
        assert_eq!(route_label(0.0), "Straight");
        assert_eq!(route_label(1.0), "Orthogonal");
        // Um valor fora da faixa (save corrompido) não pode pintar vazio nem entrar em pânico.
        assert_eq!(route_label(7.0), "Orthogonal");
    }

    /// Os campos numéricos declaram faixas que o clamp respeita — é ela que a caixa do
    /// painel registra (`set_number_range`), e sem ela o arrasto escalaria errado.
    #[test]
    fn the_numeric_fields_clamp_to_their_declared_range() {
        assert!((clamp_to(&JETTY, 99.0) - JETTY.max).abs() < f64::EPSILON);
        assert!((clamp_to(&JETTY, -1.0) - JETTY.min).abs() < f64::EPSILON);
        // O spread é COM SINAL: um negativo é legítimo e não pode ser clampado a zero.
        assert!((clamp_to(&SPREAD, -0.5) + 0.5).abs() < f64::EPSILON);
    }
}
