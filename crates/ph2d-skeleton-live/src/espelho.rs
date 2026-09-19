//! ⭐⭐⭐ **ESPELHAR UM RAMO DO ESQUELETO** — construir o lado esquerdo a partir do direito.
//!
//! # ⚠️ A lei é uma CONJUGAÇÃO, e ela calcula-se à mão
//!
//! Seja `M` a reflexão do MUNDO na vertical `x = c` e `G` a reflexão que troca o sinal do `y`
//! **dentro do referencial de um osso**. Um osso que vai de `cabeça` a `ponta` tem de ficar a ir de
//! `M(cabeça)` a `M(ponta)` — e com determinante **positivo**, senão a pose deixa de ser
//! representável sem escala negativa e todo o resto do módulo (o comprimento sobre o `+X`, o gizmo,
//! o lado da dobra) passa a mentir.
//!
//! O referencial que faz as duas coisas é `W' = M ∘ W ∘ G`: o `G` fixa o eixo `+X` (logo a cabeça e
//! a ponta vão para onde se quer) e repõe o sinal do determinante.
//!
//! Daí sai tudo, e sem uma única constante escolhida:
//!
//! ```text
//! filho:  T' = G ∘ T ∘ G   ⇒   translação.y := −y · rotação := −r · escala := igual · skews := −
//! raiz:   T' = P⁻¹ ∘ M ∘ P ∘ T ∘ G   (derivada da cabeça e da ponta REFLECTIDAS, no referencial do pai)
//! ```
//!
//! E o CONTEÚDO de um osso vive no referencial dele, onde o `G` também manda:
//!
//! | o que | como espelha | porquê |
//! |---|---|---|
//! | `length` | **igual** | o `G` fixa o eixo `+X`, e o comprimento vive nele |
//! | `curve.inn[1]` / `out[1]` | **negado** | o arco é um desvio em `y`, e o `y` inverte |
//! | `curve.inn[0]` / `out[0]` | **igual** | eles medem-se **ao longo** do eixo |
//! | `BoneLimit { min, max }` | **`{ −max, −min }`** | sob `r ↦ −r` a faixa inverte e troca de ponta |
//! | `strength`, `segments` | **iguais** | não têm lado |
//!
//! # ⛔ O que NÃO viaja, e é uma decisão declarada
//!
//! A **âncora de IK** e o **osso inteligente** nomeiam **OUTROS OBJECTOS** da cena por identidade.
//! Copiá-los daria duas correntes a puxar o **mesmo losango** — o braço espelhado seguiria a mão do
//! original —, e *mirrorar uma referência a um objecto é uma segunda decisão que este verbo não pode
//! tomar sozinho* (qual alvo? uma cópia espelhada dele? o mesmo?). É a mesma lei que a cópia
//! profunda já aplica aos documentos possuídos.
//!
//! ⇒ os dois são **retirados** das cópias, e o artista carrega em *Add IK* / *Look At* no ramo novo.

use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, scene::ComponentRegistry};
use ph2d_skeleton_ecs::{Bone, BoneLimit, BoneRest};

/// ⭐⭐ **O NOME DO OUTRO LADO** — `None` quando o nome não declara lado nenhum.
///
/// ⚠️ **A tabela é uma LISTA e não um `replace` cego:** trocar todo `L` por `R` renomearia
/// `"Leg"` para `"Reg"`. O que se troca é um **marcador de lado**, e ele é um sufixo (`.L`, `_R`) ou
/// uma PALAVRA inteira (`Left`, `right`) — que é a forma que as quatro referências usam.
///
/// ⚠️ **`None` é uma resposta**, e o chamador trata-a: um osso chamado `"Bone 7"` não tem lado, e
/// inventar-lhe um seria escrever uma decisão do artista. Ali o nome da cópia sai do
/// `ph2d_unique_name`, como o de qualquer objecto novo.
#[must_use]
pub fn nome_espelhado(nome: &str) -> Option<String> {
    /// Os pares de SUFIXO, em ordem de comprimento decrescente — o mais específico primeiro, senão
    /// `_L` nunca é visto num nome acabado em `_Left`.
    const SUFIXOS: [(&str, &str); 8] = [
        (".Left", ".Right"),
        ("_Left", "_Right"),
        (".left", ".right"),
        ("_left", "_right"),
        (".L", ".R"),
        ("_L", "_R"),
        (".l", ".r"),
        ("_l", "_r"),
    ];
    for (a, b) in SUFIXOS {
        if let Some(base) = nome.strip_suffix(a) {
            return Some(format!("{base}{b}"));
        }
        if let Some(base) = nome.strip_suffix(b) {
            return Some(format!("{base}{a}"));
        }
    }
    // ⚠️ E a PALAVRA inteira, em qualquer posição — `"Left Arm"` é tão comum como `"Arm.L"`. A
    // troca é por *token* delimitado por espaço, senão `"Deleft"` viraria `"Deright"`.
    const PALAVRAS: [(&str, &str); 2] = [("Left", "Right"), ("left", "right")];
    for (a, b) in PALAVRAS {
        if nome.split(' ').any(|t| t == a) {
            return Some(troca_token(nome, a, b));
        }
        if nome.split(' ').any(|t| t == b) {
            return Some(troca_token(nome, b, a));
        }
    }
    None
}

fn troca_token(nome: &str, de: &str, para: &str) -> String {
    nome.split(' ')
        .map(|t| if t == de { para } else { t })
        .collect::<Vec<_>>()
        .join(" ")
}

/// ⭐⭐ **O EIXO do espelho** — a vertical que passa pela origem do osso **RAIZ** do esqueleto.
///
/// ⚠️ **Derivado do documento e não escolhido** (§0.0): num personagem a raiz é o quadril, e é
/// exactamente o `X = 0` da armadura que o Blender espelha. ⛔ Um número num campo seria uma
/// terceira coisa a manter coerente com a pose.
#[must_use]
pub fn eixo_do_esqueleto(sim: &SimWorld, osso: Entity) -> Option<f64> {
    let raiz = crate::esqueletos::raiz_do_osso(sim, osso);
    let m = ph2d_vec_entities::transform::xform_of_transform(
        ph2d_vec_entities::transform::world_transform(sim, raiz),
    );
    let o = m.apply([0.0, 0.0]);
    o[0].is_finite().then_some(o[0])
}

/// Reflecte um ponto do mundo na vertical `x = eixo`.
fn reflecte(p: [f64; 2], eixo: f64) -> [f64; 2] {
    [2.0f64.mul_add(eixo, -p[0]), p[1]]
}

/// ⭐⭐⭐ **ESPELHA o ramo que começa em `osso`** — devolve a raiz da cópia.
///
/// `None` quando `osso` não é osso, ou quando a pose do pai é singular (não há espaço local em que
/// pôr a cópia).
///
/// ⚠️ **A cópia nasce IRMÃ do original** (mesmo pai), que é o que faz o esqueleto continuar um só:
/// o braço esquerdo pendura-se onde o direito pendura.
pub fn espelha(sim: &mut SimWorld, registry: &ComponentRegistry, osso: Entity) -> Option<Entity> {
    // ⚠️ `?` e não um `if`: a pergunta é *«isto é osso?»*, e a resposta `None` é a mesma recusa que
    // as duas linhas seguintes já dão.
    sim.world().get::<Bone>(osso)?;
    let eixo = eixo_do_esqueleto(sim, osso)?;
    // ⚠️ **Os segmentos são fotografados ANTES da cópia**: depois dela o mundo tem o dobro dos
    // ossos, e procurar o original numa lista que já contém a cópia é como um censo se engana.
    let segs = crate::skin_live::bone_segments(sim);
    let pai = sim.world().get::<ChildOf>(osso).map(ChildOf::parent);
    let copia =
        ph2d_ecs::instantiate::deep_copy_subtree(sim.world_mut(), registry, osso, pai).ok()?;
    // ⚠️ **Pré-ordem**: a pose local de um filho deriva-se do pai **já espelhado**, e o mapa da
    // cópia profunda é ordenado por entidade e não pela árvore. ⇒ a ordem sai da ÁRVORE.
    let ordem = ordem_da_arvore(sim, osso);
    for original in ordem {
        let Some(&nova) = copia.entities.get(&original) else {
            continue;
        };
        if original == osso {
            raiz_espelhada(sim, original, nova, &segs, eixo);
        } else {
            filho_espelhado(sim, nova);
        }
        espelha_o_conteudo(sim, nova);
        remapeia_a_ponta_da_curva(sim, nova, &copia.stable_ids);
        renomeia(sim, nova);
        // ⛔ **A âncora e o osso inteligente NÃO viajam** — ver o cabeçalho: os dois nomeiam outros
        // objectos da cena, e duas correntes a puxar o mesmo losango é o defeito que isto evita.
        sim.world_mut()
            .entity_mut(nova)
            .remove::<ph2d_skeleton_ecs::IkGoal>();
        sim.world_mut()
            .entity_mut(nova)
            .remove::<ph2d_skeleton_ecs::SmartBone>();
    }
    Some(copia.root)
}

/// A sub-árvore em PRÉ-ORDEM (pai antes dos filhos), que é a ordem em que a pose se deriva.
fn ordem_da_arvore(sim: &SimWorld, raiz: Entity) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut pilha = vec![raiz];
    while let Some(e) = pilha.pop() {
        out.push(e);
        if let Some(f) = sim.world().get::<ph2d_ecs::Children>(e) {
            // ⚠️ Invertido, para o `pop` devolver os filhos na ordem em que o `Children` os lista —
            // a mesma ordem determinística que a cópia profunda visitou.
            let mut filhos: Vec<Entity> = f.iter().copied().collect();
            filhos.reverse();
            pilha.extend(filhos);
        }
    }
    out
}

/// A RAIZ do ramo: a pose sai da cabeça e da ponta **reflectidas**, lidas no referencial do pai.
fn raiz_espelhada(
    sim: &mut SimWorld,
    original: Entity,
    nova: Entity,
    segs: &[(u64, [f64; 2], [f64; 2])],
    eixo: f64,
) {
    let Some(&(_, a, b)) = segs.iter().find(|(x, _, _)| *x == original.to_bits()) else {
        return;
    };
    let pai_mundo = sim
        .world()
        .get::<ChildOf>(nova)
        .map(ChildOf::parent)
        .map_or(ph2d_vec_scene::Xform::IDENTITY, |p| {
            ph2d_vec_entities::transform::xform_of_transform(
                ph2d_vec_entities::transform::world_transform(sim, p),
            )
        });
    let Some(inv) = pai_mundo.inverse() else {
        return;
    };
    let ha = inv.apply(reflecte(a, eixo));
    let hb = inv.apply(reflecte(b, eixo));
    let d = [hb[0] - ha[0], hb[1] - ha[1]];
    let Some(mut t) = sim.world_mut().get_mut::<Transform>(nova) else {
        return;
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o `Transform` da casa é f32; a geometria do documento é f64"
    )]
    {
        t.translation = ph2d_core::Vec2::new(ha[0] as f32, ha[1] as f32);
        t.rotation = d[1].atan2(d[0]) as f32;
    }
    // ⚠️ **A escala e os cisalhamentos ficam como estão**: a reflexão é uma ISOMETRIA, logo não
    // muda tamanhos — e o `G` que repõe o determinante nega os cisalhamentos, que é o que a linha
    // seguinte faz (a mesma conta do filho, sobre os dois campos que sobram).
    t.skew_x = -t.skew_x;
    t.skew_y = -t.skew_y;
}

/// Um FILHO: a conjugação `G ∘ T ∘ G`, que em 2D são três sinais.
fn filho_espelhado(sim: &mut SimWorld, nova: Entity) {
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(nova) {
        t.translation.y = -t.translation.y;
        t.rotation = -t.rotation;
        t.skew_x = -t.skew_x;
        t.skew_y = -t.skew_y;
    }
}

/// O CONTEÚDO de um osso — o que vive no referencial dele, onde o `G` também manda. Ver a tabela no
/// cabeçalho.
fn espelha_o_conteudo(sim: &mut SimWorld, nova: Entity) {
    if let Some(mut b) = sim.world_mut().get_mut::<Bone>(nova) {
        b.curve.inn[1] = -b.curve.inn[1];
        b.curve.out[1] = -b.curve.out[1];
    }
    if let Some(mut l) = sim.world_mut().get_mut::<BoneLimit>(nova) {
        // ⚠️ **Troca E nega**: sob `r ↦ −r` o extremo mais horário passa a ser o mais
        // anti-horário. ⛔ Só negar deixaria `min > max`, e a lei trava a junta no CENTRO do que
        // estiver escrito — o cotovelo espelhado ficaria preso a meio caminho.
        let (min, max) = (l.min, l.max);
        l.min = -max;
        l.max = -min;
    }
    // ⭐ **E o REPOUSO acompanha a pose**: ele é a pose em que este osso repousa, logo espelha-se
    // com ela. ⚠️ Escrito da pose JÁ espelhada, e não conjugado à parte: duas contas para a mesma
    // coisa divergiriam no primeiro ajuste.
    if sim.world().get::<BoneRest>(nova).is_some()
        && let Some(t) = sim.world().get::<Transform>(nova).copied()
    {
        sim.world_mut().entity_mut(nova).insert(BoneRest::de(&t));
    }
}

/// ⛔⛔⛔ **O `curve_tip` nomeia um FILHO por IDENTIDADE, e a cópia profunda NÃO remapeia nada** —
/// o doc dela di-lo por escrito.
///
/// Sem esta linha, um osso que declara *«a ponta da minha curva é ESTE filho»* ficaria, na cópia, a
/// apontar para o filho do **ORIGINAL** — e o ramo espelhado arquearia a seguir a um osso do outro
/// lado do corpo, **em silêncio**. É o mesmo defeito que a cópia profunda de uma instância pagou
/// com o `VecPathRef`, e a cura é a mesma: o mapa `StableId → StableId` que a cópia devolve.
///
/// ⚠️ **Um id que não está no mapa é DEIXADO como está**, e não apagado: ele nomeia um osso **fora**
/// do ramo copiado, e ali a referência do original continua a ser a resposta certa. ⛔ A lei do
/// [`ph2d_skeleton_ecs::CurveTip::Bone`] já trata um id que não resolve como `Straight`.
fn remapeia_a_ponta_da_curva(
    sim: &mut SimWorld,
    nova: Entity,
    mapa: &std::collections::BTreeMap<u64, u64>,
) {
    let Some(mut b) = sim.world_mut().get_mut::<Bone>(nova) else {
        return;
    };
    if let ph2d_skeleton_ecs::CurveTip::Bone(id) = b.curve_tip
        && let Some(&novo) = mapa.get(&id.0)
    {
        b.curve_tip = ph2d_skeleton_ecs::CurveTip::Bone(ph2d_ecs::StableId(novo));
    }
}

/// O nome do outro lado, ou um nome único quando o original não declara lado.
fn renomeia(sim: &mut SimWorld, nova: Entity) {
    let atual = sim
        .world()
        .get::<Name>(nova)
        .map(|n| n.0.clone())
        .unwrap_or_default();
    // ⚠️ **A porta ÚNICA da unicidade** (`unique_name`) por cima das duas saídas: ela devolve o
    // nome pedido quando ele está livre e só numera quando não está. ⛔ Escrever o nome espelhado
    // sem passar por ela daria dois ossos com o MESMO nome — e a referência durável entre objectos
    // nesta casa é o NOME, logo a timeline passaria a animar os dois como se fossem um.
    let base = nome_espelhado(&atual).unwrap_or(atual);
    let novo = ph2d_unique_name::unique_name(sim, &base);
    sim.world_mut().entity_mut(nova).insert(Name::new(novo));
}

#[cfg(test)]
#[path = "espelho_tests.rs"]
mod tests;
