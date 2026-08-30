//! `ph2d-asset-index` — **a JUNÇÃO**, e o vocabulário dela.
//!
//! Este é o modelo que o navegador de assets lê ([plano 07](../../docs/Components/07_plano_do_navegador_de_assets.md),
//! wave A1). Ele é uma crate **folha**: sem ECS, sem `AssetDb`, sem UI, sem I/O — só tipos e a
//! consulta. Quem o **preenche** é o shell (wave A2), que é o único sítio que conhece as duas
//! fontes.
//!
//! # ⭐⭐ Por que um índice existe, se o app já sabe o que tem
//!
//! Porque **as duas fontes não se parecem** (plano 07 §2, medido 30/08):
//!
//! | Fonte | O que é | Onde vive | Identidade |
//! |---|---|---|---|
//! | **Componente** | uma sub-árvore MARCADA (`MasterRoot`) | no MUNDO (ECS) | `StableId` |
//! | **Textura** | bytes decodificados | no `AssetDb` | `AssetId` (blake3) |
//!
//! Um componente **não é um ficheiro** — ele é o *«Mark as Asset»* do Blender aplicado a uma
//! sub-árvore. Uma textura **é** conteúdo. Perguntar *«que assets existem?»* exige hoje duas
//! travessias diferentes e **nenhum sítio as junta**. ⇒ é isso que esta crate é, e é por isso que
//! ela **não** é uma cache de disco.
//!
//! ⚠️ **Consequência que decide a arquitectura:** ⛔ não há árvore de PASTAS. O `res://` do Godot e
//! o sistema de ficheiros do Blender não existem aqui; a taxonomia é de **catálogos**
//! ([`CatalogId`]), que é a escolha do Blender Asset Browser e já era a do ADR-0165.
//!
//! # A cerca desta crate
//!
//! ⛔ Ela **não decide o que existe** — ela guarda o que lhe entregam. Um componente apagado do
//! mundo continua aqui até quem constrói reconstruir; é por isso que a wave A2 reconstrói o índice
//! a partir da verdade em vez de o mutar por evento (*duas fontes de verdade sobre «o que existe»
//! é exactamente o defeito que a lente 1 da auditoria procura*).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// A que família um asset pertence. A ordem é a de apresentação (`SortBy::Kind`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssetKind {
    /// Uma sub-árvore marcada como receita (`MasterRoot`).
    Component,
    /// Pixels endereçados por conteúdo.
    Texture,
}

impl AssetKind {
    /// O rótulo que a UI mostra. ⚠️ Inglês — a UI deste app é inglesa (HR-15 / memória).
    ///
    /// ⭐⭐ **É «Prefab» e não «Component», por decisão do Enio (2026-08-30: *«não acha que
    /// componente é um nome ruim?»*), e o argumento é uma COLISÃO real dentro deste app:** o
    /// Inspector chama **componente** ao `Transform`, ao `Sprite`, ao `RigidBody` — é o modelo
    /// inteiro do [ADR-0166](../../docs/architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md)
    /// (*«components attach through one palette»*). Usar a mesma palavra para *«um objecto guardado
    /// na biblioteca»* punha duas coisas diferentes com o mesmo nome na mesma UI.
    ///
    /// ⚠️ **E «Prefab» não é uma escolha de gosto — é a palavra da FAMÍLIA que este app já fala.**
    /// O cartão do Inspector já diz *Instance of* e *Variant of*, que é o vocabulário da Unity
    /// (Prefab · Prefab Instance · Prefab Variant). ⛔ O Figma diz *Component* e pode: ele não tem
    /// componentes de ECS para colidir.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            AssetKind::Component => "Prefab",
            AssetKind::Texture => "Image",
        }
    }

    /// Todas as famílias, na ordem de apresentação. **É a fonte** — um filtro por família deriva
    /// daqui, nunca de uma lista escrita à mão ao lado (memória
    /// `feedback_a_hand_written_list_beside_a_predicate_is_two_answers`).
    pub const ALL: &'static [AssetKind] = &[AssetKind::Component, AssetKind::Texture];
}

/// **O endereço durável de um asset** — e note-se que ele é de tipos DIFERENTES por família.
///
/// ⚠️ Um `u64` para o componente (o `StableId`, que sobrevive ao respawn do undo por construção) e
/// 32 bytes para a textura (o blake3 do conteúdo). Colapsá-los num id só obrigaria um dos dois a
/// mentir sobre a própria identidade.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssetRef {
    /// O `StableId` da raiz da receita.
    Component { stable_id: u64 },
    /// O `AssetId` (blake3) dos pixels.
    Texture { asset: [u8; 32] },
}

impl AssetRef {
    /// A família a que este endereço pertence — **derivada da variante**, para que uma entrada não
    /// possa declarar uma família que o endereço contradiz.
    #[must_use]
    pub fn kind(&self) -> AssetKind {
        match self {
            AssetRef::Component { .. } => AssetKind::Component,
            AssetRef::Texture { .. } => AssetKind::Texture,
        }
    }
}

/// Identidade de um **catálogo** (a taxonomia do Blender: um UUID, e o nome é um rótulo mutável).
///
/// ⚠️ **Ele já existe no modelo, e ainda não tem UI** (wave A3). A razão de nascer agora é a do
/// plano 07 §6: acrescentá-lo depois mexeria no registo de toda entrada.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CatalogId(pub u128);

/// ⭐⭐ **A MINIATURA de um cartão** — RGBA8 **reto** (não pré-multiplicado), justo, `w * h * 4`.
///
/// ⚠️ **A igualdade é por IDENTIDADE do `Arc`, e é `O(1)` de propósito.** Um cartão redesenha-se a
/// cada quadro e o painel pergunta *«é a mesma imagem da anterior?»* para não reconstruir a textura
/// — comparar 37 KB de bytes por cartão, 512 cartões, por quadro, mediria o que a pergunta existe
/// para evitar. ⇒ quem produz uma miniatura NOVA produz um `Arc` novo; ⛔ **nunca mute um `Arc`
/// existente em sítio**, porque o `vello` indexa o atlas pelo id da `Blob` e serviria os pixels
/// velhos, sem erro nenhum.
///
/// ⚠️ **Ela é um tipo desta crate FOLHA, e não o `PreviewThumb` do painel do Motion**, embora os
/// três campos sejam os mesmos: esta crate não conhece painel nenhum, e a **redução** (a lei) tem
/// dono único em `shells/desktop/src/thumbnail.rs`. *Duas vocabulários sobre uma lei é isolamento;
/// duas leis seria o defeito.*
#[derive(Clone, Debug)]
pub struct Thumb {
    /// Os bytes, partilhados por contagem de referências.
    pub rgba: std::sync::Arc<Vec<u8>>,
    /// Largura em px.
    pub w: u32,
    /// Altura em px.
    pub h: u32,
}

impl PartialEq for Thumb {
    fn eq(&self, other: &Self) -> bool {
        self.w == other.w && self.h == other.h && std::sync::Arc::ptr_eq(&self.rgba, &other.rgba)
    }
}
impl Eq for Thumb {}

/// Uma entrada do índice — o que o navegador desenha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetEntry {
    /// O endereço durável.
    pub key: AssetRef,
    /// O nome que o artista vê e por que ele busca.
    pub name: String,
    /// Uma linha curta de detalhe (`"3 pieces"`, `"512x512"`). ⚠️ Nunca é o nome outra vez.
    pub detail: String,
    /// **A cor dominante**, em sRGB não pré-multiplicado.
    ///
    /// ⚠️ Ela está no modelo desde a A1 de propósito (plano 07 §6) e a UI da A4 usa-a como o
    /// corpo do cartão enquanto a miniatura verdadeira não existe (A6) — *um cartão com a cor da
    /// imagem é informação; um cartão cinzento é um lugar vazio.*
    pub swatch: [u8; 4],
    /// ⭐⭐ **A miniatura de verdade**, quando existe (wave A6). `None` = o cartão desenha-se com a
    /// [`Self::swatch`].
    ///
    /// ⚠️ **A ausência é informação, não um buraco.** Um **Prefab** sem peça nenhuma com pixels não
    /// tem imagem que o descreva, e uma imagem de 16 bits ainda não a tem; nos dois casos a cor
    /// dominante é o que existe, e desenhá-la é honesto. *Um quadrado cinzento é que seria um
    /// lugar vazio.*
    pub thumb: Option<Thumb>,
    /// O catálogo a que pertence (A3). `None` = *Unassigned*.
    pub catalog: Option<CatalogId>,
    /// Etiquetas livres (A3).
    pub tags: Vec<String>,
    /// **De que este asset depende** — as texturas de um componente, por exemplo.
    ///
    /// ⚠️ Guarda-se **só uma direcção**: [`AssetIndex::owners`] inverte-a. Guardar as duas seria
    /// duas respostas à mesma pergunta, e a que envelhece é sempre a que ninguém escreve.
    pub deps: Vec<AssetRef>,
    /// Ordem de descoberta — é o que `SortBy::Recent` ordena (maior primeiro).
    pub seq: u64,
}

impl AssetEntry {
    /// Uma entrada mínima. Os campos ricos preenchem-se por atribuição — este construtor existe
    /// para que acrescentar um campo **não** parta todos os sítios de construção.
    #[must_use]
    pub fn new(key: AssetRef, name: impl Into<String>) -> Self {
        Self {
            key,
            name: name.into(),
            detail: String::new(),
            swatch: [0x50, 0x50, 0x58, 0xFF],
            thumb: None,
            catalog: None,
            tags: Vec::new(),
            deps: Vec::new(),
            seq: 0,
        }
    }

    /// A família — **lida do endereço**, nunca de um campo próprio (ver [`AssetRef::kind`]).
    #[must_use]
    pub fn kind(&self) -> AssetKind {
        self.key.kind()
    }

    /// Este asset casa com o texto procurado?
    ///
    /// Sub-string sem distinguir maiúsculas, sobre nome **e** etiquetas — que é o que o Godot e o
    /// Blender fazem. ⚠️ Texto vazio casa com tudo (é *«sem filtro»*, não *«nada»*).
    #[must_use]
    pub fn matches(&self, needle_lower: &str) -> bool {
        if needle_lower.is_empty() {
            return true;
        }
        if self.name.to_lowercase().contains(needle_lower) {
            return true;
        }
        self.tags
            .iter()
            .any(|t| t.to_lowercase().contains(needle_lower))
    }
}

/// Como ordenar a grade. ⚠️ **Duas ordenações, por metade** (plano 07 D6): esta é a da GRADE.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SortBy {
    /// Alfabética. O default, e o do Godot.
    #[default]
    Name,
    /// Família primeiro, nome dentro dela.
    Kind,
    /// O descoberto por último em primeiro.
    Recent,
}

impl SortBy {
    /// O rótulo do chip.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            SortBy::Name => "Name",
            SortBy::Kind => "Type",
            SortBy::Recent => "Recent",
        }
    }

    /// **A fonte** da fileira de chips de ordenação.
    pub const ALL: &'static [SortBy] = &[SortBy::Name, SortBy::Kind, SortBy::Recent];
}

/// O que o painel está a pedir ao índice.
#[derive(Clone, Debug, Default)]
pub struct Query {
    /// O texto da busca da GRADE (plano 07 D1).
    pub text: String,
    /// Restringir a uma família. `None` = todas.
    pub kind: Option<AssetKind>,
    /// Restringir a um catálogo (A3). `None` = todos.
    pub catalog: Option<CatalogId>,
    /// A ordem.
    pub sort: SortBy,
}

/// O índice — a junção das duas fontes, reconstruída pelo shell.
#[derive(Clone, Debug, Default)]
pub struct AssetIndex {
    entries: Vec<AssetEntry>,
    next_seq: u64,
}

impl AssetIndex {
    /// Um índice vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Acrescenta uma entrada, carimbando a ordem de descoberta.
    ///
    /// ⚠️ **Idempotente por ENDEREÇO:** inserir duas vezes o mesmo `key` substitui a entrada e
    /// **mantém o `seq` original**. Sem isto, o mesmo asset visto por dois caminhos (uma textura
    /// usada por três sprites) apareceria três vezes na grade — que é o defeito que a lente 1
    /// procura.
    pub fn push(&mut self, mut entry: AssetEntry) {
        if let Some(slot) = self.entries.iter_mut().find(|e| e.key == entry.key) {
            entry.seq = slot.seq;
            *slot = entry;
            return;
        }
        entry.seq = self.next_seq;
        self.next_seq += 1;
        self.entries.push(entry);
    }

    /// Quantas entradas existem, sem filtro nenhum.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// O índice está vazio?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Todas as entradas, na ordem de descoberta.
    #[must_use]
    pub fn entries(&self) -> &[AssetEntry] {
        &self.entries
    }

    /// A entrada com este endereço.
    #[must_use]
    pub fn get(&self, key: &AssetRef) -> Option<&AssetEntry> {
        self.entries.iter().find(|e| &e.key == key)
    }

    /// Quantas entradas há por família — o que a linha de resumo do painel diz.
    #[must_use]
    pub fn counts(&self) -> BTreeMap<AssetKind, usize> {
        let mut out = BTreeMap::new();
        for e in &self.entries {
            *out.entry(e.kind()).or_insert(0) += 1;
        }
        out
    }

    /// A consulta: filtrar e ordenar.
    ///
    /// ⚠️ **A ordem é TOTAL em todos os modos** — o desempate final é sempre o `seq`, que é único.
    /// Uma grade cuja ordem empata muda de posição entre quadros, e o cartão debaixo do dedo deixa
    /// de ser o que o artista mirou.
    #[must_use]
    pub fn query(&self, q: &Query) -> Vec<&AssetEntry> {
        let needle = q.text.to_lowercase();
        let mut hits: Vec<&AssetEntry> = self
            .entries
            .iter()
            .filter(|e| q.kind.is_none_or(|k| e.kind() == k))
            .filter(|e| q.catalog.is_none_or(|c| e.catalog == Some(c)))
            .filter(|e| e.matches(&needle))
            .collect();
        match q.sort {
            SortBy::Name => hits.sort_by(|a, b| {
                a.name
                    .to_lowercase()
                    .cmp(&b.name.to_lowercase())
                    .then(a.seq.cmp(&b.seq))
            }),
            SortBy::Kind => hits.sort_by(|a, b| {
                a.kind()
                    .cmp(&b.kind())
                    .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                    .then(a.seq.cmp(&b.seq))
            }),
            SortBy::Recent => hits.sort_by_key(|e| std::cmp::Reverse(e.seq)),
        }
        hits
    }

    /// **De que `key` depende** (plano 07 D9, um dos dois sentidos).
    #[must_use]
    pub fn deps(&self, key: &AssetRef) -> Vec<&AssetEntry> {
        let Some(entry) = self.get(key) else {
            return Vec::new();
        };
        entry.deps.iter().filter_map(|d| self.get(d)).collect()
    }

    /// **Quem depende de `key`** — o outro sentido, DERIVADO por inversão.
    ///
    /// ⚠️ É esta a metade que o Godot chama *Owners* e que o Blender não tem. Ela é a que
    /// responde *«posso apagar isto?»*.
    #[must_use]
    pub fn owners(&self, key: &AssetRef) -> Vec<&AssetEntry> {
        self.entries
            .iter()
            .filter(|e| e.deps.contains(key))
            .collect()
    }
}
