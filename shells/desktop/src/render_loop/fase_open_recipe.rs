//! **Fase do quadro: A RECEITA ABERTA** (F4.6) — a marca derivada de QUAL receita está a ser editada (a
//! selecção inteira, primária e extras), o pedido de palco para a primeira que abriu, a trava que só se
//! solta pelo `Done`/`Cancel`, e o pedido da fotografia que o `Cancel` repõe (OBRA 2 da
//! `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **Antes do extract e antes da vista do vetor**, que são os dois leitores da marca — a fase seguinte
//! é o extract. O pedido de palco é SERVIDO muito depois, quando o `snapshots` já publicou o gizmo.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_open_recipe(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim, hero_screen, ..
        } = FrameGfx::of(gfx);

        // ⭐⭐⭐ **QUAL RECEITA está a ser EDITADA** (F4.6) — a marca derivada que faz *«uma receita
        // não está na cena»* e *«a forma do mestre tem de ser editável»* deixarem de se
        // contradizer. ⚠️ **Antes do extract e antes da vista do vetor**, que são os dois leitores.
        // ⚠️ **A selecção INTEIRA — a primária e os extras** (auditoria §1.6): com só o primário,
        // Shift-clicar a linha de uma receita realçava-a na Hierarquia e não a trazia à cena.
        let opened = ph2d_app_components::master_editing::mark(
            sim,
            hero_screen.as_ref().into_iter().flat_map(|h| {
                h.gizmo
                    .selection
                    .into_iter()
                    .chain(h.gizmo.extra_selection.iter().copied())
            }),
            // ⭐⭐⭐ **A TRAVA** (Enio, 2026-09-07) — enquanto ela aponta uma receita, clicar no
            // vazio já não fecha a sessão. Solta-se pelo `Done`/`Enter`, pelo `Cancel`/`Esc`, ou
            // sozinha se a receita morrer.
            &mut self.prefab_editing,
        )
        .opened;
        // ⭐⭐⭐ **A RECEITA QUE ABRE SOBE AO PALCO** (Enio, 2026-09-07: *«o prefab deve aparecer na
        // posição central do canvas onde o canvas está»*). ⚠️ O pedido é **armado aqui e servido
        // depois** — a caixa da receita só existe quando o `snapshots` publicar o gizmo dela, e é
        // dela que o deslocamento sai (ver [`crate::prefab_stage::run`]).
        //
        // ⚠️ **A primeira, e não todas:** seleccionar duas receitas de uma vez abre as duas, e o
        // palco tem um centro só. Empilhá-las nele poria uma em cima da outra — o artista abriu
        // uma linha primeiro, e é essa que sobe.
        if let Some(first) = opened.first() {
            self.prefab_stage_pending = Some(first.to_bits());
            // ⭐⭐⭐ **A trava fecha-se na abertura** — a partir daqui só `Done`/`Enter` ou
            // `Cancel`/`Esc` a soltam.
            //
            // ⛔ **Ela guarda o `StableId`, e não os bits**: o `Ctrl+Z` respawna tudo com bits
            // novos, e uma trava em bits expulsava o artista da sessão ao desfazer. Ver o doc do
            // `master_editing::mark`.
            self.prefab_editing = sim.world().get::<ph2d_ecs::StableId>(*first).map(|s| s.0);
            // ⭐⭐⭐ **E a FOTOGRAFIA que o `Cancel` repõe** (Enio, 2026-09-07: *«um botão Cancel
            // para cancelar as modificações e deixar a edição sem fazer mudanças»*).
            //
            // ⚠️ **PEDIDA aqui e TIRADA no fim do quadro**, e não por conforto: a captura leva o
            // `&mut self` inteiro (ela reconcilia o documento antes de fotografar) e aqui o `gfx`
            // está emprestado. O fim do quadro é o sítio onde ela já vive — e o documento não muda
            // entre os dois pontos, porque abrir uma receita não é uma edição.
            self.prefab_cancel_pending = true;
        }
    }
}
