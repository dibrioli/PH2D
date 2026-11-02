//! **A porta SÍNCRONA da óptica pelo espelho** (J13b P4, [plano](../../../../../../../docs/Painter/Arquitetura/J13b_plano_otica_fora.md)
//! §2 passos 2–3–1, só sob `cfg(test)`): a sequência do compositor futuro, toda no fio do teste — o motor copia para
//! um [`Espelho`], [`compoe_a_otica`] corre sobre ele em planos de TRABALHO próprios (sem `CacheCampo`: a chave leva
//! a `revisao`, que anda a cada passo), as caixas copiam-se para os planos da sessão e o laço de píxel corre como hoje.

use super::composite_otica::{PlanosDaOtica, compoe_a_otica};
use ph2d_wet_paint::painter::espelho::Espelho;
use std::cell::{Cell, RefCell};

#[derive(Default)]
struct Porta {
    espelho: Espelho,
    trabalho: PlanosDaOtica,
}

thread_local! {
    static LIGADA: Cell<bool> = const { Cell::new(false) };
    static PORTA: RefCell<Option<Porta>> = const { RefCell::new(None) };
    static CAIXAS: Cell<usize> = const { Cell::new(0) };
}

/// Quantas caixas a porta compôs desde a última pergunta (o controlo: a cena passou mesmo por ela).
pub(super) fn take_caixas() -> usize {
    CAIXAS.with(|n| n.replace(0))
}

/// A composição deste fio vai pela porta?
pub(super) fn ligada() -> bool {
    LIGADA.with(Cell::get)
}

/// Corre `f` com as composições deste fio pela porta.
pub(super) fn pela_porta<R>(f: impl FnOnce() -> R) -> R {
    LIGADA.with(|l| l.set(true));
    let r = f();
    LIGADA.with(|l| l.set(false));
    r
}

/// Esquece o espelho e os planos de trabalho (uma tela nova).
pub(super) fn esquece() {
    PORTA.with(|p| *p.borrow_mut() = None);
}

/// Copia a caixa `(cx0, cy0, cx1, cy1)` (células, 1-based) de `de` para `para`; um plano vazio na óptica esvazia o
/// da sessão, um plano que nasceu lá nasce a zeros cá — o que a óptica faria aos planos da sessão.
fn copia_a_caixa(
    de: &PlanosDaOtica,
    para: &mut PlanosDaOtica,
    (cx0, cy0, cx1, cy1): (usize, usize, usize, usize),
    gw: usize,
    gh: usize,
) {
    fn plano(
        de: &[u8],
        para: &mut Vec<u8>,
        n: usize,
        k: usize,
        caixa: (usize, usize, usize, usize),
        gw: usize,
    ) {
        if de.is_empty() {
            if !para.is_empty() {
                *para = Vec::new();
            }
            return;
        }
        if para.len() != n * k {
            *para = vec![0u8; n * k];
        }
        let (cx0, cy0, cx1, cy1) = caixa;
        for cy in cy0..=cy1 {
            let (a, b) = (((cy - 1) * gw + cx0 - 1) * k, ((cy - 1) * gw + cx1) * k);
            para[a..b].copy_from_slice(&de[a..b]);
        }
    }
    let caixa = (cx0, cy0, cx1, cy1);
    plano(&de.pigment, &mut para.pigment, gw * gh, 4, caixa, gw);
    plano(&de.sombra, &mut para.sombra, gw * gh, 1, caixa, gw);
    plano(&de.grao, &mut para.grao, gw * gh, 1, caixa, gw);
}

impl crate::PainterTool {
    /// [`Self::wetpaint_composite_veiled`] pela porta (ver o módulo).
    pub(super) fn wetpaint_composite_pelo_espelho(&mut self, veil: bool) {
        let antes = self.wetpaint_antes_da_otica();
        let Some(entrada) = self.wetpaint_entrada_da_otica(antes.factos) else {
            return;
        };
        let mut p = PORTA.with(|p| p.borrow_mut().take()).unwrap_or_default();
        let sess = self
            .paint
            .wetpaint
            .session
            .as_mut()
            .expect("a entrada só existe com sessão");
        let (gw, gh) = entrada.grade;
        // 2. O motor copia para o espelho as linhas que mudaram desde a última cópia.
        sess.engine.copia_para(&mut p.espelho);
        // 3. A óptica sobre o espelho, nos planos de trabalho.
        if p.trabalho.pigment.len() != gw * gh * 4 {
            p.trabalho = PlanosDaOtica::da_grade((gw, gh));
        }
        p.trabalho.campo_cache = None;
        let saida = {
            let layers = p.espelho.camadas_render();
            compoe_a_otica(&entrada, &layers, &mut p.trabalho)
        };
        p.trabalho.campo_cache = None;
        // 1. O fio da tela copia as caixas para os planos da sessão.
        if let Some(c) = &saida.caixas {
            CAIXAS.with(|n| n.set(n.get() + 1));
            copia_a_caixa(&p.trabalho, &mut sess.planos, c.caixa_cel, gw, gh);
        }
        let ativa = sess.engine.active_layer;
        {
            let veu = match p.espelho.camadas.get(ativa) {
                Some(c) => super::veu::VeuDe::Grade(&c.grid),
                None => super::veu::VeuDe::Motor,
            };
            self.wetpaint_depois_da_otica(&antes, saida, veil, veu);
        }
        PORTA.with(|q| *q.borrow_mut() = Some(p));
    }
}
