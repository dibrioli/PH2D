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

/// A malha desenhada pela porta do produto — `costura` liga a costura (os anéis da borda).
fn desenhada(p: &Palco, costura: bool, placa: bool) -> SpriteMesh {
    let aneis = if costura {
        ph2d_skeleton_live::skin_image_fecho::aneis_da_borda(&p.mesh.tris)
    } else {
        Vec::new()
    };
    ph2d_skeleton_live::skin_image_fecho::malha_desenhada_com(
        p.mesh.clone(),
        p.p2l,
        &p.pele,
        &p.pesos,
        p.quad,
        &p.correcoes,
        &aneis,
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
/// ⚠️ **O alcance é a pergunta.** Um vão FECHADO (o risquinho) lê-se a `4 px`; junto de uma baía
/// ABERTA (um «V») só o alcance pequeno separa um fio da baía, cuja ponta é mais estreita que
/// `2·alcance` por geometria (medido a `120°`: `25` amostras a `4 px` com a forma certa).
fn buracos(m: &SpriteMesh, [x0, y0, x1, y1]: [f64; 4], alcance_px: f64) -> usize {
    buracos_de(m, [x0, y0, x1, y1], alcance_px, false)
}

/// A [`buracos`] com a régua escolhida: `tinta` conta como coberto só onde há TINTA (alfa `≥ 128`).
///
/// ⚠️ **As duas réguas respondem a perguntas diferentes.** A geométrica (`false`) mede a LEI — o vão
/// entre BORDAS da malha, que é o que a costura cose. A de tinta vê também a cúspide da ARTE onde a
/// tampa redonda encosta tangente noutra borda (a tampa vive dentro das células; a borda da malha ali
/// é a escada da grelha) — medido `24`–`35` amostras a `1 px` (`~2 px²`, a ponta tangente), com e
/// sem costura: um limite conhecido da lei, quase invisível na foto.
fn buracos_de(m: &SpriteMesh, [x0, y0, x1, y1]: [f64; 4], alcance_px: f64, tinta: bool) -> usize {
    assert!(m.skin.is_none(), "a régua lê posições POSADAS");
    let p = |i: u32| {
        let q = m.local[i as usize];
        [f64::from(q[0]), f64::from(q[1])]
    };
    let alcance = alcance_px / f64::from(PPM);
    let (ex0, ey0, ex1, ey1) = (x0, y0 - alcance, x1, y1 + alcance);
    let perto: Vec<([[f64; 2]; 3], [u32; 3])> = m
        .tris
        .iter()
        .map(|t| ([p(t[0]), p(t[1]), p(t[2])], *t))
        .filter(|(t, _)| {
            let (a, b) = (
                t.iter()
                    .fold([f64::MAX; 2], |c, q| [c[0].min(q[0]), c[1].min(q[1])]),
                t.iter()
                    .fold([f64::MIN; 2], |c, q| [c[0].max(q[0]), c[1].max(q[1])]),
            );
            a[0] <= ex1 && b[0] >= ex0 && a[1] <= ey1 && b[1] >= ey0
        })
        .collect();
    // ⭐ «Coberto» é TINTA: o triângulo que cobre o ponto amostra ali um texel com alfa `≥ 128`. ⛔ A
    // 1.ª régua contava qualquer triângulo, e a escada da grelha à volta de uma tampa (margem
    // transparente) lia-se como borda — um «fio» que na tela é a baía inteira.
    let alfa = alfa_da_arte();
    let coberto = |q: [f64; 2]| {
        perto.iter().any(|(t, i)| {
            if !dentro(q, t[0], t[1], t[2]) {
                return false;
            }
            if !tinta {
                return true;
            }
            let area = (t[1][0] - t[0][0]) * (t[2][1] - t[0][1])
                - (t[2][0] - t[0][0]) * (t[1][1] - t[0][1]);
            if area == 0.0 {
                return false;
            }
            let b1 = ((q[0] - t[0][0]) * (t[2][1] - t[0][1])
                - (t[2][0] - t[0][0]) * (q[1] - t[0][1]))
                / area;
            let b2 = ((t[1][0] - t[0][0]) * (q[1] - t[0][1])
                - (q[0] - t[0][0]) * (t[1][1] - t[0][1]))
                / area;
            let b0 = 1.0 - b1 - b2;
            let uv = |k: usize| m.uv[i[k] as usize];
            let u = b0 * f64::from(uv(0)[0]) + b1 * f64::from(uv(1)[0]) + b2 * f64::from(uv(2)[0]);
            let v = b0 * f64::from(uv(0)[1]) + b1 * f64::from(uv(1)[1]) + b2 * f64::from(uv(2)[1]);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "texel"
            )]
            let (x, y) = (
                ((u * f64::from(IMG_W)) as u32).min(IMG_W - 1),
                ((v * f64::from(IMG_H)) as u32).min(IMG_H - 1),
            );
            alfa[(y * IMG_W + x) as usize] >= 128
        })
    };
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
                    if std::env::var_os("SONDA_ONDE").is_some() && n % 4 == 1 {
                        eprintln!("   fio em ({x:.4}, {y:.4})");
                    }
                }
            }
            x += h;
        }
        y += h;
    }
    n
}

/// O alfa da arte da cena, uma vez.
fn alfa_da_arte() -> &'static [u8] {
    static ALFA: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    ALFA.get_or_init(|| pixels().iter().skip(3).step_by(4).copied().collect())
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

/// A área que a costura acrescenta, em texel² (a `100 %` um texel é um pixel).
fn area_cosida(p: &Palco) -> f64 {
    let (sem, com) = (desenhada(p, false, false), desenhada(p, true, false));
    let px2 = f64::from(PPM).powi(2);
    com.tris[sem.tris.len()..]
        .iter()
        .map(|t| {
            let q = t.map(|i| {
                [
                    f64::from(com.local[i as usize][0]),
                    f64::from(com.local[i as usize][1]),
                ]
            });
            ((q[1][0] - q[0][0]) * (q[2][1] - q[0][1]) - (q[2][0] - q[0][0]) * (q[1][1] - q[0][1]))
                .abs()
                / 2.0
                * px2
        })
        .sum()
}

/// ⭐⭐⭐ **A TINTA da costura é a das BEIRAS** — cada pedaço cosido (`4` pontos: dois da beira que
/// cose, dois da outra) parte de uma beira com arte (pelo menos dois texels com alfa `> 0`).
///
/// ⚠️ Não «os quatro»: a outra ponta pode cair na ESCADA da grelha à volta da tampa redonda, que é
/// margem transparente — medido na pose do report, o texel `(586, 1)` com alfa `0` —, e aí a costura
/// esmaece até ela (fotografado: sem névoa). ⛔ Sem isto, uma UV errada (o canto da imagem,
/// transparente) deixava o vão cosido com NADA e todos os gates de geometria verdes.
#[test]
fn a_tinta_da_costura_e_a_das_beiras() {
    let px = pixels();
    let novos = acrescentados(&palco(POSE_DO_REPORT));
    assert!(!novos.is_empty(), "a pose do report não coseu nada");
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "texel"
    )]
    let texel = |t: f32, n: u32| ((t * n as f32) as u32).min(n - 1);
    for pedaco in novos.chunks(4) {
        let com_arte = pedaco
            .iter()
            .filter(|(_, uv)| {
                let (x, y) = (texel(uv[0], IMG_W), texel(uv[1], IMG_H));
                px[((y * IMG_W + x) * 4 + 3) as usize] > 0
            })
            .count();
        assert!(
            com_arte >= 2,
            "um pedaço cosido sem beira de arte: {pedaco:?}"
        );
    }
}

/// ⭐⭐⭐ **Nenhuma pose à volta do report deixa o fio** — e é a resposta ao *«ora redonda ora
/// pontuda»* do dono: a costura é contínua na pose, logo a varredura fina não pode ter UMA pose com
/// fundo entalado. Alcance `1 px`: só um vão mais fino que `2 px` conta (a costura cose até `2`
/// texels, e a `100 %` um texel é um pixel); a baía de um «V» é mais larga e não conta.
#[test]
fn nenhuma_pose_a_volta_do_report_deixa_o_fio() {
    let mut com_fio = Vec::new();
    for k in 0..=24 {
        let g2 = -150.0 + 0.5 * k as f32;
        let p = palco((36.0, g2));
        let sem = buracos(&desenhada(&p, false, false), JANELA_DO_VAO, 1.0);
        let com = buracos(&desenhada(&p, true, false), JANELA_DO_VAO, 1.0);
        if com > 0 {
            com_fio.push((g2, sem, com));
        }
    }
    assert!(
        com_fio.is_empty(),
        "(pose, sem costura, com costura): {com_fio:?}"
    );
}

/// ⭐⭐⭐ **A costura nunca cose mais que `VAO_MAXIMO_EM_TEXELS`** — as pontas de cada troço caem
/// onde o vão REAL passa os dois texels (bissecção). ⛔ A interpolação linear da 1.ª redacção cosia
/// vãos de `3,9` texels (medido, `(36°, −145,2°)`), quando o ponto mais perto mudava de segmento
/// entre duas amostras; sem corte nenhum, `6,2`.
#[test]
fn a_costura_nunca_passa_dos_dois_texels() {
    let lei = ph2d_skeleton_live::skin_image_fecho::VAO_MAXIMO_EM_TEXELS;
    for k in 0..=24 {
        let g2 = -150.0 + 0.5 * k as f32;
        let v = maior_vao_cosido(&palco((36.0, g2)));
        assert!(
            v <= lei * (1.0 + 1e-3),
            "(36°, {g2}°): coseu um vão de {v:.4} texels"
        );
    }
}

/// ⭐⭐ **Onde os membros se SOBREPÕEM nada se cose por cima da tinta** — a costura só liga bordas
/// que se encaram de FORA. ⛔ Medido sem esse teste: `5`–`6` de `10` centros cosidos sobre tinta
/// de `−150°` a `−160°`, e `83` de `150` a `−144°`.
#[test]
fn onde_os_membros_se_sobrepoem_nada_se_cose_por_cima() {
    for g2 in [-144.0, -150.0, -155.0, -160.0] {
        let (sobre, de) = cosidos_sobre_tinta(&palco((36.0, g2)));
        assert_eq!(
            sobre, 0,
            "(36°, {g2}°): {sobre} de {de} centros cosidos sobre tinta"
        );
    }
}

/// ⭐⭐ **Os «V» das juntas ficam como a arte** — entre membros VIZINHOS nada se cose (a
/// `OSSOS_DE_DISTANCIA`), e a placa continua a posar.
#[test]
fn os_v_das_juntas_ficam_como_a_arte() {
    for pose in [(DOBRA, DOBRA), (DOBRA_FORTE, DOBRA_FORTE)] {
        let p = palco(pose);
        assert!(acrescentados(&p).is_empty(), "{pose:?}: coseu um «V»");
        assert!(
            desenhada(&p, true, true).skin.is_some(),
            "{pose:?}: a placa deixou de posar"
        );
    }
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
        // ⚠️ Os anéis fora do relógio: o produto guarda-os por malha (`bordas_da`).
        let todos = ph2d_skeleton_live::skin_image_fecho::aneis_da_borda(&p.mesh.tris);
        for (costura, placa) in [(false, true), (true, true), (false, false), (true, false)] {
            let aneis: &[Vec<u32>] = if costura { &todos } else { &[] };
            let mut melhor = std::time::Duration::MAX;
            let mut cosidos = 0;
            for _ in 0..7 {
                let t = std::time::Instant::now();
                let m = ph2d_skeleton_live::skin_image_fecho::malha_desenhada_com(
                    p.mesh.clone(),
                    p.p2l,
                    &p.pele,
                    &p.pesos,
                    p.quad,
                    &p.correcoes,
                    aneis,
                    placa,
                );
                melhor = melhor.min(t.elapsed());
                cosidos = m.as_ref().map_or(0, |m| m.tris.len() - p.mesh.tris.len());
                std::hint::black_box(m);
            }
            eprintln!(
                "{pose:?} costura {costura} placa {placa}: {melhor:?} (+{cosidos} triangulos)"
            );
        }
    }
}

/// ⏱️ **SONDA — a largura dos vãos entre partes da borda que se ENCARAM**, em texels, e o custo das
/// peças (anéis, borda posada). Por nó da borda: a distância ao segmento mais perto de OUTRA parte
/// (a mais de `SONDA_L` texels ao longo do anel), que fica do lado de FORA dos dois.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_a_largura_dos_vaos() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (g1, de, ate, passo) = (
        g("SONDA_G1", 36.0),
        g("SONDA_DE", -120.0),
        g("SONDA_ATE", -160.0),
        g("SONDA_PASSO", 4.0),
    );
    let l_tx = f64::from(g("SONDA_L", 6.0));
    let n = ((ate - de) / passo).abs().round() as i32;
    for k in 0..=n {
        let g2 = de + (ate - de).signum() * passo * k as f32;
        let p = palco((g1, g2));
        let t = std::time::Instant::now();
        let aneis = ph2d_skeleton_live::skin_image_fecho::aneis_da_borda(&p.mesh.tris);
        let t_aneis = t.elapsed();
        let m = desenhada(&p, false, false);
        let pos = |v: u32| {
            [
                f64::from(m.local[v as usize][0]),
                f64::from(m.local[v as usize][1]),
            ]
        };
        let [a, b, c, d, _, _] = p.p2l.0;
        let texel = (a * d - b * c).abs().sqrt();
        let area = |r: &[u32]| {
            (0..r.len())
                .map(|i| {
                    let (p0, p1) = (pos(r[i]), pos(r[(i + 1) % r.len()]));
                    p0[0] * p1[1] - p1[0] * p0[1]
                })
                .sum::<f64>()
                / 2.0
        };
        let sinal = aneis
            .iter()
            .map(|r| area(r))
            .max_by(|x, y| x.abs().total_cmp(&y.abs()))
            .unwrap_or(1.0)
            .signum();
        // Segmentos: (anel, índice, p0, p1, s0 = arco acumulado no anel).
        let mut segs = Vec::new();
        for (ri, r) in aneis.iter().enumerate() {
            let mut s = 0.0;
            for i in 0..r.len() {
                let (p0, p1) = (pos(r[i]), pos(r[(i + 1) % r.len()]));
                segs.push((ri, i, p0, p1, s));
                s += (p1[0] - p0[0]).hypot(p1[1] - p0[1]);
            }
        }
        let perim: Vec<f64> = aneis
            .iter()
            .enumerate()
            .map(|(ri, _)| {
                segs.iter()
                    .filter(|s| s.0 == ri)
                    .map(|s| (s.3[0] - s.2[0]).hypot(s.3[1] - s.2[1]))
                    .sum()
            })
            .collect();
        let normal = |p0: [f64; 2], p1: [f64; 2]| {
            let (tx, ty) = (p1[0] - p0[0], p1[1] - p0[1]);
            let l = tx.hypot(ty).max(1e-30);
            [ty / l * sinal, -tx / l * sinal]
        };
        let mut bins = [0usize; 6];
        let mut perfil = Vec::new();
        for (si, &(ri, _, p0, p1, s0)) in segs.iter().enumerate() {
            let v = p0;
            let nv = {
                let prev = segs[if si == 0 { segs.len() - 1 } else { si - 1 }];
                let a = normal(prev.2, prev.3);
                let b = normal(p0, p1);
                [a[0] + b[0], a[1] + b[1]]
            };
            let mut melhor = f64::INFINITY;
            for &(rj, _, q0, q1, s1) in &segs {
                if rj == ri {
                    let ds = (s1 - s0).abs();
                    if ds.min(perim[ri] - ds) < l_tx * texel {
                        continue;
                    }
                }
                let dd = [q1[0] - q0[0], q1[1] - q0[1]];
                let l2 = dd[0] * dd[0] + dd[1] * dd[1];
                let t = if l2 > 0.0 {
                    (((v[0] - q0[0]) * dd[0] + (v[1] - q0[1]) * dd[1]) / l2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let q = [q0[0] + t * dd[0], q0[1] + t * dd[1]];
                let w = [q[0] - v[0], q[1] - v[1]];
                let nq = normal(q0, q1);
                if w[0] * nv[0] + w[1] * nv[1] <= 0.0 || -(w[0] * nq[0] + w[1] * nq[1]) <= 0.0 {
                    continue;
                }
                melhor = melhor.min(w[0].hypot(w[1]) / texel);
            }
            let bin = [0.5, 1.0, 1.5, 2.0, 3.0, 6.0]
                .iter()
                .position(|&x| melhor < x);
            if let Some(b) = bin {
                bins[b] += 1;
            }
            if melhor < 6.0 {
                perfil.push(format!("{melhor:.2}"));
            }
        }
        let t = std::time::Instant::now();
        let ids: Vec<u32> = aneis.iter().flatten().copied().collect();
        let ossos = p.pesos.len() / p.mesh.rest.len();
        let linhas: Vec<f64> = ids
            .iter()
            .flat_map(|&v| {
                p.pesos[v as usize * ossos..(v as usize + 1) * ossos]
                    .iter()
                    .copied()
            })
            .collect();
        let so = ph2d_poly2d::Mesh2d {
            rest: ids.iter().map(|&v| p.mesh.rest[v as usize]).collect(),
            tris: Vec::new(),
            size: p.mesh.size,
        };
        let _ = ph2d_skeleton_live::skin_image::posed_sprite_mesh_corrigida(
            so,
            p.p2l,
            &p.pele,
            &linhas,
            p.quad[0],
            p.quad[1],
            &p.correcoes,
        );
        let t_posa = t.elapsed();
        eprintln!(
            "({g1}, {g2:.1}) aneis {t_aneis:?} posa-borda {t_posa:?} ({} nos)  <0.5:{} <1:{} <1.5:{} <2:{} <3:{} <6:{}  {:?}",
            ids.len(),
            bins[0],
            bins[1],
            bins[2],
            bins[3],
            bins[4],
            bins[5],
            if perfil.len() < 40 {
                perfil
            } else {
                vec![format!("{} nos", perfil.len())]
            }
        );
    }
}

/// ⏱️ **SONDA — a área cosida ao longo de uma varredura**, em texel².
/// `SONDA_DE=-146 SONDA_ATE=-140 SONDA_PASSO=0.1 cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture diag_a_area_cosida`
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_a_area_cosida() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (g1, de, ate, passo) = (
        g("SONDA_G1", 36.0),
        g("SONDA_DE", -160.0),
        g("SONDA_ATE", -138.0),
        g("SONDA_PASSO", 1.0),
    );
    let n = ((ate - de) / passo).abs().round() as i32;
    for k in 0..=n {
        let g2 = de + (ate - de).signum() * passo * k as f32;
        eprintln!("({g1}, {g2:.2}) {:.3}", area_cosida(&palco((g1, g2))));
    }
}

/// ⏱️ **SONDA — o que se VÊ na janela do vão**, por pose: fundo entalado a `1 px` (o fio) e a `4 px`
/// (a baía), com e sem costura.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_o_que_se_ve_no_vao() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (g1, de, ate, passo) = (
        g("SONDA_G1", 36.0),
        g("SONDA_DE", -147.0),
        g("SONDA_ATE", -142.0),
        g("SONDA_PASSO", 0.05),
    );
    let n = ((ate - de) / passo).abs().round() as i32;
    for k in 0..=n {
        let g2 = de + (ate - de).signum() * passo * k as f32;
        let p = palco((g1, g2));
        let (sem, com) = (desenhada(&p, false, false), desenhada(&p, true, false));
        let j = [
            g("SONDA_X0", -1.8).into(),
            g("SONDA_Y0", 0.3).into(),
            g("SONDA_X1", -0.5).into(),
            g("SONDA_Y1", 0.75).into(),
        ];
        eprintln!(
            "({g1}, {g2:.2}) fio {}->{} baia {}->{}",
            buracos_de(&sem, j, 1.0, true),
            buracos_de(&com, j, 1.0, true),
            buracos_de(&sem, j, 4.0, true),
            buracos_de(&com, j, 4.0, true)
        );
    }
}

/// Quantos centros de triângulo COSIDOS caem em cima de tinta da malha sem costura — e de quantos.
fn cosidos_sobre_tinta(p: &Palco) -> (usize, usize) {
    let (sem, com) = (desenhada(p, false, false), desenhada(p, true, false));
    let mut sobre = 0;
    let novos = &com.tris[sem.tris.len()..];
    for t in novos {
        let c = t.iter().fold([0.0, 0.0], |a, &i| {
            [
                a[0] + f64::from(com.local[i as usize][0]) / 3.0,
                a[1] + f64::from(com.local[i as usize][1]) / 3.0,
            ]
        });
        let h = 1e-4;
        if buracos(&sem, [c[0] - h, c[1] - h, c[0] + h, c[1] + h], 0.0) == 0 && tinta_em(&sem, c) {
            sobre += 1;
        }
    }
    (sobre, novos.len())
}

/// Há tinta da malha `m` no ponto `q`?
fn tinta_em(m: &SpriteMesh, q: [f64; 2]) -> bool {
    let p = |i: u32| {
        [
            f64::from(m.local[i as usize][0]),
            f64::from(m.local[i as usize][1]),
        ]
    };
    let alfa = alfa_da_arte();
    m.tris.iter().any(|t| {
        let (a, b, c) = (p(t[0]), p(t[1]), p(t[2]));
        if !dentro(q, a, b, c) {
            return false;
        }
        let area = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if area == 0.0 {
            return false;
        }
        let b1 = ((q[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (q[1] - a[1])) / area;
        let b2 = ((b[0] - a[0]) * (q[1] - a[1]) - (q[0] - a[0]) * (b[1] - a[1])) / area;
        let w = [1.0 - b1 - b2, b1, b2];
        let (mut u, mut v) = (0.0, 0.0);
        for k in 0..3 {
            u += w[k] * f64::from(m.uv[t[k] as usize][0]);
            v += w[k] * f64::from(m.uv[t[k] as usize][1]);
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "texel"
        )]
        let (x, y) = (
            ((u * f64::from(IMG_W)) as u32).min(IMG_W - 1),
            ((v * f64::from(IMG_H)) as u32).min(IMG_H - 1),
        );
        alfa[(y * IMG_W + x) as usize] >= 128
    })
}

/// ⏱️ **SONDA — continuidade e sobreposição**: a área cosida a passos de `SONDA_PASSO` e os centros
/// cosidos que caem em tinta.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_continuidade_e_sobreposicao() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (de, passo, n) = (
        g("SONDA_DE", -146.0),
        g("SONDA_PASSO", 0.0025),
        g("SONDA_N", 20.0) as i32,
    );
    let mut antes: Option<f64> = None;
    let mut pior = 0.0_f64;
    for k in 0..=n {
        let a = area_cosida(&palco((36.0, de + passo * k as f32)));
        if let Some(b) = antes {
            pior = pior.max((a - b).abs());
        }
        antes = Some(a);
    }
    eprintln!("continuidade: pior salto {pior:.4} texel2 em passos de {passo}°");
    for g2 in [-150.0, -152.0, -155.0, -158.0, -160.0, -144.0, -143.0] {
        let (s, t) = cosidos_sobre_tinta(&palco((36.0, g2)));
        eprintln!("sobreposicao (36, {g2}): {s} de {t} centros cosidos sobre tinta");
    }
}

/// O maior vão (em texels) que um pedaço cosido atravessa nas pontas — `|pa − qa|` e `|pb − qb|` dos
/// quatro pontos `[pa, pb, qb, qa]` de cada pedaço.
fn maior_vao_cosido(p: &Palco) -> f64 {
    let (sem, com) = (desenhada(p, false, false), desenhada(p, true, false));
    let [a, b, c, d, _, _] = p.p2l.0;
    let texel = (a * d - b * c).abs().sqrt();
    let q = |i: usize| [f64::from(com.local[i][0]), f64::from(com.local[i][1])];
    (sem.local.len()..com.local.len())
        .step_by(4)
        .flat_map(|k| [(k, k + 3), (k + 1, k + 2)])
        .map(|(i, j)| {
            let (u, v) = (q(i), q(j));
            (u[0] - v[0]).hypot(u[1] - v[1]) / texel
        })
        .fold(0.0, f64::max)
}

/// ⏱️ **SONDA — o maior vão cosido** numa varredura.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_o_maior_vao_cosido() {
    let mut pior = (0.0_f64, 0.0_f32);
    for k in 0..=120 {
        let g2 = -150.0 + 0.1 * k as f32;
        let v = maior_vao_cosido(&palco((36.0, g2)));
        if v > pior.0 {
            pior = (v, g2);
        }
    }
    eprintln!("maior vao cosido: {:.4} texel em (36, {})", pior.0, pior.1);
}
