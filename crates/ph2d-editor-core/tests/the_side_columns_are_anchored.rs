//! **As COLUNAS LATERAIS são ANCORADAS** — Enio, 2026-08-30, com foto: *«Funciona, mas legal não
//! ficou. Acho que só fica legal depois de fixar os painéis nas laterais»*.
//!
//! # O que mudou
//!
//! O `screens/hero/paint.rs` lia o *offset* de arrasto e o *resize-delta* do Inspector e da
//! Hierarchy, clampava-os e **escrevia o resultado por cima** de `layout.inspector` /
//! `layout.hierarchy`. ⭐⭐ Esse bloco governava **dezasseis** painéis sem que nenhum soubesse:
//! quatro recebiam o rect por espelho (`bgremoval`, `padding`, `painter_sidebar`,
//! `painter_layers`) e outros doze lêem `ctx.layout.inspector` directamente, de outras crates.
//! ⇒ *arrastar o Inspector arrastava os dezasseis.*
//!
//! # ⚠️ A lei que este gate defende: as alças saem EM PAR
//!
//! Um controlo **registado no `HitIndex` cujo braço deixou de existir** é chrome morto sob o
//! dedo — a espécie que este repo varre a cada wave. As três alças (arrasto, resize BR, resize
//! BL) saíram de **três** camadas ao mesmo tempo:
//!
//! 1. o `InteractiveState::BlenderHit` do `pre_populate.rs` (o que as torna focáveis);
//! 2. o `hit_index.register` de **cada** painel que partilha o dock — e eram **quinze** crates,
//!    não duas: o `architecture_panel_wiring_parity` acusou catorze delas de uma vez, porque
//!    cada painel de *takeover* re-registava os ids do Inspector;
//! 3. o **re-registo de fim de quadro** (`close_frame_hits` no Inspector, o bloco final na
//!    Hierarchy), que existia para ganhar o z-order ao corpo do painel.
//!
//! ⭐ A camada 3 não estava no mapa que abriu a wave — quem a encontrou foi o **compilador**,
//! porque os rects eram campos de uma struct. *A metade que um grep não vê é a que viaja dentro
//! de um tipo.*

use ph2d_editor_core::screens::layout::{
    DockSides, HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroLayout, rail_w,
};
use ph2d_editor_core::zones::Rect;

const PRE_POPULATE: &str = include_str!("../src/screens/hero/pre_populate.rs");

#[path = "common/hero_sources.rs"]
mod hero_sources;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

/// **O rect que o layout calcula É o rect que a coluna ocupa** — nenhum offset entre os dois.
#[test]
fn the_dock_rects_never_pass_through_a_drag_offset() {
    // ⛔⛔ **A pergunta é ao MÓDULO, não ao ficheiro** — e a diferença não é estilo. Este gate
    // lia só o `paint.rs`; em 2026-08-30 o tecto de LOC cortou dali o bloco da geometria para o
    // irmão `frame_layout.rs`, e a partir daí a ausência que ele exige passou a ser **de graça**:
    // o offset podia voltar no ficheiro ao lado com este gate VERDE.
    //
    // ⚠️ *Um gate de AUSÊNCIA que nomeia um ficheiro é desarmado por um corte, e em silêncio* —
    // ao contrário do irmão de PRESENÇA, que reprova alto. Ver `common/hero_sources.rs`.
    for needle in [
        "blender_picker_offset(ids::INSP_PANEL)",
        "blender_picker_offset(ids::HIER_PANEL)",
        "panel_resize_delta(ids::INSP_PANEL)",
        "panel_resize_delta(ids::HIER_PANEL)",
        "clamp_panel_rect(layout.inspector",
        "clamp_panel_rect(layout.hierarchy",
    ] {
        hero_sources::assert_hero_never_contains(
            needle,
            "a coluna volta a flutuar, e leva os DEZASSEIS paineis do dock com ela",
        );
    }
    // ⭐ E o espelho continua a existir — os quatro aliases TE^M de receber o rect, senao os
    // paineis de imagem pintam-se noutro sitio que o hit-test.
    for alias in [
        "layout.bgremoval = layout.inspector;",
        "layout.padding = layout.inspector;",
        "layout.painter_sidebar = layout.inspector;",
        "layout.painter_layers = layout.inspector;",
    ] {
        assert!(
            hero_sources::hero_file_containing(alias).is_some(),
            "o alias `{alias}` sumiu — o painel dele passa a pintar num rect e a responder noutro"
        );
    }
}

/// **As alças saíram das TRÊS camadas** — e o gate lê as três, porque uma sobrevivente basta
/// para deixar um controlo morto sob o dedo.
#[test]
fn no_layer_still_arms_the_dock_drag_handles() {
    for id in [
        "INSP_DRAG_HANDLE",
        "INSP_RESIZE_HANDLE",
        "HIER_DRAG_HANDLE",
        "HIER_RESIZE_HANDLE",
    ] {
        assert!(
            !PRE_POPULATE.contains(id),
            "`{id}` ainda tem `InteractiveState` no pre_populate: focavel, e sem braco"
        );
    }
    // A varredura das crates de painel — a camada que acusou CATORZE de uma vez.
    let crates_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/");
    let mut offenders: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    for entry in std::fs::read_dir(crates_dir).expect("ler crates/") {
        let dir = entry.expect("entry").path();
        let Some(name) = dir.file_name().and_then(|n| n.to_str()).map(str::to_owned) else {
            continue;
        };
        if !name.starts_with("ph2d-panel-") {
            continue;
        }
        let mut stack = vec![dir.join("src")];
        while let Some(d) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else {
                continue;
            };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if p.extension().and_then(|x| x.to_str()) != Some("rs") {
                    continue;
                }
                let Ok(txt) = std::fs::read_to_string(&p) else {
                    continue;
                };
                scanned += 1;
                for raw in txt.lines() {
                    let l = raw.trim_start();
                    if l.starts_with("//") {
                        continue; // documentar a cura nao pode reprovar o portao
                    }
                    if l.contains(".register(")
                        && (l.contains("INSP_DRAG_HANDLE")
                            || l.contains("INSP_RESIZE_HANDLE")
                            || l.contains("HIER_DRAG_HANDLE")
                            || l.contains("HIER_RESIZE_HANDLE"))
                    {
                        offenders.push(format!("{}: {l}", p.display()));
                    }
                }
            }
        }
    }
    assert!(
        scanned > 100,
        "controlo positivo: a varredura leu {scanned} ficheiros — deixou de alcancar as crates \
         de painel e passaria a aprovar por vacuo"
    );
    assert!(
        offenders.is_empty(),
        "alcas do dock ainda registadas no HitIndex sem braco que as consuma:\n  {}",
        offenders.join("\n  ")
    );
}

/// **As colunas ENCOSTAM** — e a igualdade é a asserção, porque um `>` passa com o cartão a
/// flutuar a 14 px da borda, que é exactamente o que havia.
#[test]
fn the_columns_are_flush_in_both_orientations() {
    for mirrored in [false, true] {
        let l = HeroLayout::for_viewport_docked(
            viewport(),
            mirrored,
            rail_w(),
            ph2d_editor_core::screens::layout::CenterSplit::None,
            DockSides::BOTH,
        );
        let (left_col, right_col) = l.side_columns();
        let rail_right = l.left_rail.x + l.left_rail.w;
        assert!(
            (left_col.x - rail_right).abs() < f32::EPSILON,
            "mirrored={mirrored}: a coluna da esquerda tem de encostar no trilho \
             ({} contra {rail_right})",
            left_col.x
        );
        assert!(
            (right_col.x + right_col.w - HERO_VIEWPORT_W).abs() < 0.01,
            "mirrored={mirrored}: a coluna da direita tem de encostar na borda ({} contra {})",
            right_col.x + right_col.w,
            HERO_VIEWPORT_W
        );
    }
}
