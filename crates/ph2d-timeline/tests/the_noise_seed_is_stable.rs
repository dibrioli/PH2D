//! **O `__seed` de uma binding — o que ele garante, e o que ele NÃO garante** (B5 do plano 12).
//!
//! O plano abria com uma suspeita: *"se `target` é alocado por ordem de criação, **adicionar
//! uma track re-rola o Jitter de todos**. Medir."*
//!
//! ⛔ **MEDIDO E REFUTADO.** O alocador é um contador **monotônico por binding**
//! (`TimelineDoc::bind`), então uma binding nova toma o PRÓXIMO número e não toca nenhum
//! outro. A tabela medida:
//!
//! | gesto | o seed de quem já existia |
//! |---|---|
//! | criar outra track (outro objeto, ou outra prop) | **não se move** |
//! | salvar e abrir de novo | **não se move** (o `target` viaja no arquivo) |
//! | apagar a track e criá-la de novo | **MOVE** (0 → 300 na medição) |
//! | o documento inteiro RESETAR (último objeto apagado) | volta a contar do zero |
//!
//! ⛔ **E a cura que o plano propunha foi MEDIDA e REJEITADA — não a refaça sem um número
//! novo.** Semear de `stable_name_id(Name)` (o `wire_id` que a binding já carrega) tornaria o
//! seed estável através do re-bind, mas:
//!
//! 1. **Re-rola TODA arte já salva, uma vez.** Todo `wiggle`/`jitter` de todo projeto em
//!    disco passaria a desenhar outro tremor. Isso não é migração de dado, é mudança de
//!    aparência de trabalho pronto.
//! 2. **Troca um re-roll por outro:** o seed passaria a mudar quando o artista **RENOMEIA** o
//!    objeto — um gesto mais comum, num editor, do que apagar uma track e recriá-la.
//! 3. E teria de misturar a `prop` no hash, senão X e Y do mesmo objeto passariam a tremer
//!    **idênticos** — o que não é ruído, é uma diagonal.
//!
//! O que estes gates fazem é **pinar o que é verdade** (para ninguém re-derivar a premissa
//! falsa) e **deixar o resíduo num teste executável** em vez de numa nota que envelhece.

use ph2d_timeline::{PropKind, TimelineIntent as I, TimelineState, apply_intent, seed_of_target};

fn bind(st: &mut TimelineState, e: u64, p: PropKind) -> u64 {
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(st, &mut ph, I::Bind { entity: e, prop: p });
    st.doc
        .binding_for(e, p)
        .expect("a binding existe depois do Bind")
        .target
        .get()
}

fn seed_of(st: &TimelineState, e: u64, p: PropKind) -> f32 {
    seed_of_target(st.doc.binding_for(e, p).expect("bound").target.get())
}

/// **Criar uma track nova não move o tremor de ninguém.**
///
/// A refutação, pinada. É a premissa que o plano pedia para medir, e é a que mais assusta:
/// se fosse verdade, animar um objeto novo re-desenharia o ruído da cena inteira.
///
/// **Mutação que deve sangrar:** `bind` reatribuir os targets (ex.: `target = índice na
/// lista de bindings`), que é exactamente a implementação que a suspeita descrevia.
#[test]
fn adding_a_track_does_not_move_anyone_elses_seed() {
    let mut st = TimelineState::new();
    bind(&mut st, 1, PropKind::TranslationX);
    bind(&mut st, 2, PropKind::TranslationX);
    let before = [
        seed_of(&st, 1, PropKind::TranslationX),
        seed_of(&st, 2, PropKind::TranslationX),
    ];

    // Mais três tracks, em objetos e props diferentes.
    bind(&mut st, 3, PropKind::TranslationY);
    bind(&mut st, 1, PropKind::Rotation);
    bind(&mut st, 4, PropKind::Opacity);

    assert_eq!(
        before,
        [
            seed_of(&st, 1, PropKind::TranslationX),
            seed_of(&st, 2, PropKind::TranslationX),
        ],
        "o alocador é monotônico POR BINDING: quem já existia mantém o número, logo o tremor"
    );
}

/// **Duas bindings nunca dividem um seed** — nem dois objetos, nem duas props do mesmo.
///
/// A segunda metade importa tanto quanto a primeira: X e Y do mesmo objeto com o MESMO seed
/// não seria ruído, seria uma diagonal.
#[test]
fn no_two_bindings_share_a_seed() {
    let mut st = TimelineState::new();
    for e in 1..=4u64 {
        for p in [
            PropKind::TranslationX,
            PropKind::TranslationY,
            PropKind::Rotation,
        ] {
            bind(&mut st, e, p);
        }
    }
    let mut seeds: Vec<u32> = st
        .doc
        .bindings()
        .iter()
        .map(|b| seed_of_target(b.target.get()).to_bits())
        .collect();
    let n = seeds.len();
    seeds.sort_unstable();
    seeds.dedup();
    assert_eq!(
        seeds.len(),
        n,
        "{n} bindings, {} seeds distintos",
        seeds.len()
    );
}

/// **O tremor sobrevive a fechar e reabrir o projeto.**
///
/// É a propriedade que de fato importa para o artista: a arte que ele aprovou ontem tem de
/// desenhar a mesma coisa hoje. O `target` viaja no arquivo (o `entity` não — ele é
/// re-resolvido pelo nome), e é por isso que o seed atravessa a sessão.
///
/// **Mutação que deve sangrar:** `reseat_allocator` renumerar as bindings carregadas.
#[test]
fn the_seed_survives_a_save_and_a_load() {
    let mut st = TimelineState::new();
    bind(&mut st, 1, PropKind::TranslationX);
    bind(&mut st, 2, PropKind::TranslationX);
    bind(&mut st, 3, PropKind::Rotation);
    let before: Vec<f32> = st
        .doc
        .bindings()
        .iter()
        .map(|b| seed_of_target(b.target.get()))
        .collect();

    let bytes = st.doc.to_bytes().expect("serializa");
    let mut doc = ph2d_timeline::TimelineDoc::from_bytes(&bytes).expect("desserializa");
    doc.reseat_allocator();
    let after: Vec<f32> = doc
        .bindings()
        .iter()
        .map(|b| seed_of_target(b.target.get()))
        .collect();

    assert_eq!(before, after, "o seed é o mesmo depois do round-trip");
}

/// **O RESÍDUO, num teste executável: apagar a track e recriá-la re-rola o tremor.**
///
/// Não é um gate de regressão — é o registro do que hoje NÃO é garantido, com o número, para
/// que a próxima pessoa que abrir este item saiba que ele foi medido e por que a cura foi
/// recusada (ver o doc do módulo).
///
/// Se algum dia o seed passar a vir do `wire_id`, é ESTE teste que muda de sinal, e a
/// mudança tem de vir com a decisão de re-rolar a arte já salva.
#[test]
fn rebinding_the_same_property_re_rolls_the_noise_and_this_is_the_number() {
    let mut st = TimelineState::new();
    let first = bind(&mut st, 1, PropKind::TranslationX);
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut st,
        &mut ph,
        I::Unbind {
            entity: 1,
            prop: PropKind::TranslationX,
        },
    );
    let again = bind(&mut st, 1, PropKind::TranslationX);

    assert_ne!(
        seed_of_target(first),
        seed_of_target(again),
        "hoje o número NÃO volta: o alocador nunca reutiliza um target"
    );
    assert!(
        again > first,
        "e ele só anda para a frente ({first} -> {again}), que é o que garante o gate acima"
    );
}
