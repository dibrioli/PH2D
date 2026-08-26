//! **O host do Morph vivo** — o `t` animável (a forma única entre duas outras).
//!
//! Espelho do [`crate::connector_live`], e irmão do [`crate::blend_live`]. A diferença entre os
//! dois irmãos é o que cada um põe na CENA: o blend põe o *spine* (invisível) e desenha os N passos
//! como overlay virtual; o morph põe **a forma**, porque é a forma que o artista quer — para
//! pintar, exportar e, sobretudo, **animar**.
//!
//! Ninguém desenha um morph: move-se uma fonte, ou mexe-se no `t`, e a forma se refaz. A geometria
//! é **função pura da relação**, re-cozida a cada frame.
//!
//! # O `Plan` é CACHEADO aqui, e no blend não é — a diferença não é gosto
//!
//! O `blend_live::cook_links` monta um [`Plan`] por frame e ninguém reclamou: o blend só re-coze
//! quando o artista mexe em alguma coisa. No morph o `t` muda **todo frame** — é essa a feature —,
//! e sem cache a busca de fase (256×256) rodaria por frame: os **5,9 ms** que o `Plan` foi
//! inventado para matar, de volta, agora dentro do orçamento de quadro.
//!
//! A correspondência é função do **par**, não do `t`. Então a chave do cache é a **geometria em
//! MUNDO das duas fontes** — geometria *e* pose, não uma delas: sem a pose, o plano descreveria a
//! forma onde ela **estava** (a mesma lição da chave de sessão do Shape Builder).

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, Name, SimWorld, Transform, VecMorph, VecMorphMachine};
use ph2d_vec_blend::Plan;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// O [`Plan`] de cada morph, guardado enquanto a relação não muda.
///
/// `BTreeMap`, não `HashMap`: a ordem de iteração de uma `HashMap` não é determinística, e o
/// repo a proíbe por isso — o mesmo motivo pelo qual o `blend_live` usa `BTreeMap`.
///
/// Runtime-only (como o `SideCache` do conector): não é documento, não entra no save nem no undo —
/// é derivável das fontes a qualquer momento. Perdê-lo custa uma busca, não um bit de trabalho.
#[derive(Default)]
pub(crate) struct MorphPlans {
    live: BTreeMap<u64, Cached>,
    /// Quantos planos foram **construídos** (não reusados) desde o início.
    ///
    /// É o que deixa o gate do cache ser **exato** em vez de um cronômetro (que mede a máquina, não
    /// o código) — a mesma escolha do `SEARCH_COUNT` do motor, que é `cfg(test)` e privado à crate
    /// dele, e por isso não alcança daqui.
    #[cfg(test)]
    pub(super) builds: usize,
}

/// O plano de um morph, **com as duas formas que o produziram**.
///
/// Guardar as formas é o que torna a invalidação honesta: a pergunta *"o plano ainda vale?"* é
/// *"as fontes ainda são estas?"*, e ela se responde comparando o que se tem com o que se vê. Um
/// contador de versão, ou um flag `dirty` posto pelos sítios que mexem em forma, seria uma 2ª
/// fonte de verdade — e quem esquecesse de o marcar teria um morph que **para de acompanhar** a
/// fonte, em silêncio.
struct Cached {
    a: VecPath,
    b: VecPath,
    plan: Plan,
}

impl MorphPlans {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// O plano para `(a, b)`, montado só se as formas mudaram desde o último frame.
    fn plan_for(&mut self, id: VecPathId, a: &VecPath, b: &VecPath) -> Option<&Plan> {
        let hit = self.live.get(&id).is_some_and(|c| c.a == *a && c.b == *b);
        if !hit {
            let plan = Plan::new(a, b)?;
            #[cfg(test)]
            {
                self.builds += 1;
            }
            self.live.insert(
                id,
                Cached {
                    a: a.clone(),
                    b: b.clone(),
                    plan,
                },
            );
        }
        self.live.get(&id).map(|c| &c.plan)
    }
}

/// A forma assada no MUNDO (ADR-0111: as fontes podem ter poses diferentes, e a transição vive num
/// frame só). `None` se a forma sumiu.
fn world(scene: &VecScene, xforms: &VecXforms, id: u64) -> Option<VecPath> {
    let mut p = scene.paths().iter().find(|p| p.id == id)?.clone();
    bake_xform(&mut p, &xform_of(xforms, id));
    Some(p)
}

/// **A forma-fonte na pose LOCAL dela** — o referencial do conjunto que a contém.
///
/// ⚠️ **É a irmã de [`world`], e a diferença é o afim que ela assa:** aquela leva a forma ao MUNDO
/// (a cadeia inteira de pais); esta assa só o `Transform` do próprio, que — para um membro de um
/// conjunto de estados — é exactamente a pose relativa ao conjunto, porque o conjunto é o pai dele.
///
/// ⛔ Um membro que não seja filho directo do conjunto daria uma pose relativa ao pai ERRADO. Não
/// pode acontecer: quem os reparenta é o [`crate::morph_set::upkeep`], que os põe directamente
/// debaixo do conjunto — e há gate sobre isso.
fn local(scene: &VecScene, sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecPath> {
    let mut p = scene.paths().iter().find(|p| p.id == id)?.clone();
    let t = map
        .get(&id)
        .map(|&bits| Entity::from_bits(bits))
        .and_then(|e| sim.world().get::<Transform>(e).copied())
        .unwrap_or(Transform::IDENTITY);
    bake_xform(&mut p, &crate::vec_transform::xform_of_transform(t));
    Some(p)
}

/// **O re-cook de todo frame.** Para cada entidade com um [`VecMorph`]: resolve as duas fontes no
/// MUNDO, avalia o plano em `t`, e escreve a forma **em lugar**.
///
/// Roda DEPOIS de `vec_entities::sync` (a entidade existe) e depois de `vec_transform::build` (os
/// afins das fontes já são os deste frame), e ANTES do render — o mesmo lugar do
/// `connector_live::recook`.
pub(crate) fn recook(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    xforms: &VecXforms,
    plans: &mut MorphPlans,
) {
    let morphs: Vec<(VecPathId, Entity, VecMorph)> = map
        .iter()
        .filter_map(|(&id, &bits)| {
            let e = Entity::from_bits(bits);
            let m = sim.world().get::<VecMorph>(e)?.clone();
            Some((id, e, m))
        })
        .collect();
    if morphs.is_empty() {
        plans.live.clear();
        return;
    }
    plans
        .live
        .retain(|id, _| morphs.iter().any(|(m, _, _)| m == id));

    for (id, entity, m) in morphs {
        // ⭐⭐ **UM CONJUNTO DE ESTADOS COZE NO REFERENCIAL DELE, e é isso que o torna ARRASTÁVEL**
        // (plano 32 W9; Enio, 2026-08-25: *"o objeto criado como pai tem que ser arrastável no
        // canvas como um objeto qualquer (…) e deve arrastar os filhos junto"*).
        //
        // ⚠️ **A diferença é UMA linha e o mecanismo é inteiro.** Um Morph autorado coze as fontes
        // em **MUNDO** e vive na identidade — se tivesse pose, a geometria de mundo levaria o afim
        // dele **outra vez** e a forma deslocar-se-ia duas vezes. Um conjunto coze as fontes nas
        // poses **LOCAIS** dos filhos (que são filhos DELE), então o que fica guardado é geometria
        // do referencial do conjunto, e o `Transform` dele aplica-se **uma** vez, como em qualquer
        // forma do documento.
        //
        // ⭐ **E o plano não se refaz ao arrastar:** as poses locais dos filhos não mudam quando o
        // pai anda, então `a` e `b` são os mesmos bytes e o cache do `plan_for` acerta. Arrastar um
        // conjunto de nove estados custa o que custa arrastar um rectângulo.
        let is_set = sim.world().get::<VecMorphMachine>(entity).is_some();
        let src = |sid| {
            if is_set {
                local(scene, sim, map, sid)
            } else {
                world(scene, xforms, sid)
            }
        };
        // Uma fonte sumiu (apagada) ⇒ a forma **CONGELA** onde está. Apagar o morph junto
        // destruiria trabalho; deixá-lo cozer com uma fonte só o faria sumir da tela. Congelar é o
        // que o conector faz com uma ponta perdida, e é o único que preserva o desenho.
        let (Some(a), Some(b)) = (src(m.sources[0]), src(m.sources[1])) else {
            continue;
        };
        let Some(cooked) = plans.plan_for(id, &a, &b).map(|p| p.at(f64::from(m.t))) else {
            continue; // forma degenerada: sem plano, e nada a escrever
        };
        write_shape(scene, id, cooked);

        // O morph vive na IDENTIDADE: a geometria acima é MUNDO, e uma pose por cima a
        // deslocaria. É a mesma regra do conector — e é ela que torna o gizmo inócuo sobre ele.
        // Arrastar um morph não quer dizer nada: move-se uma FONTE.
        //
        // ⛔ **E é EXACTAMENTE por isto que um conjunto de estados escapa:** ele não é uma forma
        // derivada de duas fontes que o artista move, é um OBJECTO cujos estados são filhos dele.
        // Zerar a pose dele faria o arrasto voltar ao sítio no quadro seguinte — que é, palavra por
        // palavra, o mecanismo que tornava um Morph não-arrastável.
        if !is_set
            && sim
                .world()
                .get::<Transform>(entity)
                .is_some_and(|t| *t != Transform::IDENTITY)
            && let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity)
        {
            *t = Transform::IDENTITY;
        }
    }
}

/// Escreve a forma cozida no path `id`, **em lugar**.
///
/// Em lugar, e não um `push_path` novo: um path novo por frame daria um `VecPathId` novo por
/// frame, e a entidade, a seleção e o gizmo **piscariam**. (É a razão, palavra por palavra, do
/// `connector_live::write_route`.)
///
/// O estilo (fill/stroke) vem do plano — ele é **derivado**, interpolado em OKLab entre as duas
/// fontes. Recolorir o objeto morph não faria sentido: o frame seguinte o sobrescreveria.
/// Recolore-se uma FONTE, e a cor viaja.
fn write_shape(scene: &mut VecScene, id: VecPathId, cooked: VecPath) {
    let Some(p) = scene.path_mut(id) else {
        return;
    };
    p.verts = cooked.verts;
    p.subpaths = cooked.subpaths;
    p.closed = cooked.closed;
    p.fill_rule = cooked.fill_rule;
    p.fill = cooked.fill;
    p.stroke = cooked.stroke;
}

/// Pendura (ou atualiza) o [`VecMorph`] na entidade do path `id` — espelho de
/// `blend_live::attach`. Idempotente.
pub(crate) fn attach(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    morph: &VecMorph,
) -> bool {
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let entity = Entity::from_bits(bits);
    if sim.world().get::<VecMorph>(entity) == Some(morph) {
        return true;
    }
    let first = sim.world().get::<VecMorph>(entity).is_none();
    let Ok(mut e) = sim.world_mut().get_entity_mut(entity) else {
        return false;
    };
    e.insert(morph.clone());
    if first {
        // O nome que a Hierarquia mostra — é por ele que o artista acha o morph na árvore, e é por
        // ele que a TIMELINE o reencontra depois de um undo (o `wire_id` é hash do `Name`).
        e.insert(Name::new(format!("Morph {id}")));
    }
    true
}

/// Drena a fila `pending` (o morph recém-criado, esperando a entidade dele nascer no `sync`) —
/// espelho de `blend_live::upkeep`. Roda entre o `sync` e o [`recook`].
pub(crate) fn upkeep(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    pending: &mut Option<(VecPathId, VecMorph)>,
) {
    if let Some((id, morph)) = pending.as_ref() {
        let gone = !scene.paths().iter().any(|p| p.id == *id);
        if gone || attach(sim, map, *id, morph) {
            *pending = None;
        }
    }
}

/// **Cria um morph** entre as duas formas `a` e `b` (na ordem de z). Devolve o path novo (a forma
/// morfada, ainda vazia — o `recook` do frame a preenche) e o componente a pendurar.
///
/// As fontes **sobrevivem** (≠ booleana, que consome os operandos): o morph é não-destrutivo, e é
/// editando as fontes que o artista o dirige.
pub(crate) fn create(scene: &mut VecScene, a: VecPathId, b: VecPathId) -> (VecPathId, VecMorph) {
    // Nasce vazio de propósito: a geometria é DERIVADA, e inventá-la aqui seria uma 2ª porta para
    // a mesma pergunta que o `recook` responde todo frame.
    let id = scene.push_path(VecPath::default());
    (id, VecMorph::new(a, b))
}

#[cfg(test)]
#[path = "morph_live_tests.rs"]
mod tests;
