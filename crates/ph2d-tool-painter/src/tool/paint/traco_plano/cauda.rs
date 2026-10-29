//! A CAUDA do plano do traço: planeá-la e re-pintá-la — filho de `traco_plano` por LOC.

use super::*;

impl PainterTool {
    /// O plano da cauda: os lotes a re-carimbar, a janela que se limpa e a que muda.
    ///
    /// ⭐ **A lei do Procreate para o traço CURTO**: a cabeça e a cauda escalam com o comprimento (`S' + E' ≤ L`).
    /// Num traço mais curto que as duas janelas (um toque: `L = 0`) o traço INTEIRO re-pinta-se com as janelas
    /// escaladas — o toque fica um ponto cheio, e a cabeça não afina por cima da cauda.
    pub(super) fn planeia_a_cauda(&self, s: &PlanoDoTraco) -> Option<Cauda> {
        let taper = self.paint.brush.taper;
        let d = 2.0 * self.paint.brush.clamped_radius();
        let todos = s.lotes.iter().flat_map(|l| l.dabs.iter());
        let total = todos.clone().map(|d| d.arc_len).fold(0.0f32, f32::max);
        let (cabeca, fim_px) = (
            if taper.is_active() {
                taper.start_px(d)
            } else {
                0.0
            },
            taper.end_px(d),
        );
        let escala = if cabeca + fim_px > total {
            if cabeca + fim_px > 0.0 {
                total / (cabeca + fim_px)
            } else {
                0.0
            }
        } else {
            1.0
        };
        let curto = escala < 1.0;
        let janela = Janelas {
            taper,
            d,
            total,
            cabeca: cabeca * escala,
            fim: fim_px * escala,
            curto,
        };
        let inicio = total - janela.fim;
        let da_cauda = |x: &Dab| curto || x.arc_len > inicio;
        let cauda_orig: Vec<Dab> = todos.copied().filter(|x| da_cauda(x)).collect();
        if cauda_orig.is_empty() {
            return None;
        }
        // A janela que se limpa: a pegada dos dabs da cauda ANTES e DEPOIS de re-afilados (o toque volta a ser o
        // ponto cheio: a pegada nova é maior que a velha), as cópias do Tiling incluídas.
        let ladrilhada = |v: &[Dab]| -> Vec<Dab> {
            if self.paint.tiling[0] || self.paint.tiling[1] {
                super::super::tiling::tiled_dabs(v, self.source_size, self.paint.tiling)
            } else {
                v.to_vec()
            }
        };
        let cauda_nova: Vec<Dab> = cauda_orig.iter().map(|x| janela.afila(*x)).collect();
        let limpa = match (
            self.dab_batch_region(&ladrilhada(&cauda_orig)),
            self.dab_batch_region(&ladrilhada(&cauda_nova)),
        ) {
            (Some(a), Some(b)) => union_region(a, b),
            (a, b) => a.or(b)?,
        };
        let reach = self.paint.brush.tip_reach();
        // A ponta fina ADENSA-SE pela cápsula: cada dab da cauda sem passo passa a levá-lo até ao dab anterior do mesmo
        // caminho, também da cauda (a faixa de antes da janela fica a do traço sem cauda), só num caminho único: sem simetria, spray nem ponta de píxel, onde dois dabs seguidos não são do
        // mesmo caminho), e os sub-dabs dele seguem o passo da largura nova. A cápsula é UMA passagem por passo — à
        // taxa do Flow repartida pelos sub-dabs sob o Accumulate desligado (`dab/bands.rs`, `c_cap`) —, como o
        // corpo; os dabs SOLTOS que adensavam a ponta pousavam cada um a passagem inteira: a cauda escurecia e,
        // com o Continuous, saía em discos (dono 25/10).
        // ⚠️ Só onde a passagem é uma TAXA (o Accumulate desligado com o Flow < 1) ou a cápsula já é a lei (o Continuous):
        // a cápsula interpola a largura em recta entre dois dabs, e com o Flow 1 os dabs soltos re-afilados um a um seguem
        // a lei exacta e batem no tecto sem escurecer nada (`o_toque_e_o_traco_curto_escalam_as_janelas`). Com o
        // Accumulate LIGADO a cápsula compõe cada sub-dab à opacidade da passagem (o traço denso, 229 sobre um corpo
        // espaçado de 72, medido 25/10): não é uma passagem por passo, e fica o adensar, RALO ([`ralo`]).
        let b = &self.paint.brush;
        let um_caminho = !b.symmetry.enabled && b.spray_count <= 1 && !self.pixel_vivo(b);
        let pela_capsula =
            (!b.accumulate && b.flow < 1.0) || cauda_orig.iter().any(|x| x.step != [0.0, 0.0]);
        let passo = self.passo_px();
        let mut anterior: Option<Dab> = None;
        // ⭐ A 1.ª cápsula da cadeia ABRE do zero (a cobertura de trás a `0`): o disco do dab de onde ela parte já
        // pousou a sua metade do passo, e a cápsula inteira por cima dele dava +10 % na entrada da janela (74 sobre um
        // corpo de 67, medido 25/10); encadear com o último dab do corpo só mudava o ressalto de sítio (75).
        let mut a_ligado = false;
        let mut encadeia = |x: &Dab| -> Dab {
            let mut y = *x;
            let mut ligou = false;
            if let Some(a) = anterior.filter(|a| {
                pela_capsula && um_caminho && da_cauda(a) && da_cauda(x) && x.step == [0.0, 0.0]
            }) {
                let p = [x.center[0] - a.center[0], x.center[1] - a.center[1]];
                let dist = p[0].hypot(p[1]);
                if dist > 0.0 && dist <= 1.5 * passo {
                    let mut ante = ph2d_painter_brush::Ante::de(&a);
                    if !a_ligado {
                        ante.coverage = 0.0;
                    }
                    (y.step, y.ante) = (p, Some(ante));
                    ligou = true;
                }
            }
            a_ligado = ligou;
            anterior = Some(*x);
            y
        };
        // ⭐ Sob o Accumulate cada dab é uma passagem source-over: o adensar põe `n = ⌈x⌉` dabs onde o passo da
        // largura nova pede `x` — logo abaixo da largura cheia `x = 1,05` dava `n = 2`, o DOBRO das passagens (a cauda
        // a 146 sobre um corpo de 72, medido 25/10). Cada dab da ponta passa a `1 − (1 − a)^r`, `r` = passo posto / passo pedido: a densidade do corpo.
        let densa = !pela_capsula && um_caminho;
        let flow = (densa && b.accumulate && b.flow > 0.0).then_some(b.flow.min(1.0));
        let (extra, razao) = if densa {
            adensa(&cauda_orig, &janela, passo, flow)
        } else {
            (Vec::new(), Vec::new())
        };
        let mut k = 0usize;
        let mut afila_na_cauda = |x: Dab| -> Dab {
            if !da_cauda(&x) {
                return x;
            }
            let y = janela.afila(x);
            k += 1;
            match (flow, razao.get(k - 1)) {
                (Some(f), Some(&r)) => ralo(y, f, r),
                _ => y,
            }
        };
        // Os lotes que tocam a janela, INTEIROS e pela ordem, com a cauda re-afilada no lugar.
        let mut lotes: Vec<Lote> = s
            .lotes
            .iter()
            .map(|l| (l, l.dabs.iter().map(&mut encadeia).collect::<Vec<Dab>>()))
            .filter(|(l, _)| ladrilhada(&l.dabs).iter().any(|c| toca(c, limpa, reach)))
            .map(|(l, dabs)| Lote {
                dabs: dabs.into_iter().map(&mut afila_na_cauda).collect(),
                ..l.clone()
            })
            .collect();
        if !extra.is_empty() {
            lotes.push(Lote {
                rng: self.paint.tex_rng,
                ancorado: false,
                dabs: extra,
            });
        }
        let tudo: Vec<Dab> = lotes.iter().flat_map(|l| l.dabs.iter().copied()).collect();
        let tocados = self
            .dab_batch_region(&ladrilhada(&tudo))
            .map_or(limpa, |t| union_region(t, limpa));
        Some(Cauda {
            lotes,
            limpa,
            tocados,
        })
    }

    /// Re-pinta a cauda no plano: limpa a janela, re-carimba os lotes com o sorteio que tiveram e repõe o que
    /// estava FORA dela (os dabs re-carimbados transbordam). O fluxo de sorteio do tool fica onde estava.
    pub(super) fn repinta_a_cauda(&mut self, s: &mut PlanoDoTraco, c: &Cauda) {
        let w = self.source_size.0 as usize;
        let antes = Arc::clone(&s.plano);
        let mascara_antes = self.paint.stroke_mask.clone();
        let p: &mut Vec<u8> = Arc::make_mut(&mut s.plano);
        limpa_regiao(p, w, c.limpa, 4);
        if self.paint.stroke_mask.len() * 4 == s.plano.len() {
            limpa_regiao(&mut self.paint.stroke_mask, w, c.limpa, 1);
        }
        let (rng, ancorado) = (self.paint.tex_rng, self.paint.pixel_trilhos.ja_ancorado);
        // O trilho de píxel (L18) anda de novo do início do re-carimbo; o último píxel pendente já está no lote
        // ANCORADO que o pen-up gravou — o que sobra no trilho no fim é ele outra vez, e descarta-se.
        self.paint.pixel_trilhos.zera();
        let escala = s.escala;
        self.dentro_do_plano(s, |t| {
            for l in &c.lotes {
                t.paint.tex_rng = l.rng;
                t.paint.pixel_trilhos.ja_ancorado = l.ancorado;
                t.stamp_dabs_dispatch(&sem_a_strength(&l.dabs, escala));
            }
        });
        self.paint.pixel_trilhos.zera();
        self.paint.pixel_trilhos.ja_ancorado = ancorado;
        self.paint.tex_rng = rng;
        let p: &mut Vec<u8> = Arc::make_mut(&mut s.plano);
        repoe_fora(p, &antes, w, c.tocados, c.limpa, 4);
        if mascara_antes.len() == self.paint.stroke_mask.len() && !mascara_antes.is_empty() {
            repoe_fora(
                &mut self.paint.stroke_mask,
                &mascara_antes,
                w,
                c.tocados,
                c.limpa,
                1,
            );
        }
    }
}
