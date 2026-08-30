//! **AGRUPAR e DESAGRUPAR** — o verbo que muda a árvore (ADR-0110).
//!
//! ⚠️ **Irmão de [`super`] por ASSUNTO e pelo tecto de 600 LOC da shell:** lá mora o que a ponte
//! MANTÉM (a identidade `path ⟺ entidade`, a ordem, o que a árvore esconde); aqui, o gesto que
//! reparenta. O grupo é o MESMO que os sprites usam — por isso ele aceita qualquer mistura de
//! tipos, e não há um tipo de nó especial.

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform, VecPathRef};

use super::{next_root_order, top_ancestor};

/// ⭐ **Os ancestrais de TOPO distintos que estas entidades representam** — a normalização que
/// agrupar faz: pegar num filho traz o grupo dele junto (aninhamento), não o filho solto.
///
/// ⚠️ Extraída do [`group_entities`] porque ela tem **dois** leitores agora: o próprio verbo, e
/// quem lhe dá o NOME (`crate::hier_group`), que precisa de contar os membros antes de o chamar.
/// *Dois caminhos do mesmo grupo são um membro só* — um `Group 2` sobre uma coisa só seria mentira
/// no primeiro sítio que o artista lê, e contar os `members` crus daria exactamente isso.
pub(crate) fn top_members(sim: &SimWorld, members: &[u64]) -> Vec<Entity> {
    let mut tops: Vec<Entity> = Vec::new();
    for &bits in members {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let t = top_ancestor(sim, e);
        if !tops.contains(&t) {
            tops.push(t);
        }
    }
    tops
}

/// Agrupa as entidades `members` sob uma **entidade comum nova** (nome, `Transform`,
/// `RootOrder`), preservando a ordem. É o mesmo grupo que os sprites usam — por isso
/// ele aceita qualquer mistura de tipos (ADR-0110). Devolve o grupo, ou `None` se
/// sobrar menos de 2 ancestrais de topo distintos.
///
/// Agrupar normaliza para os ancestrais de topo: pegar um filho traz o grupo dele
/// junto (aninhamento), não o filho solto — a convenção de qualquer editor.
pub(crate) fn group_entities(sim: &mut SimWorld, members: &[u64], name: String) -> Option<u64> {
    let tops = top_members(sim, members);
    if tops.len() < 2 {
        return None;
    }
    let order = next_root_order(sim);
    // ⭐⭐⭐ **O GRUPO NASCE ENTRE OS FILHOS, e não na origem do mundo** (report do Enio,
    // 2026-08-30: *"o gizmo do objeto pai deveria nascer na posição entre os filhos, mas nasceu no
    // zero do mundo"*).
    //
    // Um grupo não desenha nada, então o gizmo dele **é** a pose dele. Com `Transform::default()`
    // essa pose era `(0, 0)` — o artista agrupava duas formas ao pé uma da outra e a alça aparecia
    // longe, muitas vezes fora do ecrã. ⚠️ E não é só estética: **arrastar** e **girar** o grupo
    // usam essa pose, então girar acontecia em torno de um ponto que não tem nada a ver com o
    // conteúdo.
    let centro = centro_dos_membros(sim, &tops);
    let group = sim
        .world_mut()
        .spawn((
            Transform {
                translation: centro,
                ..Transform::default()
            },
            Name::new(name),
            RootOrder(order),
        ))
        .id();
    // `Children` preserva a ordem de inserção → os membros entram na ordem de z.
    for t in tops {
        if let Ok(mut e) = sim.world_mut().get_entity_mut(t) {
            e.remove::<RootOrder>();
            e.insert(ChildOf(group));
        }
        // ⚠️⚠️ **A COMPENSAÇÃO, e sem ela agrupar MOVIA tudo.** Um topo é uma raiz, logo o
        // `translation` dele **é** a posição no mundo; depois do `ChildOf` ele passa a ser LOCAL ao
        // grupo. Como o grupo nasce sem rotação nem escala, a composição é uma soma pura — subtrair
        // o centro devolve exactamente a mesma posição no mundo. *Um verbo de organização que
        // desloca o desenho não é um verbo de organização.*
        if let Some(mut tr) = sim.world_mut().get_mut::<Transform>(t) {
            tr.translation -= centro;
        }
    }
    Some(group.to_bits())
}

/// O ponto **entre** os membros: a média das poses de topo deles.
///
/// ⚠️ **A média das POSES, e não o centro das caixas de desenho.** A pose é a âncora que o artista
/// já vê e arrasta em cada objecto, existe para sprite, forma e objecto vazio por igual, e não
/// precisa de resolver geometria nenhuma — o centro da caixa exigiria o documento vectorial, e
/// daria um ponto que **muda** quando um filho é editado sem se ter movido.
///
/// Um membro sem `Transform` não entra na conta (ele também não é propagado).
fn centro_dos_membros(sim: &SimWorld, tops: &[Entity]) -> ph2d_core::Vec2 {
    let mut soma = ph2d_core::Vec2::ZERO;
    let mut n = 0u32;
    for &t in tops {
        if let Some(tr) = sim.world().get::<Transform>(t) {
            soma += tr.translation;
            n += 1;
        }
    }
    if n == 0 {
        return ph2d_core::Vec2::ZERO;
    }
    #[allow(clippy::cast_precision_loss)] // n <= contagem de objectos de topo da cena
    {
        soma / n as f32
    }
}

/// Dissolve os grupos de topo tocados por `members`. Um "grupo" aqui é o que a
/// árvore chama de grupo: uma entidade **sem geometria própria** (nem `VecPathRef`
/// nem sprite) que tem filhos. Um sprite com filhos é um pai, não um grupo — e
/// dissolvê-lo apagaria um objeto. Devolve quantos grupos sumiram.
pub(crate) fn ungroup_entities(sim: &mut SimWorld, members: &[u64]) -> usize {
    let mut tops: Vec<Entity> = Vec::new();
    for &bits in members {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        // ⭐⭐⭐ **O PRÓPRIO GRUPO conta, e não contava** — achado por gate, 2026-08-30.
        //
        // A condição era `t != e`, ou seja: só dissolvia o grupo de quem estivesse **dentro** dele.
        // Isso bastava enquanto o único chamador era o `Ctrl+Shift+G`, que passa CAMINHOS; o verbo
        // da Hierarquia passa a **selecção**, e depois de agrupar a selecção **é o grupo** ⇒
        // *Ungroup* logo a seguir a *Group* era um no-op que dizia *"nada na selecção está dentro
        // de um grupo"*. O gesto mais natural do artista era o único que não funcionava.
        //
        // ⚠️ **E não basta apagar o `t != e`:** um grupo ANINHADO tem por ancestral de topo o grupo
        // de FORA, então subir dissolveria o pai em vez do que foi clicado. ⇒ quem já é um grupo
        // responde por si; quem não é, sobe.
        let alvo = if is_plain_group(sim, e) {
            e
        } else {
            top_ancestor(sim, e)
        };
        if !tops.contains(&alvo) && is_plain_group(sim, alvo) {
            tops.push(alvo);
        }
    }
    let mut order = next_root_order(sim);
    for g in &tops {
        let parent = sim.world().get::<ChildOf>(*g).map(|c| c.parent());
        // ⚠️⚠️ **A METADE INVERSA DA COMPENSAÇÃO, e ela é obrigatória desde que o grupo nasce
        // CENTRADO.** Enquanto o grupo estava sempre na origem isto era somar zero e ninguém dava
        // por ele; com a pose no meio dos filhos, dissolver sem a devolver deslocaria o desenho
        // inteiro de `-centro`. *Uma cura num sentido que não é aplicada no inverso é meia cura*, e
        // o inverso aqui é o gesto que o artista usa para verificar o primeiro.
        let pose = sim
            .world()
            .get::<Transform>(*g)
            .map_or(ph2d_core::Vec2::ZERO, |t| t.translation);
        let kids: Vec<Entity> = sim
            .world()
            .get::<ph2d_ecs::Children>(*g)
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default();
        for k in kids {
            if let Some(mut tr) = sim.world_mut().get_mut::<Transform>(k) {
                tr.translation += pose;
            }
            if let Ok(mut e) = sim.world_mut().get_entity_mut(k) {
                match parent {
                    Some(p) => {
                        e.insert(ChildOf(p));
                    }
                    None => {
                        e.remove::<ChildOf>();
                        e.insert(RootOrder(order));
                        order = order.saturating_add(1);
                    }
                }
            }
        }
        if let Ok(e) = sim.world_mut().get_entity_mut(*g) {
            e.despawn();
        }
    }
    tops.len()
}

/// A entidade é um grupo puro: sem geometria própria, mas com filhos. Um sprite ou
/// um path com filhos NÃO é um grupo — dissolvê-lo apagaria um objeto.
fn is_plain_group(sim: &SimWorld, e: Entity) -> bool {
    let w = sim.world();
    w.get::<VecPathRef>(e).is_none()
        && w.get::<ph2d_render::Sprite>(e).is_none()
        && w.get::<ph2d_ecs::Children>(e)
            .is_some_and(|c| !c.is_empty())
}

#[cfg(test)]
#[path = "vec_entities_group_tests.rs"]
mod tests;
