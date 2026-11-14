//! ⭐ **As amostras do Rive** (doc `CartoesUI/08` §1.4; dono 03/11: *«Se encontrar layouts e widgets
//! interessantes em Rive, traga algumas amostras»*). O Rive é ORÁCULO que se observa (CLAUDE.md §0.9): o
//! conceito confere-se na documentação pública, o desenho é nosso e fala a lei do cartão.

use crate::controlos::rive::{INPUT_ROWS, INPUT_TYPES, STOP_ROWS, STOP_SWATCH};
use crate::controlos::{
    CAMPO_MIN, caixa_para, card, dropdown, field, head, letra, marca, num, segs,
};
use crate::ids as g;
use crate::state::{Choices, Flag, Pick, WidgetGalleryState};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::paint_shapes::fill_rounded_rect_srgb8;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, StrokeToken};
use std::cell::Cell as Celula;
use std::time::Duration;

/// Quanto o botão de disparo fica aceso.
const FLASH: Duration = Duration::from_millis(300);
/// Os segmentos da curva e da barra do gradiente.
const AMOSTRAS: usize = 48;
/// A folga da curva no quadro: o Y dos cubic pode passar de 0 e de 1.
const FOLGA: f32 = 0.25; // LITERAL-PX-OK: fração do quadro
/// O coeficiente da Bézier cúbica.
const TRES: f32 = 3.0; // LITERAL-PX-OK: a definição da Bézier
const CEM: f32 = 100.0; // LITERAL-PX-OK: percentagem
/// A altura do quadro da curva, em linhas de campo.
const QUADRO_H: f32 = m::FIELD_H * 4.0; // LITERAL-PX-OK: quatro linhas de campo

/// O que o quadro da curva pintou por último: `(tipo, [x1, y1, x2, y2], quadro)`.
pub type Publicado = (u8, [f32; 4], Rect);

thread_local! {
    static PUBLICADO: Celula<Option<Publicado>> = const { Celula::new(None) };
}

/// O que a pré-visualização da curva pintou no último quadro.
#[must_use]
pub fn curva_publicada() -> Option<Publicado> {
    PUBLICADO.with(Celula::get)
}

fn numero(store: &WidgetStore, id: ph2d_a11y::NodeId) -> f32 {
    store.number_value(id).unwrap_or_default() as f32
}

/// O ponto `(x, y)` da curva (0..1) no quadro.
fn no_quadro(r: Rect, x: f32, y: f32) -> (f32, f32) {
    let alcance = 1.0 + FOLGA * 2.0;
    (r.x + x * r.w, r.y + r.h * (1.0 + FOLGA - y) / alcance)
}

/// A pré-visualização da curva: o quadro, a diagonal, a curva e (no Cubic) as duas pegas.
fn quadro(c: &mut CardCtx<'_>, ch: &Choices, y: f32) -> f32 {
    let y = y + m::FIELD_ROW_GAP;
    let r = Rect::new(c.x, y, c.w, QUADRO_H);
    fill_rounded_rect(
        c.scene,
        r,
        m::FIELD_RADIUS,
        c.ink.color(ColorToken::CardField),
    );
    let tipo = ch.pick(Pick::Interp);
    let [x1, y1, x2, y2] = [g::CU_X1, g::CU_Y1, g::CU_X2, g::CU_Y2].map(|id| numero(c.store, id));
    PUBLICADO.with(|p| p.set(Some((tipo, [x1, y1, x2, y2], r))));
    let (ini, fim) = (no_quadro(r, 0.0, 0.0), no_quadro(r, 1.0, 1.0));
    let fino = StrokeToken::Thin.px();
    stroke_polyline(
        c.scene,
        &[ini, fim],
        fino,
        c.ink.color(ColorToken::CardLabel),
    );
    let curva: Vec<(f32, f32)> = match tipo {
        0 => vec![ini, fim],
        2 => vec![ini, no_quadro(r, 1.0, 0.0), fim],
        _ => (0..=AMOSTRAS)
            .map(|i| {
                let t = i as f32 / AMOSTRAS as f32;
                let u = 1.0 - t;
                let (a, b) = (TRES * u * u * t, TRES * u * t * t);
                let e = t * t * t;
                no_quadro(r, a * x1 + b * x2 + e, a * y1 + b * y2 + e)
            })
            .collect(),
    };
    stroke_polyline(
        c.scene,
        &curva,
        StrokeToken::Thick.px(),
        c.ink.color(ColorToken::Accent),
    );
    if tipo == 1 {
        let raio = m::KEY_MARK_BOX * 0.5;
        for (id, o, x, y) in [(g::CU_H1, ini, x1, y1), (g::CU_H2, fim, x2, y2)] {
            let p = no_quadro(r, x, y);
            stroke_polyline(c.scene, &[o, p], fino, c.ink.color(ColorToken::Text2));
            fill_circle(
                c.scene,
                p.0,
                p.1,
                raio * 0.5,
                c.ink.color(ColorToken::Text1),
            );
            let pega = Rect::new(p.0 - raio, p.1 - raio, raio * 2.0, raio * 2.0);
            c.hit_index.register(id, pega);
        }
    }
    r.y + r.h
}

/// As pegas da curva escrevem nos quatro campos: o arrasto chega como deslocamento acumulado, e a
/// diferença para o último visto é o que se soma (o campo escrito à mão continua a mandar).
pub(crate) fn sync_pegas(state: &mut WidgetGalleryState, store: &mut WidgetStore) {
    let Some((_, _, r)) = curva_publicada() else {
        return;
    };
    let alcance = 1.0 + FOLGA * 2.0;
    for (i, (pega, cx, cy)) in [
        (g::CU_H1, g::CU_X1, g::CU_Y1),
        (g::CU_H2, g::CU_X2, g::CU_Y2),
    ]
    .into_iter()
    .enumerate()
    {
        let (ox, oy) = store.blender_picker_offset(pega);
        let (lx, ly) = state.pegas[i];
        if (ox, oy) != (lx, ly) {
            let x = numero(store, cx) + (ox - lx) / r.w;
            let y = numero(store, cy) - (oy - ly) * alcance / r.h;
            store.set_number_value(cx, f64::from(x.clamp(0.0, 1.0))); // CLAMP-OK: o X do cubic vive em 0..1
            store.set_number_value(cy, f64::from(y));
            state.pegas[i] = (ox, oy);
        }
    }
}

pub(crate) fn interpolation(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_INTERP,
        tr("panel.widget_gallery.rive.interpolation"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let tipos = segs([g::IN_LINEAR, g::IN_CUBIC, g::IN_HOLD], &ch, c.store);
            let y = c.row(&[Cell::new(Widget::Icons(&tipos))], y);
            let y = quadro(c, &ch, y);
            if ch.pick(Pick::Interp) != 1 {
                return y;
            }
            let y = c.row(
                &[
                    Cell::new(field(
                        g::CU_X1,
                        Glyph::Text(tr("panel.widget_gallery.rive.x1")),
                    ))
                    .label(tr("panel.widget_gallery.rive.handle_in")),
                    Cell::new(field(
                        g::CU_Y1,
                        Glyph::Text(tr("panel.widget_gallery.rive.y1")),
                    )),
                ],
                y,
            );
            c.row(
                &[
                    Cell::new(field(
                        g::CU_X2,
                        Glyph::Text(tr("panel.widget_gallery.rive.x2")),
                    ))
                    .label(tr("panel.widget_gallery.rive.handle_out")),
                    Cell::new(field(
                        g::CU_Y2,
                        Glyph::Text(tr("panel.widget_gallery.rive.y2")),
                    )),
                ],
                y,
            )
        }),
    )
}

/// O botão de disparo da entrada `i` está aceso?
#[must_use]
pub fn a_piscar(st: &WidgetGalleryState, i: usize) -> bool {
    st.fired_at[i].is_some_and(|t| t.elapsed() < FLASH)
}

/// Uma entrada: o widget do tipo dela (número · caixa · disparo) e o «−» no fim da linha.
fn input_row(c: &mut CardCtx<'_>, ch: &Choices, st: Disparos, i: usize, y: f32) -> f32 {
    let r = g::INPUTS[i];
    let menos = segs([r.remove], ch, c.store);
    let menos = Cell::new(Widget::Bare(&menos));
    match ch.pick(INPUT_TYPES[i]) {
        0 => {
            let k = [r.number];
            c.row(
                &[
                    Cell::new(field(
                        r.number,
                        letra(tr("panel.widget_gallery.rive.input.number")),
                    ))
                    .label(tr("panel.widget_gallery.rive.input.number"))
                    .key(&k),
                    menos,
                ],
                y,
            )
        }
        // A caixa e o «−» numa linha só: a caixa em moldura abre a faixa do nome para a linha inteira.
        1 => c.row(
            &[
                ph2d_editor_core::panel::cards::ferramenta::caixa_em_celula(
                    r.check,
                    ph2d_editor_core::widget::key_mark::Rotulo::from(
                        ph2d_editor_core::panel::TextKey::new(
                            "panel.widget_gallery.rive.input.boolean",
                        ),
                    ),
                    ch.on([Flag::InputBool0, Flag::InputBool1, Flag::InputBool2][i]),
                ),
                menos,
            ],
            y,
        ),
        _ => {
            let rotulo = tr("panel.widget_gallery.rive.fired").replace("{n}", &st.0[i].to_string());
            c.row(
                &[
                    Cell::new(Widget::Button {
                        id: r.fire,
                        text: tr("panel.widget_gallery.rive.fire"),
                        primary: st.1[i],
                    })
                    .label(&rotulo),
                    menos,
                ],
                y,
            )
        }
    }
}

/// `(quantas vezes disparou, aceso agora)` de cada entrada.
#[derive(Clone, Copy)]
struct Disparos([u32; 3], [bool; 3]);

pub(crate) fn inputs(st: &WidgetGalleryState, store: &WidgetStore) -> CardDef<'static> {
    let ch = st.choices;
    let disparos = Disparos(
        st.fires,
        [a_piscar(st, 0), a_piscar(st, 1), a_piscar(st, 2)],
    );
    let actions = if INPUT_ROWS.iter().all(|f| ch.on(*f)) {
        Vec::new()
    } else {
        vec![head(g::SM_ADD, &ch, store)]
    };
    card(
        g::CARD_INPUTS,
        tr("panel.widget_gallery.rive.inputs"),
        actions,
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let mut y = y;
            for (i, f) in INPUT_ROWS.iter().enumerate() {
                if ch.on(*f) {
                    y = input_row(c, &ch, disparos, i, y);
                }
            }
            y
        }),
    )
}

pub(crate) fn trim(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_TRIM,
        tr("panel.widget_gallery.rive.trim"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let y = c.row(
                &[
                    num!(g::TR_START, tr("panel.widget_gallery.rive.trim.start")),
                    num!(g::TR_END, tr("panel.widget_gallery.rive.trim.end")),
                ],
                y,
            );
            let (de, ate) = (numero(c.store, g::TR_START), numero(c.store, g::TR_END));
            let y = crate::controlos::barra(c, de / CEM, ate / CEM, y);
            let modos = segs([g::TR_SEQUENTIAL, g::TR_SYNCED], &ch, c.store);
            let desvio = num!(
                g::TR_OFFSET,
                IconId::CardAngle,
                tr("panel.widget_gallery.rive.trim.offset")
            );
            let modo = Cell::new(dropdown(g::TR_MODE, &modos))
                .label(tr("panel.widget_gallery.rive.trim.mode"));
            let larga = caixa_para(c, tr("panel.widget_gallery.rive.trim.sequential"));
            if c.w >= larga + CAMPO_MIN + m::FIELD_GAP {
                return c.row(&[desvio, modo], y);
            }
            let y = c.row(&[desvio], y);
            c.row(&[modo], y)
        }),
    )
}

/// Um grupo em que só um filho se pinta: o clique num faz dele o único, à maneira de um rádio com
/// olhos.
pub(crate) fn solo(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_SOLO,
        tr("panel.widget_gallery.rive.solo"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let nomes = [
                tr("panel.widget_gallery.rive.solo.idle"),
                tr("panel.widget_gallery.rive.solo.walk"),
                tr("panel.widget_gallery.rive.solo.jump"),
            ];
            let filhos: Vec<Cell<'_>> = nomes
                .into_iter()
                .enumerate()
                .map(|(i, nome)| {
                    let ativo = usize::from(ch.pick(Pick::Solo)) == i;
                    Cell::new(Widget::FieldFace {
                        id: g::SOLO[i],
                        glyph: Glyph::Icon(if ativo {
                            IconId::Eye
                        } else {
                            IconId::EyeClosed
                        }),
                        text: nome,
                    })
                })
                .collect();
            // Irmãos curtos dividem a linha; onde o maior nome não coube, cada um na sua.
            let maior = nomes.iter().fold(0.0_f32, |a, n| a.max(caixa_para(c, n)));
            let n = filhos.len() as f32;
            if c.w >= maior * n + m::FIELD_GAP * (n - 1.0) {
                return c.row(&filhos, y);
            }
            let mut y = y;
            for f in &filhos {
                y = c.row(std::slice::from_ref(f), y);
            }
            y
        }),
    )
}

/// A cor de uma paragem no ponto `t` dos dois vizinhos presentes.
fn mistura(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    let mut out = [0; 4];
    for i in 0..4 {
        out[i] = (f32::from(a[i]) + (f32::from(b[i]) - f32::from(a[i])) * t) as u8;
    }
    out
}

/// A paragem escolhida, se ainda existe; senão a primeira presente.
#[must_use]
pub fn paragem_escolhida(ch: &Choices) -> usize {
    let i = usize::from(ch.pick(Pick::GradStop));
    if ch.on(STOP_ROWS[i]) {
        i
    } else {
        STOP_ROWS.iter().position(|f| ch.on(*f)).unwrap_or(0)
    }
}

/// A barra do gradiente com a pega de cada paragem (a escolhida contornada no acento).
fn barra_do_gradiente(c: &mut CardCtx<'_>, ch: &Choices, y: f32) -> f32 {
    let y = y + m::FIELD_ROW_GAP;
    let r = Rect::new(c.x, y, c.w, m::FIELD_H);
    let mut presentes: Vec<(f32, [u8; 4], usize)> = (0..3)
        .filter(|i| ch.on(STOP_ROWS[*i]))
        .map(|i| {
            let p = numero(c.store, g::STOPS[i].position) / CEM;
            (p, crate::valores::rgba(ch, STOP_SWATCH + i, c.ink), i)
        })
        .collect();
    presentes.sort_by(|a, b| a.0.total_cmp(&b.0));
    let passo = r.w / AMOSTRAS as f32;
    for n in 0..AMOSTRAS {
        let t = (n as f32 + 0.5) / AMOSTRAS as f32;
        let dir = presentes
            .iter()
            .position(|p| p.0 >= t)
            .unwrap_or(presentes.len());
        let cor = if dir == 0 {
            presentes[0].1
        } else if dir == presentes.len() {
            presentes[dir - 1].1
        } else {
            let (a, b) = (presentes[dir - 1], presentes[dir]);
            mistura(a.1, b.1, (t - a.0) / (b.0 - a.0).max(f32::EPSILON))
        };
        let tira = Rect::new(r.x + passo * n as f32, r.y, passo.ceil(), r.h);
        fill_rounded_rect_srgb8(c.scene, tira, 0.0, cor);
    }
    let lado = m::KEY_MARK_BOX;
    let escolhida = paragem_escolhida(ch);
    for (p, cor, i) in presentes {
        let cx = (r.x + p * r.w).clamp(r.x, r.x + r.w); // CLAMP-OK: a pega fica dentro da barra
        let pega = Rect::new(cx - lado * 0.5, r.y + (r.h - lado) * 0.5, lado, lado);
        fill_rounded_rect_srgb8(c.scene, pega, m::SEG_RADIUS, cor);
        let token = if i == escolhida {
            ColorToken::Accent
        } else {
            ColorToken::Text1
        };
        // FRAME-RAW-OK: o contorno da PEGA diz qual paragem está escolhida (Accent) — a mensagem é o contorno
        stroke_rounded_rect(
            c.scene,
            pega,
            m::SEG_RADIUS,
            StrokeToken::Default.px(),
            c.ink.color(token),
        );
        c.hit_index.register(g::STOPS[i].handle, pega);
    }
    r.y + r.h
}

pub(crate) fn gradient(ch: Choices, store: &WidgetStore) -> CardDef<'static> {
    let gradiente = ch.pick(Pick::FillType) != 0;
    let actions = if gradiente && !STOP_ROWS.iter().all(|f| ch.on(*f)) {
        vec![head(g::GR_ADD, &ch, store)]
    } else {
        Vec::new()
    };
    card(
        g::CARD_GRADIENT,
        tr("panel.widget_gallery.rive.gradient"),
        actions,
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let tipos = segs([g::GR_SOLID, g::GR_LINEAR, g::GR_RADIAL], &ch, c.store);
            let y = c.row(&[Cell::new(Widget::Icons(&tipos))], y);
            let i = paragem_escolhida(&ch);
            let s = g::STOPS[i];
            let rgba = crate::valores::rgba(&ch, STOP_SWATCH + i, c.ink);
            let code = format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
            let (k_pos, k_cor) = ([s.position], [s.swatch]);
            let cor = Cell::new(Widget::Swatch {
                id: s.swatch,
                rgba: Some(rgba),
                text: &code,
                edit: None,
                detach: None,
            })
            .label(tr("panel.widget_gallery.color"))
            .key(&k_cor);
            if ch.pick(Pick::FillType) == 0 {
                return c.row(&[cor], y);
            }
            let y = barra_do_gradiente(c, &ch, y);
            let menos = segs([s.remove], &ch, c.store);
            let quantas = STOP_ROWS.iter().filter(|f| ch.on(**f)).count();
            let posicao = Cell::new(field(
                s.position,
                letra(tr("panel.widget_gallery.rive.stop_position")),
            ))
            .label(tr("panel.widget_gallery.rive.stop_position"))
            .key(&k_pos);
            // Um gradiente tem sempre duas paragens: com duas, o «−» não aparece.
            if quantas > 2 {
                c.row(&[posicao, cor, Cell::new(Widget::Bare(&menos))], y)
            } else {
                c.row(&[posicao, cor], y)
            }
        }),
    )
}
