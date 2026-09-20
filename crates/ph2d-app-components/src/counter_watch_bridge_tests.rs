//! Os gates da PONTE — a lei pura tem os dela na `ph2d-ecs`. Aqui pergunta-se o que só o mundo
//! responde: a soma pela porta, o relógio, e o que o painel vai mostrar.

use ph2d_ecs::{
    Compare, Counter, CounterRuntime, CounterWatch, CounterWatchRow, CounterWatchRuntime, SimWorld,
};

use super::frame;

fn regra(counter: &str, compare: Compare, value: i64, signal: &str) -> CounterWatchRow {
    CounterWatchRow {
        counter: counter.into(),
        compare,
        value,
        signal: signal.into(),
        once: false,
        scope: Default::default(),
    }
}

/// Um contador com um valor vivo.
fn contador(sim: &mut SimWorld, nome: &str, valor: i64) -> ph2d_ecs::Entity {
    sim.world_mut()
        .spawn((
            Counter {
                name: nome.into(),
                start: valor,
                keep_on_restart: false,
            },
            CounterRuntime { value: valor },
        ))
        .id()
}

fn soma_de(sim: &mut SimWorld, e: ph2d_ecs::Entity, delta: i64) {
    let mut rt = sim
        .world_mut()
        .get_mut::<CounterRuntime>(e)
        .expect("o contador vivo");
    rt.value += delta;
}

/// ⭐⭐⭐ **A travessia fala, pela porta do produto** — e o planalto cala-se.
///
/// **Mutação que deve sangrar:** correr o `advance` sem o `held`, ou chamar a lei com `Some(0)`
/// onde a porta devolve `None`.
#[test]
fn a_vigia_fala_na_travessia_e_cala_se_no_planalto() {
    let mut sim = SimWorld::default();
    let c = contador(&mut sim, "vidas", 3);
    sim.world_mut().spawn(CounterWatch(vec![regra(
        "vidas",
        Compare::AtMost,
        0,
        "morri",
    )]));

    assert!(frame(&mut sim, true, 1).disparos.is_empty(), "tres vidas");
    soma_de(&mut sim, c, -3);
    let f = frame(&mut sim, true, 1);
    assert_eq!(f.disparos.len(), 1, "a travessia tem de falar");
    assert_eq!(f.disparos[0].2, "morri");
    assert_eq!(f.disparos[0].1, 0, "a regra 0 da lista");
    for _ in 0..10 {
        assert!(
            frame(&mut sim, true, 1).disparos.is_empty(),
            "o planalto voltou a falar"
        );
    }
}

/// ⭐⭐⭐ **A PORTA: a vigia lê a MESMA soma que o placar** — dois contadores com o mesmo nome
/// somam, e é a soma que atravessa o limiar.
///
/// ⚠️ **É esta a fixtura que distingue a porta de uma segunda conta:** com **um** contador só, ler
/// «o primeiro» e ler «a soma» dão a mesma resposta, e a mutação que troca a porta por um
/// `.find()` sobrevive.
///
/// **Mutação que deve sangrar:** trocar `counter::soma` por «o primeiro que casar».
#[test]
fn a_vigia_le_a_soma_e_nao_o_primeiro() {
    let mut sim = SimWorld::default();
    let a = contador(&mut sim, "moedas", 4);
    let _b = contador(&mut sim, "moedas", 4);
    sim.world_mut().spawn(CounterWatch(vec![regra(
        "moedas",
        Compare::AtLeast,
        10,
        "abre",
    )]));

    // 4 + 4 = 8 — «o primeiro» leria 4 e a soma le^ 8; nenhum dos dois chega a 10.
    assert!(frame(&mut sim, true, 1).disparos.is_empty(), "8 nao e' 10");
    soma_de(&mut sim, a, 2);
    // 6 + 4 = 10 — a SOMA chega; «o primeiro» leria 6 e ficaria calado.
    let f = frame(&mut sim, true, 1);
    assert_eq!(f.disparos.len(), 1, "a soma dos dois e' que atravessa");
    assert_eq!(f.disparos[0].2, "abre");
}

/// ⭐⭐ **Com o relógio PARADO ela não fala — e mesmo assim ganha slots e conta as órfãs.**
///
/// ⚠️ As duas metades: sem a primeira, um editor parado mataria o herói; sem a segunda, uma regra
/// acabada de escrever não teria onde guardar a aresta e o aviso do painel não existiria enquanto
/// o artista autora, que é exactamente quando ele precisa dele.
#[test]
fn parada_ela_nao_fala_mas_arruma_a_casa() {
    let mut sim = SimWorld::default();
    contador(&mut sim, "vidas", 0);
    let w = sim
        .world_mut()
        .spawn(CounterWatch(vec![
            regra("vidas", Compare::AtMost, 0, "morri"),
            regra("vidaas", Compare::AtMost, 0, "erro de dedo"),
        ]))
        .id();

    let f = frame(&mut sim, false, 0);
    assert!(f.disparos.is_empty(), "o relogio esta' parado");
    assert_eq!(f.orfas, 1, "so' a segunda aponta a um contador inexistente");
    assert_eq!(
        sim.world()
            .get::<CounterWatchRuntime>(w)
            .expect("slots")
            .0
            .len(),
        2,
        "os slots nascem com o relogio parado"
    );

    // E o CONTROLO: a andar, só a primeira fala.
    let f = frame(&mut sim, true, 1);
    assert_eq!(f.disparos.len(), 1);
    assert_eq!(f.disparos[0].2, "morri");
}

/// ⭐⭐⭐ **Uma vigia sobre um contador que NÃO EXISTE nunca fala** — nem quando o limiar seria
/// satisfeito por zero.
///
/// **Mutação que deve sangrar:** `soma(...).unwrap_or(0)`.
#[test]
fn um_contador_inexistente_e_orfao_e_mudo() {
    let mut sim = SimWorld::default();
    sim.world_mut().spawn(CounterWatch(vec![regra(
        "vidaas",
        Compare::AtMost,
        0,
        "morri",
    )]));
    for _ in 0..5 {
        let f = frame(&mut sim, true, 1);
        assert!(f.disparos.is_empty(), "ausente nao e' zero");
        assert_eq!(f.orfas, 1);
    }
    // O CONTROLO: criado o contador, a MESMA regra fala.
    contador(&mut sim, "vidaas", 0);
    let f = frame(&mut sim, true, 1);
    assert_eq!(f.orfas, 0);
    assert_eq!(f.disparos.len(), 1, "com o contador a existir, ela fala");
}

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ UMA VIDA POR INIMIGO — o ÂMBITO da vigia (2026-09-20)
// ─────────────────────────────────────────────────────────────────────────────

/// Três inimigos iguais: cada um com a vida DELE, a vigia DELE e o sinal DELE.
///
/// ⚠️ **O `scope` é parâmetro do arranjo** — é ele que os dois gates variam, e tudo o resto é
/// idêntico: *dois arranjos que só diferem no que se está a medir*.
fn arena(scope: ph2d_ecs::CounterScope, vidas: &[i64]) -> (SimWorld, Vec<ph2d_ecs::Entity>) {
    let mut sim = SimWorld::default();
    let ids = vidas
        .iter()
        .map(|v| {
            let mut r = regra("vida", Compare::AtMost, 0, "morri");
            r.scope = scope;
            sim.world_mut()
                .spawn((
                    Counter {
                        name: "vida".into(),
                        start: 3,
                        keep_on_restart: false,
                    },
                    CounterRuntime { value: *v },
                    CounterWatch(vec![r]),
                    CounterWatchRuntime(vec![ph2d_ecs::counter_watch::born()]),
                ))
                .id()
        })
        .collect();
    (sim, ids)
}

/// ⭐⭐⭐ **Com `Own`, só o inimigo que chegou a zero morre.**
///
/// Vidas `[0, 2, 2]`: UM disparo, e é o do primeiro.
///
/// **Mutação que deve sangrar:** a ponte passar `Ambito::Mundo` (a soma é `4`, ninguém dispara).
#[test]
fn com_o_ambito_do_objecto_so_o_que_chegou_a_zero_morre() {
    let (mut sim, ids) = arena(ph2d_ecs::CounterScope::Own, &[0, 2, 2]);
    let out = frame(&mut sim, true, 1);
    assert_eq!(out.disparos.len(), 1, "UM so' morre: {:?}", out.disparos);
    assert_eq!(
        out.disparos[0].0,
        ids[0].to_bits(),
        "e e' o que tem a vida a zero"
    );
}

/// ⚠️ **CONTROLO: com o âmbito da CENA (o de fábrica) o mesmo arranjo é MUDO.**
///
/// A soma das três vidas é `4`, que não passa o `AtMost 0` — e é exactamente este o defeito que a
/// wave curou: *dez inimigos com a mesma regra liam a soma dos dez*.
#[test]
fn com_o_ambito_da_cena_o_mesmo_arranjo_e_mudo() {
    let (mut sim, _) = arena(ph2d_ecs::CounterScope::World, &[0, 2, 2]);
    let out = frame(&mut sim, true, 1);
    assert!(
        out.disparos.is_empty(),
        "a soma e' 4: ninguem devia falar — {:?}",
        out.disparos
    );
}

/// ⛔⛔ **E a outra metade do defeito: com a CENA eles morrem TODOS JUNTOS.**
///
/// Vidas `[0, 0, 0]` — a soma é `0`, logo as três regras disparam **no mesmo tique**, mesmo que
/// só uma delas descrevesse um inimigo morto. *Um gate que só medisse o caso mudo não distinguiria
/// «a lei está certa» de «a lei nunca fala».*
#[test]
fn com_o_ambito_da_cena_morrem_todos_juntos() {
    let (mut sim, _) = arena(ph2d_ecs::CounterScope::World, &[0, 0, 0]);
    let out = frame(&mut sim, true, 1);
    assert_eq!(out.disparos.len(), 3, "a soma e' zero: falam os tres");
}
