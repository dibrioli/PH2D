//! Gates da SIMETRIA. Duas famílias com propriedades OPOSTAS convivem aqui, e é isso que os
//! gates existem para separar: a **reflexão** tem determinante −1 (inverte o winding, e tem de
//! ser reposto) e a **rotação** tem +1 (preserva-o, e não pode ser "uniformizada" com a outra).

use super::*;
use crate::VertexKind;
use crate::fx_repeat::{RepeatSpec, repeat_path};

/// Meia forma ABERTA com as duas pontas em `x = 0`: o meio-perfil de um vaso, que é o caso de uso
/// inteiro da fusão. Números do produto (unidades de documento), não `1.0`.
fn half_profile() -> VecPath {
    VecPath {
        verts: [[0.0, -1.0], [0.8, -0.4], [0.5, 0.4], [0.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: false,
        ..VecPath::default()
    }
}

/// Um quadrado FECHADO à direita do eixo `x = 0`, de lado 1.
fn square_right() -> VecPath {
    VecPath {
        verts: [[0.5, -0.5], [1.5, -0.5], [1.5, 0.5], [0.5, 0.5]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// Uma simetria com o centro EXPLÍCITO.
///
/// ⚠️ O centro é argumento em todo gate de propósito: ele é um lugar **AUTORADO**, e a 1ª versão
/// desta wave derivava-o da caixa da forma — foi exactamente essa derivação que o Enio reprovou.
fn spec(kind: SymmetryKind, center: [f64; 2], fuse: bool) -> SymmetrySpec {
    SymmetrySpec {
        kind,
        center,
        fuse,
        ..SymmetrySpec::default()
    }
}

/// A área COM SINAL de um contorno (o dobro dela, o que basta — só o sinal e a razão importam).
/// É o determinante a fazer o seu trabalho: reflectir troca-lhe o sinal, rodar não.
fn signed_area(verts: &[VecVertex]) -> f64 {
    let n = verts.len();
    (0..n)
        .map(|i| {
            let a = verts[i].anchor;
            let b = verts[(i + 1) % n].anchor;
            a[0].mul_add(b[1], -(b[0] * a[1]))
        })
        .sum()
}

/// Os contornos de um caminho, como lista.
fn contours(p: &VecPath) -> Vec<Vec<VecVertex>> {
    (0..p.contour_count())
        .filter_map(|k| p.contour(k).map(|(v, _)| v.to_vec()))
        .collect()
}

/// A extensão em `x` de um conjunto de caminhos.
fn span_x(ps: &[VecPath]) -> f64 {
    let (lo, hi) = ps
        .iter()
        .flat_map(VecPath::verts_all)
        .fold((f64::MAX, f64::MIN), |(a, b), v| {
            (a.min(v.anchor[0]), b.max(v.anchor[0]))
        });
    hi - lo
}

// ---------------------------------------------------------------------------------------------
// A REFLEXÃO — as três propriedades que a definem
// ---------------------------------------------------------------------------------------------

/// **Um ponto do eixo não se move, e um ponto fora vai para o outro lado à MESMA distância.**
/// É a definição de reflexão; qualquer outra coisa é uma translação disfarçada.
#[test]
fn the_reflection_flips_the_side_and_keeps_the_distance() {
    let p = square_right();
    let out = symmetry_paths(&p, &spec(SymmetryKind::MirrorX, [0.0, 0.0], false));
    assert_eq!(out.len(), 2, "original + reflexo, como caminhos SEPARADOS");
    let (a, b) = (contours(&out[0])[0].clone(), contours(&out[1])[0].clone());
    for (x, y) in a.iter().zip(b.iter().rev()) {
        assert!(
            (x.anchor[0] + y.anchor[0]).abs() < 1e-12,
            "as distâncias ao eixo x=0 têm de ser simétricas: {x:?} vs {y:?}"
        );
        assert!(
            (x.anchor[1] - y.anchor[1]).abs() < 1e-12,
            "a coordenada ao LONGO do eixo não pode mudar"
        );
    }
}

/// **Espelhar duas vezes pelo mesmo eixo devolve o ponto ao lugar** — a reflexão é uma involução.
/// Um sinal trocado na fórmula sobrevive a um gate de "mudou de lado" e morre aqui.
#[test]
fn reflecting_twice_is_the_identity() {
    let ax = Axis {
        at: [0.3, -0.2],
        n: [0.6, 0.8],
    };
    for p in [[0.0, 0.0], [1.0, 2.0], [-3.5, 0.25], [0.3, -0.2]] {
        let back = ax.reflect(ax.reflect(p));
        assert!(
            (back[0] - p[0]).abs() < 1e-12 && (back[1] - p[1]).abs() < 1e-12,
            "{p:?} não voltou: {back:?}"
        );
    }
    let on = [0.3, -0.2];
    let fixed = ax.reflect(on);
    assert!((fixed[0] - on[0]).abs() < 1e-12 && (fixed[1] - on[1]).abs() < 1e-12);
}

/// **O reflexo mantém o WINDING.** Sob `NonZero`, dois contornos sobrepostos de sentidos opostos
/// cancelam-se: o artista veria um BURACO onde espelhou.
#[test]
fn the_reflected_copy_keeps_the_winding() {
    let p = square_right();
    let out = symmetry_paths(&p, &spec(SymmetryKind::MirrorX, [1.0, 0.0], false));
    let (a, b) = (
        signed_area(&contours(&out[0])[0]),
        signed_area(&contours(&out[1])[0]),
    );
    assert!(
        a * b > 0.0,
        "as duas cópias têm de percorrer no MESMO sentido (áreas {a} e {b}) — \
         sentidos opostos abrem um buraco na sobreposição sob NonZero"
    );
    assert!(
        (a.abs() - b.abs()).abs() < 1e-12,
        "e a reflexão é uma isometria: as áreas têm o mesmo módulo"
    );
}

/// **Uma reflexão está FORA do alcance do Repeater**, e o oráculo é o determinante.
///
/// O Repeater compõe rotações e translações — det +1 — então TODA cópia dele preserva o sinal da
/// área. Uma reflexão crua troca-o. É o que separa esta família da outra.
#[test]
fn a_reflection_is_out_of_the_repeaters_reach() {
    let p = square_right();
    let base = signed_area(&contours(&p)[0]);

    for (spin, orbit) in [(0.0, 0.0), (37.0, 0.0), (0.0, 180.0), (90.0, 90.0)] {
        let r = repeat_path(
            &p,
            &RepeatSpec {
                copies_x: 4.0,
                move_x: 120.0,
                spin,
                orbit,
                ..RepeatSpec::default()
            },
        );
        for c in contours(&r) {
            assert!(
                signed_area(&c) * base > 0.0,
                "o Repeater NÃO pode inverter a orientação (spin {spin}, orbit {orbit})"
            );
        }
    }

    let ax = Axis {
        at: [0.0, 0.0],
        n: [1.0, 0.0],
    };
    let flipped: Vec<VecVertex> = contours(&p)[0]
        .iter()
        .map(|v| reflect_vert(v, ax))
        .collect();
    assert!(
        signed_area(&flipped) * base < 0.0,
        "uma reflexão TEM de trocar o sinal da área com sinal"
    );
}

// ---------------------------------------------------------------------------------------------
// A ROTAÇÃO — a outra família, com a propriedade OPOSTA
// ---------------------------------------------------------------------------------------------

/// **A rosácea tem `segments` cópias, e TODAS preservam o sentido.**
///
/// ⚠️ É a metade que impede alguém de "uniformizar" o tratamento das duas famílias: a rotação tem
/// determinante +1 e **não** passa pela reposição de winding. Inverter as cópias radiais seria
/// invisível numa forma isolada e abriria buracos assim que duas se tocassem.
#[test]
fn the_rosette_has_n_copies_and_none_of_them_flips() {
    let p = square_right();
    let base = signed_area(&contours(&p)[0]);
    for n in [3u32, 6, 12] {
        let s = SymmetrySpec {
            kind: SymmetryKind::Radial,
            center: [0.0, 0.0],
            segments: n,
            ..SymmetrySpec::default()
        };
        let out = symmetry_paths(&p, &s);
        assert_eq!(out.len(), n as usize, "a rosácea de {n} tem {n} cópias");
        for q in &out {
            let a = signed_area(&contours(q)[0]);
            assert!(
                a * base > 0.0,
                "uma cópia RADIAL não pode inverter o sentido (n {n}, área {a})"
            );
            assert!(
                (a.abs() - base.abs()).abs() < 1e-9,
                "e a rotação é isometria: mesma área"
            );
        }
    }
}

/// **A rosácea FECHA o anel**: a cópia `k` está a `k·360/n` do original.
///
/// ⚠️ `n = 7` de propósito — `360/7` **não** é um número inteiro de graus, e é exactamente onde
/// um rotor composto passo a passo deixa de fechar (a nota que o Painter carrega sobre a tabela
/// de rotores dele).
#[test]
fn the_rosette_closes_the_ring() {
    let p = square_right();
    let n = 7u32;
    let s = SymmetrySpec {
        kind: SymmetryKind::Radial,
        center: [0.0, 0.0],
        segments: n,
        ..SymmetrySpec::default()
    };
    let out = symmetry_paths(&p, &s);
    let r0 = out[0].verts[0].anchor;
    let radius = r0[0].hypot(r0[1]);
    for (k, q) in out.iter().enumerate() {
        let v = q.verts[0].anchor;
        assert!(
            (v[0].hypot(v[1]) - radius).abs() < 1e-9,
            "a cópia {k} saiu do círculo"
        );
        #[allow(clippy::cast_precision_loss)]
        let want = r0[1].atan2(r0[0]) + core::f64::consts::TAU * (k as f64) / f64::from(n);
        let got = v[1].atan2(v[0]);
        let d = (got - want).rem_euclid(core::f64::consts::TAU);
        assert!(
            d < 1e-9 || (core::f64::consts::TAU - d) < 1e-9,
            "a cópia {k} está no ângulo errado (delta {d})"
        );
    }
}

/// **A contagem é presa à faixa da UI** — a mesma `3..=12` do Painter.
#[test]
fn the_segment_count_is_clamped_to_the_painters_range() {
    for (given, want) in [
        (0u32, MIN_SEGMENTS),
        (2, MIN_SEGMENTS),
        (7, 7),
        (99, MAX_SEGMENTS),
    ] {
        let s = SymmetrySpec {
            kind: SymmetryKind::Radial,
            segments: given,
            ..SymmetrySpec::default()
        };
        assert_eq!(s.segments(), want, "segments({given})");
        assert_eq!(symmetry_paths(&square_right(), &s).len(), want as usize);
    }
}

// ---------------------------------------------------------------------------------------------
// O EIXO — as quatro palavras, e de onde vêm
// ---------------------------------------------------------------------------------------------

/// **Mirror X espelha esquerda↔direita e Mirror Y cima↔baixo** — o vocabulário do Painter, e a
/// única leitura que um artista aceita (a "X" espelha a coordenada X).
///
/// ⚠️ **O eixo fica FORA da forma nos dois casos, e é o que torna o gate capaz de falhar.** A 1ª
/// versão espelhava o quadrado em `y = 0`, que é o eixo de simetria dele próprio: o reflexo saía
/// **idêntico ao original** e a asserção não podia distinguir um espelho correto de um no-op. Um
/// fixture simétrico em relação ao eixo sob teste não contém o fenómeno.
///
/// E o oráculo é o LADO de todas as âncoras, não a âncora `[0]`: a cópia reflectida é percorrida
/// ao contrário (o winding é reposto), então o vértice 0 dela **não** é o reflexo do vértice 0 da
/// fonte.
#[test]
fn mirror_x_flips_left_to_right_and_mirror_y_top_to_bottom() {
    let p = square_right(); // x em [0,5 .. 1,5], y em [-0,5 .. 0,5]
    let side = |ps: &[VecPath], k: usize, at: f64| -> (bool, bool) {
        let (lo, hi) = ps
            .iter()
            .flat_map(VecPath::verts_all)
            .fold((f64::MAX, f64::MIN), |(a, b), v| {
                (a.min(v.anchor[k]), b.max(v.anchor[k]))
            });
        (hi < at, lo > at)
    };

    // Mirror X num eixo à ESQUERDA da forma: a fonte fica toda à direita, a cópia toda à esquerda.
    let x = symmetry_paths(&p, &spec(SymmetryKind::MirrorX, [0.0, 0.0], false));
    assert!(side(&x[0..1], 0, 0.0).1, "a fonte está à direita de x=0");
    assert!(
        side(&x[1..2], 0, 0.0).0,
        "e a cópia tem de ir toda para a ESQUERDA"
    );
    let (ylo, yhi) = x[1].verts_all().fold((f64::MAX, f64::MIN), |(a, b), v| {
        (a.min(v.anchor[1]), b.max(v.anchor[1]))
    });
    assert!(
        (ylo + 0.5).abs() < 1e-12 && (yhi - 0.5).abs() < 1e-12,
        "e o Y não pode mudar: {ylo}..{yhi}"
    );

    // Mirror Y num eixo ACIMA da forma: a fonte fica toda abaixo, a cópia toda acima.
    let y = symmetry_paths(&p, &spec(SymmetryKind::MirrorY, [0.0, 1.0], false));
    assert!(side(&y[0..1], 1, 1.0).0, "a fonte está abaixo de y=1");
    assert!(
        side(&y[1..2], 1, 1.0).1,
        "e a cópia tem de ir toda para CIMA"
    );
    let (xlo, xhi) = y[1].verts_all().fold((f64::MAX, f64::MIN), |(a, b), v| {
        (a.min(v.anchor[0]), b.max(v.anchor[0]))
    });
    assert!(
        (xlo - 0.5).abs() < 1e-12 && (xhi - 1.5).abs() < 1e-12,
        "e o X não pode mudar: {xlo}..{xhi}"
    );
}

/// **A linha Custom é a que o artista desenhou**, e o centro é PONTO FIXO — é isso que faz dele
/// um lugar, e não um número relativo à forma. Uma `dir` degenerada cai numa vertical em vez de
/// produzir `NaN`.
#[test]
fn the_custom_line_is_the_authored_one_and_a_degenerate_direction_falls_back() {
    let s = SymmetrySpec {
        kind: SymmetryKind::Custom,
        center: [2.0, 1.0],
        dir: [1.0, 1.0],
        ..SymmetrySpec::default()
    };
    let d = s.mirror_dir();
    let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
    assert!((d[0] - inv_sqrt2).abs() < 1e-12 && (d[1] - inv_sqrt2).abs() < 1e-12);
    let ax = Axis::of(&s);
    let f = ax.reflect(s.center);
    assert!((f[0] - 2.0).abs() < 1e-12 && (f[1] - 1.0).abs() < 1e-12);

    let bad = SymmetrySpec {
        kind: SymmetryKind::Custom,
        dir: [0.0, 0.0],
        ..SymmetrySpec::default()
    };
    assert_eq!(bad.mirror_dir(), [0.0, 1.0], "degenerada cai na vertical");
}

/// **`mirror_dir` é a PORTA ÚNICA** que o kernel e o overlay perguntam. Duas respostas
/// desenhariam a linha num sítio e espelhariam noutro — e ninguém lê um número numa screenshot.
#[test]
fn the_axis_the_kernel_uses_is_the_one_the_overlay_would_draw() {
    for kind in [
        SymmetryKind::MirrorX,
        SymmetryKind::MirrorY,
        SymmetryKind::Custom,
    ] {
        let s = SymmetrySpec {
            kind,
            center: [1.0, -2.0],
            dir: [3.0, 1.0],
            ..SymmetrySpec::default()
        };
        let d = s.mirror_dir();
        let ax = Axis::of(&s);
        // Um ponto ANDANDO ao longo da direcção publicada não pode sair da linha.
        let walked = [s.center[0] + d[0] * 5.0, s.center[1] + d[1] * 5.0];
        assert!(
            ax.signed_distance(walked).abs() < 1e-12,
            "{kind:?}: a direcção publicada não é a da linha que o kernel usa"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// A FUSÃO
// ---------------------------------------------------------------------------------------------

/// **Um meio-perfil com as pontas no eixo funde-se num ÚNICO caminho fechado** — sem isto o vaso
/// fica em duas metades que se tocam e não preenchem.
#[test]
fn an_open_half_whose_ends_touch_the_axis_fuses_into_one_closed_path() {
    let p = half_profile();
    let out = symmetry_paths(&p, &spec(SymmetryKind::MirrorX, [0.0, 0.0], true));
    assert_eq!(out.len(), 1, "a fusão dá UM caminho, não duas metades");
    assert!(out[0].closed, "e ele fecha, senão não preenche");
    assert_eq!(
        out[0].verts.len(),
        6,
        "as pontas do eixo não podem duplicar"
    );
    assert!(
        (span_x(&out) - 2.0 * span_x(std::slice::from_ref(&p))).abs() < 1e-9,
        "o vaso mede o DOBRO do meio-perfil"
    );
}

/// **A fusão degrada para o espelho simples quando não se aplica** — visivelmente (duas metades),
/// nunca em silêncio. E um contorno FECHADO nunca funde.
#[test]
fn fuse_degrades_to_a_plain_mirror_when_it_does_not_apply() {
    let p = half_profile();
    let far = symmetry_paths(&p, &spec(SymmetryKind::MirrorX, [-1.5, 0.0], true));
    assert_eq!(far.len(), 2, "sem pontas no eixo, duas metades");
    let sq = square_right();
    let closed = symmetry_paths(&sq, &spec(SymmetryKind::MirrorX, [0.0, 0.0], true));
    assert_eq!(closed.len(), 2, "um contorno fechado nunca funde");
}

/// **A costura carrega as alças reflectidas** — sem isto o fecho corta reto exactamente onde a
/// simetria devia ser mais visível.
#[test]
fn the_seam_carries_the_reflected_handles() {
    let mut p = half_profile();
    p.verts[0].out_handle = [0.4, -0.8];
    p.verts[0].kind = VertexKind::Corner;
    let s = spec(SymmetryKind::MirrorX, [0.0, 0.0], true);
    let ax = Axis::of(&s);
    let out = symmetry_paths(&p, &s);
    let want = ax.reflect([0.4, -0.8]);
    let got = out[0].verts[0].in_handle;
    assert!(
        (got[0] - want[0]).abs() < 1e-12 && (got[1] - want[1]).abs() < 1e-12,
        "a alça que CHEGA à costura tem de ser o reflexo da que sai: {got:?} vs {want:?}"
    );
}

/// **A tolerância da fusão é a MEDIDA que o `FUSE_TOL_FRAC` declara.**
///
/// ⚠️ Este gate existe para que o número do doc-comment não envelheça sozinho: ele varre o vão e
/// afirma onde a fusão de facto pára.
#[test]
fn the_fuse_tolerance_is_the_fraction_the_constant_declares() {
    let base = half_profile();
    let ext = extent(&base);
    for (frac, want) in [
        (0.000, true),
        (0.002, true),
        (0.009, true),
        (0.020, false),
        (0.050, false),
    ] {
        let gap = frac * ext;
        let mut p = base.clone();
        p.verts[0].anchor[0] -= gap;
        let n = p.verts.len() - 1;
        p.verts[n].anchor[0] -= gap;
        let fused = symmetry_paths(&p, &spec(SymmetryKind::MirrorX, [0.0, 0.0], true)).len() == 1;
        assert_eq!(
            fused, want,
            "vão de {frac} da forma (extensão {ext}): fundiu = {fused}, esperado {want}"
        );
    }
}

/// **A fusão é INERTE no Radial** — não há costura a fechar numa rosácea, e o flag não pode
/// engolir cópias.
#[test]
fn fuse_is_inert_for_the_rosette() {
    let p = half_profile();
    let s = SymmetrySpec {
        kind: SymmetryKind::Radial,
        center: [0.0, 0.0],
        segments: 5,
        fuse: true,
        ..SymmetrySpec::default()
    };
    assert_eq!(symmetry_paths(&p, &s).len(), 5);
}

// ---------------------------------------------------------------------------------------------
// O COMPOUND
// ---------------------------------------------------------------------------------------------

/// **Um buraco espelhado continua buraco.** A reposição do winding é uniforme sobre TODOS os
/// contornos — se fosse só no primário, a rosquinha reflectida encheria.
#[test]
fn a_mirrored_hole_is_still_a_hole() {
    let mut p = VecPath {
        verts: [[0.5, -1.0], [2.5, -1.0], [2.5, 1.0], [0.5, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    let mut hole: Vec<VecVertex> = [[1.0, -0.5], [2.0, -0.5], [2.0, 0.5], [1.0, 0.5]]
        .map(VecVertex::corner)
        .to_vec();
    hole.reverse();
    p.subpaths.push(Contour {
        verts: hole,
        closed: true,
    });

    let outer0 = signed_area(&contours(&p)[0]);
    let hole0 = signed_area(&contours(&p)[1]);
    assert!(outer0 * hole0 < 0.0, "o fixture tem de conter o fenómeno");

    let out = symmetry_paths(&p, &spec(SymmetryKind::MirrorX, [0.0, 0.0], false));
    assert_eq!(out.len(), 2);
    for q in &out {
        let cs = contours(q);
        assert_eq!(cs.len(), 2, "cada cópia mantém os dois contornos");
        assert!(
            signed_area(&cs[0]) * signed_area(&cs[1]) < 0.0,
            "a cópia perdeu a relação fora/buraco"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// O VOCABULÁRIO
// ---------------------------------------------------------------------------------------------

/// **Os quatro tipos fecham a volta pelo wire e têm rótulo próprio** — um `from_u8` que
/// devolvesse o vizinho leria um documento salvo como outra simetria, em silêncio.
#[test]
fn every_kind_round_trips_and_is_named() {
    let mut seen: Vec<&str> = Vec::new();
    for k in SymmetryKind::ALL {
        assert_eq!(SymmetryKind::from_u8(k.to_u8()), *k);
        assert!(!seen.contains(&k.label()), "rótulo repetido: {}", k.label());
        seen.push(k.label());
    }
    assert_eq!(
        SymmetryKind::from_u8(200),
        SymmetryKind::MirrorX,
        "um wire fora de alcance cai no default, nunca em pânico"
    );
    // E só o Radial NÃO reflete — é a partição que separa as duas famílias.
    for k in SymmetryKind::ALL {
        assert_eq!(k.reflects(), *k != SymmetryKind::Radial);
    }
}

/// **`copy_count` diz a verdade** — o painel mostra-o antes de o artista ligar, então ele não
/// pode discordar do que o kernel de facto emite.
#[test]
fn the_advertised_copy_count_is_what_the_kernel_emits() {
    let p = square_right();
    for k in SymmetryKind::ALL {
        for segments in [3u32, 6, 12] {
            let s = SymmetrySpec {
                kind: *k,
                center: [0.0, 0.0],
                segments,
                fuse: false,
                ..SymmetrySpec::default()
            };
            assert_eq!(
                symmetry_paths(&p, &s).len(),
                s.copy_count(),
                "{k:?} com {segments} segmentos"
            );
        }
    }
}
