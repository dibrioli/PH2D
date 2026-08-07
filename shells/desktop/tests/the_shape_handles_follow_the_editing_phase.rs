//! **Arch-gate: o CONTORNO da figura é desenhado nas duas fases; as ALÇAS, só na de edição.**
//!
//! ## O defeito (Enio, 2026-08-07)
//!
//! *"O gizmo está invisível ao ser criado."* Com o gesto rascunhando a tinta
//! (`ph2d-tool-painter::tool::paint::shape_draft`), o overlay virou a ÚNICA coisa na tela durante um
//! arrasto — e o `ellipse_overlay`/`polygon_overlay` devolviam `None` até o Up. O tool foi corrigido:
//! o overlay existe nas duas fases e carrega `editing`.
//!
//! ## Por que um gate de TEXTO
//!
//! A outra metade da lei mora no DESENHO, que exige janela e câmera: nenhum teste de unidade alcança o
//! `draw_ellipse_overlay`. Os gates da `ph2d-tool-painter` provam que o overlay **existe** e o que a
//! flag diz; este prova que a shell **a honra** — sem ele, alças apareceriam durante o arrasto de
//! criação, onde nenhum Down as alcança (`ellipse_down` sai por *"mid radius-drag — ignore extra
//! Downs"*), e alça desenhada que não responde é o chrome morto que este codebase recusa.
//!
//! ⚠️ **A outra metade do report — *"fica invisível ao criar outro círculo"* — NÃO precisa de gate de
//! texto:** o `StrokeOpBadge` perdeu o campo `bbox` e ganhou `outline`, então voltar a desenhar a
//! moldura AABB **não compila**. Estrutural vence disciplinar.

const OVERLAYS: &str = include_str!("../src/render_loop/painter_bridge_overlays.rs");

/// Os dois desenhadores de figura fecham a guarda de fase ANTES do laço de alças.
///
/// **Mutação que deve sangrar:** apagar qualquer um dos dois `if !overlay.editing { continue; }`.
#[test]
fn both_shape_drawers_gate_their_handles_on_the_editing_phase() {
    // A guarda tem de estar ENTRE o traço do contorno (`stroke_box`) e o laço que percorre as alças —
    // afirmar só a presença dela deixaria passar uma guarda posta depois do laço, que não guarda nada.
    for (name, fun) in [
        ("ellipse", "fn draw_ellipse_overlay("),
        ("polygon", "fn draw_polygon_overlay("),
    ] {
        let body = OVERLAYS
            .split(fun)
            .nth(1)
            .unwrap_or_else(|| panic!("`{fun}` sumiu de painter_bridge_overlays.rs"));
        let outline = body
            .find("stroke_box(scene")
            .unwrap_or_else(|| panic!("{name}: o contorno deixou de ser tracado"));
        let guard = body
            .find("if !overlay.editing {")
            .unwrap_or_else(|| panic!(
                "{name}: o laco de alcas nao e mais gateado em `overlay.editing` — elas apareceriam \
                 durante o arrasto de CRIACAO, onde nenhum Down as alcanca"
            ));
        let handles = body
            .find("overlay.handles.iter()")
            .unwrap_or_else(|| panic!("{name}: o laco de alcas sumiu"));
        assert!(
            outline < guard && guard < handles,
            "{name}: a ordem tem de ser contorno -> guarda -> alcas (veio {outline} / {guard} / {handles})"
        );
    }
}

/// Controle positivo: o arquivo lido é mesmo o dos overlays do Painter, e não um vazio que faria as
/// buscas acima passarem por vácuo.
#[test]
fn the_scanned_file_is_the_painter_overlay_drawer() {
    assert!(OVERLAYS.len() > 4_000, "o fonte lido veio curto demais");
    assert!(OVERLAYS.contains("fn draw_overlays("));
    assert!(OVERLAYS.contains("painter.ellipse_overlay()"));
    assert!(OVERLAYS.contains("painter.polygon_overlay()"));
}
