//! **O COMPOSITOR DA ÓPTICA fora do fio da tela** (J13b P5, [plano](../../../../../../../docs/Painter/Arquitetura/J13b_plano_otica_fora.md)
//! §2–§3; filho de [`super`] por LOC). A maquinaria; o tique que pede e aplica é [`super::super::otica_no_tique`] (P6):
//!
//! - o fio principal publica um [`PedidoDeImagem`] na CAIXA DE CORREIO do worker ([`DoFioPrincipal::pede`]) e
//!   recolhe as [`RespostaDaOtica`] sem esperar ([`DoFioPrincipal::try_recv`]);
//! - o worker, num passo FECHADO e com folga ([`DoWorker::serve`]), junta a dívida ao sujo, tira-o, copia as
//!   linhas mudadas para o [`Espelho`] e entrega-o ao compositor;
//! - o compositor corre [`compoe_a_otica`] sobre o espelho (planos próprios, sem `CacheCampo`), manda a
//!   [`Imagem`] ao fio principal e devolve o espelho ao worker.
//!
//! Só existe no worker AO RELÓGIO com o interruptor [`ligada`] (`PH2D_WET_OTICA_FORA`); nunca no ritmado, no
//! roteiro nem no `wet_step_sync`. Morre com o worker: o worker sai (o `Drop` do `SimWorker`), o canal de
//! trabalho fecha e o compositor acaba no `recv`.

use super::super::composite_oleo::SemOleo;
use super::super::composite_otica::{
    CaixasDaOtica, EntradaDaOtica, PlanosDaOtica, TrabalhoDaOtica, compoe_a_otica,
};
use super::super::fita::MarcaDaAgua;
use super::super::otica_factos::FactosDaOtica;
use super::STEP_S;
use ph2d_wet_paint::oleo_real::OleoReal;
use ph2d_wet_paint::painter::espelho::Espelho;
use ph2d_wet_paint::painter::{Dirty, Engine};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;

#[path = "offthread_agenda.rs"]
mod agenda; // quando servir: a regra pura (T1-42)
use agenda::{Agenda, Momento};

/// Caixa em células, `(x0, y0, x1, y1)` inclusiva, `1..=gw × 1..=gh`.
pub(in crate::tool::paint::wetpaint) type CaixaCel = (usize, usize, usize, usize);

/// A geometria da tela com que se compõe (o que o `composite` lê da sessão e da ferramenta).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::tool::paint::wetpaint) struct FormaDaTela {
    pub(in crate::tool::paint::wetpaint) ratio: u8,
    /// `(gw, gh)`, células.
    pub(in crate::tool::paint::wetpaint) grade: (usize, usize),
    /// `(w, h)`, píxeis.
    pub(in crate::tool::paint::wetpaint) tela: (usize, usize),
}

/// O pedido de UMA imagem, escrito só pelo fio principal.
#[derive(Clone, Copy)]
pub(in crate::tool::paint::wetpaint) struct PedidoDeImagem {
    pub(in crate::tool::paint::wetpaint) factos: FactosDaOtica,
    /// Ecoada na resposta: o fio principal sabe a que pedido ela responde.
    pub(in crate::tool::paint::wetpaint) epoca: u64,
    /// A caixa de uma imagem descartada (tocou a composição local): entra no sujo antes de o tirar.
    pub(in crate::tool::paint::wetpaint) divida: Option<CaixaCel>,
    pub(in crate::tool::paint::wetpaint) forma: FormaDaTela,
    /// `show_wet`: a imagem leva o filme e a molha da caixa para o véu.
    pub(in crate::tool::paint::wetpaint) com_veu: bool,
}

/// O filme e a molha da grade ACTIVA nas linhas `linha0..` (largura `s`, a da grade): as linhas da caixa de
/// píxeis com uma de folga de cada lado (o menisco lê `film[i ± s]`).
pub(in crate::tool::paint::wetpaint) struct VeuDaCaixa {
    pub(in crate::tool::paint::wetpaint) linha0: usize,
    pub(in crate::tool::paint::wetpaint) s: usize,
    pub(in crate::tool::paint::wetpaint) film: Vec<f32>,
    pub(in crate::tool::paint::wetpaint) wet: Vec<u8>,
}

/// As caixas de uma imagem: os planos SÓ da `caixa_cel`, linha a linha.
pub(in crate::tool::paint::wetpaint) struct CaixasDaImagem {
    pub(in crate::tool::paint::wetpaint) caixa_cel: CaixaCel,
    pub(in crate::tool::paint::wetpaint) caixa_px: (usize, usize, usize, usize),
    /// `4` bytes por célula.
    pub(in crate::tool::paint::wetpaint) pigmento: Vec<u8>,
    /// `None` = o plano ficou VAZIO (sem campo do óleo), como a óptica o deixa.
    pub(in crate::tool::paint::wetpaint) sombra: Option<Vec<u8>>,
    pub(in crate::tool::paint::wetpaint) grao: Option<Vec<u8>>,
    /// O plano do Smooth Edges, na caixa dele (`None` = vazio, como a óptica o deixa).
    pub(in crate::tool::paint::wetpaint) suave: Option<Vec<u8>>,
    pub(in crate::tool::paint::wetpaint) caixa_suave: CaixaCel,
    pub(in crate::tool::paint::wetpaint) sem_oleo: SemOleo,
    pub(in crate::tool::paint::wetpaint) veu: Option<VeuDaCaixa>,
}

/// Uma imagem composta fora: o que a óptica decidiu e as caixas.
pub(in crate::tool::paint::wetpaint) struct Imagem {
    pub(in crate::tool::paint::wetpaint) oleo: Option<OleoReal>,
    pub(in crate::tool::paint::wetpaint) marca: MarcaDaAgua,
    pub(in crate::tool::paint::wetpaint) epoca: u64,
    /// Os passos do worker no instante da cópia (⇒ `seen_steps`).
    pub(in crate::tool::paint::wetpaint) passos: u64,
    /// Uma por faixa composta (B14.1); vazio = a caixa ficou vazia depois de presa à grade.
    pub(in crate::tool::paint::wetpaint) caixas: Vec<CaixasDaImagem>,
}

/// O que volta ao fio principal por pedido servido.
pub(in crate::tool::paint::wetpaint) enum RespostaDaOtica {
    /// O motor estava limpo (`Dirty::Clean`, sem dívida): nada a compor.
    Nada {
        epoca: u64,
        passos: u64,
        marca: MarcaDaAgua,
    },
    Imagem(Box<Imagem>),
}

impl CaixasDaImagem {
    /// Algum plano que `planos` tem e esta imagem diz vazio?
    pub(in crate::tool::paint::wetpaint) fn esvazia(&self, planos: &PlanosDaOtica) -> bool {
        [
            (&self.sombra, &planos.sombra),
            (&self.grao, &planos.grao),
            (&self.suave, &planos.suave),
        ]
        .iter()
        .any(|(dela, nosso)| dela.is_none() && !nosso.is_empty())
    }
}

impl Imagem {
    /// Escreve a imagem nos planos da sessão: o mesmo que [`compoe_a_otica`] lá teria escrito.
    pub(in crate::tool::paint::wetpaint) fn aplica(
        &self,
        planos: &mut PlanosDaOtica,
        grade: (usize, usize),
    ) {
        let (gw, gh) = grade;
        for c in &self.caixas {
            cola(&mut planos.pigment, 4, gw, c.caixa_cel, &c.pigmento);
            for (plano, caixa, onde) in [
                (&mut planos.sombra, &c.sombra, c.caixa_cel),
                (&mut planos.grao, &c.grao, c.caixa_cel),
                (&mut planos.suave, &c.suave, c.caixa_suave),
            ] {
                match caixa {
                    Some(b) => {
                        if plano.len() != gw * gh {
                            *plano = vec![0u8; gw * gh];
                        }
                        cola(plano, 1, gw, onde, b);
                    }
                    None if !plano.is_empty() => *plano = Vec::new(),
                    None => {}
                }
            }
        }
        if self.caixas.is_empty() {
            return;
        }
        if self.oleo.is_none() {
            planos.campo_cache = None;
        }
    }
}

/// O que o worker manda ao compositor.
struct Trabalho {
    espelho: Box<Espelho>,
    /// `trabalho.caixa` = a envolvente das `faixas`.
    entrada: EntradaDaOtica,
    /// O sujo em pedaços, cada um preso à grade (B14.1).
    faixas: Vec<CaixaCel>,
    epoca: u64,
    passos: u64,
    /// A camada activa, se o pedido quer o véu.
    veu: Option<usize>,
}

/// A caixa de correio entre o fio principal e o worker.
#[derive(Default)]
struct Correio {
    pedido: Option<PedidoDeImagem>,
    /// `(época, caixa)` do último pedido servido: o sujo saiu do motor e a imagem dele vem a caminho.
    servido: Option<(u64, CaixaCel)>,
}

/// A ponta do fio principal.
pub(in crate::tool::paint::wetpaint) struct DoFioPrincipal {
    correio: Arc<Mutex<Correio>>,
    imagens: Receiver<RespostaDaOtica>,
    /// O fio do compositor, para os `leak_tests` verem que acabou (a thread é solta de propósito).
    #[cfg(test)]
    pub(super) fio: Option<std::thread::JoinHandle<()>>,
}

impl DoFioPrincipal {
    /// Publica `p`; devolve o pedido ainda por servir que ele substituiu (a dívida dele é de quem pede).
    pub(in crate::tool::paint::wetpaint) fn pede(
        &self,
        p: PedidoDeImagem,
    ) -> Option<PedidoDeImagem> {
        trava(&self.correio).pedido.replace(p)
    }

    /// O sujo que o pedido `epoca` levou do motor: a dívida dele por servir e a caixa já servida. Só vale com o motor em
    /// casa: o worker não está a meio de um `serve`.
    pub(in crate::tool::paint::wetpaint) fn sujo_do_pedido(
        &self,
        epoca: u64,
    ) -> [Option<CaixaCel>; 2] {
        let c = trava(&self.correio);
        [
            c.pedido.filter(|p| p.epoca == epoca).and_then(|p| p.divida),
            c.servido.filter(|s| s.0 == epoca).map(|s| s.1),
        ]
    }

    /// A próxima resposta, sem esperar.
    pub(in crate::tool::paint::wetpaint) fn try_recv(&self) -> Option<RespostaDaOtica> {
        self.imagens.try_recv().ok()
    }
}

/// A ponta do worker: a caixa de correio, o espelho (quando está com ele) e a previsão da cópia.
pub(super) struct DoWorker {
    correio: Arc<Mutex<Correio>>,
    para_o_compositor: Sender<Trabalho>,
    volta: Receiver<Box<Espelho>>,
    ui: Sender<RespostaDaOtica>,
    espelho: Option<Box<Espelho>>,
    agenda: Agenda,
    relogio: Instant,
}

fn trava<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
thread_local! {
    static FORCA: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// A porta de teste do interruptor: lida por [`ligada`] no fio que sobe o worker.
#[cfg(test)]
pub(in crate::tool::paint::wetpaint) fn forca_no_teste(v: Option<bool>) {
    FORCA.with(|c| c.set(v));
}

/// O compositor existe? (`PH2D_WET_OTICA_FORA`, [`crate::wet_diag::otica_fora`]).
pub(in crate::tool::paint::wetpaint) fn ligada() -> bool {
    #[cfg(test)]
    if let Some(v) = FORCA.with(std::cell::Cell::get) {
        return v;
    }
    crate::wet_diag::otica_fora()
}

/// Sobe o compositor e devolve as duas pontas.
pub(super) fn par() -> (DoWorker, DoFioPrincipal) {
    let correio = Arc::new(Mutex::new(Correio::default()));
    let (para_o_compositor, rx) = channel::<Trabalho>();
    let (de_volta, volta) = channel::<Box<Espelho>>();
    let (ui, imagens) = channel::<RespostaDaOtica>();
    let ui_c = ui.clone();
    let fio = std::thread::Builder::new()
        .name("ph2d-wet-otica".into())
        .spawn(move || compositor(&rx, &de_volta, &ui_c))
        .expect("a thread do compositor da optica");
    #[cfg(not(test))]
    drop(fio);
    (
        DoWorker {
            correio: Arc::clone(&correio),
            para_o_compositor,
            volta,
            ui,
            espelho: Some(Box::default()),
            agenda: Agenda::default(),
            relogio: Instant::now(),
        },
        DoFioPrincipal {
            correio,
            imagens,
            #[cfg(test)]
            fio: Some(fio),
        },
    )
}

/// O sujo do motor preso à grade (o mesmo do `composite`).
pub(in crate::tool::paint::wetpaint) fn caixa_da_grade(
    d: Dirty,
    (gw, gh): (usize, usize),
) -> Option<CaixaCel> {
    match d {
        Dirty::Clean => None,
        Dirty::Full => Some((1, 1, gw, gh)),
        Dirty::Rect { x0, y0, x1, y1 } => Some((
            (x0.max(1) as usize).min(gw),
            (y0.max(1) as usize).min(gh),
            (x1.max(1) as usize).min(gw),
            (y1.max(1) as usize).min(gh),
        )),
    }
}

impl DoWorker {
    /// O gancho do `worker_loop`, num passo FECHADO: serve o pedido se o espelho está aqui e a [`Agenda`] deixa (a
    /// cópia cabe na folga, ou ocupa ≤ 20 % do worker com a água atrasada). O correio lê-se primeiro: sem pedido
    /// nada anda.
    pub(super) fn serve(&mut self, engine: &mut Engine, acc: f64, passos: u64) {
        if engine.sim.step_pending() {
            return;
        }
        if self.espelho.is_none() {
            self.espelho = self.volta.try_recv().ok();
        }
        if self.espelho.is_none() {
            return;
        }
        if trava(&self.correio).pedido.is_none() {
            return;
        }
        let m = Momento {
            agora_s: self.relogio.elapsed().as_secs_f64(),
            acc,
            sim_corre: engine.sim_should_run(),
        };
        if !self.agenda.deve_servir(m) {
            return;
        }
        let Some(p) = trava(&self.correio).pedido.take() else {
            return;
        };
        if let Some((x0, y0, x1, y1)) = p.divida {
            let i = |v: usize| i32::try_from(v).unwrap_or(i32::MAX);
            engine.merge_dirty_otica(i(x0), i(y0), i(x1), i(y1));
        }
        let marca = MarcaDaAgua::de(engine);
        // B14.1: o sujo do passo da sim reduz-se às faixas de células que diferem do espelho (a entrada da imagem
        // anterior).
        let sujo = match self.espelho.as_deref() {
            Some(e) => engine.take_dirty_pela_otica_em_faixas(e),
            None => vec![engine.take_dirty()],
        };
        let faixas: Vec<CaixaCel> = sujo
            .into_iter()
            .filter_map(|d| caixa_da_grade(d, p.forma.grade))
            .collect();
        let Some(caixa) = faixas.iter().copied().reduce(|a, b| super::super::otica_no_tique::une(Some(a), b)) else {
            let _ = self.ui.send(RespostaDaOtica::Nada {
                epoca: p.epoca,
                passos,
                marca,
            });
            return;
        };
        trava(&self.correio).servido = Some((p.epoca, caixa));
        let Some(mut espelho) = self.espelho.take() else {
            unreachable!("conferido acima")
        };
        let t = Instant::now();
        engine.copia_para(&mut espelho);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        self.agenda.copiou(ms, self.relogio.elapsed().as_secs_f64());
        crate::wet_diag::note_copia(ms as f32);
        let trabalho = Trabalho {
            espelho,
            entrada: EntradaDaOtica {
                factos: p.factos,
                marca,
                trabalho: Some(TrabalhoDaOtica {
                    caixa,
                    params: engine.sim.gather_params(&engine.tuning),
                }),
                ratio: p.forma.ratio,
                grade: p.forma.grade,
                tela: p.forma.tela,
                revisao: engine.revisao(),
            },
            faixas,
            epoca: p.epoca,
            passos,
            veu: p.com_veu.then_some(engine.active_layer),
        };
        if let Err(e) = self.para_o_compositor.send(trabalho) {
            self.espelho = Some(e.0.espelho);
        }
    }
}

/// O laço do compositor: até o worker fechar o canal de trabalho.
fn compositor(rx: &Receiver<Trabalho>, volta: &Sender<Box<Espelho>>, ui: &Sender<RespostaDaOtica>) {
    let mut planos = PlanosDaOtica::default();
    while let Ok(t) = rx.recv() {
        let t0 = Instant::now();
        let (espelho, imagem) = no_pool(|| compoe(t, &mut planos));
        crate::wet_diag::note_otica(t0.elapsed().as_secs_f32() * 1000.0);
        // O espelho volta primeiro: o worker pode copiar a próxima enquanto a tela aplica esta.
        let _ = volta.send(espelho);
        let _ = ui.send(RespostaDaOtica::Imagem(Box::new(imagem)));
    }
}

/// Corre `f` DENTRO do pool do rayon, pela razão do `PainterTool::no_pool` (`por_quadro.rs`): os passes paralelos
/// lançados de fora do pool entram pela fila global e adormecem quem chama no trinco.
fn no_pool<R: Send>(f: impl FnOnce() -> R + Send) -> R {
    if rayon::current_thread_index().is_some()
        || !crate::wet_diag::NO_POOL.load(std::sync::atomic::Ordering::Relaxed)
    {
        return f();
    }
    rayon::join(f, || ()).0
}

/// A metade de cima sobre o espelho, e a imagem tirada dos planos de trabalho.
fn compoe(t: Trabalho, planos: &mut PlanosDaOtica) -> (Box<Espelho>, Imagem) {
    let (gw, gh) = t.entrada.grade;
    if planos.pigment.len() != gw * gh * 4 {
        *planos = PlanosDaOtica::da_grade((gw, gh));
    }
    let layers = t.espelho.camadas_render();
    let faixas = super::super::composite_otica::faixas_da_otica(&t.entrada, &layers, &t.faixas);
    let oleo = t.entrada.factos.oleo_da_tela(&layers);
    let mut caixas = Vec::with_capacity(faixas.len());
    let mut entrada = t.entrada;
    for f in faixas {
        if let Some(tr) = entrada.trabalho.as_mut() {
            tr.caixa = f;
        }
        // Sem cache: a chave leva a `revisao`, que anda a cada passo (J13 §2); o campo é exacto sem ela.
        planos.campo_cache = None;
        let saida = compoe_a_otica(&entrada, &layers, planos);
        caixas.extend(saida.caixas.map(|c| {
            let veu = t
                .veu
                .and_then(|i| t.espelho.camadas.get(i))
                .map(|c_| veu_da_caixa(&c_.grid, c.caixa_px, entrada.ratio, (gw, gh)));
            caixas_da_imagem(c, planos, (gw, gh), veu)
        }));
    }
    planos.campo_cache = None;
    drop(layers);
    let imagem = Imagem {
        oleo,
        marca: entrada.marca,
        epoca: t.epoca,
        passos: t.passos,
        caixas,
    };
    (t.espelho, imagem)
}

fn caixas_da_imagem(
    c: CaixasDaOtica,
    planos: &PlanosDaOtica,
    (gw, gh): (usize, usize),
    veu: Option<VeuDaCaixa>,
) -> CaixasDaImagem {
    let corta = |plano: &[u8]| (plano.len() == gw * gh).then(|| recorta(plano, 1, gw, c.caixa_cel));
    CaixasDaImagem {
        caixa_cel: c.caixa_cel,
        caixa_px: c.caixa_px,
        pigmento: recorta(&planos.pigment, 4, gw, c.caixa_cel),
        sombra: corta(&planos.sombra),
        grao: corta(&planos.grao),
        suave: (planos.suave.len() == gw * gh)
            .then(|| recorta(&planos.suave, 1, gw, c.caixa_suave)),
        caixa_suave: c.caixa_suave,
        sem_oleo: c.sem_oleo,
        veu,
    }
}

/// As linhas de células que a caixa de píxeis toca, com uma de folga de cada lado.
fn veu_da_caixa(
    g: &ph2d_wet_paint::grid::Grid,
    (_, py0, _, py1): (usize, usize, usize, usize),
    ratio: u8,
    (_, gh): (usize, usize),
) -> VeuDaCaixa {
    let rr = usize::from(ratio).max(1);
    let lo = (py0 / rr + 1).min(gh);
    let hi = (py1.saturating_sub(1) / rr + 1).min(gh);
    let (a, b) = (lo - 1, (hi + 1).min(g.rows - 1));
    VeuDaCaixa {
        linha0: a,
        s: g.s,
        film: g.film[a * g.s..(b + 1) * g.s].to_vec(),
        wet: g.wet[a * g.s..(b + 1) * g.s].to_vec(),
    }
}

/// Os bytes da caixa de um plano da grade (`bpp` bytes por célula, linha `gw`).
fn recorta(plano: &[u8], bpp: usize, gw: usize, (x0, y0, x1, y1): CaixaCel) -> Vec<u8> {
    let mut v = Vec::with_capacity((x1 + 1 - x0) * (y1 + 1 - y0) * bpp);
    for cy in y0..=y1 {
        let o = (cy - 1) * gw * bpp;
        v.extend_from_slice(&plano[o + (x0 - 1) * bpp..o + x1 * bpp]);
    }
    v
}

/// O inverso de [`recorta`].
fn cola(plano: &mut [u8], bpp: usize, gw: usize, (x0, y0, x1, y1): CaixaCel, b: &[u8]) {
    let cw = (x1 + 1 - x0) * bpp;
    for (k, cy) in (y0..=y1).enumerate() {
        let o = (cy - 1) * gw * bpp + (x0 - 1) * bpp;
        plano[o..o + cw].copy_from_slice(&b[k * cw..(k + 1) * cw]);
    }
}

/// A sessão não tem compositor: o pedido de imagem não foi publicado.
#[derive(Debug)]
pub(in crate::tool::paint::wetpaint) struct SemCompositor;

impl super::super::WetSession {
    /// Publica um pedido de imagem; `Err(SemCompositor)` = não há compositor (interruptor desligado, worker
    /// ritmado ou ainda sem worker) e o pedido cai. `Ok(Some(antigo))` = o pedido ainda por servir que este
    /// substituiu.
    pub(in crate::tool::paint::wetpaint) fn pede_imagem(
        &self,
        p: PedidoDeImagem,
    ) -> Result<Option<PedidoDeImagem>, SemCompositor> {
        let o = self
            .worker
            .as_ref()
            .and_then(|w| w.otica.as_ref())
            .ok_or(SemCompositor)?;
        Ok(o.pede(p))
    }

    /// Esta sessão tem compositor (o worker ao relógio subiu-o)?
    pub(in crate::tool::paint::wetpaint) fn tem_compositor(&self) -> bool {
        self.worker.as_ref().is_some_and(|w| w.otica.is_some())
    }

    /// [`DoFioPrincipal::sujo_do_pedido`] desta sessão.
    pub(in crate::tool::paint::wetpaint) fn sujo_do_pedido(
        &self,
        epoca: u64,
    ) -> [Option<CaixaCel>; 2] {
        self.worker
            .as_ref()
            .and_then(|w| w.otica.as_ref())
            .map_or([None, None], |o| o.sujo_do_pedido(epoca))
    }

    /// A próxima resposta do compositor, sem esperar.
    pub(in crate::tool::paint::wetpaint) fn try_recv_imagem(&self) -> Option<RespostaDaOtica> {
        self.worker.as_ref()?.otica.as_ref()?.try_recv()
    }
}

#[cfg(test)]
#[path = "offthread_otica_tests.rs"]
mod tests;
