//! Os gates do **gizmo de navegação** (W49).

use super::*;

fn area() -> EditorRect {
    EditorRect::new(0.0, 0.0, 800.0, 600.0)
}

fn at(cam: &Orbit, v: Standard) -> [f32; 2] {
    balls(cam, area())
        .into_iter()
        .find(|b| b.view == v)
        .expect("as seis estão sempre lá")
        .at
}

/// ⭐⭐ **A BOLA DA VISTA EM QUE ESTAMOS FICA NO MEIO, E É A DA FRENTE.**
///
/// ⚠️ É a lei que faz o widget ser um **indicador** e não só um punhado de botões: olhar de frente
/// põe o eixo dessa vista a apontar para o observador, logo ele projeta-se no **centro** do gizmo e
/// tem a maior profundidade. Se isto falhasse, o widget diria uma orientação e a tela mostraria
/// outra — que é pior do que não ter widget nenhum.
#[test]
fn the_view_we_are_in_is_the_ball_at_the_centre_and_the_frontmost() {
    for v in Standard::ALL {
        let cam = Orbit {
            rotation: v.rotation(),
            ..Orbit::default()
        };
        let bs = balls(&cam, area());
        let c = centre(area());
        let me = bs.iter().find(|b| b.view == v).expect("ela existe");
        assert!(
            (me.at[0] - c[0]).abs() < 0.01 && (me.at[1] - c[1]).abs() < 0.01,
            "{v:?}: a bola da vista atual devia estar no centro e está em {:?}",
            me.at
        );
        assert!(
            bs.last().is_some_and(|b| b.view == v),
            "{v:?}: a bola da vista atual tem de ser a ÚLTIMA a pintar (a da frente)"
        );
        // …e a oposta é a primeira, atrás de todas.
        let opposite = Standard::ALL
            .into_iter()
            .find(|o| {
                let (a, b) = (v.eye_axis(), o.eye_axis());
                (0..3).all(|i| (a[i] + b[i]).abs() < 1.0e-5)
            })
            .expect("toda vista tem oposta");
        assert!(
            bs.first().is_some_and(|b| b.view == opposite),
            "{v:?}: a bola oposta tem de ser a primeira a pintar"
        );
    }
}

/// ⭐⭐ **O CLIQUE ESCOLHE A BOLA DA FRENTE** quando duas se sobrepõem.
///
/// ⚠️ Numa vista nomeada, a bola do eixo e a do eixo **oposto** caem exatamente no mesmo pixel — o
/// centro. A que o artista vê é a da frente, e é essa que o clique tem de dar. *A ordem de apontar é
/// a INVERSA da de desenhar*, e escrevê-las com a mesma é o defeito clássico do gizmo que responde
/// pelo eixo escondido — aqui ele levaria a câmera para o lado oposto ao que se clicou.
#[test]
fn a_click_where_two_balls_overlap_takes_the_front_one() {
    for v in Standard::ALL {
        let cam = Orbit {
            rotation: v.rotation(),
            ..Orbit::default()
        };
        let bs = balls(&cam, area());
        assert_eq!(
            pick(&bs, centre(area())),
            Some(v),
            "{v:?}: o clique no centro deu a vista escondida, não a da frente"
        );
    }
}

/// ⭐⭐ **CIMA NA TELA É CIMA NO MUNDO** — e nada media isto.
///
/// ⚠️ **Achado por uma mutação sobrevivente:** trocar o sinal do `y` da projeção espelha o widget na
/// vertical e **todos** os outros gates continuavam verdes — a bola do meio fica no meio de qualquer
/// forma, e a lei do «cabe na área» é simétrica. Um gizmo espelhado é a pior falha possível dele:
/// ele diz uma orientação e a tela mostra outra, com toda a confiança.
///
/// A régua é a vista de **frente**: dali o eixo `+Y` aponta para cima na tela, logo a bola do
/// **Topo** tem de ter `y` MENOR que o centro (o `y` da tela cresce para baixo), e a da **Base**
/// maior. O mesmo para a direita em `x`.
#[test]
fn up_on_screen_is_up_in_the_world() {
    let cam = Orbit {
        rotation: Standard::Front.rotation(),
        ..Orbit::default()
    };
    let c = centre(area());
    let top = at(&cam, Standard::Top);
    let bottom = at(&cam, Standard::Bottom);
    let right = at(&cam, Standard::Right);
    let left = at(&cam, Standard::Left);
    assert!(
        top[1] < c[1] - 1.0,
        "o TOPO devia estar ACIMA do centro e está em y={} (centro {})",
        top[1],
        c[1]
    );
    assert!(
        bottom[1] > c[1] + 1.0,
        "a BASE devia estar abaixo do centro"
    );
    assert!(
        right[0] > c[0] + 1.0,
        "a DIREITA devia estar à direita do centro e está em x={}",
        right[0]
    );
    assert!(left[0] < c[0] - 1.0, "a ESQUERDA devia estar à esquerda");
}

/// **Cada bola é apontável no sítio dela**, e o vazio entre elas não é de ninguém.
#[test]
fn each_ball_is_pickable_at_its_own_place_and_the_gap_is_nobodys() {
    let cam = Orbit::default();
    let bs = balls(&cam, area());
    for b in &bs {
        assert_eq!(
            pick(&bs, b.at),
            Some(b.view),
            "{:?} não é apontável no próprio centro",
            b.view
        );
    }
    // Fora do widget inteiro: um ponto bem longe.
    assert_eq!(pick(&bs, [10.0, 10.0]), None);
    assert!(!hits_widget(area(), [10.0, 10.0]));
    assert!(hits_widget(area(), centre(area())));
}

/// ⭐ **O widget SEGUE a câmera** — girar move as bolas.
///
/// ⚠️ Sem esta metade, tudo o que está acima passaria com um gizmo **desenhado uma vez e congelado**:
/// as posições estariam certas, o clique funcionaria, e o widget mentiria sobre a orientação a
/// partir do primeiro arrasto.
#[test]
fn the_widget_follows_the_camera() {
    let mut cam = Orbit::default();
    let before = at(&cam, Standard::Front);
    crate::field3d_input::law::orbit(&mut cam, 40.0, 0.0);
    let after = at(&cam, Standard::Front);
    assert!(
        (before[0] - after[0]).abs() + (before[1] - after[1]).abs() > 1.0,
        "orbitar não moveu a bola: {before:?} → {after:?}"
    );
}

/// ⚠️ **As bolas cabem dentro da área** — um widget metade fora da janela é metade inalcançável.
#[test]
fn the_whole_widget_fits_inside_the_area() {
    let a = area();
    for yaw in [0.0_f32, 0.7, 1.9, 3.4, 5.1] {
        let cam = Orbit::from_yaw_pitch(yaw, 0.3);
        for b in balls(&cam, a) {
            assert!(
                b.at[0] - BALL_R_PX >= 0.0
                    && b.at[1] - BALL_R_PX >= 0.0
                    && b.at[0] + BALL_R_PX <= a.w
                    && b.at[1] + BALL_R_PX <= a.h,
                "{:?} saiu da área em yaw={yaw}: {:?}",
                b.view,
                b.at
            );
        }
    }
}
