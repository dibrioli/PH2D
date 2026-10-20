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
use ph2d_tool_painter::macro_track::tocar::ErroDaMacro;
use ph2d_tool_painter::macro_track::tutorial::{
    self as pt, EnsinoDoPainter, Medida, Preparo, TipoDoCriterio,
};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::JoinHandle;

/// A chave da avaliação do Painter: a versão da trilha, a imagem ligada, o tamanho da tela, a
/// revisão do conteúdo ([`PainterTool::content_revision`]), as medidas chegadas e a idade da água
/// ([`EnsinoDoPainter::idade_na_chave`]) — a mesma chave, o mesmo estado.
type ChaveDoProgresso = (u64, Option<u64>, (u32, u32), (u64, u64), usize, u32);

/// O que o fio do ensino manda ao quadro, por ordem.
enum Chegada {
    Ensino(Result<EnsinoDoPainter, ErroDaMacro>),
    Medida(Medida),
    /// A medição acabou em erro (a reprodução divergiu).
    Erro(ErroDaMacro),
}

/// **O ensino de uma versão FORA do quadro** (W6c/W6d, `docs/Macros/03_plano_W6.md` §6.3): o fio
/// `macro-referencias` lê a trilha ([`EnsinoDoPainter::de`]: 30–170 ms a 1024²) e depois mede cada
/// pincelada — as curvas da gravada e da base, em poucos fios trabalhadores dele (W7). Até o ensino chegar o balão está «a
/// preparar»; uma pincelada sem medida não se cumpre. Largar o trabalho pára o fio.
struct Trabalho {
    parar: Arc<AtomicBool>,
    chegadas: Receiver<Chegada>,
    fio: Option<JoinHandle<()>>,
}

impl Drop for Trabalho {
    fn drop(&mut self) {
        self.parar.store(true, Ordering::Relaxed);
    }
}

/// O trabalho do fio: o ensino, depois as medidas.
fn ensina(track: &Track, parar: &AtomicBool, tx: &Sender<Chegada>) {
    let e = match EnsinoDoPainter::de(track) {
        Ok(e) => e,
        Err(x) => {
            let _ = tx.send(Chegada::Ensino(Err(x)));
            return;
        }
    };
    let medir = e.a_medir().then(|| e.clone());
    if tx.send(Chegada::Ensino(Ok(e))).is_err() {
        return;
    }
    if let Some(e) = medir
        && let Err(x) = e.mede(track, parar, |m| {
            let _ = tx.send(Chegada::Medida(m));
        })
    {
        let _ = tx.send(Chegada::Erro(x));
    }
}

impl Trabalho {
    fn arranca(track: &Track) -> Self {
        let parar = Arc::new(AtomicBool::new(false));
        let (tx, chegadas) = mpsc::channel();
        let (t, p, tx2) = (track.clone(), Arc::clone(&parar), tx.clone());
        let fio = std::thread::Builder::new()
            .name("macro-referencias".into())
            .spawn(move || ensina(&t, &p, &tx2))
            .ok();
        if fio.is_none() {
            // O fio não arrancou: faz-se aqui.
            ensina(track, &parar, &tx);
        }
        Self {
            parar,
            chegadas,
            fio,
        }
    }
}

/// O ensino de uma versão da trilha: `None` = a preparar.
struct Ensino {
    e: Option<Result<EnsinoDoPainter, ErroDaMacroView>>,
    trabalho: Option<Trabalho>,
}

impl Ensino {
    fn novo(track: &Track) -> Self {
        Self {
            e: None,
            trabalho: Some(Trabalho::arranca(track)),
        }
    }

    /// Recebe o que o fio mandou desde o último quadro.
    fn colhe(&mut self) {
        let Some(t) = &mut self.trabalho else {
            return;
        };
        loop {
            match t.chegadas.try_recv() {
                Ok(Chegada::Ensino(r)) => {
                    self.e = Some(r.map_err(|x| crate::erros::do_painter(&x)));
                }
                Ok(Chegada::Medida(m)) => {
                    if let Some(Ok(e)) = &mut self.e {
                        e.com_medida(m);
                    }
                }
                Ok(Chegada::Erro(x)) => self.e = Some(Err(crate::erros::do_painter(&x))),
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        // O fio acabou: um pânico (ou um ensino que nunca chegou) diz-se no balão em vez de esperar
        // para sempre.
        let panico = t.fio.take().is_some_and(|f| f.join().is_err());
        if panico || self.e.is_none() {
            self.e = Some(Err(crate::erros::erro(crate::erros::ESTRAGADA)));
        }
        self.trabalho = None;
    }
}

/// O estado do tutorial na família: a última `Macro` lida e a recusa de um botão.
#[derive(Default)]
pub(crate) struct Tutorial {
    /// (bits, bytes, o hash deles, o documento).
    lido: Option<(u64, Vec<u8>, u64, MacroDoc)>,
    erro: Option<(u64, ErroDaMacroView)>,
    /// O ensino do Painter, por versão da trilha (as trilhas do Painter da última avaliação).
    ensinos: BTreeMap<u64, Ensino>,
    /// A 1.ª pincelada por cumprir no último estado avaliado: o progresso é função do estado, e um
    /// estado igual não se reavalia (a cada quadro seriam todas as pinceladas cumpridas outra vez).
    progresso: Option<(ChaveDoProgresso, usize)>,
    /// A dica do autor na etapa corrente do Motion, por (hash dos bytes, trilha, k): lida uma vez — por
    /// quadro seria descodificar a trilha inteira (o Painter lê-a no fio).
    dica_do_motion: Option<((u64, usize, usize), Option<String>)>,
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
    /// A dica do autor, se a família a leu fora do quadro (o Painter).
    dica: Option<String>,
}

/// Uma etapa como a família a dá: o preparo, a etapa e a dica lida fora do quadro.
type DaFamilia = (Option<PedidoDoTutorial>, EtapaDoTutorial, Option<String>);

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
    /// Um ensino está a ser preparado ou medido (fora do quadro).
    pub(crate) fn a_medir(&self) -> bool {
        self.ensinos.values().any(|e| e.trabalho.is_some())
    }

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

    /// As pinceladas da trilha no estado do aluno (com a dica do autor): cumpridas até à 1.ª por
    /// cumprir (que só se avalia com o Painter na imagem, da tela da gravação); o meio pedido é o da
    /// pincelada corrente. Só a corrente leva a área marcada. Antes de o ensino chegar do fio, uma etapa
    /// só, «a preparar».
    fn etapas_do_painter(
        &mut self,
        f: &mut Familias<'_>,
        alvo: Option<u64>,
        track: &Track,
        versao: u64,
    ) -> Result<Vec<DaFamilia>, ErroDaMacroView> {
        let ensino = self
            .ensinos
            .entry(versao)
            .or_insert_with(|| Ensino::novo(track));
        ensino.colhe();
        let Some(e) = &ensino.e else {
            let etapa = EtapaDoTutorial {
                k: 0,
                falta: Some(PedidoDoTutorial::APreparar),
                alvos: Vec::new(),
                area: None,
            };
            return Ok(vec![(Some(PedidoDoTutorial::APreparar), etapa, None)]);
        };
        let e = e.as_ref().map_err(Clone::clone)?;
        let na_mao = f.painter_activo;
        let t = painter_em(f, alvo).map(|t| &*t);
        // A cor que falta no pincel para o Fill corrente (W13): pedida no lugar do soltar.
        let mut cor_falta = None;
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
                    e.chegadas(),
                    e.idade_na_chave(t),
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
                cor_falta = e.cor_que_falta(t, feitas);
                (meio, feitas)
            }
        };
        Ok(e.criterios
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let fill = c.regiao.tipo == TipoDoCriterio::Preenchimento;
                let cor = c.regiao.cor;
                let pedido = match (fill, i == feitas && cor_falta.is_some()) {
                    (false, _) => PedidoDoTutorial::Pinta,
                    (true, true) => PedidoDoTutorial::Cor { cor },
                    (true, false) => PedidoDoTutorial::Solta { cor },
                };
                let etapa = EtapaDoTutorial {
                    k: c.regiao.k,
                    falta: (i >= feitas).then_some(pedido),
                    // O Fill começa no botão C&F: o anel aponta-o.
                    alvos: if fill && i == feitas {
                        vec![ph2d_editor_core::ids::PAINTER_RAIL_FILL]
                    } else {
                        Vec::new()
                    },
                    area: alvo.filter(|_| i == feitas).map(|a| AreaNaImagem {
                        entity_bits: a,
                        pontos: c.regiao.pontos.clone(),
                        raio: c.regiao.raio,
                        solta: fill,
                    }),
                };
                (preparo.clone(), etapa, c.dica.clone())
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
        // Os ensinos de outras versões (e as medições deles) largam-se.
        let painter: Vec<u64> = (0..doc.tracks.len()).map(|tr| versao(h, tr)).collect();
        self.ensinos.retain(|v, _| painter.contains(v));
        for (tr, track) in doc.tracks.iter().enumerate() {
            let Some(familia) = da_trilha(f.etapas, track).map(|x| x.familia()) else {
                continue;
            };
            let es = match familia {
                FamiliaDaMacro::Motion => f
                    .motion
                    .avalia(dono, track, versao(h, tr))?
                    .into_iter()
                    .map(|e| ((!editando).then_some(PedidoDoTutorial::Edita), e, None))
                    .collect(),
                FamiliaDaMacro::Painter => {
                    self.etapas_do_painter(f, alvo(sim, bits, track), track, versao(h, tr))?
                }
            };
            out.extend(es.into_iter().map(|(preparo, e, dica)| Etapa {
                trilha: tr,
                familia,
                preparo,
                e,
                dica,
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
        // Um ensino por preparar: o total ainda não se sabe e nenhuma etapa conta.
        if etapas
            .iter()
            .any(|x| x.preparo == Some(PedidoDoTutorial::APreparar))
        {
            b.pedido = Some(PedidoDoTutorial::APreparar);
            return Some(b);
        }
        b.total = etapas.len();
        match etapas.iter().position(|x| x.e.falta.is_some()) {
            Some(i) if etapas[i].preparo.is_some() => b.pedido = etapas[i].preparo.clone(),
            Some(i) => {
                b.passo = i + 1;
                b.pedido = etapas[i].e.falta.clone();
                b.alvos = etapas[i].e.alvos.clone();
                b.area = etapas[i].e.area.clone();
                b.dica = match etapas[i].familia {
                    FamiliaDaMacro::Painter => etapas[i].dica.clone(),
                    FamiliaDaMacro::Motion => {
                        let chave = (
                            self.le(bits, bytes).ok()?.0,
                            etapas[i].trilha,
                            etapas[i].e.k,
                        );
                        if self
                            .dica_do_motion
                            .as_ref()
                            .is_none_or(|(c, _)| *c != chave)
                        {
                            let (_, doc) = self.le(bits, bytes).ok()?;
                            let d = crate::etapas::dica(f.etapas, &doc.tracks[chave.1], chave.2);
                            self.dica_do_motion = Some((chave, d));
                        }
                        self.dica_do_motion.as_ref().and_then(|(_, d)| d.clone())
                    }
                };
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
