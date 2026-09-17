//! **O vocabulário da secção COUNTER WATCH** — o que o painel e a shell dizem um ao outro.
//!
//! ⚠️ **Irmão do [`crate::sequence_edits`]**, e pela mesma razão: o painel não conhece a
//! `ph2d-ecs` (ADR-0029), logo o tipo que atravessa a fronteira vive aqui, na fundação que as duas
//! pontas já carregam.

/// Uma edição de um campo da secção.
///
/// ⚠️ **O `u8` é o ÍNDICE na lista, nunca o nome do contador.** O nome é editável e pode repetir-se
/// (de propósito: dois contadores homónimos SOMAM); o índice é o que liga `CounterWatch[i]` ao
/// `CounterWatchRuntime[i]`, que é onde a aresta vive.
#[derive(Clone, Debug, PartialEq)]
pub enum CounterWatchFieldEdit {
    /// Cria uma regra no fim da lista.
    Add,
    /// Retira a regra deste índice. ⚠️ O runtime encolhe com ela pela reconciliação do quadro
    /// seguinte — nada aqui lhe toca.
    Remove(u8),
    /// `(regra, nome do contador)`. **Vazio é permitido** e a regra fica órfã — o painel di-lo.
    Counter(u8, String),
    /// `(regra, comparação)` — `0` = `AtMost`, `1` = `AtLeast`, `2` = `Exactly`.
    ///
    /// ⚠️ **Viaja como `u8` e não como o enum do motor**, pela mesma cerca de dependência: o
    /// painel não vê a `ph2d-ecs`. A tradução é da shell, e há gate de ida-e-volta.
    Compare(u8, u8),
    /// `(regra, limiar)`.
    Value(u8, i64),
    /// `(regra, nome do sinal)` — vazio **cala** a regra sem a apagar.
    Signal(u8, String),
    /// `(regra, só a primeira vez)`.
    Once(u8, bool),
}

/// Uma linha da lista, como o painel a mostra.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorWatchRow {
    /// O nome do contador que esta regra vigia.
    pub counter: String,
    /// `0` = `AtMost`, `1` = `AtLeast`, `2` = `Exactly`.
    pub compare: u8,
    /// O limiar.
    pub value: i64,
    /// O sinal. Vazio = calada.
    pub signal: String,
    /// Só da primeira vez.
    pub once: bool,
    /// ⭐⭐ **Existe um contador com este nome na cena?**
    ///
    /// ⚠️ **É o snapshot que responde, e não o painel** — a resposta é uma varredura do mundo, e o
    /// painel não o vê. Sem esta coluna, uma regra com um `d` a mais no nome é indistinguível de
    /// uma que está a funcionar: as duas mostram o mesmo texto e nenhuma dispara.
    pub counter_existe: bool,
    /// ⭐ **Quanto vale esse contador AGORA** — leitura, nunca edição. `None` quando ele não existe.
    pub valor_vivo: Option<i64>,
}

/// O que a secção precisa de saber para se pintar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorCounterWatchInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// As regras, pela ordem do componente.
    pub rows: Vec<InspectorWatchRow>,
    /// O relógio da cena está a tocar? Sem ele nenhuma vigia avalia.
    pub clock_playing: bool,
    /// Quantos objectos estão escolhidos — a secção edita o primário, e di-lo.
    pub selected_count: usize,
}

impl InspectorCounterWatchInfo {
    /// Quantas regras apontam a um contador que não existe.
    #[must_use]
    pub fn orfas(&self) -> usize {
        self.rows.iter().filter(|r| !r.counter_existe).count()
    }

    /// Quantas regras estão **caladas** — sem nome de sinal.
    ///
    /// ⚠️ **É diferente de órfã**, e as duas curas são opostas: uma órfã tem o contador errado, uma
    /// calada não tem o que dizer. Contá-las juntas mandaria o artista arranjar a metade errada.
    #[must_use]
    pub fn caladas(&self) -> usize {
        self.rows
            .iter()
            .filter(|r| r.signal.trim().is_empty())
            .count()
    }
}
