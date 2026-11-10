//! **O SOM DE UI** (estudo de UI viva, D1) — *o gesto confirma-se pelo ouvido*.
//!
//! # ⛔ DESLIGADO por omissão, e não é cautela — é a lei do item
//!
//! O estudo escreve `D1 · som de UI **opt-in**` com um ⛔ ao lado. Um app de desenho vive em cima de
//! música, de referências em vídeo e de chamadas — e um som que ninguém pediu é um som que se
//! desliga no primeiro minuto e leva a feature com ele. Ele mora no
//! `~/.ph2d/prefs.txt`, ao lado do `motion_character` e do `reduced_motion`, e nasce em `0`.

// ⭐ **O enum e a `voice()` mudaram-se para a `ph2d-audio-desktop`** (hoje `ph2d-app-audio`) — o motor é que os lê, e um
// tipo que o motor lê não pode ficar do lado de cá de uma fronteira que ele não atravessa. O
// re-export mantém os chamadores (`crate::ui_sound::UiSound`) byte a byte como estavam.
pub(crate) use ph2d_app_audio::ui_sound::UiSound;

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
            // ⛔ **O ponteiro para o QUINTO elo, uma vez por processo.** Ele não mede nada — não
            // pode (vide o cabeçalho: o mute por-aplicação é do servidor de som, e o `cpal` não o
            // vê). O que ele faz é impedir que a próxima pessoa gaste a sessão a interrogar os
            // quatro elos verdes, que foi exactamente o que aconteceu em 23/08.
            static PONTEIRO: std::sync::Once = std::sync::Once::new();
            PONTEIRO.call_once(|| {
                eprintln!(
                    "[ui-sound] se TUDO abaixo estiver verde e ainda assim for mudo, o elo que \
                     falta e' o mixer do SO (mute por-aplicacao, gravado em disco):"
                );
                eprintln!("[ui-sound]   pactl list sink-inputs | grep -E 'application.name|Mute:'");
            });
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
