//! **Arch-gate do PREVIEW do Offset** — um retune SUBSTITUI o próprio passo de undo, e o
//! Apply Offset com preview vivo CONSOLIDA em vez de re-offsetar (Enio 2026-07-21: *"os
//! botões Miter, Round e Bevel são previsualizações em tempo real … para consolidar a
//! curva deve-se apertar Apply Offset"*).
//!
//! ## O que este gate protege
//!
//! Os chips de Corner/Side re-offsetam o resultado recém-solto ao vivo. Sem a
//! substituição de passo, cada clique registrava um passo de undo próprio: testar os 3
//! modos custava 3 Ctrl+Z, e o artista lia cada clique como um BAKE — o report que
//! motivou o modelo de preview. A mecânica: no braço `Retune`, o passo no topo da fila
//! (que é o deste offset — o oráculo de profundidade acabou de conferir) sai via
//! `forget_last` e o estado-pré volta ao `undo_baseline` **ANTES** do `win.apply`; o
//! diff do fim do frame re-registra UM passo pre-gesto → resultado novo. N retunes = 1
//! passo, e um Ctrl+Z devolve a cena de antes do offset.
//!
//! ## Por que um gate de TEXTO
//!
//! A política mora no corpo do `render_frame`, entre o `step()` da janela e o `apply` —
//! nenhum unit test alcança aquele trecho (o `App` headless não tem `gfx`/`vec_scene`), e
//! um espelho escrito à mão não veria uma reordenação real (a lição do gate irmão
//! `the_z_projection_reads_the_tree_after_the_sync`). O smoke 18/19 mostra o efeito vivo
//! (o `undo=` da telemetria fica CONSTANTE através dos cliques de Corner).

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// A posição (em bytes) da 1ª ocorrência de `needle`, ou pânico com a razão.
fn at(needle: &str) -> usize {
    SRC.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu do render_loop — se foi renomeado, atualize este gate (e \
             confira que o modelo de PREVIEW do offset continua de pé: retune substitui o \
             próprio passo; Apply Offset consolida)"
        )
    })
}

#[test]
fn a_retune_replaces_its_own_undo_step_before_reapplying() {
    let retune_arm = at("RetuneStep::Retune =>");
    let forget = at("self.undo.forget_last()");
    let rebaseline = at("self.undo_baseline = Some(pre)");
    let apply = SRC[retune_arm..]
        .find("win.apply(")
        .map(|i| retune_arm + i)
        .expect("o braço Retune perdeu o win.apply");
    assert!(
        retune_arm < forget && forget < rebaseline && rebaseline < apply,
        "a substituição de passo tem de acontecer DENTRO do braço Retune e ANTES do \
         win.apply (forget_last → baseline → apply) — fora dessa ordem, cada clique de \
         Corner volta a empilhar um passo de undo (o bake por clique do report de \
         2026-07-21)"
    );
}

#[test]
fn apply_offset_with_a_live_preview_consolidates_instead_of_reoffsetting() {
    // A consolidação pergunta pela janela e a CONSOME antes do caminho numérico do botão.
    let consolidate = at("self.vec_offset_retune.take().is_some()");
    let numeric = at("crate::vec_expand::apply_vec_expand(");
    assert!(
        consolidate < numeric,
        "o botão Apply Offset tem de checar o preview vivo ANTES do caminho numérico — \
         senão o clique aplica um 2º offset por cima do preview (o botão desfazendo a \
         promessa de testar antes de aplicar)"
    );
}
