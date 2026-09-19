//! ⭐⭐⭐ **O PINCEL DE PESO ESTÁ FIADO** — as três costuras da shell que nenhum gate de crate vê.
//!
//! A lei vive na [`ph2d_skeleton`], o gesto na [`ph2d_skeleton_live::peso_a_mao`] e a decisão do
//! press na [`ph2d_app_skeleton::bone_gesture`] — as três com gates próprios. O que **só** esta
//! shell pode responder é se o fio existe:
//!
//! 1. o verbo armado chega ao painel pelo **ÍNDICE derivado** e não por uma comparação,
//! 2. o **arrasto** pinta (e não só o press),
//! 3. o traço **acaba** no soltar.
//!
//! ⚠️ **Um censo textual, e assumidamente a régua mais fraca da família** — mas as três coisas que
//! ele mede vivem num caminho que pede um `wgpu::Device` (o `on_mouse_input` da shell), e um gate
//! `#[ignore]` é um gate que **o CI nunca corre**. *Uma régua fraca que corre vale mais que uma
//! forte que ninguém arma.*

static DISPATCH: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(crate::input_text::dispatch);

/// **O fonte sem comentários** — a mesma porta do [`crate::the_bone_pickers_are_modal`], e pela
/// mesma razão: *uma nota que cita a chamada conta como chamada*.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") { "" } else { l }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O ÍNDICE DO VERBO SAI DA PORTA, NUNCA DE UMA COMPARAÇÃO.**
///
/// ⛔⛔ Ele era `usize::from(acao == BoneAction::Transform)`, que está **certo com dois verbos e
/// mente em silêncio com três**: o terceiro lê `0` e acende o primeiro segmento — o artista carrega
/// em *Weight*, o pincel arma, e o painel continua a dizer *Create*.
///
/// ⚠️ **As duas metades:** a chamada existe **e** a comparação não voltou. Sem a segunda, alguém
/// que reponha o `bool` ao lado da porta deixa o gate verde.
#[test]
fn o_indice_do_verbo_do_osso_sai_da_porta() {
    let src = code_only(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/render_loop/fase_selection_mirror_skin.rs"
        ))
        .expect("a fase que publica o espelho do esqueleto"),
    );
    assert!(
        src.contains("bone_action.indice()"),
        "o índice do verbo deixou de sair da porta `BoneAction::indice` — com três verbos, uma \
         comparação acende sempre o segmento errado"
    );
    assert!(
        !src.contains("BoneAction::Transform)"),
        "a comparação com `Transform` voltou ao lado da porta — duas respostas para o mesmo índice"
    );
}

/// ⭐⭐⭐ **O ARRASTO PINTA, e o SOLTAR acaba o traço.**
///
/// ⛔⛔ **As duas metades são defeitos opostos e os dois MUDOS:** sem a primeira o pincel pinta um
/// dab por clique e o arrasto não faz nada (*«o pincel quase não pinta»*); sem a segunda o
/// `weight_drag` fica armado depois do soltar, e o movimento do rato **continua a pintar** sem o
/// botão premido — que é indistinguível de tinta a aparecer sozinha.
#[test]
fn o_arrasto_do_peso_pinta_e_o_soltar_acaba_o_traco() {
    let mover = code_only(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/input_dispatch/despacho_mover.rs"
        ))
        .expect("o despacho do movimento"),
    );
    assert!(
        mover.contains("self.vec_peso_move()"),
        "o movimento não chama o pincel de peso — o arrasto não pinta, e só o press pinta"
    );
    let solto = code_only(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/input_dispatch/despacho_clique_vetor_solto.rs"
        ))
        .expect("o despacho do soltar"),
    );
    assert!(
        solto.contains("self.skeleton.weight_drag = None"),
        "o soltar não larga a arte do traço — o pincel continua a pintar sem o botão premido"
    );
}

/// ⭐⭐ **E o PRESS pinta também** — a 1.ª pincelada sai no pen-down.
///
/// ⚠️ **Sem ela, um CLIQUE (sem arrasto) não faz nada**, e um clique que não faz nada lê-se
/// exactamente como um pincel partido — a família de reports que o `CLAUDE.md` §5.0 nomeia.
#[test]
fn o_press_do_peso_ja_pinta_a_primeira_pincelada() {
    let src = code_only(&DISPATCH);
    let armou = src
        .find("self.skeleton.weight_drag = alvo;")
        .expect("o press arma o traço de peso");
    let pintou = src[armou..]
        .find("self.vec_pinta_peso(")
        .expect("o press pinta a 1.ª pincelada — sem isto um clique não faz nada");
    assert!(
        pintou < 200,
        "a pincelada do press está longe de onde o traço é armado — as duas linhas são um gesto só"
    );
}
