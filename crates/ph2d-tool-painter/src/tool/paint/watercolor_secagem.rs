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
//! poça separa-se do que fica por mais que DUAS vezes o alcance do composite, medido ao texel
//! ([`PainterTool::alcance_da_janela`]): nenhum texel de um lado lê um texel do outro, e o re-render
//! da união reproduz a tela ao byte. Secar DENTRO de uma poça (a frente que recua das bordas, doc 14
//! #12b) continua por fazer, por esta razão.

use super::*;
use rayon::prelude::*;

/// Um troço de linha da união da sessão: os texels `[a, b)` da linha `y`, e se algum tem papel molhado.
#[derive(Clone, Copy)]
struct Troco {
    y: usize,
    a: usize,
    b: usize,
    molhado: bool,
}

fn raiz(pai: &mut [u32], mut i: u32) -> u32 {
    while pai[i as usize] != i {
        pai[i as usize] = pai[pai[i as usize] as usize];
        i = pai[i as usize];
    }
    i
}

fn une(pai: &mut [u32], i: usize, j: usize) {
    let (ri, rj) = (raiz(pai, i as u32), raiz(pai, j as u32));
    if ri != rj {
        pai[ri.max(rj) as usize] = ri.min(rj);
    }
}

/// A folga das leituras LOCAIS do ponto amostrado: o bilinear (1 px) e as sub-amostras do AA (½ px).
const FOLGA: usize = 2;

/// **A distância (Chebyshev, px) a partir da qual duas poças não se leem** — ligam-se se estão a
/// `≤` isto. Cada texel de saída lê a união em volta de UM ponto amostrado `s` (o texel deslocado pelo
/// Ragged/Paper Edge): a cobertura que decide se ele pinta, a `FOLGA` de `s`; os campos (aro,
/// dissolução, reserva), a `reach + FOLGA` de `s`. Assar não muda um byte se: (1) nenhum `s` com
/// cobertura do que FICA lê dados do que ASSA — `reach + 2·FOLGA`; (2) a zona onde a base passa a ser a
/// tela (`desloca + FOLGA` do que assa) não toca um texel que o que fica pinta (`desloca + FOLGA`
/// dele) — `2·(desloca + FOLGA)`; (3) com a dissolução viva (Rewet ou água) ela lê a BASE à volta de
/// `s`, e a base mudou na zona — mais `desloca + FOLGA`; (4) a água lê-se no texel serrilhado (sem
/// deslocamento), a `BACKRUN_JAG_PX + 1` dele — mais `desloca` e o serrilhado. A soma `2·pad` da 1.ª
/// versão contava o deslocamento DUAS vezes nos campos e pedia 28 px ao pincel de fábrica (16 basta).
fn separacao(al: &super::watercolor_render::Alcance) -> usize {
    let w = al.desloca + FOLGA;
    let dissolve = al.wet_any > 0.0 || al.watered;
    let campos = al.reach + 2 * FOLGA + if dissolve { w } else { 0 };
    let agua = if al.watered {
        al.reach + FOLGA + w + super::watercolor_field::BACKRUN_JAG_PX.ceil() as usize + 1
    } else {
        0
    };
    campos.max(2 * w).max(agua)
}

impl PainterTool {
    /// Assa as poças da sessão que já secaram e estão longe de toda a tinta molhada. Devolve quantos
    /// troços de linha assaram (`0` = nada mudou). Só no pen-down de um traço que continua a sessão.
    ///
    /// ⭐ **A poça é medida ao TEXEL**, a [`separacao`] px. A 1.ª versão media em células de 16 px com
    /// um elo de quatro células e exigia ~65 px: traços a 40 px uns dos outros eram uma poça só e nada
    /// secava enquanto se pintava ao lado (smoke do dono, 2026-10-04).
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
        let al = self.alcance_da_janela();
        let pad = al.pad;
        let elo = separacao(&al);
        // A união vive no retângulo cumulativo; a folga de `pad` apanha o soak, que se derrama além
        // do disco do carimbo.
        let x0 = (r.x as usize).saturating_sub(pad).min(fw);
        let y0 = (r.y as usize).saturating_sub(pad).min(fh);
        let x1 = ((r.x + r.w) as usize + pad).min(fw);
        let y1 = ((r.y + r.h) as usize + pad).min(fh);
        if x0 >= x1 || y0 >= y1 {
            return 0;
        }
        let cov = &self.paint.stroke_coverage;
        let molhado = &self.paint.canvas_wet;
        let agua = (self.paint.stroke_water.len() == n).then_some(&self.paint.stroke_water[..]);
        let soak = (self.paint.wet_soak.len() == n).then_some(&self.paint.wet_soak[..]);
        let linhas: Vec<Vec<Troco>> = (y0..y1)
            .into_par_iter()
            .map(|y| {
                let mut v = Vec::new();
                let mut aberto: Option<Troco> = None;
                for x in x0..x1 {
                    let i = y * fw + x;
                    let ocupado = cov[i] > 0
                        || agua.is_some_and(|a| a[i] > 0)
                        || soak.is_some_and(|s| s[i] > 0);
                    match (&mut aberto, ocupado) {
                        (Some(t), true) => {
                            t.b = x + 1;
                            t.molhado |= molhado[i] > 0;
                        }
                        (None, true) => {
                            aberto = Some(Troco {
                                y,
                                a: x,
                                b: x + 1,
                                molhado: molhado[i] > 0,
                            });
                        }
                        (Some(_), false) => v.extend(aberto.take()),
                        (None, false) => {}
                    }
                }
                v.extend(aberto);
                v
            })
            .collect();
        let mut inicio = Vec::with_capacity(linhas.len() + 1);
        let mut trocos: Vec<Troco> = Vec::new();
        for l in &linhas {
            inicio.push(trocos.len());
            trocos.extend_from_slice(l);
        }
        inicio.push(trocos.len());
        if trocos.is_empty() {
            return 0;
        }
        // As poças: união dos troços a `elo` px um do outro (na mesma linha e nas `elo` de cima).
        let mut pai: Vec<u32> = (0..trocos.len() as u32).collect();
        for j in 0..linhas.len() {
            let (qa, qb) = (inicio[j], inicio[j + 1]);
            for k in qa + 1..qb {
                if trocos[k].a < trocos[k - 1].b + elo {
                    une(&mut pai, k - 1, k);
                }
            }
            for jj in j.saturating_sub(elo)..j {
                let (pa, pb) = (inicio[jj], inicio[jj + 1]);
                let mut lo = pa;
                for q in qa..qb {
                    let tq = trocos[q];
                    while lo < pb && trocos[lo].b + elo <= tq.a {
                        lo += 1;
                    }
                    let mut p = lo;
                    while p < pb && trocos[p].a < tq.b + elo {
                        une(&mut pai, p, q);
                        p += 1;
                    }
                }
            }
        }
        let mut fica = vec![false; trocos.len()];
        for k in 0..trocos.len() {
            if trocos[k].molhado {
                fica[raiz(&mut pai, k as u32) as usize] = true;
            }
        }
        let assa: Vec<Troco> = (0..trocos.len())
            .filter(|&k| !fica[raiz(&mut pai, k as u32) as usize])
            .map(|k| trocos[k])
            .collect();
        if assa.is_empty() {
            return 0;
        }
        // A ZONA do que ASSOU: os texels cujo ponto amostrado pode cair a `FOLGA` de um troço assado
        // (a `desloca + FOLGA` dele). Lá a base da sessão passa a ser a tela — o re-render (cobertura
        // zero) devolve a base; nenhum desses texels tem cobertura do que fica perto do ponto dele.
        let zona_r = al.desloca + FOLGA;
        let (zx0, zy0) = (x0.saturating_sub(zona_r), y0.saturating_sub(zona_r));
        let (zx1, zy1) = ((x1 + zona_r).min(fw), (y1 + zona_r).min(fh));
        let zw = zx1 - zx0;
        let mut zona = vec![false; zw * (zy1 - zy0)];
        for t in &assa {
            let (ya, yb) = (
                t.y.saturating_sub(zona_r).max(zy0),
                (t.y + zona_r + 1).min(zy1),
            );
            let (xa, xb) = (t.a.saturating_sub(zona_r).max(zx0), (t.b + zona_r).min(zx1));
            for y in ya..yb {
                zona[(y - zy0) * zw + xa - zx0..(y - zy0) * zw + xb - zx0].fill(true);
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
        base[zy0 * fw * 4..zy1 * fw * 4]
            .par_chunks_mut(fw * 4)
            .zip(tela[zy0 * fw * 4..zy1 * fw * 4].par_chunks(fw * 4))
            .zip(zona.par_chunks(zw))
            .for_each(|((linha, de), marca)| {
                for (k, &m) in marca.iter().enumerate() {
                    if m {
                        let i = (zx0 + k) * 4;
                        linha[i..i + 4].copy_from_slice(&de[i..i + 4]);
                    }
                }
            });
        // A união ESQUECE a poça assada: ela está na base, e um traço que lhe passe por cima começa do
        // zero ali, como numa sessão nova.
        let zera = |plano: &mut Vec<u8>, canais: usize| {
            if plano.len() != n * canais {
                return;
            }
            for t in &assa {
                plano[(t.y * fw + t.a) * canais..(t.y * fw + t.b) * canais].fill(0);
            }
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
        assa.len()
    }
}
