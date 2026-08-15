//! **O PERCURSO É LIMITADO** — os gates do batente de memória do [`crate::stroke::Stroke`].
//!
//! Nascidos de um OOM REAL (2026-08-14): um integrador que divergiu entregou ao percurso uma
//! posição absurda, o passo tem PISO de 1 px, e o laço escreveu um `Dab` por pixel até a RAM
//! acabar — **90,2 GB de RSS**, o processo morto pelo kernel e a janela do editor derrubada junto.
//!
//! ⚠️ **A causa daquele dia foi curada noutro lugar** (a tríade de sub-passos da fita, com gate
//! próprio). Estes gates existem para a PRÓXIMA origem de posição absurda — um `Transform`
//! degenerado, um `inf` vindo de um param, um integrador futuro — e o que eles compram é o **modo
//! de falha**: uma linha truncada que se vê, em vez da máquina no chão.

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};

fn spec() -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        ..Default::default()
    }
}

fn plain() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

/// Percorre `[0,0] → to` num traço só e devolve quantos dabs o percurso emitiu.
fn dabs_reaching(to: [f32; 2]) -> usize {
    let mut s = Stroke::new(spec(), plain(), 7);
    let mut out = Vec::new();
    s.begin(
        StrokePoint {
            pos: [0.0, 0.0],
            pressure: 1.0,
        },
        &mut out,
    );
    out.clear();
    s.extend(
        StrokePoint {
            pos: to,
            pressure: 1.0,
        },
        &mut out,
    );
    out.len()
}

/// **UMA POSIÇÃO ABSURDA TRUNCA O PERCURSO; NÃO ESGOTA A MÁQUINA.**
///
/// ⚠️ **O oráculo é a CONTAGEM, e tem de ser** — o modo de falha que este gate vigia não produz um
/// pixel errado nem uma asserção vermelha: ele produz um `Vec` que cresce até o alocador desistir,
/// e nesse regime **o teste nunca chega ao próprio `assert`** (foi exactamente assim que o OOM de
/// 2026-08-14 se apresentou: a suíte parada, sem `ok` e sem falha). Contar os dabs é a pergunta que
/// termina.
#[test]
fn an_absurd_position_truncates_the_walk_instead_of_eating_the_machine() {
    // Cem milhões de pixels: 8 600× a diagonal da maior tela que o app abre, e ainda MUITO abaixo do
    // que uma mola instável alcança em dois quadros.
    let n = dabs_reaching([1.0e8, 0.0]);
    assert!(
        n <= crate::stroke::MAX_DABS_PER_WALK,
        "o percurso emitiu {n} dabs -- o batente de memória não segurou"
    );
    // CONTROLE: sem o batente esta corda emitiria ~1e6/2,4 ≈ 416 666 dabs; com ele, para no teto.
    assert_eq!(
        n,
        crate::stroke::MAX_DABS_PER_WALK,
        "premissa: nesta distância é o batente que termina o laço, não a corda"
    );
}

/// **UM GESTO REAL NUNCA O ALCANÇA** — o outro lado do mesmo número.
///
/// ⚠️ Sem esta metade o batente poderia ser posto em `8` e o gate acima continuaria verde, com o
/// produto a cortar todo traço longo. *Um teto sem o seu controle é um teto que ninguém sabe se
/// está no lugar certo.*
#[test]
fn the_longest_real_gesture_never_reaches_the_backstop() {
    // A diagonal da maior tela que o app abre, atravessada numa tacada só.
    let diag = (8192.0f32 * 8192.0 + 8192.0 * 8192.0).sqrt();
    let n = dabs_reaching([diag, 0.0]);
    assert!(
        n < crate::stroke::MAX_DABS_PER_WALK,
        "a maior corda legítima ({diag:.0} px) bateu no batente com {n} dabs -- ele está baixo demais"
    );
}

/// **UMA CORDA NÃO-FINITA É RECUSADA, e o traço sobrevive.**
///
/// ⚠️ `NaN` não é só *"mais um número grande"*: com ele **toda comparação é falsa**, então o `break`
/// do laço nunca dispara e nem o batente de contagem seria alcançado pelo caminho que o desenho
/// pretende. Recusar cedo é o barato — não há caminho a desenhar até um ponto que não existe.
#[test]
fn a_non_finite_target_is_refused_and_the_stroke_survives() {
    assert_eq!(
        dabs_reaching([f32::NAN, 0.0]),
        0,
        "um alvo NaN não pode produzir tinta"
    );
    assert_eq!(
        dabs_reaching([f32::INFINITY, 0.0]),
        0,
        "um alvo infinito não pode produzir tinta"
    );
    // CONTROLE: a MESMA fixture com um alvo são pinta — senão este gate afirmaria que o percurso
    // nunca pinta nada.
    assert!(
        dabs_reaching([240.0, 0.0]) > 10,
        "controle: um alvo são tem de pintar"
    );
}
