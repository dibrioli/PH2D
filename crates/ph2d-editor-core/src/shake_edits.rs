//! **O vocabulário das duas secções do ABANÃO** — o que o painel e a shell dizem um ao outro
//! (suplente #25).
//!
//! ⚠️ **Irmão do [`crate::action_trigger_edits`]**, e pela mesma razão: o painel não conhece a
//! `ph2d-ecs` (ADR-0029) nem a `ph2d-shake`, logo os tipos que atravessam a fronteira vivem aqui,
//! na fundação que as duas pontas já carregam.
//!
//! # ⚠️ Porque são DUAS secções e não uma
//!
//! Elas moram em objectos **diferentes** — a `CameraShake` na câmera e o `ShakeEmitter` em quem
//! explode —, logo nunca aparecem juntas no mesmo Inspector. *Uma secção só faz sentido onde o
//! componente dela está.*

/// Uma edição de um campo da secção **CAMERA SHAKE** (na câmera).
#[derive(Clone, Debug, PartialEq)]
pub enum ShakeFieldEdit {
    /// Metros no pico.
    Amplitude(f32),
    /// Hz.
    Frequencia(f32),
    /// Trauma por segundo.
    Decaimento(f32),
    /// A potência do trauma. ⚠️ **Viaja como `u8` e não como uma tag de chip**: aqui o valor É o
    /// expoente, e a faixa é a da lei (a shell coage).
    Expoente(u8),
    /// A semente do ruído. ⚠️ `u64` porque é o que o *splitmix* come — o painel edita-a como um
    /// número comum e a shell não converte nada.
    Semente(u64),
}

/// Uma edição de um campo da secção **SHAKE EMITTER** (em quem explode).
///
/// ⚠️ **O `u8` é o ÍNDICE na lista, nunca o nome do sinal** — a mesma lei do gatilho: o nome é
/// editável, pode estar vazio e pode repetir-se de propósito.
#[derive(Clone, Debug, PartialEq)]
pub enum EmitterFieldEdit {
    /// Cria uma fonte no fim da lista.
    Add,
    /// Retira a fonte deste índice.
    Remove(u8),
    /// `(fonte, nome do sinal)`. **Vazio é permitido** e a fonte fica calada — o painel di-lo.
    On(u8, String),
    /// `(fonte, cerca)` — `0` = `Anyone`, `1` = `Myself`.
    ///
    /// ⚠️ **Viaja como `u8` e não como o enum do motor**, pela mesma cerca de dependência do
    /// `Edge` do gatilho. A tradução é da shell, e há gate de ida-e-volta.
    De(u8, u8),
    /// `(fonte, força)` — quanto trauma ela levanta à queima-roupa.
    Forca(u8, f32),
    /// `(fonte, raio interno)` em metros.
    Dentro(u8, f32),
    /// `(fonte, raio externo)` em metros.
    Fora(u8, f32),
}

/// O que a secção **CAMERA SHAKE** precisa de saber para se pintar.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorShakeInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// Os cinco números, como estão no componente.
    pub amplitude: f32,
    /// Ver [`Self::amplitude`].
    pub frequencia: f32,
    /// Ver [`Self::amplitude`].
    pub decaimento: f32,
    /// Ver [`Self::amplitude`].
    pub expoente: u8,
    /// Ver [`Self::amplitude`].
    pub semente: u64,
    /// ⭐⭐ **O trauma que a câmera tem AGORA** (`0..1`) — a única coluna que vem do estado VIVO.
    ///
    /// ⚠️ **Ela é um FACTO e não um evento**, e é isso que faz o painel dizer *«a tremer»* enquanto
    /// treme em vez de piscar uma vez: é a lição que o `projectiles_finished` do #14 pagou.
    pub trauma: f32,
    /// ⭐ **Esta é a câmera que MANDA?** ⚠️ Sem esta coluna, um `CameraShake` numa câmera desligada
    /// ou de prioridade menor lê-se exactamente como um a funcionar — a lei que a secção CAMERA já
    /// escreve para o `follow`.
    pub activa: bool,
    /// O relógio da cena está a tocar? ⚠️ Com ele parado o abanão **congela** (o `dt` é o do passo
    /// fixo), e um artista que não saiba isso lê o congelamento como um defeito.
    pub clock_playing: bool,
    /// Quantos objectos estão escolhidos — a secção edita o primário, e di-lo.
    pub selected_count: usize,
}

/// Uma fonte da lista, como o painel a mostra.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorEmitterRow {
    /// O sinal que ela ouve. Vazio = calada.
    pub on: String,
    /// `0` = `Anyone`, `1` = `Myself`.
    pub de: u8,
    /// Quanto trauma ela levanta à queima-roupa.
    pub forca: f32,
    /// O raio interno, em metros.
    pub dentro: f32,
    /// O raio externo, em metros.
    pub fora: f32,
}

impl InspectorEmitterRow {
    /// ⚠️ **Os raios estão ao contrário?** `fora <= dentro` é um **corte duro** em `dentro` — a lei
    /// responde-o sozinha e não estoura, mas o artista que escreveu `fora = 2` num `dentro = 5`
    /// **não** queria um corte duro: ele trocou os campos. *Uma degenerescência legal que quase
    /// nunca é intencional merece uma frase, nunca um erro.*
    #[must_use]
    pub fn raios_trocados(&self) -> bool {
        self.fora <= self.dentro
    }
}

/// O que a secção **SHAKE EMITTER** precisa de saber para se pintar.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorEmitterInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// As fontes, pela ordem do componente.
    pub rows: Vec<InspectorEmitterRow>,
    /// ⭐⭐⭐ **A cena tem uma câmera que treme?**
    ///
    /// ⚠️ **Sem ela NADA do que este componente faz é visível**, e o silêncio é total: o emissor
    /// ouve, a distância mede-se, e não há quem abane. *É a metade que o artista não consegue
    /// adivinhar olhando para este objecto*, porque a causa está noutro — a mesma forma da recusa
    /// dos pincéis de escultura.
    pub ha_camera_que_treme: bool,
    /// O relógio da cena está a tocar?
    pub clock_playing: bool,
    /// Quantos objectos estão escolhidos — a secção edita o primário, e di-lo.
    pub selected_count: usize,
}

impl InspectorEmitterInfo {
    /// Quantas fontes estão **caladas** — sem nome de sinal.
    #[must_use]
    pub fn caladas(&self) -> usize {
        self.rows.iter().filter(|r| r.on.trim().is_empty()).count()
    }

    /// Quantas têm os raios trocados — ver [`InspectorEmitterRow::raios_trocados`].
    #[must_use]
    pub fn trocadas(&self) -> usize {
        self.rows.iter().filter(|r| r.raios_trocados()).count()
    }
}

#[cfg(test)]
#[path = "shake_edits_tests.rs"]
mod tests;
