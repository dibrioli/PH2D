//! Gates do Zig Zag. O oráculo central é o que separa este motor do do Illustrator:
//! **a mesma forma, autorada com mais âncoras, tem de ondular igual.**

use super::*;
use crate::arclen::arclen;
use crate::corner_live::segment;

const R: f64 = 60.0;

/// Um círculo em quatro cúbicas.
fn circle() -> Vec<VecVertex> {
    const K: f64 = 0.552_284_749_830_793_4;
    let p = [[R, 0.0], [0.0, R], [-R, 0.0], [0.0, -R]];
    let tang = [[0.0, K * R], [-K * R, 0.0], [0.0, -K * R], [K * R, 0.0]];
    (0..4)
        .map(|i| VecVertex {
            anchor: p[i],
            in_handle: [p[i][0] - tang[i][0], p[i][1] - tang[i][1]],
            out_handle: [p[i][0] + tang[i][0], p[i][1] + tang[i][1]],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect()
}

/// O MESMO círculo, mas com cada cúbica picada ao meio: geometria idêntica, oito âncoras.
///
/// ⚠️ A 1ª versão deste fixture punha `in_handle: a[0]` na âncora de junção — o que **quebra**
/// a alça que chega e faz dele outra curva. O gate abaixo apanhou isso e acusou o motor; o
/// culpado era o fixture. Um contorno remontado a partir de sub-cúbicas tem de encadear a
/// alça de SAÍDA de uma na de ENTRADA da seguinte.
fn circle_subdivided() -> Vec<VecVertex> {
    let v = circle();
    let n = v.len();
    let mut cubics = Vec::new();
    for i in 0..n {
        let seg = segment(&v, i, n);
        cubics.push(crate::arclen::subsegment(&seg, 0.0, 0.5));
        cubics.push(crate::arclen::subsegment(&seg, 0.5, 1.0));
    }
    let m = cubics.len();
    (0..m)
        .map(|k| VecVertex {
            anchor: cubics[k][0],
            in_handle: cubics[(k + m - 1) % m][2],
            out_handle: cubics[k][1],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect()
}

fn spec(amplitude: f64) -> ZigZagSpec {
    ZigZagSpec {
        amplitude,
        ridges: 6.0,
        smooth: false,
        rough_seed: None,
    }
}

/// A distância de cada âncora ao centro — a assinatura da onda num círculo.
fn radii(v: &[VecVertex]) -> Vec<f64> {
    v.iter().map(|w| w.anchor[0].hypot(w.anchor[1])).collect()
}

/// **O ponto neutro é um no-op byte-idêntico** — o que a pilha exige de todo efeito.
#[test]
fn the_neutral_point_is_a_byte_identical_no_op() {
    let v = circle();
    let (out, closed) = zigzag_contour(&v, true, &spec(0.0));
    assert_eq!(out, v);
    assert!(closed);
}

/// **A onda alterna para os DOIS lados** do caminho, e com a amplitude pedida.
#[test]
fn the_ridges_alternate_to_both_sides_by_the_amplitude() {
    let amp = 8.0;
    let (out, _) = zigzag_contour(&circle(), true, &spec(amp));
    let r = radii(&out);
    let (hi, lo) = (
        r.iter().copied().fold(f64::MIN, f64::max),
        r.iter().copied().fold(f64::MAX, f64::min),
    );
    assert!(
        (hi - (R + amp)).abs() < 0.5,
        "a crista externa devia chegar a {}, chegou a {hi}",
        R + amp
    );
    assert!(
        (lo - (R - amp)).abs() < 0.5,
        "a interna devia descer a {}, desceu a {lo}",
        R - amp
    );
}

/// **Ondular não abre nem fecha uma forma.**
#[test]
fn the_contour_keeps_its_closedness() {
    let (_, closed) = zigzag_contour(&circle(), true, &spec(5.0));
    assert!(closed);
    let open: Vec<VecVertex> = circle().into_iter().take(3).collect();
    let (_, still_open) = zigzag_contour(&open, false, &spec(5.0));
    assert!(!still_open);
}

/// **O GATE QUE IMPORTA: a mesma forma com mais âncoras ondula IGUAL.**
///
/// O Illustrator conta cristas *por segmento*, então picar uma aresta muda o desenho — o efeito
/// passa a descrever como o caminho foi autorado. Aqui as cristas são contadas no caminho
/// inteiro, por comprimento de ARCO, e a subdivisão é invisível.
#[test]
fn subdividing_the_path_does_not_change_the_wave() {
    let a = zigzag_contour(&circle(), true, &spec(7.0)).0;
    let b = zigzag_contour(&circle_subdivided(), true, &spec(7.0)).0;
    assert_eq!(
        a.len(),
        b.len(),
        "a contagem de cristas não pode depender da contagem de âncoras"
    );
    for (i, (p, q)) in a.iter().zip(b.iter()).enumerate() {
        let d = (p.anchor[0] - q.anchor[0]).hypot(p.anchor[1] - q.anchor[1]);
        assert!(
            d < 0.5,
            "crista {i}: {:?} vs {:?} — {d} de distância. As duas formas são a MESMA curva; \
             se a onda difere, o efeito está a medir a parametrização e não a geometria.",
            p.anchor,
            q.anchor
        );
    }
}

/// **Mais cristas = mais cristas** — o knob faz o que diz.
#[test]
fn asking_for_more_ridges_produces_more_ridges() {
    let few = zigzag_contour(
        &circle(),
        true,
        &ZigZagSpec {
            ridges: 4.0,
            ..spec(5.0)
        },
    )
    .0;
    let many = zigzag_contour(
        &circle(),
        true,
        &ZigZagSpec {
            ridges: 16.0,
            ..spec(5.0)
        },
    )
    .0;
    assert!(
        many.len() > few.len() * 3,
        "4 cristas deram {} âncoras e 16 deram {}",
        few.len(),
        many.len()
    );
}

/// **Suave produz alças; quina não.** É o que distingue a onda do serrote.
#[test]
fn smooth_gives_the_anchors_arms_and_corner_does_not() {
    let corner = zigzag_contour(&circle(), true, &spec(6.0)).0;
    assert!(
        corner
            .iter()
            .all(|v| v.in_handle == v.anchor && v.out_handle == v.anchor),
        "em quina a alça colapsa na âncora"
    );
    let smooth = zigzag_contour(
        &circle(),
        true,
        &ZigZagSpec {
            smooth: true,
            ..spec(6.0)
        },
    )
    .0;
    assert!(
        smooth.iter().all(|v| v.in_handle != v.anchor),
        "suave tem de dar braço a cada âncora, senão é o mesmo serrote"
    );
}

/// **Roughen é DETERMINÍSTICO**: a mesma seed desenha a mesma coisa, e seeds diferentes não.
/// Sem isto o documento mudaria de aparência entre execuções — e um save não descreveria a arte.
#[test]
fn roughen_is_deterministic_in_its_seed() {
    let rough = |seed| {
        zigzag_contour(
            &circle(),
            true,
            &ZigZagSpec {
                rough_seed: Some(seed),
                ..spec(9.0)
            },
        )
        .0
    };
    assert_eq!(
        radii(&rough(7)),
        radii(&rough(7)),
        "a mesma seed, o mesmo desenho"
    );
    assert_ne!(
        radii(&rough(7)),
        radii(&rough(8)),
        "seeds diferentes têm de dar desenhos diferentes, senão a seed é decoração"
    );
}

/// O Roughen ainda respeita a AMPLITUDE — aleatório não quer dizer sem teto.
#[test]
fn roughen_still_respects_the_amplitude() {
    let amp = 9.0;
    let out = zigzag_contour(
        &circle(),
        true,
        &ZigZagSpec {
            rough_seed: Some(3),
            ..spec(amp)
        },
    )
    .0;
    for r in radii(&out) {
        assert!(
            r <= R + amp + 0.5 && r >= R - amp - 0.5,
            "raio {r} saiu da faixa [{}, {}]",
            R - amp,
            R + amp
        );
    }
}

/// **A onda segue o CAMINHO**: cada crista fica a uma amplitude do círculo, não de um centro
/// qualquer. É o gate que pega uma normal trocada por uma direção global.
#[test]
fn every_ridge_sits_one_amplitude_off_the_original_curve() {
    let amp = 6.0;
    let out = zigzag_contour(&circle(), true, &spec(amp)).0;
    for r in radii(&out) {
        assert!(
            ((r - R).abs() - amp).abs() < 0.5,
            "uma crista caiu a {} do caminho, e a amplitude é {amp}",
            (r - R).abs()
        );
    }
}

/// O comprimento cresce com a onda — uma verificação independente de que houve deslocamento
/// mesmo (e não só âncoras remexidas no lugar).
#[test]
fn the_wave_makes_the_path_longer() {
    let plain: f64 = {
        let v = circle();
        let n = v.len();
        (0..n).map(|i| arclen(&segment(&v, i, n))).sum()
    };
    let waved = zigzag_contour(&circle(), true, &spec(8.0)).0;
    let n = waved.len();
    let len: f64 = (0..n).map(|i| arclen(&segment(&waved, i, n))).sum();
    // 6 cristas de amplitude 8 num círculo de raio 60 medem ~1,10x o liso (416 vs 377) — o
    // número está aqui porque um limite inventado ou é frouxo demais para pegar o bug ou
    // apertado demais e flaka. A barra fica um pouco abaixo do medido.
    assert!(
        len > plain * 1.05,
        "onda de {len} vs caminho liso de {plain} — sem crescimento não houve deslocamento"
    );
}
