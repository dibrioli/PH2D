//! As peças comuns das linhas de cartão do painel Painter (`docs/CartoesUI/13_PAINTER_PLANO.md`): as listas de caixa do
//! cartão, a linha esmaecida da lei, e os pares.

use crate::paint_wetpaint::Numero;
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, corpo as linhas};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::widget::DropdownState;
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;
use ph2d_vector::{Affine, Compose, Mix, RoundedRect, VelloBlend};

/// As listas destes cartões: a caixa é a do cartão, e a lista que abre é a de sempre (a do Preset arrasta linhas).
pub(crate) fn registra_lista(store: &mut WidgetStore, id: NodeId, atual: usize) {
    store.register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(atual),
        },
    );
}

pub(crate) fn aberta(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// Os itens de dois em dois, na mesma linha, cada um na sua metade; o que sobra fica com a linha inteira.
pub(crate) fn em_pares<T>(
    c: &mut CardCtx<'_>,
    itens: &[T],
    y: f32,
    mut pinta: impl FnMut(&mut CardCtx<'_>, &T, f32) -> f32,
) -> f32 {
    let (x, w) = (c.x, c.w);
    let meia = ((w - m::FIELD_GAP) * 0.5).max(0.0);
    let mut topo = y;
    // Coluna estreita: o par desce, nunca corta o valor (05 §4, lei 5).
    let por_linha = if meia < m::ICON_CELL + m::FIELD_VALUE_MIN_W {
        1
    } else {
        2
    };
    for par in itens.chunks(por_linha) {
        let mut fundo = topo;
        for (i, item) in par.iter().enumerate() {
            (c.x, c.w) = if par.len() == 2 {
                (x + i as f32 * (meia + m::FIELD_GAP), meia)
            } else {
                (x, w)
            };
            fundo = fundo.max(pinta(c, item, topo));
        }
        topo = fundo;
    }
    c.x = x;
    c.w = w;
    topo
}

/// Botões de acção na mesma linha; a fundação desce o que não cabe inteiro (05 §4, lei 7).
pub(crate) fn botoes<S: AsRef<str>>(c: &mut CardCtx<'_>, lista: &[(NodeId, S)], y: f32) -> f32 {
    let cells: Vec<Cell<'_>> = lista
        .iter()
        .map(|(id, text)| {
            Cell::new(Widget::Button {
                id: *id,
                text: text.as_ref(),
                primary: false,
            })
        })
        .collect();
    c.row(&cells, y)
}

/// Uma caixa de marcar como célula de uma linha: a 2.ª coluna de um campo, ou o par de outra caixa.
pub(crate) fn marca<'a>(id: NodeId, chave: &'static str, on: bool) -> Cell<'a> {
    Cell::new(Widget::Check {
        id,
        label: Rotulo::vista(TextKey::new(chave)),
        value: if on {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        },
        off: false,
    })
}

/// Um número das tabelas destes cartões.
pub(crate) fn numero(
    chave: &'static str,
    id: NodeId,
    valor: f32,
    (min, max): (f32, f32),
    passo: f64,
    casas: usize,
) -> Numero {
    Numero {
        chave,
        id,
        valor,
        min,
        max,
        passo,
        casas,
    }
}

/// Os números de um padrão, na escala em que se mostram e se digitam.
pub(crate) fn do_padrao(
    kind: ph2d_tool_painter::TextureKind,
    ids: &[NodeId],
    valores: &[f32],
) -> Vec<Numero> {
    ph2d_tool_painter::param_specs(kind)
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let e = ph2d_tool_painter::param_escala(kind, i);
            numero(
                s.label,
                ids[i],
                e.mostra(valores[i]),
                (e.min, e.max),
                e.step,
                e.decimals(),
            )
        })
        .collect()
}

/// A linha de grupo: um filete e o nome. A lei é «nunca um 2.º título», por isso o tom é o dos rótulos.
pub(crate) fn grupo(c: &mut CardCtx<'_>, chave: &str, y: f32) -> f32 {
    let fim = filete(c, y);
    c.hint(tr(chave), fim)
}

/// O grupo cuja 1.ª linha não tem rótulo: o nome dele é o rótulo dela, a `LABEL_BOTTOM` do controlo.
pub(crate) fn grupo_sobre_controlo(c: &mut CardCtx<'_>, chave: &str, y: f32) -> f32 {
    let fim = filete(c, y);
    rotulo_sobre_controlo(c, tr(chave), fim)
}

/// Um rótulo solto a `LABEL_TOP` de cima: o `y` devolvido é para quem traz `FIELD_ROW_GAP` e fica a
/// `LABEL_BOTTOM` dele.
pub(crate) fn rotulo_sobre_controlo(c: &mut CardCtx<'_>, texto: &str, y: f32) -> f32 {
    let topo = y + m::LABEL_TOP;
    ph2d_editor_core::widget::card_field::paint_label(
        c.scene,
        c.text_system,
        c.ink,
        c.x,
        topo,
        c.w,
        texto,
    );
    topo + m::LABEL_PX + m::LABEL_BOTTOM - m::FIELD_ROW_GAP
}

/// Só o filete: o grupo cujo nome já é o rótulo da 1.ª linha dele. Devolve o fundo.
pub(crate) fn filete(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let r = Rect::new(c.x, y + m::CARD_HEAD_BOTTOM, c.w, m::FIELD_BORDER);
    let cor = c.ink.color(ColorToken::CardField);
    fill_rounded_rect(c.scene, r, 0.0, cor);
    r.y + r.h
}

/// O grupo que abre o corpo: por cima dele já está o cabeçalho, e o filete sobrava.
pub(crate) fn grupo_inicial(c: &mut CardCtx<'_>, chave: &str, y: f32) -> f32 {
    c.hint(tr(chave), y - m::FIELD_ROW_GAP)
}

/// O passo do chip e o número que ele mostra para a pista: a lei é a do store.
pub(crate) fn arma_barra(
    store: &mut WidgetStore,
    [slider, chip]: [NodeId; 2],
    pista: f32,
    de_fabrica: f32,
    casas: usize,
) {
    let passo = store
        .number_range(chip)
        .map_or(crate::cards_topo::PASSO_DO_PINCEL, |(_, _, s)| s);
    let padrao = store.linked_chip_display(chip, de_fabrica);
    store.mark_card_field(
        chip,
        ph2d_editor_core::widget::card_field::CardFieldSpec {
            step: passo,
            scrub: passo,
            default: Some(f64::from(padrao)),
        },
    );
    let mostrado = store.linked_chip_display(chip, pista);
    crate::number_field::mirror_value(store, chip, mostrado, casas);
    let _ = crate::esmaecer::poe_dica(store, &[slider, chip]);
}

/// Uma barra com o chip ligado, numa tabela: a pista lê-se do pincel e o número é o que o store projecta.
pub(crate) struct Pista {
    pub(crate) chave: &'static str,
    pub(crate) ids: [NodeId; 2],
    pub(crate) le: fn(&ph2d_tool_painter::BrushSettings) -> f32,
    pub(crate) casas: usize,
}

impl Pista {
    pub(crate) fn item(&self) -> Item {
        Item::Barra {
            chave: self.chave,
            ids: self.ids,
        }
    }

    pub(crate) fn arma(&self, store: &mut WidgetStore, b: &ph2d_tool_painter::BrushSettings) {
        let fabrica = (self.le)(&crate::paint_brush::FALLBACK_BRUSH);
        arma_barra(store, self.ids, (self.le)(b), fabrica, self.casas);
    }
}

/// Um controlo curto de uma fila: divide a linha com o vizinho, se os dois cabem inteiros.
pub(crate) enum Item {
    Num(Numero),
    /// Lista sem marca de animação, como `paint_dropdown_row`.
    Lista {
        id: NodeId,
        chave: &'static str,
        texto: &'static str,
    },
    Caixa {
        id: NodeId,
        chave: &'static str,
        on: bool,
    },
    /// Lista com a marca de animação do id, como `paint_dropdown_row_de_caixa`.
    ListaComMarca {
        id: NodeId,
        chave: &'static str,
        texto: &'static str,
    },
    /// Uma barra com o chip ligado: `[barra, chip]`. A caixa é o chip, e a barra continua a avisar o painel.
    Barra {
        chave: &'static str,
        ids: [NodeId; 2],
    },
    /// Uma amostra com o código, na cor que o `prepare` pôs no store.
    Cor {
        id: NodeId,
        chave: &'static str,
        rgba: [u8; 4],
        codigo: String,
    },
}

impl Item {
    pub(crate) fn cor(store: &WidgetStore, id: NodeId, chave: &'static str) -> Self {
        let rgba = store.widget_color(id).unwrap_or([0, 0, 0, u8::MAX]);
        Self::Cor {
            id,
            chave,
            rgba,
            codigo: format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]),
        }
    }

    fn id(&self) -> NodeId {
        match self {
            Self::Num(n) => n.id,
            Self::Barra { ids, .. } => ids[1],
            Self::Lista { id, .. }
            | Self::ListaComMarca { id, .. }
            | Self::Caixa { id, .. }
            | Self::Cor { id, .. } => *id,
        }
    }

    fn inerte(&self, inercias: &ph2d_tool_painter::Inercias) -> bool {
        if let Self::Barra { ids, .. } = self
            && inercias.dica_de(ids[0]).is_some()
        {
            return true;
        }
        inercias.dica_de(self.id()).is_some()
    }

    pub(crate) fn cell(&self) -> Cell<'_> {
        match self {
            Self::Num(n) => {
                let rotulo = tr(n.chave);
                Cell::new(Widget::Field {
                    id: n.id,
                    glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
                })
                .label(rotulo)
                .key(std::slice::from_ref(&n.id))
            }
            Self::Lista { id, chave, texto } => Cell::new(Widget::Dropdown {
                id: *id,
                text: texto,
                options: &[],
            })
            .label(tr(chave)),
            Self::ListaComMarca { id, chave, texto } => Cell::new(Widget::Dropdown {
                id: *id,
                text: texto,
                options: &[],
            })
            .label(tr(chave))
            .key(std::slice::from_ref(id)),
            Self::Barra { chave, ids } => {
                let rotulo = tr(chave);
                Cell::new(Widget::Field {
                    id: ids[1],
                    glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
                })
                .label(rotulo)
                .key(ids)
            }
            Self::Caixa { id, chave, on } => marca(*id, chave, *on),
            Self::Cor {
                id,
                chave,
                rgba,
                codigo,
            } => Cell::new(Widget::Swatch {
                id: *id,
                rgba: Some(*rgba),
                text: codigo,
                edit: None,
                detach: None,
            })
            .label(tr(chave))
            .key(std::slice::from_ref(id)),
        }
    }
}

/// Os itens dois a dois enquanto couberem (05 §4, leis 1 e 5). O que a ferramenta diz inerte fica esmaecido na sua
/// metade, com o vizinho vivo.
pub(crate) fn fila(c: &mut CardCtx<'_>, itens: &[Item], y: f32) -> f32 {
    let inercias = state::current_inercias();
    let inerte = |i: &Item| i.inerte(&inercias);
    let mut y = y;
    let mut resto = itens;
    while !resto.is_empty() {
        let vivos = resto.iter().take_while(|i| !inerte(i)).count();
        // Um número ímpar de vivos antes de um inerte: o último faz par com ele, em vez de ficar sozinho.
        let corrida = if vivos < resto.len() && vivos % 2 == 1 {
            vivos - 1
        } else {
            vivos
        };
        let cells: Vec<Cell<'_>> = resto[..corrida].iter().map(Item::cell).collect();
        y = c.compact(&cells, y);
        resto = &resto[corrida..];
        if !resto.is_empty() {
            let n = resto.len().min(2);
            y = em_pares(c, &resto[..n], y, |c, item, y| {
                com_lei(c, item.id(), y, |c, y| c.row(&[item.cell()], y))
            });
            resto = &resto[n..];
        }
    }
    y
}

/// Dois números de UMA grandeza: um rótulo, as letras X e Y, uma marca para o par.
pub(crate) fn par_xy(c: &mut CardCtx<'_>, chave: &str, ids: [NodeId; 2], y: f32) -> f32 {
    let inerte = {
        let i = state::current_inercias();
        i.dica_de(ids[0]).is_some() || i.dica_de(ids[1]).is_some()
    };
    let cells = [
        Cell::new(Widget::Field {
            id: ids[0],
            glyph: Glyph::Text(tr("card.glyph.x")),
        })
        .label(tr(chave)),
        Cell::new(Widget::Field {
            id: ids[1],
            glyph: Glyph::Text(tr("card.glyph.y")),
        })
        .key(&ids),
    ];
    if inerte {
        esmaecida(c, y, |c, y| c.row(&cells, y))
    } else {
        c.row(&cells, y)
    }
}

/// O véu da opção que a ferramenta diz inerte: continua clicável, como a linha esmaecida.
fn opcoes_inertes(c: &mut CardCtx<'_>, segs: &[Seg<'_>]) {
    let inercias = state::current_inercias();
    let alfa = ph2d_editor_core::property_row::inativa::alfa_inativo(c.ink.theme);
    for s in segs {
        if inercias.dica_de(s.id).is_some()
            && let Some(r) = c.hit_index.rect_for(s.id)
        {
            let veu = c
                .ink
                .color(ColorToken::CardField)
                .multiply_alpha(1.0 - alfa);
            fill_rounded_rect(c.scene, r, m::SEG_RADIUS, veu);
        }
    }
}

/// Uma escolha com rótulo. Se é longa, a fundação desce-a em filas, sem cortar os nomes.
pub(crate) fn escolha(c: &mut CardCtx<'_>, rotulo: &str, segs: &[Seg<'_>], y: f32) -> f32 {
    let fim = c.full(Some(rotulo), Widget::Choice(segs), y);
    opcoes_inertes(c, segs);
    fim
}

/// Um número e a cor irmã na mesma linha (Wax + Wax Color, Intensity + Color).
pub(crate) fn numero_e_cor(
    c: &mut CardCtx<'_>,
    n: &Numero,
    chave_da_cor: &'static str,
    id_da_cor: NodeId,
    y: f32,
) -> f32 {
    let [r, g, b, a] = c
        .store
        .widget_color(id_da_cor)
        .unwrap_or([0, 0, 0, u8::MAX]);
    let codigo = format!("{r:02X}{g:02X}{b:02X}");
    let rotulo = tr(n.chave);
    let cells = [
        Cell::new(Widget::Field {
            id: n.id,
            glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
        })
        .label(rotulo)
        .key(std::slice::from_ref(&n.id)),
        Cell::new(Widget::Swatch {
            id: id_da_cor,
            rgba: Some([r, g, b, a]),
            text: &codigo,
            edit: None,
            detach: None,
        })
        .label(tr(chave_da_cor))
        .key(std::slice::from_ref(&id_da_cor)),
    ];
    c.row(&cells, y)
}

/// A altura de uma linha com rótulo, de sobra: a camada só recorta, nunca empurra.
fn altura_de_linha() -> f32 {
    m::LABEL_TOP + 2.0 * (m::LABEL_PX + m::LABEL_BOTTOM) + m::FIELD_ROW_GAP + m::FIELD_H
}

/// Esmaecida como a linha de antes: continua editável, salvo a do Color inativo, que o chamador tira do índice.
pub(crate) fn esmaecida(
    c: &mut CardCtx<'_>,
    y: f32,
    pintar: impl FnOnce(&mut CardCtx<'_>, f32) -> f32,
) -> f32 {
    let forma = RoundedRect::new(
        f64::from(c.x),
        f64::from(y),
        f64::from(c.x + c.w),
        f64::from(y + altura_de_linha()),
        0.0,
    );
    c.scene.push_layer_shape(
        VelloBlend::new(Mix::Normal, Compose::SrcOver),
        ph2d_editor_core::property_row::inativa::alfa_inativo(c.ink.theme),
        Affine::IDENTITY,
        &forma,
    );
    let fim = pintar(c, y);
    c.scene.pop_layer();
    fim
}

/// A linha do id `id`, esmaecida se a ferramenta o diz inerte.
pub(crate) fn com_lei(
    c: &mut CardCtx<'_>,
    id: NodeId,
    y: f32,
    pintar: impl FnOnce(&mut CardCtx<'_>, f32) -> f32,
) -> f32 {
    if state::current_inercias().dica_de(id).is_some() {
        esmaecida(c, y, pintar)
    } else {
        pintar(c, y)
    }
}
