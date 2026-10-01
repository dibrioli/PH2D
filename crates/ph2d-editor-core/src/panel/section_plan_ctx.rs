//! ⭐⭐⭐ **O LAÇO DO PLANO para os painéis que pintam por um [`PaintCtx`]** — a maioria deles.
//!
//! A lei (ordem · tema · corredor · marca de queda · fantasma) mora em [`super::section_plan`]; o
//! laço que a aplica dependia de como o painel guarda o contexto de pintura (o Inspector empurra
//! fechos com uma `Tela`, o Vector chama métodos de si mesmo). Os painéis do Painter, da Física, da
//! escultura, da grelha, do áudio e o do tuning partilham o MESMO contexto — o `PaintCtx` —, logo
//! partilham também este laço: escrito em cada um seria a lei que diverge no primeiro painel novo.
//!
//! O painel declara as secções pela ordem NATURAL ([`PlanoCtx::seccao`]); cada tarefa recebe o
//! contexto, o TEMA da secção e o `y`, e devolve o `y` seguinte — uma secção que não se aplica
//! devolve o `y` intacto e não ocupa faixa nenhuma.
//!
//! ⭐ **Três espécies de entrada**, todas pela ordem natural: a secção que se ARRASTA
//! ([`PlanoCtx::seccao`]), a secção FIXA que muda de tema e não se arrasta ([`PlanoCtx::fixa`]) e o
//! BLOCO sem título nenhum ([`PlanoCtx::bloco`] — as linhas de topo de um painel, que não são uma
//! secção). ⚠️ **As fixas e os blocos pintam-se PRIMEIRO**, pela ordem em que foram declarados, e
//! as que se arrastam a seguir, pela ordem do artista — um painel que declare uma fixa depois de uma
//! que se arrasta vê-a subir. É a lei do Inspector (o Nome e a Visibilidade no topo).
//!
//! ⚠️ **A tarefa tem de pintar o cabeçalho pela porta do plano** — [`super::section_plan::cabecalho`]
//! (acende a pega) e [`super::section_plan::regista_cabecalho`] (escreve a secção no livro do
//! quadro). Sem a segunda, o botão direito no título não abre o menu de tema e a pega não arrasta.

use super::PaintCtx;
use super::section_plan::{self, Faixa, Fantasma};
use ph2d_a11y::NodeId;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Uma secção à espera: recebe o contexto, o TEMA dela e o `y`, e devolve o `y` seguinte.
pub type TarefaCtx<'s> = Box<dyn for<'c, 'h> FnOnce(&'c mut PaintCtx<'h>, Theme, f32) -> f32 + 's>;

/// A espécie de uma entrada do plano.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Especie {
    /// Arrasta-se e muda de tema.
    Movel(NodeId),
    /// Muda de tema e fica no lugar.
    Fixa(NodeId),
    /// Linhas sem título — nem tema nem arrasto.
    Bloco,
}

/// ⭐⭐ **As secções de um quadro, pela ordem NATURAL em que o painel as declara.**
pub struct PlanoCtx<'s> {
    itens: Vec<(Especie, TarefaCtx<'s>)>,
}

impl Default for PlanoCtx<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> PlanoCtx<'s> {
    /// Um plano vazio.
    #[must_use]
    pub fn new() -> Self {
        Self {
            itens: Vec::with_capacity(16),
        }
    }

    /// Declara a secção `id`, que se ARRASTA, e como ela se pinta.
    pub fn seccao(
        &mut self,
        id: NodeId,
        tarefa: impl for<'c, 'h> FnOnce(&'c mut PaintCtx<'h>, Theme, f32) -> f32 + 's,
    ) {
        self.itens.push((Especie::Movel(id), Box::new(tarefa)));
    }

    /// Declara a secção `id`, FIXA no lugar dela, que muda de tema.
    pub fn fixa(
        &mut self,
        id: NodeId,
        tarefa: impl for<'c, 'h> FnOnce(&'c mut PaintCtx<'h>, Theme, f32) -> f32 + 's,
    ) {
        self.itens.push((Especie::Fixa(id), Box::new(tarefa)));
    }

    /// Declara um BLOCO de linhas sem título — pinta-se no tema do painel e fica no lugar.
    pub fn bloco(
        &mut self,
        tarefa: impl for<'c, 'h> FnOnce(&'c mut PaintCtx<'h>, Theme, f32) -> f32 + 's,
    ) {
        self.itens.push((Especie::Bloco, Box::new(tarefa)));
    }

    /// Os ids das secções que se ARRASTAM, pela ordem natural — para os gates e as pegas.
    #[must_use]
    pub fn ids(&self) -> Vec<NodeId> {
        self.itens
            .iter()
            .filter_map(|(e, _)| match e {
                Especie::Movel(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    /// Os ids das secções FIXAS, pela ordem natural.
    #[must_use]
    pub fn ids_fixas(&self) -> Vec<NodeId> {
        self.itens
            .iter()
            .filter_map(|(e, _)| match e {
                Especie::Fixa(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    /// ⭐⭐ **Pinta as secções pela ordem do artista**, cada uma no tema dela, e devolve o `y`
    /// depois do cartão da última — que fica FECHADO aqui. Durante
    /// um arrasto, a secção arrastada pinta-se numa cena à parte — pousada no sítio de sempre e
    /// reusada como o FANTASMA que segue o cursor, pintado no fim por cima do corpo. ⚠️ Os alvos
    /// dela registam-se no sítio real: o fantasma não é clicável.
    #[allow(clippy::too_many_arguments)]
    pub fn corre(
        self,
        ctx: &mut PaintCtx<'_>,
        panel: NodeId,
        painel: Theme,
        inner_x: f32,
        inner_w: f32,
        header_h: f32,
        mut y: f32,
    ) -> f32 {
        let (paradas, mut moveis): (Vec<_>, Vec<_>) = self
            .itens
            .into_iter()
            .partition(|(e, _)| !matches!(e, Especie::Movel(_)));
        let natural: Vec<NodeId> = moveis
            .iter()
            .filter_map(|(e, _)| match e {
                Especie::Movel(id) => Some(*id),
                _ => None,
            })
            .collect();
        let ordem = section_plan::ordem(&natural, ctx.host.store());
        let arrastada = ctx
            .host
            .store()
            .section_drag()
            .filter(|d| d.active)
            .map(|d| d.section);
        let mut faixas: Vec<Faixa> = Vec::with_capacity(ordem.len());
        let mut fantasma: Option<VectorScene> = None;
        let mut corredor = section_plan::Corredor::default();
        let mut fixas: Vec<Faixa> = Vec::new();
        for (especie, tarefa) in paradas {
            let tema = match especie {
                Especie::Fixa(id) => section_plan::tema_da_seccao(ctx.host.store(), id, painel),
                _ => painel,
            };
            y = corredor.antes(ctx.scene, painel, inner_x, inner_w, y);
            let y0 = y;
            y = tarefa(ctx, tema, y);
            if let Especie::Fixa(id) = especie
                && y > y0
            {
                y = cromo(ctx, panel, id, inner_x, inner_w, y0, y);
            }
            corredor.depois(y0, y);
            if let Especie::Fixa(id) = especie
                && y > y0
            {
                fixas.push((id, y0, y, tema));
            }
        }
        for id in ordem {
            let Some(i) = moveis.iter().position(|(e, _)| *e == Especie::Movel(id)) else {
                continue;
            };
            let (_, tarefa) = moveis.swap_remove(i);
            let tema = section_plan::tema_da_seccao(ctx.host.store(), id, painel);
            y = corredor.antes(ctx.scene, painel, inner_x, inner_w, y);
            let y0 = y;
            if arrastada == Some(id) {
                let mut parte = VectorScene::new();
                std::mem::swap(ctx.scene, &mut parte);
                y = tarefa(ctx, tema, y);
                if y > y0 {
                    y = cromo(ctx, panel, id, inner_x, inner_w, y0, y);
                }
                std::mem::swap(ctx.scene, &mut parte);
                ctx.scene.inner_mut().append(parte.inner(), None);
                fantasma = Some(parte);
            } else {
                y = tarefa(ctx, tema, y);
                if y > y0 {
                    y = cromo(ctx, panel, id, inner_x, inner_w, y0, y);
                }
            }
            corredor.depois(y0, y);
            if y > y0 {
                faixas.push((id, y0, y, tema));
            }
        }
        // ⛔⛔ **O cartão da ÚLTIMA fecha-se AQUI** (achado do agente da Física, 2026-09-30): o
        //    `end_section_cards` só pinta os cartões que já foram FECHADOS — deixá-la aberta deixava
        //    a última secção sem cartão e sem o tema que o artista lhe deu. É o que o plano do
        //    Inspector e o do Vector já faziam.
        y = corredor.fecha_a_ultima(ctx.scene, painel, inner_x, inner_w, y);
        // ⚠️ O tema de uma FIXA só se aplica DEPOIS de o cartão dela existir — o `retheme` muda os
        //    cartões já fechados, e o dela fecha-se no corredor da secção SEGUINTE. Por isso as fixas
        //    re-tematizam-se aqui, com todos os cartões fechados.
        for &(_, y0, y1, tema) in &fixas {
            if tema != painel {
                crate::widget::section_cards::retheme(y0, y1, tema);
            }
        }
        section_plan::conclui(
            ctx.scene,
            ctx.host.store(),
            painel,
            &faixas,
            inner_x,
            inner_w,
            header_h,
        );
        if let Some(f) = Fantasma::de(fantasma, &faixas, arrastada, inner_x, inner_w) {
            f.pinta(ctx.scene, ctx.host.store());
        }
        y
    }
}

/// ⭐⭐ **O cromo de uma secção que pintou** — as notas dela e o contorno à volta das duas
/// ([`crate::widget::showcase::notes_chrome::fecha_seccao`], 2026-10-01: até aqui estes painéis
/// mudavam o tema e arrastavam, e o contorno escolhido no menu do título não se pintava).
fn cromo(
    ctx: &mut PaintCtx<'_>,
    panel: NodeId,
    section: NodeId,
    inner_x: f32,
    inner_w: f32,
    y0: f32,
    y: f32,
) -> f32 {
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    crate::widget::showcase::notes_chrome::fecha_seccao(
        ctx.scene,
        ctx.text_system,
        hit_index,
        store,
        panel,
        section,
        inner_x,
        inner_w,
        y0,
        y,
    )
}
