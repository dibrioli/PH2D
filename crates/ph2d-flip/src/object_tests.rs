//! Testes do [`FlipObject`] — módulo-irmão pelo cap de LOC (HR-18).
//!
//! Declarado pelo pai via `#[path]`, então `super` é o módulo `object`.

use super::*;

/// Um objeto com uma camada e um desenho não-vazio no quadro 0.
fn object_with_one_frame() -> (FlipObject, LayerId, DrawingId) {
    let mut o = FlipObject::new(FlipObjectId(1), "Obj");
    let l = o.add_layer("Layer 1");
    let d = o
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    (o, l, d)
}

#[test]
fn insert_frame_allocates_a_drawing_with_one_user() {
    let (o, l, d) = object_with_one_frame();
    assert_eq!(d, DrawingId(0));
    assert_eq!(o.drawings().len(), 1);
    assert_eq!(o.drawing(d).unwrap().users(), 1);
    assert_eq!(o.drawing_at(l, 0), Some(d));
    assert_eq!(o.drawing_at(l, 5), Some(d), "segura");
}

#[test]
fn raise_and_lower_reorder_layers_without_touching_drawings() {
    let mut o = FlipObject::new(FlipObjectId(1), "Obj");
    let a = o.add_layer("A"); // índice 0 (fundo)
    let b = o.add_layer("B"); // índice 1
    let c = o.add_layer("C"); // índice 2 (topo)
    let ids = |o: &FlipObject| o.layers().iter().map(|l| l.id).collect::<Vec<_>>();
    assert_eq!(ids(&o), vec![a, b, c]);

    assert!(o.raise_layer(a)); // A sobe: [B, A, C]
    assert_eq!(ids(&o), vec![b, a, c]);
    assert!(!o.raise_layer(c), "C já é o topo");
    assert!(o.lower_layer(c)); // C desce: [B, C, A]
    assert_eq!(ids(&o), vec![b, c, a]);
    assert!(!o.lower_layer(b), "B já é o fundo");
    assert!(!o.raise_layer(LayerId(999)), "id inexistente");
}

/// Empurra um traço de 2 pontos `a`→`b` no desenho `d`.
fn push_seg(o: &mut FlipObject, d: DrawingId, a: [f32; 2], b: [f32; 2]) {
    let mut s = crate::stroke::FlipStroke::new();
    s.push_default(ph2d_core::Vec2::new(a[0], a[1]));
    s.push_default(ph2d_core::Vec2::new(b[0], b[1]));
    o.drawing_mut(d).unwrap().strokes.push(s);
}

#[test]
fn geometry_bbox_unions_all_strokes_and_drawings() {
    let (mut o, l, d0) = object_with_one_frame();
    assert_eq!(o.geometry_bbox(), None, "objeto sem pontos: sem bbox");
    push_seg(&mut o, d0, [10.0, 20.0], [30.0, 15.0]);
    // 2º desenho noutro quadro estende a bbox.
    let d1 = o
        .insert_frame(l, 5, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    push_seg(&mut o, d1, [-5.0, 40.0], [12.0, 41.0]);
    assert_eq!(
        o.geometry_bbox(),
        Some(([-5.0, 15.0], [30.0, 41.0])),
        "união de todos os pontos de todos os desenhos"
    );
}

#[test]
fn bake_affine_translation_shifts_every_point() {
    let (mut o, _l, d0) = object_with_one_frame();
    push_seg(&mut o, d0, [10.0, 20.0], [30.0, 40.0]);
    // Translação pura (-10, -20): a geometria recua, a bbox acompanha.
    o.bake_affine([1.0, 0.0, 0.0, 1.0, -10.0, -20.0]);
    assert_eq!(o.geometry_bbox(), Some(([0.0, 0.0], [20.0, 20.0])));
    // Largura/opacidade não são tocadas por um bake de translação.
    let s = &o.drawing(d0).unwrap().strokes[0];
    assert_eq!(s.positions()[0], ph2d_core::Vec2::new(0.0, 0.0));
}

#[test]
fn insert_frame_collision_allocates_no_drawing() {
    let (mut o, l, _d) = object_with_one_frame();
    // Segunda chave real em 0 → falha, sem desenho novo.
    assert_eq!(
        o.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe),
        None
    );
    assert_eq!(o.drawings().len(), 1, "não vazou um desenho órfão");
}

/// T0.4 espelhado a nível de objeto: duplicar-como-instância compartilha o
/// desenho; remover um frame decrementa; compactar remapeia.
#[test]
fn instance_shares_drawing_and_removal_decrements() {
    let (mut o, l, d0) = object_with_one_frame();
    assert!(o.duplicate_frame(l, 0, 10, DupMode::Instance));
    assert_eq!(o.drawing(d0).unwrap().users(), 2, "instanciado");
    assert!(o.drawing(d0).unwrap().is_instanced());
    assert_eq!(o.drawing_at(l, 10), Some(d0), "o clone-instância aponta d0");

    // Remove a chave 10 → users volta a 1.
    assert!(o.remove_frame(l, 10));
    assert_eq!(o.drawing(d0).unwrap().users(), 1);
}

#[test]
fn deep_duplicate_copies_the_drawing() {
    let (mut o, l, d0) = object_with_one_frame();
    // Põe conteúdo no d0 para provar a cópia profunda.
    o.drawing_mut(d0).unwrap().strokes.push(Default::default());
    assert!(o.duplicate_frame(l, 0, 10, DupMode::Deep));
    assert_eq!(o.drawings().len(), 2);
    let d1 = o.drawing_at(l, 10).unwrap();
    assert_ne!(d1, d0, "desenho NOVO");
    assert_eq!(o.drawing(d1).unwrap().users(), 1);
    assert_eq!(o.drawing(d0).unwrap().users(), 1, "d0 não ganhou user");
    assert_eq!(o.drawing(d1).unwrap().strokes.len(), 1, "conteúdo copiado");
}

/// T0.4: compactação com remap. Cria 3 desenhos, remove o do MEIO, compacta;
/// os `DrawingId` dos frames sobreviventes são remapeados.
#[test]
fn remove_unused_drawings_compacts_and_remaps() {
    let mut o = FlipObject::new(FlipObjectId(1), "Obj");
    let l = o.add_layer("L");
    let d0 = o
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let d1 = o
        .insert_frame(l, 5, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let d2 = o
        .insert_frame(l, 10, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    assert_eq!((d0, d1, d2), (DrawingId(0), DrawingId(1), DrawingId(2)));

    // Remove o frame do meio (chave 5, desenho d1). users(d1) → 0.
    assert!(o.remove_frame(l, 5));
    assert_eq!(o.drawing(d1).unwrap().users(), 0);

    o.remove_unused_drawings();
    // d1 sumiu; d2 foi remapeado de 2 → 1.
    assert_eq!(o.drawings().len(), 2);
    // Os frames restantes ainda resolvem para desenhos válidos e distintos.
    let at0 = o.drawing_at(l, 0).unwrap();
    let at10 = o.drawing_at(l, 10).unwrap();
    assert_eq!(at0, DrawingId(0), "d0 ficou no lugar");
    assert_eq!(at10, DrawingId(1), "d2 remapeado para 1");
    assert_ne!(at0, at10);
    // Nada aparece em 5..9 (o frame foi removido; d0 é implicit → estende? NÃO:
    // remove_frame com anterior implicit apaga a chave, então d0 estende).
    assert_eq!(o.drawing_at(l, 7), Some(at0), "d0 estende sobre o buraco");
}

#[test]
fn move_frame_relocates_key() {
    let (mut o, l, d0) = object_with_one_frame();
    assert!(o.move_frame(l, 0, 20));
    assert_eq!(o.drawing_at(l, 0), None, "não há mais nada antes de 20");
    assert_eq!(o.drawing_at(l, 20), Some(d0));
    // Mover para cima de uma chave real falha.
    o.insert_frame(l, 25, Hold::Implicit, KeyKind::Keyframe);
    assert!(!o.move_frame(l, 20, 25));
}

#[test]
fn remove_layer_drops_its_drawings_users() {
    let (mut o, l, d0) = object_with_one_frame();
    assert!(o.remove_layer(l));
    assert_eq!(o.layers().len(), 0);
    assert_eq!(o.drawing(d0).unwrap().users(), 0, "sem camada, sem user");
    o.remove_unused_drawings();
    assert_eq!(o.drawings().len(), 0);
}

/// T0.12: amostragem por playhead. FPS 24: t=0.25s → quadro 6.
#[test]
fn sample_by_playhead_resolves_active_drawing_per_layer() {
    let mut o = FlipObject::new(FlipObjectId(1), "Obj");
    o.fps = 24.0;
    let l = o.add_layer("L");
    let d0 = o
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let d6 = o
        .insert_frame(l, 6, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();

    let mut ph = Playhead::new(1.0 / 24.0);
    ph.seek(0.0);
    assert_eq!(o.frame_at(&ph), 0);
    assert_eq!(o.sample_at(&ph), vec![(l, Some(d0))]);

    ph.seek(0.25); // 0.25 * 24 = quadro 6
    assert_eq!(o.frame_at(&ph), 6);
    assert_eq!(o.sample_at(&ph), vec![(l, Some(d6))]);
}

// ── A POSE DA CHAVE (W7.2) — a outra metade da instância ──────────────────

/// Um objeto com DUAS chaves compartilhando o MESMO desenho (uma instância), com um
/// traço na arte.
fn instanced_pair() -> (FlipObject, LayerId, DrawingId) {
    let (mut o, l, d) = object_with_one_frame();
    let mut st = crate::stroke::FlipStroke::new();
    st.push_default(Vec2::new(0.0, 0.0));
    st.push_default(Vec2::new(10.0, 0.0));
    o.drawing_mut(d).unwrap().strokes.push(st);
    assert!(o.duplicate_frame(l, 0, 5, DupMode::Instance));
    assert!(
        o.drawing(d).unwrap().is_instanced(),
        "o fixture nao instanciou"
    );
    (o, l, d)
}

/// 🔴 **A pose é da CHAVE, não do desenho** — é isto que faz uma instância ser mais
/// que um hold.
///
/// Duas chaves compartilham a arte; mover UMA move só ela. A geometria não é tocada
/// (senão o gêmeo andaria junto e as duas ficariam eternamente uma sobre a outra — o
/// bug que o Enio viu: *"a instância não pode ser movida sozinha"*).
///
/// Mutação que sangra: `translate_frame` escrever na geometria do desenho.
#[test]
fn moving_one_instanced_key_moves_only_that_frame() {
    let (mut o, l, d) = instanced_pair();
    let before: Vec<Vec2> = o.drawing(d).unwrap().strokes[0].positions().to_vec();

    assert!(o.translate_frame(l, 5, Vec2::new(100.0, 0.0)));

    assert_eq!(
        o.frame_pose(l, 5).translation(),
        Vec2::new(100.0, 0.0),
        "a chave 5 nao andou"
    );
    assert_eq!(
        o.frame_pose(l, 0).translation(),
        Vec2::ZERO,
        "a chave 0 andou junto — a pose vazou para o gemeo"
    );
    assert_eq!(
        o.drawing(d).unwrap().strokes[0].positions(),
        &before[..],
        "a GEOMETRIA foi tocada: o desenho e dos dois quadros, e um deles o reescreveu"
    );
}

/// **Duplicar carrega a pose** (as duas formas). Sem isto, duplicar uma chave
/// deslocada faria a cópia saltar para a origem do objeto — o desenho pularia no
/// quadro seguinte, e o animador veria a arte "voltar" sozinha.
#[test]
fn duplicating_a_posed_key_carries_the_pose() {
    for mode in [DupMode::Deep, DupMode::Instance] {
        let (mut o, l, _d) = object_with_one_frame();
        o.translate_frame(l, 0, Vec2::new(7.0, -3.0));
        assert!(o.duplicate_frame(l, 0, 4, mode));
        assert_eq!(
            o.frame_pose(l, 4).translation(),
            Vec2::new(7.0, -3.0),
            "{mode:?}: a chave nova nasceu na origem, e nao onde a arte esta"
        );
    }
}

/// **Unlink** (`make_single_user`): a chave larga a arte compartilhada e ganha a
/// própria cópia. A saída de emergência da instância — e ela precisa de fato SEPARAR
/// (editar uma não pode mais alcançar a outra).
#[test]
fn unlinking_a_key_gives_it_art_of_its_own() {
    let (mut o, l, d) = instanced_pair();

    assert!(o.make_single_user(l, 5), "o unlink recusou uma instancia");

    let d5 = o.layer(l).unwrap().drawing_at(5).unwrap();
    assert_ne!(d5, d, "a chave 5 continua na arte compartilhada");
    assert_eq!(o.drawing(d).unwrap().users(), 1, "o refcount nao desceu");
    assert!(!o.drawing(d).unwrap().is_instanced());
    assert_eq!(
        o.drawing(d5).unwrap().strokes.len(),
        1,
        "a copia nasceu vazia: o unlink perdeu a arte"
    );

    // E agora os dois quadros divergem de verdade.
    o.drawing_mut(d5).unwrap().strokes.clear();
    assert_eq!(
        o.drawing(d).unwrap().strokes.len(),
        1,
        "apagar no quadro 5 alcancou o quadro 0: o vinculo nao foi quebrado"
    );
}

/// **Unlink numa arte exclusiva é no-op honesto** — não há vínculo a quebrar, e
/// duplicar o desenho à toa deixaria lixo no documento.
#[test]
fn unlinking_an_exclusive_drawing_is_a_no_op() {
    let (mut o, l, _d) = object_with_one_frame();
    let n = o.drawings().len();
    assert!(!o.make_single_user(l, 0));
    assert_eq!(o.drawings().len(), n, "criou um desenho a toa");
}

/// **`posed_bbox` mede a arte COMO ELA APARECE; `geometry_bbox`, onde os pontos
/// ESTÃO.** As duas respondem perguntas diferentes, e trocá-las põe a caixa do gizmo
/// longe do desenho no instante em que alguém move uma instância.
#[test]
fn the_posed_bbox_includes_the_key_offsets_and_the_geometry_bbox_does_not() {
    let (mut o, l, _d) = instanced_pair(); // traço de (0,0) a (10,0), nas chaves 0 e 5
    o.translate_frame(l, 5, Vec2::new(100.0, 0.0));

    assert_eq!(
        o.geometry_bbox(),
        Some(([0.0, 0.0], [10.0, 0.0])),
        "a bbox de GEOMETRIA nao pode saber de pose"
    );
    assert_eq!(
        o.posed_bbox(),
        Some(([0.0, 0.0], [110.0, 0.0])),
        "a bbox da APARENCIA ignorou a chave deslocada"
    );
}

// ── open_gap_at: criar chave no MEIO da tira, não só depois da última ─────────

/// 🔴 **`open_gap_at` abre espaço numa chave real** — o que deixa Key Add/Dup/Instance
/// criarem quadro no meio da tira, e não só na última chave (smoke do Enio 2026-07-14).
///
/// Empurra à frente **só o bloco contíguo** que começa em `at`, com a pose de cada chave
/// junto. Quadros separados por um buraco não são tocados.
///
/// Mutação que sangra: fazer `open_gap_at` empurrar TUDO `>= at` (o `10` andaria à toa),
/// ou não empurrar nada (o insert em cima da chave real falharia).
#[test]
fn open_gap_at_ripples_only_the_contiguous_block() {
    let mut o = FlipObject::new(FlipObjectId(1), "O");
    let l = o.add_layer("L");
    for k in [0, 5, 6, 10] {
        o.insert_frame(l, k, Hold::Implicit, KeyKind::Keyframe);
    }
    // Uma pose na chave 5, para provar que ela viaja no ripple.
    o.translate_frame(l, 5, Vec2::new(7.0, -3.0));

    assert!(o.open_gap_at(l, 5), "nao abriu espaco numa chave ocupada");

    let keys: Vec<Frame> = o.layer(l).unwrap().frames().keys().copied().collect();
    assert_eq!(
        keys,
        vec![0, 6, 7, 10],
        "o bloco contiguo 5-6 andou +1; a chave 10 (apos o buraco) nao pode ter se mexido"
    );
    assert_eq!(
        o.frame_pose(l, 6).translation(),
        Vec2::new(7.0, -3.0),
        "a pose da chave 5 nao viajou com ela para 6"
    );
}

/// **`open_gap_at` numa chave livre é no-op** — não há bloco a empurrar.
#[test]
fn open_gap_at_a_free_frame_moves_nothing() {
    let mut o = FlipObject::new(FlipObjectId(1), "O");
    let l = o.add_layer("L");
    for k in [0, 5] {
        o.insert_frame(l, k, Hold::Implicit, KeyKind::Keyframe);
    }
    assert!(!o.open_gap_at(l, 3), "quadro 3 esta livre: nada a empurrar");
    let keys: Vec<Frame> = o.layer(l).unwrap().frames().keys().copied().collect();
    assert_eq!(keys, vec![0, 5]);
}

// ── duplicate_layer (§4.C) ────────────────────────────────────────────────────────

/// 🔴 **Duplicar uma camada é uma cópia INDEPENDENTE, acima da original.** A cópia leva os
/// mesmos frames e propriedades, mas desenhos PRÓPRIOS: editar a cópia não toca o original.
///
/// Mutação que sangra: reusar o mesmo `DrawingId` (não deep-copiar) — editar a cópia mudaria
/// o original, e o `assert_ne` de conteúdo pega.
#[test]
fn duplicate_layer_is_an_independent_copy_above_the_original() {
    let (mut o, l, d) = object_with_one_frame();
    push_seg(&mut o, d, [0.0, 0.0], [10.0, 0.0]);
    o.layer_mut(l).unwrap().opacity = 0.5;
    o.layer_mut(l).unwrap().blend = crate::BlendMode::Multiply;

    let dup = o.duplicate_layer(l).expect("duplicou");
    assert_ne!(dup, l, "id novo");
    // Acima da original: [original, cópia].
    let order: Vec<LayerId> = o.layers().iter().map(|x| x.id).collect();
    assert_eq!(order, vec![l, dup], "a cópia entra ACIMA da original");
    // Propriedades copiadas.
    assert_eq!(o.layer(dup).unwrap().opacity, 0.5);
    assert_eq!(o.layer(dup).unwrap().blend, crate::BlendMode::Multiply);
    assert!(o.layer(dup).unwrap().name.contains("copy"));

    // Desenho PRÓPRIO: editar a cópia não muda o original.
    let dup_did = o.drawing_at(dup, 0).unwrap();
    let src_did = o.drawing_at(l, 0).unwrap();
    assert_ne!(dup_did, src_did, "a cópia tem desenho próprio");
    push_seg(&mut o, dup_did, [99.0, 99.0], [100.0, 100.0]);
    assert_eq!(
        o.drawing(src_did).unwrap().strokes.len(),
        1,
        "o original intacto"
    );
    assert_eq!(
        o.drawing(dup_did).unwrap().strokes.len(),
        2,
        "a cópia divergiu"
    );
}

/// 🔴 **A instância DENTRO da camada é preservada** — dois quadros que compartilham UM
/// desenho na origem compartilham o MESMO desenho novo na cópia (um ciclo continua ciclo),
/// e o refcount reflete isso.
///
/// Mutação que sangra: deep-copiar por-QUADRO (um desenho novo por frame) — os dois quadros
/// da cópia teriam ids diferentes e `users` seria 1 em vez de 2.
#[test]
fn duplicate_layer_preserves_intra_layer_instancing() {
    let (mut o, l, d) = object_with_one_frame();
    push_seg(&mut o, d, [0.0, 0.0], [10.0, 0.0]);
    // Instancia o MESMO desenho no quadro 5 (um ciclo de 2 quadros, 1 arte).
    assert!(o.duplicate_frame(l, 0, 5, DupMode::Instance));
    assert_eq!(
        o.drawing(d).unwrap().users(),
        2,
        "1 arte, 2 quadros (origem)"
    );

    let dup = o.duplicate_layer(l).unwrap();
    let a = o.drawing_at(dup, 0).unwrap();
    let b = o.drawing_at(dup, 5).unwrap();
    assert_eq!(
        a, b,
        "os dois quadros da cópia compartilham UM desenho (instância)"
    );
    assert_ne!(a, d, "e é um desenho NOVO, não o da origem");
    assert_eq!(
        o.drawing(a).unwrap().users(),
        2,
        "o refcount reflete os 2 quadros"
    );
    // A origem intacta.
    assert_eq!(o.drawing(d).unwrap().users(), 2);
}

/// 🔴 **A cópia sobrevive ao delete da original** (e vice-versa) — os refcounts são
/// independentes, então `remove_unused_drawings` não reclama arte viva.
///
/// Mutação que sangra: `users = 0` nos desenhos novos — deletar a original rodaria o GC e
/// levaria a arte da cópia junto.
#[test]
fn the_copy_and_the_original_can_be_deleted_independently() {
    let (mut o, l, d) = object_with_one_frame();
    push_seg(&mut o, d, [0.0, 0.0], [10.0, 0.0]);
    let dup = o.duplicate_layer(l).unwrap();
    let dup_did = o.drawing_at(dup, 0).unwrap();

    assert!(o.remove_layer(l));
    o.remove_unused_drawings();
    // A cópia ainda desenha algo.
    assert_eq!(
        o.drawing(o.drawing_at(dup, 0).unwrap())
            .unwrap()
            .strokes
            .len(),
        1,
        "a arte da cópia sobreviveu ao delete da original"
    );
    let _ = dup_did;
}

/// Id de camada inexistente = `None`, sem efeito colateral.
#[test]
fn duplicate_layer_of_a_missing_id_is_none() {
    let (mut o, _l, _d) = object_with_one_frame();
    let before = o.layers().len();
    assert_eq!(o.duplicate_layer(LayerId(999)), None);
    assert_eq!(o.layers().len(), before, "nada foi criado");
}

// ── rename_layer (§4.C) ────────────────────────────────────────────────────────

/// Renomear troca SÓ o nome da camada — id, frames e arte intactos.
///
/// Mutação que sangra: `rename_layer` não escrever o nome (retornar `true` sem tocar
/// `l.name`) — o `assert_eq!` do nome cai.
#[test]
fn rename_layer_sets_only_the_name() {
    let (mut o, l, d) = object_with_one_frame();
    let name_before = o.layer(l).unwrap().name.clone();
    assert_ne!(name_before, "Rough", "fixture não pode já se chamar Rough");
    let frames_before = o.layer(l).unwrap().frames().len();

    assert!(o.rename_layer(l, "Rough"));
    assert_eq!(o.layer(l).unwrap().name, "Rough");
    assert_eq!(
        o.layer(l).unwrap().id,
        l,
        "o id é estável (máscaras apontam pra cá)"
    );
    assert_eq!(
        o.layer(l).unwrap().frames().len(),
        frames_before,
        "renomear não toca os frames"
    );
    assert_eq!(o.drawing(d).map(|_| ()), Some(()), "a arte segue viva");
}

/// Id inexistente = `false`, sem efeito colateral (a recusa mora no modelo, não só na UI).
#[test]
fn rename_layer_of_a_missing_id_is_false() {
    let (mut o, l, _d) = object_with_one_frame();
    let name_before = o.layer(l).unwrap().name.clone();
    assert!(!o.rename_layer(LayerId(999), "Nope"));
    assert_eq!(
        o.layer(l).unwrap().name,
        name_before,
        "uma camada real não pode ter sido renomeada por um id fantasma"
    );
}
