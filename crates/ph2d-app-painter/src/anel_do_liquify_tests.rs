//! ⭐⭐⭐ **O ANEL DO LIQUIFY CAI ONDE O KERNEL DEFORMA** — sobre uma imagem presa aos ossos e
//! DOBRADA, que é onde o Liquify trabalha pela regra F6-s.
//!
//! A régua não sabe como o anel foi feito: cada ponto dele volta à imagem pela porta do PONTEIRO do
//! Painter ([`ph2d_render::mesh_uv`], a que resolve onde o dab pousa), e a distância ao centro, em px
//! do canvas, tem de ser o raio do kernel (`deform_size_px`).
//!
//! ⚠️ **A arte está FORA da origem e RODADA**: com a pose identidade, um mapa que esquecesse a base
//! ou a posição da instância passaria.

use super::anel_do_liquify;
use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef, SimWorld, Transform};
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_render::{Camera2d, MeshUv, RenderInstance, Sprite, SpriteMesh};
use ph2d_tool_painter::PainterTool;

/// O canvas: `400 × 200` px sobre um quad de `4 × 2` m.
const CANVAS: (u32, u32) = (400, 200);
const QUAD: [f32; 2] = [4.0, 2.0];
/// O raio da dobra: a aresta de cima estica `1,25×` e a de baixo encolhe `0,75×`.
const R: f32 = 4.0;
const JANELA: ph2d_host::WindowSize = ph2d_host::WindowSize {
    width: 800,
    height: 800,
};

fn pose() -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = -1.2;
    t.translation.y = 0.9;
    t.rotation = -0.5;
    t
}

/// A faixa enrolada num arco de `R` m, com a UV da imagem.
fn malha_dobrada() -> SpriteMesh {
    let (cols, rows) = (32_u32, 16_u32);
    let mut m = SpriteMesh {
        local: Vec::new(),
        uv: Vec::new(),
        tris: Vec::new(),
        skin: None,
    };
    for j in 0..=rows {
        for i in 0..=cols {
            let (u, v) = (i as f32 / cols as f32, j as f32 / rows as f32);
            let (x, y) = ((u - 0.5) * QUAD[0], (0.5 - v) * QUAD[1]);
            let (theta, raio) = (x / R, R + y);
            m.local.push([raio * theta.sin(), raio * theta.cos() - R]);
            m.uv.push([u, v]);
        }
    }
    let id = |i: u32, j: u32| j * (cols + 1) + i;
    for j in 0..rows {
        for i in 0..cols {
            m.tris.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            m.tris.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
        }
    }
    m
}

/// A instância desenhada, com ou sem a malha.
fn cena(dobrada: bool) -> (PresentWorld, u64, Camera2d) {
    let mut sim = SimWorld::default();
    let sprite = Sprite::atlas(0, QUAD, [1.0; 4]);
    let e = sim.world_mut().spawn((pose(), sprite)).id();
    let global = GlobalTransform::from_transform(pose());
    let a = global.affine();
    let mut present = PresentWorld::new();
    let p = present
        .world_mut()
        .spawn((
            SimRef(e),
            global,
            RenderInstance {
                world_pos: [a[4], a[5]],
                size: QUAD,
                atlas_uv: [0.0, 0.0, 1.0, 1.0],
                tint: [1.0; 4],
                basis: [a[0], a[1], a[2], a[3]],
                texture_id: 0,
                premultiplied: 0.0,
                anchor: sprite.resolve_anchor(100.0),
                per_corner_tint: [[1.0; 4]; 4],
                opacity: 1.0,
                flip_uv: 0,
                z_order: 0,
                sampling: 0,
                uv_xform: RenderInstance::IDENTITY_UV_XFORM,
                clip_group: RenderInstance::CLIP_GROUP_NONE,
                clip_meta: 0,
                sub_order: 0,
            },
        ))
        .id();
    if dobrada {
        present.world_mut().entity_mut(p).insert(malha_dobrada());
    }
    (present, e.to_bits(), Camera2d::new([-1.2, 0.9], 6.0))
}

fn painter_em(modo: &str) -> PainterTool {
    let mut p = PainterTool::default();
    p.set_source(
        vec![255u8; (CANVAS.0 * CANVAS.1 * 4) as usize],
        CANVAS.0,
        CANVAS.1,
    );
    p.set_paint_tool_mode(modo);
    assert_eq!(p.active_paint_mode_id(), modo, "o modo {modo} nao pegou");
    // ⚠️ O raio de omissão (`~128` px) não cabe numa faixa de `200` px: o disco sairia da arte e o
    // anel partir-se-ia em arcos, que é outro gate (o da `ph2d-sprite-screen`).
    p.set_deform_size_norm(0.2);
    p
}

/// A UV que a porta do ponteiro do Painter dá a um ponto do ecrã, perguntada como um gesto que
/// COMEÇA — ⚠️ num traço já aberto ela responde também FORA da malha (pela lei do quad), e aqui a
/// pergunta é se o ponto está na arte desenhada. `None` fora dela.
fn uv_pelo_ponteiro(
    present: &mut PresentWorld,
    bits: u64,
    camera: &Camera2d,
    p: (f32, f32),
) -> Option<[f32; 2]> {
    let w = camera.screen_to_world(p, JANELA);
    match ph2d_render::mesh_uv(present.world_mut(), bits, w, true, [0.0, 0.0]) {
        MeshUv::Use { u, v, .. } => Some([u, v]),
        _ => None,
    }
}

/// ⭐⭐⭐ **O anel cai no raio do kernel, na aresta que a dobra estica e na que ela encolhe** — e o
/// anel de ANTES (o disco levado pela escala do quad de repouso) erra pelo que a dobra faz.
///
/// (Mutações: o anel com o raio do PINCEL de pintura (`size_px`) ⇒ RED; sem o ramo da malha — o
/// anel de antes — ⇒ RED pelo `None`.)
#[test]
fn the_liquify_ring_lies_where_the_kernel_deforms_on_bent_art() {
    let (mut present, bits, camera) = cena(true);
    let painter = painter_em("liquify");
    let raio = painter.brush_settings().deform_size_px;
    assert!(
        (8.0..35.0).contains(&raio),
        "o raio do Liquify ({raio}) nao e' o que a fixtura pediu"
    );
    let mut antes_pior = 0.0_f32;
    for centro_uv in [[0.5_f32, 0.2], [0.3, 0.5], [0.7, 0.8]] {
        let w = ph2d_render::drawn_mesh_of(present.world(), bits)
            .and_then(|m| m.world_at_uv(centro_uv))
            .expect("o centro esta' na arte");
        let cursor = camera.world_to_screen(w, JANELA);
        let centro = uv_pelo_ponteiro(&mut present, bits, &camera, cursor).expect("na arte");

        let arcos = anel_do_liquify(
            &painter,
            present.world(),
            bits,
            &camera,
            JANELA,
            cursor,
            0.0,
        )
        .expect("o Liquify sobre arte dobrada desenha o anel pela malha");
        assert_eq!(arcos.len(), 1, "o disco cabe inteiro na arte");
        assert_eq!(
            arcos[0].len(),
            ph2d_sprite_screen::LADOS_DO_ANEL as usize + 1,
            "e o anel fecha"
        );
        let mut pior = 0.0_f32;
        for p in &arcos[0] {
            let uv = uv_pelo_ponteiro(&mut present, bits, &camera, (p[0] as f32, p[1] as f32))
                .expect("um ponto do anel caiu fora da arte");
            let d = ((uv[0] - centro[0]) * CANVAS.0 as f32)
                .hypot((uv[1] - centro[1]) * CANVAS.1 as f32);
            pior = pior.max((d / raio - 1.0).abs());
        }

        // ⛔ O CONTROLO: o anel de antes — o disco pela escala do quad de repouso, à volta do cursor.
        let afim = ph2d_sprite_screen::sprite_image_to_screen_affine(
            CANVAS.0,
            CANVAS.1,
            pose(),
            &Sprite::atlas(0, QUAD, [1.0; 4]),
            None,
            &camera,
            JANELA,
        )
        .as_coeffs();
        let r_ecra = f64::from(raio) * afim[0].hypot(afim[1]);
        let mut antes = 0.0_f32;
        for i in 0..64 {
            let a = f64::from(i) / 64.0 * std::f64::consts::TAU;
            let p = (
                (f64::from(cursor.0) + r_ecra * a.cos()) as f32,
                (f64::from(cursor.1) + r_ecra * a.sin()) as f32,
            );
            if let Some(uv) = uv_pelo_ponteiro(&mut present, bits, &camera, p) {
                let d = ((uv[0] - centro[0]) * CANVAS.0 as f32)
                    .hypot((uv[1] - centro[1]) * CANVAS.1 as f32);
                antes = antes.max((d / raio - 1.0).abs());
            }
        }
        println!("centro {centro_uv:?}: anel {pior:.2e} | anel de antes {antes:.3}");
        assert!(
            pior < 2e-3,
            "o anel do Liquify erra o raio do kernel em {pior:.4}"
        );
        antes_pior = antes_pior.max(antes);
    }
    assert!(
        antes_pior > 0.15,
        "o anel de antes so' errava {antes_pior:.3} — a fixtura nao dobra o bastante"
    );
}

/// ⚠️ **Só o Deform pergunta à malha, e só quando ela existe** — um modo de PINTAR fica com a pegada
/// do motor (debaixo dele a arte está achatada, F6-s), e uma sprite sem malha fica com o anel de
/// sempre.
#[test]
fn only_the_deform_ring_asks_the_mesh_and_only_when_there_is_one() {
    let (mut present, bits, camera) = cena(true);
    let w = ph2d_render::drawn_mesh_of(present.world(), bits)
        .and_then(|m| m.world_at_uv([0.5, 0.5]))
        .expect("na arte");
    let cursor = camera.world_to_screen(w, JANELA);
    let pinta = painter_em("brush");
    assert!(
        anel_do_liquify(&pinta, present.world(), bits, &camera, JANELA, cursor, 0.0).is_none(),
        "um modo de pintar desenhou o anel do Liquify"
    );
    // Fora da arte: não há disco na malha, e o anel volta ao de sempre.
    let liquify = painter_em("liquify");
    assert!(
        anel_do_liquify(
            &liquify,
            present.world(),
            bits,
            &camera,
            JANELA,
            (3.0, 3.0),
            0.0
        )
        .is_none(),
        "fora da malha o anel e' o do quad"
    );
    assert!(uv_pelo_ponteiro(&mut present, bits, &camera, (3.0, 3.0)).is_none());
    // Sem malha (a sprite desenhada como quad): o anel de sempre.
    let (plano, bits_plano, camera_plano) = cena(false);
    assert!(
        anel_do_liquify(
            &liquify,
            plano.world(),
            bits_plano,
            &camera_plano,
            JANELA,
            cursor,
            0.0
        )
        .is_none(),
        "sobre uma sprite plana o anel do Liquify e' o de sempre"
    );
}
