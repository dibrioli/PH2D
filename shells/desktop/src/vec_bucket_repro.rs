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
    use crate::vec_bucket_claim::{Regiao, donos, herda_dos_vizinhos, terreno_novo};
    use ph2d_vec_scene::detection_polyline as dp;

    fn ler(nome: &str) -> Vec<super::PathLido> {
        let p = format!("{}/tests/fixtures/{nome}.svg", env!("CARGO_MANIFEST_DIR"));
        paths_do_svg(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")))
    }

    /// Um ponto BEM DENTRO de um polígono — a mesma lei do `Rede::interior_point`: o centroide
    /// quando ele cai lá dentro, senão a amostra da grelha mais afastada da borda.
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

    fn paredes(v: &[super::PathLido]) -> Vec<(Vec<VecVertex>, bool)> {
        v.iter()
            .filter(|(_, e, _, _)| !*e)
            .flat_map(|(_, _, _, d)| contornos_de(d))
            .collect()
    }

    fn regioes(v: &[super::PathLido]) -> Vec<RegiaoLida> {
        v.iter()
            .filter(|(_, e, _, _)| *e)
            .map(|(id, _, cor, d)| {
                let cs = contornos_de(d);
                let polis: Vec<Vec<[f64; 2]>> = cs.iter().map(|(vs, c)| dp(vs, *c)).collect();
                // ⚠️⚠️ **A semente é o MIOLO, e a 1.ª redacção desta sonda usou o primeiro ponto do
                // polígono** — que está EM CIMA da fronteira, não dentro. Com ele, o caso de
                // IDENTIDADE (alimentar o estado a si próprio) já não se reproduzia: uma face
                // mudava de dono e um preenchimento congelava, e nada disso vinha do produto.
                // O app re-semeia no ponto mais fundo da face; a sonda tem de fazer o mesmo.
                let semente = polis.first().map_or([0.0, 0.0], |p| miolo(p));
                (
                    *id,
                    cor.clone(),
                    Regiao {
                        poligonos: polis,
                        semente,
                    },
                )
            })
            .collect()
    }

    fn relata(nome: &str, base: &str) {
        let alvo = ler(nome);
        let b = ler(base);
        let rs = regioes(&b);
        let regs: Vec<Regiao> = rs
            .iter()
            .map(|(_, _, r)| Regiao {
                poligonos: r.poligonos.clone(),
                semente: r.semente,
            })
            .collect();
        let ant = ph2d_vec_fill::rede(&paredes(&b));
        let r = ph2d_vec_fill::rede(&paredes(&alvo));
        let faces: Vec<_> = r.faces().into_iter().filter(|f| f.area > 0.0).collect();
        let mut d = donos(&r, &faces, &regs);
        let so_voto = d.clone();
        let nova = terreno_novo(&r, &faces, &ant);
        let areas: Vec<f64> = faces.iter().map(|f| f.area).collect();
        herda_dos_vizinhos(&r.adjacencias(&faces), &areas, &nova, &mut d);
        println!(
            "\n=== {nome} (regioes vindas de {base}) === {} faces",
            faces.len()
        );
        for (i, f) in faces.iter().enumerate() {
            let nome_de = |x: Option<usize>| {
                x.map_or_else(|| "-".to_string(), |k| format!("{} {}", rs[k].0, rs[k].1))
            };
            println!(
                "  face#{i} area={:7.4} nova={} voto={:14} final={}",
                f.area,
                u8::from(nova[i]),
                nome_de(so_voto[i]),
                nome_de(d[i])
            );
        }
        let orfas = d.iter().filter(|x| x.is_none()).count();
        let perdidos: Vec<u64> = (0..regs.len())
            .filter(|k| !d.contains(&Some(*k)))
            .map(|k| rs[k].0)
            .collect();
        println!("  ==> faces sem cor = {orfas} | preenchimentos que congelam = {perdidos:?}");
    }

    #[test]
    #[ignore = "sonda; roda com --ignored --nocapture"]
    fn o_report_de_02_09() {
        relata("drawing_base", "drawing_base");
        relata("drawing01", "drawing_base");
        relata("drawing02", "drawing_base");
        relata("drawing02", "drawing01");
    }

    /// Uma região lida do SVG: `(id do caminho, cor, a região)`.
    type RegiaoLida = (u64, String, Regiao);

    /// O veredito de um passo: `(dono por face, faces, rede, regiões)`.
    type Veredito = (
        Vec<Option<usize>>,
        Vec<ph2d_vec_fill::Face>,
        ph2d_vec_fill::Rede,
        Vec<RegiaoLida>,
    );

    fn passo(nome: &str, base: &str) -> Veredito {
        let alvo = ler(nome);
        let b = ler(base);
        let rs = regioes(&b);
        let regs: Vec<Regiao> = rs
            .iter()
            .map(|(_, _, r)| Regiao {
                poligonos: r.poligonos.clone(),
                semente: r.semente,
            })
            .collect();
        let ant = ph2d_vec_fill::rede(&paredes(&b));
        let r = ph2d_vec_fill::rede(&paredes(&alvo));
        let faces: Vec<_> = r.faces().into_iter().filter(|f| f.area > 0.0).collect();
        let mut d = donos(&r, &faces, &regs);
        let nova = terreno_novo(&r, &faces, &ant);
        let areas: Vec<f64> = faces.iter().map(|f| f.area).collect();
        herda_dos_vizinhos(&r.adjacencias(&faces), &areas, &nova, &mut d);
        (d, faces, r, rs)
    }

    /// ⭐⭐⭐ **O CASO DE IDENTIDADE: alimentar um estado a si próprio não muda NADA.**
    ///
    /// É o desenho real do Enio (`drawing_base.svg`, exportado por ele): sete preenchimentos sobre
    /// uma rede soldada de doze arcos. Recozer sem mexer numa linha tem de devolver **cada face ao
    /// seu preenchimento** — nem uma órfã, nem um congelado.
    ///
    /// ⚠️⚠️ **Foi este gate que apanhou a sonda a mentir.** A 1.ª redacção dela punha a semente no
    /// PRIMEIRO PONTO do polígono — em cima da fronteira, não dentro — e o caso de identidade
    /// deixava de se reproduzir: uma face mudava de dono e um preenchimento congelava. *Nada disso
    /// vinha do produto; vinha da régua.*
    #[test]
    fn feeding_a_state_back_to_itself_changes_nothing() {
        let (d, faces, _, rs) = passo("drawing_base", "drawing_base");
        assert_eq!(faces.len(), 7, "a cena do report tem sete regioes");
        assert!(
            d.iter().all(std::option::Option::is_some),
            "nenhuma face pode ficar sem cor: {d:?}"
        );
        let mut vistos: Vec<usize> = d.iter().flatten().copied().collect();
        vistos.sort_unstable();
        vistos.dedup();
        assert_eq!(
            vistos.len(),
            rs.len(),
            "todo preenchimento tem de ficar com a SUA face — nenhum congela"
        );
    }

    /// ⭐⭐⭐ **PUXAR UM NÓ NÃO APAGA UMA ÁREA** — o report de 2026-09-02: *"puxei um nó para
    /// esquerda e apagou a área da seta"*.
    ///
    /// A fixtura são os dois ficheiros que ele exportou. ⚠️ **Um passo do estado base para o estado
    /// dele não perde face nenhuma**; o que o app mostrou perdeu, e a diferença são os quadros
    /// INTERMÉDIOS — é para eles que a semente passou a mandar.
    #[test]
    fn pulling_a_node_erases_no_area() {
        let (d, faces, _, rs) = passo("drawing01", "drawing_base");
        assert!(faces.len() > 7, "o no' puxado parte a rede em mais faces");
        assert!(
            d.iter().all(std::option::Option::is_some),
            "nenhuma face pode ficar sem cor depois do arrasto: {d:?}"
        );
        let mut vistos: Vec<usize> = d.iter().flatten().copied().collect();
        vistos.sort_unstable();
        vistos.dedup();
        assert_eq!(vistos.len(), rs.len(), "e nenhum preenchimento congela");
    }

    /// ⭐⭐⭐ **NINGUÉM PINTA POR CIMA DA SEMENTE DE OUTRO** — o *"pintou outra área com a cor
    /// errada"* do mesmo report.
    ///
    /// ⚠️⚠️ **A receita DERIVA**: o que um quadro decide vira a régua do seguinte, e um único quadro
    /// de topologia confusa reatribui a tinta para sempre. Medido nos ficheiros dele: a partir do
    /// estado `drawing01`, o corpo do círculo direito (área `2,83`) vota **azul** — o app tinha-o
    /// **verde**, ganho num quadro intermédio e nunca devolvido. A semente é o clique do artista, e
    /// não deriva: *uma face que contém a semente de alguém é dessa pessoa.*
    #[test]
    fn nobody_paints_over_someone_elses_seed() {
        for (alvo, base) in [
            ("drawing_base", "drawing_base"),
            ("drawing01", "drawing_base"),
            ("drawing02", "drawing_base"),
            ("drawing02", "drawing01"),
        ] {
            let (d, faces, r, rs) = passo(alvo, base);
            for (i, f) in faces.iter().enumerate() {
                let poly = r.contorno(f);
                let Some(dono) = d[i] else { continue };
                // ⚠️ **Várias sementes dentro é a FUSÃO, e é legítima** — a lei não é *"só uma pode
                // lá estar"*, é *"se alguma lá está, o dono é uma delas"*. Uma asserção mais forte
                // proibiria o caso que o gate `merging_two_regions…` exige.
                let dentro: Vec<u64> = rs
                    .iter()
                    .filter(|(_, _, reg)| ph2d_vec_scene::point_in_polygon(&poly, reg.semente))
                    .map(|(id, _, _)| *id)
                    .collect();
                assert!(
                    dentro.is_empty() || dentro.contains(&rs[dono].0),
                    "{alvo} (de {base}): a face #{i} (area {:.4}) foi para o preenchimento {}, \
                     que NAO tem semente nela — e {dentro:?} tem",
                    f.area,
                    rs[dono].0
                );
            }
        }
    }
}
