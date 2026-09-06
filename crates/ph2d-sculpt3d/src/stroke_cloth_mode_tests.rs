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
use crate::{Brush, Dab, SculptStroke, Symmetry};

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

/// ⭐⭐⭐ **GATE — o gesto que o tecido lê é o CAMINHO PROJECTADO no plano do
/// ecrã, e é o OLHO que define esse plano.**
///
/// ⛔⛔ **Esta costura não tinha régua nenhuma, e a razão é a fixtura**: as
/// grelhas deste ficheiro são planas e vistas de frente, e ali a projecção é um
/// **no-op** — o adaptador podia entregar o caminho 3D cru que nenhum gate desta
/// crate mudava de cor. *Uma lei que só é exercida por uma vista que a fixtura
/// não tem é uma lei sem gate.* A cura é inclinar o OLHO, não a malha.
///
/// A propriedade é observável pela porta pública, e tem DUAS metades:
/// - **inclinar o olho MUDA** o pano nos sete modos que lêem `δ`, porque o plano
///   do ecrã roda e a projecção do mesmo caminho passa a ser outra;
/// - **e NÃO muda** no arrasto, que é o único modo cuja direcção sai da
///   diferença dos dois pontos 3D (espec §4.2/§4.3).
///
/// ⚠️ **Sem a segunda metade o gate ficaria verde sobre um adaptador que
/// projectasse TUDO**, arrasto incluído.
#[test]
fn o_olho_define_o_plano_em_que_o_tecido_le_o_gesto() {
    let de_frente = [0.0f32, 0.0, -1.0];
    let inclinado = {
        // ⚠️ A inclinação tem de ser no eixo do TRAÇO: com ela no eixo `y` o
        // caminho, que corre em `x`, já é perpendicular ao olho, a projecção não
        // tira nada — a primeira redacção deste gate era VAZIA por isso.
        let v = [0.6f32, 0.0, -1.0];
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / l, v[1] / l, v[2] / l]
    };
    let correr = |olho: [f32; 3], modo: crate::ClothMode| -> Vec<[f32; 3]> {
        let mut mesh = plano();
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        let b = Brush {
            cloth_mode: modo,
            ..pincel()
        };
        for k in 0..8 {
            // ⚠️ O caminho é derivado dos CENTROS pelo traço, não do construtor
            // — e ele é o mesmo nas duas corridas. O que muda é só o olho.
            let c = [0.02 * k as f32, 0.0, 0.0];
            s.dab(
                &mut mesh,
                &b,
                &Dab::at(c, b.radius, olho),
                Symmetry::default(),
            );
        }
        mesh.positions().to_vec()
    };
    let antes = plano();
    let frente_grab = correr(de_frente, crate::ClothMode::Grab);
    let movidos = frente_grab
        .iter()
        .zip(antes.positions())
        .filter(|(a, b)| a != b)
        .count();
    assert!(movidos > 100, "so' {movidos} movidos -- vacuo");
    assert_ne!(
        frente_grab,
        correr(inclinado, crate::ClothMode::Grab),
        "inclinar o OLHO nao mudou o pano no Grab -- o adaptador esta a entregar o \
         caminho 3D cru onde a lei pede a projeccao no plano do ecra (espec §4.3)"
    );
    assert_eq!(
        correr(de_frente, crate::ClothMode::Drag),
        correr(inclinado, crate::ClothMode::Drag),
        "inclinar o OLHO mudou o ARRASTO -- ele e' o unico modo cuja direccao sai da \
         diferenca dos dois pontos 3D, e nao de `δ`"
    );
}
