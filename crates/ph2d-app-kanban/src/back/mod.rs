//! ⭐ **O verso do cartão** (W3, o modal do Trello): título, lista, seguir, capa, membros, etiquetas,
//! datas, campos, descrição rica, checklists, anexos, localização, atividade e a coluna «Add to
//! card»/«Actions».
//!
//! O estado do verso é da VISTA (vive na [`crate::tab::Tab`]); o que ele mostra lê-se do documento a
//! cada quadro, e o que escreve vai por [`write`] (lotes com atividade, um passo de desfazer cada).
//! O pintor regista os alvos do clique ([`Back::hits`]) — o clique seguinte lê-os, como a disposição
//! VISTA do quadro.

mod activity;
pub(crate) mod attach;
mod click;
pub(crate) mod dates;
mod keys;
mod lists;
pub(crate) mod outside;
pub(crate) mod paint;
mod pop;
mod probe;
pub(crate) mod scene;
mod side;
mod thumb;
pub(crate) mod ui;
pub(crate) mod write;

pub(crate) use click::{down, moved, up, wheel};
pub(crate) use keys::{compose, key};
pub(crate) use paint::paint;
pub(crate) use pop::card_link;

/// W6 — carrega no botão `s` da coluna do verso aberto (a edição rápida e os atalhos `D` `L` `M`).
pub(crate) fn press_side(docs: &mut crate::docs::KanbanDocs, id: u64, s: Side) {
    if is_open(docs, id) {
        click::click(docs, id, BackHit::Side(s));
    }
}

use ph2d_editor_core::widget::{LineField, RichEditor, RichEditorLabels};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{
    AttachmentId, CardId, CheckItemId, ChecklistId, CommentId, FieldId, LabelColor, LabelId,
    ListId, Member,
};

/// O que o rato acerta no verso.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BackHit {
    /// Fora do modal: fecha.
    Scrim,
    /// Dentro do modal, sem alvo.
    Inside,
    Close,
    Title,
    ListLink,
    Watch,
    CoverButton,
    MemberAdd,
    LabelAdd,
    Dates,
    DueDone,
    FieldValue(FieldId),
    DescEdit,
    DescArea,
    /// O menu «Aa» aberto da descrição (pintado por cima do verso inteiro).
    DescMenu,
    DescSave,
    DescCancel,
    ClTitle(ChecklistId),
    ClHide(ChecklistId),
    ClDelete(ChecklistId),
    ItemBox(CheckItemId),
    ItemText(CheckItemId),
    ItemMember(CheckItemId),
    ItemDue(CheckItemId),
    ItemConvert(CheckItemId),
    ItemDelete(CheckItemId),
    AddItem(ChecklistId),
    /// O campo de uma linha aberto no lugar, e os botões dele.
    EditField,
    EditSave,
    EditCancel,
    AttOpen(AttachmentId),
    AttCover(AttachmentId),
    AttDelete(AttachmentId),
    AttAdd,
    Location,
    Details,
    CommentArea,
    /// O menu «Aa» aberto do comentário.
    CommentMenu,
    CommentSave,
    CommentCancel,
    CommentEdit(CommentId),
    CommentDelete(CommentId),
    /// Alternar a reacção `emoji` (índice em [`REACTIONS`]) num comentário.
    React(CommentId, u8),
    Mention(u8),
    Side(Side),
    Pop(PopHit),
    /// J2 — um botão de cartão do Butler (a secção «Buttons»).
    CardButton(ph2d_kanban_model::AutomationId),
    /// J2 — uma ligação ao projecto: abrir, ou tirá-la.
    LinkOpen(ph2d_kanban_model::ProjectLinkId),
    LinkRemove(ph2d_kanban_model::ProjectLinkId),
    /// J2 — o painel de automação (o Butler), que pinta e clica pelas mesmas peças.
    Auto(crate::auto::AutoHit),
}

/// Os botões da coluna lateral.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Members,
    Labels,
    Checklist,
    Dates,
    Attachment,
    Cover,
    Fields,
    Move,
    Copy,
    /// W6 — «Mirror» (o espelho noutra lista ou Kanban).
    Mirror,
    /// «Make template» (liga/desliga o modelo).
    Template,
    /// W6 — num modelo: «Create card from template».
    FromTemplate,
    Archive,
    Delete,
    Share,
    Watch,
    Vote,
    /// J2 — «Scene object»: ligar o cartão a um objecto/script/macro/trecho/Quadro do projecto.
    SceneObject,
    /// J2 — «Add button»: o painel de automação, num botão de cartão novo.
    AddButton,
    /// J2 — «Convert to sticky note» (com o Kanban num Quadro).
    ToNote,
}

/// O que se acerta dentro de um popover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PopHit {
    Inside,
    Close,
    Member(u16),
    List(ListId),
    Label(LabelId),
    LabelEdit(LabelId),
    LabelCreate,
    /// Uma das 30 cores (índice em [`LabelColor::all`]).
    Color(u8),
    NoColor,
    Save,
    Remove,
    /// A caixa de texto do popover.
    Input,
    Input2,
    /// O seletor de datas (o widget decide o resto).
    Picker,
    CoverHue(u8),
    CoverImage(AttachmentId),
    CoverSize(bool),
    FieldKind(u8),
    FieldFront(FieldId),
    FieldDelete(FieldId),
    /// «Choose a file»: o diálogo nativo do sistema.
    PickFile,
    /// «Copy link» do «Share».
    CopyLink,
    /// J2 — a linha `n` do catálogo do projecto no popover «Scene object».
    Scene(u16),
}

/// As reacções que se oferecem (o Trello abre o seletor inteiro; estas são as mais usadas).
pub(crate) const REACTIONS: [&str; 5] = [
    "\u{1F44D}",
    "\u{2764}\u{FE0F}",
    "\u{1F389}",
    "\u{1F604}",
    "\u{1F440}",
];

/// Um campo de uma linha aberto no lugar.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum EditAt {
    Title,
    Location,
    ClTitle(ChecklistId),
    Item(CheckItemId),
    NewItem(ChecklistId),
    Field(FieldId),
}

#[derive(Clone, Debug)]
pub(crate) struct Edit {
    pub at: EditAt,
    pub field: LineField,
}

/// Um popover aberto, ancorado num alvo do verso.
#[derive(Debug)]
pub(crate) enum Pop {
    Move,
    /// Os membros do cartão (`None`) ou o responsável de um item.
    Members(Option<CheckItemId>),
    Labels,
    /// Criar (`None`) ou editar uma etiqueta: o nome e a cor escolhida.
    Label(Option<LabelId>, LineField, Option<LabelColor>),
    Checklist(LineField),
    /// As datas do cartão (`None`) ou de um item.
    Dates(
        Option<CheckItemId>,
        Box<ph2d_kanban_views::datepick::Picker>,
    ),
    /// Anexar um link: o endereço e o nome.
    Attach(LineField, LineField),
    Cover,
    /// Os campos personalizados: o nome do novo, o tipo e as opções (lista suspensa).
    Fields(LineField, u8, LineField),
    Delete,
    Share,
    /// J2 — «Scene object»: a pesquisa no catálogo do projecto.
    Scene(LineField),
}

impl Pop {
    /// O alvo onde o popover se ancora.
    pub(crate) fn anchor(&self) -> BackHit {
        match self {
            Pop::Move => BackHit::Side(Side::Move),
            Pop::Members(Some(i)) => BackHit::ItemMember(*i),
            Pop::Members(None) => BackHit::Side(Side::Members),
            Pop::Labels | Pop::Label(..) => BackHit::Side(Side::Labels),
            Pop::Checklist(_) => BackHit::Side(Side::Checklist),
            Pop::Dates(Some(i), ..) => BackHit::ItemDue(*i),
            Pop::Dates(None, ..) => BackHit::Side(Side::Dates),
            Pop::Attach(..) => BackHit::Side(Side::Attachment),
            Pop::Cover => BackHit::Side(Side::Cover),
            Pop::Fields(..) => BackHit::Side(Side::Fields),
            Pop::Delete => BackHit::Side(Side::Delete),
            Pop::Share => BackHit::Side(Side::Share),
            Pop::Scene(_) => BackHit::Side(Side::SceneObject),
        }
    }
}

/// O verso aberto de um cartão.
#[derive(Debug)]
pub(crate) struct Back {
    pub card: CardId,
    /// A rolagem do verso (px desde o topo).
    pub scroll: f32,
    /// Os alvos que o último quadro pintou, pela ordem (o último por cima).
    pub hits: Vec<(Rect, BackHit)>,
    /// O modal e a altura do conteúdo no último quadro.
    pub modal: Rect,
    pub content_h: f32,
    pub area: Rect,
    pub hover: Option<BackHit>,
    pub press: Option<BackHit>,
    pub edit: Option<Edit>,
    pub pop: Option<Pop>,
    /// A 2.ª caixa de texto do popover tem o teclado (anexar: o nome; campos: as opções).
    pub pop_focus: bool,
    /// A descrição: o editor e o markdown que ele carregou (outro markdown no documento recarrega-o
    /// fora da edição — o desfazer, outra janela).
    pub desc: RichEditor,
    pub desc_md: String,
    /// A caixa de comentário, e o comentário que ela edita (`None` = um novo).
    pub comment: RichEditor,
    pub comment_edit: Option<CommentId>,
    /// A sugestão do `@` realçada.
    pub mention_pick: usize,
    /// Os corpos dos comentários pintados (o markdown de cada um e o editor só de leitura).
    pub bodies: std::collections::BTreeMap<CommentId, (String, RichEditor)>,
    /// O histórico com os detalhes (o «Show details» do Trello).
    pub details: bool,
    /// O que o artista pediu para abrir fora do PH2D (a shell serve-o: [`outside`]).
    pub opened: Vec<outside::Outside>,
    /// O diálogo «Choose a file» pedido (a shell abre-o: [`outside`]).
    pub pick: bool,
    /// O texto que um botão copiou (a shell entrega-o à área de transferência).
    pub copy: Option<String>,
    /// O último carregar num editor rico: o alvo, quando (ns) e quantos cliques seguidos (o duplo
    /// e o triplo clique contam-se aqui, à janela do sistema `DOUBLE_CLICK_WINDOW_NS`).
    pub last_down: Option<(BackHit, u128, u8)>,
    /// O que o último quadro pintou na atividade (as sondas leem-no).
    pub feed: Vec<String>,
    /// J2 — o catálogo do projecto (o que existe agora e o nome de cada coisa), seguido a cada quadro
    /// ([`scene::follow`]); as ligações do cartão resolvem-se contra ele.
    pub catalog: ph2d_kanban_links::Catalog,
}

pub(crate) fn rich_labels(placeholder: &str) -> RichEditorLabels {
    use ph2d_i18n::tr;
    RichEditorLabels {
        placeholder: tr(placeholder).to_owned(),
        tools: [
            "kanban.back.tool.style",
            "kanban.back.tool.bold",
            "kanban.back.tool.italic",
            "kanban.back.tool.strike",
            "kanban.back.tool.code",
            "kanban.back.tool.bullet",
            "kanban.back.tool.numbered",
            "kanban.back.tool.check",
            "kanban.back.tool.quote",
            "kanban.back.tool.code_block",
            "kanban.back.tool.link",
            "kanban.back.tool.rule",
        ]
        .map(|k| tr(k).to_owned()),
        styles: [
            "kanban.back.style.normal",
            "kanban.back.style.h1",
            "kanban.back.style.h2",
            "kanban.back.style.h3",
        ]
        .map(|k| tr(k).to_owned()),
    }
}

/// O id a11y de um widget do verso (derivado do cartão; o sal separa-o do resto da aba).
pub(crate) fn node(card: CardId, salt: u64) -> ph2d_a11y::NodeId {
    ph2d_a11y::NodeId(crate::A11Y_SALT ^ 0x0000_BAC0_0000_0000 ^ (card.0 << 8) ^ salt)
}

impl Back {
    pub(crate) fn new(card: CardId, description: &str) -> Self {
        Self {
            card,
            scroll: 0.0,
            hits: Vec::new(),
            modal: Rect::new(0.0, 0.0, 0.0, 0.0),
            content_h: 0.0,
            area: Rect::new(0.0, 0.0, 0.0, 0.0),
            hover: None,
            press: None,
            edit: None,
            pop: None,
            pop_focus: false,
            desc: RichEditor::new(
                node(card, 1),
                ph2d_kanban_md::parse(description),
                rich_labels("kanban.back.desc_placeholder"),
            ),
            desc_md: description.to_owned(),
            comment: RichEditor::new(
                node(card, 2),
                ph2d_richdoc::RichDoc::new(),
                rich_labels("kanban.back.comment_placeholder"),
            ),
            comment_edit: None,
            mention_pick: 0,
            bodies: std::collections::BTreeMap::new(),
            details: false,
            opened: Vec::new(),
            pick: false,
            copy: None,
            last_down: None,
            feed: Vec::new(),
            catalog: ph2d_kanban_links::Catalog::default(),
        }
    }

    /// O alvo do último quadro em `(x, y)` — o de cima.
    /// O alvo sob `(x, y)`; o menu «Aa» de um editor responde como o editor (o editor reconhece a
    /// opção pela geometria dele).
    pub(crate) fn hit(&self, x: f32, y: f32) -> Option<BackHit> {
        self.hits
            .iter()
            .rev()
            .find(|(r, _)| r.contains(x, y))
            .map(|(_, h)| match h {
                BackHit::DescMenu => BackHit::DescArea,
                BackHit::CommentMenu => BackHit::CommentArea,
                h => *h,
            })
    }

    /// O rect pintado de um alvo.
    pub(crate) fn rect_of(&self, h: BackHit) -> Option<Rect> {
        self.hits
            .iter()
            .rev()
            .find(|(_, x)| *x == h)
            .map(|(r, _)| *r)
    }

    /// Algum texto tem o teclado (o campo, a descrição, o comentário, ou uma caixa de um popover)?
    pub(crate) fn typing(&self) -> bool {
        self.edit.is_some()
            || self.desc.editing
            || self.comment.editing
            || matches!(
                self.pop,
                Some(
                    Pop::Label(..)
                        | Pop::Checklist(_)
                        | Pop::Attach(..)
                        | Pop::Fields(..)
                        | Pop::Scene(_)
                )
            )
            || matches!(&self.pop, Some(Pop::Dates(_, p)) if p.widget.editing.is_some())
    }
}

/// A altura de uma linha de controlo do verso (botões, chips, campos): a dos botões do quadro (`32`,
/// a da maquete).
pub(crate) const ROW: f32 = crate::metrics::BUTTON_H;

/// Quem se pode pôr num cartão e chamar com `@`: «eu», as equipas dos quadros do projecto e quem
/// já está nalgum cartão deste Kanban — sem repetir, por ordem de nome («eu» primeiro).
pub(crate) fn people(k: &ph2d_kanban_model::Kanban, team: &[String], me: &Member) -> Vec<Member> {
    let mut v: Vec<Member> = team
        .iter()
        .map(|n| Member(n.clone()))
        .chain(
            k.cards
                .values()
                .filter(|c| !c.meta.deleted)
                .flat_map(|c| c.members.iter().cloned()),
        )
        .chain(k.comments.values().map(|c| c.author.clone()))
        .filter(|m| m != me && !m.0.trim().is_empty())
        .collect();
    v.sort_by_key(|m| m.0.to_lowercase());
    v.dedup();
    v.insert(0, me.clone());
    v
}

/// O verso de um cartão (ou o painel de automação, que entra pelas mesmas portas) está aberto na aba:
/// ele toma o rato e o teclado.
pub(crate) fn is_open(docs: &mut crate::docs::KanbanDocs, id: u64) -> bool {
    docs.tab_mut(id)
        .is_some_and(|t| t.back.is_some() || t.auto.is_some())
}

/// Abre o verso do cartão `card` na aba `id` (o clique sem arrasto e o `Enter`).
pub(crate) fn open(docs: &mut crate::docs::KanbanDocs, id: u64, card: CardId) {
    let desc = docs
        .kanban_of(id)
        .and_then(|kid| {
            docs.set
                .borrow()
                .get(kid)
                .and_then(|k| k.cards.get(&card).map(|c| c.description.clone()))
        })
        .unwrap_or_default();
    if let Some(t) = docs.tab_mut(id) {
        t.focus = Some(card);
        t.field = None;
        t.menu = None;
        t.w6.over = None;
        t.w6.quick = None;
        if !matches!(t.w6.input, Some((crate::w6::Boxed::Title, _))) {
            t.w6.input = None;
        }
        t.back = Some(Box::new(Back::new(card, &desc)));
    }
}

/// Fecha o verso (o que está escrito num campo aberto grava-se, como no Trello).
pub(crate) fn close(docs: &mut crate::docs::KanbanDocs, id: u64) {
    click::commit_edit(docs, id, false);
    keys::save_description(docs, id);
    if let Some(t) = docs.tab_mut(id) {
        t.back = None;
    }
}
