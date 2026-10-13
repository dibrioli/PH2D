//! ⭐⭐ **A MISTURA POR CÓPIA NA PLACA** (B5, família B do Motion) — o que o passe de formas compõe
//! como o Vello ([`ph2d_shape_gpu::Mistura`]), como a camada da placa POUSA no mundo (P4 (b), doc 132
//! §4) e o que devolve o quadro ao Vello.

use ph2d_eval_motion::{BlendWith, VectorInstance};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::NodeId;
use ph2d_render::Pousa;
use ph2d_shape_gpu::Mistura;
use std::ops::Range;

/// ⭐⭐ **O que a placa compõe como o Vello neste quadro** (B5) — a mistura por cópia do
/// [`ph2d_shape_gpu::Mistura`]. A camada da placa começa VAZIA, logo:
/// - o `Copies` (as cópias só entre si, o grupo em `Normal`) vai sempre e pousa em `Normal`;
/// - na camada ISOLADA do Motion ([`ph2d_eval_motion::MisturaDoSink::pousa_na_camada`]) o
///   `Everything` e a linha com modo próprio misturam-se só com a camada, que pousa em `Normal`;
/// - ⭐ P4 (b) — FORA dela, o que mistura com o mundo POUSA com o modo ([`Composicao::pousa`]): as
///   leis Soma, Tela e Multiplica (pré-multiplicadas, as do Vello) são ASSOCIATIVAS, logo
///   `f(f(W, c₁), c₂) = f(W, f(c₁, c₂))` — o `Everything` é cada cópia em `f` na camada vazia e a
///   camada pousa em `f` (a mistura do HARDWARE lê o mundo: nenhuma cópia dele); o `Scene` é o grupo
///   em `Normal` na camada, pousado em `f`;
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

/// ⭐ **Uma cópia na placa**: a mistura e o grupo NA CAMADA, e como a camada pousa no mundo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Composicao {
    pub mistura: Mistura,
    /// `sink + 1` no `Copies` (dois sinks são dois grupos), `0` sem grupo.
    pub grupo: u32,
    /// Como a camada pousa no mundo (P4 (b)): `Normal` salvo o que mistura com o mundo.
    pub pousa: Pousa,
    /// `Some(sink)` ⇒ o `Scene` fora da camada isolada: o grupo em `Normal` na camada.
    pub cena: Option<u32>,
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

    /// A mistura e o grupo de uma cópia, ou o MOTIVO de o quadro ficar no Vello.
    pub fn da_copia(self, inst: &VectorInstance) -> Result<(Mistura, u32), MotivoDoVello> {
        self.da_mistura(inst.blend_linha, &inst.mistura)
    }

    /// A [`Self::composicao`] sem a pousa.
    pub fn da_mistura(
        self,
        blend_linha: u8,
        mistura: &ph2d_eval_motion::MisturaDoSink,
    ) -> Result<(Mistura, u32), MotivoDoVello> {
        self.composicao(blend_linha, mistura)
            .map(|c| (c.mistura, c.grupo))
    }

    /// ⭐⭐ **A lei** pelas duas coisas que ela lê — o modo da LINHA (`0` = sem) e a mistura do sink.
    /// ⭐ A rota do dispositivo (B6) pergunta-a por SINK: lá a linha nunca tem modo (a coluna `blend`
    /// numa saída de formas ou em grupo recusa o cozimento).
    pub fn composicao(
        self,
        blend_linha: u8,
        mistura: &ph2d_eval_motion::MisturaDoSink,
    ) -> Result<Composicao, MotivoDoVello> {
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
        let fora = !self.isolada;
        let c = |mistura, grupo, pousa, cena| Composicao {
            mistura,
            grupo,
            pousa,
            cena,
        };
        let r = match grupo {
            None => match linha {
                Some(m) if fora => c(m, 0, pousa_de(m), None),
                l => c(l.unwrap_or(Mistura::Normal), 0, Pousa::Normal, None),
            },
            // Dentro da camada isolada a bomba já o fez pousar (`pousa_na_camada`): um `Scene` aqui é a
            // cerca a falhar, e fica no Vello como sempre ficou.
            Some((_, BlendWith::Scene, _)) if !fora => return Err(MotivoDoVello::MundoPartido),
            Some((tag, BlendWith::Scene, sink)) => {
                let m = de(tag).unwrap_or_default();
                c(linha.unwrap_or(Mistura::Normal), 0, pousa_de(m), Some(sink))
            }
            Some((tag, BlendWith::Everything, _)) => {
                let m = linha.or(de(tag)).unwrap_or_default();
                c(m, 0, if fora { pousa_de(m) } else { Pousa::Normal }, None)
            }
            Some((tag, BlendWith::Copies, sink)) => c(
                linha.or(de(tag)).unwrap_or_default(),
                sink + 1,
                Pousa::Normal,
                None,
            ),
        };
        let multiplica = r.mistura == Mistura::Multiplica || r.pousa == Pousa::Multiplica;
        if multiplica && !self.multiplica {
            return Err(MotivoDoVello::SemMultiplica);
        }
        Ok(r)
    }
}

/// A pousa que compõe como a mistura `m` (a lei do [`Pousa::lei`] é a do Vello, como a do passe).
fn pousa_de(m: Mistura) -> Pousa {
    match m {
        Mistura::Normal => Pousa::Normal,
        Mistura::Soma => Pousa::Soma,
        Mistura::Tela => Pousa::Tela,
        Mistura::Multiplica => Pousa::Multiplica,
    }
}

/// ⭐ P4 (b) — **a camada pousa UMA vez**: todas as cópias do quadro têm de pedir a mesma pousa, e
/// uma pousa com o mundo só sai certa de UM arranjo — cópias em `f` (o `Everything`, a linha sem
/// grupo) ou UM só `Scene` (dois `Scene`, ou um `Scene` e um `Everything`, misturam-se entre si em `f`,
/// e a camada juntá-los-ia em `Normal`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PousaDoQuadro {
    pousa: Option<Pousa>,
    cena: Option<u32>,
    por_copia: bool,
}

impl PousaDoQuadro {
    /// Junta uma cópia (ou um sink) — o motivo se o quadro deixa de pousar numa vez.
    pub fn junta(&mut self, c: &Composicao) -> Result<(), MotivoDoVello> {
        if self.pousa.is_some_and(|p| p != c.pousa) {
            return Err(MotivoDoVello::MundoPartido);
        }
        self.pousa = Some(c.pousa);
        if c.pousa == Pousa::Normal {
            return Ok(());
        }
        match c.cena {
            Some(s) if self.por_copia || self.cena.is_some_and(|x| x != s) => {
                Err(MotivoDoVello::MundoPartido)
            }
            Some(s) => {
                self.cena = Some(s);
                Ok(())
            }
            None if self.cena.is_some() => Err(MotivoDoVello::MundoPartido),
            None => {
                self.por_copia = true;
                Ok(())
            }
        }
    }

    /// A pousa do quadro (`Normal` sem cópia nenhuma).
    #[must_use]
    pub fn pousa(&self) -> Pousa {
        self.pousa.unwrap_or_default()
    }
}

/// A faixa de um sink nos dois buffers do cozimento: as cópias de forma e as linhas de imagem.
pub type FaixaDoSink = (NodeId, Range<u32>, Range<u32>);

/// ⭐⭐ **As corridas das cópias do DISPOSITIVO** (B6) — uma por FAIXA de sink no buffer do
/// cozimento (`GpuFormas::faixas`, contíguas e na ordem do buffer), com a mistura do sink pela
/// MESMA [`Misturas::composicao`] da rota da CPU; vizinhas iguais juntam-se. Nenhuma mistura ⇒
/// vazia, o `draw` de sempre.
pub fn corridas_por_faixa(
    faixas: &[(NodeId, Range<u32>)],
    mistura_de: impl Fn(NodeId) -> ph2d_eval_motion::MisturaDoSink,
    misturas: Misturas,
) -> Result<Vec<ph2d_shape_gpu::Corrida>, MotivoDoVello> {
    let com_imagens: Vec<FaixaDoSink> = faixas.iter().map(|(s, r)| (*s, r.clone(), 0..0)).collect();
    corridas_das_faixas(&com_imagens, mistura_de, misturas).map(|(c, _)| c)
}

/// ⭐ P4 — o [`corridas_por_faixa`] com as IMAGENS de cada sink ([`ph2d_shape_gpu::Corrida::imagens`])
/// e a POUSA do quadro. ⛔ Um `Scene` com imagens recusa ([`MotivoDoVello::CenaComImagens`]): as cópias
/// dele juntam-se em `Normal`, e a placa desenha as imagens DEPOIS das formas da corrida.
pub fn corridas_das_faixas(
    faixas: &[FaixaDoSink],
    mistura_de: impl Fn(NodeId) -> ph2d_eval_motion::MisturaDoSink,
    misturas: Misturas,
) -> Result<(Vec<ph2d_shape_gpu::Corrida>, Pousa), MotivoDoVello> {
    let mut corridas: Vec<ph2d_shape_gpu::Corrida> = Vec::with_capacity(faixas.len());
    let mut pousa = PousaDoQuadro::default();
    for (sink, copias, imagens) in faixas {
        // Um sink sem formas nem imagens não desenha nada, e não pede mistura nenhuma.
        if copias.is_empty() && imagens.is_empty() {
            continue;
        }
        let c = misturas.composicao(0, &mistura_de(*sink))?;
        if !imagens.is_empty() && c.cena.is_some() {
            return Err(MotivoDoVello::CenaComImagens);
        }
        pousa.junta(&c)?;
        let junta = |k: &ph2d_shape_gpu::Corrida| {
            k.mistura == c.mistura
                && k.grupo == c.grupo
                && k.copias.end == copias.start
                && (imagens.is_empty() || k.imagens.is_empty() || k.imagens.end == imagens.start)
        };
        match corridas.last_mut() {
            Some(k) if junta(k) => {
                k.copias.end = copias.end;
                if !imagens.is_empty() {
                    let ini = if k.imagens.is_empty() {
                        imagens.start
                    } else {
                        k.imagens.start
                    };
                    k.imagens = ini..imagens.end;
                }
            }
            _ => corridas.push(ph2d_shape_gpu::Corrida {
                copias: copias.clone(),
                mistura: c.mistura,
                grupo: c.grupo,
                imagens: imagens.clone(),
            }),
        }
    }
    if corridas
        .iter()
        .all(|c| c.mistura == Mistura::Normal && c.grupo == 0 && c.imagens.is_empty())
    {
        corridas.clear();
    }
    Ok((corridas, pousa.pousa()))
}

/// **A mistura de um sink neste quadro** — a regra da bomba (`MotionCookPump::pump`) para quem não
/// tem as cópias: o modo e o alcance do `Output`, o índice dele entre as saídas, e o `Scene` que
/// pousa na camada isolada sem agrupar.
#[must_use]
pub fn mistura_no_quadro(
    graph: &ph2d_nodegraph::graph::Graph,
    sinks: &[NodeId],
    sink: NodeId,
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

/// As faixas do último cozimento, sink a sink na ordem de [`crate::motion_state::MotionState::sinks`]:
/// as cópias de forma e as linhas de imagem que as saídas em grupo entregaram à placa.
#[must_use]
pub fn faixas_do_dispositivo(motion: &crate::motion_state::MotionState) -> Vec<FaixaDoSink> {
    let (_, formas) = motion.gpu_cook.faixas_dos_sinks();
    let imagens = motion
        .gpu_cook
        .imagens_do_grupo()
        .map_or(&[][..], |i| i.faixas());
    let de = |lista: &[(NodeId, Range<u32>)], s: NodeId| {
        lista
            .iter()
            .find(|(k, _)| *k == s)
            .map_or(0..0, |(_, r)| r.clone())
    };
    motion
        .sinks
        .iter()
        .map(|&s| (s, de(formas, s), de(imagens, s)))
        .collect()
}

/// ⭐ **As corridas do quadro do DISPOSITIVO** (B6) — [`corridas_das_faixas`] sobre as faixas do
/// último cozimento. A ponte já recusou (`motion_bridge_gpu_mistura`) o sink cuja mistura a placa não
/// compõe, logo um motivo aqui é a cerca a falhar: vazio (`Normal`) e o `debug_assert!` grita.
#[must_use]
pub fn corridas_do_dispositivo(
    motion: &crate::motion_state::MotionState,
    misturas: Misturas,
) -> (Vec<ph2d_shape_gpu::Corrida>, Pousa) {
    let isolada = motion.pump.camada_isolada();
    corridas_das_faixas(
        &faixas_do_dispositivo(motion),
        |s| mistura_no_quadro(&motion.doc.graph, &motion.sinks, s, isolada),
        misturas,
    )
    .unwrap_or_else(|m| {
        debug_assert!(false, "a ponte devia ter recusado esta mistura ({m:?})");
        (Vec::new(), Pousa::Normal)
    })
}

/// ⭐ **O que a shell passa à placa neste quadro** (B6): as misturas do dispositivo e da camada
/// isolada e, com o cozimento VIVO, as corridas por faixa e a pousa — vazias na rota da CPU, que
/// monta as dela.
#[must_use]
pub fn misturas_do_quadro(
    gpu: &GpuContext,
    motion: &crate::motion_state::MotionState,
) -> (Misturas, Vec<ph2d_shape_gpu::Corrida>, Pousa) {
    let misturas = Misturas::do_quadro(gpu, motion.pump.camada_isolada());
    let (corridas, pousa) = if motion.gpu_live {
        corridas_do_dispositivo(motion, misturas)
    } else {
        (Vec::new(), Pousa::Normal)
    };
    (misturas, corridas, pousa)
}

/// Por que uma cópia com mistura deixa o quadro no Vello — dito no terminal na mudança de rota.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotivoDoVello {
    /// [`Misturas::ligadas`] desligado.
    Desligadas,
    /// ⛔ P4 (b) — a camada teria de pousar no mundo mais de uma vez ([`PousaDoQuadro`]).
    MundoPartido,
    /// ⛔ P4 — um `Scene` com imagens no dispositivo: as cópias juntam-se em `Normal`, e a ordem
    /// entre imagens e formas é a das linhas.
    CenaComImagens,
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
        assert_eq!(dentro.da_copia(&tudo), Ok((Mistura::Soma, 0)));
        // ⭐ P4 (b) — fora da camada isolada o que mistura com o mundo POUSA com o modo.
        let pousa = |m: Misturas, i: &VectorInstance| {
            m.composicao(i.blend_linha, &i.mistura)
                .map(|c| (c.mistura, c.grupo, c.pousa, c.cena))
        };
        assert_eq!(
            pousa(fora, &tudo),
            Ok((Mistura::Soma, 0, Pousa::Soma, None)),
            "Everything: cada cópia em f na camada, a camada em f"
        );
        assert_eq!(
            pousa(dentro, &tudo),
            Ok((Mistura::Soma, 0, Pousa::Normal, None))
        );
        assert_eq!(
            pousa(fora, &copia(4, BlendWith::Scene, 0)),
            Ok((Mistura::Normal, 0, Pousa::Tela, Some(4))),
            "Scene: o grupo em Normal na camada, a camada em f"
        );
        assert_eq!(
            dentro.da_copia(&copia(4, BlendWith::Scene, 0)),
            Err(MotivoDoVello::MundoPartido),
            "dentro, a bomba já o fez pousar"
        );
        assert_eq!(
            pousa(m(false, false), &copia(3, BlendWith::Scene, 0)),
            Err(MotivoDoVello::SemMultiplica),
            "a POUSA em Multiplica também pede a 2.ª saída"
        );
        assert_eq!(pousa(fora, &copies).map(|c| c.2), Ok(Pousa::Normal));
        // A linha substitui a do grupo; sem grupo, pede o que está por baixo.
        assert_eq!(
            dentro.da_copia(&copia(1, BlendWith::Copies, 5)),
            Ok((Mistura::Tela, 5))
        );
        assert_eq!(
            pousa(fora, &copia(0, BlendWith::Everything, 5)),
            Ok((Mistura::Tela, 0, Pousa::Tela, None)),
            "a linha sem grupo pousa com o modo dela"
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
        // ⭐ P4 (b) — o mundo por baixo, fora da camada isolada: a camada pousa com o modo.
        let fora = Misturas {
            isolada: false,
            ..placa
        };
        let todo = |_| m(1, BlendWith::Everything, 0);
        let (c, p) =
            corridas_das_faixas(&[(a, 0..2, 0..0), (b, 2..3, 0..0)], todo, fora).expect("compõe");
        assert_eq!((c.len(), c[0].mistura, p), (1, Mistura::Soma, Pousa::Soma));
        // Um sink `Normal` ao lado parte a pousa.
        let mista = |s| {
            if s == a {
                m(1, BlendWith::Everything, 0)
            } else {
                m(0, BlendWith::Copies, 1)
            }
        };
        assert_eq!(
            corridas_das_faixas(&[(a, 0..2, 0..0), (d, 2..3, 0..0)], mista, fora).err(),
            Some(MotivoDoVello::MundoPartido)
        );
        // ⭐ P4 — as IMAGENS de cada sink vão na corrida dele; um sink só de imagens também corre.
        let (c, _) =
            corridas_das_faixas(&[(a, 0..3, 0..4), (b, 3..3, 4..6)], de, placa).expect("compõe");
        let vistas: Vec<_> = c
            .iter()
            .map(|r| (r.copias.clone(), r.imagens.clone(), r.grupo))
            .collect();
        assert_eq!(vistas, vec![(0..3, 0..4, 1), (3..3, 4..6, 2)]);
        // ⛔ Um `Scene` com imagens no dispositivo.
        let cena = |_| m(4, BlendWith::Scene, 0);
        assert_eq!(
            corridas_das_faixas(&[(a, 0..2, 0..1)], cena, fora).err(),
            Some(MotivoDoVello::CenaComImagens)
        );
        assert!(corridas_das_faixas(&[(a, 0..2, 0..0)], cena, fora).is_ok());
    }

    /// ⭐ P4 (b) — **a camada pousa UMA vez**: cópias em `f` juntam-se (associativas); um `Scene` só;
    /// dois `Scene`, um `Scene` com um `Everything`, ou uma pousa `Normal` ao lado partem-na.
    #[test]
    fn a_camada_pousa_no_mundo_uma_vez() {
        let c = |pousa, cena| Composicao {
            mistura: Mistura::Normal,
            grupo: 0,
            pousa,
            cena,
        };
        let junta = |lista: &[Composicao]| {
            let mut p = PousaDoQuadro::default();
            lista
                .iter()
                .try_for_each(|x| p.junta(x))
                .map(|()| p.pousa())
        };
        let soma = c(Pousa::Soma, None);
        assert_eq!(junta(&[]), Ok(Pousa::Normal));
        assert_eq!(junta(&[soma, soma, soma]), Ok(Pousa::Soma));
        assert_eq!(
            junta(&[c(Pousa::Tela, Some(2)), c(Pousa::Tela, Some(2))]),
            Ok(Pousa::Tela)
        );
        let partido = Err(MotivoDoVello::MundoPartido);
        assert_eq!(junta(&[soma, c(Pousa::Normal, None)]), partido);
        assert_eq!(junta(&[soma, c(Pousa::Tela, None)]), partido);
        assert_eq!(
            junta(&[c(Pousa::Soma, Some(1)), c(Pousa::Soma, Some(2))]),
            partido
        );
        assert_eq!(junta(&[c(Pousa::Soma, Some(1)), soma]), partido);
        assert_eq!(junta(&[soma, c(Pousa::Soma, Some(1))]), partido);
    }
}
