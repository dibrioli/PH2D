//! **Fase do quadro: O INSTANTÂNEO DO PAINEL TAGS** (TOP-20 #9) — fase-filha da
//! [`super::fase_snapshots_publish`], num ficheiro irmão.
//!
//! ⚠️ **O corte foi imposto pelo tecto de FUNÇÃO da mãe** (ela chegou a `206` contra `200`) **e é o
//! certo por RESPONSABILIDADE**, por duas razões que são uma só: este instantâneo é de **OUTRO
//! painel**, e o custo dele depende de esse painel estar **ABERTO**.
//!
//! ⚠️ **A coluna «quantos objectos» é `O(mundo)` e só se paga com o painel à vista** — a tabela do
//! `ph2d_ecs::tags::counts` mede `2,97 ms` no extremo, contra um quadro de `16,7`. Fechado, o
//! painel recebe uma árvore vazia: ele não pinta nada, e nada nele lê o instantâneo enquanto está
//! escondido (o `paint` dele sai na primeira linha).
//!
//! ⛔ **E as LINHAS não se registam aqui**: quem as pinta é quem as regista, dentro do painel — um
//! registo do lado de cá é um registo que esta fase esquece no dia em que alguém lhe mexer, e o
//! gate de costura apanhou-as mortas sob o dedo exactamente assim.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::panel::Panel;
use ph2d_editor_core::screens::hero::HeroScreen;

/// Ver o cabeçalho. No-op com o painel fechado.
pub(super) fn publica(
    sim: &SimWorld,
    hero: &HeroScreen,
    tags: &ph2d_tags::TagTree,
    problema: Option<&(u64, String)>,
) {
    if !hero.is_panel_visible(<ph2d_panel_tags::TagsPanel as Panel>::ID) {
        return;
    }
    ph2d_panel_tags::set_current_tags(super::tags_panel::build_tags_panel_info(
        sim.world(),
        tags,
        problema,
    ));
}
