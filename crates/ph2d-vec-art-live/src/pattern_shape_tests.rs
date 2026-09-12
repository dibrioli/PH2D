//! ⭐⭐ **A ARTE QUE VEM DO DOCUMENTO** — os gates da `PatternSource::Shape` (plano 33, W7+).
//!
//! Irmão do [`super::texture_pattern_live_tests`], e o corte é por RESPONSABILIDADE: aquele mede o
//! MEMO (o que entra na chave, o que não re-assa, o handle estável, o tecto do atlas); este mede o
//! que é próprio de a arte ser uma FORMA da cena — a recusa do ciclo, o re-assado ao editá-la, e a
//! POSE dos membros quando ela é um grupo.
//!
//! ⚠️ Todos usam o assador INJECTADO: assar uma forma é render + readback de GPU, e cablá-lo faria
//! todo gate desta família depender de uma placa.

use super::tests::{scene_with, solo, still};
use super::*;
use ph2d_vec_scene::{PatternFill, Rgba8, VecPath, VecVertex};

/// ⛔⛔ **UMA FORMA NÃO PODE SER O PRÓPRIO PADRÃO** (plano 33, W7).
///
/// Assá-la exigiria desenhá-la, desenhá-la exigiria o ladrilho, e o ladrilho exigiria assá-la. ⚠️ E
/// o sintoma não seria um erro: seria o app a parar, ou um ladrilho de uma versão anterior de si
/// mesmo a cada quadro.
#[test]
fn a_pattern_whose_source_is_itself_is_refused() {
    let db = AssetDb::new();
    let (scene, path) = scene_with(PatternFill::new(
        PatternSource::Shape(0),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    // A forma da fixtura é a primeira da cena; apontar o padrão a ela é o ciclo.
    let mut f = PatternFill::new(
        PatternSource::Shape(path),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    );
    f.origin = [0.0, 0.0];
    let (mut ciclo, id) = scene_with(f);
    let _ = (&scene, path);
    let mut assou = false;
    let mut bake = |_| {
        assou = true;
        Some((2u32, 2, vec![9u8; 2 * 2 * 4]))
    };
    let mut live = TexturePatternLive::default();
    live.recook(
        &ciclo,
        &db,
        ImageQuality::Medium,
        &mut bake,
        &solo(),
        &still(),
    );
    assert!(
        !assou,
        "o assador foi chamado para uma forma que e' a propria fonte"
    );
    assert!(
        live.tiles()
            .get(&(id, ph2d_vec_render::PatternSlot::Fill))
            .is_none(),
        "o ciclo produziu ladrilho"
    );
    // CONTROLO: apontar a OUTRA forma assa.
    let outra = ciclo.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    if let Some(Paint::Pattern(p)) = ciclo.path_mut(id).and_then(|p| p.fill.as_mut()) {
        p.source = PatternSource::Shape(outra);
    }
    let mut bake2 = |_| Some((2u32, 2, vec![9u8; 2 * 2 * 4]));
    live.recook(
        &ciclo,
        &db,
        ImageQuality::Medium,
        &mut bake2,
        &solo(),
        &still(),
    );
    assert!(
        live.tiles()
            .get(&(id, ph2d_vec_render::PatternSlot::Fill))
            .is_some(),
        "a fonte valida nao assou"
    );
}

/// ⭐⭐ **EDITAR A FORMA-FONTE RE-ASSA o ladrilho** — é o *"pattern fills are dynamic"* do Figma, e é
/// o que separa *um preenchimento de imagem* de *um sistema de padrões*.
///
/// ⚠️ Sem a forma na chave, a `PatternSource::Shape(id)` seria estável e mexer nos nós da fonte não
/// mudaria a tela — o defeito EXACTO que o `FxKey` da crate irmã documenta.
#[test]
fn editing_the_source_shape_rebakes_the_tile() {
    let db = AssetDb::new();
    let mut scene = VecScene::default();
    let fonte = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let alvo = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(PatternFill::new(
            PatternSource::Shape(fonte),
            [4.0, 4.0],
            Rgba8::new(1, 2, 3, 255),
        )))),
        ..VecPath::default()
    });
    let mut n = 0usize;
    let mut live = TexturePatternLive::default();
    {
        let mut bake = |_| {
            n += 1;
            Some((2u32, 2, vec![9u8; 2 * 2 * 4]))
        };
        live.recook(
            &scene,
            &db,
            ImageQuality::Medium,
            &mut bake,
            &solo(),
            &still(),
        );
        live.recook(
            &scene,
            &db,
            ImageQuality::Medium,
            &mut bake,
            &solo(),
            &still(),
        );
    }
    assert_eq!(n, 1, "o memo re-assou o que nao mudou");
    assert!(
        live.tiles()
            .contains_key(&(alvo, ph2d_vec_render::PatternSlot::Fill))
    );
    // Mexer num NÓ da fonte tem de re-assar.
    if let Some(p) = scene.path_mut(fonte) {
        p.verts[2].anchor = [5.0, 5.0];
    }
    {
        let mut bake = |_| {
            n += 1;
            Some((2u32, 2, vec![9u8; 2 * 2 * 4]))
        };
        live.recook(
            &scene,
            &db,
            ImageQuality::Medium,
            &mut bake,
            &solo(),
            &still(),
        );
    }
    assert_eq!(
        n, 2,
        "editar a forma-fonte NAO re-assou - o padrao ficaria morto"
    );
}

/// ⭐⭐⭐ **MOVER UM MEMBRO DO GRUPO-ARTE RE-ASSA; ARRASTAR O GRUPO INTEIRO NÃO.**
///
/// Report do Enio (2026-08-30): *"ao mover os objetos do grupo que serve como shape, a pattern não
/// atualiza em tempo real"*.
///
/// # As duas metades, e porque nenhuma se mede sozinha
///
/// **A 1.ª é a cura.** Desde o [ADR-0110](../../../docs/architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md)
/// a geometria de um `VecPath` é **local** e quem a põe no mundo é o `Xform` que a shell publica ⇒
/// uma chave feita só de `VecPath`s **não muda quando o artista move um membro**, e o ladrilho fica
/// congelado *para sempre* (não é um atraso de um quadro: a chave nunca mais difere).
///
/// **A 2.ª é o preço de não a curar à bruta.** Pôr a pose CRUA na chave faria arrastar o grupo
/// re-assar — render + readback de GPU — a **cada quadro**, para produzir o mesmo desenho: o assado
/// põe a caixa da UNIÃO na origem do ladrilho, logo ele é invariante a mover o conjunto. É a mesma
/// razão pela qual o `origin` e o `angle` do padrão também ficam fora desta chave.
///
/// ⭐ **E as duas metades são o CONTROLO uma da outra.** Se a pose não chegasse à chave (uma
/// `art_pose` vazia, um `object_of` que devolvesse um membro só), a 2.ª metade ficaria verde por
/// **ausência** — e a 1.ª reprovaria. *Uma delas sozinha aprova o defeito que a outra mede.*
///
/// ⚠️ A fixtura é um grupo de **DOIS** caminhos de propósito: com um só, *"mover um membro"* e
/// *"arrastar o grupo"* são o mesmo gesto, e a 2.ª metade não afirmaria nada.
#[test]
fn moving_one_member_rebakes_and_dragging_the_whole_group_does_not() {
    let db = AssetDb::new();
    let mut scene = VecScene::default();
    let tri = |p: [f64; 2]| VecPath {
        verts: [p, [p[0] + 2.0, p[1]], [p[0] + 2.0, p[1] + 2.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    let a = scene.push_path(tri([0.0, 0.0]));
    let b = scene.push_path(tri([3.0, 0.0]));
    let alvo = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(PatternFill::new(
            PatternSource::Shape(a),
            [4.0, 4.0],
            Rgba8::new(1, 2, 3, 255),
        )))),
        ..VecPath::default()
    });
    // O grupo: tocar em `a` ou em `b` apanha os dois, que é a lei do `object_selection_for`.
    let grupo = move |id: VecPathId| {
        if id == a || id == b {
            vec![a, b]
        } else {
            vec![id]
        }
    };
    // As poses, como a shell as publica: só translação.
    let em = move |pa: [f64; 2], pb: [f64; 2]| {
        move |id: VecPathId| {
            let t = if id == a {
                pa
            } else if id == b {
                pb
            } else {
                return ph2d_vec_scene::Xform::IDENTITY;
            };
            ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, t[0], t[1]])
        }
    };

    let mut n = 0usize;
    let mut live = TexturePatternLive::default();
    let assar = |live: &mut TexturePatternLive,
                 n: &mut usize,
                 pose: &dyn Fn(VecPathId) -> ph2d_vec_scene::Xform| {
        let mut bake = |_| {
            *n += 1;
            Some((2u32, 2, vec![9u8; 2 * 2 * 4]))
        };
        live.recook(&scene, &db, ImageQuality::Medium, &mut bake, &grupo, pose);
    };

    assar(&mut live, &mut n, &em([0.0, 0.0], [3.0, 0.0]));
    assert_eq!(n, 1, "o primeiro assado nao aconteceu");
    assert!(
        live.tiles()
            .contains_key(&(alvo, ph2d_vec_render::PatternSlot::Fill)),
        "a fixtura nao produziu ladrilho nenhum - o resto deste gate nao mede nada"
    );

    // Nada se mexeu: nao re-assa. (O controlo de que este gate nao conta um assado por quadro.)
    assar(&mut live, &mut n, &em([0.0, 0.0], [3.0, 0.0]));
    assert_eq!(n, 1, "o memo re-assou o que nao mudou");

    // ⭐ O GRUPO INTEIRO arrastado `+[5,5]`: o desenho e' o mesmo, e nao se re-assa.
    assar(&mut live, &mut n, &em([5.0, 5.0], [8.0, 5.0]));
    assert_eq!(
        n, 1,
        "arrastar o grupo INTEIRO re-assou - o assado poe a caixa da uniao na origem do ladrilho, \
         entao o desenho e' o mesmo, e isto e' um render + readback de GPU por quadro de arrasto"
    );

    // ⭐⭐ UM MEMBRO mexeu-se em relacao ao outro: tem de re-assar. Este e' o report.
    assar(&mut live, &mut n, &em([5.0, 5.0], [9.0, 5.0]));
    assert_eq!(
        n, 2,
        "mover UM membro do grupo-arte NAO re-assou - o ladrilho fica com o desenho de antes para \
         sempre, que e' exactamente o report de 30/08"
    );
}
