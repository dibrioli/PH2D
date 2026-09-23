//! ⏱️⭐⭐⭐⭐ **QUANTO CUSTA UMA ARESTA LIDA DE UM BUFFER, CONTRA UMA DOBRADA NO TEXTO.**
//!
//! # O número que esta sonda existe para produzir
//!
//! Duas decisões desta casa dependem dele e nenhuma o tinha:
//!
//! 1. **A consulta do TORNO** (`docs/Render3d/03` §W9): a cena `5` paga `931` linhas de WGSL em
//!    toda amostra, e a poda por célula deixa `79` (p50) a `270` (pior). ⚠️ Mas as linhas de uma
//!    árvore especializada são **constantes dobradas no texto** e as de uma consulta são **leituras
//!    de um buffer** — *se uma leitura custar 3× uma constante, a poda não paga a wave*.
//! 2. **A recusa de INTERPRETAR a fita no dispositivo** ([`ph2d_field_eval::wgsl`]): o cabeçalho
//!    dela diz, por escrito, que a rota *«deixou de ser impossível e passou a ser uma medição por
//!    fazer — e ela não se abre sem um número, que hoje não existe»*.
//!
//! # A régua
//!
//! A MESMA aritmética (a distância² ponto-segmento da casa, com `e` e `e/‖e‖²` pré-calculados) em
//! duas formas, sobre os MESMOS pontos:
//!
//! | forma | as arestas |
//! |---|---|
//! | **dobrada** | literais no texto, `N` blocos desenrolados — a forma da FITA |
//! | **lida** | `array<vec4<f32>>`, laço com o limite em `arrayLength` — a forma da CONSULTA |
//!
//! ⭐ **O que se lê é a INCLINAÇÃO em `N`**, não o tempo absoluto: assim o custo de despachar, de
//! ler de volta e de compilar cancela-se. E a compilação fica fora do relógio pela porta
//! [`ph2d_field_gpu::probe::evaluate_medido`], que existe por causa desta medição.
//!
//! ⚠️ **As duas formas têm de dar o MESMO número** — é a 1.ª asserção, e sem ela a tabela pode
//! estar a cronometrar duas leis diferentes.

/// Uma forma a medir: a fonte, os uniformes e os storages que ela pede.
type Forma<'a> = (String, Vec<&'a [f32]>, Vec<&'a [f32]>);

/// A aritmética partilhada — escrita UMA vez, para as duas formas serem a mesma lei.
const SEG: &str = r"
fn passo(p: vec2<f32>, a: vec2<f32>, e: vec2<f32>, g: vec2<f32>) -> f32 {
  let w = p - a;
  let h = clamp(dot(w, g), 0.0, 1.0);
  let q = w - e * h;
  return dot(q, q);
}
";

/// As `n` arestas de um polígono regular de raio `1` — cada uma com `a`, `e` e `g = e/‖e‖²`.
fn arestas(n: usize) -> Vec<[f32; 4]> {
    let mut v = Vec::with_capacity(n * 2);
    for i in 0..n {
        #[allow(clippy::cast_precision_loss)]
        let t = |k: usize| std::f32::consts::TAU * k as f32 / n as f32;
        let (a, b) = ([t(i).cos(), t(i).sin()], [t(i + 1).cos(), t(i + 1).sin()]);
        let e = [b[0] - a[0], b[1] - a[1]];
        let inv = 1.0 / e[0].mul_add(e[0], e[1] * e[1]);
        v.push([a[0], a[1], e[0], e[1]]);
        v.push([e[0] * inv, e[1] * inv, 0.0, 0.0]);
    }
    v
}

/// A forma **DOBRADA**: `n` blocos com as arestas como literais.
fn fonte_dobrada(es: &[[f32; 4]]) -> String {
    let mut s = String::from(
        "@group(0) @binding(0) var<storage, read> entrada: array<vec4<f32>>;\n\
         @group(0) @binding(1) var<storage, read_write> saida: array<vec4<f32>>;\n",
    );
    s.push_str(SEG);
    s.push_str(
        "@compute @workgroup_size(64)\nfn main(@builtin(global_invocation_id) gid: vec3<u32>) {\n\
         let i = gid.x;\n  if (i >= arrayLength(&entrada)) { return; }\n\
         let p = entrada[i].xy;\n  var d2 = 1.0e30;\n",
    );
    for par in es.as_chunks::<2>().0 {
        let (a, g) = (par[0], par[1]);
        s.push_str(&format!(
            "  d2 = min(d2, passo(p, vec2<f32>({:e}, {:e}), vec2<f32>({:e}, {:e}), \
             vec2<f32>({:e}, {:e})));\n",
            a[0], a[1], a[2], a[3], g[0], g[1]
        ));
    }
    s.push_str("  saida[i] = vec4<f32>(sqrt(d2), 0.0, 0.0, 0.0);\n}\n");
    s
}

/// A forma **LIDA DE `storage`**, com o limite do laço a ser `arrayLength` — o compilador não o
/// pode desenrolar nem dobrar o endereço.
fn fonte_storage_dinamica() -> String {
    cabeca_storage()
        + SEG
        + &laco(
            "let n = arrayLength(&arestas) / 2u;",
            "n",
            "arestas[2u * k]",
            "arestas[2u * k + 1u]",
        )
}

/// A MESMA leitura de `storage` com a CONTAGEM no texto — o laço passa a ser desenrolável.
fn fonte_storage_fixa(n: usize) -> String {
    cabeca_storage()
        + SEG
        + &laco(
            "",
            &format!("{n}u"),
            "arestas[2u * k]",
            "arestas[2u * k + 1u]",
        )
}

/// A contagem no texto **e** as arestas num buffer `uniform` — o banco de constantes da placa.
///
/// ⚠️ **O acesso é UNIFORME dentro do grupo** (toda thread lê o mesmo `k` na mesma iteração), que é
/// o caso para que um banco de constantes existe. *Se o preço de uma leitura for do `storage` e não
/// do facto de não ser um literal, é aqui que ele desaparece.*
fn fonte_uniform_fixa(n: usize) -> String {
    format!(
        "struct Arestas {{ v: array<vec4<f32>, {UNIF}> }};\n\
         @group(0) @binding(0) var<uniform> arestas: Arestas;\n\
         @group(0) @binding(1) var<storage, read> entrada: array<vec4<f32>>;\n\
         @group(0) @binding(2) var<storage, read_write> saida: array<vec4<f32>>;\n"
    ) + SEG
        + &laco(
            "",
            &format!("{n}u"),
            "arestas.v[2u * k]",
            "arestas.v[2u * k + 1u]",
        )
}

/// Quantos `vec4` o array `uniform` declara — `2` por aresta × `COPIAS`, e o buffer tem de ter
/// exactamente este tamanho.
const UNIF: usize = 1024;

/// Quantas cópias iguais da lista o buffer leva — é o que deixa cada thread ler de um sítio
/// DIFERENTE com a MESMA resposta.
const COPIAS: usize = 4;

/// ⭐⭐⭐ **A forma HONESTA de uma consulta por célula: o acesso é DIVERGENTE.**
///
/// ⚠️ **As formas de cima leem todas o mesmo `k` em toda thread do grupo** — é o caso ideal de um
/// banco de constantes, e não é o caso de uma consulta: threads vizinhas caem em células vizinhas e
/// lêem listas **diferentes**. ⇒ aqui cada thread escolhe uma de [`COPIAS`] cópias IGUAIS da lista
/// (a resposta é a mesma, e é isso que mantém a asserção de pé) e o acesso deixa de ser uniforme.
fn fonte_divergente(n: usize, unif: bool) -> String {
    fonte_divergente_com(n, unif, COPIAS)
}

/// A mesma, com **quantas listas distintas um grupo de threads lê** como parâmetro — é essa a
/// grandeza que a curva da divergência varre.
fn fonte_divergente_com(n: usize, unif: bool, copias: usize) -> String {
    let cab = if unif {
        format!(
            "struct Arestas {{ v: array<vec4<f32>, {UNIF}> }};\n\
             @group(0) @binding(0) var<uniform> arestas: Arestas;\n\
             @group(0) @binding(1) var<storage, read> entrada: array<vec4<f32>>;\n\
             @group(0) @binding(2) var<storage, read_write> saida: array<vec4<f32>>;\n"
        )
    } else {
        cabeca_storage()
    };
    let (a, g) = if unif {
        ("arestas.v[base + 2u * k]", "arestas.v[base + 2u * k + 1u]")
    } else {
        ("arestas[base + 2u * k]", "arestas[base + 2u * k + 1u]")
    };
    #[allow(clippy::cast_possible_truncation)]
    let prologo = format!("let base = (i % {}u) * {}u;", copias, n * 2);
    cab + SEG + &laco(&prologo, &format!("{n}u"), a, g)
}

fn cabeca_storage() -> String {
    String::from(
        "@group(0) @binding(0) var<storage, read> arestas: array<vec4<f32>>;\n\
         @group(0) @binding(1) var<storage, read> entrada: array<vec4<f32>>;\n\
         @group(0) @binding(2) var<storage, read_write> saida: array<vec4<f32>>;\n",
    )
}

/// O corpo com laço — uma função só, para as três formas serem a MESMA lei.
fn laco(prologo: &str, limite: &str, a: &str, g: &str) -> String {
    format!(
        "@compute @workgroup_size(64)\nfn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
         let i = gid.x;\n  if (i >= arrayLength(&entrada)) {{ return; }}\n\
         let p = entrada[i].xy;\n  var d2 = 1.0e30;\n  {prologo}\n\
         for (var k = 0u; k < {limite}; k = k + 1u) {{\n\
         let a = {a};\n    let g = {g};\n\
         d2 = min(d2, passo(p, a.xy, a.zw, g.xy));\n  }}\n\
         saida[i] = vec4<f32>(sqrt(d2), 0.0, 0.0, 0.0);\n}}\n"
    )
}

/// Os pontos de amostra — espalhados pela caixa do polígono, como a marcha os traria.
fn pontos(n: usize) -> Vec<[f32; 4]> {
    (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f32;
            let a = (t * 0.618_034).fract().mul_add(2.4, -1.2);
            let b = (t * 0.414_213_5).fract().mul_add(2.4, -1.2);
            [a, b, 0.0, 0.0]
        })
        .collect()
}

/// ⏱️⭐⭐⭐⭐ **A sonda.** Ver o cabeçalho do módulo.
#[test]
#[ignore = "sonda de diagnóstico: precisa de GPU e lê relógio"]
fn diag_o_custo_de_uma_aresta_lida_contra_dobrada() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let guarda = t.lock().expect("o traçador");
    let (device, queue) = guarda.parts();
    const AMOSTRAS: usize = 1 << 20;
    const REPETICOES: usize = 7;
    let ps = pontos(AMOSTRAS);
    println!(
        "\n  {AMOSTRAS} amostras · mínimo de {REPETICOES} despachos · a compilação FORA do relógio"
    );
    println!(
        "     N ·  dobrada · stor.dinâm · stor.fixa · unif.fixa · stor.DIVERG · unif.DIVERG   (ms)"
    );
    let mut linhas: Vec<(usize, [f32; 6])> = Vec::new();
    for n in [8usize, 16, 32, 64, 128] {
        let es = arestas(n);
        let planos: Vec<f32> = es.iter().flat_map(|a| *a).collect();
        // ⚠️ O buffer `uniform` tem o tamanho que o array DECLARA, e não o que a peça usa.
        // ⚠️ O buffer leva `COPIAS` cópias IGUAIS: é o que deixa a forma divergente ler de sítios
        // diferentes com a MESMA resposta.
        let mut muitos: Vec<f32> = Vec::new();
        for _ in 0..COPIAS {
            muitos.extend_from_slice(&planos);
        }
        let mut unif = muitos.clone();
        unif.resize(UNIF * 4, 0.0);
        let formas: [Forma<'_>; 6] = [
            (fonte_dobrada(&es), vec![], vec![]),
            (fonte_storage_dinamica(), vec![], vec![&planos]),
            (fonte_storage_fixa(n), vec![], vec![&planos]),
            (fonte_uniform_fixa(n), vec![&unif], vec![]),
            (fonte_divergente(n, false), vec![], vec![&muitos]),
            (fonte_divergente(n, true), vec![&unif], vec![]),
        ];
        let mut ms = [0.0f32; 6];
        let mut base: Option<Vec<[f32; 4]>> = None;
        for (k, (fonte, us, ss)) in formas.iter().enumerate() {
            let (out, t) = ph2d_field_gpu::probe::evaluate_medido(
                device, queue, fonte, "main", us, ss, &ps, 1, REPETICOES,
            );
            ms[k] = t;
            // ⚠️ **A 1.ª asserção**: sem ela a tabela pode estar a cronometrar leis diferentes.
            match &base {
                None => base = Some(out),
                Some(b) => {
                    let pior = b
                        .iter()
                        .zip(&out)
                        .map(|(a, c)| (a[0] - c[0]).abs())
                        .fold(0.0f32, f32::max);
                    assert!(
                        pior <= 1.0e-6,
                        "a forma {k} discorda da dobrada em {pior:e} — a tabela mediria duas leis"
                    );
                }
            }
        }
        println!(
            "  {n:>4} · {:>8.3} · {:>10.3} · {:>9.3} · {:>9.3} · {:>11.3} · {:>11.3}",
            ms[0], ms[1], ms[2], ms[3], ms[4], ms[5],
        );
        linhas.push((n, ms));
    }
    // ⭐⭐⭐ **A CURVA DA DIVERGÊNCIA** — a tabela de cima usa `COPIAS` listas distintas por grupo,
    // que é um extremo. Aqui a grandeza varre-se: quantas listas DIFERENTES um grupo de threads lê.
    //
    // ⚠️ **A pegada cresce com a coluna e isso é declarado:** com `N = 32` e `8` listas ela é
    // `8 × 32 × 2 × 16 B = 8 KiB`, que cabe na cache de primeiro nível de qualquer placa — logo o
    // que a coluna move é a DIVERGÊNCIA e não a memória.
    const N_DIV: usize = 32;
    let es = arestas(N_DIV);
    let planos: Vec<f32> = es.iter().flat_map(|a| *a).collect();
    println!("  listas distintas por grupo (N = {N_DIV}) · storage ms · uniform ms");
    for c in [1usize, 2, 4, 8] {
        let mut muitos: Vec<f32> = Vec::new();
        for _ in 0..c {
            muitos.extend_from_slice(&planos);
        }
        let mut unif = muitos.clone();
        unif.resize(UNIF * 4, 0.0);
        let st = ph2d_field_gpu::probe::evaluate_medido(
            device,
            queue,
            &fonte_divergente_com(N_DIV, false, c),
            "main",
            &[],
            &[&muitos],
            &ps,
            1,
            REPETICOES,
        )
        .1;
        let un = ph2d_field_gpu::probe::evaluate_medido(
            device,
            queue,
            &fonte_divergente_com(N_DIV, true, c),
            "main",
            &[&unif],
            &[],
            &ps,
            1,
            REPETICOES,
        )
        .1;
        println!("  {c:>38} · {st:>10.3} · {un:>10.3}");
    }
    println!();

    // A INCLINAÇÃO, que é o que cancela os custos fixos (despacho, leitura, compilação).
    let (n0, m0) = linhas[0];
    let (n1, m1) = linhas[linhas.len() - 1];
    #[allow(clippy::cast_precision_loss)]
    let por = |k: usize| f64::from(m1[k] - m0[k]) * 1.0e6 / ((n1 - n0) as f64 * AMOSTRAS as f64);
    let d = por(0);
    println!(
        "  ⇒ inclinação de {n0} a {n1}, em ns por aresta por amostra (× a dobrada):\n     \
         dobrada {:.5}\n     storage dinâmica {:.5} ({:.2}×) · storage fixa {:.5} ({:.2}×)\n     \
         uniform fixa   {:.5} ({:.2}×)\n     storage DIVERGENTE {:.5} ({:.2}×) · uniform \
         DIVERGENTE {:.5} ({:.2}×)\n",
        d,
        por(1),
        por(1) / d,
        por(2),
        por(2) / d,
        por(3),
        por(3) / d,
        por(4),
        por(4) / d,
        por(5),
        por(5) / d,
    );
}
