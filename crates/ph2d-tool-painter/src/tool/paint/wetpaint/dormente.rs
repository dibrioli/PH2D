//! **A tinta adormecida comprimida em RAM** ([plano](../../../../../../../docs/Painter/TINTA_VIVA_2026-11-06.md)): os planos da grade
//! e da óptica, byte a byte em planos separados e zstd 1, ao bit. Corre num fio à parte depois de adormecer: nem a troca
//! de modo nem os outros meios a pagam; acordar espera-a e descomprime.

use super::*;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::grid::aparencia::AparenciaCelula;
use ph2d_wet_paint::grid::celula_bytes::{Celula, de_bytes, para_bytes};
use ph2d_wet_paint::grid::oleo_celula::OleoCelula;
use ph2d_wet_paint::material::MaterialCelula;
use rayon::prelude::*;

const NIVEL: i32 = 1;

/// Um plano comprimido, com o comprimento que tinha.
#[derive(Clone, Default)]
pub(super) struct Bloco {
    n: usize,
    z: Vec<u8>,
}

impl Bloco {
    fn de_bytes(cru: &[u8], n: usize) -> Self {
        let z = zstd::bulk::compress(cru, NIVEL).expect("o zstd comprime em memória");
        Self { n, z }
    }

    fn bytes(&self, cap: usize) -> Vec<u8> {
        zstd::bulk::decompress(&self.z, cap).expect("o zstd devolve o que comprimiu")
    }
}

fn encolhe_u8(v: &mut Vec<u8>) -> Bloco {
    let cru = std::mem::take(v);
    if cru.is_empty() {
        return Bloco::default();
    }
    Bloco::de_bytes(&cru, cru.len())
}

fn volta_u8(b: &Bloco, v: &mut Vec<u8>) {
    if b.n > 0 {
        *v = b.bytes(b.n);
    }
}

fn encolhe<T: Celula>(v: &mut Vec<T>) -> Bloco {
    let cru = std::mem::take(v);
    if cru.is_empty() {
        return Bloco::default();
    }
    let mut buf = Vec::with_capacity(cru.len() * T::N * 4);
    para_bytes(&cru, &mut buf);
    drop(cru); // o pico é um plano só, nunca os dois
    Bloco::de_bytes(&buf, buf.len())
}

fn volta<T: Celula>(b: &Bloco, v: &mut Vec<T>) {
    if b.n > 0 {
        *v = de_bytes(&b.bytes(b.n));
    }
}

/// Um plano da grade, pelo tipo da célula.
enum Plano<'a> {
    F(&'a mut Vec<f32>),
    F3(&'a mut Vec<[f32; 3]>),
    M(&'a mut Vec<MaterialCelula>),
    A(&'a mut Vec<AparenciaCelula>),
    O(&'a mut Vec<OleoCelula>),
    B(&'a mut Vec<u8>),
}

impl Plano<'_> {
    fn encolhe(self) -> Bloco {
        match self {
            Plano::F(v) => encolhe(v),
            Plano::F3(v) => encolhe(v),
            Plano::M(v) => encolhe(v),
            Plano::A(v) => encolhe(v),
            Plano::O(v) => encolhe(v),
            Plano::B(v) => encolhe_u8(v),
        }
    }

    fn volta(self, b: &Bloco) {
        match self {
            Plano::F(v) => volta(b, v),
            Plano::F3(v) => volta(b, v),
            Plano::M(v) => volta(b, v),
            Plano::A(v) => volta(b, v),
            Plano::O(v) => volta(b, v),
            Plano::B(v) => volta_u8(b, v),
        }
    }
}

/// Os planos que viajam. O padrão sem `..` obriga um campo novo a dizer se viaja.
fn planos_da_grade(g: &mut Grid) -> Vec<Plano<'_>> {
    let Grid {
        w: _,
        h: _,
        s: _,
        rows: _,
        cells: _,
        film,
        susp,
        susp_rgb,
        sett,
        sett_rgb,
        relevo,
        altura,
        espessura,
        crista,
        nivelado,
        vela,
        material,
        aparencia,
        suave_teto: _,
        borda_fundo: _,
        oleo,
        oleo_real: _,
        oleo_antes,
        oleo_antes_assente,
        chao,
        sett_do_traco,
        // Lei arquivada: vazias com a lei antiga; ficam como estão.
        tinta: _,
        pigmento: _,
        flow: _,
        vel_x,
        vel_y,
        flow_x,
        flow_y,
        frow_lo: _,
        frow_hi: _,
        wet,
        paper,
        active,
        bloom,
        has_fluid: _,
        row_lo: _,
        row_hi: _,
        live_lo: _,
        live_hi: _,
        spans_enabled: _,
        scratch: _,
        paper_preset: _,
        paper_sheet: _,
        bx0: _,
        by0: _,
        bx1: _,
        by1: _,
    } = g;
    vec![
        Plano::F(film),
        Plano::F(susp),
        Plano::F3(susp_rgb),
        Plano::F(sett),
        Plano::F3(sett_rgb),
        Plano::F(relevo),
        Plano::F(altura),
        Plano::F(espessura),
        Plano::F(crista),
        Plano::F(nivelado),
        Plano::F(vela),
        Plano::M(material),
        Plano::A(aparencia),
        Plano::O(oleo),
        Plano::F(oleo_antes),
        Plano::F(oleo_antes_assente),
        Plano::F(chao),
        Plano::F(sett_do_traco),
        Plano::F(vel_x),
        Plano::F(vel_y),
        Plano::F(flow_x),
        Plano::F(flow_y),
        Plano::F(paper),
        Plano::B(wet),
        Plano::B(active),
        Plano::B(bloom),
    ]
}

fn encolhe_a_grade(g: &mut Grid) -> Vec<Bloco> {
    let saida = planos_da_grade(g)
        .into_par_iter()
        .map(Plano::encolhe)
        .collect();
    // O rascunho do solver nasce vazio numa cópia, e o passo seguinte volta a fazê-lo.
    g.scratch = Default::default();
    saida
}

fn volta_a_grade(g: &mut Grid, blocos: &[Bloco]) {
    planos_da_grade(g)
        .into_par_iter()
        .zip(blocos)
        .for_each(|(p, b)| p.volta(b));
}

/// O que uma sessão adormecida guardou comprimido. `Arc`: uma cópia do estado (um tocador de macro) partilha-o.
#[derive(Clone)]
pub(super) struct Dormente {
    grades: Vec<Vec<Bloco>>,
    otica: [Bloco; 4],
    /// Os tiles do histórico do motor, registo a registo (`None`: ficou lá).
    registos: Vec<Option<Bloco>>,
    /// As fotografias dos passos do undo vivo: cada tile distinto uma vez.
    fotos: Bloco,
    mapa_das_fotos: super::foto_em_tiles::MapaDosTiles,
}

impl Dormente {
    pub(super) fn bytes(&self) -> usize {
        let z = |b: &Bloco| b.z.len();
        self.grades.iter().flatten().map(z).sum::<usize>()
            + self.otica.iter().map(z).sum::<usize>()
            + self.registos.iter().flatten().map(z).sum::<usize>()
            + z(&self.fotos)
    }
}

impl WetSession {
    /// Comprime o que pesa: a sessão só volta a servir depois de [`Self::reidrata`]. Devolve os bytes que ela retém.
    pub(super) fn encolhe(&mut self) -> usize {
        if self.dormente.is_none() {
            let grades = self
                .engine
                .layers
                .iter_mut()
                .map(|l| encolhe_a_grade(&mut l.grid))
                .collect();
            let p = &mut self.planos;
            p.campo_cache = None; // a altura guardada refaz-se da grade
            let otica = [
                encolhe_u8(&mut p.pigment),
                encolhe_u8(&mut p.sombra),
                encolhe_u8(&mut p.grao),
                encolhe_u8(&mut p.suave),
            ];
            let registos = self
                .engine
                .registos_mut()
                .into_par_iter()
                .map(|r| r.em_bytes().map(|b| Bloco::de_bytes(&b, b.len())))
                .collect();
            let (mut celulas, mapa_das_fotos) = super::foto_em_tiles::tira_os_tiles(
                self.antes_do_passo
                    .iter_mut()
                    .flat_map(super::undo_vivo::PassoVivo::planos_mut)
                    .collect(),
            );
            let fotos = encolhe(&mut celulas);
            self.dormente = Some(Arc::new(Dormente {
                grades,
                otica,
                registos,
                fotos,
                mapa_das_fotos,
            }));
        }
        self.bytes_retidos() + self.dormente.as_ref().map_or(0, |d| d.bytes())
    }

    /// Devolve o que [`Self::encolhe`] guardou, ao bit.
    pub(super) fn reidrata(&mut self) {
        let Some(d) = self.dormente.take() else {
            return;
        };
        for (l, blocos) in self.engine.layers.iter_mut().zip(&d.grades) {
            volta_a_grade(&mut l.grid, blocos);
        }
        let p = &mut self.planos;
        volta_u8(&d.otica[0], &mut p.pigment);
        volta_u8(&d.otica[1], &mut p.sombra);
        volta_u8(&d.otica[2], &mut p.grao);
        volta_u8(&d.otica[3], &mut p.suave);
        self.engine
            .registos_mut()
            .into_par_iter()
            .zip(&d.registos)
            .for_each(|(r, b)| {
                if let Some(b) = b {
                    r.de_bytes(&b.bytes(b.n));
                }
            });
        let mut celulas = Vec::new();
        volta(&d.fotos, &mut celulas);
        super::foto_em_tiles::poe_os_tiles(
            self.antes_do_passo
                .iter_mut()
                .flat_map(super::undo_vivo::PassoVivo::planos_mut)
                .collect(),
            &celulas,
            &d.mapa_das_fotos,
        );
    }
}
