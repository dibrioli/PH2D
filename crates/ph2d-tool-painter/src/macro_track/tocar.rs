//! **O tocador da trilha**: um [`PainterTool`] PRÓPRIO (nunca a ferramenta ativa do editor), no estado
//! inicial, que recebe cada passo pela MESMA porta que o gravou. As respostas do worker da água
//! ([`Passo::Agua`]) carregam-se todas à partida numa fita que responde pela ordem em que a thread do
//! quadro as pede — então a ordem delas no tempo da macro não importa.
//!
//! Uma divergência (a fita não responde o que se pergunta, uma marca inalcançável), uma porta recusada
//! ou um passo desconhecido PÁRAM o tocador em voz alta ([`ErroDaMacro`]): nunca se salta um passo.

use super::inicial::{self, ErroDoInicial};
use super::passo::{Passo, abs};
use crate::tool::PainterTool;
use crate::tool::paint::fita::{self, Divergencia, Fita, FitaPartilhada};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, RasterEditTool, Tool};
use std::collections::VecDeque;
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
}

pub struct TocadorDoPainter {
    tool: PainterTool,
    fita: FitaPartilhada,
    /// Um passo de pré-visualização viu a tela mudar desde o último [`Self::toma_a_tela`] (nasce
    /// `true`: a tela inicial também se mostra).
    mudou: bool,
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
        Self::com_tool(tool, cargas)
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
        Self::com_tool(tool, cargas)
    }

    fn com_tool<'a>(
        mut tool: PainterTool,
        cargas: impl IntoIterator<Item = &'a [u8]>,
    ) -> Result<Self, ErroDaMacro> {
        let mut fila = VecDeque::new();
        for c in cargas {
            if let Passo::Agua(r) = Passo::descodifica(c).map_err(ErroDaMacro::Passo)? {
                fila.push_back(r);
            }
        }
        let fita: FitaPartilhada = Arc::new(Mutex::new(Fita::Toca {
            fila,
            divergiu: None,
        }));
        tool.wet_instala_fita(Some(Arc::clone(&fita)));
        Ok(Self {
            tool,
            fita,
            mudou: true,
        })
    }

    /// **A tela para a cena**, se mudou desde a última vez: a composição do editor
    /// ([`PainterTool::take_preview_arc`]), straight, `w × h`. O sinal de mudança é o do editor — os
    /// passos `Preview` (pista CPU) e `PreviewSujo` (pista da placa) que a gravação drenou —, logo só
    /// recompõe quando o artista viu a tela mudar. Ler não muda a reprodução (gate
    /// `macro_em_cena::ler_a_cena_nao_muda_a_reproducao`).
    pub fn toma_a_tela(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        if !std::mem::take(&mut self.mudou) {
            return None;
        }
        self.tool.composto_da_cena()
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
        self.tool.wet_passa_ao_relogio_fixo();
        *fita::trava(&self.fita) = Fita::Toca {
            fila: VecDeque::new(),
            divergiu: None,
        };
    }

    /// Aplica UMA carga da trilha.
    ///
    /// # Errors
    /// [`ErroDaMacro`] — o tocador não avança mais (quem chama pára).
    pub fn aplica(&mut self, carga: &[u8]) -> Result<(), ErroDaMacro> {
        let passo = Passo::descodifica(carga).map_err(ErroDaMacro::Passo)?;
        self.aplica_passo(passo)?;
        if let Fita::Toca {
            divergiu: Some(d), ..
        } = &*fita::trava(&self.fita)
        {
            return Err(ErroDaMacro::Divergiu(*d));
        }
        Ok(())
    }

    fn aplica_passo(&mut self, passo: Passo) -> Result<(), ErroDaMacro> {
        if matches!(passo, Passo::SaiDaGravacao) {
            self.passa_ao_relogio_fixo();
            return Ok(());
        }
        self.mudou |= Self::aplica_ao_tool(&mut self.tool, passo)?;
        Ok(())
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
            Passo::Preview => mudou = t.take_preview_arc().is_some(),
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
                t.wet_preset_aplicar(&p);
            }
            Passo::PresetVolumes(j) => {
                let v = serde_json::from_str(&j).map_err(|e| ErroDaMacro::Preset(e.to_string()))?;
                t.wet_preset_aplicar_volumes(&v);
            }
            Passo::PresetModo(m) => t.wet_preset_aplicar_modo(&m),
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
            Passo::Recusada(porta) => return Err(ErroDaMacro::PortaRecusada(porta)),
            Passo::Desconhecido(k) => return Err(ErroDaMacro::PassoDesconhecido(k)),
        }
        Ok(mudou)
    }
}
