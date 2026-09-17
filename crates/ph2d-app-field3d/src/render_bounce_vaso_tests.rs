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

/// ⭐⭐⭐ **A FOTO DO DONO DE 2026-09-17 (2.ª ronda): o vaso VERMELHO com um cubo BRANCO encostado.**
///
/// *«Absolutamente nenhuma qualidade. Completamente imprestável. Parece mais um reflexo mal
/// feito»* — e a seta dele aponta à FACE do cubo virada ao vaso, onde a luz devolvida sai como uma
/// imagem esborratada e com arestas.
///
/// ⚠️ Esta fixtura existe porque a anterior (o vaso sozinho) não continha o receptor que ele
/// fotografou: uma **face PLANA** ao lado de um emissor colorido, que é onde uma soma de projecções
/// se lê como imagem.
fn vaso_e_cubo() -> (
    ph2d_field::FieldDoc,
    Vec<ph2d_field::FieldDoc>,
    Vec<ph2d_material::Surface>,
    Orbit,
) {
    use ph2d_field::{Node, NodeId, NodeKind, Op, Primitive, Xform};
    let vaso = crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION);
    let vaso_no = vaso.nodes()[0].clone();
    // ⚠️ O cubo vai para o lado de onde a câmera VÊ a face dele virada ao vaso — derivado da base
    // da câmera, e não escrito: a 1.ª redacção pô-lo a `+x` e a face ficou de costas para a lente
    // (`0` pixels), porque a câmera de omissão olha de `+x`.
    let mut cam = Orbit::default();
    let (_, _, olho) = cam.basis();
    let lado = if olho[0] >= 0.0 { -1.0f32 } else { 1.0 };
    let frente = if olho[2] >= 0.0 { 1.0f32 } else { -1.0 };
    let cubo = ph2d_field_eval::leaf(
        Primitive::Box {
            half: [0.35, 0.35, 0.35],
            round: 0.03,
            chamfer: 0.0,
        },
        Xform::at(0.74 * lado, -0.10, 0.10 * frente),
    );
    let folhas = [vaso_no, cubo];
    let mut nos: Vec<Node> = folhas.to_vec();
    nos.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(ph2d_field::Blend::Sharp),
            children: vec![NodeId(0), NodeId(1)],
        },
    ));
    let doc = ph2d_field::FieldDoc::new(nos, NodeId(2)).expect("vaso e cubo");
    let postas = folhas
        .iter()
        .map(|n| ph2d_field::FieldDoc::new(vec![n.clone()], NodeId(0)).expect("a folha posta"))
        .collect();
    let mats = vec![
        ph2d_material::OpenPbr {
            base_color: [0.80, 0.08, 0.06],
            ..ph2d_material::OpenPbr::default()
        }
        .prepare(),
        ph2d_material::OpenPbr {
            base_color: [0.90, 0.90, 0.90],
            ..ph2d_material::OpenPbr::default()
        }
        .prepare(),
    ];
    cam.target = [0.35 * lado, 0.0, 0.0];
    cam.half_extent = 1.0;
    (doc, postas, mats, cam)
}

/// ⏱️⭐⭐⭐ **A RÉGUA NA FACE DO CUBO — e a soma aberta direcção a direcção numa LINHA dela.**
#[test]
#[ignore = "sonda de diagnóstico: a foto do cubo, direcção a direcção"]
fn sonda_o_reflexo_mal_feito_na_face_do_cubo() {
    let (doc, postas, mats, cam) = vaso_e_cubo();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (w, h) = (256u32, 256u32);
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let donos = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(cam.half_extent, 256.0),
    );
    let surfaces = Surfaces {
        all: &mats,
        owners: Some(&donos),
    };
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    // A FACE: os pixels do cubo (dono 1) cuja normal aponta para o vaso, em mundo.
    // A normal do g-buffer é de VISTA; a base da câmera (a mesma que o pintor usa) devolve-a ao mundo.
    let (right, up, fwd) = cam.basis();
    let para_o_vaso = if fwd[0] >= 0.0 { 1.0f32 } else { -1.0 };
    let na_face: Vec<bool> = (0..g.hit.len())
        .map(|i| {
            g.hit[i] && donos.at(g.point[i]) == Some(1) && {
                let v = g.normal[i];
                let nx = right[0] * v[0] + up[0] * v[1] + fwd[0] * v[2];
                nx * para_o_vaso > 0.9
            }
        })
        .collect();
    let face_px = na_face.iter().filter(|b| **b).count();
    println!("  a face do cubo virada ao vaso: {face_px} px");
    assert!(face_px > 500, "a face não está na tela");

    let cru = |dirs: u32| {
        ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], dirs)
    };
    // Um g-buffer só com a face, para a régua dos terraços medir só ela.
    // O `Gbuffer` não é `Clone`; traçar outra vez a `256²` custa nada e dá a régua um g-buffer
    // com a MÁSCARA da face.
    let mut gf = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    gf.hit.copy_from_slice(&na_face);
    let convergida = cru(1024);
    let lum = |c: &[[f32; 3]], i: usize| (c[i][0] + c[i][1] + c[i][2]) / 3.0;
    let media_face = {
        let (mut s, mut n) = (0.0f64, 0usize);
        for (i, &face) in na_face.iter().enumerate() {
            if face {
                s += f64::from(lum(&convergida, i));
                n += 1;
            }
        }
        (s / n.max(1) as f64) as f32
    };
    println!("  ricochete médio na face (convergida, 1024 dir): {media_face:.5}");
    println!("  direcções ·  terraços p99 / máx NA FACE  ·  erro médio vs convergida (% da média)");
    for dirs in [48u32, 96, 256] {
        let c = ph2d_field_render::blur_bounce(&g, &cru(dirs));
        let (p99, mx) = ph2d_field_render::banda::terracos(&gf, &|i| lum(&c, i));
        let (mut e, mut n) = (0.0f64, 0usize);
        for (i, &face) in na_face.iter().enumerate() {
            if face {
                e += f64::from((lum(&c, i) - lum(&convergida, i)).abs());
                n += 1;
            }
        }
        let erro = 100.0 * (e / n.max(1) as f64) as f32 / media_face.max(1e-9);
        println!("  {dirs:>9} · {p99:>9.4} / {mx:>8.4} · {erro:>6.2} %");
    }
    let ceu = ph2d_field_render::blur_occlusion(
        &g,
        &ph2d_field_render::occlusion(&doc, &reg, &cam, &g, 48),
    );
    let (cp, cm) = ph2d_field_render::banda::terracos(&gf, &|i| ceu[i]);
    println!("  o CÉU na mesma face, para comparar: {cp:.4} / {cm:.4}");

    // ── A SOMA ABERTA numa linha da face: cada direcção como uma FITA de '#' (contribui) e '.' ──
    let linhas_da_face: Vec<usize> = (0..h as usize)
        .filter(|y| {
            (0..w as usize)
                .filter(|x| na_face[y * w as usize + x])
                .count()
                > 40
        })
        .collect();
    let y = linhas_da_face[linhas_da_face.len() / 2];
    let xs: Vec<usize> = (0..w as usize)
        .filter(|x| na_face[y * w as usize + x])
        .collect();
    let (x0, x1) = (xs[0], *xs.last().unwrap());
    println!(
        "  linha y={y}, x∈[{x0},{x1}] ({} px) — uma fita por direcção que contribui:",
        xs.len()
    );
    let dirs = 48u32;
    let mut fitas = 0usize;
    let mut arestas = 0usize;
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
        let pesa = xs.iter().any(|&x| f.weight[y * w as usize + x] > 0.0);
        if !pesa {
            continue;
        }
        fitas += 1;
        let mut fita = String::new();
        let mut antes = false;
        for (m, &x) in xs.iter().enumerate() {
            let s = f.sum[y * w as usize + x];
            let acende = (s[0] + s[1] + s[2]) > 1e-6;
            if m > 0 && acende != antes {
                arestas += 1;
            }
            antes = acende;
            if m % 2 == 0 {
                fita.push(if acende { '#' } else { '.' });
            }
        }
        println!("  {k:>3} {fita}");
    }
    println!(
        "  {fitas} direcções contribuem nesta linha e trocam de resposta {arestas} vezes ao longo dela"
    );
}

/// ⏱️⭐⭐⭐ **AS SONDAS contra a recolha POR PIXEL, na face do cubo e no vaso — as duas contra a
/// mesma convergida.** A pergunta que decide a wave: *a interpolação entre sondas apaga a estrutura
/// que o dono fotografou, e a que preço de exactidão?*
#[test]
#[ignore = "sonda de decisão: sondas contra por-pixel"]
fn sonda_as_sondas_contra_o_por_pixel() {
    let (doc, postas, mats, cam) = vaso_e_cubo();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (w, h) = (256u32, 256u32);
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let donos = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(cam.half_extent, 256.0),
    );
    let surfaces = Surfaces {
        all: &mats,
        owners: Some(&donos),
    };
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    let (right, up, fwd) = cam.basis();
    let para_o_vaso = if fwd[0] >= 0.0 { 1.0f32 } else { -1.0 };
    let na_face: Vec<bool> = (0..g.hit.len())
        .map(|i| {
            g.hit[i] && donos.at(g.point[i]) == Some(1) && {
                let v = g.normal[i];
                let nx = right[0] * v[0] + up[0] * v[1] + fwd[0] * v[2];
                nx * para_o_vaso > 0.9
            }
        })
        .collect();
    let no_vaso: Vec<bool> = (0..g.hit.len())
        .map(|i| g.hit[i] && donos.at(g.point[i]) == Some(0))
        .collect();
    let lum = |c: &[[f32; 3]], i: usize| (c[i][0] + c[i][1] + c[i][2]) / 3.0;
    let convergida =
        ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], 1024);
    let mascara = |m: &[bool]| {
        let mut gm = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
        gm.hit.copy_from_slice(m);
        gm
    };
    let (gf, gv) = (mascara(&na_face), mascara(&no_vaso));
    let regua = |nome: &str, c: &[[f32; 3]]| {
        for (rot, m, gm) in [("face", &na_face, &gf), ("vaso", &no_vaso, &gv)] {
            let (p99, mx) = ph2d_field_render::banda::terracos(gm, &|i| lum(c, i));
            let (mut e, mut s) = (0.0f64, 0.0f64);
            for (i, &dentro) in m.iter().enumerate() {
                if dentro {
                    e += f64::from((lum(c, i) - lum(&convergida, i)).abs());
                    s += f64::from(lum(&convergida, i));
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            let erro = (100.0 * e / s.max(1e-9)) as f32;
            println!("  {nome:>28} · {rot} · terraços {p99:>7.4} / {mx:>7.4} · erro {erro:>6.2} %");
        }
    };
    let ceu = ph2d_field_render::blur_occlusion(
        &g,
        &ph2d_field_render::occlusion(&doc, &reg, &cam, &g, 48),
    );
    for (rot, gm) in [("face", &gf), ("vaso", &gv)] {
        let (p99, mx) = ph2d_field_render::banda::terracos(gm, &|i| ceu[i]);
        println!(
            "  {:>28} · {rot} · terraços {p99:>7.4} / {mx:>7.4}",
            "o CÉU (48 cones)"
        );
    }
    let atual = ph2d_field_render::blur_bounce(
        &g,
        &ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], 48),
    );
    regua("por pixel, 48 dir + 2 borrões", &atual);
    // ⚠️ A dureza da visibilidade ficou BINÁRIA por medição (a suave leu `0,67`/`0,76` na face
    // contra `0,32`), e o borrão de duas passagens é o que apaga os vincos da interpolação —
    // ver o cabeçalho das `probes`. O que se mede aqui é o TAMANHO da grelha e o que os nove
    // coeficientes custam contra a soma directa.
    for (n, dirs) in [(16usize, 256u32), (24, 256), (32, 256)] {
        let t0 = std::time::Instant::now();
        let grid = ph2d_field_render::probes::bake_probes(
            &doc,
            &reg,
            &cam,
            &surfaces,
            &[lampada],
            n,
            dirs,
            256,
        );
        let assar = t0.elapsed();
        let dentro = grid.inside.iter().filter(|b| **b).count();
        println!(
            "  sondas {n}³ × {dirs} dir · {dentro} dentro · assar {:.0} ms",
            assar.as_secs_f64() * 1e3
        );
        for directa in [true, false] {
            let t1 = std::time::Instant::now();
            let sondas =
                ph2d_field_render::probes::gather_probes_por(&doc, &reg, &cam, &g, &grid, directa);
            let recolher = t1.elapsed().as_secs_f64() * 1e3;
            let como = if directa {
                "soma directa"
            } else {
                "9 coeficientes"
            };
            regua(
                &format!("{n}³ {como} ({recolher:.0} ms) + 2 borrões"),
                &ph2d_field_render::blur_bounce(&g, &sondas),
            );
        }
    }
    // ── a ESTRUTURA do resíduo (janela 4 px): a régua que separa as duas leis ─────────────────
    println!("  estrutura de média frequência do resíduo (RMS / média, janela 4 px), face · vaso:");
    let estr = |c: &[[f32; 3]]| -> (f32, f32) {
        (
            ph2d_field_render::banda::estrutura(&gf, &|i| lum(c, i), &|i| lum(&convergida, i), 4),
            ph2d_field_render::banda::estrutura(&gv, &|i| lum(c, i), &|i| lum(&convergida, i), 4),
        )
    };
    for dirs in [48u32, 256] {
        let c = ph2d_field_render::blur_bounce(
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
        let (ef, ev) = estr(&c);
        println!(
            "  {:>22} · {ef:.4} · {ev:.4}",
            format!("por pixel {dirs} dir")
        );
    }
    for n in [16usize, 24, 32] {
        let grid = ph2d_field_render::probes::bake_probes(
            &doc,
            &reg,
            &cam,
            &surfaces,
            &[lampada],
            n,
            256,
            256,
        );
        let c = ph2d_field_render::blur_bounce(
            &g,
            &ph2d_field_render::probes::gather_probes(&doc, &reg, &cam, &g, &grid),
        );
        let (ef, ev) = estr(&c);
        println!("  {:>22} · {ef:.4} · {ev:.4}", format!("sondas {n}³"));
    }

    // ── o PERFIL numa linha da face: o que o olho vê ─────────────────────────────────────────
    let linhas: Vec<usize> = (0..h as usize)
        .filter(|y| {
            (0..w as usize)
                .filter(|x| na_face[y * w as usize + x])
                .count()
                > 40
        })
        .collect();
    let y = linhas[linhas.len() / 2];
    let xs: Vec<usize> = (0..w as usize)
        .filter(|x| na_face[y * w as usize + x])
        .collect();
    let grid = ph2d_field_render::probes::bake_probes(
        &doc,
        &reg,
        &cam,
        &surfaces,
        &[lampada],
        24,
        256,
        256,
    );
    let sondas = ph2d_field_render::blur_bounce(
        &g,
        &ph2d_field_render::probes::gather_probes(&doc, &reg, &cam, &g, &grid),
    );
    let escala = xs
        .iter()
        .map(|&x| lum(&convergida, y * w as usize + x))
        .fold(0.0f32, f32::max)
        .max(1e-9);
    let perfil = |nome: &str, c: &[[f32; 3]]| {
        let s: String = xs
            .iter()
            .step_by(2)
            .map(|&x| {
                let v = (lum(c, y * w as usize + x) / escala * 9.0)
                    .round()
                    .clamp(0.0, 9.0);
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let d = v as u8;
                char::from(b'0' + d)
            })
            .collect();
        println!("  {nome:>12} {s}");
    };
    println!(
        "  o perfil da luz devolvida ao longo da linha y={y} da face (0..9 = fracção do máximo):"
    );
    perfil("convergida", &convergida);
    perfil("por pixel", &atual);
    perfil("sondas 24³", &sondas);
}

/// ⭐⭐⭐ **AS SONDAS NÃO DESENHAM A PEÇA NA FACE DO CUBO** — o gate do report (*«um reflexo mal
/// feito»*), pela régua que apanha o mecanismo ([`ph2d_field_render::banda::estrutura`]).
///
/// # ⭐ De onde a barra sai (`sonda_as_sondas_contra_o_por_pixel`, `256²`, janela `4 px`)
///
/// | lei | estrutura na face |
/// |---|---:|
/// | por pixel, `48` dir (a lei do report) | `0,106` |
/// | por pixel, `256` dir (`5,3×` o preço) | `0,045` |
/// | **sondas `32³`** | **`0,035`** |
///
/// ⇒ a barra é **`0,06`**: no vale entre o que as sondas leem e o que a lei do report lia, e
/// abaixo até do por-pixel cinco vezes mais caro. ⚠️ **O CONTROLO é metade do gate:** a lei do
/// report tem de ler ACIMA da barra nesta fixtura — senão a fixtura não contém o fenómeno e a
/// asserção sobre as sondas não afirma nada (a armadilha da paridade sobre três formas convexas).
#[test]
fn as_sondas_nao_desenham_a_peca_na_face_do_cubo() {
    let (doc, postas, mats, cam) = vaso_e_cubo();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (w, h) = (128u32, 128u32);
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let donos = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(cam.half_extent, 128.0),
    );
    let surfaces = Surfaces {
        all: &mats,
        owners: Some(&donos),
    };
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    let (right, up, fwd) = cam.basis();
    let para_o_vaso = if fwd[0] >= 0.0 { 1.0f32 } else { -1.0 };
    let mut gf = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    for i in 0..gf.hit.len() {
        gf.hit[i] = g.hit[i] && donos.at(g.point[i]) == Some(1) && {
            let v = g.normal[i];
            (right[0] * v[0] + up[0] * v[1] + fwd[0] * v[2]) * para_o_vaso > 0.9
        };
    }
    let face = gf.hit.iter().filter(|b| **b).count();
    assert!(face > 200, "a face do cubo não está na tela: {face} px");
    let lum = |c: &[[f32; 3]], i: usize| (c[i][0] + c[i][1] + c[i][2]) / 3.0;
    let verdade =
        ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], 512);
    let estrutura = |c: &[[f32; 3]]| {
        ph2d_field_render::banda::estrutura(&gf, &|i| lum(c, i), &|i| lum(&verdade, i), 4)
    };
    const BARRA: f32 = 0.06;
    let report = ph2d_field_render::blur_bounce(
        &g,
        &ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], 48),
    );
    let sondas = ph2d_field_render::blur_bounce(
        &g,
        &ph2d_field_render::probes::probe_bounce(&doc, &reg, &cam, &g, &surfaces, &[lampada]),
    );
    let (er, es) = (estrutura(&report), estrutura(&sondas));
    println!("  estrutura na face · lei do report {er:.4} · sondas {es:.4} · barra {BARRA}");
    assert!(
        er > BARRA,
        "o CONTROLO caiu: a lei do report lê {er:.4} nesta fixtura, abaixo da barra — a face já não \
         contém o fenómeno e o gate não pode afirmar nada sobre as sondas"
    );
    assert!(
        es <= BARRA,
        "as sondas desenham a peça na face do cubo: estrutura {es:.4} contra a barra {BARRA} (a lei \
         do report lia {er:.4})"
    );
}
