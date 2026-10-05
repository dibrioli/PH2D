//! A12 — as passagens NOVAS que o traço enche cortam-se pela corda.
//!
//! ⚠️ A régua dos casos sintéticos é a GEOMETRIA conhecida da fixtura (o bico da fenda, a ponta do
//! dente, o vale da fonte): está ou não está sobre o contorno do desenho.

use super::{SEM_FECHO, fecha_as_fendas_que_o_traco_enche};
use ph2d_vec_scene::{Contour, VecPath, VecVertex};

/// A largura do traço das fixturas sintéticas.
const W: f64 = 2.0;

fn poligono(pts: &[[f64; 2]]) -> Contour {
    Contour {
        verts: pts.iter().map(|&p| VecVertex::corner(p)).collect(),
        closed: true,
    }
}

/// O caminho com estes contornos fechados e o traço `W`.
fn caminho(cs: Vec<Contour>) -> VecPath {
    let mut it = cs.into_iter();
    let c0 = it.next().expect("um contorno");
    VecPath {
        verts: c0.verts,
        closed: true,
        subpaths: it.collect(),
        stroke: Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            W,
        )),
        ..VecPath::default()
    }
}

/// A união da fonte, com ou sem a lei.
fn uniao(fonte: &VecPath, com_a_lei: bool) -> VecPath {
    let mut u = ph2d_vec_boolean::resolve_overlap(fonte).expect("sobrepõe-se");
    if com_a_lei {
        fecha_as_fendas_que_o_traco_enche(&mut u, fonte);
    }
    u
}

/// A distância de `p` ao contorno desenhado de `u`.
fn ao_contorno(u: &VecPath, p: [f64; 2]) -> f64 {
    (0..u.contour_count())
        .filter_map(|c| u.contour(c))
        .flat_map(|(v, _)| {
            super::polilinha(v)
                .into_iter()
                .collect::<Vec<_>>()
                .windows(2)
                .map(|s| super::ao_troco(p, s[0], s[1]).0)
                .collect::<Vec<_>>()
        })
        .fold(f64::MAX, f64::min)
}

/// A soma das áreas com sinal dos contornos de `u`.
fn area(u: &VecPath) -> f64 {
    (0..u.contour_count())
        .filter_map(|c| u.contour(c))
        .map(|(v, _)| {
            let p = super::polilinha(v);
            0.5 * (0..p.len())
                .map(|i| {
                    let (a, b) = (p[i], p[(i + 1) % p.len()]);
                    a[0] * b[1] - b[0] * a[1]
                })
                .sum::<f64>()
        })
        .sum()
}

/// O rectângulo `[0, 10] × [0, 4]` com um VALE (da fonte) na base, bico em `(5, 3)`, e por cima um
/// quadrilátero cuja aresta de baixo (declive `0,3`) cruza o topo dele em `(9,33…, 4)`: entre os dois
/// fica uma FENDA nova, `0,2` de largura no canto `(10, 4)`.
fn fenda_e_vale() -> VecPath {
    caminho(vec![
        poligono(&[
            [0.0, 0.0],
            [4.4, 0.0],
            [5.0, 3.0],
            [5.6, 0.0],
            [10.0, 0.0],
            [10.0, 4.0],
            [0.0, 4.0],
        ]),
        poligono(&[[6.0, 3.0], [20.0, 7.2], [20.0, 20.0], [6.0, 20.0]]),
    ])
}

/// ⭐⭐ **GATE — a fenda NOVA fecha-se e o vale da FONTE fica.** ⛔ **O CONTROLO:** sem a lei o bico
/// da fenda está no contorno, e a união é menor.
#[test]
fn a_fenda_nova_fecha_e_o_vale_da_fonte_fica() {
    let fonte = fenda_e_vale();
    let (sem, com) = (uniao(&fonte, false), uniao(&fonte, true));
    // O bico da fenda: onde a aresta do triângulo cruza o topo do rectângulo.
    let bico = [6.0 + 1.0 / 0.3, 4.0];
    assert!(
        ao_contorno(&sem, bico) < 1e-6,
        "controlo: o bico da fenda está no contorno"
    );
    assert!(ao_contorno(&com, bico) > 0.1, "a fenda nova ficou");
    assert!(
        area(&com).abs() > area(&sem).abs() + 1e-3,
        "a fenda não se encheu"
    );
    // O vale da fonte (bico em (5, 3), boca de 1,2 < W) fica.
    assert!(
        ao_contorno(&com, [5.0, 3.0]) < 1e-6,
        "o vale da fonte foi fechado"
    );
}

/// O anel `[0, 20]²` com o buraco `[5, 15]²` e um DENTE fino que sai da parede de baixo e entra no
/// buraco até `(10, 9)` — na boca do buraco (`y = 5`) ele tem `0,69 < W`.
fn dente_no_buraco() -> VecPath {
    caminho(vec![
        poligono(&[[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]]),
        poligono(&[[5.0, 5.0], [5.0, 15.0], [15.0, 15.0], [15.0, 5.0]]),
        poligono(&[[9.4, 2.0], [10.6, 2.0], [10.0, 9.0]]),
    ])
}

/// ⭐⭐ **GATE — a PONTA nova que entra num buraco corta-se pela parede** (a outra marca da `=5` a
/// `100°`). ⛔ **O CONTROLO:** sem a lei a ponta do dente está no contorno.
#[test]
fn a_ponta_nova_que_entra_num_buraco_corta_se() {
    let fonte = dente_no_buraco();
    let (sem, com) = (uniao(&fonte, false), uniao(&fonte, true));
    assert!(
        ao_contorno(&sem, [10.0, 9.0]) < 1e-6,
        "controlo: a ponta está no contorno"
    );
    assert!(ao_contorno(&com, [10.0, 9.0]) > 1.0, "a ponta nova ficou");
    // A parede do buraco continua lá.
    assert!(
        ao_contorno(&com, [7.0, 5.0]) < 1e-6,
        "a parede do buraco mexeu"
    );
}

/// ⭐⭐ **GATE — um contorno que o traço engole INTEIRO fica** (o buraco pequeno: fechá-lo foi
/// recusado pelo dono, F59-b), mesmo novo. ⛔ **O CONTROLO:** o buraco é novo (a fonte não o tem) e
/// mais estreito que o traço.
#[test]
fn o_buraco_que_o_traco_engole_inteiro_fica() {
    let fora = poligono(&[[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]]);
    let furo = poligono(&[[10.0, 10.0], [10.0, 10.6], [10.6, 10.6], [10.6, 10.0]]);
    let fonte = caminho(vec![fora.clone()]);
    let antes = caminho(vec![fora, furo]);
    assert!(super::raio_inscrito(&super::polilinha(&antes.subpaths[0].verts)) < 0.5 * W);
    let mut u = antes.clone();
    fecha_as_fendas_que_o_traco_enche(&mut u, &fonte);
    assert_eq!(u, antes, "o buraco pequeno mexeu");
}

/// A lei desligada não mexe (a porta do controlo dos gates da cena).
#[test]
fn sem_a_lei_nada_muda() {
    let fonte = fenda_e_vale();
    let mut u = ph2d_vec_boolean::resolve_overlap(&fonte).expect("sobrepõe-se");
    let antes = u.clone();
    SEM_FECHO.with(|c| c.set(true));
    fecha_as_fendas_que_o_traco_enche(&mut u, &fonte);
    SEM_FECHO.with(|c| c.set(false));
    assert_eq!(u, antes);
}

/// A largura do traço da barra da cena `=5` (`0,75 · 0,06`).
const LARGURA_DA_CENA: f64 = 0.75 * 0.06;

/// A barra *Zig Zag* da cena `=5` (`4,5 × 0,75`, três ossos), presa recta e dobrada em S a `graus`,
/// desenhada numa thread NOVA (o memo do quadro é por thread), com ou sem a lei: o desenho e a
/// entrada da lei (`(união, fonte)`).
pub(super) fn zig_zag_em_s(graus: f32, sem_fecho: bool) -> (VecPath, Option<(VecPath, VecPath)>) {
    use crate::barra_da_cena_tests_support::osso;
    use crate::skin_desenho::Leis;
    use ph2d_vec_scene::effect::{FxEntry, PathEffect};
    std::thread::scope(|s| {
        s.spawn(|| {
            SEM_FECHO.with(|c| c.set(sem_fecho));
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
                LARGURA_DA_CENA,
            ));
            barra.effects = vec![FxEntry::new(PathEffect::ZigZag(
                ph2d_vec_scene::fx_zigzag::ZigZagSpec {
                    amplitude: 6.0,
                    ridges: 24.0,
                    ..Default::default()
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
            let forma =
                crate::skin_live::recook_leis(&sim, &mut scene.clone(), Leis::do_ambiente())
                    .remove(&id)
                    .expect("desenho")
                    .forma;
            (forma, super::ULTIMA.with(|c| c.borrow_mut().take()))
        })
        .join()
        .expect("thread do desenho")
    })
}

/// ⭐ **SONDA — as passagens da `=5`**, contorno a contorno: o raio inscrito, os pares estreitos e,
/// para o de arco maior de cada passagem, se é nova e o raio da região.
#[test]
#[ignore = "sonda: imprime"]
fn diag_as_passagens_do_zig_zag() {
    let w = LARGURA_DA_CENA;
    for graus in [100f32, 105.0, 110.0, 120.0] {
        let (_, entrada) = zig_zag_em_s(graus, true);
        let (u, fonte) = entrada.expect("a lei correu");
        let aneis: Vec<super::Anel> = (0..fonte.contour_count())
            .filter_map(|c| fonte.contour(c))
            .filter(|(v, f)| *f && v.len() > 1)
            .map(|(v, _)| super::Anel::novo(super::polilinha(v)))
            .collect();
        let ancoras: Vec<[f64; 2]> = (0..fonte.contour_count())
            .filter_map(|c| fonte.contour(c))
            .filter(|(v, f)| *f && v.len() > 1)
            .flat_map(|(v, _)| v.iter().map(|x| x.anchor).collect::<Vec<_>>())
            .collect();
        println!("  {graus}°: {} contornos na união", u.contour_count());
        let mut svg = String::from(
            "<svg xmlns='http://www.w3.org/2000/svg' viewBox='-0.6 -0.9 1.2 1.1' width='1400' \
             height='1283'><rect x='-9' y='-9' width='99' height='99' fill='white'/>",
        );
        for c in 0..u.contour_count() {
            let Some((v, _)) = u.contour(c) else { continue };
            let pl = super::polilinha(v);
            let r = super::raio_inscrito(&pl) / w;
            super::EXAMINADAS.with(|x| x.borrow_mut().clear());
            let cruz = super::cruzamentos(v, &aneis, &ancoras, 1e-3 * w);
            super::REGISTA.with(|x| x.set(true));
            let achou = super::fendas(&pl, &cruz, w, false, 1.0);
            super::REGISTA.with(|x| x.set(false));
            for (s, e, arco, corda, velha, r) in super::EXAMINADAS.with(|x| x.take()) {
                println!(
                    "      {s}→{e} arco {:.2} corda {:.2} velha {velha} raio {:.2} (larg.)",
                    arco / w,
                    corda / w,
                    r / w
                );
            }
            println!(
                "    contorno {c}: {} amostras · raio {r:.2} larg. · passagem {achou:?}",
                pl.len()
            );
            let cor = [
                "#d00", "#0a0", "#00d", "#d0d", "#0aa", "#a60", "#555", "#f80",
            ][c % 8];
            let d: String = pl
                .iter()
                .enumerate()
                .map(|(i, q)| {
                    format!(
                        "{}{:.5} {:.5} ",
                        if i == 0 { 'M' } else { 'L' },
                        q[0],
                        -q[1]
                    )
                })
                .collect();
            svg.push_str(&format!(
                "<path d='{d}Z' fill='none' stroke='{cor}' stroke-width='{}'/>",
                w / 6.0
            ));
            for &(a, b) in &achou {
                svg.push_str(&format!(
                    "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='black' stroke-width='{}'/>",
                    pl[a][0],
                    -pl[a][1],
                    pl[b][0],
                    -pl[b][1],
                    w / 4.0
                ));
            }
        }
        svg.push_str("</svg>");
        std::fs::create_dir_all("target/prova/fendas").expect("pasta");
        std::fs::write(format!("target/prova/fendas/passagens_{graus}.svg"), svg).expect("svg");
    }
}

/// Os contornos que o traço engole inteiros (`raio < W/2`): `(área, caixa)` arredondadas.
fn engolidos(u: &VecPath, w: f64) -> Vec<String> {
    let mut v: Vec<String> = (0..u.contour_count())
        .filter_map(|c| u.contour(c))
        .map(|(v, _)| super::polilinha(v))
        .filter(|p| super::raio_inscrito(p) < 0.5 * w)
        .map(|p| {
            let a: f64 = 0.5
                * (0..p.len())
                    .map(|i| {
                        let (x, y) = (p[i], p[(i + 1) % p.len()]);
                        x[0] * y[1] - y[0] * x[1]
                    })
                    .sum::<f64>();
            format!("{a:.7} {:.5} {:.5}", p[0][0], p[0][1])
        })
        .collect();
    v.sort();
    v
}

/// ⭐⭐⭐ **GATE — na `=5` as passagens NOVAS saem e os buracos que o traço engole FICAM** (`100°`,
/// `110°`): depois da lei a união não tem passagem nova nenhuma, e os contornos engolidos inteiros
/// são os mesmos ao `10⁻⁷`. ⛔ **O CONTROLO:** sem a lei há passagens (a lei mexe no desenho).
#[test]
fn na_cena_as_passagens_novas_saem_e_os_buracos_engolidos_ficam() {
    let w = LARGURA_DA_CENA;
    for graus in [100f32, 110.0] {
        let (sem, entrada) = zig_zag_em_s(graus, true);
        let (com, _) = zig_zag_em_s(graus, false);
        assert_ne!(sem, com, "controlo: a {graus}° a lei não mexeu");
        assert_eq!(
            engolidos(&sem, w),
            engolidos(&com, w),
            "a {graus}° um buraco engolido mexeu"
        );
        let (_, fonte) = entrada.expect("a lei correu");
        let aneis: Vec<super::Anel> = (0..fonte.contour_count())
            .filter_map(|c| fonte.contour(c))
            .filter(|(v, f)| *f && v.len() > 1)
            .map(|(v, _)| super::Anel::novo(super::polilinha(v)))
            .collect();
        let ancoras: Vec<[f64; 2]> = (0..fonte.contour_count())
            .filter_map(|c| fonte.contour(c))
            .filter(|(v, f)| *f && v.len() > 1)
            .flat_map(|(v, _)| v.iter().map(|x| x.anchor).collect::<Vec<_>>())
            .collect();
        let maior = (0..com.contour_count())
            .filter_map(|c| com.contour(c))
            .max_by_key(|(v, _)| v.len())
            .expect("contorno")
            .0;
        let cruz = super::cruzamentos(maior, &aneis, &ancoras, 1e-3 * w);
        assert_eq!(
            super::fendas(&super::polilinha(maior), &cruz, w, false, 1.0),
            vec![],
            "a {graus}° ficou uma passagem nova no contorno de fora"
        );
    }
}

/// ⭐ **SONDA — o preço da lei das passagens** na `=5`, µs por chamada sobre a entrada real (a cópia
/// da união fica fora do relógio): 7 rodadas de 20 chamadas, o mínimo (a mediana como controlo).
#[test]
#[ignore = "sonda: imprime"]
fn diag_o_preco_das_passagens() {
    for graus in [0f32, 100.0, 110.0, 120.0] {
        let (_, entrada) = zig_zag_em_s(graus, true);
        let Some((u, fonte)) = entrada else {
            println!("  {graus}°: sem união (a lei não corre)");
            continue;
        };
        super::EXAMINADAS.with(|x| x.borrow_mut().clear());
        super::REGISTA.with(|x| x.set(true));
        let mut c = u.clone();
        fecha_as_fendas_que_o_traco_enche(&mut c, &fonte);
        super::REGISTA.with(|x| x.set(false));
        let ex = super::EXAMINADAS.with(|x| x.take());
        println!(
            "  {graus}°: {} candidatos examinados ({} da fonte) · {} amostras",
            ex.len(),
            ex.iter().filter(|x| x.4).count(),
            (0..u.contour_count())
                .filter_map(|k| u.contour(k))
                .map(|(v, _)| v.len() * 16)
                .sum::<usize>()
        );
        {
            let w = LARGURA_DA_CENA;
            let fechados: Vec<&[VecVertex]> = (0..fonte.contour_count())
                .filter_map(|c| fonte.contour(c))
                .filter(|(v, f)| *f && v.len() > 1)
                .map(|(v, _)| v)
                .collect();
            let ancoras: Vec<[f64; 2]> = fechados
                .iter()
                .flat_map(|v| v.iter().map(|x| x.anchor))
                .collect();
            let t0 = std::time::Instant::now();
            let aneis: Vec<super::Anel> = fechados
                .iter()
                .map(|v| super::Anel::novo(super::polilinha(v)))
                .collect();
            let t1 = t0.elapsed().as_secs_f64() * 1e6;
            let mut n_cruz = 0;
            let mut iguais = 0;
            for k in 0..u.contour_count() {
                let Some((v, _)) = u.contour(k) else { continue };
                iguais += v
                    .iter()
                    .filter(|x| {
                        ancoras
                            .iter()
                            .any(|a| (a[0] - x.anchor[0]).hypot(a[1] - x.anchor[1]) <= 1e-3 * w)
                    })
                    .count();
                n_cruz += super::cruzamentos(v, &aneis, &ancoras, 1e-3 * w)
                    .iter()
                    .filter(|x| **x)
                    .count();
            }
            println!(
                "  {graus}°: anéis {t1:.0} µs · cruzamentos {:.0} µs · {n_cruz} cruzamentos · âncoras iguais às da fonte {iguais} de {}",
                t0.elapsed().as_secs_f64() * 1e6 - t1,
                (0..u.contour_count())
                    .filter_map(|k| u.contour(k))
                    .map(|(v, _)| v.len())
                    .sum::<usize>()
            );
        }
        let mut t: Vec<f64> = Vec::new();
        for _ in 0..7 {
            let copias: Vec<VecPath> = (0..20).map(|_| u.clone()).collect();
            let t0 = std::time::Instant::now();
            for mut c in copias {
                fecha_as_fendas_que_o_traco_enche(&mut c, &fonte);
                std::hint::black_box(&c);
            }
            t.push(t0.elapsed().as_secs_f64() * 1e6 / 20.0);
        }
        t.sort_by(f64::total_cmp);
        // E o quadro inteiro da forma, com e sem a lei, intercalados (o memo nunca acerta: uma
        // thread nova por desenho).
        let mut q: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
        for r in 0..7 {
            for v in [r % 2, 1 - r % 2] {
                let t0 = std::time::Instant::now();
                let _ = zig_zag_em_s(graus, v == 1);
                q[v].push(t0.elapsed().as_secs_f64() * 1e6);
            }
        }
        for x in &mut q {
            x.sort_by(f64::total_cmp);
        }
        println!(
            "  {graus}°: bind + quadro com a lei {:.0} µs · sem {:.0} µs (mínimos)",
            q[0][0], q[1][0]
        );
        println!(
            "  {graus}°: {:.0} µs (med {:.0}) · loadavg {}",
            t[0],
            t[3],
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
    }
}
