//! ⭐⭐⭐ **O INSTRUMENTO QUE MEDE A CURVATURA NOS DOIS MOTORES — e não o pixel.**
//!
//! # ⛔⛔ A dívida que este ficheiro paga, e porque ela não se podia pagar com uma imagem
//!
//! O gate `o_dispositivo_e_a_referencia_pintam_o_mesmo_estilo` (`estilo_tests`) achou uma
//! divergência **pré-existente** entre os dois motores que só um consumidor LINEAR revela, e
//! atribuiu-a à CURVATURA por dose-resposta: a contagem de píxeis fora satura em `~165` e o pior
//! byte **escala com o ganho** (`nitidez 1 → 7`, `2 → 13`, `8 → 45`). ⇒ *uma diferença pequena
//! numa grandeza, amplificada.*
//!
//! ⚠️⚠️ **Uma paridade de imagem não sabe dizer QUANTO nem ONDE.** Ela mede a saída de uma cadeia de
//! dez leis e devolve um byte; a pergunta é sobre UM número a montante. Este ficheiro mede esse
//! número, ponto a ponto, nos **três** avaliadores que o produto tem:
//!
//! | quem | o quê | onde vive |
//! |---|---|---|
//! | `cpu` | `Hybrid::eval`, **`f32` em lote** (fidget) — *o que a referência de CPU pinta* | [`ph2d_field_eval::hybrid`] |
//! | `gpu` | a fita em WGSL, **`f32`** — *o que o dispositivo pinta* | [`ph2d_field_gpu::paint_wgsl::CURVATURA`] |
//! | `f64` | `Field::at`, **`f64` ponto a ponto** — *o árbitro; nenhum motor o pinta* | [`ph2d_field_eval::Field`] |
//!
//! ⛔⛔⛔ **E a PRIMEIRA coisa que ele refutou foi a premissa com que eu vim.** O briefing desta
//! auditoria dizia *«a fita da CPU é `f64` achatada e o WGSL é `f32`»*. **É falso para este
//! caminho:** o `f64` é o [`ph2d_field_eval::Field::at`], que responde *um ponto de cada vez* e
//! serve as sondas; quem a [`ph2d_field_render::curvatura::curvaturas`] chama é o
//! [`ph2d_field_eval::hybrid::Hybrid::eval`], que é **`f32` em lote** e é quem desenha. ⇒ os dois
//! motores são **os dois `f32`**, e a divergência não é de precisão declarada — é de **dois
//! avaliadores `f32` diferentes do mesmo campo**. *O doc do `Field` escreve-o na segunda linha, e a
//! frase do briefing lê-se como verdadeira sem ninguém abrir o ficheiro.*
//!
//! # ⭐⭐⭐ A ATRIBUIÇÃO: as cinco amostras CRUAS atravessam o barramento
//!
//! Medir só `H` de cada lado responde *«divergem»* e não *«porquê»*. A sonda devolve, por ponto, o
//! `H` do dispositivo **e as cinco amostras de campo que ele somou** — e com elas a mesma soma
//! corre outra vez, em `f64`, na CPU:
//!
//! - `H_gpu` contra `H_gpu` **refeito em `f64` sobre as amostras do GPU** ⇒ isola a **ARITMÉTICA**
//!   do Laplaciano (a soma, o `4·f(p)`, o `/2ε²`);
//! - `f_gpu[j]` contra `f_cpu[j]` ⇒ isola o **AVALIADOR DE CAMPO**;
//! - as posições das amostras são bit-idênticas nos dois motores **por construção**, e há gate:
//!   o CPU faz `d.mul_add(ε, p)` e o WGSL faz `p + o·ε` com `o ∈ {±1}`, logo o produto é exacto e a
//!   forma fundida e a solta arredondam a mesma vez. ⚠️ *Esta é a única das três que se prova sem
//!   placa, e é por isso que ela é um teste à parte que corre em toda a máquina.*
//!
//! # ⚠️ Porque `ε` não pode ser a explicação, e como se vê isso
//!
//! O passo é **um número da CPU** que viaja no uniforme (`PaintSetup::curv_eps`, cujo doc já diz
//! *«ele vem da CPU e não se deriva aqui»*), logo os dois lados dividem pelo mesmo `ε` **ao bit**.
//! A sonda devolve-o na saída e o gate compara-o com o que a referência usou — *uma explicação que
//! se pode fechar com uma igualdade não deve ficar aberta numa tabela.*

use ph2d_field_render::curvatura;

/// ⚠️ **O arnês declara um `Pintor` com um `knobs`** porque a [`ph2d_field_gpu::paint_wgsl::CURVATURA`]
/// lê `pintor.knobs.z` — ver lá porque o `ε` **não** passa por argumento. A ordem dos bindings é a
/// do [`ph2d_field_gpu::probe::evaluate`]: uniformes, storages, entrada, saída.
const ARNES: &str = r"
struct Pintor { knobs: vec4<f32> };
@group(0) @binding(0) var<uniform> pintor: Pintor;
@group(0) @binding(1) var<storage, read> k: array<f32>;
@group(0) @binding(2) var<storage, read> entrada: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> saida: array<vec4<f32>>;

@compute @workgroup_size(64, 1, 1)
fn avalia(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.x;
    if (i >= arrayLength(&entrada)) { return; }
    let p = entrada[i].xyz;
    let e = pintor.knobs.z;
    // ⭐ As CINCO amostras cruas ao lado do resultado — é com elas que a divergência se atribui ao
    // avaliador ou à aritmética, em vez de ficar num «os dois motores discordam».
    let f0 = field(p + vec3<f32>( 1.0, -1.0, -1.0) * e);
    let f1 = field(p + vec3<f32>(-1.0, -1.0,  1.0) * e);
    let f2 = field(p + vec3<f32>(-1.0,  1.0, -1.0) * e);
    let f3 = field(p + vec3<f32>( 1.0,  1.0,  1.0) * e);
    saida[i * 2u + 0u] = vec4<f32>(curvatura_em(p), f0, f1, f2);
    saida[i * 2u + 1u] = vec4<f32>(f3, field(p), e, 0.0);
}
";

/// Os quatro vértices do tetraedro — **os mesmos** da [`curvatura`], e lidos dela seria melhor:
/// ⚠️ ela declara-os `const` privado. *Fica nomeado; enquanto assim for, o gate
/// [`os_offsets_do_arnes_sao_os_do_produto`] é quem prende as duas cópias.*
const OFFSETS: [[f32; 3]; 4] = [
    [1.0, -1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [1.0, 1.0, 1.0],
];

/// ⭐⭐⭐ **AS POSIÇÕES DAS CINCO AMOSTRAS SÃO BIT-IDÊNTICAS NOS DOIS MOTORES** — e prova-se sem
/// placa nenhuma.
///
/// A CPU escreve `d.mul_add(ε, p)` (uma multiplicação-soma **fundida**, um arredondamento) e o WGSL
/// escreve `p + o·ε` (duas operações, dois arredondamentos). ⛔ Em geral **não** são o mesmo `f32` —
/// é exactamente a divergência por construção que o [`ph2d_style`] proíbe por escrito. ⭐ **Aqui
/// são**, e por uma propriedade e não por sorte: `o ∈ {+1, −1}`, logo `o·ε` é **exacto** e as duas
/// formas arredondam a mesma soma uma só vez.
///
/// ⚠️ *Sem este gate, a hipótese «as duas placas amostram em sítios diferentes» ficaria aberta numa
/// tabela — e ela é a primeira que ocorre a quem vê duas curvaturas diferentes no mesmo pixel.*
#[test]
fn as_cinco_amostras_caem_no_mesmo_sitio_nos_dois_motores() {
    // Uma varredura larga de posições e de passos, incluindo os que o produto usa.
    let mut vistos = 0usize;
    for escala in [0.05f32, 0.4, 1.0, 2.5, 17.0] {
        let eps = curvatura::eps_para(escala);
        for i in -37..=37i32 {
            let c = escala * (i as f32) / 37.0;
            for d in OFFSETS.iter().flatten() {
                let fundido = d.mul_add(eps, c);
                let solto = c + d * eps;
                assert_eq!(
                    fundido.to_bits(),
                    solto.to_bits(),
                    "o `mul_add` da CPU e o `p + o·ε` do WGSL divergem em c={c} ε={eps} d={d}"
                );
                vistos += 1;
            }
        }
    }
    // ⚠️ **O piso de população**: uma varredura vazia passa este `assert_eq!` zero vezes e lê-se
    // como prova.
    assert!(vistos > 1_000, "varredura pobre: {vistos}");
}

/// ⚠️ **O CONTROLO da mesma lei**: com um `o` que **não** é `±1` as duas formas separam-se.
///
/// *Sem ele, o gate acima poderia estar a afirmar que `mul_add` e `a*b + c` são sempre o mesmo
/// número — que é falso, e é a razão de o [`ph2d_style`] ter uma lei a proibir o `mul_add`.*
#[test]
fn e_o_controlo_com_um_offset_que_nao_e_unitario() {
    let mut separou = 0usize;
    let eps = curvatura::eps_para(1.0);
    for i in 1..5_000i32 {
        let c = (i as f32) * 1e-3;
        let d = 1.0000001f32;
        if d.mul_add(eps, c).to_bits() != (c + d * eps).to_bits() {
            separou += 1;
        }
    }
    assert!(
        separou > 0,
        "as duas formas nunca se separaram — o gate irmão está a afirmar o trivial"
    );
}

/// Os offsets do arnês são os do produto — ver a nota de [`OFFSETS`].
#[test]
fn os_offsets_do_arnes_sao_os_do_produto() {
    let fonte = include_str!("../../ph2d-field-render/src/curvatura.rs");
    for d in OFFSETS {
        let linha = format!("[{:?}, {:?}, {:?}],", d[0], d[1], d[2]);
        assert!(
            fonte.contains(&linha),
            "o offset {d:?} não está no produto — a cópia envelheceu ({linha})"
        );
    }
    // ⚠️ E a lei do arnês é a do produto: o WGSL que ele corre é a const do pintor, não uma cópia.
    assert!(
        ph2d_field_gpu::paint_wgsl::CURVATURA.contains("fn curvatura_em(p: vec3<f32>) -> f32"),
        "a const do pintor deixou de declarar a função que este arnês chama"
    );
}

/// O que a sonda mede num ponto — as três leituras e a atribuição.
struct Leitura {
    /// O índice do pixel na tela — ⚠️ **é ele que faz a pergunta da BORDA ser exprimível**: uma
    /// borda é uma propriedade de VIZINHANÇA, e uma lista de pontos soltos não a tem.
    i: usize,
    /// O ponto do G-buffer, em mundo.
    p: [f32; 3],
    /// `H` da referência de CPU (`Hybrid`, `f32` em lote) — o que a imagem de CPU pinta.
    cpu: f32,
    /// `H` do dispositivo (a fita em WGSL, `f32`) — o que a imagem do dispositivo pinta.
    gpu: f32,
    /// `H` em `f64` sobre as MESMAS posições — o árbitro.
    f64: f64,
    /// `H` refeito em `f64` sobre as amostras que o DISPOSITIVO devolveu — isola a aritmética.
    gpu_refeito: f64,
    /// O maior `|f_gpu[j] − f_cpu[j]|` das cinco amostras — isola o avaliador.
    df: f64,
}

/// ⭐⭐⭐ **A SONDA** — corre os três avaliadores sobre os pontos que a peça de facto ocupa.
///
/// ⚠️ **Os pontos vêm da MARCHA DO DISPOSITIVO** (`gpu_frame::march`), que é a mesma porta por que o
/// gate do estilo os obtém: os dois motores sombreiam o **mesmo** G-buffer, logo a curvatura é
/// medida no mesmo `p` dos dois lados. *Se cada motor marchasse o seu, a divergência de curvatura
/// ficaria misturada com a do traçado, e esta sonda mediria duas coisas somadas.*
fn mede() -> Option<(Vec<Leitura>, f32, f32)> {
    let doc = crate::gpu_frame::estilo_tests::peca_com_aresta_e_cova();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let t = crate::gpu_frame::shared()?;
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let mundos: Vec<[f32; 3]> = luz.iter().map(|l| l.world).collect();
    let (w, h) = (
        crate::gpu_frame::paint_parity_tests::W,
        crate::gpu_frame::paint_parity_tests::H,
    );
    let (g, _) = crate::gpu_frame::march(t, &doc, &reg, &cam, &mundos, None, w, h, true)?;

    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)?;
    // ⭐ **O MESMO `ε` dos dois lados, pela mesma porta** — ver o cabeçalho.
    let eps = curvatura::eps_para(bola.radius);

    // (1) a referência de CPU, pela porta do produto.
    let mut hybrid = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
    let cpu = curvatura::do_gbuffer(&mut hybrid, &g, eps);

    let vivos: Vec<usize> = (0..g.hit.len()).filter(|&i| g.hit[i]).collect();
    let pontos: Vec<[f32; 3]> = vivos.iter().map(|&i| g.point[i]).collect();

    // (2) o dispositivo, com a lei do PINTOR e a fita deste documento.
    let campo = ph2d_field_eval::device::DeviceField::new(&doc, &reg)?;
    assert!(
        campo.sculpts().is_empty(),
        "a fixtura ganhou uma escultura — o arnês não liga a grade e a fita chamaria `escultura_*`"
    );
    let fita = campo.tape_wgsl()?;
    let fonte = format!(
        "{}\n{}\n{ARNES}",
        fita.source,
        ph2d_field_gpu::paint_wgsl::CURVATURA
    );
    let entradas: Vec<[f32; 4]> = pontos.iter().map(|p| [p[0], p[1], p[2], 0.0]).collect();
    let guarda = t.lock().expect("o traçador");
    let (device, queue) = guarda.parts();
    let saida = ph2d_field_gpu::probe::evaluate(
        device,
        queue,
        &fonte,
        "avalia",
        &[&[0.0, 0.0, eps, 0.0]],
        &[&fita.consts],
        &entradas,
        2,
    );
    drop(guarda);

    // (3) o árbitro em `f64`, sobre as MESMAS posições `f32` promovidas.
    let arbitro = ph2d_field_eval::Field::new(&doc);
    let denom = 2.0 * f64::from(eps) * f64::from(eps);

    let mut out = Vec::with_capacity(pontos.len());
    for (j, (&i, p)) in vivos.iter().zip(&pontos).enumerate() {
        let a = saida[j * 2];
        let b = saida[j * 2 + 1];
        assert_eq!(
            b[2].to_bits(),
            eps.to_bits(),
            "o `ε` do dispositivo não é o da referência — a hipótese do passo ficaria aberta"
        );
        // As cinco posições, escritas como a CPU as escreve (e provadas iguais às do WGSL).
        let amostra = |d: [f32; 3]| [0, 1, 2].map(|k| d[k].mul_add(eps, p[k]));
        let mut soma64 = 0.0f64;
        let mut df = 0.0f64;
        let gpu_f = [a[1], a[2], a[3], b[0]];
        for (n, d) in OFFSETS.into_iter().enumerate() {
            let q = amostra(d);
            let v = arbitro.at(f64::from(q[0]), f64::from(q[1]), f64::from(q[2]));
            soma64 += v;
            // ⚠️ A comparação é contra o ÁRBITRO, e não contra o `Hybrid`: pedir ao `Hybrid` uma
            // amostra solta exigiria um segundo lote e o produto nunca o faz. *O que interessa é a
            // magnitude de `δf`, e o árbitro é quem a mede sem ser nenhum dos dois motores.*
            df = df.max((f64::from(gpu_f[n]) - v).abs());
        }
        let centro = arbitro.at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
        df = df.max((f64::from(b[1]) - centro).abs());
        let h64 = (soma64 - 4.0 * centro) / denom * 0.5;
        // ⭐ A MESMA fórmula, em `f64`, sobre as amostras que o DISPOSITIVO devolveu.
        let soma_gpu = f64::from(a[1]) + f64::from(a[2]) + f64::from(a[3]) + f64::from(b[0]);
        let refeito = (soma_gpu - 4.0 * f64::from(b[1])) / denom * 0.5;
        out.push(Leitura {
            i,
            p: *p,
            cpu: cpu[i],
            gpu: a[0],
            f64: h64,
            gpu_refeito: refeito,
            df,
        });
    }
    Some((out, eps, bola.radius))
}

/// Um resumo de uma coluna — `p50` e `max` do módulo.
fn resume(v: &mut [f64]) -> (f64, f64) {
    if v.is_empty() {
        return (0.0, 0.0);
    }
    v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    (v[v.len() / 2], v[v.len() - 1])
}

/// ⭐⭐⭐ **A TABELA: onde e quanto os dois motores divergem na CURVATURA, e de quem é a culpa.**
///
/// ```text
/// bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo test -p ph2d-app-field3d \
///   curvatura_parity_tests::onde_e_quanto_os_dois_motores_divergem_na_curvatura \
///   -- --ignored --nocapture
/// ```
///
/// ⚠️ **Ela imprime VALORES e não relógios** — nada aqui depende da carga da máquina.
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn onde_e_quanto_os_dois_motores_divergem_na_curvatura() {
    let Some((v, eps, _raio)) = mede() else {
        println!("sem adaptador — saltado");
        return;
    };
    assert!(v.len() > 2_000, "a peça encheu só {} píxeis", v.len());
    println!(
        "\nε = {eps:e} · {} pontos · raio da peça {:.4}\n",
        v.len(),
        eps / 0.0064
    );

    // ── por MAGNITUDE de H ────────────────────────────────────────────────────────────────────
    // ⚠️ As bandas são do FENÓMENO e não escolhidas: a bola é `+1/0,55 = 1,82`, a cratera é
    // `−1/0,30 = −3,33` e o lábio com filete passa de `10`. Uma banda «plana» exige `|H|` pequeno,
    // que nesta peça só acontece onde as duas esferas se encontram.
    println!(
        "  banda de |H|      n     |Δcpu−gpu| p50 / max     |Δgpu−f64| p50 / max     |Δcpu−f64| p50 / max"
    );
    for (rot, lo, hi) in [
        ("|H| < 0,5 (quase plano)", 0.0, 0.5),
        ("0,5 ≤ |H| < 2 (a bola)", 0.5, 2.0),
        ("2 ≤ |H| < 5 (a cratera)", 2.0, 5.0),
        ("|H| ≥ 5 (o lábio)", 5.0, f64::INFINITY),
    ] {
        let sel: Vec<&Leitura> = v
            .iter()
            .filter(|l| {
                let m = l.f64.abs();
                m >= lo && m < hi
            })
            .collect();
        if sel.is_empty() {
            println!("  {rot:<24} 0  —");
            continue;
        }
        let mut a: Vec<f64> = sel
            .iter()
            .map(|l| (f64::from(l.cpu) - f64::from(l.gpu)).abs())
            .collect();
        let mut b: Vec<f64> = sel
            .iter()
            .map(|l| (f64::from(l.gpu) - l.f64).abs())
            .collect();
        let mut c: Vec<f64> = sel
            .iter()
            .map(|l| (f64::from(l.cpu) - l.f64).abs())
            .collect();
        let (ap, am) = resume(&mut a);
        let (bp, bm) = resume(&mut b);
        let (cp, cm) = resume(&mut c);
        println!(
            "  {rot:<24} {:>5}   {ap:>9.2e} {am:>9.2e}   {bp:>9.2e} {bm:>9.2e}   {cp:>9.2e} {cm:>9.2e}",
            sel.len()
        );
    }

    // ── a ATRIBUIÇÃO ──────────────────────────────────────────────────────────────────────────
    let mut arit: Vec<f64> = v
        .iter()
        .map(|l| (f64::from(l.gpu) - l.gpu_refeito).abs())
        .collect();
    let mut aval: Vec<f64> = v.iter().map(|l| l.df).collect();
    let mut cpugpu: Vec<f64> = v
        .iter()
        .map(|l| (f64::from(l.cpu) - f64::from(l.gpu)).abs())
        .collect();
    let (arp, arm) = resume(&mut arit);
    let (avp, avm) = resume(&mut aval);
    let (cgp, cgm) = resume(&mut cpugpu);
    println!("\n  ATRIBUIÇÃO (todos os pontos)");
    println!("    |Δ cpu − gpu| em H .................. p50 {cgp:.3e}  max {cgm:.3e}");
    println!("    δf nas cinco amostras (contra f64) .. p50 {avp:.3e}  max {avm:.3e}");
    println!("    a ARITMÉTICA do Laplaciano (gpu vs f64 sobre as amostras DO gpu)");
    println!("                                          p50 {arp:.3e}  max {arm:.3e}");
    // ⭐⭐⭐ A previsão: um erro `δf` numa soma que deixa `ε²·2H` sai amplificado por `1/ε²`.
    println!(
        "    previsto por δf/ε² = {:.3e}   (medido |Δcpu−gpu| p50 = {cgp:.3e})",
        avp / f64::from(eps * eps)
    );

    // ── por POSIÇÃO ───────────────────────────────────────────────────────────────────────────
    // A cratera está centrada em `z = +0,52`; a distância a ela separa o interior do resto.
    println!("\n  por POSIÇÃO (distância ao centro da cratera, em unidades da peça)");
    for (lo, hi) in [(0.0, 0.30), (0.30, 0.45), (0.45, 0.70), (0.70, 9.0)] {
        let sel: Vec<&Leitura> = v
            .iter()
            .filter(|l| {
                let d = ((l.p[0]).powi(2) + (l.p[1]).powi(2) + (l.p[2] - 0.52).powi(2)).sqrt();
                d >= lo && d < hi
            })
            .collect();
        if sel.is_empty() {
            continue;
        }
        let mut a: Vec<f64> = sel
            .iter()
            .map(|l| (f64::from(l.cpu) - f64::from(l.gpu)).abs())
            .collect();
        let mut hs: Vec<f64> = sel.iter().map(|l| l.f64).collect();
        let (ap, am) = resume(&mut a);
        hs.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
        println!(
            "    d ∈ [{lo:.2}, {hi:.2})  n={:>5}  H p50 {:>7.3}   |Δcpu−gpu| p50 {ap:.3e}  max {am:.3e}",
            sel.len(),
            hs[hs.len() / 2]
        );
    }
    println!();
}

/// ⭐⭐⭐ **O VEREDITO: a divergência é do AVALIADOR DE CAMPO, nunca da aritmética nem do passo.**
///
/// As três metades, e cada uma reprova por um motivo diferente:
///
/// 1. **a aritmética do Laplaciano é a mesma lei** — refazer a soma do dispositivo em `f64` dá o
///    `H` dele a menos do arredondamento de `f32` daquela soma, e não a menos da divergência
///    inteira. ⛔ Se a lei do WGSL fosse outra (outro `ε`, outro denominador, outro estêncil), esta
///    metade estouraria e as outras duas não.
/// 2. **os avaliadores de campo divergem**, e a magnitude é a de um `f32` (`δf` da ordem de `1e-7`
///    num campo cujos intermédios são da ordem de `1`);
/// 3. **e a divergência de `H` é `δf` amplificado por `1/ε²`** — é isto que explica porque uma
///    diferença invisível no campo dá dezenas de bytes num consumidor linear com ganho.
///
/// ⚠️ **As barras são ORDENS DE GRANDEZA e não afinações**: elas separam `1e-7` de `1e-3` de `1e0`,
/// que é a distância entre as três hipóteses. *Uma barra apertada aqui mediria a placa.*
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn a_divergencia_da_curvatura_e_do_avaliador_de_campo() {
    let Some((v, eps, _raio)) = mede() else {
        panic!("sem adaptador de GPU — este gate não pode ser saltado em silêncio");
    };
    assert!(v.len() > 2_000, "a peça encheu só {} píxeis", v.len());

    // (1) a ARITMÉTICA é a mesma lei — o `H` do dispositivo é a soma dele, refeita.
    //     A folga é o arredondamento de `f32` da própria divisão, relativo a `|H|`.
    let mau: Vec<&Leitura> = v
        .iter()
        .filter(|l| {
            let rel = (f64::from(l.gpu) - l.gpu_refeito).abs() / l.gpu_refeito.abs().max(1e-3);
            rel > 1e-5
        })
        .collect();
    assert!(
        mau.is_empty(),
        "a aritmética do Laplaciano do dispositivo não é a da CPU em {} pontos (o 1.º: gpu={} refeito={})",
        mau.len(),
        mau[0].gpu,
        mau[0].gpu_refeito
    );

    // (2) os AVALIADORES divergem, e na magnitude de um `f32`.
    let mut df: Vec<f64> = v.iter().map(|l| l.df).collect();
    let (dfp, dfm) = resume(&mut df);
    assert!(
        dfp > 0.0,
        "os dois avaliadores deram o MESMO `f32` em toda amostra — este gate não tem sujeito"
    );
    assert!(
        dfm < 1e-4,
        "δf = {dfm:e} é grande demais para ser arredondamento de `f32` — procure uma LEI diferente"
    );

    // (3) e a divergência de `H` é `δf` amplificado por `1/ε²`.
    let mut dh: Vec<f64> = v
        .iter()
        .map(|l| (f64::from(l.cpu) - f64::from(l.gpu)).abs())
        .collect();
    let (dhp, _) = resume(&mut dh);
    let previsto = dfp / f64::from(eps * eps);
    assert!(
        dhp > 0.0,
        "os dois motores deram o MESMO `H` em toda parte — a divergência do estilo seria outra"
    );
    // ⚠️ Uma ordem de grandeza de folga dos dois lados: `δf` é um MÁXIMO sobre cinco amostras que
    // se cancelam em parte, logo a previsão é um majorante solto por construção.
    assert!(
        dhp <= previsto * 10.0 && dhp >= previsto / 100.0,
        "Δ H p50 = {dhp:e} não é δf/ε² = {previsto:e} — a amplificação não explica a divergência"
    );
}

/// ⭐⭐⭐ **D4 — A BORDA DURA: ela é IGUALMENTE dura nos dois motores?**
///
/// # ⚠️ A pergunta, e porque ela é do INSTRUMENTO e não da lei
///
/// O dono reportou *«Edge tint e Cavity tint com bordas muito duras sem ajustes finos»*. **A LEI é
/// de outro agente.** O que só este instrumento pode responder é se o DISPOSITIVO a faz mais dura —
/// ou mais serrilhada — do que a referência: se fizesse, o diagnóstico mudava de sítio.
///
/// # ⭐⭐⭐ A régua: o peso da tinta, e o SALTO dele entre píxeis vizinhos
///
/// O que o olho lê como «dureza» é o peso `c = clamp(H·R·nitidez, −1, 1)` a saltar de `0` para `1`
/// em poucos píxeis. ⇒ três colunas, e cada uma responde a uma coisa diferente:
///
/// | coluna | o que ela diz |
/// |---|---|
/// | **na banda** | quantos píxeis têm `\|c\| < 1` — *o material de que uma transição suave seria feita*. Perto de zero ⇒ a borda é um DEGRAU, e a dureza é da LEI. |
/// | **salto p99 / máx** | o maior `\|Δc\|` entre vizinhos de 4 — a dureza medida, por motor |
/// | **lados trocados** | píxeis em que um motor satura e o outro não — *a serrilha que só o dispositivo teria* |
///
/// ⛔ **Sem a terceira coluna esta sonda não responde à pergunta**: as duas primeiras podem bater
/// entre motores e a borda ainda assim serrilhar, se os píxeis que discordam forem os da transição.
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn a_borda_dura_e_igualmente_dura_nos_dois_motores() {
    let Some((v, _eps, raio)) = mede() else {
        println!("sem adaptador — saltado");
        return;
    };
    let w = crate::gpu_frame::paint_parity_tests::W as usize;
    // Um mapa esparso `pixel → (c_cpu, c_gpu)`, para a vizinhança de 4.
    // ⚠️ `BTreeMap` e não `HashMap` — HR-5 / ADR-0022. Aqui a ordem até ajuda: a varredura dos
    // vizinhos fica determinística, e uma sonda que imprime números tem de os imprimir iguais.
    let mut mapa: std::collections::BTreeMap<usize, (f32, f32)> = std::collections::BTreeMap::new();

    println!("\n  raio da peça = {raio:.4}\n");
    println!(
        "  nitidez   na banda |c|<1     salto p99/máx (cpu)    salto p99/máx (gpu)   lados trocados"
    );
    for nitidez in [0.2f32, 1.0, 2.0, 8.0] {
        mapa.clear();
        let mut na_banda = 0usize;
        let mut trocados = 0usize;
        for l in &v {
            let c = |h: f32| (h * raio * nitidez).clamp(-1.0, 1.0);
            let (a, b) = (c(l.cpu), c(l.gpu));
            if a.abs() < 1.0 || b.abs() < 1.0 {
                na_banda += 1;
            }
            // ⭐ **«lados trocados» é o que a serrilha É**: um motor entrega tinta cheia e o outro
            // não, no MESMO pixel. ⚠️ Conta-se também a troca de SINAL — ali a tinta muda de
            // ARESTA para COVA, que são duas cores diferentes.
            if (a.abs() >= 1.0) != (b.abs() >= 1.0) || (a > 0.0) != (b > 0.0) {
                trocados += 1;
            }
            mapa.insert(l.i, (a, b));
        }
        let mut saltos_cpu: Vec<f64> = Vec::new();
        let mut saltos_gpu: Vec<f64> = Vec::new();
        for (&i, &(a, b)) in &mapa {
            for viz in [i + 1, i + w] {
                if let Some(&(a2, b2)) = mapa.get(&viz) {
                    saltos_cpu.push(f64::from((a - a2).abs()));
                    saltos_gpu.push(f64::from((b - b2).abs()));
                }
            }
        }
        let q = |s: &mut Vec<f64>| -> (f64, f64) {
            if s.is_empty() {
                return (0.0, 0.0);
            }
            s.sort_by(|x, y| x.partial_cmp(y).expect("sem NaN"));
            (s[s.len() * 99 / 100], s[s.len() - 1])
        };
        let (c99, cmax) = q(&mut saltos_cpu);
        let (g99, gmax) = q(&mut saltos_gpu);
        println!(
            "  {nitidez:>6.1}   {na_banda:>6} / {:<6}   {c99:>8.4} / {cmax:<8.4}   {g99:>8.4} / {gmax:<8.4}   {trocados:>6}",
            v.len()
        );
    }
    println!();
}
