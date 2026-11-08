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
    let key = editing.and_then(|bits| key_of(world, bits))?;
    let mut chord = Chord::new(code);
    (chord.ctrl, chord.alt, chord.shift, chord.space) = (m.ctrl, m.alt, m.shift, app.space);
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
            KeyAnswer::Toast(Tool::Transform.named())
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
        A::PixelGrid => {
            app.grid = !app.grid;
            let k = if app.grid {
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
            ph2d_panel_pixel_timeline::pede_rename_layer();
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

fn ferramenta(app: &mut PixelApp, t: Tool) -> KeyAnswer {
    set_tool(app, t);
    KeyAnswer::Toast(t.named())
}
