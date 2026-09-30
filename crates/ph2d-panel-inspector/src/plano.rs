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
//! 1. **Fecha o cartão anterior ANTES** — um corredor só; as molduras que ainda fecham dentro de si
//!    batem num fecho vazio, que é um no-op ([`close_section`]).
//! 2. **Resolve o TEMA da secção** — o que o artista lhe escolheu pelo botão direito no título, ou
//!    o do painel — e passa-o à secção como o `theme` dela.
//! 3. **Regista a faixa pintada**, e no fim pinta os cartões de cada faixa no tema dela
//!    ([`retheme`]) e a marca de onde uma secção arrastada vai cair.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{
    HitIndex, SECCOES_FIXAS, WidgetStore, alvo_da_queda, ordena_seccoes,
};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::widget::section_cards::{close_section, retheme};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// ⭐ **Onde uma secção pinta** — os quatro mutáveis que toda moldura de secção pede.
pub(crate) struct Tela<'c> {
    pub scene: &'c mut VectorScene,
    pub text: &'c mut TextSystem,
    pub hit: &'c mut HitIndex,
    pub tops: &'c mut Vec<f32>,
}

/// Uma secção à espera: recebe a tela, o TEMA dela e o `y`, e devolve o `y` seguinte.
type Tarefa<'a> = Box<dyn for<'c> FnOnce(&mut Tela<'c>, Theme, f32) -> f32 + 'a>;

/// ⭐⭐ **A lista das secções deste quadro, pela ordem NATURAL em que os grupos as empurram.**
pub(crate) struct Plano<'a> {
    tarefas: Vec<(NodeId, Tarefa<'a>)>,
}

/// A faixa que uma secção ocupou — `(secção, topo, fundo, tema)`.
type Faixa = (NodeId, f32, f32, Theme);

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
        let (fixas, moveis): (Vec<NodeId>, Vec<NodeId>) =
            natural.iter().partition(|id| SECCOES_FIXAS.contains(id));
        let mut out = fixas;
        out.extend(ordena_seccoes(&moveis, store.section_order()));
        out
    }

    /// ⭐⭐ **Pinta todas as secções** e devolve o `y` depois da última.
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
    ) -> f32 {
        let ordem = self.ordem(store);
        let mut faixas: Vec<Faixa> = Vec::with_capacity(ordem.len());
        for id in ordem {
            let Some(i) = self.tarefas.iter().position(|(t, _)| *t == id) else {
                continue;
            };
            let (_, tarefa) = self.tarefas.swap_remove(i);
            let tema = store.section_theme(id).unwrap_or(painel);
            y = close_section(tela.scene, painel, inner_x, inner_w, y);
            let y0 = y;
            y = tarefa(tela, tema, y);
            if y > y0 {
                faixas.push((id, y0, y, tema));
            }
        }
        y = close_section(tela.scene, painel, inner_x, inner_w, y);
        for (_, y0, y1, tema) in &faixas {
            if *tema != painel {
                retheme(*y0, *y1, *tema);
            }
        }
        paint_marca_da_queda(
            tela.scene, store, painel, &faixas, inner_x, inner_w, header_h,
        );
        y
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
    body_top_y: f32,
    banda: f32,
    notas: &'a [(usize, ph2d_editor_core::interaction::NoteData)],
    corpo: impl for<'c> FnOnce(&mut Tela<'c>, Theme, f32) -> f32 + 'a,
) {
    plano.push(id, move |c, tema, y| {
        crate::paint_frame::begin_section(
            c.tops, c.hit, inner_x, inner_w, body_top_y, y, id, banda,
        );
        let novo = corpo(c, tema, y);
        crate::paint_frame::finish_section(
            c.scene, c.text, c.hit, store, inner_x, inner_w, id, y, novo, notas,
        )
    });
}

/// ⭐ **A marca de onde a secção arrastada vai cair** — uma barra de acento no meio do vão entre
/// dois cartões, e o contorno do cartão que se está a mover.
///
/// ⚠️ **A queda é a MESMA lei do despacho** ([`alvo_da_queda`]) sobre os mesmos meios de cabeçalho
/// (o `begin_section` regista o cabeçalho com `header_h` a partir do topo da faixa). Duas contas
/// punham a marca num sítio e a secção noutro.
fn paint_marca_da_queda(
    scene: &mut VectorScene,
    store: &WidgetStore,
    painel: Theme,
    faixas: &[Faixa],
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
) {
    let Some(drag) = store.section_drag().filter(|d| d.active) else {
        return;
    };
    let moveis: Vec<&Faixa> = faixas
        .iter()
        .filter(|f| !SECCOES_FIXAS.contains(&f.0))
        .collect();
    let heads: Vec<(NodeId, f32)> = moveis.iter().map(|f| (f.0, f.1 + header_h * 0.5)).collect();
    let pad = ph2d_tokens::card_pad_px();
    let meio_do_vao = pad + ph2d_tokens::card_gap_px() * 0.5;
    let y = match alvo_da_queda(&heads, drag.section, drag.cursor_y) {
        Some(alvo) => moveis
            .iter()
            .find(|f| f.0 == alvo)
            .map(|f| f.1 - meio_do_vao),
        None => moveis.last().map(|f| f.2 + meio_do_vao),
    };
    let acento = resolve(ColorToken::Accent, painel);
    let espessura = StrokeToken::Thick.px();
    if let Some(y) = y {
        fill_rounded_rect(
            scene,
            Rect::new(
                inner_x - pad,
                y - espessura * 0.5,
                inner_w + pad * 2.0,
                espessura,
            ),
            espessura * 0.5,
            acento,
        );
    }
    if let Some(f) = moveis.iter().find(|f| f.0 == drag.section) {
        // FRAME-RAW-OK: o contorno de ARRASTO — um estado do gesto, da cor do acento, como a marca.
        ph2d_editor_core::paint::stroke_rounded_rect(
            scene,
            Rect::new(
                inner_x - pad,
                f.1 - pad,
                inner_w + pad * 2.0,
                f.2 - f.1 + pad * 2.0,
            ),
            ph2d_editor_core::paint::frame_radius(painel, ph2d_tokens::Radius::Md.px()),
            espessura,
            acento,
        );
    }
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
        let mut tops = Vec::new();
        let mut tela = Tela {
            scene: &mut scene,
            text: &mut text,
            hit: &mut hit,
            tops: &mut tops,
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
