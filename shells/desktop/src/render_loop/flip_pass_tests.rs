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
    let (layers, _) = collect_layers(&doc, &ph, None, None, &[], Some(&[]));

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
    let (layers, _) = collect_layers(&doc, &playing, None, None, &[], Some(&[]));
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
