//! SONDA do report de 2026-09-02 — reproduz a cena do Enio a partir dos SVG que ele exportou.
#![cfg(test)]

use ph2d_vec_scene::{VecVertex, VertexKind};

/// `d` de SVG → contornos `(vértices, fecha?)`, o inverso exacto do que o exportador escreve.
pub(crate) fn contornos_de(d: &str) -> Vec<(Vec<VecVertex>, bool)> {
    use ph2d_vector::PathEl;
    let bp = ph2d_vector::BezPath::from_svg(d).expect("d valido");
    let mut out: Vec<(Vec<VecVertex>, bool)> = Vec::new();
    let mut cur: Vec<VecVertex> = Vec::new();
    let mut fecha = false;
    let empurra = |out: &mut Vec<_>, cur: &mut Vec<VecVertex>, fecha: &mut bool| {
        if cur.len() >= 2 {
            if *fecha
                && cur.len() > 2
                && (cur[0].anchor[0] - cur[cur.len() - 1].anchor[0]).abs() < 1e-9
                && (cur[0].anchor[1] - cur[cur.len() - 1].anchor[1]).abs() < 1e-9
            {
                let ultimo = cur.pop().expect("nao vazio");
                cur[0].in_handle = ultimo.in_handle;
            }
            out.push((std::mem::take(cur), *fecha));
        }
        cur.clear();
        *fecha = false;
    };
    for el in bp.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                empurra(&mut out, &mut cur, &mut fecha);
                cur.push(VecVertex {
                    anchor: [p.x, p.y],
                    in_handle: [p.x, p.y],
                    out_handle: [p.x, p.y],
                    kind: VertexKind::Corner,
                    corner_radius: 0.0,
                });
            }
            PathEl::CurveTo(c1, c2, p) => {
                if let Some(l) = cur.last_mut() {
                    l.out_handle = [c1.x, c1.y];
                }
                cur.push(VecVertex {
                    anchor: [p.x, p.y],
                    in_handle: [c2.x, c2.y],
                    out_handle: [p.x, p.y],
                    kind: VertexKind::Corner,
                    corner_radius: 0.0,
                });
            }
            PathEl::ClosePath => fecha = true,
            PathEl::LineTo(p) => {
                if let Some(l) = cur.last() {
                    let a = l.anchor;
                    cur.last_mut().expect("ha' ultimo").out_handle = a;
                }
                cur.push(VecVertex {
                    anchor: [p.x, p.y],
                    in_handle: [p.x, p.y],
                    out_handle: [p.x, p.y],
                    kind: VertexKind::Corner,
                    corner_radius: 0.0,
                });
            }
            PathEl::QuadTo(..) => unreachable!("o exportador so' escreve cubicas"),
        }
    }
    empurra(&mut out, &mut cur, &mut fecha);
    out
}

/// Um `<path>` lido do SVG: `(id, é preenchimento?, cor, o `d`)`.
pub(crate) type PathLido = (u64, bool, String, String);

/// `(id, é preenchimento?, cor, d)` de cada `<path>` de um SVG exportado.
pub(crate) fn paths_do_svg(txt: &str) -> Vec<PathLido> {
    let mut out = Vec::new();
    for l in txt.lines().filter(|l| l.contains("<path ")) {
        let pega = |chave: &str| -> Option<String> {
            let i = l.find(chave)? + chave.len();
            let j = l[i..].find('"')? + i;
            Some(l[i..j].to_string())
        };
        let Some(id) = pega("data-ph2d-id=\"").and_then(|s| s.parse::<u64>().ok()) else {
            continue;
        };
        let Some(d) = pega(" d=\"") else { continue };
        out.push((
            id,
            l.contains("data-ph2d-fill=\"1\""),
            pega(" fill=\"").unwrap_or_default(),
            d,
        ));
    }
    out
}

#[cfg(test)]
mod probe {
    use super::*;

    /// Um contorno como a rede o recebe.
    type Contorno = (Vec<VecVertex>, bool);
    use crate::vec_bucket_claim::{Receita, ancoras_da_face, donos};
    use ph2d_ecs::FillAnchor;

    fn ler(nome: &str) -> Vec<super::PathLido> {
        let p = format!("{}/tests/fixtures/{nome}.svg", env!("CARGO_MANIFEST_DIR"));
        paths_do_svg(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")))
    }

    /// As PAREDES e as etiquetas: o SVG traz um `<path>` por objecto, com `data-ph2d-id`, e cada um
    /// pode ter vários contornos — que é exactamente o que o app etiqueta.
    fn paredes(v: &[super::PathLido]) -> (Vec<Contorno>, Vec<(u64, u16)>) {
        let mut cs = Vec::new();
        let mut tags = Vec::new();
        for (id, e, _, d) in v.iter().filter(|(_, e, _, _)| !*e) {
            let _ = e;
            for (k, c) in contornos_de(d).into_iter().enumerate() {
                cs.push(c);
                tags.push((*id, u16::try_from(k).unwrap_or(u16::MAX)));
            }
        }
        (cs, tags)
    }

    /// ⭐ **As receitas do estado BASE**: por preenchimento, as âncoras da face em que ele está.
    ///
    /// ⚠️ É o que o clique grava — e depois **nunca mais se reescreve**. A sonda tem de fazer o
    /// mesmo, senão mede um modelo que o produto não tem.
    fn receitas_da_base(nome: &str) -> (Vec<u64>, Vec<Vec<FillAnchor>>, Vec<[f64; 2]>) {
        let b = ler(nome);
        let (cs, tags) = paredes(&b);
        let r = ph2d_vec_fill::rede(&cs);
        let (mut ids, mut anc, mut sem) = (Vec::new(), Vec::new(), Vec::new());
        for (id, _, _, d) in b.iter().filter(|(_, e, _, _)| *e) {
            let cts = contornos_de(d);
            let Some((verts, _)) = cts.first() else {
                continue;
            };
            let poly = ph2d_vec_scene::detection_polyline(verts, true);
            let p = miolo(&poly);
            let Some(f) = r.face_em(p) else { continue };
            ids.push(*id);
            anc.push(ancoras_da_face(&r, &tags, &f));
            sem.push(p);
        }
        (ids, anc, sem)
    }

    /// Um ponto BEM DENTRO de um polígono. ⚠️⚠️ **A 1.ª redacção desta sonda usava o primeiro ponto
    /// do polígono** — que está EM CIMA da fronteira, não dentro — e o caso de identidade deixava de
    /// se reproduzir. *Nada disso vinha do produto; vinha da régua.*
    fn miolo(poly: &[[f64; 2]]) -> [f64; 2] {
        use ph2d_vec_scene::point_in_polygon;
        let n = poly.len() as f64;
        let c = [
            poly.iter().map(|p| p[0]).sum::<f64>() / n,
            poly.iter().map(|p| p[1]).sum::<f64>() / n,
        ];
        if point_in_polygon(poly, c) {
            return c;
        }
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in poly {
            lo = [lo[0].min(p[0]), lo[1].min(p[1])];
            hi = [hi[0].max(p[0]), hi[1].max(p[1])];
        }
        let mut melhor = (c, f64::NEG_INFINITY);
        for i in 0..31 {
            for j in 0..31 {
                let t = |k: usize| (k as f64 + 0.5) / 31.0;
                let p = [
                    lo[0] + (hi[0] - lo[0]) * t(i),
                    lo[1] + (hi[1] - lo[1]) * t(j),
                ];
                if !point_in_polygon(poly, p) {
                    continue;
                }
                let d = poly
                    .windows(2)
                    .map(|w| {
                        let (a, b) = (w[0], w[1]);
                        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                        let ll = dx * dx + dy * dy;
                        let t = if ll <= 1e-18 {
                            0.0
                        } else {
                            (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / ll).clamp(0.0, 1.0)
                        };
                        (p[0] - (a[0] + t * dx)).hypot(p[1] - (a[1] + t * dy))
                    })
                    .fold(f64::INFINITY, f64::min);
                if d > melhor.1 {
                    melhor = (p, d);
                }
            }
        }
        melhor.0
    }

    /// Resolve as receitas da base sobre o desenho `alvo`.
    fn resolve(alvo: &str) -> (Vec<u64>, Vec<Option<usize>>, Vec<ph2d_vec_fill::Face>) {
        let (ids, anc, sem) = receitas_da_base("drawing_base");
        let a = ler(alvo);
        let (cs, tags) = paredes(&a);
        let r = ph2d_vec_fill::rede(&cs);
        let faces: Vec<_> = r.faces().into_iter().filter(|f| f.area > 0.0).collect();
        let receitas: Vec<Receita> = anc
            .iter()
            .zip(&sem)
            .map(|(ancoras, s)| Receita {
                ancoras,
                semente: *s,
            })
            .collect();
        let d = donos(&r, &faces, &tags, &receitas);
        (ids, d, faces)
    }

    /// SONDA — arrasta um nó soldado por passos e vê se algum dono SALTA sem a topologia mudar.
    ///
    /// ⚠️ Move **todos** os vértices que coincidem com o escolhido, que é o que a seta branca faz
    /// numa rede soldada: um nó é um só para todas as linhas.
    #[test]
    #[ignore = "sonda; roda com --ignored --nocapture"]
    fn probe_deriva_ao_arrastar() {
        let (ids, anc, sem) = receitas_da_base("drawing_base");
        let b = ler("drawing_base");
        let (cs0, tags) = paredes(&b);
        // O nó escolhido: o primeiro vértice do primeiro contorno.
        let alvo = cs0[0].0[0].anchor;
        for passo in 0..=10 {
            let d = f64::from(passo) * 0.06;
            let mover = |q: [f64; 2]| {
                if (q[0] - alvo[0]).hypot(q[1] - alvo[1]) < 1e-6 {
                    [q[0] + d, q[1] - d]
                } else {
                    q
                }
            };
            let cs: Vec<Contorno> = cs0
                .iter()
                .map(|(vs, c)| {
                    (
                        vs.iter()
                            .map(|v| {
                                let a = mover(v.anchor);
                                let (dx, dy) = (a[0] - v.anchor[0], a[1] - v.anchor[1]);
                                VecVertex {
                                    anchor: a,
                                    in_handle: [v.in_handle[0] + dx, v.in_handle[1] + dy],
                                    out_handle: [v.out_handle[0] + dx, v.out_handle[1] + dy],
                                    ..*v
                                }
                            })
                            .collect(),
                        *c,
                    )
                })
                .collect();
            let r = ph2d_vec_fill::rede(&cs);
            let faces: Vec<_> = r.faces().into_iter().filter(|f| f.area > 0.0).collect();
            let receitas: Vec<Receita> = anc
                .iter()
                .zip(&sem)
                .map(|(ancoras, s)| Receita {
                    ancoras,
                    semente: *s,
                })
                .collect();
            let donos = donos(&r, &faces, &tags, &receitas);
            let area: Vec<f64> = (0..ids.len())
                .map(|k| {
                    faces
                        .iter()
                        .zip(&donos)
                        .filter(|(_, x)| **x == Some(k))
                        .map(|(f, _)| f.area)
                        .sum()
                })
                .collect();
            let sem_cor: f64 = faces
                .iter()
                .zip(&donos)
                .filter(|(_, x)| x.is_none())
                .map(|(f, _)| f.area)
                .sum();
            for (f, x) in faces.iter().zip(&donos) {
                if x.is_none() {
                    let poly = r.contorno(f);
                    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
                    for q in &poly {
                        lo = [lo[0].min(q[0]), lo[1].min(q[1])];
                        hi = [hi[0].max(q[0]), hi[1].max(q[1])];
                    }
                    println!(
                        "    SEM COR: area={:.5} caixa={:.4}x{:.4} centro=({:.3},{:.3}) arcos={} \
                         no' arrastado=({:.3},{:.3})",
                        f.area,
                        hi[0] - lo[0],
                        hi[1] - lo[1],
                        f64::midpoint(lo[0], hi[0]),
                        f64::midpoint(lo[1], hi[1]),
                        f.arcos.len(),
                        alvo[0] + d,
                        alvo[1] - d
                    );
                }
            }
            println!(
                "d={d:.2} faces={:2} sem_cor={sem_cor:6.3} areas={}",
                faces.len(),
                area.iter()
                    .map(|a| format!("{a:.2}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }

    /// ⭐⭐⭐ **CADA PREENCHIMENTO FICA COM A SUA REGIÃO — nos três estados que o Enio exportou.**
    ///
    /// A fixtura é o desenho real dele: sete preenchimentos sobre uma rede soldada de doze arcos, e
    /// dois estados a que ele chegou arrastando **o mesmo nó**. As receitas são gravadas **uma vez**
    /// sobre o estado base, como um clique faz, e nunca reescritas.
    ///
    /// ⚠️⚠️ **É esta a propriedade que os quatro reports pediam**: *"apagou a área"*, *"resíduo de
    /// preenchimento"*, *"pintou outra área com a cor errada"*. Com a receita a ser a região do
    /// quadro anterior, a resposta dependia do CAMINHO por que se lá chegou; com âncoras, ela é
    /// função do desenho e de mais nada.
    #[test]
    fn every_fill_keeps_its_own_region_in_all_three_states() {
        for alvo in ["drawing_base", "drawing01", "drawing02"] {
            let (ids, d, faces) = resolve(alvo);
            assert_eq!(ids.len(), 7, "{alvo}: a cena tem sete preenchimentos");
            let mut donos_vistos: Vec<usize> = d.iter().flatten().copied().collect();
            donos_vistos.sort_unstable();
            donos_vistos.dedup();
            assert_eq!(
                donos_vistos.len(),
                ids.len(),
                "{alvo}: um preenchimento ficou sem regiao nenhuma (donos {d:?})"
            );
            assert!(
                d.iter().filter(|x| x.is_some()).count() >= faces.len().min(7),
                "{alvo}: faces demais ficaram sem cor: {d:?}"
            );
        }
    }
}
