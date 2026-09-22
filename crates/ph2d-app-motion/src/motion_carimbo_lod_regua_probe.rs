//! ⭐⭐⭐ **A RÉGUA DO LOD — o que o olho pode ver** (report do Enio, 2026-09-21). Irmã do
//! [`super`] por responsabilidade: ali mede-se o CUSTO e onde o LOD arma; aqui mede-se a
//! FIDELIDADE que autoriza a troca, que é a metade que decide a BARRA.
//!
//! A tabela que estas sondas produziram está no cabeçalho do irmão, com as duas conclusões:
//! largar o arredondamento **nunca** fica invisível (`7` a `255` níveis de 255), e a tile fica —
//! a partir de `4 px` de lado.
//!
//! ⚠️⚠️ **E a régua precisou do CONTROLO DELA PRÓPRIA**: a `1 px` ela lia `23,9` níveis com `8×8`
//! amostras por pixel e lê `1,5` com `32×32`. *A régua não pode ter a ordem de grandeza do que
//! ela mede* — e eu já tinha escrito uma explicação para o `23,9` que o controlo desmentiu.

use super::super::{carga, fatia, segmentos};
use super::forma_da_cena;

/// Amostra densa de um contorno cozido — o polígono que aproxima a silhueta ao nível a que a
/// comparação deixa de depender da amostragem (o controlo está na sonda: dobrar `POR_SEGMENTO`
/// não move o desvio).
const POR_SEGMENTO: usize = 64;

/// A silhueta de um `VecPath` como uma polilinha densa em coordenadas LOCAIS.
///
/// ⚠️ **Amostro a cúbica à mão em vez de pedir um achatamento à biblioteca**, porque o que se mede
/// aqui é a distância entre duas silhuetas e um achatamento com tolerância é, ele próprio, um
/// desvio — *a régua não pode ter a ordem de grandeza do que ela mede*.
fn silhueta(p: &ph2d_vec_scene::VecPath) -> Vec<[f64; 2]> {
    let c = p.cooked();
    let mut pontos = Vec::new();
    for i in 0..c.contour_count() {
        let Some((verts, fechado)) = c.contour(i) else {
            continue;
        };
        if verts.len() < 2 {
            continue;
        }
        let n = verts.len();
        let pares = if fechado { n } else { n - 1 };
        for k in 0..pares {
            let a = &verts[k];
            let b = &verts[(k + 1) % n];
            let (p0, p1, p2, p3) = (a.anchor, a.out_handle, b.in_handle, b.anchor);
            for s in 0..POR_SEGMENTO {
                #[expect(clippy::cast_precision_loss, reason = "índice de amostra, < 64")]
                let t = s as f64 / POR_SEGMENTO as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                pontos.push([
                    w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
                    w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
                ]);
            }
        }
    }
    pontos
}

/// Distância de um ponto ao SEGMENTO `[a, b]`.
fn ao_segmento(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (vx, vy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = vx * vx + vy * vy;
    let t = if l2 <= 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * vx + (p[1] - a[1]) * vy) / l2).clamp(0.0, 1.0)
    };
    let (dx, dy) = (p[0] - (a[0] + t * vx), p[1] - (a[1] + t * vy));
    (dx * dx + dy * dy).sqrt()
}

/// O desvio máximo da polilinha `de` à polilinha `para` (Hausdorff unilateral).
fn desvio(de: &[[f64; 2]], para: &[[f64; 2]]) -> f64 {
    de.iter()
        .map(|&p| {
            (0..para.len())
                .map(|i| ao_segmento(p, para[i], para[(i + 1) % para.len()]))
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0, f64::max)
}

/// A maior extensão da silhueta — o denominador que torna o desvio ADIMENSIONAL (e portanto
/// comparável entre tamanhos, que é a lei que esta casa já paga noutras réguas).
fn diametro(s: &[[f64; 2]]) -> f64 {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in s {
        x0 = x0.min(p[0]);
        y0 = y0.min(p[1]);
        x1 = x1.max(p[0]);
        y1 = y1.max(p[1]);
    }
    (x1 - x0).max(y1 - y0)
}

/// ⭐⭐⭐ **PERGUNTA 3 — A BARRA: a que tamanho no ecrã o arredondamento deixa de ser VISÍVEL?**
/// É daqui que sai o limiar do LOD, e não de um palpite (`CLAUDE.md` §0.0: *meça, e depois escreva
/// o número que a medição deu, com a tabela ao lado*).
///
/// A régua é **geométrica e adimensional**: o desvio de Hausdorff entre a silhueta arredondada e a
/// afiada, dividido pelo diâmetro da forma. Multiplicado pelos píxeis que a forma ocupa no ecrã, dá
/// o desvio **em píxeis** — e abaixo de meio píxel ele vive inteiro dentro do anti-serrilhado, que
/// é o mesmo que dizer que nenhum ecrã o pode mostrar.
///
/// ⚠️ **O CONTROLO é o `corner = 0`**: ali as duas silhuetas são a mesma e o desvio tem de ler
/// `0,000` — sem ele, uma régua partida daria «invisível» a tudo e o limiar sairia de nada.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_when_the_rounding_stops_showing() {
    let _fatia = fatia();
    let afiada = forma_da_cena(0.0);
    let s_afiada = silhueta(&afiada);
    let d = diametro(&s_afiada);
    assert!(d > 0.0, "controlo: a forma afiada tem extensão");

    eprintln!(
        "\n  ═══ QUANDO O ARREDONDAMENTO DEIXA DE SE VER (load {}) ═══\n",
        carga()
    );
    eprintln!("   corner | segmentos | desvio (frac. do lado) | invisível abaixo de");
    eprintln!("  --------|-----------|------------------------|--------------------");
    for c in [0.0f32, 0.1, 0.25, 0.5, 0.75, 1.0] {
        let p = forma_da_cena(c);
        let s = silhueta(&p);
        let h = desvio(&s_afiada, &s).max(desvio(&s, &s_afiada)) / d;
        let limite = if h > 0.0 { 0.5 / h } else { f64::INFINITY };
        eprintln!(
            "   {c:>6.2} | {:>9} | {h:>22.5} | {}",
            segmentos(&p),
            if limite.is_finite() {
                format!("{limite:>10.1} px de lado")
            } else {
                "       (não há desvio)".to_string()
            }
        );
    }
    let px = crate::motion_state::carimbo_demo::ESTRELA_PX;
    eprintln!(
        "\n  ⇒ a cena `=126` nasce com a estrela a {px:.1} px de lado; ao AFASTAR ela só encolhe.\n"
    );
}

/// Amostras por eixo dentro de um pixel — a cobertura de referência. `8×8 = 64` põe o degrau de
/// quantização em `1/64` da cobertura, uma ordem de grandeza abaixo do que se compara.
const SUPER: usize = 8;

/// **A cobertura de um polígono num pixel**, por amostragem — a rasterização de REFERÊNCIA.
///
/// ⭐ Ela existe porque a régua que decide um LOD é **o que o olho vê**, e o olho vê píxeis, não
/// distâncias. A geométrica (Hausdorff) diz o desvio da SILHUETA; esta diz quanto disso sobrevive
/// à grelha do ecrã — e é a segunda que manda, porque é ela que descreve a imagem.
///
/// ⚠️ Sem GPU **de propósito**: uma régua de LOD que precise de placa não corre no portão, e a
/// placa desta máquina está tomada. `par` conta cruzamentos (regra ímpar-par, que é a do
/// preenchimento de uma estrela).
fn rasteriza(poli: &[[f64; 2]], lado_px: usize, caixa: ([f64; 2], f64), sup: usize) -> Vec<f64> {
    let (min, extensao) = caixa;
    let mut img = vec![0.0f64; lado_px * lado_px];
    #[expect(
        clippy::cast_precision_loss,
        reason = "lado da imagem, dezenas de píxeis"
    )]
    let passo = extensao / lado_px as f64;
    for py in 0..lado_px {
        for px in 0..lado_px {
            let mut dentro = 0usize;
            for sy in 0..sup {
                for sx in 0..sup {
                    #[expect(clippy::cast_precision_loss, reason = "índices pequenos")]
                    let p = [
                        min[0] + (px as f64 + (sx as f64 + 0.5) / sup as f64) * passo,
                        min[1] + (py as f64 + (sy as f64 + 0.5) / sup as f64) * passo,
                    ];
                    let mut cruza = false;
                    for i in 0..poli.len() {
                        let (a, b) = (poli[i], poli[(i + 1) % poli.len()]);
                        if (a[1] > p[1]) != (b[1] > p[1]) {
                            let x = (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0];
                            if p[0] < x {
                                cruza = !cruza;
                            }
                        }
                    }
                    if cruza {
                        dentro += 1;
                    }
                }
            }
            #[expect(clippy::cast_precision_loss, reason = "contagem de amostras")]
            let c = dentro as f64 / (sup * sup) as f64;
            img[py * lado_px + px] = c;
        }
    }
    img
}

/// A caixa comum às duas silhuetas — o MESMO enquadramento para as duas imagens, senão o que se
/// mede é o enquadramento e não a forma.
fn caixa_comum(a: &[[f64; 2]], b: &[[f64; 2]]) -> ([f64; 2], f64) {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in a.iter().chain(b) {
        x0 = x0.min(p[0]);
        y0 = y0.min(p[1]);
        x1 = x1.max(p[0]);
        y1 = y1.max(p[1]);
    }
    let e = (x1 - x0).max(y1 - y0);
    ([x0, y0], e)
}

/// ⭐⭐⭐ **PERGUNTA 5 — A BARRA QUE O OLHO USA: quantos NÍVEIS de 255 separam a forma exacta da
/// simplificada, a cada tamanho?** É esta que manda sobre a geométrica, porque descreve a IMAGEM.
///
/// ⚠️ **O CONTROLO é a primeira coluna** (a forma comparada consigo mesma): ela tem de ler `0`
/// níveis a todo tamanho. Sem ele, uma rasterização partida daria «idêntico» a tudo.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_how_many_levels_the_eye_could_tell() {
    let _fatia = fatia();
    let afiada = silhueta(&forma_da_cena(0.0));
    let arredondada = silhueta(&forma_da_cena(0.5));
    let caixa = caixa_comum(&afiada, &arredondada);

    eprintln!(
        "\n  ═══ QUANTOS NÍVEIS DE 255 SEPARAM AS DUAS FORMAS, POR TAMANHO (load {}) ═══\n",
        carga()
    );
    eprintln!(
        "   lado (px) | CONTROLO (ela mesma) | arredondada × afiada | × +ÁREA (8×8) | RÉGUA FINA (32×32) | o olho vê?"
    );
    eprintln!(
        "  -----------|----------------------|----------------------|---------------|--------------------|------------"
    );
    // ⭐⭐⭐ **A COMPENSAÇÃO DE ÁREA** — a cura que a própria tabela encomenda. A `1 px` a forma
    // inteira cabe num pixel e o que sobra da silhueta é só a ÁREA: se a simplificada não tiver a
    // mesma, ela lê-se como uma mancha de outro BRILHO, a todo tamanho. `k` é o factor linear que
    // iguala as duas áreas (a área vai com o quadrado do lado).
    let area = |p: &[[f64; 2]]| {
        (0..p.len())
            .map(|i| {
                let (a, b) = (p[i], p[(i + 1) % p.len()]);
                a[0] * b[1] - b[0] * a[1]
            })
            .sum::<f64>()
            .abs()
            * 0.5
    };
    let k = (area(&arredondada) / area(&afiada)).sqrt();
    let centro_de = |p: &[[f64; 2]]| {
        let n = f64::from(u32::try_from(p.len()).unwrap_or(1));
        let s = p.iter().fold([0.0, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
        [s[0] / n, s[1] / n]
    };
    let c = centro_de(&afiada);
    let compensada: Vec<[f64; 2]> = afiada
        .iter()
        .map(|p| [c[0] + (p[0] - c[0]) * k, c[1] + (p[1] - c[1]) * k])
        .collect();
    eprintln!(
        "   (o factor de área: {k:.5} — a afiada é {:.2}% maior)\n",
        (1.0 / k - 1.0) * 100.0
    );
    for lado in [24usize, 12, 8, 6, 4, 3, 2, 1] {
        let a = rasteriza(&arredondada, lado, caixa, SUPER);
        let b = rasteriza(&afiada, lado, caixa, SUPER);
        let comp = rasteriza(&compensada, lado, caixa, SUPER);
        let ctl = rasteriza(&arredondada, lado, caixa, SUPER);
        // ⚠️⚠️ **O CONTROLO DA PRÓPRIA RÉGUA** — a MESMA pergunta com `4×` as amostras por eixo
        // (`16×` por pixel). Uma coluna que se mexa entre as duas é RUÍDO DE AMOSTRAGEM, não uma
        // diferença entre as formas: *a régua não pode ter a ordem de grandeza do que ela mede*.
        let a_f = rasteriza(&arredondada, lado, caixa, SUPER * 4);
        let b_f = rasteriza(&afiada, lado, caixa, SUPER * 4);
        let niveis = |x: &[f64], y: &[f64]| {
            x.iter()
                .zip(y)
                .map(|(p, q)| (p - q).abs() * 255.0)
                .fold(0.0f64, f64::max)
        };
        let (n_ctl, n, n_c) = (niveis(&a, &ctl), niveis(&a, &b), niveis(&a, &comp));
        let n_fino = niveis(&a_f, &b_f);
        eprintln!(
            "   {lado:>9} | {n_ctl:>20.1} | {n:>20.1} | {n_c:>13.1} | {n_fino:>13.1} | {}",
            if n_fino < 1.0 { "não" } else { "SIM" }
        );
    }
    eprintln!(
        "\n  ⚠️ `1 nível de 255` é o degrau do canal: abaixo dele as duas imagens são o MESMO\n  \
         byte em cada pixel, e nenhum ecrã as pode distinguir.\n"
    );
}

/// Reamostra uma imagem quadrada `de × de` para `para × para` pela MÉDIA das células que cada
/// pixel novo cobre — o filtro de caixa, que é o que uma placa faz ao minificar uma textura com
/// mipmap. ⚠️ `de` é múltiplo de `para` por construção nas chamadas, logo não há meia célula.
fn reamostra(img: &[f64], de: usize, para: usize) -> Vec<f64> {
    #[expect(clippy::cast_precision_loss, reason = "lados de imagem, ≤ 2048")]
    let (de_f, para_f) = (de as f64, para as f64);
    let passo = de_f / para_f;
    let mut fora = vec![0.0f64; para * para];
    for y in 0..para {
        for x in 0..para {
            // A célula do destino, em coordenadas da origem — com pesos FRACCIONÁRIOS, porque o
            // lado da tile não é múltiplo do lado no ecrã (e fingir que é mediria outro caso).
            #[expect(clippy::cast_precision_loss, reason = "índices de pixel")]
            let (u0, v0) = (x as f64 * passo, y as f64 * passo);
            let (u1, v1) = (u0 + passo, v0 + passo);
            let (mut soma, mut peso) = (0.0f64, 0.0f64);
            #[expect(
                clippy::cast_possible_truncation,
                reason = "coordenadas dentro da imagem"
            )]
            #[expect(clippy::cast_sign_loss, reason = "u0/v0 ≥ 0")]
            for sy in (v0 as usize)..=((v1.ceil() as usize).min(de).saturating_sub(1)) {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "coordenadas dentro da imagem"
                )]
                #[expect(clippy::cast_sign_loss, reason = "u0 ≥ 0")]
                for sx in (u0 as usize)..=((u1.ceil() as usize).min(de).saturating_sub(1)) {
                    #[expect(clippy::cast_precision_loss, reason = "índices de pixel")]
                    let w = (u1.min(sx as f64 + 1.0) - u0.max(sx as f64)).max(0.0)
                        * (v1.min(sy as f64 + 1.0) - v0.max(sy as f64)).max(0.0);
                    soma += img[sy * de + sx] * w;
                    peso += w;
                }
            }
            fora[y * para + x] = if peso > 0.0 { soma / peso } else { 0.0 };
        }
    }
    fora
}

/// ⭐⭐⭐ **PERGUNTA 6 — A ROTA QUE A CASA JÁ TEM: quanto erra uma TILE?** A pergunta 5 refutou
/// *«largar o arredondamento»* (ele nunca fica invisível — a `2 px` ainda são `3` níveis). Sobra a
/// rota que o [`apply_object_lod`](crate::motion_bridge_objects::apply_object_lod) já implementa e
/// que esta cena não alcança: assar a forma UMA vez e instanciar um quad texturado.
///
/// A régua é a mesma e o veredito é outro: uma tile é a forma **rasterizada e reamostrada**, logo
/// o erro dela é o da reamostragem — que encolhe com o tamanho em vez de crescer.
///
/// ⚠️ **O CONTROLO continua a ser a coluna da forma comparada consigo mesma**, e a coluna da
/// silhueta simplificada fica ao lado: é a comparação entre as DUAS curas que decide, não o valor
/// absoluto de nenhuma.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_how_much_a_tile_would_err() {
    let _fatia = fatia();
    let arredondada = silhueta(&forma_da_cena(0.5));
    let afiada = silhueta(&forma_da_cena(0.0));
    let caixa = caixa_comum(&arredondada, &arredondada);
    // ⛔⛔⛔ **O LADO DA TILE SAI DA GEOMETRIA, NÃO DO TAMANHO DE MUNDO — e a 1.ª redacção
    // errou-o por `14×`.** Ela fazia `TAMANHO × 2 × BAKE_DPI = 28 px`, usando o tamanho que a
    // INSTÂNCIA pede. O assador (`ShapeBake::bake_one`) mede `standalone_path_screen_bounds(path,
    // scale(BAKE_DPI))` sobre o que está no STORE — e a geometria de um `source.shape` é
    // **NORMALIZADA** (caixa `~1,55`, medido em `diag_o_que_o_size_faz`): a tile real tem
    // **`~396 px`**. ⚠️ *Medir com uma tile 14× menor dá à rota da tile um erro que ela não tem —
    // o erro para o lado CARO, que é o que faz uma barra ficar conservadora de mais.*
    #[expect(
        clippy::cast_possible_truncation,
        reason = "um lado de tile, centenas de px"
    )]
    #[expect(clippy::cast_sign_loss, reason = "uma extensão é positiva")]
    let tile_px =
        ((caixa_comum(&arredondada, &arredondada).1 * crate::motion_object_bake::BAKE_DPI).round()
            as usize)
            .clamp(4, 2048);
    let assada = rasteriza(&arredondada, tile_px, caixa, 4);
    eprintln!("   (o assador da casa dá a esta forma uma tile de {tile_px} px de lado)\n");

    eprintln!(
        "\n  ═══ O ERRO DE UMA TILE, CONTRA O DE SIMPLIFICAR A SILHUETA (load {}) ═══\n",
        carga()
    );
    eprintln!(
        "   lado (px) | CONTROLO | TILE reamostrada | silhueta simplificada | quem erra menos"
    );
    eprintln!(
        "  -----------|----------|------------------|-----------------------|----------------"
    );
    for lado in [128usize, 64, 32, 28, 16, 8, 4, 2, 1] {
        let exacta = rasteriza(&arredondada, lado, caixa, SUPER * 4);
        let ctl = rasteriza(&arredondada, lado, caixa, SUPER * 4);
        let tile = reamostra(&assada, tile_px, lado);
        let simples = rasteriza(&afiada, lado, caixa, SUPER * 4);
        let niveis = |x: &[f64], y: &[f64]| {
            x.iter()
                .zip(y)
                .map(|(p, q)| (p - q).abs() * 255.0)
                .fold(0.0f64, f64::max)
        };
        let (c, t, s) = (
            niveis(&exacta, &ctl),
            niveis(&exacta, &tile),
            niveis(&exacta, &simples),
        );
        eprintln!(
            "   {lado:>9} | {c:>8.1} | {t:>16.1} | {s:>21.1} | {}",
            if t < s { "a TILE" } else { "a silhueta" }
        );
    }
    eprintln!(
        "\n  ⚠️ A tile é assada UMA vez por geometria e depois é um quad por cópia — o erro dela é\n  \
         o da REAMOSTRAGEM, e ele encolhe com o tamanho. O da silhueta simplificada CRESCE com\n  \
         ele, porque a forma é outra.\n"
    );
}

/// A forma que o catálogo constrói para `kind`, pelo caminho do PRODUTO (o nó `source.shape` mais
/// o `publish`), no tamanho da cena `=126`.
///
/// ⚠️ **Uma cadeia de UMA cópia**, não as `90 000`: o que se mede aqui é a GEOMETRIA, e cozinhar o
/// campo inteiro `45` vezes seria pagar a população para medir a forma.
fn forma_do_catalogo(kind: ph2d_node_motion_shape::ShapeKind) -> Option<ph2d_vec_scene::VecPath> {
    let mut m = crate::motion_state::MotionState::new();
    let f = m.doc.graph.add_node("source.shape");
    let saida = m.doc.graph.add_node("motion.output");
    let i = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == kind)?;
    #[expect(clippy::cast_precision_loss, reason = "um índice de enum, < 64")]
    m.doc
        .graph
        .set_param(f, ph2d_node_motion_shape::param::KIND, i as f32);
    m.doc.graph.set_param(
        f,
        ph2d_node_motion_shape::param::SIZE,
        crate::motion_state::carimbo_demo::TAMANHO,
    );
    m.doc
        .graph
        .set_param(f, ph2d_node_motion_shape::param::CORNER, 0.5);
    m.doc
        .graph
        .connect(ph2d_nodegraph::graph::Edge {
            from: (f, 0),
            to: (saida, 0),
            delayed: false,
        })
        .ok()?;
    crate::motion_shape_gen::publish(&mut m, 0.0);
    m.pump.mark_dirty();
    m.pump.pump(
        &m.doc.graph,
        &m.registry,
        &[saida],
        1,
        0.0,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
    );
    let gid = m.pump.vector_instances.first()?.geometry_id;
    m.shape_store.get(gid).cloned()
}

/// ⭐⭐⭐ **PERGUNTA 9 — A BARRA VALE PARA TODAS AS FORMAS, ou só para a estrela?** (report do
/// Enio, 2026-09-22: *«isso já funciona para todas as shapes?»*).
///
/// A lei do LOD é por `geometry_id` e não conhece forma nenhuma — ela vale para qualquer
/// geometria do store **por construção**. O que NÃO é automático é a **BARRA**: ela saiu da
/// medição de UMA forma, e uma com detalhe mais fino pode errar mais ao ser reamostrada.
///
/// Esta sonda corre a régua do pixel sobre **todo o catálogo** (`ALL_KINDS`), no tamanho da cena
/// `=126`, e imprime o erro da tile ao lado do erro de simplificar. ⚠️ **O CONTROLO é a coluna
/// `4 px`**: é lá que a barra vive, e nenhuma forma pode passar `1` nível — se alguma passar, a
/// barra é dela e não da estrela.
/// ⚠️ **Ela demora `~16 min`** (as `45` formas × a tile que cada uma recebe × o controlo da
/// régua), e imprime linha a linha desde a primeira — *o silêncio de um `| tail` lê-se como
/// pendurada, e não é*.
#[test]
#[ignore = "sonda de medição — ~16 min; corra à mão, em RELEASE e com a máquina calma"]
fn audit_whether_the_bar_holds_for_every_shape() {
    let _fatia = fatia();
    eprintln!(
        "\n  ═══ O ERRO DA TILE EM TODO O CATÁLOGO (load {}) ═══\n",
        carga()
    );
    eprintln!(
        "   forma                |   tile  | erro @4px | @2px | @1px | ruído da régua | pior"
    );
    eprintln!(
        "  ----------------------|---------|-----------|------|------|----------------|------"
    );
    let (mut pior_nome, mut pior_valor) = (String::new(), 0.0f64);
    let (mut medidas, mut sem_caixa) = (0usize, Vec::new());
    for &kind in ph2d_node_motion_shape::ALL_KINDS {
        let Some(p) = forma_do_catalogo(kind) else {
            sem_caixa.push(format!("{kind:?}"));
            continue;
        };
        let s = silhueta(&p);
        if s.len() < 8 {
            sem_caixa.push(format!("{kind:?} (silhueta com {} pontos)", s.len()));
            continue;
        }
        let caixa = caixa_comum(&s, &s);
        // ⛔⛔ **O lado da tile é DERIVADO da caixa de CADA forma, nunca cravado.** A 1.ª
        // redacção usava `28` para todas — o valor da ESTRELA — e uma forma mais larga ou mais
        // estreita recebe outra tile do assador (`bbox × BAKE_DPI`): a tabela media uma tile que
        // aquela forma nunca teria.
        #[expect(
            clippy::cast_possible_truncation,
            reason = "um lado de tile, dezenas de px"
        )]
        #[expect(clippy::cast_sign_loss, reason = "uma extensão é positiva")]
        let tile_px =
            ((caixa.1 * crate::motion_object_bake::BAKE_DPI).round() as usize).clamp(4, 2048);
        let assada = rasteriza(&s, tile_px, caixa, 4);
        // ⚠️ **E o CONTROLO DA PRÓPRIA RÉGUA**: a tile assada com `4×` as amostras. Uma coluna que
        // se mexa entre as duas é ruído da régua, não erro da tile.
        let assada_fina = rasteriza(&s, tile_px, caixa, 16);
        let mut linha = [0.0f64; 3];
        let mut controlo = 0.0f64;
        for (j, lado) in [4usize, 2, 1].into_iter().enumerate() {
            let sup = (1024 / lado).clamp(2, 32);
            let exacta = rasteriza(&s, lado, caixa, sup);
            let niveis = |t: &[f64]| {
                exacta
                    .iter()
                    .zip(t)
                    .map(|(a, b)| (a - b).abs() * 255.0)
                    .fold(0.0f64, f64::max)
            };
            linha[j] = niveis(&reamostra(&assada, tile_px, lado));
            controlo =
                controlo.max((linha[j] - niveis(&reamostra(&assada_fina, tile_px, lado))).abs());
        }
        let pior = linha.iter().fold(0.0f64, |a, &b| a.max(b));
        if pior > pior_valor {
            pior_valor = pior;
            pior_nome = format!("{kind:?}");
        }
        medidas += 1;
        eprintln!(
            "   {:<20} | {tile_px:>4} px | {:>9.2} | {:>4.2} | {:>4.2} | {controlo:>4.2} | {}",
            format!("{kind:?}"),
            linha[0],
            linha[1],
            linha[2],
            if pior >= 1.0 { "⛔" } else { "ok" }
        );
    }
    eprintln!(
        "\n  medidas: {medidas} de {}",
        ph2d_node_motion_shape::ALL_KINDS.len()
    );
    if !sem_caixa.is_empty() {
        eprintln!(
            "  ⚠️ SEM geometria mensurável ({}): {}",
            sem_caixa.len(),
            sem_caixa.join(", ")
        );
    }
    eprintln!("  pior do catálogo: {pior_nome} a {pior_valor:.2} níveis\n");
}
