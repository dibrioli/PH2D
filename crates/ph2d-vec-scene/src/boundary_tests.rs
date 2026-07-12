//! Testes de `boundary.rs` — arquivo irmao (teto de LOC).

use super::*;

/// O afim identidade (a forma ja esta em mundo).
const ID: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// **O caso simples que tem de continuar simples.** Um retângulo de 4×2 centrado na origem:
/// saindo do centro para leste, a linha encosta em `x = 2`.
#[test]
fn the_line_stops_on_the_boundary_not_at_the_centre() {
    let r = crate::rectangle([-2.0, -1.0], [2.0, 1.0]);
    let hit = boundary_hit(&r, ID, [0.0, 0.0], [1.0, 0.0], 0.0).expect("cruza");
    assert!(
        (hit[0] - 2.0).abs() < 1e-6 && hit[1].abs() < 1e-6,
        "a linha tem de encostar na borda direita (x=2), nao no centro: {hit:?}"
    );
}

/// **A estrela — o teste que importa.**
///
/// Numa forma CÔNCAVA o raio entra e sai várias vezes. Ficar com o PRIMEIRO cruzamento prende
/// a linha numa reentrância: o conector sairia do fundo de um vale da estrela e ficaria por
/// baixo dela. É preciso o ÚLTIMO — o ponto onde o raio sai de vez.
///
/// Numa forma convexa os dois critérios coincidem, e é por isso que este erro só aparece
/// quando alguém conecta uma estrela.
#[test]
fn on_a_concave_shape_the_line_exits_at_the_tip_not_inside_a_notch() {
    // Uma estrela de 5 pontas, raio externo 2, interno 0,8 — bem côncava.
    let s = crate::star([0.0, 0.0], 2.0, 2.0, 5, 0.4);
    // A primeira ponta fica em 12h (mundo Y-para-cima). Um raio para lá tem de sair NA PONTA.
    let hit = boundary_hit(&s, ID, [0.0, 0.0], [0.0, 1.0], 0.0).expect("cruza");
    let r = hit[0].hypot(hit[1]);
    assert!(
        (r - 2.0).abs() < 0.05,
        "o raio da ponta e 2, mas a linha parou em r={r} — ela ficou presa DENTRO de um vale \
         (o criterio esta pegando o PRIMEIRO cruzamento, nao o ultimo)"
    );

    // E um raio na direção de um VALE sai no vale (r ≈ 0,8) — o menor raio possível. Isto é o
    // par do teste acima: prova que o "maior t" não é só "sempre o raio externo".
    let notch_dir = {
        // O 1º vale fica a meio passo (36°) da 1ª ponta.
        let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::PI / 5.0;
        [a.cos(), a.sin()]
    };
    let hit2 = boundary_hit(&s, ID, [0.0, 0.0], notch_dir, 0.0).expect("cruza");
    let r2 = hit2[0].hypot(hit2[1]);
    assert!(
        r2 < 1.2,
        "na direcao do VALE a linha tem de sair perto de r=0,8, nao em {r2} — o criterio \
         virou 'sempre o raio maximo', que e outro bug"
    );
}

/// A linha encosta no contorno **de verdade**, não na caixa envolvente — que é o que o draw.io
/// faz para um stencil arbitrário. Numa elipse bem achatada a diferença é gritante.
#[test]
fn the_line_touches_the_real_outline_not_the_bounding_box() {
    let e = crate::ellipse([0.0, 0.0], 4.0, 1.0);
    // Na diagonal, a bbox daria (4, 1); a elipse de verdade dá bem menos.
    let hit = boundary_hit(&e, ID, [0.0, 0.0], [1.0, 1.0], 0.0).expect("cruza");
    // Na elipse: (t/√2 / 4)² + (t/√2 / 1)² = 1 → t ≈ 1.372
    let on_ellipse = (hit[0] / 4.0).powi(2) + (hit[1] / 1.0).powi(2);
    assert!(
        (on_ellipse - 1.0).abs() < 0.01,
        "o ponto {hit:?} tem de estar SOBRE a elipse, nao na bbox dela"
    );
    assert!(
        hit[0] < 3.0,
        "a bbox daria x=4; a elipse de verdade da bem menos: {hit:?}"
    );
}

/// O `gap` afasta a linha da forma — a folga estética entre a caixa e a seta.
#[test]
fn the_gap_pulls_the_line_back_from_the_shape() {
    let r = crate::rectangle([-2.0, -1.0], [2.0, 1.0]);
    let touching = boundary_hit(&r, ID, [0.0, 0.0], [1.0, 0.0], 0.0).expect("cruza");
    let with_gap = boundary_hit(&r, ID, [0.0, 0.0], [1.0, 0.0], 0.5).expect("cruza");
    assert!(
        (touching[0] - with_gap[0] - 0.5).abs() < 1e-9,
        "o gap de 0,5 tem de recuar a linha em 0,5: {touching:?} vs {with_gap:?}"
    );
}

/// A forma pode estar **transformada** (movida, girada, escalada — o `Transform` da entidade,
/// ADR-0111). A borda é a da forma como ela aparece na tela, não a da geometria local.
#[test]
fn the_boundary_follows_the_shapes_transform() {
    let r = crate::rectangle([-1.0, -1.0], [1.0, 1.0]);
    // Escalado por 3 e movido para (10, 0): a borda direita passa a ser x = 13.
    let xf = [3.0, 0.0, 0.0, 3.0, 10.0, 0.0];
    let hit = boundary_hit(&r, xf, [10.0, 0.0], [1.0, 0.0], 0.0).expect("cruza");
    assert!(
        (hit[0] - 13.0).abs() < 1e-6,
        "a forma foi escalada x3 e movida: a borda esta em x=13, nao {hit:?}"
    );
}

/// Uma forma **aberta** (uma linha, uma espiral) não tem borda — não há por onde uma linha
/// "encostar" nela. Devolve `None`, e o chamador decide (a bbox).
#[test]
fn an_open_path_has_no_boundary_to_touch() {
    let l = crate::line([0.0, 0.0], [4.0, 0.0]);
    assert!(
        boundary_hit(&l, ID, [2.0, 5.0], [0.0, -1.0], 0.0).is_none(),
        "uma linha nao tem interior, logo nao tem borda onde encostar"
    );
}
