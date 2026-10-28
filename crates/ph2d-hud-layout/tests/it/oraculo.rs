//! O kiwisolver CORRIDO (`docs/BlenderConstraints/ferramentas/oraculo_kiwisolver_hud.py`) contra o nosso, caso a caso.

use std::collections::BTreeMap;

use ph2d_hud_layout::{Arranjo, Caixa, Elemento, Forca, Relacao};
use serde::Deserialize;

/// O ótimo destes sistemas é ÚNICO (o oráculo grava `unico`: a mesma resposta com a ordem de inserção trocada) e
/// os números passam pela grelha diádica — dois simplex que chegam ao mesmo vértice dão a mesma conta. Medido em
/// 27/10 com tolerância `0.0`: verde (ao bit). A folga de `1e-9` só cobre um kiwisolver futuro com outro caminho.
const TOLERANCIA: f64 = 1e-9;

#[derive(Deserialize)]
struct Fixture {
    casos: Vec<Caso>,
}

#[derive(Deserialize)]
struct Caso {
    nome: String,
    familia: String,
    canvas: Caixa,
    elementos: Vec<Elemento>,
    relacoes: Vec<(Relacao, Forca)>,
    quadros: Vec<Quadro>,
    unico: bool,
}

#[derive(Deserialize)]
struct Quadro {
    canvas: Caixa,
    conteudos: Vec<Option<f64>>,
    caixas: Vec<Caixa>,
}

#[test]
fn the_layout_matches_kiwisolver_case_by_case() {
    let f: Fixture =
        serde_json::from_str(include_str!("../../fixtures/kiwisolver_1_5_1_hud.json")).unwrap();
    let mut familias: BTreeMap<&str, usize> = BTreeMap::new();
    let mut falhas = Vec::new();
    for caso in &f.casos {
        assert!(
            caso.unico,
            "{}: o ótimo não é único — a fixture não pode ser régua",
            caso.nome
        );
        *familias.entry(caso.familia.as_str()).or_default() += 1;
        let mut arr = Arranjo::new(caso.canvas, &caso.elementos, &caso.relacoes);
        for (q, quadro) in caso.quadros.iter().enumerate() {
            let nosso = arr.atualiza(quadro.canvas, &quadro.conteudos);
            if nosso.len() != quadro.caixas.len() {
                falhas.push(format!(
                    "{} q{q}: {} caixas, oráculo {}",
                    caso.nome,
                    nosso.len(),
                    quadro.caixas.len()
                ));
                continue;
            }
            for (i, (n, o)) in nosso.iter().zip(&quadro.caixas).enumerate() {
                let d = [
                    (n.x - o.x).abs(),
                    (n.y - o.y).abs(),
                    (n.w - o.w).abs(),
                    (n.h - o.h).abs(),
                ];
                if d.iter().any(|d| d.is_nan() || *d > TOLERANCIA) {
                    falhas.push(format!(
                        "{} q{q} e{i}: nosso {n:?} oráculo {o:?}",
                        caso.nome
                    ));
                }
            }
        }
    }
    for fam in [
        "ao_lado",
        "pilha",
        "centrado",
        "nao_invade",
        "largura_com_forca",
    ] {
        assert!(
            familias.get(fam).copied().unwrap_or(0) >= 2,
            "família {fam} com menos de 2 casos: {familias:?}"
        );
    }
    assert!(
        falhas.is_empty(),
        "{} divergências:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}
