//! ⭐⭐ **AS ALÇAS DO OSSO SÃO DO EDIT E DO POSE** (A14, a *Armature* do Blender): com a ferramenta
//! de osso fora da mão (Object, ou o Edit de uma forma) os ossos desenham-se sem alças, nada acende
//! sob o rato e nada se agarra — o esqueleto é um objecto e move-se pelo gizmo dele.
//!
//! ⛔ Substitui os gates de 08/09 que agarravam as alças «em todo modo de vector»: a lei deles era
//! *o que acende tem de responder*, e agora fora do modo nada acende.
//!
//! ⚠️ Textual: as três decisões vivem em fases da `App`. Controlo positivo em cada uma: a chamada
//! que a decisão governa é achada.

use std::path::PathBuf;

fn code_only(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{} não se leu: {e}", p.display()))
        .lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_hover_needs_the_bone_tool_in_hand() {
    let src = code_only("src/skeleton_app_bridge.rs");
    let corpo = src
        .split("pub fn refresh_bone_hover(")
        .nth(1)
        .expect("controlo: o realce do osso é resolvido aqui");
    let guarda = corpo
        .find(".filter(|_| self.skeleton.tool_in_hand)")
        .expect("o realce do osso deixou de pedir a ferramenta de osso na mão");
    let hover = corpo.find("hover(").expect("controlo: a porta do realce");
    assert!(guarda < hover, "o realce é resolvido antes da guarda");
}

#[test]
fn the_overlay_draws_no_handle_without_the_bone_tool() {
    let src = code_only("src/render_loop/fase_vector_bone_overlay.rs");
    assert!(src.contains("let posar = self.skeleton.tool_in_hand;"));
    let foco = src
        .find("let osso_focado = posar")
        .expect("o osso em foco não pede o modo");
    let pontas = src
        .find("let pontas = if posar")
        .expect("os anéis das pontas não pedem o modo");
    // O fundo e a curvatura dependem do osso em FOCO; os anéis entram no `draw_bones`.
    for (chamada, decisao) in [
        ("fundo_do_osso_focado(\n", foco),
        ("draw_bend(", foco),
        ("draw_bones(", pontas),
    ] {
        let at = src
            .find(chamada)
            .unwrap_or_else(|| panic!("controlo: {chamada:?} existe"));
        assert!(decisao < at, "{chamada:?} corre antes da decisão do modo");
    }
}

#[test]
fn the_select_dispatch_grabs_no_bone_handle() {
    let src = code_only("src/input_dispatch/despacho_clique_select.rs");
    assert!(
        src.contains("smart_pick_click("),
        "controlo: é o ficheiro do Select"
    );
    assert!(
        !src.contains("skeleton.bone_pose = Some("),
        "o Select voltou a agarrar alças de osso fora do Edit/Pose do esqueleto"
    );
}
