//! Os gates dos EFEITOS DE VIZINHANÇA NA PEÇA (`docs/3D/30` §5 e §14, a W6) — a
//! pilha da peça da lição (a esfera de `768` faces), na CPU (a referência).
//!
//! ⚠️ A risca é a faixa `|x·n| < w` em torno de um círculo MÁXIMO oblíquo: ela
//! atravessa arestas e faces da malha em todos os ângulos, e por simetria da
//! esfera o desfoque dela é função só de `|x·n|` — logo uma amostra numa aresta
//! da malha e uma no meio de uma face, à mesma distância da risca, têm de ler o
//! mesmo. As cores são CINZENTOS (nem `0` nem `255`: pontos fixos da curva).

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;
use ph2d_tool_painter::{
    AdjustmentKind, AdjustmentParams, BloomParams, GaussianBlurParams, LayerId, SURFACE_RADIUS_MAX,
    SpatialUnits, adjustment_slider_params_in,
};

use crate::pilha_da_peca::{PilhaDaPeca, RecusaDaPilha, dobra};
use crate::vizinhanca_da_peca::unidades;

pub(crate) const CLARO: [u8; 4] = [214, 208, 196, 255];
pub(crate) const ESCURO: [u8; 4] = [38, 44, 70, 255];

/// A normal do plano do círculo máximo da risca (oblíqua à grelha da esfera).
fn normal() -> [f32; 3] {
    let n = [1.0f32, 2.0, 0.5];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    n.map(|c| c / l)
}

pub(crate) fn dist(x: [f32; 3]) -> f32 {
    let n = normal();
    (x[0] * n[0] + x[1] * n[1] + x[2] * n[2]).abs()
}

pub(crate) use crate::vizinhanca_da_peca::posicoes;

/// A peça da lição ao degrau `k`, com a BASE pintada pela cor de cada sítio.
pub(crate) fn peca(
    k: u8,
    cor: impl Fn([f32; 3]) -> [u8; 4],
) -> (Mesh, Tinta, PilhaDaPeca, Vec<[f32; 3]>) {
    let mesh = crate::scenes::tinta_fina::peca();
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    let tinta = Tinta::nova(mesh.vert_count(), faces(), k);
    let xs = posicoes(&tinta, &mesh);
    let mut p = PilhaDaPeca::de_tinta(&tinta);
    let base = p.base().expect("base");
    let px: Vec<[u8; 4]> = xs.iter().map(|&x| cor(x)).collect();
    p.plano_mut(base).expect("plano").escreve(&px, None);
    (mesh, tinta, p, xs)
}

fn risca(w: f32) -> impl Fn([f32; 3]) -> [u8; 4] {
    move |x| if dist(x) < w { ESCURO } else { CLARO }
}

/// A risca com a borda SUAVE (uma rampa de `largura` no mundo): a dura serrilha
/// na retícula — a fronteira dela cai a `±h/2` conforme o sítio — e o desfoque
/// herdava esse serrilhado (`3` degraus ao longo de arestas E no meio, medido).
pub(crate) fn risca_suave(w: f32, largura: f32) -> impl Fn([f32; 3]) -> [u8; 4] {
    move |x| {
        let t = ((dist(x) - w) / largura + 0.5).clamp(0.0, 1.0);
        let m = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
        [
            m(ESCURO[0], CLARO[0]),
            m(ESCURO[1], CLARO[1]),
            m(ESCURO[2], CLARO[2]),
            255,
        ]
    }
}

/// Um desfoque gaussiano de raio `raio` (unidades da peça) no topo.
pub(crate) fn desfoque(p: &mut PilhaDaPeca, raio: f32, tinta: &Tinta, mesh: &Mesh) -> LayerId {
    let id = p
        .novo_ajuste(AdjustmentKind::GaussianBlur, unidades(mesh))
        .expect("o desfoque serve na peça");
    p.define_parametros(
        id,
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: raio }),
    )
    .expect("parâmetros");
    p.garante_vizinhanca(tinta, mesh);
    id
}

fn canal(c: &[u8], i: usize) -> u8 {
    c[i * 4]
}

/// ⭐⭐⭐⭐ **O desfoque da peça é na SUPERFÍCIE** (plano §5): a risca alarga no
/// mundo e NADA vaza para longe dela — a mais de `w + 5σ` a peça fica como
/// estava, byte a byte.
///
/// ⛔ O CONTROLO que dá direito ao gate: o mesmo desfoque pela ORDEM das
/// amostras (a dobra lida como imagem, o raio em amostras) espalha a cor escura
/// por faces longe da risca — e a régua tem de o ver.
#[test]
fn o_desfoque_da_peca_e_na_superficie_e_nada_vaza() {
    let (w, raio) = (0.05f32, 0.15f32);
    let sigma = raio / 3.0;
    let (mesh, tinta, mut p, xs) = peca(5, risca(w));
    desfoque(&mut p, raio, &tinta, &mesh);
    let c = p.compor();
    let longe: Vec<usize> = (0..xs.len())
        .filter(|&i| dist(xs[i]) > w + 5.0 * sigma)
        .collect();
    let vaza = |c: &[u8]| {
        longe
            .iter()
            .filter(|&&i| canal(c, i).abs_diff(CLARO[0]) > 1)
            .count()
    };
    assert!(
        longe.len() > xs.len() / 2,
        "a fixtura tem de ter peça longe da risca"
    );
    assert_eq!(vaza(&c), 0, "a cor da risca chegou a amostras longe dela");
    // A risca ALARGOU: logo fora dela (`w < d < w + σ`) escureceu.
    let perto: Vec<u8> = (0..xs.len())
        .filter(|&i| dist(xs[i]) > w && dist(xs[i]) < w + sigma)
        .map(|i| canal(&c, i))
        .collect();
    let media = perto.iter().map(|&v| f32::from(v)).sum::<f32>() / perto.len() as f32;
    assert!(
        media < f32::from(CLARO[0]) - 40.0,
        "a risca não alargou no mundo (média {media} perto dela)"
    );
    // ⛔ O CONTROLO: pela ordem das amostras, com o raio em amostras.
    let h = 2.0 * std::f32::consts::PI / 32.0 / 32.0;
    let mut ordem = p.clone();
    let id = ordem.pilha().root()[0];
    ordem
        .define_parametros(
            id,
            AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: raio / h }),
        )
        .expect("parâmetros");
    let (l, a) = dobra(ordem.amostras());
    let pela_ordem = ph2d_tool_painter::composite(ordem.pilha(), &ordem, l, a);
    assert!(
        vaza(&pela_ordem) > longe.len() / 100,
        "CONTROLO: borrar pela ordem tinha de vazar ({} de {})",
        vaza(&pela_ordem),
        longe.len()
    );
}

/// ⭐⭐⭐⭐ **Uma ARESTA da malha não se vê** (plano §5, gate 1): cada amostra que
/// mora numa aresta ou num vértice lê o mesmo que a amostra do MEIO de uma face
/// mais perto dela em distância à risca (a simetria da esfera).
///
/// ⚠️ O mesmo par entre duas amostras do MEIO, longe uma da outra, é o
/// controlo: ele diz o que «o mesmo» quer dizer nesta malha. Medido a `32x`
/// (a risca com borda suave de `3h`): aresta pior `1`, `5,7 %` delas a `1`
/// degrau; meio-meio pior `1`, `0,2 %`. Com o estêncil de eixos nos quads (a
/// 1.ª lei) a aresta lia pior `3`, média `0,195` — o gate apanhou-o.
#[test]
fn uma_aresta_da_malha_le_o_mesmo_que_o_meio_de_uma_face() {
    let (w, raio) = (0.05f32, 0.15f32);
    let h = 2.0 * std::f32::consts::PI / 32.0 / 32.0;
    let (mesh, tinta, mut p, xs) = peca(5, risca_suave(w, 3.0 * h));
    desfoque(&mut p, raio, &tinta, &mesh);
    let c = p.compor();
    let fronteira = tinta.topologia().verts() + tinta.topologia().arestas_amostras() as usize;
    let mut meio: Vec<(f32, u8, usize)> = (fronteira..xs.len())
        .map(|i| (dist(xs[i]), canal(&c, i), i))
        .collect();
    meio.sort_by(|a, b| a.0.total_cmp(&b.0));
    // O desvio de `i` à amostra do meio à mesma distância da risca e LONGE dela
    // no mundo (a vizinha leria igual por estar perto, não por simetria).
    let desvio = |i: usize| -> Option<u8> {
        let d = dist(xs[i]);
        let k = meio.partition_point(|m| m.0 < d);
        let longe = |j: usize| {
            let (a, b) = (xs[i], xs[meio[j].2]);
            (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2) > 0.09
        };
        (k.saturating_sub(64)..(k + 64).min(meio.len()))
            .filter(|&j| longe(j))
            .min_by(|&a, &b| (meio[a].0 - d).abs().total_cmp(&(meio[b].0 - d).abs()))
            .filter(|&j| (meio[j].0 - d).abs() <= 2e-4)
            .map(|j| canal(&c, i).abs_diff(meio[j].1))
    };
    let perto = |i: &usize| dist(xs[*i]) <= w + 2.0 * raio;
    let aresta: Vec<u8> = (0..fronteira).filter(perto).filter_map(desvio).collect();
    let controlo: Vec<u8> = (fronteira..xs.len())
        .step_by(7)
        .filter(perto)
        .filter_map(desvio)
        .collect();
    let media = |v: &[u8]| v.iter().map(|&x| f64::from(x)).sum::<f64>() / v.len() as f64;
    assert!(
        aresta.len() > 500 && controlo.len() > 500,
        "a fixtura tem de pôr amostras na risca"
    );
    let pior = aresta.iter().copied().max().unwrap_or(0);
    let pior_controlo = controlo.iter().copied().max().unwrap_or(0);
    assert!(
        pior <= 1 && pior <= pior_controlo.max(1),
        "uma amostra de aresta afasta-se do meio da face em {pior} degraus (o meio: {pior_controlo})"
    );
    assert!(
        media(&aresta) <= 0.10,
        "{:.1} % das amostras de aresta a um degrau do meio",
        media(&aresta) * 100.0
    );
}

/// ⭐⭐⭐ **Um campo constante continua constante, ao bit**, e **raio zero é
/// NO-OP ao bit** (plano §5, gates 4 e 5).
#[test]
fn um_campo_constante_e_o_raio_zero_nao_mudam_um_bit() {
    let (mesh, tinta, mut p, _) = peca(4, |_| CLARO);
    let antes = p.compor();
    let id = desfoque(&mut p, 0.2, &tinta, &mesh);
    assert_eq!(p.compor(), antes, "o desfoque mudou um campo constante");
    let (mesh, tinta, mut p, _) = peca(4, risca(0.05));
    let antes = p.compor();
    desfoque(&mut p, 0.0, &tinta, &mesh);
    assert_eq!(p.compor(), antes, "raio zero mudou a peça");
    let _ = id;
}

/// ⭐⭐⭐ **O raio é do MUNDO** (plano §5, gate 2): a mesma risca a `8x` e a
/// `32x`, lida nos MESMOS sítios da superfície, dá o mesmo desfoque a `3 %`.
#[test]
fn o_mesmo_raio_desfoca_igual_a_8x_e_a_32x() {
    let (w, raio) = (0.1f32, 0.3f32);
    let leitura = |k: u8| {
        let (mesh, mut tinta, mut p, _) = peca(k, risca(w));
        desfoque(&mut p, raio, &tinta, &mesh);
        p.pinta_tinta(&mut tinta, Vec::new);
        mesh.faces()
            .iter()
            .enumerate()
            .step_by(3)
            .map(|(f, face)| {
                let c = face.verts();
                if c.len() == 3 {
                    tinta.cor_tri(f, c, [0.2, 0.3, 0.5])[0]
                } else {
                    tinta.cor_quad(f, c, [0.35, 0.6])[0]
                }
            })
            .collect::<Vec<f32>>()
    };
    let (a, b) = (leitura(3), leitura(5));
    let pior = a
        .iter()
        .zip(&b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0f32, f32::max);
    let variacao =
        b.iter().fold(0.0f32, |m, v| m.max(*v)) - b.iter().fold(1.0f32, |m, v| m.min(*v));
    assert!(
        variacao > 0.3,
        "a fixtura tem de ter a risca nos sítios lidos ({variacao})"
    );
    assert!(
        pior < 0.03,
        "8x e 32x afastam-se {pior} com o mesmo raio no mundo"
    );
}

/// ⭐⭐ **O alfa FEATHERA** (plano §5, gate 4): uma camada com uma mancha opaca
/// sobre transparente — depois do desfoque a cobertura espalha-se perto da
/// borda da mancha e só aí.
#[test]
fn o_alfa_da_camada_feathera_na_superficie() {
    let (raio, borda) = (0.15f32, 0.2f32);
    let sigma = raio / 3.0;
    let (mesh, tinta, mut p, xs) = peca(4, |_| CLARO);
    let base = p.base().expect("base");
    p.define_visivel(base, false);
    let mancha = p.nova_camada("mancha").expect("camada");
    let n = normal();
    let lado = |x: [f32; 3]| x[0] * n[0] + x[1] * n[1] + x[2] * n[2] - borda;
    let px: Vec<[u8; 4]> = xs
        .iter()
        .map(|&x| if lado(x) > 0.0 { ESCURO } else { [0; 4] })
        .collect();
    p.plano_mut(mancha).expect("plano").escreve(&px, None);
    desfoque(&mut p, raio, &tinta, &mesh);
    let c = p.compor();
    let mut meio_tom = 0;
    for (i, &x) in xs.iter().enumerate() {
        let a = c[i * 4 + 3];
        if a > 0 && a < 255 {
            meio_tom += 1;
            assert!(
                lado(x).abs() < 5.0 * sigma,
                "cobertura parcial a {} da borda (σ = {sigma})",
                lado(x)
            );
        }
        if lado(x) > 5.0 * sigma {
            assert_eq!(a, 255, "dentro da mancha a cobertura mudou");
        }
    }
    assert!(
        meio_tom > 100,
        "a cobertura não feathera ({meio_tom} amostras de meio-tom)"
    );
}

/// ⭐ **Na peça só os três do PLANO da imagem ficam de fora** — os de
/// vizinhança entram e a pilha continua sincronizada.
#[test]
fn so_os_ajustes_do_plano_da_imagem_ficam_de_fora() {
    let (mesh, _, mut p, _) = peca(3, |_| CLARO);
    let u = unidades(&mesh);
    for k in [
        AdjustmentKind::MotionBlur,
        AdjustmentKind::ChromaticAberration,
        AdjustmentKind::Halftone,
    ] {
        assert_eq!(p.novo_ajuste(k, u), Err(RecusaDaPilha::LeOPlanoDaImagem(k)));
    }
    for k in [
        AdjustmentKind::GaussianBlur,
        AdjustmentKind::Sharpen,
        AdjustmentKind::Bloom,
        AdjustmentKind::ShadowsHighlights,
    ] {
        assert!(p.novo_ajuste(k, u).is_ok(), "{k:?} serve na peça");
    }
    assert!(p.sincronizada());
}

/// ⭐⭐ **O raio de um ajuste novo nasce na unidade da peça com o slider onde o
/// 2D o põe**: o Bloom nasce a `20 %` do curso, que na peça é
/// `0,2 · SURFACE_RADIUS_MAX · diagonal` — e não `20` unidades.
#[test]
fn o_raio_novo_nasce_em_percentagem_da_peca() {
    let (mesh, _, mut p, _) = peca(3, |_| CLARO);
    let u = unidades(&mesh);
    let SpatialUnits::Surface { size } = u else {
        panic!("a peça mede-se em unidades dela")
    };
    let id = p.novo_ajuste(AdjustmentKind::Bloom, u).expect("bloom");
    let Some(ph2d_tool_painter::LayerKind::Adjustment(a)) = p.pilha().get(id).map(|c| &c.kind)
    else {
        panic!("ajuste")
    };
    let AdjustmentParams::Bloom(BloomParams { radius, .. }) = a.params else {
        panic!("bloom")
    };
    assert!(
        (radius - 0.2 * SURFACE_RADIUS_MAX * size).abs() < 1e-6,
        "raio {radius}"
    );
    assert!((adjustment_slider_params_in(&a.params, u)[2].1 - 0.2).abs() < 1e-6);
}
