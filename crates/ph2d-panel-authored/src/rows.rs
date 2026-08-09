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
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::widget::WidgetKind;
use ph2d_vector::BezPath;
use std::cell::RefCell;
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
    /// Os rótulos das opções, para a família de LISTA — os `Name` dos filhos que o artista
    /// desenhou dentro do controle. Vazio para todo o resto.
    pub options: &'static [&'static str],
}

/// Uma linha do painel, com o id já derivado da chave.
///
/// ⚠️ **As strings são OWNED, e é isso que deixa a tabela ser VIVA.** O `RowConst` fica com
/// `&'static str` porque é código gerado; uma row que nasce do documento nasce do `Name` que o
/// artista está a digitar agora, e esse nome não é `'static`. É a mesma divisão de sempre: o
/// `RowConst` é a forma SERIALIZADA, o `Row` é a RESOLVIDA.
pub struct Row {
    pub kind: WidgetKind,
    /// O rótulo que a row mostra — o `Name` que o artista digitou.
    pub label: String,
    /// A chave estável, o slug do rótulo. O id sai dela.
    pub key: String,
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
    /// **Os rótulos das opções** — os `Name` dos filhos que o controle POSSUI.
    ///
    /// ⚠️ Uma lista VAZIA num tipo de lista é um estado legítimo, não um erro: é o que um
    /// controle sem filhos descreve, e a pele desenha a moldura vazia. Inventar uma opção seria
    /// mostrar ao artista um item que o documento dele não tem.
    pub options: Vec<String>,
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
                | WidgetKind::Tabs
                | WidgetKind::RadioGroup
                | WidgetKind::SegmentedAdaptive
                | WidgetKind::Dropdown
        )
    }
}

/// **Uma row resolvida a partir da forma que o gerador escreve.**
///
/// ⚠️ `ok()` e não `expect`: a string vem de código GERADO por `to_svg`, então malformada ela não
/// pode estar — mas um painel que entra em pânico ao abrir trocaria um ícone ausente pela
/// aplicação inteira. Sem curva, o botão desenha a moldura, que é o neutro que o pintor já tem.
fn resolve(kind: WidgetKind, label: &str, key: &str, rgba: Option<[u8; 4]>) -> Row {
    Row {
        kind,
        label: label.to_string(),
        key: key.to_string(),
        id: ids::authored_row_id(key),
        rgba,
        icon: None,
        icon_id: None,
        options: Vec::new(),
    }
}

/// **A tabela COMPILADA** — o que o gerador escreveu e alguém colou.
///
/// ⚠️ `OnceLock` porque o id sai de um hash de string em runtime (o gerador não cunha ids — ver o
/// doc de [`ids::authored_row_id`]), e recomputá-lo por frame seria um `format!` por row por
/// quadro. O padrão é o `rows()` do painel de Wet Tuning.
///
/// ⚠️ **`pub` para os GATES, e não para pintar.** Quem pinta pergunta ao [`with_rows`], que aplica
/// a precedência; ler daqui no laço de pintura seria ignorar o documento aberto — a segunda porta
/// que esta wave existe para não ter. Os gates a usam quando o que eles pinam é *o que o gerador
/// escreveu*, que é uma pergunta legítima e diferente.
pub fn baked() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| {
        crate::generated::ROWS
            .iter()
            .map(|r| Row {
                icon: r.icon.and_then(|d| BezPath::from_svg(d).ok()),
                icon_id: r.icon_slug.and_then(IconId::from_slug),
                options: r.options.iter().map(|o| (*o).to_string()).collect(),
                ..resolve(r.kind, r.label, r.key, r.rgba)
            })
            .collect()
    })
}

// **A tabela VIVA** — publicada pelo host a partir do documento aberto, quando há um.
//
// ⚠️ **`thread_local` e não um lock global**, por duas razões que apontam para o mesmo lado:
// publicar e pintar acontecem os dois na thread da UI, então um lock seria custo por quadro para
// coordenar uma thread consigo mesma; e um estado global partilhado entre TESTES que correm em
// paralelo é a flake que a `ph2d-painter-brush` já pagou (contadores atómicos com uma trava ao
// lado). Por thread, a interferência é **estruturalmente impossível** em vez de disciplinada.
//
// ⚠️ O modo de falha de alguém publicar noutra thread é ver a tabela COMPILADA — degrada para o
// painel que o build traz, nunca para dados errados.
//
// E ela é `Option` porque *não há documento autorado* e *há um documento com zero rows* são
// coisas diferentes: a primeira cai na tabela compilada, a segunda mostra um painel vazio, que é
// o que o artista de facto tem.
thread_local! {
    static LIVE: RefCell<Option<Vec<Row>>> = const { RefCell::new(None) };
}

/// **O host publica o que o documento descreve** — `None` devolve o painel à tabela compilada.
///
/// ⚠️ Ele publica **na mudança, não por quadro**: quem chama compara a descrição com a anterior e
/// só resolve quando ela difere. Resolver por quadro seria um hash de string e um parser de SVG
/// por row no laço de pintura — exactamente o que o `OnceLock` da tabela compilada existe para
/// evitar.
pub fn set_live_rows(rows: Option<Vec<Row>>) {
    LIVE.with_borrow_mut(|slot| *slot = rows);
}

/// **A tabela que este painel pinta AGORA** — a viva se houver, senão a compilada.
///
/// ⚠️ A precedência é a lei desta wave e mora aqui, num lugar só: **o documento aberto vence o
/// código colado**. Quatro consumidores a perguntam (`paint`, `populate`, `event` e a varredura de
/// seam), e duas cópias divergiriam com modos de falha diferentes — o painel pintaria a tabela
/// viva e registaria a compilada, que é a row viva sob o rato e morta ao clique.
///
/// ⚠️ E é uma CLOSURE porque uma tabela viva não é `'static`: devolver `&[Row]` obrigaria a vazar
/// a cada edição, o que é memória que só cresce numa sessão de autoria.
pub fn with_rows<R>(f: impl FnOnce(&[Row]) -> R) -> R {
    LIVE.with_borrow(|live| match live.as_deref() {
        Some(rows) => f(rows),
        None => f(baked()),
    })
}

/// **Qual opção este estado vivo diz que está marcada** — a porta única da seleção.
///
/// ⚠️ Quatro variantes do catálogo guardam uma seleção e cada uma a chama de outra coisa
/// (`Tabs.selected`, `Radio.selected_index`, `Dropdown.selected_index`). Perguntar aqui, uma vez,
/// é o que impede o `paint` e o `event` de discordarem sobre qual delas ler — e o modo de falha
/// dessa discordância é o controle desenhar uma opção marcada e devolver outra ao ser clicado.
///
/// ⚠️ **`Option`, e o `None` é o que torna a porta ÚNICA em vez de duas.** Ele responde duas
/// perguntas de uma vez — *este controle É de escolha?* e *qual?* — e os dois consumidores querem
/// metades diferentes: o `paint` cai em `0` porque um controle de opções tem de desenhar ALGUMA
/// marcada, e o `event` precisa do `None` para saber que aquela row não emite uma escolha. Um
/// `usize` com zero de reserva daria ao `event` um índice inventado para toda row do painel.
#[must_use]
pub fn selected_of(live: Option<&InteractiveState>) -> Option<usize> {
    match live {
        Some(InteractiveState::Tabs { selected }) => Some(*selected),
        Some(InteractiveState::Radio { selected_index, .. }) => Some(*selected_index),
        Some(InteractiveState::Dropdown { selected_index, .. }) => {
            Some(selected_index.unwrap_or(0))
        }
        _ => None,
    }
}

/// **MARCA a opção `index`** — a irmã de ESCRITA do [`selected_of`], e a lista aberta FECHA.
///
/// ⚠️ **Duas portas para o mesmo campo, e é por isso que elas moram lado a lado:** a de leitura
/// diz ao `paint` qual desenhar, a de escrita diz ao `event` qual marcar. Se discordarem sobre
/// qual variante guarda o quê, o controle desenha uma opção e devolve outra ao ser clicado — e o
/// artista vê a aba certa acesa a fazer a coisa errada.
///
/// ⚠️ **Fechar é da lista ESCONDIDA, e só dela.** O despacho genérico alterna o `open` de quem foi
/// CLICADO, e quem foi clicado é a opção, não o chip — sem esta linha a lista fica aberta depois
/// da escolha e o clique seguinte, o que o artista dá para a fechar, escolhe outra coisa. Uma
/// faixa de abas não tem nada a fechar, e é por isso que o braço dela não menciona `open`.
///
/// Devolve `false` quando o estado não é de escolha — o chamador larga o gesto em vez de o
/// inventar.
pub fn select_in(live: &mut InteractiveState, index: usize) -> bool {
    match live {
        InteractiveState::Tabs { selected } => *selected = index,
        InteractiveState::Radio { selected_index, .. } => *selected_index = index,
        InteractiveState::Dropdown {
            open,
            selected_index,
            ..
        } => {
            *open = false;
            *selected_index = Some(index);
        }
        _ => return false,
    }
    true
}

/// **A que row e a que opção pertence este id** — a inversa do [`ids::authored_option_id`].
///
/// ⚠️ **Ela procura, em vez de decodificar.** O id é um HASH, então não há nada a inverter: a
/// resposta sai de re-derivar o id de cada opção de cada row de lista e comparar. É o mesmo
/// desenho do [`row_key_for`], e o custo é o mesmo que o do hit-test que já aconteceu — uma vez
/// por clique, nunca por quadro.
///
/// ⚠️ **A varredura é da família INTEIRA, e não só de quem esconde as opções.** A primeira versão
/// filtrava por `defers_a_popover` — a leitura era que os três em linha teriam os segmentos
/// registados pelo pintor do catálogo. **Medido: nenhum pintor da pele regista nada**, e nenhum
/// lugar deste repo escreve `InteractiveState::Tabs`, então a faixa desenhava as três abas, ficava
/// viva sob o rato como **uma** row, e clicar nela não mudava a marcada (report do Enio,
/// 2026-08-09). Quem cunha o id de cada opção é este painel, para as quatro.
#[must_use]
pub fn option_for(id: NodeId) -> Option<(String, usize)> {
    with_rows(|rows| {
        rows.iter()
            .filter(|r| r.kind.takes_options())
            .find_map(|r| {
                (0..r.options.len())
                    .find(|i| ids::authored_option_id(&r.key, *i) == id)
                    .map(|i| (r.key.clone(), i))
            })
    })
}

/// A CHAVE da row a que `id` pertence, se alguma.
///
/// ⚠️ Devolve a chave e não a `Row` porque uma row viva vive sob o lock: deixá-la escapar não
/// compila, e é o único consumidor ([`crate::event`]) só precisa da chave para montar o intent.
#[must_use]
pub fn row_key_for(id: NodeId) -> Option<String> {
    with_rows(|rows| rows.iter().find(|r| r.id == id).map(|r| r.key.clone()))
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
    with_rows(|all| {
        all.iter()
            .enumerate()
            .filter(|(i, r)| all[..*i].iter().any(|p| p.key == r.key))
            .count()
    })
}

#[cfg(test)]
#[path = "rows_tests.rs"]
mod tests;
