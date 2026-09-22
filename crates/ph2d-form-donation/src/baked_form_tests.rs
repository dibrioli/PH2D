//! **Os gates do [`super`]** — irmão (`#[path]`), e o corte foi o tecto de LOC (700).
//!
//! ⚠️ Os cinco irmãos de teste que já viviam ao lado (`baked_form_lei_tests`,
//! `prova_da_placa`, `mede_o_acender_por_quadro`, `costura_residente_tests`) mostram que este
//! era o último bloco inline — ⛔ e a cura de um tecto é CORTE, nunca uma isenção.
use super::*;

/// **A FORMA SOBREVIVE À VIAGEM POR 8 BITS.**
///
/// ⚠️ A barra é a que a medição deu (`bake_form_bytes`), não um número escolhido: um
/// canal erra no máximo meio degrau (`1/510`), e depois da renormalização o desvio angular fica
/// abaixo de um grau. É o que faz de `≤ 3/255` no pixel aceso uma consequência e não uma
/// esperança.
#[test]
fn the_form_survives_the_round_trip() {
    // Direções variadas, cada uma unitária, mais um texel VAZIO (peso 0).
    let dirs: [[f32; 4]; 5] = [
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 0.0, 0.0, 1.0],
        [-0.577_35, 0.577_35, 0.577_35, 1.0],
        [0.267_26, -0.534_52, 0.801_78, 0.5],
        [0.0, 0.0, 0.0, 0.0],
    ];
    let flat: Vec<f32> = dirs.iter().flatten().copied().collect();
    let back = form_from_rgba8(&form_to_rgba8(&flat));

    for (i, want) in dirs.iter().enumerate() {
        let got = &back[i * 4..i * 4 + 4];
        assert!(
            (got[3] - want[3]).abs() <= 1.0 / 255.0,
            "o peso do texel {i} andou: {} contra {}",
            got[3],
            want[3]
        );
        if want[3] == 0.0 {
            // Texel vazio: a normal não significa nada, mas tem de ser um NÚMERO.
            assert!(
                got[..3].iter().all(|v| v.is_finite()),
                "texel vazio devolveu nao-numero: {got:?}"
            );
            continue;
        }
        let len = (got[0] * got[0] + got[1] * got[1] + got[2] * got[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 1e-5,
            "a normal {i} voltou com comprimento {len}, e a luz a le' como DIRECAO"
        );
        let dot = got[0] * want[0] + got[1] * want[1] + got[2] * want[2];
        assert!(
            dot > 0.999_8,
            "a normal {i} girou demais na viagem: cos = {dot}"
        );
    }
}

/// **UM CANAL SÓ ERRA MEIO DEGRAU** — a propriedade que torna a barra acima uma consequência.
///
/// ⚠️ RED sem o `+0,5` do [`quantise`]: truncar erra até um degrau inteiro, sempre para o mesmo
/// lado, e um viés sistemático numa normal é um deslocamento de brilho que se acumula em vez de
/// se cancelar.
#[test]
fn quantising_rounds_to_nearest_instead_of_biasing_down() {
    for step in 0..=1000 {
        let v = step as f32 / 1000.0;
        let back = f32::from(super::bytes::quantise(v)) / 255.0;
        assert!(
            (back - v).abs() <= 0.5 / 255.0 + 1e-6,
            "quantizar {v} devolveu {back}, mais de meio degrau de erro"
        );
    }
}

/// **MEXER EM QUALQUER LÂMPADA MOVE O CARIMBO — e o gate existe porque esquecer uma é
/// invisível.** Um carimbo que ignora a intensidade deixa o sprite aceso pelo rig anterior
/// enquanto o slider anda, e nada na tela diz que a luz é velha.
#[test]
fn every_way_the_rig_can_change_moves_the_stamp() {
    let base = LightRig::default();
    let here = rig_stamp(&base);
    assert_eq!(here, rig_stamp(&base), "premissa: e' estavel");

    for (name, mutate) in [
        (
            "azimute",
            (|r: &mut LightRig| r.current_mut().angle_deg += 30) as fn(&mut LightRig),
        ),
        ("elevacao", |r| {
            let e = r.current().elev_deg;
            r.current_mut().elev_deg = e + 10;
        }),
        ("intensidade", |r| r.current_mut().intensity *= 0.5),
    ] {
        let mut moved = base;
        mutate(&mut moved);
        assert_ne!(
            here,
            rig_stamp(&moved),
            "mexer em `{name}` tem de mover o carimbo"
        );
    }
}

/// **A LÂMPADA ANDA E O OBJETO RE-ACENDE — e um fracasso não finge que acendeu.**
///
/// ⚠️ A segunda metade é a que tem consequência permanente: carimbar uma acendida que não
/// aconteceu (um rig todo apagado, por exemplo) deixaria os pixels marcados como *"acesos por
/// este rig"*, e a próxima lâmpada acesa não os re-acenderia **nunca mais**.
#[test]
fn a_lamp_that_moved_relights_and_a_failure_does_not_pretend_it_did() {
    let a = rig_stamp(&LightRig::default());
    let mut moved_rig = LightRig::default();
    moved_rig.current_mut().angle_deg += 45;
    let b = rig_stamp(&moved_rig);
    assert_ne!(a, b, "premissa: o rig andou");

    assert!(needs_relight(None, a), "nunca aceso pede acendida");
    assert!(needs_relight(Some(a), b), "a lampada andou: pede de novo");
    assert!(!needs_relight(Some(a), a), "parado nao pede nada");

    assert_eq!(stamp_after(true, b, Some(a)), Some(b), "acendeu: carimba");
    assert_eq!(
        stamp_after(false, b, Some(a)),
        Some(a),
        "falhou: o carimbo VELHO fica, senao a proxima lampada acesa nao re-acende"
    );
    assert_eq!(
        stamp_after(false, b, None),
        None,
        "e o nunca-aceso continua"
    );
}

/// **UM OBJETO RECÉM-CARREGADO PEDE ACENDIDA, E O RIG QUE ELE PEDE É O DELE.**
///
/// ⚠️ Esta é a metade que o load precisa e que nenhuma das outras cobre: um objeto que volta do
/// arquivo nunca foi aceso *nesta sessão* (`lit_with: None`), então ele **tem** de disparar; e o
/// rig que a acendida usa é o que veio no documento, não o que a cena tem na mão.
#[test]
fn a_form_that_came_from_a_file_asks_to_be_lit_by_its_own_rig() {
    let mut authored = LightRig::default();
    authored.current_mut().angle_deg += 90;
    let loaded = BakedForm {
        form_occ: Vec::new(),
        size: (2, 1),
        base: vec![255; 8],
        form: vec![0.0; 8],
        texture_id: 7,
        rig: authored,
        lit_with: None,
        lei: crate::lei_da_luz::Lei::default(),
        materia_da_forma: false,
        recorte: None,
    };
    assert!(
        needs_relight(loaded.lit_with, rig_stamp(&loaded.rig)),
        "recem-carregado tem de pedir acendida"
    );
    assert_ne!(
        rig_stamp(&loaded.rig),
        rig_stamp(&LightRig::default()),
        "premissa: o rig autorado NAO e' o default -- senao o gate nao distingue os dois"
    );
}
