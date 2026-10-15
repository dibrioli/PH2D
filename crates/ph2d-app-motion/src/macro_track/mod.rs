//! **A trilha do Motion numa macro** (ADR-0186; `docs/Macros/01_plano_onda1.md` §2 e §5.3).
//!
//! - **Gravar** = os dois drenos de intenção gravam o lote JÁ RESOLVIDO (ids de vista decodificados,
//!   nós pela ordem de criação, o destino dos params, o caminho escolhido, a forma que o botão viu), e
//!   o quadro grava o relógio do playhead. Uma **testemunha** compara o documento com o que os lotes
//!   explicam: o que mudou por fora deles (desfazer, gizmos, o conta-gotas da cor) grava-se como
//!   escritas de params ou como o documento inteiro, e o que tocou a bomba como um [`passo::Toque`].
//! - **Tocar** = um palco ([`inicial::palco`]: um `MotionState` próprio, o autorado não se toca) recebe
//!   cada passo pelas MESMAS portas — o lote do grafo, o lote de params, o quadro ([`tocar`]).

pub mod em_cena;
/// As etapas da trilha (W5): nomes, re-cronometrar, editar.
pub mod etapas;
pub mod inicial;
#[cfg(feature = "panel-motion-graph")]
pub mod intencao;
pub mod palcos;
pub mod passo;
mod passo_grafo;
#[cfg(feature = "panel-motion-graph")]
pub(crate) mod roteiro;
/// `PH2D_MACRO_SMOKE=2` — a cena do Motion nó a nó.
#[cfg(feature = "panel-motion-graph")]
pub mod smoke;
pub mod tocar;

#[cfg(test)]
mod alvo_tests;
#[cfg(all(test, feature = "panel-motion-graph"))]
mod etapas_sonda_tests;
#[cfg(all(test, feature = "panel-motion-graph"))]
mod etapas_tests;
#[cfg(all(test, feature = "panel-motion-graph"))]
pub(crate) mod gate_tests;
#[cfg(all(test, feature = "panel-motion-graph"))]
mod painel_tests;
#[cfg(all(test, feature = "panel-motion-graph"))]
mod palcos_custo_tests;
#[cfg(all(test, feature = "panel-motion-graph"))]
mod palcos_tests;
#[cfg(all(test, feature = "panel-motion-graph"))]
mod placa_tests;
#[cfg(all(test, feature = "panel-motion-graph"))]
mod smoke_tests;
#[cfg(test)]
mod tests;

use crate::motion_state::MotionState;
use passo::{Alvo, Escrita, IntencaoDeParam, Passo, Toque};
use ph2d_motion_doc::MotionDoc;

/// Um `MotionState` vazio, a cozer na CPU, com as filas do painel vazias.
#[cfg(test)]
pub(crate) fn estado_de_teste() -> MotionState {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    m.sinks.clear();
    m.gpu_enabled = false;
    #[cfg(feature = "panel-motion-graph")]
    let _ = ph2d_panel_motion_graph::drain_intents();
    let _ = crate::drain_param_intents();
    m
}

/// A chave da família na [`ph2d_macro::Track`].
pub const FAMILIA: &str = "motion";

/// Para onde vai uma edição de param: a selecção do painel decide-o no dreno, e a gravação guarda a
/// decisão (a selecção não viaja).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Destino {
    No,
    /// As linhas de um fundo (título, cor).
    Fundo(u32),
    /// A linha de um cartão de grupo (o nome).
    Cartao(u32),
}

/// Porque armar a gravação não começou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recusa {
    JaGrava,
}

/// A gravação em curso, dentro do `MotionState`.
pub struct Gravacao {
    passos: Vec<Passo>,
    /// O `next_id` do grafo ao armar.
    base: u32,
    /// O documento que os passos gravados explicam.
    esperado: MotionDoc,
    /// A bomba quando o `esperado` foi fechado.
    edicoes: u64,
    sujidades: u64,
    /// As marcas da bomba no último quadro gravado.
    sujidades_no_quadro: u64,
    /// O lote de params em curso (fecha com o dreno).
    params: Option<(Destino, Vec<IntencaoDeParam>)>,
    /// A área de transferência do grafo veio de uma cópia GRAVADA (o palco também a tem).
    clip_gravado: bool,
}

// As RECUSAS gravadas levam a CHAVE da frase (`ph2d_i18n::tr`): ela não depende da língua, e a UI
// traduz-a (gate `cada_recusa_tem_frase_na_tabela`).

/// Colar o que se copiou ANTES de armar: a área de transferência não viaja na macro.
pub const COLAR_DE_ANTES: &str = "app.motion.macro_track.paste_of_a_copy_made_before_recording";
/// O documento mudou por fora das intenções de uma forma que o texto canónico não guarda.
pub const DOCUMENTO_SEM_TEXTO: &str =
    "app.motion.macro_track.the_document_changed_in_a_way_its_text_cannot_keep";

impl MotionState {
    /// **Arma a gravação** e devolve o estado inicial ([`inicial`]).
    ///
    /// # Errors
    /// [`Recusa`].
    pub fn gravacao_arma(&mut self) -> Result<Vec<u8>, Recusa> {
        if self.macro_gravacao.is_some() {
            return Err(Recusa::JaGrava);
        }
        let estado = inicial::captura(self);
        self.macro_gravacao = Some(Box::new(Gravacao {
            passos: Vec::new(),
            base: self.doc.graph.next_id(),
            esperado: self.doc.clone(),
            edicoes: self.pump.edicoes(),
            sujidades: self.pump.sujidades(),
            sujidades_no_quadro: self.pump.sujidades(),
            params: None,
            clip_gravado: false,
        }));
        Ok(estado)
    }

    /// **Abre a trilha do Motion** em `rec`: arma a gravação ([`Self::gravacao_arma`]) e abre a
    /// `Track` com o `StableId` do objecto Motion em edição — o dono dos nós novos
    /// (`objeto.editing`; `0` sem nenhum).
    ///
    /// # Errors
    /// [`Recusa`].
    pub fn gravacao_abre(
        &mut self,
        rec: &mut ph2d_macro::MacroRecorder,
    ) -> Result<ph2d_macro::TrackId, Recusa> {
        let inicial = self.gravacao_arma()?;
        Ok(rec.open_track(FAMILIA, self.objeto.editing.unwrap_or(0), inicial))
    }

    #[must_use]
    pub fn gravacao_ativa(&self) -> bool {
        self.macro_gravacao.is_some()
    }

    /// **Os passos desde a última chamada**, codificados. Quem chama dá-lhes o tempo do quadro
    /// (`MacroRecorder::push`).
    pub fn gravacao_passos(&mut self) -> Vec<passo::Carga> {
        self.macro_gravacao
            .as_mut()
            .map(|g| g.passos.drain(..).map(|p| p.codifica()).collect())
            .unwrap_or_default()
    }

    /// **Pára a gravação** e devolve os últimos passos (o que mudou por fora desde o último quadro).
    pub fn gravacao_para(&mut self) -> Vec<passo::Carga> {
        self.macro_testemunha();
        let resto = self.gravacao_passos();
        self.macro_gravacao = None;
        resto
    }

    /// **A testemunha**: grava o que mudou o documento ou tocou a bomba desde o último fecho, por
    /// fora dos lotes gravados.
    pub(crate) fn macro_testemunha(&mut self) {
        let Some(g) = self.macro_gravacao.as_deref_mut() else {
            return;
        };
        if self.doc != g.esperado {
            let p = mudanca(&g.esperado, &self.doc, g.base);
            g.passos.push(p);
        }
        let t = Toque {
            editou: self.pump.edicoes() != g.edicoes,
            sujou: self.pump.sujidades() != g.sujidades,
        };
        if !t.vazio() {
            g.passos.push(Passo::Toque(t));
        }
        fecha(g, &self.doc, &self.pump);
    }

    /// Um lote acabou de ser aplicado: o documento e a bomba de agora são o que os passos explicam.
    pub(crate) fn macro_fecha(&mut self) {
        let Some(g) = self.macro_gravacao.as_deref_mut() else {
            return;
        };
        if let Some((destino, intencoes)) = g.params.take()
            && !intencoes.is_empty()
        {
            g.passos.push(Passo::Params { destino, intencoes });
        }
        fecha(g, &self.doc, &self.pump);
    }

    /// O dreno de params aplica `intent` em `destino` (já resolvido).
    pub(crate) fn macro_param(&mut self, destino: Destino, intent: &crate::MotionParamIntent) {
        use crate::MotionParamIntent as I;
        let Some(g) = self.macro_gravacao.as_deref_mut() else {
            return;
        };
        let no = |n: u32| Alvo::de_no(n, g.base);
        let i = match intent {
            I::SetParam { node, param, value } => IntencaoDeParam::SetParam {
                node: no(*node),
                param: (*param).to_owned(),
                value: *value,
            },
            I::SetTextParam { node, param, value } => IntencaoDeParam::SetTextParam {
                node: no(*node),
                param: (*param).to_owned(),
                value: value.clone(),
            },
            I::ResetParam { node, param } => IntencaoDeParam::ResetParam {
                node: no(*node),
                param: param.clone(),
            },
            // No nó chega resolvido (o caminho escolhido); num fundo ou num cartão não muda nada.
            I::PickFile { .. } => return,
        };
        match &mut g.params {
            Some((d, v)) if *d == destino => v.push(i),
            _ => g.params = Some((destino, vec![i])),
        }
    }

    /// O quadro vai cozer: o relógio do playhead e o estado da bomba.
    pub(crate) fn macro_quadro(&mut self, playhead: &ph2d_core::Playhead, dt: f64) {
        self.macro_testemunha();
        let agora = self.pump.sujidades();
        if let Some(g) = self.macro_gravacao.as_deref_mut() {
            g.passos.push(Passo::Quadro {
                tempo: playhead.time(),
                a_tocar: playhead.is_playing(),
                dt,
                sujou: agora != g.sujidades_no_quadro,
            });
            g.sujidades_no_quadro = agora;
        }
    }

    /// O quadro coseu: a bomba de agora é a que o palco também terá.
    pub(crate) fn macro_cozeu(&mut self) {
        if let Some(g) = self.macro_gravacao.as_deref_mut() {
            fecha(g, &self.doc, &self.pump);
        }
    }
}

#[cfg(feature = "panel-motion-graph")]
impl MotionState {
    /// O dreno do grafo vai aplicar `intents`: grava o lote resolvido, no nível e com a ajuda de agora.
    pub(crate) fn macro_lote_de_grafo(&mut self, intents: &[ph2d_panel_motion_graph::GraphIntent]) {
        use intencao::Decisao;
        if self.macro_gravacao.is_none() {
            return;
        }
        self.macro_testemunha();
        let Some(g) = self.macro_gravacao.as_deref_mut() else {
            return;
        };
        let mut lote = Vec::new();
        for i in intents {
            match intencao::decide(i, g.base, &self.selected_shape, g.clip_gravado) {
                Decisao::Grava(x) => {
                    g.clip_gravado |= matches!(x, passo::Intencao::CopySelection(_));
                    lote.push(x);
                }
                Decisao::Recusa(porta) => g.passos.push(Passo::Recusada(porta.to_owned())),
                Decisao::Encaminha | Decisao::Vista => {}
            }
        }
        if !lote.is_empty() {
            g.passos.push(Passo::Grafo {
                nivel: self.level,
                ajuda: self.node_help_enabled,
                intencoes: lote,
            });
        }
    }
}

fn fecha(g: &mut Gravacao, doc: &MotionDoc, pump: &ph2d_eval_motion::MotionCookPump) {
    if *doc != g.esperado {
        g.esperado = doc.clone();
    }
    g.edicoes = pump.edicoes();
    g.sujidades = pump.sujidades();
}

/// O que mudou de `antes` para `agora`: as escritas de params, se o resto ficou igual; senão o
/// documento inteiro — e só se o texto o reproduz ao bit.
fn mudanca(antes: &MotionDoc, agora: &MotionDoc, base: u32) -> Passo {
    let escritas = escritas_entre(antes, agora, base);
    if !escritas.is_empty() {
        let mut prova = antes.clone();
        if aplica_escritas(&mut prova, &escritas, base).is_ok() && prova == *agora {
            return Passo::Escritas(escritas);
        }
    }
    let texto = agora.to_text();
    let proximo_id = agora.graph.next_id();
    let arestas = agora.graph.edges().to_vec();
    match documento_de(&texto, proximo_id, &arestas) {
        Ok(d) if d == *agora => Passo::Documento {
            texto,
            proximo_id,
            arestas,
        },
        _ => Passo::Recusada(DOCUMENTO_SEM_TEXTO.to_owned()),
    }
}

/// O documento de um [`Passo::Documento`].
pub(crate) fn documento_de(
    texto: &str,
    proximo_id: u32,
    arestas: &[ph2d_nodegraph::graph::Edge],
) -> Result<MotionDoc, tocar::ErroDaMacro> {
    let mut d =
        MotionDoc::from_text(texto).map_err(|e| tocar::ErroDaMacro::Documento(format!("{e:?}")))?;
    d.graph.restore_next_id(proximo_id);
    if !d.graph.order_edges_as(arestas) {
        return Err(tocar::ErroDaMacro::ArestasDoDocumento);
    }
    Ok(d)
}

fn escritas_entre(antes: &MotionDoc, agora: &MotionDoc, base: u32) -> Vec<Escrita> {
    use std::collections::{BTreeMap, BTreeSet};
    let (g0, g1) = (&antes.graph, &agora.graph);
    let mut v = Vec::new();
    let ids: BTreeSet<_> = g0
        .node_params()
        .keys()
        .chain(g1.node_params().keys())
        .copied()
        .collect();
    let vazio = BTreeMap::new();
    for id in ids {
        let a = g0.node_param_overrides(id).unwrap_or(&vazio);
        let b = g1.node_param_overrides(id).unwrap_or(&vazio);
        let node = Alvo::de_no(id.0, base);
        for (k, x) in b {
            if a.get(k).map(|y| y.to_bits()) != Some(x.to_bits()) {
                v.push(Escrita::Valor {
                    node,
                    param: k.clone(),
                    valor: Some(*x),
                });
            }
        }
        for k in a.keys().filter(|k| !b.contains_key(*k)) {
            v.push(Escrita::Valor {
                node,
                param: k.clone(),
                valor: None,
            });
        }
    }
    let ids: BTreeSet<_> = g0
        .node_text_params()
        .keys()
        .chain(g1.node_text_params().keys())
        .copied()
        .collect();
    let vazio = BTreeMap::new();
    for id in ids {
        let a = g0.node_text_param_overrides(id).unwrap_or(&vazio);
        let b = g1.node_text_param_overrides(id).unwrap_or(&vazio);
        let node = Alvo::de_no(id.0, base);
        for (k, x) in b {
            if a.get(k) != Some(x) {
                v.push(Escrita::Texto {
                    node,
                    param: k.clone(),
                    valor: Some(x.clone()),
                });
            }
        }
        for k in a.keys().filter(|k| !b.contains_key(*k)) {
            v.push(Escrita::Texto {
                node,
                param: k.clone(),
                valor: None,
            });
        }
    }
    v
}

/// Aplica as escritas — a porta da testemunha (para provar que bastam) e a do tocador.
pub(crate) fn aplica_escritas(
    doc: &mut MotionDoc,
    escritas: &[Escrita],
    base: u32,
) -> Result<(), Alvo> {
    use ph2d_nodegraph::graph::NodeId;
    for e in escritas {
        match e {
            Escrita::Valor { node, param, valor } => {
                let id = NodeId(node.no(base).ok_or(*node)?);
                match valor {
                    Some(x) => doc.graph.set_param(id, param.clone(), *x),
                    None => {
                        doc.graph.clear_param(id, param);
                    }
                }
            }
            Escrita::Texto { node, param, valor } => {
                let id = NodeId(node.no(base).ok_or(*node)?);
                match valor {
                    Some(x) => doc.graph.set_text_param(id, param.clone(), x.clone()),
                    None => {
                        doc.graph.clear_text_param(id, param);
                    }
                }
            }
        }
    }
    Ok(())
}
