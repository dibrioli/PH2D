//! **O tocador da trilha**: um [`PainterTool`] PRÓPRIO (nunca a ferramenta ativa do editor), no estado
//! inicial, que recebe cada passo pela MESMA porta que o gravou. As respostas do worker da água
//! ([`Passo::Agua`]) carregam-se todas à partida numa fita que responde pela ordem em que a thread do
//! quadro as pede — então a ordem delas no tempo da macro não importa.
//!
//! Uma divergência (a fita não responde o que se pergunta, uma marca inalcançável), uma porta recusada
//! ou um passo desconhecido PÁRAM o tocador em voz alta ([`ErroDaMacro`]): nunca se salta um passo.

use super::inicial::{self, ErroDoInicial};
use super::passo::{Passo, abs, faixa_de};
use crate::tool::paint::fita::{
    self, Divergencia, Fita, FitaPartilhada, MarcaDaAgua, RespostaDoWorker,
};
use crate::tool::paint::tracos::{PincelDoTraco, TracoGuardado};
use crate::tool::{PainterTool, SemInstantaneo};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, RasterEditTool, Tool};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroDaMacro {
    /// O `ph2d_ecs::Macro.doc` não se lê.
    Ficheiro(ph2d_macro::CodecError),
    Inicial(ErroDoInicial),
    Passo(ph2d_macro::CodecError),
    /// A gravação passou por uma porta que a macro não reproduz.
    PortaRecusada(String),
    /// Um passo de uma versão mais nova.
    PassoDesconhecido(u64),
    /// A reprodução deixou de ser a gravação.
    Divergiu(Divergencia),
    Preset(String),
    /// Uma edição de etapa que não cabe na gravação.
    Edicao(ph2d_macro::ErroDeEdicao),
    /// O fio que calculava algo da macro (o nome dele) entrou em pânico.
    FioCaiu(&'static str),
}

pub struct TocadorDoPainter {
    tool: PainterTool,
    fita: FitaPartilhada,
    /// Um passo de pré-visualização viu a tela mudar desde o último [`Self::toma_a_tela`] (nasce
    /// `true`: a tela inicial também se mostra).
    mudou: bool,
    /// As respostas do worker que a trilha carregou, pela ordem (a fita consome-as de frente).
    respostas: Arc<[RespostaDoWorker]>,
    /// A água ainda responde pela fita (`false` depois de [`Self::passa_ao_relogio_fixo`]).
    fita_viva: bool,
    /// A cena ainda não leu ESTE tocador (nasce `true`, e numa cópia também): a caixa suja dele é a do
    /// tocador de que veio, não o que difere do que a cena tem — a 1.ª [`Self::caixa_da_tela`] é a tela
    /// inteira (o arrasto da Timeline, gate `palcos_arrasto_tests`).
    por_ler: bool,
    /// Os traços das faixas que não estão no tool (W16: pinceladas sobrepostas, uma por faixa).
    faixas: BTreeMap<u32, TracoGuardado>,
    /// O pincel que cada faixa leva ([`Passo::PincelDaFaixa`]), lido antes de tocar.
    pinceis: BTreeMap<u32, PincelDoTraco>,
    /// As faixas que já abriram um traço (o pincel delas já pintou).
    pintaram: BTreeSet<u32>,
}

/// **O que um instantâneo precisa de uma trilha nova para recomeçar nela** ([`TocadorDoPainter::recomeca`]):
/// as respostas do worker e o pincel de cada faixa.
pub struct Recomeco {
    respostas: Arc<[RespostaDoWorker]>,
    pinceis: BTreeMap<u32, PincelDoTraco>,
}

impl Recomeco {
    /// De todas as cargas da trilha que toca nova.
    ///
    /// # Errors
    /// [`ErroDaMacro`].
    pub fn de<'a>(
        inicial: &[u8],
        cargas: impl IntoIterator<Item = &'a [u8]>,
    ) -> Result<Self, ErroDaMacro> {
        let cargas: Vec<&[u8]> = cargas.into_iter().collect();
        Ok(Self {
            pinceis: super::etapas::pinceis_das_faixas(inicial, &cargas)?,
            respostas: respostas_de(cargas)?.into(),
        })
    }
}

impl From<Arc<[RespostaDoWorker]>> for Recomeco {
    /// Uma trilha sem etapas deslocadas (sem faixas).
    fn from(respostas: Arc<[RespostaDoWorker]>) -> Self {
        Self {
            respostas,
            pinceis: BTreeMap::new(),
        }
    }
}

/// As respostas do worker de uma trilha, pela ordem das cargas.
///
/// # Errors
/// [`ErroDaMacro::Passo`].
pub fn respostas_de<'a>(
    cargas: impl IntoIterator<Item = &'a [u8]>,
) -> Result<Vec<RespostaDoWorker>, ErroDaMacro> {
    let mut out = Vec::new();
    for c in cargas {
        if let Passo::Agua(r) = Passo::descodifica(c).map_err(ErroDaMacro::Passo)? {
            out.push(r);
        }
    }
    Ok(out)
}

impl TocadorDoPainter {
    /// No estado inicial gravado (o documento E as configurações). `cargas` são TODAS as cargas da
    /// trilha (as respostas do worker tiram-se daqui).
    ///
    /// # Errors
    /// [`ErroDaMacro`].
    pub fn novo<'a>(
        inicial: &[u8],
        cargas: impl IntoIterator<Item = &'a [u8]>,
    ) -> Result<Self, ErroDaMacro> {
        let tool = inicial::tool_no_inicial(inicial).map_err(ErroDaMacro::Inicial)?;
        Self::com_tool(inicial, tool, cargas)
    }

    /// Com as configurações do estado inicial mas sobre OUTRA tela (`rgba` `w × h`): as posições caem no
    /// mesmo lugar relativo.
    ///
    /// # Errors
    /// [`ErroDaMacro`].
    pub fn sobre_tela<'a>(
        inicial: &[u8],
        cargas: impl IntoIterator<Item = &'a [u8]>,
        rgba: Vec<u8>,
        w: u32,
        h: u32,
    ) -> Result<Self, ErroDaMacro> {
        let mut tool = inicial::tool_no_inicial(inicial).map_err(ErroDaMacro::Inicial)?;
        tool.bind_document(inicial::ENTIDADE_DO_TOCADOR + 1, rgba, w, h);
        Self::com_tool(inicial, tool, cargas)
    }

    fn com_tool<'a>(
        inicial: &[u8],
        mut tool: PainterTool,
        cargas: impl IntoIterator<Item = &'a [u8]>,
    ) -> Result<Self, ErroDaMacro> {
        let cargas: Vec<&[u8]> = cargas.into_iter().collect();
        let pinceis = super::etapas::pinceis_das_faixas(inicial, &cargas)?;
        let respostas: Arc<[RespostaDoWorker]> = respostas_de(cargas)?.into();
        let fita: FitaPartilhada = Arc::new(Mutex::new(Fita::Toca {
            fila: respostas.iter().copied().collect(),
            divergiu: None,
        }));
        tool.wet_instala_fita(Some(Arc::clone(&fita)));
        Ok(Self {
            tool,
            fita,
            mudou: true,
            respostas,
            fita_viva: true,
            por_ler: true,
            faixas: BTreeMap::new(),
            pinceis,
            pintaram: BTreeSet::new(),
        })
    }

    /// **O instantâneo deste tocador**: continuar a cópia dá, ao bit, o que este daria ([`PainterTool::instantaneo`];
    /// a fita é dela).
    ///
    /// # Errors
    /// [`SemInstantaneo`] — o tool do tocador nunca grava, nunca tem worker nem ponte de dispositivo.
    pub fn instantaneo(&self) -> Result<Self, SemInstantaneo> {
        let tool = self.tool.instantaneo()?;
        let fita = tool
            .wet_fita()
            .unwrap_or_else(|| Arc::new(Mutex::new(fita::trava(&self.fita).clone())));
        Ok(Self {
            tool,
            fita,
            mudou: true,
            respostas: Arc::clone(&self.respostas),
            fita_viva: self.fita_viva,
            por_ler: true,
            faixas: self.faixas.clone(),
            pinceis: self.pinceis.clone(),
            pintaram: self.pintaram.clone(),
        })
    }

    /// **O que um instantâneo deste tocador retém** ([`PainterTool::bytes_do_instantaneo`]) — o que ele
    /// declara ao razão da RAM.
    #[must_use]
    pub fn bytes_retidos(&self) -> usize {
        self.tool.bytes_do_instantaneo()
    }

    /// **Recomeça deste instantâneo uma trilha EDITADA** ([`Recomeco`]): a fita passa a ser a da trilha
    /// nova, a partir das que este já consumiu, e os pincéis das faixas os dela. `None` = este instantâneo
    /// não é um estado da trilha nova (ela mudou alguma resposta que ele já consumiu, ou o pincel de uma
    /// faixa que já pintou).
    ///
    /// Só a fita e os pincéis se conferem aqui: quem chama garante que os eventos já despachados são os
    /// mesmos.
    #[must_use]
    pub fn recomeca(&self, r: &Recomeco) -> Option<Self> {
        let respostas = &r.respostas;
        if self
            .pintaram
            .iter()
            .any(|n| self.pinceis.get(n) != r.pinceis.get(n))
        {
            return None;
        }
        let mut c = self.instantaneo().ok()?;
        c.pinceis = r.pinceis.clone();
        if c.fita_viva {
            let ja = self.respostas.len() - self.respostas_por_pedir();
            if respostas.get(..ja)? != self.respostas.get(..ja)? {
                return None;
            }
            *fita::trava(&c.fita) = Fita::Toca {
                fila: respostas[ja..].iter().copied().collect(),
                divergiu: None,
            };
        }
        c.respostas = Arc::clone(respostas);
        Some(c)
    }

    /// **A tela para a cena**, se mudou desde a última vez: a composição do editor
    /// ([`PainterTool::take_preview_arc`]), straight, `w × h`, composta UMA vez sobre o que mudou desde
    /// a leitura anterior. O sinal de mudança é o do editor — os passos `Preview` (pista CPU) e
    /// `PreviewSujo` (pista da placa) que a gravação drenou —, logo só compõe quando o artista viu a tela
    /// mudar. Ler não muda a reprodução (gate `macro_em_cena::ler_a_cena_nao_muda_a_reproducao`).
    pub fn toma_a_tela(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        if !std::mem::take(&mut self.mudou) {
            return None;
        }
        self.tool.composto_da_cena()
    }

    /// O retângulo `(x, y, w, h)` que mudou na última [`Self::toma_a_tela`] — `None` = a tela inteira.
    pub fn caixa_da_tela(&mut self) -> Option<(u32, u32, u32, u32)> {
        let caixa = self.tool.take_preview_upload_bbox();
        if std::mem::take(&mut self.por_ler) {
            return None;
        }
        caixa
    }

    #[must_use]
    pub fn tool(&self) -> &PainterTool {
        &self.tool
    }

    pub fn tool_mut(&mut self) -> &mut PainterTool {
        &mut self.tool
    }

    /// Quantas respostas do worker ainda não se pediram (`0` no fim de uma reprodução fiel).
    #[must_use]
    pub fn respostas_por_pedir(&self) -> usize {
        match &*fita::trava(&self.fita) {
            Fita::Toca { fila, .. } => fila.len(),
            Fita::Grava(_) => 0,
        }
    }

    /// **Daqui em diante a água corre no relógio fixo**, sem a fita (a reprodução de uma etapa editada:
    /// [`TocadorDoPainter::aplica`] deixa de conferir as respostas gravadas do worker).
    pub fn passa_ao_relogio_fixo(&mut self) {
        self.fita_viva = false;
        self.tool.wet_passa_ao_relogio_fixo();
        *fita::trava(&self.fita) = Fita::Toca {
            fila: VecDeque::new(),
            divergiu: None,
        };
    }

    /// **A água depois do último passo** (W14), no relógio fixo: `dt_ms`, até assentar ou até `ate`.
    /// `false` = já não anda.
    pub fn corre_a_cauda(&mut self, dt_ms: f32, ate: Option<MarcaDaAgua>) -> bool {
        if self.fita_viva {
            self.passa_ao_relogio_fixo();
        }
        let Some(mudou) = self.tool.wet_cauda(dt_ms, ate) else {
            return false;
        };
        self.mudou |= mudou;
        true
    }

    /// Aplica UMA carga da trilha.
    ///
    /// # Errors
    /// [`ErroDaMacro`] — o tocador não avança mais (quem chama pára).
    pub fn aplica(&mut self, carga: &[u8]) -> Result<(), ErroDaMacro> {
        let passo = Passo::descodifica(carga).map_err(ErroDaMacro::Passo)?;
        match faixa_de(carga) {
            0 => self.aplica_passo(passo)?,
            n => self.aplica_na_faixa(n, passo)?,
        }
        if let Fita::Toca {
            divergiu: Some(d), ..
        } = &*fita::trava(&self.fita)
        {
            return Err(ErroDaMacro::Divergiu(*d));
        }
        Ok(())
    }

    fn aplica_passo(&mut self, passo: Passo) -> Result<(), ErroDaMacro> {
        match passo {
            Passo::SaiDaGravacao => {
                self.passa_ao_relogio_fixo();
                return Ok(());
            }
            Passo::PincelDaFaixa(_) => return Ok(()),
            _ => {}
        }
        let tique = match passo {
            Passo::Tique(dt) => Some(dt),
            _ => None,
        };
        self.tool.poe_tracos_fora(self.abertas(None));
        self.mudou |= Self::aplica_ao_tool(&mut self.tool, passo)?;
        if let Some(dt) = tique {
            self.tique_das_faixas(dt);
        }
        Ok(())
    }

    /// Os traços guardados abertos (os gates do quadro de cada faixa).
    #[cfg(test)]
    pub(crate) fn faixas_abertas(&self) -> Vec<&TracoGuardado> {
        self.faixas.values().filter(|g| g.aberto()).collect()
    }

    /// Quantos traços estão abertos, fora do tool, sem contar a faixa `menos`.
    fn abertas(&self, menos: Option<u32>) -> u32 {
        let n = self
            .faixas
            .iter()
            .filter(|(k, g)| Some(**k) != menos && g.aberto())
            .count();
        u32::try_from(n).unwrap_or(u32::MAX)
    }

    /// **Um passo da faixa `n`** (W16): o traço dela entra no tool, o passo aplica-se, e o do tool volta.
    fn aplica_na_faixa(&mut self, n: u32, passo: Passo) -> Result<(), ErroDaMacro> {
        let mut g = match self.faixas.remove(&n) {
            Some(g) => g,
            None => {
                let pincel = self
                    .pinceis
                    .get(&n)
                    .copied()
                    .unwrap_or_else(|| self.tool.pincel_do_traco());
                self.tool.traco_novo(&pincel)
            }
        };
        self.pintaram.insert(n);
        let fora = self.abertas(None) + u32::from(self.tool.traco_aberto());
        self.tool.troca_o_traco(&mut g);
        self.tool.poe_tracos_fora(fora);
        let r = Self::aplica_ao_tool(&mut self.tool, passo);
        self.tool.troca_o_traco(&mut g);
        if g.aberto() {
            self.faixas.insert(n, g);
        }
        self.tool.poe_tracos_fora(self.abertas(None));
        self.mudou |= r?;
        Ok(())
    }

    /// O quadro de cada traço guardado, depois do `on_tick` do tool (que já correu o quadro inteiro e o
    /// traço dele).
    fn tique_das_faixas(&mut self, dt_ms: f32) {
        let abertas: Vec<u32> = self
            .faixas
            .iter()
            .filter(|(_, g)| g.aberto())
            .map(|(k, _)| *k)
            .collect();
        for n in abertas {
            let Some(mut g) = self.faixas.remove(&n) else {
                continue;
            };
            let fora = self.abertas(None) + u32::from(self.tool.traco_aberto());
            self.tool.troca_o_traco(&mut g);
            self.tool.poe_tracos_fora(fora);
            self.tool.tique_do_traco(dt_ms);
            self.tool.troca_o_traco(&mut g);
            self.faixas.insert(n, g);
        }
        self.tool.poe_tracos_fora(self.abertas(None));
    }

    /// **Um passo pela porta que o gravou**, num tool qualquer (o do tocador, ou o da leitura das
    /// propriedades das etapas). `true` = a tela da cena mudou.
    ///
    /// # Errors
    /// [`ErroDaMacro`].
    pub(crate) fn aplica_ao_tool(t: &mut PainterTool, passo: Passo) -> Result<bool, ErroDaMacro> {
        let (tw, th) = t.source_size();
        let (tw, th) = (f64::from(tw), f64::from(th));
        let escala = |p: [f64; 2]| abs(p, tw, th);
        let mut mudou = false;
        match passo {
            Passo::Ponteiro {
                pos,
                pressao,
                tilt,
                fase,
            } => {
                t.on_canvas_pointer(CanvasPointer {
                    pos: escala(pos),
                    pressure: pressao,
                    tilt,
                    phase: fase,
                });
            }
            Passo::Hover(p) => t.on_canvas_hover(escala(p)),
            Passo::Tique(dt) => t.on_tick(dt),
            Passo::Painel(ev) => t.handle_panel_event(ev),
            Passo::Ativa => t.on_activate(),
            Passo::Desativa => t.on_deactivate(),
            Passo::Desfaz => {
                t.undo_last();
            }
            Passo::Refaz => {
                t.redo_last();
            }
            Passo::PedeCommit => t.request_commit(),
            Passo::TiraCommit => {
                t.take_pending_commit();
            }
            Passo::Assado => t.mark_baked(),
            Passo::TiraAssadoAdiado => {
                t.take_deferred_bake();
            }
            // A cena compõe UMA vez quando lê ([`TocadorDoPainter::toma_a_tela`]).
            Passo::Preview => mudou = t.drena_sem_compor(),
            Passo::PreviewBbox => {
                t.take_preview_upload_bbox();
            }
            Passo::PreviewSujo => mudou = t.take_preview_dirty(),
            Passo::PreviewAtual => {
                let _ = t.current_preview();
            }
            Passo::Completo => {
                t.run_full();
            }
            Passo::Borracha => t.toggle_brush_eraser(),
            Passo::TamanhoPx(px) => t.set_brush_size_px(px),
            Passo::EmpurraTamanho(d) => {
                t.nudge_brush_size(d);
            }
            Passo::Cor(c) => t.set_brush_color_srgb8(c),
            Passo::RestringeLinha(b) => t.set_line_constrain(b),
            Passo::Warp(w) => t.set_canvas_warp(w),
            Passo::GridSnap(p) => t.set_grid_snap(p.map(escala)),
            Passo::GridApaga(b) => t.set_grid_stamp_erase(b),
            Passo::ComporPorQuadro(b) => t.set_compor_por_quadro(b),
            Passo::TolDePega(px) => t.set_shape_grab_tol_px(px),
            Passo::Modificadores([s, c, a]) => t.set_gizmo_modifiers(s, c, a),
            Passo::SeguraRascunho(b) => t.set_shape_draft_hold(b),
            Passo::PresetAplica {
                parametros,
                texturas,
            } => {
                let parametros = serde_json::from_str(&parametros)
                    .map_err(|e| ErroDaMacro::Preset(e.to_string()))?;
                let p = ph2d_brush_presets::Preset {
                    parametros,
                    origem: ph2d_brush_presets::Origem::default(),
                    miniatura: ph2d_brush_presets::Miniatura {
                        largura: 1,
                        altura: 1,
                        rgba: vec![0; 4],
                    },
                    texturas: texturas.into_iter().collect(),
                };
                // O meio do ficheiro (L1): um preset recusado não mudou nada ao gravar, e não muda ao tocar.
                let _ = t.preset_aplicar(&p);
            }
            Passo::PresetVolumes(j) => {
                let v = serde_json::from_str(&j).map_err(|e| ErroDaMacro::Preset(e.to_string()))?;
                t.preset_aplicar_volumes(&v);
            }
            Passo::PresetModo(m) => t.preset_aplicar_modo(&m),
            Passo::TiraPedidoPreset => {
                t.take_wet_preset_request();
            }
            Passo::TiraPedidoGrao => {
                t.take_brush_texture_image_request();
            }
            Passo::TiraPedidoForma => {
                t.take_brush_shape_image_request();
            }
            Passo::TexturaGrao { lum, w, h } => t.set_brush_texture_image(lum, w, h),
            Passo::FormaRgba { rgba, w, h } => t.set_brush_shape_image_rgba(&rgba, w, h, None),
            Passo::TiraPedidoMangueira => {
                t.take_brush_shape_hose_request();
            }
            Passo::MangueiraRgba { rgba, w, h } => t.add_brush_shape_hose_image_rgba(&rgba, w, h),
            Passo::FalloffPonto { id, d, s } => t.set_brush_falloff_point(id, d, s),
            Passo::FalloffAcrescenta { d, s } => {
                t.add_brush_falloff_point_at(d, s);
            }
            Passo::FalloffPega { id, handle } => t.set_brush_falloff_point_handle(id, handle),
            Passo::FalloffTira(id) => t.remove_brush_falloff_point(id),
            Passo::RefrescaFonteDaForma => t.refresh_shape_source_if_changed(),
            Passo::RefrescaCorDaForma => {
                t.refresh_shape_color_preview();
            }
            Passo::TiraPreviewDaFonte => {
                t.take_shape_source_preview();
            }
            Passo::Para => {
                // O fim da gravação assenta a água na tela e nenhum dreno gravado vem depois dele.
                t.wet_traz_para_casa();
                mudou = true;
            }
            Passo::TipoDaTextura(k) => t.set_brush_texture_kind(k),
            Passo::TipoDaForma(k) => t.set_brush_shape_kind(k),
            Passo::SnapDaLinha(b) => t.set_line_snap(b),
            Passo::EscalaUniforme(b) => t.set_uniform_scale(b),
            Passo::Agua(_) => {}
            Passo::SaiDaGravacao => t.wet_passa_ao_relogio_fixo(),
            // O pincel de uma faixa é do tocador ([`TocadorDoPainter::aplica`]); um tool sem faixas não o lê.
            Passo::PincelDaFaixa(_) => {}
            Passo::ColorDropComeca(modo) => t.begin_colordrop_fill(&modo),
            Passo::LimiarDoFill(v) => t.set_fill_threshold(v),
            Passo::FillCommit => t.fill_commit(),
            Passo::ColorDropRestaura => t.restore_after_colordrop(),
            Passo::Recusada(porta) => return Err(ErroDaMacro::PortaRecusada(porta)),
            Passo::Desconhecido(k) => return Err(ErroDaMacro::PassoDesconhecido(k)),
        }
        Ok(mudou)
    }
}
