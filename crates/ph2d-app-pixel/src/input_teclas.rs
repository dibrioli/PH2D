//! **As teclas do Draw** (de [`crate::input`]): a tabela `ph2d_pixel_keys` diz a acção pelo
//! contexto (`docs/PixelArt/P12_teclas.md`); aqui só o que precisa do estado.

use ph2d_editor_core::pixel_panel::PixelRequest;
use ph2d_pixel_keys::{Chord, Context, PixelAction as A};
use ph2d_pixel_tiles::desenho::Modo;
use winit::keyboard::KeyCode;

use crate::gesture::Mods;
use crate::input::{KeyAnswer, cancel, finish, key_of, set_tool};
use crate::{PixelApp, Tool};

/// Com a caixa do Text aberta a tecla é do texto (`text` = o que ela escreveu); `None` = não há
/// caixa no sprite em Draw e a tecla segue para os atalhos.
pub fn text_key(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    editing: Option<u64>,
    code: KeyCode,
    m: Mods,
    text: Option<&str>,
) -> Option<KeyAnswer> {
    editing.and_then(|bits| key_of(world, bits))?;
    crate::texto::tecla(app, code, m, text)
}

/// [`key_with`] sem o Alt.
pub fn key(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    editing: Option<u64>,
    code: KeyCode,
    ctrl: bool,
    shift: bool,
) -> Option<KeyAnswer> {
    let m = Mods {
        alt: false,
        ctrl,
        shift,
    };
    key_with(app, world, editing, code, m)
}

/// O contexto da tabela para o sprite `key` (o laço poligonal aberto é um gesto a meio).
pub(crate) fn contexto(app: &PixelApp, key: u32) -> Context {
    Context::of(
        app.stroking() || app.laco.as_ref().is_some_and(|l| l.key == key),
        app.solta.as_ref().is_some_and(|s| s.key == key),
        app.tool == Tool::Slice && !app.slices_selected(key).is_empty(),
        crate::selecao::tem(app, key),
    )
}

/// A tecla premida em Draw. `None` = não é do Pixel (as teclas do app nunca o são, §9.7).
pub fn key_with(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    editing: Option<u64>,
    code: KeyCode,
    m: Mods,
) -> Option<KeyAnswer> {
    let mut chord = Chord::new(code);
    (chord.ctrl, chord.alt, chord.shift, chord.space) = (m.ctrl, m.alt, m.shift, app.space);
    if ph2d_panel_pixel_keys::oferece(chord) {
        return Some(KeyAnswer::Quiet);
    }
    let key = editing.and_then(|bits| key_of(world, bits))?;
    if ph2d_pixel_keys::reserved(chord).is_some() {
        return None;
    }
    if code == KeyCode::Escape && !app.stroking() && crate::slices::esc_do_menu(app) {
        return Some(KeyAnswer::Quiet);
    }
    let a = app.keys.resolve(chord, contexto(app, key))?;
    acao(app, key, a)
}

/// Faz a acção `a` no sprite `key`. `None` = não há nada a fazer (a tecla passa).
pub(crate) fn acao(app: &mut PixelApp, key: u32, a: A) -> Option<KeyAnswer> {
    let tiles = |app: &mut PixelApp, m: Modo| crate::tiles::tecla(app, |h| h.modo = m);
    Some(match a {
        A::ToolBlurJumble => ferramenta(
            app,
            if app.tool == Tool::Blur {
                Tool::Jumble
            } else {
                Tool::Blur
            },
        ),
        A::ToolTransform => {
            app.pede(PixelRequest::Tool(Tool::Transform));
            KeyAnswer::Toast(Tool::Transform.named(&ph2d_pixel_keys::tool_keys(&app.keys)))
        }
        A::FinishStroke if app.laco.is_some() => {
            crate::selecao::fecha_poligono(app);
            KeyAnswer::Quiet
        }
        A::FinishStroke => {
            finish(app);
            KeyAnswer::Quiet
        }
        A::CancelStroke if app.stroking() => {
            if !crate::viva::escape(app) {
                cancel(app);
            }
            KeyAnswer::Quiet
        }
        A::CancelStroke | A::CancelTransform | A::ClearSelection => crate::selecao::esc(app, key)?,
        A::DeleteSlices | A::DeselectSlices => {
            crate::slices::tecla(app, key, a == A::DeleteSlices)?
        }
        A::Undo | A::DropFloating if crate::selecao::larga(app, key) => KeyAnswer::Quiet,
        A::Undo | A::DropFloating => {
            if crate::viva::desfaz(app) || app.stroking() || app.undo(key) {
                KeyAnswer::Quiet
            } else {
                KeyAnswer::Toast(ph2d_i18n::tr("pixel.nothing_to_undo").to_owned())
            }
        }
        A::Redo => {
            if crate::viva::refaz(app) || app.stroking() || app.redo(key) {
                KeyAnswer::Quiet
            } else {
                KeyAnswer::Toast(ph2d_i18n::tr("pixel.nothing_to_redo").to_owned())
            }
        }
        _ if a.zoom().is_some() => KeyAnswer::Zoom(a.zoom()?),
        A::BrushSmaller | A::BrushBigger => {
            let mut o = app.options;
            o.size = if a == A::BrushBigger {
                o.size + 1
            } else {
                o.size.saturating_sub(1)
            }
            .clamp(1, ph2d_editor_core::pixel_panel::PIXEL_MAX_BRUSH);
            app.pede(PixelRequest::Options(Box::new(o)));
            KeyAnswer::Quiet
        }
        A::ColorPrevious | A::ColorNext => {
            let pal = app.docs.get(&key)?.sprite.palette();
            let n = pal.len();
            let i = (app.ultima.filter(|&i| pal.get(i) == Some(&app.fg)))
                .or_else(|| pal.iter().position(|c| *c == app.fg));
            if n > 0 {
                let index = match (i, a == A::ColorNext) {
                    (Some(i), true) => (i + 1) % n,
                    (Some(i), false) => (i + n - 1) % n,
                    (None, _) => 0,
                };
                let how = ph2d_editor_core::pixel_panel::PixelSelect::Only;
                app.pede(PixelRequest::Swatch { index, how });
            }
            KeyAnswer::Quiet
        }
        A::LayerAbove | A::LayerBelow => {
            let doc = app.docs.get(&key)?;
            let ls = doc.sprite.layers();
            let i = ls.iter().position(|l| l.id == doc.layer).unwrap_or(0);
            let j = if a == A::LayerAbove {
                (i + 1).min(ls.len().saturating_sub(1))
            } else {
                i.saturating_sub(1)
            };
            if let Some(l) = ls.get(j).filter(|_| j != i) {
                let id = l.id.0;
                app.pede(PixelRequest::LayerSelect(id));
            }
            KeyAnswer::Quiet
        }
        A::FramePreviousInTag | A::FrameNextInTag => {
            let doc = app.docs.get_mut(&key)?;
            let s = &doc.sprite;
            let (i, n) = (s.frame_index(doc.frame).unwrap_or(0), s.frames().len());
            let (de, ate) = ph2d_pixel_core::play::tag_at(s.tags(), i)
                .and_then(|t| s.tags().get(t))
                .map_or((0, n.saturating_sub(1)), |t| {
                    (t.from as usize, t.to as usize)
                });
            let j = match (a == A::FrameNextInTag, i) {
                (true, i) if i >= ate => de,
                (true, i) => i + 1,
                (false, i) if i <= de => ate,
                (false, i) => i - 1,
            };
            if let Some(f) = s.frames().get(j) {
                doc.frame = f.id;
                doc.play = None;
            }
            KeyAnswer::Quiet
        }
        A::PixelGrid | A::ShowGrid => {
            let mut on = false;
            app.muda_prefs(key, |p| {
                let v = if a == A::ShowGrid {
                    &mut p.grid.visible
                } else {
                    &mut p.pixel_grid.visible
                };
                (*v, on) = (!*v, !*v);
            });
            let k = if on {
                "pixel.grid_on"
            } else {
                "pixel.grid_off"
            };
            KeyAnswer::Toast(ph2d_i18n::tr(k).to_owned())
        }
        A::LayerVisibility => {
            let id = app.docs.get(&key)?.layer.0;
            app.pede(PixelRequest::LayerVisible(id));
            KeyAnswer::Quiet
        }
        A::LayerRename => {
            // Com a Timeline fechada ela abre: o campo do nome é dela (o que o artista vê).
            app.mostra_timeline = true;
            ph2d_panel_pixel_timeline::pede_rename_layer();
            KeyAnswer::Quiet
        }
        A::KeyboardShortcuts => {
            ph2d_panel_pixel_keys::pede_abrir();
            KeyAnswer::Quiet
        }
        A::GridSettings => {
            ph2d_panel_pixel_prefs::canal::pede_abrir();
            KeyAnswer::Quiet
        }
        A::SwapColors => {
            let bg = app.bg;
            app.bg = app.fg;
            app.escolhe_fg(bg);
            KeyAnswer::Quiet
        }
        A::Timeline => {
            app.alterna_timeline = true;
            KeyAnswer::Quiet
        }
        A::TileModeToggle => crate::tiles::tecla(app, |h| h.alterna_alvo()),
        A::TileManual => tiles(app, Modo::Manual),
        A::TileAuto => tiles(app, Modo::Auto),
        A::TileStack => tiles(app, Modo::Stack),
        _ if a.tool().is_some() => ferramenta(app, a.tool()?),
        _ => {
            app.pede(a.request()?);
            KeyAnswer::Quiet
        }
    })
}

/// O Import/Export da janela de atalhos pelo diálogo de ficheiro do app (a porta cronometrada).
pub(crate) fn troca_de_teclas() {
    use ph2d_panel_pixel_keys::Troca;
    let Some(t) = ph2d_panel_pixel_keys::toma_troca() else {
        return;
    };
    let d = rfd::FileDialog::new().add_filter(ph2d_i18n::tr("pixel.keys.file_kind"), &["txt"]);
    let p = match t {
        Troca::Export => ph2d_app_host::modal::save_file(d.set_file_name("pixel-keys.txt")),
        Troca::Import => ph2d_app_host::modal::pick_file(d),
    };
    if let Some(p) = p {
        ph2d_panel_pixel_keys::entrega_troca(t, p);
    }
}

fn ferramenta(app: &mut PixelApp, t: Tool) -> KeyAnswer {
    set_tool(app, t);
    KeyAnswer::Toast(t.named(&ph2d_pixel_keys::tool_keys(&app.keys)))
}
