//! ⭐⭐⭐ **OS TERRAÇOS — o report do dono de 2026-09-17** sobre o ricochete no dispositivo:
//! *«funciona, é rápido, mas é de baixa qualidade (como se fosse muitas sombras duras)»*, com a foto
//! do interior do vaso e os **arcos concêntricos** dentro dele.
//!
//! # ⛔⛔⛔ O que ele fotografou não é RUÍDO, e a diferença decide a cura
//!
//! Ruído é desvio INDEPENDENTE por pixel; o que a foto tem são **arcos**, que é desvio
//! CORRELACIONADO. A [`crate::occlusion::cone_dir`] traz a recusa medida de 2026-09-15 contra a cura
//! do ruído (sortear por pixel) — ela foi apagada com o report ANTERIOR do dono na mão, porque
//! semear no índice do pixel fazia a peça **ferver** ao rodar a câmera.
//!
//! ⇒ *a pergunta não é «como suavizo isto», é «porque é que o céu, com AS MESMAS 48 direcções, não
//! tem terraços e o ricochete tem».*
//!
//! ⚠️ **A régua vive em [`crate::banda`]**, não aqui: ela tem dois consumidores (esta caixa e a peça
//! do dono, em `ph2d_app_field3d::render_bounce_vaso_tests`) e duas cópias divergiriam.

use super::cornell::{Escuro, FUNDO, LAMPADA, caixa, camara, donos};
use crate::banda::terracos;
use crate::{Gbuffer, Orbit, Surfaces, trace};
use ph2d_field_eval::hybrid::Registry;
use ph2d_material::OpenPbr;

/// A cena, o g-buffer e as superfícies — a caixa de Cornell, que é **fechada**: ali quase toda
/// direcção acerta em alguma coisa.
fn cena(
    lado: u32,
) -> (
    ph2d_field::FieldDoc,
    Registry,
    Orbit,
    Gbuffer,
    Vec<ph2d_material::Surface>,
    ph2d_field_eval::owners::Owners,
) {
    let (doc, postas, materiais) = caixa();
    let reg = Registry::new();
    let cam = camara();
    let g = trace(&doc, &reg, &cam, lado, lado);
    let donos = donos(&postas, &reg, &cam, lado);
    let prontos: Vec<ph2d_material::Surface> = materiais.iter().map(OpenPbr::prepare).collect();
    (doc, reg, cam, g, prontos, donos)
}

/// ⏱️⭐⭐⭐ **A SONDA QUE NOMEIA A CAUSA: o céu e o ricochete, lado a lado, nas MESMAS direcções.**
///
/// As duas metades correm o mesmo conjunto, com o mesmo peso, na mesma cena, no mesmo `k`, e passam
/// pelo mesmo borrão ⇒ *tudo o que difere entre as duas colunas é o que se pergunta por direcção.*
#[test]
#[ignore = "sonda de diagnóstico: imprime a tabela dos terraços"]
fn sonda_os_terracos_das_duas_metades() {
    let lado = 192u32;
    let (doc, reg, cam, g, prontos, donos) = cena(lado);
    let surfaces = Surfaces {
        all: &prontos,
        owners: Some(&donos),
    };
    println!("  cena: caixa de Cornell FECHADA, {lado}×{lado}, borrão 3×3 nos dois lados");
    println!("  direcções ·  CÉU p99 / máx   ·  RICOCHETE p99 / máx");
    for dirs in [16u32, 32, 48, 96, 256] {
        let ceu = crate::blur_occlusion(&g, &crate::occlusion(&doc, &reg, &cam, &g, dirs));
        let ric = crate::blur_bounce(
            &g,
            &crate::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[LAMPADA], dirs),
        );
        let (ce, cm) = terracos(&g, &|i| ceu[i]);
        let (re, rm) = terracos(&g, &|i| (ric[i][0] + ric[i][1] + ric[i][2]) / 3.0);
        println!("  {dirs:>9} · {ce:>8.4} / {cm:>7.4} · {re:>9.4} / {rm:>7.4}");
    }
    let _ = (Escuro, FUNDO);
}

/// ⏱️⭐⭐⭐ **ONDE está o degrau — a decomposição POR DIRECÇÃO do pior terraço.**
///
/// Ela não testa hipótese nenhuma: **abre** a soma. Para o trio de pixels com a maior quebra,
/// imprime, direcção a direcção, o que cada uma contribuiu nos três — e a coluna que salta nomeia o
/// mecanismo.
///
/// ⭐ Foi ela que provou, em vez de inferir, que nesta CAIXA o degrau é o polo `1/r²` da lâmpada
/// (uma direcção de `48` a bater a `6 cm` dela, com `1/r²` a ler `385` contra `1,3–5,6` de todas as
/// outras) — e que essa lâmpada **não é a do produto**. Ver
/// `ph2d_app_field3d::render_bounce_vaso_tests`.
#[test]
#[ignore = "sonda de diagnóstico: abre a soma do pior terraço"]
fn sonda_de_onde_vem_o_degrau() {
    let lado = 192u32;
    let (doc, reg, cam, g, prontos, donos) = cena(lado);
    let surfaces = Surfaces {
        all: &prontos,
        owners: Some(&donos),
    };
    let dirs = 48u32;
    let ric = crate::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[LAMPADA], dirs);
    let lum = |i: usize| (ric[i][0] + ric[i][1] + ric[i][2]) / 3.0;

    // O pior trio, SEM borrão — o borrão espalha o degrau por três pixels e esconde qual é o do meio.
    let (w, h) = (lado as usize, lado as usize);
    let mesma = |a: usize, b: usize| {
        let (u, v) = (g.normal[a], g.normal[b]);
        u[0] * v[0] + u[1] * v[1] + u[2] * v[2] >= crate::OCCLUSION_BLUR_COS
    };
    let (mut pior, mut onde) = (0.0f32, None);
    for y in 0..h {
        for x in 1..w - 1 {
            let (a, b, c) = (y * w + x - 1, y * w + x, y * w + x + 1);
            if !g.hit[a] || !g.hit[b] || !g.hit[c] || !mesma(a, b) || !mesma(b, c) {
                continue;
            }
            let q = (lum(a) - 2.0 * lum(b) + lum(c)).abs();
            if q > pior {
                pior = q;
                onde = Some((a, b, c));
            }
        }
    }
    let Some((a, b, c)) = onde else {
        println!("  nenhum trio");
        return;
    };
    println!(
        "  pior quebra: {pior:.5} (lum {:.5} / {:.5} / {:.5})",
        lum(a),
        lum(b),
        lum(c)
    );
    println!("  dir ·   peso  ·   contribuição A/B/C   ·  salto");
    let mut linhas: Vec<(f32, String)> = Vec::new();
    for k in 0..dirs {
        let mut col = [(0.0f32, 0.0f32); 3];
        let f =
            crate::bounce::bounce_slice(&doc, &reg, &cam, &g, &surfaces, &[LAMPADA], k, 1, dirs);
        for (m, &i) in [a, b, c].iter().enumerate() {
            let s = f.sum[i];
            col[m] = (f.weight[i], (s[0] + s[1] + s[2]) / 3.0);
        }
        let salto = (col[0].1 - 2.0 * col[1].1 + col[2].1).abs();
        linhas.push((
            salto,
            format!(
                "  {k:>3} · {:>6.4} · {:>9.5} {:>9.5} {:>9.5} · {salto:.5}",
                col[1].0, col[0].1, col[1].1, col[2].1
            ),
        ));
    }
    linhas.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap_or(std::cmp::Ordering::Equal));
    for (_, l) in linhas.iter().take(8) {
        println!("{l}");
    }
    let soma: f32 = linhas.iter().map(|l| l.0).sum();
    println!("  soma dos saltos das {dirs}: {soma:.5}   (a quebra total é {pior:.5})");
    let _ = (Escuro, FUNDO);
}

/// ⭐⭐⭐ **O RICOCHETE LÊ A GÉMEA FOSCA — e o gate afirma-o por IGUALDADE, não por «melhorou».**
///
/// A lei ([`ph2d_material::Surface::matte`]) diz que a luz recolhida do ponto acertado é a **difusa**
/// dele. ⇒ o ricochete de um material **brilhante** tem de ser **exactamente** o ricochete da gémea
/// fosca dele, porque `matte(matte(X)) == matte(X)`.
///
/// ⚠️⚠️ **E o CONTROLO é metade do gate:** sem ele isto passaria sobre uma cena onde o brilho não
/// muda nada — a mesma armadilha que a fixtura de paridade desta linha já pagou (*«3 formas convexas
/// não contêm o fenómeno»*). O controlo mede que os dois materiais são de facto diferentes **pela
/// mesma porta**, sombreando o mesmo ponto.
#[test]
fn o_ricochete_recolhe_a_difusa_do_ponto_acertado() {
    let lado = 96u32;
    let (doc, reg, cam, g, _prontos, donos) = cena(lado);
    // ⭐ Um material MUITO brilhante: é nele que a diferença entre a lei e a sua ausência é maior.
    let brilhante = OpenPbr {
        base_color: [0.70, 0.70, 0.70],
        specular_weight: 1.0,
        specular_roughness: 0.05,
        ..OpenPbr::default()
    };
    let lustroso: Vec<ph2d_material::Surface> =
        std::iter::repeat_n(brilhante.prepare(), 7).collect();
    let fosco: Vec<ph2d_material::Surface> =
        lustroso.iter().map(ph2d_material::Surface::matte).collect();

    // ── O CONTROLO: os dois materiais NÃO são o mesmo, medidos pela porta que o ricochete usa ──
    // ⚠️⚠️ **O olho vai no ESPELHO da luz**, `2(n·l)n − l`, e não num ângulo qualquer: a 1.ª
    // redacção pôs `v` a `[0, 0,30, 0,95]` e o controlo mediu `0,0041` — *um lóbulo especular é uma
    // quase-delta, e fora do pico dele o brilhante e o fosco sombreiam igual*. É exactamente a
    // propriedade que esta wave existe para nomear, e ela mordeu primeiro na régua.
    let n = [0.0f32, 1.0, 0.0];
    let l = [0.0f32, 0.55, 0.84];
    let v = [0.0f32, 2.0f32.mul_add(l[1], -l[1]), -l[2]];
    let a = lustroso[0].direct(n, v, l, [1.0; 3]);
    let b = fosco[0].direct(n, v, l, [1.0; 3]);
    let separacao = (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0f32, f32::max);
    assert!(
        separacao > 0.05,
        "o CONTROLO caiu: o brilhante e a gémea fosca dele sombreiam igual ({separacao:.6}) — \
         então este gate não pode afirmar nada sobre qual dos dois o ricochete lê"
    );

    let com = |mats: &[ph2d_material::Surface]| {
        crate::bounce::bounce_pass(
            &doc,
            &reg,
            &cam,
            &g,
            &Surfaces {
                all: mats,
                owners: Some(&donos),
            },
            &[LAMPADA],
            16,
        )
    };
    let (x, y) = (com(&lustroso), com(&fosco));
    let mut pior = 0.0f32;
    let mut onde = 0usize;
    for i in 0..x.len() {
        for k in 0..3 {
            if (x[i][k] - y[i][k]).abs() > pior {
                pior = (x[i][k] - y[i][k]).abs();
                onde = i;
            }
        }
    }
    assert!(
        pior == 0.0,
        "o ricochete do material BRILHANTE tinha de ser o da gémea FOSCA dele, ao bit — \
         pior desvio {pior:e} no pixel {onde} ({:?} contra {:?})",
        x[onde],
        y[onde]
    );
    // ⚠️ E que o ricochete não é trivialmente ZERO — senão a igualdade acima é a de dois nadas.
    let maior = y
        .iter()
        .flat_map(|c| c.iter())
        .fold(0.0f32, |m, &c| m.max(c));
    assert!(maior > 1e-4, "o ricochete desta cena é nulo ({maior:e})");
    let _ = (Escuro, FUNDO);
}
