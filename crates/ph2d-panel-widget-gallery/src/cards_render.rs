//! ⭐ **O cartão Render no idioma de exportar do Figma e do Rive** (dono 03/11, doc `CartoesUI/08` §1.2:
//! *«Card Render poderia melhorar muito seu layout e widgets»*): a linha de estado com o `%` à direita, a
//! barra fina, os medidores L/R deitados com a escala em dB, etiquetas que dizem o que são e uma corrida
//! fictícia que anda um passo por quadro pintado — determinista nos testes. O corpo antigo só vale em
//! `PH2D_CARDS=0`.

use crate::controlos::{card, dropdown, segs};
use crate::ids as g;
use crate::state::{Choices, Pick, WidgetGalleryState};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::paint_text_elided;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::card_field;
use ph2d_editor_core::widget::{
    ProgressBar, Spinner, Tag, TagTone, fraction_of_db, paint_level_channel_on_card,
    paint_progress_bar_on_card, paint_spinner, paint_tag_with_radius,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;
use ph2d_vector::{Affine, VectorScene};
use std::cell::RefCell;

/// Os quadros de cada qualidade (Draft · Good · Best).
pub const QUADROS: [u32; 3] = [60, 120, 240]; // LITERAL-PX-OK: dado da corrida fictícia
/// Os quadros pintados por quadro da corrida: uma corrida Good dura ~4 s a 60 Hz.
pub const PASSOS_POR_QUADRO: u32 = 2;
/// Quanto o pico retido desce por passo, na fração da escala.
const QUEDA_DO_PICO: f32 = 0.004; // LITERAL-PX-OK: fração da escala do medidor
/// Os níveis em repouso (a pré-visualização a tocar) e os picos retidos dela.
const REPOUSO: [f32; 2] = [0.42, 0.58]; // LITERAL-PX-OK: dado da corrida fictícia
const PICO_REPOUSO: [f32; 2] = [0.55, 0.72]; // LITERAL-PX-OK: dado da corrida fictícia
/// A forma da música fictícia de cada canal: `(base, amplitude, velocidade)`.
const ONDAS: [(f32, f32, f32); 2] = [
    (0.45, 0.25, 0.21), // LITERAL-PX-OK: dado da corrida fictícia
    (0.55, 0.3, 0.13),  // LITERAL-PX-OK: dado da corrida fictícia
];
/// As marcas da escala, em dBFS.
const ESCALA_DB: [i32; 4] = [-20, -12, -6, 0];
const ALTURA_DO_MEDIDOR: f32 = 8.0; // LITERAL-PX-OK: a espessura do canal deitado
const ALTURA_DA_ETIQUETA: f32 = m::ICON_CELL - m::FIELD_GAP;
/// O giro do spinner por passo, em radianos.
const GIRO: f64 = 0.2; // LITERAL-PX-OK: radianos, animação
const CEM: f32 = 100.0; // LITERAL-PX-OK: percentagem

/// Em que pé a corrida está.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fase {
    #[default]
    Pronta,
    Correndo,
    Feita,
}

/// A corrida fictícia: anda um passo por quadro pintado, sem relógio.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Corrida {
    pub fase: Fase,
    pub passos: u32,
    /// O pico retido de cada canal, enquanto corre.
    pub picos: [f32; 2],
    /// O smoke fotografa um quadro parado.
    pub presa: bool,
}

impl Corrida {
    /// O quadro em que vai, em `0..=total`.
    #[must_use]
    pub fn quadro(&self, total: u32) -> u32 {
        if self.fase == Fase::Feita {
            return total;
        }
        (self.passos / PASSOS_POR_QUADRO).min(total)
    }

    pub fn start(&mut self) {
        *self = Self {
            fase: Fase::Correndo,
            ..Self::default()
        };
    }

    pub fn cancel(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn prende_em(&mut self, quadro: u32) {
        self.start();
        self.passos = quadro * PASSOS_POR_QUADRO;
        self.picos = niveis(self);
        self.presa = true;
    }
}

/// O nível de cada canal: um desenho determinista do passo, ou o repouso.
#[must_use]
pub fn niveis(c: &Corrida) -> [f32; 2] {
    if c.fase != Fase::Correndo {
        return REPOUSO;
    }
    let t = c.passos as f32;
    ONDAS.map(|(base, amp, vel)| base + amp * (t * vel).sin())
}

/// Um passo da corrida, por quadro pintado, antes de pintar.
pub(crate) fn avanca(st: &mut WidgetGalleryState) {
    let total = QUADROS[usize::from(st.choices.pick(Pick::RenderQuality))];
    let c = &mut st.render;
    if c.fase != Fase::Correndo || c.presa {
        return;
    }
    c.passos += 1;
    let n = niveis(c);
    for (p, v) in c.picos.iter_mut().zip(n) {
        *p = (*p - QUEDA_DO_PICO).max(v);
    }
    if c.passos >= total * PASSOS_POR_QUADRO {
        c.fase = Fase::Feita;
    }
}

/// O que o cartão pintou por último (os testes leem daqui).
#[derive(Clone, Debug, PartialEq)]
pub struct Pintado {
    pub status: String,
    pub percent: u32,
    pub spinner: bool,
    pub tags: Vec<(String, TagTone)>,
    pub niveis: [f32; 2],
}

thread_local! {
    static PINTADO: RefCell<Option<Pintado>> = const { RefCell::new(None) };
}

/// O que o cartão Render pintou no último quadro.
#[must_use]
pub fn pintado() -> Option<Pintado> {
    PINTADO.with(|p| p.borrow().clone())
}

/// `(texto do estado, % , etiquetas)` da corrida em `ch`.
fn leitura(ch: &Choices, run: Corrida) -> Pintado {
    let total = QUADROS[usize::from(ch.pick(Pick::RenderQuality))];
    let q = run.quadro(total);
    let percent = (q as f32 * CEM / total as f32).round() as u32;
    let preview = (
        tr("panel.widget_gallery.render.tag.preview").to_owned(),
        TagTone::Accent,
    );
    let hot = tr("panel.widget_gallery.render.tag.hot_reload").to_owned();
    let (status, meio, tom_hot) = match run.fase {
        Fase::Pronta => (
            tr("panel.widget_gallery.render.status.ready").to_owned(),
            (
                tr("panel.widget_gallery.render.tag.ready").to_owned(),
                TagTone::Success,
            ),
            TagTone::Warn,
        ),
        Fase::Correndo => (
            tr("panel.widget_gallery.render.status.running")
                .replace("{frame}", &q.to_string())
                .replace("{total}", &total.to_string()),
            (
                tr("panel.widget_gallery.render.tag.rendering").to_owned(),
                TagTone::Warn,
            ),
            TagTone::Neutral,
        ),
        Fase::Feita => {
            let formatos = [
                "panel.widget_gallery.render.format.png",
                "panel.widget_gallery.render.format.mp4",
                "panel.widget_gallery.render.format.webp",
                "panel.widget_gallery.render.format.gif",
            ];
            let f = tr(formatos[usize::from(ch.pick(Pick::RenderFormat))]).to_lowercase();
            (
                tr("panel.widget_gallery.render.status.saved").replace("{ext}", &f),
                (
                    tr("panel.widget_gallery.render.tag.done").to_owned(),
                    TagTone::Success,
                ),
                TagTone::Warn,
            )
        }
    };
    Pintado {
        status,
        percent,
        spinner: run.fase == Fase::Correndo,
        tags: vec![preview, meio, (hot, tom_hot)],
        niveis: niveis(&run),
    }
}

/// Format e Quality lado a lado, ou um por linha quando a pílula não cabe em meia coluna.
fn saida(c: &mut CardCtx<'_>, ch: &Choices, y: f32) -> f32 {
    let formatos = segs(g::RN_FORMATS, ch, c.store);
    let quals = segs(g::RN_QUALITY, ch, c.store);
    let formato = Cell::new(dropdown(g::RN_FORMAT, &formatos))
        .label(tr("panel.widget_gallery.render.format"));
    let qualidade =
        Cell::new(Widget::Choice(&quals)).label(tr("panel.widget_gallery.render.quality"));
    let pilula: f32 = quals
        .iter()
        .map(|s| {
            c.text_system.prefix_width(s.text, m::SEG_TEXT_PX) + m::SEG_INSET * 2.0 + m::SEG_PAD
        })
        .sum::<f32>()
        + m::SEG_PAD;
    if pilula <= (c.w - m::FIELD_GAP) * 0.5 {
        return c.row(&[formato, qualidade], y);
    }
    let y = c.row(&[formato], y);
    c.row(&[qualidade], y)
}

/// O spinner gira um degrau por passo: o glifo pinta-se numa cena à parte e entra rodado.
fn spinner(
    scene: &mut VectorScene,
    r: Rect,
    passos: u32,
    ink: ph2d_editor_core::widget::card_ink::CardInk,
) {
    let mut s = VectorScene::new();
    paint_spinner(&Spinner::new(g::CARD_RENDER, ""), r, &mut s, ink.theme);
    let centro = (f64::from(r.x + r.w * 0.5), f64::from(r.y + r.h * 0.5));
    scene.append_transformed(&s, Affine::rotate_about(f64::from(passos) * GIRO, centro));
}

/// A linha de estado (spinner · texto · `%` à direita) e a barra fina por baixo. Devolve o fundo.
fn estado(c: &mut CardCtx<'_>, run: Corrida, l: &Pintado, y: f32) -> f32 {
    let y = y + m::LABEL_TOP;
    let linha = m::ICON_GLYPH;
    let texto_y = y + ph2d_editor_core::paint::folga_do_texto(linha, m::VALUE_PX);
    let mut x = c.x;
    if l.spinner {
        spinner(c.scene, Rect::new(x, y, linha, linha), run.passos, c.ink);
        x += linha + m::SEG_ICON_GAP;
    }
    let pct = tr("panel.widget_gallery.render.percent").replace("{n}", &l.percent.to_string());
    let pct_w = c.text_system.prefix_width(&pct, m::VALUE_PX).ceil();
    paint_text_elided(
        c.text_system,
        c.scene,
        &pct,
        c.x + c.w - pct_w,
        texto_y,
        m::VALUE_PX,
        pct_w,
        c.ink.color(ColorToken::Text2),
    );
    paint_text_elided(
        c.text_system,
        c.scene,
        &l.status,
        x,
        texto_y,
        m::VALUE_PX,
        (c.x + c.w - pct_w - m::FIELD_GAP - x).max(0.0),
        c.ink.color(ColorToken::Text1),
    );
    let y = y + linha + m::LABEL_BOTTOM;
    let h = m::FIELD_GAP;
    let bar = ProgressBar::new(g::CARD_RENDER, "").determinate(l.percent as f32 / CEM);
    paint_progress_bar_on_card(&bar, Rect::new(c.x, y, c.w, h), c.scene, c.ink);
    y + h
}

/// Os dois canais deitados, cada um com a sua letra, e a escala em dB por baixo. Devolve o fundo.
fn medidor(c: &mut CardCtx<'_>, run: Corrida, l: &Pintado, y: f32) -> f32 {
    let y = y + m::LABEL_TOP;
    card_field::paint_label(
        c.scene,
        c.text_system,
        c.ink,
        c.x,
        y,
        c.w,
        tr("panel.widget_gallery.render.output_level"),
    );
    let mut y = y + m::LABEL_PX + m::LABEL_BOTTOM;
    let letra = m::KEY_MARK_BOX;
    let (bx, bw) = (c.x + letra + m::SEG_PAD, c.w - letra - m::SEG_PAD);
    let picos = if run.fase == Fase::Correndo {
        run.picos
    } else {
        PICO_REPOUSO
    };
    for (i, (nome, nivel)) in [
        (tr("panel.widget_gallery.render.channel_l"), l.niveis[0]),
        (tr("panel.widget_gallery.render.channel_r"), l.niveis[1]),
    ]
    .into_iter()
    .enumerate()
    {
        let ly = y + ph2d_editor_core::paint::folga_do_texto(ALTURA_DO_MEDIDOR, m::LABEL_PX);
        card_field::paint_label(c.scene, c.text_system, c.ink, c.x, ly, letra, nome);
        paint_level_channel_on_card(
            c.scene,
            Rect::new(bx, y, bw, ALTURA_DO_MEDIDOR),
            nivel,
            picos[i],
            false,
            c.ink,
        );
        y += ALTURA_DO_MEDIDOR + m::FIELD_GAP;
    }
    for db in ESCALA_DB {
        let t = db.to_string();
        let w = c.text_system.prefix_width(&t, m::LABEL_PX).ceil();
        let centro = bx + bw * fraction_of_db(db as f32);
        let tx = (centro - w * 0.5).min(bx + bw - w);
        card_field::paint_label(c.scene, c.text_system, c.ink, tx, y, w, &t);
    }
    y + m::LABEL_PX
}

/// As etiquetas, com a largura do texto; quebram de linha quando a coluna acaba. Devolve o fundo.
fn etiquetas(c: &mut CardCtx<'_>, l: &Pintado, y: f32) -> f32 {
    let mut y = y + m::LABEL_TOP;
    let mut x = c.x;
    for (text, tom) in &l.tags {
        let tag = Tag::new(g::CARD_RENDER, text.as_str()).tone(*tom);
        let w = tag.natural_width(c.text_system, ALTURA_DA_ETIQUETA);
        if x > c.x && x + w > c.x + c.w {
            x = c.x;
            y += ALTURA_DA_ETIQUETA + m::FIELD_GAP;
        }
        paint_tag_with_radius(
            &tag,
            Rect::new(x, y, w, ALTURA_DA_ETIQUETA),
            m::SEG_RADIUS,
            c.scene,
            c.text_system,
            c.ink.theme,
        );
        x += w + m::FIELD_GAP;
    }
    y + ALTURA_DA_ETIQUETA
}

pub(crate) fn render(ch: Choices, run: Corrida) -> CardDef<'static> {
    card(
        g::CARD_RENDER,
        tr("panel.widget_gallery.card.render"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let l = leitura(&ch, run);
            let y = saida(c, &ch, y);
            let y = estado(c, run, &l, y);
            let y = medidor(c, run, &l, y);
            let y = etiquetas(c, &l, y);
            let correr = Cell::new(Widget::Button {
                id: g::RN_RUN,
                text: tr("panel.widget_gallery.render.run"),
                primary: true,
            });
            let cancelar = Cell::new(Widget::Button {
                id: g::RN_CANCEL,
                text: tr("panel.widget_gallery.render.cancel"),
                primary: false,
            });
            // Sem corrida não há o que cancelar: um Cancel à vista seria um no-op (zero no-op, §2).
            let y = y + m::FIELD_ROW_GAP;
            let y = if run.fase == Fase::Pronta {
                c.row(&[correr], y)
            } else {
                c.row(&[correr, cancelar], y)
            };
            PINTADO.with(|p| *p.borrow_mut() = Some(l));
            y
        }),
    )
}
