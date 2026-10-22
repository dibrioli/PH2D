//! **As etapas como o painel as mostra** e as edições que ele pede, pela porta de cada família
//! ([`EtapasDaFamilia`]): o Painter é desta crate; as outras (o Motion) chegam pela composição.

use crate::Cena;
use crate::erros;
use ph2d_ecs::{Entity, Macro};
use ph2d_editor_core::macro_edits::{
    CampoDaEtapa, ErroDaMacroView, EtapaView, FamiliaDaMacro, PropsDaEtapa, TipoDaEtapaView,
};
use ph2d_editor_core::macro_etapas::{EtapasDaFamilia, espera_s, segundos, us};
use ph2d_macro::{MacroDoc, Track};
use ph2d_tool_painter::macro_track::etapas as pi;

/// As famílias que a composição injecta, além do Painter.
pub type Injectadas<'a> = &'a [&'a dyn EtapasDaFamilia];

/// As etapas do Painter pela porta comum.
pub struct EtapasDoPainter;

impl EtapasDaFamilia for EtapasDoPainter {
    fn familia(&self) -> FamiliaDaMacro {
        FamiliaDaMacro::Painter
    }

    fn chave(&self) -> &'static str {
        ph2d_tool_painter::macro_track::FAMILIA
    }

    fn leitor(&self) -> &'static dyn ph2d_macro::LeitorDaFamilia {
        &pi::LeitorDoPainter
    }

    fn vistas(&self, tr: usize, track: &Track) -> Result<Vec<EtapaView>, ErroDaMacroView> {
        let es = pi::etapas(track).map_err(|e| erros::do_painter(&e))?;
        Ok(es
            .into_iter()
            .map(|e| EtapaView {
                trilha: tr,
                k: e.k,
                familia: FamiliaDaMacro::Painter,
                tipo: match e.tipo {
                    pi::TipoDaEtapa::Pincelada(n) => TipoDaEtapaView::Pincelada(n),
                    pi::TipoDaEtapa::Preenchimento(n) => TipoDaEtapaView::Preenchimento(n),
                    pi::TipoDaEtapa::Recusada(p) => TipoDaEtapaView::Recusada(p),
                },
                nome: e.nome,
                dica: None,
                t0_s: segundos(e.t0),
                t1_s: segundos(e.t1),
                espera_s: espera_s(track, e.etapa.inicio),
                props: PropsDaEtapa::Pincelada {
                    cor: e.cor,
                    tamanho_px: e.tamanho_px,
                },
            })
            .collect())
    }

    fn edita(
        &self,
        track: &mut Track,
        k: usize,
        campo: &CampoDaEtapa,
    ) -> Result<(), ErroDaMacroView> {
        let r = match *campo {
            CampoDaEtapa::Cor(c) => pi::edita_cor(track, k, c),
            CampoDaEtapa::Tamanho(px) => pi::edita_tamanho(track, k, px),
            CampoDaEtapa::Espera(s) => pi::espera(track, k, us(s)),
            CampoDaEtapa::Valor(_) | CampoDaEtapa::Nome(_) | CampoDaEtapa::Dica(_) => {
                return Err(erros::sem_etapa());
            }
        };
        r.map_err(|e| erros::de_edicao_do_painter(&e))
    }
}

/// A família de `track`: o Painter, ou uma das injectadas.
pub(crate) fn da_trilha<'a>(
    outras: Injectadas<'a>,
    track: &Track,
) -> Option<&'a dyn EtapasDaFamilia> {
    std::iter::once(&EtapasDoPainter as &dyn EtapasDaFamilia)
        .chain(outras.iter().copied())
        .find(|f| f.chave() == track.family)
}

pub(crate) fn leitor(
    outras: Injectadas<'_>,
    track: &Track,
) -> Option<&'static dyn ph2d_macro::LeitorDaFamilia> {
    da_trilha(outras, track).map(EtapasDaFamilia::leitor)
}

/// O que o painel mostra de uma `Macro`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vistas {
    pub etapas: Vec<EtapaView>,
    pub duracao_s: f64,
    pub editada_desde: Option<usize>,
    pub erro: Option<ErroDaMacroView>,
}

/// **As etapas de todas as trilhas**, por ordem de trilha. Uma trilha que não se lê dá o erro e as
/// outras continuam à vista.
#[must_use]
pub fn vistas(bytes: &[u8], outras: Injectadas<'_>) -> Vistas {
    let doc: MacroDoc = match ph2d_macro::codec::read(bytes) {
        Ok(d) => d,
        Err(e) => {
            return Vistas {
                erro: Some(erros::do_ficheiro(&e)),
                ..Vistas::default()
            };
        }
    };
    let mut v = Vistas {
        duracao_s: crate::instala::segundos(&doc, outras),
        ..Vistas::default()
    };
    for (tr, track) in doc.tracks.iter().enumerate() {
        let Some(f) = da_trilha(outras, track) else {
            continue;
        };
        let r = f.vistas(tr, track).map(|mut es| {
            for e in &mut es {
                e.dica = dica(outras, track, e.k);
            }
            es
        });
        match r {
            Ok(es) => {
                // A 1.ª etapa À VISTA desde a qual a trilha já não é a gravação (a de uma apagada é a
                // que ficou no lugar dela).
                let primeira = leitor(outras, track)
                    .and_then(|l| {
                        l.etapas(&track.events).iter().position(|g| {
                            track
                                .edicoes
                                .iter()
                                .any(|e| e.ancora == g.inicio && muda_o_toque(e))
                        })
                    })
                    .map(|k| es.iter().position(|e| e.k >= k).unwrap_or(es.len()));
                if let Some(k) = primeira.map(|k| v.etapas.len() + k) {
                    v.editada_desde = Some(v.editada_desde.map_or(k, |a| a.min(k)));
                }
                v.etapas.extend(es);
            }
            Err(e) => v.erro = v.erro.or(Some(e)),
        }
    }
    v
}

/// **O texto do balão** que o autor escreveu na etapa `k` (a contar da gravação).
pub(crate) fn dica(outras: Injectadas<'_>, track: &Track, k: usize) -> Option<String> {
    let ancora = leitor(outras, track)?.etapas(&track.events).get(k)?.inicio;
    track
        .edicoes
        .iter()
        .find(|e| e.ancora == ancora)
        .and_then(|e| e.dica.clone())
}

/// Uma edição que muda o que toca (o nome que o artista escreve não muda).
fn muda_o_toque(e: &ph2d_macro::EdicaoDeEtapa) -> bool {
    e.espera_us != 0
        || !e.trocas.is_empty()
        || !e.insercoes.is_empty()
        || e.sai_da_gravacao
        || e.apagada
}

/// As vistas do último objecto lido — refeitas só quando os bytes da `Macro` mudam (ler as etapas
/// do Painter aplica os passos de configuração de cada pincelada).
#[derive(Default)]
pub struct Cache {
    chave: Option<(u64, Vec<u8>)>,
    vistas: Vistas,
}

impl Cache {
    pub fn de(&mut self, bits: u64, bytes: &[u8], outras: Injectadas<'_>) -> &Vistas {
        let fresca = self
            .chave
            .as_ref()
            .is_some_and(|(b, v)| *b == bits && v == bytes);
        if !fresca {
            self.vistas = vistas(bytes, outras);
            self.chave = Some((bits, bytes.to_vec()));
        }
        &self.vistas
    }
}

/// **Edita a etapa `k` da trilha `trilha`** da `Macro` do objecto `bits`; o clipe acompanha a duração.
/// Uma edição de VALOR do Painter é conferida fora do quadro (`conferencia`).
///
/// # Errors
/// O porquê, para o painel o dizer.
pub fn edita(
    c: &mut Cena<'_>,
    outras: Injectadas<'_>,
    bits: u64,
    trilha: usize,
    k: usize,
    campo: &CampoDaEtapa,
) -> Result<(), ErroDaMacroView> {
    let Some(m) = c.sim.world().get::<Macro>(Entity::from_bits(bits)) else {
        return Ok(());
    };
    let mut doc = ph2d_macro::codec::read(&m.doc).map_err(|x| erros::do_ficheiro(&x))?;
    let track = doc.tracks.get_mut(trilha).ok_or_else(erros::sem_etapa)?;
    if let CampoDaEtapa::Nome(n) | CampoDaEtapa::Dica(n) = campo {
        let ancora = leitor(outras, track)
            .and_then(|l| l.etapas(&track.events).get(k).copied())
            .ok_or_else(erros::sem_etapa)?
            .inicio;
        let texto = n.clone().filter(|n| !n.trim().is_empty());
        let ed = track.edicao_mut(ancora);
        if matches!(campo, CampoDaEtapa::Nome(_)) {
            ed.nome = texto;
        } else {
            ed.dica = texto;
        }
    } else {
        da_trilha(outras, track)
            .ok_or_else(erros::sem_etapa)?
            .edita(track, k, campo)?;
    }
    // Uma edição que a trilha não toca nunca se grava: a macro de antes fica inteira (o arrasto que
    // sobrepunha duas pinceladas gravava-se e apagava TODAS as etapas — smoke do dono 22/10).
    let track = &doc.tracks[trilha];
    if let Some(l) = leitor(outras, track) {
        ph2d_macro::etapa::efetiva(track, l).map_err(|e| erros::de_edicao(&e))?;
    }
    escreve(c, bits, &doc);
    Ok(())
}

/// Grava `doc` na `Macro` de `bits`, só se mudou (o undo é por diff). O clipe segue a duração pela
/// sincronização (`Macros::sincroniza`): uma verdade só.
fn escreve(c: &mut Cena<'_>, bits: u64, doc: &MacroDoc) {
    let bytes = ph2d_macro::codec::write(doc);
    if let Some(mut m) = c.sim.world_mut().get_mut::<Macro>(Entity::from_bits(bits))
        && m.doc != bytes
    {
        m.doc = bytes;
    }
}

/// **Apaga a etapa `k` da trilha `trilha`** da `Macro` de `bits`: o tempo dela fecha-se e o clipe
/// encurta pela sincronização.
///
/// # Errors
/// O porquê, para o painel o dizer.
pub fn apaga(
    c: &mut Cena<'_>,
    outras: Injectadas<'_>,
    bits: u64,
    trilha: usize,
    k: usize,
) -> Result<(), ErroDaMacroView> {
    let Some(m) = c.sim.world().get::<Macro>(Entity::from_bits(bits)) else {
        return Ok(());
    };
    let mut doc = ph2d_macro::codec::read(&m.doc).map_err(|x| erros::do_ficheiro(&x))?;
    let track = doc.tracks.get_mut(trilha).ok_or_else(erros::sem_etapa)?;
    let l = leitor(outras, track).ok_or_else(erros::sem_etapa)?;
    if !track.apaga_etapa(l, k) {
        return Err(erros::sem_etapa());
    }
    escreve(c, bits, &doc);
    Ok(())
}
