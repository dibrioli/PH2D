//! **O CANAL DE PARÂMETRO POR-TIPO** — o que um tipo pede e que o retângulo, o rótulo, os tokens
//! e o estado vivo não determinam.
//!
//! ⚠️ **Corte por ASSUNTO, e as duas metades falham por motivos diferentes:** aqui mora *o que um
//! tipo PEDE* (a forma do canal e a lei de cada campo); no pai, *quem DESENHA* (o `match` que
//! termina no pintor real do catálogo). Uma cresce com o número de campos do canal, a outra com o
//! catálogo do design system — e foi a segunda que empurrou este arquivo contra o teto de LOC.

use super::Tabs;
use crate::widget::IconGlyph;

/// **O parâmetro que o TIPO pede e que o retângulo, o rótulo, os tokens e o estado vivo não
/// determinam.**
///
/// # Por que ele existe, e a cerca que o previu
///
/// O doc deste módulo declarava um mapeamento por-tipo como *"uma tabela de 44 casos especiais,
/// deliberadamente NÃO construída"*, e dizia quando ele nasceria: **no dia em que um tipo precisar
/// de um parâmetro que o token não exprime**. Medido, esse dia chegou e o número era outro: de
/// dezesseis tipos, **dois** — a `rgba` de uma `ColorSwatch` (o valor que ela existe para mostrar)
/// e o ícone de um `IconButton` (*qual* ícone?). Todo o resto continua determinado.
///
/// # A FORMA é side-metadata, e isso não é estilo
///
/// Um `Copy` struct de campos **opcionais com default neutro**, ao lado do `kind` — o molde do
/// `KernelResolver` dos Motion Nodes. Um canal novo é um **campo** novo, nunca um argumento novo
/// nem uma variante de contrato: assim o dia em que o décimo-sétimo tipo pedir o seu parâmetro
/// custa uma linha, e nenhum dos quinze chamadores muda.
///
/// ⚠️ **`Default` é o caminho SUPORTADO, não um esquecimento:** ele é o que um build que não
/// conhece o tipo produz, e o que a prévia sem documento tem em mãos. Cada braço decide o que
/// mostrar sem ele — e a decisão é sempre *o neutro que se LÊ como neutro*, nunca um valor
/// inventado que o artista tomaria por escolha sua.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SkinParam<'a> {
    /// A cor de uma [`WidgetKind::ColorSwatch`] — o **preenchimento da forma** que a veste.
    ///
    /// ⚠️ Ela não é um campo autorado novo: é o fill que o documento já carrega, lido pela
    /// `Paint::primary_color`, cujo próprio doc diz *"pra swatch de UI"*. Zero schema, zero
    /// controle a mais — o artista pinta o retângulo e a swatch é daquela cor.
    pub rgba: Option<[u8; 4]>,
    /// O glifo de um [`WidgetKind::IconButton`] — **a forma que o artista desenhou**, normalizada
    /// na caixa de 24×24 que o pintor de ícone espera.
    ///
    /// ⚠️ **O tipo é o [`IconGlyph`] do catálogo, e a escolha é deliberada.** Ele já tem
    /// exactamente as duas rotas que um ícone pode ter — *o desenho* (`Path`) e *a escolha do
    /// `IconId`* (`Builtin`) —, então a segunda rota não custa um campo novo nem uma variante
    /// nova: ela é o outro braço de um enum que existe para isto. Quem escolhe é o CONSTRUTOR; o
    /// pintor recebe um glifo e desenha-o.
    pub icon: Option<IconGlyph<'a>>,
    /// **Os rótulos das opções**, para a família de LISTA — vazio para todo o resto.
    ///
    /// ⚠️ **O terceiro campo do canal, e ele é o que prova que o molde não exige um parâmetro
    /// PEQUENO.** Foi supor isso que me fez escrever, no levantamento, que a família de lista
    /// *"não entra pelo mesmo canal"*: nada em *um campo com neutro* diz que o campo tem de caber
    /// numa palavra. O neutro é a fatia vazia, e sob ela um tipo de lista desenha a moldura sem
    /// opção nenhuma — que é o que um documento sem filhos de facto descreve.
    pub options: &'a [String],
    /// Qual opção está marcada. Fora do alcance ⇒ nenhuma, que é o que um documento vazio diz.
    ///
    /// ⚠️ **Esta frase era FALSA em metade da família, e a medição é o que a tornou verdadeira.**
    /// Os quatro braços chegavam a ela por construtores diferentes, e três comportamentos
    /// saíam de um campo só: com três opções e `selected = 7`, `Tabs` e `SegmentedAdaptive`
    /// pintavam **a ÚLTIMA** (o construtor `Tabs::selected` faz `idx.min(len - 1)`), enquanto
    /// `RadioGroup::select` e `Dropdown::select` **no-opam em silêncio** e não marcavam nenhuma.
    /// Hoje os quatro perguntam a [`marked_option`] e o texto acima descreve o que acontece.
    ///
    /// ⚠️ **Quem decidiu qual das duas leis vence foi o princípio que este `struct` já declara**
    /// — *o neutro que se LÊ como neutro, nunca um valor inventado que o artista tomaria por
    /// escolha sua*. Clampar produz exactamente esse valor inventado: a última opção fica acesa
    /// e nada na tela diz que ela é um substituto.
    ///
    /// ⚠️ **E isto NÃO contradiz a lei do painel, que CLAMPA** (`rows::clamp_selection_to`, na
    /// porta de reconciliação por-quadro). As duas respondem perguntas diferentes: o painel
    /// responde *"o artista apagou um filho; em que valor a seleção GUARDADA passa a viver?"* —
    /// e a resposta tem de manter o controle utilizável. A pele responde *"deram-me um índice
    /// que não nomeia nada; o que DESENHO?"* — e a resposta não pode inventar uma escolha.
    pub selected: usize,
}

/// **Qual opção esta pele marca** — `None` quando o índice não nomeia nenhuma.
///
/// A porta única da lei do [`SkinParam::selected`], perguntada pelos quatro braços da família de
/// LISTA. Antes dela cada braço herdava a política do construtor que por acaso usava, e os
/// construtores discordam.
///
/// ⚠️ **MEDIDO: ela é INERTE hoje, e o fato fica escrito em vez de gateado.** Substituí-la por
/// `Some(param.selected)` deixa os **29 gates da pele VERDES** — porque os quatro braços honram a
/// lei por acidentes downstream DIFERENTES: `paint_tabs` compara `i == selected` por item (um
/// índice além do fim não casa com nenhum) e `RadioGroup::select`/`Dropdown::select` no-opam num
/// valor que a lista não tem. O que de facto conserta o defeito é **não passar pelo construtor
/// `Tabs::selected`**, que clampa.
///
/// ⚠️ **Ela fica mesmo assim, e não é higiene:** sem ela a lei não está escrita em lugar nenhum —
/// ela vive espalhada por três mecanismos alheios, e o dia em que um deles ficar *tolerante* (um
/// `select` que clampa, um pintor que satura) quebra UM braço em silêncio. O gate
/// `an_index_past_the_end_marks_nothing_rather_than_inventing_a_choice` é a garantia executável;
/// esta função é onde um leitor descobre qual é a regra.
pub(super) fn marked_option(param: &SkinParam) -> Option<usize> {
    (param.selected < param.options.len()).then_some(param.selected)
}

/// Marca a aba escolhida — ou **nenhuma**, quando o índice está fora do alcance.
///
/// ⚠️ **Escreve o CAMPO em vez de chamar `Tabs::selected`, e a distinção é a wave inteira:** o
/// construtor clampa (`idx.min(len - 1)`), então por ele é *inexprimível* dizer "nenhuma". O
/// pintor, esse, compara `i == selected` por item — um índice além do fim simplesmente não casa
/// com nenhum, que é o desenho pedido. O sentinela é `items.len()`, o menor índice que a lista
/// não tem.
///
/// ⛔ **E o construtor NÃO pode ser corrigido no lugar dele:** ele tem 132 chamadas no app, e a
/// política de clamp é a certa para uma faixa de abas de painel, que sempre tem uma marcada.
pub(super) fn mark_selected_tab(tabs: &mut Tabs, param: &SkinParam) {
    tabs.selected = marked_option(param).unwrap_or(tabs.items.len());
}
