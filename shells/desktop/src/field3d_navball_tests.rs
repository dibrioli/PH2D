//! Os gates do **gizmo de navegação** (W49).

use super::*;

fn area() -> EditorRect {
    EditorRect::new(0.0, 0.0, 800.0, 600.0)
}

/// Sem moldura nenhuma, a parte livre **é** a área — o caso base de todos os gates abaixo.
fn free() -> EditorRect {
    safe_corner(area(), &[])
}

fn at(cam: &Orbit, v: Standard) -> [f32; 2] {
    balls(cam, area(), free())
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
        let bs = balls(&cam, area(), free());
        let c = centre_in(area(), free());
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
        let bs = balls(&cam, area(), free());
        assert_eq!(
            pick(&bs, centre_in(area(), free())),
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
    let c = centre_in(area(), free());
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
    let bs = balls(&cam, area(), free());
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
    assert!(!hits_widget(area(), free(), [10.0, 10.0]));
    assert!(hits_widget(area(), free(), centre_in(area(), free())));
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
        "orbitar não moveu a bola: {before:?} -> {after:?}"
    );
}

/// ⚠️ **As bolas cabem dentro da área** — um widget metade fora da janela é metade inalcançável.
#[test]
fn the_whole_widget_fits_inside_the_area() {
    let a = area();
    for yaw in [0.0_f32, 0.7, 1.9, 3.4, 5.1] {
        let cam = Orbit::from_yaw_pitch(yaw, 0.3);
        for b in balls(&cam, a, safe_corner(a, &[])) {
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

// ───────── W50: a moldura do app empurra o gizmo ─────────

/// ⭐⭐ **UM PAINEL À DIREITA EMPURRA O GIZMO PARA A ESQUERDA; A FAIXA DO TOPO BAIXA-O.**
///
/// ⚠️ É a frase do Enio, à letra (smoke da W49): *"fica escondido entre botões. Quando houver painel
/// à direita melhor deslocar o gizmo para esquerda e abaixar um pouco para não sobrepor os botões
/// superiores."* A área que o módulo recebe é o **viewport inteiro**, e a moldura é pintada por cima
/// dele — pôr o gizmo na quina daquela área é pô-lo **debaixo** da moldura.
#[test]
fn the_chrome_pushes_the_gizmo_left_and_down() {
    let a = area();
    let bare = centre_in(a, safe_corner(a, &[]));

    // Um painel encostado à direita, da altura toda.
    let panel = EditorRect::new(a.w - 300.0, 0.0, 300.0, a.h);
    let with_panel = centre_in(a, safe_corner(a, &[panel]));
    assert!(
        (bare[0] - with_panel[0] - 300.0).abs() < 0.01,
        "o painel de 300 px devia mover o gizmo 300 px para a esquerda: {} -> {}",
        bare[0],
        with_panel[0]
    );
    assert!(
        (with_panel[1] - bare[1]).abs() < 0.01,
        "um painel à direita não pode mexer na ALTURA do gizmo"
    );

    // A faixa do topo, da largura toda.
    let bar = EditorRect::new(0.0, 0.0, a.w, 60.0);
    let with_bar = centre_in(a, safe_corner(a, &[bar]));
    assert!(
        (with_bar[1] - bare[1] - 60.0).abs() < 0.01,
        "a faixa de 60 px devia baixar o gizmo 60 px: {} -> {}",
        bare[1],
        with_bar[1]
    );
    assert!(
        (with_bar[0] - bare[0]).abs() < 0.01,
        "a faixa do topo não pode mexer na horizontal"
    );

    // As duas juntas — o caso da foto.
    let both = centre_in(a, safe_corner(a, &[panel, bar]));
    assert!(
        (both[0] - with_panel[0]).abs() < 0.01 && (both[1] - with_bar[1]).abs() < 0.01,
        "com os dois obstáculos o gizmo tem de acumular as duas folgas"
    );
}

/// ⚠️ **Um painel FLUTUANTE no meio do canvas não move o gizmo.**
///
/// Ele mudaria de sítio a cada vez que alguém arrastasse uma janela, o que é pior do que ficar
/// quieto atrás dela. A lei só conta quem toca a **aresta** da área — e é este gate que separa as
/// duas coisas.
#[test]
fn a_floating_panel_in_the_middle_does_not_move_the_gizmo() {
    let a = area();
    let bare = centre_in(a, safe_corner(a, &[]));
    let floating = EditorRect::new(200.0, 200.0, 250.0, 200.0);
    let after = centre_in(a, safe_corner(a, &[floating]));
    assert_eq!(bare, after, "um painel flutuante moveu o gizmo");
}

/// ⚠️ **E com a moldura o widget continua DENTRO da área** — a lei da W49, sob as folgas novas.
#[test]
fn the_widget_still_fits_with_the_chrome_in_the_way() {
    let a = area();
    let safe = safe_corner(
        a,
        &[
            EditorRect::new(a.w - 300.0, 0.0, 300.0, a.h),
            EditorRect::new(0.0, 0.0, a.w, 60.0),
        ],
    );
    for yaw in [0.0_f32, 0.9, 2.4, 4.6] {
        let cam = Orbit::from_yaw_pitch(yaw, 0.3);
        for b in balls(&cam, a, safe) {
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

/// ⚠️ **Uma faixa em BAIXO não move o gizmo** — ele vive em cima, e a tira de quadros do Flip é
/// larga e encostada à direita. *Uma lei que só olhasse «toca a aresta direita» empurraria o gizmo
/// pela largura inteira da janela por causa dela.*
#[test]
fn a_bottom_strip_does_not_move_the_gizmo() {
    let a = area();
    let bare = centre_in(a, safe_corner(a, &[]));
    let strip = EditorRect::new(0.0, a.h - 120.0, a.w, 120.0);
    assert_eq!(
        centre_in(a, safe_corner(a, &[strip])),
        bare,
        "a tira de baixo moveu o gizmo, que vive em cima"
    );
}

/// ⚠️ **A ordem dos obstáculos não muda o resultado** — a lei é iterativa, e uma lei iterativa que
/// dependesse da ordem daria um gizmo que salta de sítio conforme o painel que abriu primeiro.
#[test]
fn the_order_of_the_obstacles_does_not_matter() {
    let a = area();
    let panel = EditorRect::new(a.w - 300.0, 0.0, 300.0, a.h);
    let bar = EditorRect::new(0.0, 0.0, a.w, 60.0);
    assert_eq!(
        centre_in(a, safe_corner(a, &[panel, bar])),
        centre_in(a, safe_corner(a, &[bar, panel])),
    );
}

/// ⭐⭐ **UMA COLUNA DOCADA JÁ NÃO EMPURRA O GIZMO** — a fuga ficou inerte por construção.
///
/// A **D1** manda retirar a fuga quando os painéis passam a ser regiões irmãs: *«ela é o remédio do
/// sintoma; com os painéis fora da vista passaria a fugir de uma moldura que já não a alcança»*.
///
/// ⚠️ **A cura não foi apagar a lei — foi dar-lhe a ÁREA CERTA.** Ela recebia o viewport inteiro,
/// que as colunas docadas tocam; hoje recebe a `HeroLayout::draw_area`, que **começa depois delas**.
/// A lei fica, porque o que ainda a alcança são as janelas que declaram flutuar (Grid Snap,
/// galeria), e sem ela o gizmo ficaria por baixo de uma dessas.
///
/// ⛔ **O controlo é a metade que importa:** os MESMOS obstáculos, medidos contra a área ANTIGA,
/// **têm** de mover o gizmo. Sem ele este teste passaria com a lei apagada, com a área a zero, ou
/// com obstáculos que não tocam nada — três formas de medir coisa nenhuma.
#[test]
fn a_docked_column_no_longer_pushes_the_gizmo() {
    // A geometria REAL do quadro, e não rectângulos inventados.
    let viewport = ph2d_editor::zones::Rect::new(0.0, 0.0, 1366.0, 1024.0);
    let layout = ph2d_editor::screens::layout::HeroLayout::for_viewport_bands(
        viewport,
        false,
        ph2d_editor::screens::layout::ChromeBands {
            rail_w: 0.0,
            top_bar_h: 28.0,
            ..ph2d_editor::screens::layout::ChromeBands::DEFAULT
        },
        ph2d_editor::screens::layout::CenterSplit::None,
        ph2d_editor::screens::layout::DockSides::BOTH,
    );
    let to_editor = |r: ph2d_editor::zones::Rect| EditorRect::new(r.x, r.y, r.w, r.h);
    let columns = [to_editor(layout.hierarchy), to_editor(layout.inspector)];
    let draw = to_editor(layout.draw_area);

    let free_now = safe_corner(draw, &columns);
    assert_eq!(
        centre_in(draw, free_now),
        centre_in(draw, safe_corner(draw, &[])),
        "uma coluna DOCADA continua a empurrar o gizmo — a fuga virou remédio duplo"
    );

    // ⛔ O CONTROLO: a área de antes (a janela inteira) É empurrada pelos mesmos rectângulos.
    let whole = to_editor(viewport);
    assert_ne!(
        centre_in(whole, safe_corner(whole, &columns)),
        centre_in(whole, safe_corner(whole, &[])),
        "o controlo caiu: nem a área ANTIGA é movida por estas colunas, então o teste acima não \
         mede a cura"
    );
}

/// **E uma janela FLUTUANTE encostada à direita continua a empurrar** — é para isso que a lei fica.
#[test]
fn a_floating_window_on_the_edge_still_pushes() {
    let a = area();
    let floating = EditorRect::new(a.w - 240.0, 0.0, 240.0, 320.0);
    assert_ne!(
        centre_in(a, safe_corner(a, &[floating])),
        centre_in(a, safe_corner(a, &[])),
        "a fuga deixou de funcionar para quem DECLARA flutuar — apagá-la era isto"
    );
}

/// ⭐⭐ **E a ÁREA que o produto entrega é a de DESENHO** — o gate sobre quem ALIMENTA a lei.
///
/// ⛔⛔ **Sem ele, a mutação que devolve a janela ao produto SOBREVIVE.** Foi medido: os dois gates
/// acima passam a lei à mão, então eles ficam verdes com o produto a alimentar o rectângulo errado.
/// *Um gate sobre a lei não é um gate sobre quem a alimenta* — a mesma família do
/// `the_chrome_swallows_the_click_it_was_given`, que afirmava que todos PERGUNTAM e nunca que
/// alguém RESPONDE.
#[test]
fn the_product_feeds_the_gizmo_the_drawing_area() {
    let viewport = EditorRect::new(0.0, 0.0, 1366.0, 1024.0);
    let mut hero =
        ph2d_editor::screens::hero::HeroScreen::new(ph2d_editor::screens::hero::ids::NodeId(1));

    // (a) com um quadro publicado, a área é a DE DESENHO.
    let drawing = ph2d_editor::zones::Rect::new(308.0, 28.0, 754.0, 996.0);
    hero.last_content = drawing;
    let got = crate::field3d_layout::area(&hero, viewport);
    assert_eq!(
        (got.x, got.y, got.w, got.h),
        (drawing.x, drawing.y, drawing.w, drawing.h),
        "o gizmo recebeu a janela em vez da área de desenho — a fuga volta a ser remédio duplo"
    );

    // (b) no PRIMEIRO quadro ainda não há área publicada, e aí vale a janela.
    hero.last_content = ph2d_editor::zones::Rect::new(0.0, 0.0, 0.0, 0.0);
    let got = crate::field3d_layout::area(&hero, viewport);
    assert_eq!(
        (got.w, got.h),
        (viewport.w, viewport.h),
        "sem quadro publicado o gizmo ficou com uma área degenerada — ele sai da janela"
    );
}
