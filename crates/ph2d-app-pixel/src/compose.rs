//! **A porta da shell** (catraca `the_shell_only_shrinks`): o ponteiro, a roda e o teclado chegam
//! como os eventos crus da janela, com a [`View`] do quadro, e a família decide aqui. A shell fica
//! com uma chamada por evento.

use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetStore};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_host::{PointerButton, PointerKind, WindowSize};
use ph2d_render::Camera2d;
use winit::event::MouseScrollDelta;
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::PixelApp;
pub use crate::gesture::Mods;
use crate::input::{self, Button, Canvas, KeyAnswer, Press};

/// O viewport deste quadro, como a shell o tem: a câmera e os avisos emprestados.
pub struct View<'a> {
    pub world: &'a ph2d_ecs::World,
    pub camera: &'a mut Camera2d,
    pub toasts: &'a mut ToastQueue,
    pub window: WindowSize,
    pub ppm: f32,
    /// O objecto em Pixel Sprite ▸ Draw, se há.
    pub editing: Option<u64>,
}

impl<'a> View<'a> {
    /// A vista lida do `hero` (régua do projecto, objecto em Draw) e da janela da CENA.
    #[must_use]
    pub fn of(
        world: &'a ph2d_ecs::World,
        hero: &HeroScreen,
        camera: &'a mut Camera2d,
        toasts: &'a mut ToastQueue,
        window: WindowSize,
    ) -> Self {
        let editing = crate::mode::editing(world, hero);
        let ppm = hero.project.pixels_per_meter;
        Self {
            world,
            camera,
            toasts,
            window,
            ppm,
            editing,
        }
    }

    fn canvas(&self) -> Canvas<'_> {
        Canvas {
            world: self.world,
            camera: self.camera,
            window: self.window,
            ppm: self.ppm,
            editing: self.editing,
        }
    }
}

/// O que um premir/largar fez — a shell só guarda a âncora do pan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pointer {
    NotMine,
    Done,
    /// A mão: a shell ancora o pan da câmera no ponteiro.
    PanStart,
    /// Largou a mão (a câmera já realinhou as células): a shell larga a âncora.
    PanEnd,
}

impl PixelApp {
    /// Premir/largar. `free` = o ponteiro está no canvas e nenhum menu estava aberto ao premir.
    pub fn pointer(
        &mut self,
        v: &mut View<'_>,
        kind: PointerKind,
        button: PointerButton,
        p: (f32, f32),
        mods: Mods,
        free: bool,
    ) -> Pointer {
        if kind == PointerKind::Up {
            if input::release_pan(self, v.world, v.camera, v.window, v.ppm, v.editing) {
                return Pointer::PanEnd;
            }
            return if input::release(self) {
                Pointer::Done
            } else {
                Pointer::NotMine
            };
        }
        let button = match button {
            PointerButton::Primary => Button::Primary,
            PointerButton::Secondary => Button::Secondary,
            _ => return Pointer::NotMine,
        };
        if kind != PointerKind::Down || !free {
            return Pointer::NotMine;
        }
        match input::press(self, &v.canvas(), p, button, mods) {
            Press::Pan => Pointer::PanStart,
            Press::Done => Pointer::Done,
            Press::NotMine => Pointer::NotMine,
        }
    }

    /// Mover: o traço em curso continua (consumido); sem traço, só a célula sob o cursor.
    pub fn pointer_moved(&mut self, v: &View<'_>, p: (f32, f32), mods: Mods) -> bool {
        input::moved(self, &v.canvas(), p, mods)
    }

    /// A roda sobre o canvas (`on_canvas`) em Draw: zoom inteiro, sem aviso (o degrau lê-se no
    /// rodapé, [`crate::panel::publish_bars`]).
    pub fn wheel_event(
        &mut self,
        v: &mut View<'_>,
        delta: MouseScrollDelta,
        p: (f32, f32),
        on_canvas: bool,
    ) -> bool {
        let turn = match delta {
            MouseScrollDelta::LineDelta(_, y) => input::Turn::Lines(y),
            MouseScrollDelta::PixelDelta(d) => input::Turn::Pixels(d.y as f32),
        };
        if matches!(turn, input::Turn::Lines(0.0) | input::Turn::Pixels(0.0)) || !on_canvas {
            return false;
        }
        let notch = input::Notch { at: p, turn };
        input::wheel(self, v.world, v.camera, v.window, v.ppm, v.editing, notch)
    }

    /// As teclas do Draw (B, E, I, H, X, Espaço, Ctrl+Z…). A shell já descartou o campo de texto
    /// com o foco. `true` = consumida.
    pub fn keyboard(
        &mut self,
        v: &mut View<'_>,
        key: PhysicalKey,
        pressed: bool,
        ctrl: bool,
        shift: bool,
    ) -> bool {
        let PhysicalKey::Code(code) = key else {
            return false;
        };
        if diag() {
            eprintln!(
                "[pixel-diag] tecla {code:?} premida={pressed} ctrl={ctrl} shift={shift} draw={} gesto={}",
                v.editing.is_some(),
                self.stroking()
            );
        }
        if code == KeyCode::Space {
            return input::space(self, v.world, v.editing, pressed);
        }
        if !pressed {
            return false;
        }
        let Some(answer) = input::key(self, v.world, v.editing, code, ctrl, shift) else {
            return false;
        };
        if let KeyAnswer::Toast(t) = answer {
            v.toasts.push(ph2d_editor_core::Toast::info(t));
        }
        true
    }
}

impl PixelApp {
    /// O [`PixelApp::frame_in`] com a régua, o Draw e a área livre do desenho lidos do `hero`.
    pub fn frame_of(
        &mut self,
        sim: &mut ph2d_ecs::SimWorld,
        camera: &mut Camera2d,
        window: WindowSize,
        hero: &HeroScreen,
        spawn_root: impl FnOnce(&mut ph2d_ecs::SimWorld, &str) -> u64,
    ) {
        let editing = crate::mode::editing(sim.world(), hero);
        let ppm = hero.project.pixels_per_meter;
        // O relógio do tocar: os ms reais desde o quadro anterior (o núcleo não lê relógio).
        let now = std::time::Instant::now();
        let dt = self.relogio.replace(now).map_or(0, |t| {
            u32::try_from(now.duration_since(t).as_millis()).unwrap_or(u32::MAX)
        });
        self.tick(dt);
        let whole =
            ph2d_editor_core::zones::Rect::new(0.0, 0.0, window.width as f32, window.height as f32);
        let (r, _) = ph2d_app_host::canvas_area::frame_target(hero, whole);
        let area = ph2d_render::camera_frame::Area {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
        };
        self.frame_in(sim, camera, window, ppm, editing, area, spawn_root);
    }

    /// O [`crate::draw::draw`] com a régua, o Draw e o tema lidos do `hero`, e por cima a etiqueta
    /// da forma viva ([`crate::viva::readout`]).
    pub fn draw_of(
        &mut self,
        world: &ph2d_ecs::World,
        camera: &Camera2d,
        window: WindowSize,
        hero: &HeroScreen,
        (scene, text): (&mut ph2d_vector::VectorScene, &mut ph2d_text::TextSystem),
    ) {
        let (ppm, editing) = (
            hero.project.pixels_per_meter,
            crate::mode::editing(world, hero),
        );
        crate::draw::draw(self, world, camera, window, ppm, editing, hero.theme, scene);
        let c = crate::input::Canvas {
            world,
            camera,
            window,
            ppm,
            editing,
        };
        crate::viva::ficha(self, &c, hero.last_content, hero.theme, (scene, text));
    }
}

/// `PH2D_PIXEL_DIAG=1`: cada tecla que chega à família vai para o stderr (report do dono 25/10: o
/// `Esc` não cancelava o polígono na máquina dele e cancelava pelo `key_input` inteiro aqui).
fn diag() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_PIXEL_DIAG").is_some())
}

/// Um diálogo do Pixel Art (*New Sprite*, *Color Mode*) está aberto — MODAL: toda tecla vai só para
/// ele.
#[must_use]
pub fn dialog_open(store: &WidgetStore) -> bool {
    store.context_menu().is_some_and(|r| {
        matches!(
            r.kind,
            ContextMenuKind::NewSpriteDialog
                | ContextMenuKind::PixelColorModeDialog
                | ContextMenuKind::PixelEditDialog
        )
    })
}

/// Os caracteres do texto digitado que vão para o campo do diálogo (sem controlo, sem os atalhos
/// `Cmd`/`Ctrl` + letra).
pub fn dialog_text(text: Option<&str>, cmd: bool) -> impl Iterator<Item = char> + '_ {
    text.unwrap_or_default()
        .chars()
        .filter(move |ch| !ch.is_control() && !(cmd && ch.is_ascii_alphabetic()))
}
