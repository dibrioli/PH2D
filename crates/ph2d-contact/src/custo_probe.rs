//! ⭐⭐⭐ **O QUE A SEPARAÇÃO CUSTA À POPULAÇÃO QUE O DONO NOMEOU** — a W2 do doc 115.
//!
//! # A pergunta, e porque ela não é a que o plano escrevia
//!
//! A ordem do dono de 2026-09-17 põe um colisor em cada objecto da cena (Sprite · vector · Flip) e
//! manda o app separá-los sozinho. Perguntado quantas cópias de um objecto com colisor uma cena
//! dele costuma ter, respondeu **«centenas»** — e é essa a população que esta sonda mede.
//!
//! ⛔⛔ **O plano assustou-se com o número ERRADO.** A tabela `§1-ter` da sonda da corda lê `200`
//! peças a `10,467 ms` (`62,8 %` de um quadro) e mede o `push_apart` do NÓ, que é um laço de
//! **todos os pares** (`for i { for j in i+1.. }`, lido no ficheiro). A rota que os objectos tomam
//! é a deste motor, que tem **grelha espacial** — e o cabeçalho da crate declara que ela *«dá os
//! MESMOS BITS que todos-os-pares»*. ⇒ *as duas colunas abaixo são a mesma lei, e só a forma de
//! achar os pares muda.*
//!
//! # ⚠️ A régua desta máquina, e porque ela é o MÍNIMO e não a mediana
//!
//! Nenhuma leitura de relógio desta workstation vale nada acima de `load ~5`, e esta sonda foi
//! escrita com a máquina entre `45` e `91`. O mínimo de `N` corridas é a técnica que a casa já
//! registou para isto: *a corrida mais rápida é a menos interferida*, logo o mínimo aproxima-se do
//! custo verdadeiro por baixo, enquanto a mediana segue a carga.
//!
//! ⚠️ **Ele é um LIMITE SUPERIOR do melhor caso, não o custo calmo.** Se um número aterrar perto do
//! orçamento, ele tem de ser re-tirado numa máquina calma antes de decidir seja o que for — e a
//! linha `load` impressa ao lado de cada corrida é o que permite dizer isso depois.

use super::*;
use std::time::Instant;

/// O orçamento: um quadro a 60 Hz.
const QUADRO_MS: f64 = 16.67;

/// Quantas corridas por célula. O mínimo delas é a leitura.
const CORRIDAS: usize = 15;

/// As varreduras por quadro — o número que a corda precisou para fechar (doc 114 §11).
const VARREDURAS: usize = 32;

/// ⭐ **O espaçamento de uma CENA**, em múltiplos da aresta da caixa — e ele é MEDIDO, não
/// escolhido: a tabela `§9.1` do doc 115 varre a densidade e este é o ponto onde a separação de
/// facto CONVERGE (zero pares sobrepostos), que é a definição de *«uma cena e não uma pilha»*.
const ESPACO_DE_CENA: f32 = 2.0;

/// Um inteiro espalhado em `[0, 1)`, determinístico — a nuvem é a mesma em toda corrida.
fn acaso(i: u32, faixa: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9e37_79b9) ^ faixa.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    #[expect(clippy::cast_precision_loss, reason = "24 bits cabem num f32")]
    let v = (h >> 8) as f32 / 16_777_216.0;
    v
}

/// ⛔⛔⛔ **A 1.ª REDACÇÃO DESTA FIXTURA ERA UMA PILHA, e o doc dela afirmava o contrário sem o
/// medir.** Ela punha o lado do campo em `√n × 1,25` com caixas `1 × 1` ⇒ **64 % de empacotamento**,
/// e a tabela de convergência lia `709` pares sobrepostos antes e `649` depois de 32 varreduras.
/// *Não era o motor a não convergir: era um campo sem para onde as peças irem.*
///
/// ⚠️ **A lição é a da casa:** uma fixtura cuja densidade não foi medida não afirma nada sobre
/// custo nem sobre convergência — e a minha prosa dizia *«sem a cena ser uma pilha compacta»* como
/// se fosse um facto.
///
/// ⇒ a densidade passa a ser **ARGUMENTO**, e as tabelas varrem-na: `espaco` é o passo médio entre
/// centros em múltiplos da ARESTA da caixa, logo o empacotamento é `1/espaco²`.
fn campo(n: usize, espaco: f32) -> (Vec<[f32; 2]>, Vec<Option<Colisor>>, Vec<f32>) {
    const MEIA: f32 = 0.5;
    #[expect(clippy::cast_precision_loss, reason = "n e' uma contagem de cena")]
    let lado = (n as f32).sqrt() * (2.0 * MEIA) * espaco;
    let mut p = Vec::with_capacity(n);
    let mut c = Vec::with_capacity(n);
    for i in 0..n {
        #[expect(clippy::cast_possible_truncation, reason = "i < n, e n cabe num u32")]
        let k = i as u32;
        p.push([acaso(k, 1) * lado, acaso(k, 2) * lado]);
        // Caixas com orientações diferentes — o caso que só a CPU resolve hoje, e o que um
        // Sprite rodado na cena traz.
        let a = acaso(k, 3) * std::f32::consts::TAU;
        c.push(Some(Colisor::caixa([MEIA, MEIA], [a.cos(), a.sin()])));
    }
    let w = vec![1.0; n];
    (p, c, w)
}

/// O mínimo de [`CORRIDAS`] chamadas, em ms.
fn minimo(n: usize, espaco: f32, grelha: bool) -> f64 {
    let (p0, c, w) = campo(n, espaco);
    let inv = vec![0.0; n];
    let pecas = Pecas::novas(&c, &w, &inv);
    let mut melhor = f64::INFINITY;
    for _ in 0..CORRIDAS {
        let mut p = p0.clone();
        let mut giro = vec![0.0; n];
        let mut saida = Saida { giro: &mut giro };
        let agora = Instant::now();
        if grelha {
            separate(&mut p, &mut saida, &pecas, VARREDURAS);
        } else {
            separate_all_pairs(&mut p, &mut saida, &pecas, VARREDURAS);
        }
        melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e3);
    }
    melhor
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(str::to_string))
        .unwrap_or_else(|| "?".to_string())
}

/// ⛔⛔⛔ **A 1.ª REDACÇÃO DESTA RÉGUA ACUSAVA A PRÓPRIA CONVERGÊNCIA** — ela contava
/// `contato(..).is_some()`, e o solver pousa cada par **exactamente a TOCAR**, onde a função
/// devolve `Some` com penetração `~0`. ⇒ o campo mais esparso da varredura lia `44 → 28` e parecia
/// não convergir, quando o que não convergia era a régua.
///
/// ⚠️⚠️ **É a SEGUNDA vez nesta linha** — a §12 do doc 114 pagou-a com discos (`d < 2·RAIO`
/// estrito contava discos a tocar como sobrepostos) e escreveu a cura ao lado: a barra é a
/// **penetração VISÍVEL**, e `2 %` *«não é um epsilon de vírgula flutuante — é «a penetração é
/// visível?»»*. Aqui a aresta da caixa é `1,0`, logo a barra é `0,02`.
const PENETRACAO_VISIVEL: f32 = 0.02;

/// Quantos PARES continuam **visivelmente** sobrepostos — a régua da convergência.
///
/// ⚠️ **É uma CONTAGEM e não um relógio**, e é por isso que ela vale numa máquina a `load 50`:
/// *um par sobreposto é um facto determinístico da saída*, e nenhuma carga o muda.
fn pares_sobrepostos(p: &[[f32; 2]], c: &[Option<Colisor>]) -> usize {
    let n = p.len();
    let mut k = 0;
    for i in 0..n {
        for j in (i + 1)..n {
            if let (Some(a), Some(b)) = (c[i].as_ref(), c[j].as_ref())
                && contato(a, p[i], b, p[j], false)
                    .is_some_and(|t| t.penetracao > PENETRACAO_VISIVEL)
            {
                k += 1;
            }
        }
    }
    k
}

/// ⭐⭐⭐ **A DENSIDADE É QUEM MANDA — e é ela que diz o que «uma cena» quer dizer.**
///
/// A 1.ª fixtura desta sonda era uma PILHA a `64 %` de empacotamento, e nela a separação não
/// converge: não há para onde as peças irem. ⇒ esta tabela varre o espaçamento e mede, para cada
/// densidade, se a lei CHEGA a zero pares sobrepostos.
///
/// ⚠️ A coluna dos pares é uma CONTAGEM determinística — ela vale com a máquina carregada.
#[test]
#[ignore = "sonda de medição, não gate"]
fn a_densidade_decide_se_a_separacao_converge() {
    const N: usize = 500;
    eprintln!("\n  ═══ A DENSIDADE DECIDE (doc 115 W2) ═══\n");
    eprintln!(
        "  {N} caixas orientadas, {VARREDURAS} varreduras. `espaço` é o passo médio entre centros\n  \
         em múltiplos da ARESTA, logo o empacotamento é `1/espaço²`.\n"
    );
    eprintln!(
        "  {:<8} │ {:>13} │ {:>10} │ {:>10} │ {:>13}",
        "espaço", "empacotamento", "antes", "depois", "min de relógio"
    );
    eprintln!("  ---------|---------------|------------|------------|---------------");
    let inv = vec![0.0; N];
    for espaco in [1.25f32, 1.5, 2.0, 3.0, 4.0] {
        let (p0, c, w) = campo(N, espaco);
        let antes = pares_sobrepostos(&p0, &c);
        let pecas = Pecas::novas(&c, &w, &inv);
        let mut p = p0.clone();
        let mut giro = vec![0.0; N];
        separate(&mut p, &mut Saida { giro: &mut giro }, &pecas, VARREDURAS);
        let depois = pares_sobrepostos(&p, &c);
        let ms = minimo(N, espaco, true);
        eprintln!(
            "  {espaco:<8.2} │ {:>12.0}% │ {antes:>10} │ {depois:>10} │ {ms:>10.3} ms",
            100.0 / (espaco * espaco)
        );
    }
    eprintln!("\n  load durante a corrida: {}\n", carga());
}

/// ⭐⭐⭐ **QUANTAS VARREDURAS UM CAMPO DE OBJECTOS PRECISA — e ela NÃO é a da corda.**
///
/// O `32` que a coluna acima usa veio da corda (doc 114 §11), que é uma CADEIA: cada ponto só toca
/// os vizinhos, logo a informação viaja um elo por varredura e um laço fechado precisa de muitas.
/// *Um campo de caixas espalhadas não tem essa cadeia* — cada peça vê as vizinhas dela de uma vez.
///
/// ⚠️ **A régua é a CONTAGEM de pares sobrepostos, não o relógio** — logo esta tabela é a única
/// desta sonda que uma máquina carregada não estraga.
#[test]
#[ignore = "sonda de medição, não gate"]
fn quantas_varreduras_um_campo_de_objectos_precisa() {
    const N: usize = 500;
    eprintln!("\n  ═══ QUANTAS VARREDURAS O CAMPO PRECISA (doc 115 W2) ═══\n");
    eprintln!(
        "  {N} caixas orientadas, a mesma densidade de cena. A coluna dos pares é uma CONTAGEM\n  \
         determinística — ela vale mesmo com a máquina carregada; a do relógio não.\n"
    );
    let (p0, c, w) = campo(N, ESPACO_DE_CENA);
    let antes = pares_sobrepostos(&p0, &c);
    eprintln!("  antes de separar: {antes} pares sobrepostos\n");
    eprintln!(
        "  {:<12} │ {:>16} │ {:>13} │ {:>9}",
        "varreduras", "pares sobrepostos", "min de relógio", "% quadro"
    );
    eprintln!("  -------------|------------------|---------------|----------");
    let inv = vec![0.0; N];
    let pecas = Pecas::novas(&c, &w, &inv);
    for v in [2usize, 4, 8, 16, 32] {
        let mut p = p0.clone();
        let mut giro = vec![0.0; N];
        separate(&mut p, &mut Saida { giro: &mut giro }, &pecas, v);
        let pares = pares_sobrepostos(&p, &c);
        let mut melhor = f64::INFINITY;
        for _ in 0..CORRIDAS {
            let mut q = p0.clone();
            let mut g = vec![0.0; N];
            let agora = Instant::now();
            separate(&mut q, &mut Saida { giro: &mut g }, &pecas, v);
            melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e3);
        }
        eprintln!(
            "  {v:<12} │ {pares:>16} │ {melhor:>10.3} ms │ {:>8.1}%",
            melhor / QUADRO_MS * 100.0
        );
    }
    eprintln!("\n  load durante a corrida: {}\n", carga());
}

/// A SONDA da W2. Corra-a assim, e **de preferência numa máquina calma**:
///
/// ```text
/// cargo test -p ph2d-contact custo_probe -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medição, não gate"]
fn custo_da_separacao_na_populacao_do_dono() {
    eprintln!("\n  ═══ O QUE A SEPARAÇÃO CUSTA — «centenas» de objectos (doc 115 W2) ═══\n");
    eprintln!(
        "  Caixas ORIENTADAS num campo da densidade de uma cena, {VARREDURAS} varreduras,\n  \
         mínimo de {CORRIDAS} corridas. Um quadro tem {QUADRO_MS} ms.\n\n  \
         ⚠️ As duas colunas são a MESMA lei — o cabeçalho da crate declara que a grelha dá os\n  \
         mesmos bits que todos-os-pares. Só muda como os pares são achados.\n"
    );
    eprintln!(
        "  {:<8} │ {:>13} │ {:>15} │ {:>9} │ {:>9}",
        "peças", "GRELHA", "TODOS-OS-PARES", "% quadro", "razão"
    );
    eprintln!("  ---------|---------------|-----------------|-----------|----------");
    for n in [100usize, 250, 500, 1000] {
        let g = minimo(n, ESPACO_DE_CENA, true);
        let t = minimo(n, ESPACO_DE_CENA, false);
        eprintln!(
            "  {n:<8} │ {g:>10.3} ms │ {t:>12.3} ms │ {:>8.1}% │ {:>8.1}×",
            g / QUADRO_MS * 100.0,
            t / g
        );
    }
    eprintln!("\n  load durante a corrida: {}\n", carga());
}

/// ⭐⭐⭐ **A ESCADA QUE O DONO CORREU — o report de 18/09: *«com 1024 FPS cai para 7, usando
/// Boids»***.
///
/// ⛔⛔ **A tabela do §18 mediu `n = 16` e escreveu o custo como «`0,2 µs` por peça-varredura»** —
/// uma lei **LINEAR em `n`**. Esta sonda existe para dizer se ela é verdade fora daquela fixtura:
/// o `motion.boids` nasce com `count = 48` e o artista sobe-o.
///
/// A coluna que decide é a **razão** entre duas linhas da mesma coluna de varreduras: se ela seguir
/// o `n`, a lei era linear e o tecto está honesto; se subir mais depressa, o número do §18 descreve
/// uma fixtura e não o produto.
#[test]
#[ignore = "sonda de medição, não gate"]
fn a_escada_das_varreduras_contra_a_populacao() {
    eprintln!("\n  ═══ O QUE 1024 VARREDURAS CUSTAM, POR POPULAÇÃO (report de 18/09) ═══\n");
    eprintln!(
        "  Caixas orientadas na densidade de uma cena, mínimo de {CORRIDAS} corridas.\n  \
         Um quadro tem {QUADRO_MS} ms. As duas últimas colunas são o que o dono vê.\n"
    );
    eprintln!(
        "  {:<7} │ {:>10} │ {:>10} │ {:>11} │ {:>11} │ {:>9} │ {:>7}",
        "peças", "8", "64", "1024", "4096", "% quadro", "FPS"
    );
    eprintln!(
        "  --------|------------|------------|-------------|-------------|-----------|--------"
    );
    for n in [16usize, 48, 100, 250, 500, 1000] {
        let (p0, c, w) = campo(n, ESPACO_DE_CENA);
        // ⚠️⚠️ **A INÉRCIA É A DO PRODUTO, e a 1.ª redacção desta sonda travava-a (`inv = 0`).**
        // Uma caixa travada não roda, logo a nuvem chega a um PONTO FIXO e o atalho do `separate`
        // dispara; com a rotação solta — que é o que `passe::separa_o_que_se_desenha` faz, via
        // `inv_inercias` — as peças continuam a acertar-se por um ULP e o atalho nunca arma.
        // *Uma fixtura que trava um grau de liberdade mede outro programa.*
        let inv: Vec<f32> = (0..n)
            .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
            .collect();
        let pecas = Pecas::novas(&c, &w, &inv);
        let mut col = [0.0f64; 4];
        for (i, v) in [8usize, 64, 1024, 4096].into_iter().enumerate() {
            let mut melhor = f64::INFINITY;
            for _ in 0..CORRIDAS {
                let mut p = p0.clone();
                let mut g = vec![0.0; n];
                let agora = Instant::now();
                separate(&mut p, &mut Saida { giro: &mut g }, &pecas, v);
                melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e3);
            }
            col[i] = melhor;
        }
        eprintln!(
            "  {n:<7} │ {:>7.3} ms │ {:>7.3} ms │ {:>8.3} ms │ {:>8.3} ms │ {:>8.0}% │ {:>7.1}",
            col[0],
            col[1],
            col[2],
            col[3],
            col[2] / QUADRO_MS * 100.0,
            1000.0 / col[2].max(1e-9)
        );
    }
    eprintln!("\n  load durante a corrida: {}\n", carga());
}

/// ⭐⭐⭐ **A CENA DA FOTO — 189 DISCOS ENCOSTADOS** (report do dono, 2026-09-18: *«Boids 190
/// objetos com collide on, Sweeps 1024 = 3 FPS»*).
///
/// ⛔⛔ **A minha tabela dizia `3,5 ms` a 250 peças e ele mede `333 ms` a 189.** A diferença está na
/// foto: ali os círculos estão **ENCOSTADOS**, empacotados a preencher o ecrã — e o boids continua a
/// puxá-los para dentro enquanto a separação os empurra para fora. *Uma pilha sob compressão
/// permanente não assenta nunca*, logo o repouso visível **não arma** e a cena paga o tecto inteiro.
///
/// ⚠️ E são **DISCOS**, não caixas: sem rotação, com o raio da foto (`100 px`) e o passo dela.
#[test]
#[ignore = "sonda de medição, não gate"]
fn a_cena_da_foto_do_dono() {
    const RAIO: f32 = 100.0;
    eprintln!("\n  ═══ A CENA DA FOTO: discos ENCOSTADOS, 1024 varreduras ═══\n");
    eprintln!(
        "  {:<8} │ {:<10} │ {:>9} │ {:>12} │ {:>11} │ {:>8} │ {:>7}",
        "discos", "passo", "vizinhos", "varreduras", "relógio", "% quadro", "FPS"
    );
    eprintln!(
        "  ---------|------------|-----------|--------------|-------------|----------|--------"
    );
    for (n, passo_r) in [
        (189usize, 2.0f32),
        (189, 1.8),
        (189, 1.6),
        (189, 1.4),
        (500, 1.8),
    ] {
        // Uma grelha hexagonal apertada, como a da foto.
        let passo = passo_r * RAIO;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "lado de grelha"
        )]
        let cols = ((n as f32).sqrt() * 1.15).ceil() as usize;
        let mut p = Vec::with_capacity(n);
        let mut c = Vec::with_capacity(n);
        for i in 0..n {
            let (gx, gy) = (i % cols, i / cols);
            #[expect(clippy::cast_precision_loss, reason = "coordenadas de fixtura")]
            let (fx, fy) = (gx as f32, gy as f32);
            #[expect(clippy::cast_possible_truncation, reason = "um indice de fixtura")]
            let k = i as u32;
            p.push([
                fx * passo + (gy % 2) as f32 * passo * 0.5 + (acaso(k, 1) - 0.5) * passo * 0.08,
                fy * passo * 0.87 + (acaso(k, 2) - 0.5) * passo * 0.08,
            ]);
            c.push(Some(Colisor::disco(RAIO)));
        }
        let w = vec![1.0; n];
        let inv: Vec<f32> = (0..n)
            .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
            .collect();
        let pecas = Pecas::novas(&c, &w, &inv);
        // Quantos vizinhos cada disco vê — o multiplicador do custo de uma varredura.
        let ativo: Vec<bool> = (0..n).map(|i| ativo(p[i], c[i].as_ref())).collect();
        let mut grade = crate::grelha::Grelha::default();
        grade.constroi(&p, &ativo, 2.0 * RAIO);
        let mut viz: Vec<u32> = Vec::new();
        let mut soma = 0usize;
        for k in 0..n {
            grade.vizinhos_de(k, &mut viz);
            soma += viz.len();
        }
        let mut usadas = 0usize;
        let mut melhor = f64::INFINITY;
        for _ in 0..CORRIDAS.min(5) {
            let mut q = p.clone();
            let mut g = vec![0.0; n];
            let agora = Instant::now();
            usadas = separate(&mut q, &mut Saida { giro: &mut g }, &pecas, 1024);
            melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e3);
        }
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let vizinhos = soma as f64 / n as f64;
        eprintln!(
            "  {n:<8} │ {passo_r:<10.1} │ {vizinhos:>9.1} │ {usadas:>12} │ {melhor:>8.1} ms │ {:>7.0}% │ {:>7.1}",
            melhor / QUADRO_MS * 100.0,
            1000.0 / melhor.max(1e-9)
        );
    }
    eprintln!(
        "\n  ⚠️ `passo` é o espaçamento entre centros em múltiplos do RAIO: `2,0` = a tocar."
    );
    eprintln!("  load: {}\n", carga());
}

/// As sondas do REPOUSO — irmãs desta pelo tecto de LOC (HR-18) e por ASSUNTO: aqui mora o que a
/// separação CUSTA, ali mora onde ela PÁRA.
#[path = "custo_probe_repouso.rs"]
mod repouso;

/// As sondas da ATRIBUIÇÃO — irmãs desta pelo tecto de LOC (HR-18) e por ASSUNTO: aqui mora QUANTO
/// custa, ali mora ONDE dentro de uma varredura o tempo mora e quanto o paralelo de facto rende.
#[path = "custo_probe_atribuicao.rs"]
mod atribuicao;
