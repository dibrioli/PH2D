//! ⭐ **Os frames pelos PIXELS** (W5) — desenhados pelo `VelloPass` do produto e lidos de volta:
//! o filho que atravessa a borda corta-se nela (oráculo Excalidraw A10), um frame pequeno no ecrã
//! mostra só o fundo (o nível de detalhe), e uma nota de longe é um bloco sem sombra.
//!
//! Sem adaptador a placa não corre e o teste diz-o (a régua da placa faz o mesmo):
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --test it frames_paint`.

use ph2d_board_model::{
    BoardOp, BoardSet, Camera, Element, Frame, Rgba, STICKY_COLORS, STICKY_SIDE, Shape, ShapeType,
    Style,
};
use ph2d_board_render::{Detail, RenderCache};
use ph2d_board_route::RouteCache;
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const SIZE: (u32, u32) = (640, 480);
const AREA: [f64; 4] = [0.0, 0.0, 640.0, 480.0];
const RED: Rgba = Rgba([220, 20, 20, 255]);
const WHITE: Rgba = Rgba([255, 255, 255, 255]);

struct Px {
    rgba: Vec<u8>,
}

impl Px {
    fn at(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * SIZE.0 + x) * 4) as usize;
        [
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ]
    }

    fn is(&self, x: u32, y: u32, c: Rgba) -> bool {
        let p = self.at(x, y);
        (0..3).all(|k| p[k].abs_diff(c.0[k]) <= 2)
    }
}

fn gpu() -> Option<(GpuContext, VelloPass)> {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        println!("sem adaptador: o teste dos frames pelos pixels não corre");
        return None;
    };
    let pass = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, SIZE).expect("VelloPass");
    Some((gpu, pass))
}

fn render(g: &mut (GpuContext, VelloPass), set: &BoardSet, detail: Detail) -> Px {
    let (gpu, pass) = g;
    let board = &set.boards()[0];
    let mut routes = RouteCache::default();
    routes.sync(&board.doc);
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    let mut cache = RenderCache::default();
    cache.detail = detail;
    ph2d_board_render::paint(
        board,
        AREA,
        &mut scene,
        Theme::Forge,
        &mut ts,
        &mut cache,
        &routes,
        None,
    );
    Px {
        rgba: pass
            .render_and_readback(&*gpu, scene.inner(), SIZE)
            .expect("readback"),
    }
}

/// Um frame branco `(0,0) 200×200` com um filho vermelho `(150,50) 100×50` que atravessa a borda
/// direita; a vista centrada no frame.
fn frame_scene(zoom: f64) -> BoardSet {
    let mut set = BoardSet::default();
    let id = set.create("f".into());
    let b = set.get_mut(id).unwrap();
    let frame = Frame {
        name: String::new(),
        style: Style::new(Some(WHITE), None, RED),
    };
    let f = Element::new_frame(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        frame,
        [0.0, 0.0, 200.0, 200.0],
    );
    let fid = f.id;
    BoardOp::Put(f).apply(&mut b.doc);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(Some(RED), None, RED),
        text: Default::default(),
    };
    let mut child = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        shape,
        [150.0, 50.0, 100.0, 50.0],
    );
    child.frame = Some(fid);
    BoardOp::Put(child).apply(&mut b.doc);
    b.camera = Camera {
        center_x: 100.0,
        center_y: 100.0,
        zoom,
    };
    set
}

#[test]
fn a_child_is_cut_at_the_frame_border_and_a_far_frame_shows_only_its_back() {
    let Some(mut g) = gpu() else {
        return;
    };
    // Zoom 1: o frame em 220..420 × 140..340; o filho em 370..470 × 190..240.
    let near = render(&mut g, &frame_scene(1.0), Detail::default());
    assert!(near.is(395, 215, RED), "o filho dentro do frame pinta-se");
    assert!(
        !near.is(445, 215, RED),
        "fora da borda o filho é CORTADO (oráculo A10)"
    );
    assert!(near.is(250, 300, WHITE), "o fundo do frame");
    // Sem o recorte de longe, o mesmo frame a 0,4 (80 px de alto) ainda mostra o filho…
    let open = Detail {
        frame_title_only_px: 0.0,
        ..Detail::default()
    };
    let far_open = render(&mut g, &frame_scene(0.4), open);
    // Zoom 0,4: o frame em 280..360 × 200..280; o filho em 340..360 × 180+… (x 340..380 cortado a 360).
    assert!(far_open.is(350, 230, RED));
    // …e com o nível de detalhe (80 < 96 px) só o fundo.
    let far = render(&mut g, &frame_scene(0.4), Detail::default());
    assert!(
        far.is(350, 230, WHITE),
        "de longe o frame mostra só o fundo"
    );
}

#[test]
fn a_far_note_is_a_flat_block_without_shadow() {
    let Some(mut g) = gpu() else {
        return;
    };
    let mut set = BoardSet::default();
    let id = set.create("n".into());
    let b = set.get_mut(id).unwrap();
    let fill = Rgba(STICKY_COLORS[3]);
    let note = Shape {
        kind: ShapeType::Sticky,
        style: Style::new(Some(fill), None, fill.readable_ink()),
        text: "longe".into(),
    };
    let el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        note,
        [0.0, 0.0, STICKY_SIDE, STICKY_SIDE],
    );
    BoardOp::Put(el).apply(&mut b.doc);
    // Letra de 20 a 0,2 = 4 px (< READ_PX): a nota tem ~40 px; a sombra cairia logo abaixo dela.
    b.camera = Camera {
        center_x: STICKY_SIDE / 2.0,
        center_y: STICKY_SIDE / 2.0,
        zoom: 0.2,
    };
    let below = (240.0 + STICKY_SIDE * 0.2 / 2.0 + 2.0) as u32;
    let blocks = render(&mut g, &set, Detail::default());
    let shadowed = render(
        &mut g,
        &set,
        Detail {
            note_blocks: false,
            ..Detail::default()
        },
    );
    assert!(blocks.is(320, 240, fill), "o bloco tem a cor da nota");
    assert_ne!(
        blocks.at(320, below),
        shadowed.at(320, below),
        "sem o bloco, a sombra escurece a faixa por baixo; com ele não há sombra"
    );
    assert_eq!(
        blocks.at(320, below),
        blocks.at(600, 20),
        "por baixo do bloco: o fundo do quadro"
    );
}

/// Um frame branco `(0,0) 200×200` (rodado de `angle`) e uma moldura de tinta de NASCENÇA
/// (`DEFAULT_INK`, sem preenchimento) dentro dele e outra fora, no quadro.
fn ink_scene(angle: f64) -> BoardSet {
    let mut set = BoardSet::default();
    let id = set.create("i".into());
    let b = set.get_mut(id).unwrap();
    let frame = Frame {
        name: String::new(),
        style: Style::new(Some(WHITE), None, RED),
    };
    let mut f = Element::new_frame(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        frame,
        [0.0, 0.0, 200.0, 200.0],
    );
    f.angle = angle;
    let fid = f.id;
    BoardOp::Put(f).apply(&mut b.doc);
    let ink = Rgba(ph2d_board_model::DEFAULT_INK);
    for (x, home) in [(40.0, Some(fid)), (300.0, None)] {
        let mut style = Style::new(None, Some(ink), ink);
        style.stroke_width = 6.0;
        let shape = Shape {
            kind: ShapeType::Rectangle,
            style,
            text: Default::default(),
        };
        let mut el = Element::new_shape(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            shape,
            [x, 40.0, 120.0, 120.0],
        );
        el.frame = home;
        BoardOp::Put(el).apply(&mut b.doc);
    }
    b.camera = Camera {
        center_x: 100.0,
        center_y: 100.0,
        zoom: 1.0,
    };
    set
}

fn luminance(p: [u8; 4]) -> u32 {
    (u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2])) / 3
}

/// ⭐ Smoke do dono (08/10): no tema escuro a tinta de nascença desenha-se CLARA — e dentro de um
/// frame branco sumia. Ela lê-se contra o fundo de onde o elemento vive: escura no frame branco,
/// clara no quadro escuro.
#[test]
fn the_default_ink_reads_against_the_frame_it_lives_in() {
    let Some(mut g) = gpu() else {
        return;
    };
    // Zoom 1: o frame em 220..420 × 140..340; a borda esquerda da moldura de dentro em x 260, a de
    // fora em x 520.
    let px = render(&mut g, &ink_scene(0.0), Detail::default());
    let inside = px.at(260, 240);
    let outside = px.at(520, 240);
    assert!(
        luminance(inside) < 100,
        "dentro do frame branco a tinta sumia: {inside:?}"
    );
    assert!(
        luminance(outside) > 128,
        "o tema é escuro: fora a tinta é clara ({outside:?})"
    );
}

/// ⭐ Um frame RODA (ordem do dono, 08/10): o fundo desenha-se rodado — o canto da caixa por rodar
/// fica no quadro, o meio da borda de cima rodada é branco.
#[test]
fn a_rotated_frame_paints_rotated() {
    let Some(mut g) = gpu() else {
        return;
    };
    let px = render(
        &mut g,
        &ink_scene(std::f64::consts::FRAC_PI_4),
        Detail::default(),
    );
    assert!(
        !px.is(225, 145, WHITE),
        "o canto da caixa SEM rodar pintou-se"
    );
    assert!(px.is(320, 150, WHITE), "o miolo do frame rodado");
}
