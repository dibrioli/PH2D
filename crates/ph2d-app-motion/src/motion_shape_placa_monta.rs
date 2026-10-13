//! **As cópias da rota da CPU** — o [`PlacaDeFormas::monta`], cortado do `motion_shape_placa.rs` pelo
//! tecto de LOC quando ganhou as IMAGENS de um grupo (P4, doc 132 §4.1).
//!
//! ⭐ Uma linha de IMAGEM numa corrida de GRUPO (`Copies`, `Everything`, o `Scene`, a linha com modo
//! próprio) vai à placa — o MESMO passe que o dispositivo usa (`imagens`), logo as duas rotas dão a
//! mesma imagem, também nas células do `Each Shape`. Aqui a ordem das linhas é CONHECIDA: uma corrida
//! leva UMA média, e mudar de média abre outra (o grupo continua aberto entre elas) — por isso o
//! `Scene` com imagens, que o dispositivo não sabe ordenar, vai à placa por esta rota. Uma imagem fora
//! de grupo (a folha da terceira média, em `Normal`) continua a devolver o quadro ao Vello.

use ph2d_eval_motion::VectorInstance;
use ph2d_render::RenderInstance;
use ph2d_shape_gpu::{Corrida, Mistura, ShapeInstance};

use super::{GeometriasDaPlaca, Misturas, PlacaDeFormas, PousaDoQuadro, Veredito};
use crate::motion_shape_gen::VecPathStore;

/// A linha de imagem como o passe a lê: a disposição do [`RenderInstance`], com os campos que a cena
/// vectorial honra (o pedaço do `uv_cell` já vem composto no `atlas_uv`).
#[must_use]
pub(crate) fn linha_da_imagem(inst: &VectorInstance) -> RenderInstance {
    let mut r: RenderInstance = bytemuck::Zeroable::zeroed();
    r.world_pos = inst.world_pos;
    r.size = inst.size;
    r.atlas_uv = inst.atlas_uv;
    r.tint = inst.tint;
    r.basis = inst.basis;
    r.premultiplied = inst.premultiplied;
    r.anchor = inst.anchor;
    r.opacity = 1.0;
    r.uv_xform = RenderInstance::IDENTITY_UV_XFORM;
    r.texture_id = inst.texture_id;
    r.sampling = inst.sampling;
    r
}

impl PlacaDeFormas {
    /// As cópias, ou `false` ao primeiro motivo para o quadro ficar no Vello.
    pub(super) fn monta(
        &mut self,
        insts: &[VectorInstance],
        store: &VecPathStore,
        geometrias: &mut GeometriasDaPlaca,
        misturas: Misturas,
    ) -> bool {
        self.copias.clear();
        self.corridas.clear();
        self.motivo = None;
        self.fonte.clear();
        self.imagens_cpu.clear();
        self.pousa = ph2d_render::Pousa::Normal;
        let mut pousa = PousaDoQuadro::default();
        // ⚠️ Um handle que o store largou nunca volta (não é reciclado) — a entrada dele é lixo.
        geometrias.varre(store);
        for (k, inst) in insts.iter().enumerate() {
            let c = misturas
                .composicao(inst.blend_linha, &inst.mistura)
                .and_then(|c| pousa.junta(&c).map(|()| c));
            let c = match c {
                Ok(c) => c,
                Err(motivo) => {
                    self.motivo = Some(motivo);
                    return false;
                }
            };
            let (mistura, grupo) = (c.mistura, c.grupo);
            let i = u32::try_from(self.copias.len()).unwrap_or(u32::MAX);
            if inst.geometry_id == 0 {
                let em_grupo = grupo != 0 || mistura != Mistura::Normal || c.cena.is_some();
                if !em_grupo {
                    return false;
                }
                let j = u32::try_from(self.imagens_cpu.len()).unwrap_or(u32::MAX);
                match self.corridas.last_mut() {
                    Some(r) if r.mistura == mistura && r.grupo == grupo && r.copias.is_empty() => {
                        r.imagens.end = j + 1;
                    }
                    _ => self.corridas.push(Corrida {
                        copias: i..i,
                        mistura,
                        grupo,
                        imagens: j..j + 1,
                    }),
                }
                self.imagens_cpu.push(linha_da_imagem(inst));
                continue;
            }
            match geometrias.veredito(inst.geometry_id, store) {
                // ⚠️ Na rota da CPU um handle ausente devolve o quadro ao Vello, como sempre fez.
                Veredito::Recusada | Veredito::Ausente => return false,
                Veredito::Vazia => {}
                Veredito::Pronta => {
                    match self.corridas.last_mut() {
                        Some(r)
                            if r.mistura == mistura && r.grupo == grupo && r.imagens.is_empty() =>
                        {
                            r.copias.end = i + 1;
                        }
                        _ => self.corridas.push(Corrida {
                            copias: i..i + 1,
                            mistura,
                            grupo,
                            ..Corrida::default()
                        }),
                    }
                    self.fonte.push(u32::try_from(k).unwrap_or(u32::MAX));
                    self.copias.push(ShapeInstance {
                        pos: inst.world_pos,
                        size: inst.size,
                        basis: inst.basis,
                        anchor: inst.anchor,
                        geometry: inst.geometry_id,
                        _pad: 0,
                        tint: inst.tint,
                    });
                }
            }
        }
        self.pousa = pousa.pousa();
        true
    }
}
