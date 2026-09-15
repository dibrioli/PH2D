//! Os gates da porta da deformação do canvas — ver [`super`].

use super::{CanvasWarp, WarpedDab, warped_dab};

const I: [[f32; 2]; 2] = [[1.0, 0.0], [0.0, 1.0]];

/// ⭐⭐⭐ **Em repouso a porta é o NO-OP, e não «quase»** — é isso que mantém toda pincelada deste app
/// byte a byte como era. O controlo é uma `warp` que não é a identidade: ali ela TEM de mexer.
#[test]
fn at_rest_the_door_is_a_bit_exact_no_op() {
    for (flatten, angle) in [(0.0, 0u16), (0.4, 30), (0.9, 271)] {
        assert_eq!(
            warped_dab(CanvasWarp::linear(I), flatten, angle),
            WarpedDab {
                radius_scale: 1.0,
                flatten,
                angle_deg: angle,
                curve: crate::FootprintCurve::flat(),
            },
            "em repouso a porta devolve o que entrou"
        );
    }
    // ⛔ O CONTROLO: sem ele, uma porta que devolvesse sempre a entrada passaria no gate acima.
    let esticada = warped_dab(CanvasWarp::linear([[2.0, 0.0], [0.0, 1.0]]), 0.0, 0);
    assert!(
        esticada != WarpedDab::identity(),
        "uma deformacao real TEM de mover os numeros: {esticada:?}"
    );
}

/// ⭐⭐⭐ **O caso da foto: a arte comprimida num eixo.** Com o ecrã a ver `⅓` da textura em `x`, o
/// disco de ecrã é, na textura, uma elipse **3× mais larga** em `x` — logo o raio cresce `3×` e o
/// achatamento é `1 − ⅓`, com o eixo MAIOR em `x`.
#[test]
fn a_squeezed_axis_paints_a_stretched_ellipse() {
    let w = warped_dab(CanvasWarp::linear([[1.0 / 3.0, 0.0], [0.0, 1.0]]), 0.0, 0);
    assert!(
        (w.radius_scale - 3.0).abs() < 1e-5,
        "o raio segue o eixo MAIOR: {w:?}"
    );
    assert!(
        (w.flatten - (1.0 - 1.0 / 3.0)).abs() < 1e-5,
        "o menor mede um terco do maior: {w:?}"
    );
    assert!(
        w.angle_deg == 0 || w.angle_deg == 180,
        "o eixo maior e' o x: {w:?}"
    );
}

/// ⭐⭐ **A resposta é a elipse que o ecrã vê REDONDA** — a régua é o produto, não os três números:
/// levar a elipse pintada pela deformação tem de devolver um círculo.
#[test]
fn the_painted_ellipse_comes_back_round_on_screen() {
    // Uma deformação com corte (o que um triângulo dobrado de facto faz).
    let warp = [[0.5, 0.25], [0.0, 1.5]];
    let d = warped_dab(CanvasWarp::linear(warp), 0.0, 0);
    let [c, s] = crate::texture::rotate_by_degrees(d.angle_deg);
    let (maior, menor) = (d.radius_scale, d.radius_scale * (1.0 - d.flatten));
    // Os dois semi-eixos da elipse pintada, levados ao ecrã pela deformação.
    let leva = |v: [f32; 2]| {
        [
            warp[0][0] * v[0] + warp[0][1] * v[1],
            warp[1][0] * v[0] + warp[1][1] * v[1],
        ]
    };
    let a = leva([c * maior, s * maior]);
    let b = leva([-s * menor, c * menor]);
    let (la, lb) = (
        (a[0] * a[0] + a[1] * a[1]).sqrt(),
        (b[0] * b[0] + b[1] * b[1]).sqrt(),
    );
    assert!(
        (la - 1.0).abs() < 2e-2 && (lb - 1.0).abs() < 2e-2,
        "os dois eixos tinham de chegar ao ecra' com o mesmo comprimento 1: {la} e {lb} ({d:?})"
    );
    // ⚠️ A folga de `2e-2` é a QUANTIZAÇÃO do ângulo em graus inteiros (o `dab_angle_deg` é `u16` e
    // escolhe uma entrada da tabela cozida): meio grau sobre um eixo de comprimento 1 vale `~9e-3`.
}

/// ⛔ **Uma malha dobrada sobre si mesma não pede um dab infinito** — determinante nulo devolve a
/// entrada, e o traço continua a ser o que o artista pediu.
#[test]
fn a_collapsed_triangle_is_refused_not_amplified() {
    let d = warped_dab(CanvasWarp::linear([[1.0, 2.0], [0.5, 1.0]]), 0.3, 45);
    assert_eq!(
        d,
        WarpedDab {
            radius_scale: 1.0,
            flatten: 0.3,
            angle_deg: 45,
            curve: crate::FootprintCurve::flat(),
        }
    );
}

/// ⭐⭐⭐ **O QUE O ARTISTA VÊ É A ELIPSE AUTORADA — e é essa que o anel do cursor tem de desenhar.**
///
/// ⛔⛔⛔ **Report do dono com foto (2026-09-14): *«o gizmo do pincel se deforma ao passar por cima
/// das faces dobradas»*.** Eu tinha acabado de pôr o anel a percorrer a pegada que o MOTOR emite —
/// e essa é a elipse da TEXTURA, a que a deformação endireita. No ecrã ela aparece **redonda**;
/// desenhá-la directamente no ecrã mostra-a **torta**. *Corrigi uma coisa que já estava certa, e a
/// foto é a prova.*
///
/// A lei, escrita como identidade: a pegada que o motor pinta, **levada pela deformação**, é a
/// elipse que o artista autorou. `W · (W⁻¹·E) = E`, e o gate mede-o sobre a saída real da porta —
/// que passa por uma decomposição em três números e podia não voltar.
#[test]
fn the_painted_dab_seen_through_the_warp_is_the_authored_ellipse() {
    use crate::footprint::FootprintDeform;
    let mut casos = 0;
    for warp in [
        [[2.0_f32, 0.0], [0.0, 1.0]],
        [[1.0, 0.0], [0.0, 3.0]],
        [[1.0 / 3.0, 0.0], [0.0, 1.0]],
        [[1.0, 0.4], [-0.2, 1.3]],
    ] {
        for (flatten, angle) in [(0.0_f32, 0_u16), (0.4, 0), (0.4, 37), (0.25, 115)] {
            let autorada = FootprintDeform::new(flatten, angle);
            let d = warped_dab(CanvasWarp::linear(warp), flatten, angle);
            let pintada = FootprintDeform::new(d.flatten, d.angle_deg);
            // A fronteira do que o motor pinta, escalada pelo raio que ele usa e LEVADA pela
            // deformação: é isto que chega ao olho.
            let vista: Vec<[f32; 2]> = (0..256)
                .map(|k| {
                    let p = pintada.outline_at(k as f32 / 256.0);
                    let (x, y) = (p[0] * d.radius_scale, p[1] * d.radius_scale);
                    [
                        warp[0][0] * x + warp[0][1] * y,
                        warp[1][0] * x + warp[1][1] * y,
                    ]
                })
                .collect();
            // E a autorada, que é o que o anel desenha.
            for k in 0..256 {
                let a = autorada.outline_at(k as f32 / 256.0);
                let perto = vista
                    .iter()
                    .map(|v| (v[0] - a[0]).hypot(v[1] - a[1]))
                    .fold(f32::INFINITY, f32::min);
                casos += 1;
                assert!(
                    perto < 2e-2,
                    "com warp {warp:?}, flatten {flatten} e ângulo {angle}°, o ponto {k} da elipse \
                     AUTORADA está a {perto} da fronteira do que o motor de facto pinta — o anel e \
                     a tinta mostram formas diferentes"
                );
            }
        }
    }
    assert_eq!(casos, 4 * 4 * 256, "o corpus mudou de tamanho");
}

/// Os monómios de grau `2` e `3` de `d`, na ordem da [`crate::FootprintCurve`].
fn monomios(d: [f32; 2]) -> [f32; 7] {
    let (x, y) = (d[0], d[1]);
    let (xx, xy, yy) = (x * x, x * y, y * y);
    [xx, xy, yy, xx * x, xx * y, xy * y, yy * y]
}

/// O mapa VERDADEIRO da arte dobrada: `ecrã(d) = L·d + K(d)`, com o raio do footprint em `1` (as
/// contas desta porta cancelam-no, e é isso que deixa o gate viver sem malha nenhuma).
fn dobra(linear: [[f32; 2]; 2], k: [[f32; 7]; 2], d: [f32; 2]) -> [f32; 2] {
    let m = monomios(d);
    let mut out = [
        linear[0][0] * d[0] + linear[0][1] * d[1],
        linear[1][0] * d[0] + linear[1][1] * d[1],
    ];
    for (o, row) in out.iter_mut().zip(k.iter()) {
        for (c, mk) in row.iter().zip(m.iter()) {
            *o += c * mk;
        }
    }
    out
}

/// O pior desvio entre a fronteira do que o motor pinta (levada pela dobra) e a elipse autorada.
///
/// ⚠️⚠️ **É PONTUAL, e a primeira redacção não era** — ela procurava, para cada ponto da elipse
/// autorada, o ponto mais próximo da fronteira pintada. Isso mede também a **desigualdade das duas
/// amostragens**: com uma dobra forte a parametrização do contorno fica muito não-uniforme, os
/// pontos vizinhos afastam-se, e a régua lia `0,062` sobre uma composição que a álgebra diz ser
/// EXACTA. *Uma régua que compara dois conjuntos por vizinho mais próximo mede o espaçamento deles,
/// não a lei.*
///
/// ⇒ a lei é pontual e não precisa de emparelhar nada: **todo ponto da fronteira pintada, visto
/// pela dobra, está SOBRE a elipse autorada** — e «estar sobre» é o amostrador dela dar `1`.
fn desvio(
    linear: [[f32; 2]; 2],
    k: [[f32; 7]; 2],
    flatten: f32,
    angle: u16,
    d: super::WarpedDab,
) -> f32 {
    use crate::footprint::FootprintDeform;
    let autorada = FootprintDeform::new(flatten, angle);
    let pintada = FootprintDeform::new(d.flatten, d.angle_deg).with_curve(d.curve);
    (0..512)
        .map(|j| {
            let p = pintada.outline_at(j as f32 / 512.0);
            let v = dobra(linear, k, [p[0] * d.radius_scale, p[1] * d.radius_scale]);
            (autorada.falloff_t(v[0], v[1]) - 1.0).abs()
        })
        .fold(0.0_f32, f32::max)
}

/// ⭐⭐⭐ **A IDENTIDADE, GENERALIZADA PARA A DOBRA** — e é este o gate que prova a wave inteira.
///
/// O irmão [`the_painted_dab_seen_through_the_warp_is_the_authored_ellipse`] mede `W·(W⁻¹E) = E`
/// para um `W` **linear**. Aqui o mapa da arte é `L·d + K(d)`, com os graus `2` e `3` de uma dobra
/// a sério — e a lei tem de continuar a valer: *o que o motor pinta, levado pela arte dobrada, É a
/// elipse que o artista autorou*.
///
/// ⚠️ **Ele mede a cadeia toda de uma vez**, que é a razão de existir: a decomposição em três
/// números **e** a rotação `V` que leva a curvatura ao referencial da pegada
/// ([`super::canvas_warp_curve`]) **e** o `outline_at` a perseguir uma curva de nível que já não é
/// uma elipse. Qualquer uma delas errada, e o desvio salta.
///
/// # As TRÊS asserções, e porque são três
///
/// 1. ⛔⛔ **NUNCA PIOR que não corrigir** — sobre toda a população, dobras violentas incluídas.
///    *Esta é a que este módulo já pagou uma vez*: na W11b a correcção pela facete deixava a marca
///    menos redonda do que deixá-la em paz. Medido aqui: sem a cerca da
///    [`crate::FootprintDeform::is_sampleable`] o pior caso sai a **`0,994`** contra `0,611` sem
///    correcção — e com ela sai **exactamente** `0,611`, que é a degradação certa.
/// 2. ⭐ **MUITO melhor onde a dobra é amostrável** — senão a cerca podia curar tudo desistindo de
///    tudo, e as duas primeiras asserções ficavam verdes sobre o produto de ontem.
/// 3. ⛔ **E um PISO DE POPULAÇÃO**: pelo menos dois terços do corpus tem de conservar a
///    curvatura. *Sem ele, uma cerca apertada demais apaga a wave e nenhum número acusa.*
#[test]
fn the_painted_dab_seen_through_a_fold_is_the_authored_ellipse() {
    let (mut casos, mut curvos) = (0, 0);
    let (mut pior_razao, mut pior_com, mut melhor_sem) = (0.0_f32, 0.0_f32, f32::INFINITY);
    for linear in [
        [[1.0_f32, 0.0], [0.0, 1.0]],
        [[0.6, 0.0], [0.0, 1.0]],
        [[1.0, 0.35], [-0.2, 1.2]],
    ] {
        for k in [
            [[0.22_f32, 0.0, -0.13, 0.0, 0.0, 0.0, 0.0], [0.0; 7]],
            [[0.0; 7], [0.05, 0.0, 0.0, 0.16, -0.08, 0.0, 0.11]],
            [
                [0.10, -0.06, 0.04, 0.03, 0.0, -0.02, 0.0],
                [-0.05, 0.11, 0.0, 0.0, 0.06, 0.0, -0.04],
            ],
        ] {
            for (flatten, angle) in [(0.0_f32, 0_u16), (0.4, 37), (0.25, 115)] {
                let d = warped_dab(CanvasWarp { linear, curve: k }, flatten, angle);
                let com = desvio(linear, k, flatten, angle, d);
                let sem = desvio(
                    linear,
                    k,
                    flatten,
                    angle,
                    super::WarpedDab {
                        curve: crate::FootprintCurve::flat(),
                        ..d
                    },
                );
                casos += 1;
                pior_razao = pior_razao.max(com / sem);
                if !d.curve.is_flat() {
                    curvos += 1;
                    pior_com = pior_com.max(com);
                    melhor_sem = melhor_sem.min(sem);
                }
            }
        }
    }
    assert_eq!(casos, 3 * 3 * 3, "o corpus mudou de tamanho");
    assert!(
        pior_razao <= 1.0 + 1e-4,
        "houve um caso em que a curvatura deixou a marca PIOR do que não corrigir \
         (razão {pior_razao}) — é o defeito da W11b de volta, um grau acima"
    );
    assert!(
        pior_com < 2.5e-2,
        "onde a dobra é amostrável, o que o motor pinta ainda se afasta {pior_com} da elipse \
         autorada — a curvatura não está a fechar a lei"
    );
    assert!(
        melhor_sem > 4.0 * pior_com,
        "o MELHOR caso sem curvatura ({melhor_sem}) tem de estar muito acima do PIOR com ela \
         ({pior_com}) — sem margem, este gate deixou de distinguir a cura do defeito"
    );
    assert!(
        curvos * 3 >= casos * 2,
        "só {curvos} de {casos} casos conservaram a curvatura — a cerca está a apagar a wave, e \
         as asserções de cima ficariam verdes sobre o produto de ontem"
    );
}

/// Quantos dabs um caminho de `400 px` recebe, com a pegada que `warp` e `(flatten, angle)` dão.
fn dabs_no_caminho(warp: CanvasWarp, flatten: f32, angle: u16, dir: [f32; 2]) -> usize {
    use crate::{BrushSpec, Dab, Stroke, StrokePoint};
    let base = BrushSpec {
        radius_px: 24.0,
        spacing: 0.1,
        ..BrushSpec::default()
    };
    let w = warped_dab(warp, flatten, angle);
    let spec = BrushSpec {
        radius_px: base.radius_px * w.radius_scale,
        dab_flatten: w.flatten,
        dab_angle_deg: w.angle_deg,
        dab_curve: w.curve,
        ..base
    };
    let ponto = |x: f32, y: f32| StrokePoint {
        pos: [x, y],
        pressure: 1.0,
    };
    let mut s = Stroke::new(spec, crate::Dynamics::default(), 7);
    let mut out: Vec<Dab> = Vec::new();
    s.begin(ponto(0.0, 0.0), &mut out);
    s.extend(ponto(dir[0] * 400.0, dir[1] * 400.0), &mut out);
    out.len()
}

/// ⭐⭐⭐ **A DENSIDADE DO TRAÇO É DO CAMINHO, NUNCA DA DEFORMAÇÃO DA ARTE** — o report da 6.ª foto
/// do dono (*«pinta com diâmetro menor onde é mais estreito»*, 2026-09-14).
///
/// ⛔⛔ O passo era `fracção × 2 × radius_px`, e o `radius_px` é o semi-eixo **MAIOR** — que sobre
/// arte dobrada carrega a compressão do eixo do OUTRO lado. O mesmo caminho de `400 px` de ecrã,
/// a andar pelo eixo que a arte **não** comprime, recebia `44` · `22` · `11` · **`5`** dabs. Com um
/// pincel macio e cobertura abaixo de `1`, oito vezes menos marcas não constroem a tinta: o traço
/// sai mais fino **exactamente onde a arte é estreita**.
///
/// ⚠️ **É a lei-mãe deste módulo, um nível acima:** *o traço é facto do CAMINHO, nunca de quão fino
/// o motor amostrou o caminho.* Aqui quem amostrava mal era o próprio motor.
#[test]
fn the_stroke_density_is_the_paths_never_the_arts_fold() {
    let sem = dabs_no_caminho(CanvasWarp::rest(), 0.0, 0, [0.0, 1.0]);
    assert!(sem > 20, "a fixtura tem de emitir dabs que cheguem: {sem}");
    for k in [0.5_f32, 0.25, 0.125] {
        // ⚠️ O traço anda em `y`, o eixo que esta deformação NÃO toca: em ecrã ele percorre
        // exactamente os mesmos `400 px` em todas as células, logo a contagem tem de ser a mesma.
        let com = dabs_no_caminho(
            CanvasWarp::linear([[k, 0.0], [0.0, 1.0]]),
            0.0,
            0,
            [0.0, 1.0],
        );
        assert_eq!(
            com,
            sem,
            "com a arte comprimida {}× o caminho recebeu {com} dabs contra {sem} — a densidade do \
             traço está a seguir a deformação da arte",
            1.0 / k
        );
    }
}

/// ⭐⭐ **E a MESMA lei vale para o achatamento que o ARTISTA autorou** — que sempre teve o mesmo
/// defeito e que nenhum gate via, porque *todo o corpus desta crate está no ponto NEUTRO desse
/// knob*.
///
/// Uma pena calígrafica a andar pelo lado FINO dava passos do lado GROSSO e deixava o traço aos
/// bocados. ⛔ A metade anti-vácuo é a razão entre as duas direcções: sem ela, um passo que
/// ignorasse a direcção passaria (as duas contagens seriam iguais).
#[test]
fn a_flattened_nib_steps_finer_along_its_thin_axis() {
    const FLATTEN: f32 = 0.8;
    let grosso = dabs_no_caminho(CanvasWarp::rest(), FLATTEN, 0, [1.0, 0.0]);
    let fino = dabs_no_caminho(CanvasWarp::rest(), FLATTEN, 0, [0.0, 1.0]);
    assert!(
        fino as f32 > grosso as f32 * 3.0,
        "a andar pelo lado FINO da pena o traço recebeu {fino} dabs e pelo GROSSO {grosso} — o \
         passo está a ignorar a direcção do caminho, e o lado fino sai aos bocados"
    );
    // ⛔ E o lado grosso tem de continuar a ser o de sempre: esta lei não pode adensar o que já
    // estava certo.
    let redondo = dabs_no_caminho(CanvasWarp::rest(), 0.0, 0, [1.0, 0.0]);
    assert_eq!(
        grosso, redondo,
        "andar pelo eixo MAIOR de uma pena achatada deixou de dar o passo de sempre"
    );
}

/// Sonda: **quantos dabs um caminho recebe**, com e sem compressão da arte — a tabela que achou o
/// defeito da 6.ª foto. Hoje as quatro linhas dão `44`; antes da cura davam `44` · `22` · `11` ·
/// `5`.
#[test]
#[ignore = "sonda: o espacamento contra a compressao"]
fn probe_o_espacamento_contra_a_compressao() {
    use crate::{BrushSpec, Dab, Stroke, StrokePoint};
    let ponto = |x: f32, y: f32| StrokePoint {
        pos: [x, y],
        pressure: 1.0,
    };
    println!(
        "\n  compressao   raio emitido   passo   dabs em 400 px de TEXTURA   dabs por 400 px de ECRA"
    );
    for k in [1.0_f32, 0.5, 0.25, 0.125] {
        let base = BrushSpec {
            radius_px: 24.0,
            spacing: 0.1,
            ..BrushSpec::default()
        };
        let w = warped_dab(CanvasWarp::linear([[k, 0.0], [0.0, 1.0]]), 0.0, 0);
        let spec = BrushSpec {
            radius_px: base.radius_px * w.radius_scale,
            dab_flatten: w.flatten,
            dab_angle_deg: w.angle_deg,
            ..base
        };
        let mut s = Stroke::new(spec, crate::Dynamics::default(), 7);
        let mut out: Vec<Dab> = Vec::new();
        s.begin(ponto(0.0, 0.0), &mut out);
        // ⚠️ O traço anda ao longo de `y`, que é o eixo NÃO comprimido: em ecrã ele percorre
        // exactamente os mesmos `400 px` em todas as células.
        s.extend(ponto(0.0, 400.0), &mut out);
        println!(
            "  {k:8.4}   {:10.1}   {:6.2}   {:10}                  {:.0}",
            spec.radius_px,
            spec.dab_spacing_px(),
            out.len(),
            out.len() as f32
        );
    }
}
