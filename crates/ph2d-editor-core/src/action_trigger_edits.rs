//! **O vocabulário da secção GATILHO** — o que o painel e a shell dizem um ao outro.
//!
//! ⚠️ **Irmão do [`crate::counter_watch_edits`]**, e pela mesma razão: o painel não conhece a
//! `ph2d-ecs` (ADR-0029) **nem a `ph2d-input`**, logo os tipos que atravessam a fronteira vivem
//! aqui, na fundação que as duas pontas já carregam.

/// Uma edição de um campo da secção.
///
/// ⚠️ **O `u8` é o ÍNDICE na lista, nunca o nome da acção.** O nome é editável, pode estar vazio e
/// pode repetir-se (duas linhas podem ouvir a mesma tecla de propósito — uma para o som, outra para
/// a bala); o índice é o que liga a linha ao que o `SignalOrigin::Action` carrega.
#[derive(Clone, Debug, PartialEq)]
pub enum ActionTriggerFieldEdit {
    /// Cria uma linha no fim da lista.
    Add,
    /// Retira a linha deste índice.
    Remove(u8),
    /// `(linha, nome da acção)`. **Vazio é permitido** e a linha fica órfã — o painel di-lo.
    Action(u8, String),
    /// `(linha, aresta)` — `0` = `Press`, `1` = `Release`, `2` = `Hold`.
    ///
    /// ⚠️ **Viaja como `u8` e não como o enum do motor**, pela mesma cerca de dependência do
    /// `Compare` da vigia. A tradução é da shell, e há gate de ida-e-volta.
    Edge(u8, u8),
    /// `(linha, nome do sinal)` — vazio **cala** a linha sem a apagar.
    Signal(u8, String),
}

/// Uma linha da lista, como o painel a mostra.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorTriggerRow {
    /// O nome da acção do Input Map que esta linha ouve.
    pub action: String,
    /// `0` = `Press`, `1` = `Release`, `2` = `Hold`.
    pub edge: u8,
    /// O sinal. Vazio = calada.
    pub signal: String,
    /// ⭐⭐⭐ **O que o MAPA sabe desta acção** — ver [`NoMapa`].
    ///
    /// ⚠️ **É o snapshot que responde, e não o painel** — a resposta está no `InputMap`, que vive
    /// no `HeroScreen`. Sem esta coluna, uma linha com `fier` em vez de `fire` é indistinguível de
    /// uma que está a funcionar: as duas mostram o mesmo texto e nenhuma dispara.
    ///
    /// ⛔ **E o defeito que ela impede é pior do lado do `Release`:** uma acção inexistente lê-se
    /// como *«não premida»*, e sem a cerca do motor um `Release` sobre ela dispararia em TODO
    /// quadro. A lei cala-o; esta coluna **explica** o silêncio.
    pub no_mapa: NoMapa,
}

/// ⭐⭐⭐ **O que o Input Map sabe de uma acção — TRÊS estados, e não dois** (2026-09-18).
///
/// # ⛔⛔ Porque um booleano não chegava
///
/// A 1.ª redacção perguntava *«existe uma acção com este nome?»* e respondia `bool`. Medido: uma
/// acção **declarada e sem tecla nenhuma** resolve para [`ph2d_input::Sample::default`] — o
/// `ActionState::tick` percorre o MAPA e não os dispositivos, e o doc dele escreve a lei por
/// extenso (*«declarada e por atribuir não é inexistente»*) ⇒ **as três leituras dão `false`**, e o
/// gatilho fica exactamente tão calado como com um nome errado.
///
/// ⇒ *duas causas diferentes, o mesmo silêncio, e o painel dizia que estava tudo bem numa delas.*
/// É a família que o `CLAUDE.md` nomeia: **um gesto que não faz nada e não diz porquê é
/// indistinguível de um partido**, e o artista conclui que a ferramenta não funciona.
///
/// ⚠️ **E as CURAS são diferentes**, que é o que obriga a distinguir: uma pede *criar a acção*, a
/// outra pede *ligar-lhe uma tecla*. Um aviso só mandaria metade dos artistas ao sítio errado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoMapa {
    /// O mapa **não conhece** este nome. Cura: criar a acção.
    Desconhecida,
    /// Ele conhece-a e ela **não tem ligação nenhuma**. Cura: ligar-lhe uma tecla.
    ///
    /// ⚠️ **Não é um erro** — é uma configuração a meio, como o nome de sinal vazio. O aviso
    /// di-lo com outras palavras e outra cor.
    SemTecla,
    /// Tem pelo menos uma ligação. ⭐ O único estado em que a tecla pode chegar à lei.
    Ligada,
}

impl NoMapa {
    /// **Esta acção pode chegar à lei?** `false` nos DOIS estados mudos.
    ///
    /// ⚠️ Ela existe para os consumidores não terem de saber que são três: quem só quer *«isto vai
    /// funcionar?»* pergunta aqui, e quem quer explicar ao artista lê a variante.
    #[must_use]
    pub const fn fala(self) -> bool {
        matches!(self, Self::Ligada)
    }
}

/// O que a secção precisa de saber para se pintar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorActionTriggerInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// As linhas, pela ordem do componente.
    pub rows: Vec<InspectorTriggerRow>,
    /// ⭐ **O relógio da cena está a tocar?**
    ///
    /// ⚠️ Sem ele **nenhum** gatilho fala, e isso não é uma cerca de segurança: as teclas do jogo
    /// são as teclas do editor. *Play → a arma dispara · Stop → o teclado volta a ser do editor.*
    pub clock_playing: bool,
    /// Quantos objectos estão escolhidos — a secção edita o primário, e di-lo.
    pub selected_count: usize,
}

impl InspectorActionTriggerInfo {
    /// Quantas linhas ouvem uma acção que o Input Map não conhece.
    #[must_use]
    pub fn orfas(&self) -> usize {
        self.rows
            .iter()
            .filter(|r| r.no_mapa == NoMapa::Desconhecida)
            .count()
    }

    /// ⭐ **Quantas linhas nomeiam uma acção que EXISTE e não tem tecla** — o segundo silêncio.
    ///
    /// ⚠️ **Ela NÃO entra na contagem do título**, e é uma decisão: o título diz *«partidas»*, e
    /// uma acção por ligar é uma configuração a meio — a mesma fronteira que o sinal vazio já tem.
    #[must_use]
    pub fn sem_tecla(&self) -> usize {
        self.rows
            .iter()
            .filter(|r| r.no_mapa == NoMapa::SemTecla)
            .count()
    }

    /// Quantas linhas estão **caladas** — sem nome de sinal.
    ///
    /// ⚠️ **É diferente de órfã**, e as duas curas são opostas: uma órfã ouve a tecla errada, uma
    /// calada não tem o que dizer. Contá-las juntas mandaria o artista arranjar a metade errada.
    #[must_use]
    pub fn caladas(&self) -> usize {
        self.rows
            .iter()
            .filter(|r| r.signal.trim().is_empty())
            .count()
    }
}
