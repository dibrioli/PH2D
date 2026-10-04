//! A2 — a parte da FRENTE tapa as riscas da de TRÁS numa dobra forte.
//!
//! ⚠️ **A régua não usa a chave de osso da lei:** a frente é o MEMBRO da ponta (repouso `x > 20 + M`)
//! posado pela pele, e uma risca de trás é a do membro da raiz (`x < 20 − M`); entre os dois fica a
//! zona da junta, que a régua não julga — uma risca de trás por baixo DELA não conta como à vista,
//! e a trás «à vista» começa a `2M` da junta: a `150°` a dobra comprime o membro de trás sobre si
//! mesmo até `x ≈ 15,3`, onde a ordem dos pedaços é a da lei e não da régua (MEDIDO). Uma amostra conta como TINTA quando o desenho tem um
//! contorno ABERTO a menos de [`PERTO`] dela e PARALELO a ela (uma risca da frente que a cruza não
//! a pinta).

use crate::skin_desenho::Leis;
use crate::skin_live::tests::palco;
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_scene::VecPath;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};

/// A junta da fixtura (`palco`: barra `40 × 10`, dois ossos de `20`).
const JUNTA: f64 = 20.0;
/// A meia-largura da zona da junta que a régua não julga.
const M: f64 = 4.0;
/// Distância a que uma amostra se lê pintada.
const PERTO: f64 = 0.1;
/// Folga da fronteira da frente: perto dela a régua não julga.
const FOLGA: f64 = 0.25;

pub(crate) fn cubica(vs: &[ph2d_vec_scene::VecVertex], k: usize) -> [[f64; 2]; 4] {
    [
        vs[k].anchor,
        vs[k].out_handle,
        vs[k + 1].in_handle,
        vs[k + 1].anchor,
    ]
}

pub(crate) fn em(c: &[[f64; 2]; 4], t: f64) -> ([f64; 2], [f64; 2]) {
    let s = 1.0 - t;
    let p = |k: usize| {
        s * s * s * c[0][k]
            + 3.0 * s * s * t * c[1][k]
            + 3.0 * s * t * t * c[2][k]
            + t * t * t * c[3][k]
    };
    let d = |k: usize| {
        3.0 * s * s * (c[1][k] - c[0][k])
            + 6.0 * s * t * (c[2][k] - c[1][k])
            + 3.0 * t * t * (c[3][k] - c[2][k])
    };
    ([p(0), p(1)], [d(0), d(1)])
}

/// Amostras `(ponto, tangente)` dos contornos ABERTOS (ou dos fechados, `fechados = true`).
pub(crate) fn amostras(p: &VecPath, fechados: bool, por_seg: usize) -> Vec<([f64; 2], [f64; 2])> {
    let mut out = Vec::new();
    for c in 0..p.contour_count() {
        let Some((vs, fechado)) = p.contour(c) else {
            continue;
        };
        if fechado != fechados || vs.len() < 2 {
            continue;
        }
        for k in 0..vs.len() - 1 {
            let cb = cubica(vs, k);
            for i in 0..=por_seg {
                #[expect(clippy::cast_precision_loss, reason = "amostra")]
                out.push(em(&cb, i as f64 / por_seg as f64));
            }
        }
    }
    out
}

/// Os troços entre amostras consecutivas dos contornos ABERTOS.
fn amostras_em_troco(p: &VecPath) -> Vec<[[f64; 2]; 2]> {
    let mut out = Vec::new();
    for c in 0..p.contour_count() {
        let Some((vs, false)) = p.contour(c) else {
            continue;
        };
        for k in 0..vs.len().saturating_sub(1) {
            let cb = cubica(vs, k);
            let pts: Vec<[f64; 2]> = (0..=64).map(|i| em(&cb, f64::from(i) / 64.0).0).collect();
            out.extend(pts.windows(2).map(|w| [w[0], w[1]]));
        }
    }
    out
}

pub(crate) fn dentro(pol: &[[f64; 2]], q: [f64; 2]) -> bool {
    let mut d = false;
    for i in 0..pol.len() {
        let (a, b) = (pol[i], pol[(i + 1) % pol.len()]);
        if (a[1] > q[1]) != (b[1] > q[1])
            && q[0] < (b[0] - a[0]) * (q[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            d = !d;
        }
    }
    d
}

pub(crate) fn dist_pol(pol: &[[f64; 2]], q: [f64; 2]) -> f64 {
    (0..pol.len())
        .map(|i| {
            let (a, b) = (pol[i], pol[(i + 1) % pol.len()]);
            let ab = [b[0] - a[0], b[1] - a[1]];
            let l2 = (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-18);
            let t = (((q[0] - a[0]) * ab[0] + (q[1] - a[1]) * ab[1]) / l2).clamp(0.0, 1.0);
            (q[0] - a[0] - t * ab[0]).hypot(q[1] - a[1] - t * ab[1])
        })
        .fold(f64::MAX, f64::min)
}

/// `(tapadas pintadas / tapadas, frente pintada / frente, trás à vista pintada / trás à vista,
/// tapadas)` — o desenho com e sem a lei da frente (a do contacto ligada nos dois).
fn mede(graus: f32, frente: bool) -> (f64, f64, f64, usize) {
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    scene.path_mut(id).expect("path").effects = vec![FxEntry::new(PathEffect::Hatch(
        ph2d_vec_scene::fx_hatch::HatchSpec {
            angle: 45.0,
            spacing: 8.0,
            cross: false,
        },
    ))];
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None),
        1
    );
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let campo = g.campo.clone().expect("campo");
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ponta)
        .expect("Transform")
        .rotation = graus.to_radians();
    let index = crate::skin_live::bone_index(&sim);
    let pele = crate::skin_live::resolve(&sim, &skin, e, &index).expect("pele");
    let posa = |p: [f64; 2]| {
        let mut w = pele.scratch();
        pele.weights_corrected(p, campo.linha(p).as_deref(), &mut w, &[]);
        pele.blend(p, &w)
    };
    let leis = Leis {
        frente,
        ..Leis::do_ambiente()
    };
    let d = crate::skin_live::recook_leis(&sim, &mut scene.clone(), leis)
        .remove(&id)
        .expect("desenho");
    // ⚠️ A tinta é a POLILINHA das amostras, e não os pontos: uma cúbica com uma alça só corre
    // depressa numa ponta, e entre duas amostras o vão passava da régua (`0,39`, MEDIDO).
    let tinta: Vec<[[f64; 2]; 2]> = amostras_em_troco(&d);
    let pintada = |(p, t): ([f64; 2], [f64; 2])| {
        let q = posa(p);
        let q2 = posa([p[0] + 1e-4 * t[0], p[1] + 1e-4 * t[1]]);
        let dir = [q2[0] - q[0], q2[1] - q[1]];
        let nd = dir[0].hypot(dir[1]).max(1e-18);
        tinta.iter().any(|&[a, b]| {
            let u = [b[0] - a[0], b[1] - a[1]];
            let nu = u[0].hypot(u[1]).max(1e-18);
            dist_pol(&[a, b], q) < PERTO
                && ((dir[0] * u[0] + dir[1] * u[1]) / (nd * nu)).abs() > 8f64.to_radians().cos()
        })
    };
    // O repouso `x > x0` da barra, posado pela pele: a FRENTE (`x0 = JUNTA + M`) e tudo o que está
    // adiante da parte de trás (`x0 = JUNTA − M`, a zona da junta incluída).
    let posado_desde = |x0: f64| -> Vec<[f64; 2]> {
        (0..=100)
            .map(|i| [x0 + (40.0 - x0) * f64::from(i) / 100.0, 0.0])
            .chain((0..=100).map(|i| [40.0, 10.0 * f64::from(i) / 100.0]))
            .chain((0..=100).map(|i| [40.0 - (40.0 - x0) * f64::from(i) / 100.0, 10.0]))
            .chain((0..=100).map(|i| [x0, 10.0 - 10.0 * f64::from(i) / 100.0]))
            .map(&posa)
            .collect()
    };
    let (frente, adiante) = (posado_desde(JUNTA + M), posado_desde(JUNTA - M));
    let riscas = amostras(&g.path, false, 400);
    let (mut tap, mut tap_p, mut fr, mut fr_p, mut vis, mut vis_p) = (0, 0, 0, 0, 0, 0);
    for s in riscas {
        let x = s.0[0];
        if x > JUNTA + M {
            fr += 1;
            fr_p += usize::from(pintada(s));
        } else if x < JUNTA - M {
            let q = posa(s.0);
            if dentro(&frente, q) && dist_pol(&frente, q) > FOLGA {
                tap += 1;
                tap_p += usize::from(pintada(s));
            } else if x < JUNTA - 2.0 * M && !dentro(&adiante, q) && dist_pol(&adiante, q) > FOLGA {
                vis += 1;
                vis_p += usize::from(pintada(s));
            }
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "contagens")]
    let r = |a: usize, b: usize| a as f64 / b.max(1) as f64;
    (r(tap_p, tap), r(fr_p, fr), r(vis_p, vis), tap)
}

/// ⭐⭐⭐ **GATE — numa dobra forte as riscas de TRÁS não pintam por cima da FRENTE, e nada mais se
/// apaga.** Varrida a `110°`, `130°`, `150°`.
///
/// ⛔ **O CONTROLO:** sem a lei da frente as mesmas riscas tapadas pintam-se (o defeito
/// FOTOGRAFADO na `=5` a `110°`).
#[test]
fn as_riscas_de_tras_nao_pintam_por_cima_da_frente() {
    for graus in [110.0, 130.0, 150.0] {
        let (tap, fr, vis, n) = mede(graus, true);
        let (tap0, fr0, vis0, n0) = mede(graus, false);
        println!(
            "  {graus}°: tapadas {n} pintadas {tap:.3} (sem a lei {tap0:.3}, {n0}) · frente {fr:.3} \
             ({fr0:.3}) · trás à vista {vis:.3} ({vis0:.3})"
        );
        assert!(n > 20, "a {graus}° a fixtura não tem riscas tapadas ({n})");
        assert!(
            tap0 > 0.9,
            "o CONTROLO: sem a lei as tapadas não se pintam ({tap0:.3})"
        );
        assert!(
            tap < 0.02,
            "a {graus}° as riscas de trás pintam a frente ({tap:.3})"
        );
        // ⚠️ A régua lê `~2 %` das amostras sem tinta TAMBÉM sem a lei (as pontas das riscas) ⇒
        // «nada mais se apaga» compara com o controlo.
        assert!(
            fr > 0.95 && fr >= fr0 - 1e-3,
            "a {graus}° a frente perdeu riscas ({fr:.3} × {fr0:.3})"
        );
        assert!(
            vis > 0.95 && vis >= vis0 - 1e-3,
            "a {graus}° a trás à vista perdeu riscas ({vis:.3} × {vis0:.3})"
        );
    }
}

/// ⭐ **SONDA — O PREÇO por quadro** do recorte (só a porta, e dela só a malha posada) e do quadro
/// inteiro com e sem a lei da frente, a pose a MUDAR a cada chamada: na dobra (`110°` ↔ `150°`) e
/// SEM dobra que tape (`0°` ↔ `30°`, A7). Imprime; corra em `--release` com a máquina calma.
#[test]
fn diag_o_preco_do_recorte_por_quadro() {
    for poses in [(110f32, 150f32), (0.0, 30.0)] {
        preco_do_recorte(poses);
    }
}

fn preco_do_recorte((pa, pb): (f32, f32)) {
    use std::time::Instant;
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    scene.path_mut(id).expect("path").effects = vec![FxEntry::new(PathEffect::Hatch(
        ph2d_vec_scene::fx_hatch::HatchSpec {
            angle: 45.0,
            spacing: 4.0,
            cross: false,
        },
    ))];
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None),
        1
    );
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let campo = g.campo.clone().expect("campo");
    let indice = ph2d_vec_skin::pesos::IndiceDoCampo::novo(&campo.malha);
    let mut n = 0_u32;
    let mut dobra = |sim: &mut ph2d_ecs::SimWorld| {
        n += 1;
        sim.world_mut()
            .get_mut::<ph2d_ecs::Transform>(ponta)
            .expect("Transform")
            .rotation = if n.is_multiple_of(2) { pa } else { pb }.to_radians();
    };
    let mut porta_e_malha = |so_a_malha: bool| {
        let t = Instant::now();
        let mut k = 0_u32;
        while t.elapsed().as_millis() < 300 {
            dobra(&mut sim);
            let index = crate::skin_live::bone_index(&sim);
            let pele = crate::skin_live::resolve(&sim, &skin, e, &index).expect("pele");
            let prof = crate::esqueletos::profundidades(&sim, &skin, &index);
            if so_a_malha {
                let _ = super::Posada::nova(&campo, indice.as_ref(), &pele, &[], true, &prof);
            } else {
                let _ = super::so_o_que_se_ve(
                    &g.path,
                    &g.pesos,
                    (&campo, indice.as_ref()),
                    (&pele, &[], true),
                    &prof,
                );
            }
            k += 1;
        }
        t.elapsed().as_secs_f64() * 1e6 / f64::from(k)
    };
    let (porta, malha) = (porta_e_malha(false), porta_e_malha(true));
    let mut quadro = |frente: bool| {
        let leis = Leis {
            frente,
            ..Leis::do_ambiente()
        };
        let mut sc = scene.clone();
        let t = Instant::now();
        let mut k = 0_u32;
        while t.elapsed().as_millis() < 300 {
            dobra(&mut sim);
            let _ = crate::skin_live::recook_leis(&sim, &mut sc, leis);
            k += 1;
        }
        t.elapsed().as_secs_f64() * 1e6 / f64::from(k)
    };
    let (com, sem) = (quadro(true), quadro(false));
    println!(
        "  {pa}°↔{pb}°: riscas {} · malha {} triângulos · µs: o recorte {porta:.1} (a malha posada \
         {malha:.1}) · o quadro com a lei {com:.1} · sem ela {sem:.1} · loadavg {}",
        g.path.contour_count() - 1,
        campo.malha.tris.len(),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// ⭐⭐ **GATE — o que o recorte não corta sai AO BIT**: o contorno fechado e cada risca inteira à
/// vista, com os nós e as linhas de peso da fonte (o bake lê a tabela fora da malha; uma linha
/// deslocada daria pesos plausíveis ao nó errado). ⛔ **O CONTROLO:** a `150°` alguma risca é
/// mesmo cortada.
#[test]
fn o_que_nao_se_corta_sai_ao_bit() {
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    scene.path_mut(id).expect("path").effects = vec![FxEntry::new(PathEffect::Hatch(
        ph2d_vec_scene::fx_hatch::HatchSpec {
            angle: 45.0,
            spacing: 8.0,
            cross: false,
        },
    ))];
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None),
        1
    );
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let campo = g.campo.clone().expect("campo");
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ponta)
        .expect("Transform")
        .rotation = 150f32.to_radians();
    let index = crate::skin_live::bone_index(&sim);
    let pele = crate::skin_live::resolve(&sim, &skin, e, &index).expect("pele");
    let prof = crate::esqueletos::profundidades(&sim, &skin, &index);
    let (p, t) =
        super::so_o_que_se_ve(&g.path, &g.pesos, (&campo, None), (&pele, &[], true), &prof)
            .expect("o CONTROLO: nada foi cortado a 150°");
    let n = g.pesos.len() / (3 * g.path.verts_all().count());
    let linhas =
        |q: &VecPath, tab: &[f64]| -> Vec<(Vec<ph2d_vec_scene::VecVertex>, bool, Vec<f64>)> {
            let mut base = 0;
            (0..q.contour_count())
                .filter_map(|c| q.contour(c))
                .map(|(v, f)| {
                    let r = tab[3 * base * n..3 * (base + v.len()) * n].to_vec();
                    base += v.len();
                    (v.to_vec(), f, r)
                })
                .collect()
        };
    let (antes, depois) = (linhas(&g.path, &g.pesos), linhas(&p, &t));
    assert_eq!(t.len() / 3 / n, p.verts_all().count(), "três linhas por nó");
    let iguais = antes.iter().filter(|a| depois.contains(a)).count();
    let fechado = antes.iter().find(|a| a.1).expect("o fechado");
    assert!(
        depois.contains(fechado),
        "o contorno fechado mudou (nós ou linhas)"
    );
    assert!(
        iguais < antes.len(),
        "o CONTROLO: nenhuma risca foi cortada"
    );
    println!("  contornos iguais ao bit {iguais} de {}", antes.len());
    // As riscas inteiras à vista: as da fonte cujo desenho não perdeu nenhum ponto.
    assert!(
        iguais > antes.len() / 2,
        "o recorte tocou riscas que não cortou ({iguais})"
    );
}
/// A largura do traço da barra da cena (`0,75 · 0,06`).
const LARGURA: f64 = 0.75 * 0.06;

/// A barra da cena `=5` com *Hatch* (`4,5 × 0,75`, três ossos), presa recta e dobrada em S a `graus`:
/// a fonte guardada, a tabela, o campo, a pele e a profundidade.
fn barra_em_s(
    graus: f32,
) -> (
    VecPath,
    Vec<f64>,
    ph2d_vec_skin::pesos::CampoDoDominio,
    ph2d_skeleton::Skin,
    Vec<f64>,
) {
    use crate::barra_da_cena_tests_support::osso;
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
    let mut barra = ph2d_vec_scene::cook(
        ph2d_vec_scene::ShapeKind::RoundRect,
        [-2.25, -0.375],
        [2.25, 0.375],
        &[0.375],
    );
    barra.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
        ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
        LARGURA,
    ));
    barra.effects = vec![FxEntry::new(PathEffect::Hatch(
        ph2d_vec_scene::fx_hatch::HatchSpec {
            angle: 45.0,
            spacing: 8.0,
            cross: false,
        },
    ))];
    let id = scene.push_path(barra);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let passo = (4.5 - 0.75) / 3.0;
    #[expect(clippy::cast_possible_truncation, reason = "metros de uma cena")]
    let p32 = passo as f32;
    let b1 = osso(&mut sim, "b1", [-1.875, 0.0], passo, None);
    let b2 = osso(&mut sim, "b2", [p32, 0.0], passo, Some(b1));
    let b3 = osso(&mut sim, "b3", [p32, 0.0], passo, Some(b2));
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], Some(b1)),
        1
    );
    for (b, g) in [(b2, graus), (b3, -graus)] {
        sim.world_mut()
            .get_mut::<ph2d_ecs::Transform>(b)
            .expect("Transform")
            .rotation = g.to_radians();
    }
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let index = crate::skin_live::bone_index(&sim);
    let pele = crate::skin_live::resolve(&sim, &skin, e, &index).expect("pele");
    let prof = crate::esqueletos::profundidades(&sim, &skin, &index);
    (g.path, g.pesos, g.campo.expect("campo"), pele, prof)
}

/// ⭐⭐ **GATE — nenhum pedaço cortado fica mais curto que a largura do traço** (FOTOGRAFADO na `=5`
/// a `110°`: tiques soltos junto às juntas).
///
/// ⛔ **O CONTROLO:** os intervalos à vista ANTES do filtro têm pedaços assim (`0,036`–`0,042` a
/// `100°`/`120°`, MEDIDO).
#[test]
fn nenhum_pedaco_cortado_e_mais_curto_que_o_traco() {
    let mut no_controlo = 0;
    for graus in [100f32, 110.0, 120.0] {
        let (fonte, pesos, campo, pele, prof) = barra_em_s(graus);
        let f = super::Posada::nova(&campo, None, &pele, &[], true, &prof)
            .expect("posada")
            .com_a_arte(&fonte);
        let curtos = |p: &VecPath, so_cortados: bool| -> usize {
            (0..p.contour_count())
                .filter_map(|c| p.contour(c))
                .filter(|(v, fechado)| !*fechado && v.len() > 1)
                .map(|(v, _)| {
                    #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
                    let fim = (v.len() - 1) as f64;
                    let vis = if so_cortados {
                        super::a_vista(v, &f).unwrap_or_default()
                    } else {
                        vec![(0.0, fim)]
                    };
                    vis.iter()
                        .filter(|&&(a, b)| !(a <= 0.0 && b >= fim) || !so_cortados)
                        .filter(|&&(a, b)| f.comprimento(v, a, b) < LARGURA)
                        .count()
                })
                .sum()
        };
        no_controlo += curtos(&fonte, true);
        let (cortada, _) =
            super::so_o_que_se_ve(&fonte, &pesos, (&campo, None), (&pele, &[], true), &prof)
                .expect("a dobra corta");
        let n = curtos(&cortada, false);
        assert_eq!(
            n, 0,
            "a {graus}° ficaram {n} pedaços mais curtos que o traço"
        );
    }
    assert!(
        no_controlo > 0,
        "o CONTROLO: antes do filtro não há pedaço curto"
    );
}

/// ⭐⭐ **GATE — cada corte cai na FRONTEIRA** entre o que se vê e o que está tapado (a bissecção):
/// a `1e-3` do segmento de cada lado o estado é o oposto. ⛔ **O CONTROLO:** há cortes.
#[test]
fn cada_corte_cai_na_fronteira_do_que_se_ve() {
    let (fonte, _, campo, pele, prof) = barra_em_s(110.0);
    let f = super::Posada::nova(&campo, None, &pele, &[], true, &prof)
        .expect("posada")
        .com_a_arte(&fonte);
    let mut cortes = 0;
    for c in 0..fonte.contour_count() {
        let Some((v, false)) = fonte.contour(c) else {
            continue;
        };
        #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
        let fim = (v.len() - 1) as f64;
        let em = |u: f64| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "segmento"
            )]
            let k = (u.floor() as usize).min(v.len() - 2);
            #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
            f.tapado(super::avalia(&super::cubica(v, k), u - k as f64))
        };
        for (a, b) in super::a_vista(v, &f).unwrap_or_default() {
            for (u, dentro_vis) in [(a, 1.0_f64), (b, -1.0)] {
                if u <= 0.0 || u >= fim {
                    continue;
                }
                cortes += 1;
                assert!(
                    !em(dentro_vis.mul_add(1e-3, u)) && em(dentro_vis.mul_add(-1e-3, u)),
                    "o corte em {u:.5} não está na fronteira"
                );
            }
        }
    }
    assert!(cortes > 0, "o CONTROLO: nenhum corte a 110°");
}

/// ⭐⭐ **GATE — o pedaço recortado é a curva da fonte, EXACTA** (de Casteljau), numa risca CURVA
/// (as do *Hatch* são rectas e aprovariam alças erradas).
#[test]
fn o_recorte_e_a_curva_da_fonte() {
    let mut a = ph2d_vec_scene::VecVertex::corner([0.0, 0.0]);
    a.out_handle = [1.0, 3.0];
    let mut b = ph2d_vec_scene::VecVertex::corner([4.0, 0.0]);
    b.in_handle = [3.0, -2.0];
    b.out_handle = [5.0, 2.0];
    let mut c = ph2d_vec_scene::VecVertex::corner([7.0, 1.0]);
    c.in_handle = [6.0, 3.0];
    let vs = [a, b, c];
    for (u0, u1) in [(0.3, 0.7), (0.2, 1.6), (0.0, 1.25), (1.1, 2.0)] {
        let p: Vec<_> = super::recorta(&vs, u0, u1)
            .into_iter()
            .map(|(v, _)| v)
            .collect();
        let mut pior = 0.0_f64;
        for k in 0..p.len() - 1 {
            for i in 0..=20 {
                let q = super::avalia(&super::cubica(&p, k), f64::from(i) / 20.0);
                let perto = (0..=4000)
                    .map(|j| {
                        let u = (u1 - u0).mul_add(f64::from(j) / 4000.0, u0);
                        #[expect(
                            clippy::cast_possible_truncation,
                            clippy::cast_sign_loss,
                            reason = "segmento"
                        )]
                        let s = (u.floor() as usize).min(1);
                        #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
                        let r = super::avalia(&super::cubica(&vs, s), u - s as f64);
                        (r[0] - q[0]).hypot(r[1] - q[1])
                    })
                    .fold(f64::MAX, f64::min);
                pior = pior.max(perto);
            }
        }
        let fim = |u: f64| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "segmento"
            )]
            let s = (u.floor() as usize).min(1);
            #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
            super::avalia(&super::cubica(&vs, s), u - s as f64)
        };
        assert!(pior < 5e-3, "{u0}..{u1}: o pedaço sai da curva ({pior})");
        assert_eq!(
            p[0].anchor.map(|x| (x * 1e9).round()),
            fim(u0).map(|x| (x * 1e9).round())
        );
        let ultimo = p[p.len() - 1].anchor;
        assert_eq!(
            ultimo.map(|x| (x * 1e9).round()),
            fim(u1).map(|x| (x * 1e9).round())
        );
    }
}

/// ⭐⭐ **GATE — uma risca sobre um triângulo do AVESSO não se vê** (o lado de baixo da dobra; os
/// tiques FOTOGRAFADOS a `110°` junto às juntas). Cada ponto à vista mora num triângulo de pose
/// direita. ⛔ **O CONTROLO:** a fonte tem pontos de risca sobre triângulos virados.
#[test]
fn uma_risca_sobre_o_avesso_da_dobra_nao_se_ve() {
    let (fonte, pesos, campo, pele, prof) = barra_em_s(120.0);
    let f = super::Posada::nova(&campo, None, &pele, &[], true, &prof)
        .expect("posada")
        .com_a_arte(&fonte);
    let no_avesso = |p: &VecPath| -> usize {
        amostras(p, false, 200)
            .into_iter()
            .filter(|(q, _)| f.onde(*q).is_some_and(|(t, _)| f.virado[t]))
            .count()
    };
    let controlo = no_avesso(&fonte);
    let (cortada, _) =
        super::so_o_que_se_ve(&fonte, &pesos, (&campo, None), (&pele, &[], true), &prof)
            .expect("a dobra corta");
    // ⚠️ A bissecção pára a `2⁻¹²` de um segmento da fronteira: as PONTAS de cada pedaço podem cair
    // a essa distância dentro do avesso — a régua salta o 1 % de cada ponta.
    let mut depois = 0;
    for c in 0..cortada.contour_count() {
        let Some((v, false)) = cortada.contour(c) else {
            continue;
        };
        let segs = v.len() - 1;
        for k in 0..segs {
            for i in 0..=200 {
                let t = f64::from(i) / 200.0;
                if (k == 0 && t < 0.01) || (k == segs - 1 && t > 0.99) {
                    continue;
                }
                let q = super::avalia(&super::cubica(v, k), t);
                depois += usize::from(f.onde(q).is_some_and(|(tri, _)| f.virado[tri]));
            }
        }
    }
    println!("  pontos de risca no avesso: fonte {controlo} · à vista depois {depois}");
    assert!(
        controlo > 0,
        "o CONTROLO: a 120° nenhuma risca passa pelo avesso"
    );
    assert_eq!(
        depois, 0,
        "{depois} pontos de risca à vista sobre o avesso da dobra"
    );
}

#[path = "skin_desenho_frente_fechados_tests.rs"]
mod fechados;
#[path = "skin_desenho_frente_rapida_tests.rs"]
mod rapida;
