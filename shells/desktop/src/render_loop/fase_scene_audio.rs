//! **Fase do quadro: O SOM DE CENA** (TOP-20 #4) — nasce, segue e retira as vozes dos objectos, e fala UMA
//! vez por mudança com o pico do master na mesma linha (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Fora do passo fixo de propósito (ver o corpo): uma voz é recurso do dispositivo, e o som segue o
//! que o olho já vê.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_scene_audio(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, .. } = FrameGfx::of(gfx);

        // ⭐⭐⭐ **O SOM DE CENA** (TOP-20 #4) — nasce, segue e retira as vozes dos objectos.
        //
        // ⚠️ **Aqui e não no passo fixo**, ao contrário dos dois relógios acima: uma voz é um
        // recurso do dispositivo e o `ph2d-audio` declara-se **presentation (HR-5 exempt)**. O que
        // este passe faz por quadro é reescrever ganho e pan, que é seguir o que o olho já vê —
        // amarrá-lo ao passo fixo faria o som saltar entre tiques quando o quadro é mais rápido.
        //
        // ⚠️ **Ele não escreve na cena**, logo não passa pelo `preview_drive`: som é saída.
        let audio_report = audio_2d::update(sim, self.audio.as_mut());
        // ⚠️ **Ele fala UMA vez por mudança, e não por quadro** — um relatório impresso a
        // 60 Hz não é diagnóstico, é ruído que esconde o que interessa.
        if self.signal_log_reader.is_some() && audio_report != self.last_audio_report {
            self.last_audio_report = audio_report;
            // ⭐⭐⭐ **O PICO DO MASTER vai na mesma linha**, e ele é o que separa duas avarias que
            // dão o mesmo sintoma: *«não ouço nada»* com pico `0,000` é o som a não chegar ao
            // mixer; com pico a mexer é o mixer a produzir e o **dispositivo** a não entregar.
            // ⛔ Sem ele, as duas leem-se iguais — e foi essa a pergunta que o report de 2026-09-09
            // deixou sem instrumento.
            let pico = self.audio.as_ref().map_or([0.0, 0.0], |a| a.levels());
            eprintln!(
                "[audio-2d] {} fonte(s) · {} ouvinte(s) · {} voz(es) viva(s) · {} arrancada(s) · \
                 pico do master L{:.3} R{:.3}",
                audio_report.sources,
                audio_report.listeners,
                audio_report.live,
                audio_report.started,
                pico[0],
                pico[1]
            );
        }
    }
}
