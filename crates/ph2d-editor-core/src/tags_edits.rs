//! ⭐⭐⭐ **O VOCABULÁRIO DAS TAGS** (TOP-20 #9) — os dois gestos e o instantâneo do painel, num
//! módulo abaixo do [`crate::action_bus`] e do [`crate::screens`].
//!
//! # ⛔⛔ Porque ele existe, com o número atrás
//!
//! A catraca do DAG (`architecture_the_foundation_modules_form_a_dag`) tolera a aresta
//! `action_bus → screens` num **tecto** e escreve a cura ao lado dela: *«os PAYLOADS do Inspector
//! moram em `screens::hero::inspector_model*`; cura: descem para um módulo de vocabulário abaixo do
//! `action_bus`»*. A aresta estava **no tecto** quando a secção *Tags* nasceu, e o
//! [`TagsFieldEdit`] fê-la passar — em silêncio, porque o portão daquela wave correu o painel e a
//! shell, e o gate vive no `ph2d-editor-core`.
//!
//! ⇒ *a catraca só encolhe*, logo a cura não foi subir o número: foi **começar a migração que ela
//! pede**. Este módulo é o primeiro degrau dela, e as vinte irmãs descem por aqui quando alguém
//! pagar o resto.
//!
//! # ⚠️ O que fica aqui e o que fica no `screens::hero`
//!
//! Aqui: a **taxonomia** — os dois gestos (*marcar um objecto* · *mudar a árvore*) e o instantâneo
//! do painel que a mostra. As duas metades do gesto precisam de se ler uma ao lado da outra, e o
//! que as separa — o sujeito ser a entidade ou a árvore — é exactamente a distinção que os
//! doc-comments abaixo carregam.
//!
//! No `screens::hero`: o `InspectorTagRow` e o `InspectorTagsInfo`, que são o modelo de uma
//! **secção do Inspector** — construídos pelo instantâneo dele, ao lado das vinte irmãs.

/// **Uma edição da secção TAGS.**
///
/// ⚠️ **`Create` carrega TEXTO e as outras um id**, e a distinção é a que faz a caixa de escolha ter
/// uma porta só: escrever um nome que não existe e carregar em *Create “…”* cria a tag **e** marca
/// o objecto, num gesto; escolher uma da lista só marca.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagsFieldEdit {
    /// Marca o objecto com esta tag (id da árvore). Uma tag que já lá está é no-op.
    Add(u64),
    /// Tira esta tag do objecto. ⛔ **Não apaga a tag da árvore** — o painel *Tags* (W4) é quem o faz.
    Remove(u64),
    /// Cria a tag com este caminho (ou devolve a que já existe, dobrada) **e** marca o objecto.
    Create(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// W4 — o PAINEL *Tags*: a taxonomia inteira, e os gestos que a mudam
// ─────────────────────────────────────────────────────────────────────────────

/// Uma linha do painel *Tags* — a tag, o que ela vale e o que apagá-la leva.
///
/// ⚠️ **Irmã e não a mesma do [`crate::screens::hero::InspectorTagRow`]**, e a diferença é o que cada superfície
/// pergunta: o Inspector pergunta *«que tags este objecto tem»* e desenha chips; o painel pergunta
/// *«que tags o projecto tem, e quantos objectos cada uma vale»*. Fundi-las poria a contagem — que
/// é `O(mundo)` — dentro do instantâneo que o Inspector reconstrói a cada quadro.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagsPanelRow {
    /// A identidade durável (`ph2d_tags::TagId`), em `u64`.
    pub id: u64,
    /// O último nível (`"Flying"`) — o que a linha mostra, com o recuo a dizer o resto.
    pub label: String,
    /// `0` = raiz. O recuo da linha.
    pub depth: usize,
    /// ⭐ **Quantos OBJECTOS pertencem** (com a subárvore) — a coluna da direita.
    pub members: usize,
    /// ⭐⭐ **Quantas TAGS o apagar leva** (ela e as descendentes; nunca `0`).
    ///
    /// ⚠️ Ela existe para o botão *Delete* poder dizer o que faz **antes** de ser carregado: apagar
    /// `Enemy` leva `Flying` e `Boss` junto, e uma linha que diz só *«Delete»* esconde isso.
    pub subtree: usize,
}

/// **O instantâneo do painel *Tags*** — a árvore inteira e a última recusa.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TagsPanelInfo {
    /// Pela ordem da árvore (dobrada, o pai antes dos filhos) — a MESMA ordem dos chips e da caixa
    /// de escolha do Inspector. *Uma ordem para tags em todo o app.*
    pub rows: Vec<TagsPanelRow>,
    /// ⭐⭐ **A recusa do último gesto, NA LINHA em que ela aconteceu** — `(tag, frase)`.
    ///
    /// ⚠️ **A frase vem de [`ph2d_tags::TagError::message`]**, que vive ao lado da lei: o painel
    /// pinta um texto que não interpreta. ⛔ Traduzi-la aqui faria a segunda superfície que
    /// mostrasse a mesma recusa escrever outra frase.
    ///
    /// ⚠️ `tag == 0` quando a recusa não é de nenhuma linha (criar uma raiz com nome vazio).
    pub problem: Option<(u64, String)>,
}

/// **Um gesto sobre a ÁRVORE do projecto** — o que o painel *Tags* manda à shell.
///
/// ⛔⛔ **Nenhum deles é uma edição de OBJECTO**, e é isso que os separa da [`TagsFieldEdit`]: ali o
/// sujeito é a entidade escolhida e a árvore é efeito colateral de *Create*; aqui o sujeito é a
/// árvore, e o que acontece à pertença dos objectos é **consequência** — apagar leva-a, renomear e
/// mover não lhe tocam.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagTreeEdit {
    /// Cria uma tag com um nome de omissão, filha de `parent` (`None` = raiz), e **põe a linha em
    /// modo de renomear** — o fluxo do Blender: o gesto é um clique e o nome escreve-se por cima.
    Create { parent: Option<u64> },
    /// Renomeia (só o ÚLTIMO nível — o separador é recusado, senão *mover* viveria escondido aqui).
    Rename { id: u64, label: String },
    /// Move para debaixo de `parent` (`None` = raiz), com a subárvore. Um ciclo é recusado.
    Move { id: u64, parent: Option<u64> },
    /// Apaga a tag, a subárvore dela **e** a pertença dos objectos — num passo de undo só.
    Delete { id: u64 },
    /// Escolhe na cena todos os objectos que pertencem a esta tag (com a subárvore).
    ///
    /// ⚠️ **Não escreve no documento** — é um gesto de EDITOR, como o `Preview` do áudio e o da
    /// câmera. Viaja por aqui porque é aqui que o painel fala com a shell, e a selecção é dela.
    SelectTagged { id: u64 },
}
