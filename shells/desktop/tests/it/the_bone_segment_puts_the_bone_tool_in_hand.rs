//! ⭐⭐ **UM SEGMENTO DO VERBO DO OSSO PÕE A FERRAMENTA DE OSSO NA MÃO** antes de o clique chegar à
//! ferramenta activa (A14). O evento do painel vai para a ferramenta ACTIVA; com outra na mão o
//! *Create*/*Transform*/*Weight* morria nela — o osso só se mexia no Edit do vetor (report do dono,
//! 05/10). A ponte em si tem gate em `ph2d-app-skeleton` (`bone_bridge_tests`).
//!
//! ⚠️ Textual: a ordem é de chamadas numa fase do quadro, que precisa da `App`.

use std::path::PathBuf;

#[test]
fn the_bone_segment_arms_the_tool_before_the_forward() {
    let p =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/render_loop/fase_bus_tool_panel.rs");
    let src: String = std::fs::read_to_string(&p)
        .expect("a fase do canal painel → ferramenta existe")
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let forward = src
        .find("t.handle_panel_event(ev)")
        .expect("controlo: o evento segue para a ferramenta activa");
    let segmento = src
        .find("BoneAction::of_segment(")
        .expect("o segmento do verbo não é reconhecido antes do envio");
    let arm = src
        .find("bone_bridge::arm(")
        .expect("o segmento não põe a ferramenta de osso na mão");
    assert!(
        segmento < arm && arm < forward,
        "a ferramenta de osso tem de estar na mão ANTES de o clique seguir para a activa"
    );
}
