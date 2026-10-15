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

/// A chave da avaliação do Painter: a versão da trilha, a imagem ligada, o tamanho da tela e a
/// revisão do conteúdo ([`PainterTool::content_revision`]) — a mesma chave, o mesmo estado.
type ChaveDoProgresso = (u64, Option<u64>, (u32, u32), (u64, u64));

/// O estado do tutorial na família: a última `Macro` lida e a recusa de um botão.
#[derive(Default)]
pub(crate) struct Tutorial {
    /// (bits, bytes, o hash deles, o documento).
    lido: Option<(u64, Vec<u8>, u64, MacroDoc)>,
    erro: Option<(u64, ErroDaMacroView)>,
    /// O ensino do Painter, por versão da trilha.
    ensino: Option<(u64, EnsinoDoPainter)>,
    /// A 1.ª pincelada por cumprir no último estado avaliado: o progresso é função do estado, e um
    /// estado igual não se reavalia (a cada quadro seriam todas as pinceladas cumpridas outra vez).
    progresso: Option<(ChaveDoProgresso, usize)>,
}

/// O objecto do tutorial: a imagem em que o Painter pinta (a `Macro` do Painter vive na imagem que
/// gravou), senão o objecto em edição, se têm uma `Macro` em `Fazer`; senão o 1.º que tenha. Devolve
/// os bits e o `StableId` (o dono das trilhas).
fn objeto(sim: &SimWorld, pintando: Option<u64>, em_edicao: Option<u64>) -> Option<(u64, u64)> {
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
        .find(|(b, _)| Some(*b) == pintando)
        .or_else(|| todos.iter().find(|(_, s)| Some(*s) == em_edicao))
        .or(todos.first())
        .copied()
}

/// A versão da trilha `trilha` de uma `Macro` cujos bytes têm o hash `bytes`.
fn versao(bytes: u64, trilha: usize) -> u64 {
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
    /// A `Macro` de `bits`, lida (e o hash dos bytes feito) uma vez por mudança dos bytes.
    fn le(&mut self, bits: u64, bytes: &[u8]) -> Result<(u64, &MacroDoc), ErroDaMacroView> {
        let fresco = self
            .lido
            .as_ref()
            .is_some_and(|(b, v, _, _)| *b == bits && v == bytes);
        if !fresco {
            let doc = ph2d_macro::codec::read(bytes).map_err(|e| crate::erros::do_ficheiro(&e))?;
            let mut h = std::hash::DefaultHasher::new();
            bytes.hash(&mut h);
            self.lido = Some((bits, bytes.to_vec(), h.finish(), doc));
        }
        let (_, _, h, doc) = self.lido.as_ref().expect("lido acima");
        Ok((*h, doc))
    }

    /// As pinceladas da trilha no estado do aluno: cumpridas até à 1.ª por cumprir (que só se avalia
    /// com o Painter na imagem, da tela da gravação); o meio pedido é o da pincelada corrente. Só a
    /// corrente leva a área marcada.
    fn etapas_do_painter(
        &mut self,
        f: &mut Familias<'_>,
        alvo: Option<u64>,
        track: &Track,
        versao: u64,
    ) -> Result<Vec<(Option<PedidoDoTutorial>, EtapaDoTutorial)>, ErroDaMacroView> {
        if self.ensino.as_ref().is_none_or(|(v, _)| *v != versao) {
            let e = EnsinoDoPainter::de(track).map_err(|e| crate::erros::do_painter(&e))?;
            self.ensino = Some((versao, e));
        }
        let e = &self.ensino.as_ref().expect("calculado acima").1;
        let na_mao = f.painter_activo;
        let t = painter_em(f, alvo).map(|t| &*t);
        let (preparo, feitas) = match e.aberto(t, na_mao, alvo.unwrap_or(u64::MAX)) {
            // Outra tela não se avalia: o balão di-lo, em vez de fingir que nada está pintado.
            Err(Preparo::OutroTamanho { w, h }) => {
                return Err(crate::erros::erro_com(
                    crate::erros::OUTRO_TAMANHO,
                    format!("{w}\u{d7}{h}"),
                ));
            }
            Err(_) => (Some(PedidoDoTutorial::AbreParaPintar), 0),
            Ok(t) => {
                let chave = (
                    versao,
                    t.bound_document(),
                    t.canvas_size(),
                    t.content_revision(),
                );
                let feitas = match self.progresso {
                    Some((c, n)) if c == chave => n,
                    _ => {
                        let n = e.por_cumprir(t).unwrap_or(e.criterios.len());
                        self.progresso = Some((chave, n));
                        n
                    }
                };
                let meio = e.meio_que_falta(t, feitas).map(|m| PedidoDoTutorial::Meio {
                    chave_do_meio: m.name_key(),
                });
                (meio, feitas)
            }
        };
        Ok(e.criterios
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let etapa = EtapaDoTutorial {
                    k: c.regiao.k,
                    falta: (i >= feitas).then_some(PedidoDoTutorial::Pinta),
                    alvos: Vec::new(),
                    area: alvo.filter(|_| i == feitas).map(|a| AreaNaImagem {
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
        self.le(bits, bytes)?;
        // O documento sai do estado enquanto as famílias o avaliam (sem o clonar a cada quadro).
        let lido = self.lido.take().expect("lido acima");
        let r = self.etapas_de(f, sim, bits, dono, lido.2, &lido.3);
        self.lido = Some(lido);
        r
    }

    fn etapas_de(
        &mut self,
        f: &mut Familias<'_>,
        sim: &SimWorld,
        bits: u64,
        dono: u64,
        h: u64,
        doc: &MacroDoc,
    ) -> Result<Vec<Etapa>, ErroDaMacroView> {
        let editando = f.motion_em_edicao == Some(dono) && f.motion_na_mao;
        let mut out = Vec::new();
        for (tr, track) in doc.tracks.iter().enumerate() {
            let Some(familia) = da_trilha(f.etapas, track).map(|x| x.familia()) else {
                continue;
            };
            let es = match familia {
                FamiliaDaMacro::Motion => f
                    .motion
                    .avalia(dono, track, versao(h, tr))?
                    .into_iter()
                    .map(|e| ((!editando).then_some(PedidoDoTutorial::Edita), e))
                    .collect(),
                FamiliaDaMacro::Painter => {
                    self.etapas_do_painter(f, alvo(sim, bits, track), track, versao(h, tr))?
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
        let pintando = f
            .painter
            .as_deref()
            .filter(|_| f.painter_activo)
            .and_then(PainterTool::bound_document);
        let (bits, dono) = objeto(sim, pintando, f.motion_em_edicao)?;
        let bytes = &sim.world().get::<Macro>(Entity::from_bits(bits))?.doc;
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
        let etapas = match self.etapas(f, sim, bits, dono, bytes) {
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
                let (_, doc) = self.le(bits, bytes).ok()?;
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
            let (h, doc) = self.le(bits, &bytes)?;
            let doc = doc.clone();
            let track = &doc.tracks[e.trilha];
            match e.familia {
                FamiliaDaMacro::Motion => f.motion.mostra(dono, track, versao(h, e.trilha), e.e.k),
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
            let doc = self.le(bits, &bytes)?.1.clone();
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
