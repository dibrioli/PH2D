#![forbid(unsafe_code)]
//! ⭐⭐⭐ **A SECÇÃO DO GIZMO — a declaração que os nós de POSIÇÃO partilham.**
//!
//! > **Ordem do dono, 2026-09-19:** *«Para nós como Grid e outros similares vamos criar uma seção
//! > para tamanho absoluto do gizmo assim como algumas opções de forma para ele como cruz, circulo
//! > e rect.»*
//!
//! Um nó que só entrega POSIÇÕES não vira pixel sem um `motion.duplicator` — ele aparece como
//! gizmo (doc 115 §32). Esta crate é o que dá ao artista o controlo **desse** gizmo, no cartão do
//! próprio nó.
//!
//! ## Porque é uma CRATE e não dois `ParamSpec` copiados
//!
//! A população é de **14 fontes de posições**, derivada do manifesto pela sonda
//! `quem_e_como_o_grid` (emite instâncias · não recebe instâncias). Dois params copiados catorze
//! vezes são catorze sítios onde o default, o rótulo ou a faixa podem divergir — e o rótulo é a
//! **face do artista**. Aqui é uma declaração, N consumidores.
//!
//! ## Porque as escolhas viajam em COLUNAS
//!
//! O gizmo é resolvido no **SINK**, e o nó que o produziu está muitas arestas acima. A ligação é a
//! que esta casa já usa para tudo o que é por-elemento: o nó escreve uma coluna e ela **viaja**.
//!
//! ⚠️ **E isso foi MEDIDO antes de ser desenhado** (`uma_coluna_nova_sobrevive_a_cadeia`): uma
//! coluna inventada atravessa `motion.move`, `motion.scale`, `motion.rotate`, `motion.clone` (que a
//! replica por cópia) e `motion.cull`. *Sem essa medição, a secção teria de viver no sink — e o
//! dono pediu-a no nó.*
//!
//! ⭐ **Por-elemento, e é isso que faz uma JUNÇÃO ler-se certa:** um `motion.mixer` de uma grelha
//! com uma dispersão mostra **duas** formas, cada uma a do seu produtor — que é o que o artista
//! autorou. Um param de sink daria uma resposta só a duas perguntas.

use ph2d_node_registry::{ParamUiHint, ParamWidget};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::node::ParamSpec;

/// A coluna da FORMA do gizmo. Ausente ⇒ a forma de omissão (a cruz).
pub const FORMA_COL: &str = "gizmo_forma";
/// A coluna do TAMANHO ABSOLUTO do gizmo, em pixels. Ausente ou `0` ⇒ derivado da peça.
pub const TAMANHO_COL: &str = "gizmo_tamanho";

/// O param da forma, no cartão.
pub const FORMA: &str = "gizmo_shape";
/// O param do tamanho absoluto, no cartão.
pub const TAMANHO: &str = "gizmo_size";

/// O título da secção — em inglês, porque a face do artista sai por i18n (HR-15).
pub const SECCAO: &str = "Gizmo";

/// **As formas, e a escada é a do `enum` da UI.** ⚠️ `0` é a CRUZ porque é o desenho que já
/// shipava: um documento gravado antes desta secção lê `0` e continua a ver o que via.
pub const CRUZ: f32 = 0.0;
/// Um anel.
pub const CIRCULO: f32 = 1.0;
/// Um rectângulo.
pub const RECT: f32 = 2.0;
/// Quantas formas o selector oferece — ⛔ **contado aqui**, nunca escrito no pintor.
pub const FORMAS: usize = 3;

/// **O TECTO do tamanho absoluto, em pixels, e ele nomeia o recurso: o ECRÃ.**
///
/// Um glifo maior do que a menor dimensão de uma janela de trabalho deixa de marcar um ponto e
/// passa a tapar a cena. ⛔ Não é um «razoável»: `256 px` é um terço da altura útil de um canvas de
/// `~800`, que é o ponto em que uma marca deixa de ser uma marca.
pub const TAMANHO_MAX: f32 = 256.0;

/// ⛔⛔ **UM NÓ ESCREVE ESTAS CONSTANTES PELO NOME, nunca `SPECS[0]`.** A tabela de um nó cresce, e
/// um índice escrito à mão aponta para outra coisa no dia seguinte — é a armadilha que as quatro
/// constantes `SHAPES.len() − N` do catálogo do modelador já pagaram, com o botão *Extrude* a abrir
/// o diálogo errado **sem erro nenhum**.
///
/// ⚠️⚠️ **E as tabelas do nó SOMAM-SE, nunca se registam duas vezes:** o
/// `NodeRegistry::register_param_ui` é *«a última escrita vence»* ⇒ um segundo registo **APAGA** as
/// dicas que o nó já tinha. A 1.ª redacção desta wave fez isso ao `motion.grid`, e **seis** censos
/// do cartão reprovaram em voz alta (*«6 of 819 params have no `ParamUiHint`»*, com `rows`, `cols`,
/// `gap_x`, `gap_y`, `shape` e `inner` nomeados) — *a cura é concatenar na tabela do nó*.
///
/// Os dois `ParamSpec` que um nó acrescenta ao manifesto dele.
///
/// ⚠️ **`0` em `gizmo_size` quer dizer «derivado da peça»**, que é o comportamento de hoje **ao
/// bit** — e é por isso que acrescentar esta secção não muda uma única cena existente.
pub const SPEC_FORMA: ParamSpec = ParamSpec {
    name: FORMA,
    default: CRUZ,
};
/// Ver [`SPECS`].
pub const SPEC_TAMANHO: ParamSpec = ParamSpec {
    name: TAMANHO,
    default: 0.0,
};
pub const SPECS: &[ParamSpec] = &[SPEC_FORMA, SPEC_TAMANHO];

/// As dicas de UI — o rótulo, a faixa e o widget de cada um.
pub const HINT_FORMA: ParamUiHint = ParamUiHint {
    param: FORMA,
    label: "Gizmo Shape",
    min: 0.0,
    max: (FORMAS - 1) as f32,
    step: 1.0,
    widget: ParamWidget::Enum {
        labels: &["Cross", "Circle", "Rect"],
    },
};
/// Ver [`HINTS`].
pub const HINT_TAMANHO: ParamUiHint = ParamUiHint {
    param: TAMANHO,
    label: "Gizmo Size",
    min: 0.0,
    max: TAMANHO_MAX,
    step: 1.0,
    widget: ParamWidget::Slider,
};
pub const HINTS: &[ParamUiHint] = &[HINT_FORMA, HINT_TAMANHO];

/// **A SECÇÃO no cartão** — os dois params sob um título, e ela **NASCE FECHADA**: um cartão que
/// abre com duas linhas a mais empurra para baixo as que o artista veio ver.
pub const GRUPOS: &[ph2d_node_registry::ParamGroup] = &[
    ph2d_node_registry::ParamGroup {
        param: FORMA,
        group: SECCAO,
        folded: true,
    },
    ph2d_node_registry::ParamGroup {
        param: TAMANHO,
        group: SECCAO,
        folded: true,
    },
];

/// ⭐⭐ **ESCREVE as duas colunas na corrente que o nó emite** — a porta única.
///
/// ⚠️ **Ela não escreve nada quando os dois params estão no ponto NEUTRO**, e isso é a lei que
/// mantém toda cena de hoje **byte-idêntica**: uma coluna a mais muda a revisão de conteúdo da
/// corrente e, com ela, o memo de tudo o que está a jusante.
pub fn escreve(ctx: &EvalCtx<'_>, out: &mut Stream) {
    let forma = ctx.param(FORMA);
    let tamanho = ctx.param(TAMANHO);
    let n = out.count();
    if n == 0 {
        return;
    }
    if forma != CRUZ {
        out.set(FORMA_COL.to_string(), Column::Scalar(vec![forma; n]));
    }
    if tamanho > 0.0 {
        out.set(TAMANHO_COL.to_string(), Column::Scalar(vec![tamanho; n]));
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
