//! **Fase-filha do quadro: A TABELA nome → acção** (TOP-20 #5) — fase-filha da
//! [`super::fase_signal_outbox`], num ficheiro irmão.
//!
//! ⚠️ **O corte foi imposto pelo tecto de FUNÇÃO** (a mãe chegou a `232` contra `200` ao ganhar a
//! injecção do som) **e é o certo por responsabilidade:** o que ela faz tem nome próprio — *ler os
//! sinais COM quem os disse, resolvê-los contra o mundo, e aplicar*.
//!
//! ⚠️⚠️ **O prefixo `fase_` é LOAD-BEARING, e a falha sem ele é MUDA:** o texto emendado do quadro
//! (`frame_text::render_frame`, o oráculo de toda lei de ORDEM desta shell) colhe **só** as funções
//! `fn fase_*` do `render_loop/`. Uma fase-filha com outro nome desaparece dali e as leis de ordem
//! que a atravessam deixam de ser medidas sem um único teste ficar vermelho.
//!
//! ⚠️ **QUANDO, no quadro:** depois do dreno do outbox (senão os sinais deste quadro só chegariam ao
//! próximo) e antes do `post_frame_undo` (senão a escrita não seria fotografada nem declarada ao
//! ledger). É a mesma janela do toast, por construção.

use super::*;

/// ⭐ **O diagnóstico, e ele IMPRIME MESMO A ZERO** — é esse o caso que interessa: um sinal que soa
/// e não resolve efeito nenhum é o modo de falha MUDO desta tabela (um reactor sem `StableId` não
/// entra na consulta do `resolve`, e o toast aparece na mesma).
fn diga_o_que_resolveu(ligado: bool, nomes: &[&str], efeitos: usize) {
    if ligado {
        eprintln!("[signal] {nomes:?} -> {efeitos} efeito(s)");
    }
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    /// ⚠️ **O `gfx` é RE-DERIVADO aqui e não passado**, como na irmã `fase_fabrica_e_morte`: o
    /// `sim` e a árvore de tags vêm do destructure do `gfx`, e passá-los a um método `&mut self`
    /// emprestaria a `App` duas vezes. *Os guardas do quadro já correram na `fase_chrome_clock`.*
    pub(super) fn fase_tabela_de_accoes(&mut self, deaths: &mut Vec<ph2d_ecs::Death>) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, tags, .. } = FrameGfx::of(gfx);
        // ⭐⭐⭐ **A ORIGEM VIAJA COM O NOME** (suplente #24, 2026-09-19) — até aqui esta leitura era
        // `.map(|s| s.name)`, e a origem MORRIA no `.map`.
        //
        // ⚠️⚠️ **O dado já estava construído, publicado e lido:** o `SignalOrigin` tem catorze
        // variantes e **onze** carregam `source`; o `Contact` carrega `source` **e** `other`, com o
        // doc a chamar-lhes *«quem GRITOU»* e *«quem chegou, ou quem saiu»*. Deitá-lo fora aqui é o
        // que fazia um tiro num inimigo tirar vida aos dez (medido: `10` efeitos para um sinal).
        //
        // ⛔ **Quem responde «quem falou?» é UMA porta** (`SignalOrigin::quem`/`::outro`) e nunca um
        // `match` escrito aqui: com catorze variantes, um braço `_` esqueceria a décima quinta **em
        // silêncio** — e um sinal sem sujeito lê-se exactamente como uma origem que não tem sujeito.
        //
        // ⚠️ **A conversão tem NOME** (`signal_actions::lido`) e não vive neste `map`: era
        // exactamente aqui que a origem morria, e um gate não apanha o que não tem nome.
        let lidos: Vec<(String, Option<ph2d_ecs::Entity>, Option<ph2d_ecs::Entity>)> = self
            .signals
            .read(&mut self.signal_readers.action)
            .map(signal_actions::lido)
            .collect();
        if lidos.is_empty() {
            return;
        }
        let nomes: Vec<&str> = lidos.iter().map(|(n, _, _)| n.as_str()).collect();
        let disparos: Vec<ph2d_ecs::Disparo<'_>> = lidos
            .iter()
            .map(|(nome, quem, outro)| ph2d_ecs::Disparo {
                nome,
                quem: *quem,
                outro: *outro,
            })
            .collect();
        // ⚠️ **A ÁRVORE DE TAGS entra aqui** (TOP-20 #9): uma linha com alvo por TAG pergunta quem
        // pertence à subárvore dela; uma por nome nunca a lê.
        //
        // ⚠️ **A resolução é pura e a aplicação não** — ver o cabeçalho do
        // [`ph2d_app_components::signal_actions_bridge`]: escrever uma `Visibility` é escrever um
        // componente REGISTADO, e sem o `preview_drive` cada porta que abre viraria um passo de
        // `Ctrl+Z`.
        let efeitos = ph2d_ecs::resolve_signal_actions(sim.world_mut(), tags, &disparos);
        diga_o_que_resolveu(self.signal_readers.logging(), &nomes, efeitos.len());
        if efeitos.is_empty() {
            return;
        }
        // ⭐⭐⭐ **O SOM entra por INJECÇÃO** (2026-09-19): a ponte desceu para a família e o
        // `ph2d-app-audio` é outra FAMÍLIA — o `architecture_no_dependency_climbs_a_layer` recusa a
        // aresta, e a cura que ele prescreve por escrito é *«uma tabela injectada pela
        // composição»*. ⇒ a tabela pede *«toca o som deste objecto»* e a SHELL, dona dos dois
        // lados, responde.
        let mut audio = self.audio.as_mut();
        let mut som = |w: &mut ph2d_ecs::SimWorld,
                       q: ph2d_app_components::signal_actions_bridge::Som,
                       alvo: ph2d_ecs::Entity| {
            use ph2d_app_components::signal_actions_bridge::Som;
            match q {
                Som::Toca => audio_2d::play_target(w, audio.as_deref_mut(), alvo),
                Som::Cala => audio_2d::stop_target(w, audio.as_deref_mut(), alvo),
            }
        };
        let r = signal_actions::apply(sim, &efeitos, &mut self.preview_drive, &mut som);
        if self.signal_readers.logging() {
            eprintln!(
                "[signal] {} accao(oes) aplicada(s), {} inerte(s)",
                r.applied, r.inert
            );
        }
        // ⭐⭐⭐ **E quem o `Destroy` mandou sair vai ao DESPACHANTE** (suplente #24), e não a um
        // `despawn` aqui: *«quando é que isto sai da cena?»* é uma pergunta só, e a resposta dela é
        // o dreno da `fase_fabrica_e_morte`, que corre por último — *um moribundo continua visível a
        // toda consulta até ao fim do quadro*.
        deaths.extend(r.mortes);
    }
}
