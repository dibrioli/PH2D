//! ⛔⛔⛔ **AS DEZ FERRAMENTAS DE IMAGEM — o PAINTER INCLUÍDO — têm de ser alcançáveis sem a `F9`.**
//!
//! # O defeito que este gate existe para impedir de voltar
//!
//! Elas eram pintadas num sítio **só**: `paint_image_action_row`, dentro do `paint_top_bar`. Em
//! 2026-08-30 a barra de pills saiu de cena (Enio: *«pode tirar também os botões do topo»*) e a
//! auditoria do mesmo dia mediu o resto da porta:
//!
//! | caminho | havia? |
//! |---|---|
//! | linha de menu | ❌ o menu *Window* tinha o **modo**, não as ferramentas |
//! | paleta de comandos global | ❌ projecta o trilho, os painéis e as rows dos menus-folha — nenhum dos três as contém |
//! | paleta de ferramentas do canvas | ❌ `input_dispatch` gateia-a em `hero_screen.is_none()`, o caminho de demo |
//! | atalho de teclado | ❌ nenhum handler levanta `ActivateTool` |
//!
//! ⇒ o **Painter era inalcançável**, e com ele **toda a face de pintura** da fila de ferramentas:
//! `rail_shows_painter_tools()` exige `active_tool_id == Some("painter")`, que nunca podia
//! acontecer. Os 22 `PAINTER_RAIL_*` e os dois flyouts eram código que não tinha como aparecer.
//!
//! ⚠️ **Nenhum gate do repo via isto.** Os dois irmãos deste ficheiro medem que o pill *despacha*
//! e que está *registado no store* — as duas metades certas de uma pergunta que pressupõe que
//! alguém o **pinta**. *Um controlo correcto que ninguém desenha lê-se, de fora, exactamente como
//! um controlo partido.*
//!
//! ⚠️ **Este gate mora aqui e não na `ph2d-editor-core`** porque a fila é derivada do **registry
//! de ferramentas** (`installed_registry`), que esta crate instala. Lá, `image_action_pills()`
//! cai no fallback legado de três pills e o Painter nem aparece.

use ph2d_editor::screens::hero::HeroScreen;
use ph2d_editor::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tool_registry::{Registry, hash_node_id};

fn install_boot_registry() -> &'static Registry {
    let mut reg = Registry::default();
    ph2d_tool_registry_init::register_all(&mut reg);
    reg.build().expect("o registry do boot tem de construir");
    ph2d_editor::install_registry(reg);
    ph2d_editor::installed_registry().expect("acabou de ser instalado")
}

fn painted(mode_on: bool) -> (HeroScreen, &'static Registry) {
    ph2d_editor::test_support::ensure_panel_registry();
    let reg = install_boot_registry();
    let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
    hero.image_edit.mode_on = mode_on;
    assert!(
        !hero.view.legacy_chrome,
        "precondição: o chrome legado está fora, que é o estado do produto"
    );
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    ph2d_editor::screens::hero::paint_hero_screen(
        &mut hero,
        Rect::new(0.0, 0.0, 1366.0, 1024.0),
        &mut scene,
        &mut text,
    );
    (hero, reg)
}

/// ⭐ **Com o modo ligado, cada ferramenta do cluster tem um alvo no quadro.**
#[test]
fn every_image_tool_has_a_target_in_the_painted_frame() {
    let (hero, reg) = painted(true);
    let tools = reg.cluster("image_tools");
    assert!(
        tools.len() >= 6,
        "o cluster encolheu para {} — o registry não é o do boot",
        tools.len()
    );
    let missing: Vec<&str> = tools
        .iter()
        .filter(|m| hero.hit_index.rect_for(hash_node_id(m.id)).is_none())
        .map(|m| m.id)
        .collect();
    assert!(
        missing.is_empty(),
        "ferramentas de imagem SEM alvo no quadro — inalcançáveis sem a `F9`: {missing:?}"
    );
    // ⭐ O Painter pelo nome, porque é o que carrega a face inteira da fila.
    assert!(
        hero.hit_index
            .rect_for(hash_node_id("painter"))
            .is_some_and(|r| r.w > 0.0 && r.h > 0.0),
        "o PAINTER não tem alvo: a face de pintura da fila fica inalcançável por construção"
    );
}

/// **O CONTROLO NEGATIVO: com o modo desligado elas não estão lá** — senão o teste acima passaria
/// sobre um quadro que oferece tudo sempre, e não teria medido a porta.
///
/// ⚠️ A porta é o *Window → Image Tools*, e ela é a **mesma** condição que a shell exige para
/// ACTIVAR uma delas (`Some("image_tools") => hero.image_edit.mode_on`). Oferecer o chip fora do
/// modo seria oferecer um clique que o gate a jusante recusa.
#[test]
fn with_the_mode_off_they_are_not_offered() {
    let (hero, reg) = painted(false);
    let offered: Vec<&str> = reg
        .cluster("image_tools")
        .iter()
        .filter(|m| hero.hit_index.rect_for(hash_node_id(m.id)).is_some())
        .map(|m| m.id)
        .collect();
    assert!(
        offered.is_empty(),
        "com o modo desligado a fila já oferece {offered:?} — a shell recusaria o clique"
    );
}
