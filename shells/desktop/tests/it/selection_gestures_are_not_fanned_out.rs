//! **Two §11 gestures are about the SELECTION, and must not be fanned out.**
//!
//! `Join` (W3) is one click about a PAIR. `Bake` (W4) is one click about the
//! whole selection, satisfied by ONE run of the simulation. Both arrive through
//! the same `InspectorPhysicsEdit` action as the ordinary per-entity edits, and
//! both mean the opposite of what fanning out would do.
//!
//! Every other §11 physics edit is per-entity, so `render_loop` fans it out
//! over the selection — "make all of these static" is a gesture an artist
//! performs. `Join` arrives through that same action and means the opposite:
//! it is one click about two objects. Fanned out it would run once per
//! selected body and create **two joints between the same pair**, on the very
//! click that is supposed to create one.
//!
//! No unit test can see this. The rule lives inside `render_loop`'s action
//! drain — a function far too large and too entangled with the frame to drive
//! from a test — so the gate reads the source, exactly as
//! `the_z_projection_reads_the_tree_after_the_sync` does for frame ordering.
//!
//! It is deliberately a gate about the SHAPE of the code and not about a
//! literal: what it pins is that the interception exists *before* the fan-out,
//! which is the only place the distinction can be made.

/// The FRAME in the order it runs — the spliced text (`frame_text::render_frame`).
///
/// ⚠️ Since OBRA 2 of `line/render-loop` (2026-09-12) the frame is split into phases in other files, and
/// both the action drain and the rig block leave `render_loop/mod.rs`; the spliced text reads them in
/// either place, in execution order.
fn src() -> &'static str {
    static FRAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FRAME.get_or_init(crate::frame_text::render_frame)
}

/// From `at` to the brace that closes the first `{` after it — a window by STRUCTURE, never by bytes or
/// by indentation.
fn block_after(src: &str, at: usize) -> &str {
    let open = src[at..].find('{').expect("the block opens a brace") + at;
    let mut depth = 0usize;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[at..open + i];
                }
            }
            _ => {}
        }
    }
    panic!("the block never closes");
}

/// The window of source that handles `InspectorPhysicsEdit`.
fn physics_edit_arm() -> &'static str {
    let src = src();
    let start = src
        .find("EditorAction::InspectorPhysicsEdit { entity_bits, edit } => {")
        .expect(
            "the §11 physics edit arm has been renamed — this gate points at \
             nothing and has to be re-aimed",
        );
    // The arm ends at the brace that closes it. ⚠️ It used to end at the next `EditorAction::` arm found
    // by a 20-space indentation — a column the drain's move into its own phase changes, and a miss
    // silently widened the window to the end of the file.
    //
    // ⚠️⚠️ The body opens at the `=> {`, NOT at the first brace after the arm's name: that one belongs to
    // the struct PATTERN (`InspectorPhysicsEdit { entity_bits, edit }`), and a window that closed there
    // was a dozen characters long — every gate reading this arm went red over a correct frame (caught
    // by the control of this re-aim's own mutation proof).
    let arrow = start
        + src[start..]
            .find("=> {")
            .expect("the §11 physics edit arm has a block body");
    let body = block_after(src, arrow);
    &src[start..arrow + body.len()]
}

#[test]
fn join_is_intercepted_before_the_per_entity_fan_out() {
    let arm = physics_edit_arm();

    let join_at = arm.find("PhysicsFieldEdit::Join").expect(
        "the physics edit arm does not mention Join at all — it is being \
         treated as an ordinary per-entity edit, so a click on \"Join Selected \
         Bodies\" creates one joint PER selected body",
    );
    let fan_out_at = arm.find("for &t in &inspector_selection").expect(
        "the physics edit arm no longer fans out over the selection — if that \
         is deliberate this gate should be deleted along with it",
    );
    assert!(
        join_at < fan_out_at,
        "Join is handled AFTER the fan-out over the selection, so it runs \
         once per selected body. Two bodies selected means two joints between \
         the same pair, and the artist clicked once"
    );
}

/// **Join reads the WHOLE ordered selection, never an arbitrary pair.**
///
/// ⚠️ **A claim desta gate MUDOU na W-J4.** Ela exigia `if let [a, b] = …[..]` —
/// exatamente dois — pelo medo certo: *"um `.first()`/`.get(1)` aceitaria três
/// corpos em silêncio e ligaria dois arbitrários"*. A W-J4 responde esse medo de
/// outro jeito, melhor: três corpos fazem uma **CORRENTE** sobre a sequência
/// inteira, então nada é arbitrário e nada é descartado.
///
/// O que continua sendo o perigo é ler um SUBCONJUNTO: o pedido tem de ser sobre
/// a seleção inteira (um booleano + a ordem), e a construção dos pares mora na
/// `joint_draw::join_chain` (gateada headless), não aqui.
#[test]
fn join_reads_the_whole_ordered_selection() {
    let arm = physics_edit_arm();
    assert!(
        arm.contains("inspector_selection.len() >= 2"),
        "a interceptação do Join não pergunta mais pela seleção INTEIRA — se ela \
         voltar a destruturar um par, três corpos marcados perdem um elo em \
         silêncio"
    );
    for forbidden in ["inspector_selection[..]", ".get(1)"] {
        assert!(
            !arm.contains(forbidden),
            "a interceptação do Join volta a pegar um SUBCONJUNTO ({forbidden}) — \
             a corrente é sobre a sequência toda"
        );
    }
}

/// **Bake is intercepted before the fan-out too (W4).**
///
/// The failure is quieter than Join's and more expensive. A fanned-out bake
/// produces the *right numbers* — the simulation is deterministic, so every run
/// agrees — while re-simulating the entire scene once per selected body, and
/// filing a separate undo step for each. Nothing looks wrong: the curves are
/// correct, the scene is correct, and undoing "the bake" simply takes as many
/// Ctrl+Z presses as there were objects, which reads as the undo being flaky
/// rather than the bake being wrong.
#[test]
fn bake_is_intercepted_before_the_per_entity_fan_out() {
    let arm = physics_edit_arm();

    let bake_at = arm.find("PhysicsFieldEdit::Bake").expect(
        "the physics edit arm does not mention Bake at all — it is being \
         treated as an ordinary per-entity edit, so baking a selection of N \
         bodies runs the whole simulation N times and leaves N undo steps",
    );
    let fan_out_at = arm.find("for &t in &inspector_selection").expect(
        "the physics edit arm no longer fans out over the selection — if that \
         is deliberate this gate should be deleted along with it",
    );
    assert!(
        bake_at < fan_out_at,
        "Bake is handled AFTER the fan-out over the selection, so it runs once \
         per selected body: the same simulation, N times over, and N undo steps \
         for one click"
    );
}

/// **A bake with nothing selected still bakes the entity the section is about.**
///
/// The fan-out's empty case is the single-selection case, and the interception
/// has to reproduce it or the button does nothing at all when exactly one body
/// is selected — which is the commonest way anyone will use it.
#[test]
fn the_bake_request_falls_back_to_the_inspected_entity() {
    let arm = physics_edit_arm();
    assert!(
        arm.contains("vec![entity_bits]"),
        "the Bake interception does not fall back to `entity_bits` when the \
         selection list is empty — the button would be dead in the ordinary \
         one-body case"
    );
}

/// **O RIG é a terceira, e não pode ser espalhada tampouco** (W-Rig).
///
/// A falha é mais quieta que a do Join e a do Bake. Cada corrida do gerador
/// percorre a MESMA subárvore, então espalhado ele acerta: a 2ª corrida em
/// diante acha toda aresta já ligada e não faz nada. O que quebra é o RELATO —
/// o toast contaria os corpos da 1ª corrida uma vez por entidade selecionada —,
/// e um número errado num toast é como o artista aprende a não acreditar neles.
#[test]
fn rig_is_intercepted_before_the_per_entity_fan_out() {
    let arm = physics_edit_arm();

    let rig_at = arm.find("PhysicsFieldEdit::Rig").expect(
        "o braço de edição da §11 não menciona o Rig — ele está sendo tratado \
         como uma edição por-entidade, e o gerador roda uma vez por objeto \
         selecionado",
    );
    let fan_out_at = arm.find("for &t in &inspector_selection").expect(
        "o braço da §11 não faz mais fan-out sobre a seleção — se isso é \
         deliberado, este gate sai junto",
    );
    assert!(
        rig_at < fan_out_at,
        "o Rig é tratado DEPOIS do fan-out sobre a seleção"
    );
}

/// **E o Rig lê a seleção VIVA, não o `inspector_selection`.**
///
/// ⚠️ Este é o defeito que a wave quase teve, e ele é invisível em toda fixture
/// de multi-seleção: o `inspector_selection` só é colhido quando
/// `selected_count > 1` e fica **vazio** no caso único — que é precisamente o
/// gesto do rig (marcar a raiz do personagem e clicar). Lido dali, o botão
/// funcionaria em toda situação **menos** a que ele existe para servir, e um
/// gate escrito com dois objetos marcados ficaria verde por cima disso.
#[test]
fn the_rig_reads_the_live_selection_not_the_multi_select_buffer() {
    let src = src();
    let start = src
        .find("if rig_now {")
        .expect("o bloco que executa o rig sumiu do render loop");
    // O BLOCO do rig, pelas chavetas — eram 900 bytes, que numa fase dedentada cobrem o bloco seguinte.
    let block = block_after(src, start);
    assert!(
        block.contains("hero.gizmo.iter_selected()"),
        "o rig não lê a seleção viva — com UM objeto marcado (o gesto normal) \
         ele receberia uma lista vazia e não faria nada"
    );
    assert!(
        !block.contains("&inspector_selection"),
        "o rig voltou a ler o buffer de multi-seleção, que é vazio no caso de \
         um objeto só"
    );
}
