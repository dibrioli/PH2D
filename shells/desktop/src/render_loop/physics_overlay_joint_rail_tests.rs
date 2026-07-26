//! **O que um TRILHO desenha** (W-J5 / W-J6b) — extraído de
//! `physics_overlay_joints_tests` pelo cap de 600 LOC da shell.
//!
//! O Slider é o único tipo cujo alcance é uma DISTÂNCIA, e é essa diferença que
//! os três gates aqui pinam: o trilho corre pelo eixo, os tracinhos ficam nos
//! fins de curso, e o arco angular — que W-J5 esqueceu de gatear — não é dele.

use super::*;

/// **O trilho de um Slider corre pelo EIXO, e os tracinhos ficam nos fins de
/// curso** (W-J5).
///
/// O oráculo é a FORMA: com o eixo em 45° a nuvem de pontos tem de se espalhar
/// nos dois eixos de tela em proporções iguais, e com o eixo horizontal a
/// dispersão vertical cai ao tamanho dos tracinhos. Um trilho que ignorasse
/// `JointView::axis` desenharia o mesmo horizontal nos dois casos.
///
/// ⚠️ Mede-se DISPERSÃO e não um ponto: os fins de curso são mundo (crescem com o
/// zoom) e os tracinhos são tela, então nenhum ponto isolado é uma coordenada
/// estável — a razão entre as duas extensões é.
#[test]
fn the_slider_rail_runs_along_its_axis() {
    let rail = |axis: [f32; 2]| {
        let mut v = view(JointKind::Slider);
        v.axis = Some(axis);
        v.limits = Some([-0.5, 0.5]);
        let pts = points_of(&marks(&v), JOINT_RGBA);
        let (xs, ys): (Vec<f64>, Vec<f64>) = pts.iter().copied().unzip();
        let span = |v: &[f64]| {
            v.iter().copied().fold(f64::MIN, f64::max) - v.iter().copied().fold(f64::MAX, f64::min)
        };
        (span(&xs), span(&ys))
    };
    let d = std::f32::consts::FRAC_1_SQRT_2;
    let (dx, dy) = rail([d, d]);
    assert!(
        (dx / dy - 1.0).abs() < 0.25,
        "um trilho a 45 graus se espalha igual nos dois eixos: {dx} x {dy}"
    );
    let (hx, hy) = rail([1.0, 0.0]);
    assert!(
        hx > hy * 3.0,
        "um trilho horizontal e largo e baixo (a altura sao os tracinhos): {hx} x {hy}"
    );
}

/// **Sem curso não há tracinhos** — eles afirmam onde o movimento PARA, e um
/// trilho ilimitado não para em lugar nenhum.
///
/// Mutação: desenhar os tracinhos sempre ⇒ o ilimitado passa a ter a mesma
/// altura do limitado e isto fica vermelho.
#[test]
fn an_unlimited_rail_has_no_end_stops() {
    let mut v = view(JointKind::Slider);
    v.axis = Some([1.0, 0.0]);
    let height = |v: &JointView| {
        let pts = points_of(&marks(v), JOINT_RGBA);
        let ys: Vec<f64> = pts.iter().map(|p| p.1).collect();
        ys.iter().copied().fold(f64::MIN, f64::max) - ys.iter().copied().fold(f64::MAX, f64::min)
    };
    v.limits = None;
    let free = height(&v);
    v.limits = Some([-0.5, 0.5]);
    let capped = height(&v);
    assert!(
        capped > free + 4.0,
        "os tracinhos de fim de curso tem de acrescentar altura: livre {free} vs \
         limitado {capped}"
    );
}

/// **Um trilho NÃO pinta o arco de limite** (W-J6b) — o arco é o envelope de uma
/// faixa ANGULAR, e o curso de um Slider é uma distância.
///
/// Quando o Slider chegou (W-J5), `JointView::limits` deixou de significar *"uma
/// faixa em radianos"* e passou a significar *"a faixa do grau de liberdade
/// livre, na unidade DO TIPO"* — e este desenho não foi avisado. O resultado é o
/// da foto do Enio: um trilho vertical com um **anel** por cima, o arco de uma
/// dobradiça a 0,5 radiano descrevendo uma articulação que não existe.
///
/// O oráculo é a CONTRIBUIÇÃO do arco, não a banda inteira: a mesma view com e
/// sem `limits`, contando os pontos apagados. Se o arco não é desenhado os dois
/// contam igual. ⚠️ A 1ª versão deste gate exigia **zero** ponto apagado e nasceu
/// vermelha sobre produto CERTO — as linhas de posse também são pintadas nessa
/// banda, então "nada apagado" nunca foi a afirmação que eu queria fazer.
///
/// O Pin ao lado é o CONTROLE, sem o qual isto seria satisfeito por um `marks`
/// que parou de pintar envelope nenhum.
///
/// Mutação: tirar o filtro `limits_in_metres` ⇒ o trilho passa a acrescentar 43
/// pontos com os limites e isto fica vermelho.
#[test]
fn a_rail_paints_no_limit_arc_because_its_range_is_a_length() {
    let dim = |v: &JointView| points_of(&marks(v), JOINT_DIM_RGBA).len();
    // Quanto a FAIXA acrescenta à banda apagada, por tipo.
    let envelope = |mut v: JointView| {
        v.limits = None;
        let bare = dim(&v);
        v.limits = Some([-0.5, 0.5]);
        dim(&v) - bare
    };

    let mut rail = view(JointKind::Slider);
    rail.axis = Some([0.0, 1.0]);
    assert_eq!(
        envelope(rail),
        0,
        "o curso de um trilho não é um arco — ele não acrescenta envelope angular"
    );

    // O CONTROLE: a MESMA faixa num Pin desenha o arco, como sempre desenhou.
    assert!(
        envelope(view(JointKind::Pin)) > 8,
        "uma dobradiça limitada tem de continuar pintando seu arco, got {}",
        envelope(view(JointKind::Pin))
    );
}

/// **O fantasma de um TRILHO desliza pelo eixo; ele não gira** (W-J6d).
///
/// O fantasma é *"onde o corpo B estaria no limite que está sendo arrastado"*, e
/// o movimento tem de ser o do grau de liberdade LIVRE. Num trilho isso é uma
/// translação: um curso é uma distância. Girando, ele era a silhueta solta da
/// foto do Enio (*"um gizmo fantasma rodando que parece não estar relacionado
/// corretamente ao joint"*) — o **quarto** leitor de `JointView::limits` que o
/// W-J5 não avisou, girando o corpo por 0,9 *radiano* para um curso de 0,9
/// *metro*.
///
/// O oráculo é geométrico e tem as duas metades: o centro anda **ao longo do
/// eixo** (o produto vetorial com ele some) a **exatamente** a distância que
/// falta até o fim de curso — e é isso que uma rotação não pode satisfazer, já
/// que ela move o corpo por um arco.
///
/// Mutação: usar o ramo angular também no trilho — RED, o centro sai 1,17 m fora
/// da reta do eixo.
#[test]
fn a_rails_ghost_slides_along_the_axis_instead_of_turning() {
    use ph2d_core::Vec2;
    use ph2d_ecs::{Name, SimWorld, Transform};
    use ph2d_physics_ecs::{Collider, ColliderShape};

    let mut sim = SimWorld::new();
    let car = sim
        .world_mut()
        .spawn((
            Name::new("Car".to_string()),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.2,
                    half_y: 0.2,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    // Trilho a +Y, âncoras coincidentes na origem: o carrinho está no zero do
    // curso, então o fantasma de um limite `t` tem de andar exatamente `t`.
    let mut v = view(JointKind::Slider);
    v.axis = Some([0.0, 1.0]);
    v.limits = Some([-0.5, 0.9]);
    v.body_b = car;
    v.anchor_a = [0.0, 0.0];
    v.anchor_b = [0.0, 0.0];
    v.centre_b = [0.0, 0.0];
    let views = [v];

    // O centro da silhueta, em MUNDO, reconstruído da caixa que ela desenha.
    let centre = |limit: f32| -> [f64; 2] {
        let p = crate::render_loop::physics_overlay_joints::limit_ghost(
            &sim,
            &views,
            Some((views[0].entity, limit)),
            &camera(),
            window(),
        )
        .expect("fantasma");
        let pts = p
            .elements()
            .iter()
            .filter_map(|e| match e {
                ph2d_vector::PathEl::MoveTo(q) | ph2d_vector::PathEl::LineTo(q) => Some(*q),
                _ => None,
            })
            .collect::<Vec<_>>();
        let n = pts.len() as f64;
        let s = pts.iter().fold((0.0, 0.0), |a, q| (a.0 + q.x, a.1 + q.y));
        // De volta a mundo: a projeção é linear, então o centroide da silhueta na
        // tela é a imagem do centroide dela no mundo.
        let w = camera().screen_to_world(((s.0 / n) as f32, (s.1 / n) as f32), window());
        [f64::from(w[0]), f64::from(w[1])]
    };

    let zero = centre(0.0);
    for limit in [0.9f32, -0.5] {
        let c = centre(limit);
        // Sobre a reta do eixo (+Y): x não muda.
        assert!(
            (c[0] - zero[0]).abs() < 1e-3,
            "limite {limit}: o fantasma saiu do eixo, x {} -> {}",
            zero[0],
            c[0]
        );
        // E andou EXATAMENTE o que falta até o fim de curso.
        assert!(
            (c[1] - zero[1] - f64::from(limit)).abs() < 1e-2,
            "limite {limit}: tinha de andar {limit} m, andou {:.4}",
            c[1] - zero[1]
        );
    }
}
