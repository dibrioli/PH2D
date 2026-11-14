//! **O VIDRO na janela** (ADR-0205 D1) — filho de [`super`]: o journal por tile do plano do vidro, a
//! 5.ª família do documento, no molde de [`super::ReliefJournals`] (um mapa por camada: dois planos de
//! camadas diferentes têm a mesma forma, e um 2.º dono marca o journal como misturado).

use super::WriteState;
use crate::compositor::vidro::Vidro;

/// Os bytes velhos do plano do vidro, mais de QUE camada eles falam.
#[derive(Clone, Debug, Default)]
pub(super) struct VidroJournal {
    layer: Option<crate::layers::LayerId>,
    mixed: bool,
    incomplete: bool,
    /// O plano foi escrito sem forma de canvas (1.º toque da camada): não há antes a descrever (`OnlyAfter`).
    absent: bool,
    tiles: crate::undo::journal::TileJournal<Vidro>,
}

thread_local! {
    /// **A chave do A×B** (ADR-0205): `true` = o `before` e o cursor DESCREVEM o vidro (testemunha `Weak` +
    /// journal por tile); `false` = o caminho de antes, que os segura (o oráculo).
    static VIDRO_PELO_JOURNAL: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// A chave [`VIDRO_PELO_JOURNAL`] deste fio.
#[doc(hidden)]
#[must_use]
pub fn vidro_pelo_journal() -> bool {
    VIDRO_PELO_JOURNAL.with(std::cell::Cell::get)
}

/// Liga/desliga [`VIDRO_PELO_JOURNAL`] neste fio (a sonda e os gates intercalam as duas rotas).
#[doc(hidden)]
pub fn set_vidro_pelo_journal(on: bool) {
    VIDRO_PELO_JOURNAL.with(|c| c.set(on));
}

impl VidroJournal {
    fn note_layer(&mut self, l: crate::layers::LayerId) {
        match self.layer {
            None => self.layer = Some(l),
            Some(cur) if cur == l => {}
            Some(_) => {
                self.mixed = true;
                self.tiles.reset();
            }
        }
    }

    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn heap_bytes(&self) -> usize {
        self.tiles.heap_bytes()
    }
}

impl WriteState {
    /// **Captura o vidro da `layer` antes de ele ser escrito** — `area` em texels (um elemento por pixel).
    pub(crate) fn capture_vidro(
        &self,
        layer: crate::layers::LayerId,
        buf: &[Vidro],
        stride: usize,
        area: Option<(usize, usize, usize, usize)>,
    ) {
        let mut j = self.vidro.borrow_mut();
        // O plano nasceu neste passo: não há antes a descrever (`OnlyAfter`), e os bytes de agora não o são.
        if j.absent && j.layer == Some(layer) {
            return;
        }
        j.note_layer(layer);
        j.tiles.capture(buf, stride, area);
    }

    /// Uma escrita chegou a um vidro SEM forma de canvas: com o journal vazio o plano não existia no começo
    /// do passo (`absent`); com tiles já tomados ele perdeu a forma no meio (`incomplete`).
    pub(crate) fn note_absent_vidro(&self, layer: crate::layers::LayerId) {
        let mut j = self.vidro.borrow_mut();
        if !j.tiles.is_empty() {
            j.incomplete = true;
            return;
        }
        j.note_layer(layer);
        j.absent = true;
    }

    /// `true` se o journal do vidro pode responder pela camada `l` (só os portões perguntam).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn vidro_speaks_for(&self, l: crate::layers::LayerId) -> bool {
        let j = self.vidro.borrow();
        !j.mixed && !j.incomplete && j.layer.is_none_or(|cur| cur == l)
    }

    /// O antes do texel `i`, se o journal o capturou (`antes[i] = journal.get(i).unwrap_or(vivo[i])`, §5.28).
    /// Só os testes o lêem (a produção lê por corrida).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn vidro_before(&self, i: usize) -> Option<Vidro> {
        self.vidro.borrow().tiles.get(i)
    }

    /// **O antes do vidro, numa janela densa** `[x0, x1) × [y0, y1)` de um plano `w` de largura:
    /// `journal.get(i).unwrap_or(vivo[i])` (ADR-0205 D2). Tirada ANTES da escrita da janela.
    #[must_use]
    pub(crate) fn vidro_antes_na_janela(
        &self,
        vivo: &[Vidro],
        w: usize,
        (x0, x1, y0, y1): (usize, usize, usize, usize),
    ) -> Vec<Vidro> {
        let j = self.vidro.borrow();
        let mut out = Vec::with_capacity((x1 - x0) * (y1 - y0));
        for y in y0..y1 {
            // Sem tile tomado o journal pode nem estar armado (`stride == 0`): o antes é o vivo.
            if j.tiles.is_empty() {
                out.extend_from_slice(&vivo[y * w + x0..y * w + x1]);
            } else {
                j.tiles.read_row_into(y, x0, x1 - x0, vivo, &mut out);
            }
        }
        out
    }

    /// Ninguém escreveu vidro desde o último reset (o vivo É o antes, pelos mesmos bytes).
    #[must_use]
    pub(crate) fn vidro_untouched(&self) -> bool {
        let j = self.vidro.borrow();
        j.layer.is_none() && !j.mixed && !j.incomplete && !j.absent
    }

    /// **O vidro do topo COMPOSTO com o journal** — a absorção barata do escorrido (ADR-0205): `topo` = o mapa do
    /// vidro da entrada do topo, `vivo` = o mapa de agora. Devolve o mapa `antes0 → vivo` e se o escorrido mudou
    /// algum byte do vidro. `None` = o journal não o descreve, ou a forma não compõe (quem chama segura o plano).
    pub(crate) fn vidro_composto(
        &self,
        topo: &crate::undo_delta::StoredMap<Vidro>,
        vivo: &crate::compositor::vidro::Vidros,
    ) -> Option<(crate::undo_delta::StoredMap<Vidro>, bool)> {
        let j = self.vidro.borrow();
        if j.mixed || j.incomplete || j.absent {
            return None;
        }
        topo.composto_com_o_journal(vivo, j.layer?, &j.tiles)
    }

    /// **O vidro do último reset, SEGURADO** — a absorção do escorrido (ADR-0205): `testemunhas` = o cursor
    /// elidido, `vivo` = o mapa de agora. A camada do journal volta pelos tiles (`rewind_into`); as outras
    /// têm de ser o MESMO plano que a testemunha aponta. `None` = o journal não o descreve (quem chama recusa).
    pub(crate) fn vidro_no_reset(
        &self,
        testemunhas: &std::collections::BTreeMap<
            crate::layers::LayerId,
            std::sync::Weak<Vec<Vidro>>,
        >,
        vivo: &crate::compositor::vidro::Vidros,
    ) -> Option<crate::compositor::vidro::Vidros> {
        let j = self.vidro.borrow();
        if j.mixed || j.incomplete {
            return None;
        }
        let mut out = crate::compositor::vidro::Vidros::new();
        for (k, w) in testemunhas {
            let live = vivo.get(k)?;
            if j.layer == Some(*k) && (j.absent || !j.tiles.is_empty()) {
                // Plano que não existia no reset e a testemunha diz que sim: o mesmo braço recusado do `from_journal`.
                if j.absent {
                    return None;
                }
                let mut v = live.as_ref().clone();
                if !j.tiles.rewind_into(&mut v) {
                    return None;
                }
                out.insert(*k, std::sync::Arc::new(v));
            } else if w
                .upgrade()
                .is_some_and(|p| std::sync::Arc::ptr_eq(&p, live))
            {
                out.insert(*k, std::sync::Arc::clone(live));
            } else {
                return None;
            }
        }
        Some(out)
    }

    /// O journal do vidro descreve o passo que começou em `writes`: `f(camada, absent, tiles)`.
    pub(crate) fn with_vidro_before<R>(
        &self,
        writes: u64,
        fallback_layer: crate::layers::LayerId,
        f: impl FnOnce(crate::layers::LayerId, bool, &crate::undo::journal::TileJournal<Vidro>) -> R,
    ) -> Option<R> {
        if !self.journal_describes_step_at(writes) {
            return None;
        }
        let j = self.vidro.borrow();
        if j.mixed || j.incomplete {
            return None;
        }
        Some(f(j.layer.unwrap_or(fallback_layer), j.absent, &j.tiles))
    }

    /// Os bytes que o journal do vidro retém.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn vidro_journal_bytes(&self) -> usize {
        self.vidro.borrow().heap_bytes()
    }

    /// O plano foi escrito sem forma de canvas neste passo.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn vidro_absent(&self) -> bool {
        self.vidro.borrow().absent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::LayerId;

    /// O texel `(x, y)` de um plano `w` de largura, com um padrão que não se repete entre tiles.
    fn texel(x: usize, y: usize, k: u8) -> Vidro {
        let v = (x * 31 + y * 17) as u8;
        [v, v ^ k, k, 1, 2, 3, v.wrapping_add(k)]
    }

    /// ADR-0205 D1: a captura toma os tiles que a região toca (os parciais pelo tamanho real), só a 1.ª vez,
    /// e o antes reconstruído é o plano de antes ao bit; outra camada mistura o journal.
    #[test]
    fn o_journal_do_vidro_guarda_os_tiles_que_a_regiao_toca_e_so_a_primeira_vez() {
        let (w, h) = (300usize, 200usize); // tiles: x 128·128·44, y 128·72
        let antes: Vec<Vidro> = (0..w * h).map(|i| texel(i % w, i / w, 0)).collect();
        let ws = WriteState::default();
        let l = LayerId(7);
        assert_eq!(ws.vidro_journal_bytes(), 0);
        ws.capture_vidro(l, &antes, w, Some((100, 50, 200, 140)));
        let t = |a: usize, b: usize| a * b * size_of::<Vidro>();
        assert_eq!(
            ws.vidro_journal_bytes(),
            2 * t(128, 128) + 2 * t(128, 72),
            "quatro tiles, os de baixo parciais"
        );
        // O passo escreve; a 2.ª captura dos mesmos tiles não os toma de novo.
        let depois: Vec<Vidro> = (0..w * h).map(|i| texel(i % w, i / w, 9)).collect();
        ws.capture_vidro(l, &depois, w, Some((150, 0, 290, 10)));
        assert_eq!(
            ws.vidro_journal_bytes(),
            2 * t(128, 128) + 2 * t(128, 72) + t(44, 128),
            "só o tile (2, 0) é novo"
        );
        for (x, y) in [
            (0, 0),
            (127, 127),
            (128, 0),
            (255, 199),
            (256, 0),
            (299, 127),
        ] {
            let i = y * w + x;
            assert_eq!(
                ws.vidro_before(i).unwrap_or(depois[i]),
                if x < 256 { antes[i] } else { depois[i] },
                "o antes de ({x}, {y})"
            );
        }
        assert!(ws.vidro_speaks_for(l));
        ws.capture_vidro(LayerId(8), &depois, w, Some((0, 0, 1, 1)));
        assert!(!ws.vidro_speaks_for(l), "outra camada mistura o journal");
        ws.reset_journal();
        assert_eq!(ws.vidro_journal_bytes(), 0);
        assert!(ws.vidro_speaks_for(l));
    }
}
