//! Os gates da lei. ⚠️ Cada um traz o **controlo** ao lado: uma régua que não vê o fenómeno
//! acontecer não prova que ele não aconteceu.

use super::*;

fn superficie() -> Surface {
    OpenPbr::default().prepare()
}

/// Uma lâmpada branca a apontar do `+z` (de frente, como a vista).
fn lampada_de_frente() -> Lampada {
    Lampada {
        para_a_luz: [0.0, 0.0, 1.0],
        radiancia: [1.0, 1.0, 1.0],
    }
}

fn texel(normal: [f32; 3]) -> Texel {
    Texel {
        normal,
        albedo: [0.5, 0.5, 0.5],
        cobertura: 1.0,
        oclusao: 1.0,
    }
}

/// ⭐⭐⭐ **FORA DA SILHUETA A LEI É UM NO-OP EXACTO** — e é ao BIT, não «quase».
///
/// É a metade do contrato que o `baked_form` já cumpre com a tinta (*«papel nu recebe exactamente
/// nada»*), e sem ela toda re-acendida mexeria nos pixels que a forma não tocou.
#[test]
fn fora_da_silhueta_o_albedo_sai_ao_bit() {
    let s = superficie();
    let mut t = texel([0.0, 0.0, 1.0]);
    t.cobertura = 0.0;
    let fora = acende_texel(&s, &t, &[lampada_de_frente()], [0.2; 3]);
    assert_eq!(fora, t.albedo, "a cobertura 0 tem de devolver o albedo CRU");

    // ⭐ **O CONTROLO**: com cobertura cheia, a MESMA entrada tem de mover o pixel — senão este
    // gate ficaria verde sobre uma lei que não acende nada.
    t.cobertura = 1.0;
    let dentro = acende_texel(&s, &t, &[lampada_de_frente()], [0.2; 3]);
    assert_ne!(
        dentro, t.albedo,
        "controlo: com cobertura 1 a luz TEM de mudar o pixel"
    );
}

/// ⚠️ **A normal viaja quantizada em `rgba8` e chega CURTA.** Normalizar não é defensivo: sem
/// isso, uma peça inteira sai mais escura e lê-se como material errado.
#[test]
fn uma_normal_curta_acende_como_a_normalizada() {
    let s = superficie();
    let cheia = acende_texel(
        &s,
        &texel([0.0, 0.0, 1.0]),
        &[lampada_de_frente()],
        [0.0; 3],
    );
    let curta = acende_texel(
        &s,
        &texel([0.0, 0.0, 0.97]),
        &[lampada_de_frente()],
        [0.0; 3],
    );
    for c in 0..3 {
        assert!(
            (cheia[c] - curta[c]).abs() < 1e-6,
            "canal {c}: a normal curta deu {} contra {}",
            curta[c],
            cheia[c]
        );
    }

    // ⭐ **O CONTROLO**: uma normal de LADO tem de dar outra coisa — senão o teste acima passaria
    // sobre uma lei que ignora a normal por completo.
    //
    // ⛔⛔ **A 1.ª redacção usava a normal VIRADA (`-z`) e reprovou sobre produto CERTO:** a lei do
    // OpenPBR faz `forward_facing`, logo uma superfície virada ao contrário é vista pelo verso e dá
    // **exactamente** o mesmo (medido: `0,63694715` nos dois). *Isso é a referência a comportar-se
    // como uma superfície de duas faces, e não um bug* — o discriminador tem de ser uma normal
    // PERPENDICULAR à luz, onde `N·L = 0` e nenhuma inversão a salva.
    let lado = acende_texel(
        &s,
        &texel([1.0, 0.0, 0.0]),
        &[lampada_de_frente()],
        [0.0; 3],
    );
    assert!(
        (lado[0] - cheia[0]).abs() > 1e-3,
        "controlo: a normal de lado devia dar outro valor ({} contra {})",
        lado[0],
        cheia[0]
    );
}

/// ⛔ Uma normal sem direcção (o texel que a forma nunca tocou) devolve o albedo cru — **nunca
/// preto**, que pintaria um halo no contorno de toda peça assada.
#[test]
fn uma_normal_degenerada_devolve_o_albedo_e_nao_preto() {
    let s = superficie();
    let t = texel([0.0, 0.0, 0.0]);
    assert_eq!(
        acende_texel(&s, &t, &[lampada_de_frente()], [0.2; 3]),
        t.albedo
    );
}

/// ⚠️ **A oclusão pesa o AMBIENTE e não a directa.** Uma lâmpada que o artista apontou tem de
/// chegar onde ele a apontou.
#[test]
fn a_oclusao_nao_toca_a_luz_directa() {
    let s = superficie();
    let mut aberto = texel([0.0, 0.0, 1.0]);
    let mut fechado = aberto;
    aberto.oclusao = 1.0;
    fechado.oclusao = 0.0;

    // Sem ambiente, a oclusão não pode mudar um bit.
    let a = acende_texel(&s, &aberto, &[lampada_de_frente()], [0.0; 3]);
    let f = acende_texel(&s, &fechado, &[lampada_de_frente()], [0.0; 3]);
    assert_eq!(a, f, "sem ambiente, a oclusão não tem o que pesar");

    // ⭐ **O CONTROLO**: COM ambiente, ela tem de morder — senão o gate acima ficaria verde sobre
    // uma oclusão que não faz nada em lado nenhum.
    let a2 = acende_texel(&s, &aberto, &[lampada_de_frente()], [0.5; 3]);
    let f2 = acende_texel(&s, &fechado, &[lampada_de_frente()], [0.5; 3]);
    assert!(
        a2[0] > f2[0] + 1e-4,
        "controlo: com ambiente, o ocluído ({}) tem de ser mais escuro que o aberto ({})",
        f2[0],
        a2[0]
    );
}

/// ⭐⭐⭐ **A ÓPTICA É A DA `ph2d-material`, e não uma segunda redacção dela.**
///
/// Esta crate é o LAÇO; a lei é de lá. O gate reproduz a conta à mão a partir da porta pública da
/// `ph2d-material` e exige igualdade **ao bit** — no dia em que alguém escrever óptica aqui, ele
/// reprova.
#[test]
fn a_optica_e_a_da_crate_da_lei_ao_bit() {
    let s = superficie();
    let t = texel([0.3, 0.2, 0.9]);
    let l = Lampada {
        para_a_luz: normaliza([0.4, 0.5, 0.75]).unwrap(),
        radiancia: [0.9, 0.8, 0.7],
    };

    let nosso = acende_texel(&s, &t, &[l], [0.0; 3]);

    let n = normaliza(t.normal).unwrap();
    let d = s.direct(n, VISTA, l.para_a_luz, l.radiancia);
    let esperado = [t.albedo[0] * d[0], t.albedo[1] * d[1], t.albedo[2] * d[2]];
    assert_eq!(
        nosso, esperado,
        "o laço tem de ser albedo × Surface::direct, ao bit"
    );
}

/// Duas lâmpadas somam — e a soma é a das radiâncias, não um `max`.
///
/// # ⚠️ Porque a barra é a igualdade AO BIT, e não um epsilon
///
/// Com cobertura cheia e sem ambiente o resultado é `albedo × Σluz`, logo duas lâmpadas iguais dão
/// **exactamente** o dobro: dobrar um `f32` não perde um bit, e multiplicar pelo mesmo albedo
/// depois também não. *Um epsilon aqui esconderia uma lei que soma quase certo.*
///
/// ⛔⛔ **A 1.ª redacção previa `albedo + (uma − albedo) × 2` e reprovou sobre produto CERTO**
/// (`0,637` contra `0,137`): ela reconstruía o valor pela fórmula da MISTURA, que com `cobertura =
/// 1` não corre. *Uma previsão escrita com a aritmética de outro ramo mede esse outro ramo.*
#[test]
fn as_lampadas_somam() {
    let s = superficie();
    let t = texel([0.0, 0.0, 1.0]);
    let l = lampada_de_frente();
    let uma = acende_texel(&s, &t, &[l], [0.0; 3]);
    let duas = acende_texel(&s, &t, &[l, l], [0.0; 3]);
    for c in 0..3 {
        assert_eq!(
            duas[c],
            uma[c] * 2.0,
            "canal {c}: duas lâmpadas têm de dar o dobro de uma, ao bit"
        );
    }

    // ⭐ **O CONTROLO**: uma lâmpada sozinha tem de mover o pixel, senão o dobro de zero passaria.
    assert_ne!(uma, t.albedo, "controlo: uma lâmpada tem de acender");
}

/// ⭐⭐⭐ **A LÂMPADA ANTI-PARALELA À VISTA NÃO DEVOLVE `NaN`** — o achado desta crate.
///
/// Sem a cerca do [`meio_vector_degenera`] a lei do OpenPBR devolve `[NaN, NaN, NaN]` aqui, porque
/// o meio-vector `v + to_light` é o vector nulo. ⚠️ **No modelador isto tem medida nula** (a vista
/// é a do raio e varia por pixel); **num canvas 2D a [`VISTA`] é constante**, logo esta é uma
/// configuração que o artista escreve, e ela pinta a peça INTEIRA de `NaN`.
///
/// ⚠️ **A régua é o `is_nan` e NÃO só a magnitude:** `NaN` falha toda comparação, logo um gate
/// escrito só com `<=` fica **verde sobre `NaN`** — foi assim que a 1.ª redacção deste teste o
/// apanhou por acidente, com a mensagem a dizer *«a luz de trás acendeu (NaN > 0.5)»*.
#[test]
fn uma_lampada_por_tras_nao_acende() {
    let s = superficie();
    let t = texel([0.0, 0.0, 1.0]);
    let tras = Lampada {
        para_a_luz: [0.0, 0.0, -1.0],
        radiancia: [1.0, 1.0, 1.0],
    };
    let r = acende_texel(&s, &t, &[tras], [0.0; 3]);
    for (c, (&aceso, &cru)) in r.iter().zip(&t.albedo).enumerate() {
        assert!(
            !aceso.is_nan(),
            "canal {c}: a lâmpada anti-paralela à vista devolveu NaN"
        );
        assert!(
            aceso <= cru + 1e-6,
            "canal {c}: a luz de trás acendeu ({aceso} > {cru})"
        );
    }

    // ⭐ **O CONTROLO**: a `1e-3` de distância do caso degenerado a lei já responde (e responde
    // ~zero), logo a cerca não está a engolir um regime inteiro — ela corta um ponto.
    let quase = Lampada {
        para_a_luz: normaliza([0.001, 0.0, -1.0]).unwrap(),
        radiancia: [1.0, 1.0, 1.0],
    };
    for (c, &aceso) in acende_texel(&s, &t, &[quase], [0.0; 3]).iter().enumerate() {
        assert!(
            !aceso.is_nan(),
            "controlo: a quase-anti-paralela não pode dar NaN (canal {c})"
        );
    }
}

#[test]
#[ignore = "diagnostico temporario"]
fn diag_onde_nasce_o_nan() {
    let s = superficie();
    for (nome, tl) in [
        ("anti-paralela", [0.0f32, 0.0, -1.0]),
        ("quase-anti", [0.001, 0.0, -1.0]),
        ("lateral", [1.0, 0.0, 0.0]),
        ("obliqua-tras", [0.0, 0.6, -0.8]),
        ("frente", [0.0, 0.0, 1.0]),
    ] {
        let r = s.direct([0.0, 0.0, 1.0], VISTA, tl, [1.0, 1.0, 1.0]);
        println!("  {nome:16} -> {r:?}  nan={}", r[0].is_nan());
    }
    // a normal virada, para o controlo do outro gate
    let virada = s.direct([0.0, 0.0, -1.0], VISTA, [0.0, 0.0, 1.0], [1.0; 3]);
    let frente = s.direct([0.0, 0.0, 1.0], VISTA, [0.0, 0.0, 1.0], [1.0; 3]);
    println!("  normal virada  -> {virada:?}");
    println!("  normal frente  -> {frente:?}");
}

/// ⏱️ **QUANTO CUSTA ACENDER UM SPRITE INTEIRO NA CPU** — a medição que decide se o passe de
/// dispositivo é obrigatório ou optimização (§0.0: medir antes de limitar).
///
/// ⚠️ **Corre em `--release`**, e a razão está medida noutras linhas desta casa: em `debug` a mesma
/// lei lê `~20×` mais lento, e um tecto tirado dali seria um tecto sobre outro programa.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-form-pbr --release \
///     custa_acender_um_sprite -- --ignored --nocapture
/// ```
///
/// A barra que interessa: a promessa escrita no `relight_stale` é que mover a lâmpada seja um
/// **gesto contínuo** — `16,7 ms` por quadro. Um custo acima disso num tamanho que o artista usa
/// diz que o dispositivo não é optimização, é a condição de a feature existir.
///
/// # ⭐⭐⭐ O que ele MEDIU (2026-09-20, `load 3,3`, `--release`, 32 núcleos)
///
/// ```text
///  lado    texels      ms   ms/Mtexel   par ms  ganho
///   256     65536     7,2      110,1      0,8    8,7x
///   512    262144    28,5      108,6      2,5   11,3x
///  1024   1048576   111,0      105,9      9,8   11,4x
///  2048   4194304   443,5      105,7     39,2   11,3x
///
///  a 1024², em paralelo, por numero de lampadas:
///   1 lampada    11,1 ms     3 lampadas   25,3 ms
///   2 lampadas   18,3 ms     4 lampadas   34,1 ms
/// ```
///
/// ⭐⭐⭐ **O VEREDITO: o passe de dispositivo NÃO é optimização — é a condição de a re-acendida
/// continuar a ser um gesto contínuo.** A `1024²` a CPU paralela atravessa o orçamento de um quadro
/// **à SEGUNDA lâmpada**, e o rig permite quatro; a `2048²` ela estoura com uma só.
///
/// ⚠️⚠️ **E a medição em PARALELO é que torna esse veredito honesto.** Com o número de UM núcleo
/// (`111 ms` a `1024²`, `6,6×` um quadro) eu teria escrito a mesma conclusão **pela razão errada**,
/// e o §0.0 chama a isso deixar o caminho lento definir o produto. A margem real não é `6,6×`: são
/// **duas lâmpadas**, e é um número que outra pessoa pode mudar — quem puser esta lei numa máquina
/// com mais núcleos, ou quem a cozinhar por tiles, **tem de reconferir esta nota**.
///
/// ⇒ a CPU fica como o que ela é em toda esta casa: o caminho de **REFERÊNCIA**, que só precisa de
/// computar a mesma resposta.
#[test]
#[ignore = "diagnostico: mede relogio, corre a' mao em --release"]
fn diag_quanto_custa_acender_um_sprite() {
    let s = superficie();
    let l = lampada_de_frente();
    println!("  lado    texels      ms   ms/Mtexel   par ms  ganho");
    for lado in [256u32, 512, 1024, 2048] {
        let n = (lado * lado) as usize;
        // Uma peça plausível: normais espalhadas, cobertura cheia no miolo.
        let texeis: Vec<Texel> = (0..n)
            .map(|i| {
                let a = (i % 997) as f32 / 997.0 - 0.5;
                let b = (i % 991) as f32 / 991.0 - 0.5;
                Texel {
                    normal: [a, b, 1.0 - (a * a + b * b)],
                    albedo: [0.5, 0.45, 0.4],
                    cobertura: if i % 8 == 0 { 0.0 } else { 1.0 },
                    oclusao: 0.8,
                }
            })
            .collect();

        let t0 = std::time::Instant::now();
        let mut soma = 0.0f64;
        for t in &texeis {
            // O `soma` existe para o optimizador não poder deitar o laço fora — um bench cujo
            // resultado ninguém lê mede a eliminação de código morto.
            soma += f64::from(acende_texel(&s, t, &[l], [0.1; 3])[0]);
        }
        let ms = t0.elapsed().as_secs_f64() * 1e3;

        // ⚠️ **E o MESMO em paralelo** — sem isto o veredito sairia do caminho de UM núcleo, que é
        // exactamente o «deixar o fallback definir o produto» do §0.0, com o sinal trocado: eu
        // declararia o dispositivo obrigatório sem ter medido a CPU que a máquina tem.
        // ⛔ `std::thread::scope` e não `rayon`: isto é uma MEDIÇÃO, e uma dependência de
        // threading numa folha é uma costura que se decide com o número na mão, não antes dele.
        let nucleos = std::thread::available_parallelism().map_or(1, |n| n.get());
        let t1 = std::time::Instant::now();
        let par: f64 = std::thread::scope(|sc| {
            let fatias: Vec<_> = texeis
                .chunks((n / nucleos).max(1))
                .map(|f| {
                    sc.spawn(|| {
                        f.iter()
                            .map(|t| f64::from(acende_texel(&s, t, &[l], [0.1; 3])[0]))
                            .sum::<f64>()
                    })
                })
                .collect();
            fatias.into_iter().map(|h| h.join().unwrap()).sum()
        });
        let ms_par = t1.elapsed().as_secs_f64() * 1e3;
        // ⚠️ **O acumulador é `f64` dos DOIS lados, e o controlo reprovou sobre produto certo
        // até o ser:** somar `4,19 M` valores em `f32` em SÉRIE perde resolução (a soma chega a
        // `8,2e5`, onde um `f32` tem `~0,06` de passo), e as `32` somas parciais do lado paralelo
        // são MAIS exactas. *Uma soma em série de milhões de `f32` não é a referência de nada.*
        assert!(
            (par - soma).abs() < soma.abs() * 1e-9,
            "controlo: as duas leis visitaram os mesmos texels ({par} contra {soma})"
        );

        println!(
            "  {lado:>4}  {n:>8}  {ms:>6.1}  {:>9.1}  {ms_par:>7.1}  {:>5.1}x",
            ms / (n as f64 / 1e6),
            ms / ms_par,
        );
    }

    // ⚠️ **E a segunda metade da conta é o NÚMERO DE LÂMPADAS** — o termo directo é o que corre por
    // lâmpada, e o rig permite mais de uma. Medir só com uma responderia à pergunta mais fácil.
    println!("\n  a 1024², em paralelo, por numero de lampadas:");
    let n = 1024usize * 1024;
    let texeis: Vec<Texel> = (0..n)
        .map(|i| {
            let a = (i % 997) as f32 / 997.0 - 0.5;
            Texel {
                normal: [a, a, 1.0 - a * a],
                albedo: [0.5, 0.45, 0.4],
                cobertura: 1.0,
                oclusao: 0.8,
            }
        })
        .collect();
    let nucleos = std::thread::available_parallelism().map_or(1, |n| n.get());
    for k in 1..=4usize {
        let lampadas: Vec<Lampada> = (0..k)
            .map(|i| Lampada {
                para_a_luz: normaliza([i as f32 * 0.3 - 0.4, 0.2, 1.0]).unwrap(),
                radiancia: [1.0, 1.0, 1.0],
            })
            .collect();
        let t = std::time::Instant::now();
        std::thread::scope(|sc| {
            let fatias: Vec<_> = texeis
                .chunks((n / nucleos).max(1))
                .map(|f| {
                    let lampadas = &lampadas;
                    sc.spawn(move || {
                        f.iter()
                            .map(|t| f64::from(acende_texel(&s, t, lampadas, [0.1; 3])[0]))
                            .sum::<f64>()
                    })
                })
                .collect();
            let _: f64 = fatias.into_iter().map(|h| h.join().unwrap()).sum();
        });
        println!(
            "  {k} lampada(s): {:>6.1} ms",
            t.elapsed().as_secs_f64() * 1e3
        );
    }
}
