//! ⭐ **O censo da lei da compactação** ([05 §4](../../../../docs/CartoesUI/05_ESPECIFICACAO_UI_COMPLETA.md)):
//! o que uma pintura real pôs nas linhas de cada cartão e em cada botão. Os portões
//! `lei_da_compactacao` e `nenhum_texto_sai_do_botao` correm a pintura por [`grava`] e leem as
//! [`Censo::linhas_grudadas`], [`Censo::irmaos_separados`] e [`Censo::textos_cortados`]. Fora de
//! [`grava`] custa só ler uma célula.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use std::cell::{Cell, RefCell};

/// Uma coluna de uma linha de cartão: o nome, se é curta pela lei e a largura que ela pede.
#[derive(Clone, Debug, PartialEq)]
pub struct CelulaMedida {
    pub nome: String,
    pub curta: bool,
    pub pede: f32,
}

/// Uma linha pintada num cartão.
#[derive(Clone, Debug, PartialEq)]
pub struct LinhaMedida {
    pub secao: NodeId,
    pub topo: f32,
    pub fundo: f32,
    pub x: f32,
    pub w: f32,
    pub celulas: Vec<CelulaMedida>,
}

/// Um texto pintado num botão, segmento ou caixa de marcar, contra a largura que ele tem.
#[derive(Clone, Debug, PartialEq)]
pub struct TextoMedido {
    pub texto: String,
    /// O texto mais o respiro de `metrics`.
    pub pede: f32,
    pub largura: f32,
}

/// O que uma pintura gravou.
#[derive(Clone, Debug, Default)]
pub struct Censo {
    pub linhas: Vec<LinhaMedida>,
    pub textos: Vec<TextoMedido>,
}

thread_local! {
    static GRAVACAO: RefCell<Option<Censo>> = const { RefCell::new(None) };
    static SECAO: Cell<Option<NodeId>> = const { Cell::new(None) };
}

/// Corre `f` e devolve o que os cartões pintaram.
pub fn grava<R>(f: impl FnOnce() -> R) -> (R, Censo) {
    GRAVACAO.with(|g| *g.borrow_mut() = Some(Censo::default()));
    let r = f();
    let censo = GRAVACAO.with(|g| g.borrow_mut().take()).unwrap_or_default();
    (r, censo)
}

/// Há gravação: quem mede só para o censo pergunta antes.
pub fn ligado() -> bool {
    GRAVACAO.with(|g| g.borrow().is_some())
}

/// A secção do corpo que se está a pintar (`None` ao sair dele).
///
/// O corpo corre de novo a cada quadro de aquecimento e em voo: vale a última passada.
pub(crate) fn na_secao(secao: Option<NodeId>) {
    SECAO.with(|s| s.set(secao));
    if let Some(id) = secao {
        GRAVACAO.with(|g| {
            if let Some(c) = g.borrow_mut().as_mut() {
                c.linhas.retain(|l| l.secao != id);
            }
        });
    }
}

/// Um corpo pintado fora da lista (o painel flutuante de um cartão) também conta, sob o id dele.
pub fn na_secao_de<R>(secao: NodeId, f: impl FnOnce() -> R) -> R {
    na_secao(Some(secao));
    let r = f();
    na_secao(None);
    r
}

/// A secção do corpo que se está a pintar.
pub(crate) fn secao_atual() -> Option<NodeId> {
    SECAO.with(Cell::get)
}

/// Anota uma linha do corpo do cartão corrente.
pub(crate) fn linha(
    (x, w): (f32, f32),
    topo: f32,
    fundo: f32,
    celulas: impl FnOnce() -> Vec<CelulaMedida>,
) {
    let Some(secao) = secao_atual() else {
        return;
    };
    GRAVACAO.with(|g| {
        if let Some(c) = g.borrow_mut().as_mut() {
            c.linhas.push(LinhaMedida {
                secao,
                topo,
                fundo,
                x,
                w,
                celulas: celulas(),
            });
        }
    });
}

/// Anota um texto de botão, de segmento ou de caixa de marcar.
pub(crate) fn texto(texto: &str, pede: f32, largura: f32) {
    GRAVACAO.with(|g| {
        if let Some(c) = g.borrow_mut().as_mut() {
            c.textos.push(TextoMedido {
                texto: texto.to_owned(),
                pede,
                largura,
            });
        }
    });
}

/// Um botão pintado à mão num corpo: o texto, e a fronteira dele como uma linha (lei 3).
pub(crate) fn fora_da_linha(texto_do_botao: &str, pede: f32, r: crate::zones::Rect) {
    texto(texto_do_botao, pede, r.w);
    linha((r.x, r.w), r.y, r.y + r.h, || {
        vec![CelulaMedida {
            nome: texto_do_botao.to_owned(),
            curta: false,
            pede,
        }]
    });
}

/// As linhas de uma secção de cima para baixo, juntando as que começam no mesmo `y`.
fn por_secao(censo: &Censo) -> Vec<(NodeId, Vec<LinhaMedida>)> {
    let mut secoes: Vec<(NodeId, Vec<LinhaMedida>)> = Vec::new();
    for l in &censo.linhas {
        match secoes.iter_mut().find(|(s, _)| *s == l.secao) {
            Some((_, v)) => v.push(l.clone()),
            None => secoes.push((l.secao, vec![l.clone()])),
        }
    }
    for (_, v) in &mut secoes {
        v.sort_by(|a, b| a.topo.total_cmp(&b.topo));
        let mut juntas: Vec<LinhaMedida> = Vec::with_capacity(v.len());
        for l in v.drain(..) {
            match juntas.last_mut() {
                Some(u) if mesma_linha(u, &l) => {
                    u.celulas.extend(l.celulas);
                    u.fundo = u.fundo.max(l.fundo);
                    let fim = (u.x + u.w).max(l.x + l.w);
                    u.x = u.x.min(l.x);
                    u.w = fim - u.x;
                }
                _ => juntas.push(l),
            }
        }
        *v = juntas;
    }
    secoes
}

/// Duas peças da mesma linha: lado a lado com as alturas cruzadas, ou uma dentro da banda da outra
/// (o controlo que o chamador põe sob um rótulo).
fn mesma_linha(a: &LinhaMedida, b: &LinhaMedida) -> bool {
    let lado_a_lado = b.x >= a.x + a.w - 0.5 || a.x >= b.x + b.w - 0.5;
    (lado_a_lado && b.topo < a.fundo) || (b.topo >= a.topo && b.fundo <= a.fundo + 0.5)
}

impl Censo {
    /// Lei 3: duas linhas seguidas mais perto do que `FIELD_ROW_GAP`.
    #[must_use]
    pub fn linhas_grudadas(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (_, v) in por_secao(self) {
            for p in v.windows(2) {
                let vao = p[1].topo - p[0].fundo;
                if vao < m::FIELD_ROW_GAP - 0.5 {
                    out.push(format!("{} | {} (vão {vao:.1})", nome(&p[0]), nome(&p[1])));
                }
            }
        }
        out
    }

    /// Lei 1: uma linha de um controlo curto seguida de outra que caberia ao lado dela.
    #[must_use]
    pub fn irmaos_separados(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (_, v) in por_secao(self) {
            let mut i = 0;
            while i + 1 < v.len() {
                let (a, b) = (&v[i], &v[i + 1]);
                let meia = (a.w.min(b.w) - m::FIELD_GAP) * 0.5;
                let cabe = |l: &LinhaMedida| {
                    matches!(l.celulas.as_slice(), [c] if c.curta && c.pede <= meia)
                };
                let colados = b.topo - a.fundo <= m::LABEL_TOP + m::FIELD_ROW_GAP;
                if colados && cabe(a) && cabe(b) {
                    out.push(format!("{} + {}", nome(a), nome(b)));
                    i += 2;
                } else {
                    i += 1;
                }
            }
        }
        out
    }

    /// Lei 7: um texto que não cabe com o respiro de `metrics`.
    #[must_use]
    pub fn textos_cortados(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .textos
            .iter()
            .filter(|t| t.pede > t.largura + 0.5)
            .map(|t| format!("{:?} pede {:.0} em {:.0}", t.texto, t.pede, t.largura))
            .collect();
        out.sort();
        out.dedup();
        out
    }
}

fn nome(l: &LinhaMedida) -> String {
    l.celulas
        .iter()
        .map(|c| c.nome.as_str())
        .collect::<Vec<_>>()
        .join(" · ")
}
