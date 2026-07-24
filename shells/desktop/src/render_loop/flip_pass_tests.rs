//! Testes headless do passe do Flip (separados pelo cap de LOC do HR-18).
//! Declarados pelo pai como irmão via `#[path]`, então `super` é
//! `render_loop::flip_pass`.
//!
//! O que eles pinam são os DOIS bugs do smoke do Enio — e nos dois a aparência
//! reduz-se a uma propriedade estrutural, testável sem GPU:
//! - **a ordem da op-list É a ordem de z** (o compositor compõe de baixo para cima),
//!   então "o fantasma aparece sobre a camada de baixo" = "o fantasma vem depois dela
//!   na lista";
//! - **o desenho amostrado é o do ciclo**, então "o Loop funciona" = "o quadro 20
//!   resolve para o desenho do quadro 4".

use super::*;
use ph2d_core::Vec2;
use ph2d_flip::{BlendMode, FlipStroke, Hold, KeyKind};

/// Um documento com 2 camadas: **BG** (uma chave, arte que cobre tudo) e **FG**
/// (chaves em 0 e 8, blend Multiply). É o demo, reduzido.
fn doc_bg_fg() -> FlipDoc {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("O");
    let obj = doc.object_mut(oid).unwrap();
    obj.fps = 12.0;
    let art = || {
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(0.0, 0.0));
        s.push_default(Vec2::new(1.0, 1.0));
        s
    };
    let bg = obj.add_layer("BG");
    let d = obj
        .insert_frame(bg, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    obj.drawing_mut(d).unwrap().strokes.push(art());

    let fg = obj.add_layer("FG");
    obj.layer_mut(fg).unwrap().blend = BlendMode::Multiply;
    for k in [0, 8] {
        let d = obj
            .insert_frame(fg, k, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        obj.drawing_mut(d).unwrap().strokes.push(art());
    }
    doc
}

/// O playhead pausado no quadro `f` a 12 fps.
fn at(f: i32) -> Playhead {
    let mut p = Playhead::new(1.0 / 12.0);
    p.pause();
    p.seek(f64::from(f) / 12.0);
    p
}

/// **O bug do 1º corte, pinado:** o fantasma de uma camada é uma FATIA DA PILHA,
/// logo abaixo dela — e portanto ACIMA das camadas de baixo. Desenhá-lo num
/// passe por baixo de tudo fazia o BG opaco engolir o fantasma do FG.
///
/// A op-list é composta de baixo para cima, então a ordem da lista É a ordem de
/// z: exigir `BG < ghost(FG) < FG` é exigir que o fantasma apareça sobre o BG.
#[test]
fn a_layers_ghost_sits_above_the_layers_below_it() {
    let doc = doc_bg_fg();
    let ph = at(8); // sobre a 2ª chave do FG → o desenho de 0 vira fantasma
    let (layers, _) = collect_layers(&doc, &ph, None, None, &[], Some(crate::render_loop::flip_pass_ghosts::GhostSources::default()));

    let kinds: Vec<&str> = layers
        .iter()
        .map(|l| if l.ghost.is_some() { "ghost" } else { "art" })
        .collect();
    assert_eq!(
        kinds,
        vec!["art", "ghost", "art"],
        "ordem de composicao (fundo para topo): BG, o fantasma do FG, o FG"
    );

    let ghost = &layers[1];
    // O fantasma compõe em NORMAL, opacidade 1 — o fade já está no alpha do
    // tint. Herdar o Multiply do FG tingiria o fantasma com a arte do BG.
    assert_eq!(ghost.blend, BlendMode::default().to_u8());
    assert!((ghost.opacity - 1.0).abs() < f32::EPSILON);
    assert!(ghost.ghost.is_some());
    // E as chaves do compositor são todas distintas (fatias não se sobrescrevem).
    let keys: Vec<u64> = layers.iter().map(|l| l.key).collect();
    let mut uniq = keys.clone();
    uniq.sort_unstable();
    uniq.dedup();
    assert_eq!(
        uniq.len(),
        keys.len(),
        "chaves de fatia colidiram: {keys:?}"
    );
}

/// **O 2º bug do smoke:** o render tem de amostrar **pelo CICLO**. Amostrando o
/// caminho cru, Loop e Ping-Pong não faziam absolutamente nada — o último desenho
/// simplesmente segurava para sempre ("extrapola o último quadro").
///
/// Aqui a camada tem 2 desenhos e um vão de 16 quadros; em Loop, o quadro 20 tem
/// de mostrar o MESMO desenho do quadro 4.
#[test]
fn the_render_samples_through_the_cycle() {
    let mut doc = doc_bg_fg();
    let oid = doc.objects().first().unwrap().id;
    let (fg, d0, d8) = {
        let obj = doc.object(oid).unwrap();
        let fg = obj.layers().last().unwrap().id;
        let l = obj.layer(fg).unwrap();
        (fg, l.drawing_at(0).unwrap(), l.drawing_at(8).unwrap())
    };
    // Exposição real na última chave → o vão fecha em 16 (8 + 8).
    let obj = doc.object_mut(oid).unwrap();
    assert!(obj.set_exposure(fg, 8, 8));
    obj.layer_mut(fg).unwrap().cycle = ph2d_flip::LayerCycle {
        pre: ph2d_flip::CycleMode::Loop,
        post: ph2d_flip::CycleMode::Loop,
    };

    // Sem ciclo, o quadro 20 estaria além de tudo (o cru seguraria d8 pra sempre).
    // Com Loop, ele volta ao quadro 4 → d0. E o 28 volta ao 12 → d8.
    let layer_drawing = |doc: &FlipDoc, f: i32| {
        let (layers, _) = collect_layers(doc, &at(f), None, None, &[], None);
        // A última fatia é a do FG (o BG vem primeiro; sem fantasmas aqui).
        layers.last().and_then(|l| {
            let (_, did) = l.cache_key;
            (did != u32::MAX).then_some(did)
        })
    };
    assert_eq!(layer_drawing(&doc, 4), Some(d0.0), "dentro do vão: d0");
    assert_eq!(layer_drawing(&doc, 20), Some(d0.0), "20 vira 4 (Loop): d0");
    assert_eq!(layer_drawing(&doc, 28), Some(d8.0), "28 vira 12 (Loop): d8");
}

/// Sem a tool Flip (o `ghosts` é `None`) a cena aparece limpa: nenhuma fatia de
/// fantasma. E durante o PLAY também não (o fantasma é ruído na reprodução).
#[test]
fn there_are_no_ghosts_without_the_tool_or_during_play() {
    let doc = doc_bg_fg();
    let (layers, _) = collect_layers(&doc, &at(8), None, None, &[], None);
    assert!(layers.iter().all(|l| l.ghost.is_none()), "tool inativa");

    let mut playing = at(8);
    playing.play();
    let (layers, _) = collect_layers(&doc, &playing, None, None, &[], Some(crate::render_loop::flip_pass_ghosts::GhostSources::default()));
    assert!(layers.iter().all(|l| l.ghost.is_none()), "durante o play");
}

/// Aplica um `world_to_clip` col-major a um ponto homogêneo (x, y, 0, 1).
fn apply4(m: &[[f32; 4]; 4], x: f32, y: f32) -> [f32; 4] {
    let v = [x, y, 0.0, 1.0];
    let mut out = [0.0f32; 4];
    for (row, o) in out.iter_mut().enumerate() {
        let mut s = 0.0;
        for (j, vj) in v.iter().enumerate() {
            s += m[j][row] * vj;
        }
        *o = s;
    }
    out
}

/// `fold_model` dobra o `model` LOCAL→mundo no `world_to_clip`: um ponto LOCAL
/// mapeia como se estivesse no mundo `model·local`, e a espessura escala pela
/// escala média do objeto.
#[test]
fn fold_model_translates_local_into_clip_and_scales_thickness() {
    // Base = clip identidade (o ponto de mundo vira ele mesmo), zoom 100 px/mundo.
    let base = CameraRaw::new(
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        [800.0, 600.0],
        100.0,
    );
    // Translação (10, 5): o local (3, 2) deve cair no clip do mundo (13, 7).
    let t = fold_model(&base, &Xform([1.0, 0.0, 0.0, 1.0, 10.0, 5.0]));
    let p = apply4(&t.world_to_clip, 3.0, 2.0);
    assert!(
        (p[0] - 13.0).abs() < 1e-4 && (p[1] - 7.0).abs() < 1e-4,
        "{p:?}"
    );
    assert!(
        (t.px_per_world - 100.0).abs() < 1e-4,
        "translação não escala"
    );
    // Escala 2×: a espessura (px_per_world) dobra e o local (1,0) vira mundo (2,0).
    let s = fold_model(&base, &Xform([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]));
    assert!(
        (s.px_per_world - 200.0).abs() < 1e-4,
        "escala engrossa o traço"
    );
    let q = apply4(&s.world_to_clip, 1.0, 0.0);
    assert!((q[0] - 2.0).abs() < 1e-4 && q[1].abs() < 1e-4, "{q:?}");
}

/// 🔴 **Cada fatia sai na POSE DA SUA chave** (W7.2) — a arte do quadro corrente na dele,
/// e **cada fantasma na do quadro que ele representa**.
///
/// O fantasma mostra onde o desenho ESTAVA, e "onde" inclui o LUGAR: numa instância que
/// o animador deslocou, herdar a pose do quadro corrente empilharia o fantasma em cima da
/// arte de agora — o rastro sumiria justamente quando ele passa a existir.
///
/// A pose vive no `model` da fatia (o render a dobra na câmera), então o teste lê o afim:
/// a translação dele É a pose. Mutação que sangra: usar `layer.offset_at_cycled(frame)`
/// nos dois lugares, ou não usar pose nenhuma.
#[test]
fn each_slice_carries_the_pose_of_its_own_key() {
    let mut doc = doc_bg_fg();
    let oid = doc.objects()[0].id;
    let fg = doc.object(oid).unwrap().layers()[1].id; // as chaves 0 e 8 do FG
    // O quadro 8 (o que está na tela) foi deslocado; o 0 (que vira fantasma) não.
    doc.object_mut(oid)
        .unwrap()
        .translate_frame(fg, 8, Vec2::new(100.0, 0.0));

    let (layers, _) = collect_layers(&doc, &at(8), None, None, &[], Some(crate::render_loop::flip_pass_ghosts::GhostSources::default()));
    // BG · fantasma do FG (quadro 0) · FG (quadro 8)
    let (ghost, art) = (&layers[1], &layers[2]);

    assert_eq!(
        [art.model.0[4], art.model.0[5]],
        [100.0, 0.0],
        "a arte do quadro corrente ignorou a pose da chave: mover a instancia nao move nada"
    );
    assert_eq!(
        [ghost.model.0[4], ghost.model.0[5]],
        [0.0, 0.0],
        "o fantasma herdou a pose do quadro CORRENTE — ele mostraria o passado no lugar do presente"
    );
}

/// **A pose sai pelo MESMO mapa do desenho** (o do ciclo): num Loop, o quadro da 2ª volta
/// mostra a arte do vão **e a pose dela**. Amostrar o desenho pelo ciclo e a pose pelo
/// quadro cru poria a arte no lugar de um quadro que não existe — a assinatura do bug de
/// coordenada derivada (`feedback_derived_coordinate_seed_must_match_sample`).
#[test]
fn under_a_loop_the_pose_travels_with_the_drawing() {
    use ph2d_flip::{CycleMode, LayerCycle};
    let mut doc = doc_bg_fg();
    let oid = doc.objects()[0].id;
    let fg = doc.object(oid).unwrap().layers()[1].id;
    let obj = doc.object_mut(oid).unwrap();
    obj.translate_frame(fg, 0, Vec2::new(50.0, 0.0)); // a chave 0 está deslocada
    obj.set_exposure(fg, 8, 8); // vão = [0, 16)
    obj.layer_mut(fg).unwrap().cycle = LayerCycle {
        pre: CycleMode::Loop,
        post: CycleMode::Loop,
    };

    // O quadro 16 é o quadro 0 de novo (2ª volta do Loop).
    let (layers, _) = collect_layers(&doc, &at(16), None, None, &[], None);
    let fg_slice = layers.last().expect("a camada FG compoe");
    assert_eq!(
        [fg_slice.model.0[4], fg_slice.model.0[5]],
        [50.0, 0.0],
        "a 2a volta do Loop desenhou a arte do vao na pose ERRADA (a do quadro cru)"
    );
}

/// 🔴 **A espessura do traço é fixa no MUNDO — o render a projeta pelo ZOOM** (§4.C.6).
///
/// Enio 2026-07-17: *"a largura do traço está relativa ao zoom do canvas e não é fixa no
/// mundo"*. O `camera_raw` passava `1.0` na escala de espessura, o que forçava a largura
/// guardada a ser lida como PX DE TELA: o traço ficava com a mesma grossura na tela em
/// qualquer aproximação — ou seja, ENCOLHIA em relação à arte ao dar zoom.
///
/// Agora ele passa o `px_per_world` real, que é o que o `ph2d-flip-render` sempre
/// documentou querer (`thickness_px = raio_mundo · px_per_world`). Aproximar 2× engrossa
/// o traço 2× na tela, como uma foto ampliada.
///
/// Mutação que sangra: voltar a `CameraRaw::new(vp, viewport, 1.0)` — os dois zooms
/// devolvem a mesma escala e a razão vira 1.
#[test]
fn the_stroke_thickness_is_fixed_in_the_world_and_scales_with_the_zoom() {
    let window = ph2d_host::WindowSize {
        width: 1920,
        height: 1080,
    };
    let far = ph2d_render::Camera2d {
        center: [0.0, 0.0],
        height_world: 10.0,
        cull_mask: u32::MAX,
    };
    let near = ph2d_render::Camera2d {
        height_world: 5.0, // 2× mais perto
        ..far
    };

    let s_far = camera_raw(&far, window).px_per_world;
    let s_near = camera_raw(&near, window).px_per_world;

    // A escala de espessura É o px_per_world da câmera.
    assert!(
        (s_far - 1080.0 / 10.0).abs() < 1e-3,
        "escala de espessura {s_far} != px_per_world da camera"
    );
    // E aproximar 2× DOBRA a espessura na tela: a largura mora no mundo.
    assert!(
        (s_near / s_far - 2.0).abs() < 1e-6,
        "2x de zoom nao dobrou a espessura ({s_near}/{s_far}): a largura voltou a ser \
         de TELA (absoluta), e o traço encolhe em relação à arte ao aproximar"
    );
}
