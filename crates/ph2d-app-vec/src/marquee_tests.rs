//! **Gates da LEI DE CAPTURA do gesto de região** — irmão de `vec_marquee.rs`, o sujeito.
//!
//! Três propriedades, e nenhuma delas é sobre selecionar: são sobre o que o gesto GRAVA. Errá-las
//! não faz o laço falhar — faz o laço apanhar a região errada, que é pior porque parece funcionar.

use super::{LASSO_MIN_STEP_PX, VecMarquee};
use ph2d_tool_vector::params::MarqueeShape;

/// **A forma congela no press.** Andar não a muda — nem o gesto conhece o Ctrl depois de aberto.
///
/// Mutação que tem de sangrar: reler o modificador por movimento (o gesto passaria a morfar sob a
/// mão quando o artista larga o Ctrl a meio do arrasto).
#[test]
fn the_shape_is_frozen_at_the_press() {
    let mut m = VecMarquee::open(MarqueeShape::Lasso, (10.0, 10.0));
    for i in 0..20 {
        m.advance((10.0 + i as f32 * 5.0, 10.0));
    }
    assert_eq!(m.shape, MarqueeShape::Lasso);
    let mut b = VecMarquee::open(MarqueeShape::Box, (10.0, 10.0));
    b.advance((90.0, 90.0));
    assert_eq!(b.shape, MarqueeShape::Box);
}

/// **O laço não grava um ponto que a mão não andou** — o piso que impede um rato de 960 Hz de
/// descrever uma curva de dois píxeis com milhares de vértices.
#[test]
fn the_lasso_does_not_record_a_step_it_did_not_travel() {
    let mut m = VecMarquee::open(MarqueeShape::Lasso, (0.0, 0.0));
    // Cem movimentos MENORES que o piso, todos na mesma vizinhança.
    for i in 0..100 {
        m.advance((i as f32 * (LASSO_MIN_STEP_PX * 0.1), 0.0));
    }
    assert!(
        m.path.len() < 12,
        "o laco gravou {} pontos para um arrasto de {:.1} px — o piso nao esta' a filtrar",
        m.path.len(),
        99.0 * LASSO_MIN_STEP_PX * 0.1
    );
    // E um arrasto largo GRAVA: o piso filtra, não emudece.
    let mut far = VecMarquee::open(MarqueeShape::Lasso, (0.0, 0.0));
    for i in 1..=10 {
        far.advance((i as f32 * 50.0, 0.0));
    }
    assert_eq!(far.path.len(), 11, "o press + os dez passos largos");
}

/// **O laço fecha ONDE A MÃO SOLTOU** — a amostra que o piso recusou é promovida no release.
///
/// ⚠️ É a lição que o motor de traço do Flip pagou (*"o traço acaba onde a mão soltou"*), e aqui
/// ela decide uma SELEÇÃO: o vão entre o último ponto aceito e o dedo é uma aresta de fecho que
/// passa por onde o artista não desenhou — nós desse lado entram ou saem por acidente.
///
/// Mutação que tem de sangrar: `closed_path` devolver `self.path` cru ⇒ a ponta desaparece.
#[test]
fn the_release_promotes_the_sample_the_floor_refused() {
    let mut m = VecMarquee::open(MarqueeShape::Lasso, (0.0, 0.0));
    m.advance((100.0, 0.0));
    // Um passo curto: o piso recusa, e é ele que a soltura tem de promover.
    m.advance((100.5, 0.0));
    assert_eq!(
        m.path.last(),
        Some(&(100.0, 0.0)),
        "o piso recusou, como devia"
    );
    assert_eq!(
        m.closed_path().last(),
        Some(&(100.5, 0.0)),
        "o laco fechou no ultimo ponto ACEITO, e nao onde a mao soltou"
    );
    // E não duplica quando a última já é a de soltura.
    let mut n = VecMarquee::open(MarqueeShape::Lasso, (0.0, 0.0));
    n.advance((100.0, 0.0));
    assert_eq!(n.closed_path().len(), n.path.len());
}

/// **Um retângulo não tem caminho** — o campo existe para o laço, e a forma é quem decide, nunca a
/// presença da lista.
#[test]
fn a_box_records_no_path() {
    let mut m = VecMarquee::open(MarqueeShape::Box, (0.0, 0.0));
    for i in 0..50 {
        m.advance((i as f32 * 10.0, i as f32 * 10.0));
    }
    assert!(m.path.is_empty());
    assert_eq!(m.cur, (490.0, 490.0), "o canto vivo segue sempre");
}

/// **O Ctrl troca a forma de UM gesto** — a porta única, pura.
///
/// Mutação que tem de sangrar: `for_gesture` ignorar o `ctrl` ⇒ o laço fica inalcançável por quem
/// não abrir o painel, e a caixa inalcançável por quem deixou o chip no laço.
#[test]
fn ctrl_flips_the_sticky_shape_and_nothing_else_does() {
    use MarqueeShape::{Box, Lasso};
    assert_eq!(MarqueeShape::for_gesture(Box, false), Box);
    assert_eq!(MarqueeShape::for_gesture(Box, true), Lasso);
    assert_eq!(MarqueeShape::for_gesture(Lasso, false), Lasso);
    assert_eq!(
        MarqueeShape::for_gesture(Lasso, true),
        Box,
        "com o chip no laco, o Ctrl tem de devolver o RETANGULO — senao ele e' um modificador que \
         so' funciona num sentido, e quem deixou o chip la' perde a caixa"
    );
}
