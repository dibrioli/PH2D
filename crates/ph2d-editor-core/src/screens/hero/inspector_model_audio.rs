//! **O modelo da secção AUDIO** (TOP-20 #4, W3) — snapshot e edits.
//!
//! ⚠️ **Irmão de [`super::inspector_model`] por CAP de LOC** — mesmo padrão dos outros oito.
//!
//! # ⚠️ Três coisas viajam DERIVADAS, e nenhuma delas se podia derivar aqui
//!
//! O `ph2d-editor-core` é chrome e **não depende do `ph2d-ecs`** (ADR-0029). Mas há três perguntas
//! cuja resposta o painel precisa e que só quem tem o mundo — ou o disco — sabe responder:
//!
//! - **o ficheiro ainda existe?** (`file_missing`) — é disco, e é a diferença entre *«não ouço
//!   nada»* e *«alguém mudou o ficheiro de sítio»*;
//! - **alguém manda isto tocar?** (`reachable_by_signal`) — é uma varredura das tabelas de acção da
//!   CENA à procura de um `Play Sound` que aponte para este objecto;
//! - **quantas orelhas a cena tem?** (`listener_count`) — sem nenhuma, todo som toca sem posição, e
//!   o artista tem de o saber antes de afinar um alcance que não faz nada.
//!
//! ⇒ as três chegam **no snapshot**. Re-derivá-las aqui seria impossível para duas e errado para a
//! terceira. *O painel não adivinha o mundo; ele recebe-o.*

/// A **FONTE** de som, como o Inspector a lê.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorAudioSource {
    /// O caminho do ficheiro. **Vazio = calado.**
    pub sound: String,
    pub volume_db: f32,
    pub pitch: f32,
    pub looping: bool,
    pub autoplay: bool,
    pub max_distance: f32,
    pub attenuation: f32,
    pub non_spatialized_radius: f32,
    pub panning_strength: f32,
    pub max_polyphony: u8,
    /// A posição em `AudioBus::ALL`. ⚠️ Tag, e não o enum — ver o doc do módulo.
    pub bus_tag: u8,
    /// ⭐ **O caminho não abre.** Derivado na shell, contra o DISCO.
    ///
    /// ⚠️ **Distinto de `sound.is_empty()`**, e a distinção é a única coisa que separa *«ainda não
    /// escolhi»* de *«alguém mexeu na pasta»* — dois estados que o artista cura de maneiras
    /// diferentes.
    pub file_missing: bool,
    /// ⭐⭐ **Alguma linha de `Signal Actions` da cena manda ISTO tocar.**
    ///
    /// ⚠️ **Derivado do MUNDO**, e é a metade que a lei pura declara não poder responder: ela sabe
    /// que a fonte não arranca sozinha, e não sabe se alguém a manda arrancar.
    pub reachable_by_signal: bool,
}

/// O piso do silêncio, em decibéis — **o mesmo do motor** (`ph2d_ecs::db_to_linear`).
///
/// ⚠️ **Escrito e não importado**, e é a lei do ADR-0029 outra vez: este painel não depende do
/// `ph2d-ecs`. Há gate na shell a prender os dois.
const SILENCE_DB: f32 = -80.0; // LITERAL-PX-OK: decibéis, não pixels

impl InspectorAudioSource {
    /// ⛔ **Esta fonte nunca se vai ouvir** — as três maneiras, espelhadas da lei pura.
    #[must_use]
    pub fn is_mute(&self) -> bool {
        self.sound.trim().is_empty() || self.max_distance <= 0.0 || self.volume_db <= SILENCE_DB
    }

    /// ⛔ **Ninguém a vai fazer tocar** — nem sozinha, nem por sinal.
    ///
    /// ⚠️ **As DUAS metades juntas**, e é isso que a torna uma afirmação e não um palpite: um
    /// `autoplay` desligado **não** é um defeito quando há uma tabela de acções a apontar para cá,
    /// e dizê-lo seria um aviso que grita sobre uma cena correcta.
    #[must_use]
    pub fn never_sounds(&self) -> bool {
        !self.autoplay && !self.reachable_by_signal
    }
}

/// Snapshot da secção AUDIO da entidade selecionada.
///
/// ⚠️ **Ela existe se o objecto tiver a FONTE, as ORELHAS, ou as duas** — nunca por defeito. É o
/// ADR-0166: *o Inspector mostra o que o objecto TEM*.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorAudioInfo {
    pub entity_bits: u64,
    /// A fonte, se este objecto tiver uma.
    pub source: Option<InspectorAudioSource>,
    /// Este objecto é as orelhas da cena?
    pub is_listener: bool,
    /// Quantas orelhas a cena tem. ⚠️ `0` é informação e não erro — ver o doc do módulo.
    pub listener_count: usize,
    /// ⭐ **Este objecto é as orelhas que MANDAM?** Com vários ouvintes ganha o de menor
    /// identidade, e o painel tem de o dizer — senão o artista afina um ouvinte que ninguém usa.
    pub is_active_listener: bool,
    /// Os rótulos dos barramentos, na ordem de `AudioBus::ALL`. ⚠️ Viajam pela mesma razão dos
    /// verbos: o painel não conhece o enum, e uma cópia envelheceria no primeiro barramento novo.
    pub bus_labels: Vec<String>,
    pub selected_count: usize,
}

/// Uma edição de um campo da secção AUDIO.
#[derive(Clone, Debug, PartialEq)]
pub enum AudioFieldEdit {
    /// O caminho, escrito à mão.
    Sound(String),
    /// Abrir o diálogo de ficheiro. ⚠️ **Ela não carrega caminho nenhum**: quem escolhe é o
    /// diálogo, do lado da shell, e mandar um caminho daqui seria o painel a adivinhar a resposta.
    Browse,
    VolumeDb(f32),
    Pitch(f32),
    Looping(bool),
    Autoplay(bool),
    MaxDistance(f32),
    Attenuation(f32),
    Radius(f32),
    Panning(f32),
    Polyphony(u8),
    /// A posição em `AudioBus::ALL`.
    Bus(u8),
    /// **Ouvir agora.** ⚠️ Não escreve nada no documento — é um gesto de editor, como o transporte
    /// da §11.
    Preview,
    /// **Calar** o que este objecto tem a soar.
    StopPreview,
}
