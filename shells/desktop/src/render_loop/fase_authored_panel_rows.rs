//! **Fase do quadro: O PAINEL AUTORADO SEGUE O DOCUMENTO** — a moldura autorada, o retorno do picker para a forma que
//! veste a swatch, e as rows vivas publicadas ao painel. Fase-filha da [`super`] (`fase_vector_bands`), que a chama
//! no início (OBRA 3 da `line/render-bodies`, 2026-09-13).
//!
//! ⚠️ No bloco original ela corria depois do `PaintCtx` e das duas leituras `self.fx_live.images()` /
//! `self.texture_pattern_live.tiles()`; os três só LEEM (`&self` que devolve a referência), e nenhum toca o
//! `vec_scene` que o retorno do picker escreve — a ordem nova não se observa. A ordem que importa (a cor ANTES das
//! rows, `the_colour_lands_before_the_rows_are_derived`) é interna ao bloco e não mudou.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_authored_panel_rows(&mut self) -> Option<()> {
        // O `gfx` re-derivado, com os mesmos guardas da fase-mãe.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        let hero = hero_screen.as_mut()?;
        // **O PAINEL AUTORADO SEGUE O DOCUMENTO** — publicado a cada quadro em que ele
        // está na tela. Sem isto o painel desenha a tabela COMPILADA, e o artista precisa de
        // colar o código gerado e recompilar para ver o que acabou de desenhar: um ciclo de
        // compilação dentro do laço de autoria (report do Enio, 2026-08-09).
        //
        // ⚠️ **Só com o painel VISÍVEL**, e é a lei do ADR-0125: o custo de descrever um
        // painel é trabalho de autoria, não de quadro. Fechado, ele não paga nada — e a
        // publicação de `None` devolve o painel à tabela compilada, que é o que um build sem
        // documento autorado tem de mostrar.
        let authored_frame = hero
            .is_panel_visible(
                <ph2d_panel_authored::AuthoredPanel as ph2d_editor_core::panel::Panel>::ID,
            )
            .then(|| crate::ui_panel_spec::authored_frame(sim, vec_scene))
            .flatten();
        // **O RETORNO DO PICKER** — a cor escolhida pinta a forma que veste a swatch.
        //
        // ⚠️ **ANTES de derivar as rows, e a ordem é o assunto:** a row publica o
        // preenchimento da forma, então escrever a cor depois de a ler daria uma swatch a
        // mostrar a cor ANTIGA por um quadro — o piscar que faz o artista clicar duas vezes.
        //
        // ⚠️ E o alvo do picker é PARTILHADO (Painter, Vector, timeline usam o mesmo canal);
        // o `picker_shape` devolve `None` quando ele não é uma row desta moldura, que é o caso
        // comum. Escrever sem essa pergunta pintaria a forma errada a cada vez que outro
        // painel abrisse o picker.
        if let Some(frame) = authored_frame
            && let Some(target) = hero.store.picker_target()
            && let Some(path) = crate::ui_panel_spec::picker_shape(sim, vec_scene, frame, target)
            && let Some((value, _, _, _)) = hero
                .store
                .blender_picker(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
        {
            // ⚠️ A porta RECUSA a cor igual, e é ela que impede a escrita ao ABRIR: o
            // `pointer_down` semeia o picker no clique da swatch, então sem a recusa o gesto
            // de *olhar* a cor escreveria o documento — achatando um gradiente e gravando um
            // passo de undo por quadro. A lei é a do `set_piece_colour`, ali em cima.
            crate::ui_panel_spec::paint_swatch_colour(vec_scene, path, value.rgba);
        }
        let live_rows = authored_frame.map(|f| crate::ui_panel_spec::live_rows(sim, vec_scene, f));
        ph2d_panel_authored::rows::set_live_rows(live_rows);
        Some(())
    }
}
