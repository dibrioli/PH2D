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
//!
//! **O limite é de memória** ([`teto_da_historia`]), e o desfazer para lá dele não mata a tinta que ficou
//! ([`PainterTool::wet_enterra_sem_passo`]): números no [doc](../../../../../../../docs/Painter/TINTA_VIVA_2026-11-06.md).

use super::foto_em_tiles::PlanoEmTiles;
use super::*;
use ph2d_wet_paint::painter::Engine;

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
    oleo_antes: Vec<PlanoEmTiles>,
    /// O fluxo do último passo: o restauro zera-o, mas o Jacobi parte dele (medido: 13 B na aguada seguinte).
    fluxo: Vec<(PlanoEmTiles, PlanoEmTiles)>,
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
    /// A tela congelada de antes e de depois do passo: desfeito o enterro, a orla volta a ser recomposta.
    congelada_antes: Option<Arc<super::adormecida::Congelada>>,
    congelada_depois: Option<Arc<super::adormecida::Congelada>>,
    /// Só no passo de OUTRA ferramenta ([`super::adormecida`]): a tela antes do enterro. Vários traços de outro modo
    /// enterram-se de uma vez ao acordar: desfeito só o último, o que ainda difere dela volta a enterrar-se.
    tela_antes: Option<Arc<Vec<u8>>>,
}

/// A sessão tirada do caminho do restore, com a fotografia a devolver, o sentido (`true` = refazer) e a chave do
/// passo.
pub(crate) struct SessaoNoUndo(WetSession, FotoDoMotor, bool, usize);

/// **O tecto das fotografias** (RAM): o mesmo orçamento da história de undo do documento
/// ([`crate::undo::history_budget_bytes`]).
pub(crate) fn orcamento_das_fotos(w: usize, h: usize) -> usize {
    crate::undo::history_budget_bytes(w as u32, h as u32)
}

/// **O tecto do undo vivo** (RAM): registos e fotografias. Seis orçamentos é o que seis passos de largura inteira já
/// retinham a 4096×2560 (`sonda_o_preco_de_um_passo`: 2,8 GB de registos + 0,5 GB de fotografias, contra 3,4 GB):
/// o tecto não gasta mais do que se gastava, e o traço curto vai muito mais fundo.
pub(crate) fn teto_da_historia(w: usize, h: usize) -> usize {
    #[cfg(test)]
    if let Some(t) = super::undo_tinta_morre_tests::teto::get() {
        return t;
    }
    6 * orcamento_das_fotos(w, h)
}

/// Os bytes das fotografias da pilha: a mais velha inteira, cada outra só o que não partilha com a de antes.
pub(crate) fn bytes_das_fotos(passos: &[PassoVivo]) -> usize {
    let primeira = passos.first().map_or(0, |p| p.antes.bytes());
    primeira
        + passos
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let antes = if i == 0 { 0 } else { p.antes.bytes_novos() };
                antes + p.depois.as_ref().map_or(0, FotoDoMotor::bytes_novos)
            })
            .sum::<usize>()
}

/// Os bytes que os passos retêm fora do motor: as fotografias, as telas e as `base` distintas.
fn bytes_dos_passos(passos: &[PassoVivo], base_de_agora: &Arc<Vec<u8>>) -> usize {
    let mut ultima = Arc::as_ptr(base_de_agora);
    let mut soma = bytes_das_fotos(passos);
    for p in passos.iter().rev() {
        soma += p.tela_antes.as_ref().map_or(0, |t| t.len());
        if Arc::as_ptr(&p.base_antes) != ultima {
            ultima = Arc::as_ptr(&p.base_antes);
            soma += p.base_antes.len();
        }
    }
    soma
}

/// Abre o registo copy-on-write do passo `chave` no motor (em casa), fotografa o resto, e larga os passos mais velhos
/// que passem do tecto: o registo e a fotografia de cada um.
fn abre_o_passo(sess: &mut WetSession, chave: usize, tela_antes: Option<Arc<Vec<u8>>>) {
    sess.antes_do_passo.retain(|p| p.chave < chave);
    // O registo copy-on-write do passo: daqui em diante toda escrita no grid marca os seus tiles nele.
    sess.engine.capture_history();
    let antes = FotoDoMotor::de(&sess.engine, sess.antes_do_passo.last().map(|p| &p.antes));
    sess.antes_do_passo.push(PassoVivo {
        chave,
        antes,
        depois: None,
        base_antes: Arc::clone(&sess.base),
        base_depois: None,
        congelada_antes: sess.congelada.clone(),
        congelada_depois: None,
        tela_antes,
    });
    let teto = teto_da_historia(sess.engine.width, sess.engine.height);
    let mut registos = sess.engine.bytes_do_historico(sess.engine.active_layer);
    while sess.antes_do_passo.len() > 1
        && registos + bytes_dos_passos(&sess.antes_do_passo, &sess.base) > teto
    {
        registos -= sess.engine.solta_o_registo_mais_velho();
        sess.antes_do_passo.remove(0);
    }
}

fn esquece_os_passos(sess: &mut WetSession) {
    sess.antes_do_passo.clear();
    sess.engine.esquece_a_historia();
}

impl FotoDoMotor {
    /// Fotografa o motor, partilhando com `anterior` os tiles que não mudaram.
    fn de(e: &Engine, anterior: Option<&Self>) -> Self {
        let s = &e.sim;
        let ate = |i: usize| anterior.and_then(|a| a.oleo_antes.get(i));
        let fluxo = |i: usize| anterior.and_then(|a| a.fluxo.get(i));
        Self {
            oleo_antes: e
                .layers
                .iter()
                .enumerate()
                .map(|(i, l)| PlanoEmTiles::de(&l.grid.oleo_antes, l.grid.s, ate(i)))
                .collect(),
            fluxo: e
                .layers
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    (
                        PlanoEmTiles::de(&l.grid.flow_x, l.grid.s, fluxo(i).map(|f| &f.0)),
                        PlanoEmTiles::de(&l.grid.flow_y, l.grid.s, fluxo(i).map(|f| &f.1)),
                    )
                })
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
            o.repoe(&mut l.grid.oleo_antes);
        }
        for (l, (fx, fy)) in e.layers.iter_mut().zip(&self.fluxo) {
            fx.repoe(&mut l.grid.flow_x);
            fy.repoe(&mut l.grid.flow_y);
        }
        let (r, s) = (self.relogio, &mut e.sim);
        s.frame = r.frame;
        s.dry_every = r.dry_every;
        s.evap_scale = r.evap_scale;
        s.rewet_base = r.rewet_base;
        s.semente_do_po = r.semente_do_po;
        e.mark_dirty_full();
    }

    fn planos(&self) -> impl Iterator<Item = &PlanoEmTiles> {
        self.oleo_antes
            .iter()
            .chain(self.fluxo.iter().flat_map(|(x, y)| [x, y]))
    }

    /// Os bytes da fotografia sozinha.
    pub(crate) fn bytes(&self) -> usize {
        self.planos().map(PlanoEmTiles::bytes).sum()
    }

    /// Os bytes que ela não partilha com a anterior.
    fn bytes_novos(&self) -> usize {
        self.planos().map(PlanoEmTiles::bytes_novos).sum()
    }

    fn planos_mut(&mut self) -> Vec<&mut PlanoEmTiles> {
        let mut v: Vec<&mut PlanoEmTiles> = self.oleo_antes.iter_mut().collect();
        for (x, y) in &mut self.fluxo {
            v.push(x);
            v.push(y);
        }
        v
    }
}

impl PassoVivo {
    /// Os planos que a tinta adormecida comprime ([`super::dormente`]).
    pub(super) fn planos_mut(&mut self) -> Vec<&mut PlanoEmTiles> {
        let mut v = self.antes.planos_mut();
        if let Some(d) = self.depois.as_mut() {
            v.extend(d.planos_mut());
        }
        v
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
        abre_o_passo(sess, chave, None);
    }

    /// Desfazer um passo que já saiu do histórico: a tinta dele acaba e a que ficou continua viva. As células onde a tela
    /// voltou atrás enterram-se; os passos que restam já não casam com o motor.
    pub(super) fn wet_enterra_sem_passo(&self, sess: &mut WetSession, velha: &Arc<Vec<u8>>) {
        sess.bring_home();
        esquece_os_passos(sess);
        sess.enterrou = true;
        sess.otica.toca_tudo(sess.grid);
        self.wet_enterra_o_que_mudou(sess, velha);
        // O passo deixou tinta onde ela não muda nenhum píxel (a orla fina da aguada, alfa 0): voltaria a correr.
        let (gw, gh) = sess.grid;
        self.wet_planos_em_dia(sess, (1, 1, gw, gh));
        let g = sess.engine.active_grid();
        let invisivel: Vec<bool> = (0..gw * gh)
            .map(|k| {
                let c = (k / gw + 1) * g.s + k % gw + 1;
                sess.planos.pigment[k * 4 + 3] == 0 && g.sett[c] + g.susp[c] > 0.0
            })
            .collect();
        sess.engine.enterra_celulas(&invisivel, gw);
        sess.engine.take_dirty();
    }

    /// O enterro de `velha` × a tela de agora, como o passo do topo do undo: desfazê-lo devolve as células.
    pub(super) fn wet_enterra_como_passo(&self, sess: &mut WetSession, velha: &Arc<Vec<u8>>) {
        // Um registo mais novo que o motor não desfez (a história andou com a sessão parada): as pilhas já não casam.
        let chave = self.undo.chave_do_topo().checked_sub(1).filter(|c| {
            !sess
                .antes_do_passo
                .iter()
                .any(|p| p.chave > *c && p.depois.is_none())
        });
        if let Some(chave) = chave {
            sess.antes_do_passo.retain(|p| p.chave <= chave);
            // O passo já aberto no pen-down é este (a Borracha plana): o enterro entra no registo dele.
            if sess.antes_do_passo.last().is_none_or(|p| p.chave != chave) {
                abre_o_passo(sess, chave, Some(Arc::clone(velha)));
            }
        } else {
            esquece_os_passos(sess);
        }
        sess.enterrou = true;
        self.wet_enterra_o_que_mudou(sess, velha);
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
                let atual = Arc::clone(&sess.base);
                let i = sess.antes_do_passo.iter().position(|p| p.chave == chave);
                if let Some(p) = i.map(|i| &mut sess.antes_do_passo[i]) {
                    p.depois = Some(FotoDoMotor::de(&sess.engine, Some(&p.antes)));
                    p.base_depois = Some(atual);
                    sess.base = Arc::clone(&p.base_antes);
                    p.congelada_depois = sess.congelada.take();
                    sess.congelada = p.congelada_antes.clone();
                }
            } else if let Some(p) = sess.antes_do_passo.iter_mut().find(|p| p.chave == chave) {
                if let Some(b) = p.base_depois.take() {
                    sess.base = b;
                }
                sess.congelada = p.congelada_depois.take();
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
            self.undo.write_state.vigia_a_tela(Some(&self.canvas_rgba));
            sess.stroke_open = false;
            sess.lanes.clear();
            sess.adiados.clear();
            sess.preview = None;
            // Os traços de outro modo que ficaram na tela continuam por cima da tinta: voltam a enterrá-la.
            let antes = (!refaz)
                .then(|| sess.antes_do_passo.iter().find(|p| p.chave == chave))
                .flatten()
                .and_then(|p| p.tela_antes.clone());
            if let Some(tela) = antes.filter(|t| **t != *self.canvas_rgba) {
                self.wet_enterra_como_passo(&mut sess, &tela);
            }
            // A tela restaurada já é a verdade. ⛔ Medido: recompor a tela inteira com células enterradas re-ilumina a
            // orla delas (3 862 px); a orla só se recalcula sob o pincel seguinte.
            if sess.enterrou {
                sess.engine.take_dirty();
                sess.otica.toca_tudo(sess.grid);
                let (gw, gh) = sess.grid;
                self.wet_planos_em_dia(&mut sess, (1, 1, gw, gh));
                if sess.congelada.is_some() {
                    self.wet_congela(&mut sess, None);
                }
            }
        }
        // Uma sessão que o restore fez nascer (não devia: o restore acaba-as) perde para a que estava viva.
        self.paint.wetpaint.session = Some(sess);
    }
}

#[cfg(test)]
impl FotoDoMotor {
    pub(crate) fn de_para_sonda(e: &ph2d_wet_paint::painter::Engine) -> Self {
        Self::de(e, None)
    }

    /// Os bytes que a fotografia do motor `e` seguraria, sem a tirar.
    pub(crate) fn bytes_de_para_sonda(e: &ph2d_wet_paint::painter::Engine) -> usize {
        use std::mem::size_of_val as b;
        e.layers
            .iter()
            .map(|l| {
                let g = &l.grid;
                b(&g.oleo_antes[..]) + b(&g.flow_x[..]) + b(&g.flow_y[..])
            })
            .sum()
    }
}
