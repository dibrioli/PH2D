//! **Os ajustes da forma viva** (P6f, filho de [`super`]): o desfazer dela (um arrasto, um campo da
//! barra ou um deslizar do painel = um passo, a lei do Vector), a barra da moldura e a ETIQUETA do
//! arrasto do Vector.

use super::*;

/// Um instante da forma viva — a caixa e os campos do painel: o passo do desfazer dela.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Foto {
    w: u32,
    h: u32,
    m: Affine,
    pivot: Pivot,
    valores: PixelShapeValues,
    fill: bool,
}

pub(super) fn foto(app: &PixelApp, v: &Viva) -> Foto {
    Foto {
        w: v.w,
        h: v.h,
        m: v.m,
        pivot: v.pivot,
        valores: app.options.shapes.get(v.forma),
        fill: app.options.shape_fill,
    }
}

fn foto_agora(app: &PixelApp) -> Option<Foto> {
    app.live_shape().map(|v| foto(app, v))
}

/// Volta a forma viva ao instante `f` (a caixa e os campos do painel).
fn repoe(app: &mut PixelApp, f: Foto) {
    let Some(v) = viva_mut(app) else {
        return;
    };
    (v.w, v.h, v.m, v.pivot) = (f.w, f.h, f.m, f.pivot);
    let forma = v.forma;
    app.options.shapes.set(forma, f.valores);
    app.options.shape_fill = f.fill;
    app.reaviva();
}

/// Os campos (bit `i`) e o Fill (o bit 63) em que `a` e `b` diferem.
fn mexidos(a: &Foto, b: &Foto) -> u64 {
    let campos = a
        .valores
        .0
        .iter()
        .zip(b.valores.0)
        .enumerate()
        .filter(|(_, (x, y))| **x != *y)
        .fold(0, |m, (i, _)| m | (1u64 << i));
    campos | (u64::from(a.fill != b.fill) << 63)
}

/// Guarda `antes` como um passo do desfazer da forma viva (se ela mudou desde então). `painel` =
/// veio de um campo do painel: o pedido seguido que mexe nos mesmos campos junta-se ao primeiro.
pub(super) fn guarda(app: &mut PixelApp, antes: Foto, painel: bool) {
    let Some(agora) = foto_agora(app) else {
        return;
    };
    let Some(v) = viva_mut(app) else {
        return;
    };
    if agora == antes {
        return;
    }
    let m = mexidos(&antes, &agora);
    let junta = painel && v.painel == Some(m) && agora.m == antes.m;
    v.painel = painel.then_some(m);
    if junta {
        return;
    }
    v.passos.push(antes);
    v.refeitos.clear();
}

/// Um pedido das opções do painel com uma forma viva: aplica-o e, se mexeu nos campos dela ou no
/// Fill, é um passo do desfazer dela.
pub(crate) fn opcoes(app: &mut PixelApp, o: Options) {
    opcoes_com(app, o, true);
}

/// [`opcoes`] que nunca se junta ao passo anterior (a cena de smoke: um ajuste já feito).
pub(crate) fn ajuste(app: &mut PixelApp, o: Options) {
    opcoes_com(app, o, false);
}

fn opcoes_com(app: &mut PixelApp, o: Options, painel: bool) {
    let antes = foto_agora(app);
    app.options = o;
    app.reaviva();
    if let Some(antes) = antes {
        guarda(app, antes, painel);
    }
}

/// Um campo da barra da moldura com a forma viva (a conta da moldura solta). Um passo.
pub(crate) fn campo(app: &mut PixelApp, f: PixelTransformField, valor: i32) {
    let Some(antes) = foto_agora(app) else {
        return;
    };
    if let Some(v) = viva_mut(app) {
        v.m = com_campo(v.m, v.pivot, (v.w, v.h), f, valor);
    }
    app.reaviva();
    guarda(app, antes, false);
}

/// **`Ctrl+Z` com uma forma viva**: desfaz o último ajuste dela; sem ajustes, larga-a (o passo que a
/// fez nascer) — o `Ctrl+Shift+Z` traz de volta. O documento não muda. `false` = não há forma viva.
pub(crate) fn desfaz(app: &mut PixelApp) -> bool {
    let Some(agora) = foto_agora(app) else {
        return false;
    };
    let Some(v) = viva_mut(app) else {
        return false;
    };
    if v.agarra.is_some() {
        return true;
    }
    v.painel = None;
    match v.passos.pop() {
        Some(f) => {
            v.refeitos.push(agora);
            repoe(app, f);
        }
        None => {
            let Some(g) = app.gesture.take() else {
                return true;
            };
            let stamp = app.stamp();
            let Some(d) = app.docs.get_mut(&g.key) else {
                return true;
            };
            d.stamp = stamp;
            app.largada = Some((g, d.history.undo_len(), agora));
        }
    }
    true
}

/// **`Ctrl+Shift+Z`**: refaz um ajuste da forma viva; sem forma viva, traz de volta a que o
/// `Ctrl+Z` largou (se o documento está onde ela o deixou). `false` = nada disto.
pub(crate) fn refaz(app: &mut PixelApp) -> bool {
    if let Some(agora) = foto_agora(app) {
        let Some(v) = viva_mut(app) else {
            return false;
        };
        if v.agarra.is_none() {
            v.painel = None;
            if let Some(f) = v.refeitos.pop() {
                v.passos.push(agora);
                repoe(app, f);
            }
        }
        return true;
    }
    let volta = app.gesture.is_none()
        && app.largada.as_ref().is_some_and(|(g, n, _)| {
            app.docs
                .get(&g.key)
                .is_some_and(|d| d.history.undo_len() == *n)
        });
    if !volta {
        return false;
    }
    let Some((g, _, f)) = app.largada.take() else {
        return false;
    };
    if let Some(v) = g.viva.as_ref() {
        app.tool = crate::Tool::Shape(v.forma);
    }
    app.gesture = Some(g);
    repoe(app, f);
    true
}

/// A cena de smoke: a alça `k` da forma viva AGARRADA, sem botão premido (a foto do arrasto — a
/// caixa só-contorno e a etiqueta). `None` = não há forma viva ou a alça não tem ponto.
pub(crate) fn agarra_para_a_foto(app: &mut PixelApp, k: usize) -> Option<()> {
    let v = app.live_shape()?;
    let (forma, g) = (v.forma, v.geo());
    let values = valores(app, forma);
    let h = law::handles_of(forma.kind()).get(k)?;
    let half = g.half;
    let ck = move |v: &ShapeValues| crate::formas::cook_local(forma, half, v);
    let p = law::handle_point(h, &values, half, &ck)?;
    let antes = foto(app, v);
    let v = viva_mut(app)?;
    v.agarra = Some(Agarra::Alca(HandleDrag::start(k, values, p, p, half)));
    v.premido = Some(antes);
    Some(())
}

/// **A ETIQUETA** da forma viva do documento `key` (a do Vector, `face::readout`): o campo da alça
/// agarrada — ou, sem arrasto, da que está sob o cursor — com o nome do painel e o número, e onde
/// ela pousa (o cursor; na cena de smoke, a alça mostrada).
#[must_use]
pub fn readout(app: &PixelApp, c: &Canvas<'_>, key: u32) -> Option<(String, (f32, f32))> {
    let (v, screen) = alvo(app, c, key)?;
    let values = valores(app, v.forma);
    let (_, (_, em_ecra)) = pegas(v, screen, &values);
    let k = match v.agarra {
        Some(Agarra::Alca(d)) => Some(d.handle),
        Some(Agarra::Caixa { .. }) => None,
        None => v.cursor.and_then(|p| hit(&em_ecra, p)).or(v.mostra),
    }?;
    #[expect(clippy::cast_possible_truncation, reason = "píxeis de ecrã")]
    let onde = v.cursor.or_else(|| {
        em_ecra
            .get(k)
            .copied()
            .flatten()
            .map(|p| (p.x as f32, p.y as f32))
    })?;
    let kind = v.forma.kind();
    let h = law::handles_of(kind).get(k)?;
    let name = |l: &'static str| ph2d_editor_core::pixel_panel::field_name(l).to_owned();
    let length =
        |w: f64| ph2d_i18n::tr_with("pixel.shape.length", &[("n", &format!("{}", w.round()))]);
    Some((face::readout(kind, h, &values, &name, &length), onde))
}

/// Pinta a ETIQUETA da forma viva (a ficha da casa, a do Vector) por cima de tudo, dentro de `area`.
pub(crate) fn ficha(
    app: &PixelApp,
    c: &Canvas<'_>,
    area: ph2d_editor_core::zones::Rect,
    theme: Theme,
    (scene, text): (&mut VectorScene, &mut ph2d_text::TextSystem),
) {
    let Some(key) = app
        .gesture
        .as_ref()
        .filter(|g| g.viva.is_some())
        .map(|g| g.key)
    else {
        return;
    };
    let Some((t, p)) = readout(app, c, key) else {
        return;
    };
    let w = ph2d_editor_core::readout::chip_width(text, &t, 0.0);
    let at = ph2d_editor_core::readout::at_cursor([p.0, p.1], w, area);
    ph2d_editor_core::readout::paint_chip(text, scene, &t, at, 0.0, theme);
}
