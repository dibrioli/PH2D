//! ⭐⭐ **A TAXONOMIA da biblioteca** — os catálogos (plano 07, wave A3).
//!
//! # ⛔ Não há árvore de PASTAS, e a decisão é anterior a esta wave
//!
//! O `res://` do Godot e o sistema de ficheiros do Blender não existem aqui — um asset deste app
//! **nasce dentro dele** (ADR-0165), não num disco. A taxonomia é de **catálogos**, que é a escolha
//! do Blender Asset Browser: uma etiqueta hierárquica que o artista inventa, independente de onde
//! os bytes estão.
//!
//! # ⭐⭐⭐ A hierarquia vive no CAMINHO, e não numa árvore de nós
//!
//! Um catálogo é `{ id, path }`, e o `path` é `"Personagens/Heróis"`. A árvore que a UI desenha é
//! **derivada** de dividir o caminho — não existe um `parent: Option<CatalogId>` a duplicar a mesma
//! informação.
//!
//! ⚠️ **É o modelo do Blender, e a razão é a operação que mais dói:** renomear um catálogo com
//! filhos. Com um caminho, é reescrever um prefixo — uma travessia, sem invariante para partir. Com
//! nós e elos, é uma travessia recursiva que tem de manter dois lados de acordo, e o modo de falha
//! é um filho a apontar para um pai apagado.
//!
//! ⚠️ **E a identidade NÃO é o caminho** — é o `id`. Renomear *«Personagens»* não pode desligar
//! todos os assets que estão lá dentro, e é por isso que a atribuição guarda o id e nunca o texto.
//!
//! # A cerca
//!
//! ⛔ Esta crate continua **folha**: sem serde, sem ECS, sem I/O. Quem grava é o shell
//! (`project_catalogs.rs`), e é lá que vive a versão do formato.

use crate::{AssetRef, CatalogId};
use std::collections::BTreeMap;

/// O separador de níveis de um caminho de catálogo.
///
/// ⚠️ **Ele é o formato, não uma escolha de apresentação** — a árvore da UI é derivada de o
/// dividir, e mudá-lo reinterpreta todo caminho já gravado.
pub const SEP: char = '/';

/// Um catálogo. ⚠️ O `path` é a HIERARQUIA; o `id` é a IDENTIDADE.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    /// A identidade durável. ⚠️ É ela que a atribuição de um asset guarda — nunca o caminho.
    pub id: CatalogId,
    /// `"Personagens/Heróis"`. A árvore sai daqui.
    pub path: String,
}

impl Catalog {
    /// O rótulo que a linha mostra — o último nível do caminho.
    #[must_use]
    pub fn label(&self) -> &str {
        self.path.rsplit(SEP).next().unwrap_or(&self.path)
    }

    /// A que profundidade esta linha é desenhada (`0` = raiz).
    #[must_use]
    pub fn depth(&self) -> usize {
        self.path.matches(SEP).count()
    }
}

/// **O que a grade está a mostrar** — três estados, e nenhuma combinação impossível.
///
/// ⚠️ **`Option<Option<CatalogId>>` seria a mesma informação com um estado a mais que ninguém sabe
/// ler.** *«Todos»* e *«os que não estão em catálogo nenhum»* são perguntas diferentes, e a segunda
/// é a que o Blender chama *Unassigned* — sem ela um asset por arrumar fica inalcançável no dia em
/// que existir um catálogo.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum CatalogScope {
    /// Sem filtro.
    #[default]
    All,
    /// Só os que não pertencem a catálogo nenhum.
    Unassigned,
    /// O catálogo escolhido **e os descendentes dele**, já expandidos por
    /// [`CatalogTree::scope_of`].
    ///
    /// ⚠️ **Expandidos por quem tem a árvore, e não pela consulta.** Escolher *«Personagens»* tem
    /// de mostrar o que está em *«Personagens/Heróis»* — é o que todo navegador faz —, e o índice
    /// não conhece caminhos: ele guarda um `CatalogId` por entrada e mais nada.
    These(Vec<CatalogId>),
}

/// A taxonomia inteira: os catálogos e a que catálogo cada asset pertence.
///
/// ⚠️ **A `revision` fica FORA do `PartialEq`** — ela é uma chave de CACHE, nunca identidade. Duas
/// árvores com o mesmo conteúdo são iguais mesmo tendo chegado lá por caminhos diferentes, que é o
/// que faz um `restore` de undo não registar um passo espúrio contra a árvore que o produziu.
#[derive(Clone, Debug, Default, Eq)]
pub struct CatalogTree {
    /// Ordenados por caminho — é a ordem em que a UI os desenha, e é o que torna a árvore
    /// derivável por uma passagem só.
    catalogs: Vec<Catalog>,
    /// `asset → catálogo`. Ausente = *Unassigned*.
    assignments: BTreeMap<AssetRef, CatalogId>,
    /// De onde sai o próximo id. ⚠️ Monotónico e **nunca reutilizado**: um id reciclado faria os
    /// assets de um catálogo apagado reaparecerem dentro do seguinte.
    next_id: u128,
    /// ⭐⭐ **Sobe a cada mutação — e é SÓ uma chave de cache.**
    ///
    /// ⛔ Ela não entra no `PartialEq` e não é identidade: quem decide se duas taxonomias são a
    /// mesma são os **bytes**. O que ela compra é não re-codificar a árvore por quadro — medido em
    /// 2026-08-30: `collect` custa **4,8 % de um quadro** a 50 catálogos e 2 000 atribuições, e
    /// **28 %** a 200/10 000, e a captura do undo corre em todo quadro com input.
    revision: u64,
}

impl PartialEq for CatalogTree {
    /// ⚠️ **Conteúdo, nunca `revision`** — ver o campo. O `next_id` entra porque ele decide o
    /// próximo gesto: duas árvores que desenham igual e dão ids diferentes não são a mesma.
    fn eq(&self, other: &Self) -> bool {
        self.catalogs == other.catalogs
            && self.assignments == other.assignments
            && self.next_id == other.next_id
    }
}

impl CatalogTree {
    /// Uma taxonomia vazia.
    #[must_use]
    pub fn new() -> Self {
        Self {
            catalogs: Vec::new(),
            assignments: BTreeMap::new(),
            next_id: 1,
            revision: 0,
        }
    }

    /// A porta ÚNICA do bump. ⚠️ Chamada onde a mutação de facto aconteceu — um `create` que
    /// encontra o caminho ou um `rename` que recusa **não** movem a revisão, senão a cache
    /// re-codificaria a árvore por um gesto que não a mudou.
    fn touch(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    /// ⭐ **A revisão desta árvore** — sobe a cada mutação. Ver o campo: é chave de cache, e quem
    /// a lê é quem quer saber *«preciso de re-codificar?»*, nunca *«são a mesma?»*.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Os catálogos, ordenados por caminho.
    #[must_use]
    pub fn catalogs(&self) -> &[Catalog] {
        &self.catalogs
    }

    /// Está vazia?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.catalogs.is_empty()
    }

    /// O catálogo com este id.
    #[must_use]
    pub fn get(&self, id: CatalogId) -> Option<&Catalog> {
        self.catalogs.iter().find(|c| c.id == id)
    }

    /// ⭐ **Cria um catálogo em `path`, e os ANCESTRAIS que faltarem.**
    ///
    /// ⚠️ **Os ancestrais nascem aqui, e é o que impede uma linha fantasma na UI:** a árvore é
    /// derivada de dividir caminhos, então `"A/B"` sem `"A"` desenharia um filho sem pai. Criá-los
    /// na porta é a lei imposta na DERIVAÇÃO, não em cada gesto.
    ///
    /// Devolve o id do catálogo pedido — o existente, se o caminho já lá estava.
    pub fn create(&mut self, path: &str) -> CatalogId {
        let path = normalise(path);
        if let Some(c) = self.catalogs.iter().find(|c| c.path == path) {
            return c.id;
        }
        // Os ancestrais primeiro, para a lista sair ordenada e a UI nunca ver um órfão.
        let mut prefix = String::new();
        let mut last = CatalogId(0);
        for part in path.split(SEP) {
            if !prefix.is_empty() {
                prefix.push(SEP);
            }
            prefix.push_str(part);
            last = match self.catalogs.iter().find(|c| c.path == prefix) {
                Some(c) => c.id,
                None => {
                    let id = CatalogId(self.next_id);
                    self.next_id += 1;
                    self.revision = self.revision.wrapping_add(1);
                    self.catalogs.push(Catalog {
                        id,
                        path: prefix.clone(),
                    });
                    id
                }
            };
        }
        sort_as_a_tree(&mut self.catalogs);
        last
    }

    /// ⭐⭐ **Renomear um catálogo é reescrever um PREFIXO** — e é isso que leva os filhos junto.
    ///
    /// ⚠️ **As atribuições não se tocam:** elas guardam o `id`, e o id não muda. *Renomear uma
    /// gaveta não esvazia a gaveta.*
    ///
    /// `false` se o id não existe, se o nome novo é vazio, ou se ele contém o separador (um nome
    /// com `/` seria um segundo gesto — mover — escondido dentro de renomear).
    pub fn rename(&mut self, id: CatalogId, new_label: &str) -> bool {
        let label = new_label.trim();
        if label.is_empty() || label.contains(SEP) {
            return false;
        }
        let Some(old) = self.get(id).map(|c| c.path.clone()) else {
            return false;
        };
        let parent = match old.rsplit_once(SEP) {
            Some((p, _)) => format!("{p}{SEP}"),
            None => String::new(),
        };
        let new_path = format!("{parent}{label}");
        if new_path == old {
            return true;
        }
        self.touch();
        // ⚠️ **Só o próprio e os DESCENDENTES**, e a fronteira é o separador: sem ele, renomear
        // `"Hero"` reescreveria `"Heroine"` — um prefixo de texto não é um prefixo de caminho.
        let child_prefix = format!("{old}{SEP}");
        for c in &mut self.catalogs {
            if c.path == old {
                c.path = new_path.clone();
            } else if let Some(rest) = c.path.strip_prefix(&child_prefix) {
                c.path = format!("{new_path}{SEP}{rest}");
            }
        }
        sort_as_a_tree(&mut self.catalogs);
        true
    }

    /// ⭐ **Apagar leva os descendentes** — e os assets que estavam lá dentro voltam a *Unassigned*.
    ///
    /// ⚠️ **Nunca apaga um asset**, e a distinção é a que o report do Enio sobre a biblioteca
    /// pagou: um catálogo é uma etiqueta, e tirar a etiqueta não é deitar fora o que ela nomeava.
    pub fn delete(&mut self, id: CatalogId) {
        let Some(path) = self.get(id).map(|c| c.path.clone()) else {
            return;
        };
        let child_prefix = format!("{path}{SEP}");
        let doomed: Vec<CatalogId> = self
            .catalogs
            .iter()
            .filter(|c| c.path == path || c.path.starts_with(&child_prefix))
            .map(|c| c.id)
            .collect();
        self.touch();
        self.catalogs.retain(|c| !doomed.contains(&c.id));
        self.assignments.retain(|_, v| !doomed.contains(v));
    }

    /// Põe um asset num catálogo. ⚠️ **Um asset está em UM catálogo** (a escolha do Blender):
    /// atribuir tira-o do anterior sem gesto nenhum, porque a chave é o asset.
    pub fn assign(&mut self, asset: AssetRef, catalog: CatalogId) {
        if self.get(catalog).is_some() && self.assignments.insert(asset, catalog) != Some(catalog) {
            self.touch();
        }
    }

    /// Tira um asset de qualquer catálogo.
    pub fn unassign(&mut self, asset: &AssetRef) {
        if self.assignments.remove(asset).is_some() {
            self.touch();
        }
    }

    /// A que catálogo este asset pertence.
    #[must_use]
    pub fn catalog_of(&self, asset: &AssetRef) -> Option<CatalogId> {
        self.assignments.get(asset).copied()
    }

    /// Todas as atribuições, para quem as grava.
    #[must_use]
    pub fn assignments(&self) -> &BTreeMap<AssetRef, CatalogId> {
        &self.assignments
    }

    /// ⭐⭐ **O escopo de uma escolha: ele e os descendentes dele.**
    ///
    /// ⚠️ É a metade que faz *«Personagens»* mostrar o que está em *«Personagens/Heróis»* — o que
    /// todo navegador faz, e o que uma comparação de igualdade não daria.
    #[must_use]
    pub fn scope_of(&self, id: CatalogId) -> CatalogScope {
        let Some(path) = self.get(id).map(|c| c.path.clone()) else {
            return CatalogScope::All;
        };
        let child_prefix = format!("{path}{SEP}");
        CatalogScope::These(
            self.catalogs
                .iter()
                .filter(|c| c.path == path || c.path.starts_with(&child_prefix))
                .map(|c| c.id)
                .collect(),
        )
    }

    /// Quantos assets estão neste catálogo **ou nos descendentes** — o número que a linha mostra.
    #[must_use]
    pub fn count_in(&self, id: CatalogId) -> usize {
        let CatalogScope::These(ids) = self.scope_of(id) else {
            return 0;
        };
        self.assignments
            .values()
            .filter(|v| ids.contains(v))
            .count()
    }

    /// **Restaura de bytes já lidos** (a porta do shell). ⚠️ O `next_id` é empurrado para além de
    /// tudo o que chegou: um ficheiro gravado com ids altos não pode fazer o próximo gesto sentar-se
    /// em cima de um catálogo existente. É o mesmo contrato do `next_import_cell`.
    pub fn restore(catalogs: Vec<Catalog>, assignments: BTreeMap<AssetRef, CatalogId>) -> Self {
        let next_id = catalogs.iter().map(|c| c.id.0).max().unwrap_or(0) + 1;
        let mut t = Self {
            catalogs,
            assignments,
            next_id,
            revision: 0,
        };
        sort_as_a_tree(&mut t.catalogs);
        t
    }
}

/// ⭐⭐⭐ **A ordem em que a UI desenha as linhas — e ela é por SEGMENTO, não pelo texto cru.**
///
/// ⛔⛔ **Ordenar pela string parte a árvore, e um gate apanhou-o na primeira corrida.** A árvore é
/// derivada da PROFUNDIDADE de cada linha, o que exige que um pai venha imediatamente antes dos
/// filhos dele. Com a comparação de texto isso é falso: `'-'` (0x2D) é menor que `'/'` (0x2F),
/// então `"A-x"` cai **entre** `"A"` e `"A/B"` — o filho aparece indentado debaixo de um irmão.
///
/// ⇒ a chave é a **sequência de níveis**, comparada nível a nível: `["a"] < ["a","b"] < ["a-x"]`.
/// Um pai é um prefixo do filho, e um prefixo é sempre menor.
///
/// ⚠️ **Em minúsculas, que é a convenção desta casa** (a grade ordena por `name.to_lowercase()`).
/// ⛔ Ela NÃO é uma colação de Unicode: `"Ártico"` continua a vir depois de `"Zebra"`, porque uma
/// ordenação com acentos exige uma tabela que este repo não tem — e inventá-la aqui poria uma
/// política de idioma numa crate folha. *A ordenação da grade tem exactamente a mesma dívida, e é
/// melhor terem a mesma que terem duas.*
fn sort_as_a_tree(catalogs: &mut [Catalog]) {
    catalogs.sort_by_cached_key(|c| {
        c.path
            .split(SEP)
            .map(str::to_lowercase)
            .collect::<Vec<String>>()
    });
}

/// Um caminho sem espaços nas pontas de cada nível e sem níveis vazios.
///
/// ⚠️ Sem isto, `"A//B"` e `"A/ B"` criariam níveis que a UI desenha como linhas em branco, e
/// `"A/"` criaria um filho sem nome que ninguém consegue escolher nem apagar.
fn normalise(path: &str) -> String {
    path.split(SEP)
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(&SEP.to_string())
}

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod tests;
