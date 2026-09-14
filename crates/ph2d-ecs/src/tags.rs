//! ⭐⭐⭐ **A PERTENÇA a tags** — o item **#9** do TOP-20 (`docs/Components/08_plano_tags.md`).
//!
//! # O que este módulo é, e o que ele NÃO é
//!
//! A árvore de tags do projecto vive na folha [`ph2d_tags`] (sem ECS). Aqui mora só o que um
//! OBJECTO guarda — o conjunto de [`TagId`] a que ele pertence — e as **portas** que respondem às
//! perguntas sobre essa pertença. O modelo é o do Blender (decisão do dono D1, medido): a pertença é
//! uma **referência**, então renomear ou mover uma tag na árvore **nunca** toca num objecto.
//!
//! # ⭐⭐ UMA porta por pergunta
//!
//! | pergunta | porta |
//! |---|---|
//! | *este objecto pertence a `q`?* | [`belongs`] |
//! | *quem pertence a `q`?* | [`tagged`], na ordem do [`crate::StableId`] |
//! | *apaguei estas tags — tira-as dos objectos* | [`scrub`], no MESMO gesto do `TagTree::delete` |
//! | *li um documento com gémeos — reaponta a pertença* | [`remap`], no MESMO gesto do `TagTree::restore` |
//!
//! ⛔ **Pertencer a `q` é pertencer à SUBÁRVORE de `q`** — o `all_objects` do Blender (medido: o
//! objecto só da coleção-filha aparece no pai). Ler o conjunto directo para responder a *«pertence?»*
//! é o defeito que as portas existem para impedir: um objecto marcado `Enemy/Flying` pertence a
//! `Enemy`, e o conjunto directo diz que não. É por isso que o campo é privado e que o único leitor
//! do conjunto directo ([`Tags::direct_ids`]) tem censo (`only_the_door_reads_tags`).
//!
//! ⚠️ **O alcance tem UMA definição, e ela mora na [`TagTree`]** (a chave dobrada do antepassado é o
//! começo da do descendente). As duas portas que perguntam passam por ela, cada uma pela forma que a
//! frequência dela pede: [`tagged`] pede o CONJUNTO uma vez por consulta ([`TagTree::subtree`]), e
//! [`belongs`] pergunta um id de cada vez ([`TagTree::reaches`]), sem construir conjunto — o filtro da
//! física chama-o a cada evento de colisão. ⛔ Uma resposta escrita AQUI, contra o caminho de texto,
//! seria a segunda, e divergiria no dia em que a regra de descendência mudasse.
//!
//! # ⚠️ As leis que este componente herda
//!
//! - **É CONFIG, e é registado.** Sem o registo, o artista marca um objecto, grava, reabre, e o
//!   objecto volta sem tags — nada some da tela e nada dá erro, e o sinal simplesmente deixa de o
//!   atingir.
//! - **Os bytes não dependem da ordem em que as tags foram postas** (`BTreeSet`): o undo regista por
//!   DIFF de bytes, e dois gestos que chegam ao mesmo conjunto por ordens diferentes não podem ser
//!   dois estados.
//! - ⚠️ **Guarda `u64` e não `TagId`** porque a folha [`ph2d_tags`] não conhece `serde` (de propósito:
//!   o formato é da família, não da árvore). A conversão vive só aqui dentro.
//! - **Não há índice** `TagId → entidades`, e está MEDIDO (`measure_tag_scan`, plano §6.1): a
//!   varredura de 100 000 objectos todos marcados custa `0,56 ms` (`3,4 %` de um quadro). Um índice seria estado derivado a
//!   manter coerente depois de todo restore do undo — o preço que o `stable_id.rs` já recusou.

use crate::{Entity, StableId, World};
use bevy_ecs::component::Component;
use ph2d_tags::{TagId, TagTree};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// **Quantas tags um objecto pode ter.**
///
/// ⚠️ **O número sai da SECÇÃO, não de um palpite** — é a lei que o [`crate::ANIM_TAGS_MAX`] já
/// pagou: *um modelo que aceita o que o painel não mostra produz estado inalcançável*. A secção
/// *Tags* do Inspector desenha um chip por tag dentro da largura da coluna, e `16` é o que cabe sem
/// ela sozinha passar a altura útil — o mesmo argumento do [`crate::TIMERS_MAX`].
///
/// ⛔ Não é um limite da ÁRVORE: o projecto pode ter as tags que quiser (medido: `9 344` custam
/// `9,2 ms` a abrir). Este é o das que cabem num objecto.
pub const TAGS_MAX: usize = 16;

/// As tags a que um objecto pertence DIRECTAMENTE.
///
/// ⚠️ **O campo é privado**: quem pergunta *«pertence?»* passa por [`belongs`] / [`tagged`], que
/// expandem a subárvore.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tags(BTreeSet<u64>);

impl Tags {
    /// Um conjunto com estas tags. ⚠️ O `TagId(0)` nunca é dado pela árvore e é ignorado, e o que
    /// passar do [`TAGS_MAX`] fica de fora (pelos ids mais altos) — ver a lei do tecto.
    pub fn from_ids(ids: impl IntoIterator<Item = TagId>) -> Self {
        let mut set: BTreeSet<u64> = ids.into_iter().map(|t| t.0).filter(|&i| i != 0).collect();
        while set.len() > TAGS_MAX {
            let ultima = *set.iter().next_back().expect("o conjunto nao esta' vazio");
            set.remove(&ultima);
        }
        Self(set)
    }

    /// Marca o objecto com `id`. Devolve `false` se já estava, se `id` é o reservado `0`, ou se o
    /// objecto já tem [`TAGS_MAX`] tags.
    ///
    /// ⚠️ **O tecto é do CONJUNTO**: tirar uma abre espaço para outra, e não há contador ao lado a
    /// manter coerente.
    pub fn insert(&mut self, id: TagId) -> bool {
        id.0 != 0 && self.0.len() < TAGS_MAX && self.0.insert(id.0)
    }

    /// Tira `id`. Devolve `false` se não estava.
    pub fn remove(&mut self, id: TagId) -> bool {
        self.0.remove(&id.0)
    }

    /// Nenhuma tag directa.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Quantas tags directas.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// ⚠️ **As tags DIRECTAS, por ordem de id** — para quem as MOSTRA (os chips do Inspector).
    ///
    /// ⛔ **Nunca para responder *«pertence a `q`?»*** — isso é [`belongs`], que alcança a
    /// subárvore. Há censo (`only_the_door_reads_tags`) sobre quem chama isto.
    pub fn direct_ids(&self) -> impl Iterator<Item = TagId> + '_ {
        self.0.iter().map(|&i| TagId(i))
    }

    /// Alguma tag directa está em `reach`? — a metade comum das portas, privada.
    fn meets(&self, reach: &BTreeSet<TagId>) -> bool {
        self.0.iter().any(|&i| reach.contains(&TagId(i)))
    }
}

/// ⭐⭐⭐ **Este objecto pertence a `q`?** — a `q` ou a qualquer descendente dela.
///
/// ⚠️ Uma `q` que já não existe na árvore é **ninguém** (a lei do alvo que não existe): um filtro ou
/// um sinal que apontava para uma tag apagada falha FECHADO. E um id órfão no objecto não é
/// descendente de nada, porque a árvore não o encontra.
#[must_use]
pub fn belongs(tags: &Tags, tree: &TagTree, q: TagId) -> bool {
    tags.0.iter().any(|&m| tree.reaches(q, TagId(m)))
}

/// ⭐⭐⭐ **Quem pertence a `q`** (com a subárvore), na ordem da IDENTIDADE.
///
/// ⚠️ **A ordem é a do [`StableId`], nunca a da query** — a do `bevy_ecs` é a do arquétipo, que muda
/// quando um componente é inserido, e dois efeitos sobre o mesmo alvo têm de chegar sempre pela mesma
/// ordem (a lei do `signal_actions::resolve`). Um objecto ainda sem `StableId` vem depois dos que o
/// têm, por índice de entidade.
///
/// ⚠️ **`&World` e não `&mut`**: é uma leitura, e o painel conta os membros de cada tag enquanto
/// pinta, onde só há um `&World`. Um mundo que nunca viu o componente devolve ninguém.
#[must_use]
pub fn tagged(world: &World, tree: &TagTree, q: TagId) -> Vec<Entity> {
    let reach = tree.subtree(q);
    if reach.is_empty() {
        return Vec::new();
    }
    let Some(mut query) = world.try_query::<(Entity, &Tags)>() else {
        return Vec::new();
    };
    // ⛔⛔ **A identidade é lida POR ACERTO, e não pedida na query** — e a diferença não é de estilo.
    // O `try_query` do `bevy_ecs` devolve `None` quando **qualquer** componente da consulta é
    // desconhecido do mundo, e um `Option<&StableId>` conta: num mundo onde nada nunca teve
    // `StableId` (o `assign_missing_stable_ids` corre no caminho do NOME e na captura, não no
    // spawn), esta porta respondia **«ninguém»** — um sinal por tag não alcançava nada e nada o
    // dizia. Apanhado pelo gate do painel *Tags* em 2026-09-14, com a fixtura a spawnar sem ele.
    // ⚠️ O preço é uma leitura por ACERTO (não por entidade), e os acertos são poucos.
    let mut hits: Vec<(u64, Entity)> = query
        .iter(world)
        .filter(|(_, t)| t.meets(&reach))
        .map(|(e, _)| e)
        .collect::<Vec<_>>()
        .into_iter()
        .map(|e| {
            let s = world
                .get::<StableId>(e)
                .filter(|s| !s.is_none())
                .map_or(u64::MAX, |s| s.0);
            (s, e)
        })
        .collect();
    hits.sort_unstable_by_key(|&(s, e)| (s, e.index()));
    hits.into_iter().map(|(_, e)| e).collect()
}

/// ⭐⭐⭐ **Quantos objectos pertencem a CADA tag** — a coluna do painel *Tags*, numa passagem só.
///
/// A resposta é, tag a tag, a mesma do [`tagged`] (há gate a exigi-lo). O que muda é o preço: o
/// painel repinta a cada quadro e precisa das `N` contagens juntas, não de uma.
///
/// # ⚠️ Porque não é `tagged` em laço, e porque não é a soma dos filhos
///
/// `tagged` varre o mundo **por tag** ⇒ `O(tags × objectos)`; aqui cada objecto é visitado uma vez
/// e sobe a própria ancestralidade ⇒ `O(objectos × tags_do_objecto × profundidade)`.
///
/// **MEDIDO** (`measure_tag_counts`, `--release`, mediana de 25; `load 72,41` — a máquina tinha
/// outra suíte a correr, então **as absolutas são um TECTO e a razão é o que se lê**, porque as
/// duas colunas saem do mesmo mundo no mesmo instante):
///
/// | objectos | com tags | tags | `counts` | `tagged` em laço | razão |
/// |---:|---:|---:|---:|---:|---:|
/// | 1 000 | 100 | 10 | 0,0092 ms | 0,0076 ms | **0,8×** |
/// | 1 000 | 100 | 584 | 0,0288 ms | 0,3895 ms | 13,5× |
/// | 10 000 | 1 000 | 584 | 0,2897 ms | 2,1800 ms | 7,5× |
/// | 10 000 | 10 000 | 10 | 0,8820 ms | 0,4216 ms | **0,5×** |
/// | 10 000 | 10 000 | 584 | 2,9732 ms | 21,1946 ms | 7,1× |
/// | 100 000 | 10 000 | 584 | 3,0816 ms | 20,5228 ms | 6,7× |
///
/// ⚠️⚠️ **A porta rápida PERDE quando a árvore é pequena** (as duas linhas a negrito), e isso não é
/// ruído: com 10 tags, subir a ancestralidade de cada objecto custa mais do que dez varreduras do
/// mundo. ⇒ *o `counts` não é «a versão rápida», é a que não EXPLODE* — ela troca um produto
/// `tags × objectos` por uma soma, e é esse o recurso. Com a árvore pequena as duas cabem com
/// folga, e com ela grande só uma cabe.
///
/// ⛔ **E nenhuma das duas cabe num quadro no extremo** (2,97 ms é 18 % de 16,7): é por isso que o
/// chamador só a corre com o painel *Tags* **ABERTO** — a coluna não existe enquanto ninguém a vê.
///
/// ⛔ **Somar as contagens dos filhos está ERRADO por construção**, e não é uma optimização
/// recusada por preço: um objecto com `Boss` **e** `Enemy` entraria duas vezes em `Enemy`. É a
/// mesma razão por que o conjunto existe — a pergunta é *«quantos OBJECTOS»*, não *«quantas
/// pertenças»*.
///
/// ⚠️ **Toda tag da árvore tem linha, mesmo a zero** — o painel desenha as linhas da ÁRVORE, e uma
/// tag sem membros continua a ser uma tag. ⚠️ E um id órfão (tag apagada sem o [`scrub`]) não ganha
/// linha nenhuma: ele não está na árvore, logo não é membro de nada.
#[must_use]
pub fn counts(world: &World, tree: &TagTree) -> BTreeMap<TagId, usize> {
    let mut out: BTreeMap<TagId, usize> = tree.tags().map(|t| (t.id, 0usize)).collect();
    let Some(mut query) = world.try_query::<&Tags>() else {
        return out;
    };
    // ⚠️ Reaproveitado entre objectos: um `BTreeSet` novo por entidade seria uma alocação por
    // objecto por quadro, que é exactamente o que o painel não pode pagar.
    let mut alcance: BTreeSet<TagId> = BTreeSet::new();
    for tags in query.iter(world) {
        alcance.clear();
        for m in tags.direct_ids() {
            alcance.extend(tree.ancestry(m));
        }
        // ⛔⛔ **«Um id órfão não conta para ninguém» tem DOIS guardas, e nenhum é observável
        // sozinho** — medido por três redacções de mutação (W4, 2026-09-14):
        //
        // 1. o [`TagTree::ancestry`] devolve **vazio** para um id que a árvore não conhece, logo um
        //    órfão nunca entra no `alcance`;
        // 2. o `out` só tem as chaves de `tree.tags()`, logo o `get_mut` recusa-o aqui.
        //
        // ⚠️ Apagar QUALQUER UM deles deixa o gate VERDE — *uma mutação sobre um guarda defendido
        // pelo outro mede a redundância, não a lei*. A que sangra escreve o id **directo** no mapa,
        // saltando os dois de uma vez. ⇒ os dois ficam: cada um continua a ser a rede do outro no
        // dia em que a porta de cima mudar de contrato.
        for q in &alcance {
            if let Some(n) = out.get_mut(q) {
                *n += 1;
            }
        }
    }
    out
}

/// ⭐⭐ **Tira estas tags de todos os objectos** — o par obrigatório do `TagTree::delete`, no MESMO
/// gesto (senão ficam ids órfãos nos objectos, que um id futuro nunca reusa mas que o painel
/// contaria).
///
/// Devolve **quantos objectos** perderam alguma tag — o *«remove from N objects»* do painel.
///
/// ⚠️ Escreve **só** nos objectos tocados: o undo regista por DIFF, e um carimbo de mudança em quem
/// não mudou seria trabalho de captura por nada. ⚠️ O componente FICA com a lista vazia: a secção
/// do Inspector é do objecto, não das tags que ele tinha.
pub fn scrub(world: &mut World, ids: &BTreeSet<TagId>) -> usize {
    if ids.is_empty() {
        return 0;
    }
    let Some(mut query) = world.try_query::<(Entity, &Tags)>() else {
        return 0;
    };
    let touched: Vec<Entity> = query
        .iter(world)
        .filter(|(_, t)| t.meets(ids))
        .map(|(e, _)| e)
        .collect();
    for &e in &touched {
        if let Some(mut t) = world.get_mut::<Tags>(e) {
            t.0.retain(|&i| !ids.contains(&TagId(i)));
        }
    }
    touched.len()
}

/// ⭐⭐ **Reaponta a pertença dos gémeos fundidos** — o par obrigatório do `TagTree::restore`.
///
/// Devolve quantos objectos mudaram. ⚠️ Dois ids que se fundem no mesmo objecto dão UMA tag (é um
/// conjunto). Escreve só nos objectos tocados, pela razão do [`scrub`].
pub fn remap(world: &mut World, remap: &BTreeMap<TagId, TagId>) -> usize {
    if remap.is_empty() {
        return 0;
    }
    let Some(mut query) = world.try_query::<(Entity, &Tags)>() else {
        return 0;
    };
    let touched: Vec<Entity> = query
        .iter(world)
        .filter(|(_, t)| t.0.iter().any(|&i| remap.contains_key(&TagId(i))))
        .map(|(e, _)| e)
        .collect();
    for &e in &touched {
        if let Some(mut t) = world.get_mut::<Tags>(e) {
            let novo: BTreeSet<u64> =
                t.0.iter()
                    .map(|&i| remap.get(&TagId(i)).map_or(i, |d| d.0))
                    .filter(|&i| i != 0)
                    .collect();
            t.0 = novo;
        }
    }
    touched.len()
}

#[cfg(test)]
#[path = "tags_tests.rs"]
mod tests;
