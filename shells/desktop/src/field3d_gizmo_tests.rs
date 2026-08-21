//! Os gates do gizmo 3D.
//!
//! ⚠️ **Nenhum deles abre uma janela**, e é isso que os torna escrevíveis. A lei é pura: entram uma
//! âncora e uma câmera, sai onde as alças ficam e quanto o arrasto vale. O que um smoke ainda tem de
//! dizer é se o *feel* está certo; o que estes dizem é que o gesto **faz o que promete**.

use super::*;
use ph2d_field_render::{Orbit, Screen};

const W: u32 = 800;
const H: u32 = 600;

fn cam() -> Orbit {
    Orbit::default()
}

fn screen(c: &Orbit) -> Screen {
    Screen::new(W, H, c.half_extent)
}

fn anchor() -> Anchor {
    Anchor::global(1, [0.0, 0.0, 0.0])
}

fn handles(c: &Orbit, mode: Mode) -> Vec<Projected> {
    project(anchor(), c, screen(c), mode)
}

fn of(hs: &[Projected], want: Handle) -> Projected {
    hs.iter()
        .find(|h| h.handle == want)
        .unwrap_or_else(|| panic!("o gizmo não projetou {want:?}"))
        .clone()
}

fn arrow_of(hs: &[Projected], n: usize) -> ([f32; 2], [f32; 2]) {
    match of(hs, Handle::Axis(n)).shape {
        Shape::Arrow { from, to } => (from, to),
        other => panic!("um eixo é uma seta, e veio {other:?}"),
    }
}

fn translation(m: Motion) -> [f32; 3] {
    match m {
        Motion::Translate(d) => d,
        other => panic!("esperava uma translação e veio {other:?}"),
    }
}

fn angle_of(m: Motion) -> f32 {
    match m {
        Motion::Rotate { angle, .. } => angle,
        other => panic!("esperava uma rotação e veio {other:?}"),
    }
}

fn factor_of(m: Motion) -> f32 {
    match m {
        Motion::Scale(f) => f,
        other => panic!("esperava uma escala e veio {other:?}"),
    }
}

// ─────────────────────────────── MOVER ───────────────────────────────

/// ⭐ **O gizmo tem tamanho de TELA**, como o do Blender — o braço mede o mesmo em qualquer zoom.
///
/// ⚠️ Um gizmo de tamanho de mundo fica maior do que a janela ao aproximar e desaparece ao afastar,
/// e é a mesma peça que se está a manipular nos dois casos. A prova é sobre o eixo perpendicular à
/// vista: os outros dois encurtam por projeção, e é suposto.
#[test]
fn the_arm_is_the_same_length_on_screen_at_every_zoom() {
    // Vista de frente: o X e o Y ficam no plano da tela e projetam o braço inteiro.
    let mut c = Orbit::from_yaw_pitch(0.0, 0.0);
    for zoom in [0.05f32, 0.8, 3.5] {
        c.half_extent = zoom;
        let hs = handles(&c, Mode::Move);
        for axis in [0usize, 1] {
            let (from, to) = arrow_of(&hs, axis);
            let len = (to[0] - from[0]).hypot(to[1] - from[1]);
            assert!(
                (len - ARM_PX).abs() < 0.5,
                "com half_extent {zoom} o braço {axis} mediu {len} px em vez de {ARM_PX}"
            );
        }
    }
}

/// ⭐ **Arrastar uma seta move o nó ao longo daquele eixo, e mais nada.**
#[test]
fn dragging_an_axis_moves_along_that_axis_only() {
    let c = cam();
    let s = screen(&c);
    let hs = handles(&c, Mode::Move);
    for n in 0..3 {
        let (from, to) = arrow_of(&hs, n);
        // Arrasta 40 px NA DIREÇÃO da seta, na tela.
        let d = [to[0] - from[0], to[1] - from[1]];
        let len = d[0].hypot(d[1]);
        let m = [d[0] / len * 40.0, d[1] / len * 40.0];
        let delta = translation(drag(Handle::Axis(n), anchor(), &c, s, [0.0, 0.0], m));

        for k in 0..3 {
            if k == n {
                assert!(
                    delta[k] > 0.0,
                    "arrastar na direção da seta {n} tem de andar para a frente nela, e deu {delta:?}"
                );
            } else {
                assert!(
                    delta[k].abs() < 1e-6,
                    "o eixo {n} escorregou para o {k}: {delta:?}"
                );
            }
        }
    }
}

/// ⭐ **O número do arrasto é o que a tela mostra**: mover o rato o comprimento projetado do braço
/// anda exatamente um braço no mundo.
///
/// ⚠️ É a afirmação que separa "move na direção certa" de "move a quantidade certa". Um fator errado
/// aqui passa despercebido num gate de direção e é o que se sente como *"a peça foge da mão"*.
#[test]
fn one_arm_of_mouse_is_one_arm_of_world() {
    let c = cam();
    let s = screen(&c);
    let arm_world = ARM_PX / s.px_per_world();
    let (from, to) = arrow_of(&handles(&c, Mode::Move), 0);
    let m = [to[0] - from[0], to[1] - from[1]];
    let delta = translation(drag(Handle::Axis(0), anchor(), &c, s, [0.0, 0.0], m));
    assert!(
        (delta[0] - arm_world).abs() < arm_world * 1e-3,
        "o braço mede {arm_world} de mundo e o arrasto andou {}",
        delta[0]
    );
}

/// ⚠️ **Uma seta apontada ao observador não é uma alça** — e o gate mede as duas metades: ela não é
/// pintada, e arrastá-la não faz nada.
///
/// Sem isto, um pixel de rato valeria um salto arbitrário: a conta divide pelo comprimento
/// projetado, que ali tende a zero. O sintoma seria a peça a desaparecer da janela num toque.
#[test]
fn an_axis_that_points_at_the_camera_is_not_a_handle() {
    // De frente: o eixo Z aponta ao observador e projeta-se em nada.
    let c = Orbit::from_yaw_pitch(0.0, 0.0);
    let hs = handles(&c, Mode::Move);

    assert!(
        !of(&hs, Handle::Axis(2)).live,
        "o eixo Z está de frente para a câmera e continua pintado"
    );
    assert_eq!(
        translation(drag(
            Handle::Axis(2),
            anchor(),
            &c,
            screen(&c),
            [0.0, 0.0],
            [500.0, 500.0]
        )),
        [0.0; 3],
        "uma alça que não se pode ver não pode arrastar"
    );
    // ⭐ E o gesto não fica sem saída: o quadrado do plano perpendicular a ela está de FRENTE, que é
    // exatamente o que aquele enquadramento pede.
    assert!(of(&hs, Handle::Plane(2)).live);
}

/// **O centro é do disco de vista**, e não de um eixo à sorte.
///
/// ⚠️ A folga central existe para isto: sem ela as três hastes disputariam o mesmo pixel e quem
/// ganhasse dependia da ordem da lista, não da geometria.
#[test]
fn the_centre_belongs_to_the_view_disc() {
    let c = cam();
    let hs = handles(&c, Mode::Move);
    let (o2, _) = c
        .project(anchor().origin, screen(&c))
        .expect("a fixture olha a peça");
    assert_eq!(pick(&hs, o2), Some(Handle::View));
    // E um pixel logo ao lado do centro também: a folga tem raio, não é um ponto.
    assert_eq!(
        pick(&hs, [o2[0] + INNER_PX * 0.5, o2[1]]),
        Some(Handle::View)
    );
}

/// **Apontar o meio de uma haste escolhe aquela seta**, e o vazio não escolhe nada.
#[test]
fn pointing_at_a_shaft_picks_that_axis_and_empty_space_picks_nothing() {
    let c = cam();
    let hs = handles(&c, Mode::Move);
    for n in 0..3 {
        if !of(&hs, Handle::Axis(n)).live {
            continue;
        }
        let (from, to) = arrow_of(&hs, n);
        let mid = [(from[0] + to[0]) * 0.5, (from[1] + to[1]) * 0.5];
        assert_eq!(
            pick(&hs, mid),
            Some(Handle::Axis(n)),
            "o meio da haste {n} tem de ser dela"
        );
    }
    let (o2, _) = c
        .project(anchor().origin, screen(&c))
        .expect("a fixture olha a peça");
    assert_eq!(
        pick(&hs, [o2[0] + ARM_PX * 3.0, o2[1] + ARM_PX * 3.0]),
        None,
        "longe do gizmo não é de ninguém — senão o clique de selecionar viraria um arrasto"
    );
}

/// ⭐ **Um quadrado de plano move NO plano dele e nunca ao longo da normal.**
#[test]
fn a_plane_handle_never_moves_along_its_normal() {
    let c = cam();
    let s = screen(&c);
    let hs = handles(&c, Mode::Move);
    for n in 0..3 {
        if !of(&hs, Handle::Plane(n)).live {
            continue;
        }
        let delta = translation(drag(
            Handle::Plane(n),
            anchor(),
            &c,
            s,
            [10.0, 10.0],
            [90.0, 130.0],
        ));
        assert!(
            delta[n].abs() < 1e-4,
            "o plano perpendicular a {n} andou {} ao longo da própria normal",
            delta[n]
        );
        let in_plane = delta[(n + 1) % 3].abs() + delta[(n + 2) % 3].abs();
        assert!(in_plane > 1e-3, "e ele tem de andar de facto: {delta:?}");
    }
}

/// **O disco de vista nunca degenera** — é a rede de segurança de todo enquadramento.
#[test]
fn the_view_handle_works_from_every_angle() {
    for (yaw, pitch) in [(0.0, 0.0), (0.72, 0.52), (2.1, -1.4), (0.0, 1.5)] {
        let c = Orbit::from_yaw_pitch(yaw, pitch);
        assert!(of(&handles(&c, Mode::Move), Handle::View).live);
        let d = translation(drag(
            Handle::View,
            anchor(),
            &c,
            screen(&c),
            [0.0, 0.0],
            [30.0, -20.0],
        ));
        let n = d[0].abs() + d[1].abs() + d[2].abs();
        assert!(
            n > 1e-3 && n.is_finite(),
            "de ({yaw}, {pitch}) o disco de vista não moveu nada: {d:?}"
        );
    }
}

/// ⚠️ **O quadrilátero é testado por produto vetorial**, e não por caixa alinhada: um quadrado do
/// mundo projeta-se como um losango, e uma caixa reclamaria pixels do vizinho — exatamente nos
/// cantos onde as três alças de plano se tocam.
#[test]
fn a_projected_plane_is_a_rhombus_not_a_box() {
    let c = cam();
    let hs = handles(&c, Mode::Move);
    let h = of(&hs, Handle::Plane(2));
    let Shape::Quad(q) = h.shape else {
        panic!("um plano é um quadrilátero");
    };
    assert!(h.live);
    let mid = [(q[0][0] + q[2][0]) * 0.5, (q[0][1] + q[2][1]) * 0.5];
    assert_eq!(pick(&hs, mid), Some(Handle::Plane(2)));

    // E um canto da caixa envolvente que está FORA do losango não é dele. Se não houver nenhum, o
    // losango era um retângulo alinhado e o gate não teria nada a dizer — então isso é reprovação.
    let xs = q.iter().map(|p| p[0]);
    let ys = q.iter().map(|p| p[1]);
    let (x0, x1) = (
        xs.clone().fold(f32::INFINITY, f32::min),
        xs.fold(f32::NEG_INFINITY, f32::max),
    );
    let (y0, y1) = (
        ys.clone().fold(f32::INFINITY, f32::min),
        ys.fold(f32::NEG_INFINITY, f32::max),
    );
    let outside = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
        .into_iter()
        .filter(|c| pick(&hs, *c) != Some(Handle::Plane(2)))
        .count();
    assert!(
        outside > 0,
        "nenhum canto da caixa envolvente ficou de fora — o teste de losango não está a ser exercido"
    );
}

/// Os gates do **arrasto** vivem no irmão — a lei que eles medem também
/// ([`crate::field3d_gizmo::drag`]).
#[path = "field3d_gizmo_drag_tests.rs"]
mod drag;
