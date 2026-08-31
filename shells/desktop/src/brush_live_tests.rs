//! Os gates da resolução da arte dos pincéis (plano 36, W3).

use super::*;
use ph2d_vec_scene::{BrushStroke, Rgba8, StrokePaint, StrokeSpec, VecPath, VecVertex};

fn quadrado(x: f64) -> Vec<VecVertex> {
    [[x, 0.0], [x + 1.0, 0.0], [x + 1.0, 1.0], [x, 1.0]]
        .map(VecVertex::corner)
        .to_vec()
}

/// Uma cena com a ARTE e uma forma cujo traço é um pincel que a nomeia.
fn cena(aponta_para_si: bool) -> (VecScene, VecPathId, VecPathId) {
    let mut scene = VecScene::default();
    let arte = scene.push_path(VecPath {
        verts: quadrado(0.0),
        closed: true,
        ..VecPath::default()
    });
    let hospedeira = scene.push_path(VecPath {
        verts: quadrado(5.0),
        closed: true,
        ..VecPath::default()
    });
    let alvo = if aponta_para_si { hospedeira } else { arte };
    if let Some(p) = scene.path_mut(hospedeira) {
        let mut s = StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 0.5);
        s.paint = StrokePaint::Brush(Box::new(BrushStroke {
            art: Some(alvo),
            ..BrushStroke::default()
        }));
        p.stroke = Some(s);
    }
    (scene, hospedeira, arte)
}

/// ⭐⭐ **A ARTE de um pincel é resolvida, e endereçada pela forma HOSPEDEIRA.**
#[test]
fn the_brush_art_resolves_keyed_by_its_host() {
    let (scene, hospedeira, _) = cena(false);
    let mapa = resolve(&scene, &|id| vec![id]);
    assert!(
        mapa.contains_key(&hospedeira),
        "a arte do pincel nao foi resolvida para a forma hospedeira"
    );
    // CONTROLO: a forma-ARTE não é chave — a chave é quem PINTA, não quem é pintado. Trocá-las
    // faria o desenho procurar a arte pelo id errado e cair sempre na cor de recurso.
    assert_eq!(mapa.len(), 1, "o mapa tem uma entrada por HOSPEDEIRA");
}

/// ⛔⛔ **UMA FORMA NÃO PODE SER O PRÓPRIO PINCEL.**
///
/// Desenhá-la exigiria as cópias, as cópias exigiriam a arte, e a arte seria ela. ⚠️ **O sintoma não
/// seria um erro**: seria o app a parar. É a mesma recusa PURA que o padrão-forma já tem.
#[test]
fn a_shape_can_never_be_its_own_brush() {
    let (scene, hospedeira, _) = cena(true);
    let mapa = resolve(&scene, &|id| vec![id]);
    assert!(
        !mapa.contains_key(&hospedeira),
        "uma forma resolveu-se como o proprio pincel - o desenho entraria em recursao"
    );
    // CONTROLO: com a arte a apontar para OUTRA forma, ela resolve — senão este gate ficaria verde
    // sobre uma resolução que nunca devolve nada.
    assert!(resolve(&cena(false).0, &|id| vec![id]).contains_key(&hospedeira));
}

/// ⚠️ **A arte entra COZIDA, não como foi digitada.**
///
/// Um motivo com quina viva ou com pilha de efeitos tem de se repetir como **parece**, não como foi
/// autorado — a mesma lei que a arte-forma de um padrão já obedece.
#[test]
fn the_art_enters_cooked_not_as_authored() {
    let (mut scene, hospedeira, arte) = cena(false);
    // Uma quina viva na arte: o cozido ganha vértices que a fonte não tem.
    let crus = scene.path(arte).map(|p| p.verts.len()).unwrap_or(0);
    if let Some(p) = scene.path_mut(arte) {
        for v in &mut p.verts {
            v.corner_radius = 0.25;
        }
    }
    let mapa = resolve(&scene, &|id| vec![id]);
    let resolvida = mapa.get(&hospedeira).expect("resolve");
    assert!(
        resolvida[0].verts.len() > crus,
        "a arte entrou AUTORADA ({} vertices, os mesmos da fonte) - um motivo com quina viva \
         repetir-se-ia com a quina afiada",
        resolvida[0].verts.len()
    );
}

/// ⚠️ **Uma cena SEM pincéis não paga nada** — nem uma entrada.
#[test]
fn a_scene_without_brushes_costs_nothing() {
    let mut scene = VecScene::default();
    scene.push_path(VecPath {
        verts: quadrado(0.0),
        closed: true,
        stroke: Some(StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 0.5)),
        ..VecPath::default()
    });
    assert!(resolve(&scene, &|id| vec![id]).is_empty());
}

/// ⭐⭐⭐ **A ARTE DE UM PINCEL PODE SER UM GRUPO, e a recusa de ciclo é sobre PERTENÇA.**
///
/// Report do Enio (2026-08-30) — ele pediu-o para a estampa, e o pincel é a mesma metade noutra
/// tinta. O documento endereça a arte por um `VecPathId` e um grupo **não tem um**: o que muda é a
/// **resolução**, e ela passa pela porta que a estampa já usa
/// ([`crate::texture_pattern_live::art_members`]).
///
/// # As duas metades
///
/// **Expandir**: um id que pertence a um grupo traz o grupo INTEIRO.
///
/// **Recusar**: e a recusa deixou de ser `art == host` — com um grupo, o anfitrião pode ser um
/// **membro** da arte. Desenhá-lo exigiria as cópias, as cópias exigiriam a arte, e a arte seria
/// ele. ⚠️ *O sintoma não seria um erro: seria o app a parar.*
///
/// ⚠️ **A segunda metade é o CONTROLO da primeira**: uma expansão que devolvesse sempre tudo
/// passaria a primeira e reprovaria esta.
#[test]
fn a_group_is_a_brush_art_and_the_cycle_refusal_is_about_membership() {
    let (scene, hospedeira, arte) = cena(false);
    // Um irmão da arte: os dois formam o grupo.
    let mut scene = scene;
    let irmao = scene.push_path(VecPath {
        verts: quadrado(2.0),
        closed: true,
        ..VecPath::default()
    });
    let grupo_da_arte = move |id: VecPathId| {
        if id == arte || id == irmao {
            vec![arte, irmao]
        } else {
            vec![id]
        }
    };
    let mapa = resolve(&scene, &grupo_da_arte);
    assert_eq!(
        mapa.get(&hospedeira).map(Vec::len),
        Some(2),
        "a arte do pincel nao expandiu o GRUPO - so' o caminho apontado chegou ao desenho"
    );
    // ⚠️ CONTROLO: sem a expansão, o mesmo pedido dá UM — a fixtura contém o fenómeno.
    assert_eq!(
        resolve(&scene, &|id| vec![id])
            .get(&hospedeira)
            .map(Vec::len),
        Some(1)
    );
    // ⛔ E o grupo que CONTÉM o anfitrião é recusado inteiro: é o ciclo que pararia o app.
    let grupo_com_o_host = move |id: VecPathId| {
        if id == arte || id == hospedeira {
            vec![arte, hospedeira]
        } else {
            vec![id]
        }
    };
    assert!(
        !resolve(&scene, &grupo_com_o_host).contains_key(&hospedeira),
        "um grupo que contem o proprio anfitriao foi aceite - desenha-lo exige as copias, e as \
         copias exigem a arte: o app PARA"
    );
}
