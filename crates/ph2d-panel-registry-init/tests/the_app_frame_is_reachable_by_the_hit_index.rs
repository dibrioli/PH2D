//! ⭐⭐⭐ **TODO CONTROLO DA MOLDURA É ALCANÇÁVEL PELO ÍNDICE DE ACERTO** — a propriedade de que a
//! porta da cena 3D depende, medida sobre o quadro real.
//!
//! # O report que o obrigou
//!
//! Enio, 2026-08-30: *«quando coloco Model, não consigo mais clicar nos menus superiores nem nas
//! abas. É como se tudo fosse canvas.»*
//!
//! ⛔ **A causa não era o módulo 3D: eram DUAS portas para a mesma pergunta.** O
//! `field3d_pointer_down` e o `sculpt3d_pointer_down` perguntavam a
//! `forwarding::cursor_over_hero_chrome`, que era **uma lista de quatro ids de fundo escrita à
//! mão**; o resto do app pergunta a `chrome_hit::pointer_over_chrome`, que consulta o **índice de
//! acerto** — o que o chrome pintou naquele quadro.
//!
//! Quando a barra de pills saiu e a barra de menus, a fila de ferramentas e as abas entraram, a
//! lista ficou com **três entradas mortas** (a barra legada só é pintada sob `F9`) e **duas
//! superfícies novas descobertas**. A cena 3D engolia o clique nelas.
//!
//! ⭐ A cura foi **apagar a segunda porta**, não completá-la. Este gate defende a propriedade de
//! que a porta sobrevivente depende: *se o chrome pinta um controlo, o índice de acerto sabe dele
//! naquele ponto* — e por isso uma faixa nova fica coberta **no dia em que é pintada**, sem
//! ninguém escrever um nome em lado nenhum.
//!
//! ⚠️ **A metade de FONTE — «os dois módulos 3D perguntam a porta certa» — vive no shell**
//! (`shells/desktop/tests/the_scene_asks_the_one_chrome_door.rs`), porque o `chrome_hit` é privado
//! do binário. As duas metades são precisas: esta afirma que **há** o que recusar, aquela que
//! alguém **pergunta**.

use ph2d_editor_core::screens::hero::{HeroScreen, slot_tabs};
use ph2d_editor_core::screens::slot::Slot;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 1024.0,
};

fn settled() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    // Dois ocupantes na coluna da direita ⇒ a fila de abas existe neste quadro.
    h.panel_visibility.insert("audio_mixer", true);
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..3 {
        ph2d_editor_core::screens::hero::paint_hero_screen(&mut h, VIEWPORT, &mut scene, &mut text);
    }
    h
}

/// A resposta da porta, reproduzida: *quem manda neste ponto — a moldura ou o desenho?*
///
/// ⚠️ Um id de **gizmo** é desenhado SOBRE a obra e não conta como moldura; é a subtileza que o
/// `chrome_hit` documenta e que uma regra de *«o índice reclamou ⇒ é UI»* estragaria.
fn frame_owns(h: &HeroScreen, x: f32, y: f32) -> bool {
    if h.store.panel_at(x, y).is_some() {
        return true;
    }
    h.hit_index
        .hit(x, y)
        .is_some_and(|id| !ph2d_editor_core::gizmo::is_gizmo_id(id))
}

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐⭐ **As três superfícies do report**, cada uma pelo rect que ela própria publicou.
#[test]
fn the_menu_bar_the_tool_bar_and_the_tabs_all_belong_to_the_frame() {
    let h = settled();
    let l = h.last_layout.expect("o quadro publicou o layout");

    let tab_bar = l.slot_tabs[Slot::RightTop as usize];
    assert!(
        tab_bar.h > 0.0,
        "a fixtura não abriu duas abas — o gate mediria uma superfície que não está no ecrã"
    );

    let mut wild = Vec::new();
    for (what, r) in [
        ("a barra de menus", l.top_bar),
        ("a fila de ferramentas", l.tool_bar),
        ("a fila de abas", tab_bar),
        ("a coluna da direita", l.inspector),
        ("a coluna da esquerda", l.hierarchy),
    ] {
        if r.w <= 0.0 || r.h <= 0.0 {
            continue; // esta banda não existe neste quadro
        }
        let (x, y) = centre(r);
        if !frame_owns(&h, x, y) {
            wild.push(format!(
                "{what} ({r:?}) — o ponto ({x:.0},{y:.0}) lê como DESENHO"
            ));
        }
    }
    assert!(
        wild.is_empty(),
        "superfícies da moldura que a cena 3D engoliria — é o report de 2026-08-30:\n  {}",
        wild.join("\n  ")
    );
}

/// ⭐ **E cada TÍTULO e cada ABA, um a um** — a banda pode ser da moldura e o controlo dentro dela
/// estar clipado para fora do índice, que é como um controlo morre em silêncio.
#[test]
fn every_menu_title_and_every_tab_answers_at_its_own_centre() {
    let mut h = settled();
    let l = h.last_layout.expect("layout");
    let mut text = TextSystem::without_system_fonts();

    let mut targets: Vec<(String, Rect)> = Vec::new();
    for (_, title, r) in ph2d_editor_core::screens::hero::menu_bar::menu_rects(l.top_bar, &mut text)
    {
        targets.push((format!("o título {title:?}"), r));
    }
    let occ = slot_tabs::occupants(&h, Slot::RightTop);
    let bar = l.slot_tabs[Slot::RightTop as usize];
    for (o, r) in occ.iter().zip(slot_tabs::tab_rects(bar, occ.len())) {
        targets.push((format!("a aba {:?}", o.title), r));
    }

    assert!(
        targets.len() >= 7,
        "só {} alvos (5 títulos + 2 abas esperados) — a fixtura mudou e o gate mediria pouco",
        targets.len()
    );

    let mut wild = Vec::new();
    for (what, r) in &targets {
        let (x, y) = centre(*r);
        if !frame_owns(&h, x, y) {
            wild.push(format!("{what} em ({x:.0},{y:.0}) lê como DESENHO"));
        }
    }
    // `h` é usado imutavelmente acima; a linha existe para o `mut` não ser um aviso.
    h.last_viewport = VIEWPORT;
    assert!(
        wild.is_empty(),
        "controlos da moldura invisíveis ao índice de acerto:\n  {}",
        wild.join("\n  ")
    );
}
