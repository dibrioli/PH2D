//! Os cartões do topo e dos básicos, em linhas de cartão (fatia H2, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos
//! ids, mesmos eventos. O que ainda é de outra fatia entra capturado, como peça: a prévia e Clone, Inpaint, Line e
//! Composite (H8). As condições são as de `paint_brush::plano_do_corpo` e
//! `paint_top_basics`, e `every_old_id_is_still_painted_in_each_tool_and_medium` confere-as em 13 estados.

use crate::cards_captura::{Captura, x_de, y_de};
use crate::cards_linhas::{aberta, botoes, com_lei, em_pares, esmaecida, registra_lista};
use crate::paint_wetpaint_presets_biblioteca as biblioteca;
use crate::state_wet_presets::{self as st, VistaPresets};
use crate::{cards_wet, number_field, paint_brush};
use crate::{
    paint_clone, paint_composite, paint_inpaint, paint_line, paint_wetpaint_presets, state,
};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget, corpo as linhas, ferramenta};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::card_ink::CardInk;
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
    Modo,
    Preset,
    Favorito,
    Acoes,
    Politica,
    Sync,
    Blend,
    Vazio,
    Cor,
    Numeros(Vec<Numero>),
    Accumulate,
    Wash,
    Fio,
    Peca(Captura),
    /// Um volume vivo do Wet Paint, pelo índice na tabela.
    Volume(usize),
    /// Duas linhas lado a lado.
    Par(Box<[Linha; 2]>),
}

/// O que as peças capturadas precisam de saber do cartão.
struct Dono {
    secao: NodeId,
    ink: CardInk,
    x: f32,
    w: f32,
    topo: f32,
}

impl Dono {
    fn novo(ctx: &PaintCtx, secao: NodeId, ink: CardInk, x: f32, w: f32) -> Self {
        Self {
            secao,
            ink,
            x,
            w,
            topo: ctx.slot.y,
        }
    }

    /// Corre `pintor` à parte. A chave é só a da memória da posição: nunca se regista.
    fn peca(
        &self,
        ctx: &mut PaintCtx,
        n: u64,
        fora: &mut Vec<Linha>,
        pintor: impl FnOnce(&mut PaintCtx, ph2d_tokens::Theme, f32, f32, f32) -> f32,
    ) {
        let chave = NodeId(self.secao.0.wrapping_add(n));
        let (x, w, tema) = (x_de(chave, self.x), self.w, self.ink.theme);
        let c = Captura::da_peca(
            ctx,
            chave,
            self.secao,
            self.ink,
            (x, y_de(chave, self.topo)),
            |ctx, y| pintor(ctx, tema, x, w, y),
        );
        if c.pintou() {
            fora.push(Linha::Peca(c));
        }
    }
}

/// As linhas do cartão sem título do topo.
pub(crate) fn do_topo(
    ctx: &mut PaintCtx,
    ink: CardInk,
    (x, w): (f32, f32),
    brush: BrushSettings,
) -> Vec<Linha> {
    let dono = Dono::novo(ctx, pids::PAINTER_CARD_TOPO, ink, x, w);
    let vista = st::current_wet_presets();
    let mut l = Vec::new();
    if !brush.is_inpaint && brush.meios.count_ones() > 1 {
        l.push(Linha::Modo);
    }
    if let Some(v) = vista.as_deref() {
        l.push(Linha::Preset);
        if v.ativo.as_deref().and_then(|r| v.procurar(r)).is_some() {
            l.push(Linha::Favorito);
        }
    }
    dono.peca(ctx, 1, &mut l, |ctx, tema, x, w, y| {
        paint_wetpaint_presets::paint_preview(ctx, tema, x, w, y)
    });
    if vista.is_some() {
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
        l.push(Linha::Politica);
    }
    l.push(Linha::Sync);
    l
}

/// As linhas do cartão Basics. Vazio = o cartão não está à vista.
pub(crate) fn dos_basicos(
    ctx: &mut PaintCtx,
    ink: CardInk,
    (x, w): (f32, f32),
    brush: BrushSettings,
) -> Vec<Linha> {
    let dono = Dono::novo(ctx, pids::PAINTER_CARD_BASICOS, ink, x, w);
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
    l.extend(numeros.chunks(2).map(|par| Linha::Numeros(par.to_vec())));
    if brush.is_clone {
        dono.peca(ctx, 1, &mut l, |ctx, tema, x, w, y| {
            paint_clone::paint_clone_card(ctx, tema, x, w, y, brush)
        });
    }
    if brush.is_inpaint {
        dono.peca(ctx, 2, &mut l, |ctx, tema, x, w, y| {
            paint_inpaint::paint_inpaint_card(ctx, tema, x, w, y, brush)
        });
    }
    dono.peca(ctx, 3, &mut l, |ctx, tema, x, w, y| {
        paint_line::paint_line_card(ctx, tema, x, w, y, brush)
    });
    if brush.le(LinhaDaFerramenta::Composite) && brush.composite_offered() {
        dono.peca(ctx, 4, &mut l, |ctx, tema, x, w, y| {
            paint_composite::paint_composite_card(ctx, tema, x, w, y, brush)
        });
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

fn pinta_linha(c: &mut CardCtx<'_>, linha: &Linha, b: &BrushSettings, y: f32) -> f32 {
    let vista = st::current_wet_presets();
    let vista: Option<&VistaPresets> = vista.as_deref();
    match linha {
        Linha::Modo => lista(
            c,
            "panel.painter_layers.brush.paint_mode",
            pids::PAINTER_BRUSH_MEDIA,
            tr(PaintMedia::from_u8(b.media).name_key()),
            y,
        ),
        Linha::Preset => {
            let nome = vista
                .and_then(|v| v.procurar(v.ativo.as_deref()?))
                .map_or_else(
                    || tr("panel.painter_layers.wetpaint.preset_none"),
                    |a| a.nome.as_str(),
                );
            lista(
                c,
                "panel.painter_layers.wetpaint.preset",
                pids::PAINTER_WETPAINT_PRESET,
                nome,
                y,
            )
        }
        Linha::Favorito => {
            let a = vista.and_then(|v| v.procurar(v.ativo.as_deref()?));
            ferramenta::caixa(
                c,
                pids::PAINTER_WETPAINT_PRESET_FAVORITE,
                "panel.painter_layers.wetpaint.preset_favorite",
                a.is_some_and(|a| a.favorito),
                y,
            )
        }
        Linha::Acoes => vista.map_or(y, |v| {
            let mut todos = biblioteca::botoes_de_guardar(v);
            todos.extend(biblioteca::botoes_da_biblioteca(v));
            botoes(c, &todos, y)
        }),
        Linha::Politica => lista(
            c,
            "panel.painter_layers.wetpaint.preset_policy",
            pids::PAINTER_WETPAINT_PRESET_POLICY,
            paint_wetpaint_presets::nome_da_politica(vista.map_or(0, |v| v.politica)),
            y,
        ),
        Linha::Sync => ferramenta::caixa(
            c,
            pids::PAINTER_BRUSH_SYNC,
            "panel.painter_layers.brush.sync_tools",
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
        Linha::Peca(p) => p.repete(c, y + m::FIELD_ROW_GAP),
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
