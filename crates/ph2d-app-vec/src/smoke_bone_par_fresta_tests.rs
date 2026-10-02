//! ⭐⭐⭐ **O RISQUINHO da IMAGEM presa na cena `=4`** a `(36°, −144°)` (o aberto do handoff de
//! 2026-10-01 §6) — os gates da porta [`ph2d_skeleton_live::skin_image_fecho::malha_desenhada`]
//! sobre a imagem da cena, e a sonda que achou o mecanismo.
//!
//! ⚠️ O mecanismo e os números vivem no cabeçalho do `skin_image_fecho`: um VÃO entre dois membros
//! (a cor do fundo, pura), não uma fenda da malha.

use super::*;

use ph2d_ecs::Transform;
use ph2d_render::{Sprite, SpriteMesh};

const PPM: f32 = 100.0;

/// A pose do report (a do smoke do fecho de 2026-10-01).
const POSE_DO_REPORT: (f32, f32) = (36.0, -144.0);

/// O que a porta da malha recebe na cena, montado pelas portas do PRODUTO (o `bind_image` e a
/// [`super::dobra_duas`] da cena), com a sprite na origem.
struct Palco {
    mesh: ph2d_poly2d::Mesh2d,
    p2l: ph2d_skeleton::Xform,
    pele: ph2d_skeleton::Skin,
    pesos: Vec<f64>,
    quad: [[f32; 2]; 2],
    correcoes: Vec<ph2d_skeleton::Correccao>,
    /// A malha do BIND, na ordem da grelha — o controlo da ordem dos ossos.
    crua: ph2d_skeleton_live::skinned_mesh::SkinnedMesh,
}

fn palco((g1, g2): (f32, f32)) -> Palco {
    let mut sim = SimWorld::default();
    let (l, t) = peca(PPM);
    let raiz = esqueleto(&mut sim, PPM, [0.0, 0.0], "Image");
    #[expect(clippy::cast_possible_truncation, reason = "metros de uma cena")]
    let tamanho = [l as f32, t as f32];
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Sprite::atlas(0, tamanho, [1.0, 1.0, 1.0, 1.0]),
        ))
        .id();
    assert!(ph2d_skeleton_live::skin_image_bind::bind_image(
        &mut sim,
        e,
        &pixels(),
        [IMG_W, IMG_H],
        PPM,
        ph2d_poly2d::GridOptions::default(),
        raiz,
    ));
    dobra_duas(&mut sim, raiz.expect("raiz"), g1, g2);
    let crua = ph2d_skeleton_live::skin_image::skinned_mesh_of(&sim, e).expect("malha");
    let m = ph2d_skeleton_live::skin_bake_cache::assada_da_arte(&sim, e, &crua)
        .unwrap_or_else(|| crua.clone());
    let sprite = *sim.world().get::<Sprite>(e).expect("sprite");
    let anchor = sprite.resolve_anchor(PPM);
    let rect = [
        0.0,
        0.0,
        f64::from(m.mesh.size[0]),
        f64::from(m.mesh.size[1]),
    ];
    let p2l = ph2d_skeleton_live::skin_image::rect_to_quad(&sprite, rect, anchor, sprite.size)
        .expect("quad");
    let skin = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(e)
        .expect("bind");
    Palco {
        p2l,
        pele: ph2d_skeleton_live::skin_live::skin_of(&sim, e).expect("pele"),
        pesos: skin.pesos_do_quadro(&m.pesos).to_vec(),
        quad: [anchor, sprite.size],
        correcoes: skin.correcoes_resolvidas(),
        mesh: m.mesh,
        crua,
    }
}

fn desenhada(p: &Palco, contacto: bool, placa: bool) -> SpriteMesh {
    ph2d_skeleton_live::skin_image_fecho::malha_desenhada_com(
        p.mesh.clone(),
        p.p2l,
        &p.pele,
        &p.pesos,
        p.quad,
        &p.correcoes,
        contacto,
        placa,
    )
    .expect("a malha desenha-se")
}

fn dentro(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let s =
        |o: [f64; 2], u: [f64; 2]| (u[0] - o[0]) * (p[1] - o[1]) - (u[1] - o[1]) * (p[0] - o[0]);
    let (d1, d2, d3) = (s(a, b), s(b, c), s(c, a));
    let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(neg && pos)
}

/// ⭐⭐ **Os BURACOS da malha desenhada numa janela** `[x0, y0, x1, y1]` (metros locais): amostras a
/// `1/4` px de ecrã que nenhum triângulo cobre e que têm malha a `≤ alcance_px` POR CIMA e POR
/// BAIXO — o fundo a aparecer ENTRE peças; a borda de fora não conta.
///
/// ⛔ **Só serve a um vão FECHADO** (o risquinho, a `4 px`). Junto de uma baía ABERTA ela conta a
/// ponta do «V», que é mais estreita que `2·alcance` por geometria — a `120°`, com o enchimento
/// certo, contou `25` amostras a `4 px` e `1` a `1,25 px` (a ponta redonda da bola). A guarda do fio
/// junto da baía é o mecanismo, na `ph2d-vec-boolean`
/// (`a_uniao_do_fecho_da_borda_nao_desloca_a_polilinha`).
fn buracos(m: &SpriteMesh, [x0, y0, x1, y1]: [f64; 4], alcance_px: f64) -> usize {
    assert!(m.skin.is_none(), "a régua lê posições POSADAS");
    let p = |i: u32| {
        let q = m.local[i as usize];
        [f64::from(q[0]), f64::from(q[1])]
    };
    let alcance = alcance_px / f64::from(PPM);
    let (ex0, ey0, ex1, ey1) = (x0, y0 - alcance, x1, y1 + alcance);
    let perto: Vec<[[f64; 2]; 3]> = m
        .tris
        .iter()
        .map(|t| [p(t[0]), p(t[1]), p(t[2])])
        .filter(|t| {
            let (a, b) = (
                t.iter()
                    .fold([f64::MAX; 2], |c, q| [c[0].min(q[0]), c[1].min(q[1])]),
                t.iter()
                    .fold([f64::MIN; 2], |c, q| [c[0].max(q[0]), c[1].max(q[1])]),
            );
            a[0] <= ex1 && b[0] >= ex0 && a[1] <= ey1 && b[1] >= ey0
        })
        .collect();
    let coberto = |q: [f64; 2]| perto.iter().any(|t| dentro(q, t[0], t[1], t[2]));
    let h = 0.25 / f64::from(PPM);
    let mut n = 0;
    let mut y = y0;
    while y <= y1 {
        let mut x = x0;
        while x <= x1 {
            if !coberto([x, y]) {
                let lado =
                    |s: f64| (1..=16).any(|k| coberto([x, y + s * alcance * f64::from(k) / 16.0]));
                if lado(1.0) && lado(-1.0) {
                    n += 1;
                }
            }
            x += h;
        }
        y += h;
    }
    n
}

/// A janela do vão medido pela sonda (`x −1,23…−0,98`, `y 0,467…0,480`), com folga.
const JANELA_DO_VAO: [f64; 4] = [-1.30, 0.44, -0.90, 0.50];

/// ⭐⭐⭐ **O VÃO ENTRE OS MEMBROS DA IMAGEM FECHA** — e o controlo, com a lei desligada, mostra-o.
///
/// ⚠️ As duas metades: sem o controlo vermelho, uma janela no sítio errado passava por vácuo.
#[test]
fn o_vao_entre_os_membros_da_imagem_fecha() {
    let p = palco(POSE_DO_REPORT);
    let sem = buracos(&desenhada(&p, false, false), JANELA_DO_VAO, 4.0);
    assert!(
        sem > 50,
        "o controlo tem de mostrar o risquinho: {sem} amostras"
    );
    let com = buracos(&desenhada(&p, true, false), JANELA_DO_VAO, 4.0);
    assert_eq!(
        com, 0,
        "o fecho deixou {com} amostras do vão por cobrir ({sem} sem ele)"
    );
}

/// Os pontos e as UV que o fecho acrescentou (os vértices depois dos da malha).
fn acrescentados(p: &Palco) -> Vec<([f32; 2], [f32; 2])> {
    let (sem, com) = (desenhada(p, false, false), desenhada(p, true, false));
    (sem.local.len()..com.local.len())
        .map(|i| (com.local[i], com.uv[i]))
        .collect()
}

/// ⭐⭐⭐ **A TINTA do enchimento é a da BEIRA** — a UV de cada ponto acrescentado cai num texel com
/// arte (alfa `> 0`). ⚠️ Não «opaco»: a ponta da cunha encosta na tampa redonda, e ali a beira é a
/// borda suave da arte — medido, o texel `(557, 0)` com alfa `111`.
///
/// ⛔ Sem isto, uma UV errada (o canto da imagem, transparente) deixava o vão pintado de NADA e
/// todos os gates de geometria verdes.
#[test]
fn a_tinta_do_enchimento_e_a_da_beira() {
    let px = pixels();
    for pose in [POSE_DO_REPORT, (DOBRA_FORTE, DOBRA_FORTE)] {
        let novos = acrescentados(&palco(pose));
        assert!(!novos.is_empty(), "{pose:?}: o fecho não acrescentou nada");
        for (q, uv) in novos {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "texel"
            )]
            let texel = |t: f32, n: u32| ((t * n as f32) as u32).min(n - 1);
            let (x, y) = (texel(uv[0], IMG_W), texel(uv[1], IMG_H));
            let alfa = px[((y * IMG_W + x) * 4 + 3) as usize];
            assert!(
                alfa > 0,
                "{pose:?}: o ponto {q:?} pinta o texel ({x}, {y}) de alfa {alfa}"
            );
        }
    }
}

/// ⭐⭐ **Na dobra forte os bicos do «V» arredondam** — a bola rola por fora do contorno da imagem
/// como do desenho. ⚠️ Medido: `13,2 px²` em dois bicos (a `(36°, −144°)` o vão é uma ILHA, e
/// fecha pela outra metade da lei; sem este gate a bola de fora não teria guarda).
#[test]
fn na_dobra_forte_os_bicos_do_v_arredondam() {
    let novos = acrescentados(&palco((DOBRA_FORTE, DOBRA_FORTE)));
    assert!(
        !novos.is_empty(),
        "a dobra forte não arredondou nenhum bico"
    );
}

/// ⭐⭐⭐ **A dobra arredonda em TODA a varredura, sem saltar** — report do dono de 2026-10-02 com
/// duas fotos a `~0,5°` uma da outra: *«a dobra está inconsistente, ora redonda ora pontuda»*.
///
/// ⛔ Medido: a `(36°, −131,25°)` o vão do bico existia (`0,54` texel, `17,9` texel²) e saía com
/// `0` triângulos — a subtracção devolvia um anel com um FIO de área zero (sai por uma recta e
/// volta por ela), e a triangulação recusa um anel assim. Os vizinhos a `0,25°` arredondavam.
#[test]
fn a_dobra_arredonda_em_toda_a_varredura() {
    let mut sem_bico = Vec::new();
    for k in 0..=16 {
        let g2 = -128.0 - 0.25 * k as f32;
        if acrescentados(&palco((36.0, g2))).is_empty() {
            sem_bico.push(g2);
        }
    }
    assert!(sem_bico.is_empty(), "o bico ficou em ponta a {sem_bico:?}");
}

/// As violações da ordem dos ossos em `m`: pontos (a `5 px`) cobertos por faces de chaves que
/// diferem mais de `0,5` osso, onde a ÚLTIMA face desenhada não é a de chave máxima.
fn ordem_violada(m: &SpriteMesh, pesos: &[f64]) -> usize {
    let ossos = pesos.len() / m.local.len();
    let chave_v: Vec<f64> = pesos
        .chunks_exact(ossos)
        .map(|w| {
            w.iter().enumerate().map(|(j, p)| p * j as f64).sum::<f64>() / w.iter().sum::<f64>()
        })
        .collect();
    let pos = |i: u32| {
        [
            f64::from(m.local[i as usize][0]),
            f64::from(m.local[i as usize][1]),
        ]
    };
    let tris: Vec<([[f64; 2]; 3], [f64; 4], f64)> = m
        .tris
        .iter()
        .map(|t| {
            let q = t.map(pos);
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
/// 2026-10-02 (foto da dobra forte: os pedaços dos dois membros intercalavam-se). O controlo é a
/// ordem da GRELHA (a porta da CPU sem ordenar), que tem de violar — senão a pose não sobrepõe.
#[test]
fn onde_os_membros_se_sobrepoem_o_osso_de_fora_pinta_por_cima() {
    for pose in [(DOBRA_FORTE, DOBRA_FORTE), POSE_DO_REPORT] {
        let p = palco(pose);
        let grelha = ph2d_skeleton_live::skin_image::posed_sprite_mesh_corrigida(
            p.crua.mesh.clone(),
            p.p2l,
            &p.pele,
            &p.crua.pesos,
            p.quad[0],
            p.quad[1],
            &p.correcoes,
        )
        .expect("posa");
        let antes = ordem_violada(&grelha, &p.crua.pesos);
        assert!(antes > 0, "{pose:?}: o controlo não sobrepõe membros");
        let depois = ordem_violada(&desenhada(&p, false, false), &p.pesos);
        assert_eq!(
            depois, 0,
            "{pose:?}: {depois} pontos com o osso de trás por cima ({antes} na grelha)"
        );
    }
}

/// ⭐⭐ **Com a PLACA a posar, um quadro com vão desenha-se pela CPU** — o enchimento nasce no
/// espaço posado, e uma malha de repouso com ele colado desenharia o vão no sítio errado.
#[test]
fn com_vao_a_placa_cede_a_cpu_e_o_vao_fecha_igual() {
    let p = palco(POSE_DO_REPORT);
    let placa = desenhada(&p, true, true);
    assert_eq!(
        placa,
        desenhada(&p, true, false),
        "as duas portas têm de dar a MESMA malha"
    );
}

/// ⭐⭐⭐ **Sem vão nada muda, AO BIT** — nas duas portas: a placa continua a receber o repouso e a
/// tabela, e a CPU a malha de sempre. A pose é a da cena `=3` (`40°`), onde os membros não se tocam.
#[test]
fn sem_vao_a_malha_sai_ao_bit() {
    let p = palco((DOBRA, DOBRA));
    for placa in [false, true] {
        assert_eq!(
            desenhada(&p, true, placa),
            desenhada(&p, false, placa),
            "placa = {placa}: o fecho mudou uma malha sem vão"
        );
    }
    assert!(
        desenhada(&p, true, true).skin.is_some(),
        "a placa deixou de posar"
    );
}

/// ⏱️ **SONDA — o fundo entalado numa janela**, amostra a amostra, com a distância (px) à cobertura
/// por cima e por baixo.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_o_fundo_entalado() {
    let g = |k: &str, d: f64| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let p = palco((DOBRA_FORTE, DOBRA_FORTE));
    let m = desenhada(&p, g("SONDA_FECHO", 1.0) > 0.5, false);
    let pos = |i: u32| {
        let q = m.local[i as usize];
        [f64::from(q[0]), f64::from(q[1])]
    };
    let coberto = |q: [f64; 2]| {
        m.tris
            .iter()
            .any(|t| dentro(q, pos(t[0]), pos(t[1]), pos(t[2])))
    };
    let px = 1.0 / f64::from(PPM);
    let (x0, y0, x1, y1) = (
        g("SONDA_X0", -1.815),
        g("SONDA_Y0", 0.46),
        g("SONDA_X1", -1.659),
        g("SONDA_Y1", 0.54),
    );
    let mut y = y0;
    while y <= y1 {
        let mut linha = String::new();
        let mut x = x0;
        while x <= x1 {
            linha.push(if coberto([x, y]) { '#' } else { '.' });
            x += px / 2.0;
        }
        println!("{y:.4} {linha}");
        y += px / 2.0;
    }
}

/// ⏱️ **SONDA — o que o fecho ACRESCENTA numa pose**: cada triângulo a mais, com a área em px² de
/// ecrã a `100 %` e o sítio.
///
/// `SONDA_G1=<°> SONDA_G2=<°> cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture diag_o_que_o_fecho_acrescenta`
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_o_que_o_fecho_acrescenta() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let p = palco((
        g("SONDA_G1", POSE_DO_REPORT.0),
        g("SONDA_G2", POSE_DO_REPORT.1),
    ));
    let (sem, com) = (desenhada(&p, false, false), desenhada(&p, true, false));
    let px2 = f64::from(PPM).powi(2);
    let mut total = 0.0;
    for t in &com.tris[sem.tris.len()..] {
        let q = t.map(|i| {
            let v = com.local[i as usize];
            [f64::from(v[0]), f64::from(v[1])]
        });
        let a = ((q[1][0] - q[0][0]) * (q[2][1] - q[0][1])
            - (q[2][0] - q[0][0]) * (q[1][1] - q[0][1]))
            .abs()
            / 2.0
            * px2;
        total += a;
        let uv = t.map(|i| com.uv[i as usize]);
        println!("  tri {:?} area {a:.3} px2 uv {:?}", q[0], uv[0]);
    }
    println!(
        "acrescentou {} triangulos, {total:.3} px2",
        com.tris.len() - sem.tris.len()
    );
}

/// ⏱️ **SONDA — de onde vem o risquinho**: nós PENDURADOS (fenda entre vizinhos) e, onde a malha não
/// cobre, se os triângulos de cima e de baixo são vizinhos no repouso (fenda) ou de membros
/// diferentes (vão). Medido a `(36°, −144°)`: `0` pendurados em `9 091` triângulos, e os dois lados
/// de cada buraco a `~350 px` no repouso ⇒ um VÃO.
///
/// `SONDA_G1=<°> SONDA_G2=<°> cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture diag_de_onde_vem_o_risquinho`
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_de_onde_vem_o_risquinho() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let p = palco((
        g("SONDA_G1", POSE_DO_REPORT.0),
        g("SONDA_G2", POSE_DO_REPORT.1),
    ));
    let m = desenhada(&p, false, false);
    let rest = &p.mesh.rest;
    let mut pendurados = 0;
    for t in &p.mesh.tris {
        for k in 0..3 {
            let (a, b) = (rest[t[k] as usize], rest[t[(k + 1) % 3] as usize]);
            let l2 = (b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2);
            pendurados += rest
                .iter()
                .enumerate()
                .filter(|&(v, q)| {
                    let u = ((q[0] - a[0]) * (b[0] - a[0]) + (q[1] - a[1]) * (b[1] - a[1])) / l2;
                    let x = (q[0] - a[0]) * (b[1] - a[1]) - (q[1] - a[1]) * (b[0] - a[0]);
                    v as u32 != t[k]
                        && v as u32 != t[(k + 1) % 3]
                        && (1e-6..1.0 - 1e-6).contains(&u)
                        && x.abs() / l2.sqrt() < 1e-6
                })
                .count();
        }
    }
    println!(
        "malha: {} vertices, {} triangulos; pendurados: {pendurados}; buracos na janela do vao: {}",
        rest.len(),
        p.mesh.tris.len(),
        buracos(&m, JANELA_DO_VAO, 4.0)
    );
    let pos = |i: u32| {
        let q = m.local[i as usize];
        [f64::from(q[0]), f64::from(q[1])]
    };
    let quem = |q: [f64; 2]| {
        m.tris
            .iter()
            .position(|t| dentro(q, pos(t[0]), pos(t[1]), pos(t[2])))
    };
    let centro = |i: usize| {
        let t = p.mesh.tris[i];
        let r = t.map(|v| rest[v as usize]);
        [
            (r[0][0] + r[1][0] + r[2][0]) / 3.0,
            (r[0][1] + r[1][1] + r[2][1]) / 3.0,
        ]
    };
    let [x0, y0, x1, y1] = JANELA_DO_VAO;
    let h = 0.5 / f64::from(PPM);
    let mut y = y0;
    while y <= y1 {
        let mut x = x0;
        while x <= x1 {
            if quem([x, y]).is_none() {
                let viz =
                    |s: f64| (1..=16).find_map(|k| quem([x, y + s * 0.04 * f64::from(k) / 16.0]));
                if let (Some(a), Some(b)) = (viz(1.0), viz(-1.0)) {
                    let (ca, cb) = (centro(a), centro(b));
                    println!(
                        "  buraco ({x:.4}, {y:.4}) m: em cima o tri {a} (repouso {:.1},{:.1} px), em baixo o {b} ({:.1},{:.1} px) — {:.1} px no repouso",
                        ca[0],
                        ca[1],
                        cb[0],
                        cb[1],
                        (ca[0] - cb[0]).hypot(ca[1] - cb[1])
                    );
                }
            }
            x += h;
        }
        y += h;
    }
}

/// ⏱️ **SONDA — a varredura da 2.ª junta**: por pose, os vãos do fecho (`PH2D_BONE_LOG=1` imprime
/// cada um, com a espessura e o sítio) e os triângulos acrescentados.
///
/// `PH2D_BONE_LOG=1 SONDA_G1=36 SONDA_DE=-128 SONDA_ATE=-132 SONDA_PASSO=0.1 cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture diag_varre_a_segunda_junta`
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_varre_a_segunda_junta() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (g1, de, ate, passo) = (
        g("SONDA_G1", 36.0),
        g("SONDA_DE", -128.0),
        g("SONDA_ATE", -132.0),
        g("SONDA_PASSO", 0.1),
    );
    let n = ((ate - de) / passo).abs().round() as i32;
    for k in 0..=n {
        let g2 = de + (ate - de).signum() * passo * k as f32;
        eprintln!("== pose ({g1}, {g2:.2})");
        let p = palco((g1, g2));
        let (sem, com) = (desenhada(&p, false, false), desenhada(&p, true, false));
        eprintln!(
            "   acrescentou {} triangulos",
            com.tris.len() - sem.tris.len()
        );
    }
}

/// ⏱️ **SONDA — quanto custa a malha desenhada por quadro**, por porta (fecho on/off × placa on/off)
/// e por pose. `cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture diag_o_custo_da_malha_desenhada`
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_o_custo_da_malha_desenhada() {
    for pose in [
        (0.0, 0.0),
        (40.0, 40.0),
        (36.0, -131.25),
        (36.0, -144.0),
        (DOBRA_FORTE, DOBRA_FORTE),
    ] {
        let p = palco(pose);
        for (contacto, placa) in [(false, true), (true, true), (false, false), (true, false)] {
            let mut melhor = std::time::Duration::MAX;
            for _ in 0..7 {
                let t = std::time::Instant::now();
                let m = ph2d_skeleton_live::skin_image_fecho::malha_desenhada_com(
                    p.mesh.clone(),
                    p.p2l,
                    &p.pele,
                    &p.pesos,
                    p.quad,
                    &p.correcoes,
                    contacto,
                    placa,
                );
                melhor = melhor.min(t.elapsed());
                std::hint::black_box(m);
            }
            eprintln!("{pose:?} contacto {contacto} placa {placa}: {melhor:?}");
        }
    }
}
