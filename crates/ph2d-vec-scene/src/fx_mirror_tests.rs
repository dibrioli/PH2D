//! Gates do Mirror. O que este efeito acrescenta à pilha é uma transformação de determinante
//! **negativo** — e é daí que vêm as duas propriedades que nenhum outro efeito tem de provar: o
//! **winding** tem de ser reposto, e o reflexo tem de estar **fora do alcance do Repeater**.

use super::*;
use crate::VertexKind;
use crate::fx_repeat::{RepeatSpec, repeat_path};

/// Meia forma ABERTA com as duas pontas no eixo `x = 0`: o meio-perfil de um vaso, que é o caso
/// de uso inteiro da fusão. Números do produto (unidades de documento), não `1.0`.
fn half_profile() -> VecPath {
    VecPath {
        verts: [[0.0, -1.0], [0.8, -0.4], [0.5, 0.4], [0.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: false,
        ..VecPath::default()
    }
}

/// Um quadrado FECHADO deslocado para a direita do eixo, de lado 1.
fn square_right() -> VecPath {
    VecPath {
        verts: [[0.5, -0.5], [1.5, -0.5], [1.5, 0.5], [0.5, 0.5]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// O `ctx` do caminho, como a pilha o mede.
fn ctx(p: &VecPath) -> FxCtx {
    FxCtx::of(p)
}

/// Um espelho de um eixo, no ângulo e deslize dados.
///
/// ⚠️ O deslize é EXPLÍCITO em cada gate de propósito: `0` põe o eixo no centro da caixa e `100`
/// tangente a ela, e confundir os dois foi o que fez a 1ª versão destes gates medir uma fusão
/// que não podia acontecer (as pontas do meio-perfil estão na BORDA, não no meio).
fn spec(angle: f64, offset: f64, fuse: bool) -> MirrorSpec {
    MirrorSpec {
        axes: 1.0,
        angle,
        offset,
        fuse: if fuse { 1.0 } else { 0.0 },
    }
}

/// A área COM SINAL de um contorno (o dobro dela, o que basta — só o sinal e a razão importam).
/// É o determinante a fazer o seu trabalho: reflectir troca-lhe o sinal.
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

// ---------------------------------------------------------------------------------------------
// O NEUTRO — a lei executável da pilha
// ---------------------------------------------------------------------------------------------

/// **Zero eixos é um no-op byte-idêntico.** Sem isto a pilha não pode saltá-lo e o
/// `Cow::Borrowed` morre num documento que só tem a seção aberta.
#[test]
fn no_axis_is_a_byte_identical_no_op() {
    let p = square_right();
    let s = MirrorSpec::new();
    assert!(s.is_neutral(), "o Mirror tem de NASCER neutro");
    assert_eq!(mirror_path(&p, &s, &ctx(&p)), p);
    // E um `axes` fraccionário abaixo de 1 também não espelha — a contagem é saneada na porta.
    let frac = MirrorSpec {
        axes: 0.9,
        ..MirrorSpec::new()
    };
    assert_eq!(mirror_path(&p, &frac, &ctx(&p)), p);
}

// ---------------------------------------------------------------------------------------------
// A REFLEXÃO — as três propriedades que a definem
// ---------------------------------------------------------------------------------------------

/// **Um ponto do eixo não se move, e um ponto fora vai para o outro lado à MESMA distância.**
/// É a definição de reflexão; qualquer outra coisa é uma translação disfarçada.
#[test]
fn the_reflection_flips_the_side_and_keeps_the_distance() {
    let p = square_right();
    let c = ctx(&p);
    // Eixo vertical (ângulo 90) pelo centro da caixa, que está em x = 1,0.
    let out = mirror_path(&p, &spec(90.0, 0.0, false), &c);
    let cs = contours(&out);
    assert_eq!(cs.len(), 2, "original + reflexo");
    let axis_x = c.center[0];
    for (a, b) in cs[0].iter().zip(cs[1].iter().rev()) {
        assert!(
            ((a.anchor[0] - axis_x) + (b.anchor[0] - axis_x)).abs() < 1e-12,
            "as distâncias ao eixo têm de ser simétricas: {a:?} vs {b:?}"
        );
        assert!(
            (a.anchor[1] - b.anchor[1]).abs() < 1e-12,
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
    // E o ponto que ESTÁ no eixo é ponto fixo.
    let on = [0.3, -0.2];
    let fixed = ax.reflect(on);
    assert!((fixed[0] - on[0]).abs() < 1e-12 && (fixed[1] - on[1]).abs() < 1e-12);
}

/// **O reflexo mantém o WINDING.** Sob `NonZero`, dois contornos sobrepostos de sentidos opostos
/// cancelam-se: o artista veria um BURACO onde espelhou. O contorno reflectido é invertido de
/// volta, e este gate mede o sinal da área.
#[test]
fn the_reflected_contour_keeps_the_winding() {
    let p = square_right();
    let out = mirror_path(&p, &spec(90.0, 0.0, false), &ctx(&p));
    let cs = contours(&out);
    let (a, b) = (signed_area(&cs[0]), signed_area(&cs[1]));
    assert!(
        a * b > 0.0,
        "os dois contornos têm de percorrer no MESMO sentido (áreas {a} e {b}) — \
         sentidos opostos abrem um buraco na sobreposição sob NonZero"
    );
    assert!(
        (a.abs() - b.abs()).abs() < 1e-12,
        "e a reflexão é uma isometria: as áreas têm o mesmo módulo"
    );
}

/// **Um espelho está FORA do alcance do Repeater**, e o oráculo é o determinante.
///
/// O Repeater compõe rotações e translações — det +1 — então TODA cópia dele preserva o sinal da
/// área. A reflexão crua troca-o. Isto é o que justifica um variant novo em vez de um botão.
#[test]
fn a_reflection_is_out_of_the_repeaters_reach() {
    let p = square_right();
    let base = signed_area(&contours(&p)[0]);

    // Toda cópia do Repeater — inclusive com spin E orbit — preserva o sinal.
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

    // A reflexão crua inverte-o — é a assinatura do determinante −1.
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
// O EIXO — onde ele fica
// ---------------------------------------------------------------------------------------------

/// **`Offset = 100` põe a linha TANGENTE à caixa, em qualquer ângulo.**
///
/// É a propriedade do *Relative Offset* (um número redondo dá um encaixe exacto), e ela só vale
/// porque a referência é o SUPORTE da caixa na direcção da normal — com `ref_size` (a média) ela
/// só acertaria numa forma quadrada.
///
/// ⚠️ **O oráculo é a TANGÊNCIA, não a fórmula.** Afirmar que a linha passa por
/// `center + n·support` seria repetir o cálculo que está sob teste — verde por construção. Uma
/// linha tangente é uma que deixa a forma INTEIRA de um lado e toca-a: isso mede-se nos pontos.
#[test]
fn an_offset_of_one_hundred_lands_the_axis_tangent_to_the_bounding_box() {
    // Caixa deliberadamente NÃO quadrada: é onde a média e o suporte divergem.
    let p = VecPath {
        verts: [[0.0, 0.0], [4.0, 0.0], [4.0, 1.0], [0.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    let c = ctx(&p);
    for angle in [90.0, 0.0, 30.0, -115.0] {
        let ax = axes_of(&spec(angle, 100.0, false), &c)[0];
        let d: Vec<f64> = p
            .verts_all()
            .flat_map(|v| [v.anchor, v.in_handle, v.out_handle])
            .map(|q| ax.signed_distance(q))
            .collect();
        let (lo, hi) = d
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        assert!(
            lo >= -1e-9 || hi <= 1e-9,
            "a {angle}°: a forma tem de ficar toda de UM lado (distâncias {lo}..{hi})"
        );
        assert!(
            lo.abs().min(hi.abs()) < 1e-9,
            "a {angle}°: e a linha tem de TOCAR a caixa (distâncias {lo}..{hi})"
        );
    }
    // E o `0` é o outro extremo nomeado: a linha pelo CENTRO parte a forma em dois.
    let mid = axes_of(&spec(90.0, 0.0, false), &c)[0];
    let d: Vec<f64> = p
        .verts_all()
        .map(|v| mid.signed_distance(v.anchor))
        .collect();
    assert!(
        d.iter().any(|&x| x > 1e-9) && d.iter().any(|&x| x < -1e-9),
        "com offset 0 a forma tem pontos dos DOIS lados"
    );
}

/// **Os dois eixos cruzam-se no mesmo ponto e são perpendiculares** — é isso que dá as 4 dobras.
#[test]
fn the_second_axis_is_the_perpendicular_through_the_same_point() {
    let p = square_right();
    let c = ctx(&p);
    let s = MirrorSpec {
        axes: 2.0,
        angle: 33.0,
        offset: 40.0,
        fuse: 0.0,
    };
    let ax = axes_of(&s, &c);
    assert_eq!(ax.len(), 2);
    assert!(
        (ax[0].at[0] - ax[1].at[0]).abs() < 1e-12 && (ax[0].at[1] - ax[1].at[1]).abs() < 1e-12,
        "as duas linhas cruzam-se no MESMO ponto"
    );
    let dot = ax[0].n[0].mul_add(ax[1].n[0], ax[0].n[1] * ax[1].n[1]);
    assert!(dot.abs() < 1e-12, "e as normais são perpendiculares: {dot}");
}

/// **`Axes = 2` dá quatro cópias** — o original e as três reflexões (A, B, AB).
#[test]
fn two_axes_give_four_fold_symmetry() {
    let p = square_right();
    let s = MirrorSpec {
        axes: 2.0,
        angle: 90.0,
        offset: 0.0,
        fuse: 0.0,
    };
    let out = mirror_path(&p, &s, &ctx(&p));
    assert_eq!(out.contour_count(), 4, "original + 3 reflexões");
    // E todas mantêm o winding: uma delas é a composição de DUAS reflexões (det +1), as outras
    // duas são reflexões simples repostas — o gate falha se a reposição for aplicada a menos.
    let base = signed_area(&contours(&p)[0]);
    for c in contours(&out) {
        assert!(signed_area(&c) * base > 0.0);
    }
}

/// **Os DEFAULTS sozinhos fazem o caso de uso inteiro** — subir `Axes` a 1 num meio-perfil, e
/// mais nada, tem de dar o vaso fundido.
///
/// ⚠️ Este gate existe porque a decisão que ele pina não tinha nenhum: a 1ª versão nascia com
/// `offset = 0` (o eixo no CENTRO da caixa), e com isso o reflexo cai em cima da forma e o
/// meio-perfil espelha sobre o meio de si mesmo. Todos os outros gates passavam, porque cada um
/// declara o seu próprio deslize — *um default só é testado por um teste que não o menciona*.
#[test]
fn the_defaults_alone_turn_a_half_profile_into_a_fused_vase() {
    let p = half_profile();
    let armed = MirrorSpec {
        axes: 1.0,
        ..MirrorSpec::new() // e SÓ isto: ângulo, deslize e fusão vêm de fábrica
    };
    let out = mirror_path(&p, &armed, &ctx(&p));
    assert_eq!(
        out.contour_count(),
        1,
        "com os defaults o meio-perfil tem de FUNDIR (angle {}, offset {}, fuse {})",
        armed.angle,
        armed.offset,
        armed.fuse
    );
    assert!(out.closed, "e fechar, senão não preenche");
    // E o vaso mede o DOBRO do meio-perfil: o reflexo foi para o LADO, não para cima da forma.
    let width = |q: &VecPath| {
        let (lo, hi) = q.verts_all().fold((f64::MAX, f64::MIN), |(a, b), v| {
            (a.min(v.anchor[0]), b.max(v.anchor[0]))
        });
        hi - lo
    };
    assert!(
        (width(&out) - 2.0 * width(&p)).abs() < 1e-9,
        "o vaso tem de medir o dobro ({} vs {})",
        width(&out),
        2.0 * width(&p)
    );
}

// ---------------------------------------------------------------------------------------------
// A FUSÃO
// ---------------------------------------------------------------------------------------------

/// **Um meio-perfil com as pontas no eixo funde-se num ÚNICO contorno fechado** — sem isto o
/// vaso fica em duas metades que se tocam e não preenchem.
#[test]
fn an_open_half_whose_ends_touch_the_axis_fuses_into_one_closed_contour() {
    let p = half_profile();
    let out = mirror_path(&p, &spec(90.0, 100.0, true), &ctx(&p));
    assert_eq!(
        out.contour_count(),
        1,
        "a fusão tem de dar UM contorno, não duas metades"
    );
    assert!(out.closed, "e ele tem de estar FECHADO, senão não preenche");
    let verts = &contours(&out)[0];
    // 4 originais + 2 do miolo reflectido (as duas pontas no eixo não se duplicam).
    assert_eq!(verts.len(), 6, "as pontas do eixo não podem duplicar");
    // A forma fundida é simétrica EM RELAÇÃO AO EIXO — e é o eixo que se pergunta, não o centro
    // da caixa. (A 1ª versão media contra `ctx.center` e falhava sobre um vaso perfeitamente
    // simétrico: com o deslize de 100 o eixo está na BORDA, a 0,4 do centro.)
    let ax = axes_of(&spec(90.0, 100.0, true), &ctx(&p))[0];
    let (mut left, mut right, mut on) = (0, 0, 0);
    for v in verts {
        let d = ax.signed_distance(v.anchor);
        if d < -1e-9 {
            left += 1;
        } else if d > 1e-9 {
            right += 1;
        } else {
            on += 1;
        }
    }
    assert_eq!(left, right, "a forma fundida tem de ser simétrica");
    assert_eq!(on, 2, "e as duas pontas ficam NO eixo, uma vez cada");
}

/// **A fusão degrada para o espelho simples quando as pontas NÃO estão no eixo** — visivelmente
/// (duas metades), nunca em silêncio.
#[test]
fn fuse_degrades_to_a_plain_mirror_when_the_ends_are_off_the_axis() {
    // O mesmo perfil, deslocado para longe do eixo do centro dele.
    let p = half_profile();
    let s = MirrorSpec {
        axes: 1.0,
        angle: 90.0,
        offset: 150.0, // a linha sai da forma
        fuse: 1.0,
    };
    let out = mirror_path(&p, &s, &ctx(&p));
    assert_eq!(
        out.contour_count(),
        2,
        "sem pontas no eixo há duas metades, e elas não se fundem"
    );
    // E um contorno FECHADO nunca funde, mesmo com as pontas por cima do eixo.
    let sq = square_right();
    let closed = mirror_path(&sq, &spec(90.0, 100.0, true), &ctx(&sq));
    assert_eq!(closed.contour_count(), 2);
}

/// **A costura carrega as alças reflectidas** — sem isto o fecho do contorno corta reto
/// exactamente onde a simetria devia ser mais visível.
#[test]
fn the_seam_carries_the_reflected_handles() {
    let mut p = half_profile();
    // Dá à primeira ponta uma alça de saída que não é trivial.
    p.verts[0].out_handle = [0.4, -0.8];
    p.verts[0].kind = VertexKind::Corner;
    let c = ctx(&p);
    let ax = axes_of(&spec(90.0, 100.0, true), &c)[0];
    let out = mirror_path(&p, &spec(90.0, 100.0, true), &c);
    let verts = &contours(&out)[0];
    let want = ax.reflect([0.4, -0.8]);
    assert!(
        (verts[0].in_handle[0] - want[0]).abs() < 1e-12
            && (verts[0].in_handle[1] - want[1]).abs() < 1e-12,
        "a alça que CHEGA à costura tem de ser o reflexo da que sai: {:?} vs {want:?}",
        verts[0].in_handle
    );
}

/// **A tolerância da fusão é a MEDIDA que o `FUSE_TOL_FRAC` declara.**
///
/// ⚠️ Este gate existe para que o número no doc-comment não envelheça sozinho: ele varre o vão e
/// afirma onde a fusão de facto pára. Mudar a constante sem mexer aqui fica VERMELHO.
#[test]
fn the_fuse_tolerance_is_the_fraction_the_constant_declares() {
    let base = half_profile();
    let ref_size = ctx(&base).ref_size;
    // O vão de cada lado da tabela do doc-comment, em fracção do `ref_size`.
    for (frac, want) in [
        (0.000, true),
        (0.002, true),
        (0.009, true),
        (0.020, false),
        (0.050, false),
    ] {
        let gap = frac * ref_size;
        let mut p = base.clone();
        // Afasta as DUAS pontas do eixo pelo vão.
        p.verts[0].anchor[0] -= gap;
        let n = p.verts.len() - 1;
        p.verts[n].anchor[0] -= gap;
        // O eixo tem de ficar onde estava: mede-o no perfil ORIGINAL, senão mover as pontas move
        // a caixa e o vão que se pensa estar a testar não é o que o motor vê.
        let c = ctx(&base);
        let out = mirror_path(&p, &spec(90.0, 100.0, true), &c);
        let fused = out.contour_count() == 1;
        assert_eq!(
            fused, want,
            "vão de {frac} da forma (ref_size {ref_size}): fundiu = {fused}, esperado {want}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// O COMPOUND
// ---------------------------------------------------------------------------------------------

/// **Um buraco espelhado continua buraco.** A reposição do winding é uniforme sobre TODOS os
/// contornos — se ela fosse aplicada só ao primário, a rosquinha reflectida encheria.
#[test]
fn a_mirrored_hole_is_still_a_hole() {
    let mut p = VecPath {
        verts: [[0.5, -1.0], [2.5, -1.0], [2.5, 1.0], [0.5, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    // O buraco percorre ao CONTRÁRIO do contorno de fora — é isso que o torna buraco.
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

    let out = mirror_path(&p, &spec(90.0, 0.0, false), &ctx(&p));
    let cs = contours(&out);
    assert_eq!(cs.len(), 4, "dois contornos, duas cópias");
    // ⚠️ A emissão é POR CONTORNO — `[fora, fora', buraco, buraco']` —, então os pares
    // fora/buraco são (0,2) e (1,3). O contorno PRIMÁRIO continua a ser o primário original: um
    // reflexo que tomasse esse lugar mudaria o que `path.verts` significa para todo o resto do
    // editor.
    assert_eq!(
        cs[0],
        contours(&p)[0],
        "o primário tem de continuar a ser a forma autorada"
    );
    for pair in [(0usize, 2usize), (1, 3)] {
        let (a, b) = (signed_area(&cs[pair.0]), signed_area(&cs[pair.1]));
        assert!(
            a * b < 0.0,
            "a cópia {pair:?} perdeu a relação fora/buraco (áreas {a} e {b})"
        );
    }
}
