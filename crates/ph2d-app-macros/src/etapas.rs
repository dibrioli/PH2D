//! **As etapas como o painel as mostra** e as edições que ele pede, pelas portas de cada família
//! (`ph2d_tool_painter::macro_track::etapas`, `ph2d_app_motion::macro_track::etapas`).

use crate::Cena;
use crate::erros;
use ph2d_app_motion::macro_track::etapas as mo;
use ph2d_ecs::{Entity, Macro};
use ph2d_editor_core::macro_edits::{
    CampoDaEtapa, ErroDaMacroView, EtapaView, FamiliaDaMacro, PropsDaEtapa, TipoDaEtapaView,
};
use ph2d_macro::{MacroDoc, MacroTime, Track};
use ph2d_tool_painter::macro_track::etapas as pi;

fn s(t: MacroTime) -> f64 {
    #[expect(clippy::cast_precision_loss, reason = "µs de uma macro cabem num f64")]
    let v = t.0 as f64 / 1e6;
    v
}

fn espera_s(track: &Track, ancora: usize) -> f64 {
    #[expect(clippy::cast_precision_loss, reason = "µs de uma espera cabem num f64")]
    track
        .edicoes
        .iter()
        .find(|e| e.ancora == ancora)
        .map_or(0.0, |e| e.espera_us as f64 / 1e6)
}

fn familia(track: &Track) -> Option<FamiliaDaMacro> {
    match track.family.as_str() {
        ph2d_tool_painter::macro_track::FAMILIA => Some(FamiliaDaMacro::Painter),
        ph2d_app_motion::macro_track::FAMILIA => Some(FamiliaDaMacro::Motion),
        _ => None,
    }
}

fn do_painter(tr: usize, track: &Track) -> Result<Vec<EtapaView>, ErroDaMacroView> {
    let es = pi::etapas(track).map_err(|e| erros::do_painter(&e))?;
    Ok(es
        .into_iter()
        .map(|e| EtapaView {
            trilha: tr,
            k: e.k,
            familia: FamiliaDaMacro::Painter,
            tipo: match e.tipo {
                pi::TipoDaEtapa::Pincelada(n) => TipoDaEtapaView::Pincelada(n),
                pi::TipoDaEtapa::Recusada(p) => TipoDaEtapaView::Recusada(p),
            },
            nome: e.nome,
            t0_s: s(e.t0),
            t1_s: s(e.t1),
            espera_s: espera_s(track, e.etapa.inicio),
            props: PropsDaEtapa::Pincelada {
                cor: e.cor,
                tamanho_px: e.tamanho_px,
            },
        })
        .collect())
}

fn do_motion(tr: usize, track: &Track) -> Result<Vec<EtapaView>, ErroDaMacroView> {
    let es = mo::etapas(track).map_err(|e| erros::do_motion(&e))?;
    Ok(es
        .into_iter()
        .map(|e| {
            let props = match &e.tipo {
                mo::TipoDaEtapa::Param { para, .. } => PropsDaEtapa::Param { valor: *para },
                _ => PropsDaEtapa::Nenhuma,
            };
            EtapaView {
                trilha: tr,
                k: e.k,
                familia: FamiliaDaMacro::Motion,
                tipo: match e.tipo {
                    mo::TipoDaEtapa::Acrescenta(v) => TipoDaEtapaView::Acrescenta(v),
                    mo::TipoDaEtapa::Liga { de, para } => TipoDaEtapaView::Liga { de, para },
                    mo::TipoDaEtapa::Desliga { para } => TipoDaEtapaView::Desliga { para },
                    mo::TipoDaEtapa::Apaga(n) => TipoDaEtapaView::Apaga(n),
                    mo::TipoDaEtapa::Move => TipoDaEtapaView::Move,
                    mo::TipoDaEtapa::Renomeia(n) => TipoDaEtapaView::Renomeia(n),
                    mo::TipoDaEtapa::Param {
                        no,
                        param,
                        de,
                        para,
                    } => TipoDaEtapaView::Param {
                        no,
                        param,
                        de,
                        para,
                    },
                    mo::TipoDaEtapa::Grafo => TipoDaEtapaView::Grafo,
                    mo::TipoDaEtapa::Direta => TipoDaEtapaView::Direta,
                    mo::TipoDaEtapa::Recusada(p) => TipoDaEtapaView::Recusada(p),
                },
                nome: e.nome,
                t0_s: s(e.t0),
                t1_s: s(e.t1),
                espera_s: espera_s(track, e.etapa.inicio),
                props,
            }
        })
        .collect())
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
pub fn vistas(bytes: &[u8]) -> Vistas {
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
        duracao_s: crate::instala::segundos(&doc),
        ..Vistas::default()
    };
    for (tr, track) in doc.tracks.iter().enumerate() {
        let r = match familia(track) {
            Some(FamiliaDaMacro::Painter) => do_painter(tr, track),
            Some(FamiliaDaMacro::Motion) => do_motion(tr, track),
            None => continue,
        };
        match r {
            Ok(es) => {
                // A 1.ª etapa À VISTA desde a qual a trilha já não é a gravação (a de uma apagada é a
                // que ficou no lugar dela).
                let primeira = leitor(track)
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

/// Uma edição que muda o que toca (o nome que o artista escreve não muda).
fn muda_o_toque(e: &ph2d_macro::EdicaoDeEtapa) -> bool {
    e.espera_us != 0
        || !e.trocas.is_empty()
        || !e.insercoes.is_empty()
        || e.sai_da_gravacao
        || e.apagada
}

pub(crate) fn leitor(track: &Track) -> Option<&'static dyn ph2d_macro::LeitorDaFamilia> {
    match familia(track)? {
        FamiliaDaMacro::Painter => Some(&pi::LeitorDoPainter),
        FamiliaDaMacro::Motion => Some(&mo::LeitorDoMotion),
    }
}

/// As vistas do último objecto lido — refeitas só quando os bytes da `Macro` mudam (ler as etapas
/// do Painter aplica os passos de configuração de cada pincelada).
#[derive(Default)]
pub struct Cache {
    chave: Option<(u64, Vec<u8>)>,
    vistas: Vistas,
}

impl Cache {
    pub fn de(&mut self, bits: u64, bytes: &[u8]) -> &Vistas {
        let fresca = self
            .chave
            .as_ref()
            .is_some_and(|(b, v)| *b == bits && v == bytes);
        if !fresca {
            self.vistas = vistas(bytes);
            self.chave = Some((bits, bytes.to_vec()));
        }
        &self.vistas
    }
}

/// **Edita a etapa `k` da trilha `trilha`** da `Macro` do objecto `bits`; o clipe acompanha a duração.
/// `Ok(true)` = a trilha tem de ser conferida ([`assenta`]) — uma edição de VALOR do Painter.
///
/// # Errors
/// O porquê, para o painel o dizer.
pub fn edita(
    c: &mut Cena<'_>,
    bits: u64,
    trilha: usize,
    k: usize,
    campo: &CampoDaEtapa,
) -> Result<bool, ErroDaMacroView> {
    let Some(m) = c.sim.world().get::<Macro>(Entity::from_bits(bits)) else {
        return Ok(false);
    };
    let mut doc = ph2d_macro::codec::read(&m.doc).map_err(|x| erros::do_ficheiro(&x))?;
    let track = doc.tracks.get_mut(trilha).ok_or_else(erros::sem_etapa)?;
    let confere = if let CampoDaEtapa::Nome(n) = campo {
        let ancora = leitor(track)
            .and_then(|l| l.etapas(&track.events).get(k).copied())
            .ok_or_else(erros::sem_etapa)?
            .inicio;
        track.edicao_mut(ancora).nome = n.clone().filter(|n| !n.trim().is_empty());
        false
    } else {
        match familia(track) {
            Some(FamiliaDaMacro::Painter) => edita_painter(track, k, campo)?,
            Some(FamiliaDaMacro::Motion) => edita_motion(track, k, campo).map(|()| false)?,
            None => return Err(erros::sem_etapa()),
        }
    };
    escreve(c, bits, &doc);
    Ok(confere)
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

/// **Confere a trilha `trilha`** tocando-a do zero (`ph2d_tool_painter::macro_track::etapas::assenta`):
/// se a água gravada já não responde, a 1.ª etapa editada sai da gravação. Corre UMA vez, no quadro a
/// seguir ao último gesto (uma cor arrastada no selector manda uma edição por quadro, e conferir custa
/// tocar a macro inteira — plano W5 §3).
///
/// # Errors
/// O porquê, para o painel o dizer.
pub fn assenta(c: &mut Cena<'_>, bits: u64, trilha: usize) -> Result<(), ErroDaMacroView> {
    let Some(m) = c.sim.world().get::<Macro>(Entity::from_bits(bits)) else {
        return Ok(());
    };
    let mut doc = ph2d_macro::codec::read(&m.doc).map_err(|x| erros::do_ficheiro(&x))?;
    let track = doc.tracks.get_mut(trilha).ok_or_else(erros::sem_etapa)?;
    pi::assenta(track).map_err(|e| erros::do_painter(&e))?;
    escreve(c, bits, &doc);
    Ok(())
}

/// **Apaga a etapa `k` da trilha `trilha`** da `Macro` de `bits`: o tempo dela fecha-se e o clipe
/// encurta pela sincronização.
///
/// # Errors
/// O porquê, para o painel o dizer.
pub fn apaga(c: &mut Cena<'_>, bits: u64, trilha: usize, k: usize) -> Result<(), ErroDaMacroView> {
    let Some(m) = c.sim.world().get::<Macro>(Entity::from_bits(bits)) else {
        return Ok(());
    };
    let mut doc = ph2d_macro::codec::read(&m.doc).map_err(|x| erros::do_ficheiro(&x))?;
    let track = doc.tracks.get_mut(trilha).ok_or_else(erros::sem_etapa)?;
    let l = leitor(track).ok_or_else(erros::sem_etapa)?;
    if !track.apaga_etapa(l, k) {
        return Err(erros::sem_etapa());
    }
    escreve(c, bits, &doc);
    Ok(())
}

fn us(segundos: f64) -> i64 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "segundos de uma espera cabem num i64 de µs"
    )]
    let v = (segundos * 1e6).round() as i64;
    v
}

/// `Ok(true)` = uma edição de VALOR: a trilha tem de ser conferida.
fn edita_painter(
    track: &mut Track,
    k: usize,
    campo: &CampoDaEtapa,
) -> Result<bool, ErroDaMacroView> {
    let r = match *campo {
        CampoDaEtapa::Cor(c) => pi::edita_cor(track, k, c),
        CampoDaEtapa::Tamanho(px) => pi::edita_tamanho(track, k, px),
        CampoDaEtapa::Espera(s) => pi::espera(track, k, us(s)),
        CampoDaEtapa::Valor(_) | CampoDaEtapa::Nome(_) => return Err(erros::sem_etapa()),
    };
    r.map_err(|e| erros::de_edicao_do_painter(&e))?;
    Ok(!matches!(campo, CampoDaEtapa::Espera(_)))
}

fn edita_motion(track: &mut Track, k: usize, campo: &CampoDaEtapa) -> Result<(), ErroDaMacroView> {
    let r = match *campo {
        CampoDaEtapa::Valor(v) => mo::edita_valor(track, k, v),
        CampoDaEtapa::Espera(s) => mo::espera(track, k, us(s)),
        CampoDaEtapa::Cor(_) | CampoDaEtapa::Tamanho(_) | CampoDaEtapa::Nome(_) => {
            return Err(erros::sem_etapa());
        }
    };
    r.map_err(|e| erros::de_edicao_do_motion(&e))
}
