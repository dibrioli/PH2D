//! ⭐ **Component, Layout e Widget em cartões** (fatia G7, `docs/CartoesUI/12_VECTOR_PLANO.md`). Mesmos
//! ids e eventos. As mesmas perguntas de antes: cada verbo só onde tem sujeito, o resto do Layout só com
//! a moldura a fluir, Cross só no Wrap, Cols só na grade, Grow e Shrink só para quem reparte a sobra.

use crate::cards_avancado::numero;
use crate::cards_fx::com_unidade;
use crate::cards_tinta::{
    aberto, celula_de_token, escolha, par_de_botoes, popover_de_token, seg, spec,
};
use crate::state::{self, LayoutFlow};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;

use crate::ids as p;

const PASSO_DE_MUNDO: f64 = 0.01; // LITERAL-PX-OK: distância de mundo, como o nó do Vertex
const PASSO_DA_RAZAO: f64 = 0.1; // LITERAL-PX-OK: Grow e Shrink são razões sem unidade
const COLUNAS: f64 = 1.0; // LITERAL-PX-OK: contagem inteira

const MIN_W: &[NodeId] = &[p::VECTOR_LAYOUT_MIN_W];
const MAX_W: &[NodeId] = &[p::VECTOR_LAYOUT_MAX_W];
const MIN_H: &[NodeId] = &[p::VECTOR_LAYOUT_MIN_H];
const MAX_H: &[NodeId] = &[p::VECTOR_LAYOUT_MAX_H];
const VAO: &[NodeId] = &[p::VECTOR_LAYOUT_GAP_MAIN];
const VAO_CRUZADO: &[NodeId] = &[p::VECTOR_LAYOUT_GAP_CROSS];
const COLS: &[NodeId] = &[p::VECTOR_LAYOUT_COLUMNS];
const TOKEN_VAO: &[NodeId] = &[p::VECTOR_TOKEN_GAP_MAIN];
const TOKEN_CRUZADO: &[NodeId] = &[p::VECTOR_TOKEN_GAP_CROSS];

/// Com o store MUTÁVEL, antes de pintar: as marcas. As sementes são as de sempre (`paint.rs`).
pub(crate) fn prepare(store: &mut WidgetStore) {
    for id in crate::populate::layout::LAYOUT_FIELDS.iter().copied() {
        let (passo, fabrica) = if id == p::VECTOR_LAYOUT_COLUMNS {
            (COLUNAS, None)
        } else if id == p::VECTOR_LAYOUT_ITEM_GROW {
            (PASSO_DA_RAZAO, Some(0.0))
        } else if id == p::VECTOR_LAYOUT_ITEM_SHRINK {
            (PASSO_DA_RAZAO, Some(1.0))
        } else {
            (PASSO_DE_MUNDO, Some(0.0))
        };
        store.mark_card_field(id, spec(passo, passo, fabrica));
    }
}

fn botao<'a>(id: NodeId, text: &'a str) -> Widget<'a> {
    Widget::Button {
        id,
        text,
        primary: false,
    }
}

/// ⭐ **Component**: os verbos aos pares. Quem escolhe entre dois fica ao lado do outro: instanciar
/// ligado ou não, trocar ou soltar, aplicar à receita ou voltar a ela.
pub(crate) fn component(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let Some(s) = state::component_state() else {
        return y;
    };
    if !s.is_main && !s.is_instance {
        return c.full(
            None,
            botao(
                p::VECTOR_COMPONENT_CREATE,
                tr("panel.vector.component.create"),
            ),
            y,
        );
    }
    let mut primeiros = Vec::with_capacity(2);
    if s.can_make_variant {
        primeiros.push(Cell::new(botao(
            p::VECTOR_COMPONENT_CREATE,
            tr("panel.vector.component.make_variant"),
        )));
    }
    if s.can_edit_prefab {
        primeiros.push(Cell::new(botao(
            p::VECTOR_COMPONENT_EDIT,
            tr("panel.vector.component.edit"),
        )));
    }
    let mut y = if primeiros.is_empty() {
        y
    } else {
        c.row(&primeiros, y)
    };
    if s.is_main {
        y = par_de_botoes(
            c,
            (
                p::VECTOR_COMPONENT_PLACE,
                tr("panel.vector.component.place"),
            ),
            (
                p::VECTOR_COMPONENT_PLACE_LINKED,
                tr("panel.vector.component.place_linked"),
            ),
            y,
        );
    }
    if !s.is_instance {
        return y;
    }
    if s.main_missing {
        y = c.hint(tr("panel.vector.component.missing"), y);
    }
    // O rótulo diz o que o próximo clique faz enquanto está armado; se não cabe, o par desce.
    let troca = if s.swap_armed {
        tr("panel.vector.component.swap_armed")
    } else {
        tr("panel.vector.component.swap")
    };
    y = par_de_botoes(
        c,
        (p::VECTOR_COMPONENT_SWAP, troca),
        (
            p::VECTOR_COMPONENT_DETACH,
            tr("panel.vector.component.detach"),
        ),
        y,
    );
    // Sem diferenças, os dois seriam cliques que não fazem nada.
    if s.has_overrides {
        y = par_de_botoes(
            c,
            (
                p::VECTOR_COMPONENT_UPDATE_MAIN,
                tr("panel.vector.component.update_main"),
            ),
            (
                p::VECTOR_COMPONENT_RESET,
                tr("panel.vector.component.reset"),
            ),
            y,
        );
    }
    y
}

/// ⭐ **Layout**: a moldura que dispõe e o filho que se comporta; numa moldura aninhada são as duas.
pub(crate) fn layout(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const CRESCE: &[NodeId] = &[p::VECTOR_LAYOUT_ITEM_GROW];
    const ENCOLHE: &[NodeId] = &[p::VECTOR_LAYOUT_ITEM_SHRINK];
    let mut y = y;
    if state::frame_clip().is_some() {
        y = moldura(c, y);
    }
    let Some(it) = state::layout_item() else {
        return y;
    };
    // Inerte é dito, não omitido: sem isto o artista não sabe que falta ligar o fluxo no pai.
    if !it.in_flow {
        return c.hint(tr("panel.vector.layout.needs_flow"), y);
    }
    y = c.check(
        crate::ids::VECTOR_LAYOUT_ITEM_ABSOLUTE,
        tr("panel.vector.layout.absolute"),
        it.absolute,
        y,
    );
    // Fora do fluxo não reparte sobra, e a grade não lê estes dois números.
    if !it.absolute && !it.parent_is_grid {
        y = c.row(
            &[
                numero(
                    p::VECTOR_LAYOUT_ITEM_GROW,
                    tr("panel.vector.layout.grow"),
                    CRESCE,
                ),
                numero(
                    p::VECTOR_LAYOUT_ITEM_SHRINK,
                    tr("panel.vector.layout.shrink"),
                    ENCOLHE,
                ),
            ],
            y,
        );
    }
    y
}

fn moldura(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const DIRECOES: [NodeId; 5] = [
        p::VECTOR_LAYOUT_DIR_OFF,
        p::VECTOR_LAYOUT_DIR_ROW,
        p::VECTOR_LAYOUT_DIR_COL,
        p::VECTOR_LAYOUT_DIR_WRAP,
        p::VECTOR_LAYOUT_DIR_GRID,
    ];
    const NOMES: [&str; 5] = [
        "panel.vector.layout.dir.off",
        "panel.vector.layout.dir.row",
        "panel.vector.layout.dir.col",
        "panel.vector.layout.dir.wrap",
        "panel.vector.layout.dir.grid",
    ];
    let flow = state::layout_flow();
    let dir = flow.map(|f| f.dir);
    let opcoes: Vec<_> = DIRECOES
        .iter()
        .zip(NOMES)
        .enumerate()
        .map(|(i, (id, nome))| {
            let aceso = if i == 0 {
                dir.is_none()
            } else {
                dir == Some(*id)
            };
            seg(*id, tr(nome), aceso)
        })
        .collect();
    let y = escolha(c, tr("panel.vector.layout.dir"), &DIRECOES, &opcoes, y);
    // Parada, os outros controlos não mudariam um pixel.
    let Some(f) = flow else {
        return y;
    };
    let y = tamanho(c, &f, y);
    let y = vaos(c, &f, y);
    let y = recuo(c, y);
    alinhamento(c, &f, y)
}

/// Um par por eixo: uma barra que ocupa a largura e abraça a altura é o caso mais comum de uma UI.
fn tamanho(c: &mut CardCtx<'_>, f: &LayoutFlow, y: f32) -> f32 {
    const L: [NodeId; 2] = [p::VECTOR_LAYOUT_SIZE_W_FIXED, p::VECTOR_LAYOUT_SIZE_W_HUG];
    const A: [NodeId; 2] = [p::VECTOR_LAYOUT_SIZE_H_FIXED, p::VECTOR_LAYOUT_SIZE_H_HUG];
    let (fixo, abraca) = (
        tr("panel.vector.layout.size.fixed"),
        tr("panel.vector.layout.size.hug"),
    );
    let largura = [
        seg(L[0], fixo, f.size[0] == L[0]),
        seg(L[1], abraca, f.size[0] == L[1]),
    ];
    let altura = [
        seg(A[0], fixo, f.size[1] == A[0]),
        seg(A[1], abraca, f.size[1] == A[1]),
    ];
    let y = c.row(
        &[
            Cell::new(Widget::Choice(&largura))
                .label(tr("panel.vector.layout.size.w"))
                .key(&L),
            Cell::new(Widget::Choice(&altura))
                .label(tr("panel.vector.layout.size.h"))
                .key(&A),
        ],
        y,
    );
    let metros = tr("card.vector.unit.metres");
    let nome = |chave| com_unidade(tr(chave), metros);
    let (min_l, max_l, min_a, max_a) = (
        nome("panel.vector.layout.min.w"),
        nome("panel.vector.layout.max.w"),
        nome("panel.vector.layout.min.h"),
        nome("panel.vector.layout.max.h"),
    );
    let y = c.row(
        &[
            numero(p::VECTOR_LAYOUT_MIN_W, &min_l, MIN_W),
            numero(p::VECTOR_LAYOUT_MAX_W, &max_l, MAX_W),
        ],
        y,
    );
    c.row(
        &[
            numero(p::VECTOR_LAYOUT_MIN_H, &min_a, MIN_H),
            numero(p::VECTOR_LAYOUT_MAX_H, &max_a, MAX_H),
        ],
        y,
    )
}

/// A rachura sobre o número que um token governa: o valor vem dele, não da caixa.
fn rachura(c: &mut CardCtx<'_>, id: NodeId) {
    if let Some(r) = c.hit_index.rect_for(id) {
        ph2d_editor_core::paint_shapes::fill_slash(
            c.scene,
            r,
            crate::paint_tokens::SLASH_W_PX,
            c.ink.color(ColorToken::Danger),
        );
    }
}

/// O vão e o token dele. O vão transversal só no Wrap, as colunas só na grade: fora deles não moveriam
/// um pixel. Sem companheiro, o token divide a linha com o vão.
fn vaos(c: &mut CardCtx<'_>, f: &LayoutFlow, y: f32) -> f32 {
    let bound = state::token_bindings().unwrap_or_default();
    let metros = tr("card.vector.unit.metres");
    let (nome, cruzado) = (
        com_unidade(tr("panel.vector.layout.gap"), metros),
        com_unidade(tr("panel.vector.layout.cross"), metros),
    );
    let vao = numero(p::VECTOR_LAYOUT_GAP_MAIN, &nome, VAO);
    let token = celula_de_token("panel.vector.token", TOKEN_VAO, bound.gap_main.as_deref());
    let wrap = f.dir == p::VECTOR_LAYOUT_DIR_WRAP;
    let y = if wrap {
        let y = c.row(
            &[
                vao,
                numero(p::VECTOR_LAYOUT_GAP_CROSS, &cruzado, VAO_CRUZADO),
            ],
            y,
        );
        c.row(
            &[
                token,
                celula_de_token(
                    "panel.vector.token",
                    TOKEN_CRUZADO,
                    bound.gap_cross.as_deref(),
                ),
            ],
            y,
        )
    } else if f.dir == p::VECTOR_LAYOUT_DIR_GRID {
        let y = c.row(
            &[
                numero(
                    p::VECTOR_LAYOUT_COLUMNS,
                    tr("panel.vector.layout.columns"),
                    COLS,
                ),
                vao,
            ],
            y,
        );
        c.row(&[token], y)
    } else {
        c.row(&[vao, token], y)
    };
    popover_de_token(c, TOKEN_VAO[0]);
    if bound.gap_main.is_some() {
        rachura(c, p::VECTOR_LAYOUT_GAP_MAIN);
    }
    if wrap {
        popover_de_token(c, TOKEN_CRUZADO[0]);
        if bound.gap_cross.is_some() {
            rachura(c, p::VECTOR_LAYOUT_GAP_CROSS);
        }
    }
    y
}

/// O recuo: All divide a linha com o número dele; Each dá os quatro lados, dois a dois.
fn recuo(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const TODOS: &[NodeId] = &[p::VECTOR_LAYOUT_PAD_ALL];
    const T: &[NodeId] = &[p::VECTOR_LAYOUT_PAD_T];
    const R: &[NodeId] = &[p::VECTOR_LAYOUT_PAD_R];
    const B: &[NodeId] = &[p::VECTOR_LAYOUT_PAD_B];
    const E: &[NodeId] = &[p::VECTOR_LAYOUT_PAD_L];
    let each = state::layout_pad_each();
    let modos = [
        seg(
            p::VECTOR_LAYOUT_PAD_ALL_MODE,
            tr("panel.vector.layout.padding.all"),
            !each,
        ),
        seg(
            p::VECTOR_LAYOUT_PAD_EACH_MODE,
            tr("panel.vector.layout.padding.each"),
            each,
        ),
    ];
    let modo = Cell::new(Widget::Choice(&modos)).label(tr("panel.vector.layout.padding"));
    let metros = tr("card.vector.unit.metres");
    if !each {
        let todos = com_unidade(tr("panel.vector.layout.all"), metros);
        return c.row(&[modo, numero(TODOS[0], &todos, TODOS)], y);
    }
    let y = c.row(&[modo], y);
    let (topo, direita, base, esquerda) = (
        com_unidade(tr("panel.vector.anchors.top"), metros),
        com_unidade(tr("panel.vector.anchors.right"), metros),
        com_unidade(tr("panel.vector.anchors.bottom"), metros),
        com_unidade(tr("panel.vector.anchors.left"), metros),
    );
    let y = c.row(&[numero(T[0], &topo, T), numero(R[0], &direita, R)], y);
    c.row(&[numero(B[0], &base, B), numero(E[0], &esquerda, E)], y)
}

/// Numa grade não sobra espaço a repartir: Between e Around não são oferecidos, e o valor guardado
/// não se reescreve, para o artista o reencontrar ao voltar.
fn alinhamento(c: &mut CardCtx<'_>, f: &LayoutFlow, y: f32) -> f32 {
    const ALINHA: [NodeId; 4] = [
        p::VECTOR_LAYOUT_ALIGN_START,
        p::VECTOR_LAYOUT_ALIGN_CENTER,
        p::VECTOR_LAYOUT_ALIGN_END,
        p::VECTOR_LAYOUT_ALIGN_STRETCH,
    ];
    const DISTRIBUI: [NodeId; 5] = [
        p::VECTOR_LAYOUT_JUSTIFY_START,
        p::VECTOR_LAYOUT_JUSTIFY_CENTER,
        p::VECTOR_LAYOUT_JUSTIFY_END,
        p::VECTOR_LAYOUT_JUSTIFY_BETWEEN,
        p::VECTOR_LAYOUT_JUSTIFY_AROUND,
    ];
    let aceso =
        |id: NodeId, vigente: NodeId, chave: &'static str| seg(id, tr(chave), id == vigente);
    let alinha = [
        aceso(ALINHA[0], f.align, "panel.vector.layout.align.start"),
        aceso(ALINHA[1], f.align, "panel.vector.layout.align.center"),
        aceso(ALINHA[2], f.align, "panel.vector.layout.align.end"),
        aceso(ALINHA[3], f.align, "panel.vector.layout.align.stretch"),
    ];
    let y = escolha(c, tr("panel.vector.layout.align"), &ALINHA, &alinha, y);
    let distribui = [
        aceso(DISTRIBUI[0], f.justify, "panel.vector.layout.justify.start"),
        aceso(
            DISTRIBUI[1],
            f.justify,
            "panel.vector.layout.justify.center",
        ),
        aceso(DISTRIBUI[2], f.justify, "panel.vector.layout.justify.end"),
        aceso(
            DISTRIBUI[3],
            f.justify,
            "panel.vector.layout.justify.between",
        ),
        aceso(
            DISTRIBUI[4],
            f.justify,
            "panel.vector.layout.justify.around",
        ),
    ];
    let n = if f.dir == p::VECTOR_LAYOUT_DIR_GRID {
        3
    } else {
        DISTRIBUI.len()
    };
    escolha(
        c,
        tr("panel.vector.layout.justify"),
        &DISTRIBUI[..n],
        &distribui[..n],
        y,
    )
}

/// ⭐ **Widget**: vestir, o tipo, e só o que esse tipo tem: ícone e quem ele dirige.
pub(crate) fn widget(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let Some(s) = state::widget_skin_state() else {
        return y;
    };
    let voltar = botao(p::VECTOR_WIDGET_REMOVE, tr("panel.vector.widget.remove"));
    let Some(vigente) = s.selected else {
        // O documento do futuro: sem a frase o artista conclui que a pele se perdeu.
        if s.unknown {
            let y = c.hint(tr("panel.vector.widget.unknown"), y);
            return c.full(None, voltar, y);
        }
        return c.full(
            None,
            botao(p::VECTOR_WIDGET_WEAR, tr("panel.vector.widget.wear")),
            y,
        );
    };
    let canal: Vec<NodeId> = (0..s.kinds.len()).map(p::vector_widget_kind_id).collect();
    let tipos: Vec<_> = s
        .kinds
        .iter()
        .enumerate()
        .map(|(i, nome)| seg(canal[i], nome, i == vigente))
        .collect();
    let mut y = escolha(c, tr("panel.vector.stroke.type"), &canal, &tipos, y);
    // Um tipo além do teto é inalcançável: o excedente escreve-se.
    let alem = state::widget_kinds_beyond();
    if alem > 0 {
        y = c.hint(
            &format!("{alem} {}", tr("panel.vector.component.variants_beyond")),
            y,
        );
    }
    y = c.hint(tr("panel.vector.widget.label_is_name"), y);
    // O ícone e o alvo dividem a linha; o que falta deixa o outro com ela inteira.
    let face = crate::icon_dropdown::chip_label(&s);
    let chave_do_icone = [p::VECTOR_WIDGET_ICON_DD];
    let icone = face.as_deref().map(|text| {
        Cell::new(Widget::Dropdown {
            id: p::VECTOR_WIDGET_ICON_DD,
            text,
            options: &[],
        })
        .label(tr("card.vector.widget.icon"))
        .key(&chave_do_icone)
    });
    let nada = tr("panel.vector.widget.drives_none");
    let alvo = s.drives.as_ref().map(|a| {
        Cell::new(Widget::Readout {
            text: a.as_deref().unwrap_or(nada),
        })
        .label(tr("panel.vector.widget.drives"))
    });
    y = match (icone, alvo) {
        (Some(a), Some(b)) => c.pair(a, b, y),
        (Some(a), None) | (None, Some(a)) => c.row(&[a], y),
        (None, None) => y,
    };
    if face.is_some()
        && let Some(caixa) = aberto(c, p::VECTOR_WIDGET_ICON_DD)
    {
        // A caixa divide a linha com Drives; os nomes dos glifos pedem a largura do cartão.
        state::set_pending_icon_dd(Some(Rect {
            x: c.x,
            w: c.w,
            ..caixa
        }));
    }
    if let Some(preso) = s.drives.as_ref() {
        let ligar = Cell::new(botao(p::VECTOR_WIDGET_BIND, tr("panel.vector.widget.bind")));
        y = if preso.is_some() {
            let soltar = Cell::new(botao(
                p::VECTOR_WIDGET_UNBIND,
                tr("panel.vector.widget.unbind"),
            ));
            c.row(&[ligar, soltar], y)
        } else {
            c.row(&[ligar], y)
        };
    }
    c.full(None, voltar, y)
}
