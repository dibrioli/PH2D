//! **Toda re-cozedura de um filtro vivo TEM de avisar o Vello.**
//!
//! ⛔⛔ **O defeito que este gate impede** (achado por auditoria em 2026-08-29, na subida
//! `vello` 0.8 → 0.10): até à 0.8 o atlas de imagens do Vello era **limpo a cada render**, então
//! uma textura registada era re-copiada de graça e re-cozinhar nela «simplesmente funcionava». A
//! **0.10 tornou o atlas persistente** — uma imagem residente só volta a subir se estiver marcada
//! **suja**, e o `register_texture` marca-a suja **uma vez**.
//!
//! ⇒ Escrever pixels novos na mesma textura passou a ser **invisível**: a forma filtrada (glow,
//! sombra, desfoque, rgb split) congelava na primeira cozedura e só refrescava ao mudar de
//! **tamanho**. ⚠️ *O memo errava, a GPU recozia certo, e a tela não mudava.*
//!
//! ⚠️ **Por que um censo de FONTE e não um teste de pixels:** provar isto de verdade exige dois
//! renders com o conteúdo mudado entre eles e um adaptador de GPU — e os gates de GPU desta casa
//! são `#[ignore]`, logo o CI nunca os corre. Um censo corre **sempre**. O que ele defende não é o
//! valor de um pixel: é a **emparelhação** — quem escreve numa textura registada avisa quem a lê.
//!
//! ⛔ E é a emparelhação que envelhece: um sítio de re-cozedura NOVO, escrito por outra linha daqui
//! a um mês, nasce mudo e nada o denuncia. É esse o leitor deste ficheiro.

use std::path::Path;

/// Conta ocorrências de `agulha` em linhas de **código** (nunca em comentário nem doc-comment).
fn conta_em_codigo(fonte: &str, agulha: &str) -> usize {
    fonte
        .lines()
        .filter(|l| {
            let corpo = l.trim_start();
            !corpo.starts_with("//") && corpo.contains(agulha)
        })
        .count()
}

#[test]
fn fx_live_marks_every_recook_dirty() {
    let fx = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fx_live.rs");
    let fonte = std::fs::read_to_string(&fx).expect("ler fx_live.rs");

    let recozeduras = conta_em_codigo(&fonte, "stack.run_from(");
    let avisos = conta_em_codigo(&fonte, "mark_texture_dirty(");

    // A metade JUSTA: uma sonda que casa zero linhas passa sempre e não diz nada.
    assert!(
        recozeduras >= 1,
        "a sonda tem de VER a re-cozedura que ja' existe; contou {recozeduras}. Se o nome do \
         metodo mudou, este censo ficou cego e tem de ser reescrito, nao apagado."
    );
    assert_eq!(
        avisos, recozeduras,
        "ha' {recozeduras} sitio(s) a re-cozinhar uma textura registada e {avisos} a avisar o \
         Vello. Desde a `vello` 0.10 o atlas de imagens e' PERSISTENTE: pixels novos numa textura \
         ja' registada NAO chegam a tela sem `VelloPass::mark_texture_dirty`. O sintoma e' uma \
         forma filtrada congelada na primeira cozedura, que so' refresca ao mudar de tamanho."
    );
}

/// A outra ponta: a porta tem de continuar a existir e a chamar o upstream.
///
/// ⚠️ Sem isto, alguém podia satisfazer o censo acima com um `mark_texture_dirty` que **não faz
/// nada** — o gate ficaria verde sobre um no-op.
#[test]
fn the_dirty_door_still_reaches_vello() {
    let porta =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/ph2d-render/src/vello_pass.rs");
    let fonte = std::fs::read_to_string(&porta).expect("ler vello_pass.rs");
    assert!(
        conta_em_codigo(&fonte, "mark_override_image_dirty(") >= 1,
        "o `VelloPass::mark_texture_dirty` tem de chamar o `mark_override_image_dirty` do Vello. \
         Uma porta que nao alcanca o upstream deixa o censo irmao verde sobre um no-op."
    );
}
