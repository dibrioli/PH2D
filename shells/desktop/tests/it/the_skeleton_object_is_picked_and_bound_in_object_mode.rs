//! ⭐⭐ **EM OBJECT, O ESQUELETO É UM OBJECTO** (A14): o osso sob o dedo selecciona o esqueleto, o
//! Bind prende a forma-objecto escolhida ao esqueleto escolhido, e o `Ctrl+P` é o mesmo clique do
//! botão. As leis têm gate em `ph2d-app-skeleton` (`a_bone_under_the_finger_selects_its_skeleton_and_seeds_the_bind`)
//! e em `ph2d-skeleton-live` (`moving_the_skeleton_object_carries_the_bound_shape`); aqui mede-se
//! que a shell as chama. ⚠️ Textual: são fases da `App`. Controlo positivo em cada uma.

fn code_only(rel: &str) -> String {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{} não se leu: {e}", p.display()))
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_bone_click_in_object_offers_the_bone_then_its_skeleton() {
    let todo = code_only("src/input_dispatch/despacho_clique_pick.rs");
    // ⚠️ A ordem mede-se DENTRO do pick de canvas: a UIUX pôs antes dele o `vetor_node_escolhe_outro_tipo`,
    // que também chama `pick_objects_at(` — a agulha global apontava para a função errada (integração 07/10).
    let src = &todo[todo
        .find("fn ramo_gizmo_pick(")
        .expect("controlo: o pick de canvas")..];
    let pick = src
        .find("pick_objects_at(")
        .expect("controlo: a porta do pick de objecto");
    let osso = src
        .find("bone_pick::objects_at(")
        .expect("o pick de canvas não pergunta pelo osso sob o dedo");
    assert!(
        osso < pick,
        "o osso é lido depois do pick (o gfx já está emprestado)"
    );
    assert!(
        src.contains("if self.skeleton.tool_in_hand {\n            Vec::new()"),
        "o pick de objecto lê o osso também com a ferramenta de osso na mão — o clique é dela"
    );
    // ⚠️ Pela ordem: o osso à frente, o esqueleto a seguir (o 2.º clique no mesmo sítio).
    assert!(
        src.contains("for b in osso_objecto.iter().rev()") && src.contains("hits.insert(0, *b)"),
        "o osso e o esqueleto não vêm à frente da arte, pela ordem"
    );
}

#[test]
fn the_bind_reads_the_object_selection() {
    let src = code_only("src/render_loop/fase_skeleton_verbs.rs");
    let bind = src
        .find("crate::skeleton_live::bind(")
        .expect("controlo: o Bind das formas");
    let seed = src
        .find("bone_pick::bind_seed(sim, &selecao_bits)")
        .expect("a semente do Bind não sai do esqueleto escolhido");
    let formas = src
        .find("bone_pick::bind_paths(")
        .expect("o Bind não junta as formas-objecto escolhidas");
    assert!(seed < bind && formas < bind);
}

#[test]
fn ctrl_p_is_the_bind_button() {
    let src = code_only("src/input_dispatch/handlers_teclas_editor.rs");
    let tecla = src
        .find("KeyCode::KeyP if self.modifiers.super_key() || self.modifiers.control_key()")
        .expect("o Ctrl+P não está ligado");
    let resto = &src[tecla..];
    let botao = resto
        .find("ph2d_editor_core::ids::VECTOR_BONE_BIND")
        .expect("o Ctrl+P não é o clique do Bind");
    assert!(
        botao < 600,
        "o Ctrl+P empurra outra coisa antes do clique do Bind"
    );
}

/// ⭐⭐ **No Edit/Pose o press do osso NÃO aponta formas na selecção da caneta** (A14, report de 05/10:
/// *«não é possível selecionar ossos e vetor para fazer o bind»*). Era um estado invisível: o cadeado
/// não deixa escolher a forma, o Bind é de Object, e o Bind de Object lia a caneta e prendia uma forma
/// que o artista não via escolhida. Controlo: o ficheiro é o do osso (o press decide pela porta).
#[test]
fn the_bone_press_never_points_a_shape_in_the_pen() {
    let src = code_only("src/input_dispatch/despacho_clique_osso.rs");
    assert!(
        src.contains("bone_gesture::press("),
        "controlo: o press do osso"
    );
    assert!(
        !src.contains("pen.select("),
        "o press do osso voltou a apontar a forma na caneta — estado invisível que o Bind prende"
    );
}
