//! ⭐ **O undo não mata a tinta viva** (smoke do dono 30/10) — filho de [`super`] por LOC.
//!
//! Antes, todo undo/redo acabava a sessão (`restore_model_confined` → `wetpaint_end_session`): a tinta que ficava na
//! tela virava `base` seca de uma sessão nova. Agora o pen-down de cada passo fotografa o motor (grades, ladeira do
//! óleo, relógio da sim) pela CHAVE do passo ([`crate::undo::UndoController::chave_do_topo`]); o undo desse passo
//! fotografa o «depois» e devolve o «antes», o redo devolve o «depois». Sem a fotografia a sessão acaba como sempre.
//! As fotografias somam até [`orcamento_das_fotos`]; as mais velhas saem primeiro.

use super::*;
use ph2d_wet_paint::grid::{GridSnapshot, restore_grid, snapshot_grid};

/// O estado do motor fora das grades que a física lê entre dois traços (a cadência da secagem, o nº do traço).
#[derive(Clone, Copy)]
pub(crate) struct Relogio {
    frame: u64,
    dry_every: u64,
    evap_scale: f64,
    rewet_base: f64,
    traco_actual: u32,
    semente_do_po: u32,
}

/// O motor num instante: as grades de cada camada, a ladeira do óleo e o relógio.
#[derive(Clone)]
pub(crate) struct FotoDoMotor {
    grades: Vec<GridSnapshot>,
    oleo_antes: Vec<Vec<f32>>,
    /// O fluxo do último passo: o `restore_grid` zera-o, mas o Jacobi parte dele (medido: 13 B na aguada seguinte).
    fluxo: Vec<(Vec<f32>, Vec<f32>)>,
    relogio: Relogio,
}

/// Um passo de undo com a tinta viva de antes (e, depois de desfeito, a de depois para o redo).
#[derive(Clone)]
pub(crate) struct PassoVivo {
    chave: usize,
    antes: FotoDoMotor,
    depois: Option<FotoDoMotor>,
}

/// A sessão tirada do caminho do restore, com a fotografia a devolver.
pub(crate) struct SessaoNoUndo(WetSession, FotoDoMotor);

/// **O tecto das fotografias** (RAM): o mesmo orçamento da história de undo do documento
/// ([`crate::undo::history_budget_bytes`]). Medido em `diag_bytes_da_foto_por_passo`: ~55 B por célula da grade.
pub(crate) fn orcamento_das_fotos(w: usize, h: usize) -> usize {
    crate::undo::history_budget_bytes(w as u32, h as u32)
}

fn bytes_da_grade(s: &GridSnapshot) -> usize {
    use std::mem::size_of_val as b;
    b(&s.film[..])
        + b(&s.susp[..])
        + b(&s.susp_rgb[..])
        + b(&s.sett[..])
        + b(&s.sett_rgb[..])
        + b(&s.relevo[..])
        + b(&s.altura[..])
        + b(&s.espessura[..])
        + b(&s.crista[..])
        + b(&s.nivelado[..])
        + b(&s.traco[..])
        + b(&s.vela[..])
        + b(&s.material[..])
        + b(&s.aparencia[..])
        + b(&s.oleo[..])
        + b(&s.vel_x[..])
        + b(&s.vel_y[..])
        + b(&s.wet[..])
        + b(&s.active[..])
        + b(&s.bloom[..])
        + b(&s.row_lo[..])
        + b(&s.row_hi[..])
}

impl FotoDoMotor {
    fn de(e: &ph2d_wet_paint::painter::Engine) -> Self {
        let s = &e.sim;
        Self {
            grades: e.layers.iter().map(|l| snapshot_grid(&l.grid)).collect(),
            oleo_antes: e.layers.iter().map(|l| l.grid.oleo_antes.clone()).collect(),
            fluxo: e
                .layers
                .iter()
                .map(|l| (l.grid.flow_x.clone(), l.grid.flow_y.clone()))
                .collect(),
            relogio: Relogio {
                frame: s.frame,
                dry_every: s.dry_every,
                evap_scale: s.evap_scale,
                rewet_base: s.rewet_base,
                traco_actual: s.traco_actual,
                semente_do_po: s.semente_do_po,
            },
        }
    }

    fn repoe(&self, e: &mut ph2d_wet_paint::painter::Engine) {
        for ((l, f), o) in e.layers.iter_mut().zip(&self.grades).zip(&self.oleo_antes) {
            restore_grid(&mut l.grid, f);
            l.grid.oleo_antes.clone_from(o);
        }
        for (l, (fx, fy)) in e.layers.iter_mut().zip(&self.fluxo) {
            l.grid.flow_x.copy_from_slice(fx);
            l.grid.flow_y.copy_from_slice(fy);
        }
        let (r, s) = (self.relogio, &mut e.sim);
        s.frame = r.frame;
        s.dry_every = r.dry_every;
        s.evap_scale = r.evap_scale;
        s.rewet_base = r.rewet_base;
        s.traco_actual = r.traco_actual;
        s.semente_do_po = r.semente_do_po;
        e.mark_dirty_full();
    }

    /// Os bytes que a fotografia do motor `e` seguraria, sem a tirar.
    fn bytes_de(e: &ph2d_wet_paint::painter::Engine) -> usize {
        use std::mem::size_of_val as b;
        e.layers
            .iter()
            .map(|l| {
                let g = &l.grid;
                b(&g.film[..])
                    + b(&g.susp[..])
                    + b(&g.susp_rgb[..])
                    + b(&g.sett[..])
                    + b(&g.sett_rgb[..])
                    + b(&g.relevo[..])
                    + b(&g.altura[..])
                    + b(&g.espessura[..])
                    + b(&g.crista[..])
                    + b(&g.nivelado[..])
                    + b(&g.traco[..])
                    + b(&g.vela[..])
                    + b(&g.material[..])
                    + b(&g.aparencia[..])
                    + b(&g.oleo[..])
                    + b(&g.vel_x[..])
                    + b(&g.vel_y[..])
                    + b(&g.wet[..])
                    + b(&g.active[..])
                    + b(&g.bloom[..])
                    + b(&g.row_lo[..])
                    + b(&g.row_hi[..])
                    + b(&g.oleo_antes[..])
                    + b(&g.flow_x[..])
                    + b(&g.flow_y[..])
            })
            .sum()
    }

    /// Os bytes que a fotografia segura.
    pub(crate) fn bytes(&self) -> usize {
        self.grades.iter().map(bytes_da_grade).sum::<usize>()
            + self.oleo_antes.iter().map(|o| o.len() * 4).sum::<usize>()
            + self
                .fluxo
                .iter()
                .map(|(x, y)| (x.len() + y.len()) * 4)
                .sum::<usize>()
    }
}

impl PassoVivo {
    fn bytes(&self) -> usize {
        self.antes.bytes() + self.depois.as_ref().map_or(0, FotoDoMotor::bytes)
    }
}

impl PainterTool {
    /// Pen-down de um passo novo com a sessão viva: fotografa o motor com a chave do passo, dentro do orçamento.
    pub(crate) fn wet_guarda_o_passo(&mut self) {
        let chave = self.undo.chave_do_topo();
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        sess.bring_home();
        sess.antes_do_passo.retain(|p| p.chave < chave);
        let teto = orcamento_das_fotos(sess.engine.width, sess.engine.height);
        if FotoDoMotor::bytes_de(&sess.engine) > teto {
            sess.antes_do_passo.clear(); // não cabe UMA: o undo acaba a sessão, como antes
            return;
        }
        let antes = FotoDoMotor::de(&sess.engine);
        let mut soma = antes.bytes();
        let mut corte = sess.antes_do_passo.len();
        for (i, p) in sess.antes_do_passo.iter().enumerate().rev() {
            soma += p.bytes();
            if soma > teto {
                break;
            }
            corte = i;
        }
        sess.antes_do_passo.drain(..corte);
        sess.antes_do_passo.push(PassoVivo {
            chave,
            antes,
            depois: None,
        });
    }

    /// Antes de um undo (`refaz = false`) ou redo: se a sessão tem a fotografia do passo, sai do caminho do restore
    /// com ela (o undo guarda o «depois» para o redo).
    pub(crate) fn wet_solta_para_o_undo(&mut self, refaz: bool) -> Option<SessaoNoUndo> {
        let chave = if refaz {
            self.undo.chave_do_topo()
        } else {
            self.undo.chave_do_topo().checked_sub(1)?
        };
        self.wetpaint_guard();
        let sess = self.paint.wetpaint.session.as_mut()?;
        let p = sess.antes_do_passo.iter_mut().find(|p| p.chave == chave)?;
        let foto = if refaz {
            p.depois.take()?
        } else {
            sess.bring_home();
            let p = sess.antes_do_passo.iter_mut().find(|p| p.chave == chave)?;
            p.depois = Some(FotoDoMotor::de(&sess.engine));
            p.antes.clone()
        };
        let sess = self.paint.wetpaint.session.take()?;
        Some(SessaoNoUndo(sess, foto))
    }

    /// Depois do undo/redo: a sessão volta com o motor da fotografia, presa à tela restaurada.
    pub(crate) fn wet_volta_do_undo(&mut self, solta: SessaoNoUndo, aplicou: bool) {
        let SessaoNoUndo(mut sess, foto) = solta;
        if aplicou {
            sess.bring_home();
            foto.repoe(&mut sess.engine);
            sess.canvas = Arc::downgrade(&self.canvas_rgba);
            sess.stroke_open = false;
            sess.lanes.clear();
            sess.adiados.clear();
            sess.preview = None;
        }
        // Uma sessão que o restore fez nascer (não devia: o restore acaba-as) perde para a que estava viva.
        self.paint.wetpaint.session = Some(sess);
    }
}

#[cfg(test)]
impl FotoDoMotor {
    pub(crate) fn de_para_sonda(e: &ph2d_wet_paint::painter::Engine) -> Self {
        Self::de(e)
    }

    pub(crate) fn bytes_de_para_sonda(e: &ph2d_wet_paint::painter::Engine) -> usize {
        Self::bytes_de(e)
    }
}
