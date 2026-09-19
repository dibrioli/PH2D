//! **`ph2d-tags` — a ÁRVORE de tags do projecto** (TOP-20 #9, `docs/Components/08_plano_tags.md`).
//!
//! # ⭐ O modelo é o do Blender (decisão do dono D1, 2026-09-13)
//!
//! *«Faça como o Blender»*. Medido nesta máquina (§1.1 do plano): no Blender a pertença a uma coleção
//! é uma **referência** (renomear a coleção não tira ninguém dela) e a hierarquia **contém** (o objecto
//! de uma coleção-filha aparece no `all_objects` do pai). O *Asset Browser* do mesmo Blender guarda a
//! hierarquia num **caminho** e a identidade num **UUID** — e é o modelo que esta casa já tem no
//! `ph2d_asset_index::CatalogTree`. ⇒ uma tag é `{ id, path }`:
//!
//! - **a identidade é o [`TagId`]** — é ele que um objecto guarda, então renomear ou mover uma tag
//!   nunca toca num objecto;
//! - **a hierarquia é o caminho** (`Enemy/Flying/Boss`), e a árvore que a UI desenha é derivada dele.
//!
//! # ⚠️ Três divergências DECLARADAS do Blender
//!
//! 1. **Árvore, não grafo.** As coleções do Blender aceitam dois pais (medido) porque também são
//!    unidades de INSTÂNCIA; uma tag não é, e um grafo tornaria *«mover»* e o caminho mostrado ambíguos.
//! 2. **Sem gémeos.** O Blender distribui dois catálogos com o mesmo caminho e UUIDs diferentes no
//!    próprio ficheiro de dados (medido: 2 de 63). Aqui, criar uma tag que já existe devolve a que
//!    existe, e ler um documento com gémeos FUNDE-os ([`TagTree::restore`]).
//! 3. **Maiúsculas e acentos não contam** (decisão do dono D2) — o Blender distingue `Inimigo`,
//!    `inimigo` e `inímigo` (medido). A comparação é [`ph2d_label_fold::fold`]; o nome mostra-se como
//!    foi escrito primeiro.
//!
//! # ⭐⭐ A chave DOBRADA é guardada — e foi a MEDIÇÃO que a pediu
//!
//! A 1.ª versão desta árvore dobrava os caminhos a CADA comparação, e declarava aqui *«uma cache de
//! chaves dobradas seria estado derivado a manter coerente a cada renomear, e nada a pede ainda»*. A
//! sonda `measure_tag_tree_scale` (`ph2d-ecs`, plano §6.1-bis) pediu, com esta tabela (`--release`;
//! antes a load 2,57, depois a load 1,37):
//!
//! | tags | `restore`, antes → depois | `belongs` (pior caso), antes → depois |
//! |---:|---:|---:|
//! | 10 | 0,060 → 0,012 ms | 0,0025 → 0,00004 ms |
//! | 73 | 6,451 → 0,057 ms | 0,0180 → 0,00004 ms |
//! | 584 | **2 461** → 0,522 ms | 0,1374 → 0,00028 ms |
//! | 9 344 | (não acabava) → 9,242 ms | — → 0,00387 ms |
//!
//! ⇒ o `restore` era **pior que quadrático** (`8×` as tags, `107×` o tempo — cada tag lida procurava
//! gémeos dobrando a árvore inteira), e o `belongs`, que o filtro da física chama **a cada evento de
//! colisão**, era linear em chamadas à ICU.
//!
//! ⇒ **cada entrada guarda a sua chave** (`tree_order_key` pela dobra), escrita **só pelas portas que
//! mudam a árvore** (criar · renomear · mover · restaurar) e nunca exposta. ⚠️ *Um estado derivado cujo
//! ÚNICO escritor é o dono dele não tem como ficar incoerente* — a objecção da 1.ª redacção era a de
//! um índice guardado FORA do dono (o `rebuild_map` das pontes depois de cada restore), e isto não é
//! esse caso. Com a chave:
//!
//! - comparar dois caminhos é igualdade de `Vec<String>`, sem ICU;
//! - achar um caminho é uma BUSCA BINÁRIA (as entradas estão pela ordem da chave);
//! - a subárvore é um **intervalo contíguo** a começar na própria tag — um pai vem sempre
//!   imediatamente antes dos descendentes (gate `the_tree_order_puts_a_parent_immediately_before_its_children`);
//! - o `restore` indexa por chave num mapa LOCAL, `O(n log n)`.

use ph2d_label_fold::fold;
use ph2d_label_path as path;
use std::collections::{BTreeMap, BTreeSet};

/// A identidade durável de uma tag. ⚠️ `0` nunca é dado — é o que um documento sem árvore lê.
///
/// ⚠️ **`u64` e não o `u128` do `CatalogId`**: o `CatalogId` tem a largura de um UUID do Blender; um
/// `TagId` é guardado POR OBJECTO, num conjunto, e a pertença múltipla multiplica os bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TagId(pub u64);

/// Uma tag. ⚠️ O `path` é a HIERARQUIA e o nome MOSTRADO; o `id` é a IDENTIDADE.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    /// A identidade durável — o que a pertença de um objecto guarda.
    pub id: TagId,
    /// `"Enemy/Flying"`, com a grafia de quem o escreveu.
    pub path: String,
}

impl Tag {
    /// O rótulo que a linha mostra — o último nível.
    #[must_use]
    pub fn label(&self) -> &str {
        path::label(&self.path)
    }

    /// A profundidade (`0` = raiz).
    #[must_use]
    pub fn depth(&self) -> usize {
        path::depth(&self.path)
    }
}

/// **Porque um gesto sobre a árvore foi recusado** — o texto que o painel mostra NA LINHA.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagError {
    /// O nome, depois de aparado, ficou vazio.
    Empty,
    /// O nome tem o separador — seria um *mover* escondido dentro de *renomear*.
    HasSeparator,
    /// Já existe uma tag com este nome ali (comparada DOBRADA).
    Collision {
        /// A tag que já ocupa o sítio.
        existing: TagId,
    },
    /// Mover uma tag para dentro dela mesma ou de uma descendente — o ciclo que o Blender também
    /// recusa (medido: *«Collection 'Flying' already in collection 'Boss'»*).
    IntoOwnSubtree,
    /// O id não existe (apagado, ou de outro documento).
    Missing,
}

/// ⭐⭐ **As palavras de cada recusa vivem ao lado da LEI que a produz** (W4).
///
/// ⛔ **O painel não traduz um [`TagError`]**, e a razão é a lei do L-System (`TextRow.problem`):
/// duas superfícies a traduzir a mesma recusa escrevem duas frases, e elas divergem na primeira
/// vez que alguém mexe numa. Aqui o painel pinta uma `&str` que não interpreta.
///
/// ⚠️ **Em inglês e sem `ph2d_i18n::tr`, como as vizinhas**: esta crate é uma FOLHA sem
/// dependências de UI, e pô-la a depender do catálogo pelo texto de cinco frases inverteria a
/// pilha. O dia em que a segunda língua entrar, o que muda é quem chama isto, não a assinatura.
impl TagError {
    /// ⭐ **A CHAVE da frase que o painel mostra NA LINHA** — nunca num toast.
    ///
    /// ⚠️ **O doc acima previu esta cura e nomeou-a por escrito:** *«o dia em que a segunda língua
    /// entrar, o que muda é quem CHAMA isto, não a assinatura»*. Mudou quem chama — o
    /// `fase_tag_tree_commits` resolve a chave — e esta folha continua **sem dependência de UI**,
    /// que era a razão de as cinco frases estarem cruas.
    ///
    /// ⛔ Ela devolve a chave e **não** a palavra inglesa: um `message()` que fosse
    /// `tr_em(Ingles, …)` é indistinguível do caminho certo num processo em inglês, e um pintor que
    /// o chamasse por engano ficava preso à língua de omissão sem que teste nenhum o visse.
    #[must_use]
    pub fn message_key(self) -> &'static str {
        match self {
            Self::Empty => "tags.error.empty",
            Self::HasSeparator => "tags.error.has_separator",
            Self::Collision { .. } => "tags.error.collision",
            Self::IntoOwnSubtree => "tags.error.into_own_subtree",
            Self::Missing => "tags.error.missing",
        }
    }
}

/// A chave de uma tag: os níveis do caminho, cada um DOBRADO.
fn chave_de(p: &str) -> Vec<String> {
    path::tree_order_key(p, fold)
}

/// Uma tag com a chave dobrada do caminho. ⚠️ Privada, e as duas metades andam juntas: quem muda o
/// `path` muda a `chave` na mesma instrução.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entrada {
    tag: Tag,
    chave: Vec<String>,
}

/// A árvore de tags de um projecto.
///
/// ⚠️ **A `revision` fica FORA do `PartialEq`** — é chave de CACHE (a mesma lei do `CatalogTree`):
/// quem decide se duas árvores são a mesma são os BYTES que a família grava.
#[derive(Clone, Debug, Default, Eq)]
pub struct TagTree {
    /// Pela ordem da CHAVE — a ordem em que a UI as desenha, e a que faz de uma subárvore um
    /// intervalo. ⚠️ Sem gémeos: duas entradas nunca têm a mesma chave.
    entradas: Vec<Entrada>,
    /// De onde sai o próximo id. ⚠️ Monotónico e **nunca reutilizado**: um id reciclado faria os
    /// objectos de uma tag apagada reaparecerem dentro da seguinte.
    next_id: u64,
    /// Sobe a cada mutação — só chave de cache. ⚠️ Uma RECUSA não a move.
    revision: u64,
}

impl PartialEq for TagTree {
    fn eq(&self, other: &Self) -> bool {
        self.entradas == other.entradas && self.next_id == other.next_id
    }
}

impl TagTree {
    /// Uma árvore vazia.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entradas: Vec::new(),
            next_id: 1,
            revision: 0,
        }
    }

    fn touch(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    fn novo_id(&mut self) -> TagId {
        let id = TagId(self.next_id);
        self.next_id += 1;
        id
    }

    /// A revisão — chave de cache, nunca identidade.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// De onde sai o próximo id — gravado, nunca derivado.
    #[must_use]
    pub fn next_id(&self) -> u64 {
        self.next_id
    }

    /// As tags, pela ordem da árvore.
    pub fn tags(&self) -> impl ExactSizeIterator<Item = &Tag> + '_ {
        self.entradas.iter().map(|e| &e.tag)
    }

    /// Quantas tags.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entradas.len()
    }

    /// Está vazia?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entradas.is_empty()
    }

    fn indice(&self, id: TagId) -> Option<usize> {
        self.entradas.iter().position(|e| e.tag.id == id)
    }

    fn por_chave(&self, chave: &[String]) -> Option<usize> {
        self.entradas
            .binary_search_by(|e| e.chave.as_slice().cmp(chave))
            .ok()
    }

    /// Insere na posição da chave. ⚠️ Quem chama garante que a chave não existe (sem gémeos) e que
    /// ela é a do caminho — o `debug_assert` confere-o em toda corrida de teste.
    fn inserir(&mut self, tag: Tag, chave: Vec<String>) {
        debug_assert_eq!(
            chave,
            chave_de(&tag.path),
            "a chave tem de ser a do caminho"
        );
        let pos = self
            .entradas
            .binary_search_by(|e| e.chave.cmp(&chave))
            .unwrap_or_else(|p| p);
        self.entradas.insert(pos, Entrada { tag, chave });
    }

    /// A tag com este id.
    #[must_use]
    pub fn get(&self, id: TagId) -> Option<&Tag> {
        self.indice(id).map(|i| &self.entradas[i].tag)
    }

    /// ⭐ **A tag que ocupa este caminho**, comparado DOBRADO nível a nível.
    #[must_use]
    pub fn find(&self, path: &str) -> Option<TagId> {
        let p = path::normalise(path);
        if p.is_empty() {
            return None;
        }
        self.por_chave(&chave_de(&p))
            .map(|i| self.entradas[i].tag.id)
    }

    /// ⭐⭐ **Cria `path` e os ANCESTRAIS que faltarem** — ou devolve o id que já lá está (dobrado).
    ///
    /// ⚠️ **Um ancestral que já existe dá a GRAFIA dele ao filho novo**: `énemy/Flying` com `Enemy`
    /// na árvore cria `Enemy/Flying`, senão a mesma linha apareceria com duas grafias no painel.
    pub fn create(&mut self, path: &str) -> Result<TagId, TagError> {
        let p = path::normalise(path);
        if p.is_empty() {
            return Err(TagError::Empty);
        }
        let chave = chave_de(&p);
        if let Some(i) = self.por_chave(&chave) {
            return Ok(self.entradas[i].tag.id);
        }
        let mut resolvido = String::new();
        let mut ultimo = TagId(0);
        for (n, nivel) in p.split(path::SEP).enumerate() {
            let prefixo = &chave[..=n];
            if let Some(i) = self.por_chave(prefixo) {
                resolvido.clone_from(&self.entradas[i].tag.path);
                ultimo = self.entradas[i].tag.id;
                continue;
            }
            let candidato = if resolvido.is_empty() {
                nivel.to_string()
            } else {
                format!("{resolvido}{}{nivel}", path::SEP)
            };
            let id = self.novo_id();
            self.inserir(
                Tag {
                    id,
                    path: candidato.clone(),
                },
                prefixo.to_vec(),
            );
            self.touch();
            resolvido = candidato;
            ultimo = id;
        }
        Ok(ultimo)
    }

    /// ⭐⭐ **Renomear reescreve o PREFIXO** — os filhos vão junto, os ids ficam.
    ///
    /// ⚠️ Renomear para outra grafia DE SI MESMA é permitido (é assim que `enemy` passa a `Enemy`);
    /// para cima de uma IRMÃ (dobrada) é recusado.
    pub fn rename(&mut self, id: TagId, label: &str) -> Result<(), TagError> {
        let label = label.trim();
        if label.is_empty() {
            return Err(TagError::Empty);
        }
        if label.contains(path::SEP) {
            return Err(TagError::HasSeparator);
        }
        let Some(i) = self.indice(id) else {
            return Err(TagError::Missing);
        };
        let antigo = &self.entradas[i].tag.path;
        let novo = match path::parent(antigo) {
            Some(pai) => format!("{pai}{}{label}", path::SEP),
            None => label.to_string(),
        };
        if novo == *antigo {
            return Ok(());
        }
        let chave_nova = chave_de(&novo);
        if let Some(j) = self.por_chave(&chave_nova)
            && j != i
        {
            return Err(TagError::Collision {
                existing: self.entradas[j].tag.id,
            });
        }
        self.reescrever(i, &novo, &chave_nova);
        Ok(())
    }

    /// ⭐⭐ **Mover para dentro de `parent`** (`None` = para a raiz), com a subárvore.
    ///
    /// ⛔ Para dentro de si mesma ou de uma descendente é o ciclo — recusado, como no Blender (medido).
    pub fn move_under(&mut self, id: TagId, parent: Option<TagId>) -> Result<(), TagError> {
        let Some(i) = self.indice(id) else {
            return Err(TagError::Missing);
        };
        let pai = match parent {
            None => None,
            Some(pid) => match self.indice(pid) {
                Some(j) => Some(j),
                None => return Err(TagError::Missing),
            },
        };
        if let Some(j) = pai
            && path::key_is_self_or_descendant(&self.entradas[j].chave, &self.entradas[i].chave)
        {
            return Err(TagError::IntoOwnSubtree);
        }
        let rotulo = self.entradas[i].tag.label().to_string();
        let ultimo_nivel = self.entradas[i].chave.last().cloned().unwrap_or_default();
        let (novo, mut chave_nova) = match pai {
            Some(j) => (
                format!("{}{}{rotulo}", self.entradas[j].tag.path, path::SEP),
                self.entradas[j].chave.clone(),
            ),
            None => (rotulo, Vec::new()),
        };
        chave_nova.push(ultimo_nivel);
        if chave_nova == self.entradas[i].chave {
            return Ok(());
        }
        if let Some(k) = self.por_chave(&chave_nova)
            && k != i
        {
            return Err(TagError::Collision {
                existing: self.entradas[k].tag.id,
            });
        }
        self.reescrever(i, &novo, &chave_nova);
        Ok(())
    }

    /// Reescreve a subárvore da entrada `i` para o caminho `novo` (e a chave `chave_nova`) — a metade
    /// comum de renomear e mover, que só corre depois de o gesto ter sido aceite.
    ///
    /// ⚠️ **O caminho e a chave de cada descendente mudam na MESMA passagem**: os níveis abaixo guardam
    /// a grafia e a chave que já tinham, e só o prefixo troca.
    fn reescrever(&mut self, i: usize, novo: &str, chave_nova: &[String]) {
        self.touch();
        let chave_antiga = self.entradas[i].chave.clone();
        let niveis = chave_antiga.len();
        for e in &mut self.entradas {
            if path::key_is_self_or_descendant(&e.chave, &chave_antiga) {
                e.tag.path = path::replace_prefix(&e.tag.path, niveis, novo);
                let mut chave = chave_nova.to_vec();
                chave.extend_from_slice(&e.chave[niveis..]);
                e.chave = chave;
            }
        }
        self.entradas.sort_by(|a, b| a.chave.cmp(&b.chave));
    }

    /// ⭐ **Apaga a tag e a subárvore dela**, e devolve os ids que saíram — quem chama tira a
    /// pertença deles aos objectos NO MESMO gesto.
    pub fn delete(&mut self, id: TagId) -> BTreeSet<TagId> {
        let saem = self.subtree(id);
        if !saem.is_empty() {
            self.touch();
            self.entradas.retain(|e| !saem.contains(&e.tag.id));
        }
        saem
    }

    /// ⭐⭐⭐ **A própria tag e todas as descendentes** — o que *«pertence a `Enemy`»* alcança.
    ///
    /// ⚠️ Um intervalo contíguo a começar na própria tag: a ordem da chave põe um pai imediatamente
    /// antes dos descendentes, e o primeiro que não descende fecha o intervalo.
    #[must_use]
    pub fn subtree(&self, id: TagId) -> BTreeSet<TagId> {
        let Some(i) = self.indice(id) else {
            return BTreeSet::new();
        };
        let raiz = &self.entradas[i].chave;
        self.entradas[i..]
            .iter()
            .take_while(|e| path::key_is_self_or_descendant(&e.chave, raiz))
            .map(|e| e.tag.id)
            .collect()
    }

    /// ⭐⭐⭐ **A tag e os ANCESTRAIS dela, da RAIZ para baixo** — a inversa exacta do
    /// [`Self::subtree`], e a porta que faz a contagem do painel *Tags* caber numa passagem só
    /// pelo mundo (`ph2d_ecs::tags::counts`).
    ///
    /// ⚠️ **Os níveis são PREFIXOS DA CHAVE**, nunca «as tags de profundidade menor que vêm
    /// antes»: a ordem da chave põe uma raiz irmã antes de mim e ela não me ascende a nada.
    ///
    /// ⚠️ Um id que não existe devolve vazio — a mesma cerca do [`Self::reaches`].
    #[must_use]
    pub fn ancestry(&self, id: TagId) -> Vec<TagId> {
        let Some(i) = self.indice(id) else {
            return Vec::new();
        };
        let chave = &self.entradas[i].chave;
        // ⚠️ **Um ancestral pode não existir?** Não: o `create` e o `restore` fazem-no nascer, e o
        // `delete` leva a subárvore. O `filter_map` é a cerca de um documento estranho, não o
        // caminho normal — e saltar um nível em falta mantém a ordem raiz→folha dos que existem.
        (1..=chave.len())
            .filter_map(|n| self.por_chave(&chave[..n]))
            .map(|k| self.entradas[k].tag.id)
            .collect()
    }

    /// ⭐⭐ **`m` está na subárvore de `q`?** — a MESMA pergunta do [`Self::subtree`], sem construir o
    /// conjunto, para quem pergunta um id de cada vez (o filtro da física, a cada evento de colisão).
    ///
    /// ⚠️ Uma `q` ou um `m` que já não existem respondem `false`: a tag apagada não alcança ninguém, e
    /// um id órfão não descende de nada.
    #[must_use]
    pub fn reaches(&self, q: TagId, m: TagId) -> bool {
        let (Some(i), Some(j)) = (self.indice(q), self.indice(m)) else {
            return false;
        };
        path::key_is_self_or_descendant(&self.entradas[j].chave, &self.entradas[i].chave)
    }

    /// ⭐ **Restaura de tags já lidas**, e devolve o `Remap` dos gémeos que fundiu.
    ///
    /// ⚠️ **Os gémeos fundem-se no MENOR id**, e o mapa diz para onde foi cada um: quem restaura a
    /// pertença dos objectos aplica-o, senão os objectos do gémeo descartado perdiam a tag em silêncio.
    ///
    /// ⚠️ **Um ancestral em falta nasce** (com id novo, pela ordem da árvore), e a grafia dos
    /// ancestrais passa aos filhos — a mesma lei do [`Self::create`].
    ///
    /// ⚠️ O `next_id` é o maior entre o gravado e `max(id) + 1`: o gravado impede reciclar o id de uma
    /// tag apagada; o `max` é o piso de um documento que não o trazia.
    pub fn restore(tags: Vec<Tag>, saved_next_id: u64) -> (Self, BTreeMap<TagId, TagId>) {
        let maior = tags.iter().map(|t| t.id.0).max().unwrap_or(0);
        let mut next_id = saved_next_id.max(maior + 1).max(1);
        let mut remap = BTreeMap::new();

        let mut lidas: Vec<Tag> = tags
            .into_iter()
            .filter(|t| t.id.0 != 0)
            .map(|t| Tag {
                id: t.id,
                path: path::normalise(&t.path),
            })
            .filter(|t| !t.path.is_empty())
            .collect();
        lidas.sort_by_key(|t| t.id);

        // ⭐ Indexado pela chave: um gémeo é uma chave que já está no mapa, e o menor id chegou antes.
        let mut por_chave: BTreeMap<Vec<String>, Tag> = BTreeMap::new();
        for t in lidas {
            let chave = chave_de(&t.path);
            match por_chave.get(&chave) {
                Some(existente) => {
                    remap.insert(t.id, existente.id);
                }
                None => {
                    por_chave.insert(chave, t);
                }
            }
        }

        // Os ancestrais, pela ordem da árvore — a chave de um pai é prefixo da do filho, então ele é
        // resolvido (e ganha a grafia final) antes de qualquer filho o ler.
        let lidas_por_ordem: Vec<Vec<String>> = por_chave.keys().cloned().collect();
        for chave in lidas_por_ordem {
            let proprio = por_chave[&chave].path.clone();
            let niveis: Vec<&str> = proprio.split(path::SEP).collect();
            let mut grafia_do_pai = String::new();
            for n in 1..chave.len() {
                let prefixo = &chave[..n];
                if let Some(t) = por_chave.get(prefixo) {
                    grafia_do_pai.clone_from(&t.path);
                    continue;
                }
                let caminho = if grafia_do_pai.is_empty() {
                    niveis[n - 1].to_string()
                } else {
                    format!("{grafia_do_pai}{}{}", path::SEP, niveis[n - 1])
                };
                let id = TagId(next_id);
                next_id += 1;
                por_chave.insert(
                    prefixo.to_vec(),
                    Tag {
                        id,
                        path: caminho.clone(),
                    },
                );
                grafia_do_pai = caminho;
            }
            if !grafia_do_pai.is_empty()
                && let Some(t) = por_chave.get_mut(&chave)
            {
                t.path = format!("{grafia_do_pai}{}{}", path::SEP, niveis[niveis.len() - 1]);
            }
        }

        let entradas = por_chave
            .into_iter()
            .map(|(chave, tag)| Entrada { tag, chave })
            .collect();
        (
            Self {
                entradas,
                next_id,
                revision: 0,
            },
            remap,
        )
    }
}

#[cfg(test)]
#[path = "tags_tests.rs"]
mod tests;
