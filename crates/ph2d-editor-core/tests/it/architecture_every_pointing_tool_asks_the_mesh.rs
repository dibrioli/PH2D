//! **Arch-gate: TODA ferramenta que aponta para o canvas pergunta pela MALHA — não só o pincel.**
//!
//! ⛔⛔ **A wave de 2026-09-14 curou o Painter e deixou as vizinhas a mapear pelo quad de repouso**
//! (censo de 2026-09-15). O conta-gotas apanhava a cor do texel errado; e as TRÊS entradas de canvas
//! da Remoção de fundo faziam algo **pior que o afim**: montavam uma CAIXA ALINHADA AOS EIXOS a
//! partir de `translation ± size/2`, cega à **rotação**, à pose do **PAI** (ela lia o `Transform`
//! LOCAL) e à **malha**. *A mesma conta, escrita três vezes, errada nas três.*
//!
//! ⚠️ **Este gate é uma FAMÍLIA, não um sítio** — é essa a forma que a wave anterior não tinha: ela
//! gateou a porta que curou e nenhuma sonda perguntou *«quem MAIS resolve um ponteiro de canvas?»*.
//! Aqui a resposta é contada nos ficheiros, com piso de população e com a lei ANTIGA proibida pelo
//! nome, que é o que impede a recaída silenciosa.

use std::path::Path;

/// O ficheiro da shell, com os comentários retirados (uma agulha não se satisfaz com prosa).
fn shell_src(rel: &str) -> String {
    let f = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join(rel);
    std::fs::read_to_string(&f)
        .unwrap_or_else(|e| panic!("{}: {e}", f.display()))
        .lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O CONTA-GOTAS do Painter lê o texel que a arte DESENHA ali.**
#[test]
fn the_eyedropper_samples_the_texel_the_art_draws_there() {
    let rel = "shells/desktop/src/forwarding.rs";
    let src = shell_src(rel);
    // Controlo positivo: é ESTE o sítio que amostra a composição do Painter.
    assert!(
        src.contains("painter.sample_composite_at_uv(su, sv)"),
        "{rel} deixou de amostrar a composição — este gate perdeu o sujeito"
    );
    assert!(
        src.contains("let malha = ph2d_render::mesh_uv("),
        "{rel} amostra pelo afim do QUAD DE REPOUSO: numa arte dobrada o conta-gotas devolve a cor \
         de outro sítio. A lei é a `ph2d_render::mesh_uv`, e `MeshUv::Quad` deixa o caminho de \
         sempre (a grelha da folha, o *Repeat Image*) intocado."
    );
    // ⚠️ **Citar a porta não é consultá-la** — a mutação que esta metade mata é o `let _ = mesh_uv(..)`
    // ao lado do afim de sempre (ela SOBREVIVEU à 1.ª redacção do gate irmão, no pincel).
    assert!(
        src.contains("ph2d_render::MeshUv::Use { u, v, .. } => (u, v)"),
        "{rel} pergunta à malha e DEITA FORA a UV que ela devolve."
    );
    // ⚠️ E um clique FORA da arte desenhada não amostra o quad por baixo: cai para a leitura do ecrã.
    assert!(
        src.contains("if malha == ph2d_render::MeshUv::Refuse {"),
        "{rel} aceita a recusa da porta como se fosse uma UV: um clique fora da arte dobrada \
         passaria a devolver a cor do quad de repouso, que não está no ecrã."
    );
}

/// ⭐⭐⭐ **AS TRÊS ENTRADAS DA REMOÇÃO DE FUNDO passam por UMA porta, e a caixa MORREU.**
///
/// ⚠️ **O piso de população é o coração deste gate:** se alguém apagar uma das três entradas (ou lhe
/// mudar o nome) a contagem cai e o gate reprova — sem ele, um censo que varre menos lê-se como
/// *«não há mais nada»*, que é a armadilha muda do HOWTO §2.7.
#[test]
fn the_background_remover_resolves_its_pointer_through_one_door() {
    const PORTA: &str = "shells/desktop/src/input_dispatch/uv_sob_o_ponteiro.rs";
    let porta = shell_src(PORTA);
    assert!(
        porta.contains("pub(crate) fn uv_sob_o_ponteiro(")
            && porta.contains("ph2d_render::mesh_uv(")
            && porta.contains("ph2d_sprite_screen::sprite_image_to_screen_affine("),
        "{PORTA} deixou de ser a porta única: ela tem de responder pelos DOIS desenhos — a malha \
         posada e o afim do quad (que é quem desdobra a grelha de uma folha)."
    );
    // ⚠️ Os três estados são o que separa «não é nosso» de «é nosso e está fora da arte»: colapsá-los
    // troca, em silêncio, quem fica com o botão do rato.
    for estado in ["Uv(f32, f32)", "ForaDaArte", "SemSujeito"] {
        assert!(
            porta.contains(estado),
            "{PORTA} perdeu o estado `{estado}` — os três chamadores fazem coisas DIFERENTES com \
             cada um, e um `Option` aqui apaga essa diferença."
        );
    }

    let mut chamadas = 0usize;
    for rel in [
        "shells/desktop/src/input_dispatch/eyedropper.rs",
        "shells/desktop/src/input_dispatch/protect_brush.rs",
    ] {
        let src = shell_src(rel);
        chamadas += src.matches("uv_sob_o_ponteiro::uv_sob_o_ponteiro(").count();
        // ⛔⛔ **A LEI ANTIGA, proibida pelo NOME.** A caixa era
        // `camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], …)`, e ela lia a pose LOCAL — uma
        // sprite filha ou rodada já amostrava no sítio errado, antes de haver malha nenhuma.
        assert!(
            !src.contains("world_to_screen("),
            "{rel} voltou a montar a caixa alinhada aos eixos do ponteiro. Ela é cega à rotação, à \
             pose do pai e à malha — a resposta é a porta `uv_sob_o_ponteiro`."
        );
        assert!(
            !src.contains("get::<Transform>("),
            "{rel} voltou a ler a pose LOCAL da sprite: numa sprite FILHA falta a cadeia do pai, e \
             o ponteiro cai fora da pegada dela (o defeito que o Painter pagou em 2026-08-19)."
        );
    }
    assert_eq!(
        chamadas, 3,
        "as entradas de canvas da Remoção de fundo são TRÊS (o conta-gotas, o dab de protecção e o \
         *Add area*) e {chamadas} chamam a porta: uma delas voltou a ter lei própria."
    );
}
