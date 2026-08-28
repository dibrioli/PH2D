//! Os gates das alças do padrão (plano 33, W6).

use super::*;
use ph2d_vec_pattern::PatternMode;
use ph2d_vec_scene::{PatternSource, Rgba8, VecVertex};

fn shape(f: PatternFill) -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(f))),
        ..VecPath::default()
    }
}

/// ⚠️ Arte NÃO quadrada e com rotação de propósito: com `size` `[4, 4]` e ângulo `0`, os dois eixos
/// coincidem e toda aritmética errada passa.
fn fill() -> PatternFill {
    let mut f = PatternFill::new(
        PatternSource::Shape(1),
        [4.0, 1.0],
        Rgba8::new(1, 2, 3, 255),
    );
    f.origin = [2.0, 3.0];
    f
}

fn pat(path: &VecPath) -> PatternFill {
    match path.fill.as_ref() {
        Some(Paint::Pattern(p)) => (**p).clone(),
        _ => panic!("sem padrao"),
    }
}

/// **As três alças sentam nos eixos do padrão, e ROLAM com ele.**
#[test]
fn the_three_handles_sit_on_the_patterns_own_axes() {
    let f = fill();
    let [mv, sc, rot] = f.handle_points();
    assert_eq!(mv, [2.0, 3.0], "a de mover E' a origem");
    assert_eq!(sc, [6.0, 3.0], "a de escalar senta a um `size.x` no eixo X");
    assert_eq!(rot, [2.0, 4.0], "a de rodar senta a um `size.y` no eixo Y");
    // Um quarto de volta troca os eixos — as alças rolam com o padrão.
    let mut g = fill();
    g.angle = std::f64::consts::FRAC_PI_2;
    let [_, sc2, rot2] = g.handle_points();
    assert!(
        (sc2[0] - 2.0).abs() < 1e-9 && (sc2[1] - 7.0).abs() < 1e-9,
        "{sc2:?}"
    );
    assert!(
        (rot2[0] - 1.0).abs() < 1e-9 && (rot2[1] - 3.0).abs() < 1e-9,
        "{rot2:?}"
    );
}

/// ⭐ **A alça de escala escreve pela PORTA ÚNICA do tamanho** — o aspecto da arte sobrevive, tal
/// como pelo slider. A lei escrita nos dois sítios é a lei que um dia muda só num.
#[test]
fn the_scale_handle_keeps_the_arts_aspect_like_the_slider_does() {
    let mut p = shape(fill());
    assert!(drag_pattern_handle(&mut p, PatHandle::Scale, 10.0, 3.0));
    let f = pat(&p);
    assert!(
        (f.size[0] - 8.0).abs() < 1e-9,
        "o lado maior nao foi a 8: {:?}",
        f.size
    );
    assert!(
        (f.size[0] / f.size[1] - 4.0).abs() < 1e-9,
        "o aspecto 4:1 partiu: {:?}",
        f.size
    );
}

/// ⚠️ **A escala usa a PROJECÇÃO no eixo, não a distância crua.** Com a distância, arrastar
/// perpendicularmente inflaria o ladrilho sem o artista se mexer na direcção que ele vê.
#[test]
fn the_scale_handle_reads_the_projection_not_the_raw_distance() {
    let mut p = shape(fill());
    // ⚠️ A projecção tem de DIFERIR do tamanho actual, senão o arrasto é um no-op e o gate mede a
    // ausência de mudança em vez da lei. Um ponto a `7` no eixo X e MUITO longe no Y: a projecção
    // é 7, a distância crua é ~100.
    assert!(drag_pattern_handle(&mut p, PatHandle::Scale, 9.0, 103.0));
    assert!(
        (pat(&p).size[0] - 7.0).abs() < 1e-9,
        "leu a distancia crua em vez da projeccao: {:?}",
        pat(&p).size
    );
}

/// **A alça de rotação lê o ângulo do vector, descontando o quarto de volta em que ela senta.**
#[test]
fn the_rotate_handle_reads_the_angle_of_its_own_axis() {
    let mut p = shape(fill());
    // Puxar a alça de rodar para +X: o eixo Y do padrão passa a apontar para +X ⇒ −90°.
    assert!(drag_pattern_handle(&mut p, PatHandle::Rotate, 12.0, 3.0));
    assert!(
        (pat(&p).angle + std::f64::consts::FRAC_PI_2).abs() < 1e-9,
        "deu {}",
        pat(&p).angle
    );
    // ⚠️ Em cima da origem NÃO há direcção — e escrever ali daria um salto de ângulo no instante
    // em que o dedo passasse pelo centro.
    let antes = pat(&p).angle;
    assert!(!drag_pattern_handle(&mut p, PatHandle::Rotate, 2.0, 3.0));
    assert_eq!(pat(&p).angle, antes);
}

/// ⚠️⚠️ **NO `Clamp` NÃO HÁ ALÇA NENHUMA** — nem para acertar, nem para arrastar, nem para desenhar.
///
/// Ali a colocação é DERIVADA (uma cópia enquadrada na forma): `origin` e `size` não têm quem os
/// leia. Três alças que escrevem campos que ninguém lê seriam três mentiras sob o dedo — a mesma lei
/// que já esconde os knobs correspondentes no painel.
#[test]
fn the_clamp_mode_offers_no_handle_at_all() {
    let mut f = fill();
    f.mode = PatternMode::Clamp;
    let mut p = shape(f);
    assert!(
        hit_pattern_handle(&p, 2.0, 3.0, 0.4).is_none(),
        "acertou uma alca no Clamp"
    );
    assert!(
        !drag_pattern_handle(&mut p, PatHandle::Move, 9.0, 9.0),
        "arrastou no Clamp"
    );
    assert_eq!(pat(&p).origin, [2.0, 3.0], "e mexeu no documento");
    // CONTROLO: fora do Clamp as três existem. ⚠️ Raio APERTADO (`0,4`): com `1,0` a alça de mover
    // (em `2,3`) alcança a de rodar (em `2,4`) e GANHA pela precedência — a lei de desempate
    // reprovaria este controlo, que quer medir outra coisa. *Uma fixtura que deixa duas respostas
    // válidas mede o desempate, não a presença.*
    let mut q = shape(fill());
    assert_eq!(hit_pattern_handle(&q, 2.0, 3.0, 0.4), Some(PatHandle::Move));
    assert_eq!(
        hit_pattern_handle(&q, 6.0, 3.0, 0.4),
        Some(PatHandle::Scale)
    );
    assert_eq!(
        hit_pattern_handle(&q, 2.0, 4.0, 0.4),
        Some(PatHandle::Rotate)
    );
    assert!(drag_pattern_handle(&mut q, PatHandle::Move, 9.0, 9.0));
}

/// ⚠️ **Mover ganha o desempate.** Com o padrão muito pequeno as três alças caem quase no mesmo
/// sítio, e mover é o gesto que o artista quer aí — escalar e rodar exigem ver o que se faz.
#[test]
fn move_wins_the_tie_when_the_pattern_is_tiny() {
    let mut f = fill();
    f.size = [1e-3, 1e-3];
    let p = shape(f);
    assert_eq!(
        hit_pattern_handle(&p, 2.0, 3.0, 1.0),
        Some(PatHandle::Move),
        "com as tres em cima umas das outras, mover tem de ganhar"
    );
}

/// **Uma forma sem padrão não tem alça** — e o hit-test não pode inventar uma.
#[test]
fn a_shape_without_a_pattern_has_no_handles() {
    let p = VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(9, 9, 9, 255))),
        ..VecPath::default()
    };
    assert!(hit_pattern_handle(&p, 0.0, 0.0, 10.0).is_none());
}
