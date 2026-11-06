//! ⭐ **O undo não mata a tinta viva** (smoke do dono 30/10) — filho de [`super`] por LOC.
//!
//! Antes, todo undo/redo acabava a sessão (`restore_model_confined` → `wetpaint_end_session`): a tinta que ficava na
//! tela virava `base` seca de uma sessão nova. Agora o pen-down de cada passo abre um registo no histórico
//! copy-on-write do motor ([`Engine::capture_history`], tiles 64²) e fotografa o que fica FORA das grades (o fluxo, a
//! ladeira do óleo, o relógio da sim) pela CHAVE do passo ([`crate::undo::UndoController::chave_do_topo`]); o undo
//! desse passo devolve as grades pelo [`Engine::undo`] e o resto pela fotografia, o redo pelo [`Engine::redo`].
//!
//! ⛔ Regressão 31/10 («undo/redo voltou a matar a tinta»): a fotografia era a grade INTEIRA (63 B/célula) contra o
//! orçamento da história — a 4096×2560 não cabia UMA (662 MB × 604 MB) e todo undo acabava a sessão; a 3840×2160
//! cabia um passo só. O histórico por tiles guarda só o que o passo escreveu.

use super::*;
use ph2d_wet_paint::painter::{Engine, HISTORY_DEPTH};

/// O estado do motor fora das grades que a física lê entre dois traços (a cadência da secagem, o nº do traço).
#[derive(Clone, Copy)]
pub(crate) struct Relogio {
    frame: u64,
    dry_every: u64,
    evap_scale: f64,
    rewet_base: f64,
    semente_do_po: u32,
}

/// O motor num instante, FORA das grades (essas voltam pelo histórico do motor): a ladeira do óleo, o fluxo e o
/// relógio.
#[derive(Clone)]
pub(crate) struct FotoDoMotor {
    oleo_antes: Vec<Vec<f32>>,
    /// O fluxo do último passo: o restauro zera-o, mas o Jacobi parte dele (medido: 13 B na aguada seguinte).
    fluxo: Vec<(Vec<f32>, Vec<f32>)>,
    relogio: Relogio,
}

/// Um passo de undo com a tinta viva de antes (e, depois de desfeito, a de depois para o redo).
#[derive(Clone)]
pub(crate) struct PassoVivo {
    chave: usize,
    antes: FotoDoMotor,
    depois: Option<FotoDoMotor>,
    /// A tinta antiga: a Borracha muda-a ([`super::borracha_na_base`]). Passos sem Borracha partilham o mesmo `Arc`.
    base_antes: Arc<Vec<u8>>,
    base_depois: Option<Arc<Vec<u8>>>,
}

/// A sessão tirada do caminho do restore, com a fotografia a devolver, o sentido (`true` = refazer) e a chave do
/// passo.
pub(crate) struct SessaoNoUndo(WetSession, FotoDoMotor, bool, usize);

/// **O tecto das fotografias** (RAM): o mesmo orçamento da história de undo do documento
/// ([`crate::undo::history_budget_bytes`]).
pub(crate) fn orcamento_das_fotos(w: usize, h: usize) -> usize {
    crate::undo::history_budget_bytes(w as u32, h as u32)
}

/// A cópia de um plano pelos núcleos: a fotografia corre a cada pen-down (Bug #72).
fn copia(de: &[f32]) -> Vec<f32> {
    let mut v = Vec::new();
    ph2d_wet_paint::par::copia_plano(de, &mut v);
    v
}

impl FotoDoMotor {
    fn de(e: &Engine) -> Self {
        let s = &e.sim;
        Self {
            oleo_antes: e.layers.iter().map(|l| copia(&l.grid.oleo_antes)).collect(),
            fluxo: e
                .layers
                .iter()
                .map(|l| (copia(&l.grid.flow_x), copia(&l.grid.flow_y)))
                .collect(),
            relogio: Relogio {
                frame: s.frame,
                dry_every: s.dry_every,
                evap_scale: s.evap_scale,
                rewet_base: s.rewet_base,
                semente_do_po: s.semente_do_po,
            },
        }
    }

    /// Repõe o que fica fora das grades — DEPOIS do `undo`/`redo` do motor (que zera o fluxo).
    fn repoe(&self, e: &mut Engine) {
        for (l, o) in e.layers.iter_mut().zip(&self.oleo_antes) {
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
        s.semente_do_po = r.semente_do_po;
        e.mark_dirty_full();
    }

    /// Os bytes que a fotografia do motor `e` seguraria, sem a tirar.
    fn bytes_de(e: &Engine) -> usize {
        use std::mem::size_of_val as b;
        e.layers
            .iter()
            .map(|l| {
                let g = &l.grid;
                b(&g.oleo_antes[..]) + b(&g.flow_x[..]) + b(&g.flow_y[..])
            })
            .sum()
    }

    /// Os bytes que a fotografia segura.
    pub(crate) fn bytes(&self) -> usize {
        self.oleo_antes.iter().map(|o| o.len() * 4).sum::<usize>()
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
    /// Pen-down de um passo novo com a sessão viva: abre o registo do passo no histórico do motor e fotografa o
    /// resto com a chave do passo, dentro do orçamento.
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
        // O registo copy-on-write do passo: daqui em diante toda escrita no grid marca os seus tiles nele.
        sess.engine.capture_history();
        let antes = FotoDoMotor::de(&sess.engine);
        let mut soma = antes.bytes();
        let mut ultima = Arc::as_ptr(&sess.base);
        // O motor guarda HISTORY_DEPTH registos: um passo mais velho já não tem as suas grades.
        let mut corte = sess.antes_do_passo.len().saturating_sub(HISTORY_DEPTH - 1);
        for (i, p) in sess.antes_do_passo.iter().enumerate().skip(corte).rev() {
            soma += p.bytes();
            if Arc::as_ptr(&p.base_antes) != ultima {
                ultima = Arc::as_ptr(&p.base_antes);
                soma += p.base_antes.len();
            }
            if soma > teto {
                corte = i + 1;
                break;
            }
        }
        sess.antes_do_passo.drain(..corte);
        sess.antes_do_passo.push(PassoVivo {
            chave,
            antes,
            depois: None,
            base_antes: Arc::clone(&sess.base),
            base_depois: None,
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
        sess.antes_do_passo.iter().find(|p| p.chave == chave)?;
        sess.bring_home();
        let pode = if refaz {
            sess.engine.can_redo()
        } else {
            sess.engine.can_undo()
        };
        if !pode {
            return None;
        }
        let p = sess.antes_do_passo.iter_mut().find(|p| p.chave == chave)?;
        let foto = if refaz {
            p.depois.take()?
        } else {
            p.antes.clone()
        };
        let sess = self.paint.wetpaint.session.take()?;
        Some(SessaoNoUndo(sess, foto, refaz, chave))
    }

    /// Depois do undo/redo: a sessão volta com o motor do passo (grades pelo histórico do motor, o resto pela
    /// fotografia), presa à tela restaurada.
    pub(crate) fn wet_volta_do_undo(&mut self, solta: SessaoNoUndo, aplicou: bool) {
        let SessaoNoUndo(mut sess, foto, refaz, chave) = solta;
        if aplicou {
            sess.bring_home();
            if !refaz {
                // O «depois» que o redo devolve: o motor drena o passo em voo no undo, a fotografia no mesmo
                // instante — e só quando o undo se aplica (drenar antes mudava o ritmo da água à toa).
                sess.engine.drain_step();
                let depois = FotoDoMotor::de(&sess.engine);
                let atual = Arc::clone(&sess.base);
                if let Some(p) = sess.antes_do_passo.iter_mut().find(|p| p.chave == chave) {
                    p.depois = Some(depois);
                    p.base_depois = Some(atual);
                    sess.base = Arc::clone(&p.base_antes);
                }
            } else if let Some(b) = sess
                .antes_do_passo
                .iter_mut()
                .find(|p| p.chave == chave)
                .and_then(|p| p.base_depois.take())
            {
                sess.base = b;
            }
            let feito = if refaz {
                sess.engine.redo()
            } else {
                sess.engine.undo()
            };
            debug_assert!(
                feito,
                "o histórico do motor tinha o passo (verificado ao soltar)"
            );
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
