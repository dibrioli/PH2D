//! **A TABELA VIVA** — a lista gerada, resolvida uma vez (plano UI/UX W8b.2).
//!
//! ⚠️ **Uma tabela, quatro consumidores.** `paint`, `populate`, `event` e a varredura de seam
//! percorrem ESTA lista. É a lei do `SECTIONS` do painel de física, e é ela que faz do requisito
//! mais duro da W8b — *o gerado passa os gates que o repo cobra de código escrito à mão* — algo
//! **estrutural** em vez de disciplinar: uma row não pode ser pintada-e-não-registada, porque as
//! duas metades leem a mesma lista.
//!
//! # Nem toda row é um CONTROLE, e essa é a segunda metade da lei do §4
//!
//! A W8b.1 estabeleceu que *só quem VESTE vira row*. Aqui a mesma frase desce um degrau: **só quem
//! RESPONDE vira widget interativo**. Um `Divider`, um `SectionHeader`, um `Spinner` — eles
//! desenham e não têm nada a dizer sobre um clique. Registá-los faria o clique acendê-los e não
//! fazer nada, que é o item-de-menu-morto pintado com outro nome.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::widget::WidgetKind;
use std::sync::OnceLock;

/// Uma linha do painel, com o id já derivado da chave.
pub struct Row {
    pub kind: WidgetKind,
    /// O rótulo que a row mostra — o `Name` que o artista digitou.
    pub label: &'static str,
    /// A chave estável, o slug do rótulo. O id sai dela.
    pub key: &'static str,
    pub id: NodeId,
}

impl Row {
    /// **Esta row responde a um gesto?**
    ///
    /// ⚠️ A porta é ÚNICA: o `populate` pergunta para decidir se regista, o `paint` para decidir
    /// se publica um retângulo de hit, e o `event` para decidir se rota. Três cópias divergiriam
    /// no dia em que um tipo mudasse de lado — e o modo de falha de cada divergência é diferente
    /// (registado-e-sem-rect = nunca clicado · rect-e-sem-registo = clique descartado em silêncio
    /// · rotado-e-sem-registo = braço morto).
    #[must_use]
    pub fn is_control(&self) -> bool {
        matches!(
            self.kind,
            WidgetKind::Button
                | WidgetKind::Toggle
                | WidgetKind::Checkbox
                | WidgetKind::Slider
                | WidgetKind::TextInput
                | WidgetKind::Tag
                | WidgetKind::ListItem
                | WidgetKind::NumberInput
        )
    }
}

/// **As rows deste painel** — a tabela gerada, com os ids resolvidos.
///
/// ⚠️ `OnceLock` porque o id sai de um hash de string em runtime (o gerador não cunha ids — ver o
/// doc de [`ids::authored_row_id`]), e recomputá-lo por frame seria um `format!` por row por
/// quadro. O padrão é o `rows()` do painel de Wet Tuning.
#[must_use]
pub fn rows() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| {
        crate::generated::ROWS
            .iter()
            .map(|&(kind, label, key)| Row {
                kind,
                label,
                key,
                id: ids::authored_row_id(key),
            })
            .collect()
    })
}

/// A row a que `id` pertence, se alguma.
#[must_use]
pub fn row_for(id: NodeId) -> Option<&'static Row> {
    rows().iter().find(|r| r.id == id)
}

/// **Quantos rótulos o artista repetiu** — o readout que o painel mostra em vez de desempatar.
///
/// ⚠️ Duas rows de mesmo rótulo têm a mesma chave, logo o mesmo id: elas passam a ser **um**
/// controle sob o rato, e mexer numa mexe na outra. Inventar um sufixo (`opacity_2`) daria uma
/// chave que o artista não escreveu, não vê e não consegue prever — e ela mudaria sozinha no dia
/// em que ele reordenasse os filhos. O painel **diz** que há repetidos; quem desempata é ele, na
/// Hierarquia, com o nome.
#[must_use]
pub fn duplicate_keys() -> usize {
    let all = rows();
    all.iter()
        .enumerate()
        .filter(|(i, r)| all[..*i].iter().any(|p| p.key == r.key))
        .count()
}

#[cfg(test)]
#[path = "rows_tests.rs"]
mod tests;
