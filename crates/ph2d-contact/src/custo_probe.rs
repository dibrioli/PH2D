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

/// ⭐⭐⭐ **ONDE O TEMPO MORA DENTRO DE UMA VARREDURA** — a atribuição antes de qualquer cura.
///
/// ⚠️ Ela chama as MESMAS funções do produto (`grelha`, `celula`, `corrigida`), nunca uma segunda
/// cópia da lei: o que ela faz é cronometrar as fases **separadamente**, somando ao lado.
#[test]
#[ignore = "sonda de medição, não gate"]
fn onde_o_tempo_mora_dentro_de_uma_varredura() {
    const N: usize = 500;
    const REPS: usize = 200;
    eprintln!("\n  ═══ ATRIBUIÇÃO DE UMA VARREDURA ({N} peças) ═══\n");
    let (p0, c, w) = campo(N, ESPACO_DE_CENA);
    let inv = vec![0.0; N];
    let pecas = Pecas::novas(&c, &w, &inv);
    let ativo: Vec<bool> = (0..N).map(|i| ativo(p0[i], c[i].as_ref())).collect();
    let alcance = (0..N)
        .filter_map(|i| c[i].map(|x| x.alcance()))
        .fold(0.0f32, f32::max);
    let lado = 2.0 * alcance;

    let cron = |f: &mut dyn FnMut()| {
        let mut melhor = f64::INFINITY;
        for _ in 0..CORRIDAS {
            let agora = Instant::now();
            for _ in 0..REPS {
                f();
            }
            melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e6 / REPS as f64);
        }
        melhor
    };

    // (a) só construir a grelha
    let t_grelha = cron(&mut || {
        let mut g = crate::grelha::Grelha::default();
        g.constroi(&p0, &ativo, lado);
        std::hint::black_box(&g);
    });
    // (b) construir + colher os vizinhos das 9 células, sem tocar na lei
    let t_colher = cron(&mut || {
        let mut g = crate::grelha::Grelha::default();
        g.constroi(&p0, &ativo, lado);
        let mut viz: Vec<u32> = Vec::new();
        let mut total = 0usize;
        for k in 0..N {
            g.vizinhos_de(k, &mut viz);
            total += viz.len();
        }
        std::hint::black_box(total);
    });
    // (c) a varredura inteira, pela porta do produto
    let t_tudo = cron(&mut || {
        let mut p = p0.clone();
        let mut g = vec![0.0; N];
        separate(&mut p, &mut Saida { giro: &mut g }, &pecas, 1);
        std::hint::black_box(&p);
    });

    // Quantos parceiros cada peça de facto vê — o que a LEI custa é proporcional a isto.
    let mut g = crate::grelha::Grelha::default();
    g.constroi(&p0, &ativo, lado);
    let mut viz: Vec<u32> = Vec::new();
    let mut soma = 0usize;
    for k in 0..N {
        g.vizinhos_de(k, &mut viz);
        soma += viz.len();
    }
    eprintln!(
        "  parceiros por peça (média) .... {:.1}",
        soma as f64 / N as f64
    );
    eprintln!("  (a) construir a grelha ........ {t_grelha:>8.1} µs");
    eprintln!("  (b) (a) + colher e ordenar .... {t_colher:>8.1} µs");
    eprintln!("  (c) a varredura inteira ....... {t_tudo:>8.1} µs");
    eprintln!(
        "\n  ⇒ achar os pares: {:.0}%   ·   a LEI: {:.0}%",
        t_colher / t_tudo * 100.0,
        (t_tudo - t_colher) / t_tudo * 100.0
    );
    eprintln!("\n  load durante a corrida: {}\n", carga());
}

/// ⭐⭐⭐ **QUANTAS VARREDURAS ANTES DE NADA MAIS SE MEXER** — a pergunta que decide se o tecto
/// alto custa alguma coisa.
///
/// ⚠️ Chamar `separate(.., 1)` em sequência é **bit-idêntico** a uma chamada de `k`: `ativo` e
/// `alcance_max` derivam de colisores e de posições finitas (que não mudam de natureza), e o `giro`
/// acumula na [`Saida`], que é exactamente o que o laço interno faz.
#[test]
#[ignore = "sonda de medição, não gate"]
fn quantas_varreduras_antes_de_nada_mais_se_mexer() {
    eprintln!("\n  ═══ ONDE O CAMPO PÁRA DE SE MEXER ═══\n");
    eprintln!(
        "  `parado` = a 1.ª varredura que não mexe UM BIT em nenhuma peça. A partir dela, toda\n  \
         varredura seguinte lê a mesma entrada e devolve a mesma coisa — por indução.\n"
    );
    eprintln!(
        "  {:<22} │ {:>10} │ {:>12} │ {:>14}",
        "fixtura", "peças", "parado em", "de 1024, úteis"
    );
    eprintln!("  -----------------------|------------|--------------|----------------");
    for (nome, n, espaco) in [
        ("campo de cena", 500usize, ESPACO_DE_CENA),
        ("campo denso", 500, 1.25),
        ("campo de cena", 48, ESPACO_DE_CENA),
        ("campo de cena", 1000, ESPACO_DE_CENA),
    ] {
        let (p0, c, w) = campo(n, espaco);
        let inv = vec![0.0; n];
        let pecas = Pecas::novas(&c, &w, &inv);
        let mut p = p0.clone();
        let mut giro = vec![0.0; n];
        let mut parou = None;
        for v in 1..=1024usize {
            let (antes_p, antes_g) = (p.clone(), giro.clone());
            separate(&mut p, &mut Saida { giro: &mut giro }, &pecas, 1);
            if p == antes_p && giro == antes_g {
                parou = Some(v);
                break;
            }
        }
        match parou {
            Some(v) => eprintln!(
                "  {nome:<22} │ {n:>10} │ {v:>12} │ {:>13.1}%",
                v as f64 / 1024.0 * 100.0
            ),
            None => eprintln!("  {nome:<22} │ {n:>10} │  nunca parou │         100.0%"),
        }
    }
    eprintln!("\n  load durante a corrida: {}\n", carga());
}

/// ⭐⭐⭐ **A PARTIR DE QUANTAS PEÇAS O PARALELO PAGA** — de onde sai a [`PECAS_PARA_PARALELIZAR`].
///
/// ⚠️ A régua é a RAZÃO entre as duas colunas, e não um relógio absoluto: sob carga as duas sobem
/// juntas. O ponto de equilíbrio é onde a razão cruza `1`.
#[test]
#[ignore = "sonda de medição, não gate"]
fn onde_o_paralelo_passa_a_pagar() {
    const V: usize = 64;
    eprintln!("\n  ═══ ONDE O PARALELO PASSA A PAGAR ({V} varreduras) ═══\n");
    eprintln!(
        "  {:<8} │ {:>12} │ {:>12} │ {:>9}",
        "peças", "1 núcleo", "N núcleos", "razão"
    );
    eprintln!("  ---------|--------------|--------------|----------");
    for n in [16usize, 32, 64, 128, 256, 500, 1000, 4000] {
        let (p0, c, w) = campo(n, ESPACO_DE_CENA);
        let inv: Vec<f32> = (0..n)
            .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
            .collect();
        let pecas = Pecas::novas(&c, &w, &inv);
        let mut col = [0.0f64; 2];
        for (i, paralelo) in [false, true].into_iter().enumerate() {
            let mut melhor = f64::INFINITY;
            for _ in 0..CORRIDAS {
                let mut p = p0.clone();
                let mut g = vec![0.0; n];
                let agora = Instant::now();
                separate_com(
                    &mut p,
                    &mut Saida { giro: &mut g },
                    &pecas,
                    V,
                    paralelo,
                    REPOUSO_VISIVEL,
                );
                melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e3);
            }
            col[i] = melhor;
        }
        eprintln!(
            "  {n:<8} │ {:>9.3} ms │ {:>9.3} ms │ {:>8.2}×",
            col[0],
            col[1],
            col[0] / col[1]
        );
    }
    eprintln!("\n  load durante a corrida: {}\n", carga());
}

/// ⭐⭐⭐ **QUANTO UMA VARREDURA AINDA MEXE, VARREDURA A VARREDURA** — a pergunta que decide se um
/// tecto alto pode ser barato.
///
/// O atalho do ponto fixo exige igualdade **ao bit**, e com a rotação solta duas caixas acertam-se
/// por um ULP para sempre. ⇒ a pergunta útil não é *«parou?»* mas *«ainda mexe alguma coisa que se
/// VEJA?»* — e a unidade é a do artista: a maior correcção de uma varredura, em fracção da ARESTA
/// da peça.
#[test]
#[ignore = "sonda de medição, não gate"]
fn quanto_uma_varredura_ainda_mexe() {
    eprintln!("\n  ═══ A MAIOR CORRECÇÃO DE UMA VARREDURA (em fracção da aresta) ═══\n");
    eprintln!(
        "  {:<22} │ {:>7} │ {:>9} │ {:>9} │ {:>9} │ {:>9} │ {:>9}",
        "fixtura", "v=8", "v=32", "v=64", "v=128", "v=256", "v=1024"
    );
    eprintln!(
        "  -----------------------|---------|-----------|-----------|-----------|-----------|----------"
    );
    for (nome, n, espaco) in [
        ("campo de cena", 500usize, ESPACO_DE_CENA),
        ("campo denso", 500, 1.25),
        ("campo de cena", 2000, ESPACO_DE_CENA),
    ] {
        let (p0, c, w) = campo(n, espaco);
        let inv: Vec<f32> = (0..n)
            .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
            .collect();
        let pecas = Pecas::novas(&c, &w, &inv);
        let (mut p, mut g) = (p0.clone(), vec![0.0; n]);
        let mut linha = String::new();
        let marcos = [8usize, 32, 64, 128, 256, 1024];
        let mut v = 0usize;
        for alvo in marcos {
            let mut maior = 0.0f32;
            while v < alvo {
                let antes = p.clone();
                separate(&mut p, &mut Saida { giro: &mut g }, &pecas, 1);
                maior = (0..n)
                    .map(|i| {
                        let d = [p[i][0] - antes[i][0], p[i][1] - antes[i][1]];
                        d[0].hypot(d[1])
                    })
                    .fold(0.0f32, f32::max);
                v += 1;
            }
            linha.push_str(&format!(" │ {maior:>9.2e}"));
        }
        eprintln!("  {nome:<22}{linha}");
    }
    eprintln!(
        "\n  ⚠️ A aresta de uma peça é `1,0` nesta fixtura — logo a coluna lê-se em FRACÇÃO da peça."
    );
    eprintln!("  load: {}\n", carga());
}

/// ⭐⭐⭐ **PARAR QUANDO NADA MAIS SE VÊ — quanto isso custa em FIDELIDADE.**
///
/// ⚠️ **A régua certa não é o resíduo de UMA varredura, é o DESVIO da saída** contra varrer até ao
/// fim: o resíduo decai, mas se decaísse devagar a soma da cauda seria visível. Esta sonda mede as
/// duas coisas ao lado uma da outra, para cada limiar candidato.
#[test]
#[ignore = "sonda de medição, não gate"]
fn parar_quando_nada_mais_se_ve() {
    const TECTO: usize = 1024;
    eprintln!("\n  ═══ PARAR CEDO: QUANTAS VARREDURAS, E QUANTO SE PERDE ═══\n");
    eprintln!(
        "  {:<20} │ {:>9} │ {:>11} │ {:>13} │ {:>12}",
        "fixtura", "limiar", "varreduras", "desvio (peça)", "pares sobrep."
    );
    eprintln!("  ---------------------|-----------|-------------|---------------|-------------");
    for (nome, n, espaco) in [
        ("campo de cena", 500usize, ESPACO_DE_CENA),
        ("campo denso", 500, 1.25),
        ("campo de cena", 2000, ESPACO_DE_CENA),
    ] {
        let (p0, c, w) = campo(n, espaco);
        let inv: Vec<f32> = (0..n)
            .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
            .collect();
        let pecas = Pecas::novas(&c, &w, &inv);
        // A referência: varrer o tecto inteiro.
        let (mut cheio, mut g_cheio) = (p0.clone(), vec![0.0; n]);
        separate(&mut cheio, &mut Saida { giro: &mut g_cheio }, &pecas, TECTO);
        let sobrep_cheio = pares_sobrepostos(&cheio, &c);
        for limiar in [1e-3f32, 1e-4, 1e-5, 1e-6] {
            let (mut p, mut g) = (p0.clone(), vec![0.0; n]);
            let mut usadas = 0usize;
            for _ in 0..TECTO {
                let antes = p.clone();
                separate(&mut p, &mut Saida { giro: &mut g }, &pecas, 1);
                usadas += 1;
                let maior = (0..n)
                    .map(|i| (p[i][0] - antes[i][0]).hypot(p[i][1] - antes[i][1]))
                    .fold(0.0f32, f32::max);
                if maior < limiar {
                    break;
                }
            }
            let desvio = (0..n)
                .map(|i| (p[i][0] - cheio[i][0]).hypot(p[i][1] - cheio[i][1]))
                .fold(0.0f32, f32::max);
            eprintln!(
                "  {nome:<20} │ {limiar:>9.0e} │ {usadas:>11} │ {desvio:>13.2e} │ {:>5} (era {sobrep_cheio})",
                pares_sobrepostos(&p, &c)
            );
        }
    }
    eprintln!("\n  ⚠️ A aresta de uma peça é `1,0`: o desvio lê-se em FRACÇÃO da peça.");
    eprintln!("  load: {}\n", carga());
}
