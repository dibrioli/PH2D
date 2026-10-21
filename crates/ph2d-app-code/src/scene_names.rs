//! ⭐ **Os nomes da cena que o editor conhece** (item 8, `docs/CodeEditor/07_arrastar_para_o_codigo.md`
//! §3.6–8): os objectos (e quantos têm cada nome) e os mestres — a verificação de `ph2d.object("…")`, a
//! lista de sugestões dentro das aspas e o Ctrl+clique que selecciona. As cópias da corrida (`Spawned`)
//! não contam: o editor verifica a cena que o artista fez.

use ph2d_ecs::{Entity, MasterRoot, Name, Spawned, World};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SceneNames {
    /// Os objectos da cena (não-receita): o nome e quantos o têm.
    pub objects: BTreeMap<String, usize>,
    /// Os mestres (o que o `ph2d.spawn` recebe).
    pub masters: BTreeSet<String>,
}

impl SceneNames {
    pub(crate) fn read(world: &mut World) -> Self {
        let mut s = Self::default();
        let mut q = world.query::<(Entity, &Name, Option<&MasterRoot>, Option<&Spawned>)>();
        let rows: Vec<(Entity, String, bool, bool)> = q
            .iter(world)
            .map(|(e, n, m, sp)| (e, n.as_str().to_owned(), m.is_some(), sp.is_some()))
            .collect();
        for (e, name, master, spawned) in rows {
            if spawned {
                continue;
            }
            if master {
                s.masters.insert(name);
            } else if !ph2d_ecs::is_master_piece(world, e) {
                *s.objects.entry(name).or_default() += 1;
            }
        }
        s
    }

    /// O objecto da cena com este nome, o primeiro pela ordem da lista (o do `ph2d.object`).
    pub(crate) fn entity_named(world: &mut World, name: &str) -> Option<Entity> {
        ph2d_ecs::list_order(world).into_iter().find(|&e| {
            world.get::<MasterRoot>(e).is_none()
                && world.get::<Spawned>(e).is_none()
                && !ph2d_ecs::is_master_piece(world, e)
                && world.get::<Name>(e).is_some_and(|n| n.as_str() == name)
        })
    }
}

/// ⭐ **As sugestões dentro das aspas** de `ph2d.object("…`, `ph2d.find_by_name("…` (objectos) e
/// `ph2d.spawn("…` (mestres) — o `$` do Godot, que lista os caminhos dos nós. `anchor` é o início da
/// palavra da sessão; o que já está escrito entre a aspa e ela (um nome com espaços: `Caixa |`) só
/// deixa os nomes que começam por ele, e a proposta escreve o RESTO.
pub(crate) fn scene_items(
    text: &ph2d_code_buffer::Text,
    anchor: usize,
    scene: &SceneNames,
) -> Vec<crate::complete::Item> {
    let line = text.byte_of_line(text.line_of_byte(anchor));
    let before = text.slice(line..anchor);
    let Some((call, after)) = ["ph2d.object(", "ph2d.find_by_name(", "ph2d.spawn("]
        .iter()
        .filter_map(|c| before.rfind(c).map(|i| (*c, &before[i + c.len()..])))
        .max_by_key(|(_, rest)| std::cmp::Reverse(rest.len()))
    else {
        return Vec::new();
    };
    let after = after.trim_start();
    let Some(q) = after.chars().next().filter(|c| matches!(c, '"' | '\'')) else {
        return Vec::new();
    };
    let typed = &after[1..];
    if typed.contains(q) {
        return Vec::new();
    }
    let (names, detail): (Vec<&str>, _) = if call == "ph2d.spawn(" {
        (
            scene.masters.iter().map(String::as_str).collect(),
            "code.complete.scene_master",
        )
    } else {
        (
            scene.objects.keys().map(String::as_str).collect(),
            "code.complete.scene_object",
        )
    };
    names
        .into_iter()
        .filter_map(|n| n.strip_prefix(typed).map(|rest| (n, rest)))
        .filter(|(_, rest)| !rest.is_empty())
        .map(|(n, rest)| crate::complete::Item {
            label: n.to_owned(),
            kind: Some(12),
            detail: Some(ph2d_i18n::tr(detail).to_owned()),
            documentation: None,
            sort_text: "0".to_owned(),
            filter: Some(rest.to_owned()),
            insert: rest.to_owned(),
            snippet: false,
            signature: false,
            usage: 0,
        })
        .collect()
}

/// O que as abas leem (partilhado; trocado só quando a cena muda de nomes).
pub(crate) type Shared = Arc<SceneNames>;

impl crate::docs::CodeDocs {
    /// Um quadro: os nomes da cena (só com abas abertas); as abas passam a ler a cópia nova quando
    /// eles mudam — e o lint reconta (a chave dele é a cópia).
    pub(crate) fn follow_scene(&mut self, world: &mut World) {
        if self.docs.is_empty() {
            return;
        }
        let now = SceneNames::read(world);
        if *self.scene != now {
            self.scene = Arc::new(now);
        }
        for d in &mut self.docs {
            if !Arc::ptr_eq(&d.scene, &self.scene) {
                d.scene = Arc::clone(&self.scene);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_code_buffer::Text;

    #[test]
    fn as_sugestoes_dentro_das_aspas_escrevem_o_resto_do_nome() {
        let scene = SceneNames {
            objects: [("Caixa Grande", 1), ("Caixa", 1), ("Porta", 1)]
                .map(|(n, c)| (n.to_owned(), c))
                .into(),
            masters: ["Bala".to_owned()].into(),
        };
        let em = |src: &str, at: usize| -> Vec<(String, String)> {
            scene_items(&Text::new(src), at, &scene)
                .into_iter()
                .map(|i| (i.label, i.insert))
                .collect()
        };
        let src = "x = ph2d.object(\"\")";
        let tudo = em(src, src.len() - 2);
        assert_eq!(tudo.len(), 3, "dentro das aspas vazias: todos os objectos");
        let src = "x = ph2d.object(\"Caixa G\")";
        assert_eq!(
            em(src, src.find(" G").unwrap() + 1),
            [("Caixa Grande".to_owned(), "Grande".to_owned())],
            "um nome com espaço: o resto a partir da palavra"
        );
        let src = "ph2d.spawn('')";
        assert_eq!(
            em(src, src.len() - 2),
            [("Bala".to_owned(), "Bala".to_owned())],
            "mestres"
        );
        let src = "print(\"\")";
        assert!(em(src, src.len() - 2).is_empty(), "noutra função, nada");
        let src = "ph2d.object(\"Porta\", \"";
        assert!(
            em(src, src.len()).is_empty(),
            "depois da string fechada, nada"
        );
    }
}
