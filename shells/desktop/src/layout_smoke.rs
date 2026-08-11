//! **A cena do AUTO LAYOUT** — `PH2D_BUILD_SMOKE=50` (plano UI/UX W2, ADR-0153).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e NÃO arma o fluxo** — a cicatriz que o `impasto_smoke` do Painter
//! prega: um smoke que arma o estado por baixo do pano pula justamente a costura que existe para
//! provar. As duas molduras nascem SEM `VecLayout`, e é o artista que escolhe *Row* no painel. O
//! que a cena constrói é o que a W2 **não** é sobre: as formas e o parentesco.
//!
//! # A pergunta desta cena é UMA, e é de olho
//!
//! *Seis formas largadas em desordem viram uma barra de ferramentas quando a moldura empilha — e
//! voltam a obedecer sozinhas quando ela muda de tamanho.*
//!
//! O que ela monta, e por quê:
//! - **duas molduras IDÊNTICAS**, lado a lado, com o mesmo conteúdo: a de cima é a que o artista
//!   arma, a de baixo é o **CONTROLE** que nunca flui. É o par deste repo — uma diferença que
//!   aparece nas duas não é do layout;
//! - **seis filhos em DESORDEM** de propósito (alturas e vãos irregulares): uma fila que já
//!   nascesse arrumada não deixaria ver o *Row* fazer nada;
//! - o **quarto** deles é o **ESPAÇADOR** (o retângulo apagado): é nele que o `Grow` se vê, porque
//!   ele é o que come a folga e empurra os dois últimos para o fim.

use ph2d_ecs::{ChildOf, Entity, VecFrame};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// O centro vertical de cada moldura. A de cima é a do roteiro; a de baixo é o controle.
const FRAME_Y: [f64; 2] = [2.6, -2.6];
/// Meia-largura e meia-altura da moldura — larga o bastante para sobrar folga com seis filhos.
const HALF: [f64; 2] = [4.5, 1.3];
/// Os seis filhos: `(meia-largura, meia-altura, dx, dy, cor)`.
///
/// ⚠️ `dx`/`dy` são a DESORDEM inicial, medida a partir do canto da moldura. Eles existem para o
/// passo 1 do roteiro ter o que consertar — uma fila já alinhada tornaria o *Row* invisível.
type Kid = (f64, f64, f64, f64, [u8; 3]);
const KIDS: &[Kid] = &[
    (0.55, 0.42, 0.9, 0.35, [90, 140, 210]),
    (0.55, 0.42, 2.6, -0.55, [90, 140, 210]),
    (0.55, 0.42, 1.7, 0.6, [90, 140, 210]),
    // O ESPAÇADOR — apagado, e é ele que o passo 4 faz crescer.
    (0.35, 0.42, 5.2, -0.2, [70, 75, 85]),
    (0.55, 0.42, 7.1, 0.5, [235, 200, 120]),
    (0.55, 0.42, 6.2, -0.6, [235, 200, 120]),
];

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O parentesco só depois do `sync` — é ele que dá entidade a cada caminho.
        6 => adopt(app),
        7 => announce(app),
        _ => {}
    }
}

/// Os caminhos. Por moldura: os SEIS filhos primeiro, a moldura por último (o fundo do card — a
/// mesma ordem de pilha que a cena da W0 usa, e pela mesma razão).
fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let s = &mut gfx.vec_scene;
    for &cy in &FRAME_Y {
        let (left, bottom) = (-HALF[0], cy - HALF[1]);
        for &(hw, hh, dx, dy, rgb) in KIDS {
            let (x, y) = (left + dx, bottom + HALF[1] + dy);
            s.push_path(tint(rectangle([x - hw, y - hh], [x + hw, y + hh]), rgb));
        }
        s.push_path(tint(
            rectangle([-HALF[0], cy - HALF[1]], [HALF[0], cy + HALF[1]]),
            [48, 48, 56],
        ));
    }
}

/// Pendura os seis filhos em cada moldura e marca as duas como molduras.
///
/// ⚠️ **Nenhuma recebe `VecLayout`**: o fluxo é o que o artista arma, e é a costura inteira desta
/// wave. Uma cena que já o trouxesse ligado provaria o passe e não o produto.
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    let per = KIDS.len() + 1;
    if ids.len() < per * 2 {
        return;
    }
    for i in 0..2 {
        let base = i * per;
        let Some(&fb) = app.vec_entities.get(&ids[base + KIDS.len()]) else {
            continue;
        };
        let frame = Entity::from_bits(fb);
        if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(frame) {
            e.insert(VecFrame { clip: false });
        }
        for k in 0..KIDS.len() {
            let Some(&kb) = app.vec_entities.get(&ids[base + k]) else {
                continue;
            };
            if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(Entity::from_bits(kb)) {
                e.insert(ChildOf(frame));
            }
        }
    }
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let content: f64 = KIDS.iter().map(|k| k.0 * 2.0).sum();
    let slack = HALF[0] * 2.0 - content;
    eprintln!(
        "[layout] cena montada: {} formas, 2 molduras IDENTICAS — a de CIMA e' a do roteiro, a de \
         BAIXO e' o CONTROLE e nao pode mudar em passo nenhum.",
        gfx.vec_scene.paths().len()
    );
    eprintln!(
        "[layout] cada moldura mede {:.1} de largura e os seis filhos somam {content:.1} — sobra \
         {slack:.1} de FOLGA, que e' o que o Grow do espacador vai comer.",
        HALF[0] * 2.0
    );
    eprintln!("[layout] AS MOLDURAS NASCEM SEM FLUXO — e' voce que o liga.");
    eprintln!("[layout] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique no fundo escuro da moldura de CIMA. Aparecem as secoes **Frame** e");
    eprintln!("     **Layout**. Em Layout, a Direction esta' em **Off** e so' essa fileira");
    eprintln!("     aparece — vao, recuo e alinhamento nao sao pintados, porque sobre uma moldura");
    eprintln!("     que nao empilha eles nao mudariam um pixel.");
    eprintln!("  2. Direction -> **Row**. ⚠️ Os seis filhos AVULSOS viram uma FILA na hora. As");
    eprintln!("     outras fileiras (Gap, Padding, Align, Distribute) aparecem com o fluxo.");
    eprintln!("  3. Arraste **Gap** e **Padding**: a fila abre e recua ao vivo. Ponha Align em");
    eprintln!("     **Center** e a fila centra na travessa.");
    eprintln!("  4. ⚠️ **O GROW**: clique no retangulo APAGADO (o 4o). A secao Layout mostra");
    eprintln!("     'Grow'/'Shrink'. Digite **1** em Grow (Enter). Ele engorda ate' comer a folga");
    eprintln!("     e EMPURRA os dois amarelos para o fim da barra — e' a barra de ferramentas.");
    eprintln!(
        "  5. ⚠️ **REDIMENSIONE a moldura**: selecione-a e arraste uma alca de canto. A fila"
    );
    eprintln!("     RECOMPOE ao vivo, e o espacador continua a comer o que sobra.");
    eprintln!("  6. ⚠️ **ARRASTAR DENTRO DO FLUXO E' REORDENAR**: pegue um dos botoes azuis e");
    eprintln!("     arraste-o para a direita, passando o vizinho. Ele NAO fica onde voce largou —");
    eprintln!("     ele TROCA de lugar na fila. Um Ctrl+Z desfaz a troca.");
    eprintln!("  7. Direction -> **Column** e depois **Wrap**. Na coluna a fila empilha para");
    eprintln!("     BAIXO a partir do topo; no Wrap aparece o segundo campo de vao ('Cross'),");
    eprintln!("     que nao existe nos outros dois — la' nao ha entre o que ele ficar.");
    eprintln!("  7b. ⚠️ **AINDA EM WRAP, faca-o QUEBRAR**: suba o **Gap** ate' os seis nao");
    eprintln!("      caberem numa faixa (por volta de 0,6) — nascem DUAS. Agora mexa no");
    eprintln!("      **Align**: o BLOCO das duas faixas encosta em cima (Start), centra");
    eprintln!("      (Center) ou desce (End). Antes desta wave elas ficavam sempre ESPALHADAS");
    eprintln!("      pela moldura, e 'Start' nao significava comeco.");
    eprintln!("  8. Direction -> **Off**. Os filhos VOLTAM para a desordem em que nasceram: a");
    eprintln!("     posicao deles nunca foi escrita, so' derivada.");
    eprintln!("  9. ⚠️ **O CONTROLE**: a moldura de BAIXO tem de estar exactamente como comecou,");
    eprintln!("     em todos os passos acima.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena contém a FOLGA que o roteiro promete.** Sem sobra o `Grow` não teria o que comer,
    /// e o passo 4 mandaria o artista olhar um número que não faz nada.
    #[test]
    fn the_frame_has_slack_for_the_spacer_to_eat() {
        let content: f64 = KIDS.iter().map(|k| k.0 * 2.0).sum();
        let width = HALF[0] * 2.0;
        assert!(
            content < width,
            "os filhos ({content}) nao cabem na moldura ({width}) — nao ha folga"
        );
        assert!(
            width - content > 1.0,
            "a folga ({}) e' pequena demais para o Grow ser visivel",
            width - content
        );
    }

    /// **E os filhos nascem em DESORDEM.** Uma fila já alinhada tornaria o passo 2 invisível — o
    /// artista escolheria *Row* e não veria nada acontecer.
    #[test]
    fn the_children_start_out_of_order() {
        let ys: Vec<f64> = KIDS.iter().map(|k| k.3).collect();
        let spread = ys.iter().cloned().fold(f64::MIN, f64::max)
            - ys.iter().cloned().fold(f64::MAX, f64::min);
        assert!(
            spread > 0.5,
            "as alturas mal diferem ({spread}) — ja' e' uma fila"
        );
        // E os `dx` não são monotónicos: a ordem visual não é a ordem da árvore.
        let xs: Vec<f64> = KIDS.iter().map(|k| k.2).collect();
        assert!(
            xs.windows(2).any(|w| w[1] < w[0]),
            "os filhos ja' estao em ordem da esquerda para a direita: {xs:?}"
        );
    }

    /// **O espaçador é o mais ESTREITO** — é como o artista o encontra sem que a cena lhe diga
    /// qual é (a mensagem chama-lhe "o apagado", e essas duas coisas têm de concordar).
    #[test]
    fn the_spacer_is_the_narrowest_and_the_dimmest() {
        let spacer = KIDS[3];
        assert!(
            KIDS.iter().all(|k| k.0 >= spacer.0),
            "o espacador nao e' o mais estreito"
        );
        let lum = |c: [u8; 3]| u32::from(c[0]) + u32::from(c[1]) + u32::from(c[2]);
        assert!(
            KIDS.iter().all(|k| lum(k.4) >= lum(spacer.4)),
            "o espacador nao e' o mais apagado — a mensagem chama-lhe 'o apagado'"
        );
    }

    /// **A SONDA: a cena é de facto disposta, pela porta do PRODUTO.**
    ///
    /// ⚠️ Os três gates acima medem os NÚMEROS que eu escrevi; este mede o que o passe FAZ com
    /// eles. É a diferença que a política de smoke deste repo cobra — *"a sonda headless roda
    /// antes de a mensagem ser escrita"* —, porque uma cena pode ter folga, desordem e um
    /// espaçador estreito e mesmo assim não produzir fila nenhuma (uma moldura que o coletor
    /// recusa, um filho sem caminho, uma bbox degenerada).
    ///
    /// O oráculo é o que o passo 2 do roteiro promete de OLHO: **os seis centros passam a estar em
    /// ordem crescente de `x` e todos à mesma altura** — que é o que "uma fila" significa, e o que
    /// a desordem inicial não tem.
    #[test]
    fn the_scene_actually_lays_out_into_a_queue() {
        use ph2d_ecs::{LayoutDir, SimWorld, VecLayout};
        use ph2d_vec_render::LiveGeometry;
        use ph2d_vec_scene::{VecScene, VecXforms};

        let (mut sim, scene, map, frame, kids) = staged();
        sim.world_mut().entity_mut(frame).insert(VecLayout {
            dir: LayoutDir::Row,
            ..VecLayout::default()
        });
        let mut live = LiveGeometry::new();
        let mut pass = crate::layout_live::LayoutLive::default();
        pass.recook(
            &scene,
            &sim,
            &map,
            &VecXforms::default(),
            &mut live,
            crate::vec_bindings::TokenCtx::factory(),
        );

        let slots = pass
            .slots_of(frame)
            .expect("a moldura armada tem de ter sido disposta");
        assert_eq!(slots.kids.len(), KIDS.len(), "todos os filhos colocados");
        assert_eq!(
            slots.reading,
            crate::layout_live::Reading::RowX,
            "o Row le-se numa fila ao longo de X"
        );
        for w in slots.kids.windows(2) {
            let c = |b: &crate::layout_live::Box2| (b.0[0] + b.1[0]) * 0.5;
            assert!(
                c(&w[0].1) < c(&w[1].1),
                "os centros nao ficaram em ordem crescente: {:?}",
                slots.kids.iter().map(|k| k.1).collect::<Vec<_>>()
            );
        }
        // E as caixas desenhadas ficam todas à MESMA altura — a desordem em `y` desapareceu.
        let tops: Vec<f64> = kids
            .iter()
            .filter_map(|id| live.get(id))
            .filter_map(|items| {
                items
                    .iter()
                    .flat_map(|p| p.verts.iter().map(|v| v.anchor[1]))
                    .fold(None::<f64>, |a, y| Some(a.map_or(y, |a| a.max(y))))
            })
            .collect();
        assert_eq!(tops.len(), KIDS.len(), "todo filho foi redesenhado");
        let spread = tops.iter().cloned().fold(f64::MIN, f64::max)
            - tops.iter().cloned().fold(f64::MAX, f64::min);
        assert!(
            spread < 1e-6,
            "os topos ainda diferem ({spread}) — nao e' fila"
        );
        let _ = SimWorld::default;
        let _: fn() -> VecScene = VecScene::new;
    }

    /// **E o GROW come a folga** — o passo 4 do roteiro, medido.
    #[test]
    fn the_spacer_eats_the_slack_when_it_grows() {
        use ph2d_ecs::{LayoutDir, VecLayout, VecLayoutItem};
        use ph2d_vec_render::LiveGeometry;
        use ph2d_vec_scene::VecXforms;

        let width_of = |live: &LiveGeometry, id| -> f64 {
            let items = live.get(&id).expect("o espacador foi redesenhado");
            let xs: Vec<f64> = items
                .iter()
                .flat_map(|p| p.verts.iter().map(|v| v.anchor[0]))
                .collect();
            xs.iter().cloned().fold(f64::MIN, f64::max)
                - xs.iter().cloned().fold(f64::MAX, f64::min)
        };

        let (mut sim, scene, map, frame, kids) = staged();
        sim.world_mut().entity_mut(frame).insert(VecLayout {
            dir: LayoutDir::Row,
            ..VecLayout::default()
        });
        let spacer = ph2d_ecs::Entity::from_bits(map[&kids[3]]);
        sim.world_mut().entity_mut(spacer).insert(VecLayoutItem {
            grow: 1.0,
            ..VecLayoutItem::default()
        });
        let mut live = LiveGeometry::new();
        let mut pass = crate::layout_live::LayoutLive::default();
        pass.recook(
            &scene,
            &sim,
            &map,
            &VecXforms::default(),
            &mut live,
            crate::vec_bindings::TokenCtx::factory(),
        );

        let grown = width_of(&live, kids[3]);
        let authored = KIDS[3].0 * 2.0;
        let content: f64 = KIDS.iter().map(|k| k.0 * 2.0).sum();
        let slack = HALF[0] * 2.0 - content;
        assert!(
            (grown - (authored + slack)).abs() < 0.05,
            "o espacador devia comer a folga inteira ({authored} + {slack}), e mede {grown}"
        );
    }

    /// A cena montada e sincronizada: devolve `(sim, scene, map, moldura de cima, os 6 filhos)`.
    fn staged() -> (
        ph2d_ecs::SimWorld,
        ph2d_vec_scene::VecScene,
        crate::vec_entities::VecEntityMap,
        ph2d_ecs::Entity,
        Vec<ph2d_vec_scene::VecPathId>,
    ) {
        let mut sim = ph2d_ecs::SimWorld::default();
        let mut scene = ph2d_vec_scene::VecScene::new();
        let mut map = crate::vec_entities::VecEntityMap::new();
        // A MESMA geometria da cena — a de cima apenas.
        let cy = FRAME_Y[0];
        let (left, bottom) = (-HALF[0], cy - HALF[1]);
        let kids: Vec<ph2d_vec_scene::VecPathId> = KIDS
            .iter()
            .map(|&(hw, hh, dx, dy, rgb)| {
                let (x, y) = (left + dx, bottom + HALF[1] + dy);
                scene.push_path(tint(rectangle([x - hw, y - hh], [x + hw, y + hh]), rgb))
            })
            .collect();
        let frame_id = scene.push_path(tint(
            rectangle([-HALF[0], cy - HALF[1]], [HALF[0], cy + HALF[1]]),
            [48, 48, 56],
        ));
        crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
        let frame = ph2d_ecs::Entity::from_bits(map[&frame_id]);
        sim.world_mut()
            .entity_mut(frame)
            .insert(ph2d_ecs::VecFrame { clip: false });
        for id in &kids {
            let e = ph2d_ecs::Entity::from_bits(map[id]);
            sim.world_mut().entity_mut(e).insert(ChildOf(frame));
        }
        (sim, scene, map, frame, kids)
    }

    /// **As duas molduras não se sobrepõem** — o controle tem de ser julgável ao lado.
    #[test]
    fn the_control_frame_does_not_overlap_the_first() {
        let gap = (FRAME_Y[0] - FRAME_Y[1]).abs() - HALF[1] * 2.0;
        assert!(gap > 0.2, "as duas molduras encostam ou sobrepoem ({gap})");
    }
}
