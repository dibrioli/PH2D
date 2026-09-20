//! Os gates da PORTA de *«quanto vale o contador X?»* — e do ÂMBITO dela (2026-09-20).

use bevy_ecs::world::World;

use super::{Ambito, soma};
use crate::{Counter, CounterRuntime};

fn conta(nome: &str, start: i64, valor: i64) -> (Counter, CounterRuntime) {
    (
        Counter {
            name: nome.into(),
            start,
            keep_on_restart: false,
        },
        CounterRuntime { value: valor },
    )
}

/// ⭐⭐⭐ **Os dois âmbitos dão respostas DIFERENTES sobre o mesmo nome** — é a lei inteira.
///
/// Três inimigos com `vida` cada: a cena soma `6`, e cada um vale `2`.
///
/// **Mutação que deve sangrar:** o ramo do `Ambito::Objecto` cair na varredura.
#[test]
fn a_cena_soma_e_o_objecto_responde_por_si() {
    let mut w = World::new();
    let ids: Vec<_> = (0..3).map(|_| w.spawn(conta("vida", 3, 2)).id()).collect();
    assert_eq!(
        soma(&w, "vida", Ambito::Mundo),
        Some(6),
        "a cena SOMA os tres"
    );
    for e in ids {
        assert_eq!(
            soma(&w, "vida", Ambito::Objecto(e)),
            Some(2),
            "cada inimigo responde pela PROPRIA vida"
        );
    }
}

/// ⚠️ **Um objecto cujo contador tem OUTRO nome não responde** — e `None` não é zero.
///
/// Sem esta metade, uma vigia por-objecto sobre um nome escrito com erro leria `0` e anunciaria
/// *«morreste»* no arranque, que é exactamente o defeito que a porta já evita no âmbito da cena.
///
/// **Mutação que deve sangrar:** apagar a comparação do nome no ramo do objecto.
#[test]
fn o_objecto_com_outro_nome_nao_responde() {
    let mut w = World::new();
    let e = w.spawn(conta("moedas", 0, 9)).id();
    assert_eq!(soma(&w, "vida", Ambito::Objecto(e)), None);
    assert_eq!(soma(&w, "moedas", Ambito::Objecto(e)), Some(9), "CONTROLO");
    // E um objecto SEM contador nenhum também não.
    let vazio = w.spawn_empty().id();
    assert_eq!(soma(&w, "vida", Ambito::Objecto(vazio)), None);
}

/// ⭐⭐⭐ **Um contador POR ESTREAR vale o `start` — e as duas leituras DISCORDAM, de propósito.**
///
/// O `CounterRuntime` nasce quando alguém escreve, logo um contador acabado de autorar não o tem.
/// A varredura da cena exige os DOIS componentes e devolve `None` (*«não existe contador com este
/// nome a valer alguma coisa»*); o âmbito de objecto tem o objecto NOMEADO e responde por ele.
///
/// ⚠️⚠️ **Isto CUROU um defeito latente da arma** (2026-09-20): ela lia os dois componentes à mão
/// e caía no `Municao::default()`, que é **munição INFINITA** ⇒ *um pente por estrear dava balas
/// sem fim até alguém lhe escrever uma vez*. Hoje ela lê um pente CHEIO.
///
/// **Mutação que deve sangrar:** o ramo do objecto devolver `None` sem o `CounterRuntime`.
#[test]
fn um_contador_por_estrear_vale_o_start_no_objecto_e_nada_na_cena() {
    let mut w = World::new();
    let e = w
        .spawn(Counter {
            name: "balas".into(),
            start: 6,
            keep_on_restart: false,
        })
        .id();
    assert_eq!(
        soma(&w, "balas", Ambito::Objecto(e)),
        Some(6),
        "o objecto esta' NOMEADO: a resposta e' o start dele"
    );
    assert_eq!(
        soma(&w, "balas", Ambito::Mundo),
        None,
        "a varredura da cena exige os dois componentes — a lei de sempre, intocada"
    );
}

/// ⚠️ **Um nome vazio não casa com nada, nos DOIS âmbitos** — um campo por preencher não é curinga.
#[test]
fn um_nome_vazio_nao_casa_em_ambito_nenhum() {
    let mut w = World::new();
    let e = w.spawn(conta("", 0, 5)).id();
    assert_eq!(soma(&w, "  ", Ambito::Mundo), None);
    assert_eq!(soma(&w, "  ", Ambito::Objecto(e)), None);
}

/// ⚠️ **A ponte do modo gravado para o âmbito, nos DOIS sentidos.**
///
/// ⛔ Sem ela alguém escreveria o `match` no sítio onde precisa dele, e o terceiro modo divergiria.
#[test]
fn o_modo_gravado_resolve_para_o_ambito_certo() {
    use crate::counter_watch::CounterScope;
    let mut w = World::new();
    let e = w.spawn(conta("x", 0, 1)).id();
    assert_eq!(CounterScope::World.ambito(e), Ambito::Mundo);
    assert_eq!(CounterScope::Own.ambito(e), Ambito::Objecto(e));
    assert_eq!(
        CounterScope::default(),
        CounterScope::World,
        "o de FABRICA e' a cena — e' o que todo ficheiro ja' gravado herda"
    );
}
