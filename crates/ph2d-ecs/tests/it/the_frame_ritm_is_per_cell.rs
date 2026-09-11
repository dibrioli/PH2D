//! **A duração por-QUADRO** (spec §8.12) — a recusa que o importador de `.ase` reabriu.
//!
//! ⚠️ Ela foi recusada por *«não há quem produza durações por-quadro»*. Há: o importador de
//! Aseprite, construído em 2026-08-23 — e nos ficheiros REAIS elas variam (o `example.ase` de
//! terceiros vai de 50 a 500 ms, com as três tags a variar por dentro). *Quem move o número que
//! tornava algo inalcançável tem de reconferir a nota.*
//!
//! ⚠️ **A lei já perguntava por frame.** O `step_ticks` chama a duração dentro do laço, com a
//! célula que está no ecrã — era só a resposta que era uniforme. É por isso que a feature coube
//! numa função em vez de numa refactoração.

use ph2d_ecs::{AnimationTag, SpriteAnimator, advance};

/// Um animador a tocar, sobre uma grelha de `cells` células.
fn playing() -> SpriteAnimator {
    let mut a = SpriteAnimator::new("walk");
    a.playing = true;
    a
}

/// Quantos microssegundos até o frame mudar, avançando de 1 ms de cada vez.
fn ticks_until_change(tag: &AnimationTag, cells: u32, from_frame: u32) -> u64 {
    let mut a = playing();
    let mut f = from_frame;
    // Põe o animador NA célula pedida sem contaminar o acumulador.
    advance(&mut a, tag, &mut f, cells, 0);
    f = from_frame;
    let mut total = 0;
    for _ in 0..200_000 {
        let before = f;
        advance(&mut a, tag, &mut f, cells, 1_000);
        total += 1_000;
        if f != before {
            return total;
        }
    }
    panic!("o frame nunca mudou");
}

/// **UM VETOR VAZIO É O COMPORTAMENTO DE SEMPRE** — a inércia, e a metade que impede a feature de
/// mexer no que já existia.
#[test]
fn an_empty_vector_is_the_uniform_rhythm() {
    let tag = AnimationTag {
        frame_ms: 100,
        ..AnimationTag::new("walk", 0, 3)
    };
    assert!(!tag.has_per_frame_timing());
    for cell in 0..3 {
        assert_eq!(
            ticks_until_change(&tag, 4, cell),
            100_000,
            "a celula {cell} tinha de durar o `frame_ms`"
        );
    }
}

/// **CADA CÉLULA DURA O QUE ELA DIZ** — o caso que um `.ase` real produz: um *hold* de antecipação
/// no meio da tag.
///
/// **Mutação que deve sangrar:** o `step_ticks` voltar a usar o `frame_ms` uniforme.
#[test]
fn each_cell_lasts_what_it_declares() {
    let tag = AnimationTag {
        frame_ms: 50,
        per_frame_ms: vec![50, 50, 400, 50],
        ..AnimationTag::new("walk", 0, 3)
    };
    assert!(tag.has_per_frame_timing());
    assert_eq!(ticks_until_change(&tag, 4, 0), 50_000);
    assert_eq!(ticks_until_change(&tag, 4, 2), 400_000, "o hold do meio");
    assert_eq!(ticks_until_change(&tag, 4, 3), 50_000);
}

/// **UM VETOR CURTO É VÁLIDO, e um `0` também** — os dois caem no `frame_ms`.
///
/// ⛔ Um vetor que tivesse de acompanhar o intervalo seria **estado inválido a guardar**: mexer no
/// `to` deixaria a tag num estado que o modelo não sabe descrever. Com o fallback, mexer no
/// intervalo **não invalida nada** — o vetor reindexa-se sozinho.
///
/// **Mutação que deve sangrar:** tirar o `.filter(|v| *v > 0)` (um `0` faria o laço de recuperação
/// girar até ao guarda).
#[test]
fn a_short_vector_and_a_zero_both_fall_back() {
    let tag = AnimationTag {
        frame_ms: 70,
        per_frame_ms: vec![120, 0],
        ..AnimationTag::new("walk", 0, 3)
    };
    assert_eq!(ticks_until_change(&tag, 4, 0), 120_000, "a que ele declara");
    assert_eq!(
        ticks_until_change(&tag, 4, 1),
        70_000,
        "o `0` cai no default"
    );
    assert_eq!(
        ticks_until_change(&tag, 4, 2),
        70_000,
        "e o que falta tambem"
    );
    assert!(
        tag.has_per_frame_timing(),
        "um valor >0 conta como ritmo proprio"
    );
}

/// **O VETOR INDEXA-SE A PARTIR DO `from`**, não da célula 0 da grelha — uma tag que começa em 4
/// tem o primeiro valor do vetor na célula 4.
///
/// **Mutação que deve sangrar:** indexar por `frame` em vez de `frame - lo`.
#[test]
fn the_vector_is_indexed_from_the_tags_own_start() {
    let tag = AnimationTag {
        frame_ms: 60,
        per_frame_ms: vec![300, 60, 60],
        ..AnimationTag::new("attack", 4, 6)
    };
    assert_eq!(
        ticks_until_change(&tag, 8, 4),
        300_000,
        "o primeiro valor pertence a' celula `from`, nao a' celula 0"
    );
    assert_eq!(ticks_until_change(&tag, 8, 5), 60_000);
}

/// **O `hold_ms` continua a somar-se ao ritmo próprio** — os dois resolvem coisas diferentes: este
/// é o ritmo interno, aquele é a respiração entre voltas.
#[test]
fn the_hold_still_adds_on_top_of_the_last_cells_own_time() {
    let tag = AnimationTag {
        frame_ms: 50,
        hold_ms: 200,
        per_frame_ms: vec![50, 50, 50, 90],
        ..AnimationTag::new("walk", 0, 3)
    };
    // A última célula dura o que ela declara (90) MAIS o hold (200).
    assert_eq!(ticks_until_change(&tag, 4, 3), 290_000);
}

/// **E o cap da spec vale para cada célula** — um ficheiro adulterado com `0` ou com um milhão de
/// ms não escapa pelo caminho novo.
#[test]
fn the_spec_cap_holds_for_every_cell() {
    let tag = AnimationTag {
        frame_ms: 50,
        per_frame_ms: vec![u32::MAX, 50, 50, 50],
        ..AnimationTag::new("walk", 0, 3)
    };
    assert_eq!(
        ticks_until_change(&tag, 4, 0),
        u64::from(ph2d_ecs::FRAME_MS_MAX) * 1_000,
        "o teto da spec tem de valer por celula, nao so' no campo do painel"
    );
}
