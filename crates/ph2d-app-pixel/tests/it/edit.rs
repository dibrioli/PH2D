//! **O modo Draw pelo caminho real**: o objecto nasce pelo Add, o ponteiro entra pelas mesmas
//! funções que a shell chama, e o efeito lê-se no documento, no desfazer e no blob.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Canvas, KeyAnswer, Press};
use ph2d_app_pixel::{PixelApp, Tool, mode, object_add, persist};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor_core::ToolRegistry;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::Point;
use winit::keyboard::KeyCode;

pub(crate) const WINDOW: WindowSize = WindowSize {
    width: 800,
    height: 600,
};
pub(crate) const PPM: f32 = 100.0;

const ALT: Mods = Mods {
    alt: true,
    shift: false,
    ctrl: false,
};

/// O vazio da casa (`spawn_empty_root`): nome e identidade estável.
pub(crate) fn spawn_root(sim: &mut SimWorld, name: &str) -> u64 {
    let bits = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name)))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    bits
}

/// Um Pixel Sprite nascido pelo Add, com a câmera já enquadrada pela entrada em Edit.
pub(crate) fn born() -> (PixelApp, SimWorld, Camera2d, u64) {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    let bits = object_add::add(object_add::PIXEL_SPRITE, &mut sim, &mut app, spawn_root)
        .expect("é desta família")
        .expect("nasce");
    let mut camera = Camera2d::default();
    app.frame(&mut sim, &mut camera, WINDOW, PPM, Some(bits), spawn_root);
    (app, sim, camera, bits)
}

pub(crate) fn canvas<'a>(sim: &'a SimWorld, camera: &'a Camera2d, bits: u64) -> Canvas<'a> {
    Canvas {
        world: sim.world(),
        camera,
        window: WINDOW,
        ppm: PPM,
        editing: Some(bits),
    }
}

/// O ponto de ecrã do CENTRO da célula `(x, y)`.
pub(crate) fn at(
    app: &PixelApp,
    sim: &SimWorld,
    camera: &Camera2d,
    bits: u64,
    x: i32,
    y: i32,
) -> (f32, f32) {
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let m = input::to_screen(
        sim.world(),
        bits,
        app.doc(key).expect("doc"),
        camera,
        WINDOW,
        PPM,
    )
    .expect("afim");
    let p = m * Point::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
    (p.x as f32, p.y as f32)
}

pub(crate) fn value(app: &PixelApp, sim: &SimWorld, bits: u64, x: i32, y: i32) -> [u8; 4] {
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let d = app.doc(key).expect("doc");
    d.sprite
        .rgba_of_value(d.sprite.value_at((d.layer, d.frame), x, y))
}

fn drag(
    app: &mut PixelApp,
    sim: &SimWorld,
    cam: &Camera2d,
    bits: u64,
    cells: &[(i32, i32)],
    b: Button,
) {
    let p0 = at(app, sim, cam, bits, cells[0].0, cells[0].1);
    assert_eq!(
        input::press(app, &canvas(sim, cam, bits), p0, b, Mods::default()),
        Press::Done,
        "o premir não foi do sprite"
    );
    for &(x, y) in &cells[1..] {
        let p = at(app, sim, cam, bits, x, y);
        assert!(input::moved(
            app,
            &canvas(sim, cam, bits),
            p,
            Mods::default()
        ));
    }
    assert!(input::release(app));
}

#[test]
fn add_creates_a_32_by_32_sprite_that_asks_for_edit_once() {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    let bits = object_add::add(object_add::PIXEL_SPRITE, &mut sim, &mut app, spawn_root)
        .expect("família")
        .expect("nasce");
    let key = input::key_of(sim.world(), bits).expect("o objecto tem o marcador");
    let d = app.doc(key).expect("documento");
    assert_eq!((d.sprite.width(), d.sprite.height()), (32, 32));
    let mut tools = ToolRegistry::default();
    let mut fam = mode::Family::new(&mut app, sim.world());
    assert_eq!(fam.wants(&mut tools), Some((bits, ObjectMode::Draw)));
    assert_eq!(fam.wants(&mut tools), None, "o pedido é UM");
    assert!(fam.holds(ObjectMode::Draw, bits, &mut tools));
}

#[test]
fn entering_edit_frames_the_sprite_at_an_integer_zoom_with_cells_on_whole_pixels() {
    let (app, sim, camera, bits) = born();
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let m = input::to_screen(
        sim.world(),
        bits,
        app.doc(key).expect("doc"),
        &camera,
        WINDOW,
        PPM,
    )
    .expect("afim");
    let ppc = ph2d_pixel_render::px_per_cell(m);
    assert!(
        (ppc - ppc.round()).abs() < 1e-3,
        "{ppc} px por célula não é inteiro"
    );
    assert_eq!(
        ppc.round() as u32,
        ph2d_pixel_render::zoom::fit(32, 32, 480.0, 360.0)
    );
    let p0 = m * Point::new(0.0, 0.0);
    assert!(
        (p0.x - p0.x.round()).abs() < 1e-2 && (p0.y - p0.y.round()).abs() < 1e-2,
        "{p0:?}"
    );
}

#[test]
fn a_drag_paints_a_line_as_one_undo_step_and_ctrl_z_takes_it_back() {
    let (mut app, sim, cam, bits) = born();
    app.fg = [0xac, 0x32, 0x32, 0xff];
    drag(
        &mut app,
        &sim,
        &cam,
        bits,
        &[(2, 2), (12, 7), (20, 20)],
        Button::Primary,
    );
    assert_eq!(value(&app, &sim, bits, 2, 2), [0xac, 0x32, 0x32, 0xff]);
    assert_eq!(value(&app, &sim, bits, 12, 7), [0xac, 0x32, 0x32, 0xff]);
    assert_eq!(
        value(&app, &sim, bits, 4, 3),
        [0xac, 0x32, 0x32, 0xff],
        "a linha entre amostras"
    );
    let key = input::key_of(sim.world(), bits).expect("marcador");
    assert_eq!(app.doc(key).expect("doc").history.undo_len(), 1);
    let ans = input::key(
        &mut app,
        sim.world(),
        Some(bits),
        KeyCode::KeyZ,
        true,
        false,
    );
    assert_eq!(ans, Some(KeyAnswer::Quiet));
    assert_eq!(value(&app, &sim, bits, 12, 7), [0, 0, 0, 0]);
    let ans = input::key(&mut app, sim.world(), Some(bits), KeyCode::KeyZ, true, true);
    assert_eq!(ans, Some(KeyAnswer::Quiet));
    assert_eq!(
        value(&app, &sim, bits, 12, 7),
        [0xac, 0x32, 0x32, 0xff],
        "Ctrl+Shift+Z refaz"
    );
}

#[test]
fn the_right_button_paints_the_background_color_and_x_swaps_them() {
    let (mut app, sim, cam, bits) = born();
    let (fg, bg) = (app.fg, app.bg);
    drag(&mut app, &sim, &cam, bits, &[(5, 5)], Button::Secondary);
    assert_eq!(value(&app, &sim, bits, 5, 5), bg);
    input::key(
        &mut app,
        sim.world(),
        Some(bits),
        KeyCode::KeyX,
        false,
        false,
    );
    assert_eq!((app.fg, app.bg), (bg, fg));
}

#[test]
fn b_e_i_pick_the_tool_and_the_eraser_and_eyedropper_do_their_job() {
    let (mut app, sim, cam, bits) = born();
    app.fg = [0x5b, 0x6e, 0xe1, 0xff];
    drag(
        &mut app,
        &sim,
        &cam,
        bits,
        &[(3, 3), (9, 3)],
        Button::Primary,
    );
    input::key(
        &mut app,
        sim.world(),
        Some(bits),
        KeyCode::KeyE,
        false,
        false,
    );
    assert_eq!(app.tool, Tool::Eraser);
    drag(&mut app, &sim, &cam, bits, &[(4, 3)], Button::Primary);
    assert_eq!(value(&app, &sim, bits, 4, 3), [0, 0, 0, 0]);
    assert_eq!(value(&app, &sim, bits, 5, 3), [0x5b, 0x6e, 0xe1, 0xff]);
    input::key(
        &mut app,
        sim.world(),
        Some(bits),
        KeyCode::KeyI,
        false,
        false,
    );
    assert_eq!(app.tool, Tool::Eyedropper);
    app.fg = [1, 2, 3, 255];
    let p = at(&app, &sim, &cam, bits, 6, 3);
    assert_eq!(
        Press::Done,
        input::press(
            &mut app,
            &canvas(&sim, &cam, bits),
            p,
            Button::Primary,
            Mods::default()
        )
    );
    assert_eq!(
        app.fg,
        [0x5b, 0x6e, 0xe1, 0xff],
        "o conta-gotas leu a cor para a frente"
    );
    input::key(
        &mut app,
        sim.world(),
        Some(bits),
        KeyCode::KeyB,
        false,
        false,
    );
    assert_eq!(app.tool, Tool::Pencil);
}

#[test]
fn alt_held_turns_the_pencil_into_the_eyedropper_without_painting() {
    let (mut app, sim, cam, bits) = born();
    app.fg = [0x99, 0xe5, 0x50, 0xff];
    drag(&mut app, &sim, &cam, bits, &[(8, 8)], Button::Primary);
    app.fg = [0, 0, 0, 255];
    let p = at(&app, &sim, &cam, bits, 8, 8);
    assert_eq!(
        Press::Done,
        input::press(&mut app, &canvas(&sim, &cam, bits), p, Button::Primary, ALT)
    );
    assert!(!app.stroking(), "Alt não começa um traço");
    assert_eq!(app.fg, [0x99, 0xe5, 0x50, 0xff]);
}

#[test]
fn outside_edit_the_pointer_and_the_keys_are_not_the_familys() {
    let (mut app, sim, cam, bits) = born();
    let p = at(&app, &sim, &cam, bits, 4, 4);
    let mut c = canvas(&sim, &cam, bits);
    c.editing = None;
    assert_eq!(
        Press::NotMine,
        input::press(&mut app, &c, p, Button::Primary, Mods::default())
    );
    assert_eq!(
        input::key(&mut app, sim.world(), None, KeyCode::KeyB, false, false),
        None
    );
}

/// O zoom do sprite em edição, em píxeis de ecrã por célula.
fn zoom_of(app: &PixelApp, sim: &SimWorld, cam: &Camera2d, bits: u64) -> f64 {
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let doc = app.doc(key).expect("doc");
    ph2d_pixel_render::px_per_cell(
        input::to_screen(sim.world(), bits, doc, cam, WINDOW, PPM).expect("afim"),
    )
}

fn turn(
    app: &mut PixelApp,
    sim: &SimWorld,
    cam: &mut Camera2d,
    bits: u64,
    at: (f32, f32),
    t: input::Turn,
) -> bool {
    input::wheel(
        app,
        sim.world(),
        cam,
        WINDOW,
        PPM,
        Some(bits),
        input::Notch { at, turn: t },
    )
}

/// A roda do rato: uma linha = um degrau da escada, com a célula sob o cursor parada.
#[test]
fn the_wheel_walks_the_integer_ladder_around_the_cursor() {
    let (mut app, sim, mut cam, bits) = born();
    let cursor = at(&app, &sim, &cam, bits, 20, 10);
    let before = zoom_of(&app, &sim, &cam, bits);
    assert!(turn(
        &mut app,
        &sim,
        &mut cam,
        bits,
        cursor,
        input::Turn::Lines(1.0)
    ));
    let level = ph2d_pixel_render::zoom::step(before, true);
    assert!((zoom_of(&app, &sim, &cam, bits) - f64::from(level)).abs() < 1e-3);
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let m = input::to_screen(
        sim.world(),
        bits,
        app.doc(key).expect("doc"),
        &cam,
        WINDOW,
        PPM,
    )
    .expect("afim");
    let still = ph2d_pixel_render::cell_at(m, (f64::from(cursor.0), f64::from(cursor.1)));
    assert_eq!(
        still,
        Some((20, 10)),
        "a célula sob o cursor fica sob o cursor"
    );
}

/// ⭐ **O trackpad** (report do dono 25/10: a foto chegou a 4800 %): um gesto manda dezenas de
/// bocados em píxeis. Ele tem de aproximar como o viewport aproxima com o MESMO gesto — o degrau
/// final é o maior da escada que não passa do zoom do viewport, e o seguinte já passaria. Um degrau
/// por bocado (o defeito) dava 30 degraus e o tecto da escada.
#[test]
fn a_trackpad_gesture_zooms_like_the_viewport_and_not_one_step_per_event() {
    let (mut app, sim, mut cam, bits) = born();
    let cursor = at(&app, &sim, &cam, bits, 16, 16);
    let before = zoom_of(&app, &sim, &cam, bits);
    let (events, px) = (30_u8, 10.0_f32);
    for _ in 0..events {
        assert!(
            turn(
                &mut app,
                &sim,
                &mut cam,
                bits,
                cursor,
                input::Turn::Pixels(px)
            ),
            "um bocado sem degrau completo continua a ser da roda do Pixel"
        );
    }
    let after = zoom_of(&app, &sim, &cam, bits);
    let total = f32::from(events) * px;
    let viewport = before / f64::from(Camera2d::wheel_factor(total));
    assert!(
        after > before,
        "o gesto não aproximou nada: {before} -> {after}"
    );
    assert!(
        after <= viewport + 1e-6
            && f64::from(ph2d_pixel_render::zoom::step(after, true)) > viewport,
        "{events} bocados de {px} px: o Pixel foi a {after} px/célula, o viewport iria a {viewport:.2} \
         (partindo de {before})"
    );
}

/// Um bocado só não chega a um degrau (fica guardado), e trocar de sentido esquece o resto.
#[test]
fn a_lone_trackpad_event_keeps_its_remainder_and_a_reversal_forgets_it() {
    let (mut app, sim, mut cam, bits) = born();
    let cursor = at(&app, &sim, &cam, bits, 16, 16);
    let before = zoom_of(&app, &sim, &cam, bits);
    let mut roda = input::Roda::default();
    let need = Camera2d::wheel_px_for(
        (before / f64::from(ph2d_pixel_render::zoom::step(before, true))) as f32,
    );
    assert_eq!(roda.walk(input::Turn::Pixels(need * 0.6), before), None);
    assert_eq!(roda.walk(input::Turn::Pixels(-1.0), before), None);
    assert_eq!(
        roda.walk(input::Turn::Pixels(need * 0.6), before),
        None,
        "o resto de antes da inversão contou"
    );
    assert!(turn(
        &mut app,
        &sim,
        &mut cam,
        bits,
        cursor,
        input::Turn::Pixels(1.0)
    ));
    assert!((zoom_of(&app, &sim, &cam, bits) - before).abs() < 1e-9);
}

/// ⭐ Report do dono 25/10: cada degrau da roda empilhava um aviso «Zoom N %». O degrau lê-se no
/// rodapé ([`ph2d_app_pixel::PixelApp::zoom`]); a roda não fala.
#[test]
fn the_wheel_never_pushes_a_toast_and_the_footer_reads_the_level() {
    let (mut app, mut sim, mut cam, bits) = born();
    let cursor = at(&app, &sim, &cam, bits, 16, 16);
    let mut toasts = ph2d_editor_core::ToastQueue::new();
    for _ in 0..5 {
        let mut v = ph2d_app_pixel::compose::View {
            world: sim.world(),
            camera: &mut cam,
            toasts: &mut toasts,
            window: WINDOW,
            ppm: PPM,
            editing: Some(bits),
        };
        assert!(app.wheel_event(
            &mut v,
            winit::event::MouseScrollDelta::LineDelta(0.0, 1.0),
            cursor,
            true
        ));
    }
    assert!(toasts.is_empty(), "a roda empilhou {} avisos", toasts.len());
    app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
    let level = app.zoom().expect("o rodapé lê o degrau em Draw");
    assert!((f64::from(level) - zoom_of(&app, &sim, &cam, bits)).abs() < 1e-3);
}

#[test]
fn the_blob_carries_only_live_sprites_and_reopens_the_drawing() {
    let (mut app, sim, cam, bits) = born();
    app.fg = [0xd9, 0x57, 0x63, 0xff];
    drag(
        &mut app,
        &sim,
        &cam,
        bits,
        &[(1, 30), (30, 1)],
        Button::Primary,
    );
    let bytes = persist::to_bytes(&app, sim.world());
    let mut fresh = PixelApp::default();
    persist::install(&mut fresh, persist::read(&bytes).expect("lê"));
    assert_eq!(value(&fresh, &sim, bits, 1, 30), [0xd9, 0x57, 0x63, 0xff]);
    assert_eq!(
        persist::to_bytes(&fresh, sim.world()),
        bytes,
        "reabrir e regravar dá os mesmos bytes"
    );
    let empty = SimWorld::new();
    assert!(
        persist::to_bytes(&app, empty.world()).is_empty(),
        "sem objecto vivo, nada a gravar"
    );
}

#[test]
fn a_duplicated_object_gets_its_own_copy_of_the_drawing() {
    let (mut app, mut sim, mut cam, bits) = born();
    drag(&mut app, &sim, &cam, bits, &[(0, 0)], Button::Primary);
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let twin = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::PixelSprite(key)))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
    let twin_key = input::key_of(sim.world(), twin).expect("marcador");
    assert_ne!(twin_key, key, "a cópia ganhou um documento seu");
    assert_eq!(value(&app, &sim, twin, 0, 0), value(&app, &sim, bits, 0, 0));
    drag(&mut app, &sim, &cam, bits, &[(1, 0)], Button::Primary);
    assert_ne!(
        value(&app, &sim, twin, 1, 0),
        value(&app, &sim, bits, 1, 0),
        "pintar um não pinta o outro"
    );
}

#[test]
fn smoke_1_stages_a_face_in_four_undo_steps_and_asks_for_edit() {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    assert!(ph2d_app_pixel::smoke::stage(
        &mut app, &mut sim, 1, spawn_root
    ));
    let sprites = mode::sprites(sim.world());
    assert_eq!(sprites.len(), 1);
    let (bits, key) = sprites[0];
    assert_eq!(app.doc(key).expect("doc").history.undo_len(), 4);
    assert_eq!(
        value(&app, &sim, bits, 11, 12),
        [0, 0, 0, 255],
        "o olho esquerdo"
    );
    let mut tools = ToolRegistry::default();
    assert_eq!(
        mode::Family::new(&mut app, sim.world()).wants(&mut tools),
        Some((bits, ObjectMode::Draw))
    );
    assert!(
        !ph2d_app_pixel::smoke::stage(&mut app, &mut sim, 3, spawn_root),
        "nível sem cena"
    );
}

#[test]
fn space_or_h_turn_the_drag_into_a_pan_that_lands_cells_on_whole_pixels() {
    let (mut app, sim, mut cam, bits) = born();
    assert!(input::space(&mut app, sim.world(), Some(bits), true));
    let p = at(&app, &sim, &cam, bits, 4, 4);
    assert_eq!(
        input::press(
            &mut app,
            &canvas(&sim, &cam, bits),
            p,
            Button::Primary,
            Mods::default()
        ),
        Press::Pan
    );
    assert!(!app.stroking(), "a mão não pinta");
    // A shell arrasta a câmera por meio píxel de ecrã…
    cam.center[0] += 0.37 * cam.height_world / WINDOW.height as f32;
    assert!(input::release_pan(
        &mut app,
        sim.world(),
        &mut cam,
        WINDOW,
        PPM,
        Some(bits)
    ));
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let m = input::to_screen(
        sim.world(),
        bits,
        app.doc(key).expect("doc"),
        &cam,
        WINDOW,
        PPM,
    )
    .expect("afim");
    let p0 = m * Point::new(0.0, 0.0);
    assert!(
        (p0.x - p0.x.round()).abs() < 1e-2,
        "{p0:?}: largar a mão realinha"
    );
    assert!(input::space(&mut app, sim.world(), Some(bits), false));
    input::key(
        &mut app,
        sim.world(),
        Some(bits),
        KeyCode::KeyH,
        false,
        false,
    );
    assert_eq!(app.tool, Tool::Hand);
    assert_eq!(
        input::press(
            &mut app,
            &canvas(&sim, &cam, bits),
            p,
            Button::Primary,
            Mods::default()
        ),
        Press::Pan
    );
}
