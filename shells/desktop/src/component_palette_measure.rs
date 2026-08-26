//! **A sonda da PALETA** — quanto o conteúdo pede contra o que cabe (F3 / ADR-0166).
//!
//! `cargo test -p ph2d-host-desktop --bins measure_palette -- --ignored --nocapture`
//!
//! ⚠️ Nasceu do report do Enio de 25/08 (*«a janela não tem scroll ... veja que componentes estão
//! inacessíveis fora da janela»*): com **Show all** ligado num objeto vazio a lista transbordava o
//! ecrã, e o que sobrava era inalcançável.

use ph2d_component_desc::ObjectKind;
use ph2d_editor::zones::Rect;

/// A viewport do report — o ecrã do Enio, arredondado.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1187.0,
    h: 953.0,
};

fn model(kind: ObjectKind, show_all: bool) -> ph2d_editor::widget::command_palette::PaletteModel {
    let reg = crate::init::build_component_registry();
    let can_build = |n: &str| {
        reg.get_by_id(ph2d_ecs::scene::stable_type_id(n))
            .is_some_and(|e| e.insert_default.is_some())
    };
    crate::component_palette::build(kind, &[], &can_build, show_all)
}

#[test]
#[ignore = "instrumento: imprime a tabela, nao afirma nada"]
fn measure_palette() {
    let mut ts = ph2d_text::TextSystem::new();
    println!(
        "\n  paleta de componentes — viewport {}x{}",
        VIEWPORT.w, VIEWPORT.h
    );
    println!("  ┌────────────────────────────┬────────┬──────────────┐");
    println!("  │ caso                       │ items  │ falta rolar  │");
    println!("  ├────────────────────────────┼────────┼──────────────┤");
    for (label, kind, all) in [
        ("Empty · aplicavel", ObjectKind::Empty, false),
        ("Empty · Show all", ObjectKind::Empty, true),
        ("Image · aplicavel", ObjectKind::Image, false),
        ("Image · Show all", ObjectKind::Image, true),
    ] {
        let m = model(kind, all);
        let n = m.item_count();
        let over = ph2d_editor::widget::command_palette::max_scroll(&mut ts, &m, "", VIEWPORT);
        println!("  │ {label:<26} │ {n:>6} │ {over:>9.0} px │");
    }
    println!("  └────────────────────────────┴────────┴──────────────┘");
    println!("  «falta rolar» = o que NAO cabe. Antes da rolagem, era o que ficava INALCANCAVEL.");
    println!("  baseline 2026-08-25, mesma viewport:");
    println!("    lei POSICIONAL (as pequenas a` esquerda, a grande a` direita):");
    println!("      Empty·Show all 345 px  ·  Image·Show all 143 px");
    println!("    masonry sobre as 4 unidades:");
    println!("      Empty·Show all 217 px  ·  Image·Show all  99 px   (-37% / -31%)");
    println!("  ⚠️ O masonry NAO resolve sozinho: os 217 px que sobram so' a rolagem alcanca.\n");
}
