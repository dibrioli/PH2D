//! **Fase do quadro: O LAYOUT VIVO, O ALINHAMENTO E A SILHUETA** — o auto layout publica ONDE as coisas ficam
//! (as poses da vista), o alinhamento vivo e a silhueta que o FX lê (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_layout_recook(
        &mut self,
        mut vec_view: ph2d_vec_scene::VecViewState,
        vec_xf: ph2d_vec_scene::VecXforms,
        mut vec_live: ph2d_vec_render::LiveGeometry,
    ) -> Option<(
        ph2d_vec_scene::VecViewState,
        ph2d_vec_render::LiveGeometry,
        ph2d_vec_scene::VecXforms,
    )> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // **O AUTO LAYOUT roda entre a booleana e o alinhamento** (ADR-0153), e as duas
        // metades da ordem são a lei:
        //
        // - DEPOIS da booleana, porque ele coloca *o que os filhos de fato desenham* — um
        //   grupo booleano é UMA forma para o fluxo, e ela tem de estar cozida antes de ser
        //   medida;
        // - ANTES do alinhamento, porque o alinhamento recorta a faixa do traço na largura
        //   AUTORADA: escalar uma forma depois de a faixa estar recortada esticaria a
        //   espessura dela junto, e o traço do artista mudaria de peso ao redimensionar a
        //   moldura.
        //
        // E ele TRANSFORMA o mapa (não o estende) pelo motivo do `bool_live`: é um componente
        // do PAI, então convive com o offset vivo de cada filho.
        self.layout_live.recook(
            vec_scene,
            sim,
            &self.vec.entities,
            &vec_xf,
            &mut vec_live,
            crate::vec_bindings::TokenCtx {
                theme: hero.theme,
                pixels_per_meter: hero.project.pixels_per_meter,
            },
        );
        // **A POSE que cada filho colocado recebeu**, publicada para quem NÃO desenha
        // geometria: as âncoras do modo Node, a caixa do gizmo e o hit-test leem a pose
        // AUTORADA, e ela não se mexeu com o layout.
        vec_view.poses = self.layout_live.poses();
        // **E os TRÊS fatos derivados são PUBLICADOS** para quem vier depois do desenho. O
        // hit-test monta o `VecViewState` dele do zero a cada evento de ponteiro, e aquela
        // porta só sabe o que a ÁRVORE diz (escondido, travado) — sem isto ele decide como se
        // nenhuma moldura existisse, nenhuma forma tivesse sido colocada e nenhum operando
        // tivesse sido absorvido.
        self.vec.view_derived.clips.clone_from(&vec_view.clips);
        self.vec.view_derived.poses.clone_from(&vec_view.poses);
        self.vec
            .view_derived
            .absorbed
            .clone_from(&vec_view.absorbed);
        // **O ALINHAMENTO roda por ÚLTIMO, e TRANSFORMA o mapa em vez de o estender.**
        // Os cinco acima são mutuamente exclusivos (um componente cada, um por vez no
        // painel), e é isso que torna o `extend` seguro. O alinhamento não é membro dessa
        // família — é um campo do `StrokeSpec`, então convive com um offset vivo; fundido
        // por `extend` ele apagaria o offset (ou seria apagado), em silêncio.
        self.align_live.recook(vec_scene, &vec_xf, &mut vec_live);
        // A SILHUETA resolvida das formas TRAÇADAS: `preenchimento ∪ contorno-do-traço`,
        // pela booleana, memoizada na geometria de MUNDO. Sem ela o campo de distância de uma
        // forma com traço cai no caminho do raster, cuja semente discreta desenha o pente que
        // o Enio fotografou no bevel. Roda DEPOIS de `vec_live` (a união é do que se DESENHA).
        self.fx_silhouette
            .recook(vec_scene, sim, &self.vec.entities, &vec_xf, &vec_live);
        Some((vec_view, vec_live, vec_xf))
    }
}
