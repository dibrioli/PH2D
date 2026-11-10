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

mod casco;
#[cfg(test)]
mod casco_tests;
pub(crate) use casco::Cascos;
pub(super) use casco::casco_de;

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

/// Os bytes que os passos retêm fora do motor: as fotografias e cada `Arc` distinto de tela, `base` e tela congelada.
/// A `base` e a congelada de agora são da sessão, com ou sem passos.
pub(super) fn bytes_retidos_nos_passos(
    passos: &[PassoVivo],
    base_de_agora: &Arc<Vec<u8>>,
    congelada_de_agora: Option<&Arc<super::adormecida::Congelada>>,
) -> usize {
    use std::collections::BTreeSet;
    let mut telas: BTreeSet<*const Vec<u8>> = BTreeSet::from([Arc::as_ptr(base_de_agora)]);
    let mut congeladas: BTreeSet<*const super::adormecida::Congelada> =
        congelada_de_agora.map(Arc::as_ptr).into_iter().collect();
    let mut soma = bytes_das_fotos(passos);
    let mut conta = |t: &Arc<Vec<u8>>, soma: &mut usize| {
        if telas.insert(Arc::as_ptr(t)) {
            *soma += t.len();
        }
    };
    if let Some(c) = congelada_de_agora {
        conta(c.tela(), &mut 0);
        conta(c.base(), &mut 0);
    }
    for p in passos {
        conta(&p.base_antes, &mut soma);
        for t in p.base_depois.iter().chain(&p.tela_antes) {
            conta(t, &mut soma);
        }
        for c in p.congelada_antes.iter().chain(&p.congelada_depois) {
            if congeladas.insert(Arc::as_ptr(c)) {
                soma += c.bytes_proprios();
                conta(c.tela(), &mut soma);
                conta(c.base(), &mut soma);
            }
        }
    }
    soma
}

/// Abre o registo copy-on-write do passo `chave` no motor (em casa), fotografa o resto, e larga os passos mais velhos
/// que passem do tecto: o registo e a fotografia de cada um.
fn abre_o_passo(
    sess: &mut WetSession,
    chave: usize,
    tela_antes: Option<Arc<Vec<u8>>>,
    alcancavel_desde: usize,
) {
    sess.antes_do_passo.retain(|p| p.chave < chave);
    casco::guarda_o_casco(sess, alcancavel_desde, chave);
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
        && (sess.antes_do_passo[0].chave < alcancavel_desde
            || registos
                + bytes_retidos_nos_passos(
                    &sess.antes_do_passo,
                    &sess.base,
                    sess.congelada.as_ref(),
                )
                > teto)
    {
        registos -= sess.engine.solta_o_registo_mais_velho();
        sess.antes_do_passo.remove(0);
    }
}

fn esquece_os_passos(sess: &mut WetSession) {
    casco::guarda_o_casco(sess, 0, usize::MAX);
    sess.antes_do_passo.clear();
    sess.engine.esquece_a_historia();
}

/// O alcance do enterro da orla, em células: a tinta invisível de um traço desfeito fica até 7 células do que a tela
/// mudou (`sonda_alcance_da_orla`: Wet Flat de 20 a 160 px, até 3000 quadros de água). 32 é a folga sobre o medido.
const ALCANCE_DA_ORLA: usize = 32;

/// Uma caixa de células: `(x0, y0, x1, y1)`, 1-based e inclusiva.
type CaixaDeCelulas = (usize, usize, usize, usize);

/// As células a até `r` (Chebyshev) de uma marcada, e a caixa delas.
fn perto_de(
    marcadas: &[bool],
    gw: usize,
    gh: usize,
    r: usize,
) -> (Vec<bool>, Option<CaixaDeCelulas>) {
    let mut fora = vec![false; gw * gh];
    let (Some(a), Some(b)) = (
        marcadas.iter().position(|m| *m),
        marcadas.iter().rposition(|m| *m),
    ) else {
        return (fora, None);
    };
    let (y0, y1) = ((a / gw).saturating_sub(r), (b / gw + r).min(gh - 1));
    // Por linha, depois por coluna: cada passada é linear.
    let mut por_linha = vec![false; gw * gh];
    for y in a / gw..=b / gw {
        let (linha, saida) = (
            &marcadas[y * gw..(y + 1) * gw],
            &mut por_linha[y * gw..(y + 1) * gw],
        );
        let mut d = usize::MAX;
        for x in 0..gw {
            d = if linha[x] { 0 } else { d.saturating_add(1) };
            saida[x] = d <= r;
        }
        d = usize::MAX;
        for x in (0..gw).rev() {
            d = if linha[x] { 0 } else { d.saturating_add(1) };
            saida[x] |= d <= r;
        }
    }
    let mut d = vec![usize::MAX; gw];
    for y in y0..=y1 {
        for x in 0..gw {
            d[x] = if por_linha[y * gw + x] {
                0
            } else {
                d[x].saturating_add(1)
            };
            fora[y * gw + x] = d[x] <= r;
        }
    }
    d.fill(usize::MAX);
    for y in (y0..=y1).rev() {
        for x in 0..gw {
            d[x] = if por_linha[y * gw + x] {
                0
            } else {
                d[x].saturating_add(1)
            };
            fora[y * gw + x] |= d[x] <= r;
        }
    }
    let (mut cx0, mut cx1) = (gw, 0);
    for y in y0..=y1 {
        let l = &fora[y * gw..(y + 1) * gw];
        if let Some(x) = l.iter().position(|m| *m) {
            cx0 = cx0.min(x);
        }
        if let Some(x) = l.iter().rposition(|m| *m) {
            cx1 = cx1.max(x);
        }
    }
    (fora, Some((cx0 + 1, y0 + 1, cx1 + 1, y1 + 1)))
}

/// A caixa a recompor nos planos: a grade inteira só em teste, para comparar.
fn caixa_dos_planos(
    c: (usize, usize, usize, usize),
    gw: usize,
    gh: usize,
) -> (usize, usize, usize, usize) {
    #[cfg(test)]
    if super::undo_auditoria_tests::grade_inteira::get() {
        return (1, 1, gw, gh);
    }
    let _ = (gw, gh);
    c
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
                        // O fluxo mora na grade de FLUXO: outra largura que a fina, se `flow_ratio` > 1.
                        PlanoEmTiles::de(&l.grid.flow_x, l.grid.flow.s, fluxo(i).map(|f| &f.0)),
                        PlanoEmTiles::de(&l.grid.flow_y, l.grid.flow.s, fluxo(i).map(|f| &f.1)),
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
        let desde = chave - self.undo.undo_depth();
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        sess.bring_home();
        abre_o_passo(sess, chave, None, desde);
    }

    /// Desfazer um passo que já saiu do histórico: a tinta dele acaba e a que ficou continua viva. As células onde a tela
    /// voltou atrás enterram-se; os passos que restam já não casam com o motor.
    /// Só no `casco` do passo, se o houver: a água que correu longe durante o passo não se enterra.
    pub(super) fn wet_enterra_sem_passo(
        &self,
        sess: &mut WetSession,
        velha: &Arc<Vec<u8>>,
        casco: Option<CaixaDeCelulas>,
    ) {
        sess.bring_home();
        esquece_os_passos(sess);
        sess.enterrou = true;
        sess.otica.toca_tudo(sess.grid);
        let tocada = self.wet_enterra_no_casco(sess, velha, casco);
        // O passo deixou tinta onde ela não muda nenhum píxel (a orla fina da aguada, alfa 0): voltaria a correr. Só a dele.
        let (gw, gh) = sess.grid;
        let perto = perto_de(&tocada, gw, gh, ALCANCE_DA_ORLA);
        // Só a caixa: a água que corre noutro sítio recompõe-se no passo seguinte dela.
        if let Some(c) = perto.1.map(|c| caixa_dos_planos(c, gw, gh)) {
            self.wet_planos_em_dia(sess, c);
        }
        let g = sess.engine.active_grid();
        let invisivel: Vec<bool> = (0..gw * gh)
            .map(|k| {
                let c = (k / gw + 1) * g.s + k % gw + 1;
                perto.0[k] && sess.planos.pigment[k * 4 + 3] == 0 && g.sett[c] + g.susp[c] > 0.0
            })
            .collect();
        sess.engine.enterra_celulas(&invisivel, gw);
        sess.engine.take_dirty();
    }

    /// O enterro de `velha` × a tela de agora, como o passo do topo do undo: desfazê-lo devolve as células.
    pub(super) fn wet_enterra_como_passo(
        &self,
        sess: &mut WetSession,
        velha: &Arc<Vec<u8>>,
        casco: Option<CaixaDeCelulas>,
    ) {
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
                let desde = self.undo.chave_do_topo() - self.undo.undo_depth();
                abre_o_passo(sess, chave, Some(Arc::clone(velha)), desde);
            }
        } else {
            esquece_os_passos(sess);
        }
        sess.enterrou = true;
        // Uma imagem no ar é de antes do enterro: aplicada, punha a tinta velha de volta.
        sess.otica.toca_tudo(sess.grid);
        self.wet_enterra_no_casco(sess, velha, casco);
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
            // O que os planos têm de seguir: o que o restauro escreve (com a água viva, a caixa dela).
            let (gw, gh) = sess.grid;
            let ja_enterrou = sess.enterrou;
            let caixa = sess
                .engine
                .caixa_do_restauro(refaz)
                .map(|[x0, y0, x1, y1]| {
                    (
                        x0.clamp(1, gw),
                        y0.clamp(1, gh),
                        x1.saturating_sub(1).clamp(1, gw),
                        y1.saturating_sub(1).clamp(1, gh),
                    )
                });
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
            // Uma imagem no ar é de antes do restauro: aplicada, punha a tela de outro passo por cima, e nada a corrigia.
            sess.otica.toca_tudo(sess.grid);
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
                self.wet_enterra_como_passo(&mut sess, &tela, None);
            }
            // A tela restaurada já é a verdade. ⛔ Medido: recompor a tela inteira com células enterradas re-ilumina a
            // orla delas (3 862 px); a orla só se recalcula sob o pincel seguinte.
            if sess.enterrou {
                sess.engine.take_dirty();
                // Enterrou só agora (`tela_antes`): os planos nunca seguiram a grade.
                let c = if ja_enterrou {
                    caixa
                } else {
                    Some((1, 1, gw, gh))
                };
                if let Some(c) = c.map(|c| caixa_dos_planos(c, gw, gh)) {
                    self.wet_planos_em_dia(&mut sess, c);
                }
                if sess.congelada.is_some() {
                    self.wet_congela(&mut sess, None);
                }
            }
        }
        // Uma sessão que o restore fez nascer (não devia: o restore acaba-as) perde para a que estava viva.
        self.paint.wetpaint.session = Some(sess);
    }
}

/// O que os passos retêm de verdade, cada `Arc` uma vez: a régua do tecto.
#[cfg(test)]
pub(super) fn bytes_reais_dos_passos(sess: &WetSession) -> usize {
    bytes_retidos_nos_passos(&sess.antes_do_passo, &sess.base, sess.congelada.as_ref())
}

#[cfg(test)]
impl FotoDoMotor {
    pub(crate) fn de_para_sonda(e: &ph2d_wet_paint::painter::Engine) -> Self {
        Self::de(e, None)
    }

    pub(crate) fn repoe_para_sonda(&self, e: &mut ph2d_wet_paint::painter::Engine) {
        self.repoe(e);
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
