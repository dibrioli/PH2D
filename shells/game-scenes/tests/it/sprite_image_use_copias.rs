//! ⭐⭐ **O report do dono 27/10** — Player: Repeated (1 × 1) → Still → Animated → Repeated mostrava 8 × 4. O mundo
//! estava certo (a memória devolvia o ladrilho), o Inspector mostrava METROS: uma cópia da folha do Player mede
//! 8 × 4 m. Lei (dono 27/10): Width/Height de uma repetida contam **cópias da imagem**; 1 × 1 = a imagem inteira
//! uma vez. Tudo pela porta do clique (`apply_edit`) e lido como o Inspector lê (`inspector_use`).

use super::*;

/// O que o Inspector mostra em Width/Height (`None` fora de Repeated).
fn copias(sim: &SimWorld, e: Entity) -> Option<[f32; 2]> {
    let (db, atlas) = biblioteca();
    match inspector_use(sim.world(), e, &db, &atlas, PPM).2 {
        InspectorDraw::Tiled { copies } => Some(copies),
        _ => None,
    }
}

/// O que o Inspector mostra em «Area» (`None` fora de Region).
fn area_px(sim: &SimWorld, e: Entity) -> Option<[u32; 2]> {
    let (db, atlas) = biblioteca();
    match inspector_use(sim.world(), e, &db, &atlas, PPM).2 {
        InspectorDraw::Region { px } => Some(px),
        _ => None,
    }
}

/// Os píxeis da imagem que `e` mostra.
fn imagem_px(sim: &SimWorld, e: Entity) -> [u32; 2] {
    let (db, atlas) = biblioteca();
    shown_facts(sim.world(), e, &db, &atlas, PPM)
        .expect("factos")
        .px
}

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    a.iter()
        .zip(b)
        .all(|(x, y)| (x - y).abs() <= 1e-4 * x.abs().max(1.0))
}

/// A sequência literal do dono.
#[test]
fn a_volta_do_dono_devolve_o_repeated_do_passo_1() {
    let mut sim = SimWorld::new();
    crate::comp::montar(&mut sim, 4);
    let e = por_nome(&mut sim, "Player");
    assert!(usa(&mut sim, e, InspectorImageUse::Repeated));
    let passo_1 = copias(&sim, e).expect("Width/Height à vista");
    assert!(
        perto(passo_1, [1.0, 1.0]),
        "Repeated nasce 1 × 1, viu {passo_1:?}"
    );
    let mundo_1 = completo(&sim, e);
    for to in [
        InspectorImageUse::Still,
        InspectorImageUse::Animated,
        InspectorImageUse::Repeated,
    ] {
        assert!(usa(&mut sim, e, to), "→ {to:?}");
    }
    assert_eq!(copias(&sim, e), Some(passo_1), "passo 4 ≠ passo 1");
    assert_eq!(
        completo(&sim, e),
        mundo_1,
        "o mundo do passo 4 ≠ o do passo 1"
    );
}

/// ⭐ Toda a família, os 4 usos (plano GEF 16): as 12 trocas (o caminho S→R→A→G→S→A→R→G→A→S→G→R→S passa por cada
/// par ordenado; G = Region), duas voltas, sobre o Player (nasce animado), a Platform Low (nasce repetida) e um
/// bloco com a bandeira (parada). Cada uso volta TAL E QUAL o deixámos; uma repetida nova mostra 1 × 1 e uma região
/// nova a imagem inteira; a Width e a área que o artista deu ficam com elas. A meio da 2.ª volta, o desfazer (o
/// mundo gravado antes do clique) e o projecto reaberto continuam a mesma lei.
#[test]
fn cada_troca_ida_e_volta_devolve_o_uso_como_estava() {
    use InspectorImageUse::{Animated as A, Region as G, Repeated as R, Still as S};
    let caminho = [R, A, G, S, A, R, G, A, S, G, R, S];
    let (db, atlas) = biblioteca();
    for (nome, bandeira) in [
        ("Player", false),
        ("Platform Low", false),
        ("Platform High", true),
    ] {
        let mut sim = SimWorld::new();
        crate::comp::montar(&mut sim, 4);
        let mut e = por_nome(&mut sim, nome);
        let mut deixado: BTreeMap<u8, Completo> = BTreeMap::new();
        let chave = |u: ImageUse| u as u8;
        if bandeira {
            // O bloco já repetia: a bandeira entra nele, e o Repeated guardado é esse bloco (não um novo 1 × 1).
            let flag = atlas[&ph2d_render::FLAG_SHEET_KEY];
            choose_library_image(&mut sim, &mut sem_placa, &db, &atlas, PPM, e, flag)
                .expect("escolhe");
            deixado.insert(chave(ImageUse::Repeated), completo(&sim, e));
            usa(&mut sim, e, S);
        }
        let inicio = image_use(sim.world(), e).expect("sprite");
        // A Platform Low parte de Repeated: o caminho começa no uso dela.
        let mut uso_agora = match inicio {
            ImageUse::Still => S,
            ImageUse::Repeated => R,
            ImageUse::Animated => A,
            ImageUse::Region => G,
        };
        let mut largura_dada = false;
        let mut area_dada = false;
        for volta in 0..2 {
            for (i, &to) in caminho.iter().enumerate() {
                if to == uso_agora {
                    continue;
                }
                let de = image_use(sim.world(), e).expect("sprite");
                deixado.insert(chave(de), completo(&sim, e));
                let antes = instantaneo(&mut sim);
                assert!(usa(&mut sim, e, to), "{nome} v{volta} {de:?} → {to:?}");
                if volta == 1 && i == 3 {
                    // O desfazer: o mundo gravado antes do clique volta igual, e a volta segue sobre ele.
                    let mut desfeito = reaberto(&antes);
                    let e2 = por_nome(&mut desfeito, nome);
                    assert_eq!(
                        completo(&desfeito, e2),
                        deixado[&chave(de)],
                        "{nome}: desfazer"
                    );
                    assert!(usa(&mut desfeito, e2, to));
                    assert_eq!(
                        completo(&desfeito, e2),
                        completo(&sim, e),
                        "{nome}: refazer"
                    );
                }
                if volta == 1 && i == 7 {
                    let gravado = instantaneo(&mut sim);
                    sim = reaberto(&gravado);
                    e = por_nome(&mut sim, nome);
                }
                let agora = image_use(sim.world(), e).expect("sprite");
                match deixado.get(&chave(agora)) {
                    Some(antes) => {
                        assert_eq!(
                            &completo(&sim, e),
                            antes,
                            "{nome} v{volta}: {agora:?} voltou diferente"
                        );
                    }
                    None if agora == ImageUse::Repeated => {
                        let c = copias(&sim, e).expect("Width/Height");
                        assert!(perto(c, [1.0, 1.0]), "{nome}: repetida nova {c:?}");
                    }
                    None if agora == ImageUse::Region => {
                        let px = area_px(&sim, e).expect("a área à vista");
                        assert_eq!(
                            px,
                            imagem_px(&sim, e),
                            "{nome}: região nova ≠ a imagem inteira"
                        );
                    }
                    None => {}
                }
                if agora == ImageUse::Region && !area_dada {
                    // O artista escolhe a 2.ª célula de cima numa grelha 4 × 2: é isso que tem de voltar.
                    let [w, h] = imagem_px(&sim, e);
                    let celula = [w / 4, 0, w / 4, h / 2];
                    assert!(apply_edit(
                        sim.world_mut(),
                        e,
                        &db,
                        &atlas,
                        PPM,
                        ImageUseEdit::SetRegion {
                            rect: celula,
                            grid: [4, 2]
                        }
                    ));
                    assert_eq!(
                        area_px(&sim, e),
                        Some([w / 4, h / 2]),
                        "{nome}: a área dada"
                    );
                    area_dada = true;
                }
                if agora == ImageUse::Repeated && !largura_dada {
                    // O artista dá-lhe 3 cópias de largura: é isso que tem de voltar.
                    assert!(apply_edit(
                        sim.world_mut(),
                        e,
                        &db,
                        &atlas,
                        PPM,
                        ImageUseEdit::Width(3.0)
                    ));
                    let c = copias(&sim, e).expect("Width/Height");
                    assert!(perto([c[0], 0.0], [3.0, 0.0]), "{nome}: Width 3 lê {c:?}");
                    largura_dada = true;
                }
                uso_agora = to;
            }
        }
    }
}
