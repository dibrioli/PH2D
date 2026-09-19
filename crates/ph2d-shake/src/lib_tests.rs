//! Os gates da lei do abanão, e a sonda que mede o que o expoente compra.
//!
//! ⚠️ **Cada gate mede o PRODUTO da lei, nunca uma constante** — a excepção é o par
//! `EXPOENTE_MIN`/`EXPOENTE_MAX`, cujo teste é sobre o que a faixa FAZ ao barro, não sobre o
//! número.

use super::*;

/// A `Lei` de fábrica das medições: uma câmera que treme meio segundo.
fn lei() -> Lei {
    Lei {
        amplitude: 0.25,
        frequencia: 20.0,
        decaimento: 2.0,
        expoente: 2,
        semente: 0x5EED,
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// O que a vista faz
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **Sem trauma a vista é BYTE-IDÊNTICA à de antes desta wave.** É esta linha que autoriza a
/// lei a entrar no caminho de omissão da câmera de toda cena do app.
#[test]
fn sem_trauma_a_vista_nao_se_mexe_ao_bit() {
    let l = lei();
    for i in 0..500 {
        let t = i as f32 * 0.01;
        assert_eq!(
            deslocamento(&l, 0.0, t),
            [0.0, 0.0],
            "com trauma zero a vista tem de ficar exactamente onde estava (t = {t})"
        );
    }
}

/// ⭐⭐⭐ **O abanão é facto do TEMPO e não da taxa de quadros** — o cabeçalho do módulo.
///
/// ⚠️ **A régua é a CONTINUIDADE, e ela tem de ter um CONTROLO que a reprove**: amostrar mais fino
/// tem de encolher o maior salto entre amostras vizinhas. Um sorteio por chamada (o `baralha`, aqui
/// como controlo negativo) NÃO encolhe — ele lê o mesmo `~2,0` em toda densidade.
#[test]
fn o_abanao_e_facto_do_tempo_e_nao_da_taxa_de_quadros() {
    // ⚠️⚠️ **A VARREDURA TEM DE ATRAVESSAR MUITAS CÉLULAS DA GRELHA, e a 1.ª redacção NÃO
    // ATRAVESSAVA NENHUMA** — ela varria `t ∈ [0, 1]`, que é **uma** célula, e ali um ruído
    // reduzido a DEGRAU é constante. ⇒ a mutação que apaga a interpolação **SOBREVIVEU** com o
    // gate verde. *Uma régua de continuidade que não cruza uma fronteira da grelha mede um
    // planalto*, e o corpus inteiro estava no ponto onde a lei não tem nada para dizer.
    const ATE: f32 = 40.0;
    fn maior_salto(passo: f32) -> f32 {
        let mut pior: f32 = 0.0;
        let n = (ATE / passo) as usize;
        let mut anterior = ruido(7, 0.0);
        for i in 1..=n {
            let v = ruido(7, i as f32 * passo);
            pior = pior.max((v - anterior).abs());
            anterior = v;
        }
        pior
    }

    let grosso = maior_salto(ATE / 200.0);
    let fino = maior_salto(ATE / 1600.0);
    assert!(
        fino <= grosso / 4.0,
        "amostrar 8x mais fino tem de encolher o salto: grosso {grosso:.6}, fino {fino:.6}"
    );

    // ⛔ **O CONTROLO NEGATIVO** — a mesma régua sobre um sorteio por índice.
    fn maior_salto_sorteado(passo: f32) -> f32 {
        let mut pior: f32 = 0.0;
        let n = (ATE / passo) as usize;
        let mut estado = 7u64;
        let mut anterior = (baralha(&mut estado) >> 40) as f32;
        for _ in 1..=n {
            let v = (baralha(&mut estado) >> 40) as f32;
            pior = pior.max((v - anterior).abs());
            anterior = v;
        }
        pior
    }
    let c_grosso = maior_salto_sorteado(ATE / 200.0);
    let c_fino = maior_salto_sorteado(ATE / 1600.0);
    assert!(
        c_fino > c_grosso / 4.0,
        "o controlo tem de REPROVAR a régua, senão ela não mede continuidade nenhuma: \
         grosso {c_grosso:.1}, fino {c_fino:.1}"
    );
}

/// O ruído usa a faixa que promete — ⚠️ **as duas metades**: um ruído preso perto de zero passaria
/// na cerca de cima e não abanaria nada.
#[test]
fn o_ruido_fica_na_faixa_e_usa_a_faixa() {
    let (mut menor, mut maior) = (f32::MAX, f32::MIN);
    for i in 0..20_000 {
        let v = ruido(0xABCD, i as f32 * 0.037);
        assert!((-1.0..=1.0).contains(&v), "o ruído saiu da faixa: {v}");
        menor = menor.min(v);
        maior = maior.max(v);
    }
    assert!(
        maior > 0.9 && menor < -0.9,
        "o ruído tem de CHEGAR às pontas, senão o abanão é mais fraco do que a amplitude diz: \
         {menor:.3}..{maior:.3}"
    );
}

/// ⚠️ **A barra fica LONGE de `1` e não colada em `0`**, que é a lição do módulo Motion: o resíduo
/// é ruído de amostragem, e uma barra apertada mediria o tamanho da grelha em vez da lei.
#[test]
fn os_dois_eixos_nao_andam_juntos() {
    let l = lei();
    let (mut sxy, mut sxx, mut syy) = (0.0f64, 0.0f64, 0.0f64);
    for i in 0..10_000 {
        let [x, y] = deslocamento(&l, 1.0, i as f32 * 0.0137);
        sxy += f64::from(x) * f64::from(y);
        sxx += f64::from(x) * f64::from(x);
        syy += f64::from(y) * f64::from(y);
    }
    let r = (sxy / (sxx * syy).sqrt()).abs();
    assert!(
        r < 0.2,
        "os dois eixos estão correlacionados ({r:.4}) — a vista abanaria numa diagonal só"
    );
}

/// Duas sementes, dois abanões. ⛔ Sem isto, duas câmeras na mesma cena tremeriam em uníssono.
#[test]
fn a_semente_muda_o_abanao() {
    let a = Lei {
        semente: 1,
        ..lei()
    };
    let b = Lei {
        semente: 2,
        ..lei()
    };
    let mut diferentes = 0;
    for i in 0..200 {
        let t = i as f32 * 0.01;
        if deslocamento(&a, 1.0, t) != deslocamento(&b, 1.0, t) {
            diferentes += 1;
        }
    }
    assert!(diferentes > 190, "só {diferentes} de 200 amostras diferem");
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// O trauma
// ─────────────────────────────────────────────────────────────────────────────────────────────

#[test]
fn o_trauma_satura_e_nunca_passa_de_um() {
    let mut t = 0.0;
    for _ in 0..10 {
        t = acumula(t, 0.4);
    }
    assert_eq!(t, TRAUMA_MAX, "dez impulsos de 0,4 têm de saturar em 1");
    assert_eq!(
        acumula(0.5, -3.0),
        0.5,
        "um impulso negativo não tira trauma"
    );
}

/// ⭐ **O decaimento CHEGA a zero, exactamente** — um exponencial nunca chegaria, e a vista nunca
/// assentaria (nem deixaria de pagar o ruído).
#[test]
fn o_trauma_chega_a_zero_e_fica() {
    let mut t = 1.0;
    // 2 trauma/s durante meio segundo, em passos de 60 Hz.
    for _ in 0..30 {
        t = decai(t, 2.0, 1.0 / 60.0);
    }
    assert!(t < 1e-6, "sobrou trauma depois da vida inteira dele: {t}");
    assert_eq!(decai(0.0, 2.0, 1.0), 0.0, "e ele não fica negativo");
}

/// ⚠️ **O decaimento é rate-independent:** quatro passos de `dt/4` têm de deixar o mesmo trauma que
/// um de `dt`. Sem isto, um quadro perdido mudaria a duração do abanão.
#[test]
fn o_decaimento_nao_depende_do_tamanho_do_passo() {
    let grosso = decai(1.0, 2.0, 0.25);
    let mut fino = 1.0;
    for _ in 0..4 {
        fino = decai(fino, 2.0, 0.0625);
    }
    assert!(
        (grosso - fino).abs() < 1e-6,
        "grosso {grosso:.7} contra fino {fino:.7}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// A distância
// ─────────────────────────────────────────────────────────────────────────────────────────────

#[test]
fn a_atenuacao_e_um_ao_pe_e_zero_longe() {
    assert_eq!(atenuacao(0.0, 2.0, 8.0), 1.0);
    assert_eq!(
        atenuacao(2.0, 2.0, 8.0),
        1.0,
        "a borda de dentro ainda é cheia"
    );
    assert_eq!(atenuacao(8.0, 2.0, 8.0), 0.0, "a borda de fora já é nada");
    assert_eq!(atenuacao(99.0, 2.0, 8.0), 0.0);
    // Monótona e suave no meio.
    let mut anterior = 1.0;
    for i in 0..=60 {
        let d = 2.0 + (i as f32 / 60.0) * 6.0;
        let a = atenuacao(d, 2.0, 8.0);
        assert!(a <= anterior + 1e-6, "a atenuação subiu ao afastar: {a}");
        anterior = a;
    }
}

/// ⚠️ **A degenerescência que a lei responde sozinha** — ver o doc da [`atenuacao`].
#[test]
fn fora_menor_que_dentro_e_um_corte_duro() {
    assert_eq!(atenuacao(4.999, 5.0, 1.0), 1.0);
    assert_eq!(atenuacao(5.001, 5.0, 1.0), 0.0);
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// O expoente
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⛔ **`0` é coagido a `1`, e a régua é o BARRO:** com `n = 0` o trauma seria ignorado e meia vida
/// de abanão teria a amplitude cheia.
#[test]
fn o_expoente_zero_nao_apaga_o_trauma() {
    let l = Lei {
        expoente: 0,
        ..lei()
    };
    let um = Lei {
        expoente: 1,
        ..lei()
    };
    for i in 0..100 {
        let t = i as f32 * 0.01;
        assert_eq!(
            deslocamento(&l, 0.5, t),
            deslocamento(&um, 0.5, t),
            "o expoente 0 tem de ser lido como 1"
        );
    }
    // E o controlo: a meio trauma, o expoente 1 de facto entrega metade.
    let cheio = deslocamento(&um, 1.0, 0.3)[0];
    let meio = deslocamento(&um, 0.5, 0.3)[0];
    assert!(
        (meio * 2.0 - cheio).abs() < 1e-6,
        "meio trauma tem de dar meio deslocamento: {meio} contra {cheio}"
    );
}

/// ⭐ **O que o expoente compra**, com a fórmula fechada ao lado: a fracção do movimento que cai no
/// primeiro quarto da vida do abanão é `1 − (3/4)^(n+1)`.
#[test]
fn o_expoente_encurta_a_cauda() {
    fn fraccao_no_primeiro_quarto(n: u8) -> f64 {
        // ∫ do envelope `trauma^n` sobre a vida, por soma de Riemann sobre o produto REAL.
        let (mut cedo, mut total) = (0.0f64, 0.0f64);
        let passos = 20_000;
        for i in 0..passos {
            let u = i as f64 / f64::from(passos); // fracção da vida
            let l = Lei {
                expoente: n,
                ..lei()
            };
            // O envelope é o deslocamento com o ruído no pico — mede-se pela amplitude efectiva.
            let env = f64::from(deslocamento(&l, (1.0 - u) as f32, 0.0)[0].abs());
            total += env;
            if u < 0.25 {
                cedo += env;
            }
        }
        cedo / total
    }
    let mut anterior = 0.0;
    for n in EXPOENTE_MIN..=EXPOENTE_MAX {
        let f = fraccao_no_primeiro_quarto(n);
        let esperado = 1.0 - 0.75f64.powi(i32::from(n) + 1);
        assert!(
            (f - esperado).abs() < 0.01,
            "n = {n}: medido {f:.3}, a forma fechada dá {esperado:.3}"
        );
        assert!(
            f > anterior,
            "n = {n} tem de concentrar mais cedo que {anterior:.3}"
        );
        anterior = f;
    }
}

/// ⚠️ **Sonda, não gate.** Corre com `--ignored` e imprime a tabela que o doc de [`EXPOENTE_MAX`]
/// cita — *um número num doc que ninguém consegue reproduzir envelhece sozinho*.
#[test]
#[ignore = "sonda: imprime a tabela do expoente"]
fn mede_a_cauda_por_expoente() {
    eprintln!("\n  n | movimento no 1.º quarto da vida");
    eprintln!("  --+-------------------------------");
    for n in 1..=6u8 {
        let f = 1.0 - 0.75f64.powi(i32::from(n) + 1);
        eprintln!("  {n} | {:.1} %", f * 100.0);
    }
    eprintln!("\n  ⇒ a faixa que ship é {EXPOENTE_MIN}..={EXPOENTE_MAX} (as referências).\n");
}
