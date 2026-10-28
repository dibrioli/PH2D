//! **As teclas do Draw** (filho de [`super`]): os atalhos do Aseprite, com o Alt nos que o levam.

use ph2d_editor_core::pixel_panel::{
    PixelFrameAction, PixelLayerAction, PixelRequest, PixelSpriteAction,
};
use winit::keyboard::KeyCode;

use super::{KeyAnswer, cancel, finish, key_of, set_tool};
use crate::gesture::Mods;
use crate::{PixelApp, Tool};

/// As teclas do modo Draw (só ao premir; a shell já descartou o caso de um campo de texto com o
/// foco). `None` = não é desta família.
fn camada(app: &mut PixelApp, a: PixelLayerAction) -> KeyAnswer {
    app.pede(PixelRequest::LayerAction(a));
    KeyAnswer::Quiet
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

/// As teclas com os modificadores inteiros: com o Alt só os atalhos do Aseprite que o levam.
pub fn key_with(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    editing: Option<u64>,
    code: KeyCode,
    m: Mods,
) -> Option<KeyAnswer> {
    let (ctrl, shift) = (m.ctrl, m.shift);
    if m.alt {
        editing.and_then(|bits| key_of(world, bits))?;
        return com_alt(app, code, ctrl, shift);
    }
    let bits = editing?;
    let key = key_of(world, bits)?;
    if !app.stroking()
        && let Some(a) = crate::selecao::key(app, key, code, ctrl, shift)
    {
        return Some(a);
    }
    let tool = |app: &mut PixelApp, t: Tool| {
        set_tool(app, t);
        KeyAnswer::Toast(t.named())
    };
    Some(match (code, ctrl, shift) {
        (KeyCode::KeyB, false, false) => tool(app, Tool::Pencil),
        (KeyCode::KeyB, false, true) => tool(app, Tool::Spray),
        (KeyCode::KeyE, false, false) => tool(app, Tool::Eraser),
        (KeyCode::KeyI, false, false) => tool(app, Tool::Eyedropper),
        (KeyCode::KeyG, false, false) => tool(app, Tool::Bucket),
        (KeyCode::KeyL, false, false) => tool(app, Tool::Line),
        (KeyCode::KeyL, false, true) => tool(app, Tool::Curve),
        // Cheio ou vazio é a caixa «Fill» do painel (P6c), não outra ferramenta.
        (KeyCode::KeyU, false, false) => tool(app, Tool::RECTANGLE),
        (KeyCode::KeyU, false, true) => tool(app, Tool::ELLIPSE),
        (KeyCode::KeyD, false, false) => tool(app, Tool::Contour),
        (KeyCode::KeyD, false, true) => tool(app, Tool::Polygon),
        (KeyCode::KeyH, false, false) => tool(app, Tool::Hand),
        (KeyCode::Enter | KeyCode::NumpadEnter, false, _) if app.stroking() => {
            finish(app);
            KeyAnswer::Quiet
        }
        (KeyCode::Escape, _, _) if app.stroking() => {
            if !crate::viva::escape(app) {
                cancel(app);
            }
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
        // O menu Layer do Aseprite: Shift+N, Ctrl+J, Ctrl+Shift+J, Shift+X (com Alt: `com_alt`).
        // Select/Edit/Sprite: `crate::selecao::key`.
        (KeyCode::KeyN, false, true) => camada(app, PixelLayerAction::NewLayer),
        (KeyCode::KeyJ, true, false) => camada(app, PixelLayerAction::ViaCopy),
        (KeyCode::KeyJ, true, true) => camada(app, PixelLayerAction::ViaCut),
        (KeyCode::KeyX, false, true) => {
            let id = app.docs.get(&key).map(|d| d.layer.0);
            if let Some(id) = id {
                app.pede(PixelRequest::LayerVisible(id));
            }
            KeyAnswer::Quiet
        }
        (KeyCode::KeyZ, true, false) => {
            if crate::viva::desfaz(app) || app.stroking() || app.undo(key) {
                KeyAnswer::Quiet
            } else {
                KeyAnswer::Toast(ph2d_i18n::tr("pixel.nothing_to_undo").to_owned())
            }
        }
        (KeyCode::KeyZ, true, true) | (KeyCode::KeyY, true, false) => {
            if crate::viva::refaz(app) || app.stroking() || app.redo(key) {
                KeyAnswer::Quiet
            } else {
                KeyAnswer::Toast(ph2d_i18n::tr("pixel.nothing_to_redo").to_owned())
            }
        }
        (KeyCode::ArrowLeft, false, false) => quadro(app, PixelFrameAction::Previous),
        (KeyCode::ArrowRight, false, false) => quadro(app, PixelFrameAction::Next),
        (KeyCode::Home, false, false) => quadro(app, PixelFrameAction::First),
        (KeyCode::End, false, false) => quadro(app, PixelFrameAction::Last),
        (KeyCode::Enter | KeyCode::NumpadEnter, false, false) => {
            quadro(app, PixelFrameAction::Play)
        }
        (KeyCode::F3, false, false) => quadro(app, PixelFrameAction::Onion),
        (KeyCode::Tab, false, false) => {
            app.alterna_timeline = true;
            KeyAnswer::Quiet
        }
        _ => return None,
    })
}

/// Os atalhos do Aseprite com Alt (menus Sprite, Layer e Frame); a gesto a meio, nenhum.
fn com_alt(app: &mut PixelApp, code: KeyCode, ctrl: bool, shift: bool) -> Option<KeyAnswer> {
    if app.stroking() {
        return None;
    }
    Some(match (code, ctrl, shift) {
        (KeyCode::KeyI, true, false) => {
            app.pede(PixelRequest::Sprite(PixelSpriteAction::SpriteSize));
            KeyAnswer::Quiet
        }
        (KeyCode::KeyN, false, true) => camada(app, PixelLayerAction::NewGroup),
        (KeyCode::KeyN, false, false) => quadro(app, PixelFrameAction::NewFrame),
        (KeyCode::KeyB, false, false) => quadro(app, PixelFrameAction::NewEmptyFrame),
        (KeyCode::KeyM, false, false) => quadro(app, PixelFrameAction::DuplicateLinked),
        (KeyCode::KeyC, false, false) => quadro(app, PixelFrameAction::Delete),
        (KeyCode::KeyI, false, false) => quadro(app, PixelFrameAction::Reverse),
        _ => return None,
    })
}

fn quadro(app: &mut PixelApp, a: PixelFrameAction) -> KeyAnswer {
    app.pede(PixelRequest::FrameAction(a));
    KeyAnswer::Quiet
}
