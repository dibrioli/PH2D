//! As **fotografias do histórico** da grelha (SPEC §15) — filho de [`super`] por LOC.

use super::*;

#[path = "snapshot/registo_bytes.rs"]
mod registo_bytes;

// ---------------------------------------------------------------------------
// History snapshots (SPEC §15). The transient flow arrays are scratch rebuilt
// every frame from the persistent field, so they are not captured.
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct GridSnapshot {
    pub film: Vec<f32>,
    pub susp: Vec<f32>,
    pub susp_rgb: Vec<[f32; 3]>,
    pub sett: Vec<f32>,
    pub sett_rgb: Vec<[f32; 3]>,
    pub relevo: Vec<f32>,
    pub altura: Vec<f32>,
    pub espessura: Vec<f32>,
    pub crista: Vec<f32>,
    pub nivelado: Vec<f32>,
    pub vela: Vec<f32>,
    pub material: Vec<crate::material::MaterialCelula>,
    pub aparencia: Vec<aparencia::AparenciaCelula>,
    pub oleo: Vec<oleo_celula::OleoCelula>,
    /// [`Grid::tinta`] (F4-2 E3): vazia com a lei velha.
    pub tinta: Vec<tinta::CelulaTinta>,
    /// [`Grid::pigmento`] (E10c): vazio sem Pigment mix.
    pub pigmento: Vec<tinta::pigmento::LadoEspectral>,
    pub vel_x: Vec<f32>,
    pub vel_y: Vec<f32>,
    pub wet: Vec<u8>,
    pub active: Vec<u8>,
    pub bloom: Vec<u8>,
    pub bx0: i32,
    pub by0: i32,
    pub bx1: i32,
    pub by1: i32,
    pub has_fluid: bool,
    /// A faixa viva ([`Grid::row_lo`]) viaja no snapshot porque ela É estado
    /// do grid, não uma vista dele.
    ///
    /// ⚠️ A alternativa — abrir a faixa inteira no restore e deixar o próximo
    /// rebuild reapertá-la — foi MEDIDA e reprovada: a varredura viva do
    /// rebuild passa a cobrir a folha toda, **0,3 → 17,4 ms a 4096²**, ou
    /// seja um quadro perdido a cada Ctrl+Z do motor. Um snapshot que já
    /// carrega dez planos não fica mais caro por dois vetores de `i32`.
    pub row_lo: Vec<i32>,
    pub row_hi: Vec<i32>,
    pub paper_preset: PaperPreset,
    pub paper_sheet: u32,
}

pub fn snapshot_grid(g: &Grid) -> GridSnapshot {
    GridSnapshot {
        film: g.film.clone(),
        susp: g.susp.clone(),
        susp_rgb: g.susp_rgb.clone(),
        sett: g.sett.clone(),
        sett_rgb: g.sett_rgb.clone(),
        relevo: g.relevo.clone(),
        altura: g.altura.clone(),
        espessura: g.espessura.clone(),
        crista: g.crista.clone(),
        nivelado: g.nivelado.clone(),
        vela: g.vela.clone(),
        material: g.material.clone(),
        aparencia: g.aparencia.clone(),
        oleo: g.oleo.clone(),
        tinta: g.tinta.clone(),
        pigmento: g.pigmento.clone(),
        vel_x: g.vel_x.clone(),
        vel_y: g.vel_y.clone(),
        wet: g.wet.clone(),
        active: g.active.clone(),
        bloom: g.bloom.clone(),
        bx0: g.bx0,
        by0: g.by0,
        bx1: g.bx1,
        by1: g.by1,
        has_fluid: g.has_fluid,
        row_lo: g.row_lo.clone(),
        row_hi: g.row_hi.clone(),
        paper_preset: g.paper_preset,
        paper_sheet: g.paper_sheet,
    }
}

/// Restore a snapshot. Returns true when the paper identity changed (the
/// caller re-bakes the sheet).
pub fn restore_grid(g: &mut Grid, s: &GridSnapshot) -> bool {
    g.film.copy_from_slice(&s.film);
    g.susp.copy_from_slice(&s.susp);
    g.susp_rgb.copy_from_slice(&s.susp_rgb);
    g.sett.copy_from_slice(&s.sett);
    g.sett_rgb.copy_from_slice(&s.sett_rgb);
    g.relevo.copy_from_slice(&s.relevo);
    g.altura.clone_from(&s.altura);
    g.espessura.clone_from(&s.espessura);
    g.crista.clone_from(&s.crista);
    g.nivelado.clone_from(&s.nivelado);
    g.vela.copy_from_slice(&s.vela);
    g.material.clone_from(&s.material);
    g.aparencia.clone_from(&s.aparencia);
    super::aparencia::reconta_teto(g);
    g.oleo.clone_from(&s.oleo);
    super::oleo_celula::reconta_real(g);
    g.tinta.clone_from(&s.tinta);
    g.pigmento.clone_from(&s.pigmento);
    g.vel_x.copy_from_slice(&s.vel_x);
    g.vel_y.copy_from_slice(&s.vel_y);
    g.flow_x.fill(0.0);
    g.flow_y.fill(0.0);
    g.wet.copy_from_slice(&s.wet);
    g.active.copy_from_slice(&s.active);
    g.bloom.copy_from_slice(&s.bloom);
    g.bx0 = s.bx0;
    g.by0 = s.by0;
    g.bx1 = s.bx1;
    g.by1 = s.by1;
    g.has_fluid = s.has_fluid;
    // A faixa viajou junto: um snapshot descreve o grid INTEIRO, e derivá-la
    // de volta seria uma segunda resposta à pergunta que o rebuild responde.
    g.row_lo.copy_from_slice(&s.row_lo);
    g.row_hi.copy_from_slice(&s.row_hi);
    let paper_changed = g.paper_preset != s.paper_preset || g.paper_sheet != s.paper_sheet;
    g.paper_preset = s.paper_preset;
    g.paper_sheet = s.paper_sheet;
    paper_changed
}

// ---------------------------------------------------------------------------
// O REGISTO do histórico por TILES copy-on-write (perf 31/10).
//
// Lei: um registo descreve o grid no instante em que abriu (`base`) guardando SÓ os tiles de
// `TILE`×`TILE` células que alguém escreveu desde então — cada tile é copiado ANTES da primeira
// escrita (o motor chama `marca` no topo de cada porta que escreve, nunca dentro dos laços). Fora dos
// tiles guardados o grid actual É a base, então restaurar os tiles + os escalares devolve a base ao
// byte, incluindo a tinta que a água levou para longe do pincel DEPOIS do pointer_up (o passo do sim
// também marca). Uma foto inteira por traço eram 56 B/px × 6.
// ---------------------------------------------------------------------------

/// Lado do tile do registo, em células (finas e de fluxo).
pub const TILE: usize = 64;

#[derive(Clone, Copy)]
struct Ret {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
}

fn ret_do_tile(k: u32, s: usize, rows: usize) -> Ret {
    let tw = s.div_ceil(TILE);
    let (tx, ty) = (k as usize % tw, k as usize / tw);
    let (x0, y0) = (tx * TILE, ty * TILE);
    Ret {
        x0,
        y0,
        x1: (x0 + TILE).min(s),
        y1: (y0 + TILE).min(rows),
    }
}

fn tira<T: Clone>(v: &[T], s: usize, r: Ret) -> Vec<T> {
    if v.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity((r.x1 - r.x0) * (r.y1 - r.y0));
    for y in r.y0..r.y1 {
        out.extend_from_slice(&v[y * s + r.x0..y * s + r.x1]);
    }
    out
}

fn poe<T: Clone>(v: &mut [T], s: usize, r: Ret, d: &[T]) {
    if v.is_empty() || d.is_empty() {
        return;
    }
    let w = r.x1 - r.x0;
    for (i, y) in (r.y0..r.y1).enumerate() {
        v[y * s + r.x0..y * s + r.x1].clone_from_slice(&d[i * w..(i + 1) * w]);
    }
}

macro_rules! tile_fino {
    ($($f:ident : $t:ty),* $(,)?) => {
        #[derive(Clone, Default)]
        struct TileFino { $($f: Vec<$t>,)* }
        impl TileFino {
            fn tira(g: &Grid, r: Ret) -> Self { TileFino { $($f: tira(&g.$f, g.s, r),)* } }
            fn poe(&self, g: &mut Grid, r: Ret) { $(poe(&mut g.$f, g.s, r, &self.$f);)* }
            fn bytes(&self) -> usize { 0 $(+ std::mem::size_of_val(self.$f.as_slice()))* }
        }
    };
}

tile_fino!(
    film: f32, susp: f32, susp_rgb: [f32; 3], sett: f32, sett_rgb: [f32; 3], relevo: f32,
    altura: f32, espessura: f32, crista: f32, nivelado: f32, vela: f32,
    material: crate::material::MaterialCelula, aparencia: aparencia::AparenciaCelula,
    oleo: oleo_celula::OleoCelula, tinta: tinta::CelulaTinta,
    pigmento: tinta::pigmento::LadoEspectral, wet: u8, active: u8, bloom: u8,
);

/// Os planos que nascem VAZIOS (`garante`): o comprimento na base decide se o restauro os esvazia.
fn comprimentos_vazios(g: &Grid) -> [usize; 9] {
    [
        g.altura.len(),
        g.espessura.len(),
        g.crista.len(),
        g.nivelado.len(),
        g.material.len(),
        g.aparencia.len(),
        g.oleo.len(),
        g.tinta.len(),
        g.pigmento.len(),
    ]
}

/// Leva um plano preguiçoso ao comprimento da base antes de colar os tiles: vazio na base ⇒ vazio;
/// cheio na base e vazio agora só acontece depois de uma acção de tela inteira, que marcou TODOS os
/// tiles — a amostra é preenchimento que a colagem cobre.
fn ajusta<T: Clone>(v: &mut Vec<T>, base: usize, amostra: Option<&T>) {
    if base == 0 {
        v.clear();
    } else if v.len() != base
        && let Some(a) = amostra
    {
        v.resize(base, a.clone());
    }
}

#[derive(Clone)]
struct Escalares {
    bx0: i32,
    by0: i32,
    bx1: i32,
    by1: i32,
    has_fluid: bool,
    row_lo: Vec<i32>,
    row_hi: Vec<i32>,
    paper_preset: PaperPreset,
    paper_sheet: u32,
    vazios: [usize; 9],
}

impl Escalares {
    fn de(g: &Grid) -> Self {
        Escalares {
            bx0: g.bx0,
            by0: g.by0,
            bx1: g.bx1,
            by1: g.by1,
            has_fluid: g.has_fluid,
            row_lo: g.row_lo.clone(),
            row_hi: g.row_hi.clone(),
            paper_preset: g.paper_preset,
            paper_sheet: g.paper_sheet,
            vazios: comprimentos_vazios(g),
        }
    }
}

/// Um passo do histórico: a base (escalares) + os tiles escritos desde que abriu, cada um com o
/// conteúdo da BASE.
#[derive(Clone)]
pub struct Registo {
    base: Escalares,
    finos: std::collections::BTreeMap<u32, TileFino>,
    fluxo: std::collections::BTreeMap<u32, [Vec<f32>; 2]>,
    /// O passo da água no instante em que abriu (quem o preenche e repõe é o `Engine`).
    pub(crate) passo: PassoEmVoo,
    /// O casco dos dabs do passo (células finas com o pad, meio-aberto, com a folga do dab): o que o traço escreveu.
    pub(crate) dabs: Option<[i64; 4]>,
}

/// O que o passo da água guarda FORA das grades quando um registo abre: o cursor do passo em voo. O undo/redo
/// repõe-no com as grades — sem ele os estágios que faltavam nunca correm. (A caixa dos dabs, `agua_nova`, não
/// entra: todo dab alarga a caixa viva, que é escalar do registo — `historia_redo_cobre_a_agua_dos_dabs_que_a_caixa_nao_viu`.)
#[derive(Clone, Default)]
pub(crate) struct PassoEmVoo {
    pub(crate) cursor: Option<crate::sim::StepCursor>,
}

impl Registo {
    /// Abre um registo vazio sobre o estado actual.
    #[must_use]
    pub fn abre(g: &Grid) -> Self {
        Registo {
            base: Escalares::de(g),
            finos: std::collections::BTreeMap::new(),
            fluxo: std::collections::BTreeMap::new(),
            passo: PassoEmVoo::default(),
            dabs: None,
        }
    }

    /// **Antes** de escrever no retângulo `[x0, x1) × [y0, y1)` (células finas, com o anel de pad):
    /// copia cada tile ainda não guardado. Os planos de fluxo vão pela razão `rf`, com folga de 2.
    pub fn marca(&mut self, g: &Grid, x0: i64, y0: i64, x1: i64, y1: i64) {
        let cl = |v: i64, n: usize| v.clamp(0, n as i64) as usize;
        let (ax0, ax1) = (cl(x0, g.s), cl(x1, g.s));
        let (ay0, ay1) = (cl(y0, g.rows), cl(y1, g.rows));
        if ax0 >= ax1 || ay0 >= ay1 {
            return;
        }
        let tw = g.s.div_ceil(TILE);
        for ty in ay0 / TILE..=(ay1 - 1) / TILE {
            for tx in ax0 / TILE..=(ax1 - 1) / TILE {
                let k = (ty * tw + tx) as u32;
                self.finos
                    .entry(k)
                    .or_insert_with(|| TileFino::tira(g, ret_do_tile(k, g.s, g.rows)));
            }
        }
        let f = &g.flow;
        let rf = f.rf.max(1) as i64;
        let (fx0, fx1) = (cl(x0 / rf - 2, f.s), cl(x1 / rf + 3, f.s));
        let (fy0, fy1) = (cl(y0 / rf - 2, f.rows), cl(y1 / rf + 3, f.rows));
        if fx0 >= fx1 || fy0 >= fy1 {
            return;
        }
        let fw = f.s.div_ceil(TILE);
        for ty in fy0 / TILE..=(fy1 - 1) / TILE {
            for tx in fx0 / TILE..=(fx1 - 1) / TILE {
                let k = (ty * fw + tx) as u32;
                self.fluxo.entry(k).or_insert_with(|| {
                    let r = ret_do_tile(k, f.s, f.rows);
                    [tira(&g.vel_x, f.s, r), tira(&g.vel_y, f.s, r)]
                });
            }
        }
    }

    /// Antes de uma escrita sem retângulo conhecido (acções de tela inteira, `&mut Grid` cru).
    pub fn marca_tudo(&mut self, g: &Grid) {
        let (s, rows) = (g.s as i64, g.rows as i64);
        self.marca(g, 0, 0, s, rows);
    }

    /// A caixa dos tiles que o registo guarda (meio-aberta, com o pad); a grade inteira se o papel muda.
    #[must_use]
    pub fn caixa(&self, g: &Grid) -> Option<[usize; 4]> {
        if self.base.paper_preset != g.paper_preset || self.base.paper_sheet != g.paper_sheet {
            return Some([0, 0, g.s, g.rows]);
        }
        self.finos.keys().fold(None, |c, k| {
            let r = ret_do_tile(*k, g.s, g.rows);
            Some(c.map_or([r.x0, r.y0, r.x1, r.y1], |c: [usize; 4]| {
                [
                    c[0].min(r.x0),
                    c[1].min(r.y0),
                    c[2].max(r.x1),
                    c[3].max(r.y1),
                ]
            }))
        })
    }

    /// Fotografa o estado ACTUAL sobre os tiles que `a` (e `b`) guardam — o registo inverso.
    #[must_use]
    pub fn inverso(g: &Grid, a: &Registo, b: Option<&Registo>) -> Self {
        let mut out = Registo::abre(g);
        for k in a
            .finos
            .keys()
            .chain(b.into_iter().flat_map(|b| b.finos.keys()))
        {
            out.finos
                .entry(*k)
                .or_insert_with(|| TileFino::tira(g, ret_do_tile(*k, g.s, g.rows)));
        }
        let f = &g.flow;
        for k in a
            .fluxo
            .keys()
            .chain(b.into_iter().flat_map(|b| b.fluxo.keys()))
        {
            out.fluxo.entry(*k).or_insert_with(|| {
                let r = ret_do_tile(*k, f.s, f.rows);
                [tira(&g.vel_x, f.s, r), tira(&g.vel_y, f.s, r)]
            });
        }
        out
    }

    /// Devolve o grid à base deste registo. `extra` cobre os tiles escritos desde o último evento do
    /// histórico que este registo não guarda (o caso do refazer). Devolve `true` se o papel mudou.
    pub fn restaura(&self, g: &mut Grid, extra: Option<&Registo>) -> bool {
        let tiles = || {
            self.finos.iter().chain(
                extra
                    .into_iter()
                    .flat_map(|e| e.finos.iter().filter(|(k, _)| !self.finos.contains_key(k))),
            )
        };
        let v = self.base.vazios;
        macro_rules! amostra {
            ($f:ident) => {
                tiles().find_map(|(_, t)| t.$f.first().cloned())
            };
        }
        let (a0, a1, a2, a3) = (
            amostra!(altura),
            amostra!(espessura),
            amostra!(crista),
            amostra!(nivelado),
        );
        let (a5, a6, a7) = (amostra!(material), amostra!(aparencia), amostra!(oleo));
        ajusta(&mut g.altura, v[0], a0.as_ref());
        ajusta(&mut g.espessura, v[1], a1.as_ref());
        ajusta(&mut g.crista, v[2], a2.as_ref());
        ajusta(&mut g.nivelado, v[3], a3.as_ref());
        ajusta(&mut g.material, v[4], a5.as_ref());
        ajusta(&mut g.aparencia, v[5], a6.as_ref());
        ajusta(&mut g.oleo, v[6], a7.as_ref());
        let a8 = amostra!(tinta);
        ajusta(&mut g.tinta, v[7], a8.as_ref());
        let a9 = amostra!(pigmento);
        ajusta(&mut g.pigmento, v[8], a9.as_ref());
        for (k, t) in tiles() {
            t.poe(g, ret_do_tile(*k, g.s, g.rows));
        }
        // Contam o que os tiles acabaram de pôr: antes deles, um plano recém-criado ainda está neutro.
        super::aparencia::reconta_teto(g);
        super::oleo_celula::reconta_real(g);
        let fluxo = self.fluxo.iter().chain(
            extra
                .into_iter()
                .flat_map(|e| e.fluxo.iter().filter(|(k, _)| !self.fluxo.contains_key(k))),
        );
        let (fs, frows) = (g.flow.s, g.flow.rows);
        for (k, [vx, vy]) in fluxo {
            let r = ret_do_tile(*k, fs, frows);
            poe(&mut g.vel_x, fs, r, vx);
            poe(&mut g.vel_y, fs, r, vy);
        }
        g.flow_x.fill(0.0);
        g.flow_y.fill(0.0);
        let b = &self.base;
        g.bx0 = b.bx0;
        g.by0 = b.by0;
        g.bx1 = b.bx1;
        g.by1 = b.by1;
        g.has_fluid = b.has_fluid;
        g.row_lo.copy_from_slice(&b.row_lo);
        g.row_hi.copy_from_slice(&b.row_hi);
        let paper_changed = g.paper_preset != b.paper_preset || g.paper_sheet != b.paper_sheet;
        g.paper_preset = b.paper_preset;
        g.paper_sheet = b.paper_sheet;
        paper_changed
    }

    /// Bytes que o registo retém (tiles + a faixa viva).
    #[must_use]
    pub fn bytes_retidos(&self) -> usize {
        self.finos.values().map(TileFino::bytes).sum::<usize>()
            + self
                .fluxo
                .values()
                .map(|[a, b]| 4 * (a.len() + b.len()))
                .sum::<usize>()
            + 4 * (self.base.row_lo.len() + self.base.row_hi.len())
    }
}
