//! Os gates da lei do canvas — a tabela é a do ORÁCULO, e está no doc da crate.
//!
//! ⚠️ **A janela do oráculo é `720×450`**, logo a vista equivalente tem meia-janela `[360, 225]`.
//! Os números da direita saíram de `godot_hud_probe.gd` (bloco L2) e estão aqui **verbatim**.

// ⚠️ **Os zeros à direita são a precisão IMPRESSA pelo oráculo** (`%.6f`), e os números estão
// aqui verbatim de propósito: é o que torna a tabela conferível contra a saída da sonda sem
// ninguém ter de a reformatar de cabeça. O clippy quer `2.25`; nós queremos `2.250_000`.
#![allow(clippy::excessive_precision, reason = "a tabela é verbatim do oráculo")]

use super::{Canvas, Fit, View, bands, place};

/// A vista que reproduz a janela do oráculo.
fn vista_do_oraculo() -> View {
    View {
        center: [0.0, 0.0],
        half: [360.0, 225.0],
    }
}

/// `ignore` — um factor por eixo, e **nenhum** deles é o do outro.
#[test]
fn a_lei_do_stretch_e_um_factor_por_eixo() {
    // (ref_w, ref_h, escala_x, escala_y) — verbatim do oráculo, `aspect=ignore`.
    const ORACULO: [(f32, f32, f32, f32); 5] = [
        (320.0, 180.0, 2.250_000, 2.500_000),
        (640.0, 360.0, 1.125_000, 1.250_000),
        (1280.0, 360.0, 0.562_500, 1.250_000),
        (320.0, 480.0, 2.250_000, 0.937_500),
        (500.0, 400.0, 1.440_000, 1.125_000),
    ];
    for (rw, rh, ex, ey) in ORACULO {
        let c = Canvas::new(rw, rh, Fit::Stretch).expect("caixa válida");
        let p = place(&c, vista_do_oraculo());
        assert!(
            (p.scale[0] - ex).abs() <= 1e-6 && (p.scale[1] - ey).abs() <= 1e-6,
            "ref {rw}×{rh}: nós {:?}, o oráculo ({ex}, {ey})",
            p.scale
        );
    }
}

/// `keep` — uniforme, e a BANDA é a que o oráculo imprime **menos o arredondamento dele**.
///
/// ⛔ As duas últimas linhas são a **divergência declarada**: ali o alvo arredonda a banda a
/// inteiro e encolhe a escala para caber, e nós ficamos com o valor exacto.
#[test]
fn a_lei_do_keep_e_a_do_oraculo_menos_o_arredondamento() {
    // (ref_w, ref_h, escala_do_oraculo_x, banda_do_oraculo_x, banda_do_oraculo_y)
    const ORACULO: [(f32, f32, f32, f32, f32); 5] = [
        (320.0, 180.0, 2.250_000, 0.0, 23.0),
        (640.0, 360.0, 1.125_000, 0.0, 23.0),
        (1280.0, 360.0, 0.562_500, 0.0, 124.0),
        (320.0, 480.0, 0.937_500, 210.0, 0.0),
        (500.0, 400.0, 1.124_000, 79.0, 0.0),
    ];
    for (rw, rh, es, bx, by) in ORACULO {
        let c = Canvas::new(rw, rh, Fit::Keep).expect("caixa válida");
        let v = vista_do_oraculo();
        let p = place(&c, v);
        assert!(
            (p.scale[0] - p.scale[1]).abs() <= f32::EPSILON,
            "ref {rw}×{rh}: o keep NÃO pode distorcer, e deu {:?}",
            p.scale
        );
        let esperado = (2.0 * v.half[0] / rw).min(2.0 * v.half[1] / rh);
        assert!(
            (p.scale[0] - esperado).abs() <= 1e-6,
            "ref {rw}×{rh}: a escala é o MÍNIMO dos dois factores"
        );
        // A escala do oráculo é a nossa a menos do arredondamento dele (< 1 px de banda).
        assert!(
            (p.scale[0] - es).abs() <= 3e-3,
            "ref {rw}×{rh}: nós {:.6}, o oráculo {es:.6}",
            p.scale[0]
        );
        let b = bands(&c, v);
        assert!(
            (b[0] - bx).abs() <= 1.0 && (b[1] - by).abs() <= 1.0,
            "ref {rw}×{rh}: bandas nossas {b:?}, do oráculo ({bx}, {by})"
        );
    }
}

/// ⛔ A DIVERGÊNCIA tem de EXISTIR, senão o tecto acima vira licença.
///
/// Nas duas referências em que o alvo arredonda, o número dele **não** é o exacto — e é isso que
/// esta metade afirma. *Um gate que tolera uma diferença sem nunca a exigir deixa de a descrever.*
#[test]
fn onde_o_alvo_arredonda_nos_ficamos_com_o_exacto() {
    let v = vista_do_oraculo();
    // ref 1280×360: banda exacta 123,75 — o alvo imprimiu 124.
    let c = Canvas::new(1280.0, 360.0, Fit::Keep).expect("caixa válida");
    let b = bands(&c, v);
    assert!(
        (b[1] - 123.75).abs() <= 1e-4,
        "a banda exacta é 123,75 e deu {}",
        b[1]
    );
    assert!(
        (b[1] - 124.0).abs() > 1e-3,
        "se a nossa banda fosse a arredondada do alvo, a divergência não existia"
    );
    // ref 500×400: banda exacta 78,75 — o alvo imprimiu 79 e encolheu a escala para 1,124.
    let c = Canvas::new(500.0, 400.0, Fit::Keep).expect("caixa válida");
    assert!((bands(&c, v)[0] - 78.75).abs() <= 1e-4);
    assert!(
        (place(&c, v).scale[0] - 1.125).abs() <= 1e-6,
        "a nossa escala é o mínimo exacto (1,125), não o 1,124 que o arredondamento dele produz"
    );
}

/// A translação é o centro da vista — nos dois modos, e com a vista fora da origem.
#[test]
fn a_translacao_e_sempre_o_centro_da_vista() {
    let v = View {
        center: [12.5, -7.25],
        half: [16.0, 9.0],
    };
    for fit in [Fit::Keep, Fit::Stretch] {
        let c = Canvas::new(32.0, 18.0, fit).expect("caixa válida");
        assert_eq!(place(&c, v).translate, [12.5, -7.25], "modo {fit:?}");
    }
}

/// O neutro é EXACTO: caixa do tamanho da vista ⇒ escala `1` e bandas `0`, por divisão de iguais.
#[test]
fn o_neutro_e_exacto_nos_dois_modos() {
    let v = View {
        center: [0.0, 0.0],
        half: [16.0, 9.0],
    };
    for fit in [Fit::Keep, Fit::Stretch] {
        let c = Canvas::new(32.0, 18.0, fit).expect("caixa válida");
        let p = place(&c, v);
        assert_eq!(p.scale, [1.0, 1.0], "modo {fit:?}");
        assert_eq!(bands(&c, v), [0.0, 0.0], "modo {fit:?}");
    }
}

/// O CONTROLO dos dois modos: com aspectos diferentes, um distorce e o outro não.
///
/// ⚠️ Sem esta metade, um `Stretch` que por engano calculasse o mínimo passaria em tudo o que está
/// acima — as tabelas do oráculo têm linhas em que os dois modos coincidem.
#[test]
fn o_stretch_distorce_onde_o_keep_nao_distorce() {
    // ⚠️ meia-janela `[48, 9]` e não `[32, 9]`: com esta a diferença dá `2,0` e com aquela dá
    // EXACTAMENTE `1,0`, que era a barra — *uma fixtura que aterra em cima da barra não a testa.*
    let v = View {
        center: [0.0, 0.0],
        half: [48.0, 9.0],
    };
    let k = place(&Canvas::new(32.0, 18.0, Fit::Keep).expect("k"), v);
    let s = place(&Canvas::new(32.0, 18.0, Fit::Stretch).expect("s"), v);
    assert!(
        (k.scale[0] - k.scale[1]).abs() <= f32::EPSILON,
        "o keep não distorce"
    );
    assert!(
        (s.scale[0] - s.scale[1]).abs() > 1.0,
        "o stretch TEM de distorcer aqui, e deu {:?}",
        s.scale
    );
}

/// Uma caixa que não é um rectângulo utilizável é RECUSADA — e a recusa é a lei.
#[test]
fn uma_caixa_impossivel_e_recusada() {
    for (w, h) in [
        (0.0, 18.0),
        (32.0, 0.0),
        (-1.0, 18.0),
        (f32::NAN, 18.0),
        (32.0, f32::INFINITY),
    ] {
        assert!(
            Canvas::new(w, h, Fit::Keep).is_none(),
            "caixa {w}×{h} tinha de ser recusada"
        );
    }
    assert!(
        Canvas::new(32.0, 18.0, Fit::Keep).is_some(),
        "o CONTROLO positivo"
    );
}

/// O texto de um número: a contagem crua, e o tempo com uma casa e **nunca negativo**.
#[test]
fn o_tempo_que_ja_passou_mostra_zero_e_nunca_um_negativo() {
    use super::{Valor, formata};
    assert_eq!(formata(Valor::Inteiro(0)), "0");
    assert_eq!(
        formata(Valor::Inteiro(-3)),
        "-3",
        "uma CONTAGEM pode ser negativa (dívida, vidas a menos)"
    );
    assert_eq!(formata(Valor::Segundos(3.25)), "3.2", "uma casa decimal");
    assert_eq!(formata(Valor::Segundos(0.0)), "0.0");
    // ⭐ o caso que a lei existe para cobrir: o relógio passou do fim.
    assert_eq!(formata(Valor::Segundos(-1.3)), "0.0");
}

/// As QUATRO células do bloco L3 do oráculo, uma a uma.
#[test]
fn um_botao_dispara_ao_largar_e_so_se_os_dois_toques_forem_dentro() {
    use super::{Gesto, clique};

    // (a) carregar DENTRO e largar DENTRO ⇒ publica, UMA vez.
    let (mem, pub_) = clique(Gesto::Baixo, None, Some(7));
    assert_eq!((mem, pub_), (Some(7), false), "o carregar nunca publica");
    let (mem, pub_) = clique(Gesto::Cima, mem, Some(7));
    assert_eq!((mem, pub_), (None, true));
    // e a memória ficou limpa ⇒ um segundo largar não repete.
    assert_eq!(clique(Gesto::Cima, mem, Some(7)), (None, false));

    // (b) carregar DENTRO, largar FORA ⇒ não publica.
    let (mem, _) = clique(Gesto::Baixo, None, Some(7));
    assert_eq!(clique(Gesto::Cima, mem, None), (None, false));

    // (c) carregar FORA, largar DENTRO ⇒ não publica.
    let (mem, _) = clique(Gesto::Baixo, None, None::<u8>);
    assert_eq!(clique(Gesto::Cima, mem, Some(7)), (None, false));

    // (d) carregar num botão e largar noutro ⇒ não publica.
    let (mem, _) = clique(Gesto::Baixo, None, Some(7));
    assert_eq!(clique(Gesto::Cima, mem, Some(9)), (None, false));

    // ⚠️ E um `Baixo` fora LIMPA a memória — senão o largar seguinte publicaria um botão em que o
    // dedo nunca pousou.
    let (mem, _) = clique(Gesto::Baixo, Some(7), None);
    assert_eq!(mem, None);
}
