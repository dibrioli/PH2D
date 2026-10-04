//! **CADA POÇA SECA NO SEU TEMPO** (decisão do dono 2026-10-04, doc 46 §2-5). A sessão molhada era
//! tudo-ou-nada: enquanto um canto do papel estivesse molhado, toda aguada da sessão fundia com o traço
//! seguinte. No pen-down de um traço que CONTINUA a sessão, a poça que secou e está LONGE de toda a
//! tinta molhada assa-se: a base da sessão recebe a tela ali e a união esquece-a — o traço novo vela
//! por cima dela como sobre papel seco, e continua a fundir onde a tinta está molhada.
//!
//! ⭐ **A poça é a unidade, não o texel — e é isso que deixa o assar sem um byte de diferença.** O
//! composite lê a VIZINHANÇA da união (o aro é um borrão da cobertura, a dissolução um borrão de raio
//! Spread, o warp desloca a amostra). Cortar a união DENTRO de uma poça muda a vizinhança dos dois
//! lados do corte: o lado molhado ganharia um aro novo e o seco re-depositaria a tinta dos vizinhos. A
//! poça separa-se do que fica por mais que DUAS vezes o alcance do composite
//! ([`PainterTool::alcance_da_janela`]): nenhum texel de um lado lê um texel do outro, e o re-render
//! da união reproduz a tela ao byte. Secar DENTRO de uma poça (a frente que recua das bordas, doc 14
//! #12b) continua por fazer, por esta razão.

use super::*;
use rayon::prelude::*;

/// O lado mínimo da célula da grelha em que as poças se separam (px). A célula cresce com o alcance
/// (`pad / 2`), para a vizinhança de uma célula ficar em poucas células por mais largo que o pincel.
const CELULA_MIN: usize = 16;

impl PainterTool {
    /// Assa as poças da sessão que já secaram e estão longe de toda a tinta molhada. Devolve quantas
    /// células assaram (`0` = nada mudou). Só no pen-down de um traço que continua a sessão.
    pub(super) fn assa_as_pocas_secas(&mut self) -> usize {
        let (fw, fh) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let n = fw * fh;
        if n == 0
            || self.paint.canvas_wet.len() != n
            || self.paint.stroke_coverage.len() != n
            || self.canvas_rgba.len() != n * 4
            || self
                .paint
                .wet_session_base
                .as_ref()
                .is_none_or(|b| b.len() != n * 4)
        {
            return 0;
        }
        let Some(r) = self.paint.wet_cum_dirty else {
            return 0;
        };
        let pad = self.alcance_da_janela().pad_maximo;
        let lado = CELULA_MIN.max(pad.div_ceil(2));
        let k_pad = pad.div_ceil(lado);
        // Duas células ocupadas a mais que `elo` células uma da outra estão a mais de `2·pad + lado` px.
        let elo = 2 * k_pad + 2;
        // A grelha cobre a união da sessão com folga de `elo` células: uma célula molhada fora da união
        // (o botão Wet molha o papel inteiro) ainda segura a poça a que chega.
        let cx0 = (r.x as usize / lado).saturating_sub(elo);
        let cy0 = (r.y as usize / lado).saturating_sub(elo);
        let cx1 = ((r.x + r.w) as usize).div_ceil(lado).min(fw.div_ceil(lado)) + elo;
        let cy1 = ((r.y + r.h) as usize).div_ceil(lado).min(fh.div_ceil(lado)) + elo;
        let cx1 = cx1.min(fw.div_ceil(lado));
        let cy1 = cy1.min(fh.div_ceil(lado));
        if cx0 >= cx1 || cy0 >= cy1 {
            return 0;
        }
        let (gw, gh) = (cx1 - cx0, cy1 - cy0);
        // (ocupada, molhada) por célula: a união é tudo o que o composite lê — cobertura, água, soak.
        let cov = &self.paint.stroke_coverage;
        let molhado = &self.paint.canvas_wet;
        let agua = (self.paint.stroke_water.len() == n).then_some(&self.paint.stroke_water[..]);
        let soak = (self.paint.wet_soak.len() == n).then_some(&self.paint.wet_soak[..]);
        let celulas: Vec<(bool, bool)> = (0..gh)
            .into_par_iter()
            .flat_map_iter(|gy| {
                let y0 = (cy0 + gy) * lado;
                let y1 = (y0 + lado).min(fh);
                (0..gw).map(move |gx| {
                    let x0 = (cx0 + gx) * lado;
                    let x1 = (x0 + lado).min(fw);
                    let (mut ocupada, mut molhada) = (false, false);
                    for y in y0..y1 {
                        for i in y * fw + x0..y * fw + x1 {
                            ocupada |= cov[i] > 0
                                || agua.is_some_and(|a| a[i] > 0)
                                || soak.is_some_and(|s| s[i] > 0);
                            molhada |= molhado[i] > 0;
                        }
                    }
                    (ocupada, molhada)
                })
            })
            .collect();
        // A poça que FICA: as células ocupadas que uma célula molhada alcança por elos de `elo` células.
        let mut fica = vec![false; gw * gh];
        let mut fila: Vec<usize> = (0..gw * gh).filter(|&c| celulas[c].1).collect();
        for &c in &fila {
            fica[c] = true;
        }
        while let Some(c) = fila.pop() {
            let (gx, gy) = (c % gw, c / gw);
            for vy in gy.saturating_sub(elo)..(gy + elo + 1).min(gh) {
                for vx in gx.saturating_sub(elo)..(gx + elo + 1).min(gw) {
                    let v = vy * gw + vx;
                    if !fica[v] && celulas[v].0 {
                        fica[v] = true;
                        fila.push(v);
                    }
                }
            }
        }
        let assa: Vec<bool> = (0..gw * gh).map(|c| celulas[c].0 && !fica[c]).collect();
        let assadas = assa.iter().filter(|&&a| a).count();
        if assadas == 0 {
            return 0;
        }
        // A zona de LEITURA do que fica: as células a `k_pad + 1` de uma que fica. Fora dela a base da
        // sessão passa a ser a tela — o re-render do que assou (cobertura zero) devolve a base, e onde
        // nada da sessão chega a tela já É a base.
        let mut le = vec![false; gw * gh];
        for c in (0..gw * gh).filter(|&c| fica[c] && celulas[c].0) {
            let (gx, gy) = (c % gw, c / gw);
            for vy in gy.saturating_sub(k_pad + 1)..(gy + k_pad + 2).min(gh) {
                for vx in gx.saturating_sub(k_pad + 1)..(gx + k_pad + 2).min(gw) {
                    le[vy * gw + vx] = true;
                }
            }
        }
        let tela = Arc::clone(&self.canvas_rgba);
        let base = self
            .paint
            .wet_session_base
            .as_mut()
            .expect("conferida no topo");
        if Arc::get_mut(base).is_none() {
            // Partilhada: a cópia pelas cores (o `make_mut` copiava `67 MB` num fio só a 4096²).
            *base = Arc::new(crate::plane_copy::par_clone(base));
        }
        let base = Arc::get_mut(base).expect("acabou de ficar única");
        let faixa = cy0 * lado * fw * 4..(cy1 * lado).min(fh) * fw * 4;
        base[faixa.clone()]
            .par_chunks_mut(fw * 4)
            .zip(tela[faixa].par_chunks(fw * 4))
            .enumerate()
            .for_each(|(j, (linha, de))| {
                let gy = j / lado;
                for gx in 0..gw {
                    if !le[gy * gw + gx] {
                        let (x0, x1) = ((cx0 + gx) * lado * 4, ((cx0 + gx + 1) * lado).min(fw) * 4);
                        linha[x0..x1].copy_from_slice(&de[x0..x1]);
                    }
                }
            });
        // A união ESQUECE a poça assada: ela está na base, e um traço que lhe passe por cima começa do
        // zero ali, como numa sessão nova.
        let zera = |plano: &mut Vec<u8>, canais: usize| {
            if plano.len() != n * canais {
                return;
            }
            let faixa = cy0 * lado * fw * canais..(cy1 * lado).min(fh) * fw * canais;
            plano[faixa]
                .par_chunks_mut(fw * canais)
                .enumerate()
                .for_each(|(j, linha)| {
                    let gy = j / lado;
                    for gx in (0..gw).filter(|&gx| assa[gy * gw + gx]) {
                        let x0 = (cx0 + gx) * lado * canais;
                        let x1 = ((cx0 + gx + 1) * lado).min(fw) * canais;
                        linha[x0..x1].fill(0);
                    }
                });
        };
        let p = &mut self.paint;
        zera(&mut p.stroke_coverage, 1);
        zera(&mut p.stroke_color, 4);
        zera(&mut p.stroke_density, 1);
        zera(&mut p.stroke_deplete, 1);
        zera(&mut p.stroke_deplete_prox, 1);
        zera(&mut p.stroke_water, 1);
        zera(&mut p.wet_soak, 1);
        p.wet_reserve_cache = None; // escrita em massa nos planos do nível
        assadas
    }
}
