//! Os gates da tradução da luz — o sinal, o `π` e o céu. Ver [`super`].

use super::*;
use ph2d_material::Environment;

fn luminance(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// ⭐ **Uma lâmpada de intensidade `1` chega como `π`** — o contrato do rig («o plano de frente
/// devolve `1`») escrito na convenção do MaterialX.
#[test]
fn a_lamp_of_intensity_one_arrives_as_pi() {
    let mut rig = ph2d_light::LightRig::default();
    rig.lights[0].color = [1.0; 3];
    rig.lights[0].intensity = 1.0;
    let l = lamps(&rig);
    assert_eq!(l.len(), 1, "o rig de omissão tem UMA lâmpada acesa");
    for c in l[0].radiance {
        assert!(
            (c - core::f32::consts::PI).abs() < 1.0e-6,
            "{:?}",
            l[0].radiance
        );
    }
}

/// ⭐⭐ **A lâmpada de omissão ilumina de CIMA e da ESQUERDA** — o sinal de `y`, provado contra o que
/// o rig promete por escrito (*«superior-esquerda a 30°»*, o `Light::KEY` afinado pelo Enio).
///
/// ⚠️ Sem a negação de `y` a mesma lâmpada acenderia a peça por BAIXO — a regressão que a casa já
/// pagou uma vez entre a tinta e a escultura.
#[test]
fn the_default_lamp_lights_from_the_upper_left() {
    let l = lamps(&ph2d_light::LightRig::default());
    let to_light = l[0].to_light;
    assert!(
        to_light[1] > 0.0,
        "a lâmpada está por BAIXO em espaço de vista: {to_light:?}"
    );
    assert!(
        to_light[0] < 0.0,
        "a lâmpada está à DIREITA em espaço de vista: {to_light:?}"
    );
    assert!(
        to_light[2] > 0.0,
        "a lâmpada está ATRÁS da peça: {to_light:?}"
    );

    // E o render diz o mesmo: o topo acende mais do que a base.
    let s = ph2d_material::OpenPbr::default().prepare();
    let v = [0.0, 0.0, 1.0];
    let top = s.direct([0.0, 0.8, 0.6], v, to_light, l[0].radiance);
    let bottom = s.direct([0.0, -0.8, 0.6], v, to_light, l[0].radiance);
    assert!(
        luminance(top) > luminance(bottom),
        "topo {top:?} · base {bottom:?}"
    );
}

/// ⭐ **O céu é mais claro em cima** — o céu de estúdio é o `env_ambient` da casa, com o sinal certo.
#[test]
fn the_studio_sky_is_brighter_above() {
    let sky = StudioSky;
    assert!(
        luminance(sky.irradiance([0.0, 1.0, 0.0])) > luminance(sky.irradiance([0.0, -1.0, 0.0]))
    );
    assert!(
        luminance(sky.radiance([0.0, 1.0, 0.0], 0.0))
            > luminance(sky.radiance([0.0, -1.0, 0.0], 0.0))
    );
}

/// ⭐⭐ **A irradiância é a média-cosseno da radiância** — o `3/2` que desfaz a convolução do
/// `ENV_SLOPE` provado por INTEGRAÇÃO, e não pela aritmética que o escreveu.
///
/// ⚠️ É a propriedade que liga as duas perguntas do [`Environment`]: se elas discordassem, a difusa e
/// o especular do mesmo céu acenderiam a peça com duas luzes diferentes.
#[test]
fn the_irradiance_is_the_cosine_mean_of_the_radiance() {
    let sky = StudioSky;
    let normais: [[f32; 3]; 4] = [
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.6, 0.48, 0.64],
    ];
    for n in normais {
        // A base ortonormal da normal, e a quadratura do hemisfério em (cos θ, φ).
        let helper = if n[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let cross = |a: [f32; 3], b: [f32; 3]| {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        let norm = |a: [f32; 3]| {
            let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
            [a[0] / l, a[1] / l, a[2] / l]
        };
        let t = norm(cross(helper, n));
        let b = cross(n, t);
        let (rings, sectors) = (200, 200);
        let mut sum = [0.0_f64; 3];
        let mut weight = 0.0_f64;
        for i in 0..rings {
            let mu = (i as f32 + 0.5) / rings as f32; // cos θ
            let sin = (1.0 - mu * mu).sqrt();
            for j in 0..sectors {
                let phi = (j as f32 + 0.5) / sectors as f32 * core::f32::consts::TAU;
                let dir =
                    [0, 1, 2].map(|k| n[k] * mu + t[k] * sin * phi.cos() + b[k] * sin * phi.sin());
                let l = sky.radiance(dir, 0.0);
                for k in 0..3 {
                    sum[k] += f64::from(l[k] * mu);
                }
                weight += f64::from(mu);
            }
        }
        let mean = sum.map(|s| (s / weight) as f32);
        let e = sky.irradiance(n);
        for k in 0..3 {
            assert!(
                (mean[k] - e[k]).abs() < 1.0e-3,
                "normal {n:?}: a média-cosseno da radiância dá {mean:?} e a irradiância {e:?}"
            );
        }
    }
}

/// ⏱️ **SONDA (`--ignored`): o que o modo RENDER custa, e que luz ele DEVOLVE** — com o rig, o céu,
/// o material e o olhar **do produto**.
///
/// ⛔⛔ **Ela mudou de crate em 2026-09-13, e a mudança É a correcção.** Ela nasceu na
/// `ph2d-field-render`, que **não alcança** este ficheiro — e por isso escrevia a luz à mão: uma
/// lâmpada com a direcção em literal e um céu **CONSTANTE**, onde o produto tem um [`StudioSky`]
/// que é um GRADIENTE. ⇒ a tabela de luz do `docs/Render3d/05` §6 media um programa que ninguém
/// corre, e errava no sentido optimista (`7,1 %` da peça cortada a `0` stops, contra os `10,9 %`
/// reais). *Uma segunda cópia escrita à mão do valor que uma porta produz.*
///
/// ⚠️ **Não é um gate — não há barra aqui.** É a medição que responde *«isto ainda é interactivo?»*
/// e *«a peça sai preta ou estourada?»* antes de o dono olhar para ela. Corra-a com a máquina calma
/// (`CLAUDE.md` §5.0: nenhum relógio desta workstation vale acima de `load ~5`).
#[test]
#[ignore = "sonda de medição"]
fn measure_what_the_render_mode_costs_and_paints() {
    use ph2d_field_render::{Lighting, Matcap, Orbit, shade_render, trace};
    use ph2d_view_transform::{Look, ViewTransform};
    use std::time::Instant;

    const BG: [u8; 4] = [12, 34, 56, 200];
    let (w, h) = (640, 360);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let t = Instant::now();
    let g = trace(&doc, &reg, &cam, w, h);
    let trace_ms = t.elapsed().as_secs_f64() * 1e3;

    // ⭐ **A luz é a do PRODUTO, pelas mesmas portas que o `smoke_draw` chama.**
    let lamps = lamps(&ph2d_light::LightRig::default());
    let so = [ph2d_material::OpenPbr::default().prepare()];
    // ⚠️ **Um material só, e sem `owners`** — esta sonda mede a LUZ, e a lei do material por
    // objecto tem gates próprios (`materials_tests`).
    let surface = ph2d_field_render::Surfaces {
        all: &so,
        owners: None,
    };
    let light = Lighting {
        lamps: &lamps,
        sky: &StudioSky,
    };
    let t = Instant::now();
    let render_px = shade_render(&g, &cam, &surface, &light, Look::default(), BG);
    let render_ms = t.elapsed().as_secs_f64() * 1e3;

    let texels = vec![0.5_f32; 2 * 2 * 3];
    let m = Matcap {
        side: 2,
        rgb_linear: &texels,
    };
    let t = Instant::now();
    let _ = ph2d_field_render::shade(&g, &m, BG);
    let matcap_ms = t.elapsed().as_secs_f64() * 1e3;

    // ⚠️ **Saturado é os TRÊS canais em `255`**, e não só o verde: um canal no tecto ainda tem cor,
    // e o que o olho lê como «branco chapado» é a peça a perder a forma nos três.
    let stats = |px: &[u8]| -> (f64, u32, u32, usize) {
        let (mut soma, mut verde, mut branco, mut n) = (0.0_f64, 0_u32, 0_u32, 0_usize);
        for (i, p) in px.as_chunks::<4>().0.iter().enumerate() {
            if !g.hit[i] {
                continue;
            }
            n += 1;
            soma += f64::from(p[1]);
            verde += u32::from(p[1] == 255);
            branco += u32::from(p[0] == 255 && p[1] == 255 && p[2] == 255);
        }
        (soma / n as f64, verde, branco, n)
    };
    let (_, _, branco0, pixels) = stats(&render_px);
    println!("esfera {w}x{h} · {pixels} pixels de peça · traçado {trace_ms:.1} ms");
    println!(
        "sombrear: matcap (2x2 texels) {matcap_ms:.2} ms · render {render_ms:.2} ms · \
         branco chapado {branco0}"
    );
    for stops in [-2.0, -1.0, 0.0, 1.0, 2.0] {
        let de = |view| {
            let look = Look {
                exposure_stops: stops,
                view,
            };
            stats(&shade_render(&g, &cam, &surface, &light, look, BG))
        };
        let (s_media, s_verde, s_branco, _) = de(ViewTransform::Standard);
        let (n_media, n_verde, n_branco, _) = de(ViewTransform::Neutral);
        println!(
            "  {stops:+.0} stop · Standard media {s_media:6.1} verde=255 {s_verde:6} branco {s_branco:6} \
             · Neutral media {n_media:6.1} verde=255 {n_verde:6} branco {n_branco:6}"
        );
    }
}

/// ⏱️ **SONDA — o que a vista `Neutral` faria ao MATCAP**, que é a metade da pergunta do dono que
/// ninguém tinha medido (`docs/Render3d/05` §8).
///
/// # ⚠️ A pergunta, e porque ela NÃO é só sobre o modo Render
///
/// A peça sai com `8,4 %` em branco chapado no olhar de omissão, e as duas saídas escritas são *a
/// vista passa a `Neutral`* ou *a exposição desce um stop*. ⛔ **Mas o olhar é da CENA, e o matcap
/// também passa por ele** (o doc do `shade_with` escreve-o, e é a *Color Management* do Blender):
/// trocar a omissão muda **o que o modelador sempre mostrou**, não só o modo novo.
///
/// ⚠️ E a razão de a omissão ser `Standard` está escrita no `ph2d-view-transform`: *«com exposição
/// `0` e luz dentro de `0..=1` ela devolve a entrada»* — isto é, **ela foi escolhida para o matcap**,
/// que é uma fotografia em `0..=1`. A `Neutral` dobra tudo acima do joelho (`0,76`), logo o preço da
/// troca mora exactamente nos texels claros.
///
/// ⇒ esta sonda mede esse preço no **asset da casa**, texel a texel.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_neutral_would_do_to_the_matcap() {
    use ph2d_view_transform::{Look, ViewTransform};

    let side = ph2d_mesh_render::matcap::MATCAPS[0].side as usize;
    let bytes = ph2d_mesh_render::matcap::decode(0);
    let padrao = Look::default();
    let neutra = Look {
        exposure_stops: 0.0,
        view: ViewTransform::Neutral,
    };
    let escura = Look {
        exposure_stops: -1.0,
        view: ViewTransform::Standard,
    };

    let mut deltas: std::collections::BTreeMap<i32, Vec<i32>> = std::collections::BTreeMap::new();
    let mut n = 0usize;
    let mut acima_do_joelho = 0usize;
    let mut cortados = 0usize;
    let (mut mudados_neutra, mut pior_neutra) = (0usize, 0i32);
    let (mut mudados_escura, mut pior_escura) = (0usize, 0i32);
    for texel in bytes.as_chunks::<8>().0.iter() {
        let mut lin = [0.0f32; 3];
        for (c, v) in lin.iter_mut().enumerate() {
            *v =
                half::f16::from_bits(u16::from_le_bytes([texel[c * 2], texel[c * 2 + 1]])).to_f32();
        }
        n += 1;
        if lin.iter().any(|c| *c > 0.76) {
            acima_do_joelho += 1;
        }
        if lin.iter().any(|c| *c >= 1.0) {
            cortados += 1;
        }
        let byte = |l: Look| l.apply(lin).map(ph2d_color::srgb::linear_to_srgb_byte);
        let base = byte(padrao);
        for (outra, mudados, pior) in [
            (neutra, &mut mudados_neutra, &mut pior_neutra),
            (escura, &mut mudados_escura, &mut pior_escura),
        ] {
            let b = byte(outra);
            let d = (0..3)
                .map(|c| i32::from(b[c]) - i32::from(base[c]))
                .max_by_key(|d| d.abs())
                .unwrap_or(0);
            if d != 0 {
                *mudados += 1;
            }
            if d.abs() > pior.abs() {
                *pior = d;
            }
            deltas
                .entry(outra.view as u8 as i32 * 1000 + i32::from(outra.exposure_stops as i8))
                .or_default()
                .push(d.abs());
        }
    }
    let pct = |k: usize| k as f64 / n as f64 * 100.0;
    println!("matcap {side}² = {n} texels");
    println!(
        "  acima do joelho da Neutral (0,76): {:5.1} %",
        pct(acima_do_joelho)
    );
    println!(
        "  já cortados pela Standard (≥ 1,0): {:5.1} %",
        pct(cortados)
    );
    println!(
        "  Neutral   : {:5.1} % dos texels mudam · pior {pior_neutra:+} bytes",
        pct(mudados_neutra)
    );
    println!(
        "  −1 stop   : {:5.1} % dos texels mudam · pior {pior_escura:+} bytes",
        pct(mudados_escura)
    );
    // ⚠️ **O PIOR não diz se se vê** — um extremo num punhado de texels é ruído; o que decide é a
    // MEDIANA e o p95 do desvio, que é o que o olho percorre numa face grande.
    for (chave, mut v) in deltas {
        v.sort_unstable();
        let q = |f: f64| v[((v.len() - 1) as f64 * f) as usize];
        println!(
            "  {:9}: |Δ| p50 {:3} · p95 {:3} · p99 {:3} bytes",
            if chave >= 1000 {
                "Neutral"
            } else {
                "−1 stop"
            },
            q(0.5),
            q(0.95),
            q(0.99)
        );
    }
}

/// O factor de encolhimento do lóbulo, por quadratura — o ORÁCULO da lei que a sonda abaixo mede.
///
/// # ⭐⭐⭐ Porque UM escalar chega, e é exacto
///
/// O céu de estúdio é **linear na altura**: `L(ω) = A + B·ω.y`. A média de uma função linear sobre
/// qualquer distribuição é a função avaliada na **direcção MÉDIA** dela — e a distribuição do
/// pré-filtro GGX é simétrica em torno da direcção espelhada `R`. ⇒ a componente perpendicular
/// cancela-se e sobra `E[ω] = c(α)·R`, com `c ≤ 1`.
///
/// ⇒ `média(A + B·ω.y) = A + B·c(α)·R.y`. **O erro é um factor no termo da altura, e mais nada.**
///
/// ⚠️ **É a convolução em harmónicos esféricos, não uma heurística:** uma função de grau 1 convolvida
/// com um núcleo simétrico é a mesma função de grau 1, escalada pelo coeficiente de grau 1 do núcleo.
/// O `c(α)` **é** esse coeficiente.
///
/// # A distribuição
///
/// É a do *split-sum* (Karis), que é a que o `mx_environment_prefilter` assume: `N = V = R`, amostras
/// `h` do GGX, `L = 2(N·h)h − N`, peso `N·L`, e as amostras com `N·L ≤ 0` **descartadas** — é isso
/// que faz `c` deixar de ser trivial quando o lóbulo passa do hemisfério.
///
/// ⇒ `c(α) = Σ (N·L)² / Σ (N·L)`.
fn lobe_shrink_by_quadrature(alpha: f32) -> f32 {
    let a2 = f64::from(alpha).powi(2);
    let (mut num, mut den) = (0.0f64, 0.0f64);
    const N: usize = 1 << 16;
    for i in 0..N {
        // ⚠️ Quadratura do ponto médio sobre `ξ`, não um sorteio: uma sonda que é o ORÁCULO de um
        // gate não pode ter ruído de Monte Carlo maior do que a barra que ela vai justificar.
        let xi = (i as f64 + 0.5) / N as f64;
        // Amostragem de importância do GGX: `cos²θ_h = (1 − ξ) / (1 + (α² − 1)ξ)`.
        let cos2 = (1.0 - xi) / (1.0 + (a2 - 1.0) * xi);
        let ndl = 2.0 * cos2 - 1.0;
        if ndl <= 0.0 {
            continue;
        }
        num += ndl * ndl;
        den += ndl;
    }
    if den <= 0.0 {
        return 1.0;
    }
    (num / den) as f32
}

/// ⏱️ **SONDA — o erro do céu na direcção ESPELHADA, e o factor que o cura.**
///
/// O §8 do `docs/Render3d/05` leva esta desde 13/09, com a nota de que *«ela torna-se visível no dia
/// em que houver material por objecto»*. ⭐ **Esse dia foi 14/09** — o metal passou a ser autorável
/// (§12), e a `CLAUDE.md` §0.0 diz o resto: *quem move o número que tornava algo inalcançável tem de
/// reconferir a nota.*
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_the_mirrored_direction_error_of_the_sky() {
    use ph2d_material::Environment;
    const RAW: f32 = 1.5;
    let ceu_cru = |up: f32| {
        [0, 1, 2].map(|i| {
            ph2d_light::AMBIENT * (ph2d_light::ENV_BASE[i] + RAW * ph2d_light::ENV_SLOPE[i] * up)
        })
    };
    println!("  α  ·  c(α)  ·   R.y  ·  espelhada  ·  verdade  ·  erro");
    for alpha in [0.05f32, 0.09, 0.25, 0.5, 0.75, 1.0] {
        let c = lobe_shrink_by_quadrature(alpha);
        for up in [1.0f32, 0.0, -1.0] {
            // A verdade: o céu na direcção MÉDIA do lóbulo.
            let verdade = ceu_cru(c * up);
            let espelhada = StudioSky.radiance([0.0, up, (1.0 - up * up).max(0.0).sqrt()], alpha);
            let erro = (0..3)
                .map(|i| ((espelhada[i] - verdade[i]) / verdade[i].max(1.0e-6)).abs())
                .fold(0.0f32, f32::max);
            println!(
                "{alpha:5.2} · {c:6.4} · {up:5.1} · {:10.5} · {:8.5} · {:6.2} %",
                espelhada[1],
                verdade[1],
                erro * 100.0
            );
        }
    }
}

/// ⭐⭐⭐ **A FORMA FECHADA CONCORDA COM A QUADRATURA** — o oráculo da lei do lóbulo.
///
/// # ⚠️ A barra sai do VÃO entre duas populações, e não de um número escolhido
///
/// A quadratura é do **ponto médio** sobre `2¹⁶` intervalos (não um sorteio), logo ela própria não
/// tem ruído a esconder o desvio da forma fechada. O que sobra é a aritmética `f32` da porta e o
/// degrau do integrador — e o gate exige `2e-4`, que é onde as duas se separam com folga.
///
/// ⚠️ **E a vizinhança de `α = 1` é varrida DE PROPÓSITO**: é ali que o numerador e o denominador vão
/// os dois a zero como `k³` e o cancelamento come a precisão. Sem esses pontos, o gate ficaria verde
/// sobre a única região onde a forma fechada não se pode usar crua.
///
/// **Mutação que deve sangrar:** apagar o ramo do limite (`|k| < 1e-3 → 2/3`).
#[test]
fn the_closed_form_of_the_lobe_agrees_with_the_quadrature() {
    let mut pior = 0.0f32;
    let mut onde = 0.0f32;
    // ⚠️ O varrimento inclui `0`, `1` e a vizinhança fina de `1` — os três casos de fronteira.
    let mut alphas: Vec<f32> = (0..=100).map(|i| i as f32 / 100.0).collect();
    alphas.extend([0.999, 0.9995, 0.9999, 1.0, 0.001, 0.0005]);
    for alpha in alphas {
        let nosso = super::lobe_shrink(alpha);
        let oraculo = lobe_shrink_by_quadrature(alpha);
        let d = (nosso - oraculo).abs();
        if d > pior {
            pior = d;
            onde = alpha;
        }
    }
    assert!(
        pior < 2.0e-4,
        "a forma fechada afasta-se da quadratura em {pior:.2e} (pior em α = {onde}) — a barra é 2e-4"
    );
    // ⭐ **Os dois controlos que não são coincidência**, afirmados: o lóbulo que colapsa na
    // espelhada, e o hemisfério cosseno.
    assert!(
        (super::lobe_shrink(0.0) - 1.0).abs() < 1.0e-6,
        "α = 0 tem de devolver a própria direcção espelhada"
    );
    assert!(
        (super::lobe_shrink(1.0) - 2.0 / 3.0).abs() < 1.0e-6,
        "α = 1 é o hemisfério cosseno, e o coeficiente dele é 2/3 — o mesmo que o `ENV_SLOPE` da \
         `ph2d-light` já carrega"
    );
    // ⛔ **E é MONÓTONO**: um lóbulo mais largo nunca encolhe menos. Sem isto, uma forma fechada com
    // um sinal trocado num ramo passaria no desvio médio e daria um céu que clareia com a rugosidade.
    let mut anterior = 1.0f32;
    for i in 0..=100 {
        let c = super::lobe_shrink(i as f32 / 100.0);
        assert!(
            c <= anterior + 1.0e-6,
            "o encolhimento subiu em α = {} ({anterior} → {c})",
            i as f32 / 100.0
        );
        anterior = c;
    }
}

/// ⭐⭐ **O CÉU HONRA A LARGURA DO LÓBULO QUE RECEBE** — a costura, do lado do produto.
///
/// # ⛔⛔ Porque o gate da forma fechada não chega
///
/// Ela pode estar certa e **ninguém a chamar** — era exactamente o estado anterior (`_alpha`, o
/// parâmetro deitado fora). *Um gate sobre a lei é cego a um consumidor que a ignora*, e essa é a
/// espécie de morto que o `CLAUDE.md` §5.0 chama de *«o consumidor que projecta o valor fora»*.
///
/// ⚠️ **A régua é o TERMO DA ALTURA, e o gate mede-o pelo par**: num céu avaliado no equador
/// (`dir.y = 0`) o encolhimento é invisível por construção, então o gate lá afirma a **invariância** —
/// sem essa metade, um `radiance` que ignorasse o `dir` inteiro também passaria.
///
/// **Mutação que deve sangrar:** voltar a `let up = dir[1];`.
#[test]
fn the_sky_honours_the_lobe_width_it_is_handed() {
    use ph2d_material::Environment;
    let alto = [0.0, 1.0, 0.0];
    let equador = [0.0, 0.0, 1.0];
    let liso = StudioSky.radiance(alto, 0.0);
    let rugoso = StudioSky.radiance(alto, 1.0);
    assert!(
        rugoso[1] < liso[1],
        "um lóbulo largo apontado ao topo tem de ler um céu MAIS ESCURO do que um espelho — a média \
         dele desce para o equador ({rugoso:?} contra {liso:?})"
    );
    // ⭐ **E o quanto é o que a lei promete**, não um valor qualquer.
    let a = ph2d_light::AMBIENT;
    let esperado = [0, 1, 2].map(|i| {
        a * (ph2d_light::ENV_BASE[i] + 1.5 * ph2d_light::ENV_SLOPE[i] * super::lobe_shrink(1.0))
    });
    for i in 0..3 {
        assert!(
            (rugoso[i] - esperado[i]).abs() < 1.0e-6,
            "o canal {i} não é o céu na direcção média do lóbulo"
        );
    }
    // ⛔ **No equador o encolhimento é invisível** — e tem de ser: `c·0 = 0`. É este assert que
    // impede o gate de passar sobre um `radiance` que ignorasse a direcção.
    assert_eq!(
        StudioSky.radiance(equador, 0.0),
        StudioSky.radiance(equador, 1.0),
        "no equador a largura do lóbulo não pode mudar nada"
    );
    // ⭐ E o caminho de omissão: um material liso lê **exactamente** o que lia antes.
    assert_eq!(liso, {
        let up = alto[1];
        [0, 1, 2].map(|i| a * (ph2d_light::ENV_BASE[i] + 1.5 * ph2d_light::ENV_SLOPE[i] * up))
    });
}

/// O céu como ele era **antes** de honrar a largura do lóbulo — o controlo desta medição.
///
/// ⚠️ Ele existe só aqui: um segundo `Environment` no produto seria a segunda resposta à mesma
/// pergunta. Aqui ele é o **lado A** de um A/B, e sem ele a sonda não teria com que comparar.
struct MirroredSky;
impl ph2d_material::Environment for MirroredSky {
    fn radiance(&self, dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        const RAW: f32 = 1.5;
        [0, 1, 2].map(|i| {
            ph2d_light::AMBIENT
                * (ph2d_light::ENV_BASE[i] + RAW * ph2d_light::ENV_SLOPE[i] * dir[1])
        })
    }
    fn irradiance(&self, n: [f32; 3]) -> [f32; 3] {
        ph2d_light::env_ambient([n[0], -n[1], n[2]])
    }
}

/// ⏱️ **SONDA — o que a cura do lóbulo muda NO PIXEL**, por material.
///
/// O §8 previa `p95 = 23` e `max = 33` bytes num metal rugoso, e `ruído` (`p50 = 0`, `p95 = 1`) no
/// material de omissão. ⭐ **Esta sonda mede isso no caminho do produto** — a mesma esfera, a mesma
/// luz, os mesmos dois sombreamentos, com o céu velho e o novo.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_the_lobe_cure_changes_in_the_pixel() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};

    const BG: [u8; 4] = [0, 0, 0, 0];
    let (w, h) = (640, 360);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = trace(&doc, &reg, &cam, w, h);
    let lamps = lamps(&ph2d_light::LightRig::default());
    let olhar = crate::shading::OPENING_LOOK;

    println!("material                      · muda · |Δ| p50 · p95 · max");
    for (nome, m) in [
        (
            "omissão (dieléctrico, r 0,30)",
            ph2d_material::OpenPbr::default(),
        ),
        (
            "metal polido  (metal 1, r 0,10)",
            ph2d_material::OpenPbr {
                base_metalness: 1.0,
                specular_roughness: 0.10,
                ..ph2d_material::OpenPbr::default()
            },
        ),
        (
            "metal escovado (metal 1, r 0,50)",
            ph2d_material::OpenPbr {
                base_metalness: 1.0,
                specular_roughness: 0.50,
                ..ph2d_material::OpenPbr::default()
            },
        ),
        (
            "metal fosco   (metal 1, r 1,00)",
            ph2d_material::OpenPbr {
                base_metalness: 1.0,
                specular_roughness: 1.0,
                ..ph2d_material::OpenPbr::default()
            },
        ),
    ] {
        let so = [m.prepare()];
        let surface = ph2d_field_render::Surfaces {
            all: &so,
            owners: None,
        };
        let pinta = |sky: &(dyn ph2d_material::Environment + Sync)| {
            shade_render(
                &g,
                &cam,
                &surface,
                &Lighting { lamps: &lamps, sky },
                olhar,
                BG,
            )
        };
        let antes = pinta(&MirroredSky);
        let depois = pinta(&StudioSky);
        let mut d: Vec<i32> = Vec::new();
        for (i, (a, b)) in antes
            .as_chunks::<4>()
            .0
            .iter()
            .zip(depois.as_chunks::<4>().0.iter())
            .enumerate()
        {
            if !g.hit[i] {
                continue;
            }
            d.push(
                (0..3)
                    .map(|c| i32::from(b[c]) - i32::from(a[c]))
                    .max_by_key(|v| v.abs())
                    .unwrap_or(0)
                    .abs(),
            );
        }
        let mudam = d.iter().filter(|v| **v != 0).count();
        d.sort_unstable();
        let q = |f: f64| d[((d.len() - 1) as f64 * f) as usize];
        println!(
            "{nome:30} · {:4.0}% · {:7} · {:3} · {:3}",
            mudam as f64 / d.len() as f64 * 100.0,
            q(0.5),
            q(0.95),
            d[d.len() - 1]
        );
    }
}

/// ⏱️ **SONDA — o que o MATERIAL POR OBJECTO custa ao sombreamento**, por número de folhas.
///
/// # ⚠️ A pergunta que eu mexi e não tinha reconferido (`CLAUDE.md` §0.0)
///
/// A [`measure_what_the_render_mode_costs_and_paints`] mede o modo Render com **um** material
/// (`owners: None`) — o caminho de omissão. Mas desde 13/09 o sombreamento resolve **um dono por
/// pixel** quando a peça tem mais de uma folha, e isso é trabalho novo dentro do laço mais quente do
/// quadro. *Quem acrescenta um custo ao caminho quente mede-o no caminho quente, e não numa sonda ao
/// lado.*
///
/// ⚠️ A sonda de 13/09 mediu a resolução **isolada** (`1,6 ms` a 16 folhas sobre `26 100` px). Isto é
/// outra coisa: o `shade_render` inteiro, com e sem donos, sobre a mesma peça.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_material_per_object_costs_the_shading() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};
    use std::time::Instant;

    const BG: [u8; 4] = [0, 0, 0, 0];
    let (w, h) = (640u32, 360u32);
    let cam = Orbit::default();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let lamps = lamps(&ph2d_light::LightRig::default());
    let olhar = crate::shading::OPENING_LOOK;
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("load: {}", carga.split_whitespace().next().unwrap_or("?"));
    println!("folhas ·  peça px ·  sem donos ·  com donos ·  delta ·  % de um quadro");
    for k in [2usize, 4, 8, 16] {
        let lado = (k as f32).sqrt().ceil() as usize;
        let passo = 0.9 / lado as f32;
        let mut nodes: Vec<ph2d_field::Node> = (0..k)
            .map(|i| ph2d_field::Node {
                xform: ph2d_field::Xform::at(
                    ((i % lado) as f32 - (lado - 1) as f32 * 0.5) * passo,
                    ((i / lado) as f32 - (lado - 1) as f32 * 0.5) * passo,
                    0.0,
                ),
                kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Sphere {
                    radius: passo * 0.45,
                }),
                mods: Vec::new(),
                verb: None,
            })
            .collect();
        nodes.push(ph2d_field::Node {
            xform: ph2d_field::Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Combine {
                op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                children: (0..k).map(|i| ph2d_field::NodeId(i as u32)).collect(),
            },
            mods: Vec::new(),
            verb: None,
        });
        let doc = ph2d_field::FieldDoc::new(nodes, ph2d_field::NodeId(k as u32)).expect("a grelha");
        let g = trace(&doc, &reg, &cam, w, h);
        let px = g.hit.iter().filter(|b| **b).count();

        // As folhas postas no mundo, como a `materials::Table` as constrói.
        let postas: Vec<ph2d_field::FieldDoc> = (0..k)
            .map(|i| {
                ph2d_field::FieldDoc::new(vec![doc.nodes()[i].clone()], ph2d_field::NodeId(0))
                    .expect("a folha")
            })
            .collect();
        let owners = ph2d_field_eval::owners::Owners::new(
            &postas,
            &reg,
            ph2d_field_render::hit_tolerance(cam.half_extent, w.min(h) as f32),
        );
        let so: Vec<ph2d_material::Surface> = (0..k)
            .map(|i| {
                crate::materials::surface_of(ph2d_field_ecs::FieldMaterial {
                    base_color: [i as f32 / k as f32, 0.5, 0.8],
                    ..ph2d_field_ecs::FieldMaterial::default()
                })
            })
            .collect();
        let light = Lighting {
            lamps: &lamps,
            sky: &StudioSky,
        };
        // ⚠️ A mediana de 5, com um aquecimento antes — e o MÍNIMO ao lado, porque esta máquina não
        // desce de `load ~7` (a nota do `project-memory`).
        let med = |owners: Option<&ph2d_field_eval::owners::Owners>| -> (f64, f64) {
            let s = ph2d_field_render::Surfaces { all: &so, owners };
            let _ = shade_render(&g, &cam, &s, &light, olhar, BG);
            let mut v: Vec<f64> = (0..5)
                .map(|_| {
                    let t = Instant::now();
                    let _ = shade_render(&g, &cam, &s, &light, olhar, BG);
                    t.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            v.sort_by(f64::total_cmp);
            (v[0], v[v.len() / 2])
        };
        let (sem, _) = med(None);
        let (com, _) = med(Some(&owners));
        println!(
            "{k:6} · {px:8} · {sem:9.3} · {com:9.3} · {:6.3} · {:6.1} %",
            com - sem,
            com / 16.7 * 100.0
        );
    }
}
