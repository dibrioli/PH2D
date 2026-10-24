//! **O nível de UMA passagem do traço** — o máximo corrente da cobertura até o traço percorrer um diâmetro de dab,
//! e daí FIXO: a sobreposição do traço consigo (os cantos, a volta, esfregar) nunca o sobe. Com o Flow como taxa
//! (Accumulate off) a cobertura de um texel depende de quantas vezes o traço passou por ele; este é o nível de
//! uma passagem só. Dois leitores, a mesma lei: a fronteira da orla ([`super::traco_plano::PlanoDoTraco`]) e o
//! alvo da mancha do Solid ([`super::solid_deposit`]).

use ph2d_painter_brush::Dab;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UmaPassagem<T = f32> {
    /// O nível (a cobertura: `0..1` no plano, `0..255` no `stroke_mask`).
    nivel: T,
    /// O nível ficou fixo (não sobe mais).
    fixo: bool,
    /// O comprimento (px) do caminho dos centros dos dabs até agora, e o último centro.
    percurso: f32,
    ultimo_centro: Option<[f32; 2]>,
    /// O maior diâmetro de dab até agora (px).
    diametro: f32,
}

impl<T: Copy + PartialOrd> UmaPassagem<T> {
    /// O nível de uma passagem agora (`0` antes da primeira observação).
    #[must_use]
    pub(super) fn nivel(&self) -> T {
        self.nivel
    }

    /// O nível ainda pode subir ([`Self::observa`] lê alguma coisa)?
    #[must_use]
    pub(super) fn aberto(&self) -> bool {
        !self.fixo
    }

    /// Soma ao percurso os centros de `dabs` e guarda o maior diâmetro.
    pub(super) fn percorre(&mut self, dabs: &[Dab]) {
        for d in dabs {
            if let Some(u) = self.ultimo_centro {
                self.percurso += (d.center[0] - u[0]).hypot(d.center[1] - u[1]);
            }
            self.ultimo_centro = Some(d.center);
            self.diametro = self.diametro.max(2.0 * d.radius_px);
        }
    }

    /// Sobe o nível para `v` enquanto não está fixo; `true` se subiu.
    pub(super) fn observa(&mut self, v: T) -> bool {
        if self.fixo || v <= self.nivel {
            return false;
        }
        self.nivel = v;
        true
    }

    /// Um diâmetro percorrido: o eixo já recebeu a passagem inteira, e o nível fica.
    pub(super) fn fixa_se_percorreu(&mut self) {
        self.fixo |= self.diametro > 0.0 && self.percurso >= self.diametro;
    }
}

impl crate::tool::PainterTool {
    /// Depois de um lote do traço com Solid: o lote anda o percurso, e enquanto o nível está aberto ele sobe ao
    /// máximo do `stroke_mask` (a cobertura do traço sob o tecto) que o lote escreveu.
    pub(super) fn observa_a_passagem_do_solid(&mut self, dabs: &[Dab]) {
        let w = self.source_size.0 as usize;
        if self.paint.solid_passagem.aberto()
            && self.paint.stroke_mask.len() == w * self.source_size.1 as usize
            && let Some(r) = self.tiled_chord_region(dabs)
        {
            let (x0, y0) = (r.x as usize, r.y as usize);
            let max = (y0..y0 + r.h as usize)
                .flat_map(|y| self.paint.stroke_mask[y * w + x0..y * w + x0 + r.w as usize].iter())
                .copied()
                .max()
                .unwrap_or(0);
            self.paint.solid_passagem.observa(max);
        }
        self.paint.solid_passagem.percorre(dabs);
        self.paint.solid_passagem.fixa_se_percorreu();
    }

    /// O nível de uma passagem do traço com Solid (`0..255`; `None` antes de o traço o medir).
    #[must_use]
    pub(super) fn passagem_do_solid(&self) -> Option<u32> {
        let n = self.paint.solid_passagem.nivel();
        (n > 0).then_some(u32::from(n))
    }
}
