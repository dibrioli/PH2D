//! **Arch-gate: na cena do HUD, o rótulo do botão é FILHO do botão** (TOP-20 #20).
//!
//! # A lei, e o defeito MEDIDO que ela impede
//!
//! O hit-test de objecto entrega *a forma mais ao topo que contém o ponto*. O `+10` é desenhado por
//! cima do corpo do botão ⇒ **o dedo aterra sempre no RÓTULO**, e é a subida da cadeia
//! ([`ph2d_ecs::hud::botao_de`], gateada na folha) que faz disso um clique no botão.
//!
//! ⚠️ **A lei da folha não basta:** ela responde *«de quem é esta forma?»* e precisa que a CADEIA
//! exista. Com o rótulo pendurado no canvas — como ele nasceu — a resposta é `None`, e carregar no
//! meio do botão não o pressiona. A auto-conferência da cena mediu-o à letra (`achou=Some(2)`,
//! `esperado=3`), e o veredito dela é um `eprintln!` que ninguém lê num portão. Este gate é a
//! metade que reprova.
//!
//! ⛔ **Textual, e a razão é a de sempre:** a montagem vive num método de `App` que exige janela,
//! GPU e superfície — nenhum teste a alcança. É a forma dos irmãos `architecture_*` da shell.

/// ⚠️ A PROSA sai antes de a lei ser aplicada — senão este mesmo cabeçalho satisfazia o `assert`,
/// e o gate passaria a medir a explicação em vez do código.
fn sem_comentarios(src: &str) -> String {
    src.lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn o_rotulo_do_botao_do_hud_e_filho_do_botao() {
    let src = sem_comentarios(include_str!("../../src/hud_smoke.rs"));

    // Controlo positivo: a cena ainda monta as peças de que a lei fala. Sem isto, um ficheiro
    // renomeado ou esvaziado deixaria os dois `assert` abaixo trivialmente verdes.
    for agulha in [
        "let (Some(e_pontos), Some(e_resta), Some(e_botao), Some(e_rotulo))",
        "UiButton",
    ] {
        assert!(
            src.contains(agulha),
            "controlo positivo: a cena do HUD deixou de conter `{agulha}` — este gate passou a \
             medir o nada"
        );
    }

    assert!(
        src.contains("ChildOf(e_botao)"),
        "o rótulo do botão não é filho do BOTÃO na cena do HUD.\n\
         O dedo aterra no rótulo (o hit-test devolve a forma mais ao topo que contém o ponto), e a \
         subida da cadeia (`ph2d_ecs::hud::botao_de`) só encontra o botão se a cadeia existir. Sem \
         isto, carregar no MEIO do botão não o pressiona — medido em 2026-09-17."
    );

    // A outra metade: ele não pode estar TAMBÉM na lista que pendura tudo no canvas — duas poses
    // locais somam-se, e o rótulo sairia do ecrã por baixo.
    let parenta_no_canvas = src
        .split("ChildOf(canvas)")
        .next()
        .expect("split devolve sempre a 1.ª fatia");
    assert!(
        !parenta_no_canvas.contains("(e_rotulo, pend.rotulo.local)"),
        "o rótulo está na lista que pendura no CANVAS e também no botão — a pose local somaria \
         duas vezes o mesmo deslocamento"
    );
}
