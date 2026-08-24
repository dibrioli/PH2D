//! **O SOM DE UI** (estudo de UI viva, D1) — *o gesto confirma-se pelo ouvido*.
//!
//! # ⛔ DESLIGADO por omissão, e não é cautela — é a lei do item
//!
//! O estudo escreve `D1 · som de UI **opt-in**` com um ⛔ ao lado. Um app de desenho vive em cima de
//! música, de referências em vídeo e de chamadas — e um som que ninguém pediu é um som que se
//! desliga no primeiro minuto e leva a feature com ele. Ele mora no
//! `~/.ph2d/prefs.txt`, ao lado do `motion_character` e do `reduced_motion`, e nasce em `0`.
//!
//! # ⭐⭐ A LEI: um som CONFIRMA o que a mão fez, nunca ANUNCIA o que o app decidiu
//!
//! É a única linha que separa um som de UI bom de um irritante, e ela tem duas metades:
//!
//! - **só o que a MÃO causou soa** — um clique, um commit, uma recusa. O que o app faz sozinho
//!   (um quadro, uma transição a assentar, um painel a publicar) é **mudo**;
//! - **o HOVER nunca soa.** Passar o rato por cima de uma fileira de botões é o gesto mais barato
//!   e mais frequente do editor; sonorizá-lo transforma navegar num chocalho. É a irmã exacta da
//!   cerca do §6.2 do estudo (*o realce de uma lista obedece ao cursor; oito rows meio-acesas ao
//!   mesmo tempo é rasto, não vida*).
//!
//! Há gate para as duas.
//!
//! # E o vocabulário é CURTO de propósito
//!
//! Quatro sons. Um vocabulário grande obriga o ouvido a aprender uma língua para ganhar o que uma
//! confirmação já dá — e cada som a mais é uma chance a mais de o artista desligar o conjunto.
//!
//! ⚠️ **Sintetizados, sem ficheiro nenhum**: o motor já traz os geradores
//! ([`crate::audio::signals`]), e um asset de UI seria mais um binário a versionar, a licenciar e
//! a manter afinado com o tema.

/// O que aconteceu — e cada um destes é uma coisa que **a mão fez**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiSound {
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
    pub(crate) const fn voice(self) -> (f32, f32, f32) {
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

impl crate::App {
    /// **TOCA `what`**, se o artista tiver ligado o som — e não faz mais nada.
    ///
    /// ⚠️ **A guarda mora AQUI e não em cada chamador**, e é o que mantém a lei verificável: os
    /// sítios que soam são um punhado, e cada um deles diz *o que aconteceu*, nunca *se deve
    /// soar*. Um `if enabled()` espalhado seria a linha que o sítio seguinte esquece.
    ///
    /// ⚠️ **Sem motor de áudio, no-op silencioso.** Um app sem dispositivo de som continua a
    /// funcionar inteiro — é a mesma lei do `AudioSystem::new` devolver `Option`.
    pub(crate) fn ui_sound(&mut self, what: UiSound) {
        // ⚠️ **O dono do facto é o `HeroScreen`**, semeado no arranque a partir do
        // `~/.ph2d/prefs.txt` e derivado de volta para lá — nunca uma leitura de disco por som.
        let on = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.ui_sound);
        // ⭐ **`PH2D_UI_SOUND_DIAG=1` diz QUAL elo está partido**, e existe porque a cadeia tem
        // quatro (a preferência · o dispositivo · o disparo · o mixer) e o sintoma dos quatro é o
        // mesmo: silêncio. *Um sintoma que não separa as causas pede um instrumento, não um
        // palpite.*
        let diag = std::env::var_os("PH2D_UI_SOUND_DIAG").is_some();
        if diag {
            eprintln!(
                "[ui-sound] {what:?} · pref={on} · dispositivo={}",
                self.audio.is_some()
            );
        }
        if !on {
            return;
        }
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        audio.play_ui(what, diag);
    }
}

#[cfg(test)]
#[path = "ui_sound_tests.rs"]
mod tests;
