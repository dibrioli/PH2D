//! **Fase do quadro: AS MANUTENÇÕES VIVAS DO VECTOR** — o conector, o blend, a linha de corte, o morph e o
//! conjunto de morph mantidos contra o documento do quadro (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_upkeeps(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        // **Conectores, 1ª metade:** pendura o `VecConnector` na entidade (que nasceu no
        // `sync` acima) do conector EM GESTO e do recém-fechado.
        //
        // **Antes do `settle`, e isso não é arrumação:** o `settle` pula os conectores
        // (a geometria deles é MUNDO, reescrita a cada frame) — mas só pode pular o que
        // ENXERGA. Sem o componente já pendurado, a linha recém-empurrada seria assentada
        // como um path comum: origem no centro dela, geometria recuada, e a rota do frame
        // seguinte sairia deslocada exatamente por esse delta.
        crate::connector_live::upkeep(
            sim,
            vec_scene,
            &self.vec.entities,
            self.vec.connect.as_ref().map(|d| (d.path, &d.conn)),
            &mut self.vec.connect_pending,
        );
        // **Blend Objects, 1ª metade:** pendura o `VecBlend` na entidade (nascida no `sync`)
        // do blend recém-criado. ANTES do `settle`, pela mesma razão do conector: o `settle`
        // pula o blend, mas só o que ENXERGA — sem o componente já pendurado, o spine
        // recém-empurrado seria assentado como um path comum e o recook do frame seguinte
        // sairia deslocado (ADR-0128).
        crate::blend_live::upkeep(
            sim,
            vec_scene,
            &self.vec.entities,
            &mut self.vec.blend_pending,
        );
        // **A LINHA DE CORTE, 1ª metade:** pendura o `VecCutPath` na entidade (nascida no
        // `sync`) da lâmina recém-desenhada. Mesma posição e mesma razão dos dois de cima.
        crate::vec_cut_line::upkeep(
            sim,
            vec_scene,
            &self.vec.entities,
            &mut self.vec.cut_pending,
        );
        // **Morph Objects, 1ª metade:** idem, e pela MESMA razão — sem o componente pendurado
        // antes do `settle`, o path recém-empurrado seria assentado como um path comum e o
        // recook do frame seguinte sairia deslocado.
        crate::morph_live::upkeep(
            sim,
            vec_scene,
            &self.vec.entities,
            &mut self.vec.morph_pending,
        );
        // ⭐⭐ **O CONJUNTO de estados** (plano 32 W8) — mesma posição e mesma razão do irmão
        // acima, e mais uma: é aqui que os membros são reparentados e escondidos, e as quatro
        // escritas têm de cair no MESMO quadro para o Ctrl+Z desfazer o conjunto inteiro.
        ph2d_vec_entities::morph_set::upkeep(
            sim,
            vec_scene,
            &self.vec.entities,
            &mut self.vec.morph_set_pending,
        );
    }
}
