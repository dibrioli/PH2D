//! **A forma VIVA** (P6d, dono 27/10: *«os ajustes devem ser em tempo real no gizmo igual ao módulo
//! vector»*): largar o arrasto de um Retângulo, Elipse, Polígono ou Estrela não grava — a forma
//! flutua sobre o cel com a caixa do gizmo do sprite (a MESMA da moldura solta,
//! [`crate::solta::gizmo_of`]) e as alças próprias da forma, cuja LEI é a do Vector
//! ([`ph2d_tool_vector::shape_handles`]) e cuja pintura é a dele
//! ([`ph2d_vec_render::draw_shape_handles`]). Os valores moram nos campos do painel
//! (`Options::shapes`): alça e campo são duas mãos sobre o MESMO número, e qualquer das duas
//! re-cozinha a forma na hora. `Enter`, outra ferramenta, um clique fora ou sair do Draw assentam-na
//! em células (UM passo); `Esc` larga-a (a meio de um arrasto, só desfaz o arrasto).
//!
//! Espaço LOCAL = o do Vector: centrado, Y para cima, em células; [`Geo::xf`] leva-o à imagem
//! (cantos das células, Y para baixo) com o giro da caixa e escala 1 — esticar a caixa muda a
//! largura e a altura da forma, nunca os raios dela.

use ph2d_editor_core::DocPx;
use ph2d_editor_core::pixel_panel::{PixelShape, PixelShapeValues};
use ph2d_pixel_core::affine::{Affine, Handle, Linear, Pivot, Point as Ponto, wrap_deg};
use ph2d_pixel_core::raster::Cell;
use ph2d_pixel_render::px_per_cell;
use ph2d_tokens::Theme;
use ph2d_tool_vector::shape_handles::{self as law, HIT_PX, HandleDrag, PARK_PX};
use ph2d_tool_vector::shape_handles_face as face;
use ph2d_tool_vector::shapes::FieldUnit;
use ph2d_vec_scene::{ShapeValues, VecPath};
use ph2d_vector::{Point, VectorScene};

use crate::gesture::Mods;
use crate::input::{Canvas, Press, to_screen};
use crate::solta::{drag_opts, gizmo_of, hit_of, imagem, janela};
use crate::{Options, PixelApp};

/// Abaixo disto o giro e a inclinação são zero: a forma assenta pela caixa inteira (as leis medidas).
const EPS_DEG: f64 = 1e-9;

/// O que a mão agarrou.
#[derive(Debug, Clone, Copy)]
enum Agarra {
    /// Uma alça da caixa (escala, giro, mover, pivô) — a conta do núcleo, como na moldura solta.
    Caixa {
        handle: Handle,
        from: Ponto,
        m0: Affine,
        p0: Pivot,
    },
    /// Uma alça da forma (a lei do Vector).
    Alca(HandleDrag),
}

/// A forma viva: a caixa (`w × h` células levadas por `m`) e a forma do painel.
#[derive(Debug, Clone)]
pub struct Viva {
    pub(crate) forma: PixelShape,
    w: u32,
    h: u32,
    pub(crate) m: Affine,
    pivot: Pivot,
    agarra: Option<Agarra>,
    /// O último ponto de ecrã do ponteiro (a alça quente; os cantos do giro).
    cursor: Option<(f32, f32)>,
    /// O último premir numa alça da forma (o par do duplo-clique, que a repõe).
    ultimo: Option<(std::time::Instant, usize)>,
}

/// A geometria do momento: a meia-caixa local (de centro a centro de célula), o local → imagem e,
/// sem giro, a caixa inteira em células.
struct Geo {
    half: [f64; 2],
    xf: ph2d_vector::Affine,
    caixa: Option<(Cell, Cell)>,
}

/// A régua dos campos `Px`: um px do Vector vale uma CÉLULA no Pixel (P6b).
fn doc() -> DocPx {
    DocPx::new(1.0)
}

impl Viva {
    /// A forma `forma` na caixa ↖ `a` ↘ `b` (inclusiva, em células).
    #[must_use]
    pub fn new(forma: PixelShape, a: Cell, b: Cell) -> Self {
        let (w, h) = (
            (b.0 - a.0 + 1).unsigned_abs(),
            (b.1 - a.1 + 1).unsigned_abs(),
        );
        Self {
            forma,
            w,
            h,
            m: Affine::at(f64::from(a.0), f64::from(a.1)),
            pivot: Pivot {
                u: f64::from(w) / 2.0,
                v: f64::from(h) / 2.0,
            },
            agarra: None,
            cursor: None,
            ultimo: None,
        }
    }

    /// A caixa girada `deg` graus (horário na tela) em volta do centro.
    #[must_use]
    pub fn girada(mut self, deg: f64) -> Self {
        let lin = Linear {
            rot: deg,
            sx: 1.0,
            sy: 1.0,
            skew: 0.0,
        };
        let (cu, cv) = (f64::from(self.w) / 2.0, f64::from(self.h) / 2.0);
        let c = self.m.apply(cu, cv);
        let r = Affine::compose(lin, 0.0, 0.0);
        self.m = Affine {
            e: c.x - (r.a * cu + r.c * cv),
            f: c.y - (r.b * cu + r.d * cv),
            ..r
        };
        self
    }

    fn geo(&self) -> Geo {
        let (w, h) = (f64::from(self.w), f64::from(self.h));
        let lin = self.m.decompose();
        let (ww, hh) = (
            (lin.sx * w).abs().round().max(1.0),
            (lin.sy * h).abs().round().max(1.0),
        );
        let c = self.m.apply(w / 2.0, h / 2.0);
        let u = Affine::compose(
            Linear {
                rot: lin.rot,
                sx: 1.0,
                sy: 1.0,
                skew: lin.skew,
            },
            0.0,
            0.0,
        );
        // Local (Y para cima) → imagem (Y para baixo): a coluna do Y troca de sinal.
        let xf = ph2d_vector::Affine::new([u.a, u.b, -u.c, -u.d, c.x, c.y]);
        let reta = wrap_deg(lin.rot).abs() < EPS_DEG && lin.skew.abs() < EPS_DEG;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "células de um sprite (≤ 65535)"
        )]
        let caixa = reta.then(|| {
            let a = (
                (c.x - ww / 2.0).round() as i32,
                (c.y - hh / 2.0).round() as i32,
            );
            (a, (a.0 + ww as i32 - 1, a.1 + hh as i32 - 1))
        });
        Geo {
            half: [(ww - 1.0) / 2.0, (hh - 1.0) / 2.0],
            xf,
            caixa,
        }
    }

    /// `(contorno, interior)` da forma com os campos e o «Fill» do painel. Sem giro, pela caixa
    /// inteira — a MESMA conta do arrasto que a desenhou ([`crate::formas::cells`]).
    #[must_use]
    pub fn cells(&self, o: &Options) -> (Vec<Cell>, Vec<Cell>) {
        let v = o.shapes.get(self.forma);
        let g = self.geo();
        match g.caixa {
            Some((a, b)) => crate::formas::cells(self.forma, &v, a, b, o.shape_fill),
            None => {
                let to_cell = |p: [f64; 2]| {
                    let q = g.xf * Point::new(p[0], p[1]);
                    [q.x - 0.5, q.y - 0.5]
                };
                crate::formas::cells_mapped(self.forma, &v, g.half, &to_cell, o.shape_fill)
            }
        }
    }

    /// A forma do painel que ela é.
    #[must_use]
    pub fn forma(&self) -> PixelShape {
        self.forma
    }

    /// Há um arrasto (de caixa ou de alça) em curso.
    #[must_use]
    pub fn dragging(&self) -> bool {
        self.agarra.is_some()
    }

    /// As pegas das alças da forma, em LOCAL (`px` = células por px de ecrã).
    fn grips(&self, g: &Geo, v: &ShapeValues, px: f64) -> Vec<Option<[f64; 2]>> {
        let (kind, forma, half) = (self.forma.kind(), self.forma, g.half);
        let ck = move |v: &ShapeValues| crate::formas::cook_local(forma, half, v);
        law::handles_of(kind)
            .iter()
            .map(|h| law::handle_grip(kind, h, v, half, doc(), PARK_PX * px, &ck))
            .collect()
    }
}

/// A geometria, as pegas (local) e onde cada uma cai no ECRÃ (`screen` = imagem → ecrã) — a conta
/// única do desenho, do clique e da sonda.
fn pegas(
    v: &Viva,
    screen: ph2d_vector::Affine,
    values: &ShapeValues,
) -> (Geo, Vec<Option<[f64; 2]>>, Vec<Option<Point>>) {
    let g = v.geo();
    let grips = v.grips(&g, values, 1.0 / px_per_cell(screen));
    let k = screen * g.xf;
    let em_ecra = grips
        .iter()
        .map(|q| q.map(|q| k * Point::new(q[0], q[1])))
        .collect();
    (g, grips, em_ecra)
}

/// As alças da forma viva do documento `key` em píxeis de ECRÃ, pela ordem da lei do Vector
/// (`None` = a que não se desenha). Vazio sem forma viva.
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "píxeis de ecrã")]
pub fn handles_on_screen(app: &PixelApp, c: &Canvas<'_>, key: u32) -> Vec<Option<(f32, f32)>> {
    let Some((v, screen)) = alvo(app, c, key) else {
        return Vec::new();
    };
    let (.., em_ecra) = pegas(v, screen, &valores(app, v.forma));
    em_ecra
        .into_iter()
        .map(|p| p.map(|p| (p.x as f32, p.y as f32)))
        .collect()
}

/// A alça da forma sob o ponto de ecrã `p`: a ÚLTIMA da lista que o alcança (desenha-se por cima).
fn hit(screen: &[Option<Point>], p: (f32, f32)) -> Option<usize> {
    let p = Point::new(f64::from(p.0), f64::from(p.1));
    screen
        .iter()
        .enumerate()
        .rev()
        .find_map(|(k, q)| ((*q)?.distance(p) <= HIT_PX).then_some(k))
}

/// A forma viva do documento `key` e a matriz imagem → ecrã do objecto dela.
fn alvo<'a>(
    app: &'a PixelApp,
    c: &Canvas<'_>,
    key: u32,
) -> Option<(&'a Viva, ph2d_vector::Affine)> {
    let g = app.gesture.as_ref().filter(|g| g.key == key)?;
    let v = g.viva.as_ref()?;
    let screen = to_screen(
        c.world,
        g.entity,
        app.docs.get(&key)?,
        c.camera,
        c.window,
        c.ppm,
    )?;
    Some((v, screen))
}

fn viva_mut(app: &mut PixelApp) -> Option<&mut Viva> {
    app.gesture.as_mut()?.viva.as_mut()
}

/// Os valores do painel para a forma.
fn valores(app: &PixelApp, f: PixelShape) -> ShapeValues {
    app.options.shapes.get(f).values()
}

/// Escreve os valores da forma no painel (a porta única dos campos) e re-cozinha.
fn escreve(app: &mut PixelApp, f: PixelShape, v: ShapeValues) {
    let mut v = v;
    for (i, d) in ph2d_tool_vector::shapes::desc(f.kind())
        .fields
        .iter()
        .enumerate()
    {
        // Os comprimentos são células inteiras (o painel guarda-os assim).
        if matches!(d.unit, FieldUnit::Px) {
            v[i] = v[i].round();
        }
    }
    app.options
        .shapes
        .set(f, PixelShapeValues(v.map(f64::to_bits)));
    app.reaviva();
}

impl PixelApp {
    /// Re-cozinha a forma viva com os campos e o «Fill» de agora (um campo do painel mudou).
    pub(crate) fn reaviva(&mut self) {
        let o = self.options;
        let stamp = self.stamp();
        let Some(g) = self.gesture.as_mut() else {
            return;
        };
        let (Some(v), Some(doc)) = (g.viva.as_ref(), self.docs.get_mut(&g.key)) else {
            return;
        };
        let (contorno, interior) = v.cells(&o);
        g.stroke.set_shape(&doc.sprite, &contorno, &interior);
        doc.stamp = stamp;
    }

    /// Há uma forma viva à espera do `Enter`.
    #[must_use]
    pub fn live_shape(&self) -> Option<&Viva> {
        self.gesture.as_ref()?.viva.as_ref()
    }
}

/// **Largar o arrasto de uma forma**: em vez de gravar, ela nasce viva. `false` = não é uma forma
/// (ou foi um clique sem arrasto, que grava como antes).
pub(crate) fn nasce(app: &mut PixelApp) -> bool {
    let Some(forma) = PixelShape::of_tool(app.tool) else {
        return false;
    };
    let Some(g) = app.gesture.as_mut() else {
        return false;
    };
    let crate::gesture::Kind::Box { from, to } = g.kind else {
        return false;
    };
    if from == to || g.viva.is_some() {
        return false;
    }
    let (a, b) = crate::gesture::boxed(from, to, g.mods);
    g.held = false;
    g.viva = Some(Viva::new(forma, a, b));
    app.reaviva();
    true
}

/// Assenta a forma viva, se há (UM passo).
pub(crate) fn assenta(app: &mut PixelApp) {
    if app.live_shape().is_some() {
        crate::input::finish(app);
    }
}

/// **Premir com uma forma viva**: uma alça da forma ou da caixa começa o arrasto; fora delas a
/// forma ASSENTA e o premir segue (desenha a próxima). `None` = não há forma viva ou assentou.
pub(crate) fn press(app: &mut PixelApp, c: &Canvas<'_>, key: u32, p: (f32, f32)) -> Option<Press> {
    app.live_shape()?;
    let Some((v, screen)) = alvo(app, c, key) else {
        assenta(app);
        return None;
    };
    let values = valores(app, v.forma);
    let (g, grips, em_ecra) = pegas(v, screen, &values);
    let local = |p: (f32, f32)| {
        let q = (screen * g.xf).inverse() * Point::new(f64::from(p.0), f64::from(p.1));
        [q.x, q.y]
    };
    let agora = std::time::Instant::now();
    if let Some(a) = hit(&em_ecra, p) {
        let kind = v.forma.kind();
        let h = &law::handles_of(kind)[a];
        let duplo = v.ultimo.is_some_and(|(t, b)| {
            b == a
                && agora.duration_since(t).as_nanos()
                    < ph2d_editor_core::interaction::DOUBLE_CLICK_WINDOW_NS
        });
        let forma = v.forma;
        if duplo {
            let novos = law::reset(kind, a, &values);
            if let Some(v) = viva_mut(app) {
                v.ultimo = None;
            }
            escreve(app, forma, novos);
            return Some(Press::Done);
        }
        // A pega mede-se ao ponto VERDADEIRO (agarrar sem mexer não muda nada); os ângulos ao
        // DESENHADO — a lei do Vector (`shape_handles_live::press`).
        let half = g.half;
        let ck = move |v: &ShapeValues| crate::formas::cook_local(forma, half, v);
        let verdade = if law::reads_only_the_angle(h) {
            grips[a]
        } else {
            law::handle_point(h, &values, half, &ck)
        }?;
        let d = HandleDrag::start(a, values, local(p), verdade, half);
        let v = viva_mut(app)?;
        v.agarra = Some(Agarra::Alca(d));
        v.ultimo = Some((agora, a));
        return Some(Press::Done);
    }
    let view = gizmo_of(v.m, (v.w, v.h), v.pivot, screen, janela(c), Some(p));
    let Some(handle) = hit_of(&view, p) else {
        assenta(app);
        return None;
    };
    let from = imagem(screen, Point::new(f64::from(p.0), f64::from(p.1)));
    let v = viva_mut(app)?;
    v.ultimo = None;
    v.agarra = Some(Agarra::Caixa {
        handle,
        from,
        m0: v.m,
        p0: v.pivot,
    });
    Some(Press::Done)
}

/// Mover com uma forma viva: o arrasto em curso re-cozinha-a; sem arrasto, só guarda o ponteiro
/// (a alça quente). `false` = não há forma viva.
pub(crate) fn moved(app: &mut PixelApp, c: &Canvas<'_>, p: (f32, f32), m: Mods) -> bool {
    let Some(key) = app
        .gesture
        .as_ref()
        .filter(|g| g.viva.is_some())
        .map(|g| g.key)
    else {
        return false;
    };
    let stamp = app.stamp();
    if let Some(d) = app.docs.get_mut(&key) {
        d.stamp = stamp;
    }
    let Some((v, screen)) = alvo(app, c, key) else {
        return true;
    };
    let (forma, agarra, g) = (v.forma, v.agarra, v.geo());
    let to = imagem(screen, Point::new(f64::from(p.0), f64::from(p.1)));
    match agarra {
        None => {}
        Some(Agarra::Caixa {
            handle,
            from,
            m0,
            p0,
        }) => {
            let size = (v.w, v.h);
            if let Some(v) = viva_mut(app) {
                (v.m, v.pivot) = m0.drag(p0, size, handle, (from, to), drag_opts(m));
            }
            app.reaviva();
        }
        Some(Agarra::Alca(mut d)) => {
            let q = g.xf.inverse() * Point::new(to.x, to.y);
            let half = g.half;
            let ck =
                move |v: &ShapeValues| -> VecPath { crate::formas::cook_local(forma, half, v) };
            let mods = law::Mods {
                snap: m.shift,
                pairs: m.ctrl,
                alone: m.alt,
            };
            let px = 1.0 / px_per_cell(screen);
            let novos = law::drag(forma.kind(), &mut d, [q.x, q.y], mods, half, px, doc(), &ck);
            if let Some(v) = viva_mut(app) {
                v.agarra = Some(Agarra::Alca(d));
            }
            escreve(app, forma, novos);
        }
    }
    if let Some(v) = viva_mut(app) {
        v.cursor = Some(p);
    }
    true
}

/// Largar com uma forma viva: o arrasto acaba e ela continua viva. `false` = não há forma viva.
pub(crate) fn release(app: &mut PixelApp) -> bool {
    let Some(g) = app.gesture.as_mut().filter(|g| g.viva.is_some()) else {
        return false;
    };
    g.held = false;
    if let Some(v) = g.viva.as_mut() {
        v.agarra = None;
    }
    true
}

/// `Esc` a meio de um arrasto: a caixa ou os campos voltam aos do premir e a forma continua viva.
/// `false` = sem arrasto (o `Esc` larga a forma).
pub(crate) fn escape(app: &mut PixelApp) -> bool {
    let Some(v) = viva_mut(app) else {
        return false;
    };
    let forma = v.forma;
    match v.agarra.take() {
        None => false,
        Some(Agarra::Caixa { m0, p0, .. }) => {
            (v.m, v.pivot) = (m0, p0);
            app.reaviva();
            true
        }
        Some(Agarra::Alca(d)) => {
            escreve(app, forma, d.at_press);
            true
        }
    }
}

/// **O desenho da forma viva** do documento `key` (imagem → ecrã `screen`): a caixa do gizmo do
/// sprite e, por cima, as alças da forma pela pintura do Vector — a quente (agarrada ou sob o
/// cursor) com o trilho dela e, a meio do arrasto de um raio, o círculo tracejado.
pub(crate) fn draw(
    app: &PixelApp,
    key: u32,
    screen: ph2d_vector::Affine,
    (theme, window): (Theme, (f32, f32)),
    scene: &mut VectorScene,
) {
    let Some(v) = app
        .gesture
        .as_ref()
        .filter(|g| g.key == key)
        .and_then(|g| g.viva.as_ref())
    else {
        return;
    };
    let view = gizmo_of(v.m, (v.w, v.h), v.pivot, screen, window, v.cursor);
    ph2d_editor_core::gizmo::paint_sprite_gizmo_with(
        scene,
        &view,
        theme,
        &mut ph2d_editor_core::interaction::HitIndex::new(),
        true,
    );
    let values = valores(app, v.forma);
    let (g, grips, em_ecra) = pegas(v, screen, &values);
    let (kind, forma, half) = (v.forma.kind(), v.forma, g.half);
    let k = screen * g.xf;
    let agarrada = match v.agarra {
        Some(Agarra::Alca(d)) => Some(d.handle),
        _ => None,
    };
    let quente = agarrada.or_else(|| v.cursor.and_then(|p| hit(&em_ecra, p)));
    let ck = move |v: &ShapeValues| crate::formas::cook_local(forma, half, v);
    let hs = law::handles_of(kind);
    let verdade = |h: &law::HandleDesc| law::handle_point(h, &values, half, &ck);
    let track = quente
        .and_then(|i| hs.get(i))
        .map(|h| law::track(kind, h, &values, half, doc(), &ck))
        .unwrap_or_default();
    let ring = agarrada
        .and_then(|i| hs.get(i))
        .and_then(|h| verdade(h).map(|at| face::radius_ring(h, at, half, face::RING_N)))
        .unwrap_or_default();
    let mut marks = Vec::new();
    let mut hot = None;
    for (i, (h, p)) in hs.iter().zip(&grips).enumerate() {
        let Some(at) = *p else { continue };
        if quente == Some(i) {
            hot = Some(marks.len());
        }
        marks.push(ph2d_vec_render::HandleMark {
            at,
            face: face::painted(face::glyph_of(h)),
            stem: law::is_arc_angle(h).then(|| verdade(h)).flatten(),
        });
    }
    ph2d_vec_render::draw_shape_handles(&marks, hot, &track, &ring, k, theme, scene);
}
