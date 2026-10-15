//! **O tocador da trilha do Motion**: um palco ([`super::inicial::palco`]) que recebe cada passo pelas
//! MESMAS portas que o gravaram — o lote do grafo (`apply_graph_batch`), cada edição de param
//! (`apply_param_intent`), o quadro (`quadro::o_quadro`), com o mundo que a cena publica.
//!
//! - [`Modo::Fiel`] coze cada quadro no relógio gravado: reproduz o que o artista viu, e confere a
//!   cada quadro que a bomba chegou como chegou na gravação.
//! - [`Modo::Explicativo`] re-simula da semente depois de cada mudança: o efeito do grafo de agora
//!   como se ele sempre lá estivesse (o tutorial nó a nó).
//!
//! Um passo recusado, desconhecido ou que não se aplica PÁRA o tocador em voz alta ([`ErroDaMacro`]).

use super::inicial::{self, ErroDoInicial};
use super::passo::{IntencaoDeParam, Passo};
use super::{aplica_escritas, documento_de};
use crate::motion_state::MotionState;
use ph2d_macro::CodecError;
use std::collections::BTreeSet;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Modo {
    Fiel,
    Explicativo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroDaMacro {
    Inicial(ErroDoInicial),
    Passo(CodecError),
    /// A gravação passou por uma porta que a macro não reproduz.
    PortaRecusada(String),
    /// Um passo de uma versão mais nova.
    PassoDesconhecido(u64),
    IntencaoDesconhecida(u64),
    /// Um tipo de nó ou um param que este build não tem.
    Nome(String),
    /// Um alvo que não cabe no palco (ou um grupo onde só cabe um nó).
    Alvo,
    /// O texto de um documento gravado não se lê.
    Documento(String),
    /// As arestas gravadas ao lado do texto de um documento não são as dele.
    ArestasDoDocumento,
    /// A bomba chegou ao quadro noutro estado que na gravação: o palco deixou de ser a gravação.
    Divergiu,
    /// Este build não tem o painel do grafo (a feature `panel-motion-graph`).
    SemGrafo,
    /// O `Macro.doc` não se lê (cabeçalho, versão mais nova, dano).
    Ficheiro(CodecError),
    /// Uma edição de etapa que não cabe na gravação.
    Edicao(ph2d_macro::ErroDeEdicao),
}

/// Os nomes `&'static` que este build conhece — tipos de nó, params e as linhas da decoração —, para
/// devolver a uma intenção o nome que a gravação levou como texto. Qualquer `&'static` com o mesmo
/// texto serve: as portas comparam o nome pelo conteúdo.
pub struct Nomes(BTreeSet<&'static str>);

impl Nomes {
    #[must_use]
    pub fn de(registry: &ph2d_node_registry::NodeRegistry) -> Self {
        let mut s = BTreeSet::new();
        for mf in registry.manifests() {
            s.insert(mf.name);
            s.extend(mf.params.iter().map(|p| p.name));
            s.extend(
                registry
                    .param_ui(mf.id)
                    .into_iter()
                    .flatten()
                    .map(|h| h.param),
            );
            s.extend(registry.text_defaults(mf.id).iter().map(|(k, _)| *k));
        }
        // As linhas de um fundo e de um cartão, que não são params de nó nenhum.
        #[cfg(feature = "panel-motion-graph")]
        s.extend(crate::motion_bridge::LINHAS_DA_DECORACAO);
        Nomes(s)
    }

    /// # Errors
    /// [`ErroDaMacro::Nome`].
    pub fn get(&self, nome: &str) -> Result<&'static str, ErroDaMacro> {
        self.0
            .get(nome)
            .copied()
            .ok_or_else(|| ErroDaMacro::Nome(nome.to_owned()))
    }
}

pub struct TocadorDoMotion {
    palco: MotionState,
    modo: Modo,
    base: u32,
    nomes: Nomes,
    toasts: ph2d_editor_core::ToastQueue,
    /// O documento mudou desde o último quadro (o explicativo re-simula).
    mudou: bool,
    /// As marcas da bomba do palco no último quadro (o fiel confere as do quadro gravado).
    sujidades_no_quadro: u64,
}

impl TocadorDoMotion {
    /// No estado inicial gravado.
    ///
    /// # Errors
    /// [`ErroDaMacro::Inicial`].
    pub fn novo(inicial: &[u8], modo: Modo) -> Result<Self, ErroDaMacro> {
        let ini = inicial::le(inicial).map_err(ErroDaMacro::Inicial)?;
        let palco = inicial::palco(&ini);
        let nomes = Nomes::de(&palco.registry);
        let sujidades_no_quadro = palco.pump.sujidades();
        Ok(Self {
            palco,
            modo,
            base: ini.base,
            nomes,
            toasts: ph2d_editor_core::ToastQueue::default(),
            mudou: true,
            sujidades_no_quadro,
        })
    }

    /// O tocador de uma `Track` do Motion: o palco edita o objecto dela (`target`, o que o artista
    /// editava ao gravar: os nós que a macro cria são dele) e coze só os sinks dele (`0` = todos).
    ///
    /// # Errors
    /// [`ErroDaMacro::Inicial`].
    pub fn da_trilha(trilha: &ph2d_macro::Track, modo: Modo) -> Result<Self, ErroDaMacro> {
        let mut t = Self::novo(&trilha.initial, modo)?;
        let dono = (trilha.target != 0).then_some(trilha.target);
        t.palco.objeto.so_o_dono = dono;
        t.palco.objeto.editing = dono;
        Ok(t)
    }

    /// O id que o 1.º nó criado na gravação tem no palco (`Alvo::Criado(k)` = `base + k`).
    #[must_use]
    pub fn base(&self) -> u32 {
        self.base
    }

    #[must_use]
    pub fn palco(&self) -> &MotionState {
        &self.palco
    }

    /// O palco para as portas do presente que guardam estado por quadro (a camada, a lente).
    pub fn palco_mut(&mut self) -> &mut MotionState {
        &mut self.palco
    }

    /// **O mundo que a cena publicou** neste quadro ([`crate::motion_mundo`]) — antes dos passos do
    /// quadro, como o editor o recebe antes do `dispatch`.
    pub fn recebe_o_mundo(&mut self, mundo: &crate::motion_mundo::MundoDaCena) {
        self.palco.recebe_o_mundo(mundo);
    }

    /// **Um passo**, pela porta que o gravou; o quadro coze na `placa` quando há uma (o produto), e
    /// na CPU sem ela.
    ///
    /// # Errors
    /// [`ErroDaMacro`] — o tocador pára neste passo.
    pub fn aplica(
        &mut self,
        carga: &[u8],
        placa: Option<&ph2d_gpu::GpuContext>,
    ) -> Result<(), ErroDaMacro> {
        match Passo::descodifica(carga).map_err(ErroDaMacro::Passo)? {
            Passo::Grafo {
                nivel,
                ajuda,
                intencoes,
            } => self.grafo(nivel, ajuda, &intencoes)?,
            Passo::Params { destino, intencoes } => {
                for i in intencoes {
                    let i = self.param(i)?;
                    crate::motion_bridge::params::apply_param_intent(
                        &mut self.palco,
                        destino,
                        i,
                        &mut self.toasts,
                    );
                }
                self.mudou = true;
            }
            Passo::Escritas(v) => {
                aplica_escritas(&mut self.palco.doc, &v, self.base)
                    .map_err(|_| ErroDaMacro::Alvo)?;
                self.mudou = true;
            }
            Passo::Documento {
                texto,
                proximo_id,
                arestas,
            } => {
                self.palco.doc = documento_de(&texto, proximo_id, &arestas)?;
                self.mudou = true;
            }
            Passo::Toque(t) => {
                if t.editou {
                    self.palco.pump.mark_dirty();
                } else if t.sujou {
                    self.palco.pump.mark_dirty_keeping_ring();
                }
            }
            Passo::Quadro {
                tempo,
                a_tocar,
                dt,
                sujou,
            } => self.quadro(tempo, a_tocar, dt, sujou, placa)?,
            Passo::Recusada(porta) => return Err(ErroDaMacro::PortaRecusada(porta)),
            Passo::Desconhecido(n) => return Err(ErroDaMacro::PassoDesconhecido(n)),
        }
        Ok(())
    }

    #[cfg(feature = "panel-motion-graph")]
    fn grafo(
        &mut self,
        nivel: Option<u32>,
        ajuda: bool,
        intencoes: &[super::passo::Intencao],
    ) -> Result<(), ErroDaMacro> {
        use super::intencao;
        let mut lote = Vec::with_capacity(intencoes.len());
        for i in intencoes {
            if let super::passo::Intencao::PickSelection { escolha, .. } = i {
                self.palco.selected_shape = intencao::forma_de(escolha);
            }
            lote.push(intencao::para_grafo(i, self.base, &self.nomes)?);
        }
        self.palco.level = nivel;
        self.palco.node_help_enabled = ajuda;
        let mut playhead = ph2d_core::Playhead::default();
        let mut split = ph2d_editor_core::screens::layout::CenterSplit::None;
        crate::motion_bridge::apply_graph_batch(
            &mut self.palco,
            &mut playhead,
            &mut self.toasts,
            &mut split,
            lote,
        );
        self.mudou = true;
        Ok(())
    }

    #[cfg(not(feature = "panel-motion-graph"))]
    fn grafo(
        &mut self,
        _nivel: Option<u32>,
        _ajuda: bool,
        _intencoes: &[super::passo::Intencao],
    ) -> Result<(), ErroDaMacro> {
        Err(ErroDaMacro::SemGrafo)
    }

    fn param(&self, i: IntencaoDeParam) -> Result<crate::MotionParamIntent, ErroDaMacro> {
        use crate::MotionParamIntent as M;
        let no = |a: super::passo::Alvo| a.no(self.base).ok_or(ErroDaMacro::Alvo);
        Ok(match i {
            IntencaoDeParam::SetParam { node, param, value } => M::SetParam {
                node: no(node)?,
                param: self.nomes.get(&param)?,
                value,
            },
            IntencaoDeParam::SetTextParam { node, param, value } => M::SetTextParam {
                node: no(node)?,
                param: self.nomes.get(&param)?,
                value,
            },
            IntencaoDeParam::ResetParam { node, param } => M::ResetParam {
                node: no(node)?,
                param,
            },
            IntencaoDeParam::Desconhecida(k) => return Err(ErroDaMacro::IntencaoDesconhecida(k)),
        })
    }

    fn quadro(
        &mut self,
        tempo: f64,
        a_tocar: bool,
        dt: f64,
        sujou: bool,
        placa: Option<&ph2d_gpu::GpuContext>,
    ) -> Result<(), ErroDaMacro> {
        match self.modo {
            Modo::Fiel => {
                if (self.palco.pump.sujidades() != self.sujidades_no_quadro) != sujou {
                    return Err(ErroDaMacro::Divergiu);
                }
            }
            Modo::Explicativo => {
                if std::mem::take(&mut self.mudou) {
                    self.palco.pump.re_simula_da_semente();
                }
            }
        }
        self.sujidades_no_quadro = self.palco.pump.sujidades();
        let mut playhead = ph2d_core::Playhead::new(dt);
        playhead.seek(tempo);
        if a_tocar {
            playhead.play();
        } else {
            playhead.pause();
        }
        crate::motion_bridge::quadro::o_quadro(
            &mut self.palco,
            placa,
            &playhead,
            dt,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
        Ok(())
    }
}
