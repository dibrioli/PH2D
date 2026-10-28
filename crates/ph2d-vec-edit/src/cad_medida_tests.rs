//! ⏱ **O pior passo do arrasto NO EDITOR** (W4.4, `docs/BlenderConstraints/04_W4_cad.md` §3.4): as cenas da W4.0
//! (escada 202, misto 200) postas numa `VecScene`, arrastadas pela caneta (`on_drag_encaixe`: folha da cena, `resolve`,
//! `aplica`) e, intercalado no MESMO processo, pelo `resolve` cru da W4.3 — a diferença é o preço do editor. 7 rodadas
//! com a ordem rodada, resumo = o MÍNIMO (com a mediana ao lado); cada passo cumpre a régua da bancada a 1e-4 px.
//! Também: juntar uma regra (`junta_regra`, o teste de posto) e a análise das cores (`estado`, fora do quadro).
//! Sonda, não gate: `bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-vec-edit -- --ignored --nocapture
//! the_worst_drag_step_in_the_editor`.

#[allow(
    dead_code,
    reason = "o carregador é o dos gates do solver; aqui só se usa a parte do arrasto"
)]
#[path = "../../ph2d-vec-constraints/tests/it/cena.rs"]
mod cena;

use std::time::Instant;

use ph2d_vec_constraints::{Arrasto, Folha, Solucao, a_mais, estado, resolve};
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, SegCad, circulo_de};
use ph2d_vec_scene::{VecPath, VecScene, VecXforms, Xform};

use crate::PenTool;
use crate::junta_regra;
use crate::snap::SnapResult;
use cena::{Cena, TOL_PX, pior, xy};

const RODADAS: usize = 7;

/// A cena da bancada numa `VecScene`, com os MESMOS ids de path (os da tradução das regras).
fn na_cena(c: &Cena) -> VecScene {
    let mut s = VecScene::new();
    let mut tapa = Vec::new();
    let mut paths: Vec<&VecPath> = c.paths.iter().collect();
    paths.sort_by_key(|p| p.id);
    let mut proximo = 0;
    for p in paths {
        while proximo < p.id {
            tapa.push(s.push_path(VecPath::default()));
            proximo += 1;
        }
        assert_eq!(s.push_path(p.clone()), p.id);
        proximo += 1;
    }
    for t in tapa {
        s.remove_path(t);
    }
    *s.regras_mut() = c.regras.clone();
    s
}

fn posicoes(c: &Cena, s: &VecScene) -> Vec<[f64; 2]> {
    c.ponto
        .iter()
        .map(|&p| match p {
            PontoCad::Vert(id, v) => {
                s.path(id)
                    .and_then(|q| q.vert_by_id(v))
                    .expect("vértice")
                    .anchor
            }
            PontoCad::Centro(id) => {
                circulo_de(s.path(id).expect("círculo"), Xform::IDENTITY)
                    .expect("é círculo")
                    .0
            }
        })
        .collect()
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

/// O pior passo pela caneta; cada passo cumpre a régua.
fn pela_caneta(c: &Cena, base: &VecScene, i: usize, caminho: &[[f64; 2]]) -> f64 {
    let mut s = base.clone();
    let mut pen = PenTool::new();
    let PontoCad::Vert(id, v) = c.ponto[i] else {
        panic!("o arrastado é um vértice");
    };
    let em = s
        .path(id)
        .and_then(|p| p.vert_by_id(v))
        .expect("vértice")
        .anchor;
    pen.on_press_node(&mut s, em, 1e-3, false);
    let mut pior_ms: f64 = 0.0;
    for (k, &alvo) in caminho.iter().enumerate() {
        let t = Instant::now();
        assert!(pen.on_drag_encaixe(&mut s, alvo, &mut |_| SnapResult::default()));
        pior_ms = pior_ms.max(ms(t));
        let viola = pior(c, &posicoes(c, &s), Some((i, alvo)));
        assert!(viola <= TOL_PX, "passo {k}: viola {viola:e} px");
    }
    pen.on_release();
    pior_ms
}

/// O pior passo do `resolve` cru (a W4.3), sobre a mesma folha.
fn cru(c: &Cena, i: usize, caminho: &[[f64; 2]]) -> f64 {
    let folha = c.folha();
    let mut anterior: Option<Solucao> = None;
    let mut pior_ms: f64 = 0.0;
    for &alvo in caminho {
        let t = Instant::now();
        let r = resolve(
            &folha,
            &c.regras,
            Some(Arrasto {
                ponto: c.ponto[i],
                alvo,
            }),
            anterior.as_ref(),
        );
        pior_ms = pior_ms.max(ms(t));
        anterior = Some(r.solucao);
    }
    pior_ms
}

fn resumo(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

#[test]
#[ignore = "sonda de velocidade: corre em --profile smoke, à mão"]
fn the_worst_drag_step_in_the_editor() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", carga.trim());
    println!(
        "| cena | caneta (editor) | resolve cru | juntar uma regra | `a_mais` cru | análise das cores |"
    );
    println!("|---|---|---|---|---|---|");
    for nome in ["escada_202", "misto_200"] {
        let c = cena::cena(nome);
        let base = na_cena(&c);
        let i =
            usize::try_from(c.json["arrasto"]["ponto"].as_u64().expect("ponto")).expect("ponto");
        let caminho: Vec<[f64; 2]> = c.json["arrasto"]["caminho"]
            .as_array()
            .expect("caminho")
            .iter()
            .map(xy)
            .collect();
        // Aquece fora da régua.
        pela_caneta(&c, &base, i, &caminho);
        cru(&c, i, &caminho);
        let (mut a, mut b, mut j, mut e) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut q = Vec::new();
        for rodada in 0..RODADAS {
            if rodada % 2 == 0 {
                a.push(pela_caneta(&c, &base, i, &caminho));
                b.push(cru(&c, i, &caminho));
            } else {
                b.push(cru(&c, i, &caminho));
                a.push(pela_caneta(&c, &base, i, &caminho));
            }
            // Uma H nova entre dois pontos soltos que nenhuma regra liga ainda: o teste de posto inteiro.
            let mut s = base.clone();
            let lei = Lei::Horizontal(SegCad(c.ponto[0], c.ponto[c.ponto.len() / 2]));
            let t = Instant::now();
            junta_regra(&mut s, &VecXforms::default(), lei);
            j.push(ms(t));
            // O mesmo teste de posto cru, sobre a folha da bancada: o resto é o preço da porta do editor.
            let folha = c.folha();
            let t = Instant::now();
            let _ = a_mais(&folha, &c.regras, RegraCad::nova(lei));
            q.push(ms(t));
            let t = Instant::now();
            let _ = estado(
                &Folha::new(c.paths.iter().map(|p| (p, Xform::IDENTITY))),
                &c.regras,
            );
            e.push(ms(t));
        }
        let f = |(m, md): (f64, f64)| format!("{m:.3} · {md:.3}");
        println!(
            "| {nome} | {} | {} | {} | {} | {} |",
            f(resumo(&mut a)),
            f(resumo(&mut b)),
            f(resumo(&mut j)),
            f(resumo(&mut q)),
            f(resumo(&mut e))
        );
    }
}
