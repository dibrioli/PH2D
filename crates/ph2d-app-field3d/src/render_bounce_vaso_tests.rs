//! ⭐⭐⭐ **A RÉGUA NA PEÇA DO DONO** — o report de 2026-09-17 (*«funciona, é rápido, mas é de baixa
//! qualidade (como se fosse muitas sombras duras)»*, foto do interior do vaso, arcos concêntricos).
//!
//! # ⛔⛔⛔ Porque esta sonda existe SEPARADA da caixa de Cornell
//!
//! A primeira medição desta jornada correu na caixa de Cornell e nomeou um mecanismo verdadeiro
//! **daquela caixa**: a lâmpada dela é um ponto a `6 cm` do tecto, logo a irradiância indirecta tem
//! um polo `1/r²` que `48` direcções fixas amostram como pico (ver
//! [`ph2d_field_render::bounce::pegada_do_cone`], com a decomposição direcção a direcção).
//!
//! ⚠️⚠️ **E a cena do produto NÃO tem essa lâmpada.** A [`crate::lights::opening_distance`] põe a
//! primeira luz a `2 × half_extent` do alvo — **fora da peça** —, por uma razão escrita lá: *«uma
//! lâmpada que nasce dentro da peça está por dentro do sólido»*. ⇒ curar o polo da caixa de Cornell
//! podia não tocar num único pixel do que o dono fotografou.
//!
//! *É a lei que esta casa já pagou cinco vezes: a fixtura tem de ser a da cena que o dono usou.*

use ph2d_field_render::{Orbit, Surfaces};

/// A peça e a câmera do smoke `PH2D_FIELD_SMOKE=5` — o **torno**, um vaso oco, que é a peça da foto.
fn vaso() -> (ph2d_field::FieldDoc, Orbit) {
    (
        crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION),
        Orbit::default(),
    )
}

/// ⏱️⭐⭐⭐ **OS TERRAÇOS NA PEÇA DO DONO, com a luz do PRODUTO.**
#[test]
#[ignore = "sonda de diagnóstico: a régua na peça do report"]
fn sonda_os_terracos_no_vaso() {
    let (doc, cam) = vaso();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (w, h) = (256u32, 256u32);
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let mats = vec![ph2d_material::OpenPbr::default().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    // ⭐ **A luz do PRODUTO**, pela porta que a cena usa — nunca um literal.
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
    let dl = [
        lampada.world[0] - bola.center[0],
        lampada.world[1] - bola.center[1],
        lampada.world[2] - bola.center[2],
    ];
    let r = (dl[0] * dl[0] + dl[1] * dl[1] + dl[2] * dl[2]).sqrt();
    println!(
        "  vaso: raio da bola {:.4} · lâmpada a {r:.4} do centro ⇒ a superfície mais perto dela \
         está a ~{:.4}",
        bola.radius,
        (r - bola.radius).max(0.0)
    );
    println!("  direcções ·  CÉU p99 / máx   ·  RICOCHETE p99 / máx");
    for dirs in [16u32, 32, 48, 96] {
        let ceu = ph2d_field_render::blur_occlusion(
            &g,
            &ph2d_field_render::occlusion(&doc, &reg, &cam, &g, dirs),
        );
        let ric = ph2d_field_render::blur_bounce(
            &g,
            &ph2d_field_render::bounce::bounce_pass(
                &doc,
                &reg,
                &cam,
                &g,
                &surfaces,
                &[lampada],
                dirs,
            ),
        );
        let (ce, cm) = ph2d_field_render::banda::terracos(&g, &|i| ceu[i]);
        let (re, rm) =
            ph2d_field_render::banda::terracos(&g, &|i| (ric[i][0] + ric[i][1] + ric[i][2]) / 3.0);
        println!("  {dirs:>9} · {ce:>8.4} / {cm:>7.4} · {re:>9.4} / {rm:>7.4}");
    }
}

/// ⏱️⭐⭐⭐ **A ESCADA DO BORRÃO — os terraços E o que se paga por eles.**
///
/// ⚠️⚠️ **Uma régua só não decide isto.** Borrar apaga terraços apagando também o contacto
/// verdadeiro ⇒ a tabela tem de trazer, lado a lado, **os terraços** (o que o dono fotografou) e o
/// **desvio contra a referência CONVERGIDA** (a mesma lei com muitas direcções, sem borrão nenhum).
///
/// *O borrão certo é o que mata a primeira coluna sem mover a segunda.*
///
/// # ⭐ Porque o ricochete pode levar um borrão mais largo que a oclusão
///
/// Eles são grandezas diferentes. A oclusão tem conteúdo de alta frequência VERDADEIRO — o
/// escurecimento de contacto —, e alargar o borrão dela apaga-o. O ricochete é **irradiância** de um
/// recolhedor de hemisfério: ele é suave por construção, e o que ali tem alta frequência é o
/// estimador, não a resposta. ⇒ herdar a largura da oclusão foi uma escolha por parentesco, nunca
/// uma medição — e é esta tabela que a mede.
#[test]
#[ignore = "sonda de calibração: a escada do borrão do ricochete"]
fn sonda_a_escada_do_borrao() {
    let (doc, cam) = vaso();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (w, h) = (256u32, 256u32);
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let mats = vec![ph2d_material::OpenPbr::default().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    let cru = |dirs: u32| {
        ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], dirs)
    };
    // ⭐ A referência: MUITAS direcções e ZERO borrão — ela não precisa de suavização porque amostra
    // bem, e é por isso que é ela o alvo e não uma versão mais borrada de nós próprios.
    let convergida = cru(512);
    let desvio = |c: &[[f32; 3]]| -> f32 {
        let (mut s, mut n) = (0.0f64, 0usize);
        for i in 0..c.len() {
            if !g.hit[i] {
                continue;
            }
            for k in 0..3 {
                s += f64::from((c[i][k] - convergida[i][k]).abs());
            }
            n += 3;
        }
        #[allow(clippy::cast_possible_truncation)]
        let d = (s / n.max(1) as f64) as f32;
        d
    };
    let escala = {
        let (mut s, mut n) = (0.0f64, 0usize);
        for (i, c) in convergida.iter().enumerate() {
            if g.hit[i] {
                s += f64::from(c[0]) + f64::from(c[1]) + f64::from(c[2]);
                n += 3;
            }
        }
        #[allow(clippy::cast_possible_truncation)]
        let e = (s / n.max(1) as f64) as f32;
        e
    };
    println!("  vaso 256×256 · 48 direcções · referência: 512 direcções sem borrão");
    println!("  a MÉDIA da referência: {escala:.5}");
    let ceu = ph2d_field_render::blur_occlusion(
        &g,
        &ph2d_field_render::occlusion(&doc, &reg, &cam, &g, 48),
    );
    let (ce, cm) = ph2d_field_render::banda::terracos(&g, &|i| ceu[i]);
    println!("  o CÉU, para comparar: terraços {ce:.4} / {cm:.4}");
    println!("  passagens ·  terraços p99 / máx  ·  desvio da convergida (em % da média)");
    let base = cru(48);
    for passagens in 0..=4u32 {
        let mut c = base.clone();
        for _ in 0..passagens {
            c = ph2d_field_render::blur_bounce(&g, &c);
        }
        let (p99, mx) =
            ph2d_field_render::banda::terracos(&g, &|i| (c[i][0] + c[i][1] + c[i][2]) / 3.0);
        println!(
            "  {passagens:>9} · {p99:>9.4} / {mx:>8.4} · {:>6.2} %",
            100.0 * desvio(&c) / escala
        );
    }

    // ⭐⭐⭐ **AS VARIANTES DE DUAS PASSAGENS** — o preço no dispositivo é o MESMO (um despacho
    // extra), logo qual delas é pergunta de medição e não de gosto. O `salto` é o da segunda
    // passagem: `1` é a caixa `3×3` de novo, `2` é à-trous (a mesma recolha de 9 tomas, com os
    // vizinhos afastados), que alcança raio `3` com 9 tomas em vez de 25.
    println!("  duas passagens (salto da 2.ª) ·  terraços p99 / máx  ·  desvio");
    for salto in [1i32, 2, 3] {
        let c = recolhe_salto(&g, &ph2d_field_render::blur_bounce(&g, &base), salto);
        let (p99, mx) =
            ph2d_field_render::banda::terracos(&g, &|i| (c[i][0] + c[i][1] + c[i][2]) / 3.0);
        println!(
            "  {salto:>29} · {p99:>9.4} / {mx:>8.4} · {:>6.2} %",
            100.0 * desvio(&c) / escala
        );
    }

    // ⭐⭐⭐ **AS VARIANTES DE UMA PASSAGEM SÓ** — e a razão de elas serem medidas é o DISPOSITIVO.
    //
    // ⚠️⚠️ Duas passagens de `3×3` não são exprimíveis no pintor sem um buffer intermédio: ele
    // calcula o ricochete num despacho e **lê-o a 3×3 no despacho que pinta**, logo uma segunda
    // passagem no mesmo sítio leria vizinhos já reescritos — uma corrida. ⇒ o que se pode adoptar
    // sem partir a paridade de `100,000 %` é um núcleo MAIOR numa recolha só, e qual deles é
    // pergunta de medição.
    println!("  núcleo (1 passagem) ·  terraços p99 / máx  ·  desvio");
    for (nome, raio, tenda) in [
        ("3×3 caixa (hoje)", 1i32, false),
        ("5×5 caixa", 2, false),
        ("5×5 tenda", 2, true),
        ("7×7 caixa", 3, false),
    ] {
        let c = recolhe(&g, &base, raio, tenda);
        let (p99, mx) =
            ph2d_field_render::banda::terracos(&g, &|i| (c[i][0] + c[i][1] + c[i][2]) / 3.0);
        println!(
            "  {nome:>19} · {p99:>9.4} / {mx:>8.4} · {:>6.2} %",
            100.0 * desvio(&c) / escala
        );
    }
}

/// Uma recolha de raio `raio`, guardada pela normal do CENTRO — o molde que o pintor consegue
/// correr numa passagem só. `tenda` põe o peso `(raio+1−|dx|)·(raio+1−|dy|)`.
fn recolhe(
    g: &ph2d_field_render::Gbuffer,
    canal: &[[f32; 3]],
    raio: i32,
    tenda: bool,
) -> Vec<[f32; 3]> {
    let (w, h) = (g.width as usize, g.height as usize);
    let mut out = canal.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !g.hit[i] {
                continue;
            }
            let n0 = g.normal[i];
            let (mut soma, mut peso) = ([0.0f32; 3], 0.0f32);
            for dy in -raio..=raio {
                for dx in -raio..=raio {
                    let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                    if xx < 0 || yy < 0 || xx >= w as i32 || yy >= h as i32 {
                        continue;
                    }
                    #[allow(clippy::cast_sign_loss)]
                    let j = yy as usize * w + xx as usize;
                    if !g.hit[j] {
                        continue;
                    }
                    let n = g.normal[j];
                    if n0[0] * n[0] + n0[1] * n[1] + n0[2] * n[2]
                        < ph2d_field_render::OCCLUSION_BLUR_COS
                    {
                        continue;
                    }
                    #[allow(clippy::cast_precision_loss)]
                    let p = if tenda {
                        ((raio + 1 - dx.abs()) * (raio + 1 - dy.abs())) as f32
                    } else {
                        1.0
                    };
                    for k in 0..3 {
                        soma[k] += p * canal[j][k];
                    }
                    peso += p;
                }
            }
            if peso > 0.0 {
                for k in 0..3 {
                    out[i][k] = soma[k] / peso;
                }
            }
        }
    }
    out
}

/// ⏱️⭐⭐⭐ **A DECOMPOSIÇÃO do pior terraço NA PEÇA DO DONO** — a irmã da
/// `ph2d_field_render::tests::banda::sonda_de_onde_vem_o_degrau`, sobre o vaso.
///
/// Ela responde a pergunta que decide a wave seguinte: *o que sobra depois da lei fosca é UM pico
/// que se pode nomear, ou é a soma de muitas direcções?* Um pico tem cura própria; uma soma larga
/// só se cura com mais amostras ou com mais filtro.
#[test]
#[ignore = "sonda de diagnóstico: abre a soma do pior terraço do vaso"]
fn sonda_de_onde_vem_o_degrau_no_vaso() {
    let (doc, cam) = vaso();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (w, h) = (256u32, 256u32);
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let mats = vec![ph2d_material::OpenPbr::default().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    let dirs = 48u32;
    let ric =
        ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], dirs);
    let lum = |i: usize| (ric[i][0] + ric[i][1] + ric[i][2]) / 3.0;
    let (w, h) = (w as usize, h as usize);
    let mesma = |a: usize, b: usize| {
        let (u, v) = (g.normal[a], g.normal[b]);
        u[0] * v[0] + u[1] * v[1] + u[2] * v[2] >= ph2d_field_render::OCCLUSION_BLUR_COS
    };
    let (mut pior, mut onde_t) = (0.0f32, None);
    for y in 0..h {
        for x in 1..w - 1 {
            let (a, b, c) = (y * w + x - 1, y * w + x, y * w + x + 1);
            if !g.hit[a] || !g.hit[b] || !g.hit[c] || !mesma(a, b) || !mesma(b, c) {
                continue;
            }
            let q = (lum(a) - 2.0 * lum(b) + lum(c)).abs();
            if q > pior {
                pior = q;
                onde_t = Some((a, b, c));
            }
        }
    }
    let Some((a, b, c)) = onde_t else { return };
    println!(
        "  pior quebra: {pior:.6} (lum {:.6} / {:.6} / {:.6})",
        lum(a),
        lum(b),
        lum(c)
    );
    println!("  dir ·   peso  ·    contribuição A/B/C     ·  salto");
    let mut linhas: Vec<(f32, String)> = Vec::new();
    for k in 0..dirs {
        let f = ph2d_field_render::bounce::bounce_slice(
            &doc,
            &reg,
            &cam,
            &g,
            &surfaces,
            &[lampada],
            k,
            1,
            dirs,
        );
        let mut col = [(0.0f32, 0.0f32); 3];
        for (m, &i) in [a, b, c].iter().enumerate() {
            let s = f.sum[i];
            col[m] = (f.weight[i], (s[0] + s[1] + s[2]) / 3.0);
        }
        let salto = (col[0].1 - 2.0 * col[1].1 + col[2].1).abs();
        linhas.push((
            salto,
            format!(
                "  {k:>3} · {:>6.4} · {:>9.6} {:>9.6} {:>9.6} · {salto:.6}",
                col[1].0, col[0].1, col[1].1, col[2].1
            ),
        ));
    }
    linhas.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap_or(std::cmp::Ordering::Equal));
    for (_, l) in linhas.iter().take(6) {
        println!("{l}");
    }
    let soma: f32 = linhas.iter().map(|l| l.0).sum();
    let maior = linhas.first().map_or(0.0, |l| l.0);
    println!(
        "  soma dos saltos das {dirs}: {soma:.6} · a MAIOR é {maior:.6} ⇒ {:.0} % da soma",
        100.0 * maior / soma.max(1e-9)
    );
}

/// Uma recolha `3×3` com os vizinhos afastados de `salto` pixels — à-trous, guardada pela normal do
/// centro. `salto = 1` é a caixa de sempre.
fn recolhe_salto(g: &ph2d_field_render::Gbuffer, canal: &[[f32; 3]], salto: i32) -> Vec<[f32; 3]> {
    let (w, h) = (g.width as usize, g.height as usize);
    let mut out = canal.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !g.hit[i] {
                continue;
            }
            let n0 = g.normal[i];
            let (mut soma, mut cont) = ([0.0f32; 3], 0u32);
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let (xx, yy) = (x as i32 + dx * salto, y as i32 + dy * salto);
                    if xx < 0 || yy < 0 || xx >= w as i32 || yy >= h as i32 {
                        continue;
                    }
                    #[allow(clippy::cast_sign_loss)]
                    let j = yy as usize * w + xx as usize;
                    if !g.hit[j] {
                        continue;
                    }
                    let n = g.normal[j];
                    if n0[0] * n[0] + n0[1] * n[1] + n0[2] * n[2]
                        < ph2d_field_render::OCCLUSION_BLUR_COS
                    {
                        continue;
                    }
                    for k in 0..3 {
                        soma[k] += canal[j][k];
                    }
                    cont += 1;
                }
            }
            if cont > 0 {
                for k in 0..3 {
                    #[allow(clippy::cast_precision_loss)]
                    let d = cont as f32;
                    out[i][k] = soma[k] / d;
                }
            }
        }
    }
    out
}
