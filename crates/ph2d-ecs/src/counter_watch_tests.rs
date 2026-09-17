//! Os gates da **lei** da vigia. A ponte, a porta e a costura têm os deles noutros sítios.

use super::*;
use bevy_ecs::world::World;

fn regra(compare: Compare, value: i64) -> CounterWatchRow {
    CounterWatchRow {
        counter: "vidas".into(),
        compare,
        value,
        signal: "morri".into(),
        once: false,
    }
}

/// ⭐⭐⭐ **A ARESTA: ela fala na SUBIDA, e cala-se enquanto a condição continuar satisfeita.**
///
/// ⚠️ **A fixtura contém o fenómeno:** ela **atravessa** o limiar (`3 → 0`) em vez de já lá estar —
/// sem a travessia, esta lei e um nível puro são indistinguíveis.
///
/// **Mutação que deve sangrar:** trocar `tem && !state.held` por `tem`.
#[test]
fn a_vigia_fala_na_subida_e_cala_se_no_planalto() {
    let r = regra(Compare::AtMost, 0);
    let mut s = born();
    assert!(!advance(&r, &mut s, Some(3)), "tres vidas nao e' morrer");
    assert!(!advance(&r, &mut s, Some(1)));
    assert!(advance(&r, &mut s, Some(0)), "a travessia tem de falar");
    for tique in 0..60 {
        assert!(
            !advance(&r, &mut s, Some(0)),
            "o tique {tique} do planalto voltou a falar — isto e' um NIVEL, nao uma aresta"
        );
    }
}

/// **E ela volta a falar se a condição se desfizer e se refizer.**
///
/// **Mutação que deve sangrar:** nunca voltar a pôr `held = false`.
#[test]
fn desfeita_e_refeita_e_uma_travessia_nova() {
    let r = regra(Compare::AtLeast, 10);
    let mut s = born();
    assert!(advance(&r, &mut s, Some(10)));
    assert!(!advance(&r, &mut s, Some(12)));
    assert!(!advance(&r, &mut s, Some(4)), "descer nao e' falar");
    assert!(advance(&r, &mut s, Some(10)), "a segunda travessia fala");
}

/// **`once` cala a segunda travessia — e só ela.**
///
/// ⚠️ **Duas travessias na fixtura, de propósito:** com uma só, `once` ligado e desligado dão a
/// mesma saída e o gate aprova a mutação que o apaga.
#[test]
fn o_so_uma_vez_cala_a_segunda_travessia() {
    let mut r = regra(Compare::AtLeast, 10);
    r.once = true;
    let mut s = born();
    assert!(advance(&r, &mut s, Some(10)));
    assert!(!advance(&r, &mut s, Some(0)));
    assert!(
        !advance(&r, &mut s, Some(10)),
        "com `once` a segunda travessia e' muda"
    );

    // O CONTROLO: a mesma sequência sem `once` fala duas vezes.
    r.once = false;
    let mut s = born();
    assert!(advance(&r, &mut s, Some(10)));
    assert!(!advance(&r, &mut s, Some(0)));
    assert!(advance(&r, &mut s, Some(10)));
}

/// ⭐⭐⭐ **Uma vigia sobre um contador que NÃO EXISTE nunca fala.**
///
/// ⚠️ É a linha que separa `None` de zero, e sem ela escrever `vidaas` num campo faria a regra
/// `AtMost 0` anunciar *«morreste»* no arranque. O **controlo** está ao lado: o mesmo limiar com o
/// contador a existir e a valer zero **fala**.
///
/// **Mutação que deve sangrar:** `soma.unwrap_or(0)`.
#[test]
fn um_contador_que_nao_existe_nunca_dispara() {
    let r = regra(Compare::AtMost, 0);
    let mut s = born();
    for _ in 0..10 {
        assert!(!advance(&r, &mut s, None), "ausente nao e' zero");
    }
    let mut s = born();
    assert!(advance(&r, &mut s, Some(0)), "o controlo: zero E' zero");
}

/// **As três comparações, nos dois lados de cada fronteira.**
#[test]
fn as_tres_comparacoes_dizem_o_que_dizem() {
    for (c, v, limiar, esperado) in [
        (Compare::AtMost, 0, 0, true),
        (Compare::AtMost, 1, 0, false),
        (Compare::AtMost, -1, 0, true),
        (Compare::AtLeast, 10, 10, true),
        (Compare::AtLeast, 9, 10, false),
        (Compare::AtLeast, 11, 10, true),
        (Compare::Exactly, 7, 7, true),
        (Compare::Exactly, 6, 7, false),
        (Compare::Exactly, 8, 7, false),
    ] {
        assert_eq!(
            c.holds(v, limiar),
            esperado,
            "{c:?}: {v} contra {limiar} devia dar {esperado}"
        );
    }
}

/// ⭐⭐ **Uma condição JÁ satisfeita no tique 0 fala UMA vez** — a decisão do `born()`.
///
/// **Mutação que deve sangrar:** `born()` devolver `held: true`.
#[test]
fn ja_satisfeita_ao_nascer_fala_uma_vez_e_so_uma() {
    let r = regra(Compare::AtMost, 0);
    let mut s = born();
    assert!(advance(&r, &mut s, Some(0)), "a porta nasce aberta e DI-LO");
    assert!(!advance(&r, &mut s, Some(0)));
}

/// **Uma regra sem nome de sinal é muda — e continua a seguir o mundo.**
///
/// ⚠️ A segunda metade é a que importa: escrever o nome mais tarde **não** pode fazer a regra
/// disparar sobre uma condição que já estava satisfeita há minutos.
///
/// **Mutação que deve sangrar:** devolver cedo **antes** de escrever o `held`.
#[test]
fn sem_nome_de_sinal_ela_cala_se_mas_segue_o_mundo() {
    let mut r = regra(Compare::AtMost, 0);
    r.signal = String::new();
    let mut s = born();
    assert!(!advance(&r, &mut s, Some(0)));
    assert!(s.held, "o `held` descreve o MUNDO, nao o que a regra fez");

    r.signal = "morri".into();
    assert!(
        !advance(&r, &mut s, Some(0)),
        "dar-lhe nome nao e' uma travessia"
    );
}

/// **O `reconcile` cria e apaga slots — e encolher esquece o que já não existe.**
#[test]
fn o_reconcile_cresce_e_encolhe() {
    let cfg = CounterWatch(vec![regra(Compare::AtMost, 0), regra(Compare::AtLeast, 5)]);
    let mut rt = CounterWatchRuntime::default();
    assert!(reconcile(&cfg, &mut rt));
    assert_eq!(rt.0.len(), 2);
    assert!(
        !reconcile(&cfg, &mut rt),
        "nada mudou ⇒ nao suja o componente"
    );

    rt.0[1].fired = true;
    let menor = CounterWatch(vec![regra(Compare::AtMost, 0)]);
    assert!(reconcile(&menor, &mut rt));
    assert!(reconcile(&cfg, &mut rt));
    assert!(
        !rt.0[1].fired,
        "uma regra removida e reposta volta por nascer, nao ja' disparada"
    );
}

/// ⭐⭐⭐ **REBOBINAR RE-ARMA A ARESTA.**
///
/// ⚠️⚠️ **Sem isto o defeito é MUDO e só aparece na 2.ª corrida:** uma corrida que acaba com as
/// vidas a zero deixa a regra com `held = true`; a corrida seguinte começa com a condição *«já
/// satisfeita»* e **nunca mais anuncia a morte**. É a mesma forma que a fábrica pagou com o
/// `Max Total` gasto e o cérebro com o `started`.
///
/// ⚠️ **O `fired` também tem de voltar**, senão uma regra com `once` fica calada para sempre.
///
/// **Mutação que deve sangrar:** tirar a vigia da porta do `rewind_runtime`.
#[test]
fn rebobinar_re_arma_a_aresta() {
    let mut world = World::new();
    let e = world
        .spawn((
            CounterWatch(vec![CounterWatchRow {
                counter: "vidas".into(),
                compare: Compare::AtMost,
                value: 0,
                signal: "morri".into(),
                once: true,
            }]),
            CounterWatchRuntime(vec![WatchState {
                held: true,
                fired: true,
            }]),
        ))
        .id();
    crate::rewind_runtime::rewind_runtime_state(&mut world);
    let rt = world.get::<CounterWatchRuntime>(e).expect("o slot");
    assert_eq!(
        rt.0.len(),
        1,
        "renascer tem de deixar UM slot, como a config pede"
    );
    assert!(
        !rt.0[0].held,
        "a corrida seguinte comecaria com a condicao «ja' satisfeita» e nunca mais anunciaria"
    );
    assert!(
        !rt.0[0].fired,
        "com `once`, uma regra que ja' disparou ficaria calada para sempre"
    );
}
