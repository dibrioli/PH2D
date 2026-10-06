//! ⏱️ **SONDA — «GRUDAM»** (report do dono 2026-10-06, `PH2D_VEC_BONE_SMOKE=4` a `(36°, −144°)`):
//! *«partes da imagem onde a influência é de um osso grudam-se à parte do outro osso ao dobrar»* —
//! a borda de baixo da tampa redonda desce numa CAUDA colada à borda de cima do membro de baixo até à
//! ponta da cunha. Pela porta do QUADRO: a cena da `=4` (o `bind` da cena), a instância e a
//! `attach_skin_meshes`.
//!
//! `PH2D_VEC_BONE_DOBRA=36 PH2D_VEC_BONE_DOBRA2=-144 cargo test -p ph2d-app-vec --lib -- --ignored --nocapture diag_grudam`

use super::*;

use ph2d_ecs::{PresentWorld, SimRef};
use ph2d_skeleton_live::skin_image_arte::BordasDaMalha;

const PASTA: &str = "/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c6232662-5952-412a-8b96-880a517c6741/scratchpad/bug_grudam";
const JANELA: [f64; 4] = [-1.8, 0.3, -0.5, 0.75];

/// A cena `=4` pela porta da cena: a imagem e o esqueleto do 1.º tempo e o `bind` do 2.º (que dobra
/// pelas variáveis do smoke).
fn cena_real() -> (SimWorld, Entity) {
    let mut sim = SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::default();
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
    let raiz = esqueleto(&mut sim, PPM, o_img, "Image");
    st.bone_smoke_img = Some((bits, raiz));
    st.bone_smoke_pend = Some(Vec::new());
    super::super::bind(&mut scene, &mut sim, &assets, PPM, 4, &mut st);
    (sim, Entity::try_from_bits(bits).expect("imagem"))
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

/// Uma foto da janela a `1/4` px: a cor composta pela ordem dos triângulos sobre o fundo, e em cada
/// amostra o ÚLTIMO triângulo com tinta (alfa `≥ 128`).
pub(super) struct Foto {
    pub(super) w: usize,
    pub(super) a: usize,
    pub(super) cor: Vec<[f64; 3]>,
    pub(super) topo: Vec<Option<usize>>,
}

pub(super) fn foto(m: &SpriteMesh, [x0, y0, x1, y1]: [f64; 4]) -> Foto {
    let px = pixels();
    let h = 0.25 / f64::from(PPM);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "amostras"
    )]
    let (w, a) = (((x1 - x0) / h) as usize + 1, ((y1 - y0) / h) as usize + 1);
    let fundo = [0.34, 0.36, 0.42];
    let (mut cor, mut topo) = (vec![fundo; w * a], vec![None; w * a]);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "amostras"
    )]
    let ix = |x: f64, n: usize| ((x / h).max(0.0) as usize).min(n - 1);
    for (k, t) in m.tris.iter().enumerate() {
        let v = t.map(|i| m.local[i as usize].map(f64::from));
        let lo = v
            .iter()
            .fold([f64::MAX; 2], |c, q| [c[0].min(q[0]), c[1].min(q[1])]);
        let hi = v
            .iter()
            .fold([f64::MIN; 2], |c, q| [c[0].max(q[0]), c[1].max(q[1])]);
        if hi[0] < x0 || lo[0] > x1 || hi[1] < y0 || lo[1] > y1 {
            continue;
        }
        let area =
            (v[1][0] - v[0][0]) * (v[2][1] - v[0][1]) - (v[2][0] - v[0][0]) * (v[1][1] - v[0][1]);
        if area == 0.0 {
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
                let uv = [0, 1].map(|c| {
                    (0..3)
                        .map(|n| wb[n] * f64::from(m.uv[t[n] as usize][c]))
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
                if px[o + 3] >= 128 {
                    topo[j * w + i] = Some(k);
                }
            }
        }
    }
    Foto { w, a, cor, topo }
}

/// Grava a foto a `4×` por amostra (`16×` por pixel), `y` para cima; `marca(k)` tinge a magenta a
/// amostra cujo triângulo de cima é `k`.
pub(super) fn grava(nome: &str, f: &Foto, marca: &dyn Fn(usize) -> bool) {
    let s = 4;
    let (iw, ia) = (f.w * s, f.a * s);
    let mut img = vec![[0_u8; 3]; iw * ia];
    for j in 0..f.a {
        for i in 0..f.w {
            let k = j * f.w + i;
            let mut c = f.cor[k];
            if f.topo[k].is_some_and(marca) {
                c = [0.5 * c[0] + 0.5, 0.5 * c[1], 0.5 * c[2] + 0.5];
            }
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "cor"
            )]
            let c = c.map(|x| (x * 255.0).round().clamp(0.0, 255.0) as u8);
            for dy in 0..s {
                for dx in 0..s {
                    img[((f.a - 1 - j) * s + dy) * iw + i * s + dx] = c;
                }
            }
        }
    }
    let mut b = format!("P6\n{iw} {ia}\n255\n").into_bytes();
    b.extend(img.iter().flatten());
    std::fs::create_dir_all(PASTA).expect("pasta");
    std::fs::write(format!("{PASTA}/{nome}.ppm"), b).expect("ppm");
}

/// As três malhas pela porta do quadro: sem costura, a lei de antes (borda da malha) e a do produto
/// — e a do produto confere-se com a que a `attach_skin_meshes` deixa na instância.
pub(super) struct Malhas {
    pub(super) sim: SimWorld,
    pub(super) e: Entity,
    pub(super) assada: std::rc::Rc<ph2d_skeleton_live::skinned_mesh::SkinnedMesh>,
    pub(super) sem: SpriteMesh,
    pub(super) velha: SpriteMesh,
    pub(super) lei: SpriteMesh,
}

pub(super) fn malhas() -> Malhas {
    assert_eq!(
        (
            dobra_do_nivel(4),
            segunda_dobra(
                4,
                36.0,
                std::env::var("PH2D_VEC_BONE_DOBRA2").ok().as_deref()
            )
        ),
        (36.0, -144.0),
        "corra com PH2D_VEC_BONE_DOBRA=36 PH2D_VEC_BONE_DOBRA2=-144"
    );
    let (sim, e) = cena_real();
    let sprite = *sim.world().get::<Sprite>(e).expect("sprite");
    let inst = instancia(&sprite);
    let mut present = PresentWorld::new();
    let alvo = present.world_mut().spawn((SimRef(e), inst)).id();
    ph2d_skeleton_live::skin_image::attach_skin_meshes(&sim, &mut present, PPM, &[]);
    let quadro = present
        .world()
        .get::<SpriteMesh>(alvo)
        .cloned()
        .expect("o quadro desenha malha");
    let crua = ph2d_skeleton_live::skin_image::skinned_mesh_of(&sim, e).expect("malha");
    let assada = ph2d_skeleton_live::skin_bake_cache::desenhada_da_arte(&sim, e, &crua)
        .unwrap_or_else(|| std::rc::Rc::new(crua));
    let rect = [
        0.0,
        0.0,
        f64::from(assada.mesh.size[0]),
        f64::from(assada.mesh.size[1]),
    ];
    let p2l = ph2d_skeleton_live::skin_image::rect_to_quad(&sprite, rect, inst.anchor, inst.size)
        .expect("quad");
    let pele = ph2d_skeleton_live::skin_live::skin_of(&sim, e).expect("pele");
    let skin = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(e)
        .expect("bind");
    let pesos = skin.pesos_do_quadro(&assada.pesos).to_vec();
    let correcoes = skin.correcoes_resolvidas();
    let desenha = |b: &BordasDaMalha| {
        ph2d_skeleton_live::skin_image_fecho::malha_desenhada_com(
            assada.mesh.clone(),
            p2l,
            &pele,
            &pesos,
            [inst.anchor, inst.size],
            &correcoes,
            b,
            false,
        )
        .expect("desenha")
    };
    let sem = desenha(&BordasDaMalha::default());
    let velha = desenha(&BordasDaMalha::da(&assada.mesh, None));
    let lei = desenha(&ph2d_skeleton_live::skin_image_fecho::bordas_da(&assada));
    assert!(quadro.skin.is_none(), "com costura o quadro posa na CPU");
    assert_eq!(quadro, lei, "a malha do quadro não é a da porta");
    Malhas {
        sim,
        e,
        assada,
        sem,
        velha,
        lei,
    }
}

/// ⏱️ **As fotos**: sem costura, a lei de antes e a do produto, e a do produto com os triângulos
/// cosidos a magenta.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_grudam_as_fotos() {
    let m = malhas();
    let n = m.sem.local.len();
    for (nome, malha) in [
        ("real_sem_costura", &m.sem),
        ("real_lei_de_antes", &m.velha),
        ("real_lei", &m.lei),
    ] {
        let f = foto(malha, JANELA);
        grava(nome, &f, &|_| false);
        let cosido = |k: usize| malha.tris[k].iter().any(|&i| i as usize >= n);
        grava(&format!("{nome}_cosidos"), &f, &cosido);
    }
    // O palco dos gates é a mesma cena?
    let p = palco((36.0, -144.0));
    let d = desenhada(&p, false, false);
    let delta = d
        .local
        .iter()
        .zip(&m.sem.local)
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0_f32, f32::max);
    eprintln!(
        "palco vs cena real: {} vs {} pontos, {} vs {} triangulos, maior desvio {delta:e} m",
        d.local.len(),
        m.sem.local.len(),
        d.tris.len(),
        m.sem.tris.len()
    );
}

/// Amostras sem tinta ENTALADAS (tinta acima e abaixo na coluna) por coluna de `1` px.
fn vao_por_coluna(f: &Foto) -> Vec<usize> {
    (0..f.w / 4)
        .map(|c| {
            (4 * c..4 * c + 4)
                .map(|i| {
                    let col: Vec<bool> = (0..f.a).map(|j| f.topo[j * f.w + i].is_some()).collect();
                    let (Some(lo), Some(hi)) =
                        (col.iter().position(|&x| x), col.iter().rposition(|&x| x))
                    else {
                        return 0;
                    };
                    (lo..hi).filter(|&j| !col[j]).count()
                })
                .sum()
        })
        .collect()
}

/// ⏱️ **Os números**: (H1) o vão por coluna nas três leis e quanto dele a costura tapa; (H2) os pesos
/// dos nós da borda da malha na janela (a tampa encostada e o membro de baixo).
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_grudam_os_numeros() {
    let m = malhas();
    let n = m.sem.local.len();
    let (fs, fv, fl) = (
        foto(&m.sem, JANELA),
        foto(&m.velha, JANELA),
        foto(&m.lei, JANELA),
    );
    let (vs, vv, vl) = (
        vao_por_coluna(&fs),
        vao_por_coluna(&fv),
        vao_por_coluna(&fl),
    );
    let cosido_em = |f: &Foto, mm: &SpriteMesh, c: usize| {
        (4 * c..4 * c + 4)
            .flat_map(|i| (0..f.a).map(move |j| j * f.w + i))
            .filter(|&k| f.topo[k].is_some_and(|t| mm.tris[t].iter().any(|&i| i as usize >= n)))
            .count()
    };
    eprintln!(
        "x(m)     vao: sem  antes  lei | cosido: antes  lei   (amostras de 1/4 px; 4 = 1 px de altura)"
    );
    for c in 0..vs.len() {
        if vs[c] + vv[c] + vl[c] == 0 && c % 10 != 0 {
            continue;
        }
        #[expect(clippy::cast_precision_loss, reason = "colunas")]
        let x = JANELA[0] + c as f64 / f64::from(PPM);
        eprintln!(
            "{x:7.3}  {:4} {:5} {:4} | {:5} {:5}",
            vs[c],
            vv[c],
            vl[c],
            cosido_em(&fv, &m.velha, c),
            cosido_em(&fl, &m.lei, c)
        );
    }
    eprintln!(
        "TOTAL vao sem {} antes {} lei {}",
        vs.iter().sum::<usize>(),
        vv.iter().sum::<usize>(),
        vl.iter().sum::<usize>()
    );
    // H2: os pesos dos nós da borda na janela.
    let skin = m
        .sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(m.e)
        .expect("bind");
    let prof = ph2d_skeleton_live::esqueletos::profundidades(
        &m.sim,
        skin,
        &ph2d_skeleton_live::skin_live::bone_index(&m.sim),
    );
    let ossos = m.assada.pesos.len() / m.assada.mesh.rest.len();
    eprintln!("colunas -> profundidade {prof:?}");
    let aneis = ph2d_skeleton_live::skin_image_fecho::aneis_da_borda(&m.assada.mesh.tris);
    let mut hist = std::collections::BTreeMap::<(usize, u32), usize>::new();
    for &v in aneis.iter().flatten() {
        let q = m.sem.local[v as usize].map(f64::from);
        if q[0] < JANELA[0] || q[0] > JANELA[2] || q[1] < JANELA[1] || q[1] > JANELA[3] {
            continue;
        }
        let w = &m.assada.pesos[v as usize * ossos..(v as usize + 1) * ossos];
        let (dom, wmax) = w
            .iter()
            .enumerate()
            .fold((0, 0.0), |a, (j, &x)| if x > a.1 { (j, x) } else { a });
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "faixa"
        )]
        let faixa = (wmax * 100.0).floor() as u32;
        *hist.entry((dom, faixa.min(100) / 10 * 10)).or_default() += 1;
        let r = m.assada.mesh.rest[v as usize];
        if wmax < 0.999 {
            eprintln!(
                "  no {v:5} repouso ({:6.1},{:5.1}) px  posado ({:7.4},{:6.4}) m  pesos {:?}",
                r[0],
                r[1],
                q[0],
                q[1],
                w.iter()
                    .map(|x| (x * 1000.0).round() / 1000.0)
                    .collect::<Vec<_>>()
            );
        }
    }
    eprintln!("nos da borda na janela por (coluna dominante, peso dela em decis): {hist:?}");
    // Foto por OSSO: a amostra tingida pela coluna dominante do triângulo de cima.
    let dominante = |t: usize| -> usize {
        let tri = m.lei.tris[t];
        if tri.iter().any(|&i| i as usize >= n) {
            return 9;
        }
        let soma: Vec<f64> = (0..ossos)
            .map(|o| {
                tri.iter()
                    .map(|&i| m.assada.pesos[i as usize * ossos + o])
                    .sum()
            })
            .collect();
        soma.iter()
            .enumerate()
            .fold((0, 0.0), |a, (j, &x)| if x > a.1 { (j, x) } else { a })
            .0
    };
    let mut f = fl;
    let tinta = [[1.0, 0.3, 0.3], [0.3, 0.9, 0.3], [0.3, 0.4, 1.0]];
    for k in 0..f.cor.len() {
        if let Some(t) = f.topo[k] {
            let d = dominante(t);
            let c = if d == 9 {
                [1.0, 0.2, 1.0]
            } else {
                tinta[d.min(2)]
            };
            f.cor[k] = [0, 1, 2].map(|ch| 0.5 * f.cor[k][ch] + 0.5 * c[ch]);
        }
    }
    grava("real_lei_por_osso", &f, &|_| false);
}

/// ⏱️ **O que ladeia o fio que fica depois da ponte** (lei do produto, `x ∈ [−1,35; −1,22]`): para
/// cada amostra entalada, se a tinta logo acima e logo abaixo é de triângulo COSIDO ou da malha, e
/// a metade cosida (a do membro de cima, `si`, ou a do de baixo) pela linha de pesos dela.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_grudam_o_que_ladeia_o_fio() {
    let m = malhas();
    let n = m.sem.local.len();
    let f = foto(&m.lei, JANELA);
    let h = 0.25 / f64::from(PPM);
    let mut hist = std::collections::BTreeMap::<(&str, &str), usize>::new();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "amostras"
    )]
    let (i0, i1) = (
        ((-1.35 - JANELA[0]) / h) as usize,
        ((-1.22 - JANELA[0]) / h) as usize,
    );
    let tipo = |t: usize| {
        if m.lei.tris[t].iter().any(|&i| i as usize >= n) {
            "cosido"
        } else {
            "malha"
        }
    };
    for i in i0..i1 {
        let col: Vec<Option<usize>> = (0..f.a).map(|j| f.topo[j * f.w + i]).collect();
        let (Some(lo), Some(hi)) = (
            col.iter().position(Option::is_some),
            col.iter().rposition(Option::is_some),
        ) else {
            continue;
        };
        for j in lo..hi {
            if col[j].is_some() {
                continue;
            }
            let baixo = (lo..j).rev().find_map(|k| col[k]).map_or("-", tipo);
            let cima = (j..=hi).find_map(|k| col[k]).map_or("-", tipo);
            *hist.entry((baixo, cima)).or_default() += 1;
        }
    }
    eprintln!("amostras entaladas por (tinta de baixo, tinta de cima): {hist:?}");
    // Os pontos do anel da arte da TAMPA (osso da ponta) que caem em x ∈ [−1,36; −1,20] posados:
    // a que distância (px de repouso) ficam da tinta mais perto.
    let bordas = ph2d_skeleton_live::skin_image_fecho::bordas_da(&m.assada);
    let mascara = m.assada.mascara.as_ref().expect("mascara");
    let ossos = m.assada.pesos.len() / m.assada.mesh.rest.len();
    let mut dist = Vec::new();
    for pt in bordas.arte.iter().flatten() {
        let t = m.assada.mesh.tris[pt.tri as usize];
        let wb = [1.0 - pt.uv[0] - pt.uv[1], pt.uv[0], pt.uv[1]];
        let posado = [0, 1].map(|c| {
            (0..3)
                .map(|k| wb[k] * f64::from(m.sem.local[t[k] as usize][c]))
                .sum::<f64>()
        });
        let repouso = [0, 1].map(|c| {
            (0..3)
                .map(|k| wb[k] * m.assada.mesh.rest[t[k] as usize][c])
                .sum::<f64>()
        });
        let ponta: f64 = (0..3)
            .map(|k| wb[k] * m.assada.pesos[t[k] as usize * ossos])
            .sum();
        if !(-1.36..=-1.20).contains(&posado[0]) || posado[1] < 0.45 || ponta < 0.5 {
            continue;
        }
        let mut d = f64::INFINITY;
        for dy in -12..=12 {
            for dx in -12..=12 {
                let q = [
                    repouso[0].floor() + f64::from(dx) + 0.5,
                    repouso[1].floor() + f64::from(dy) + 0.5,
                ];
                if mascara.tem(q) {
                    d = d.min((q[0] - repouso[0]).hypot(q[1] - repouso[1]));
                }
            }
        }
        dist.push((posado[0], d, mascara.tem(repouso)));
    }
    dist.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (x, d, t) in &dist {
        eprintln!("  anel da tampa em x {x:7.4}: a {d:5.2} px da tinta (sobre tinta: {t})");
    }
}
