//! **O DEPÓSITO de um dab de tinta — a lei da célula, e o caminho do produto por LINHAS**
//! ([ADR-0175](../../../../docs/architecture/decisions/0175-o-deposito-do-dab-do-wet-paint-corre-em-linhas-disjuntas.md)).
//!
//! Filho de [`super`] por responsabilidade: o pai guarda *o que a janela do rasto É* (a âncora, o
//! crescimento, o transfer), aqui fica *o que um dab POUSA nela e no grid*.
//!
//! ## Porquê existe (medido 2026-09-24, `examples/mede_o_wet_paint`, tela 4096², release)
//!
//! Com a caneta em baixo a simulação está PARADA, e o quadro de pintar é quase só isto: com raio
//! 250 cada quadro pousa ~5 dabs e **cada dab custava `~5,7 ms`** (`~251 000` células), ou seja
//! `28` dos `31 ms` do quadro — o artista sentia `~33` FPS a pintar e `~22` com raio 400. E corria
//! numa thread só, com o pool do motor parado ao lado.
//!
//! ## Porquê é byte-idêntico (as três condições do [`crate::par`])
//!
//! 1. **cada linha escreve SÓ a própria linha** — o carimbo de umidade `wet[i]` na linha do grid e
//!    o pigmento/água na linha da janela do rasto; cada célula é visitada UMA vez por dab;
//! 2. **o que ela lê, ninguém escreve neste passe** — `susp`, `sett`, `paper` e `film` são só
//!    lidos, a silhueta e o grão são funções puras da célula ([`crate::brush::CellFn`]);
//! 3. **a única redução é de RETÂNGULOS** — a extensão da janela e o `wrote` são caixas envolventes,
//!    união de mínimos e máximos inteiros: associativa **e** comutativa.
//!
//! ⚠️ **E a lei da célula é UMA função** ([`DepositLaw::cell`]) com dois chamadores — o caminho do
//! produto por linhas e o do próprio motor em série ([`super::Trail::accumulate_paint`], que a
//! impressão digital da sessão fixa ao bit) —; as rotas só escolhem o caminhante, nunca a
//! aritmética. E o caminho do motor é o ORÁCULO independente da caminhada por linhas
//! (`tests/it/deposit_rows.rs`): com uma silhueta que reproduz a queda dele, os dois pousam o
//! mesmo traço ao bit.

use super::{Accumulated, TouchedRect, Trail, touch_ext};
use crate::brush::{CellFn, ColorFn, shaped_bounds, stamp_row_shaped};
use crate::colorops::ColorMix;
use crate::grid::{Grid, wet_byte_from_paper};
use crate::par::{self, Rows};
use crate::sim::Params;
use crate::trail::Dab;
use crate::tuning::Knob;

/// Com o pó a 1, a fracção das gotas que FALTAM num dab (uma em 20 cai).
const POEIRA_MAXIMA: f64 = 0.95;


/// A caixa vazia (`x1 < x0`), a sentinela que o [`touch_ext`] já usa.
const VAZIA: (i32, i32, i32, i32) = (0, 0, -1, -1);

/// A união de duas caixas envolventes, com [`VAZIA`] como identidade — a redução das linhas.
fn uniao(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> (i32, i32, i32, i32) {
    if a.2 < a.0 {
        return b;
    }
    if b.2 < b.0 {
        return a;
    }
    (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
}

/// **O pó de um dab** ([`Knob::Dust`]): cada GOTA (um quadrado de [`Knob::DustDrop`] células, a grelha com
/// a fase deste dab) cai com a probabilidade `p` e leva a dose `÷ p` — a média do carimbo é a de sempre.
#[derive(Clone, Copy)]
struct Po {
    p: f64,
    semente: u32,
    fase: [f64; 2],
    gota: f64,
}

impl Po {
    /// A semente é a do hospedeiro e o CENTRO do dab (os bits do `f64` dobrados): sem hospedeiro, dois
    /// dabs em lugares diferentes já pousam gotas diferentes.
    fn de(po: f64, gota: f64, semente: u32, dab: &Dab) -> Option<Self> {
        if po <= 0.0 {
            return None;
        }
        let dobra = |v: f64| {
            let b = v.to_bits();
            (b ^ (b >> 32)) as i32
        };
        let semente =
            (crate::rng::hash2(dobra(dab.x), dobra(dab.y), semente) * 4_294_967_296.0) as u32;
        let fase = [1, 3].map(|k| crate::rng::hash2(k, k + 1, semente) * gota);
        Some(Self {
            p: 1.0 - POEIRA_MAXIMA * po.min(1.0),
            semente,
            fase,
            gota,
        })
    }

    /// A gota que cobre a célula `(x, y)` cai neste dab?
    #[inline]
    fn cai(&self, x: i32, y: i32) -> bool {
        let g = |c: i32, f: f64| ((f64::from(c) + f) / self.gota).floor() as i32;
        crate::rng::hash2(g(x, self.fase[0]), g(y, self.fase[1]), self.semente) < self.p
    }
}

/// Os números de um dab que a lei da célula lê — tirados UMA vez por dab, fora do laço.
#[derive(Clone, Copy)]
pub(super) struct DepositLaw {
    gain: f64,
    gate: f64,
    pig_cap: f64,
    ext_dry_brush: f64,
    ext_wet_soften: f64,
    ext_bypass: bool,
    intensity: f64,
    dry_gate: f64,
    pub(super) water_amount: f64,
    /// A mistura da tinta, para a cor POR CÉLULA que dois dabs da mesma janela pousam na ponta.
    mix: ColorMix,
    /// O pó deste dab — `None` com o [`Knob::Dust`] a 0 (o carimbo contínuo, ao byte).
    po: Option<Po>,
    /// [`Dab::opacity`]: multiplica o que pousa, como o `sv`.
    opacidade: f64,
    /// [`Knob::PressureGrain`]: a parte da [`Self::opacidade`] que entra antes do portão (`0` = nenhuma).
    grao_da_pressao: f64,
}

impl DepositLaw {
    pub(super) fn new(p: &Params, dab: &Dab, ext_bypass: bool) -> Self {
        Self {
            gain: p.k(Knob::PigmentPerDab),
            gate: p.k(Knob::PaperGate),
            pig_cap: p.k(Knob::GateSaturation),
            ext_dry_brush: p.k(Knob::ExtDryBrush),
            ext_wet_soften: p.k(Knob::ExtWetSoften),
            ext_bypass,
            intensity: dab.intensity,
            dry_gate: dab.dry_gate,
            water_amount: dab.water_amount,
            mix: p.mix,
            po: Po::de(p.k(Knob::Dust), p.k(Knob::DustDrop), p.semente_do_po, dab),
            opacidade: dab.opacity,
            grao_da_pressao: p.k(Knob::PressureGrain),
        }
    }

    pub(super) fn gain(&self) -> f64 {
        self.gain
    }

    /// **A lei de UMA célula**: `Some((depósito, byte de umidade))`, ou `None` quando nada pousa —
    /// e então nem a umidade é carimbada.
    ///
    /// `sv` é o padrão procedural do Shape (plano 27 W3, **lei B**): multiplica o que PASSOU o portão,
    /// nunca entra no `stamp` que o dente corta — um padrão de média baixa (o Noise, ~0,4) sobre o grão
    /// dos pastéis pousava 0,000 do redondo, porque `env·sv·grão·I` nunca vencia o portão. `1` = o
    /// redondo ao bit (medido: [27 §6.2](../../../../docs/Painter/NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md)).
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub(super) fn cell(
        &self,
        susp: f32,
        sett: f32,
        paper: f32,
        film: f32,
        fall: f64,
        texv: f64,
        sv: f64,
        x: i32,
        y: i32,
    ) -> Option<(f64, u8)> {
        let mut stamp = fall * texv * self.intensity;
        if stamp > 1.0 {
            stamp = 1.0;
        }
        // ⭐ A pressão leve desfaz em grão: a parte `k` da opacidade entra antes do portão (`opacidade^k`), o resto
        // (`opacidade^(1-k)`) depois — `k = 0` é a lei de sempre, ao bit.
        let opacidade_depois = if self.grao_da_pressao > 0.0 {
            stamp *= self.opacidade.powf(self.grao_da_pressao);
            self.opacidade.powf(1.0 - self.grao_da_pressao)
        } else {
            self.opacidade
        };
        if stamp <= 0.0 {
            return None;
        }
        if let Some(po) = &self.po {
            // ⭐ O PÓ: a gota que não cai não pousa nada (nem a umidade); a que cai leva a dose `÷ p`.
            if !po.cai(x, y) {
                return None;
            }
            stamp /= po.p;
        }
        // Paper gate: tooth peaks always take pigment, valleys reject
        // it — that per-pixel pass/reject IS the granulation. Heavily
        // loaded cells read a flat 0.45 tooth (the grain is buried).
        let tooth = if (susp as f64 + sett as f64) < self.pig_cap {
            paper as f64
        } else {
            0.45
        };
        let mut deposit = stamp - (1.0 - tooth) * self.gate;
        if !self.ext_bypass {
            // Dry-brush extension: raise the gate subtraction.
            deposit -= self.dry_gate * self.ext_dry_brush * 0.6;
        }
        if deposit <= 0.0 {
            return None;
        }
        deposit *= sv;
        deposit *= opacidade_depois;
        if !self.ext_bypass {
            // Wet-edge softening extension: thin the rim on wet paper.
            let softness = (film as f64 / 3.0).min(1.0) * self.ext_wet_soften;
            deposit *= 1.0 - softness * (1.0 - fall);
        }
        // Wetness seed: OVERWRITE (not max) — repainting can dry the
        // byte back down.
        Some((deposit, wet_byte_from_paper(tooth)))
    }
}

/// A linha da janela do rasto: o pigmento, a água e — com a cor por célula — os três planos da tinta.
struct LinhaDaJanela<'a> {
    pig: &'a mut [f32],
    water: &'a mut [f32],
    tinta: Option<[&'a mut [f32]; 3]>,
}

/// Uma linha do depósito: o `y`, a linha do grid (`wet`) e — se a janela a cobre — a linha dela.
type Linha<'a> = (i32, &'a mut [u8], Option<LinhaDaJanela<'a>>, Option<&'a mut [f32]>);

/// **A cor de um dab numa célula da janela**: a primeira tinta na célula pousa a cor dele inteira; as
/// seguintes misturam-se pela massa (`soma ÷ (antes + soma)`), como duas tintas no pincel.
#[inline]
fn pousa_a_tinta(
    tinta: &mut [&mut [f32]; 3],
    l: usize,
    antes: f64,
    soma: f64,
    c: [f32; 4],
    mix: ColorMix,
) {
    if antes <= 0.0 {
        for k in 0..3 {
            tinta[k][l] = c[k];
        }
        return;
    }
    let mut out = [0.0f64; 3];
    let [r, g, b] = [0, 1, 2].map(|k| f64::from(tinta[k][l]));
    let w = soma / (antes + soma);
    mix.mix(
        r,
        g,
        b,
        f64::from(c[0]),
        f64::from(c[1]),
        f64::from(c[2]),
        w,
        &mut out,
    );
    for k in 0..3 {
        tinta[k][l] = out[k] as f32;
    }
}

impl Trail {
    /// O depósito de um dab com a silhueta do HOSPEDEIRO, caminhado por linhas na rota `mode`.
    ///
    /// Devolve o que a porta pública devolvia (a janela encheu? · onde o dab escreveu no grid) e
    /// actualiza a extensão da janela. `mode = None` é o produto: a rota sai de [`Rows::pick`]
    /// com o piso MEDIDO [`par::MIN_CELLS_DEPOSIT`]. `filme` = a espessura do filme do traço em dabs
    /// ([`Knob::StrokeFilm`], [`super::filme`]): cada célula recebe só o que passa do máximo que o traço já lhe deu.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn deposit_shaped(
        &mut self,
        g: &mut Grid,
        tex: &[f32],
        dab: &Dab,
        law: DepositLaw,
        sil: CellFn<'_>,
        grain: Option<CellFn<'_>>,
        padrao: Option<CellFn<'_>>,
        cor: Option<ColorFn<'_>>,
        mode: Option<Rows>,
        filme: Option<f64>,
    ) -> (i32, i32, i32, i32) {
        let Some((x0, y0, x1, y1)) = shaped_bounds(g.w, g.h, dab.x, dab.y, dab.r * dab.reach)
        else {
            return VAZIA;
        };
        let mode = mode.unwrap_or_else(|| {
            Rows::pick(
                (y1 - y0 + 1) as usize,
                (x1 - x0 + 1) as usize,
                par::MIN_CELLS_DEPOSIT,
            )
        });
        let (s, size, half) = (g.s, self.size, self.half);
        let (ax, ay) = (self.anchor_x, self.anchor_y);
        let gain = law.gain();
        let Grid {
            susp,
            sett,
            paper,
            film,
            wet,
            ..
        } = g;
        let (susp, sett, paper, film) = (&*susp, &*sett, &*paper, &*film);
        // As linhas da janela que caem dentro da caixa do dab — cada uma é a MESMA linha `y` do
        // grid, com a âncora da janela a ligá-las.
        let n_linhas = (y1 - y0 + 1) as usize;
        let mut janela: Vec<Option<LinhaDaJanela<'_>>> = (0..n_linhas).map(|_| None).collect();
        let n = size as usize;
        let mut tinta: Vec<Option<[&mut [f32]; 3]>> = (0..n_linhas).map(|_| None).collect();
        if cor.is_some() {
            for (ly, ((r, g), b)) in self
                .tinta_r
                .chunks_mut(n)
                .zip(self.tinta_g.chunks_mut(n))
                .zip(self.tinta_b.chunks_mut(n))
                .enumerate()
            {
                let y = ly as i32 - half + ay;
                if (y0..=y1).contains(&y) {
                    tinta[(y - y0) as usize] = Some([r, g, b]);
                }
            }
        }
        for (ly, (pig, water)) in self
            .pig
            .chunks_mut(n)
            .zip(self.water.chunks_mut(n))
            .enumerate()
        {
            let y = ly as i32 - half + ay;
            if (y0..=y1).contains(&y) {
                let k = (y - y0) as usize;
                janela[k] = Some(LinhaDaJanela {
                    pig,
                    water,
                    tinta: tinta[k].take(),
                });
            }
        }
        let faixa = &mut wet[y0 as usize * s..(y1 as usize + 1) * s];
        let mut do_filme: Vec<Option<&mut [f32]>> = match filme {
            Some(_) => self.filme.linhas((x0, y0, x1, y1)).map(Some).collect(),
            None => (0..n_linhas).map(|_| None).collect(),
        };
        let mut linhas: Vec<Linha<'_>> = faixa
            .chunks_mut(s)
            .zip(janela)
            .enumerate()
            .map(|(k, (w, j))| (y0 + k as i32, w, j, do_filme[k].take()))
            .collect();
        let linha = |it: &mut Linha<'_>| -> ((i32, i32, i32, i32), (i32, i32, i32, i32)) {
            let (y, wet_row, win, filme_row) = it;
            let y = *y;
            let base = y as usize * s;
            let ly = y - ay + half;
            let mut ext = VAZIA;
            let mut wrote = VAZIA;
            stamp_row_shaped(
                y,
                x0,
                x1,
                tex,
                dab.x,
                dab.y,
                sil,
                grain,
                padrao,
                |x, fall, texv, sv| {
                    let i = base + x as usize;
                    let Some((mut deposit, wb)) =
                        law.cell(susp[i], sett[i], paper[i], film[i], fall, texv, sv, x, y)
                    else {
                        return;
                    };
                    let tinta = cor.map(|f| f(x, y, fall * sv));
                    if let Some(c) = tinta {
                        deposit *= f64::from(c[3]);
                    }
                    if let (Some(k), Some(f)) = (filme, filme_row.as_deref_mut()) {
                        deposit = super::filme::acrescenta(&mut f[x as usize], deposit * k);
                    }
                    // ⚠️ **Esta é uma escrita no DOCUMENTO** — ver o doc do pai: é dela que o véu do
                    // Show Wet se alimenta, e acontece mesmo fora da janela do rasto.
                    wet_row[x as usize] = wb;
                    touch_ext(&mut wrote, x, y);
                    let lx = x - ax + half;
                    if !(0..size).contains(&lx) {
                        return;
                    }
                    let Some(jl) = win.as_mut() else {
                        return;
                    };
                    let l = lx as usize;
                    let antes = jl.pig[l] as f64;
                    if let (Some(c), Some(t)) = (tinta.filter(|_| deposit > 0.0), jl.tinta.as_mut())
                    {
                        pousa_a_tinta(t, l, antes, deposit * gain, c, law.mix);
                    }
                    jl.pig[l] = (antes + deposit * gain) as f32;
                    jl.water[l] = (jl.water[l] as f64 + deposit * law.water_amount) as f32;
                    touch_ext(&mut ext, lx, ly);
                },
            );
            (ext, wrote)
        };
        let (ext, wrote) =
            par::walk_items_reduce(mode, &mut linhas, (VAZIA, VAZIA), linha, |a, b| {
                (uniao(a.0, b.0), uniao(a.1, b.1))
            });
        let janela_antes = (self.lx0, self.ly0, self.lx1, self.ly1);
        (self.lx0, self.ly0, self.lx1, self.ly1) = uniao(janela_antes, ext);
        wrote
    }

    /// Fecha um dab: conta-o e diz ao chamador se a janela encheu e onde o grid foi escrito.
    pub(super) fn accumulated(&mut self, wrote: (i32, i32, i32, i32)) -> Accumulated {
        self.dab_count += 1;
        Accumulated {
            window_full: self.dab_count > self.window_size,
            wrote: (wrote.2 >= wrote.0).then_some(TouchedRect {
                x0: wrote.0,
                y0: wrote.1,
                x1: wrote.2,
                y1: wrote.3,
            }),
        }
    }
}
