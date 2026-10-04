//! ⭐⭐⭐ **O PAINEL DE CAMADAS SOBRE A PEÇA** (`docs/3D/30` §4 — a W3) — filho
//! (`#[path]`) de [`super`]: lá *o traço do Painter na peça*, aqui *o painel de
//! Layers do Painter a mostrar e a mudar a pilha dela*.
//!
//! A cada quadro com a tela presa: os pedidos do painel ([`PieceLayerOp`]) vão
//! à porta da [`PilhaDaPeca`] — cada um com o seu passo de desfazer —, a peça
//! recompõe-se UMA vez, e a pilha volta ao Painter como espelho do que o painel
//! mostra. ⚠️ Recompor é a peça inteira (`33,5 ms` a `64x` na CPU, doc 30 §6):
//! por isso os pedidos de um quadro pagam uma composição só.

use std::collections::BTreeMap;

use ph2d_tool_painter::{PainterTool, PieceLayerOp};

use crate::Sculpt3dScene;
use crate::history::{CamadasDaPeca, StrokeUndo};
use crate::pilha_da_peca::{PilhaDaPeca, RecusaDaPilha, TrocaDaPilha};

/// ⭐ **A frase de uma recusa**, para o painel a mostrar.
pub(crate) fn frase(r: RecusaDaPilha) -> String {
    let chave = match r {
        RecusaDaPilha::Tecto => "app.sculpt3d.camadas.recusa.tecto",
        RecusaDaPilha::Desconhecida => "app.sculpt3d.camadas.recusa.desconhecida",
        RecusaDaPilha::LeOPlanoDaImagem(_) => "app.sculpt3d.camadas.recusa.le_o_plano_da_imagem",
        RecusaDaPilha::ABase => "app.sculpt3d.camadas.recusa.a_base",
        RecusaDaPilha::DegrauAlto => "app.sculpt3d.camadas.recusa.degrau_alto",
        RecusaDaPilha::TracoAberto => "app.sculpt3d.camadas.recusa.traco_aberto",
        RecusaDaPilha::MudaAEstrutura => "app.sculpt3d.camadas.recusa.muda_a_estrutura",
        RecusaDaPilha::ActivaNaoPinta => "app.sculpt3d.camadas.recusa.activa_nao_pinta",
    };
    ph2d_i18n::tr(chave).to_owned()
}

impl Sculpt3dScene {
    /// ⭐ **O traço do Painter pode pousar na peça?** — com pilha, só numa
    /// camada de pintura (o painel diz porquê; o pen-down recusa).
    pub(crate) fn a_activa_recusa_o_traco(&self) -> Option<RecusaDaPilha> {
        let p = self.obj()?.pilha.as_ref()?;
        let activa = p.pilha().active().and_then(|a| p.pilha().get(a));
        (!activa.is_some_and(|c| matches!(c.kind, ph2d_tool_painter::LayerKind::Raster(_))))
            .then_some(RecusaDaPilha::ActivaNaoPinta)
    }

    /// ⭐⭐⭐ **O quadro do painel de camadas** — os pedidos à porta, depois o
    /// espelho. A pilha nasce aqui se a peça tem plano e ainda não a tem: o
    /// painel mostra-a antes do primeiro traço.
    pub(crate) fn camadas_do_painel(&mut self, painter: &mut PainterTool) {
        let pedidos = painter.take_piece_layer_ops();
        if !pedidos.is_empty() {
            let recusa = self.aplica_pedidos_da_pilha(pedidos);
            painter.set_piece_layer_refusal(recusa.map(frase));
        }
        if let Some(o) = self.obj_mut()
            && crate::tinta_da_peca::pilha::acompanha(o.tinta.as_mut(), &mut o.pilha)
        {
            o.tinta_suja = true;
        }
        painter.sync_piece_layers(
            self.obj()
                .and_then(|o| o.pilha.as_ref())
                .map(PilhaDaPeca::pilha),
        );
        if let Some(o) = self.obj() {
            painter.sync_piece_units(crate::vizinhanca_da_peca::unidades(o.stack.mesh()));
        }
    }

    /// ⭐⭐⭐ **Os pedidos do painel, por ordem**, cada um com o seu passo de
    /// desfazer; a peça recompõe-se uma vez no fim. Devolve a última recusa.
    pub(crate) fn aplica_pedidos_da_pilha(
        &mut self,
        pedidos: Vec<PieceLayerOp>,
    ) -> Option<RecusaDaPilha> {
        // Mexer na pilha mexe na peça: a pincelada que ainda escorre fecha antes.
        self.painter_fecha_o_que_escorre();
        let mut recusa = None;
        let mut mudou = false;
        for pedido in pedidos {
            match self.um_pedido(pedido) {
                Ok(()) => mudou = true,
                Err(r) => recusa = Some(r),
            }
        }
        if mudou && let Some(o) = self.obj_mut() {
            crate::tinta_da_peca::pilha::recompoe(o);
        }
        recusa
    }

    /// Um pedido: a porta, e o passo de desfazer (ou o do mesmo arrasto).
    fn um_pedido(&mut self, pedido: PieceLayerOp) -> Result<(), RecusaDaPilha> {
        if self.stroke.tinta_fina.is_some() || self.painter_tela.is_some() {
            return Err(RecusaDaPilha::TracoAberto);
        }
        let level = self.level();
        let o = self.obj_mut().ok_or(RecusaDaPilha::Desconhecida)?;
        let unidades = crate::vizinhanca_da_peca::unidades(o.stack.mesh());
        let (Some(peca), Some(p)) = (o.tinta.as_ref(), o.pilha.as_mut()) else {
            return Err(RecusaDaPilha::Desconhecida);
        };
        let antes = p.pilha().clone();
        let nada = BTreeMap::new;
        let (troca, arrasto) = match pedido {
            PieceLayerOp::NewLayer => {
                let nome = ph2d_i18n::tr_with(
                    "app.sculpt3d.pilha_da_peca.camada_nova",
                    &[("n", &(p.pilha().len() + 1))],
                );
                p.nova_camada(&nome)?;
                (TrocaDaPilha::de(antes, nada()), None)
            }
            PieceLayerOp::NewMask(id) => {
                p.nova_mascara(id)?;
                (TrocaDaPilha::de(antes, nada()), None)
            }
            PieceLayerOp::NewAdjustment(k) => {
                if let Some(r) = crate::vizinhanca_da_peca::recusa_do_degrau(k, peca.nivel()) {
                    return Err(r);
                }
                p.novo_ajuste(k, unidades)?;
                (TrocaDaPilha::de(antes, nada()), None)
            }
            PieceLayerOp::Duplicate(id) => {
                p.duplica(id)?;
                (TrocaDaPilha::de(antes, nada()), None)
            }
            PieceLayerOp::Delete(id) => {
                let tirados = p.apaga(id)?;
                (TrocaDaPilha::de(antes, tirados), None)
            }
            PieceLayerOp::Metadata { stack, gesture } => {
                let velha = p.troca_metadado(stack)?;
                (TrocaDaPilha::de(velha, nada()), gesture)
            }
        };
        let passo = CamadasDaPeca::da_peca(peca, troca);
        let continua = arrasto.is_some()
            && self.camadas_arrasto.map(|(g, e)| (Some(g), e)) == Some((arrasto, self.edits))
            && matches!(
                self.undo.last().map(|e| &e.undo),
                Some(StrokeUndo::Camadas { .. })
            );
        if !continua {
            self.record(StrokeUndo::Camadas {
                level,
                passo: Some(Box::new(passo)),
            });
        }
        self.edits += 1;
        self.camadas_arrasto = arrasto.map(|g| (g, self.edits));
        Ok(())
    }
}
