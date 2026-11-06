//! ⭐ **O cartão do Pixel Sprite no Inspector** (dono 06/11), pelo quadro inteiro da família (`panel::frame`): nos três
//! modos de cor, com o aspecto, e o `Arc` só muda quando o desenho muda.

use std::sync::Arc;

use ph2d_app_pixel::PixelApp;
use ph2d_app_pixel::mode::{Family, editing};
use ph2d_app_pixel::panel::frame;
use ph2d_component_desc::ObjectKind;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
use ph2d_editor_core::pixel_panel::{PixelFrameAction, PixelRequest, PixelSpriteCard};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::{ModeFamily, drive};
use ph2d_editor_core::{NodeId, ToastQueue, ToolRegistry};
use ph2d_pixel_core::{ColorMode, NewSprite, Op, Patch, Rect};

fn objecto(sim: &mut SimWorld, app: &mut PixelApp, spec: NewSprite) -> (u64, u32) {
    let key = app.create(spec).expect("o documento nasce");
    let bits = sim
        .world_mut()
        .spawn(ph2d_ecs::PixelSprite(key))
        .id()
        .to_bits();
    (bits, key)
}

/// Um passo de edição: um pixel de `valor` em (x, y).
fn pinta(app: &mut PixelApp, key: u32, x: i32, y: i32, valor: &[u8]) {
    let doc = app.doc(key).expect("o documento");
    let r = Rect::new(x, y, 1, 1);
    let op = Op::Patch(Patch {
        layer: doc.layer,
        frame: doc.frame,
        bbox: Some(r),
        rect: r,
        pixels: valor.to_vec(),
    });
    app.commit(key, op).expect("o passo aplica");
}

fn hero_com(bits: u64) -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    h.gizmo.replace_selection(Some(bits));
    h
}

fn quadro(app: &mut PixelApp, sim: &SimWorld, h: &mut HeroScreen) {
    let _ = frame(app, sim.world(), h, &mut ToastQueue::default());
}

fn cartao_de(h: &HeroScreen, bits: u64) -> PixelSpriteCard {
    h.store.pixel_sprite_card(bits).expect("o cartão").clone()
}

fn spec(mode: ColorMode, width: u32, height: u32) -> NewSprite {
    NewSprite {
        width,
        height,
        mode,
        ..NewSprite::default()
    }
}

/// O modo do quadro: `toasts` guarda os avisos do pedido.
fn modo(
    app: &mut PixelApp,
    sim: &SimWorld,
    h: &mut HeroScreen,
    toasts: &mut ToastQueue,
    pedido: Option<ModeRequest>,
) {
    let kind = |_: u64| ObjectKind::PixelSprite;
    let nome = |_: u64| String::new();
    let mut tools = ToolRegistry::new();
    let mut f = Family::new(app, sim.world());
    let fams: &mut [&mut dyn ModeFamily] = &mut [&mut f];
    drive(fams, &kind, &nome, &mut tools, h, toasts, None);
    drive(fams, &kind, &nome, &mut tools, h, toasts, pedido);
}

/// ⭐ *Mutações: refazer a miniatura sem olhar a versão ⇒ o mesmo desenho troca de `Arc` (e de textura) · nunca a refazer
/// ⇒ sair do Draw não mostra os píxeis novos.*
#[test]
fn a_miniatura_do_pixel_so_muda_quando_o_desenho_muda() {
    let mut sim = SimWorld::new();
    let mut app = PixelApp::default();
    let (bits, key) = objecto(&mut sim, &mut app, spec(ColorMode::Rgba, 4, 2));
    let mut h = hero_com(bits);
    quadro(&mut app, &sim, &mut h);
    let a = cartao_de(&h, bits);
    quadro(&mut app, &sim, &mut h);
    assert!(
        Arc::ptr_eq(&a.rgba, &cartao_de(&h, bits).rgba),
        "reenviava todo quadro"
    );

    let mut toasts = ToastQueue::default();
    let draw = Some(ModeRequest::Enter(ObjectMode::Draw));
    modo(&mut app, &sim, &mut h, &mut toasts, draw);
    assert_eq!(editing(sim.world(), &h), Some(bits));
    pinta(&mut app, key, 1, 1, &[255, 0, 0, 255]);
    quadro(&mut app, &sim, &mut h);
    assert!(
        Arc::ptr_eq(&a.rgba, &cartao_de(&h, bits).rgba),
        "em Draw a miniatura espera"
    );
    let sai = Some(ModeRequest::Enter(ObjectMode::Object));
    modo(&mut app, &sim, &mut h, &mut toasts, sai);
    quadro(&mut app, &sim, &mut h);
    let b = cartao_de(&h, bits);
    assert!(
        !Arc::ptr_eq(&a.rgba, &b.rgba),
        "ao sair do Draw, a miniatura não mudou"
    );
    let i = (4 + 1) * 4;
    assert_eq!(&b.rgba[i..i + 4], &[255, 0, 0, 255]);
}

/// Os três modos de cor dão RGBA reto no tamanho do documento, e fora da seleção não há cartão.
#[test]
fn o_cartao_do_pixel_serve_aos_tres_modos_e_guarda_o_aspecto() {
    for (mode, valor, cor) in [
        (ColorMode::Rgba, vec![10, 20, 30, 255], [10, 20, 30, 255]),
        (ColorMode::Gray, vec![200, 255], [200, 200, 200, 255]),
    ] {
        let mut sim = SimWorld::new();
        let mut app = PixelApp::default();
        let (bits, key) = objecto(&mut sim, &mut app, spec(mode, 8, 3));
        pinta(&mut app, key, 0, 0, &valor);
        let mut h = hero_com(bits);
        quadro(&mut app, &sim, &mut h);
        let c = cartao_de(&h, bits);
        assert_eq!((c.size, c.thumb), ([8, 3], [8, 3]));
        assert_eq!(c.rgba.len(), 8 * 3 * 4);
        assert_eq!(&c.rgba[..4], &cor, "{mode:?}");
    }
    let mut sim = SimWorld::new();
    let mut app = PixelApp::default();
    let (bits, key) = objecto(&mut sim, &mut app, spec(ColorMode::Indexed, 8, 3));
    let cor = app.doc(key).expect("o documento").sprite.palette()[1];
    pinta(&mut app, key, 0, 0, &[1]);
    let mut h = hero_com(bits);
    quadro(&mut app, &sim, &mut h);
    assert_eq!(
        &cartao_de(&h, bits).rgba[..4],
        &cor,
        "Indexed passa pela paleta"
    );

    let outro = sim.world_mut().spawn_empty().id().to_bits();
    h.gizmo.replace_selection(Some(outro));
    quadro(&mut app, &sim, &mut h);
    assert!(h.store.pixel_sprite_card(bits).is_none());
}

/// Um documento enorme guarda uma miniatura pequena, com o aspecto; o tamanho do cartão é o do documento.
#[test]
fn a_miniatura_do_documento_grande_guarda_o_aspecto() {
    let mut sim = SimWorld::new();
    let mut app = PixelApp::default();
    let (bits, key) = objecto(&mut sim, &mut app, spec(ColorMode::Rgba, 2048, 1024));
    pinta(&mut app, key, 2047, 1023, &[9, 8, 7, 255]);
    let mut h = hero_com(bits);
    quadro(&mut app, &sim, &mut h);
    let c = cartao_de(&h, bits);
    assert_eq!((c.size, c.thumb), ([2048, 1024], [512, 256]));
    assert_eq!(c.rgba.len(), 512 * 256 * 4);
}

/// O quadro corrente: mudar de quadro muda a miniatura.
#[test]
fn a_miniatura_do_pixel_e_a_do_quadro_corrente() {
    let mut sim = SimWorld::new();
    let mut app = PixelApp::default();
    let (bits, key) = objecto(&mut sim, &mut app, spec(ColorMode::Rgba, 2, 2));
    pinta(&mut app, key, 0, 0, &[1, 2, 3, 255]);
    let mut h = hero_com(bits);
    quadro(&mut app, &sim, &mut h);
    assert_eq!(&cartao_de(&h, bits).rgba[..4], &[1, 2, 3, 255]);
    let mut toasts = ToastQueue::default();
    let draw = Some(ModeRequest::Enter(ObjectMode::Draw));
    modo(&mut app, &sim, &mut h, &mut toasts, draw);
    for _ in 0..3 {
        app.pede(PixelRequest::FrameAction(PixelFrameAction::NewEmptyFrame));
        quadro(&mut app, &sim, &mut h);
    }
    let sai = Some(ModeRequest::Enter(ObjectMode::Object));
    modo(&mut app, &sim, &mut h, &mut toasts, sai);
    quadro(&mut app, &sim, &mut h);
    assert_eq!(app.doc(key).expect("o documento").sprite.frames().len(), 4);
    assert_eq!(&cartao_de(&h, bits).rgba[..4], &[0, 0, 0, 0]);
}

/// ⭐ O verbo do cartão entra no Draw daquele objecto: `Enter` sobre a seleção, e `OpenContent` de um cartão preso.
/// *Mutação: tirar o `Family::open_content` ⇒ o aviso «o conteúdo sumiu».*
#[test]
fn o_pedido_do_verbo_pixel_entra_no_draw_do_objecto() {
    let mut sim = SimWorld::new();
    let mut app = PixelApp::default();
    let (a, _) = objecto(&mut sim, &mut app, spec(ColorMode::Rgba, 2, 2));
    let (b, _) = objecto(&mut sim, &mut app, spec(ColorMode::Rgba, 2, 2));
    for (seleccao, pedido) in [
        (a, ModeRequest::Enter(ObjectMode::Draw)),
        (
            b,
            ModeRequest::OpenContent {
                entity: a,
                mode: ObjectMode::Draw,
                key: 0,
            },
        ),
    ] {
        let mut h = hero_com(seleccao);
        let mut toasts = ToastQueue::default();
        modo(&mut app, &sim, &mut h, &mut toasts, Some(pedido));
        assert_eq!(editing(sim.world(), &h), Some(a));
        assert_eq!(h.gizmo.selection, Some(a));
        assert_eq!(toasts.len(), 1, "só o «entrou»");
    }
}
