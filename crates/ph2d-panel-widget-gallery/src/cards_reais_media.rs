//! ⭐ **Casos reais: luz, áudio, pixel art, script** (doc `CartoesUI/08` §1.3).

use crate::controlos::{
    CAMPO_MIN, PALETTE_TOKENS, caixa_para, card, dropdown, field, head, letra, marca, num, seg_of,
    segs,
};
use crate::ids as g;
use crate::state::{Choices, Flag, Pick, RACKS};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::{LevelMeter, paint_level_meter};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, StrokeToken};

const RACK_PICKS: [Pick; 3] = [Pick::Rack0, Pick::Rack1, Pick::Rack2];
/// O nível de partida do medidor (fração da coluna, sem ganho).
const NIVEL_BASE: f64 = 0.35; // LITERAL-PX-OK: dado do objeto fictício
/// dB ⇒ fator: `10^(dB/20)`.
const DB_POR_DECADA: f64 = 20.0; // LITERAL-PX-OK: a definição do decibel
const DEZ: f64 = 10.0; // LITERAL-PX-OK: a definição do decibel
const PALETA_COLUNAS: usize = 8;

pub(crate) fn light(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_LIGHT,
        tr("component.luz.name"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let tipos = segs([g::LT_POINT, g::LT_AMBIENT], &ch, c.store);
            let y = c.row(
                &[Cell::new(Widget::Choice(&tipos)).label(tr("component.field.luz_fields.0"))],
                y,
            );
            let rgba = crate::valores::rgba(&ch, 9, c.ink);
            let code = format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
            let cor = Cell::new(Widget::Swatch {
                id: g::LT_SWATCH,
                rgba: Some(rgba),
                text: &code,
                edit: None,
                detach: None,
            })
            .label(tr("component.field.luz_fields.1"))
            .key(&[g::LT_SWATCH]);
            let forca = num!(g::LT_INTENSITY, tr("component.field.luz_fields.2"));
            let y = if c.w >= caixa_para(c, &code) + CAMPO_MIN + m::FIELD_GAP {
                c.row(&[cor, forca], y)
            } else {
                let y = c.row(&[cor], y);
                c.row(&[forca], y)
            };
            if ch.pick(Pick::LampKind) != 0 {
                return y;
            }
            let y = c.row(
                &[num!(g::LT_SIZE, tr("panel.widget_gallery.real.lamp_size"))],
                y,
            );
            // De fábrica a lâmpada não tem limite (`alcance_m` infinito): como no Inspector, o
            // alcance só aparece com a caixa marcada.
            let limitada = ch.on(Flag::LampLimit);
            let mut y = marca(
                c,
                g::LT_LIMIT,
                "panel.inspector.luz.limit_range",
                limitada,
                y,
            );
            if limitada {
                y = c.row(
                    &[num!(
                        g::LT_RANGE,
                        tr("panel.widget_gallery.real.lamp_range")
                    )],
                    y,
                );
            }
            marca(
                c,
                g::LT_SHADOWS,
                "panel.widget_gallery.real.casts_shadows",
                ch.on(Flag::LampShadows),
                y,
            )
        }),
    )
}

/// As propriedades da linha `i` do rack (o painel do lado, [`crate::lado`]).
pub(crate) fn rack_props(c: &mut CardCtx<'_>, i: usize, y: f32) -> f32 {
    let r = g::RACK[i];
    let (k_mix, k_damp) = ([r.mix], [r.damp]);
    c.row(
        &[
            Cell::new(field(r.mix, Glyph::Icon(IconId::CardMix)))
                .label(tr("panel.widget_gallery.real.mix"))
                .key(&k_mix),
            Cell::new(field(r.damp, letra(tr("audio.fx.param.damp"))))
                .label(tr("audio.fx.param.damp"))
                .key(&k_damp),
        ],
        y,
    )
}

/// `[medidor] Gain (dB) 🔊` + a lista do rack à Effects (F43): `[tipo] 👁 −`, o clique abre ao lado.
pub(crate) fn audio(ch: Choices, store: &WidgetStore) -> CardDef<'static> {
    let actions = if RACKS.iter().all(|f| ch.on(*f)) {
        Vec::new()
    } else {
        vec![head(g::AU_ADD, &ch, store)]
    };
    card(
        g::CARD_AUDIO,
        tr("panel.widget_gallery.card.audio"),
        actions,
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let (x0, w0) = (c.x, c.w);
            let largura = m::FIELD_H;
            c.x += largura + m::FIELD_GAP;
            c.w -= largura + m::FIELD_GAP;
            let mudo = segs([g::AU_MUTE], &ch, c.store);
            let fundo = c.row(
                &[
                    num!(g::AU_GAIN, tr("panel.widget_gallery.real.gain")),
                    Cell::new(Widget::Bare(&mudo)),
                ],
                y,
            );
            c.x = x0;
            c.w = w0;
            let ganho = c.store.number_value(g::AU_GAIN).unwrap_or_default();
            let nivel = if ch.on(Flag::AudioMute) {
                0.0
            } else {
                (NIVEL_BASE * DEZ.powf(ganho / DB_POR_DECADA)) as f32
            };
            let medidor = LevelMeter::new(g::AU_GAIN, "").levels(nivel, nivel);
            let topo = y + m::LABEL_TOP;
            let r = Rect::new(x0, topo, largura, fundo - topo);
            paint_level_meter(&medidor, r, c.scene, c.ink.theme);
            let mut y = fundo;
            for (i, f) in RACKS.iter().enumerate() {
                if !ch.on(*f) {
                    continue;
                }
                let row = g::RACK[i];
                let kind = usize::from(ch.pick(RACK_PICKS[i]));
                let nome = seg_of(row.options[kind], &ch, c.store).text;
                let olho = segs([row.eye, row.remove], &ch, c.store);
                y = c.row(
                    &[
                        Cell::new(Widget::FieldFace {
                            id: row.open,
                            glyph: Glyph::Icon(IconId::FxWave),
                            text: nome,
                        }),
                        Cell::new(Widget::Bare(&olho)),
                    ],
                    y,
                );
            }
            y
        }),
    )
}

pub(crate) fn pixel(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_PIXEL,
        tr("panel.widget_gallery.card.pixel"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let cor_de = |c: &CardCtx<'_>, p: Pick| {
                c.ink
                    .color(PALETTE_TOKENS[usize::from(ch.pick(p))])
                    .to_rgba8()
                    .to_u8_array()
            };
            let (fg, bg) = (cor_de(c, Pick::PixelFg), cor_de(c, Pick::PixelBg));
            let hex = |v: [u8; 4]| format!("{:02X}{:02X}{:02X}", v[0], v[1], v[2]);
            let (t_fg, t_bg) = (hex(fg), hex(bg));
            let troca = segs([g::PX_SWAP], &ch, c.store);
            let frente = Cell::new(Widget::Swatch {
                id: g::PX_FG,
                rgba: Some(fg),
                text: &t_fg,
                edit: None,
                detach: None,
            })
            .label(tr("panel.widget_gallery.real.foreground"));
            let fundo = Cell::new(Widget::Swatch {
                id: g::PX_BG,
                rgba: Some(bg),
                text: &t_bg,
                edit: None,
                detach: None,
            })
            .label(tr("panel.widget_gallery.real.background"));
            let bare = Cell::new(Widget::Bare(&troca));
            let y = if c.w
                >= caixa_para(c, &t_fg).max(caixa_para(c, &t_bg)) * 2.0
                    + m::FIELD_H
                    + m::FIELD_GAP * 2.0
            {
                c.row(&[frente, fundo, bare], y)
            } else {
                let y = c.row(&[frente, bare], y);
                c.row(&[fundo], y)
            };
            // A caixa do alvo leva o contorno: é para ela que a próxima cor da paleta vai.
            let alvo = if ch.pick(Pick::PixelTarget) == 0 {
                g::PX_FG
            } else {
                g::PX_BG
            };
            if let Some(r) = c.hit_index.rect_for(alvo) {
                stroke_rounded_rect(
                    c.scene,
                    r,
                    m::FIELD_RADIUS,
                    StrokeToken::Default.px(),
                    c.ink.color(ColorToken::Accent),
                );
            }
            let y = paleta(c, &ch, y + m::FIELD_ROW_GAP);
            let y = c.row(
                &[
                    Cell::new(field(
                        g::PX_GRID_W,
                        Glyph::Text(tr("panel.widget_gallery.glyph.w")),
                    ))
                    .label(tr("panel.widget_gallery.real.grid_size")),
                    Cell::new(field(
                        g::PX_GRID_H,
                        Glyph::Text(tr("panel.widget_gallery.glyph.h")),
                    )),
                ],
                y,
            );
            marca(
                c,
                g::PX_GRID_ON,
                "panel.widget_gallery.real.pixel_grid",
                ch.on(Flag::PixelGrid),
                y,
            )
        }),
    )
}

/// A grelha da paleta indexada: 8 × 2 quadrados; as escolhidas levam contorno.
fn paleta(c: &mut CardCtx<'_>, ch: &Choices, y: f32) -> f32 {
    let n = PALETA_COLUNAS as f32;
    let gap = m::SEG_PAD;
    let lado = (c.w - gap * (n - 1.0)) / n;
    let mut fundo = y;
    for (i, id) in g::PALETTE.iter().enumerate() {
        let (col, lin) = ((i % PALETA_COLUNAS) as f32, (i / PALETA_COLUNAS) as f32);
        let r = Rect::new(c.x + col * (lado + gap), y + lin * (lado + gap), lado, lado);
        fill_rounded_rect(c.scene, r, m::SEG_RADIUS, c.ink.color(PALETTE_TOKENS[i]));
        if usize::from(ch.pick(Pick::PixelFg)) == i || usize::from(ch.pick(Pick::PixelBg)) == i {
            stroke_rounded_rect(
                c.scene,
                r,
                m::SEG_RADIUS,
                StrokeToken::Default.px(),
                c.ink.color(ColorToken::Text1),
            );
        }
        c.hit_index.register(*id, r);
        fundo = r.y + r.h;
    }
    fundo
}

/// O script do objeto e as propriedades que ele expõe (o que `self.<nome>` declara no editor de
/// scripts): cada script tem as suas linhas.
pub(crate) fn script(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_SCRIPT,
        tr("panel.inspector.script.script"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let scripts = segs([g::SC_FILE_0, g::SC_FILE_1, g::SC_FILE_2], &ch, c.store);
            let y = c.row(
                &[Cell::new(dropdown(g::SC_FILE, &scripts))
                    .label(tr("panel.inspector.script.script"))],
                y,
            );
            let y = c.row(
                &[Cell::new(Widget::Button {
                    id: g::SC_EDIT,
                    text: tr("panel.inspector.script.edit"),
                    primary: ch.on(Flag::ScriptEdit),
                })],
                y,
            );
            match ch.pick(Pick::ScriptFile) {
                0 => {
                    let y = c.row(
                        &[
                            num!(g::SC_SPEED, tr("panel.widget_gallery.real.prop.speed")),
                            num!(g::SC_JUMP, tr("panel.widget_gallery.real.prop.jump_power")),
                        ],
                        y,
                    );
                    marca(
                        c,
                        g::SC_DOUBLE,
                        "panel.widget_gallery.real.prop.double_jump",
                        ch.on(Flag::ScriptDouble),
                        y,
                    )
                }
                1 => c.row(
                    &[
                        num!(g::SC_PATROL, tr("panel.widget_gallery.real.prop.patrol")),
                        num!(g::SC_DAMAGE, tr("panel.widget_gallery.real.prop.damage")),
                    ],
                    y,
                ),
                _ => c.row(
                    &[num!(
                        g::SC_VALUE,
                        tr("panel.widget_gallery.real.prop.value")
                    )],
                    y,
                ),
            }
        }),
    )
}
