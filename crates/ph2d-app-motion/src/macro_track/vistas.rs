//! **As etapas do Motion pela porta comum** ([`EtapasDaFamilia`], W5e): o que o painel MACROS e a
//! linha «Macro steps» mostram de uma trilha do Motion, as edições delas e o erro de cada uma numa
//! chave `panel.macros.error.*`. A composição injecta [`EtapasDoMotion`] na família MACROS.

use super::etapas::{self as mo, ErroDeEdicaoDoMotion};
use super::tocar::ErroDaMacro as Mo;
use ph2d_editor_core::macro_edits::{
    CampoDaEtapa, ErroDaMacroView, EtapaView, FamiliaDaMacro, PropsDaEtapa, TipoDaEtapaView,
};
use ph2d_editor_core::macro_etapas::erros::{self as e, erro, erro_com};
use ph2d_editor_core::macro_etapas::{EtapasDaFamilia, espera_s, segundos, us};
use ph2d_macro::{LeitorDaFamilia, Track};

/// **O erro da reprodução de uma trilha do Motion**, como o artista o lê.
#[must_use]
pub fn do_motion(x: &Mo) -> ErroDaMacroView {
    match x {
        Mo::Ficheiro(c) => e::do_ficheiro(c),
        Mo::Inicial(_) => erro(e::INICIAL),
        Mo::Passo(_) => erro(e::ESTRAGADA),
        Mo::PortaRecusada(p) => erro_com(e::PORTA, p.clone()),
        Mo::PassoDesconhecido(_) | Mo::IntencaoDesconhecida(_) => erro(e::PASSO_NOVO),
        Mo::Nome(n) => erro_com(e::NOME, n.clone()),
        Mo::Alvo => erro(e::ALVO),
        Mo::Documento(d) => erro_com(e::DOCUMENTO, d.clone()),
        Mo::ArestasDoDocumento => erro(e::DOCUMENTO),
        Mo::Divergiu => erro(e::DIVERGIU),
        Mo::SemGrafo => erro(e::SEM_GRAFO),
        Mo::Edicao(x) => e::de_edicao(x),
    }
}

fn de_edicao(x: &ErroDeEdicaoDoMotion) -> ErroDaMacroView {
    match x {
        ErroDeEdicaoDoMotion::SemEtapa(_) => erro(e::SEM_ETAPA),
        ErroDeEdicaoDoMotion::NaoEParam(_) => erro(e::NAO_VALOR),
    }
}

fn tipo(t: mo::TipoDaEtapa) -> TipoDaEtapaView {
    match t {
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
    }
}

/// As etapas do Motion, para a família MACROS (que não depende desta).
pub struct EtapasDoMotion;

impl EtapasDaFamilia for EtapasDoMotion {
    fn familia(&self) -> FamiliaDaMacro {
        FamiliaDaMacro::Motion
    }

    fn chave(&self) -> &'static str {
        super::FAMILIA
    }

    fn leitor(&self) -> &'static dyn LeitorDaFamilia {
        &mo::LeitorDoMotion
    }

    fn vistas(&self, trilha: usize, track: &Track) -> Result<Vec<EtapaView>, ErroDaMacroView> {
        let es = mo::etapas(track).map_err(|x| do_motion(&x))?;
        Ok(es
            .into_iter()
            .map(|x| {
                let props = match &x.tipo {
                    mo::TipoDaEtapa::Param { para, .. } => PropsDaEtapa::Param { valor: *para },
                    _ => PropsDaEtapa::Nenhuma,
                };
                EtapaView {
                    trilha,
                    k: x.k,
                    familia: FamiliaDaMacro::Motion,
                    tipo: tipo(x.tipo),
                    nome: x.nome,
                    dica: None,
                    t0_s: segundos(x.t0),
                    t1_s: segundos(x.t1),
                    espera_s: espera_s(track, x.etapa.inicio),
                    props,
                }
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
            CampoDaEtapa::Valor(v) => mo::edita_valor(track, k, v),
            CampoDaEtapa::Espera(s) => mo::espera(track, k, us(s)),
            CampoDaEtapa::Cor(_)
            | CampoDaEtapa::Tamanho(_)
            | CampoDaEtapa::Nome(_)
            | CampoDaEtapa::Dica(_) => {
                return Err(e::sem_etapa());
            }
        };
        r.map_err(|x| de_edicao(&x))
    }
}
