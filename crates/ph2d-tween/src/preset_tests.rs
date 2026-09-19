//! Os gates dos [`super`] — o açúcar tem de produzir algo que FUNCIONA.

use super::*;
use crate::{Relogio, valor};

/// ⭐⭐⭐ **TODO preset produz um tween que se MEXE** — e o gate mede o valor, nunca os campos.
///
/// ⚠️ *Um preset que escrevesse `de == para` seria um botão que não faz nada*, e ele lê-se
/// exactamente como um botão morto — o defeito que esta casa mede desde a caça de 30/08.
#[test]
fn todo_preset_produz_um_tween_que_se_mexe() {
    for p in Preset::ALL {
        let t = p.tween();
        let a = valor(&t, Relogio::a_correr(0.0)).expect("a correr, ele escreve");
        let b = valor(&t, Relogio::a_correr(1.0)).expect("idem");
        assert_ne!(a, b, "{:?} nao move nada", p.label());
        // …e o meio não é nenhum dos dois extremos: ele ATRAVESSA.
        let m = valor(&t, Relogio::a_correr(0.5)).unwrap();
        assert_ne!(m, a, "{}: o meio e' o principio", p.label());
        assert_ne!(m, b, "{}: o meio e' o fim", p.label());
    }
}

/// ⭐⭐ **O FLASH volta à arte, e o FADE-OUT fica** — as duas metades do `AoAcabar`, medidas pelo
/// que a lei escreve no fim.
///
/// ⚠️ **É a diferença que torna os dois presets diferentes**, e ela não está nos valores: está no
/// que acontece quando o relógio acaba. *Um flash com `Hold` deixaria o objecto uma silhueta acesa
/// para sempre.*
#[test]
fn o_flash_volta_a_arte_e_o_fade_out_fica() {
    assert_eq!(
        valor(&Preset::Flash.tween(), Relogio::ACABOU),
        None,
        "o flash tem de DEIXAR DE ESCREVER — e' assim que a arte volta"
    );
    let fim = valor(&Preset::FadeOut.tween(), Relogio::ACABOU).expect("o fade-out FICA");
    assert!(fim[0].abs() < 1e-6, "o fade-out acabou a {}", fim[0]);
    let entra = valor(&Preset::FadeIn.tween(), Relogio::ACABOU).expect("o fade-in FICA");
    assert!(
        (entra[0] - 1.0).abs() < 1e-6,
        "o fade-in acabou a {}",
        entra[0]
    );
}

/// ⚠️ **O `Flash` escreve a SILHUETA**, e é isso que faz a ponte ligar o `tint_fill`.
#[test]
fn o_flash_escreve_a_silhueta() {
    assert_eq!(Preset::Flash.tween().canal, Canal::Silhueta);
    assert_eq!(
        Preset::Flash.tween().canal.aridade(),
        4,
        "uma cor sao quatro"
    );
}

/// ⭐⭐ **A DURAÇÃO é parte do preset, e é ela que faz o «um clique»** — sem isto o artista fica com
/// um flash de **um segundo** (o valor de fábrica do timer), que é oito vezes mais lento do que a
/// coisa que ele pediu, e lê-se como *«o preset não funcionou»*.
///
/// ⚠️ **A cerca é o QUADRO**, e o gate escreve-a: um pisca tem de durar mais do que uns poucos
/// quadros (senão é um artefacto) e menos do que uma dúzia e meia (senão é uma mudança de cor).
#[test]
fn a_duracao_de_um_pisca_cabe_em_quadros_que_se_veem() {
    const QUADRO_US: u64 = 1_000_000 / 60;
    let quadros = Preset::Flash.duracao_us() / QUADRO_US;
    assert!(
        (4..=15).contains(&quadros),
        "o flash dura {quadros} quadros a 60 Hz"
    );
    // …e um fade é de outra ordem: ele tem de SER VISTO como transição.
    for p in [Preset::FadeIn, Preset::FadeOut] {
        let q = p.duracao_us() / QUADRO_US;
        assert!(
            q >= 12,
            "{} dura so' {q} quadros — le^-se como um corte",
            p.label()
        );
    }
    assert!(
        Preset::Flash.duracao_us() < Preset::FadeOut.duracao_us(),
        "um pisca tem de ser mais curto que um desvanecer"
    );
}

/// **A tag é a POSIÇÃO, e a ida-e-volta fecha** — a mesma lei dos outros três selectores.
#[test]
fn a_tag_de_um_preset_e_a_posicao() {
    for (i, p) in Preset::ALL.iter().enumerate() {
        assert_eq!(p.tag() as usize, i);
        assert_eq!(Preset::from_tag(p.tag()), *p);
        assert!(!p.label().is_empty());
    }
    assert_eq!(Preset::from_tag(200), Preset::FadeIn);
}
