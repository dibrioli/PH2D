//! ⭐⭐ **OS SELECTORES DO PINCEL DE TECIDO CHEGAM AO MOTOR** — o modo de
//! deformação e a área simulada.
//!
//! ⛔⛔ **É a pergunta que nenhum instrumento desta casa faz** (CLAUDE.md §5.0):
//! o censo de registo mede se o chip é focalizável e os `seam_*` provam que o
//! clique chega à ferramenta — **nenhum pergunta se o VALOR chega a um
//! consumidor**. Um selector que o adaptador lesse e a matemática descartasse
//! passaria em todos eles.
//!
//! Irmão do [`super::stroke_cloth_tests`], e o corte é *o que a LEI faz* (lá)
//! contra *o que o PAINEL alcança* (aqui).

use super::cloth_tests::{arrastar, pincel, plano};
use crate::Brush;

/// ⭐⭐⭐ **GATE — os OITO modos de deformação dão OITO panos diferentes.**
///
/// ⛔⛔ **É a pergunta que nenhum instrumento desta casa faz** (CLAUDE.md §5.0):
/// o `hit_indexed_ids_are_registered` mede se o chip é focalizável e os `seam_*`
/// provam que o clique chega à ferramenta — **nenhum pergunta se o VALOR chega a
/// um consumidor**. Um `cloth_mode` que o adaptador lesse e a matemática
/// descartasse passaria em todos eles.
///
/// ⚠️ **E o anti-vácuo é metade do gate:** sem o piso, oito modos que não
/// fizessem nada dariam oito panos idênticos ao repouso e a desigualdade
/// «todos diferentes» seria falsa — mas oito que fizessem a MESMA coisa também.
/// Por isso as duas metades: cada um move, e não há dois iguais.
#[test]
fn os_oito_modos_de_deformacao_dao_oito_panos_diferentes() {
    let antes = plano();
    let saidas: Vec<(crate::ClothMode, Vec<[f32; 3]>)> = crate::ClothMode::ALL
        .into_iter()
        .map(|modo| {
            let b = Brush {
                cloth_mode: modo,
                ..pincel()
            };
            let (m, _) = arrastar(8, &b);
            (modo, m.positions().to_vec())
        })
        .collect();
    for (modo, p) in &saidas {
        let pior = (0..antes.vert_count())
            .map(|v| {
                let (a, b) = (antes.positions()[v], p[v]);
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
            })
            .fold(0.0f32, f32::max);
        assert!(
            pior > 1e-4,
            "{modo:?} nao moveu NADA ({pior:.2e}) -- o chip escreve e ninguem le"
        );
    }
    for (i, (ma, pa)) in saidas.iter().enumerate() {
        for (mb, pb) in saidas.iter().skip(i + 1) {
            assert!(
                pa != pb,
                "{ma:?} e {mb:?} dao o MESMO pano ao bit -- um dos dois nao chega \
                 ao motor, ou a matematica descarta-o"
            );
        }
    }
}

/// ⭐⭐ **GATE — as TRÊS áreas simuladas dão três panos diferentes.**
///
/// A área decide o que entra na simulação, o centro da banda e — no *Local* — a
/// lista de restrições em duplicado (espec §2.1, §5.2-bis). Se ela não chegasse,
/// as três leriam igual.
#[test]
fn as_tres_areas_simuladas_dao_tres_panos_diferentes() {
    let saidas: Vec<(crate::ClothArea, Vec<[f32; 3]>)> = crate::ClothArea::ALL
        .into_iter()
        .map(|area| {
            let b = Brush {
                cloth_area: area,
                ..pincel()
            };
            let (m, _) = arrastar(8, &b);
            (area, m.positions().to_vec())
        })
        .collect();
    for (i, (aa, pa)) in saidas.iter().enumerate() {
        for (ab, pb) in saidas.iter().skip(i + 1) {
            assert!(pa != pb, "{aa:?} e {ab:?} dao o MESMO pano ao bit");
        }
    }
}
