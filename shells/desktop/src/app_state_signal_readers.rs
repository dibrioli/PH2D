//! ⭐⭐ **OS CURSORES DO OUTBOX DE SINAIS, num sítio só** (TOP-20 #11, 2026-09-14).
//!
//! # Porque eles se agrupam
//!
//! Cada consumidor do `ph2d_runtime::SignalOutbox` tem **cursor próprio** — é a lei da crate: o
//! produtor não chama ninguém, e partilhar um cursor faria um consumidor comer o sinal do outro.
//! Com cinco consumidores, isso eram **cinco campos soltos** na [`crate::App`], que é uma struct com
//! catraca de campos (`the_app_only_sheds_fields`).
//!
//! ⚠️ **A catraca não é burocracia: ela existe porque a `App` cresce por acréscimo e nunca por
//! desenho.** Agrupar os cinco tira **quatro** campos — é o mesmo gesto que os latches de smoke da
//! wave das Tags, e pela mesma razão: *quando cinco campos respondem à mesma pergunta, eles são uma
//! coisa com nome.*
//!
//! ⛔ **Não é um `Vec`**: cada cursor tem um consumidor com nome e uma ordem no quadro, e um índice
//! numérico tornaria *«quem lê isto?»* uma pergunta sem resposta no sítio onde ela se faz.

/// Os cursores dos consumidores do outbox — um por consumidor, pela ordem em que o quadro os lê.
pub(crate) struct SignalReaders {
    /// O consumidor de TOAST — a prova visível de que o canal fecha a volta.
    pub(crate) toast: ph2d_runtime::SignalReader,
    /// O consumidor de DIAGNÓSTICO (`PH2D_SIGNAL_LOG=1`), `None` sem a env var.
    ///
    /// ⚠️ Um leitor que ninguém pediu seria um cursor a envelhecer sozinho, acumulando `missed`
    /// que ninguém lê — o idioma de diagnóstico da casa (`PH2D_PAINT_PERF`, `PH2D_FLUID_PROFILE`).
    pub(crate) log: Option<ph2d_runtime::SignalReader>,
    /// ⭐ O cursor da tabela **nome → acção** (TOP-20 #5).
    pub(crate) action: ph2d_runtime::SignalReader,
    /// ⭐ O cursor dos **CÉREBROS** (TOP-20 #15).
    ///
    /// ⚠️ **Próprio, e não partilhado com a tabela de acções:** cada consumidor lê a saída com o
    /// seu cursor, e é isso que permite a esta fase correr ANTES daquela sem lhe roubar os sinais.
    pub(crate) machine: ph2d_runtime::SignalReader,
    /// ⭐ O cursor da **FÁBRICA** (TOP-20 #11) — ela escuta o mesmo sinal que a tabela de acções, e
    /// por isso precisa do seu: com um cursor partilhado, quem lesse primeiro apagava o outro.
    pub(crate) factory: ph2d_runtime::SignalReader,
    /// O cursor da máquina de estados de UI.
    pub(crate) ui: ph2d_runtime::SignalReader,
}

impl SignalReaders {
    /// Os seis cursores no arranque. ⚠️ O de diagnóstico só nasce com a env var.
    pub(crate) fn new() -> Self {
        Self {
            toast: ph2d_runtime::SignalReader::new(),
            log: std::env::var_os("PH2D_SIGNAL_LOG").map(|_| ph2d_runtime::SignalReader::new()),
            action: ph2d_runtime::SignalReader::new(),
            machine: ph2d_runtime::SignalReader::new(),
            factory: ph2d_runtime::SignalReader::new(),
            ui: ph2d_runtime::SignalReader::new(),
        }
    }

    /// **O diagnóstico está ligado?** — a pergunta que sete sítios do quadro fazem.
    pub(crate) fn logging(&self) -> bool {
        self.log.is_some()
    }
}
