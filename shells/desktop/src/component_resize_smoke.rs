//! **A cena do MESTRE REDIMENSIONADO** — `PH2D_BUILD_SMOKE=55`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não arma modificador nenhum** — a cicatriz que o `impasto_smoke`
//! prega. Quem arrasta a alça, e quem carrega no Ctrl no meio do arrasto, é o artista.
//!
//! # A pergunta desta cena é UMA, e é de olho
//!
//! *Redimensionando o MESTRE, a cópia se comporta exactamente como ele: a quina que fica imóvel
//! nele fica imóvel nela.*
//!
//! O que ela monta, e por quê:
//! - um **MESTRE** de duas peças (um corpo e uma barra dentro), porque um componente de uma peça
//!   só não exercita a sub-árvore — e a sub-árvore é metade do desenho;
//! - **DUAS cópias vivas** ao lado, já ligadas. Duas e não uma: um seguimento que confundisse os
//!   instantâneos moveria as duas o mesmo tanto, e com uma só isso é indistinguível do certo;
//! - **pinos SOLTOS** em cada quina inferior-esquerda (laranja) e em cada centro (verde). Eles não
//!   são filhos de ninguém, então não viajam: são a régua. Sem pino a pergunta *"o que ficou
//!   parado?"* depende de o artista se lembrar de onde a borda estava;
//! - um **quadrado CINZA solto**, o **CONTROLE**: nunca é arrastado nem obedece a ninguém, então
//!   tem de ficar exactamente onde nasceu. Uma diferença que apareça nele não é do seguimento.

use ph2d_ecs::{Entity, Transform, VecComponentMain, VecInstance};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, ellipse, rectangle};

/// O corpo do MESTRE: `(x0, y0, x1, y1)` em unidades de mundo. Ele É a caixa de conteúdo — a barra
/// mora dentro dele.
const MAIN: [f64; 4] = [-9.0, -3.0, -1.0, 3.0];
/// A barra dentro do mestre — a segunda peça, para a sub-árvore entrar no desenho.
const BAR: [f64; 4] = [-7.5, 0.5, -2.5, 2.0];
/// Onde cada cópia é centrada, em x. A caixa delas é a do mestre, e o suporte nasce daí.
const COPIES_X: [f64; 2] = [4.0, 14.0];
/// O CONTROLE: um quadrado solto, longe, que nunca participa de nada.
const CONTROL: [f64; 4] = [-9.0, -8.2, -7.6, -6.8];
/// O raio de um pino. Pequeno de propósito: ele marca um PONTO, não uma região.
const PIN: f64 = 0.16;

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

/// A meia-largura e a meia-altura da caixa do mestre — a medida que o suporte de cada cópia herda.
fn half() -> [f64; 2] {
    [(MAIN[2] - MAIN[0]) * 0.5, (MAIN[3] - MAIN[1]) * 0.5]
}

/// O centro de mundo de cada cópia (e do mestre, no índice 0 desta régua não entra).
fn copy_centre(k: usize) -> [f64; 2] {
    [COPIES_X[k], (MAIN[1] + MAIN[3]) * 0.5]
}

/// A quina inferior-esquerda que o pino laranja marca, para o mestre e para cada cópia.
fn sw_pins() -> Vec<[f64; 2]> {
    let [hx, hy] = half();
    let mut v = vec![[MAIN[0], MAIN[1]]];
    for k in 0..COPIES_X.len() {
        let c = copy_centre(k);
        v.push([c[0] - hx, c[1] - hy]);
    }
    v
}

/// O centro que o pino verde marca, para o mestre e para cada cópia.
fn centre_pins() -> Vec<[f64; 2]> {
    let mut v = vec![[(MAIN[0] + MAIN[2]) * 0.5, (MAIN[1] + MAIN[3]) * 0.5]];
    for k in 0..COPIES_X.len() {
        v.push(copy_centre(k));
    }
    v
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O parentesco e os componentes só depois do `sync` — é ele que dá entidade a cada caminho.
        6 => arm(app),
        7 => announce(app),
        _ => {}
    }
}

/// **Os caminhos desta cena — porta única.**
///
/// ⚠️ Os gates abaixo medem a geometria que ela de facto empurra, e não as constantes que a
/// descrevem: comparar as consts entre si é a asserção que o compilador dobra.
fn paths() -> Vec<VecPath> {
    let [hx, hy] = half();
    let mut v = vec![
        // 0: o corpo do mestre. 1: a barra dentro dele.
        tint(
            rectangle([MAIN[0], MAIN[1]], [MAIN[2], MAIN[3]]),
            [58, 96, 168],
        ),
        tint(
            rectangle([BAR[0], BAR[1]], [BAR[2], BAR[3]]),
            [235, 200, 120],
        ),
    ];
    // 2..: o SUPORTE de cada cópia — a caixa do mestre, no sítio dela. Ele não é a arte: é o que dá
    // à cópia caixa de gizmo e alvo de clique (ver `vec_component_edit`).
    for k in 0..COPIES_X.len() {
        let c = copy_centre(k);
        v.push(tint(
            rectangle([c[0] - hx, c[1] - hy], [c[0] + hx, c[1] + hy]),
            [46, 46, 54],
        ));
    }
    // Os pinos DEPOIS das formas, para desenharem por cima delas.
    for p in sw_pins() {
        v.push(tint(ellipse(p, PIN, PIN), [235, 120, 90]));
    }
    for p in centre_pins() {
        v.push(tint(ellipse(p, PIN, PIN), [120, 220, 150]));
    }
    v.push(tint(
        rectangle([CONTROL[0], CONTROL[1]], [CONTROL[2], CONTROL[3]]),
        [80, 82, 92],
    ));
    v
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for p in paths() {
        gfx.vec_scene.push_path(p);
    }
}

/// Promove o mestre, pendura a barra nele e liga as duas cópias — o estado que o artista teria
/// depois de "Create" e dois "Place".
fn arm(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    if ids.len() < 2 + COPIES_X.len() {
        return;
    }
    let (Some(&mb), Some(&bb)) = (app.vec_entities.get(&ids[0]), app.vec_entities.get(&ids[1]))
    else {
        return;
    };
    let main = Entity::from_bits(mb);
    if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(main) {
        e.insert(VecComponentMain);
    }
    // ⚠️ Pela PORTA, não por um `ChildOf` cru: o `settle_origins` já correu, então as duas formas
    // carregam a própria translação e prender uma à outra as somaria — a barra aterrava 5 unidades
    // à esquerda do corpo, fora dele.
    crate::vec_transform::reparent_keeping_world(&mut gfx.sim, Entity::from_bits(bb), main);
    for id in ids.iter().skip(2).take(COPIES_X.len()) {
        let Some(&cb) = app.vec_entities.get(id) else {
            continue;
        };
        if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(Entity::from_bits(cb)) {
            let t = e.get::<Transform>().copied().unwrap_or_default();
            e.insert((t, VecInstance::new(ids[0])));
        }
    }
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let sw = sw_pins();
    eprintln!(
        "[copy-resize] cena montada: {} formas — um mestre de duas pecas com pino de CANTO em \
         ({:.1}, {:.1}), {} copias vivas, e um controle.",
        gfx.vec_scene.paths().len(),
        sw[0][0],
        sw[0][1],
        COPIES_X.len()
    );
    eprintln!("[copy-resize] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique no MESTRE (a forma azul da esquerda). O gizmo aparece com as alcas.");
    eprintln!("  2. Arraste a alca do canto SUPERIOR-DIREITO do mestre. ⚠️ O pino LARANJA dele");
    eprintln!("     fica IMOVEL — e' o canto oposto, a ancora por default. E **o pino LARANJA de");
    eprintln!("     CADA COPIA tem de ficar imovel tambem**: elas crescem para o mesmo lado que o");
    eprintln!("     mestre. E' a pergunta inteira desta cena.");
    eprintln!(
        "  3. Desfaca (Ctrl+Z). Agora arraste a alca do canto INFERIOR-ESQUERDO. ⚠️ Agora e'"
    );
    eprintln!("     a quina SUPERIOR-DIREITA que fica parada, no mestre **e em cada copia** — o");
    eprintln!("     crescimento nao pode sair espelhado.");
    eprintln!("  4. Desfaca. Arraste qualquer alca com **Ctrl premido**. ⚠️ Agora e' o pino VERDE");
    eprintln!("     (o centro) que fica parado, no mestre e em cada copia: as duas bordas andam.");
    eprintln!("  5. Desfaca. **MOVA** o mestre (arraste-o pelo meio). ⚠️ As copias NAO podem se");
    eprintln!("     mexer — herdar a FORMA e nao o LUGAR e' a lei que faz um componente ser util.");
    eprintln!("  6. ⚠️ **O CONTROLE**: o quadrado CINZA em baixo a esquerda nunca foi arrastado e");
    eprintln!("     nao obedece a ninguem. Se ele se mexeu, o que voce viu nao foi o seguimento.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A caixa de mundo de um dos caminhos que a cena empurra.
    fn bbox(i: usize) -> ([f64; 2], [f64; 2]) {
        let p = &paths()[i];
        let mut lo = [f64::MAX; 2];
        let mut hi = [f64::MIN; 2];
        for v in p.verts_all() {
            for a in 0..2 {
                lo[a] = lo[a].min(v.anchor[a]);
                hi[a] = hi[a].max(v.anchor[a]);
            }
        }
        (lo, hi)
    }

    /// **Cada suporte tem a MEDIDA do mestre** — medido na geometria, não nas consts.
    ///
    /// ⚠️ Um suporte de outro tamanho deixaria a arte derivada a boiar fora dele desde o frame
    /// zero, e o artista julgaria o seguimento por uma régua que já nasceu torta.
    #[test]
    fn every_support_is_the_size_of_the_master() {
        let (mlo, mhi) = bbox(0);
        for k in 0..COPIES_X.len() {
            let (lo, hi) = bbox(2 + k);
            assert!(
                (hi[0] - lo[0] - (mhi[0] - mlo[0])).abs() < 1e-9
                    && (hi[1] - lo[1] - (mhi[1] - mlo[1])).abs() < 1e-9,
                "o suporte {k} nao tem a medida do mestre: {:?} contra {:?}",
                [hi[0] - lo[0], hi[1] - lo[1]],
                [mhi[0] - mlo[0], mhi[1] - mlo[1]]
            );
        }
    }

    /// **Os pinos laranja estão nas quinas inferiores-esquerdas que o roteiro nomeia.**
    #[test]
    fn the_orange_pins_sit_on_the_bottom_left_corners() {
        let n = 2 + COPIES_X.len();
        let boxes: Vec<_> = std::iter::once(bbox(0))
            .chain((0..COPIES_X.len()).map(|k| bbox(2 + k)))
            .collect();
        for (k, (lo, _)) in boxes.iter().enumerate() {
            let (plo, phi) = bbox(n + k);
            let c = [(plo[0] + phi[0]) * 0.5, (plo[1] + phi[1]) * 0.5];
            assert!(
                (c[0] - lo[0]).abs() < 1e-9 && (c[1] - lo[1]).abs() < 1e-9,
                "o pino laranja {k} nao esta' na quina inferior-esquerda: {c:?} contra {lo:?}"
            );
        }
    }

    /// **A barra está DENTRO do corpo** — se ela transbordasse, a caixa de conteúdo do mestre
    /// deixaria de ser o corpo e todo pino desta cena marcaria o sítio errado.
    #[test]
    fn the_bar_is_inside_the_body() {
        let (mlo, mhi) = bbox(0);
        let (blo, bhi) = bbox(1);
        assert!(
            blo[0] >= mlo[0] && blo[1] >= mlo[1] && bhi[0] <= mhi[0] && bhi[1] <= mhi[1],
            "a barra transborda o corpo: {blo:?}..{bhi:?} contra {mlo:?}..{mhi:?}"
        );
    }

    /// **Os dois pinos são DISTINGUÍVEIS na tela** — dois pontos da mesma cor fariam o roteiro
    /// nomear uma coisa que o artista não consegue apontar.
    #[test]
    fn the_two_pin_colours_are_told_apart() {
        let ps = paths();
        let n = 2 + COPIES_X.len();
        assert_ne!(ps[n].fill, ps[n + COPIES_X.len() + 1].fill);
    }

    /// **A arte derivada nasce EM CIMA do suporte.**
    ///
    /// ⚠️ Este é o gate que de-risca a cena inteira. O clique e a caixa de gizmo de uma cópia leem
    /// a geometria GUARDADA (o suporte); o que se vê é derivado. Se os dois nascerem desalinhados,
    /// o artista aponta para a arte e seleciona outra coisa — foi exactamente a regressão que
    /// matou a tentativa anterior desta wave, e um smoke montado assim julgaria o seguimento por
    /// uma cena que já estava errada antes de ele correr.
    #[test]
    fn the_drawn_copy_lands_on_its_own_support() {
        let mut sim = ph2d_ecs::SimWorld::default();
        let mut scene = ph2d_vec_scene::VecScene::new();
        let mut map = crate::vec_entities::VecEntityMap::new();
        for p in paths() {
            scene.push_path(p);
        }
        let ids: Vec<u64> = scene.paths().iter().map(|p| p.id).collect();
        crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
        let main = Entity::from_bits(map[&ids[0]]);
        sim.world_mut().entity_mut(main).insert(VecComponentMain);
        for id in ids.iter().skip(2).take(COPIES_X.len()) {
            sim.world_mut()
                .entity_mut(Entity::from_bits(map[id]))
                .insert(VecInstance::new(ids[0]));
        }
        // ⚠️ **O `settle_origins` é premissa, não arrumação**, e a ORDEM contra o parentesco é a
        // metade que importa — é a mesma sequência do app, onde o passe corre a cada frame e o
        // `arm` da cena só chega no frame 6. O produtor tira a TRANSLAÇÃO do mestre e põe a da
        // cópia; com todo pivô na identidade essas duas são zero e a cópia desenha-se **em cima do
        // mestre**. Sem o chamar aqui, a fixture descreve um mundo que o produto não tem.
        crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);
        crate::vec_transform::reparent_keeping_world(
            &mut sim,
            Entity::from_bits(map[&ids[1]]),
            main,
        );
        let xf = crate::vec_transform::build(&sim, &map);
        let mut live = crate::instance_live::InstanceLive::default();
        live.recook(&scene, &sim, &map, &xf);
        for (k, id) in ids.iter().skip(2).take(COPIES_X.len()).enumerate() {
            let items = live.live().get(id).expect("a cópia desenha");
            let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
            for it in items {
                for v in it.verts_all() {
                    for a in 0..2 {
                        lo[a] = lo[a].min(v.anchor[a]);
                        hi[a] = hi[a].max(v.anchor[a]);
                    }
                }
            }
            let (slo, shi) = bbox(2 + k);
            assert!(
                (lo[0] - slo[0]).abs() < 1e-6
                    && (lo[1] - slo[1]).abs() < 1e-6
                    && (hi[0] - shi[0]).abs() < 1e-6
                    && (hi[1] - shi[1]).abs() < 1e-6,
                "a arte da cópia {k} nao nasce em cima do suporte dela: {lo:?}..{hi:?} contra \
                 {slo:?}..{shi:?}"
            );
        }
    }

    /// **O controle está LONGE de tudo** — um controle que uma alça arrastada alcance não responde
    /// à pergunta que ele existe para responder.
    #[test]
    fn the_control_is_far_from_the_action() {
        let (mlo, _) = bbox(0);
        let (_, ghi) = bbox(paths().len() - 1);
        assert!(
            ghi[1] < mlo[1],
            "o controle ({:.1}) encosta na faixa do mestre ({:.1})",
            ghi[1],
            mlo[1]
        );
    }
}
