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
    let (layers, _) = collect_layers(
        &doc,
        &ph,
        None,
        None,
        &[],
        Some(crate::render_loop::flip_pass_ghosts::GhostSources::default()),
        None,
    );

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
        let (layers, _) = collect_layers(doc, &at(f), None, None, &[], None, None);
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
    let (layers, _) = collect_layers(&doc, &at(8), None, None, &[], None, None);
    assert!(layers.iter().all(|l| l.ghost.is_none()), "tool inativa");

    let mut playing = at(8);
    playing.play();
    let (layers, _) = collect_layers(
        &doc,
        &playing,
        None,
        None,
        &[],
        Some(crate::render_loop::flip_pass_ghosts::GhostSources::default()),
        None,
    );
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

    let (layers, _) = collect_layers(
        &doc,
        &at(8),
        None,
        None,
        &[],
        Some(crate::render_loop::flip_pass_ghosts::GhostSources::default()),
        None,
    );
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
    let (layers, _) = collect_layers(&doc, &at(16), None, None, &[], None, None);
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

/// 🔴 **A folha deslizada (Shift & Trace) entra no model do FANTASMA — e só nele.**
///
/// Três metades num gate, porque as três falham separadas: (a) um mapa VAZIO é
/// byte-idêntico ao caminho de sempre (o pin de regressão do modo — fora do Trace nada
/// pode mudar um bit); (b) o shift desloca o model do fantasma exatamente pelo afim
/// autorado; (c) a ARTE (a fatia não-fantasma) nunca o veste — o deslocamento é da
/// referência, não da obra. Mutação que sangra: o `collect` ignorar `sources.trace`
/// (b falha), ou aplicar o shift na fatia da camada (c falha).
#[test]
fn a_traced_ghost_wears_its_shift_and_an_empty_map_changes_nothing() {
    use crate::render_loop::flip_pass_ghosts::GhostSources;
    let doc = doc_bg_fg();
    let ph = at(8); // sobre a 2ª chave do FG → o desenho de 0 vira fantasma
    let (base, _) = collect_layers(
        &doc,
        &ph,
        None,
        None,
        &[],
        Some(GhostSources::default()),
        None,
    );

    // (a) Mapa vazio fornecido = byte-idêntico ao default (nenhum model se move).
    let empty = std::collections::BTreeMap::new();
    let src_empty = GhostSources {
        trace: Some(&empty),
        ..Default::default()
    };
    let (with_empty, _) = collect_layers(&doc, &ph, None, None, &[], Some(src_empty), None);
    for (a, b) in base.iter().zip(with_empty.iter()) {
        assert_eq!(a.model.0, b.model.0, "mapa vazio moveu um model");
    }

    // (b)+(c) Um shift na chave 0 (a do fantasma): SÓ o fantasma o veste.
    let mut map = std::collections::BTreeMap::new();
    map.insert(0, ph2d_flip::Pose::from_translation(Vec2::new(3.0, -2.0)));
    let src_map = GhostSources {
        trace: Some(&map),
        ..Default::default()
    };
    let (shifted, _) = collect_layers(&doc, &ph, None, None, &[], Some(src_map), None);
    for (a, b) in base.iter().zip(shifted.iter()) {
        if b.ghost.is_some() {
            assert_eq!(
                (b.model.0[4] - a.model.0[4], b.model.0[5] - a.model.0[5]),
                (3.0, -2.0),
                "o fantasma tinha de deslizar exatamente pelo shift autorado"
            );
            assert_eq!(
                (b.model.0[0], b.model.0[1], b.model.0[2], b.model.0[3]),
                (a.model.0[0], a.model.0[1], a.model.0[2], a.model.0[3]),
                "translacao pura nao mexe na parte linear"
            );
        } else {
            assert_eq!(
                a.model.0, b.model.0,
                "a ARTE nunca veste o shift da referencia"
            );
        }
    }
}

/// 🔴 **O PEEK folheia a camada ATIVA para o desenho vizinho — e só ela.**
///
/// F1/F3 presos = a folha anterior/seguinte na mão (`docs/Flip/04 §4`): a fatia da
/// camada ativa amostra o desenho vizinho DA CHAVE ATIVA, sem mover o playhead; o BG (o
/// contexto sobre o qual se folheia) não se mexe. Mutação que sangra: ignorar o `peek`
/// (a fatia não muda), ou retimar TODAS as camadas (o BG muda junto).
#[test]
fn holding_the_flip_keys_peeks_the_neighbour_drawing_of_the_active_layer() {
    // ⚠️ As DUAS camadas têm chaves em 0 e 8 — de propósito: com um BG de chave única
    // a mutação "retima TODAS as camadas" ficou VERDE (retimar um BG sem vizinho não o
    // move; a fixture não continha o fenômeno). Aqui um BG folheado por engano MOSTRA.
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("O");
    let obj = doc.object_mut(oid).unwrap();
    obj.fps = 12.0;
    let mut lids = Vec::new();
    for name in ["BG", "FG"] {
        let l = obj.add_layer(name);
        for k in [0, 8] {
            let d = obj
                .insert_frame(l, k, Hold::Implicit, KeyKind::Keyframe)
                .unwrap();
            let mut st = FlipStroke::new();
            st.push_default(Vec2::new(k as f32, 0.0));
            st.push_default(Vec2::new(k as f32 + 1.0, 1.0));
            obj.drawing_mut(d).unwrap().strokes.push(st);
        }
        lids.push(l);
    }
    let obj = &doc.objects()[0];
    let fg = obj.layer(lids[1]).unwrap();
    let (fg_id, d0, d8) = (
        fg.id,
        fg.drawing_at(0).unwrap().0,
        fg.drawing_at(8).unwrap().0,
    );

    // No quadro 8 (chave ativa 8), F1 = a folha ANTERIOR: a fatia do FG mostra o
    // desenho da chave 0. O BG — que TEM um vizinho para onde folhear — fica parado.
    let (base, _) = collect_layers(&doc, &at(8), None, Some(fg_id), &[], None, None);
    let (prev, _) = collect_layers(
        &doc,
        &at(8),
        None,
        Some(fg_id),
        &[],
        None,
        Some(crate::flip_peek::PeekDir::Prev),
    );
    assert_eq!(base.len(), 2, "BG + FG (sem fantasmas: ghosts None)");
    assert_eq!(base[1].cache_key.1, d8, "sem peek o FG mostra a chave 8");
    assert_eq!(
        prev[1].cache_key.1, d0,
        "com F1 o FG mostra a folha ANTERIOR"
    );
    assert_eq!(
        prev[0].cache_key, base[0].cache_key,
        "o BG (contexto) nao folheia junto"
    );
    assert_eq!(prev[0].model.0, base[0].model.0);

    // E no quadro 0, F3 = a folha SEGUINTE (a chave 8).
    let (next, _) = collect_layers(
        &doc,
        &at(0),
        None,
        Some(fg_id),
        &[],
        None,
        Some(crate::flip_peek::PeekDir::Next),
    );
    assert_eq!(
        next[1].cache_key.1, d8,
        "com F3 o FG mostra a folha SEGUINTE"
    );
}

/// **Onde não há para onde folhear, o peek fica onde está** — F2 (a folha atual,
/// sozinha) e F1 na PRIMEIRA chave são byte-idênticos ao sem-peek: o que muda na tela
/// nesses casos é só a ausência dos fantasmas, que é decisão do SHELL (`ghosts: None`).
#[test]
fn peeking_where_there_is_no_neighbour_stays_put() {
    let doc = doc_bg_fg();
    let fg_id = doc.objects()[0].layers().last().unwrap().id;
    for (ph_frame, dir) in [
        (8, crate::flip_peek::PeekDir::Here),
        (0, crate::flip_peek::PeekDir::Prev),
    ] {
        let (base, _) = collect_layers(&doc, &at(ph_frame), None, Some(fg_id), &[], None, None);
        let (peeked, _) =
            collect_layers(&doc, &at(ph_frame), None, Some(fg_id), &[], None, Some(dir));
        for (a, b) in base.iter().zip(peeked.iter()) {
            assert_eq!(
                a.cache_key, b.cache_key,
                "{dir:?} em {ph_frame} moveu a arte"
            );
            assert_eq!(a.model.0, b.model.0);
        }
    }
}

/// 🔴 **A âncora do peek é a CHAVE ATIVA, não o quadro cru** — no meio de um hold,
/// `prev_drawing_key(quadro)` devolve o INÍCIO da exposição atual: o MESMO desenho que
/// já está na tela, e um peek que mostra o que já se vê não é um peek. Fixture com
/// chaves 0/4/8, playhead no 5 (hold da 4): F1 tem de mostrar a folha da chave 0.
#[test]
fn mid_hold_the_peek_anchors_on_the_active_key_not_the_raw_frame() {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("O");
    let obj = doc.object_mut(oid).unwrap();
    obj.fps = 12.0;
    let l = obj.add_layer("L");
    for k in [0, 4, 8] {
        let d = obj
            .insert_frame(l, k, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(k as f32, 0.0));
        s.push_default(Vec2::new(k as f32 + 1.0, 1.0));
        obj.drawing_mut(d).unwrap().strokes.push(s);
    }
    let layer = doc.objects()[0].layer(l).unwrap();
    let d0 = layer.drawing_at(0).unwrap().0;
    let (peeked, _) = collect_layers(
        &doc,
        &at(5),
        None,
        Some(l),
        &[],
        None,
        Some(crate::flip_peek::PeekDir::Prev),
    );
    assert_eq!(
        peeked[0].cache_key.1, d0,
        "no meio do hold da chave 4, a folha ANTERIOR e' a da chave 0 — nao a propria 4"
    );
}
