//! ⭐⭐⭐ **ONDE O MÓDULO 3D VIVE** — os gates da porta [`super::area`].
//!
//! ⚠️ **Irmão do `field3d_layout_tests`, cortado pelo tecto de LOC (657/600) em 2026-08-31**, e o
//! corte é por **responsabilidade**: aquele responde *«como a área se DIVIDE»* (o ladrilhamento, as
//! costuras, as vistas nomeadas) e este responde *«que área é essa»*. Foi o segundo que o report do
//! Enio pôs em causa — a divisão sempre esteve certa; o que estava errado era o rectângulo a que
//! ela era aplicada.

use super::{Split, rects};
use ph2d_editor::zones::Rect as EditorRect;

/// ⭐⭐⭐ **A ÁREA DO MÓDULO É A QUE SOBRA DEPOIS DAS RÉGUAS** — e o produto lê-a de lá.
///
/// # ⛔⛔⛔ O report, com duas setas (Enio, 2026-08-31)
///
/// > *«A viewport ainda não se encaixa na área correta para ela. Veja que atravessa as réguas.
/// > Tente encaixar corretamente, inclusive com as 4 viewports ao mesmo tempo.»*
///
/// O `render_loop` entregava a **JANELA INTEIRA** ao desenho, então os quatro quadrantes
/// ladrilhavam o ecrã: por baixo da barra de menus, da fila de ferramentas, da coluna da esquerda
/// e das duas réguas — e com a divisão aberta as costuras caíam onde não há área nenhuma.
///
/// # ⚠️ Ele mede QUEM ALIMENTA, e por isso as duas fontes são DIFERENTES
///
/// `last_canvas` e `last_content` recebem rects distintos de propósito: um gate que os pusesse
/// iguais passaria com o produto a ler o errado. *Um gate sobre a lei não é um gate sobre quem a
/// alimenta* — é a nota que a versão anterior desta porta já carregava.
#[test]
fn the_module_lives_inside_the_rulers_and_the_four_pieces_stay_there() {
    let viewport = EditorRect::new(0.0, 0.0, 1366.0, 1024.0);
    let mut hero =
        ph2d_editor::screens::hero::HeroScreen::new(ph2d_editor::screens::hero::ids::NodeId(1));

    // A área de desenho que um quadro real resolve, e o que sobra dela depois das réguas — pela
    // MESMA porta que o produto usa (`ruler::content`), nunca por um `- 20.0` escrito aqui.
    let drawing = ph2d_editor::zones::Rect::new(308.0, 88.0, 754.0, 900.0);
    let inner = ph2d_editor::ruler::content(drawing, true);
    hero.last_canvas = drawing;
    hero.last_content = inner;

    let area = super::area(&hero, viewport);
    assert_eq!(
        (area.x, area.y, area.w, area.h),
        (inner.x, inner.y, inner.w, inner.h),
        "o modulo recebeu a area de desenho CRUA (ou a janela) — ele volta a desenhar por baixo \
         das reguas"
    );

    // As duas faixas que ele não pode tocar, lidas da porta delas.
    let (top, left) = ph2d_editor::ruler::live_bands(drawing).expect("a area comporta reguas");
    // ⭐ **Com a divisão ABERTA**, que é o caso do report — e em várias posições da costura, porque
    // é ao arrastar que um quadrante escaparia da área.
    for (tx, ty) in [(0.5f32, 0.5f32), (0.25, 0.75), (0.75, 0.25)] {
        let r = rects(area, Split::quad().with_t(tx, ty));
        let q = r.as_slice();
        assert_eq!(q.len(), 4);
        for (i, p) in q.iter().enumerate() {
            for (name, band) in [("de cima", top), ("da esquerda", left)] {
                assert!(
                    !overlaps(*p, band),
                    "o quadrante {i} atravessa a regua {name} (t = {tx}/{ty})"
                );
            }
            assert!(
                p.x >= area.x - 0.5
                    && p.y >= area.y - 0.5
                    && p.x + p.w <= area.x + area.w + 0.5
                    && p.y + p.h <= area.y + area.h + 0.5,
                "o quadrante {i} saiu da area (t = {tx}/{ty})"
            );
        }
    }
}

/// ⭐ **Sem réguas na tela, a área é a de desenho inteira** — um recuo contra uma régua que não
/// existe seria uma faixa morta.
#[test]
fn with_the_rulers_off_the_module_takes_the_whole_drawing_area() {
    let drawing = ph2d_editor::zones::Rect::new(308.0, 88.0, 754.0, 900.0);
    let off = ph2d_editor::ruler::content(drawing, false);
    assert_eq!(
        (off.x, off.y, off.w, off.h),
        (drawing.x, drawing.y, drawing.w, drawing.h),
        "com as reguas desligadas o modulo perdeu area por nada"
    );
}

fn overlaps(a: EditorRect, b: ph2d_editor::zones::Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}
