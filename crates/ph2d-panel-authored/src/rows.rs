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
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::widget::WidgetKind;
use ph2d_vector::BezPath;
use std::sync::OnceLock;

/// **Uma linha TAL COMO O GERADOR A ESCREVE** — o dado bruto, antes de o id sair da chave.
///
/// ⚠️ **Era uma TUPLA, e o `clippy::type_complexity` foi quem disse que ela tinha crescido**
/// (`(WidgetKind, &str, &str, Option<[u8; 4]>, Option<&str>)`, cinco posições). O doc do
/// `ph2d-ui-codegen` justificava a tupla dizendo que *"uma struct teria de ser definida em algum
/// lugar, e o único lugar honesto seria junto do `WidgetKind`"* — e a premissa estava errada: o
/// lugar honesto é **aqui**, na crate que COMPILA o gerado. O emissor escreve o nome deste tipo
/// sem o conhecer, exactamente como já escreve `WidgetKind::Slider`.
///
/// ⚠️ E o ganho não é cosmético: uma tupla é **posicional**, então cada campo novo re-escreve todo
/// consumidor que a desestrutura — aconteceu duas vezes no mesmo dia, com a cor e com o glifo. Um
/// campo novo aqui não toca em quem não o lê.
pub struct RowConst {
    pub kind: WidgetKind,
    pub label: &'static str,
    pub key: &'static str,
    pub rgba: Option<[u8; 4]>,
    /// O glifo, em texto SVG — ver [`Row::icon`], que é a curva reconstituída dele.
    pub icon: Option<&'static str>,
    /// O slug do ícone ESCOLHIDO do catálogo — a outra rota, exclusiva com a de cima.
    pub icon_slug: Option<&'static str>,
}

/// Uma linha do painel, com o id já derivado da chave.
pub struct Row {
    pub kind: WidgetKind,
    /// O rótulo que a row mostra — o `Name` que o artista digitou.
    pub label: &'static str,
    /// A chave estável, o slug do rótulo. O id sai dela.
    pub key: &'static str,
    pub id: NodeId,
    /// A cor que esta row mostra, quando o tipo dela É uma cor — ver [`ph2d_editor_core::widget::SkinParam`].
    pub rgba: Option<[u8; 4]>,
    /// O glifo DESENHADO, quando o tipo dela É um botão de ícone e o artista não escolheu um.
    ///
    /// ⚠️ **Curva, e não o texto SVG que o gerado carrega** — a conversão acontece UMA vez, aqui
    /// no `OnceLock`, ao lado da derivação do id. Reparsear por quadro seria um parser de string
    /// no laço de pintura.
    pub icon: Option<BezPath>,
    /// O ícone ESCOLHIDO do catálogo, resolvido do slug. ⚠️ Um slug que este build não conhece
    /// vira `None` e o desenho assume — o mesmo canal de compatibilidade do `kind`.
    pub icon_id: Option<IconId>,
}

impl Row {
    /// **Esta row responde a um gesto?**
    ///
    /// ⚠️ A porta é ÚNICA: o `populate` pergunta para decidir se regista, o `paint` para decidir
    /// se publica um retângulo de hit, e o `event` para decidir se rota. Três cópias divergiriam
    /// no dia em que um tipo mudasse de lado — e o modo de falha de cada divergência é diferente
    /// (registado-e-sem-rect = nunca clicado · rect-e-sem-registo = clique descartado em silêncio
    /// · rotado-e-sem-registo = braço morto).
    /// **Esta row DOBRA a seção sob ela?**
    ///
    /// ⚠️ **Pergunta SEPARADA da [`Self::is_control`], e a separação é o ponto:** um cabeçalho de
    /// seção não é um controle — ele não tem valor, não emite intent, e registá-lo como widget
    /// daria um `InteractiveState` que ninguém lê. Mas ele **é clicável**, porque dobrar é um
    /// gesto de VISTA e não uma edição do documento.
    ///
    /// Colapsar as duas perguntas numa quebraria uma das duas metades: ou o cabeçalho vira um
    /// controle que emite um intent que ninguém pediu, ou ele fica sem retângulo de hit e a seção
    /// nunca dobra — *pintada, com o chevron desenhado, e morta sob o rato*.
    #[must_use]
    pub fn folds_a_section(&self) -> bool {
        matches!(self.kind, WidgetKind::SectionHeader)
    }

    /// **Esta row quer o PONTEIRO?** — a porta única do retângulo de hit.
    ///
    /// A soma das duas perguntas acima, e ela existe para o `paint` não as repetir: um `||`
    /// escrito no laço de pintura seria a regra que o próximo caso especial nasce sem.
    #[must_use]
    pub fn wants_pointer(&self) -> bool {
        self.is_control() || self.folds_a_section()
    }

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
                | WidgetKind::IconButton
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
            .map(|r| Row {
                kind: r.kind,
                label: r.label,
                key: r.key,
                id: ids::authored_row_id(r.key),
                rgba: r.rgba,
                // ⚠️ `ok()` e não `expect`: a string vem de código GERADO por `to_svg`, então
                // malformada ela não pode estar — mas um painel que entra em pânico ao abrir
                // trocaria um ícone ausente pela aplicação inteira. Sem curva, o botão desenha a
                // moldura, que é o neutro que o pintor já tem.
                icon: r.icon.and_then(|d| BezPath::from_svg(d).ok()),
                icon_id: r.icon_slug.and_then(IconId::from_slug),
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
