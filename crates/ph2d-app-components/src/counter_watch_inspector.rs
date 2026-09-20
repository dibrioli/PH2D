//! **O instantâneo e o dreno da secção COUNTER WATCH** — as duas pontas entre o painel e o mundo.
//!
//! ⚠️ **O instantâneo responde o que o painel não pode responder:** *«existe um contador com este
//! nome?»* é uma varredura do mundo, e o painel não o vê. Sem essa coluna, uma regra com um `d` a
//! mais no nome é indistinguível de uma que funciona — as duas mostram o mesmo texto e nenhuma
//! dispara.

use ph2d_ecs::{
    Compare, CounterScope, CounterWatch, CounterWatchRow, CounterWatchRuntime, SimWorld,
    WATCHES_MAX, counter,
};
use ph2d_editor_core::counter_watch_edits::{
    CounterWatchFieldEdit as E, InspectorCounterWatchInfo, InspectorWatchRow,
};

/// O `u8` que atravessa a fronteira ⇄ o enum do motor.
///
/// ⚠️ **A tradução vive AQUI e em mais lado nenhum** — escrita duas vezes, o dia em que uma quarta
/// comparação nascer deixa o painel a escolher uma e o motor a fazer outra. Há gate de ida-e-volta.
#[must_use]
pub const fn compare_de_u8(v: u8) -> Compare {
    match v {
        1 => Compare::AtLeast,
        2 => Compare::Exactly,
        _ => Compare::AtMost,
    }
}

/// O caminho de volta.
#[must_use]
pub const fn u8_de_compare(c: Compare) -> u8 {
    match c {
        Compare::AtMost => 0,
        Compare::AtLeast => 1,
        Compare::Exactly => 2,
    }
}

/// ⭐ **Constrói o instantâneo** da entidade escolhida, ou `None` se ela não tem o componente.
#[must_use]
pub fn build_info(
    sim: &SimWorld,
    bits: u64,
    clock_playing: bool,
    selected_count: usize,
) -> Option<InspectorCounterWatchInfo> {
    let e = ph2d_ecs::Entity::from_bits(bits);
    let world = sim.world();
    let cfg = world.get::<CounterWatch>(e)?;
    let rows = cfg
        .0
        .iter()
        .map(|r| {
            // ⚠️ **Pela PORTA** — a mesma soma que o placar mostra e que a vigia lê.
            // ⚠️ **O MESMO âmbito que a lei lê** — senão o painel mostra a soma da cena e a
            // regra julga a vida de um inimigo, e o artista vê dois números para um facto.
            let valor_vivo = counter::soma(world, &r.counter, r.scope.ambito(e));
            InspectorWatchRow {
                counter: r.counter.clone(),
                compare: u8_de_compare(r.compare),
                value: r.value,
                signal: r.signal.clone(),
                once: r.once,
                scope_own: r.scope == CounterScope::Own,
                counter_existe: valor_vivo.is_some(),
                valor_vivo,
            }
        })
        .collect();
    Some(InspectorCounterWatchInfo {
        entity_bits: bits,
        rows,
        clock_playing,
        selected_count,
    })
}

/// Aplica uma edição. `true` = o mundo mudou.
///
/// ⚠️ **Escrever o MESMO valor não é uma mudança** — devolver `true` aqui faria cada quadro com o
/// campo focado marcar o componente como sujo, e o undo regista por diff.
fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = ph2d_ecs::Entity::from_bits(bits);
    let Some(mut cfg) = sim.world_mut().get_mut::<CounterWatch>(e) else {
        return false;
    };
    let i = match edit {
        E::Add => {
            // ⚠️ **O tecto é o do MODELO**, e o painel esconde o `+` quando lá chega — mas a
            // guarda mora aqui na mesma: uma edição pode vir de um barramento drenado tarde.
            if cfg.0.len() >= WATCHES_MAX {
                return false;
            }
            cfg.0.push(CounterWatchRow::default());
            return true;
        }
        E::Remove(i) => {
            let i = usize::from(*i);
            if i >= cfg.0.len() {
                return false;
            }
            cfg.0.remove(i);
            return true;
        }
        E::Counter(i, _)
        | E::Compare(i, _)
        | E::Value(i, _)
        | E::Signal(i, _)
        | E::Once(i, _)
        | E::Scope(i, _) => usize::from(*i),
    };
    let Some(row) = cfg.0.get_mut(i) else {
        return false;
    };
    match edit {
        E::Counter(_, nome) => {
            if row.counter == *nome {
                return false;
            }
            row.counter.clone_from(nome);
        }
        E::Compare(_, v) => {
            let c = compare_de_u8(*v);
            if row.compare == c {
                return false;
            }
            row.compare = c;
        }
        E::Value(_, v) => {
            if row.value == *v {
                return false;
            }
            row.value = *v;
        }
        E::Signal(_, s) => {
            if row.signal == *s {
                return false;
            }
            row.signal.clone_from(s);
        }
        E::Once(_, b) => {
            if row.once == *b {
                return false;
            }
            row.once = *b;
        }
        // ⚠️ **A ponte `bool` ⇄ enum vive AQUI, na shell** — o painel não vê a `ph2d-ecs`, e é a
        // mesma cerca de dependência que o `Compare` já paga.
        E::Scope(_, b) => {
            let novo = if *b {
                CounterScope::Own
            } else {
                CounterScope::World
            };
            if row.scope == novo {
                return false;
            }
            row.scope = novo;
        }
        E::Add | E::Remove(_) => return false,
    }
    true
}

/// Aplica todas as edições do quadro. `true` = alguma coisa mudou.
///
/// ⚠️ **O `CounterWatchRuntime` NÃO se toca aqui** — ele reconcilia-se sozinho no quadro seguinte,
/// pela ponte. Mexer-lhe daqui seria a segunda resposta a *«quantos slots há?»*.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    let mut mudou = false;
    for (bits, edit) in edits {
        mudou |= apply(sim, *bits, edit);
    }
    mudou
}

/// ⚠️ **A porta que o gate da shell lê** para provar que a tabela de ids do painel cobre as
/// variantes do enum do motor.
#[must_use]
pub const fn comparacoes() -> usize {
    3
}

/// O runtime existe para esta entidade? — leitura de diagnóstico.
#[must_use]
pub fn tem_runtime(sim: &SimWorld, bits: u64) -> bool {
    sim.world()
        .get::<CounterWatchRuntime>(ph2d_ecs::Entity::from_bits(bits))
        .is_some()
}

#[cfg(test)]
#[path = "counter_watch_inspector_tests.rs"]
mod tests;
