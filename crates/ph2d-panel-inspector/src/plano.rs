//! ⭐⭐⭐ **O PLANO DO INSPECTOR — as secções como uma LISTA, pintada pela ordem que o artista
//! escolheu.**
//!
//! Ordem do dono, 2026-09-29: *«o blender tem no topo de cada seção um ícone de 10 pontos que
//! serve para arrastar e reorganizar as seções. Vamos criar isso.»*
//!
//! Até aqui a ordem das secções ERA o código: cada grupo pintava as suas por sequência, e mudar a
//! ordem era mudar a sequência. Agora cada grupo **empurra** as secções dele para um [`Plano`], e o
//! plano pinta-as de uma vez pela ordem [`ph2d_editor_core::interaction::ordena_seccoes`] — a ordem
//! natural (a da paleta, derivada em 2026-09-21) com o que o artista moveu por cima.
//!
//! ⭐ **A disposição é REAL, não uma translação do desenho:** cada secção é pintada no sítio onde
//! fica, logo o hit-index, os popovers e as notas registam-se lá — nenhum canal paralelo tem de ser
//! corrigido depois. *Mover o desenho e deixar os alvos onde estavam seria a forma mais barata de
//! um botão ficar morto debaixo do rato.*
//!
//! ## O que o plano faz por cada secção, e só ele
//!
//! 1. **Fecha o cartão anterior ANTES, se ele pintou** — um corredor só
//!    ([`section_plan::Corredor`]); fechar depois de uma secção vazia era um separador a mais no
//!    tema clássico.
//!
//! ⭐ **A lei (ordem · tema · marca de queda · fantasma) mora em
//! [`ph2d_editor_core::panel::section_plan`] desde 2026-09-30** — partilhada com o Vector e os
//! painéis que vierem; aqui fica o laço da `Tela`.
//! 2. **Resolve o TEMA da secção** — o que o artista lhe escolheu pelo botão direito no título, ou
//!    o do painel — e passa-o à secção como o `theme` dela.
//! 3. **Regista a faixa pintada**, e no fim pinta os cartões de cada faixa no tema dela
//!    ([`retheme`]) e a marca de onde uma secção arrastada vai cair.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
pub(crate) use ph2d_editor_core::panel::section_plan::Fantasma;
use ph2d_editor_core::panel::section_plan::{self, Faixa};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// ⭐ **Onde uma secção pinta** — os três mutáveis que toda moldura de secção pede.
pub(crate) struct Tela<'c> {
    pub scene: &'c mut VectorScene,
    pub text: &'c mut TextSystem,
    pub hit: &'c mut HitIndex,
}

/// Uma secção à espera: recebe a tela, o TEMA dela e o `y`, e devolve o `y` seguinte.
type Tarefa<'a> = Box<dyn for<'c> FnOnce(&mut Tela<'c>, Theme, f32) -> f32 + 'a>;

/// ⭐⭐ **A lista das secções deste quadro, pela ordem NATURAL em que os grupos as empurram.**
pub(crate) struct Plano<'a> {
    tarefas: Vec<(NodeId, Tarefa<'a>)>,
}

impl<'a> Plano<'a> {
    pub(crate) fn new() -> Self {
        Self {
            tarefas: Vec::with_capacity(48),
        }
    }

    /// Empurra a secção `id`. ⚠️ Empurrar é barato e incondicional: uma secção cujo objecto não tem
    /// o componente devolve o `y` intacto, e o plano lê isso como *«não está à vista»*.
    pub(crate) fn push(
        &mut self,
        id: NodeId,
        tarefa: impl for<'c> FnOnce(&mut Tela<'c>, Theme, f32) -> f32 + 'a,
    ) {
        self.tarefas.push((id, Box::new(tarefa)));
    }

    /// A ordem em que o plano vai pintar: as fixas primeiro, pela ordem natural; o resto pela
    /// ordem do artista.
    pub(crate) fn ordem(&self, store: &WidgetStore) -> Vec<NodeId> {
        let natural: Vec<NodeId> = self.tarefas.iter().map(|(id, _)| *id).collect();
        section_plan::ordem(&natural, store)
    }

    /// ⭐⭐ **Pinta todas as secções** e devolve o `y` depois da última — e, durante um arrasto, o
    /// [`Fantasma`] da secção arrastada, que o chamador pinta DEPOIS do resto do corpo (as notas do
    /// fim do painel incluídas), para ele ficar por cima de tudo.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn run(
        mut self,
        tela: &mut Tela<'_>,
        store: &WidgetStore,
        painel: Theme,
        inner_x: f32,
        inner_w: f32,
        header_h: f32,
        mut y: f32,
    ) -> (f32, Option<Fantasma>) {
        let ordem = self.ordem(store);
        let mut faixas: Vec<Faixa> = Vec::with_capacity(ordem.len());
        let arrastada = store.section_drag().filter(|d| d.active).map(|d| d.section);
        let mut fantasma: Option<VectorScene> = None;
        let mut corredor = section_plan::Corredor::default();
        for id in ordem {
            let Some(i) = self.tarefas.iter().position(|(t, _)| *t == id) else {
                continue;
            };
            let (_, tarefa) = self.tarefas.swap_remove(i);
            let tema = section_plan::tema_da_seccao(store, id, painel);
            y = corredor.antes(tela.scene, painel, inner_x, inner_w, y);
            let y0 = y;
            if arrastada == Some(id) {
                // ⭐⭐ **A secção arrastada pinta-se numa cena À PARTE** (2026-09-30): ela é pousada
                //    no sítio de sempre (onde o contorno de arrasto a marca) e reusada, no fim,
                //    como o FANTASMA que segue o cursor. ⚠️ Os alvos dela registam-se no sítio
                //    real — o fantasma não é clicável.
                let mut parte = VectorScene::new();
                let mut sub = Tela {
                    scene: &mut parte,
                    text: &mut *tela.text,
                    hit: &mut *tela.hit,
                };
                y = tarefa(&mut sub, tema, y);
                tela.scene.inner_mut().append(parte.inner(), None);
                fantasma = Some(parte);
            } else {
                y = tarefa(tela, tema, y);
            }
            corredor.depois(y0, y);
            if y > y0 {
                faixas.push((id, y0, y, tema));
            }
        }
        y = corredor.antes(tela.scene, painel, inner_x, inner_w, y);
        section_plan::conclui(
            tela.scene, store, painel, &faixas, inner_x, inner_w, header_h,
        );
        let fantasma = Fantasma::de(fantasma, &faixas, arrastada, inner_x, inner_w);
        (y, fantasma)
    }
}

/// ⭐ **Uma secção com a moldura de sempre** — `begin_section`, o corpo, `finish_section`. É a
/// forma das secções que moravam em grupos (o núcleo, a sprite, as partilhadas, a física): o
/// plano fecha o cartão, a moldura regista cabeçalho, pega e notas, e o corpo só pinta.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emoldurada<'a>(
    plano: &mut Plano<'a>,
    id: NodeId,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    banda: f32,
    corpo: impl for<'c> FnOnce(&mut Tela<'c>, Theme, f32) -> f32 + 'a,
) {
    plano.push(id, move |c, tema, y| {
        crate::paint_frame::begin_section(c.hit, inner_x, inner_w, y, id, banda);
        let novo = corpo(c, tema, y);
        crate::paint_frame::finish_section(
            c.scene, c.text, c.hit, store, inner_x, inner_w, id, y, novo,
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::ids;

    fn tela_run(plano: Plano<'_>, store: &WidgetStore) -> Vec<NodeId> {
        let order = plano.ordem(store);
        let _ = plano;
        order
    }

    /// ⭐⭐ **O plano pinta pela ordem que o artista largou, e as fixas ficam no topo.** *Mutação:
    /// `ordem` a devolver a natural ⇒ a Transform continua em cima.*
    #[test]
    fn o_plano_segue_a_ordem_do_artista_e_prende_as_fixas() {
        let mut store = WidgetStore::with_capacity(4);
        store.set_section_order(vec![
            ids::INSP_LIVE_SHEET_SECTION,
            ids::INSP_LIVE_TRANSFORM_SECTION,
        ]);
        let mut plano = Plano::new();
        for id in [
            ids::INSP_LIVE_NAME_SECTION,
            ids::INSP_LIVE_VISIBILITY_SECTION,
            ids::INSP_LIVE_TRANSFORM_SECTION,
            ids::INSP_LIVE_RENDER_SECTION,
            ids::INSP_LIVE_SHEET_SECTION,
        ] {
            plano.push(id, |_, _, y| y);
        }
        assert_eq!(
            tela_run(plano, &store),
            vec![
                ids::INSP_LIVE_NAME_SECTION,
                ids::INSP_LIVE_VISIBILITY_SECTION,
                ids::INSP_LIVE_SHEET_SECTION,
                ids::INSP_LIVE_TRANSFORM_SECTION,
                ids::INSP_LIVE_RENDER_SECTION,
            ]
        );
    }

    /// ⭐⭐ **Cada secção recebe o TEMA dela, e as outras o do painel.** *Mutação: passar sempre o
    /// `painel` ⇒ a secção escolhida não muda de tema.*
    #[test]
    fn cada_seccao_recebe_o_tema_que_o_artista_lhe_deu() {
        let mut store = WidgetStore::with_capacity(4);
        let painel = Theme::Dark;
        store.set_section_theme(ids::INSP_LIVE_RENDER_SECTION, Some(Theme::Light));
        let vistos = std::cell::RefCell::new(Vec::new());
        let mut plano = Plano::new();
        for id in [
            ids::INSP_LIVE_TRANSFORM_SECTION,
            ids::INSP_LIVE_RENDER_SECTION,
        ] {
            let vistos = &vistos;
            plano.push(id, move |_, t, y| {
                vistos.borrow_mut().push((id, t));
                y + 10.0
            });
        }
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut hit = HitIndex::default();
        let mut tela = Tela {
            scene: &mut scene,
            text: &mut text,
            hit: &mut hit,
        };
        let _ = plano.run(&mut tela, &store, painel, 0.0, 100.0, 20.0, 0.0);
        assert_eq!(
            *vistos.borrow(),
            vec![
                (ids::INSP_LIVE_TRANSFORM_SECTION, painel),
                (ids::INSP_LIVE_RENDER_SECTION, Theme::Light),
            ]
        );
    }
}
