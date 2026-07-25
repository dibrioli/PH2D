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

/// A referência da forma dos fixtures: o círculo tem caixa de controle ~2R x 2R (as alças
/// saem um pouco), então a média fica perto de 2R. Os gates a MEDEM em vez de a supor.
fn ref_of(v: &[VecVertex]) -> f64 {
    crate::effect::FxCtx::of(&crate::VecPath {
        verts: v.to_vec(),
        closed: true,
        ..crate::VecPath::default()
    })
    .ref_size
}

/// A amplitude em MUNDO que uma percentagem produz nesta forma.
fn world_amp(v: &[VecVertex], pct: f64) -> f64 {
    pct / 100.0 * ref_of(v)
}

/// Roda o efeito com a referência de escala TIRADA DA PRÓPRIA FORMA — que é o que o
/// `run_stack` faz no produto.
fn zz(v: &[VecVertex], closed: bool, spec: &ZigZagSpec) -> (Vec<VecVertex>, bool) {
    zigzag_contour(v, closed, spec, ref_of(v), None)
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

/// Uma polilinha ABERTA com quinas — o outro regime (o `closed` muda a contagem de segmentos e a
/// grade das cristas, e a cúspide de uma quina é onde a tangente não existe).
fn open_corner() -> Vec<VecVertex> {
    [[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [90.0, 30.0]]
        .into_iter()
        .map(VecVertex::corner)
        .collect()
}

/// FNV-1a sobre os bits — determinístico, sem dependência, e sensível ao último ulp.
fn fnv(mut h: u64, x: u64) -> u64 {
    for b in x.to_le_bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// **A IMPRESSÃO DIGITAL do Zig Zag.** É o oráculo da extração do walker de arco para
/// [`crate::arc_path`] (W1 do plano de texto em caminho, doc 22): o Zig Zag **já shipou**, e um
/// refactor que "só move código" tem de o provar ao BIT — não por inspeção.
///
/// Cobre os quatro regimes que a extração pode partir: fechado · o mesmo fechado com o dobro de
/// âncoras (a UNIÃO com a grade) · **aberto com quinas** (outra contagem de segmentos, e cúspides
/// onde a tangente não existe) · Roughen com semente (o outro deslocamento).
///
/// ⚠️ Se este gate falhar num commit que **pretendia** mudar o desenho do efeito, o número é que
/// está velho. Num commit de refactor, o número está certo e o refactor não é o que diz ser.
#[test]
fn the_zigzag_is_byte_identical_across_the_arc_walker_extraction() {
    let cases: [(Vec<VecVertex>, bool, ZigZagSpec); 4] = [
        (circle(), true, spec(20.0)),
        (circle_subdivided(), true, spec(7.5)),
        (
            open_corner(),
            false,
            ZigZagSpec {
                amplitude: 15.0,
                ridges: 5.0,
                smooth: true,
                rough_seed: None,
            },
        ),
        (
            circle(),
            true,
            ZigZagSpec {
                amplitude: 12.0,
                ridges: 9.0,
                smooth: false,
                rough_seed: Some(42),
            },
        ),
    ];
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (v, closed, s) in cases {
        let (out, c) = zigzag_contour(&v, closed, &s, ref_of(&v), None);
        h = fnv(h, u64::from(c));
        h = fnv(h, out.len() as u64);
        for w in &out {
            for x in [
                w.anchor[0],
                w.anchor[1],
                w.in_handle[0],
                w.in_handle[1],
                w.out_handle[0],
                w.out_handle[1],
            ] {
                h = fnv(h, x.to_bits());
            }
            h = fnv(h, w.kind as u64);
        }
    }
    // Medida em 2026-07-22, ANTES da extração do walker (commit 5bc175013).
    assert_eq!(
        h, 0xf6eb_ed49_0c31_322e,
        "o desenho do Zig Zag mudou (impressao digital {h:#018x})"
    );
}

/// **O ponto neutro é um no-op byte-idêntico** — o que a pilha exige de todo efeito.
#[test]
fn the_neutral_point_is_a_byte_identical_no_op() {
    let v = circle();
    let (out, closed) = zz(&v, true, &spec(0.0));
    assert_eq!(out, v);
    assert!(closed);
}

/// **A onda alterna para os DOIS lados** do caminho, e com a amplitude pedida.
#[test]
fn the_ridges_alternate_to_both_sides_by_the_amplitude() {
    let pct = 8.0;
    let amp = world_amp(&circle(), pct);
    let (out, _) = zz(&circle(), true, &spec(pct));
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
    let (_, closed) = zz(&circle(), true, &spec(5.0));
    assert!(closed);
    let open: Vec<VecVertex> = circle().into_iter().take(3).collect();
    let (_, still_open) = zz(&open, false, &spec(5.0));
    assert!(!still_open);
}

/// **O GATE QUE IMPORTA: a mesma forma com mais âncoras ondula IGUAL.**
///
/// O Illustrator conta cristas *por segmento*, então picar uma aresta muda o desenho — o efeito
/// passa a descrever como o caminho foi autorado. Aqui as cristas são contadas no caminho
/// inteiro, por comprimento de ARCO, e a subdivisão não move nada.
///
/// ⚠️ **O oráculo mudou de forma quando a união entrou** (2026-07-18). Comparar as listas de
/// vértices lado a lado deixou de ser possível — o caminho picado tem âncoras a mais, e elas
/// entram nas amostras (é isso que faz o efeito compor). Comparar contagens teria de escolher
/// entre *reprovar uma correção* e *afrouxar a distância* até deixar de pegar bug.
///
/// A propriedade real, e é mais forte do que a contagem: **cada vértice do caminho magro existe,
/// no mesmo sítio, no caminho picado.** Subdividir só ACRESCENTA amostras entre as que já havia;
/// não desloca nenhuma. Se o efeito medisse a parametrização, os pontos partilhados sairiam
/// diferentes e este gate cairia.
#[test]
fn subdividing_the_path_does_not_change_the_wave() {
    let a = zz(&circle(), true, &spec(7.0)).0;
    let b = zz(&circle_subdivided(), true, &spec(7.0)).0;
    assert!(
        b.len() > a.len(),
        "o caminho picado tem de trazer as amostras extra ({} vs {})",
        b.len(),
        a.len()
    );
    // Tolerância `1e-6`: o comprimento total sai da SOMA de 4 quadraturas num caso e de 8 no
    // outro, então a última casa do `f64` difere — medido, o pior desvio real é ~1e-10.
    for (i, p) in a.iter().enumerate() {
        let best = b
            .iter()
            .map(|q| (p.anchor[0] - q.anchor[0]).hypot(p.anchor[1] - q.anchor[1]))
            .fold(f64::MAX, f64::min);
        assert!(
            best < 1e-6,
            "o vértice {i} do caminho magro ({:?}) não tem par no picado — o mais perto está a \
             {best}. As duas formas são a MESMA curva; se um ponto se move, o efeito está a \
             medir a parametrização e não a geometria.",
            p.anchor
        );
    }
}

/// **Um 2º Zig Zag tem de ondular SOBRE o 1º, não apagá-lo** (Enio, 2026-07-18).
///
/// A 1ª versão reamostrava em `2·ridges` posições e descartava a entrada — sobre um caminho que
/// já tem onda, isso é amostrar mais grosso do que o sinal que lá está: **aliasing**. Medido no
/// círculo de raio 60: 16 cristas davam comprimento 718, e um 2º zigzag de 4 cristas deixava
/// **310 — mais curto do que o próprio círculo (339)**. A onda não compunha mal; desaparecia, e
/// a forma encolhia.
///
/// O oráculo é o COMPRIMENTO, porque é o que se vê: acrescentar uma onda a um caminho ondulado
/// só o pode fazer crescer. A barra é o comprimento do 1º zigzag sozinho — o mínimo que sobra se
/// o 1º sobreviver intacto e o 2º não contribuir nada.
#[test]
fn a_second_zigzag_rides_on_the_first_instead_of_erasing_it() {
    let len_of = |v: &[VecVertex]| -> f64 {
        let n = v.len();
        (0..n).map(|i| arclen(&segment(v, i, n))).sum::<f64>()
    };
    let one = zz(
        &circle(),
        true,
        &ZigZagSpec {
            ridges: 16.0,
            ..spec(8.0)
        },
    )
    .0;
    let first = len_of(&one);

    // As DUAS direções: o 2º mais grosso (o caso que apagava) e o 2º mais fino.
    for r2 in [4.0, 8.0, 32.0] {
        let two = zz(
            &one,
            true,
            &ZigZagSpec {
                ridges: r2,
                ..spec(4.0)
            },
        )
        .0;
        let stacked = len_of(&two);
        assert!(
            stacked >= first,
            "o 2º zigzag de {r2} cristas deixou o caminho com {stacked}, e o 1º sozinho media \
             {first} — encurtar significa que a onda do 1º foi reamostrada para longe"
        );
        // E os picos do 1º continuam lá: cada âncora dele tem par no resultado.
        for p in &one {
            let best = two
                .iter()
                .map(|q| (p.anchor[0] - q.anchor[0]).hypot(p.anchor[1] - q.anchor[1]))
                .fold(f64::MAX, f64::min);
            assert!(
                best < 6.0,
                "um pico do 1º zigzag ({:?}) ficou a {best} do resultado com {r2} cristas — a \
                 amplitude do 2º é 4, então mais do que isso é o pico ter sido saltado",
                p.anchor
            );
        }
    }
}

/// **Mais cristas = mais cristas** — o knob faz o que diz.
#[test]
fn asking_for_more_ridges_produces_more_ridges() {
    let few = zz(
        &circle(),
        true,
        &ZigZagSpec {
            ridges: 4.0,
            ..spec(5.0)
        },
    )
    .0;
    let many = zz(
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
    let corner = zz(&circle(), true, &spec(6.0)).0;
    assert!(
        corner
            .iter()
            .all(|v| v.in_handle == v.anchor && v.out_handle == v.anchor),
        "em quina a alça colapsa na âncora"
    );
    let smooth = zz(
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
        zz(
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
    let pct = 9.0;
    let amp = world_amp(&circle(), pct);
    let out = zz(
        &circle(),
        true,
        &ZigZagSpec {
            rough_seed: Some(3),
            ..spec(pct)
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
    let pct = 6.0;
    let amp = world_amp(&circle(), pct);
    let out = zz(&circle(), true, &spec(pct)).0;
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
    let waved = zz(&circle(), true, &spec(8.0)).0;
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

/// **O MESMO `Size` desenha a MESMA coisa em qualquer escala** — é o ponto inteiro de a
/// amplitude ser percentagem da forma (Enio, 2026-07-18).
///
/// Duas cópias do círculo, uma 5× maior. Com a amplitude em unidades de MUNDO a onda da grande
/// seria 5× mais discreta; em percentagem, as duas ondulações são semelhantes ao fator de
/// escala. O oráculo é a RAZÃO entre o desvio e o raio, que é adimensional.
#[test]
fn the_same_size_looks_the_same_at_any_scale() {
    let scaled = |k: f64| -> Vec<VecVertex> {
        circle()
            .into_iter()
            .map(|v| VecVertex {
                anchor: [v.anchor[0] * k, v.anchor[1] * k],
                in_handle: [v.in_handle[0] * k, v.in_handle[1] * k],
                out_handle: [v.out_handle[0] * k, v.out_handle[1] * k],
                ..v
            })
            .collect()
    };
    let bulge = |k: f64| -> f64 {
        let v = scaled(k);
        let out = zz(&v, true, &spec(12.0)).0;
        // O desvio máximo relativo ao raio daquela cópia — adimensional.
        radii(&out)
            .into_iter()
            .map(|r| (r - R * k).abs() / (R * k))
            .fold(0.0_f64, f64::max)
    };
    let (small, big) = (bulge(1.0), bulge(5.0));
    assert!(
        (small - big).abs() / small < 1e-6,
        "a mesma percentagem deu {small} na forma pequena e {big} na grande — a amplitude \
         voltou a ser absoluta, e o slider deixa de significar a mesma coisa em cada forma"
    );
    assert!(small > 0.05, "e a onda tem de existir de facto ({small})");
}

/// Sem referência de tamanho (uma forma degenerada num ponto) o efeito é **inerte**, não um
/// pânico nem uma divisão que produz `NaN`.
#[test]
fn a_degenerate_shape_has_no_scale_and_the_effect_is_inert() {
    let dot: Vec<VecVertex> = (0..4).map(|_| VecVertex::corner([3.0, 7.0])).collect();
    let (out, _) = zigzag_contour(&dot, true, &spec(50.0), 0.0, None);
    assert_eq!(out, dot);
}
