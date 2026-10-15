//! **O tutorial «fazer»** (W6, `docs/Macros/03_plano_W6.md`): o objecto com a `Macro` em modo
//! `Fazer`, a família de cada trilha diz o que o aluno já fez ([`TutorialDaFamilia`]), e o balão mostra
//! a 1.ª etapa por cumprir — função do estado, por isso o Ctrl+Z volta o passo sozinho. Os botões do
//! balão chegam como `MacroRequest::{MostraMe, Recomeca}`.
//!
//! Cada etapa traz o PREPARO da família dela: o Motion pede o objecto em Edit; o Painter, a imagem
//! da trilha em modo Paint e o meio da gravação (W6b).

use crate::Familias;
use crate::etapas::da_trilha;
use ph2d_ecs::{Entity, Macro, ModoDaMacro, SimWorld, StableId};
use ph2d_editor_core::macro_edits::{ErroDaMacroView, FamiliaDaMacro};
use ph2d_editor_core::macro_tutorial::{
    AreaNaImagem, BalaoDoTutorial, EtapaDoTutorial, PedidoDoTutorial,
};
use ph2d_macro::{MacroDoc, Track};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::tutorial::{self as pt, EnsinoDoPainter, Preparo};
use std::hash::{Hash, Hasher};

/// O estado do tutorial na família: a última `Macro` lida e a recusa de um botão.
#[derive(Default)]
pub(crate) struct Tutorial {
    /// (bits, bytes, o documento).
    lido: Option<(u64, Vec<u8>, MacroDoc)>,
    erro: Option<(u64, ErroDaMacroView)>,
    /// O ensino do Painter, por (versão da trilha, tamanho da tela).
    ensino: Option<((u64, Option<(u32, u32)>), EnsinoDoPainter)>,
}

/// O objecto do tutorial: o que está em edição, se tem uma `Macro` em `Fazer`; senão o 1.º que tenha.
/// Devolve os bits e o `StableId` (o dono das trilhas).
fn objeto(sim: &SimWorld, em_edicao: Option<u64>) -> Option<(u64, u64)> {
    let w = sim.world();
    let mut todos: Vec<(u64, u64)> = w
        .iter_entities()
        .filter(|e| {
            e.get::<Macro>()
                .is_some_and(|m| m.modo == ModoDaMacro::Fazer && !m.doc.is_empty())
        })
        .filter_map(|e| Some((e.id().to_bits(), e.get::<StableId>()?.0)))
        .collect();
    todos.sort_unstable();
    todos
        .iter()
        .find(|(_, s)| Some(*s) == em_edicao)
        .or(todos.first())
        .copied()
}

fn versao(bytes: &[u8], trilha: usize) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    bytes.hash(&mut h);
    trilha.hash(&mut h);
    h.finish()
}

/// Uma etapa do tutorial, a trilha dela e o que a família pede antes dela.
struct Etapa {
    trilha: usize,
    familia: FamiliaDaMacro,
    preparo: Option<PedidoDoTutorial>,
    e: EtapaDoTutorial,
}

/// A imagem que a trilha do Painter pinta: `target` 0 = a entidade da própria `Macro`.
fn alvo(sim: &SimWorld, bits: u64, track: &Track) -> Option<u64> {
    if track.target == 0 {
        return Some(bits);
    }
    ph2d_ecs::entity_with_stable_id(sim.world(), StableId(track.target)).map(Entity::to_bits)
}

/// O Painter da composição, se está ligado à imagem `alvo`.
fn painter_em<'a>(f: &'a mut Familias<'_>, alvo: Option<u64>) -> Option<&'a mut PainterTool> {
    f.painter
        .as_deref_mut()
        .filter(|t| alvo.is_some() && t.bound_document() == alvo)
}

impl Tutorial {
    /// A `Macro` de `bits`, lida uma vez por mudança dos bytes.
    fn le(&mut self, bits: u64, bytes: &[u8]) -> Result<&MacroDoc, ErroDaMacroView> {
        let fresco = self
            .lido
            .as_ref()
            .is_some_and(|(b, v, _)| *b == bits && v == bytes);
        if !fresco {
            let doc = ph2d_macro::codec::read(bytes).map_err(|e| crate::erros::do_ficheiro(&e))?;
            self.lido = Some((bits, bytes.to_vec(), doc));
        }
        Ok(&self.lido.as_ref().expect("lido acima").2)
    }

    /// O ensino da trilha do Painter para a tela do aluno (`None` = não a tem aberta).
    fn ensino(
        &mut self,
        track: &Track,
        versao: u64,
        tela: Option<(u32, u32)>,
    ) -> Result<&EnsinoDoPainter, ErroDaMacroView> {
        let chave = (versao, tela);
        if self.ensino.as_ref().is_none_or(|(c, _)| *c != chave) {
            let e = EnsinoDoPainter::de(track, tela).map_err(|e| crate::erros::do_painter(&e))?;
            self.ensino = Some((chave, e));
        }
        Ok(&self.ensino.as_ref().expect("calculado acima").1)
    }

    /// As pinceladas da trilha no estado do aluno: cumpridas até à 1.ª por cumprir (que só se avalia
    /// com o Painter na imagem, no meio gravado).
    fn etapas_do_painter(
        &mut self,
        f: &mut Familias<'_>,
        alvo: Option<u64>,
        track: &Track,
        versao: u64,
    ) -> Result<Vec<(Option<PedidoDoTutorial>, EtapaDoTutorial)>, ErroDaMacroView> {
        let na_mao = f.painter_activo;
        let t = painter_em(f, alvo).map(|t| &*t);
        let e = self.ensino(track, versao, t.map(PainterTool::canvas_size))?;
        let preparo = e
            .preparo(t, na_mao, alvo.unwrap_or(u64::MAX))
            .map(|p| match p {
                Preparo::AbreParaPintar => PedidoDoTutorial::AbreParaPintar,
                Preparo::Meio(m) => PedidoDoTutorial::Meio {
                    chave_do_meio: pt::chave_do_meio(m),
                },
            });
        let feitas = match (preparo.is_none(), t) {
            (true, Some(t)) => e.por_cumprir(t).unwrap_or(e.criterios.len()),
            _ => 0,
        };
        Ok(e.criterios
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let falta = i >= feitas;
                let etapa = EtapaDoTutorial {
                    k: c.regiao.k,
                    falta: falta.then_some(PedidoDoTutorial::Pinta),
                    alvos: Vec::new(),
                    area: alvo.filter(|_| falta).map(|a| AreaNaImagem {
                        entity_bits: a,
                        pontos: c.regiao.pontos.clone(),
                        raio: c.regiao.raio,
                    }),
                };
                (preparo.clone(), etapa)
            })
            .collect())
    }

    /// As etapas com critério de todas as trilhas que a família sabe ensinar, por ordem.
    fn etapas(
        &mut self,
        f: &mut Familias<'_>,
        sim: &SimWorld,
        bits: u64,
        dono: u64,
        bytes: &[u8],
    ) -> Result<Vec<Etapa>, ErroDaMacroView> {
        let doc = self.le(bits, bytes)?.clone();
        let editando = f.motion_em_edicao == Some(dono) && f.motion_na_mao;
        let mut out = Vec::new();
        for (tr, track) in doc.tracks.iter().enumerate() {
            let Some(familia) = da_trilha(f.etapas, track).map(|x| x.familia()) else {
                continue;
            };
            let es = match familia {
                FamiliaDaMacro::Motion => f
                    .motion
                    .avalia(dono, track, versao(bytes, tr))?
                    .into_iter()
                    .map(|e| ((!editando).then_some(PedidoDoTutorial::Edita), e))
                    .collect(),
                FamiliaDaMacro::Painter => {
                    self.etapas_do_painter(f, alvo(sim, bits, track), track, versao(bytes, tr))?
                }
            };
            out.extend(es.into_iter().map(|(preparo, e)| Etapa {
                trilha: tr,
                familia,
                preparo,
                e,
            }));
        }
        Ok(out)
    }

    /// **O balão do quadro** (`None` = nenhum objecto com um tutorial).
    pub(crate) fn balao(
        &mut self,
        f: &mut Familias<'_>,
        sim: &SimWorld,
    ) -> Option<BalaoDoTutorial> {
        let (bits, dono) = objeto(sim, f.motion_em_edicao)?;
        let bytes = sim
            .world()
            .get::<Macro>(Entity::from_bits(bits))?
            .doc
            .clone();
        let erro = self
            .erro
            .as_ref()
            .filter(|(b, _)| *b == bits)
            .map(|(_, e)| e.clone());
        let mut b = BalaoDoTutorial {
            entity_bits: bits,
            passo: 0,
            total: 0,
            pedido: None,
            dica: None,
            alvos: Vec::new(),
            area: None,
            erro,
        };
        let etapas = match self.etapas(f, sim, bits, dono, &bytes) {
            Ok(e) => e,
            Err(e) => {
                b.erro = Some(e);
                return Some(b);
            }
        };
        b.total = etapas.len();
        match etapas.iter().position(|x| x.e.falta.is_some()) {
            Some(i) if etapas[i].preparo.is_some() => b.pedido = etapas[i].preparo.clone(),
            Some(i) => {
                b.passo = i + 1;
                b.pedido = etapas[i].e.falta.clone();
                b.alvos = etapas[i].e.alvos.clone();
                b.area = etapas[i].e.area.clone();
                let doc = self.le(bits, &bytes).ok()?;
                b.dica =
                    crate::etapas::dica(f.etapas, &doc.tracks[etapas[i].trilha], etapas[i].e.k);
            }
            None => b.passo = b.total,
        }
        Some(b)
    }

    /// «Mostra-me»: a etapa corrente feita pelo aluno, pela família dela.
    pub(crate) fn mostra(&mut self, f: &mut Familias<'_>, sim: &SimWorld, bits: u64) {
        let r = (|| {
            let dono = sim
                .world()
                .get::<StableId>(Entity::from_bits(bits))
                .ok_or_else(crate::erros::sem_etapa)?
                .0;
            let bytes = sim
                .world()
                .get::<Macro>(Entity::from_bits(bits))
                .ok_or_else(crate::erros::sem_etapa)?
                .doc
                .clone();
            let etapas = self.etapas(f, sim, bits, dono, &bytes)?;
            let Some(e) = etapas.iter().find(|x| x.e.falta.is_some()) else {
                return Ok(());
            };
            if e.preparo.is_some() {
                return Err(crate::erros::sem_etapa());
            }
            let doc = self.le(bits, &bytes)?.clone();
            let track = &doc.tracks[e.trilha];
            match e.familia {
                FamiliaDaMacro::Motion => {
                    f.motion
                        .mostra(dono, track, versao(&bytes, e.trilha), e.e.k)
                }
                FamiliaDaMacro::Painter => {
                    let t = painter_em(f, alvo(sim, bits, track))
                        .ok_or_else(crate::erros::sem_etapa)?;
                    pt::mostra(t, track, e.e.k).map_err(|e| crate::erros::do_painter(&e))
                }
            }
        })();
        self.erro = r.err().map(|e| (bits, e));
    }

    /// «Recomeçar»: cada trilha do objecto volta o grafo do aluno ao estado inicial dela.
    pub(crate) fn recomeca(&mut self, f: &mut Familias<'_>, sim: &SimWorld, bits: u64) {
        let r = (|| {
            let dono = sim
                .world()
                .get::<StableId>(Entity::from_bits(bits))
                .ok_or_else(crate::erros::sem_etapa)?
                .0;
            let bytes = sim
                .world()
                .get::<Macro>(Entity::from_bits(bits))
                .ok_or_else(crate::erros::sem_etapa)?
                .doc
                .clone();
            let doc = self.le(bits, &bytes)?.clone();
            for track in &doc.tracks {
                match da_trilha(f.etapas, track).map(|x| x.familia()) {
                    Some(FamiliaDaMacro::Motion) => f.motion.recomeca(dono, track)?,
                    Some(FamiliaDaMacro::Painter) => {
                        let t = painter_em(f, alvo(sim, bits, track))
                            .ok_or_else(crate::erros::sem_etapa)?;
                        pt::recomeca(t, track).map_err(|e| crate::erros::do_painter(&e))?;
                    }
                    None => {}
                }
            }
            Ok(())
        })();
        self.erro = r.err().map(|e| (bits, e));
    }
}
