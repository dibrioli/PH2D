//! ⭐ W5b (F4) — a mesma entrada SEM e COM o mundo fora da grade (`DISTANTES`). O quadro sem nenhum
//! objeto distante corre a sem: o código dele custa mesmo com as listas vazias (+2,5 % a +6 % no Low,
//! +1,3 % a +2,3 % no High — rodada `medida_luz2d_w5b`, A2). ⭐ W5o: as duas compilam em PARALELO
//! (a frio, a grade fina leva 2–5 s por pipeline no compilador do driver — rodada `medida_luz2d_w5o`).
//! Módulo-filho de `solver_c3.rs` (o tecto de 700 LOC).

pub(super) struct Par {
    pub(super) sem: wgpu::ComputePipeline,
    pub(super) com: Option<wgpu::ComputePipeline>,
}

impl Par {
    pub(super) fn de(&self, com: bool) -> &wgpu::ComputePipeline {
        if com {
            self.com.as_ref().unwrap_or(&self.sem)
        } else {
            &self.sem
        }
    }

    /// A sem e (com `com`) a com, cada uma no seu fio.
    pub(super) fn em_paralelo(com: bool, em: impl Fn(f64) -> wgpu::ComputePipeline + Sync) -> Self {
        std::thread::scope(|s| {
            let outra = com.then(|| s.spawn(|| em(1.0)));
            let sem = em(0.0);
            Self {
                sem,
                com: outra.map(|h| h.join().expect("a compilação da pipeline com os distantes")),
            }
        })
    }
}
