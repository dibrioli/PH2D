//! **ONDE A SEPARAÇÃO PÁRA** — irmã do [`super`] pelo tecto de LOC (HR-18) e por ASSUNTO: ali mora
//! o que ela CUSTA, aqui mora onde ela deixa de valer a pena.
//!
//! As três perguntas, em ordem: *a nuvem chega a um ponto fixo AO BIT?* (quase nunca — com a
//! rotação solta duas caixas acertam-se por um ULP para sempre) · *quanto uma varredura ainda mexe?*
//! · e a que decide o número que o produto usa: *parar cedo custa quanto em FIDELIDADE?*

use super::*;

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

/// ⭐⭐⭐ **NUMA PILHA COMPRIMIDA, AS VARREDURAS COMPRAM ALGUMA COISA?** — a pergunta que o readout
/// do dono levantou (2026-09-18: `1000 pecas x 68 varreduras x 156 vizinhos`).
///
/// ⚠️ Com `156` candidatos por peça a cena está **muito** acima da capacidade: o passe empurra e não
/// há para onde ir. ⇒ antes de optimizar o motor, a pergunta é se aquelas varreduras mudam o que se
/// VÊ — *um tecto gasto que não compra nada é o knob mais caro do app*.
#[test]
#[ignore = "sonda de medição, não gate"]
fn numa_pilha_comprimida_as_varreduras_compram_alguma_coisa() {
    const N: usize = 1000;
    const RAIO: f32 = 100.0;
    eprintln!("\n  ═══ O QUE AS VARREDURAS COMPRAM, POR DENSIDADE ═══\n");
    eprintln!(
        "  {:<8} │ {:>9} │ {:>8} │ {:>8} │ {:>8} │ {:>8} │ {:>8} │ {:>9}",
        "passo", "vizinhos", "antes", "v=8", "v=32", "v=64", "v=256", "relógio v=64"
    );
    eprintln!(
        "  ---------|-----------|----------|----------|----------|----------|----------|-----------"
    );
    for passo_r in [1.8f32, 1.2, 0.7, 0.5] {
        let (p0, c, w) = super::atribuicao::campo_de_discos(N, RAIO, passo_r);
        let inv: Vec<f32> = (0..N)
            .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
            .collect();
        let pecas = Pecas::novas(&c, &w, &inv);
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let vizinhos = crate::candidatos(&c, &p0, &w) as f32 / N as f32;
        // A régua é a PENETRAÇÃO VISÍVEL, em fracção do raio — o que o artista vê.
        let sobrepostos = |p: &[[f32; 2]]| {
            let mut k = 0usize;
            for i in 0..N {
                for j in (i + 1)..N {
                    if let (Some(a), Some(b)) = (c[i].as_ref(), c[j].as_ref())
                        && contato(a, p[i], b, p[j], false)
                            .is_some_and(|t| t.penetracao > 0.02 * RAIO)
                    {
                        k += 1;
                    }
                }
            }
            k
        };
        let mut col = [0usize; 4];
        let mut ms = 0.0f64;
        for (i, v) in [8usize, 32, 64, 256].into_iter().enumerate() {
            let mut p = p0.clone();
            let mut g = vec![0.0; N];
            let agora = Instant::now();
            separate(&mut p, &mut Saida { giro: &mut g }, &pecas, v);
            if v == 64 {
                ms = agora.elapsed().as_secs_f64() * 1e3;
            }
            col[i] = sobrepostos(&p);
        }
        eprintln!(
            "  {passo_r:<8.1} │ {vizinhos:>9.0} │ {:>8} │ {:>8} │ {:>8} │ {:>8} │ {:>8} │ {ms:>6.1} ms",
            sobrepostos(&p0),
            col[0],
            col[1],
            col[2],
            col[3]
        );
    }
    eprintln!(
        "\n  ⚠️ A coluna é PARES com penetração visível (> 2% do raio) — o que o artista vê."
    );
    eprintln!("  load: {}\n", carga());
}
