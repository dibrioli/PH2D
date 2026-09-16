//! ⭐⭐⭐ **O ANEL MOSTRA ONDE O PINCEL PINTA** — sobre uma imagem desenhada como malha DOBRADA.
//!
//! A régua não sabe como o anel foi feito: cada ponto dele volta à imagem pela porta do PONTEIRO
//! ([`crate::uv_sob_o_ponteiro`], a que o pincel de protecção usa para pintar), e a distância ao
//! centro, em px de origem, tem de ser o raio do pincel.

use crate::UvSobOPonteiro;
use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef, SimWorld, Transform};
use ph2d_render::{Camera2d, RenderInstance, Sprite, SpriteMesh};

/// A imagem de origem: `400 × 200` px sobre um quad de `4 × 2` m.
const ORIGEM: (u32, u32) = (400, 200);
const QUAD: [f32; 2] = [4.0, 2.0];
/// O raio da dobra: a linha do meio da faixa enrola-se num arco de `R` m, logo a aresta de cima
/// estica `1 + (H/2)/R = 1,25×` e a de baixo encolhe `0,75×`.
const R: f32 = 4.0;
const JANELA: ph2d_host::WindowSize = ph2d_host::WindowSize {
    width: 800,
    height: 800,
};

/// A faixa enrolada: grelha de repouso `cols × rows`, com a UV da imagem e a pose do arco.
fn malha_dobrada(cols: u32, rows: u32) -> SpriteMesh {
    let mut m = SpriteMesh {
        local: Vec::new(),
        uv: Vec::new(),
        tris: Vec::new(),
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

/// ⚠️ **A pose da sprite: FORA da origem e RODADA.** Com a pose identidade, um mapa que esquecesse
/// a posição ou a base da instância passaria em todos os gates deste ficheiro (a fixtura alinhada
/// aos eixos não mede uma base).
fn pose() -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = 1.5;
    t.translation.y = -0.7;
    t.rotation = 0.4;
    t
}

/// A cena: a sprite na simulação e a instância DESENHADA como a faixa dobrada.
fn cena() -> (SimWorld, PresentWorld, u64, Camera2d) {
    cena_com(true)
}

/// A [`cena`], com ou sem a malha — sem ela a sprite é desenhada como o quad.
fn cena_com(dobrada: bool) -> (SimWorld, PresentWorld, u64, Camera2d) {
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
        present
            .world_mut()
            .entity_mut(p)
            .insert(malha_dobrada(32, 16));
    }
    (sim, present, e.to_bits(), Camera2d::new([1.5, -0.7], 6.0))
}

/// O pior erro relativo do raio de uma volta, medido pela porta do ponteiro — e quantos pontos
/// ela conseguiu ler.
fn erro_pela_porta(
    sim: &SimWorld,
    present: &mut PresentWorld,
    camera: &Camera2d,
    bits: u64,
    centro_uv: [f32; 2],
    raio: f32,
    pontos: &[[f64; 2]],
) -> (f32, usize) {
    let (mut pior, mut lidos) = (0.0_f32, 0usize);
    for p in pontos {
        let UvSobOPonteiro::Uv(u, v) = crate::uv_sob_o_ponteiro(
            sim,
            present.world_mut(),
            camera,
            JANELA,
            bits,
            p[0] as f32,
            p[1] as f32,
        ) else {
            continue;
        };
        let d = ((u - centro_uv[0]) * ORIGEM.0 as f32).hypot((v - centro_uv[1]) * ORIGEM.1 as f32);
        pior = pior.max((d / raio - 1.0).abs());
        lidos += 1;
    }
    (pior, lidos)
}

/// ⭐⭐⭐ **O ANEL CAI ONDE O PINCEL PINTA, na aresta que a dobra estica e na que ela encolhe** —
/// e o anel de ANTES (um círculo com a escala do quad de repouso) erra pelo que a dobra faz.
///
/// (Red-first: a porta a ignorar a malha — o afim do quad para todo ponto — ⇒ RED.)
#[test]
fn the_ring_lies_where_the_brush_paints_on_bent_art() {
    let (sim, mut present, bits, camera) = cena();
    const RAIO: f32 = 12.0;
    let mut antes_pior = 0.0_f32;
    for centro_uv in [[0.5_f32, 0.12], [0.3, 0.5], [0.7, 0.88]] {
        // O ponteiro: o centro levado ao ecrã pela malha desenhada.
        let malha =
            ph2d_render::drawn_mesh_of(present.world(), bits).expect("desenhada como malha");
        let w = malha
            .world_at_uv(centro_uv)
            .expect("o centro esta' na arte");
        let cursor = camera.world_to_screen(w, JANELA);

        let arcos = crate::anel_do_pincel(
            &sim,
            present.world_mut(),
            &camera,
            JANELA,
            bits,
            cursor,
            RAIO,
            ORIGEM,
        )
        .expect("o ponteiro esta' na arte");
        assert_eq!(
            arcos.len(),
            1,
            "o disco cabe inteiro na arte, logo o anel e' UM arco"
        );
        assert_eq!(
            arcos[0].len(),
            crate::LADOS_DO_ANEL as usize + 1,
            "e fechado"
        );
        let (erro, lidos) = erro_pela_porta(
            &sim,
            &mut present,
            &camera,
            bits,
            centro_uv,
            RAIO,
            &arcos[0],
        );
        assert_eq!(lidos, arcos[0].len(), "um ponto do anel caiu fora da arte");

        // ⛔ O CONTROLO: o anel de antes — redondo, com a escala do quad de repouso.
        let sprite = *sim
            .world()
            .get::<Sprite>(ph2d_ecs::Entity::from_bits(bits))
            .expect("sprite");
        let quad = crate::sprite_image_to_screen_affine(
            ORIGEM.0,
            ORIGEM.1,
            pose(),
            &sprite,
            None,
            &camera,
            JANELA,
        )
        .as_coeffs();
        let r_ecra = f64::from(RAIO) * quad[0].hypot(quad[1]);
        let antes: Vec<[f64; 2]> = (0..64)
            .map(|i| {
                let a = f64::from(i) / 64.0 * std::f64::consts::TAU;
                [
                    f64::from(cursor.0) + r_ecra * a.cos(),
                    f64::from(cursor.1) + r_ecra * a.sin(),
                ]
            })
            .collect();
        let (erro_antes, _) =
            erro_pela_porta(&sim, &mut present, &camera, bits, centro_uv, RAIO, &antes);
        println!("centro {centro_uv:?}: anel {erro:.2e} | anel de antes {erro_antes:.3}");
        assert!(erro < 2e-3, "o anel erra o raio do pincel em {erro:.4}");
        antes_pior = antes_pior.max(erro_antes);
    }
    assert!(
        antes_pior > 0.15,
        "o anel de antes so' errava {antes_pior:.3} — a fixtura nao dobra o bastante"
    );
}

/// ⭐ **Sem malha, o anel é o disco levado pelo AFIM do quad** — e cai onde o pincel pinta também.
#[test]
fn without_a_mesh_the_ring_follows_the_quad() {
    let (sim, mut present, bits, camera) = cena_com(false);
    let centro = crate::sprite_image_to_screen_affine(
        1,
        1,
        pose(),
        &Sprite::atlas(0, QUAD, [1.0; 4]),
        None,
        &camera,
        JANELA,
    ) * ph2d_vector::Point::new(0.3, 0.6);
    let arcos = crate::anel_do_pincel(
        &sim,
        present.world_mut(),
        &camera,
        JANELA,
        bits,
        (centro.x as f32, centro.y as f32),
        12.0,
        ORIGEM,
    )
    .expect("o ponteiro esta' no quad");
    assert_eq!(arcos.len(), 1);
    let (erro, lidos) = erro_pela_porta(
        &sim,
        &mut present,
        &camera,
        bits,
        [0.3, 0.6],
        12.0,
        &arcos[0],
    );
    assert_eq!(lidos, arcos[0].len());
    assert!(erro < 2e-3, "sem malha o anel erra {erro:.4}");
}

/// ⚠️ **Um disco que sai da arte parte o anel em ARCOS** — e com o ponteiro fora da arte não há
/// anel nenhum.
#[test]
fn a_disc_that_leaves_the_art_breaks_into_arcs_and_off_the_art_there_is_none() {
    let (sim, mut present, bits, camera) = cena();
    let malha = ph2d_render::drawn_mesh_of(present.world(), bits).expect("malha");
    let w = malha
        .world_at_uv([0.5, 0.02])
        .expect("junto da aresta de cima");
    let cursor = camera.world_to_screen(w, JANELA);
    let arcos = crate::anel_do_pincel(
        &sim,
        present.world_mut(),
        &camera,
        JANELA,
        bits,
        cursor,
        12.0,
        ORIGEM,
    )
    .expect("o ponteiro esta' na arte");
    assert_eq!(arcos.len(), 1, "a parte de fora corta UM arco da volta");
    assert!(
        arcos[0].len() < crate::LADOS_DO_ANEL as usize,
        "e ele nao fecha"
    );

    // Um ponto do ecrã longe da faixa.
    let fora = crate::anel_do_pincel(
        &sim,
        present.world_mut(),
        &camera,
        JANELA,
        bits,
        (5.0, 5.0),
        12.0,
        ORIGEM,
    );
    assert!(
        fora.is_none(),
        "fora da arte o pincel nao pinta, e o anel nao aparece"
    );
}

/// ⭐⭐ **A PORTA DE LEITURA DÁ O MESMO ANEL, ponto a ponto** — é a mesma lei, com a UV debaixo do
/// ponteiro achada pela metade inversa do mapa (`DrawnMesh::uv_at_world`) em vez da porta que precisa
/// de `&mut World`.
///
/// (Mutação: o `uv_at_world` a ignorar a base — o ponto de mundo lido como local ⇒ RED, porque a
/// fixtura tem o centro FORA da origem.)
#[test]
fn the_read_only_ring_is_the_same_ring() {
    let (sim, mut present, bits, camera) = cena();
    for centro_uv in [[0.5_f32, 0.12], [0.3, 0.5], [0.7, 0.88], [0.5, 0.02]] {
        let malha = ph2d_render::drawn_mesh_of(present.world(), bits).expect("malha");
        let w = malha.world_at_uv(centro_uv).expect("na arte");
        let cursor = camera.world_to_screen(w, JANELA);
        let leitura = crate::anel_na_malha(&malha, &camera, JANELA, cursor, 12.0, ORIGEM);
        let porta = crate::anel_do_pincel(
            &sim,
            present.world_mut(),
            &camera,
            JANELA,
            bits,
            cursor,
            12.0,
            ORIGEM,
        );
        assert!(
            leitura.is_some(),
            "o ponteiro esta' na arte ({centro_uv:?})"
        );
        assert_eq!(leitura, porta, "as duas portas discordam em {centro_uv:?}");
    }
    let malha = ph2d_render::drawn_mesh_of(present.world(), bits).expect("malha");
    assert!(
        crate::anel_na_malha(&malha, &camera, JANELA, (5.0, 5.0), 12.0, ORIGEM).is_none(),
        "fora da arte nao ha' anel"
    );
}
