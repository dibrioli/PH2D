//! **Uma macro do Painter em cena** (ADR-0186, W4 item 5.5 D): a trilha `"painter"` de um
//! `ph2d_ecs::Macro`, num [`TocadorDoPainter`] próprio, a seguir o instante que a Timeline escreveu
//! (`ph2d_ecs::MacroTempo`). Para a frente despacha os passos até lá, e no início de cada etapa guarda um
//! instantâneo do tocador ([`super::instantaneos`]); para trás recomeça do instantâneo mais tardio que
//! ainda não passou o instante pedido (ou do estado inicial gravado) e avança — a lei do
//! `MacroPlayer::seek`, encurtada pelos instantâneos. Uma macro editada ([`MacroEmCena::reedita`])
//! recomeça do último instantâneo que a trilha nova ainda tem.
//!
//! Um passo que falha PÁRA a macro nesse passo e o erro fica guardado ([`MacroEmCena::erro`]); recuar
//! para antes dele recomeça e limpa-o.

use super::FAMILIA;
use super::encaixe::Encaixe;
use super::etapas::LeitorDoPainter;
use super::instantaneos::Instantaneos;
use super::tocar::{ErroDaMacro, Recomeco, TocadorDoPainter};
use ph2d_core::razao::Razao;
use ph2d_macro::{LeitorDaFamilia, MacroDoc, MacroPlayer, MacroTime};
use std::sync::Arc;

pub struct MacroEmCena {
    /// Só a trilha do Painter, já a que TOCA (as edições aplicadas): as outras famílias tocam a sua
    /// lendo o mesmo componente.
    doc: MacroDoc,
    /// O 1.º evento de cada etapa, na trilha que toca.
    inicios: Vec<usize>,
    tocador: TocadorDoPainter,
    player: MacroPlayer,
    erro: Option<ErroDaMacro>,
    fotos: Instantaneos,
    /// A etapa de cujo instantâneo a última volta atrás recomeçou (`None` = do estado inicial).
    recomecou_de: Option<usize>,
    /// Onde a pintura cai por cima ([`Self::poe_encaixe`]).
    encaixe: Option<Encaixe>,
    /// Qual das trilhas do Painter do `Macro.doc` toca aqui (W19: várias imagens no mesmo objecto).
    indice: usize,
}

fn tocador_de(doc: &MacroDoc, encaixe: Option<Encaixe>) -> Result<TocadorDoPainter, ErroDaMacro> {
    let t = &doc.tracks[0];
    let mut toc =
        TocadorDoPainter::novo(&t.initial, t.events.iter().map(|e| e.payload.as_slice()))?;
    toc.poe_encaixe(encaixe);
    Ok(toc)
}

/// A `indice`-ésima trilha do Painter de um `Macro.doc`, já a que toca. `Ok(None)` = não há.
fn le(bytes: &[u8], indice: usize) -> Result<Option<MacroDoc>, ErroDaMacro> {
    let mut doc = ph2d_macro::codec::read(bytes).map_err(ErroDaMacro::Ficheiro)?;
    doc.tracks.retain(|t| t.family == FAMILIA);
    if doc.tracks.len() <= indice {
        return Ok(None);
    }
    doc.tracks.swap(0, indice);
    doc.tracks.truncate(1);
    doc.tracks[0] = doc.tracks[0]
        .tocavel(&LeitorDoPainter)
        .map_err(ErroDaMacro::Edicao)?;
    Ok(Some(doc))
}

fn inicios_de(doc: &MacroDoc) -> Vec<usize> {
    LeitorDoPainter
        .etapas(&doc.tracks[0].events)
        .iter()
        .map(|e| e.inicio)
        .collect()
}

/// O tocador de uma macro nunca grava nem tem worker ou ponte de dispositivo: copia-se sempre.
fn copia(t: &TocadorDoPainter) -> TocadorDoPainter {
    t.instantaneo()
        .expect("o tocador de uma macro não grava nem tem worker nem ponte de dispositivo")
}

impl MacroEmCena {
    /// A macro de um `Macro.doc`, no estado inicial gravado. `Ok(None)` = sem trilha do Painter.
    ///
    /// # Errors
    /// [`ErroDaMacro::Ficheiro`] (o `doc` não se lê) e os do [`TocadorDoPainter::novo`].
    pub fn de(bytes: &[u8]) -> Result<Option<Self>, ErroDaMacro> {
        Self::no_razao(bytes, super::instantaneos::razao())
    }

    /// Como [`Self::de`], com os instantâneos noutro razão (os gates usam um próprio).
    ///
    /// # Errors
    /// Os do [`Self::de`].
    pub fn no_razao(bytes: &[u8], razao: Arc<Razao>) -> Result<Option<Self>, ErroDaMacro> {
        Self::da_trilha(bytes, 0, razao)
    }

    /// Como [`Self::no_razao`], com a `indice`-ésima trilha do Painter do doc. `Ok(None)` = não há.
    ///
    /// # Errors
    /// Os do [`Self::de`].
    pub fn da_trilha(
        bytes: &[u8],
        indice: usize,
        razao: Arc<Razao>,
    ) -> Result<Option<Self>, ErroDaMacro> {
        let Some(doc) = le(bytes, indice)? else {
            return Ok(None);
        };
        let tocador = tocador_de(&doc, None)?;
        let player = MacroPlayer::new(&doc);
        Ok(Some(Self {
            inicios: inicios_de(&doc),
            doc,
            tocador,
            player,
            erro: None,
            fotos: Instantaneos::em(razao),
            recomecou_de: None,
            encaixe: None,
            indice,
        }))
    }

    /// **Segue o relógio da macro até `segundos`.**
    ///
    /// # Errors
    /// O [`ErroDaMacro`] do passo que parou a macro (também guardado em [`Self::erro`]).
    pub fn segue(&mut self, segundos: f64) -> Result<(), ErroDaMacro> {
        let to = MacroTime::from_secs_f64(segundos);
        let recua = to < self.player.time();
        if !recua && let Some(e) = &self.erro {
            return Err(e.clone());
        }
        if recua {
            self.recomeca_ate(to)?;
        }
        self.avanca(to)
    }

    /// O tocador volta ao instantâneo mais tardio até `to`, ou ao estado inicial.
    fn recomeca_ate(&mut self, to: MacroTime) -> Result<(), ErroDaMacro> {
        self.erro = None;
        if let Some(f) = self.fotos.ate(to) {
            self.tocador = copia(&f.tocador);
            self.player = f.player.clone();
            self.recomecou_de = Some(f.etapa);
            return Ok(());
        }
        self.recomecou_de = None;
        self.player = MacroPlayer::new(&self.doc);
        match tocador_de(&self.doc, self.encaixe) {
            Ok(t) => {
                self.tocador = t;
                Ok(())
            }
            Err(e) => {
                self.erro = Some(e.clone());
                Err(e)
            }
        }
    }

    /// Avança até `to`, a guardar o instantâneo de cada etapa que começa pelo caminho.
    fn avanca(&mut self, to: MacroTime) -> Result<(), ErroDaMacro> {
        let feitos = self.player.dispatched();
        let primeira = self.inicios.partition_point(|&i| i < feitos);
        for k in primeira..self.inicios.len() {
            let i = self.inicios[k];
            if i > 0 && self.doc.tracks[0].events[i - 1].t > to {
                break;
            }
            self.despacha(to, i)?;
            if self.fotos.quer(k) {
                let t = copia(&self.tocador);
                self.fotos.guarda(k, self.player.clone(), t);
            }
        }
        self.despacha(to, usize::MAX)
    }

    fn despacha(&mut self, to: MacroTime, limite: usize) -> Result<(), ErroDaMacro> {
        let Self {
            doc,
            tocador,
            player,
            ..
        } = self;
        let feito = player
            .advance_to_limit(doc, to, limite, |_, _, ev| tocador.aplica(&ev.payload))
            .map(|_| ());
        if let Err(e) = &feito {
            self.erro = Some(e.clone());
        }
        feito
    }

    /// **A macro mudou** (uma etapa editada, `Macro.doc` novo): fica com os instantâneos que a trilha
    /// nova ainda tem — a mesma inicial, os passos despachados até eles iguais, as respostas do worker
    /// que já consumiram iguais — e recomeça do mais tardio. `Ok(false)` = já não há trilha do Painter.
    ///
    /// # Errors
    /// Os do [`Self::de`] (este palco deixa de valer: quem chama larga-o).
    pub fn reedita(&mut self, bytes: &[u8]) -> Result<bool, ErroDaMacro> {
        let Some(doc) = le(bytes, self.indice)? else {
            return Ok(false);
        };
        let tr = &doc.tracks[0];
        let recomeco = Recomeco::de(&tr.initial, tr.events.iter().map(|e| e.payload.as_slice()))?;
        let (velho, novo) = (&self.doc.tracks[0], &doc.tracks[0]);
        let comum = if velho.initial == novo.initial && velho.target == novo.target {
            velho
                .events
                .iter()
                .zip(&novo.events)
                .take_while(|(a, b)| a == b)
                .count()
        } else {
            0
        };
        self.fotos.refaz(|f| {
            if f.player.dispatched() > comum {
                return None;
            }
            let t = f.tocador.recomeca(&recomeco)?;
            Some(Instantaneos::foto(f.etapa, f.player.clone(), t))
        });
        self.inicios = inicios_de(&doc);
        self.doc = doc;
        self.recomeca_ate(MacroTime(u64::MAX))?;
        Ok(true)
    }

    /// **O objecto Macro mudou de sítio** (aplicar por cima, W18 §2.7): o encaixe lê-se vivo, nunca se
    /// assa — um encaixe diferente larga os instantâneos (pintados no sítio velho) e a reprodução recomeça
    /// do estado inicial. Numa trilha sem tamanho gravado o encaixe não muda nada.
    pub fn poe_encaixe(&mut self, e: Option<Encaixe>) {
        if e == self.encaixe {
            return;
        }
        self.encaixe = e;
        if super::inicial::tamanho_gravado(&self.doc.tracks[0].initial).is_none() {
            return;
        }
        self.fotos.refaz(|_| None);
        // O erro de recomeçar fica em [`Self::erro`], como no [`Self::reedita`].
        let _ = self.recomeca_ate(MacroTime(0));
    }

    /// O encaixe posto ([`Self::poe_encaixe`]).
    #[must_use]
    pub fn encaixe(&self) -> Option<Encaixe> {
        self.encaixe
    }

    /// O tamanho da tela da gravação, quando a trilha toca por cima de outra imagem (etiqueta 6).
    #[must_use]
    pub fn tamanho_gravado(&self) -> Option<(u32, u32)> {
        super::inicial::tamanho_gravado(&self.doc.tracks[0].initial)
    }

    /// **Um ponto de partida para conferir `track`** (a trilha com as edições, [`super::etapas::confere`]):
    /// o instantâneo mais tardio que ainda é um estado dela — os passos despachados são prefixo comum
    /// desta macro e da trilha que toca de `track`, até ao [`super::etapas::limite_da_partida`] —, com a
    /// fita dela. `None` = do zero.
    #[must_use]
    pub fn partida(&self, track: &ph2d_macro::Track) -> Option<super::etapas::Partida> {
        let novo = track.tocavel(&LeitorDoPainter).ok()?;
        let velho = &self.doc.tracks[0];
        if velho.initial != novo.initial || velho.target != novo.target {
            return None;
        }
        let comum = velho
            .events
            .iter()
            .zip(&novo.events)
            .take_while(|(a, b)| a == b)
            .count();
        let limite = super::etapas::limite_da_partida(track).ok()?.min(comum);
        let f = self.fotos.ate_feitos(limite)?;
        let recomeco = Recomeco::de(
            &novo.initial,
            novo.events.iter().map(|e| e.payload.as_slice()),
        )
        .ok()?;
        Some(super::etapas::Partida {
            tocador: f.tocador.recomeca(&recomeco)?,
            feitos: f.player.dispatched(),
        })
    }

    /// A tela para a cena, se mudou desde a última leitura ([`TocadorDoPainter::toma_a_tela`]).
    pub fn toma_a_tela(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        self.tocador.toma_a_tela()
    }

    /// O retângulo que mudou na última [`Self::toma_a_tela`] — `None` = a tela inteira.
    pub fn caixa_da_tela(&mut self) -> Option<(u32, u32, u32, u32)> {
        self.tocador.caixa_da_tela()
    }

    #[must_use]
    pub fn tocador(&self) -> &TocadorDoPainter {
        &self.tocador
    }

    /// Quando a trilha começa no relógio da macro: o nascimento do objecto, senão o 1.º passo.
    #[must_use]
    pub fn inicio(&self) -> MacroTime {
        let t = &self.doc.tracks[0];
        t.nasce
            .as_ref()
            .map(|n| n.inicio)
            .or_else(|| t.events.first().map(|e| e.t))
            .unwrap_or(MacroTime(0))
    }

    /// A sprite que a macro pinta (o `target` da trilha, `StableId`; `0` = a do próprio `Macro`).
    #[must_use]
    pub fn alvo(&self) -> u64 {
        self.doc.tracks[0].target
    }

    /// **A água depois do fim do clipe** (W14, decisão do dono 20/10): com todos os passos tocados, a água
    /// da gravação continua `dt_ms`, até assentar ou até `ate` (onde a imagem a guardou,
    /// [`super::marca::MarcaDoArtista::agua_da_gravacao_parou`]). `false` = já não anda.
    pub fn corre_a_cauda(&mut self, dt_ms: f32, ate: Option<super::MarcaDaAgua>) -> bool {
        let no_fim = self.player.dispatched() >= self.doc.tracks[0].events.len();
        no_fim && self.erro.is_none() && self.tocador.corre_a_cauda(dt_ms, ate)
    }

    /// O tempo até onde a macro já tocou.
    #[must_use]
    pub fn tempo(&self) -> MacroTime {
        self.player.time()
    }

    /// O passo que parou a macro, se algum.
    #[must_use]
    pub fn erro(&self) -> Option<&ErroDaMacro> {
        self.erro.as_ref()
    }

    /// As etapas que têm instantâneo agora, e de quantas em quantas se tira.
    #[must_use]
    pub fn instantaneos(&self) -> (Vec<usize>, usize) {
        (self.fotos.etapas(), self.fotos.passo())
    }

    /// A etapa de cujo instantâneo a última volta atrás (ou edição) recomeçou; `None` = do estado inicial.
    #[must_use]
    pub fn recomecou_de(&self) -> Option<usize> {
        self.recomecou_de
    }
}
