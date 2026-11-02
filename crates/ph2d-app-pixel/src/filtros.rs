//! **Edit ▸ Fill/Stroke, Invert e FX do lado da família** (P8, `docs/PixelArt/P8_plano.md` §2/§4):
//! Fill pinta a selecção com a cor de frente, Stroke a borda interna dela (`Mask::edge4`) — sem
//! diálogo, desligados sem selecção (Aseprite). Invert/Despeckle/Outline abrem a janelinha; com
//! Preview o canvas mostra o op de [`filter::apply`] sem gravar (o mesmo caminho da moldura solta,
//! `Stroke::with_values`), recalculado só quando o pedido muda; o OK grava UM passo.

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::pixel_panel::{
    PixelDialogChoice, PixelDialogKind, PixelEditAction as E, PixelOutlineBg, PixelOutlineMatrix,
    PixelOutlinePlace,
};
use ph2d_pixel_core::filter::adjust::Invert;
use ph2d_pixel_core::filter::convolution::{Convolution, KERNELS};
use ph2d_pixel_core::filter::median::Median;
use ph2d_pixel_core::filter::outline::{self, Background, Outline, Place};
use ph2d_pixel_core::filter::{self, Channels, Edge, Filter};
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::{Op, Patch, Stroke};

use crate::PixelApp;

/// A pré-visualização em curso: o documento, o pedido e as cores de que ele depende, e o traço.
#[derive(Clone)]
pub(crate) struct Previa {
    key: u32,
    pedido: PixelDialogChoice,
    cores: [[u8; 4]; 2],
    traco: Option<Stroke>,
}

/// Fill/Stroke (um passo) ou a abertura de um diálogo de filtro.
pub(crate) fn menu(app: &mut PixelApp, key: u32, a: E) {
    let kind = match a {
        E::Invert => Some(PixelDialogKind::Invert),
        E::Despeckle => Some(PixelDialogKind::Despeckle),
        E::Outline => Some(PixelDialogKind::Outline),
        E::BrightnessContrast => Some(PixelDialogKind::BrightnessContrast),
        E::HueSaturation => Some(PixelDialogKind::HueSaturation),
        E::ColorCurve => Some(PixelDialogKind::ColorCurve),
        E::ConvolutionMatrix => Some(PixelDialogKind::ConvolutionMatrix),
        _ => None,
    };
    if let Some(kind) = kind {
        app.abre_dialogo = Some(kind);
        return;
    }
    let Some(doc) = app.docs.get(&key) else {
        return;
    };
    // Sem selecção o Aseprite desliga Fill e Stroke: nada acontece.
    let Some(m) = doc.sprite.mask().cloned() else {
        return;
    };
    if let Some(motivo) = crate::camadas::recusa_pintar(doc) {
        return app.recusa(motivo);
    }
    let s = &doc.sprite;
    let m = if a == E::Stroke { m.edge4() } else { m };
    let op = pinta(s, (doc.layer, doc.frame), &m, s.value_of_rgba(app.fg));
    app.passo(key, op);
}

/// As células de `m` no cel `ck` com o valor `v` (`None` = nada muda).
fn pinta(s: &ph2d_pixel_core::Sprite, ck: ph2d_pixel_core::CelKey, m: &Mask, v: u32) -> Option<Op> {
    let ck = s.owner(ck);
    let rect = m.bounds()?;
    let bpp = s.mode().bpp();
    let before = s.read_rect(ck, rect);
    let mut pixels = before.clone();
    for (x, y) in m.cells() {
        let i = ((y - rect.y) as usize * rect.w as usize + (x - rect.x) as usize) * bpp;
        pixels[i..i + bpp].copy_from_slice(&v.to_le_bytes()[..bpp]);
    }
    (pixels != before).then(|| {
        Op::Patch(Patch {
            layer: ck.0,
            frame: ck.1,
            bbox: Some(s.cel(ck).map_or(rect, |c| c.bbox.union(rect))),
            rect,
            pixels,
        })
    })
}

/// Os cels do alvo: o activo, ou com «All» todos os quadros × as camadas que se pintam (nem grupo,
/// nem trancada, nem escondida) — o Trial com «All» inverte os 2 quadros × 2 camadas (rodada 3 `C`).
fn alvo(doc: &crate::Doc, todas: bool) -> Vec<ph2d_pixel_core::CelKey> {
    if !todas {
        return vec![(doc.layer, doc.frame)];
    }
    let s = &doc.sprite;
    s.layers()
        .iter()
        .filter(|l| !l.is_group() && !l.locked && s.is_shown(l.id))
        .flat_map(|l| s.frames().iter().map(move |f| (l.id, f.id)))
        .collect()
}

/// O op do filtro `c` sobre o alvo dele em `key`; `todas = false` força o cel activo (o Preview
/// mostra só o cel activo). `None` = nada muda ou não é um filtro.
pub(crate) fn op(app: &PixelApp, key: u32, c: PixelDialogChoice, todas: bool) -> Option<Op> {
    let doc = app.docs.get(&key)?;
    let (f, ch): (Box<dyn Filter>, _) = match c {
        PixelDialogChoice::Invert { channels, .. } => (Box::new(Invert), channels),
        PixelDialogChoice::Despeckle {
            width,
            height,
            tiled,
            channels,
            ..
        } => (
            Box::new(Median {
                w: width,
                h: height,
                edge: if tiled { Edge::Wrap } else { Edge::Clamp },
            }),
            channels,
        ),
        PixelDialogChoice::Outline {
            place,
            matrix,
            background,
            tiled,
            channels,
            ..
        } => (
            Box::new(Outline {
                color: app.fg,
                background: match background {
                    PixelOutlineBg::Transparent => Background::Mask,
                    PixelOutlineBg::BackgroundColor => Background::Color(app.bg),
                },
                matrix: match matrix {
                    PixelOutlineMatrix::Circle => outline::CIRCLE,
                    PixelOutlineMatrix::Square => outline::SQUARE,
                    PixelOutlineMatrix::Horizontal => outline::HORIZONTAL,
                    PixelOutlineMatrix::Vertical => outline::VERTICAL,
                },
                place: match place {
                    PixelOutlinePlace::Outside => Place::Outside,
                    PixelOutlinePlace::Inside => Place::Inside,
                },
                tiled,
            }),
            channels,
        ),
        PixelDialogChoice::Convolution {
            kernel,
            tiled,
            channels,
            ..
        } => (
            Box::new(Convolution {
                kernel: KERNELS.get(usize::from(kernel))?,
                edge: if tiled { Edge::Wrap } else { Edge::Clamp },
            }),
            channels,
        ),
        c => crate::ajustes::filtro(c)?,
    };
    let [r, g, b, a] = ch;
    let channels = Channels { r, g, b, a };
    let keys = alvo(doc, todas && c.targets_all());
    filter::apply(&doc.sprite, &keys, channels, f.as_ref())
}

/// O OK de um filtro: UM passo (a camada que não se pinta recusa). `false` = não é um filtro.
pub(crate) fn aplica(app: &mut PixelApp, key: u32, c: PixelDialogChoice) -> bool {
    if !matches!(
        c,
        PixelDialogChoice::Invert { .. }
            | PixelDialogChoice::Despeckle { .. }
            | PixelDialogChoice::Outline { .. }
            | PixelDialogChoice::Convolution { .. }
    ) && crate::ajustes::filtro(c).is_none()
    {
        return false;
    }
    app.previa_filtro = None;
    // Com «All» a camada activa não decide: o alvo já tira as que não se pintam.
    let recusa = app.docs.get(&key).and_then(crate::camadas::recusa_pintar);
    if let (Some(motivo), false) = (recusa, c.targets_all()) {
        app.recusa(motivo);
        return true;
    }
    let o = op(app, key, c, true);
    app.passo(key, o);
    true
}

/// Uma vez por quadro: acompanha o diálogo aberto. Recalcula só quando o pedido (ou as cores de
/// que ele depende) muda; fechado ou com Preview desligado, larga a pré-visualização.
pub(crate) fn acompanha(app: &mut PixelApp, store: &WidgetStore) {
    let quer = store.pixel_filter_preview().zip(app.chave);
    let Some((pedido, key)) = quer else {
        app.previa_filtro = None;
        return;
    };
    let cores = [app.fg, app.bg];
    let igual = app
        .previa_filtro
        .as_ref()
        .is_some_and(|p| p.key == key && p.pedido == pedido && p.cores == cores);
    if igual {
        return;
    }
    let cel = app.docs.get(&key).map(|d| (d.layer, d.frame));
    let traco = op(app, key, pedido, false)
        .zip(cel)
        .map(|(o, cel)| traco(&o, cel));
    app.previa_filtro = Some(Previa {
        key,
        pedido,
        cores,
        traco,
    });
}

/// Os nomes dos kernels da Convolution Matrix, na ordem do núcleo — o que o diálogo lista.
pub(crate) fn nomes_dos_kernels() -> &'static [&'static str] {
    static NOMES: std::sync::LazyLock<Vec<&'static str>> =
        std::sync::LazyLock::new(|| KERNELS.iter().map(|k| k.name).collect());
    &NOMES
}

/// Os botões R G B A de cada kernel, na mesma ordem.
pub(crate) fn canais_dos_kernels() -> &'static [[bool; 4]] {
    static CANAIS: std::sync::LazyLock<Vec<[bool; 4]>> =
        std::sync::LazyLock::new(|| KERNELS.iter().map(|k| k.channels).collect());
    &CANAIS
}

/// O traço da pré-visualização do filtro sobre `key`, se há.
pub(crate) fn previa(app: &PixelApp, key: u32) -> Option<Stroke> {
    app.previa_filtro
        .as_ref()
        .filter(|p| p.key == key)
        .and_then(|p| p.traco.clone())
}

/// Os valores que o op escreve, como um traço de valores prontos.
fn traco(o: &Op, cel: ph2d_pixel_core::CelKey) -> Stroke {
    let mut values = Vec::new();
    let mut alvo = None;
    let mut patch = |p: &Patch| {
        alvo.get_or_insert((p.layer, p.frame));
        let bpp = p.pixels.len() / (p.rect.w as usize * p.rect.h as usize).max(1);
        for y in 0..p.rect.h {
            for x in 0..p.rect.w {
                let i = (y as usize * p.rect.w as usize + x as usize) * bpp;
                let mut b = [0u8; 4];
                b[..bpp].copy_from_slice(&p.pixels[i..i + bpp]);
                values.push((
                    (p.rect.x + x as i32, p.rect.y + y as i32),
                    u32::from_le_bytes(b),
                ));
            }
        }
    };
    match o {
        Op::Patch(p) => patch(p),
        Op::Batch(ops) => ops.iter().for_each(|o| {
            if let Op::Patch(p) = o {
                patch(p);
            }
        }),
        _ => {}
    }
    let (layer, frame) = alvo.unwrap_or(cel);
    Stroke::with_values(layer, frame, values)
}

impl PixelApp {
    /// A cor que o canvas MOSTRA na célula `(x, y)` do cel activo de `key`: a pré-visualização de
    /// um filtro ou de uma moldura solta por cima do documento (os testes de costura leem-na).
    pub fn shown_rgba(&mut self, key: u32, x: i32, y: i32) -> Option<[u8; 4]> {
        let previa = self.previa(key).map(|(t, _)| t);
        let doc = self.docs.get(&key)?;
        let s = &doc.sprite;
        let v = previa
            .and_then(|t| t.value_at(s, x, y))
            .unwrap_or_else(|| s.value_at((doc.layer, doc.frame), x, y));
        Some(s.rgba_of_value(v))
    }
}
