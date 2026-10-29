//! As **fotografias do histórico** da grelha (SPEC §15) — filho de [`super`] por LOC.

use super::*;

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
    pub traco: Vec<u32>,
    pub vela: Vec<f32>,
    pub material: Vec<crate::material::MaterialCelula>,
    pub aparencia: Vec<aparencia::AparenciaCelula>,
    pub oleo: Vec<oleo_celula::OleoCelula>,
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
        traco: g.traco.clone(),
        vela: g.vela.clone(),
        material: g.material.clone(),
        aparencia: g.aparencia.clone(),
        oleo: g.oleo.clone(),
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
    g.traco.clone_from(&s.traco);
    g.vela.copy_from_slice(&s.vela);
    g.material.clone_from(&s.material);
    g.aparencia.clone_from(&s.aparencia);
    g.oleo.clone_from(&s.oleo);
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
