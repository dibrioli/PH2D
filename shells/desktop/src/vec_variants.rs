//! **OS VARIANTS** — a instância escolhe QUAL versão do componente ela é (plano UI/UX W5c).
//!
//! Irmão do [`crate::vec_component_pieces`] pelo assunto: ali moram as diferenças que esta cópia
//! autorou, aqui **de que mestre ela deriva** quando o catálogo tem mais de um.
//!
//! # Um conjunto de variants é DERIVADO, não declarado
//!
//! ⚠️ **A wave não acrescenta componente nenhum ao ECS, e é isto que a torna barata:** um conjunto
//! de variants é *os mestres IRMÃOS* — os caminhos marcados [`ph2d_ecs::VecComponentMain`] que
//! pendem do mesmo pai. É literalmente o que um *component set* do Figma é (uma moldura com
//! componentes dentro), e é o que o artista já sabe autorar com os gestos que existem: desenhar as
//! três versões, *Create Component* em cada uma, e agrupá-las.
//!
//! Um marcador `VecVariantSet` no pai seria uma **segunda** resposta a *"isto é um conjunto?"* —
//! e divergiria no dia em que alguém agrupasse dois mestres sem o pôr. Aqui a pergunta tem uma
//! resposta só, e ela é a estrutura.
//!
//! ⚠️ **Corolário que decide a ausência da row:** um mestre-RAIZ não tem irmãos, logo não tem
//! variants, logo a seção não pinta rows nenhumas. Sem o pai, *"os irmãos"* seriam *todos os
//! mestres do documento* — e cada componente do projeto viraria variant de todos os outros.
//!
//! # A instância já aponta o variant escolhido
//!
//! [`ph2d_ecs::VecInstance::main`] é o mestre de que ela deriva. Escolher outro variant é
//! **religá-la a um irmão** — ou seja, é exatamente o *Swap Main* que a W5b construiu, restrito
//! aos irmãos e oferecido como uma fileira de chips em vez de um conta-gotas. **Zero schema, e a
//! porta de escrita é a que já está gateada.**
//!
//! # Os EIXOS saem do NOME, e isso é a convenção do Figma
//!
//! Um variant chamado `Size=Small, State=Idle` declara duas propriedades. Quando **todos** os
//! irmãos declaram as MESMAS propriedades, a seção mostra uma fileira por propriedade; quando não
//! (um nome sem `=`, ou eixos que não batem), mostra **uma** fileira `Variant` com os nomes crus.
//!
//! ⚠️ **A tabela de propriedades não é guardada em lado nenhum** — ela é lida dos nomes, que já
//! viajam no `Name` da Hierarquia e que o artista já renomeia ali. Guardá-la seria um segundo
//! lugar onde o mesmo facto envelhece.
//!
//! ⚠️ **Um valor que não leva a lado nenhum não é OFERECIDO.** Numa matriz com buracos
//! (`Size=Large, State=Hover` que ninguém desenhou) o chip existiria, seria clicável, e não faria
//! nada — o botão-morto que este repo persegue. A lista de cada eixo é *o que se alcança a partir
//! da combinação vigente*, e por isso é sempre honesta.

use ph2d_ecs::{Entity, SimWorld, VecComponentMain};
use ph2d_vec_scene::VecPathId;

use crate::vec_entities::VecEntityMap;

/// **Um eixo de propriedade e o que ele oferece** — a projeção que o painel pinta.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VariantAxis {
    /// O nome da propriedade (`Size`), ou `Variant` no modo de nomes crus.
    pub name: String,
    /// Os valores alcançáveis a partir da combinação vigente, em ordem de documento.
    pub values: Vec<String>,
    /// Qual deles é o vigente.
    pub selected: usize,
}

/// O que a seção mostra, e quantas opções o teto de ids deixou de fora.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VariantRows {
    pub axes: Vec<VariantAxis>,
    /// Opções que a tabela de ids não endereça — **escritas**, nunca truncadas em silêncio.
    pub beyond: usize,
}

/// **`"Size=Small, State=Idle"` → `[("Size","Small"), ("State","Idle")]`.**
///
/// `None` quando o nome não é uma combinação: sem `=`, com `=` a mais, ou com um lado vazio. É a
/// porta que decide entre as duas modalidades da seção, e ela é deliberadamente ESTRITA — um nome
/// meio-parseado daria um eixo com um valor só, que é uma fileira que não escolhe nada.
#[must_use]
pub(crate) fn parse_combo(name: &str) -> Option<Vec<(String, String)>> {
    let mut out = Vec::new();
    for part in name.split(',') {
        let mut it = part.splitn(2, '=');
        let k = it.next()?.trim();
        let v = it.next()?.trim();
        if k.is_empty() || v.is_empty() || v.contains('=') {
            return None;
        }
        out.push((k.to_string(), v.to_string()));
    }
    (!out.is_empty()).then_some(out)
}

/// O valor do eixo `key` nesta combinação.
fn value_of<'a>(combo: &'a [(String, String)], key: &str) -> Option<&'a str> {
    combo
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// **As fileiras que o painel pinta, e o mapa de resolução de um clique — a MESMA travessia.**
///
/// ⚠️ Porta única de propósito: quem PINTA e quem HONRA o clique perguntam a mesma coisa ao mesmo
/// mundo no mesmo frame. Duas travessias dariam ordens que poderiam divergir, e o sintoma seria o
/// chip `Large` a escolher `Medium` — sem erro nenhum. É a lição literal do
/// [`crate::vec_component_pieces::addressed_pieces`].
///
/// Devolve `(rows, targets)`, onde `targets[eixo][valor]` é o mestre a que aquele chip religa.
#[must_use]
pub(crate) fn rows_and_targets(
    sim: &SimWorld,
    map: &VecEntityMap,
    main: VecPathId,
) -> (VariantRows, Vec<Vec<VecPathId>>) {
    let sibs = sibling_mains(sim, map, main);
    // Um mestre sozinho não é um conjunto: uma fileira com um chip só é uma escolha que não
    // escolhe.
    if sibs.len() < 2 {
        return (VariantRows::default(), Vec::new());
    }
    let names: Vec<String> = sibs.iter().map(|(_, n)| n.clone()).collect();
    let combos: Vec<Option<Vec<(String, String)>>> = names.iter().map(|n| parse_combo(n)).collect();
    let keys = shared_keys(&combos);
    let (mut axes, mut targets) = match keys {
        Some(keys) => multi_axis(&sibs, &combos, &keys, main),
        None => flat_axis(&sibs, &names, main),
    };
    // ⚠️ O teto é de **TABELA DE IDS**: o `populate` regista `AXES × VALUES` chips e o roteador
    // varre o mesmo intervalo. Não é teto do catálogo — os variants que passam daqui continuam a
    // existir, a desenhar e a ser alcançáveis pelo *Swap Main*, que não tem lista.
    let mut beyond = 0;
    for (a, ax) in axes.iter_mut().enumerate() {
        if a >= ph2d_editor::ids::MAX_VARIANT_AXES {
            beyond += ax.values.len();
            continue;
        }
        let cap = ph2d_editor::ids::MAX_VARIANT_VALUES;
        if ax.values.len() > cap {
            beyond += ax.values.len() - cap;
            ax.values.truncate(cap);
            targets[a].truncate(cap);
            // O vigente pode ter caído fora do teto; sem isto a fileira acenderia o chip errado.
            ax.selected = ax.selected.min(cap.saturating_sub(1));
        }
    }
    axes.truncate(ph2d_editor::ids::MAX_VARIANT_AXES);
    targets.truncate(ph2d_editor::ids::MAX_VARIANT_AXES);
    (VariantRows { axes, beyond }, targets)
}

/// As chaves que **todos** os irmãos declaram, na ordem do primeiro — ou `None` se discordarem.
///
/// ⚠️ A exigência é de IGUALDADE de conjunto, e não de interseção: com `{Size}` num irmão e
/// `{Size, State}` noutro, uma interseção esconderia o `State` do segundo e o artista perderia um
/// eixo sem nada a dizer porquê. Discordar cai no modo de nomes crus, onde tudo aparece.
fn shared_keys(combos: &[Option<Vec<(String, String)>>]) -> Option<Vec<String>> {
    let first = combos.first()?.as_ref()?;
    let keys: Vec<String> = first.iter().map(|(k, _)| k.clone()).collect();
    let same = |c: &Option<Vec<(String, String)>>| {
        c.as_ref()
            .is_some_and(|c| c.len() == keys.len() && keys.iter().all(|k| value_of(c, k).is_some()))
    };
    combos.iter().all(same).then_some(keys)
}

/// O modo de **propriedades**: uma fileira por chave, com os valores alcançáveis daqui.
fn multi_axis(
    sibs: &[(VecPathId, String)],
    combos: &[Option<Vec<(String, String)>>],
    keys: &[String],
    main: VecPathId,
) -> (Vec<VariantAxis>, Vec<Vec<VecPathId>>) {
    let me = sibs.iter().position(|(p, _)| *p == main);
    let Some(mine) = me.and_then(|i| combos[i].as_ref()) else {
        return (Vec::new(), Vec::new());
    };
    let mut axes = Vec::new();
    let mut targets = Vec::new();
    for key in keys {
        let mut values = Vec::new();
        let mut tgt = Vec::new();
        let mut selected = 0;
        for (i, (path, _)) in sibs.iter().enumerate() {
            let Some(c) = combos[i].as_ref() else {
                continue;
            };
            // ⚠️ Alcançável = difere de mim **só** neste eixo. É o que faz de cada chip uma
            // escolha que chega a algum lado, e o que apaga os buracos da matriz sem um estado
            // de erro.
            let reachable = keys
                .iter()
                .all(|k| k == key || value_of(c, k) == value_of(mine, k));
            if !reachable {
                continue;
            }
            let Some(v) = value_of(c, key) else { continue };
            if values.iter().any(|x: &String| x == v) {
                continue;
            }
            if *path == main {
                selected = values.len();
            }
            values.push(v.to_string());
            tgt.push(*path);
        }
        if values.len() > 1 {
            axes.push(VariantAxis {
                name: key.clone(),
                values,
                selected,
            });
            targets.push(tgt);
        }
    }
    (axes, targets)
}

/// O modo de **nomes crus**: uma fileira `Variant` com os irmãos tal como se chamam.
fn flat_axis(
    sibs: &[(VecPathId, String)],
    names: &[String],
    main: VecPathId,
) -> (Vec<VariantAxis>, Vec<Vec<VecPathId>>) {
    let selected = sibs.iter().position(|(p, _)| *p == main).unwrap_or(0);
    (
        vec![VariantAxis {
            name: String::new(), // o rótulo é i18n do painel — ver `paint_components`
            values: names.to_vec(),
            selected,
        }],
        vec![sibs.iter().map(|(p, _)| *p).collect()],
    )
}

/// **Os mestres irmãos de `main`**, na ordem dos filhos do pai, com o nome de cada um.
///
/// ⚠️ A ordem é a de `Children`, que é a mesma que a Hierarquia mostra — os chips aparecem na
/// ordem em que o artista arrumou as versões, e não numa ordem de hash que mudaria entre sessões.
fn sibling_mains(sim: &SimWorld, map: &VecEntityMap, main: VecPathId) -> Vec<(VecPathId, String)> {
    let Some(&bits) = map.get(&main) else {
        return Vec::new();
    };
    let w = sim.world();
    let me = Entity::from_bits(bits);
    // Sem pai não há conjunto — ver o § do cabeçalho.
    let Some(parent) = w
        .get::<ph2d_ecs::ChildOf>(me)
        .map(ph2d_ecs::ChildOf::parent)
    else {
        return Vec::new();
    };
    let Some(kids) = w.get::<ph2d_ecs::Children>(parent) else {
        return Vec::new();
    };
    kids.iter()
        .copied()
        .filter(|&e| w.get::<VecComponentMain>(e).is_some())
        .filter_map(|e| {
            let path = map
                .iter()
                .find(|(_, b)| **b == e.to_bits())
                .map(|(k, _)| *k)?;
            let name = w
                .get::<ph2d_ecs::Name>(e)
                .map_or_else(|| format!("#{path}"), |n| n.0.clone());
            Some((path, name))
        })
        .collect()
}

/// **O mestre a que o chip `(axis, value)` religa** — `None` quando o par não endereça nada.
#[must_use]
pub(crate) fn target_of(
    sim: &SimWorld,
    map: &VecEntityMap,
    main: VecPathId,
    axis: usize,
    value: usize,
) -> Option<VecPathId> {
    let (_, targets) = rows_and_targets(sim, map, main);
    targets.get(axis)?.get(value).copied()
}

#[cfg(test)]
#[path = "vec_variants_tests.rs"]
mod tests;
