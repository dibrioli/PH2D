//! **O tutorial «fazer»** (W6, `docs/Macros/03_plano_W6.md`): a macro em `ModoDaMacro::Fazer` não
//! toca — o aluno faz cada etapa no objecto, e um balão diz qual. O critério de cada etapa deriva-o a
//! FAMÍLIA da gravação ([`TutorialDaFamilia`]); o progresso é a 1.ª etapa por cumprir no estado do
//! aluno, por isso o Ctrl+Z volta o passo sem gancho nenhum.
//!
//! O balão é chrome do herói; a família publica-o por [`set_balao`] (thread-local, como o painel
//! MACROS) e os botões voltam como `EditorAction::Macro` ([`crate::macro_edits::MacroRequest`]).

use crate::macro_edits::ErroDaMacroView;
use ph2d_a11y::NodeId;
use ph2d_macro::Track;
use std::cell::RefCell;

/// O que falta ao aluno numa etapa — os nós pelo nome canónico do tipo (`node.<tipo>.name`).
#[derive(Clone, Debug, PartialEq)]
pub enum PedidoDoTutorial {
    /// Antes das etapas: o objecto do tutorial tem de estar em edição.
    Edita,
    Acrescenta {
        tipo: String,
    },
    Liga {
        de: String,
        para: String,
    },
    Desliga {
        para: String,
    },
    Valor {
        no: String,
        param: String,
        valor: f64,
    },
    Apaga {
        tipo: String,
    },
    /// Antes de uma pincelada: a imagem do tutorial aberta em modo Paint.
    AbreParaPintar,
    /// Antes de uma pincelada: o meio da gravação (`chave_do_meio` = a chave i18n do nome dele).
    Meio {
        chave_do_meio: &'static str,
    },
    /// Pintar a área marcada ([`AreaNaImagem`]).
    Pinta,
}

/// Uma área numa imagem — o caminho de uma pincelada com o raio dela, em píxeis da imagem. A família
/// que a pinta (o Painter) desenha-a por cima da imagem `entity_bits`.
#[derive(Clone, Debug, PartialEq)]
pub struct AreaNaImagem {
    pub entity_bits: u64,
    pub pontos: Vec<[f32; 2]>,
    pub raio: f32,
}

/// Uma etapa COM critério, no estado do aluno.
#[derive(Clone, Debug, PartialEq)]
pub struct EtapaDoTutorial {
    /// A etapa a contar da GRAVAÇÃO (o endereço de [`TutorialDaFamilia::mostra`]).
    pub k: usize,
    /// O que ainda falta; `None` = cumprida.
    pub falta: Option<PedidoDoTutorial>,
    /// Onde o aluno o faz: ids de interação que o painel da família regista por quadro.
    pub alvos: Vec<NodeId>,
    /// Ou uma área numa imagem.
    pub area: Option<AreaNaImagem>,
}

/// O que uma família faz por um tutorial dela. Os métodos por omissão dizem «esta família não tem
/// tutorial»: nenhuma etapa, e os botões recusam com a frase.
pub trait TutorialDaFamilia {
    /// As etapas com critério da trilha `track` no estado do objecto `dono` (o `StableId`).
    /// `versao` muda sempre que os bytes da `Macro` mudam (a cache dos critérios é dela).
    ///
    /// # Errors
    /// A gravação não se lê.
    fn avalia(
        &mut self,
        _dono: u64,
        _track: &Track,
        _versao: u64,
    ) -> Result<Vec<EtapaDoTutorial>, ErroDaMacroView> {
        Ok(Vec::new())
    }
    /// **«Mostra-me»**: faz a etapa `k` pelo aluno, pelas portas do editor (um passo de undo).
    ///
    /// # Errors
    /// O porquê.
    fn mostra(
        &mut self,
        _dono: u64,
        _track: &Track,
        _versao: u64,
        _k: usize,
    ) -> Result<(), ErroDaMacroView> {
        Err(crate::macro_etapas::erros::sem_etapa())
    }
    /// **«Recomeçar»**: o objecto volta ao estado inicial da trilha (um passo de undo).
    ///
    /// # Errors
    /// O porquê.
    fn recomeca(&mut self, _dono: u64, _track: &Track) -> Result<(), ErroDaMacroView> {
        Err(crate::macro_etapas::erros::sem_etapa())
    }
}

/// A gravadora de uma família que também ensina — o que a composição passa à família MACROS.
pub trait GravadoraQueEnsina: ph2d_macro::Gravadora + TutorialDaFamilia {}
impl<T: ph2d_macro::Gravadora + TutorialDaFamilia> GravadoraQueEnsina for T {}

/// **O balão** como o herói o pinta.
#[derive(Clone, Debug, PartialEq)]
pub struct BalaoDoTutorial {
    /// Os bits da entidade com a `Macro` — o endereço dos botões.
    pub entity_bits: u64,
    /// A etapa corrente a contar de 1 entre as que têm critério; `0` = o preparo.
    pub passo: usize,
    pub total: usize,
    /// O que fazer; `None` = o tutorial acabou.
    pub pedido: Option<PedidoDoTutorial>,
    /// O texto que o autor escreveu para a etapa (conteúdo, não UI).
    pub dica: Option<String>,
    pub alvos: Vec<NodeId>,
    pub area: Option<AreaNaImagem>,
    /// A última recusa de um botão.
    pub erro: Option<ErroDaMacroView>,
}

impl BalaoDoTutorial {
    /// «Mostra-me» faz sentido: há uma etapa por fazer (e não um preparo).
    #[must_use]
    pub fn pode_mostrar(&self) -> bool {
        self.pedido.as_ref().is_some_and(|p| {
            !matches!(
                p,
                PedidoDoTutorial::Edita
                    | PedidoDoTutorial::AbreParaPintar
                    | PedidoDoTutorial::Meio { .. }
            )
        })
    }

    /// **O título e a frase** do balão, traduzidos agora.
    #[must_use]
    pub fn frase(&self) -> (String, String) {
        use ph2d_i18n::{tr, tr_with};
        let Some(p) = &self.pedido else {
            return (
                tr("panel.macros.tutorial.done_title").to_owned(),
                tr("panel.macros.tutorial.done").to_owned(),
            );
        };
        let titulo = if self.passo == 0 {
            tr("panel.macros.tutorial.before").to_owned()
        } else {
            tr_with(
                "panel.macros.tutorial.step",
                &[("n", &self.passo), ("total", &self.total)],
            )
        };
        let corpo = match p {
            PedidoDoTutorial::Edita => tr("panel.macros.tutorial.edit").to_owned(),
            PedidoDoTutorial::Acrescenta { tipo } => {
                tr_with("panel.macros.tutorial.add", &[("node", &no(tipo))])
            }
            PedidoDoTutorial::Liga { de, para } => tr_with(
                "panel.macros.tutorial.connect",
                &[("from", &no(de)), ("to", &no(para))],
            ),
            PedidoDoTutorial::Desliga { para } => {
                tr_with("panel.macros.tutorial.disconnect", &[("to", &no(para))])
            }
            PedidoDoTutorial::Valor {
                no: t,
                param,
                valor,
            } => tr_with(
                "panel.macros.tutorial.value",
                &[
                    (
                        "param",
                        &tr_se_ha(&format!("node.{t}.param.{param}")).unwrap_or(param.as_str()),
                    ),
                    ("node", &no(t)),
                    ("value", &crate::interaction::format_number(*valor)),
                ],
            ),
            PedidoDoTutorial::Apaga { tipo } => {
                tr_with("panel.macros.tutorial.delete", &[("node", &no(tipo))])
            }
            PedidoDoTutorial::AbreParaPintar => tr("panel.macros.tutorial.open_paint").to_owned(),
            PedidoDoTutorial::Meio { chave_do_meio } => tr_with(
                "panel.macros.tutorial.medium",
                &[("medium", &tr(chave_do_meio))],
            ),
            PedidoDoTutorial::Pinta => tr("panel.macros.tutorial.paint").to_owned(),
        };
        (titulo, corpo)
    }
}

/// A tradução de `chave`, ou `None` se a tabela não a tem.
fn tr_se_ha(chave: &str) -> Option<&'static str> {
    let s = ph2d_i18n::tr(chave);
    (s != chave).then_some(s)
}

/// O nome do tipo de nó no catálogo (`node.<tipo>.name`), ou o tipo cru.
fn no(tipo: &str) -> &str {
    tr_se_ha(&format!("node.{tipo}.name")).unwrap_or(tipo)
}

/// O cartão do balão: come os cliques que caem nele.
pub const TUTORIAL_BALAO: NodeId = ph2d_tool_registry::hash_node_id("macro.tutorial.balloon");
/// «Mostra-me».
pub const TUTORIAL_MOSTRA: NodeId = ph2d_tool_registry::hash_node_id("macro.tutorial.show_me");
/// «Recomeçar».
pub const TUTORIAL_RECOMECA: NodeId = ph2d_tool_registry::hash_node_id("macro.tutorial.restart");

thread_local! {
    static BALAO: RefCell<Option<BalaoDoTutorial>> = const { RefCell::new(None) };
}

/// A composição publica o balão do quadro (`None` = não há tutorial em curso).
pub fn set_balao(b: Option<BalaoDoTutorial>) {
    BALAO.with(|s| *s.borrow_mut() = b);
}

/// O balão publicado.
#[must_use]
pub fn balao() -> Option<BalaoDoTutorial> {
    BALAO.with(|s| s.borrow().clone())
}
