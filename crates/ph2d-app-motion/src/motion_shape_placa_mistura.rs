//! ⭐⭐ **A MISTURA POR CÓPIA NA PLACA** (B5, família B do Motion) — o que o passe de formas compõe
//! como o Vello ([`ph2d_shape_gpu::Mistura`]) e o que devolve o quadro ao Vello.

use ph2d_eval_motion::{BlendWith, VectorInstance};
use ph2d_gpu::GpuContext;
use ph2d_shape_gpu::Mistura;

/// ⭐⭐ **O que a placa compõe como o Vello neste quadro** (B5) — a mistura por cópia do
/// [`ph2d_shape_gpu::Mistura`]. A camada da placa começa VAZIA e pousa no mundo em `Normal`, logo:
/// - o `Copies` (as cópias só entre si, o grupo em `Normal`) vai sempre;
/// - o `Everything` e a linha com modo próprio sem grupo misturam com o que está por baixo — só a
///   camada ISOLADA do Motion ([`ph2d_eval_motion::MisturaDoSink::pousa_na_camada`]) os deixa sem o
///   mundo; fora dela ficam no Vello, que pede o mundo por baixo;
/// - o `Scene` fora da camada isolada mistura o GRUPO com o mundo: Vello;
/// - o Multiplica só onde o dispositivo o tem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Misturas {
    /// `false` ⇒ toda mistura devolve o quadro ao Vello (a decisão de antes do B5).
    pub ligadas: bool,
    /// A camada filtrada do Motion está ligada.
    pub isolada: bool,
    /// O dispositivo tem o Multiplica ([`ph2d_shape_gpu::multiplica_no_dispositivo`]).
    pub multiplica: bool,
}

impl Misturas {
    /// As do quadro: o dispositivo de `gpu` e a camada isolada.
    #[must_use]
    pub fn do_quadro(gpu: &GpuContext, isolada: bool) -> Self {
        Self {
            ligadas: true,
            isolada,
            multiplica: ph2d_shape_gpu::multiplica_no_dispositivo(&gpu.device),
        }
    }

    /// A mistura e o grupo de uma cópia, ou o MOTIVO de o quadro ficar no Vello. O grupo é
    /// `sink + 1` no `Copies` (dois sinks são dois grupos) e `0` sem grupo.
    pub fn da_copia(self, inst: &VectorInstance) -> Result<(Mistura, u32), MotivoDoVello> {
        let de = |tag: u8| match tag {
            1 => Some(Mistura::Soma),
            3 => Some(Mistura::Multiplica),
            4 => Some(Mistura::Tela),
            _ => None,
        };
        // A linha SUBSTITUI a mistura do grupo (`camada_da_linha`); sem camada, a do grupo.
        let linha = inst.blend_linha.checked_sub(1).and_then(de);
        let grupo = crate::motion_shape_gen::mistura::chave_de_mistura(&inst.mistura);
        if !self.ligadas && (linha.is_some() || grupo.is_some()) {
            return Err(MotivoDoVello::Desligadas);
        }
        let (m, g) = match grupo {
            None if linha.is_some() && !self.isolada => return Err(MotivoDoVello::PedeOMundo),
            Some((_, BlendWith::Scene, _)) => return Err(MotivoDoVello::PedeOMundo),
            Some((_, BlendWith::Everything, _)) if !self.isolada => {
                return Err(MotivoDoVello::PedeOMundo);
            }
            None => (linha.unwrap_or(Mistura::Normal), 0),
            Some((tag, BlendWith::Everything, _)) => (linha.or(de(tag)).unwrap_or_default(), 0),
            Some((tag, BlendWith::Copies, sink)) => {
                (linha.or(de(tag)).unwrap_or_default(), sink + 1)
            }
        };
        if m == Mistura::Multiplica && !self.multiplica {
            return Err(MotivoDoVello::SemMultiplica);
        }
        Ok((m, g))
    }
}

/// Por que uma cópia com mistura deixa o quadro no Vello — dito no terminal na mudança de rota.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotivoDoVello {
    /// [`Misturas::ligadas`] desligado.
    Desligadas,
    /// A mistura compõe com o mundo por baixo e a camada não está isolada.
    PedeOMundo,
    /// Multiply sem `DUAL_SOURCE_BLENDING` neste dispositivo.
    SemMultiplica,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_eval_motion::MisturaDoSink;

    fn copia(blend: u8, com: BlendWith, linha: u8) -> VectorInstance {
        VectorInstance {
            geometry_id: 1,
            texture_id: 0,
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            premultiplied: 0.0,
            world_pos: [0.0; 2],
            size: [1.0; 2],
            basis: [1.0, 0.0, 0.0, 1.0],
            tint: [1.0; 4],
            anchor: [0.0; 2],
            sampling: 0,
            blend_linha: linha,
            mistura: MisturaDoSink {
                blend,
                com,
                sink: 4,
            },
        }
    }

    /// ⭐ A tabela inteira da rota: cada alcance, com e sem a camada isolada, a linha com modo
    /// próprio, e o Multiplica sem a 2.ª saída — com o MOTIVO de cada recusa.
    #[test]
    fn cada_mistura_vai_a_placa_so_onde_ela_compoe_como_o_vello() {
        let m = |isolada, multiplica| Misturas {
            ligadas: true,
            isolada,
            multiplica,
        };
        let (fora, dentro) = (m(false, true), m(true, true));
        let normal = copia(0, BlendWith::Everything, 0);
        assert_eq!(fora.da_copia(&normal), Ok((Mistura::Normal, 0)));
        assert_eq!(
            Misturas::default().da_copia(&normal),
            Ok((Mistura::Normal, 0))
        );
        let copies = copia(3, BlendWith::Copies, 0);
        assert_eq!(
            fora.da_copia(&copies),
            Ok((Mistura::Multiplica, 5)),
            "o grupo é sink + 1"
        );
        assert_eq!(dentro.da_copia(&copies), Ok((Mistura::Multiplica, 5)));
        assert_eq!(
            m(true, false).da_copia(&copies),
            Err(MotivoDoVello::SemMultiplica)
        );
        let tudo = copia(1, BlendWith::Everything, 0);
        assert_eq!(fora.da_copia(&tudo), Err(MotivoDoVello::PedeOMundo));
        assert_eq!(dentro.da_copia(&tudo), Ok((Mistura::Soma, 0)));
        assert_eq!(
            dentro.da_copia(&copia(4, BlendWith::Scene, 0)),
            Err(MotivoDoVello::PedeOMundo)
        );
        // A linha substitui a do grupo; sem grupo, pede o que está por baixo.
        assert_eq!(
            dentro.da_copia(&copia(1, BlendWith::Copies, 5)),
            Ok((Mistura::Tela, 5))
        );
        assert_eq!(
            fora.da_copia(&copia(0, BlendWith::Everything, 5)),
            Err(MotivoDoVello::PedeOMundo)
        );
        assert_eq!(
            dentro.da_copia(&copia(0, BlendWith::Everything, 5)),
            Ok((Mistura::Tela, 0))
        );
        // `Subtract` na linha não tem camada: fica a do grupo.
        assert_eq!(
            dentro.da_copia(&copia(4, BlendWith::Copies, 3)),
            Ok((Mistura::Tela, 5))
        );
        assert_eq!(
            Misturas::default().da_copia(&copies),
            Err(MotivoDoVello::Desligadas)
        );
    }
}
