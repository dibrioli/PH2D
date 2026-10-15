//! **O tutorial «fazer»** (W6, `docs/Macros/03_plano_W6.md`): o objecto com a `Macro` em modo
//! `Fazer`, a família de cada trilha diz o que o aluno já fez ([`TutorialDaFamilia`]), e o balão mostra
//! a 1.ª etapa por cumprir — função do estado, por isso o Ctrl+Z volta o passo sozinho. Os botões do
//! balão chegam como `MacroRequest::{MostraMe, Recomeca}`.
//!
//! W6a: as trilhas do Motion. As do Painter não têm critério ainda (W6b) e não contam.

use crate::Familias;
use crate::etapas::da_trilha;
use ph2d_ecs::{Entity, Macro, ModoDaMacro, SimWorld, StableId};
use ph2d_editor_core::macro_edits::{ErroDaMacroView, FamiliaDaMacro};
use ph2d_editor_core::macro_tutorial::{BalaoDoTutorial, EtapaDoTutorial, PedidoDoTutorial};
use ph2d_macro::MacroDoc;
use std::hash::{Hash, Hasher};

/// O estado do tutorial na família: a última `Macro` lida e a recusa de um botão.
#[derive(Default)]
pub(crate) struct Tutorial {
    /// (bits, bytes, o documento).
    lido: Option<(u64, Vec<u8>, MacroDoc)>,
    erro: Option<(u64, ErroDaMacroView)>,
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

/// Uma etapa do tutorial e a trilha dela.
struct Etapa {
    trilha: usize,
    e: EtapaDoTutorial,
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

    /// As etapas com critério de todas as trilhas que a família sabe ensinar, por ordem.
    fn etapas(
        &mut self,
        f: &mut Familias<'_>,
        bits: u64,
        dono: u64,
        bytes: &[u8],
    ) -> Result<Vec<Etapa>, ErroDaMacroView> {
        let doc = self.le(bits, bytes)?.clone();
        let mut out = Vec::new();
        for (tr, track) in doc.tracks.iter().enumerate() {
            let Some(fam) = da_trilha(f.etapas, track).map(|x| x.familia()) else {
                continue;
            };
            let es = match fam {
                FamiliaDaMacro::Motion => f.motion.avalia(dono, track, versao(bytes, tr))?,
                FamiliaDaMacro::Painter => continue,
            };
            out.extend(es.into_iter().map(|e| Etapa { trilha: tr, e }));
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
            erro,
        };
        let etapas = match self.etapas(f, bits, dono, &bytes) {
            Ok(e) => e,
            Err(e) => {
                b.erro = Some(e);
                return Some(b);
            }
        };
        b.total = etapas.len();
        let editando = f.motion_em_edicao == Some(dono) && f.motion_na_mao;
        if !editando && !etapas.is_empty() {
            b.pedido = Some(PedidoDoTutorial::Edita);
            return Some(b);
        }
        match etapas.iter().position(|x| x.e.falta.is_some()) {
            Some(i) => {
                b.passo = i + 1;
                b.pedido = etapas[i].e.falta.clone();
                b.alvos = etapas[i].e.alvos.clone();
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
            let etapas = self.etapas(f, bits, dono, &bytes)?;
            let Some(e) = etapas.iter().find(|x| x.e.falta.is_some()) else {
                return Ok(());
            };
            let doc = self.le(bits, &bytes)?.clone();
            let track = &doc.tracks[e.trilha];
            f.motion
                .mostra(dono, track, versao(&bytes, e.trilha), e.e.k)
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
                if da_trilha(f.etapas, track).map(|x| x.familia()) == Some(FamiliaDaMacro::Motion) {
                    f.motion.recomeca(dono, track)?;
                }
            }
            Ok(())
        })();
        self.erro = r.err().map(|e| (bits, e));
    }
}
