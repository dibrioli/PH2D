//! ⭐⭐ **A lei da compactação na fundação** ([05 §4](../../../../docs/CartoesUI/05_ESPECIFICACAO_UI_COMPLETA.md)).
//!
//! - [`CardCtx::row`]: botões e segmentos medem o texto; o que não cabe desce de linha (lei 7).
//! - [`CardCtx::pair`] e [`CardCtx::compact`]: controlos curtos em duas colunas iguais; o par desce
//!   para duas linhas quando a coluna é estreita, sem cortar nome nem valor (leis 1 e 5).
//! - [`fluir`]: o corpo que pinta por pedidos soltos (`property_row`, os ~45 componentes do Inspector):
//!   cada pedido curto à largura do corpo fica à espera do seguinte; dois seguidos saem lado a lado.

use super::censo::{self, CelulaMedida};
use super::corpo::ctx as ctx_de_fora;
use super::{CardCtx, Cell, Seg, Widget, rotulos};
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::CheckboxValue;
use crate::widget::card_field::{CardFieldSpec, Glyph};
use crate::widget::card_ink::CardInk;
use crate::widget::key_mark::Rotulo;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use std::cell::RefCell;

impl CardCtx<'_> {
    /// ⭐ **Uma linha**: os rótulos (se algum) e os widgets lado a lado. Botão ou segmento cujo texto
    /// não cabe na coluna desce para a linha seguinte (lei 7).
    pub fn row(&mut self, cells: &[Cell<'_>], y: f32) -> f32 {
        if cells.len() > 1 && !self.textos_cabem(cells) {
            return self.desce(cells, y);
        }
        if let [c] = cells
            && let Widget::Choice(segs) = c.widget
        {
            return self.escolha_em_linhas(c, segs, y);
        }
        self.row_inteira(cells, y)
    }

    /// ⭐ **Dois irmãos curtos**: lado a lado em colunas iguais, ou um por linha se a coluna é estreita.
    pub fn pair(&mut self, a: Cell<'_>, b: Cell<'_>, y: f32) -> f32 {
        let par = [a, b];
        if self.tudo_cabe(&par) {
            return self.row_inteira(&par, y);
        }
        let y = self.row_inteira(&[a], y);
        self.row_inteira(&[b], y)
    }

    /// ⭐ **N irmãos curtos**: dois a dois enquanto couberem, pela ordem dada.
    pub fn compact(&mut self, cells: &[Cell<'_>], y: f32) -> f32 {
        let mut y = y;
        let mut resto = cells;
        while let [a, cauda @ ..] = resto {
            match cauda {
                [b, ..] if self.tudo_cabe(&[*a, *b]) => {
                    y = self.row_inteira(&[*a, *b], y);
                    resto = &cauda[1..];
                }
                _ => {
                    y = self.row_inteira(&[*a], y);
                    resto = cauda;
                }
            }
        }
        y
    }

    /// [`Self::compact`] poria estas células na mesma linha?
    pub fn cabem_lado_a_lado(&mut self, cells: &[Cell<'_>]) -> bool {
        self.tudo_cabe(cells)
    }

    /// A largura em que `c` mostra o conteúdo inteiro.
    pub fn largura_pedida(&mut self, c: &Cell<'_>) -> f32 {
        self.pede(c)
    }

    /// Os botões e segmentos de `cells` mostram o texto inteiro nas colunas desta linha.
    fn textos_cabem(&mut self, cells: &[Cell<'_>]) -> bool {
        let cols = self.columns(cells);
        cells.iter().zip(&cols).all(|(c, (_, w))| {
            let pede = match c.widget {
                Widget::Button { .. } | Widget::ButtonOff { .. } => self.pede(c),
                Widget::Choice(segs) => self.escolha_pede(segs),
                _ => 0.0,
            };
            pede <= *w
        })
    }

    /// A largura em que os segmentos cabem numa linha, sem ícones ao lado das palavras.
    pub fn escolha_pede(&mut self, segs: &[Seg<'_>]) -> f32 {
        let soma: f32 = segs.iter().map(|s| self.seg_need(s, false).0).sum();
        soma + m::SEG_PAD * (segs.len() + 1) as f32
    }

    /// Reparte `cells` em linhas: em cada uma entram quantas couberem, pela ordem.
    fn desce(&mut self, cells: &[Cell<'_>], y: f32) -> f32 {
        let mut y = y;
        let mut resto = cells;
        while !resto.is_empty() {
            let n = (1..resto.len())
                .rev()
                .find(|&n| self.textos_cabem(&resto[..n]))
                .unwrap_or(1);
            y = self.row(&resto[..n], y);
            resto = &resto[n..];
        }
        y
    }

    /// Uma escolha que não cabe quebra em filas; o rótulo e a marca ficam na primeira.
    fn escolha_em_linhas(&mut self, c: &Cell<'_>, segs: &[Seg<'_>], y: f32) -> f32 {
        if self.escolha_pede(segs) <= self.w || segs.len() < 2 {
            return self.row_inteira(&[*c], y);
        }
        let pedidos: Vec<f32> = segs.iter().map(|s| self.seg_need(s, false).0).collect();
        let mut y = y;
        let mut resto = segs;
        let mut primeira = true;
        for n in filas_equilibradas(&pedidos, self.w) {
            let cell = if primeira {
                Cell {
                    widget: Widget::Choice(&resto[..n]),
                    ..*c
                }
            } else {
                Cell::new(Widget::Choice(&resto[..n]))
            };
            y = self.row_inteira(&[cell], y);
            primeira = false;
            resto = &resto[n..];
        }
        y
    }

    /// Anota no censo a linha já pintada de `topo` a `fundo`.
    pub(super) fn anota_linha(&mut self, cells: &[Cell<'_>], topo: f32, fundo: f32) {
        if !censo::ligado() {
            return;
        }
        let medidas: Vec<CelulaMedida> = cells
            .iter()
            .map(|c| CelulaMedida {
                nome: nome_da_celula(c),
                curta: curta(c),
                pede: self.pede(c),
            })
            .collect();
        censo::linha((self.x, self.w), topo, fundo, || medidas);
    }
}

/// Quantos segmentos entram em cada fila à largura `w`, enchendo cada uma pela ordem.
fn filas_gulosas(pedidos: &[f32], w: f32) -> Vec<usize> {
    let mut filas = Vec::new();
    let (mut n, mut soma) = (0, m::SEG_PAD);
    for p in pedidos {
        if n > 0 && soma + p + m::SEG_PAD > w {
            filas.push(n);
            (n, soma) = (0, m::SEG_PAD);
        }
        n += 1;
        soma += p + m::SEG_PAD;
    }
    if n > 0 {
        filas.push(n);
    }
    filas
}

/// O mesmo número de filas que a largura `w` dá, à menor largura que ainda o dá: nenhum segmento fica órfão na última
/// (o `text-wrap: balance` do CSS). Medido na lista TOOL do Impasto: `4, 5, 1` passa a `4, 3, 3`.
fn filas_equilibradas(pedidos: &[f32], w: f32) -> Vec<usize> {
    const BISSECCOES: usize = 16;
    let quantas = filas_gulosas(pedidos, w).len();
    let (mut cabe, mut nao) = (w, 0.0_f32);
    for _ in 0..BISSECCOES {
        let meio = (cabe + nao) * 0.5;
        if filas_gulosas(pedidos, meio).len() <= quantas {
            cabe = meio;
        } else {
            nao = meio;
        }
    }
    filas_gulosas(pedidos, cabe)
}

/// A lei 1: número, amostra + código, caixa de marcar, chip, facto e escolha de duas opções curtos.
/// Com 3+ opções de texto a lei 2 dá-lhe a linha inteira. Uma escolha sem nome é a fila de uma grelha
/// que quebrou (os cinco modos do Flip), não um controlo com vizinho.
fn curta(c: &Cell<'_>) -> bool {
    match c.widget {
        Widget::Choice(segs) => c.label.is_some() && segs.len() <= ESCOLHA_CURTA_MAX,
        Widget::Field { .. }
        | Widget::FieldFace { .. }
        | Widget::Swatch { .. }
        | Widget::Check { .. }
        | Widget::Dropdown { .. }
        | Widget::Readout { .. } => true,
        _ => false,
    }
}

/// Quantas opções de texto uma escolha curta tem (lei 2 de 05 §4).
pub(super) const ESCOLHA_CURTA_MAX: usize = 2;

fn nome_da_celula(c: &Cell<'_>) -> String {
    if let Some(l) = c.label {
        return l.to_owned();
    }
    match c.widget {
        Widget::Check { label, .. } => label.texto.to_owned(),
        Widget::Button { text, .. } | Widget::ButtonOff { text } | Widget::Readout { text } => {
            text.to_owned()
        }
        Widget::Choice(s) | Widget::Icons(s) | Widget::Bare(s) => {
            s.iter().map(|s| s.text).collect::<Vec<_>>().join("/")
        }
        _ => String::new(),
    }
}

/// Um pedido curto à espera do vizinho.
#[derive(Clone)]
enum Curto {
    Numero {
        rotulo: String,
        letra: String,
        id: NodeId,
        key: [NodeId; 1],
        spec: CardFieldSpec,
    },
    Amostra {
        rotulo: String,
        id: [NodeId; 1],
        rgba: Option<[u8; 4]>,
        codigo: String,
    },
    Marca {
        id: NodeId,
        nome: String,
        rotulo: Rotulo<'static>,
        value: CheckboxValue,
        off: bool,
    },
    Fato {
        rotulo: String,
        texto: String,
    },
    Escolha {
        rotulo: String,
        ids: Vec<NodeId>,
        opcoes: Vec<(NodeId, Option<IconId>, String, bool)>,
    },
}

impl Curto {
    /// Os segmentos de uma escolha, para [`Self::celula`]; vazio nas outras.
    fn segs(&self) -> Vec<Seg<'_>> {
        match self {
            Self::Escolha { opcoes, .. } => opcoes
                .iter()
                .map(|(id, icon, text, on)| Seg {
                    id: *id,
                    icon: *icon,
                    text,
                    on: *on,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    fn celula<'a>(&'a self, segs: &'a [Seg<'a>]) -> Cell<'a> {
        match self {
            Self::Escolha { rotulo, ids, .. } => {
                Cell::new(Widget::Choice(segs)).label(rotulo).key(ids)
            }
            Self::Numero {
                rotulo,
                letra,
                id,
                key,
                ..
            } => Cell::new(Widget::Field {
                id: *id,
                glyph: Glyph::Text(letra),
            })
            .label(rotulo)
            .key(key),
            Self::Amostra {
                rotulo,
                id,
                rgba,
                codigo,
            } => Cell::new(Widget::Swatch {
                id: id[0],
                rgba: *rgba,
                text: codigo,
                edit: None,
                detach: None,
            })
            .label(rotulo)
            .key(id),
            Self::Marca {
                id,
                nome,
                rotulo,
                value,
                off,
            } => Cell::new(Widget::Check {
                id: *id,
                label: rotulo.com_texto(nome),
                value: *value,
                off: *off,
            }),
            Self::Fato { rotulo, texto } => {
                Cell::new(Widget::Readout { text: texto }).label(rotulo)
            }
        }
    }

    fn fundo_sozinho(&self, ts: &mut TextSystem, w: f32, y: f32) -> f32 {
        match self {
            Self::Marca { .. } => y + m::FIELD_ROW_GAP + m::FIELD_H,
            Self::Numero { rotulo, .. }
            | Self::Amostra { rotulo, .. }
            | Self::Fato { rotulo, .. }
            | Self::Escolha { rotulo, .. } => {
                let reserva = if matches!(self, Self::Fato { .. }) {
                    0.0
                } else {
                    m::KEY_MARK_BOX
                };
                let (_, duas) = rotulos::linhas(ts, rotulo, w - reserva);
                let extra = if duas.is_some() {
                    m::LABEL_PX + m::LABEL_BOTTOM
                } else {
                    0.0
                };
                y + m::LABEL_TOP + m::LABEL_PX + m::LABEL_BOTTOM + extra + m::FIELD_H
            }
        }
    }
}

struct Espera {
    curto: Curto,
    x: f32,
    w: f32,
    y: f32,
    fim: f32,
}

struct Fluxo {
    espera: Option<Espera>,
}

thread_local! {
    static FLUXO: RefCell<Option<Fluxo>> = const { RefCell::new(None) };
}

/// ⭐ **Corre o corpo `f` com os pedidos curtos a fluir** (ver o módulo). O que ficou à espera pinta-se
/// no fim, recortado como a dobra da secção recorta o resto do corpo enquanto ela abre.
pub fn fluir(c: &mut CardCtx<'_>, f: impl FnOnce(&mut CardCtx<'_>) -> f32) -> f32 {
    FLUXO.with(|g| {
        *g.borrow_mut() = Some(Fluxo { espera: None });
    });
    crate::widget::card_ink::marca_fluxo(true);
    let fim = f(c);
    crate::widget::card_ink::marca_fluxo(false);
    let sobra = FLUXO.with(|g| g.borrow_mut().take()).and_then(|f| f.espera);
    if let Some(e) = sobra {
        let aberto = censo::secao_atual().map_or(1.0, |s| c.store.section_open_live(s));
        let banda = censo::secao_atual()
            .filter(|_| aberto < 1.0)
            .map(|s| c.store.section_body_h(s).unwrap_or(0.0) * aberto);
        if let Some(h) = banda {
            // O corpo está a abrir: o que passa da banda ainda não se vê.
            if e.y + m::FIELD_H > fim.min(e.y + h) {
                return fim;
            }
        }
        marca_spec(c.store, &e.curto);
        ctx_de_fora(
            c.scene,
            c.text_system,
            c.store,
            c.hit_index,
            c.ink,
            e.x,
            e.w,
        )
        .row_inteira(&[e.curto.celula(&e.curto.segs())], e.y);
    }
    fim
}

fn marca_spec(store: &WidgetStore, c: &Curto) {
    if let Curto::Numero { id, spec, .. } = c {
        store.mark_card_field_at_paint(*id, *spec);
    }
}

/// O pedido entra no fluxo: devolve o fundo, ou `None` sem fluxo (quem pede pinta como sempre). Só
/// se juntam pedidos da mesma coluna: um grupo recuado emparelha dentro dele.
#[allow(clippy::too_many_arguments)]
fn pede(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    store: &WidgetStore,
    hit: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    novo: impl FnOnce() -> Curto,
) -> Option<f32> {
    let antes = FLUXO.with(|g| g.borrow_mut().as_mut().map(|f| f.espera.take()))?;
    let novo = novo();
    if let Some(e) = antes {
        marca_spec(store, &e.curto);
        let mesma_coluna = (e.x - x).abs() < 0.5 && (e.w - w).abs() < 0.5;
        let vizinho = y >= e.fim - 0.5 && y <= e.fim + m::LABEL_TOP + m::FIELD_ROW_GAP;
        let mut c = ctx_de_fora(scene, ts, store, hit, ink, e.x, e.w);
        let (sa, sb) = (e.curto.segs(), novo.segs());
        let par = [e.curto.celula(&sa), novo.celula(&sb)];
        if mesma_coluna && vizinho && c.tudo_cabe(&par) {
            marca_spec(store, &novo);
            return Some(c.row_inteira(&par, e.y));
        }
        c.row_inteira(&par[..1], e.y);
    }
    let fim = novo.fundo_sozinho(ts, w, y);
    FLUXO.with(|g| {
        if let Some(f) = g.borrow_mut().as_mut() {
            f.espera = Some(Espera {
                curto: novo,
                x,
                w,
                y,
                fim,
            });
        }
    });
    Some(fim)
}

/// Um número de um campo só no fluxo.
#[allow(clippy::too_many_arguments)]
pub(super) fn pede_numero(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    store: &WidgetStore,
    hit: &mut HitIndex,
    ink: CardInk,
    (x, w, y): (f32, f32, f32),
    rotulo: String,
    letra: &str,
    (id, key): (NodeId, NodeId),
    spec: CardFieldSpec,
) -> Option<f32> {
    pede(scene, ts, store, hit, ink, x, w, y, || Curto::Numero {
        rotulo,
        letra: letra.to_owned(),
        id,
        key: [key],
        spec,
    })
}

/// Uma amostra com código no fluxo.
#[allow(clippy::too_many_arguments)]
pub(super) fn pede_amostra(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    store: &WidgetStore,
    hit: &mut HitIndex,
    ink: CardInk,
    (x, w, y): (f32, f32, f32),
    rotulo: &str,
    id: NodeId,
    rgba: Option<[u8; 4]>,
    codigo: &str,
) -> Option<f32> {
    pede(scene, ts, store, hit, ink, x, w, y, || Curto::Amostra {
        rotulo: rotulo.to_owned(),
        id: [id],
        rgba,
        codigo: codigo.to_owned(),
    })
}

/// Uma caixa de marcar no fluxo.
#[allow(clippy::too_many_arguments)]
pub(super) fn pede_marca(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    store: &WidgetStore,
    hit: &mut HitIndex,
    ink: CardInk,
    (x, w, y): (f32, f32, f32),
    id: NodeId,
    label: Rotulo<'_>,
    value: CheckboxValue,
    off: bool,
) -> Option<f32> {
    pede(scene, ts, store, hit, ink, x, w, y, || Curto::Marca {
        id,
        nome: label.texto.to_owned(),
        rotulo: label.com_texto(""),
        value,
        off,
    })
}

/// Uma escolha de duas opções no fluxo.
#[allow(clippy::too_many_arguments)]
pub(super) fn pede_escolha(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    store: &WidgetStore,
    hit: &mut HitIndex,
    ink: CardInk,
    (x, w, y): (f32, f32, f32),
    rotulo: &str,
    ids: &[NodeId],
    segs: &[Seg<'_>],
) -> Option<f32> {
    pede(scene, ts, store, hit, ink, x, w, y, || Curto::Escolha {
        rotulo: rotulo.to_owned(),
        ids: ids.to_vec(),
        opcoes: segs
            .iter()
            .map(|s| (s.id, s.icon, s.text.to_owned(), s.on))
            .collect(),
    })
}

/// ⭐ Um facto só de leitura como vizinho no fluxo; sem fluxo, a linha dele. Devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub fn fact(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    store: &WidgetStore,
    hit: &mut HitIndex,
    ink: CardInk,
    (x, w, y): (f32, f32, f32),
    rotulo: &str,
    texto: &str,
) -> f32 {
    pede(scene, ts, store, hit, ink, x, w, y, || Curto::Fato {
        rotulo: rotulo.to_owned(),
        texto: texto.to_owned(),
    })
    .unwrap_or_else(|| {
        let cell = Cell::new(Widget::Readout { text: texto }).label(rotulo);
        ctx_de_fora(scene, ts, store, hit, ink, x, w).row_inteira(&[cell], y)
    })
}

#[cfg(test)]
#[path = "cards_compacta_tests.rs"]
mod tests;
