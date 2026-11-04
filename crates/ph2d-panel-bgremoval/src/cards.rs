//! ⭐ **O painel Background Removal em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): os
//! mesmos ids; `populate` e `event` intocados. Todo liga/desliga é uma caixa, e o Detect subject vem
//! antes do Pick colors porque é ele que decide o que essa linha é (o Add area toma-lhe o lugar).

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::ferramenta::{
    Numero, Verbos, aos_pares, caixa, campo, cartao, opcoes, prepara_numeros,
};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Widget};
use ph2d_editor_core::widget::{ColorSwatch, SwatchSize, paint_color_swatch};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;
use ph2d_tool_bgremoval::ids as t;
use ph2d_tool_bgremoval::params::{
    BgRemovalUiSnapshot, BrushFalloff, MIN_ISLAND_PIXELS_FULL_SCALE,
};

/// Os números de `0..1`: um centésimo por seta, meio centésimo por px de arrasto.
const PASSO_UNIT: f64 = 0.01; // LITERAL-PX-OK: passo de seta na faixa 0..1 do valor da ferramenta
const ARRASTO_UNIT: f64 = 0.005; // LITERAL-PX-OK: valor por px de arrasto na faixa 0..1 da ferramenta
const PASSO_PX: f64 = 1.0;
/// `Grow` mostra `-1..+1` sobre um trilho `0..1` com o neutro no meio.
const GROW_CENTRO: f32 = 0.5;
const GROW_ESCALA: f32 = 2.0;

fn grow(snap: &BgRemovalUiSnapshot) -> f64 {
    f64::from((snap.grow01 - GROW_CENTRO) * GROW_ESCALA)
}

fn ilha_minima(snap: &BgRemovalUiSnapshot) -> f64 {
    let n = snap.min_island_pixels01.clamp(0.0, 1.0) * (MIN_ISLAND_PIXELS_FULL_SCALE - 1.0);
    f64::from(n.round() + 1.0)
}

pub(crate) fn prepare(store: &mut WidgetStore, snap: &BgRemovalUiSnapshot) {
    let f = BgRemovalUiSnapshot::default();
    let unit = |chip, fab: f32, valor: f32| Numero {
        chip,
        step: PASSO_UNIT,
        scrub: ARRASTO_UNIT,
        fabrica: f64::from(fab),
        valor: f64::from(valor),
    };
    prepara_numeros(
        store,
        &[
            unit(t::BGR_TOLERANCE_NUM, f.tolerance01, snap.tolerance01),
            unit(t::BGR_FEATHER_NUM, f.feather01, snap.feather01),
            unit(t::BGR_REFINE_NUM, f.refine01, snap.refine01),
            unit(t::BGR_BRUSH_SIZE_NUM, f.brush_size01, snap.brush_size01),
            Numero {
                chip: t::BGR_GROW_NUM,
                step: PASSO_UNIT,
                scrub: ARRASTO_UNIT,
                fabrica: grow(&f),
                valor: grow(snap),
            },
            Numero {
                chip: t::BGR_MIN_ISLAND_PX_NUM,
                step: PASSO_PX,
                scrub: PASSO_PX,
                fabrica: ilha_minima(&f),
                valor: ilha_minima(snap),
            },
        ],
    );
}

fn cartao_da_mascara() -> CardDef<'static> {
    cartao(t::BGR_SEC_MASK, tr("card.bgr.mask"), None, move |c, y| {
        let cells = [
            campo(t::BGR_TOLERANCE_NUM, tr("panel.bg_removal.mask.tolerance")),
            campo(t::BGR_FEATHER_NUM, tr("panel.bg_removal.mask.feather")),
            campo(t::BGR_REFINE_NUM, tr("panel.bg_removal.mask.refine")),
            campo(t::BGR_GROW_NUM, tr("panel.bg_removal.mask.grow")),
        ];
        aos_pares(c, &cells, y)
    })
}

/// As amostras das cores extra: o mesmo id por posição, e o botão direito remove-as como antes.
fn amostras(c: &mut CardCtx<'_>, cores: &[[u8; 3]], y: f32) -> f32 {
    if cores.is_empty() {
        return y;
    }
    c.legacy(y, |r, y| {
        let lado = SwatchSize::Sm.px();
        let vao = Spacing::Xs.px();
        let por_linha = (((r.inner_w + vao) / (lado + vao)).floor() as usize).max(1);
        let mut fundo = y;
        for (i, cor) in cores.iter().enumerate() {
            let sx = r.inner_x + (i % por_linha) as f32 * (lado + vao);
            let sy = y + (i / por_linha) as f32 * (lado + vao);
            let caixa = Rect::new(sx, sy, lado, lado);
            let id = ph2d_editor_core::ids::BGR_SWATCHES[i];
            let sw = ColorSwatch::new(
                id,
                tr("panel.bg_removal.mask.extra_bg_colour"),
                [cor[0], cor[1], cor[2], u8::MAX],
            )
            .size(SwatchSize::Sm);
            paint_color_swatch(&sw, caixa, r.scene, r.theme);
            r.hit_index.register(id, caixa);
            fundo = sy + lado;
        }
        fundo
    })
}

fn verbo(c: &mut CardCtx<'_>, id: ph2d_a11y::NodeId, chave: &'static str, y: f32) -> f32 {
    c.full(
        None,
        Widget::Button {
            id,
            text: tr(chave),
            primary: false,
        },
        y,
    )
}

fn cartao_das_cores(snap: BgRemovalUiSnapshot) -> CardDef<'static> {
    cartao(
        t::BGR_SEC_COLORS,
        tr("card.bgr.colors"),
        None,
        move |c, y| {
            let y = caixa(
                c,
                t::BGR_AUTO_PROTECT_SUBJECT,
                "panel.bg_removal.mask.detect_subject",
                snap.auto_protect_subject,
                y,
            );
            if snap.auto_protect_subject {
                let y = caixa(
                    c,
                    t::BGR_ADD_AREA,
                    "panel.bg_removal.mask.add_area",
                    snap.add_area_armed,
                    y,
                );
                if snap.has_force_remove_mask {
                    return verbo(
                        c,
                        t::BGR_ADD_AREA_CLEAR,
                        "panel.bg_removal.mask.clear_added_areas",
                        y,
                    );
                }
                return y;
            }
            let y = caixa(
                c,
                t::BGR_EYEDROPPER,
                "panel.bg_removal.mask.pick_colors",
                snap.eyedropper_armed,
                y,
            );
            amostras(c, &snap.extra_colors, y)
        },
    )
}

fn cartao_das_ilhas(snap: &BgRemovalUiSnapshot) -> CardDef<'static> {
    let separa = snap.separate_islands;
    cartao(
        t::BGR_SEC_ISLANDS,
        tr("card.bgr.islands"),
        None,
        move |c, y| {
            let y = caixa(
                c,
                t::BGR_SEPARATE_ISLANDS,
                "panel.bg_removal.mask.separate_islands",
                separa,
                y,
            );
            if separa {
                let cell = campo(t::BGR_MIN_ISLAND_PX_NUM, tr("panel.bg_removal.mask.min_px"));
                return c.row(&[cell], y);
            }
            y
        },
    )
}

fn pincel(c: &mut CardCtx<'_>, snap: &BgRemovalUiSnapshot, y: f32) -> f32 {
    let y = c.row(
        &[campo(
            t::BGR_BRUSH_SIZE_NUM,
            tr("panel.bg_removal.mask.size"),
        )],
        y,
    );
    let aceso = match snap.falloff {
        BrushFalloff::Smooth => 0,
        BrushFalloff::Sphere => 1,
        BrushFalloff::Sharp => 2,
        BrushFalloff::Constant => 3,
    };
    let segs = opcoes(
        &[
            (t::BGR_FALLOFF_SMOOTH, tr(BrushFalloff::Smooth.label_key())),
            (t::BGR_FALLOFF_SPHERE, tr(BrushFalloff::Sphere.label_key())),
            (t::BGR_FALLOFF_SHARP, tr(BrushFalloff::Sharp.label_key())),
            (
                t::BGR_FALLOFF_CONSTANT,
                tr(BrushFalloff::Constant.label_key()),
            ),
        ],
        aceso,
    );
    c.full(Some(tr("card.bgr.falloff")), Widget::Choice(&segs), y)
}

fn cartao_da_protecao(snap: BgRemovalUiSnapshot) -> CardDef<'static> {
    cartao(
        t::BGR_SEC_PROTECT,
        tr("card.bgr.protection"),
        None,
        move |c, y| {
            let mut y = caixa(
                c,
                t::BGR_PROTECT,
                "card.bgr.protect_brush",
                snap.protect_brush_armed,
                y,
            );
            if snap.protect_brush_armed {
                y = pincel(c, &snap, y);
            }
            if snap.mask_overlay_toggle_is_reachable() {
                y = caixa(
                    c,
                    t::BGR_SHOW_MASK,
                    "panel.bg_removal.mask.show_mask",
                    snap.show_mask,
                    y,
                );
            }
            if snap.has_protect_mask {
                y = verbo(
                    c,
                    t::BGR_PROTECT_CLEAR,
                    "panel.bg_removal.mask.clear_protection",
                    y,
                );
            }
            y
        },
    )
}

pub(crate) fn cards(snap: &BgRemovalUiSnapshot) -> Vec<CardDef<'static>> {
    vec![
        cartao_da_mascara(),
        cartao_das_cores(snap.clone()),
        cartao_das_ilhas(snap),
        cartao_da_protecao(snap.clone()),
    ]
}

pub(crate) fn verbos() -> Verbos<'static> {
    Verbos {
        reset: (t::BGR_RESET, tr("panel.bg_removal.mask.reset_to_defaults")),
        cancel: (t::BGR_CANCEL, tr("panel.bg_removal.mask.cancel")),
        apply: (t::BGR_APPLY, tr("panel.bg_removal.mask.apply")),
    }
}
