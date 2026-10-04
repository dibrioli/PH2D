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
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::VecPath;

/// A junta da fixtura (`palco`: barra `40 × 10`, dois ossos de `20`).
const JUNTA: f64 = 20.0;
/// A meia-largura da zona da junta que a régua não julga.
const M: f64 = 4.0;
/// Distância a que uma amostra se lê pintada.
const PERTO: f64 = 0.1;
/// Folga da fronteira da frente: perto dela a régua não julga.
const FOLGA: f64 = 0.25;

fn cubica(vs: &[ph2d_vec_scene::VecVertex], k: usize) -> [[f64; 2]; 4] {
    [vs[k].anchor, vs[k].out_handle, vs[k + 1].in_handle, vs[k + 1].anchor]
}

fn em(c: &[[f64; 2]; 4], t: f64) -> ([f64; 2], [f64; 2]) {
    let s = 1.0 - t;
    let p = |k: usize| {
        s * s * s * c[0][k] + 3.0 * s * s * t * c[1][k] + 3.0 * s * t * t * c[2][k] + t * t * t * c[3][k]
    };
    let d = |k: usize| {
        3.0 * s * s * (c[1][k] - c[0][k]) + 6.0 * s * t * (c[2][k] - c[1][k]) + 3.0 * t * t * (c[3][k] - c[2][k])
    };
    ([p(0), p(1)], [d(0), d(1)])
}

/// Amostras `(ponto, tangente)` dos contornos ABERTOS (ou dos fechados, `fechados = true`).
fn amostras(p: &VecPath, fechados: bool, por_seg: usize) -> Vec<([f64; 2], [f64; 2])> {
    let mut out = Vec::new();
    for c in 0..p.contour_count() {
        let Some((vs, fechado)) = p.contour(c) else { continue };
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
        let Some((vs, false)) = p.contour(c) else { continue };
        for k in 0..vs.len().saturating_sub(1) {
            let cb = cubica(vs, k);
            let pts: Vec<[f64; 2]> = (0..=64).map(|i| em(&cb, f64::from(i) / 64.0).0).collect();
            out.extend(pts.windows(2).map(|w| [w[0], w[1]]));
        }
    }
    out
}

fn dentro(pol: &[[f64; 2]], q: [f64; 2]) -> bool {
    let mut d = false;
    for i in 0..pol.len() {
        let (a, b) = (pol[i], pol[(i + 1) % pol.len()]);
        if (a[1] > q[1]) != (b[1] > q[1]) && q[0] < (b[0] - a[0]) * (q[1] - a[1]) / (b[1] - a[1]) + a[0] {
            d = !d;
        }
    }
    d
}

fn dist_pol(pol: &[[f64; 2]], q: [f64; 2]) -> f64 {
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
    assert_eq!(crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None), 1);
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
        assert!(tap0 > 0.9, "o CONTROLO: sem a lei as tapadas não se pintam ({tap0:.3})");
        assert!(tap < 0.02, "a {graus}° as riscas de trás pintam a frente ({tap:.3})");
        // ⚠️ A régua lê `~2 %` das amostras sem tinta TAMBÉM sem a lei (as pontas das riscas) ⇒
        // «nada mais se apaga» compara com o controlo.
        assert!(fr > 0.95 && fr >= fr0 - 1e-3, "a {graus}° a frente perdeu riscas ({fr:.3} × {fr0:.3})");
        assert!(
            vis > 0.95 && vis >= vis0 - 1e-3,
            "a {graus}° a trás à vista perdeu riscas ({vis:.3} × {vis0:.3})"
        );
    }
}

/// ⭐ **SONDA — O PREÇO por quadro** do recorte (só a porta) e do quadro inteiro com e sem a lei da
/// frente, a pose a MUDAR a cada chamada (`110°` ↔ `150°`). Imprime; corra em `--release` com a
/// máquina calma.
#[test]
fn diag_o_preco_do_recorte_por_quadro() {
    use std::time::Instant;
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    scene.path_mut(id).expect("path").effects = vec![FxEntry::new(PathEffect::Hatch(
        ph2d_vec_scene::fx_hatch::HatchSpec {
            angle: 45.0,
            spacing: 4.0,
            cross: false,
        },
    ))];
    assert_eq!(crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None), 1);
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
            .rotation = if n.is_multiple_of(2) { 110f32 } else { 150f32 }.to_radians();
    };
    let t = Instant::now();
    let mut k = 0_u32;
    while t.elapsed().as_millis() < 300 {
        dobra(&mut sim);
        let index = crate::skin_live::bone_index(&sim);
        let pele = crate::skin_live::resolve(&sim, &skin, e, &index).expect("pele");
        let prof = crate::esqueletos::profundidades(&sim, &skin, &index);
        let _ = super::so_o_que_se_ve(&g.path, &g.pesos, (&campo, indice.as_ref()), (&pele, &[], true), &prof);
        k += 1;
    }
    let porta = t.elapsed().as_secs_f64() * 1e6 / f64::from(k);
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
        "  riscas {} · malha {} triângulos · µs: o recorte {porta:.1} · o quadro com a lei {com:.1} \
         · sem ela {sem:.1} · loadavg {}",
        g.path.contour_count() - 1,
        campo.malha.tris.len(),
        std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim()
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
    assert_eq!(crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None), 1);
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
    let (p, t) = super::so_o_que_se_ve(&g.path, &g.pesos, (&campo, None), (&pele, &[], true), &prof)
        .expect("o CONTROLO: nada foi cortado a 150°");
    let n = g.pesos.len() / (3 * g.path.verts_all().count());
    let linhas = |q: &VecPath, tab: &[f64]| -> Vec<(Vec<ph2d_vec_scene::VecVertex>, bool, Vec<f64>)> {
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
    assert!(depois.contains(fechado), "o contorno fechado mudou (nós ou linhas)");
    assert!(iguais < antes.len(), "o CONTROLO: nenhuma risca foi cortada");
    println!("  contornos iguais ao bit {iguais} de {}", antes.len());
    // As riscas inteiras à vista: as da fonte cujo desenho não perdeu nenhum ponto.
    assert!(iguais > antes.len() / 2, "o recorte tocou riscas que não cortou ({iguais})");
}
#[test]
fn sonda_ilhas() {
    use crate::barra_da_cena_tests_support::osso;
    for graus in [100f32, 110.0, 120.0] {
        let mut sim = ph2d_ecs::SimWorld::default();
        let mut scene = ph2d_vec_scene::VecScene::new();
        let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
        let id = scene.push_path(ph2d_vec_scene::cook(ph2d_vec_scene::ShapeKind::RoundRect, [-2.25, -0.375], [2.25, 0.375], &[0.375]));
        scene.path_mut(id).expect("path").effects = vec![FxEntry::new(PathEffect::Hatch(
            ph2d_vec_scene::fx_hatch::HatchSpec { angle: 45.0, spacing: 8.0, cross: false },
        ))];
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
        let passo = (4.5 - 0.75) / 3.0;
        let b1 = osso(&mut sim, "b1", [-1.875, 0.0], passo, None);
        let b2 = osso(&mut sim, "b2", [passo as f32, 0.0], passo, Some(b1));
        let b3 = osso(&mut sim, "b3", [passo as f32, 0.0], passo, Some(b2));
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], Some(b1));
        sim.world_mut().get_mut::<ph2d_ecs::Transform>(b2).expect("T").rotation = graus.to_radians();
        sim.world_mut().get_mut::<ph2d_ecs::Transform>(b3).expect("T").rotation = -graus.to_radians();
        let e = ph2d_ecs::Entity::from_bits(map[&id]);
        let skin = sim.world().get::<SkinBind>(e).expect("p").clone();
        let g = crate::skinned_mesh::le(&skin.source).expect("f");
        let campo = g.campo.clone().expect("c");
        let index = crate::skin_live::bone_index(&sim);
        let pele = crate::skin_live::resolve(&sim, &skin, e, &index).expect("pele");
        let prof = crate::esqueletos::profundidades(&sim, &skin, &index);
        let f = super::Posada::nova(&campo, None, &pele, &[], true, &prof).expect("p");
        let mut curtas = Vec::new();
        let mut n = 0;
        for c in 0..g.path.contour_count() {
            let (v, fechado) = g.path.contour(c).expect("c");
            if fechado { continue; }
            if let Some(vis) = super::a_vista(v, &f) {
                let segs = (v.len() - 1) as f64;
                let cb = super::cubica(v, 0);
                let total = (cb[3][0]-cb[0][0]).hypot(cb[3][1]-cb[0][1]);
                for (a, b) in vis {
                    n += 1;
                    let pa = super::avalia(&cb, a / segs);
                    let posa = |p: [f64; 2]| { let mut w = pele.scratch(); pele.weights_corrected(p, campo.linha(p).as_deref(), &mut w, &[]); pele.blend(p, &w) };
                    let pts: Vec<[f64; 2]> = (0..=40).map(|i| posa(super::avalia(&cb, (a + (b - a) * f64::from(i) / 40.0) / segs))).collect();
                    let l: f64 = pts.windows(2).map(|w| (w[1][0]-w[0][0]).hypot(w[1][1]-w[0][1])).sum();
                    let tipo = match (a > 0.0, b < segs) { (true, true) => "ilha", (false, true) => "inicio", (true, false) => "fim", _ => "inteira" };
                    if l < 0.4 { curtas.push(format!("{tipo} ({:.2},{:.2}) l={l:.3} de {total:.2}", pa[0], pa[1])); }
                }
            }
        }
        println!("{graus}°: {n} pedaços cortados, curtos (<0.4): {curtas:?}");
    }
}
