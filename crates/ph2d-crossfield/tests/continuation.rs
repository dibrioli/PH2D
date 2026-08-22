//! **OS GATES DA CONTINUAÇÃO** — o ponto de extensão do alinhamento.
//!
//! ⚠️ **Os dois lados, de propósito** (`reference_topic_gate_discipline`): um gate
//! que só provasse a **ausência** (com `align = 0` nada muda) ficaria verde sobre
//! uma continuação que não faz nada; um que só provasse a **presença** ficaria
//! verde sobre uma que mexe no caminho que shipa. *São duas afirmações e precisam
//! de dois testes.*

use ph2d_crossfield::{Continuation, Dual, Rounding, solve_miq_continued, solve_miq_with};
use ph2d_mesh::{Mesh, shapes};

/// A lei antiga, escrita por extenso: decidir de uma vez, sem semente.
const RAW: Continuation = Continuation {
    recentres: 0,
    warm: false,
    ramp_steps: 0,
};

fn tri(mut m: Mesh) -> Mesh {
    m.triangulate();
    m
}

/// ⭐⭐ **AUSÊNCIA — com o peso a zero a continuação é INERTE, byte a byte.**
///
/// ⚠️ **É a metade que garante que a semente não é um custo escondido.** Uma
/// [`Continuation`] que gastasse uma resolução a mais com `align = 0` estaria a
/// cobrar por uma feature que naquele ponto não existe — e o `align = 0` é o
/// caminho de recurso do produto (`quad_remesh_global` cai nele quando a cadeia
/// alinhada recusa), não um caso de laboratório.
#[test]
fn the_continuation_is_inert_without_alignment() {
    for (name, mesh) in [
        ("esfera 24x36", tri(shapes::uv_sphere(24, 36, 1.0))),
        ("toro", tri(shapes::torus(48, 24, 1.0, 0.35))),
    ] {
        let dual = Dual::build(&mesh);
        let (base, br) = solve_miq_continued(&dual, Rounding::default(), 0.0, RAW);
        let (cont, cr) =
            solve_miq_continued(&dual, Rounding::default(), 0.0, Continuation::default());
        assert_eq!(
            base, cont,
            "{name}: a continuacao mexeu no campo sem alinhamento"
        );
        assert_eq!(
            br.solves, cr.solves,
            "{name}: a continuacao gastou resolucoes sem alinhamento"
        );
        assert_eq!(
            cr.recentres, 0,
            "{name}: re-centrou sem ter termo a re-centrar"
        );

        // ⭐⭐ **E ESTA LINHA É O TRIPWIRE DO `ALIGN_WEIGHT`.** Enquanto ele for
        // `0`, o caminho do produto tem de ser byte-idêntico à lei antiga; no dia
        // em que alguém o mover, é aqui que ele repara — e o doc da constante
        // explica-lhe porque é que ela está a zero apesar de o termo funcionar (o
        // toro perde o buraco). ⛔ Se esta asserção reprovar, o conserto é o gate
        // `the_genus_survives_every_alignment_weight`, não esta linha.
        let (shipped, _) = solve_miq_with(&dual, Rounding::default());
        assert_eq!(
            base, shipped,
            "{name}: o caminho do produto divergiu da lei so'-suavidade -- \
             o ALIGN_WEIGHT saiu de zero? leia o doc dele antes de mexer aqui"
        );
    }
}

/// ⭐⭐ **PRESENÇA — com alinhamento o aquecimento MUDA o campo.**
///
/// ⛔ **Sem este lado, `warm` podia ser um `if` morto e o gate irmão continuaria
/// verde** — e o irmão é hoje um gate de *inércia*, que passaria feliz sobre uma
/// continuação que não faz nada.
///
/// ⚠️ **E ele é o que mantém a semente VIVA enquanto o `ALIGN_WEIGHT` é zero.** O
/// termo está desligado por causa de um defeito de topologia no traçado, não por
/// causa desta peça: a semente é a cura medida da explosão do arredondamento
/// (`PLAN.md` §4-duovicies), e código que nenhum teste exercita apodrece antes de o
/// bloqueio sair da frente.
///
/// ⚠️ **A fixtura tem de CONTER o fenómeno** (`reference_topic_fixture_discipline`):
/// numa esfera perfeita as duas curvaturas são iguais, a anisotropia é ~zero e o
/// termo encolhe-se sozinho — *ali as duas leis dariam o mesmo resultado por razão
/// errada.* O toro tem anisotropia real em toda a parte.
#[test]
fn the_warm_start_changes_the_field_when_alignment_is_on() {
    let mesh = tri(shapes::torus(48, 24, 1.0, 0.35));
    let dual = Dual::build(&mesh);
    let (raw, _) = solve_miq_continued(&dual, Rounding::default(), 0.05, RAW);
    let (warm, wr) = solve_miq_continued(
        &dual,
        Rounding::default(),
        0.05,
        Continuation {
            recentres: 0,
            warm: true,
            ramp_steps: 0,
        },
    );
    assert_ne!(
        raw, warm,
        "o aquecimento nao mudou nada -- ou ele e' um `if` morto, ou a fixtura nao tem anisotropia"
    );
    assert!(
        wr.solves > 0,
        "o aquecimento tem de custar resolucoes: ele e' uma passagem inteira"
    );
}

/// ⭐ **A RE-CENTRAGEM só fica com uma rodada que MELHOROU o objectivo.**
///
/// ⚠️ **O contador [`ph2d_crossfield::SolveReport::recentres`] é a prova.** Ele
/// conta as rodadas **aceites**, não as tentadas — então `recentres <= pedidas` é a
/// afirmação de que o laço nunca fica com uma piora, e o gate morre se alguém
/// trocar o critério de saída por "corre sempre as N".
#[test]
fn a_recentre_is_only_kept_when_it_improves() {
    let mesh = tri(shapes::torus(48, 24, 1.0, 0.35));
    let dual = Dual::build(&mesh);
    for asked in [1usize, 4] {
        let (_, rep) = solve_miq_continued(
            &dual,
            Rounding::default(),
            0.05,
            Continuation {
                recentres: asked,
                warm: true,
                ramp_steps: 0,
            },
        );
        assert!(
            rep.recentres <= asked,
            "aceitou {} re-centragens tendo pedido {asked}",
            rep.recentres
        );
    }
}
