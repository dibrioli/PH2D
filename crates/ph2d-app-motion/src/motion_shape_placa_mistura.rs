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
        self.da_mistura(inst.blend_linha, &inst.mistura)
    }

    /// A [`Self::da_copia`] pelas duas coisas que ela lê — o modo da LINHA (`0` = sem) e a mistura
    /// do sink. ⭐ A rota do dispositivo (B6) pergunta-a por SINK: lá a linha nunca tem modo (a
    /// coluna `blend` com formas recusa o cozimento).
    pub fn da_mistura(
        self,
        blend_linha: u8,
        mistura: &ph2d_eval_motion::MisturaDoSink,
    ) -> Result<(Mistura, u32), MotivoDoVello> {
        let de = |tag: u8| match tag {
            1 => Some(Mistura::Soma),
            3 => Some(Mistura::Multiplica),
            4 => Some(Mistura::Tela),
            _ => None,
        };
        // A linha SUBSTITUI a mistura do grupo (`camada_da_linha`); sem camada, a do grupo.
        let linha = blend_linha.checked_sub(1).and_then(de);
        let grupo = crate::motion_shape_gen::mistura::chave_de_mistura(mistura);
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

/// ⭐⭐ **As corridas das cópias do DISPOSITIVO** (B6) — uma por FAIXA de sink no buffer do
/// cozimento (`GpuFormas::faixas`, contíguas e na ordem do buffer), com a mistura do sink pela
/// MESMA [`Misturas::da_mistura`] da rota da CPU; vizinhas iguais juntam-se. Nenhuma mistura ⇒
/// vazia, o `draw` de sempre.
pub fn corridas_por_faixa(
    faixas: &[(ph2d_nodegraph::graph::NodeId, std::ops::Range<u32>)],
    mistura_de: impl Fn(ph2d_nodegraph::graph::NodeId) -> ph2d_eval_motion::MisturaDoSink,
    misturas: Misturas,
) -> Result<Vec<ph2d_shape_gpu::Corrida>, MotivoDoVello> {
    let mut corridas: Vec<ph2d_shape_gpu::Corrida> = Vec::with_capacity(faixas.len());
    for (sink, copias) in faixas {
        // Um sink sem formas tem a faixa vazia: não desenha nada, e não pede mistura nenhuma.
        if copias.is_empty() {
            continue;
        }
        let (mistura, grupo) = misturas.da_mistura(0, &mistura_de(*sink))?;
        match corridas.last_mut() {
            Some(c) if c.mistura == mistura && c.grupo == grupo && c.copias.end == copias.start => {
                c.copias.end = copias.end;
            }
            _ => corridas.push(ph2d_shape_gpu::Corrida {
                copias: copias.clone(),
                mistura,
                grupo,
            }),
        }
    }
    if corridas
        .iter()
        .all(|c| c.mistura == Mistura::Normal && c.grupo == 0)
    {
        corridas.clear();
    }
    Ok(corridas)
}

/// **A mistura de um sink neste quadro** — a regra da bomba (`MotionCookPump::pump`) para quem não
/// tem as cópias: o modo e o alcance do `Output`, o índice dele entre as saídas, e o `Scene` que
/// pousa na camada isolada sem agrupar.
#[must_use]
pub fn mistura_no_quadro(
    graph: &ph2d_nodegraph::graph::Graph,
    sinks: &[ph2d_nodegraph::graph::NodeId],
    sink: ph2d_nodegraph::graph::NodeId,
    isolada: bool,
) -> ph2d_eval_motion::MisturaDoSink {
    let mut m = ph2d_eval_motion::MisturaDoSink {
        blend: ph2d_eval_motion::sink_style(graph, sink).blend,
        com: ph2d_eval_motion::sink_blend_with(graph, sink),
        sink: sinks
            .iter()
            .position(|&s| s == sink)
            .and_then(|i| u32::try_from(i).ok())
            .unwrap_or(0),
    };
    if m.pousa_na_camada(isolada) {
        m.blend = 0;
    }
    m
}

/// ⭐ **As corridas do quadro do DISPOSITIVO** (B6) — [`corridas_por_faixa`] sobre as faixas do
/// último cozimento. A ponte já recusou (`motion_bridge_gpu_mistura`) o sink cuja mistura a placa não
/// compõe, logo um motivo aqui é a cerca a falhar: vazio (`Normal`) e o `debug_assert!` grita.
#[must_use]
pub fn corridas_do_dispositivo(
    motion: &crate::motion_state::MotionState,
    misturas: Misturas,
) -> Vec<ph2d_shape_gpu::Corrida> {
    let isolada = motion.pump.camada_isolada();
    let (_, faixas) = motion.gpu_cook.faixas_dos_sinks();
    corridas_por_faixa(
        faixas,
        |s| mistura_no_quadro(&motion.doc.graph, &motion.sinks, s, isolada),
        misturas,
    )
    .unwrap_or_else(|m| {
        debug_assert!(false, "a ponte devia ter recusado esta mistura ({m:?})");
        Vec::new()
    })
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

    /// ⭐ **As corridas do dispositivo, faixa a faixa** (B6): dois sinks `Copies` são DOIS grupos
    /// (`sink + 1`, mutação «grupo 0» sobrevivente da prova até este gate); vizinhas iguais juntam-se;
    /// a faixa vazia não pede mistura; tudo `Normal` é vazio (o `draw` de sempre); o mundo por baixo
    /// recusa pelo motivo.
    #[test]
    fn as_corridas_por_faixa_separam_os_grupos_de_cada_sink() {
        use ph2d_nodegraph::graph::Graph;
        let mut g = Graph::new();
        let [a, b, c, d] = std::array::from_fn(|_| g.add_node("motion.output"));
        let m = |blend, com, sink| MisturaDoSink { blend, com, sink };
        let placa = Misturas {
            ligadas: true,
            isolada: true,
            multiplica: true,
        };
        let de = |s| {
            if s == a {
                m(1, BlendWith::Copies, 0)
            } else if s == b {
                m(1, BlendWith::Copies, 1)
            } else if s == c {
                m(4, BlendWith::Scene, 2)
            } else {
                m(0, BlendWith::Copies, 3)
            }
        };
        let corridas = corridas_por_faixa(&[(a, 0..3), (b, 3..5), (c, 5..5), (d, 5..9)], de, placa)
            .expect("a placa compõe-nas");
        let vistas: Vec<_> = corridas
            .iter()
            .map(|r| (r.copias.clone(), r.mistura, r.grupo))
            .collect();
        assert_eq!(
            vistas,
            vec![
                (0..3, Mistura::Soma, 1),
                (3..5, Mistura::Soma, 2),
                (5..9, Mistura::Normal, 0),
            ],
            "um grupo por sink; a faixa vazia do Scene não recusa nem corre"
        );
        // Vizinhas IGUAIS (o mesmo sink partido em duas faixas) juntam-se.
        let juntas = corridas_por_faixa(&[(a, 0..2), (a, 2..4)], de, placa).expect("compõe");
        assert_eq!(juntas.len(), 1);
        assert_eq!(juntas[0].copias, 0..4);
        // Tudo Normal: vazio.
        assert!(
            corridas_por_faixa(&[(d, 0..4)], de, placa)
                .expect("Normal")
                .is_empty()
        );
        // O mundo por baixo, fora da camada isolada.
        let fora = Misturas {
            isolada: false,
            ..placa
        };
        let todo = |_| m(1, BlendWith::Everything, 0);
        assert_eq!(
            corridas_por_faixa(&[(a, 0..2)], todo, fora).err(),
            Some(MotivoDoVello::PedeOMundo)
        );
    }
}
