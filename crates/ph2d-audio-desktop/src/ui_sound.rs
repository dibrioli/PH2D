//! ⭐ **A TAXONOMIA do som de UI** — o que aconteceu, e a voz de cada um.
//!
//! ⚠️ **Ela vive com o MOTOR e não com o gatilho** (`line/shell-folhas`, 12/09): o
//! [`crate::AudioSystem::play_ui`] lê-a, e um tipo que o motor lê não pode ficar do outro lado de
//! uma fronteira que o motor não atravessa. O gatilho (`App::ui_sound`, a preferência do artista e
//! o diagnóstico dos cinco elos) fica em `shells/desktop/src/ui_sound.rs`, onde tem a `App`.

/// O que aconteceu — e cada um destes é uma coisa que **a mão fez**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiSound {
    /// Um botão/chip foi premido e solto sobre si próprio.
    Click,
    /// Um interruptor mudou de estado (o par do `Click`, meio tom acima ou abaixo).
    Toggle,
    /// Um gesto **consolidou** alguma coisa (Apply, commit, um Rec).
    Commit,
    /// O app **recusou** o gesto (a trava do Painter, um menu vazio, um contrato).
    ///
    /// ⚠️ Ele é o único que soa a *"não"*, e é o mais importante do quatro: uma recusa silenciosa
    /// lê-se como um clique que não funcionou.
    Refuse,
}

impl UiSound {
    /// `(frequência em Hz, duração em segundos, ganho)` — a voz de cada um.
    ///
    /// ⚠️ **Curtos e graves.** Um som de UI que dure mais que o gesto chega DEPOIS dele, e um agudo
    /// corta a música que o artista tem a tocar. Os quatro cabem em 90 ms.
    pub const fn voice(self) -> (f32, f32, f32) {
        match self {
            UiSound::Click => (660.0, 0.035, 0.18),
            // Um tom abaixo do clique: o ouvido lê a diferença sem ter de a aprender.
            UiSound::Toggle => (550.0, 0.045, 0.18),
            // Uma quinta acima do clique, e um bocadinho mais longo: "isto ficou feito".
            UiSound::Commit => (880.0, 0.09, 0.16),
            // Grave e curto — a única voz que desce.
            UiSound::Refuse => (180.0, 0.08, 0.22),
        }
    }
}

