//! **O tempo de uma [`AnimationTag`]** — a duração de cada célula, o ritmo e os tempos PRÓPRIOS
//! por passo (multiplicador do ritmo, a lei do Godot). Irmão de [`crate::sprite_anim`]: a tag e o
//! tocador moram lá, as contas de tempo da tag moram aqui.

use crate::sprite_anim::{AnimationTag, FRAME_MS_MAX, FRAME_MS_MIN};

impl AnimationTag {
    /// **A duração da célula `frame`**, em microssegundos, com o cap da spec imposto.
    ///
    /// ⚠️ **A lei já perguntava POR FRAME** — o `step_ticks` chama-a dentro do laço, com a célula
    /// que está no ecrã. Era só a resposta que era uniforme, e é por isso que a duração por-quadro
    /// coube numa função e não numa refactoração: *quando a pergunta certa já está no sítio certo,
    /// a feature é a resposta*.
    ///
    /// `lo` é o `from` **resolvido** (o intervalo pode ter sido cortado pela grelha de hoje), e é
    /// contra ele que o vetor se indexa.
    pub(crate) fn frame_ticks_at(&self, frame: u32, lo: u32) -> u64 {
        let ms = self
            .per_frame_ms
            .get(frame.saturating_sub(lo) as usize)
            .copied()
            // `0` é «não declarado», e não «instantâneo»: um zero real faria o laço de recuperação
            // do `advance` girar até ao guarda, e um ficheiro adulterado não pode fazer isso.
            .filter(|v| *v > 0)
            .unwrap_or(self.frame_ms);
        u64::from(ms.clamp(FRAME_MS_MIN, FRAME_MS_MAX)) * 1_000
    }

    /// Esta tag tem ritmo próprio por célula? (O que o painel mostra, e o que o importador diz.)
    #[must_use]
    pub fn has_per_frame_timing(&self) -> bool {
        // Um valor igual ao `frame_ms` não é tempo PRÓPRIO: é o ritmo da tag escrito por extenso.
        self.per_frame_ms
            .iter()
            .any(|&v| v > 0 && v != self.frame_ms)
    }

    /// **O tempo do passo `i` como MULTIPLICADOR do ritmo** (a lei do Godot: quadros/s + duração
    /// relativa por quadro). `1.0` = herda o ritmo.
    #[must_use]
    pub fn frame_factor(&self, i: usize) -> f32 {
        match self.per_frame_ms.get(i).copied().filter(|v| *v > 0) {
            Some(ms) => ms as f32 / self.frame_ms.max(1) as f32,
            None => 1.0,
        }
    }

    /// O ms que um multiplicador dá sobre o ritmo desta tag; `≤ 0` = herda (`0`).
    #[must_use]
    pub fn ms_of_factor(&self, factor: f32) -> u32 {
        if !(factor.is_finite() && factor > 0.0) {
            return 0;
        }
        ((factor * self.frame_ms as f32).round() as u32).clamp(FRAME_MS_MIN, FRAME_MS_MAX)
    }

    /// **Escreve o tempo PRÓPRIO do passo `index`** no vetor esparso; `0` ou um valor igual ao ritmo
    /// = herda (fica `0`, e os zeros do fim podam-se — limpar é idempotente e o ficheiro não guarda
    /// resíduo).
    pub fn set_own_ms(&mut self, index: usize, ms: u32) {
        let ms = if ms == 0 || ms == self.frame_ms {
            0
        } else {
            ms.clamp(FRAME_MS_MIN, FRAME_MS_MAX)
        };
        if index >= self.per_frame_ms.len() {
            self.per_frame_ms.resize(index + 1, 0);
        }
        self.per_frame_ms[index] = ms;
        while self.per_frame_ms.last() == Some(&0) {
            self.per_frame_ms.pop();
        }
    }

    /// **Muda o ritmo e LEVA os tempos próprios na mesma proporção** (report do dono 25/10: com
    /// tempos próprios gravados o Frames/s «não funcionava»). Um quadro a 2× continua a 2×.
    ///
    /// ⚠️ Escala pelo multiplicador em duas casas (o que o painel mostra), e não pela razão crua:
    /// a razão crua arredondada a ms anda 1 ms a cada ida e volta (266 → 110 → 265).
    pub fn set_rhythm(&mut self, ms: u32) {
        let old = f64::from(self.frame_ms.max(1));
        let new = ms.clamp(FRAME_MS_MIN, FRAME_MS_MAX);
        let own = std::mem::take(&mut self.per_frame_ms);
        self.frame_ms = new;
        for (i, v) in own.into_iter().enumerate() {
            if v > 0 {
                let factor = (f64::from(v) / old * 100.0).round() / 100.0;
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let scaled = (factor * f64::from(new)).round().max(1.0) as u32;
                self.set_own_ms(i, scaled);
            }
        }
    }
}
