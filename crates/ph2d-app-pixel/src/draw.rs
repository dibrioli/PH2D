//! **O desenho**: todo Pixel Sprite visível como imagem nítida no lugar do objecto; o que está em
//! Draw leva o xadrez por baixo, a borda, a grelha de pixels (se ligada) e o cursor de célula.
//!
//! ⭐ GEF 19 E2: a ARTE entra na ordem total (faixa `Family::Pixel`); o chrome do Draw fica por cima.

use ph2d_editor_core::paint::resolve;
use ph2d_host::WindowSize;
use ph2d_pixel_render::{
    CURSOR_CORE, draw_border, draw_cell_cursor, draw_marching_ants, draw_pixel_grid,
    draw_symmetry_axes, mask_edges, px_per_cell,
};
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

use crate::PixelApp;
use crate::input::to_view;

/// Um Pixel Sprite como o último quadro o desenhou: o clique e a caixa do gizmo perguntam AQUI, à
/// mesma geometria que a tinta usou (nunca uma segunda conta).
#[derive(Debug, Clone, Copy)]
struct Drawn {
    bits: u64,
    to_screen: ph2d_vector::Affine,
    size: (u32, u32),
}

thread_local! {
    static DRAWN: std::cell::RefCell<Vec<Drawn>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Os Pixel Sprites sob o ponto de ecrã `p` (dentro do canvas), do de cima para o de baixo.
#[must_use]
pub fn pick_at(p: (f32, f32)) -> Vec<u64> {
    DRAWN.with(|d| {
        d.borrow()
            .iter()
            .rev()
            .filter(|s| {
                ph2d_pixel_render::cell_at(s.to_screen, (f64::from(p.0), f64::from(p.1)))
                    .is_some_and(|(x, y)| {
                        x >= 0 && y >= 0 && (x as u32) < s.size.0 && (y as u32) < s.size.1
                    })
            })
            .map(|s| s.bits)
            .collect()
    })
}

/// A meia-extensão no espaço LOCAL do objecto (metros, antes da escala) do Pixel Sprite `bits` —
/// a caixa do gizmo de objecto. `None` = não foi desenhado no último quadro.
#[must_use]
pub fn half_extents(bits: u64, ppm: f32) -> Option<[f32; 2]> {
    let ppm = ppm.max(f32::MIN_POSITIVE);
    DRAWN.with(|d| {
        d.borrow()
            .iter()
            .find(|s| s.bits == bits)
            .map(|s| [s.size.0 as f32 * 0.5 / ppm, s.size.1 as f32 * 0.5 / ppm])
    })
}

/// Espessura das linhas do modo Draw, em píxeis de ECRÃ (saem com `Affine::IDENTITY`).
const LINE_PX: f64 = 1.0; // LITERAL-PX-OK: chrome de sobreposição, espessura de ecrã
/// Abaixo disto a grelha de pixels vira um borrão cinzento (uma linha por célula de 1–3 px).
const GRID_MIN_PX_PER_CELL: f64 = 4.0; // LITERAL-PX-OK: legibilidade da grelha, em píxeis de ecrã

thread_local! {
    /// A arte do EDITOR já desenhada nas faixas deste quadro (`Some`) — o [`draw`] não a repete.
    static PENDING: std::cell::RefCell<Option<Vec<Drawn>>> = const { std::cell::RefCell::new(None) };
}

/// Início do quadro: nenhuma arte nas faixas ainda (chamado antes de codificar as faixas).
pub fn begin_frame() {
    PENDING.with(|p| *p.borrow_mut() = None);
}

/// ⭐ **A ARTE** (GEF 19 E2) dos Pixel Sprites `ids`, de trás para a frente, com a câmera `cam`
/// (mundo → ecrã) — numa faixa da ordem total ou, sem intercalação, na cena do chrome. `editor` =
/// a vista do editor: a geometria vai para o clique ([`pick_at`]); a miniatura da câmera não.
#[allow(clippy::too_many_arguments)]
pub fn art(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    cam: ph2d_vector::Affine,
    ppm: f32,
    editing: Option<u64>,
    ids: &[u64],
    editor: bool,
    scene: &mut VectorScene,
) {
    let drawn = art_of(app, world, cam, ppm, editing, ids, scene);
    if editor {
        PENDING.with(|p| p.borrow_mut().get_or_insert_with(Vec::new).extend(drawn));
    }
}

fn art_of(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    cam: ph2d_vector::Affine,
    ppm: f32,
    editing: Option<u64>,
    ids: &[u64],
    scene: &mut VectorScene,
) -> Vec<Drawn> {
    let mut drawn = Vec::new();
    for &bits in ids {
        let Some(key) = crate::input::key_of(world, bits) else {
            continue;
        };
        let Some(entity) = ph2d_ecs::Entity::try_from_bits(bits) else {
            continue;
        };
        if ph2d_ecs::visibility::is_hidden_in_tree(world, entity) {
            continue;
        }
        // A moldura solta mostra-se como o traço do seu assentar (o mesmo op).
        let previa = app.previa(key);
        let pending = app
            .gesture
            .as_ref()
            .filter(|g| g.key == key)
            .map(|g| g.stroke.clone())
            .or_else(|| previa.as_ref().map(|(s, _)| s.clone()));
        let Some(doc) = app.docs.get_mut(&key) else {
            continue;
        };
        let Some(m) = to_view(world, bits, doc, cam, ppm) else {
            continue;
        };
        let (w, h) = (doc.sprite.width(), doc.sprite.height());
        let edit = editing == Some(bits);
        let onion = app.onion_view(key, edit);
        crate::tiles::compoe(app, key, pending.as_ref(), onion.as_ref());
        let Some(doc) = app.docs.get_mut(&key) else {
            continue;
        };
        let o = app.options;
        if edit && (o.tile_x || o.tile_y) {
            // O Tiled Mode mostra as vizinhas: o desenho que sai por um lado entra pelo outro.
            let (fw, fh) = (f64::from(w), f64::from(h));
            let xs: &[f64] = if o.tile_x { &[-1.0, 0.0, 1.0] } else { &[0.0] };
            let ys: &[f64] = if o.tile_y { &[-1.0, 0.0, 1.0] } else { &[0.0] };
            for &ty in ys {
                for &tx in xs {
                    if (tx, ty) != (0.0, 0.0) {
                        let k = m * ph2d_vector::Affine::translate((tx * fw, ty * fh));
                        doc.view.draw_checker_under(scene, w, h, k);
                        doc.view.draw(scene, k);
                    }
                }
            }
        }
        if edit {
            doc.view.draw_checker_under(scene, w, h, m);
        }
        doc.view.draw(scene, m);
        drawn.push(Drawn {
            bits,
            to_screen: m,
            size: (w, h),
        });
    }
    drawn
}

/// Os Pixel Sprites do quadro: a arte de `ids` (rank crescente) se as faixas não a levaram, e por
/// cima o chrome do Draw (grelha, borda, simetria, selecção, cursor) do que está em edição.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    camera: &Camera2d,
    window: WindowSize,
    ppm: f32,
    editing: Option<u64>,
    ids: &[u64],
    theme: Theme,
    scene: &mut VectorScene,
) {
    let drawn = PENDING.with(|p| p.borrow_mut().take()).unwrap_or_else(|| {
        let cam = camera.world_to_screen_affine(window);
        art_of(app, world, cam, ppm, editing, ids, scene)
    });
    let edited = drawn.iter().find(|d| Some(d.bits) == editing).copied();
    DRAWN.with(|d| *d.borrow_mut() = drawn);
    let Some(Drawn {
        bits,
        to_screen: m,
        size: (w, h),
    }) = edited
    else {
        return;
    };
    let Some(key) = crate::input::key_of(world, bits) else {
        return;
    };
    let previa = app.previa(key);
    let o = app.options;
    if app.grid && px_per_cell(m) >= GRID_MIN_PX_PER_CELL {
        draw_pixel_grid(scene, w, h, m, resolve(ColorToken::Border, theme), LINE_PX);
    }
    if let Some(d) = app.docs.get(&key) {
        ph2d_pixel_tiles::grelha::desenha(
            &d.sprite,
            (d.layer, d.frame),
            crate::tiles::realce(app, bits),
            m,
            (LINE_PX, GRID_MIN_PX_PER_CELL),
            (
                resolve(ColorToken::Border, theme),
                resolve(ColorToken::Accent, theme),
            ),
            scene,
        );
    }
    draw_border(
        scene,
        w,
        h,
        m,
        resolve(ColorToken::Selection, theme),
        LINE_PX,
    );
    if let Some(doc) = app.docs.get(&key)
        && app.tool.paints()
        && (o.sym_h || o.sym_v || o.sym_d1 || o.sym_d2)
    {
        let s = crate::gesture::symmetry(&o, doc);
        draw_symmetry_axes(
            scene,
            (w, h),
            m,
            (s.axis2_x, s.axis2_y),
            (o.sym_h, o.sym_v, o.sym_d1, o.sym_d2),
            resolve(ColorToken::Accent, theme),
            LINE_PX,
        );
    }
    selecao(
        app,
        key,
        m,
        previa.and_then(|(_, mk)| mk),
        (theme, window),
        scene,
    );
    #[expect(clippy::cast_precision_loss, reason = "tamanho de janela")]
    let janela = (window.width as f32, window.height as f32);
    crate::viva::draw(app, key, m, (theme, janela), scene);
    // P8: a caixa do Text aberta — a moldura (pelo menos uma célula de largo) e o cursor.
    if app.text_box().is_some_and(|b| b.0 == key)
        && let Some(((_, (cx, cy, ch)), (x, y, bw, bh))) =
            app.text_box_frame().zip(app.text_box_outline())
    {
        let em =
            |px: i32, py: i32| m * ph2d_vector::Affine::translate((f64::from(px), f64::from(py)));
        let cor = resolve(ColorToken::Accent, theme);
        draw_border(scene, bw.max(1), bh.max(1), em(x, y), cor, LINE_PX);
        let alto = u32::try_from(ch).unwrap_or(0).max(1);
        draw_border(scene, 0, alto, em(cx, cy), cor, LINE_PX);
    }
    let Some(doc) = app.docs.get(&key) else {
        return;
    };
    if let Some((hb, cell)) = app.hover
        && hb == bits
        && doc.sprite.bounds().contains(cell.0, cell.1)
    {
        draw_cell_cursor(scene, cell, m, CURSOR_CORE, LINE_PX);
    }
}

/// Quantos ms as formigas levam a andar um píxel de ecrã (escolha visual: o passo do Aseprite).
const ANTS_STEP_MS: u64 = 60;

/// **A selecção por cima da arte** (P6): as formigas da máscara (a da moldura solta quando há
/// uma), as do laço em curso, e a moldura com as alças.
fn selecao(
    app: &PixelApp,
    key: u32,
    m: ph2d_vector::Affine,
    mascara_solta: Option<ph2d_pixel_core::selection::Mask>,
    (theme, window): (Theme, WindowSize),
    scene: &mut VectorScene,
) {
    let fase =
        ((app.formigas_ms / ANTS_STEP_MS) % (2.0 * ph2d_pixel_render::ANTS_DASH_PX) as u64) as f64;
    let Some(s) = app.docs.get(&key).map(|d| &d.sprite) else {
        return;
    };
    let solta = app.solta.as_ref().filter(|so| so.key == key);
    let mascara = match solta {
        Some(_) => mascara_solta,
        None => s.mask().cloned(),
    };
    if let Some(mk) = &mascara {
        draw_marching_ants(scene, &mask_edges(mk), m, fase, LINE_PX);
    }
    if let Some(l) = app.laco.as_ref().filter(|l| l.key == key)
        && let Some(mk) = crate::selecao::preview(l, s.width(), s.height())
    {
        draw_marching_ants(scene, &mask_edges(&mk), m, fase, LINE_PX);
    }
    if let Some(so) = solta {
        #[expect(clippy::cast_precision_loss, reason = "tamanho de janela")]
        let janela = (window.width as f32, window.height as f32);
        let view = so.gizmo(m, janela, so.cursor);
        ph2d_editor_core::gizmo::paint_sprite_gizmo_skewing(
            scene,
            &view,
            theme,
            &mut ph2d_editor_core::interaction::HitIndex::new(),
            true,
        );
    }
}
