//! Gates dos CANAIS do `motion.drive` — cortados do `channel.rs` no teto de LOC do HR-18.
//!
//! O corte é por RESPONSABILIDADE: ali mora *que canal recebe e como o stream é escrito*, e
//! aqui *o que cada um deles tem de fazer*.
use super::*;

/// **Todo canal que o nó ROTEIA tem um chip no menu.**
///
/// ⚠️ **Achado por mutação em 2026-08-18:** apagar um rótulo da lista deixa a suíte
/// inteira VERDE e o canal do fim **inalcançável** — ele continua a existir no
/// `channel_column`, no braço da CPU e no `variant_by_param`, e nenhuma superfície o
/// oferece. ⚠️ E o `max` do hint **não** é quem o guarda: medido, a row de enum do
/// painel clampa por `labels.len()` e **ignora `min`/`max`**, então para um enum
/// aquela faixa é decorativa — um gate sobre ela pinaria uma lei que o produto não
/// consome (foi escrito, medido e descartado, e ele acusava três nós corretos).
///
/// A régua é o **ÚLTIMO índice implementado**, o único número que cresce quando um
/// canal nasce.
#[test]
fn every_channel_the_node_routes_has_a_chip_in_the_menu() {
    let hint = crate::PARAM_HINTS
        .iter()
        .find(|h| h.param == "channel")
        .expect("o canal tem hint");
    let ph2d_node_registry::ParamWidget::Enum { labels } = hint.widget else {
        panic!("o canal e' um enum");
    };
    // CONTROLE: a varredura achou a lista do CANAL, e nao uma vazia.
    assert!(labels.contains(&"Custom…"), "os rotulos sao os do canal");
    assert_eq!(
        labels.len() as i32,
        CH_SIZE_Y + 1,
        "a lista de chips tem de cobrir 0..={}; ela oferece {}",
        CH_SIZE_Y,
        labels.len()
    );
}

fn sizes(out: &Stream) -> Vec<[f32; 2]> {
    match out.get("size").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("size e' Vec2"),
    }
}

/// Uma fileira com tamanhos NÃO-uniformes e NÃO-unitários de propósito: com
/// `[1,1]` o eixo intocado seria indistinguível da identidade que o
/// `base_vec2` inventa, e com `x == y` a troca de eixo passaria.
fn sized_row() -> Stream {
    Stream::new(3).with(
        "size",
        Column::Vec2(vec![[0.5, 2.0], [1.5, 0.25], [3.0, 0.75]]),
    )
}

/// **Dirigir UM eixo deixa o outro exactamente onde estava.**
#[test]
fn driving_one_size_axis_leaves_the_other_alone() {
    let input = sized_row();
    let before = sizes(&input);

    let x = sizes(&drive_channel(
        &input,
        CH_SIZE_X,
        &[7.0],
        1.0,
        Combine::Set,
        false,
    ));
    for (i, s) in x.iter().enumerate() {
        assert_eq!(s[0], 7.0, "o eixo X recebe (elemento {i})");
        assert_eq!(
            s[1].to_bits(),
            before[i][1].to_bits(),
            "o eixo Y fica AO BIT (elemento {i})"
        );
    }

    let y = sizes(&drive_channel(
        &input,
        CH_SIZE_Y,
        &[7.0],
        1.0,
        Combine::Set,
        false,
    ));
    for (i, s) in y.iter().enumerate() {
        assert_eq!(s[1], 7.0, "o eixo Y recebe (elemento {i})");
        assert_eq!(
            s[0].to_bits(),
            before[i][0].to_bits(),
            "o eixo X fica AO BIT (elemento {i})"
        );
    }
}

/// **O canal `Size` de sempre continua a escrever os DOIS**, ao bit.
///
/// ⚠️ Este é o CONTROLE que impede a wave de mudar o mundo que já shipava: sem ele,
/// um braço novo que roubasse o `Size` passaria pelos gates dos eixos.
#[test]
fn the_uniform_size_channel_still_writes_both_axes() {
    let out = sizes(&drive_channel(
        &sized_row(),
        3,
        &[7.0],
        1.0,
        Combine::Set,
        false,
    ));
    for (i, s) in out.iter().enumerate() {
        assert_eq!((s[0], s[1]), (7.0, 7.0), "os dois eixos (elemento {i})");
    }
}

/// **DOIS campos independentes, um por eixo — o que a composição NÃO dava.**
///
/// Medido antes de construir (`measure_size_axes`): `drive(Size) → motion.scale`
/// dá anisotropia com razão FIXA (o `motion.scale` escala com um PARAM, igual para
/// toda peça) e o `Custom…` **recusa** escrever um escalar sobre um `Vec2`. Aqui
/// os dois eixos recebem campos que **não são múltiplos um do outro**, e nenhuma
/// razão constante reproduz isso.
#[test]
fn two_independent_fields_can_drive_the_two_axes() {
    let input = Stream::new(3).with("size", Column::Vec2(vec![[1.0, 1.0]; 3]));
    let step1 = drive_channel(
        &input,
        CH_SIZE_X,
        &[1.0, 2.0, 3.0],
        1.0,
        Combine::Set,
        false,
    );
    let out = sizes(&drive_channel(
        &step1,
        CH_SIZE_Y,
        &[3.0, 1.0, 2.0],
        1.0,
        Combine::Set,
        false,
    ));
    assert_eq!(out, vec![[1.0, 3.0], [2.0, 1.0], [3.0, 2.0]]);
    // O discriminante: a razão x/y MUDA de peça para peça. Uma anisotropia fixa
    // (o que o `motion.scale` dá) teria a MESMA razão nas três.
    let r: Vec<f32> = out.iter().map(|s| s[0] / s[1]).collect();
    assert!(
        (r[0] - r[1]).abs() > 0.1 && (r[1] - r[2]).abs() > 0.1,
        "as razoes tem de diferir entre pecas, e medem {r:?}"
    );
}

/// **A máscara alcança o eixo** — `falloff = 0` deixa o tamanho onde estava, ao bit.
#[test]
fn a_masked_element_keeps_its_size_axis() {
    let input = Stream::new(2)
        .with("size", Column::Vec2(vec![[0.5, 2.0], [1.5, 0.25]]))
        .with("falloff", Column::Scalar(vec![1.0, 0.0]));
    let out = sizes(&drive_channel(
        &input,
        CH_SIZE_X,
        &[9.0],
        1.0,
        Combine::Set,
        false,
    ));
    assert_eq!(out[0][0], 9.0, "falloff 1 leva o drive inteiro");
    assert_eq!(
        out[1][0].to_bits(),
        1.5f32.to_bits(),
        "falloff 0 nao deixa o drive tocar o eixo"
    );
}

#[test]
fn a_length_one_value_broadcasts_to_every_instance() {
    // Three instances, ONE value (2.0) → all three shift by 2 in X.
    let input = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]));
    let out = drive_channel(&input, 0, &[2.0], 1.0, Combine::Add, false);
    match out.get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v, &vec![[2.0, 0.0], [3.0, 0.0], [4.0, 0.0]]),
        _ => panic!(),
    }
}

#[test]
fn a_length_n_value_applies_element_wise() {
    let input = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3]));
    let out = drive_channel(&input, 0, &[1.0, 2.0, 3.0], 1.0, Combine::Add, false);
    match out.get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v, &vec![[1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
        _ => panic!(),
    }
}

#[test]
fn set_and_multiply_combine_against_the_existing_channel() {
    let input = Stream::new(1).with("P", Column::Vec2(vec![[5.0, 0.0]]));
    let set = drive_channel(&input, 0, &[2.0], 1.0, Combine::Set, false);
    let mul = drive_channel(&input, 0, &[2.0], 1.0, Combine::Multiply, false);
    assert_eq!(px(&set), 2.0, "set overwrites");
    assert_eq!(px(&mul), 10.0, "multiply scales the existing 5");
}

#[test]
fn falloff_zero_leaves_the_channel_untouched() {
    let input = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]))
        .with("falloff", Column::Scalar(vec![1.0, 0.0]));
    let out = drive_channel(&input, 0, &[3.0], 1.0, Combine::Add, false);
    match out.get("P").unwrap() {
        Column::Vec2(v) => {
            assert_eq!(v[0], [3.0, 0.0], "focused instance driven");
            assert_eq!(v[1], [0.0, 0.0], "masked instance untouched");
        }
        _ => panic!(),
    }
}

#[test]
fn size_channel_drives_both_components_from_unit_identity() {
    // A bare P-only stream driven on Size (multiply by 2) → unit×2 on both.
    let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
    let out = drive_channel(&input, 3, &[2.0], 1.0, Combine::Multiply, false);
    match out.get("size").unwrap() {
        Column::Vec2(v) => assert_eq!(v[0], [2.0, 2.0], "unit identity × 2"),
        _ => panic!(),
    }
}

fn px(s: &Stream) -> f32 {
    match s.get("P").unwrap() {
        Column::Vec2(v) => v[0][0],
        _ => panic!(),
    }
}

// ── Os gates do canal da MÁSCARA (o antigo `mod falloff_tests`) ──

/// **A computed number becomes a MASK.** The wall five families hit at once (doc 89 §10.0):
/// the `falloff` column is what the `field.*`, the `force.*`, the deformers, the transforms
/// and the stylistics all read, and it could only ever be DERIVED FROM GEOMETRY. Nothing
/// computed — a noise, a luma, an age — could become one.
#[test]
fn a_value_field_becomes_the_mask_itself() {
    let input = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3]));
    let out = drive_channel(
        &input,
        CH_FALLOFF,
        &[0.25, 0.5, 1.0],
        1.0,
        Combine::Set,
        false,
    );
    match out
        .get("falloff")
        .expect("the mask is now a column anyone can write")
    {
        Column::Scalar(v) => assert_eq!(v, &vec![0.25, 0.5, 1.0]),
        _ => panic!("the mask is a scalar column"),
    }
}

/// **An absent mask is `1.0`, not `0.0`** — every reader in the library falls back to full
/// effect (`falloff_at`), so a writer that started from zero would disagree with all of them
/// about what "no mask" means, and `Add` would silently halve the world.
#[test]
fn an_absent_mask_starts_at_full_effect() {
    let input = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0]; 2]));
    let out = drive_channel(&input, CH_FALLOFF, &[0.0], 1.0, Combine::Add, false);
    match out.get("falloff").unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![1.0, 1.0], "1.0 + 0.0, not 0.0 + 0.0"),
        _ => panic!(),
    }
}

/// **The mask does not mask ITSELF.** Every other channel lerps its result toward the
/// original by `falloff`; doing that here would make `Set` non-idempotent and would mean
/// nothing to an artist. ⚠️ On the device this is not a choice at all — a variant whose
/// target is `falloff` binds it once as `ReadWrite`, so the common read is absent and the
/// self-mask is *inexpressible*.
#[test]
fn the_mask_does_not_mask_itself() {
    let input = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 2]))
        .with("falloff", Column::Scalar(vec![0.0, 0.5]));
    let out = drive_channel(&input, CH_FALLOFF, &[1.0], 1.0, Combine::Set, false);
    match out.get("falloff").unwrap() {
        // Self-masking would have pinned the first element at 0.0 forever — a mask you
        // could never turn back on.
        Column::Scalar(v) => assert_eq!(v, &vec![1.0, 1.0]),
        _ => panic!(),
    }
}

/// **A NEGATIVE weight survives** — and it is not an oversight. A negative `falloff` inverts
/// the force that consumes it, which the conference found is ours alone (C4D and Cavalry are
/// `[0,1]` by construction). Clamping here would delete the capability before anyone used it.
#[test]
fn a_negative_mask_is_not_clamped_away() {
    let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
    let out = drive_channel(&input, CH_FALLOFF, &[-1.0], 1.0, Combine::Set, false);
    match out.get("falloff").unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![-1.0]),
        _ => panic!(),
    }
}

/// **The five channels that shipped are untouched** — the new one is a sixth index, so
/// already-authored art cannot move.
#[test]
fn the_channels_that_shipped_are_untouched() {
    let input = Stream::new(1).with("P", Column::Vec2(vec![[3.0, 7.0]]));
    let out = drive_channel(&input, 0, &[1.0], 1.0, Combine::Add, false);
    match out.get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v, &vec![[4.0, 7.0]]),
        _ => panic!(),
    }
    assert!(out.get("falloff").is_none(), "driving X invents no mask");
}
