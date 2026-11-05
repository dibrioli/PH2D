//! Os cartões do topo e dos básicos, em linhas de cartão (fatia H2, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos
//! ids, mesmos eventos. As condições são as de `paint_brush::plano_do_corpo` e
//! `paint_top_basics`, e `every_old_id_is_still_painted_in_each_tool_and_medium` confere-as em 13 estados.

use crate::cards_linhas::{aberta, botoes, com_lei, em_pares, esmaecida, marca, registra_lista};
use crate::paint_wetpaint_presets_biblioteca as biblioteca;
use crate::state_wet_presets::{self as st, VistaPresets};
use crate::{cards_composto, cards_ferramenta, cards_wet, number_field, paint_brush};
use crate::{paint_wetpaint_presets, state};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, corpo as linhas, ferramenta};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{
    BrushBlend, BrushSettings, FerramentaDoPincel, LinhaDaFerramenta, PaintMedia,
};

/// O passo dos campos 0 a 1 do pincel, o de `populate_brush_chips`.
pub(crate) const PASSO_DO_PINCEL: f64 = 0.01; // LITERAL-PX-OK: passo da faixa 0..1, não é medida de UI
/// As casas com que a caixa mostra um 0 a 1, as de antes.
pub(crate) const CASAS_DO_PINCEL: usize = 3;
/// «On empty» é um interruptor em forma de número: 0 ou 1.
const VAZIO_FAIXA: (f32, f32, f64) = (0.0, 1.0, 1.0);

/// Um número básico do pincel; dois irmãos partilham a linha.
#[derive(Copy, Clone)]
pub(crate) enum Numero {
    Tamanho,
    Forca,
    Fluxo,
}

impl Numero {
    fn dados(self) -> (&'static str, &'static [NodeId; 2]) {
        match self {
            Self::Tamanho => (
                "panel.painter_layers.brush.size",
                &[
                    pids::PAINTER_BRUSH_SIZE_SLIDER,
                    pids::PAINTER_BRUSH_SIZE_CHIP,
                ],
            ),
            Self::Forca => (
                "panel.painter_layers.brush.strength",
                &[
                    pids::PAINTER_BRUSH_STRENGTH_SLIDER,
                    pids::PAINTER_BRUSH_STRENGTH_CHIP,
                ],
            ),
            Self::Fluxo => (
                "panel.painter_layers.brush.flow",
                &[
                    pids::PAINTER_BRUSH_FLOW_SLIDER,
                    pids::PAINTER_BRUSH_FLOW_CHIP,
                ],
            ),
        }
    }

    fn de_fabrica(self, b: &BrushSettings) -> f32 {
        match self {
            Self::Tamanho => b.size_norm,
            Self::Forca => b.strength,
            Self::Fluxo => b.flow,
        }
    }
}

/// Uma linha de um destes cartões.
pub(crate) enum Linha {
    /// `true` = Sync divide a linha com ela.
    Modo(bool),
    /// A lista do Preset, com a estrela do favorito à direita se há preset activo.
    Preset,
    Acoes,
    /// `true` = Sync divide a linha com ela.
    Politica(bool),
    Sync,
    Blend,
    Vazio,
    Cor,
    Numeros(Vec<Numero>),
    Accumulate,
    Wash,
    Fio,
    Clone,
    Inpaint,
    /// A linha do traço, com o número que sobrou à frente e a caixa do Composite atrás: irmãos curtos.
    Traco {
        sobra: Option<(&'static str, &'static [NodeId; 2])>,
        composto: bool,
    },
    Composite,
    /// A prévia do pincel, se há quadro publicado e ela não está presa na faixa.
    Previa,
    /// Um volume vivo do Wet Paint, pelo índice na tabela.
    Volume(usize),
    /// Duas linhas lado a lado.
    Par(Box<[Linha; 2]>),
}

/// O cartão do topo está a voar?
pub(crate) fn topo_a_voar(store: &WidgetStore) -> bool {
    store
        .card_drag()
        .is_some_and(|d| d.active && !d.loose && d.section == pids::PAINTER_CARD_TOPO)
}

/// As linhas do cartão sem título do topo.
pub(crate) fn do_topo(brush: BrushSettings) -> Vec<Linha> {
    let vista = st::current_wet_presets();
    let mut l = Vec::new();
    let tem_modo = !brush.is_inpaint && brush.meios.count_ones() > 1;
    if tem_modo {
        l.push(Linha::Modo(vista.is_none()));
    }
    l.push(Linha::Previa);
    if vista.is_some() {
        l.push(Linha::Preset);
        l.push(Linha::Acoes);
    }
    if brush.wetpaint {
        // Size inteiro; Dilution e Volume, as duas grandezas da tinta, dividem a linha.
        let n = paint_wetpaint_presets::numeros_dos_volumes(&brush, &brush).len();
        let sozinhos = n % 2;
        l.extend((0..sozinhos).map(Linha::Volume));
        let pares = (sozinhos..n).step_by(2);
        l.extend(pares.map(|i| Linha::Par(Box::new([Linha::Volume(i), Linha::Volume(i + 1)]))));
    }
    if vista.is_some() {
        l.push(Linha::Politica(true));
    }
    if vista.is_none() && !tem_modo {
        l.push(Linha::Sync);
    }
    l
}

/// As linhas do cartão Basics. Vazio = o cartão não está à vista.
pub(crate) fn dos_basicos(brush: BrushSettings) -> Vec<Linha> {
    let mut l = Vec::new();
    let le_a_cor = brush.le(LinhaDaFerramenta::Cor);
    if le_a_cor && !brush.watercolor && brush.le(LinhaDaFerramenta::Blend) {
        if !brush.paints_no_color() && !brush.is_mask {
            l.push(Linha::Par(Box::new([Linha::Blend, Linha::Vazio])));
        } else {
            l.push(Linha::Blend);
        }
    }
    let mut numeros = Vec::new();
    if !brush.wetpaint {
        numeros.push(Numero::Tamanho);
    }
    if !brush.composite_on() && !brush.is_inpaint && !brush.wetpaint && !brush.watercolor_active {
        numeros.push(Numero::Forca);
    }
    if brush.le_o_flow() {
        numeros.push(Numero::Fluxo);
    }
    // A cor divide a linha com o número que ficar sozinho, e os outros vão aos pares.
    if le_a_cor {
        if numeros.len() % 2 == 1 {
            let sozinho = numeros.remove(numeros.len() - 1);
            l.push(Linha::Par(Box::new([
                Linha::Cor,
                Linha::Numeros(vec![sozinho]),
            ])));
        } else {
            l.push(Linha::Cor);
        }
    }
    // O número que sobra divide a fila da linha do traço, se nada se põe entre os dois.
    let sobra = if !brush.is_clone && !brush.is_inpaint && numeros.len() % 2 == 1 {
        numeros.pop()
    } else {
        None
    };
    l.extend(numeros.chunks(2).map(|par| Linha::Numeros(par.to_vec())));
    if brush.is_clone {
        l.push(Linha::Clone);
    }
    if brush.is_inpaint {
        l.push(Linha::Inpaint);
    }
    let composto = brush.le(LinhaDaFerramenta::Composite) && brush.composite_offered();
    l.push(Linha::Traco {
        sobra: sobra.map(|n| n.dados()),
        composto,
    });
    if composto {
        l.push(Linha::Composite);
    }
    if brush.accumulate_offered() {
        l.push(Linha::Par(Box::new([Linha::Accumulate, Linha::Wash])));
    }
    if brush.ferramenta() == FerramentaDoPincel::Smear
        && PaintMedia::from_u8(brush.media) == PaintMedia::Digital
    {
        l.push(Linha::Fio);
    }
    l
}

/// Com o store MUTÁVEL, antes de pintar: o que estas linhas leem e não podem escrever.
pub(crate) fn prepare(store: &mut WidgetStore, b: &BrushSettings) {
    let fabrica = paint_brush::FALLBACK_BRUSH;
    for n in [Numero::Tamanho, Numero::Forca, Numero::Fluxo] {
        let chip = n.dados().1[1];
        store.mark_card_field(
            chip,
            CardFieldSpec {
                step: PASSO_DO_PINCEL,
                scrub: PASSO_DO_PINCEL,
                default: Some(f64::from(n.de_fabrica(&fabrica))),
            },
        );
        number_field::mirror_value(store, chip, n.de_fabrica(b), CASAS_DO_PINCEL);
    }
    let vazio = pids::PAINTER_BRUSH_BLEND_ON_EMPTY;
    store.mark_card_field(
        vazio,
        CardFieldSpec {
            step: VAZIO_FAIXA.2,
            scrub: VAZIO_FAIXA.2,
            default: Some(f64::from(u8::from(fabrica.blend_sobre_vazio))),
        },
    );
    let (min, max, passo) = VAZIO_FAIXA;
    number_field::arm_field(
        store,
        vazio,
        f32::from(u8::from(b.blend_sobre_vazio)),
        min,
        max,
        passo,
        0,
    );
    registra_lista(store, pids::PAINTER_BRUSH_MEDIA, usize::from(b.media));
    registra_lista(store, pids::PAINTER_BRUSH_BLEND, usize::from(b.blend));
    registra_lista(store, pids::PAINTER_WETPAINT_PRESET, 0);
    let politica = st::current_wet_presets().map_or(0, |v| v.politica);
    registra_lista(
        store,
        pids::PAINTER_WETPAINT_PRESET_POLICY,
        usize::from(politica),
    );
    let cor = core_ids::PAINTER_COLOR_THUMB;
    if b.color_inactive && store.picker_target() == Some(cor) {
        store.set_picker_target(None);
    }
    if b.wetpaint {
        cards_wet::prepara_volumes(store, b);
    }
    for id in COM_LEI {
        let _ = crate::esmaecer::poe_dica(store, &[id]);
    }
    store.set_tooltip(
        pids::PAINTER_BRUSH_SYNC,
        tr("panel.painter_layers.brush.sync_tools"),
    );
    let favorita = st::current_wet_presets().is_some_and(|v| {
        v.ativo
            .as_deref()
            .and_then(|r| v.procurar(r))
            .is_some_and(|a| a.favorito)
    });
    store.set_tooltip(
        pids::PAINTER_WETPAINT_PRESET_FAVORITE,
        tr(if favorita {
            "card.painter.preset.favorite_remove"
        } else {
            "card.painter.preset.favorite_add"
        }),
    );
}

/// Os ids destas linhas com linha esmaecida (`Dependente`): a lei é da ferramenta, e aqui só se pinta.
const COM_LEI: [NodeId; 3] = [
    pids::PAINTER_BRUSH_ACCUMULATE,
    pids::PAINTER_BRUSH_WASH,
    pids::PAINTER_BRUSH_BLEND_ON_EMPTY,
];

/// Depois da lista: onde cada caixa aberta ficou é o que a lista das opções precisa.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    let vistos = [
        pids::PAINTER_BRUSH_MEDIA,
        pids::PAINTER_BRUSH_BLEND,
        pids::PAINTER_WETPAINT_PRESET,
        pids::PAINTER_WETPAINT_PRESET_POLICY,
    ];
    for id in vistos {
        if !aberta(ctx.host.store(), id) {
            continue;
        }
        let Some(r) = ctx.host.hit_index_mut().rect_for(id) else {
            continue;
        };
        if id == pids::PAINTER_BRUSH_MEDIA {
            state::set_pending_brush_media_dd(Some((r, b.media)));
        } else if id == pids::PAINTER_BRUSH_BLEND {
            state::set_pending_brush_blend_dd(Some((r, b.blend)));
        } else if id == pids::PAINTER_WETPAINT_PRESET {
            st::PENDING_PRESET_DD.with(|c| c.set(Some(r)));
        } else if let Some(v) = st::current_wet_presets() {
            st::PENDING_POLICY_DD.with(|c| c.set(Some((r, v.politica))));
        }
    }
    for vol in 0..pids::PAINTER_WETPAINT_VOL_QUICK.len() {
        let id = pids::PAINTER_WETPAINT_VOL_QUICK[vol];
        if aberta(ctx.host.store(), id)
            && let Some(r) = ctx.host.hit_index_mut().rect_for(id)
        {
            st::PENDING_QUICK_DD.with(|c| c.set(Some((r, vol))));
        }
    }
}

fn lista(c: &mut CardCtx<'_>, chave: &str, id: NodeId, texto: &str, y: f32) -> f32 {
    c.full(
        Some(tr(chave)),
        Widget::Dropdown {
            id,
            text: texto,
            options: &[],
        },
        y,
    )
}

/// Uma lista e o Sync, irmãos curtos na mesma linha.
fn lista_e_sync(
    c: &mut CardCtx<'_>,
    b: &BrushSettings,
    com_sync: bool,
    (chave, id): (&str, NodeId),
    texto: &str,
    opcoes: &[&str],
    y: f32,
) -> f32 {
    if !com_sync {
        return lista(c, chave, id, texto, y);
    }
    let celula = Cell::new(Widget::Dropdown {
        id,
        text: texto,
        options: &[],
    })
    .label(tr(chave));
    let sync = marca(
        pids::PAINTER_BRUSH_SYNC,
        "card.painter.brush.sync",
        b.link_shared,
    );
    // Lei 5 de 05 §4: o par desce em vez de cortar «Sync tools» ou a opção mais longa da lista. Mede-se
    // a mais longa, não a de agora: escolher outra opção não muda a arrumação.
    let coluna = (c.w - m::FIELD_GAP) * 0.5;
    let nome = c
        .text_system
        .prefix_width(tr("card.painter.brush.sync"), m::VALUE_PX);
    let longa = opcoes
        .iter()
        .map(|o| c.text_system.prefix_width(o, m::VALUE_PX).ceil())
        .fold(0.0_f32, f32::max);
    let caixa = longa + m::FIELD_PAD_R * 2.0 + m::ICON_GLYPH;
    if nome + m::ICON_CELL + m::FIELD_PAD_R > coluna || caixa > coluna {
        let y = c.row(std::slice::from_ref(&celula), y);
        return c.row(std::slice::from_ref(&sync), y);
    }
    c.row(&[celula, sync], y)
}

fn numero_em_caixa(rotulo: &str, id: NodeId) -> Widget<'_> {
    Widget::Field {
        id,
        glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
    }
}

fn numeros(c: &mut CardCtx<'_>, par: &[Numero], y: f32) -> f32 {
    let cells: Vec<Cell<'_>> = par
        .iter()
        .map(|n| {
            let (chave, ids) = n.dados();
            let rotulo = tr(chave);
            Cell::new(numero_em_caixa(rotulo, ids[1]))
                .label(rotulo)
                .key(ids)
        })
        .collect();
    c.row(&cells, y)
}

fn cor(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let id = core_ids::PAINTER_COLOR_THUMB;
    let [r, g, bl, a] = c.store.widget_color(id).unwrap_or([0, 0, 0, u8::MAX]);
    let codigo = format!("{r:02X}{g:02X}{bl:02X}");
    let linha = |c: &mut CardCtx<'_>, y: f32| {
        let cell = Cell::new(Widget::Swatch {
            id,
            rgba: Some([r, g, bl, a]),
            text: &codigo,
            edit: None,
            detach: None,
        })
        .label(tr("panel.painter_layers.brush.color"))
        .key(&[core_ids::PAINTER_COLOR_THUMB]);
        c.row(&[cell], y)
    };
    if !b.color_inactive {
        return linha(c, y);
    }
    // Per-Layer Color: cada camada é dona da sua cor. A linha vê-se e não tem alvo.
    let mut fora = HitIndex::new();
    std::mem::swap(c.hit_index, &mut fora);
    let fim = esmaecida(c, y, linha);
    std::mem::swap(c.hit_index, &mut fora);
    fim
}

/// A lista do Preset e, com preset activo, a estrela à direita: mesmo id e mesmo clique da caixa `Favorite`.
fn preset(c: &mut CardCtx<'_>, vista: Option<&VistaPresets>, y: f32) -> f32 {
    let (chave, id) = (
        "panel.painter_layers.wetpaint.preset",
        pids::PAINTER_WETPAINT_PRESET,
    );
    let ativo = vista.and_then(|v| v.procurar(v.ativo.as_deref()?));
    let Some(a) = ativo else {
        let nada = tr("panel.painter_layers.wetpaint.preset_none");
        return lista(c, chave, id, nada, y);
    };
    let celula = Cell::new(Widget::Dropdown {
        id,
        text: a.nome.as_str(),
        options: &[],
    })
    .label(tr(chave));
    let estrela = [Seg {
        id: pids::PAINTER_WETPAINT_PRESET_FAVORITE,
        icon: Some(if a.favorito {
            IconId::CardStarOn
        } else {
            IconId::CardStar
        }),
        text: "",
        on: a.favorito,
    }];
    c.row(&[celula, Cell::new(Widget::Bare(&estrela))], y)
}

fn pinta_linha(c: &mut CardCtx<'_>, linha: &Linha, b: &BrushSettings, y: f32) -> f32 {
    let vista = st::current_wet_presets();
    let vista: Option<&VistaPresets> = vista.as_deref();
    match linha {
        Linha::Modo(com_sync) => lista_e_sync(
            c,
            b,
            *com_sync,
            (
                "panel.painter_layers.brush.paint_mode",
                pids::PAINTER_BRUSH_MEDIA,
            ),
            tr(PaintMedia::from_u8(b.media).name_key()),
            &(0..PaintMedia::COUNT)
                .map(|i| tr(PaintMedia::from_u8(i).name_key()))
                .collect::<Vec<_>>(),
            y,
        ),
        Linha::Preset => preset(c, vista, y),
        Linha::Acoes => vista.map_or(y, |v| {
            let mut todos = biblioteca::botoes_de_guardar(v);
            todos.extend(biblioteca::botoes_da_biblioteca(v));
            botoes(c, &todos, y)
        }),
        Linha::Politica(com_sync) => lista_e_sync(
            c,
            b,
            *com_sync,
            (
                "panel.painter_layers.wetpaint.preset_policy",
                pids::PAINTER_WETPAINT_PRESET_POLICY,
            ),
            paint_wetpaint_presets::nome_da_politica(vista.map_or(0, |v| v.politica)),
            &(0..pids::PAINTER_WETPAINT_PRESET_POLICY_COUNT)
                .map(paint_wetpaint_presets::nome_da_politica)
                .collect::<Vec<_>>(),
            y,
        ),
        Linha::Sync => ferramenta::caixa(
            c,
            pids::PAINTER_BRUSH_SYNC,
            "card.painter.brush.sync",
            b.link_shared,
            y,
        ),
        Linha::Blend => lista(
            c,
            "panel.painter_layers.brush.blend",
            pids::PAINTER_BRUSH_BLEND,
            tr(BrushBlend::from_u8(b.blend).name_key()),
            y,
        ),
        Linha::Vazio => {
            let id = pids::PAINTER_BRUSH_BLEND_ON_EMPTY;
            com_lei(c, id, y, |c, y| {
                let rotulo = tr("panel.painter_layers.brush.blend_on_empty");
                let cell = Cell::new(numero_em_caixa(rotulo, id))
                    .label(rotulo)
                    .key(&[pids::PAINTER_BRUSH_BLEND_ON_EMPTY]);
                c.row(&[cell], y)
            })
        }
        Linha::Cor => cor(c, b, y),
        Linha::Numeros(par) => numeros(c, par, y),
        Linha::Accumulate => com_lei(c, pids::PAINTER_BRUSH_ACCUMULATE, y, |c, y| {
            ferramenta::caixa(
                c,
                pids::PAINTER_BRUSH_ACCUMULATE,
                "panel.painter_layers.brush.accumulate",
                b.accumulate,
                y,
            )
        }),
        Linha::Wash => com_lei(c, pids::PAINTER_BRUSH_WASH, y, |c, y| {
            ferramenta::caixa(
                c,
                pids::PAINTER_BRUSH_WASH,
                "panel.painter_layers.brush.wash",
                b.wash,
                y,
            )
        }),
        Linha::Fio => ferramenta::caixa(
            c,
            pids::PAINTER_BRUSH_SMEAR_FIO,
            "panel.painter_layers.brush.smear_fine_thread",
            b.smear_fio,
            y,
        ),
        Linha::Clone => cards_ferramenta::clone(c, b, y),
        Linha::Inpaint => cards_ferramenta::inpaint(c, y),
        Linha::Traco { sobra, composto } => cards_ferramenta::linha(c, b, *sobra, *composto, y),
        Linha::Composite => cards_composto::corpo(c, b, y),
        Linha::Previa => {
            if crate::preview_pino::preview_fixada(c.store) {
                return y;
            }
            let (x, w) = (c.x, c.w);
            let voa = topo_a_voar(c.store);
            let guardado = crate::preview_pino::rect_da_preview();
            let mut t = crate::cards_tela::Tela::do_cartao(c);
            let fim = crate::preview_pino::na_tela(&mut t, x, w, y + m::FIELD_ROW_GAP).unwrap_or(y);
            if voa {
                crate::preview_pino::lembra(guardado);
            }
            fim
        }
        Linha::Volume(i) => {
            let volumes = paint_wetpaint_presets::numeros_dos_volumes(b, b);
            volumes
                .get(*i)
                .map_or(y, |(n, rapido)| cards_wet::volume(c, n, *rapido, y))
        }
        Linha::Par(par) => em_pares(c, par.as_slice(), y, |c, l, y| pinta_linha(c, l, b, y)),
    }
}

/// O corpo destes cartões: as linhas, pela ordem.
pub(crate) fn corpo(c: &mut CardCtx<'_>, l: &[Linha], y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    l.iter().fold(y, |y, linha| pinta_linha(c, linha, &b, y))
}
