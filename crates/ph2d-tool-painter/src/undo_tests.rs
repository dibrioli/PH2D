//! Gates do [`crate::undo`] — extraídos do arquivo do produto pelo cap de LOC (HR-18); seguem
//! sendo um `mod tests` FILHO, então `use super::*` continua alcançando os privados.
use super::*;

fn model(active_px: u8) -> ModelSnapshot {
    ModelSnapshot {
        layers: LayerStack::new(),
        images: BTreeMap::new(),
        heights: BTreeMap::new(),
        mats: BTreeMap::new(),
        covers: BTreeMap::new(),
        canvas_rgba: Arc::new(vec![active_px; 16]),
        canvas_size: (4, 4),
        selection: BTreeSet::new(),
        shape: None,
        offset_norm: 0.5,
        offset_base_px: 0.0,
        preview_patch: None,
        parked_shapes: Vec::new(),
        active_op: 0,
        mask_scratch: Arc::new(Vec::new()),
        mask_scratch_target: None,
        selection_mask: Arc::new(Vec::new()),
        selection_active: false,
        selection_crisp: Arc::new(Vec::new()),
        selection_feather: 0.0,
        selection_shapes: Vec::new(),
        deform: WarpSnap {
            disp: Arc::new(Vec::new()),
            pre: Arc::new(Vec::new()),
            pre_h: Arc::new(Vec::new()),
            pre_cover: Arc::new(Vec::new()),
            pre_mats: Arc::new(Vec::new()),
            relief_layer: None,
            active: false,
        },
        sculpt: SculptSnap::default(),
    }
}

#[test]
fn undo_rolls_back_to_before() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    c.record_structural(model(0x11), model(0x22));
    assert!(c.can_undo());
    let restored = c.undo().expect("one entry to undo");
    assert_eq!(restored.canvas_rgba.as_slice(), &[0x11; 16]);
    assert!(!c.can_undo());
    assert!(c.can_redo());
}

#[test]
fn redo_rolls_forward_to_after() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    c.record_structural(model(0x11), model(0x22));
    c.undo();
    let restored = c.redo().expect("one entry to redo");
    assert_eq!(restored.canvas_rgba.as_slice(), &[0x22; 16]);
    assert!(c.can_undo());
    assert!(!c.can_redo());
}

#[test]
fn new_edit_clears_redo_branch() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    c.record_structural(model(0), model(1));
    c.record_structural(model(1), model(2));
    c.undo();
    assert!(c.can_redo());
    c.record_structural(model(1), model(3));
    assert!(!c.can_redo(), "a new edit must invalidate the redo branch");
}

/// A run of same-kind coalescible entries collapses to ONE undo step spanning first-before →
/// latest-after; a plain entry breaks the run; an undo/redo boundary never merges across.
#[test]
fn coalesced_runs_merge_and_break_correctly() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    c.record_structural_coalesced(CoalesceKind::Simplify, model(0), model(1));
    c.record_structural_coalesced(CoalesceKind::Simplify, model(1), model(2));
    c.record_structural_coalesced(CoalesceKind::Simplify, model(2), model(3));
    assert_eq!(c.undo_depth(), 1, "three Simplify presses = one entry");
    let restored = c.undo().expect("entry");
    assert_eq!(
        restored.canvas_rgba.as_slice(),
        &[0; 16],
        "one undo restores the state before the FIRST press"
    );
    let fwd = c.redo().expect("entry");
    assert_eq!(
        fwd.canvas_rgba.as_slice(),
        &[3; 16],
        "redo lands on the LATEST press"
    );
    // A different kind never merges.
    c.record_structural_coalesced(CoalesceKind::OpCycleSelection(0), model(3), model(4));
    c.record_structural_coalesced(CoalesceKind::OpCycleSelection(1), model(4), model(5));
    assert_eq!(
        c.undo_depth(),
        3,
        "different shapes' taps stay separate entries"
    );
    // A plain entry breaks the run: the next same-kind action starts a NEW entry.
    c.record_structural_coalesced(CoalesceKind::Simplify, model(5), model(6));
    c.record_structural(model(6), model(7));
    c.record_structural_coalesced(CoalesceKind::Simplify, model(7), model(8));
    assert_eq!(
        c.undo_depth(),
        6,
        "a plain entry between runs prevents merging"
    );
}

/// After an undo, a new same-kind action must NOT merge into the undone entry (the redo branch is
/// discarded and a fresh run starts) — merging across the boundary would corrupt the timeline.
#[test]
fn coalescing_never_merges_across_an_undo_boundary() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    c.record_structural_coalesced(CoalesceKind::Simplify, model(0), model(1));
    c.undo();
    c.record_structural_coalesced(CoalesceKind::Simplify, model(0), model(9));
    assert_eq!(c.undo_depth(), 1);
    assert!(!c.can_redo(), "the redo branch was discarded");
    let restored = c.undo().expect("entry");
    assert_eq!(restored.canvas_rgba.as_slice(), &[0; 16]);
}

#[test]
fn clear_drops_both_stacks() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    c.record_structural(model(0), model(1));
    c.undo();
    c.clear();
    assert!(!c.can_undo() && !c.can_redo());
}

// ───────────────────────── U1: o histórico guarda um DELTA, capeado em BYTES ─────────────────────────

/// Um snapshot com **os dezenove planos canvas-shaped preenchidos**, todos carimbados com `seed` numa
/// **JANELA pequena** sobre um fundo comum. Um canvas 4×4: os planos RGBA têm 64 elementos (stride 16),
/// os escalares 16 (stride 4).
///
/// ⚠️ **Duas propriedades da fixture, e as duas foram pagas.**
///
/// 1. **Todo plano difere.** Uma fixture que deixasse um vazio testaria o fato onde ele é *conveniente*
///    em vez de onde pode ser CONTRADITO — o buraco pelo qual o `mats` escapou do snapshot em 2026-07-13
///    (na tela vazia a cobertura é zero, a luz pesa o material por zero, e nada aparece).
/// 2. **A diferença é uma JANELA, não o plano inteiro** — e isto é o que faz o gate do CURSOR morder. Na
///    1ª versão os endpoints diferiam em todos os elementos, então cada plano caía em `Whole`, a
///    materialização nunca consultava o cursor, e a mutação *"o cursor não anda com a história"*
///    **SOBREVIVEU aos onze gates**. A fixture não continha o fenômeno; um traço real muda uma janela.
fn model_all_planes(seed: u8) -> ModelSnapshot {
    use ph2d_painter_brush::material::MaterialBytes;
    // Um canvas 8×8: os planos RGBA têm 256 elementos (stride 32), os escalares 64 (stride 8).
    //
    // ⚠️ A janela é UMA célula na linha 3, e **o seed escolhe a coluna** — dois estados quaisquer tocam
    // lugares DIFERENTES. Foi isto que finalmente fez a mutação do cursor sangrar: com a janela no mesmo
    // lugar em todos os estados, o patch reescrevia exatamente o que o cursor errado trazia de errado, e
    // o fundo comum cobria o resto. Traços de um artista caem em lugares diferentes, e é o cenário que o
    // `measure_undo_memory` já usava.
    let col = |s: u8| usize::from(s % 6);
    let rgba_i = |s: u8| 3 * 32 + col(s);
    let scal_i = |s: u8| 3 * 8 + col(s);
    let rgba = |s: u8| {
        Arc::new(
            (0..256usize)
                .map(|i| {
                    if i == rgba_i(s) {
                        s
                    } else {
                        u8::try_from(i % 251).unwrap_or(0)
                    }
                })
                .collect::<Vec<u8>>(),
        )
    };
    let bytes = |s: u8| {
        Arc::new(
            (0..64usize)
                .map(|i| {
                    if i == scal_i(s) {
                        s
                    } else {
                        u8::try_from(i).unwrap_or(0)
                    }
                })
                .collect::<Vec<u8>>(),
        )
    };
    let floats = |s: u8| {
        Arc::new(
            (0..64usize)
                .map(|i| {
                    if i == scal_i(s) {
                        f32::from(s)
                    } else {
                        u16::try_from(i).map_or(0.0, f32::from)
                    }
                })
                .collect::<Vec<f32>>(),
        )
    };
    let mats = |s: u8| {
        Arc::new(
            (0..64usize)
                .map(|i| {
                    [if i == scal_i(s) {
                        s
                    } else {
                        u8::try_from(i).unwrap_or(0)
                    }; 7] as MaterialBytes
                })
                .collect::<Vec<_>>(),
        )
    };
    let disp = |s: u8| {
        Arc::new(
            (0..64usize)
                .map(|i| {
                    if i == scal_i(s) {
                        [f32::from(s), 1.0]
                    } else {
                        [0.0, 0.0]
                    }
                })
                .collect::<Vec<[f32; 2]>>(),
        )
    };
    let layer = crate::layers::LayerId(1);
    let mut m = model(seed);
    m.canvas_size = (8, 8);
    m.canvas_rgba = rgba(seed);
    m.images = [(
        crate::layers::LayerId(2),
        Arc::new(LayerImage {
            width: 8,
            height: 8,
            rgba8: rgba(seed).as_ref().clone(),
        }),
    )]
    .into_iter()
    .collect();
    m.heights = [(layer, floats(seed))].into_iter().collect();
    m.covers = [(layer, bytes(seed))].into_iter().collect();
    m.mats = [(layer, mats(seed))].into_iter().collect();
    m.mask_scratch = rgba(seed.wrapping_add(1));
    m.selection_mask = bytes(seed.wrapping_add(2));
    m.selection_crisp = bytes(seed.wrapping_add(3));
    m.deform = WarpSnap {
        disp: disp(seed),
        pre: rgba(seed.wrapping_add(4)),
        pre_h: floats(seed.wrapping_add(5)),
        pre_cover: bytes(seed.wrapping_add(6)),
        pre_mats: mats(seed.wrapping_add(7)),
        relief_layer: Some(layer),
        active: true,
    };
    m.sculpt = SculptSnap {
        pre: floats(seed.wrapping_add(8)),
        amount: floats(seed.wrapping_add(9)),
        plane_sum: floats(seed.wrapping_add(10)),
        pre_cover: bytes(seed.wrapping_add(11)),
        pre_mats: mats(seed.wrapping_add(12)),
        pre_rgba: rgba(seed.wrapping_add(13)),
        layer: Some(layer),
        bbox: None,
    };
    m
}

/// Iguala dois snapshots **plano a plano**, nomeando qual falhou.
fn assert_same_planes(got: &ModelSnapshot, want: &ModelSnapshot, what: &str) {
    assert_eq!(got.canvas_rgba, want.canvas_rgba, "{what}: canvas_rgba");
    assert_eq!(
        got.images.len(),
        want.images.len(),
        "{what}: images (contagem)"
    );
    for (k, v) in &want.images {
        assert_eq!(
            got.images.get(k).map(|i| &i.rgba8),
            Some(&v.rgba8),
            "{what}: images[{k:?}]"
        );
    }
    assert_eq!(got.heights, want.heights, "{what}: heights");
    assert_eq!(got.covers, want.covers, "{what}: covers");
    assert_eq!(got.mats, want.mats, "{what}: mats");
    assert_eq!(got.mask_scratch, want.mask_scratch, "{what}: mask_scratch");
    assert_eq!(
        got.selection_mask, want.selection_mask,
        "{what}: selection_mask"
    );
    assert_eq!(
        got.selection_crisp, want.selection_crisp,
        "{what}: selection_crisp"
    );
    assert_eq!(got.deform.disp, want.deform.disp, "{what}: deform.disp");
    assert_eq!(got.deform.pre, want.deform.pre, "{what}: deform.pre");
    assert_eq!(got.deform.pre_h, want.deform.pre_h, "{what}: deform.pre_h");
    assert_eq!(
        got.deform.pre_cover, want.deform.pre_cover,
        "{what}: deform.pre_cover"
    );
    assert_eq!(
        got.deform.pre_mats, want.deform.pre_mats,
        "{what}: deform.pre_mats"
    );
    assert_eq!(got.sculpt.pre, want.sculpt.pre, "{what}: sculpt.pre");
    assert_eq!(
        got.sculpt.amount, want.sculpt.amount,
        "{what}: sculpt.amount"
    );
    assert_eq!(
        got.sculpt.plane_sum, want.sculpt.plane_sum,
        "{what}: sculpt.plane_sum"
    );
    assert_eq!(
        got.sculpt.pre_cover, want.sculpt.pre_cover,
        "{what}: sculpt.pre_cover"
    );
    assert_eq!(
        got.sculpt.pre_mats, want.sculpt.pre_mats,
        "{what}: sculpt.pre_mats"
    );
    assert_eq!(
        got.sculpt.pre_rgba, want.sculpt.pre_rgba,
        "{what}: sculpt.pre_rgba"
    );
}

/// **O oráculo A7 (o do ADR-0117), por PLANO.** Ida e volta byte-idêntica, com os dezenove planos
/// diferindo entre os endpoints.
///
/// ⚠️ É o gate que pega o modo de falha de CORREÇÃO do delta: um plano esquecido na materialização volta
/// como o do **cursor** — isto é, o undo simplesmente não desfaz aquele plano, e os outros dezoito
/// continuam perfeitos. Nada mais no sistema pisca.
#[test]
fn every_plane_of_a_snapshot_survives_the_round_trip() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    let before = model_all_planes(0x10);
    let after = model_all_planes(0x40);
    c.record_structural(before.clone(), after.clone());

    let back = c.undo().expect("um passo a desfazer");
    assert_same_planes(&back, &before, "undo");
    let fwd = c.redo().expect("um passo a refazer");
    assert_same_planes(&fwd, &after, "redo");
    // …e a segunda volta usa um cursor que a primeira instalou: é aqui que um cursor que não anda com a
    // história (ou que ficou preso no estado vivo) produz o segundo undo errado.
    let back2 = c.undo().expect("de novo");
    assert_same_planes(&back2, &before, "undo (2a volta)");
}

/// Uma CADEIA de passos desfaz na ordem inversa, cada um restaurando exatamente o seu endpoint.
///
/// ⚠️ Este é o gate do CURSOR. Um delta sozinho está sempre certo; o que pode estar errado é a base de
/// que ele parte, e ela só é observável a partir do SEGUNDO passo — encadear é a única forma de perguntar.
#[test]
fn a_chain_of_deltas_walks_back_through_every_state() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    // Seeds cujas colunas (`seed % 6`) sao todas DISTINTAS: cada estado toca outro lugar.
    let states: Vec<ModelSnapshot> = [10u8, 21, 32, 43, 54, 65]
        .into_iter()
        .map(model_all_planes)
        .collect();
    for w in states.windows(2) {
        c.record_structural(w[0].clone(), w[1].clone());
    }
    for i in (0..states.len() - 1).rev() {
        let back = c.undo().expect("passo");
        assert_same_planes(&back, &states[i], &format!("undo até o estado {i}"));
    }
    for (i, want) in states.iter().enumerate().skip(1) {
        let fwd = c.redo().expect("passo");
        assert_same_planes(&fwd, want, &format!("redo até o estado {i}"));
    }
}

/// **O cap é em BYTES, não em passos** — a mutação que o plano 26 prescreve.
///
/// Com um orçamento apertado o histórico descarta os passos mais ANTIGOS e o retido para de crescer,
/// enquanto a contagem de passos continuaria subindo. ⚠️ E ele nunca desce abaixo de UM passo: uma
/// edição de camada inteira é irredutivelmente uma camada por passo (ADR-0117), e um cap que come o
/// único passo possível é um histórico que não existe.
#[test]
fn the_history_is_capped_in_bytes_not_in_steps() {
    // Cada entrada aqui guarda dois lados de um plano de 64 B (o `Whole`, porque a janela é o plano
    // inteiro) → ~128 B. Um orçamento de 300 B segura duas, não vinte.
    let mut c = UndoController::new(300);
    for v in 0..20u8 {
        c.record_structural(model(v), model(v + 1));
    }
    assert!(
        c.retained_bytes() <= 300 || c.undo_depth() == 1,
        "o cap nao segurou: {} bytes em {} passos",
        c.retained_bytes(),
        c.undo_depth()
    );
    assert!(c.undo_depth() >= 1, "o cap comeu o unico passo possivel");
    assert!(
        c.undo_depth() < 20,
        "vinte passos cabendo em 300 bytes: o cap esta contando PASSOS, nao BYTES"
    );
    // E o passo que sobreviveu ainda desfaz de verdade.
    let back = c.undo().expect("o passo mais recente");
    assert_eq!(back.canvas_rgba.as_slice(), &[19; 16]);
}

/// Um orçamento generoso **não morde**: o cap existe para o irredutível, não para racionar traços.
#[test]
fn a_generous_budget_keeps_every_step() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    for v in 0..40u8 {
        c.record_structural(model(v), model(v + 1));
    }
    assert_eq!(c.undo_depth(), 40);
}

/// Os bytes retidos **voltam** quando a ramificação de redo é descartada por uma edição nova, e quando o
/// histórico é limpo. Um contador que só sobe é um cap que aperta cedo demais e nunca solta.
#[test]
fn the_byte_ledger_gives_the_memory_back() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    c.record_structural(model_all_planes(1), model_all_planes(2));
    c.record_structural(model_all_planes(2), model_all_planes(3));
    let two = c.retained_bytes();
    c.undo();
    assert_eq!(
        c.retained_bytes(),
        two,
        "um undo MOVE bytes de pilha, nao os devolve"
    );
    // Uma edição nova mata o redo — e os bytes dele.
    c.record_structural(model_all_planes(2), model_all_planes(9));
    assert!(
        c.retained_bytes() <= two,
        "a ramificacao de redo descartada nao devolveu os bytes: {} > {two}",
        c.retained_bytes()
    );
    c.clear();
    assert_eq!(c.retained_bytes(), 0);
}

/// Um plano que o stride **não consegue medir** é guardado INTEIRO, nunca dado como inalterado.
///
/// ⚠️ Este gate existe por um defeito que eu escrevi e peguei relendo: `diff_window` devolve `None` para
/// *"idênticos"*, e a 1ª versão do `split` também caía nele quando o stride não dividia o plano — os dois
/// buffers diferiam, o passo gravava `Unchanged`, e o undo **perdia a edição em silêncio**. As duas
/// perguntas (*sei medir?* e *diferem?*) têm de ser separadas, e é isso que o `fits` faz.
///
/// O caso não é hipotético: um snapshot tirado antes do `set_source` carrega `canvas_size` `(0, 0)`.
#[test]
fn a_plane_the_stride_cannot_measure_is_stored_whole_not_unchanged() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    let mut before = model_all_planes(3);
    let mut after = model_all_planes(9);
    // A largura que o snapshot declara não mede plano nenhum destes.
    before.canvas_size = (0, 0);
    after.canvas_size = (0, 0);
    c.record_structural(before.clone(), after.clone());
    let back = c.undo().expect("um passo");
    assert_same_planes(&back, &before, "undo sem stride utilizavel");
    let fwd = c.redo().expect("um passo");
    assert_same_planes(&fwd, &after, "redo sem stride utilizavel");
}

/// **Um run coalescido recompõe o delta, não concatena dois.**
///
/// N presses de Simplify colapsam num passo cujo `before` é o estado antes do PRIMEIRO. Com o histórico
/// por delta isso deixou de ser *"troque o `after` do topo"*: o `before` guardado está esvaziado, então
/// ele é materializado do cursor ANTIGO e re-partido contra o `after` novo. ⚠️ Concatenar os dois deltas
/// seria a alternativa óbvia e ela erra quando as duas janelas se sobrepõem — o segundo passo escreveria
/// por cima do primeiro na ordem errada.
#[test]
fn a_coalesced_run_recomposes_the_delta_instead_of_stacking_two() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    let s0 = model_all_planes(10);
    let s1 = model_all_planes(21);
    let s2 = model_all_planes(32);
    c.record_structural_coalesced(CoalesceKind::Simplify, s0.clone(), s1.clone());
    c.record_structural_coalesced(CoalesceKind::Simplify, s1, s2.clone());
    assert_eq!(c.undo_depth(), 1, "dois presses = um passo");
    let back = c.undo().expect("o passo colapsado");
    assert_same_planes(&back, &s0, "undo de um run coalescido");
    let fwd = c.redo().expect("de volta");
    assert_same_planes(&fwd, &s2, "redo de um run coalescido");
}
