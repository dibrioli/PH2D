//! **O PLANO DO TRAÇO** (L8 + a cauda da L5, `docs/Painter/Pinceis/03_MOTORES.md` §4): no Digital, um traço
//! com tecto, orla ou cauda pinta num plano TRANSPARENTE só dele — todas as rotas de carimbo, em `Mix`, com o
//! tecto do Accumulate e tudo o resto como sempre — e cada lote compõe o plano sobre a base congelada no
//! pen-down, com o modo do pincel, UMA vez (`ph2d_painter_brush::traco::TracoComposto::compoe`). As orlas são
//! o aro da aguada sobre a cobertura do plano ([`super::watercolor_rim::aro_da_cobertura`], a mesma porta).
//!
//! **A cauda** (L5): ao levantar, a janela do fim limpa-se no plano e voltam a carimbar-se os LOTES que lhe
//! tocam, inteiros, pela porta do despacho (as corridas da ponta de N imagens) e com o sorteio e o estado do
//! trilho de píxel que tiveram (sem costura); os dabs da cauda afilados pela rampa da cabeça espelhada
//! (`Taper::width_end`) e adensados ao passo da largura nova; fora da janela o plano volta ao que era. Num traço
//! mais curto que as duas janelas elas escalam (a lei do Procreate) e o traço inteiro re-pinta-se. Tudo antes do
//! fecho do traço: UMA transacção.
//!
//! ⚠️ **Quem não arma o plano não muda um byte**: o predicado [`PainterTool::traco_no_plano`] é falso no
//! neutro e fora do Digital (a linha do painel esmaece pela MESMA pergunta, `inercia`).

use super::{PaintMode, Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushBlend, Dab};
use rayon::prelude::*;
use std::sync::Arc;

/// A janela (texels) a partir da qual a composição de um lote vai ao pool (ADR-0173), em blocos de
/// [`LINHAS_POR_TAREFA`]. Medido intercalado (`o_dual_brush::mede_o_custo_do_dual`): a pegada de 40 px (~1,8 k
/// texels) custava 1–2,5 ms por lote só a acordar o pool (0,07 ms em série) e 20–40 ms com um trabalhador
/// desagendado; a de 200 px (~41 k) em série é 1,7 ms e no pool ~0,2 ms (série a 200 px: mediana 0,81 ms/quadro
/// contra 0,24 no pool).
const COMPOE_EM_PARALELO_DESDE: usize = 16_384;
/// Linhas por tarefa no pool.
const LINHAS_POR_TAREFA: usize = 16;

/// A sessão do plano — vive do 1.º lote ao pen-up.
#[derive(Clone, Debug)]
pub(crate) struct PlanoDoTraco {
    /// A tela no 1.º lote do traço (o pré-traço).
    base: Arc<Vec<u8>>,
    /// O traço em Normal sobre o transparente (RGBA reto).
    plano: Arc<Vec<u8>>,
    /// Os LOTES carimbados (só com a cauda armada), cada um com o sorteio com que pintou.
    lotes: Vec<Lote>,
    /// A MÁSCARA da 2.ª ponta do dual brush (L6, [`super::dual`]): o alfa é a união das marcas dela.
    mascara: Option<Arc<Vec<u8>>>,
}

/// Porque o plano não arma (o que a linha do painel diz ao esmaecer).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SemPlano {
    /// O meio não é o Digital: a aguada, o relevo e o fluido têm a sua própria composição.
    ForaDoDigital,
    /// O gesto não é um traço livre que acumula: um método de re-carimbo, o Solid, os fios, a borracha, o
    /// Composite ou um modo que não é Paint.
    ForaDoTraco,
}

impl PainterTool {
    /// Se o pincel pede o plano (o tecto, uma orla ou a cauda).
    fn pede_o_plano(&self) -> bool {
        let b = &self.paint.brush;
        b.traco.is_active() || b.taper.end_active() || b.dual.is_some()
    }

    /// ⭐ **A porta**: porque é que o plano NÃO arma agora (`None` = arma, se o pincel o pede).
    pub(crate) fn plano_recusado(&self) -> Option<SemPlano> {
        if self.paint_media() != crate::PaintMedia::Digital {
            return Some(SemPlano::ForaDoDigital);
        }
        let b = &self.paint.brush;
        let livre = matches!(self.paint.paint_mode, PaintMode::Paint)
            && b.stroke_method.is_incremental()
            && !b.style_solid
            && !b.sews_threads()
            && !self.stroke_erases()
            && !self.composite_active();
        (!livre).then_some(SemPlano::ForaDoTraco)
    }

    /// ⭐ **O traço segue a SUA cobertura?** — o tecto do Accumulate desligado ([`Self::stroke_cover_wanted`])
    /// ou a carga que se esgota (o fall-off da L5): a carga que acaba baixa o TECTO do traço (`cap = cobertura do
    /// dab`), e sem o buffer os dabs empilhariam por cima dela (a Strength 1 não tem tecto). A cap `1` é a
    /// sequência de sempre ao bit (`stroke_cover`), logo armar o buffer é seguro.
    pub(super) fn cobertura_tapada(&self, brush: &ph2d_painter_brush::BrushSpec) -> bool {
        self.stroke_cover_wanted(brush) || (!brush.accumulate && brush.taper.fall_active())
    }

    /// O traço de agora pinta no plano?
    pub(super) fn traco_no_plano(&self) -> bool {
        self.pede_o_plano() && self.plano_recusado().is_none()
    }

    /// O alcance (px) que a orla acrescenta à volta do que mudou: o raio do borrão e um texel.
    fn margem_da_orla(&self) -> u32 {
        let t = &self.paint.brush.traco;
        if t.tem_orla() {
            t.core_r(self.paint.brush.radius_px) as u32 + 1
        } else {
            0
        }
    }

    /// A janela de um lote no plano: a dos dabs mais a orla (o gate projecta-a inteira).
    pub(super) fn regiao_do_lote_no_plano(&mut self, dabs: &[Dab]) -> Option<Region> {
        une(self.dab_batch_region(dabs), self.regiao_do_dual())
            .map(|r| self.alarga(r, self.margem_da_orla()))
    }

    /// ⭐ **Um lote no plano**: carimba-o no plano (todas as rotas, em `Mix`), guarda os dabs para a cauda e
    /// recompõe a janela. Chamado pela porta do carimbo (`stamp_route`) dentro do gate de proteção/seleção.
    pub(super) fn stamp_dabs_no_plano(&mut self, dabs: &[Dab]) {
        let r = self.dab_batch_region(dabs);
        // Marcas da 2.ª ponta sem lote da 1.ª esperam pelo lote seguinte (ou pelo fim do traço): compor só por elas
        // dobrava as composições por quadro (medido, `o_dual_brush::mede_o_custo_do_dual`).
        if r.is_none() && !self.dual_no_fim() {
            return;
        }
        let mut s = self.abre_o_plano();
        // A 2.ª ponta (L6) primeiro: as marcas dela até este ponto do caminho recortam o lote.
        let r_dual = self.carimba_a_segunda(&mut s.mascara);
        if r.is_some() {
            // A cauda (L5) re-carimba lotes INTEIROS com o MESMO sorteio (`tex_rng`) e o mesmo estado do trilho
            // de píxel: o que não é cauda volta igual, sem costura na janela.
            if self.paint.brush.taper.end_active() {
                s.lotes.push(Lote {
                    rng: self.paint.tex_rng,
                    ancorado: self.paint.pixel_trilhos.ja_ancorado,
                    dabs: dabs.to_vec(),
                });
            }
            self.carimba_no_plano(&mut s, dabs);
        }
        let Some(r) = une(r, r_dual) else {
            self.paint.plano_do_traco = Some(s);
            return;
        };
        let r = self.alarga(r, self.margem_da_orla());
        self.compoe_o_plano(&s, r);
        self.paint.plano_do_traco = Some(s);
    }

    /// A sessão deste traço, ou uma nova sobre a tela de agora.
    fn abre_o_plano(&mut self) -> PlanoDoTraco {
        let n4 = self.canvas_rgba.len();
        match self.paint.plano_do_traco.take() {
            Some(s) if s.plano.len() == n4 && s.base.len() == n4 => s,
            _ => PlanoDoTraco {
                base: Arc::clone(&self.canvas_rgba),
                plano: Arc::new(vec![0u8; n4]),
                lotes: Vec::new(),
                mascara: None,
            },
        }
    }

    /// Carimba `dabs` no plano: troca o plano para dentro da tela, `Mix` e sem o trinco de alfa (o modo e o
    /// trinco são da composição), e volta.
    fn carimba_no_plano(&mut self, s: &mut PlanoDoTraco, dabs: &[Dab]) {
        self.dentro_do_plano(s, |t| t.stamp_dabs_dispatch(dabs));
    }

    /// Corre `f` com o plano do traço DENTRO da tela (`Mix`, `acumulando_no_plano`): pela porta do despacho
    /// (`stamp_dabs_dispatch`) as corridas da ponta de N imagens (L19) chegam ao plano, e a porta devolve o lote
    /// às rotas sem voltar ao plano (`acumulando_no_plano` + `traco_no_plano`, `stamp_route`).
    fn dentro_do_plano(&mut self, s: &mut PlanoDoTraco, f: impl FnOnce(&mut Self)) {
        let ws = &self.undo.write_state;
        super::plane_fork::swap_canvas_plane(&mut self.canvas_rgba, &mut s.plano, ws);
        let (blend, plano) = (self.paint.brush.blend, self.paint.acumulando_no_plano);
        self.paint.brush.blend = BrushBlend::Mix;
        self.paint.acumulando_no_plano = true;
        f(self);
        self.paint.brush.blend = blend;
        self.paint.acumulando_no_plano = plano;
        let ws = &self.undo.write_state;
        super::plane_fork::swap_canvas_plane(&mut self.canvas_rgba, &mut s.plano, ws);
    }

    /// `r` alargada `m` px, presa à tela.
    fn alarga(&self, r: Region, m: u32) -> Region {
        let (w, h) = self.source_size;
        let x0 = r.x.saturating_sub(m);
        let y0 = r.y.saturating_sub(m);
        let x1 = (r.x + r.w + m).min(w);
        let y1 = (r.y + r.h + m).min(h);
        Region {
            x: x0,
            y: y0,
            w: x1.saturating_sub(x0),
            h: y1.saturating_sub(y0),
        }
    }

    /// ⭐ **A composição** de `r`: `base ⊕ plano` no modo do pincel, com o tecto e as orlas.
    fn compoe_o_plano(&mut self, s: &PlanoDoTraco, r: Region) {
        if r.w == 0 || r.h == 0 {
            return;
        }
        let brush = self.paint.brush;
        let t = brush.traco;
        let w = self.source_size.0 as usize;
        // O aro lê a cobertura numa janela com contexto (o borrão e a régua do teto), e escreve-se só em `r`.
        let aro = t.tem_orla().then(|| {
            let core = t.core_r(brush.radius_px);
            let jan = self.alarga(r, 3 * core as u32 + 4);
            let (jw, jh) = (jan.w as usize, jan.h as usize);
            let mut cov = vec![0.0f32; jw * jh];
            for y in 0..jh {
                for x in 0..jw {
                    let i = ((jan.y as usize + y) * w + jan.x as usize + x) * 4 + 3;
                    cov[y * jw + x] = f32::from(s.plano[i]) / 255.0;
                }
            }
            (
                jan,
                super::watercolor_rim::aro_da_cobertura(&cov, jw, jh, core),
            )
        });
        let trinco = self
            .layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| l.alpha_locked);
        let buf = super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            Some(r),
        );
        let dec = |v: u8| f32::from(v) / 255.0;
        let enc = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        // O modo, a fórmula geral (L7) e a mistura K–M do pincel: o carimbo de sempre, composto UMA vez.
        let (vazio, pigmento) = (brush.blend_sobre_vazio, brush.effective_pigment_mix());
        // A 2.ª ponta (L6): o modo e a máscara resolvidos UMA vez por lote, não por texel.
        let dual = brush.dual.map(|d| d.modo).zip(s.mascara.as_deref());
        // ADR-0173: por LINHA e em paralelo — cada texel é função pura da base, do plano e do aro dele.
        let linhas = &mut buf[r.y as usize * w * 4..(r.y + r.h) as usize * w * 4];
        let linha_composta = |(dy, linha): (usize, &mut [u8])| {
            let y = r.y as usize + dy;
            for x in r.x as usize..(r.x + r.w) as usize {
                let i = (y * w + x) * 4;
                // Sem tinta do traço (o plano vazio, ou a 2.ª ponta que o recorta a nada) e sem orla que a
                // alcance, o texel é a base: a composição devolve-a ao bit (`compoe` com `a = 0`).
                let sem_aro = aro.is_none();
                if sem_aro && s.plano[i + 3] == 0 {
                    linha[x * 4..x * 4 + 4].copy_from_slice(&s.base[i..i + 4]);
                    continue;
                }
                let base = [0, 1, 2, 3].map(|k| dec(s.base[i + k]));
                let mut p = [0, 1, 2, 3].map(|k| dec(s.plano[i + k]));
                if trinco {
                    p[3] *= base[3];
                }
                if let Some((modo, m)) = dual {
                    p[3] = modo.recorta(p[3], dec(m[i + 3]));
                    if sem_aro && p[3] <= 0.0 {
                        linha[x * 4..x * 4 + 4].copy_from_slice(&s.base[i..i + 4]);
                        continue;
                    }
                }
                let a = aro.as_ref().map_or(0.0, |(jan, v)| {
                    v[(y - jan.y as usize) * jan.w as usize + (x - jan.x as usize)]
                });
                let mut out = t.compoe(brush.blend, vazio, pigmento, base, p, a);
                if trinco {
                    out[3] = base[3];
                }
                for k in 0..4 {
                    linha[x * 4 + k] = enc(out[k]);
                }
            }
        };
        // Uma pegada pequena compõe-se em SÉRIE: acordar o pool por ela custava 1–2 ms por lote e, com um
        // trabalhador desagendado, 20–40 ms (medido, `o_dual_brush::mede_o_custo_do_dual`, a 40 px).
        if (r.w as usize) * (r.h as usize) < COMPOE_EM_PARALELO_DESDE {
            linhas
                .chunks_mut(w * 4)
                .enumerate()
                .for_each(linha_composta);
        } else {
            // Em blocos de linhas: uma tarefa por LINHA eram ~200 encontros por lote a 200 px.
            linhas
                .par_chunks_mut(w * 4 * LINHAS_POR_TAREFA)
                .enumerate()
                .for_each(|(b, bloco)| {
                    for (k, l) in bloco.chunks_mut(w * 4).enumerate() {
                        linha_composta((b * LINHAS_POR_TAREFA + k, l));
                    }
                });
        }
        self.declare_wrote(Some(r));
        self.mark_dirty(r);
    }

    /// ⭐ **O pen-up do plano**: a cauda (L5) re-pintada, e a sessão fecha. Antes do fecho do traço, e pelo
    /// MESMO gate de proteção/seleção que os lotes atravessam.
    pub(super) fn fecha_o_plano_do_traco(&mut self) {
        let Some(mut s) = self.paint.plano_do_traco.take() else {
            return;
        };
        if !self.paint.brush.taper.end_active() {
            return;
        }
        let Some(cauda) = self.planeia_a_cauda(&s) else {
            return;
        };
        let r = self.alarga(cauda.tocados, self.margem_da_orla());
        let mut corpo = move |t: &mut Self| {
            t.repinta_a_cauda(&mut s, &cauda);
            t.compoe_o_plano(&s, r);
        };
        let (mask, sel) = (
            self.mask_protection_active(),
            self.selection_restricts_paint(),
        );
        if mask || sel {
            self.gate_scoped(r, mask, sel, corpo);
        } else {
            corpo(self);
        }
    }

    /// O plano da cauda: os lotes a re-carimbar, a janela que se limpa e a que muda.
    ///
    /// ⭐ **A lei do Procreate para o traço CURTO**: a cabeça e a cauda escalam com o comprimento (`S' + E' ≤ L`).
    /// Num traço mais curto que as duas janelas (um toque: `L = 0`) o traço INTEIRO re-pinta-se com as janelas
    /// escaladas — o toque fica um ponto cheio, e a cabeça não afina por cima da cauda.
    fn planeia_a_cauda(&self, s: &PlanoDoTraco) -> Option<Cauda> {
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
                super::tiling::tiled_dabs(v, self.source_size, self.paint.tiling)
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
        // Os lotes que tocam a janela, INTEIROS e pela ordem, com a cauda re-afilada no lugar.
        let mut lotes: Vec<Lote> = s
            .lotes
            .iter()
            .filter(|l| ladrilhada(&l.dabs).iter().any(|c| toca(c, limpa, reach)))
            .map(|l| Lote {
                dabs: l
                    .dabs
                    .iter()
                    .map(|x| if da_cauda(x) { janela.afila(*x) } else { *x })
                    .collect(),
                ..l.clone()
            })
            .collect();
        // Os dabs que adensam a ponta fina — novos, num lote no fim (só num caminho único: sem simetria, spray
        // nem ponta de píxel, onde dois dabs seguidos não são do mesmo caminho).
        let b = &self.paint.brush;
        if !b.symmetry.enabled && b.spray_count <= 1 && !self.pixel_vivo(b) {
            let extra = adensa(&cauda_orig, &janela, self.passo_px());
            if !extra.is_empty() {
                lotes.push(Lote {
                    rng: self.paint.tex_rng,
                    ancorado: false,
                    dabs: extra,
                });
            }
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
    fn repinta_a_cauda(&mut self, s: &mut PlanoDoTraco, c: &Cauda) {
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
        self.dentro_do_plano(s, |t| {
            for l in &c.lotes {
                t.paint.tex_rng = l.rng;
                t.paint.pixel_trilhos.ja_ancorado = l.ancorado;
                t.stamp_dabs_dispatch(&l.dabs);
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

    /// O passo do traço em px (o Spacing do pincel no diâmetro cheio).
    fn passo_px(&self) -> f32 {
        let b = &self.paint.brush;
        (b.spacing * 2.0 * b.clamped_radius()).max(0.5)
    }
}

/// O plano da cauda ([`PainterTool::planeia_a_cauda`]).
struct Cauda {
    /// Os lotes a re-carimbar: os que tocam a janela (a cauda afilada no lugar) e o adensar da ponta.
    lotes: Vec<Lote>,
    /// A janela que se limpa (a pegada da cauda original).
    limpa: Region,
    /// A janela que muda (a pegada do lote, que transborda da limpa).
    tocados: Region,
}

/// A união de duas janelas que podem faltar.
fn une(a: Option<Region>, b: Option<Region>) -> Option<Region> {
    match (a, b) {
        (Some(a), Some(b)) => Some(union_region(a, b)),
        (a, b) => a.or(b),
    }
}

/// O dab `d` toca a região `r`?
fn toca(d: &Dab, r: Region, reach: f32) -> bool {
    let e = d.radius_px * reach + 1.0;
    d.center[0] + e >= r.x as f32
        && d.center[0] - e <= (r.x + r.w) as f32
        && d.center[1] + e >= r.y as f32
        && d.center[1] - e <= (r.y + r.h) as f32
}

/// Um lote do traço como foi carimbado: o sorteio (`tex_rng`) e o estado do trilho de píxel ANTES dele.
#[derive(Clone, Debug)]
struct Lote {
    rng: u64,
    ancorado: bool,
    dabs: Vec<Dab>,
}

/// As janelas da cabeça e da cauda no fim do traço (já escaladas pela lei do traço curto).
struct Janelas {
    taper: ph2d_painter_brush::taper::Taper,
    d: f32,
    total: f32,
    cabeca: f32,
    fim: f32,
    curto: bool,
}

impl Janelas {
    /// A largura a `arc`: a rampa da cauda e, no traço curto, também a da cabeça re-escalada.
    fn largura(&self, arc: f32) -> f32 {
        let rampa = |dist: f32, len: f32, ponta: f32| {
            let t = ph2d_painter_brush::taper::Taper {
                start: if self.d > 0.0 { len / self.d } else { 0.0 },
                tip_start: ponta,
                ..self.taper
            };
            t.width(dist, self.d)
        };
        let cauda = rampa(self.total - arc, self.fim, self.taper.tip_end);
        if self.curto {
            cauda * rampa(arc, self.cabeca, self.taper.tip_start)
        } else {
            cauda
        }
    }

    /// O dab re-afilado — e o estado do dab ANTERIOR que a cápsula dele interpola (`Dab::ante`), pela MESMA lei no
    /// arco dele: sem isto o sub-dab de trás voltava ao raio cheio e a cauda acabava numa bolha (J3, canetas 2.ª onda).
    fn afila(&self, x: Dab) -> Dab {
        let mut y = self.afila_so(x);
        if let Some(mut a) = x.ante {
            let antes = self.afila_so(Dab {
                radius_px: a.radius_px,
                coverage: a.coverage,
                arc_len: a.arc_len,
                ante: None,
                ..x
            });
            (a.radius_px, a.coverage) = (antes.radius_px, antes.coverage);
            y.ante = Some(a);
        }
        y
    }

    /// O dab re-afilado: no traço curto desfaz-se primeiro a cabeça que o motor aplicou ao vivo.
    fn afila_so(&self, mut x: Dab) -> Dab {
        if self.curto && self.taper.is_active() {
            let w = self.taper.width(x.arc_len, self.d);
            let m = self.taper.coverage_mult(w);
            x.radius_px = if w > 1e-3 && x.radius_px > 0.5 + 1e-4 {
                x.radius_px / w
            } else {
                x.stroke_radius_px
            };
            if m > 1e-3 {
                x.coverage = (x.coverage / m).clamp(0.0, 1.0);
            }
        }
        let w = self.largura(x.arc_len);
        ph2d_painter_brush::taper::scale_dab(&mut x, &self.taper, w);
        x
    }
}

/// Os dabs que ADENSAM a ponta fina: entre dois dabs seguidos do mesmo caminho entram dabs interpolados ao passo
/// da largura nova, para a ponta não sair em contas (os originais já estão nos lotes).
fn adensa(orig: &[Dab], j: &Janelas, passo: f32) -> Vec<Dab> {
    let mut out = Vec::new();
    for (a, b) in orig.iter().zip(orig.iter().skip(1)) {
        let dist =
            ((b.center[0] - a.center[0]).powi(2) + (b.center[1] - a.center[1]).powi(2)).sqrt();
        if dist > 1.5 * passo || dist <= 0.0 {
            continue;
        }
        let fino = (j.largura(a.arc_len).min(j.largura(b.arc_len)) * passo).max(0.5);
        let n = (dist / fino).ceil() as usize;
        for k in 1..n {
            let t = k as f32 / n as f32;
            let mut m = *a;
            m.center = [
                a.center[0] + (b.center[0] - a.center[0]) * t,
                a.center[1] + (b.center[1] - a.center[1]) * t,
            ];
            m.arc_len = a.arc_len + (b.arc_len - a.arc_len) * t;
            m.radius_px = a.radius_px + (b.radius_px - a.radius_px) * t;
            m.coverage = a.coverage + (b.coverage - a.coverage) * t;
            // Um dab SOLTO: a cápsula do `a` (o passo e o estado de trás dele) não é a deste ponto do meio.
            (m.step, m.ante) = ([0.0, 0.0], None);
            out.push(j.afila(m));
        }
    }
    out
}

/// Zera `r` num plano de `canais` canais por texel e `w` de largura.
fn limpa_regiao(p: &mut [u8], w: usize, r: Region, canais: usize) {
    for y in r.y as usize..(r.y + r.h) as usize {
        let a = (y * w + r.x as usize) * canais;
        p[a..a + r.w as usize * canais].fill(0);
    }
}

/// Repõe, em `tocados` menos `limpa`, os texels de `antes`.
fn repoe_fora(p: &mut [u8], antes: &[u8], w: usize, tocados: Region, limpa: Region, canais: usize) {
    for y in tocados.y as usize..(tocados.y + tocados.h) as usize {
        let dentro_y = y >= limpa.y as usize && y < (limpa.y + limpa.h) as usize;
        for x in tocados.x as usize..(tocados.x + tocados.w) as usize {
            if dentro_y && x >= limpa.x as usize && x < (limpa.x + limpa.w) as usize {
                continue;
            }
            let i = (y * w + x) * canais;
            p[i..i + canais].copy_from_slice(&antes[i..i + canais]);
        }
    }
}
