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
    /// A profundidade de cada coluna na corrente (a coluna vem por `to_bits`, não pela corrente).
    prof: Vec<f64>,
    /// Onde há tinta (A5-a), guardada pelo bind.
    mascara: Option<ph2d_skeleton_live::skin_image_arte::Mascara>,
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
        prof: ph2d_skeleton_live::esqueletos::profundidades(
            &sim,
            skin,
            &ph2d_skeleton_live::skin_live::bone_index(&sim),
        ),
        mascara: m.mascara,
        mesh: m.mesh,
        crua,
    }
}

/// As bordas que o produto guarda para esta malha (`bordas_da`): os anéis e, com `arte`, o anel da
/// arte sobre a máscara do bind (A5-a); sem `costura`, nenhumas.
fn bordas(
    p: &Palco,
    costura: bool,
    arte: bool,
) -> ph2d_skeleton_live::skin_image_arte::BordasDaMalha {
    if !costura {
        return ph2d_skeleton_live::skin_image_arte::BordasDaMalha::default();
    }
    ph2d_skeleton_live::skin_image_arte::BordasDaMalha::da(
        &p.mesh,
        p.mascara.as_ref().filter(|_| arte),
    )
}

/// A malha desenhada pela porta do produto — `costura` liga a costura (com o anel da arte).
fn desenhada(p: &Palco, costura: bool, placa: bool) -> SpriteMesh {
    desenhada_com(p, &bordas(p, costura, true), placa)
}

/// A malha desenhada com as `bordas` dadas — o CONTROLO do A5-a passa a costura sem a arte.
fn desenhada_com(
    p: &Palco,
    aneis: &ph2d_skeleton_live::skin_image_arte::BordasDaMalha,
    placa: bool,
) -> SpriteMesh {
    ph2d_skeleton_live::skin_image_fecho::malha_desenhada_com(
        p.mesh.clone(),
        p.p2l,
        &p.pele,
        &p.pesos,
        p.quad,
        &p.correcoes,
        aneis,
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
    cosidos(&sem, &com)
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

/// ⭐⭐⭐ **A TINTA da costura é a das BEIRAS** — cada pedaço cosido são DUAS metades de `4` pontos,
/// cada uma a esticar a cor da SUA beira até ao meio do vão (A5-a: um quadrilátero só misturava a
/// UV dos dois membros e apanhava as pintas entre elas — FOTOGRAFADO na cúspide). Cada metade tem a
/// mesma UV nos dois pontos de cada ponta e parte de uma beira com arte (alfa `> 0` nos seus dois
/// pontos de beira, ou — onde a beira é a escada transparente da tampa — no da outra metade).
///
/// ⛔ Sem isto, uma UV errada (o canto da imagem, transparente) deixava o vão cosido com NADA e
/// todos os gates de geometria verdes.
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
    let arte = |uv: [f32; 2]| {
        let (x, y) = (texel(uv[0], IMG_W), texel(uv[1], IMG_H));
        px[((y * IMG_W + x) * 4 + 3) as usize] > 0
    };
    assert_eq!(novos.len() % 8, 0, "pedaços de duas metades de 4 pontos");
    for pedaco in novos.chunks(8) {
        for metade in pedaco.chunks(4) {
            // [beira a, beira b, meio b, meio a]: a UV do meio é a da beira do mesmo lado.
            assert_eq!(metade[0].1, metade[3].1, "a metade mistura UVs: {pedaco:?}");
            assert_eq!(metade[1].1, metade[2].1, "a metade mistura UVs: {pedaco:?}");
        }
        let com_arte = pedaco.iter().filter(|(_, uv)| arte(*uv)).count();
        assert!(
            com_arte >= 4,
            "um pedaço cosido sem beira de arte: {pedaco:?}"
        );
    }
}

/// ⭐⭐⭐ **Nenhuma pose à volta do report deixa mais VÃO que a lei de antes, e na cúspide fica
/// menos no total** — a lei é contínua na pose (o *«ora redonda ora pontuda»* do dono). Régua do VÃO
/// FIXO ([`reguas::vao_aberto`]): o vão é o da malha SEM costura (o fecho a `1 px` da tinta, menos
/// a tinta) e conta-se o que dele cada lei deixa sem tinta — mais tinta nunca sobe a conta. O
/// CONTROLO é a lei de antes (a costura sobre a borda da MALHA, a de um bind sem máscara).
///
/// Medido (27 poses, amostras de `1/4` px): cúspide `1852` contra `2368`, risquinho `75` contra
/// `161`; sem costura `3114` e `720`. ⛔ A régua de antes (fio na VERTICAL a `1 px`, `buracos_de`)
/// premiava o vão fechado EM PARTE: lia `12` contra `4` a `−149,5°`, onde o vão fixo lê `44` contra
/// `54`.
#[test]
fn nenhuma_pose_a_volta_do_report_deixa_mais_fio_que_a_lei_de_antes() {
    const CUSPIDE: [f64; 4] = [-1.8, 0.3, -0.5, 0.75];
    let mut poses: Vec<f32> = (0..=24).map(|k| -150.0 + 0.5 * k as f32).collect();
    poses.extend([-155.0, -160.0]);
    for (janela, nome) in [(JANELA_DO_VAO, "risquinho"), (CUSPIDE, "cúspide")] {
        let mut total = [0; 3];
        for &g2 in &poses {
            let p = palco((36.0, g2));
            let sem = desenhada(&p, false, false);
            let antes = desenhada_com(&p, &bordas(&p, true, false), false);
            let agora = desenhada(&p, true, false);
            let [s, v, n] = vao_aberto(&sem, [&sem, &antes, &agora], janela);
            assert!(
                n <= v,
                "{nome} (36°, {g2}°): a lei deixa {n} amostras de vão, a de antes {v}"
            );
            total = [total[0] + s, total[1] + v, total[2] + n];
        }
        let [s, v, n] = total;
        println!("  {nome}: vão — sem costura {s}, lei de antes {v}, lei {n}");
        assert!(v < s, "o CONTROLO: a lei de antes não fecha nada na {nome}");
        if nome == "cúspide" {
            assert!(n < v, "{nome}: a lei deixa {n}, a de antes {v}");
        } else {
            assert!(n <= v, "{nome}: a lei deixa {n}, a de antes {v}");
        }
    }
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

/// ⭐⭐⭐ **Onde os membros se sobrepõem, a costura não tapa À VISTA a tinta de outro membro** —
/// cada triângulo cosido entra à profundidade do SEU membro, e a metade do membro de trás fica por
/// baixo da tinta do da frente. Régua que conhece a ORDEM ([`reguas::visiveis_sobre_tinta`]). O
/// CONTROLO é a mesma malha com a costura no FIM (a ordem de antes): medido `4`, `2` e `3` amostras
/// a `−144,5°`, `−145,5°` e `−147,5°`. ⛔ A régua geométrica de antes (centros cosidos sobre tinta)
/// era cega à ordem e ficava vermelha com a lei certa.
#[test]
fn onde_os_membros_se_sobrepoem_nada_se_cose_por_cima() {
    for g2 in [-144.5, -145.5, -147.5, -155.0, -160.0] {
        let p = palco((36.0, g2));
        let (sem, com) = (desenhada(&p, false, false), desenhada(&p, true, false));
        let vistas = visiveis_sobre_tinta(&sem, &com);
        assert_eq!(
            vistas, 0,
            "(36°, {g2}°): {vistas} amostras de tinta tapadas"
        );
        if g2 > -150.0 {
            let controlo = visiveis_sobre_tinta(&sem, &no_fim(&sem, &com));
            assert!(
                controlo > 0,
                "(36°, {g2}°): o CONTROLO (costura no fim) não tapa nada"
            );
        }
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
fn ordem_violada(m: &SpriteMesh, pesos: &[f64], prof: &[f64]) -> usize {
    let ossos = pesos.len() / m.local.len();
    assert_eq!(prof.len(), ossos, "uma profundidade por coluna");
    let chave_v: Vec<f64> = pesos
        .chunks_exact(ossos)
        .map(|w| w.iter().zip(prof).map(|(p, d)| p * d).sum::<f64>() / w.iter().sum::<f64>())
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
        let antes = ordem_violada(&grelha, &p.crua.pesos, &p.prof);
        assert!(antes > 0, "{pose:?}: o controlo não sobrepõe membros");
        let depois = ordem_violada(&desenhada(&p, false, false), &p.pesos, &p.prof);
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

#[path = "smoke_bone_par_fresta_reguas.rs"]
mod reguas;
use reguas::*;

#[cfg(test)]
#[path = "smoke_bone_par_fresta_sondas.rs"]
mod sondas;
