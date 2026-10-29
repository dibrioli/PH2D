//! O campo de ALTURA do [`super::CampoAltura`]: a janela, as três escalas, o material, a luz e o céu guardados por
//! célula (partido de `oleo_real.rs`, 29/10, sem mudar uma conta).

use super::*;

impl CampoAltura {
    /// A janela `[x0..=x1]×[y0..=y1]` (células, `1..=W`) alargada pela margem e presa à grade.
    pub fn de(
        camadas: &[(&Grid, bool)],
        o: &OleoReal,
        ratio: usize,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
    ) -> Self {
        Self::de_com(&EmSerie, camadas, o, ratio, (x0, y0, x1, y1))
    }

    /// [`CampoAltura::de`] com as linhas de cada passagem repartidas por `rep` — o mesmo campo bit a bit
    /// (gate `o_campo_em_faixas_e_bit_identico_ao_em_serie`).
    pub fn de_com(
        rep: &impl Reparte,
        camadas: &[(&Grid, bool)],
        o: &OleoReal,
        ratio: usize,
        (x0, y0, x1, y1): (usize, usize, usize, usize),
    ) -> Self {
        let g = camadas[0].0;
        let af = &o.af;
        // A janela leva a margem da marcha e as duas passagens do desfoque (e o alcance da Subsurface).
        let sss = oleo_material::raio_subsuperficie_max(camadas, o, ratio, (x0, y0, x1, y1))
            .map(|r| oleo_material::alcance_subsuperficie(r, ratio));
        // D1 W3b: o raio do Smooth Edges em células.
        let r_suave = o.suave_px / ratio.max(1) as f64;
        let m = MARGEM + 2 * af.raio_suave + sss.unwrap_or(0) + alcance_do_suave(r_suave);
        let (ax0, ay0) = (x0.saturating_sub(m).max(1), y0.saturating_sub(m).max(1));
        let (ax1, ay1) = ((x1 + m).min(g.w), (y1 + m).min(g.h));
        let (w, h) = (ax1 + 1 - ax0, ay1 + 1 - ay0);
        let escala = af.px_por_massa * o.profundidade / ratio.max(1) as f64;
        let mut massa = vec![0f64; w * h];
        let mut fios = vec![0f64; w * h];
        let com_material = camadas.iter().any(|&(lg, v)| v && !lg.material.is_empty());
        // Dono 28/10: o Real Oil é do traço — com o campo em alguma camada visível, só a tinta com óleo tem altura.
        let todo_oleo = !camadas.iter().any(|&(lg, v)| v && !lg.oleo.is_empty());
        let mut cob = vec![0f64; w * h];
        por_linha(rep, w, &mut cob, &|y, linha| {
            for (x, c) in linha.iter_mut().enumerate() {
                *c = cobertura_da_celula(camadas, (ax0 + x) + (ay0 + y) * g.s);
            }
        });
        // §BUG-bordas: a cobertura com o contorno sub-célula — a parede e a cor caem no contorno LISO, não na escada.
        let borda = sem_entalhes(rep, &cob, borda_suave(rep, &cob, w, h), w, h);
        // D1 W3b: o Smooth Edges da tela amacia ESSA silhueta antes do campo — a cor e o relevo seguem a mesma.
        let borda = amacia_a_silhueta(rep, &cob, borda, (w, h), r_suave);
        // A massa (antes do desfoque: o ombro da forma também segue o contorno) e as três alturas vão com ela.
        for (c, b) in cob.iter_mut().zip(&borda) {
            *c *= f64::from(*b);
        }
        rep.por_linhas(w, &mut massa, &mut fios, &|y, massa, fios| {
            for x in 0..w {
                let i = (ax0 + x) + (ay0 + y) * g.s;
                for &(lg, visivel) in camadas {
                    if !visivel {
                        continue;
                    }
                    let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
                    // O Depth da tinta (dono 28/10, por célula) escala a massa ANTES do desfoque: a parede entre traços
                    // de Depth diferentes é a rampa do desfoque, sem degrau (doc W3b §Depth e Gloss).
                    let k =
                        crate::grid::espessura::factor(lg, i) * ganho_da_celula(lg, i, todo_oleo);
                    massa[x] += m * k;
                    // Os sulcos dos fios fundam-se com a tinta que se empilhou (a amplitude ∝ espessura).
                    if !lg.altura.is_empty() {
                        fios[x] += f64::from(lg.altura[i]) * k;
                    }
                }
                // Na borda a altura vai a zero com a cobertura: a orla fina não é um degrau.
                massa[x] *= cob[x + y * w];
                fios[x] *= cob[x + y * w];
            }
        });
        // A CRISTA da faca (`Grid::crista`, K3a) entra na altura da SOMBRA (K6): projecta a banda escura por dentro
        // da borda (`49_realshader/E4_lamina.md` §Rebordo). Só dela — os fios do Oil Bristle ficam na normal.
        let crista = camadas
            .iter()
            .any(|&(lg, v)| v && !lg.crista.is_empty())
            .then(|| {
                let mut crista = vec![0f64; w * h];
                por_linha(rep, w, &mut crista, &|y, linha| {
                    for (x, c) in linha.iter_mut().enumerate() {
                        let i = (ax0 + x) + (ay0 + y) * g.s;
                        let mut cobre = 0.0;
                        for &(lg, visivel) in camadas {
                            if !visivel {
                                continue;
                            }
                            let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
                            let k = crate::grid::espessura::factor(lg, i)
                                * ganho_da_celula(lg, i, todo_oleo);
                            *c += f64::from(crate::grid::crista::em(lg, i)) * k;
                            cobre += crate::opacity::alpha_of_mass(m) * (1.0 - cobre);
                        }
                        *c *= cobre;
                    }
                });
                crista
            });
        let material = if com_material {
            let mut material = vec![MaterialCelula::NEUTRO; w * h];
            por_linha(rep, w, &mut material, &|y, linha| {
                for (x, q) in linha.iter_mut().enumerate() {
                    let i = (ax0 + x) + (ay0 + y) * g.s;
                    for &(lg, visivel) in camadas {
                        let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
                        if visivel && m > 0.0 {
                            let alfa = crate::opacity::alpha_of_mass(m);
                            *q = MaterialCelula::mistura(*q, lg.material_da_celula(i), alfa as f32);
                        }
                    }
                }
            });
            material
        } else {
            Vec::new()
        };
        // O Gloss e o Real Oil que se VÊEM em cada célula (dono 28/10): os da camada de baixo com tinta, e cada camada
        // visível por cima pela cobertura dela. Vazios sem o campo em camada nenhuma = `1` (toda a tinta é óleo).
        let (brilho, oleo) = if todo_oleo {
            (Vec::new(), Vec::new())
        } else {
            let visto = |f: fn(crate::grid::oleo_celula::OleoCelula) -> f32, sem_tinta: f64| {
                plano_f32(rep, w, h, &|k| {
                    let i = (ax0 + k % w) + (ay0 + k / w) * g.s;
                    let mut q: Option<f64> = None;
                    for &(lg, visivel) in camadas {
                        let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
                        if visivel && m > 0.0 {
                            let b = f64::from(f(crate::grid::oleo_celula::da_celula(&lg.oleo, i)));
                            let alfa = crate::opacity::alpha_of_mass(m);
                            q = Some(q.map_or(b, |q| q + (b - q) * alfa));
                        }
                    }
                    q.unwrap_or(sem_tinta) as f32
                })
            };
            (visto(|c| c.brilho, 1.0), visto(|c| c.oleo, 0.0))
        };
        let d = |v: &[f64], r: usize| desfoca(rep, v, w, h, r);
        let suave = match marca_do_nivelado(rep, camadas, (ax0, ay0, w, h)) {
            Some((mu, peso)) => forma_viva(&massa, (&mu, &peso), &|v| {
                d(&d(v, af.raio_suave), af.raio_suave)
            }),
            None => d(&d(&massa, af.raio_suave), af.raio_suave),
        };
        let k_fios = af.fios * o.profundidade;
        // §BUG-ressalto: o desfoque alisa DENTRO da tinta; a altura de cada escala vai com a cobertura da célula,
        // e no papel sem pigmento é 0 — a parede cai no contorno do pigmento, sem saia no papel.
        // W3b: o Relevo do papel — a superfície da tinta assenta no dente do papel (`Grid::paper`, o mesmo papel do
        // substrato), pela cobertura; sem Relevo do papel (ou sem papel) a soma nem se faz (ao bit).
        let dente = o.dente_papel * af.papel_na_tinta / ratio.max(1) as f64;
        let relevo_papel =
            |k: usize| crate::material::peso(material.get(k).map_or(0.0, |m| m.relevo_papel));
        let com_papel = dente > 0.0 && !g.paper.is_empty() && com_material;
        // Paper Press (dono 28/10): `1 − amassa` da célula põe o dente do papel na ALTURA e na SOMBRA — a superfície
        // é o substrato mais a tinta, como a tinta sobre tinta. Toda a célula a `1` (a omissão): nem se soma (ao bit).
        let base = o.dente_papel * af.papel_na_base / ratio.max(1) as f64;
        let substrato = (com_papel && material.iter().any(|m| m.amassa_papel < 1.0)).then(|| {
            plano_f32(rep, w, h, &|k| {
                let segue = 1.0 - crate::material::peso(material[k].amassa_papel);
                let i = (ax0 + k % w) + (ay0 + k / w) * g.s;
                (segue * base * f64::from(g.paper[i]) * cob[k]) as f32
            })
        });
        let papel_em = |k: usize| substrato.as_ref().map_or(0.0, |s| s[k]);
        let v = plano_f32(rep, w, h, &|k| match &substrato {
            Some(s) => (suave[k] * escala * cob[k]) as f32 + s[k],
            None => (suave[k] * escala * cob[k]) as f32,
        });
        let (massa_d, fios_d) = match af.raio_detalhe {
            0 => (massa.clone(), fios),
            r => (d(&massa, r), d(&fios, r)),
        };
        let detalhe = plano_f32(rep, w, h, &|k| {
            let d = (af.grao * (massa_d[k] - suave[k]) * escala + fios_d[k] * k_fios) * cob[k];
            let r = if com_papel { relevo_papel(k) } else { 0.0 };
            if r > 0.0 {
                let i = (ax0 + k % w) + (ay0 + k / w) * g.s;
                (d + dente * r * f64::from(g.paper[i]) * cob[k]) as f32
            } else {
                d as f32
            }
        });
        let alta = match af.raio_sombra {
            0 => massa,
            r => d(&d(&massa, r), r),
        };
        let sombra = plano_f32(rep, w, h, &|k| {
            let s = (alta[k] * escala * cob[k]) as f32;
            let s = if substrato.is_some() {
                s + papel_em(k)
            } else {
                s
            };
            match &crista {
                Some(c) => s + (c[k] * k_fios) as f32,
                None => s,
            }
        });
        let albedo = if af.rebote_papel > 0.0 && o.sombra > 0.0 {
            let mut albedo = vec![[1.0f32; 3]; w * h];
            por_linha(rep, w, &mut albedo, &|y, linha| {
                for (x, q) in linha.iter_mut().enumerate() {
                    *q = albedo_da_celula(camadas, (ax0 + x) + (ay0 + y) * g.s);
                }
            });
            albedo
        } else {
            Vec::new()
        };
        let mut campo = Self {
            x0: ax0,
            y0: ay0,
            w,
            h,
            v,
            detalhe,
            sombra,
            material,
            sss: Vec::new(),
            luz_ceu: Vec::new(),
            luz_ceu_janela: (0, 0, 0, 0),
            luz_ceu_chave: None,
            albedo,
            borda,
            brilho,
            oleo,
        };
        if let Some(rr) = sss {
            let (sss, luz_ceu, janela) =
                oleo_material::espalha(rep, &campo, o, (ratio, rr), (x0, y0, x1, y1), (g.w, g.h));
            campo.sss = sss;
            campo.luz_ceu = luz_ceu;
            campo.luz_ceu_janela = janela;
            campo.luz_ceu_chave = Some(ChaveLuz::de(o));
        }
        campo
    }

    /// O que a Subsurface soma ao difuso de cada canal na célula `(x, y)`, antes do peso (`0` sem ela).
    #[inline]
    #[must_use]
    pub fn subsuperficie_em(&self, x: usize, y: usize) -> [f64; 3] {
        if self.sss.is_empty() {
            return [0.0; 3];
        }
        self.sss[self.idx(x as isize, y as isize)].map(f64::from)
    }

    /// A [`LuzCeu`] guardada da célula `(x, y)`, se a Subsurface a calculou com a mesma óptica `o`.
    #[inline]
    pub(super) fn luz_ceu_em(&self, o: &OleoReal, x: isize, y: isize) -> Option<LuzCeu> {
        let (jx, jy, jw, jh) = self.luz_ceu_janela;
        let (dx, dy) = (x - jx as isize, y - jy as isize);
        if self.luz_ceu.is_empty()
            || dx < 0
            || dy < 0
            || dx >= jw as isize
            || dy >= jh as isize
            || self.luz_ceu_chave != Some(ChaveLuz::de(o))
        {
            return None;
        }
        Some(self.luz_ceu[dx as usize + dy as usize * jw])
    }

    /// Larga a luz e o céu guardados (o sombreado volta a contá-los célula a célula; a MESMA imagem — gate
    /// `a_luz_do_ceu_guardada_e_a_de_sempre`). Para medir o que eles poupam.
    pub fn esquece_luz_do_ceu(&mut self) {
        self.luz_ceu = Vec::new();
        self.luz_ceu_chave = None;
    }

    /// O Gloss da tinta que se vê na célula `(x, y)` (`1` sem o campo em camada nenhuma).
    #[inline]
    #[must_use]
    pub fn brilho_em(&self, x: usize, y: usize) -> f64 {
        if self.brilho.is_empty() {
            return 1.0;
        }
        f64::from(self.brilho[self.idx(x as isize, y as isize)])
    }

    /// A fracção da tinta que se vê na célula `(x, y)` com o Real Oil (`1` sem o campo em camada nenhuma).
    #[inline]
    #[must_use]
    pub fn oleo_em(&self, x: usize, y: usize) -> f64 {
        if self.oleo.is_empty() {
            return 1.0;
        }
        f64::from(self.oleo[self.idx(x as isize, y as isize)])
    }

    /// O material que se vê na célula `(x, y)` ([`MaterialCelula::NEUTRO`] sem material em camada nenhuma).
    #[inline]
    #[must_use]
    pub fn material_em(&self, x: usize, y: usize) -> MaterialCelula {
        if self.material.is_empty() {
            return MaterialCelula::NEUTRO;
        }
        self.material[self.idx(x as isize, y as isize)]
    }

    /// A altura da SOMBRA na célula `(x, y)` (presa à janela): a da marcha da luz.
    #[inline]
    pub fn at_sombra(&self, x: isize, y: isize) -> f64 {
        f64::from(self.sombra[self.idx(x, y)])
    }

    /// A altura da SOMBRA num ponto qualquer, bilinear: a marcha diagonal pela célula mais próxima
    /// desenhava em escada a borda de cada estria (o pente da folha W1b).
    #[inline]
    pub(super) fn at_sombra_em(&self, x: f64, y: f64) -> f64 {
        let (fx, fy) = (exato::floor(x), exato::floor(y));
        let (tx, ty) = (x - fx, y - fy);
        let (ix, iy) = (fx as isize, fy as isize);
        let l0 =
            self.at_sombra(ix, iy) + (self.at_sombra(ix + 1, iy) - self.at_sombra(ix, iy)) * tx;
        let l1 = self.at_sombra(ix, iy + 1)
            + (self.at_sombra(ix + 1, iy + 1) - self.at_sombra(ix, iy + 1)) * tx;
        l0 + (l1 - l0) * ty
    }

    #[inline]
    pub(super) fn idx(&self, x: isize, y: isize) -> usize {
        let x = (x - self.x0 as isize).clamp(0, self.w as isize - 1) as usize;
        let y = (y - self.y0 as isize).clamp(0, self.h as isize - 1) as usize;
        x + y * self.w
    }

    /// §BUG-bordas: a fracção da célula `(x, y)` dentro do contorno sub-célula da tinta (`1` fora da orla); `> 1` no
    /// entalhe que [`sem_entalhes`] enche.
    #[inline]
    #[must_use]
    pub fn cobertura_da_borda(&self, x: usize, y: usize) -> f64 {
        if self.borda.is_empty() {
            return 1.0;
        }
        f64::from(self.borda[self.idx(x as isize, y as isize)])
    }

    /// A altura da FORMA na célula `(x, y)` (presa à janela).
    #[inline]
    pub fn at(&self, x: isize, y: isize) -> f64 {
        f64::from(self.v[self.idx(x, y)])
    }

    /// A altura com o DETALHE (a que dá a normal).
    #[inline]
    pub fn at_fina(&self, x: isize, y: isize) -> f64 {
        let i = self.idx(x, y);
        f64::from(self.v[i]) + f64::from(self.detalhe[i])
    }
}
