//! **A escrita por NOME** ([`StreamOp::WriteNamed`], doc 131 D.14) — o irmão de escrita do
//! `Project`. O nome é um text param; a coluna do artista é apresentada ao kernel sob o `alias`
//! estático e devolvida com o nome dele depois do despacho. A escada é a da CPU
//! (`motion.drive::channel::drive_named` e o «sem valor não se escreve» do `eval`).

use crate::plan::{DrivenParams, resolve_param};
use crate::{GpuColumn, GpuStream};
use ph2d_nodegraph::gpu::{GpuKernel, StreamOp};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;
use ph2d_nodegraph::port::Dim;

/// O que a escrita por nome pede ao sequenciador.
pub(crate) enum Escrita<'k> {
    /// Não é uma `WriteNamed`, ou o `when` é falso: o kernel registado, como sempre.
    Normal,
    /// A escada da CPU recusou: a saída é a entrada.
    Passa,
    /// Corre este kernel sobre a coluna emprestada ao alias; [`Renome::devolve`] no fim.
    Escreve(&'k GpuKernel, Renome),
}

/// A coluna do artista emprestada ao alias — e a coluna que já se chamava como o alias, guardada.
pub(crate) struct Renome {
    alias: &'static str,
    nome: String,
    guardada: Option<GpuColumn>,
}

impl Renome {
    /// Devolve a coluna ao nome do artista (e a guardada ao alias).
    pub(crate) fn devolve(self, out: &mut GpuStream) {
        if self.nome == self.alias {
            return;
        }
        if let Some(c) = out.cols.remove(self.alias) {
            out.cols.insert(self.nome, c);
        }
        if let Some(g) = self.guardada {
            out.cols.insert(self.alias.to_string(), g);
        }
    }
}

/// Decide a escrita por nome de `node` e, se ela corre, empresta a coluna ao alias em `base` e na
/// porta 0. A escada, degrau a degrau a da CPU: nome vazio · coluna de escrituração · coluna
/// existente que não é escalar · nenhuma leitura difundida (`ReadBroadcast`) com valor ⇒ [`Escrita::Passa`].
pub(crate) fn escrita_nomeada<'k>(
    op: Option<&'k StreamOp>,
    (graph, node, manifest, driven): (&Graph, NodeId, &NodeManifest, &DrivenParams),
    base: &mut GpuStream,
    inputs: &mut [GpuStream],
) -> Escrita<'k> {
    let Some(StreamOp::WriteNamed {
        text_param,
        alias,
        kernel,
        when,
    }) = op
    else {
        return Escrita::Normal;
    };
    if !when(&|n: &str| resolve_param(graph, node, manifest, n, driven)) {
        return Escrita::Normal;
    }
    let nome = graph
        .node_text_param_overrides(node)
        .and_then(|m| m.get(*text_param))
        .map_or("", String::as_str);
    let mut valores = kernel
        .bindings
        .iter()
        .filter(|b| b.port != 0 && b.access.broadcasts())
        .peekable();
    let sem_valor = valores.peek().is_some()
        && valores.all(|b| {
            inputs
                .get(b.port)
                .is_none_or(|s| s.count == 0 || s.cols.get(b.column).is_none_or(|c| c.dim != b.dim))
        });
    if sem_valor
        || nome.is_empty()
        || ph2d_nodegraph::attr::is_bookkeeping_column(nome)
        || base.cols.get(nome).is_some_and(|c| c.dim != Dim::Scalar)
    {
        return Escrita::Passa;
    }
    let mut renome = Renome {
        alias,
        nome: nome.to_string(),
        guardada: None,
    };
    if nome != *alias {
        renome.guardada = base.cols.remove(*alias);
        for s in std::iter::once(base).chain(inputs.first_mut()) {
            s.cols.remove(*alias);
            if let Some(c) = s.cols.remove(nome) {
                s.cols.insert((*alias).to_string(), c);
            }
        }
    }
    Escrita::Escreve(kernel, renome)
}
