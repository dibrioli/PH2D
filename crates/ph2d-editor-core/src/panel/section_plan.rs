//! ⭐⭐⭐ **AS SECÇÕES DE UM PAINEL COMO UMA LISTA — a lei partilhada da ordem, do tema e do
//! arrasto.**
//!
//! Nasceu no Inspector (2026-09-29, `ph2d-panel-inspector/src/plano.rs`) e mudou-se para aqui em
//! 2026-09-30, por ordem do dono: *«siga com os outros painéis»*. O que um painel com secções
//! precisa para as deixar ARRASTAR e mudar de TEMA é sempre o mesmo, e escrito em cada painel
//! seria a lei que diverge no primeiro painel novo:
//!
//! 1. **A ordem** — [`ordem`]: as fixas primeiro, o resto pela ordem do artista
//!    ([`ordena_seccoes`]).
//! 2. **O tema de cada secção** — [`tema_da_seccao`]: o que o artista lhe escolheu pelo botão
//!    direito no título, ou o do painel.
//! 3. **O cabeçalho** — [`cabecalho`] acende a pega e [`regista_cabecalho`] torna o título e a
//!    pega clicáveis E escreve a secção no livro do quadro ([`HitIndex::register_section`]), que é
//!    o que o despacho lê para abrir o menu e resolver a queda.
//! 4. **O fecho** — [`conclui`]: pinta cada faixa no tema dela ([`retheme`]) e a marca de onde a
//!    secção arrastada vai cair; [`Fantasma`] é o cartão que segue o cursor.
//!
//! ⚠️ **O laço que pinta fica em cada painel** — ele depende de como o painel guarda o contexto de
//! pintura (o Inspector empurra fechos com uma `Tela`; o Vector chama métodos de si mesmo). O que
//! ele faz por cada secção está escrito UMA vez aqui.

use crate::interaction::{HitIndex, SECCOES_FIXAS, WidgetStore, alvo_da_queda, ordena_seccoes};
use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::SectionHeader;
use crate::widget::section_cards::retheme;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// A faixa que uma secção ocupou — `(secção, topo, fundo, tema)`.
pub type Faixa = (NodeId, f32, f32, Theme);

/// ⭐ **A ordem em que um painel pinta as secções** — as fixas primeiro, pela ordem natural; o
/// resto pela ordem do artista. `natural` é a ordem em que o painel as declara.
#[must_use]
pub fn ordem(natural: &[NodeId], store: &WidgetStore) -> Vec<NodeId> {
    let (fixas, moveis): (Vec<NodeId>, Vec<NodeId>) =
        natural.iter().partition(|id| SECCOES_FIXAS.contains(id));
    let mut out = fixas;
    out.extend(ordena_seccoes(&moveis, store.section_order()));
    out
}

/// ⭐ **O tema de uma secção** — o que o artista lhe escolheu, ou o do painel.
#[must_use]
pub fn tema_da_seccao(store: &WidgetStore, id: NodeId, painel: Theme) -> Theme {
    store.section_theme(id).unwrap_or(painel)
}

/// ⭐ **O cabeçalho de uma secção que se arrasta** — o `SectionHeader` de sempre (dobra viva) com
/// a PEGA, acesa sob o rato e durante o arrasto da própria secção. ⛔ As fixas não a têm.
#[must_use]
pub fn cabecalho(store: &WidgetStore, id: NodeId, label: &str) -> SectionHeader {
    let h = SectionHeader::new(id, label)
        .collapsible(!store.is_collapsed(id))
        .open_t(store.section_open_live(id));
    if SECCOES_FIXAS.contains(&id) {
        return h;
    }
    let arrastando = store.section_drag().is_some_and(|d| d.section == id);
    h.grip(arrastando || store.hot_id() == Some(crate::ids::grip_de(id)))
}

/// ⭐⭐ **Torna o cabeçalho clicável e a pega agarrável**, e escreve a secção no livro do quadro.
/// ⚠️ A pega regista-se DEPOIS do cabeçalho — o hit-index resolve o último primeiro, e ela fica
/// por cima do rect que dobra a secção.
pub fn regista_cabecalho(hit_index: &mut HitIndex, id: NodeId, head: Rect) {
    hit_index.register_section(
        id,
        head,
        crate::ids::grip_de(id),
        crate::widget::section_grip::grip_hit_rect(head),
    );
}

/// ⭐⭐ **O corredor entre duas secções** — fecha o cartão da anterior SÓ se ela pintou.
///
/// ⛔ Medido no Vector (2026-09-30): fechar antes de TODA secção da lista, pintada ou não, é inócuo
/// num tema moderno (fechar um corredor vazio não avança o `y`) e é um SEPARADOR inteiro no
/// clássico — onde o fecho é `paint_section_separator`. Com quarenta secções na lista e meia dúzia à
/// vista, o corpo ganhava trinta e tal riscos e o resto caía para fora do painel.
#[derive(Default)]
pub struct Corredor {
    pendente: bool,
}

impl Corredor {
    /// Antes de uma secção: fecha a anterior, se ela pintou. Devolve o `y` onde esta começa.
    pub fn antes(
        &mut self,
        scene: &mut VectorScene,
        painel: Theme,
        inner_x: f32,
        inner_w: f32,
        y: f32,
    ) -> f32 {
        if std::mem::take(&mut self.pendente) {
            crate::widget::section_cards::close_section(scene, painel, inner_x, inner_w, y)
        } else {
            y
        }
    }

    /// Depois de uma secção que começou em `y0` e acabou em `y`.
    pub fn depois(&mut self, y0: f32, y: f32) {
        self.pendente |= y > y0;
    }
}

/// ⭐⭐ **Fecha o plano de um quadro**: pinta os cartões de cada faixa no tema dela e a marca de
/// onde a secção arrastada vai cair. Chama-se DEPOIS do último `close_section` do corpo.
#[allow(clippy::too_many_arguments)]
pub fn conclui(
    scene: &mut VectorScene,
    store: &WidgetStore,
    painel: Theme,
    faixas: &[Faixa],
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
) {
    for (_, y0, y1, tema) in faixas {
        if *tema != painel {
            retheme(*y0, *y1, *tema);
        }
    }
    paint_marca_da_queda(scene, store, painel, faixas, inner_x, inner_w, header_h);
}

/// ⭐⭐ **O FANTASMA da secção arrastada** — o cartão dela, menor e meio transparente, com o ponto
/// por onde a mão pegou debaixo do cursor (ordem do dono, 2026-09-30: *«permita ver o card sendo
/// arrastado, menor e meio transparente»*). O fundo é o do cartão, no TEMA da secção.
pub struct Fantasma {
    conteudo: VectorScene,
    cartao: Rect,
    tema: Theme,
}

impl Fantasma {
    /// O fantasma da secção `arrastada`, pintada à parte em `conteudo` — `None` sem arrasto, ou se
    /// ela não pintou nada neste quadro.
    #[must_use]
    pub fn de(
        conteudo: Option<VectorScene>,
        faixas: &[Faixa],
        arrastada: Option<NodeId>,
        inner_x: f32,
        inner_w: f32,
    ) -> Option<Self> {
        let conteudo = conteudo?;
        let &(_, y0, y1, tema) = faixas.iter().find(|f| Some(f.0) == arrastada)?;
        let pad = ph2d_tokens::card_pad_px();
        Some(Self {
            conteudo,
            cartao: Rect::new(
                inner_x - pad,
                y0 - pad,
                inner_w + pad * 2.0,
                y1 - y0 + pad * 2.0,
            ),
            tema,
        })
    }

    /// Pinta-o — por último, sobre o corpo inteiro.
    pub fn pinta(self, scene: &mut VectorScene, store: &WidgetStore) {
        let Some(drag) = store.section_drag().filter(|d| d.active) else {
            return;
        };
        let cor = resolve(
            crate::widget::section_cards::CardDepth::Section.token(),
            self.tema,
        );
        crate::widget::paint_card_ghost(
            scene,
            &self.conteudo,
            self.cartao,
            crate::paint::frame_radius(self.tema, ph2d_tokens::Radius::Md.px()),
            Some(cor),
            (drag.down_x, drag.down_y),
            (drag.cursor_x, drag.cursor_y),
        );
    }
}

/// ⭐ **A marca de onde a secção arrastada vai cair** — uma barra de acento no meio do vão entre
/// dois cartões, e o contorno do cartão que se está a mover.
///
/// ⚠️ **A queda é a MESMA lei do despacho** ([`alvo_da_queda`]) sobre os mesmos meios de cabeçalho
/// (o cabeçalho regista-se com `header_h` a partir do topo da faixa). Duas contas punham a marca
/// num sítio e a secção noutro.
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
    // ⚠️ Só as faixas DESTE painel: um arrasto noutro painel não desenha marca aqui.
    if !faixas.iter().any(|f| f.0 == drag.section) {
        return;
    }
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
        crate::paint::stroke_rounded_rect(
            scene,
            Rect::new(
                inner_x - pad,
                f.1 - pad,
                inner_w + pad * 2.0,
                f.2 - f.1 + pad * 2.0,
            ),
            crate::paint::frame_radius(painel, ph2d_tokens::Radius::Md.px()),
            espessura,
            acento,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐ **Uma secção que não pintou não deixa corredor** — no tema clássico o fecho é um
    /// separador inteiro, e fechar depois de uma secção vazia punha um risco por cada uma.
    /// *Mutação: o `antes` a fechar sempre ⇒ o `y` anda sem nenhuma secção ter pintado.*
    #[test]
    fn so_uma_seccao_que_pintou_deixa_corredor() {
        let mut scene = VectorScene::new();
        let classico = Theme::Forge;
        let mut c = Corredor::default();
        let y = c.antes(&mut scene, classico, 0.0, 200.0, 100.0);
        assert!(
            (y - 100.0).abs() < f32::EPSILON,
            "fechou antes da 1.ª secção"
        );
        c.depois(y, y); // uma secção vazia
        let y2 = c.antes(&mut scene, classico, 0.0, 200.0, y);
        assert!(
            (y2 - y).abs() < f32::EPSILON,
            "uma secção vazia deixou um separador"
        );
        c.depois(y2, y2 + 40.0); // uma que pintou
        let y3 = c.antes(&mut scene, classico, 0.0, 200.0, y2 + 40.0);
        assert!(
            y3 > y2 + 40.0,
            "a secção que pintou não fechou o corredor (o separador)"
        );
    }

    /// ⭐ **A ordem: as fixas primeiro, o resto pela ordem do artista.**
    #[test]
    fn a_ordem_prende_as_fixas_e_segue_o_artista() {
        let mut store = WidgetStore::with_capacity(4);
        let (a, b, c) = (NodeId(11), NodeId(22), NodeId(33));
        store.set_section_order(vec![c, a]);
        let fixa = SECCOES_FIXAS[0];
        assert_eq!(ordem(&[a, fixa, b, c], &store), vec![fixa, c, a, b]);
    }
}
