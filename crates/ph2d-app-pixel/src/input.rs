//! **A entrada do modo Draw**: ponteiro (lápis, borracha, conta-gotas), roda (zoom inteiro) e
//! teclado (os atalhos do Aseprite). Cada função devolve `true` quando CONSUMIU o evento — a shell
//! não o passa a mais ninguém.

use ph2d_host::WindowSize;
use ph2d_pixel_core::raster::Cell;
use ph2d_pixel_core::{Paint, Sample, Stroke, pick};
use ph2d_pixel_render::{cell_at, image_to_world, px_per_cell};
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, Point};
use winit::keyboard::KeyCode;

use crate::gesture::{Kind, Mods};
use crate::state::{Doc, Gesture};
use crate::{PixelApp, Tool};

/// O que a família precisa de ver do viewport.
#[derive(Clone, Copy)]
pub struct Canvas<'a> {
    pub world: &'a ph2d_ecs::World,
    pub camera: &'a Camera2d,
    pub window: WindowSize,
    /// Píxeis por metro do projecto (a régua que põe uma célula no mundo).
    pub ppm: f32,
    /// O objecto em Pixel Sprite ▸ Draw, se há.
    pub editing: Option<u64>,
}

/// O que um premir fez.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// Não é da família (fora de Draw).
    NotMine,
    /// Consumido: pintou, apagou ou leu uma cor.
    Done,
    /// A mão: a shell ancora o pan da câmera neste ponto.
    Pan,
}

/// O botão do ponteiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Primary,
    Secondary,
}

/// `pixel da imagem → ecrã` do objecto `bits` com o documento `doc`.
#[must_use]
pub fn to_screen(
    world: &ph2d_ecs::World,
    bits: u64,
    doc: &Doc,
    camera: &Camera2d,
    window: WindowSize,
    ppm: f32,
) -> Option<Affine> {
    let entity = ph2d_ecs::Entity::try_from_bits(bits)?;
    let tr = ph2d_ecs::world_transform(world, entity)?;
    let img = image_to_world(
        doc.sprite.width(),
        doc.sprite.height(),
        f64::from(ppm),
        (f64::from(tr.translation.x), f64::from(tr.translation.y)),
        f64::from(tr.rotation),
        (f64::from(tr.scale.x), f64::from(tr.scale.y)),
        (f64::from(tr.skew_x), f64::from(tr.skew_y)),
    );
    Some(camera.world_to_screen_affine(window) * img)
}

/// A chave do documento do objecto `bits`.
#[must_use]
pub fn key_of(world: &ph2d_ecs::World, bits: u64) -> Option<u32> {
    let e = ph2d_ecs::Entity::try_from_bits(bits)?;
    world.get::<ph2d_ecs::PixelSprite>(e).map(|p| p.0)
}

fn cell(app: &PixelApp, c: &Canvas<'_>, bits: u64, key: u32, p: (f32, f32)) -> Option<(i32, i32)> {
    let doc = app.docs.get(&key)?;
    let m = to_screen(c.world, bits, doc, c.camera, c.window, c.ppm)?;
    cell_at(m, (f64::from(p.0), f64::from(p.1)))
}

/// A distância ao eixo da simetria, em píxeis de ECRÃ, que ainda o agarra.
const AXIS_GRAB_PX: f64 = 4.0; // LITERAL-PX-OK: alcance do clique sobre uma linha de 1 px

/// O ponto de ecrã `p` em coordenadas da IMAGEM (células, contínuas).
fn image_point(m: Affine, p: (f32, f32)) -> Point {
    m.inverse() * Point::new(f64::from(p.0), f64::from(p.1))
}

/// O eixo da simetria sob o ponto `p`, se há (vertical = o do espelho H).
fn axis_under(o: &crate::Options, doc: &Doc, m: Affine, p: (f32, f32)) -> Option<bool> {
    if !(o.sym_h || o.sym_v) {
        return None;
    }
    let s = crate::gesture::symmetry(o, doc);
    let scale = px_per_cell(m);
    let q = image_point(m, p);
    let (w, h) = (
        f64::from(doc.sprite.width()),
        f64::from(doc.sprite.height()),
    );
    let near = |d: f64| d.abs() * scale <= AXIS_GRAB_PX;
    let inside = (-0.5..=w + 0.5).contains(&q.x) && (-0.5..=h + 0.5).contains(&q.y);
    if !inside {
        return None;
    }
    if o.sym_h && near(q.x - f64::from(s.axis2_x) / 2.0) {
        return Some(true);
    }
    if o.sym_v && near(q.y - f64::from(s.axis2_y) / 2.0) {
        return Some(false);
    }
    None
}

/// Premir no canvas. Em Draw, todo clique esquerdo/direito é do sprite (não selecciona outros
/// objectos): as ferramentas que pintam usam a cor de frente (esquerdo) ou de fundo (direito); a
/// borracha apaga (direito: troca a cor de frente pela de fundo); o conta-gotas (ou `Alt`) lê a cor
/// para a frente (esquerdo) ou o fundo (direito). A curva no 2.º tempo e o polígono continuam o
/// gesto que já existe.
pub fn press(app: &mut PixelApp, c: &Canvas<'_>, p: (f32, f32), button: Button, m: Mods) -> Press {
    let Some(bits) = c.editing else {
        return Press::NotMine;
    };
    let Some(key) = key_of(c.world, bits) else {
        return Press::NotMine;
    };
    if app.space || app.tool == Tool::Hand {
        app.panning = true;
        return Press::Pan;
    }
    let Some((x, y)) = cell(app, c, bits, key, p) else {
        return Press::Done;
    };
    if continue_pending(app, key, (x, y)) {
        return Press::Done;
    }
    let tool = if m.alt { Tool::Eyedropper } else { app.tool };
    let (fg, bg, options) = (app.fg, app.bg, app.options);
    let sel = app.sel.clone();
    let stamp = app.stamp();
    // O Aseprite recusa pintar numa camada trancada, escondida ou num grupo, e DIZ porquê.
    if tool.paints()
        && let Some(motivo) = app.docs.get(&key).and_then(crate::camadas::recusa_pintar)
    {
        app.avisos.push((true, ph2d_i18n::tr(motivo).to_owned()));
        return Press::Done;
    }
    let Some(doc) = app.docs.get_mut(&key) else {
        return Press::NotMine;
    };
    if tool.paints()
        && let Some(screen) = to_screen(c.world, bits, doc, c.camera, c.window, c.ppm)
        && let Some(vertical) = axis_under(&options, doc, screen, p)
    {
        app.gesture = Some(Gesture {
            entity: bits,
            key,
            stroke: Stroke::new(doc.layer, doc.frame, 0, Paint::default()),
            kind: Kind::Axis { vertical },
            held: true,
            cursor: (x, y),
        });
        return Press::Done;
    }
    let s = &doc.sprite;
    let (layer, frame) = (doc.layer, doc.frame);
    let mut paint = crate::gesture::paint(&options, doc);
    // A borracha pinta sempre com a Simple (Aseprite).
    if tool == Tool::Eraser {
        paint.ink = ph2d_pixel_core::ink::Ink::Simple;
    }
    let value = match button {
        Button::Primary => s.value_of_rgba(fg),
        Button::Secondary => s.value_of_rgba(bg),
    };
    let at = (x, y);
    let (stroke, kind) = match tool {
        Tool::Hand => return Press::Pan,
        Tool::Eyedropper => {
            if let Some(v) = pick(s, frame, x, y, Sample::AllLayers) {
                let rgba = s.rgba_of_value(v);
                match button {
                    Button::Primary => app.fg = rgba,
                    Button::Secondary => app.bg = rgba,
                }
            }
            return Press::Done;
        }
        Tool::Pencil | Tool::Eraser => {
            let mut st = match (tool, button) {
                (Tool::Eraser, Button::Primary) => {
                    let v = erase_value(s, layer, bg);
                    let from = if m.shift { doc.last_point } else { None };
                    Stroke::begin(s, layer, frame, v, paint, from.unwrap_or(at))
                }
                (Tool::Eraser, Button::Secondary) => {
                    let pair = (s.value_of_rgba(fg), s.value_of_rgba(bg));
                    let from = if m.shift { doc.last_point } else { None };
                    Stroke::begin_replacing(s, layer, frame, pair, paint, from.unwrap_or(at))
                }
                _ => {
                    let from = if m.shift { doc.last_point } else { None };
                    Stroke::begin(s, layer, frame, value, paint, from.unwrap_or(at))
                }
            };
            st.to(s, x, y);
            (st, Kind::Free)
        }
        Tool::Spray => {
            let mut st = Stroke::begin(s, layer, frame, value, paint, at);
            st.spray(s, at, options.spray_width, options.spray_speed);
            (st, Kind::Spray)
        }
        Tool::Bucket => {
            let mut st = Stroke::new(layer, frame, value, paint);
            st.fill(s, at, crate::gesture::bucket(&options));
            (st, Kind::Fill)
        }
        Tool::Line
        | Tool::Rectangle
        | Tool::FilledRectangle
        | Tool::Ellipse
        | Tool::FilledEllipse => {
            let kind = Kind::Box { from: at, to: at };
            let mut st = Stroke::new(layer, frame, value, paint);
            crate::gesture::reshape(&mut st, s, tool, &kind, &options, m);
            (st, kind)
        }
        Tool::Curve => {
            let kind = Kind::Curve {
                from: at,
                to: at,
                ctrl: None,
            };
            let mut st = Stroke::new(layer, frame, value, paint);
            crate::gesture::reshape(&mut st, s, tool, &kind, &options, m);
            (st, kind)
        }
        Tool::Contour => {
            let st = Stroke::begin(s, layer, frame, value, paint, at);
            (st, Kind::Contour(vec![at]))
        }
        Tool::Polygon => {
            let kind = Kind::Polygon(vec![at, at]);
            let mut st = Stroke::new(layer, frame, value, paint);
            crate::gesture::reshape(&mut st, s, tool, &kind, &options, m);
            (st, kind)
        }
    };
    // Shading: a rampa é a selecção da paleta; esquerdo anda +1, direito −1.
    let stroke = if paint.ink == ph2d_pixel_core::ink::Ink::Shading {
        let dir = if button == Button::Primary { 1 } else { -1 };
        stroke.with_shading(crate::paleta::rampa(s, &sel), dir)
    } else {
        stroke
    };
    doc.stamp = stamp;
    app.gesture = Some(Gesture {
        entity: bits,
        key,
        stroke,
        kind,
        held: true,
        cursor: at,
    });
    Press::Done
}

/// Um premir sobre um gesto que continua com o botão solto: a curva fixa o controlo (o largar
/// acaba-a); o polígono põe um vértice — sobre o PRIMEIRO, fecha. `true` = foi isso.
fn continue_pending(app: &mut PixelApp, key: u32, at: Cell) -> bool {
    let (tool, options) = (app.tool, app.options);
    let stamp = app.stamp();
    let Some(g) = app.gesture.as_mut().filter(|g| g.key == key) else {
        return false;
    };
    match &mut g.kind {
        Kind::Curve { ctrl, .. } if ctrl.is_some() => {
            *ctrl = Some(at);
        }
        Kind::Polygon(pts) => {
            let first = pts[0];
            let len = pts.len();
            if at == first && len > 3 {
                pts.pop();
            } else {
                pts[len - 1] = at;
                pts.push(at);
            }
        }
        _ => return false,
    }
    g.held = true;
    g.cursor = at;
    let closed = matches!(&g.kind, Kind::Polygon(pts) if pts.last() != Some(&at));
    if let Some(doc) = app.docs.get_mut(&key) {
        let kind = g.kind.clone();
        crate::gesture::reshape(
            &mut g.stroke,
            &doc.sprite,
            tool,
            &kind,
            &options,
            Mods::default(),
        );
        doc.stamp = stamp;
    }
    if closed {
        finish(app);
    }
    true
}

/// O que a borracha deixa: transparente; na camada Background, a cor de fundo (Aseprite).
fn erase_value(s: &ph2d_pixel_core::Sprite, layer: ph2d_pixel_core::LayerId, bg: [u8; 4]) -> u32 {
    if s.layer(layer).is_some_and(|l| l.background) {
        s.value_of_rgba(bg)
    } else {
        s.transparent_value()
    }
}

/// Mover. Com um gesto em curso, ele continua (consumido); sem gesto, só a célula sob o cursor.
pub fn moved(app: &mut PixelApp, c: &Canvas<'_>, p: (f32, f32), m: Mods) -> bool {
    let Some((bits, key)) = app.gesture.as_ref().map(|g| (g.entity, g.key)) else {
        app.hover = c.editing.and_then(|bits| {
            let key = key_of(c.world, bits)?;
            cell(app, c, bits, key, p).map(|xy| (bits, xy))
        });
        return false;
    };
    let Some((x, y)) = cell(app, c, bits, key, p) else {
        return true;
    };
    app.hover = Some((bits, (x, y)));
    let stamp = app.stamp();
    let (tool, options, space) = (app.tool, app.options, app.space);
    let (Some(doc), Some(g)) = (app.docs.get_mut(&key), app.gesture.as_mut()) else {
        return true;
    };
    let at = (x, y);
    let delta = (at.0 - g.cursor.0, at.1 - g.cursor.1);
    g.cursor = at;
    let s = &doc.sprite;
    match &mut g.kind {
        Kind::Free => g.stroke.to(s, x, y),
        Kind::Spray => {
            if g.held {
                g.stroke
                    .spray(s, at, options.spray_width, options.spray_speed);
            }
        }
        Kind::Fill => {}
        Kind::Axis { vertical } => {
            if let Some(screen) = to_screen(c.world, bits, doc, c.camera, c.window, c.ppm) {
                let q = image_point(screen, p);
                let mut s2 = crate::gesture::symmetry(&options, doc);
                let half = |v: f64, max: u32| ((v * 2.0).round() as i32).clamp(0, 2 * max as i32);
                if *vertical {
                    s2.axis2_x = half(q.x, doc.sprite.width());
                } else {
                    s2.axis2_y = half(q.y, doc.sprite.height());
                }
                doc.axis2 = Some((s2.axis2_x, s2.axis2_y));
            }
        }
        Kind::Box { from, to } => {
            if space {
                *from = (from.0 + delta.0, from.1 + delta.1);
                *to = (to.0 + delta.0, to.1 + delta.1);
            } else {
                *to = at;
            }
            let kind = g.kind.clone();
            crate::gesture::reshape(&mut g.stroke, s, tool, &kind, &options, m);
        }
        Kind::Curve { to, ctrl, .. } => {
            match ctrl {
                Some(c) => *c = at,
                None if g.held => *to = at,
                None => {}
            }
            let kind = g.kind.clone();
            crate::gesture::reshape(&mut g.stroke, s, tool, &kind, &options, m);
        }
        Kind::Contour(path) => {
            if path.last() != Some(&at) {
                path.push(at);
            }
            g.stroke.to(s, x, y);
        }
        Kind::Polygon(pts) => {
            if let Some(last) = pts.last_mut() {
                *last = at;
            }
            let kind = g.kind.clone();
            crate::gesture::reshape(&mut g.stroke, s, tool, &kind, &options, m);
        }
    }
    doc.stamp = stamp;
    true
}

/// Largar a mão: a câmera realinha as arestas das células a píxeis inteiros. `false` = não havia
/// arrasto da mão.
pub fn release_pan(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    camera: &mut Camera2d,
    window: WindowSize,
    ppm: f32,
    editing: Option<u64>,
) -> bool {
    if !std::mem::take(&mut app.panning) {
        return false;
    }
    if let Some(bits) = editing
        && let Some(doc) = key_of(world, bits).and_then(|k| app.docs.get(&k))
    {
        snap(world, bits, doc, camera, window, ppm);
    }
    true
}

/// Largar: o gesto acaba e vira UM passo do desfazer do sprite — salvo a curva no 1.º tempo
/// (passa ao controlo) e o polígono (espera o próximo vértice).
pub fn release(app: &mut PixelApp) -> bool {
    let Some(g) = app.gesture.as_mut() else {
        return false;
    };
    g.held = false;
    match &mut g.kind {
        Kind::Curve { from, to, ctrl } if ctrl.is_none() => {
            if from == to {
                app.gesture = None;
                return true;
            }
            *ctrl = Some(((from.0 + to.0) / 2, (from.1 + to.1) / 2));
            return true;
        }
        Kind::Polygon(_) => return true,
        Kind::Contour(path) => {
            let path = path.clone();
            if let Some(doc) = app.docs.get(&g.key) {
                let (outline, interior) = ph2d_pixel_core::raster::polygon_cells(&path);
                g.stroke.set_shape(&doc.sprite, &outline, &interior);
            }
        }
        _ => {}
    }
    finish(app);
    true
}

/// **Acaba o gesto em curso**: o traço vira UM passo do desfazer (nada mudou = nenhum passo).
pub(crate) fn finish(app: &mut PixelApp) {
    let Some(g) = app.gesture.take() else {
        return;
    };
    if matches!(g.kind, Kind::Axis { .. }) {
        return;
    }
    let free = matches!(g.kind, Kind::Free);
    let last = g.stroke.last();
    let op = app.docs.get(&g.key).and_then(|d| g.stroke.op(&d.sprite));
    match op {
        Some(op) => {
            let _ = app.commit(g.key, op);
        }
        None => {
            // Nada mudou: a imagem volta à do documento.
            let stamp = app.stamp();
            if let Some(d) = app.docs.get_mut(&g.key) {
                d.stamp = stamp;
            }
        }
    }
    if free && let Some(d) = app.docs.get_mut(&g.key) {
        d.last_point = Some(last);
    }
}

/// Troca a ferramenta: o gesto que esperava (a curva, o polígono) acaba como está.
pub(crate) fn set_tool(app: &mut PixelApp, t: Tool) {
    if app.tool != t {
        finish(app);
    }
    app.tool = t;
}

/// Larga o gesto sem gravar (`Esc`).
pub(crate) fn cancel(app: &mut PixelApp) -> bool {
    let Some(g) = app.gesture.take() else {
        return false;
    };
    let stamp = app.stamp();
    if let Some(d) = app.docs.get_mut(&g.key) {
        d.stamp = stamp;
    }
    true
}

#[path = "input_roda.rs"]
mod roda;
pub(crate) use roda::set_zoom;
use roda::snap;
pub use roda::{Notch, Roda, Turn, wheel};

/// A resposta ao teclado: consumida, e a mensagem a mostrar (se há).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAnswer {
    Quiet,
    Toast(String),
}

/// `Espaço` segurado/solto em Draw — a mão temporária. `true` = consumido.
pub fn space(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    editing: Option<u64>,
    held: bool,
) -> bool {
    if editing.and_then(|b| key_of(world, b)).is_none() {
        app.space = false;
        return false;
    }
    app.space = held;
    true
}

/// As teclas do modo Draw (só ao premir; a shell já descartou o caso de um campo de texto com o
/// foco). `None` = não é desta família.
pub fn key(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    editing: Option<u64>,
    code: KeyCode,
    ctrl: bool,
    shift: bool,
) -> Option<KeyAnswer> {
    let bits = editing?;
    let key = key_of(world, bits)?;
    let tool = |app: &mut PixelApp, t: Tool| {
        set_tool(app, t);
        KeyAnswer::Toast(ph2d_i18n::tr_with(
            "pixel.tool_switched",
            &[
                ("tool", &ph2d_i18n::tr(t.label_key())),
                ("key", &t.shortcut_label()),
            ],
        ))
    };
    Some(match (code, ctrl, shift) {
        (KeyCode::KeyB, false, false) => tool(app, Tool::Pencil),
        (KeyCode::KeyB, false, true) => tool(app, Tool::Spray),
        (KeyCode::KeyE, false, false) => tool(app, Tool::Eraser),
        (KeyCode::KeyI, false, false) => tool(app, Tool::Eyedropper),
        (KeyCode::KeyG, false, false) => tool(app, Tool::Bucket),
        (KeyCode::KeyL, false, false) => tool(app, Tool::Line),
        (KeyCode::KeyL, false, true) => tool(app, Tool::Curve),
        // A mesma tecla de novo alterna a variante cheia (Aseprite).
        (KeyCode::KeyU, false, false) => {
            let t = if app.tool == Tool::Rectangle {
                Tool::FilledRectangle
            } else {
                Tool::Rectangle
            };
            tool(app, t)
        }
        (KeyCode::KeyU, false, true) => {
            let t = if app.tool == Tool::Ellipse {
                Tool::FilledEllipse
            } else {
                Tool::Ellipse
            };
            tool(app, t)
        }
        (KeyCode::KeyD, false, false) => tool(app, Tool::Contour),
        (KeyCode::KeyD, false, true) => tool(app, Tool::Polygon),
        (KeyCode::KeyH, false, false) => tool(app, Tool::Hand),
        (KeyCode::Enter | KeyCode::NumpadEnter, false, _) if app.stroking() => {
            finish(app);
            KeyAnswer::Quiet
        }
        (KeyCode::Escape, _, _) if app.stroking() => {
            cancel(app);
            KeyAnswer::Quiet
        }
        (KeyCode::KeyX, false, false) => {
            std::mem::swap(&mut app.fg, &mut app.bg);
            KeyAnswer::Quiet
        }
        (KeyCode::Quote, true, true) => {
            app.grid = !app.grid;
            KeyAnswer::Toast(
                ph2d_i18n::tr(if app.grid {
                    "pixel.grid_on"
                } else {
                    "pixel.grid_off"
                })
                .to_owned(),
            )
        }
        (KeyCode::KeyZ, true, false) => {
            if app.stroking() || app.undo(key) {
                KeyAnswer::Quiet
            } else {
                KeyAnswer::Toast(ph2d_i18n::tr("pixel.nothing_to_undo").to_owned())
            }
        }
        (KeyCode::KeyZ, true, true) | (KeyCode::KeyY, true, false) => {
            if app.stroking() || app.redo(key) {
                KeyAnswer::Quiet
            } else {
                KeyAnswer::Toast(ph2d_i18n::tr("pixel.nothing_to_redo").to_owned())
            }
        }
        _ => return None,
    })
}
