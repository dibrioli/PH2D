//! ⭐⭐⭐ **Cada membro de uma IMAGEM presa desenha só a SUA imagem** — decisão do dono (06/10):
//! *«mesmo se sobrepondo não puxe nada da imagem cuja influência é do outro osso»*. Onde dois membros
//! se sobrepõem, manda a ORDEM das faces (o osso de fora por cima); onde só se encostam pode ver-se
//! o contorno real de cada um. Pela porta do QUADRO: a cena `=4` (o `bind` dela), a instância e a
//! `attach_skin_meshes`.

use super::*;

use ph2d_ecs::{PresentWorld, SimRef};
use ph2d_render::{Sprite, SpriteMesh};

const PPM: f32 = 100.0;

/// A cena `=4` pela porta da cena — a imagem e o esqueleto do 1.º tempo, o [`bind`] do 2.º — com as
/// duas juntas depois postas em `(g1, g2)`.
struct Cena {
    sim: SimWorld,
    e: Entity,
}

fn cena((g1, g2): (f32, f32)) -> Cena {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::default();
    let assets = ph2d_asset::AssetDb::new();
    let mut st = crate::state::VecState::default();
    let (l, t) = peca(PPM);
    let [_, o_img] = origens(PPM, 4);
    let pixels_id = assets.insert_image_rgba8(IMG_W, IMG_H, pixels());
    #[expect(clippy::cast_possible_truncation, reason = "metros de uma cena")]
    let (_, bits) = ph2d_image_import::spawn_sprite(
        &mut sim,
        ph2d_image_import::PackedSource::Individual {
            texture_id: 1,
            pixels_id,
        },
        ph2d_core::Vec2::new(o_img[0] as f32, o_img[1] as f32),
        [l as f32, t as f32],
        "Image",
    );
    let raiz = esqueleto(&mut sim, PPM, o_img, "Image").expect("raiz");
    st.bone_smoke_img = Some((bits, Some(raiz)));
    st.bone_smoke_pend = Some(Vec::new());
    bind(&mut scene, &mut sim, &assets, PPM, 4, &mut st);
    // O `bind` dobra pelas variáveis do smoke; desfaz-se e põe-se a pose pedida.
    let a = dobra_do_nivel(4);
    let b = segunda_dobra(4, a, std::env::var("PH2D_VEC_BONE_DOBRA2").ok().as_deref());
    dobra_duas(&mut sim, raiz, -a, -b);
    dobra_duas(&mut sim, raiz, g1, g2);
    Cena {
        e: Entity::try_from_bits(bits).expect("imagem"),
        sim,
    }
}

fn instancia(s: &Sprite) -> ph2d_render::RenderInstance {
    ph2d_render::RenderInstance {
        world_pos: [0.0, 0.0],
        size: s.size,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: ph2d_render::RenderInstance::IDENTITY_BASIS,
        texture_id: 0,
        premultiplied: 0.0,
        anchor: s.resolve_anchor(PPM),
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: ph2d_render::RenderInstance::pack_flip_flags(s.flip_x, s.flip_y, false),
        z_order: 0,
        sampling: 0,
        uv_xform: ph2d_render::RenderInstance::IDENTITY_UV_XFORM,
        clip_group: ph2d_render::RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

/// O que o quadro deixa na instância da imagem.
fn no_quadro(c: &Cena) -> SpriteMesh {
    let sprite = *c.sim.world().get::<Sprite>(c.e).expect("sprite");
    let mut present = PresentWorld::new();
    let alvo = present
        .world_mut()
        .spawn((SimRef(c.e), instancia(&sprite)))
        .id();
    ph2d_skeleton_live::skin_image::attach_skin_meshes(&c.sim, &mut present, PPM, &[]);
    present
        .world()
        .get::<SpriteMesh>(alvo)
        .cloned()
        .expect("o quadro desenha malha")
}

/// A malha do bind que o quadro desenha (a assada, já na ordem dos ossos), os pesos do quadro e a
/// profundidade de cada coluna.
fn do_bind(c: &Cena) -> (ph2d_poly2d::Mesh2d, Vec<f64>, Vec<f64>) {
    let crua = ph2d_skeleton_live::skin_image::skinned_mesh_of(&c.sim, c.e).expect("malha");
    let assada =
        ph2d_skeleton_live::skin_bake_cache::assada_da_arte(&c.sim, c.e, &crua).unwrap_or(crua);
    let skin = c
        .sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(c.e)
        .expect("bind");
    let prof = ph2d_skeleton_live::esqueletos::profundidades(
        &c.sim,
        skin,
        &ph2d_skeleton_live::skin_live::bone_index(&c.sim),
    );
    let pesos = skin.pesos_do_quadro(&assada.pesos).to_vec();
    (assada.mesh, pesos, prof)
}

/// A malha `mesh` posada na CPU pela porta da pele, com o quad da instância.
fn posada(c: &Cena, mesh: ph2d_poly2d::Mesh2d, pesos: &[f64]) -> SpriteMesh {
    let sprite = *c.sim.world().get::<Sprite>(c.e).expect("sprite");
    let inst = instancia(&sprite);
    let rect = [0.0, 0.0, f64::from(mesh.size[0]), f64::from(mesh.size[1])];
    let p2l = ph2d_skeleton_live::skin_image::rect_to_quad(&sprite, rect, inst.anchor, inst.size)
        .expect("quad");
    let skin = c
        .sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(c.e)
        .expect("bind");
    ph2d_skeleton_live::skin_image::posed_sprite_mesh_corrigida(
        mesh,
        p2l,
        &ph2d_skeleton_live::skin_live::skin_of(&c.sim, c.e).expect("pele"),
        pesos,
        inst.anchor,
        inst.size,
        &skin.correcoes_resolvidas(),
    )
    .expect("posa")
}

/// As poses: a do report do dono e a varredura à volta (F48/F49/F59/A5-a), o vinco fechado e as
/// dobras de sempre.
fn poses() -> Vec<(f32, f32)> {
    let mut v: Vec<(f32, f32)> = (0..=12).map(|k| (36.0, -150.0 + k as f32)).collect();
    v.extend([
        (36.0, -149.5),
        (36.0, -155.0),
        (36.0, -160.0),
        (DOBRA, DOBRA),
        (DOBRA_FORTE, DOBRA_FORTE),
    ]);
    v
}

/// ⭐⭐⭐ **Nada de um membro é puxado para o outro**: em cada pose a malha que o quadro desenha é a
/// do BIND — os mesmos triângulos, na mesma ordem, e nenhum ponto a mais —, e cada ponto está onde a
/// pele o põe pelos SEUS pesos (a placa recebe o repouso; a CPU, a malha do bind posada). ⛔ O
/// CONTROLO é a lei recusada (a costura entre membros): a `(36°, −144°)` ela acrescentava
/// triângulos com pontos de dois membros.
#[test]
fn nada_de_um_membro_e_puxado_para_o_outro() {
    let mut erros = Vec::new();
    for pose in poses() {
        let c = cena(pose);
        let (mesh, pesos, _) = do_bind(&c);
        let q = no_quadro(&c);
        if q.tris != mesh.tris || q.local.len() != mesh.rest.len() {
            erros.push(format!(
                "{pose:?}: {} triângulos e {} pontos, o bind tem {} e {}",
                q.tris.len(),
                q.local.len(),
                mesh.tris.len(),
                mesh.rest.len()
            ));
            continue;
        }
        if q.skin.is_none() {
            let cpu = posada(&c, mesh, &pesos);
            if q.local != cpu.local {
                erros.push(format!("{pose:?}: um ponto saiu de onde a pele o põe"));
            }
        }
    }
    assert!(erros.is_empty(), "{erros:#?}");
}

fn dentro(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let s =
        |o: [f64; 2], u: [f64; 2]| (u[0] - o[0]) * (p[1] - o[1]) - (u[1] - o[1]) * (p[0] - o[0]);
    let (d1, d2, d3) = (s(a, b), s(b, c), s(c, a));
    !((d1 < 0.0 || d2 < 0.0 || d3 < 0.0) && (d1 > 0.0 || d2 > 0.0 || d3 > 0.0))
}

/// As violações da ordem dos ossos em `m`: pontos (a `5 px`) cobertos por faces de chaves que
/// diferem mais de `0,5` osso, onde a ÚLTIMA face desenhada não é a de chave máxima.
fn ordem_violada(m: &SpriteMesh, pesos: &[f64], prof: &[f64]) -> usize {
    let ossos = pesos.len() / m.local.len();
    assert_eq!(prof.len(), ossos, "uma profundidade por coluna");
    let chave_v: Vec<f64> = pesos
        .chunks_exact(ossos)
        .map(|w| w.iter().zip(prof).map(|(p, d)| p * d).sum::<f64>() / w.iter().sum::<f64>())
        .collect();
    let tris: Vec<([[f64; 2]; 3], [f64; 4], f64)> = m
        .tris
        .iter()
        .map(|t| {
            let q = t.map(|i| m.local[i as usize].map(f64::from));
            let c = [
                q.iter().map(|p| p[0]).fold(f64::MAX, f64::min),
                q.iter().map(|p| p[1]).fold(f64::MAX, f64::min),
                q.iter().map(|p| p[0]).fold(f64::MIN, f64::max),
                q.iter().map(|p| p[1]).fold(f64::MIN, f64::max),
            ];
            (
                q,
                c,
                t.iter().map(|&v| chave_v[v as usize]).sum::<f64>() / 3.0,
            )
        })
        .collect();
    let caixa = tris
        .iter()
        .fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |a, (_, c, _)| {
            [
                a[0].min(c[0]),
                a[1].min(c[1]),
                a[2].max(c[2]),
                a[3].max(c[3]),
            ]
        });
    let h = 5.0 / f64::from(PPM);
    let mut n = 0;
    let mut y = caixa[1];
    while y <= caixa[3] {
        let mut x = caixa[0];
        while x <= caixa[2] {
            let cobrem: Vec<f64> = tris
                .iter()
                .filter(|(q, c, _)| {
                    x >= c[0]
                        && x <= c[2]
                        && y >= c[1]
                        && y <= c[3]
                        && dentro([x, y], q[0], q[1], q[2])
                })
                .map(|(_, _, k)| *k)
                .collect();
            if let (Some(&ultima), Some(max), Some(min)) = (
                cobrem.last(),
                cobrem.iter().copied().reduce(f64::max),
                cobrem.iter().copied().reduce(f64::min),
            ) && max - min > 0.5
                && ultima < max - 1e-9
            {
                n += 1;
            }
            x += h;
        }
        y += h;
    }
    n
}

/// ⭐⭐⭐ **Onde dois membros se sobrepõem, o osso mais adiante pinta por cima** — ordem do dono de
/// 2026-10-02 (F48-c), e é ela que resolve a sobreposição desde que nada se cose. O controlo é a
/// ordem da GRELHA (a malha do bind antes de ordenar), que tem de violar — senão a pose não
/// sobrepõe.
#[test]
fn onde_os_membros_se_sobrepoem_o_osso_de_fora_pinta_por_cima() {
    for pose in [(DOBRA_FORTE, DOBRA_FORTE), (36.0, -144.0)] {
        let c = cena(pose);
        let crua = ph2d_skeleton_live::skin_image::skinned_mesh_of(&c.sim, c.e).expect("malha");
        let (_, _, prof) = do_bind(&c);
        let grelha = posada(&c, crua.mesh.clone(), &crua.pesos);
        let antes = ordem_violada(&grelha, &crua.pesos, &prof);
        assert!(antes > 0, "{pose:?}: o controlo não sobrepõe membros");
        let (mesh, pesos, _) = do_bind(&c);
        let depois = ordem_violada(&posada(&c, mesh, &pesos), &pesos, &prof);
        assert_eq!(
            depois, 0,
            "{pose:?}: {depois} pontos com o osso de trás por cima ({antes} na grelha)"
        );
    }
}

/// ⏱️ **SONDA — as fotos do que o quadro desenha** (CPU), a `16×` por pixel, com a textura sobre o
/// fundo da cena: `SONDA_PASTA=<dir> cargo test -p ph2d-app-vec --lib -- --ignored --nocapture diag_membros_fotos`.
/// Janelas: a cúspide da tampa e o vão do report F48, a `(36°, −144°)`, `−147°` e `−160°`.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_membros_fotos() {
    let pasta = std::env::var("SONDA_PASTA").unwrap_or_else(|_| "target/prova".into());
    std::fs::create_dir_all(&pasta).expect("pasta");
    let px = pixels();
    let h = 0.25 / f64::from(PPM);
    for g2 in [-144.0_f32, -147.0, -160.0] {
        let c = cena((36.0, g2));
        let (mesh, pesos, _) = do_bind(&c);
        let m = posada(&c, mesh, &pesos);
        for (nome, [x0, y0, x1, y1]) in [
            ("cuspide", [-1.8, 0.3, -0.5, 0.75]),
            ("vao_f48", [-1.30, 0.44, -0.90, 0.50]),
        ] {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "amostras"
            )]
            let (w, a) = (((x1 - x0) / h) as usize + 1, ((y1 - y0) / h) as usize + 1);
            let mut cor = vec![[0.34, 0.36, 0.42]; w * a];
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "amostras"
            )]
            let ix = |x: f64, n: usize| ((x / h).max(0.0) as usize).min(n - 1);
            for t in &m.tris {
                let v = t.map(|i| m.local[i as usize].map(f64::from));
                let lo = v
                    .iter()
                    .fold([f64::MAX; 2], |c, q| [c[0].min(q[0]), c[1].min(q[1])]);
                let hi = v
                    .iter()
                    .fold([f64::MIN; 2], |c, q| [c[0].max(q[0]), c[1].max(q[1])]);
                let area = (v[1][0] - v[0][0]) * (v[2][1] - v[0][1])
                    - (v[2][0] - v[0][0]) * (v[1][1] - v[0][1]);
                if hi[0] < x0 || lo[0] > x1 || hi[1] < y0 || lo[1] > y1 || area == 0.0 {
                    continue;
                }
                for j in ix(lo[1] - y0, a)..=ix(hi[1] - y0 + h, a) {
                    for i in ix(lo[0] - x0, w)..=ix(hi[0] - x0 + h, w) {
                        #[expect(clippy::cast_precision_loss, reason = "amostras")]
                        let q = [x0 + i as f64 * h, y0 + j as f64 * h];
                        if !dentro(q, v[0], v[1], v[2]) {
                            continue;
                        }
                        let b1 = ((q[0] - v[0][0]) * (v[2][1] - v[0][1])
                            - (v[2][0] - v[0][0]) * (q[1] - v[0][1]))
                            / area;
                        let b2 = ((v[1][0] - v[0][0]) * (q[1] - v[0][1])
                            - (q[0] - v[0][0]) * (v[1][1] - v[0][1]))
                            / area;
                        let wb = [1.0 - b1 - b2, b1, b2];
                        let uv = [0, 1].map(|k| {
                            (0..3)
                                .map(|n| wb[n] * f64::from(m.uv[t[n] as usize][k]))
                                .sum::<f64>()
                        });
                        #[expect(
                            clippy::cast_possible_truncation,
                            clippy::cast_sign_loss,
                            reason = "texel"
                        )]
                        let (x, y) = (
                            ((uv[0] * f64::from(IMG_W)) as u32).min(IMG_W - 1),
                            ((uv[1] * f64::from(IMG_H)) as u32).min(IMG_H - 1),
                        );
                        let o = ((y * IMG_W + x) * 4) as usize;
                        let al = f64::from(px[o + 3]) / 255.0;
                        let c = &mut cor[j * w + i];
                        for (ch, cc) in c.iter_mut().enumerate() {
                            *cc = *cc * (1.0 - al) + al * f64::from(px[o + ch]) / 255.0;
                        }
                    }
                }
            }
            let s = 4;
            let mut b = format!("P6\n{} {}\n255\n", w * s, a * s).into_bytes();
            for j in (0..a).rev() {
                for _ in 0..s {
                    for i in 0..w {
                        #[expect(
                            clippy::cast_possible_truncation,
                            clippy::cast_sign_loss,
                            reason = "cor"
                        )]
                        let c = cor[j * w + i].map(|x| (x * 255.0).round().clamp(0.0, 255.0) as u8);
                        for _ in 0..s {
                            b.extend(c);
                        }
                    }
                }
            }
            std::fs::write(format!("{pasta}/depois_{nome}_36_{g2}.ppm"), b).expect("ppm");
        }
    }
}
