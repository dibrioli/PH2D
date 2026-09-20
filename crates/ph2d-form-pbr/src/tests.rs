//! Os gates da lei. ⚠️ Cada um traz o **controlo** ao lado: uma régua que não vê o fenómeno
//! acontecer não prova que ele não aconteceu.

use super::*;
use ph2d_view_transform::Look;

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
    let fora = acende_texel(
        &s,
        &t,
        &[lampada_de_frente()],
        Ceu::chapado([0.2; 3]),
        Look::default(),
    );
    assert_eq!(fora, t.albedo, "a cobertura 0 tem de devolver o albedo CRU");

    // ⭐ **O CONTROLO**: com cobertura cheia, a MESMA entrada tem de mover o pixel — senão este
    // gate ficaria verde sobre uma lei que não acende nada.
    t.cobertura = 1.0;
    let dentro = acende_texel(
        &s,
        &t,
        &[lampada_de_frente()],
        Ceu::chapado([0.2; 3]),
        Look::default(),
    );
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
        Ceu::chapado([0.0; 3]),
        Look::default(),
    );
    let curta = acende_texel(
        &s,
        &texel([0.0, 0.0, 0.97]),
        &[lampada_de_frente()],
        Ceu::chapado([0.0; 3]),
        Look::default(),
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
        Ceu::chapado([0.0; 3]),
        Look::default(),
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
        acende_texel(
            &s,
            &t,
            &[lampada_de_frente()],
            Ceu::chapado([0.2; 3]),
            Look::default()
        ),
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
    let a = acende_texel(
        &s,
        &aberto,
        &[lampada_de_frente()],
        Ceu::chapado([0.0; 3]),
        Look::default(),
    );
    let f = acende_texel(
        &s,
        &fechado,
        &[lampada_de_frente()],
        Ceu::chapado([0.0; 3]),
        Look::default(),
    );
    assert_eq!(a, f, "sem ambiente, a oclusão não tem o que pesar");

    // ⭐ **O CONTROLO**: COM ambiente, ela tem de morder — senão o gate acima ficaria verde sobre
    // uma oclusão que não faz nada em lado nenhum.
    let a2 = acende_texel(
        &s,
        &aberto,
        &[lampada_de_frente()],
        Ceu::chapado([0.5; 3]),
        Look::default(),
    );
    let f2 = acende_texel(
        &s,
        &fechado,
        &[lampada_de_frente()],
        Ceu::chapado([0.5; 3]),
        Look::default(),
    );
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
///
/// ⛔⛔ **A PREMISSA ANTERIOR DESTE GATE MORREU, e ele é que a matou.** Ele dizia, por escrito, *«o
/// laço tem de ser albedo × `Surface::direct`, ao bit»* — e essa composição está **errada com
/// número**: o `compose` não é linear no `base_color`, logo multiplicar depois **tinge o destaque
/// especular** (medido: razão `R/B` de `8,00` num texel vermelho, contra `1,50` da lei). A cor de
/// um texel é o `base_color` dele, e a porta que o faz vive onde a óptica mora
/// ([`Surface::at_base_color`]). *O diff mostra a morte da premissa.*
#[test]
fn a_optica_e_a_da_crate_da_lei_ao_bit() {
    let s = superficie();
    let t = texel([0.3, 0.2, 0.9]);
    let l = Lampada {
        para_a_luz: normaliza([0.4, 0.5, 0.75]).unwrap(),
        radiancia: [0.9, 0.8, 0.7],
    };

    let nosso = acende_texel(&s, &t, &[l], Ceu::chapado([0.0; 3]), Look::default());

    let n = normaliza(t.normal).unwrap();
    let esperado = s
        .at_base_color(t.albedo)
        .direct(n, VISTA, l.para_a_luz, l.radiancia);
    assert_eq!(
        nosso, esperado,
        "o laço tem de ser Surface::at_base_color(albedo).direct, ao bit"
    );

    // ⛔ **O CONTROLO que prova que a barra discrimina:** a lei REJEITADA (multiplicar depois) dá
    // outra coisa. Sem ele, um dia em que a `at_base_color` virasse um no-op este gate ficaria
    // verde sobre exactamente o defeito que ele existe para impedir.
    let d = s.direct(n, VISTA, l.para_a_luz, l.radiancia);
    let rejeitada = [t.albedo[0] * d[0], t.albedo[1] * d[1], t.albedo[2] * d[2]];
    assert_ne!(
        nosso, rejeitada,
        "controlo: as duas leis TÊM de diferir neste texel"
    );
}

/// Duas lâmpadas somam — e a soma é a das radiâncias, não um `max`.
///
/// # ⚠️ Porque a barra é a igualdade AO BIT, e não um epsilon
///
/// Com cobertura cheia e sem ambiente o resultado é `Σ direct`, logo duas lâmpadas iguais dão
/// **exactamente** o dobro: dobrar um `f32` não perde um bit. *Um epsilon aqui esconderia uma lei
/// que soma quase certo.*
///
/// ⛔⛔ **A PREMISSA MORREU UMA VEZ E RENASCEU MAIS ESTREITA.** A 1.ª redacção usava a lâmpada de
/// frente com radiância `1` e reprovou no dia em que a lei ganhou a VISTA: a soma dava `1,0909` e o
/// `ViewTransform::Standard` corta em `1` ⇒ `duas != uma × 2`. *A linearidade é da RADIÂNCIA, e
/// acima do branco quem manda é a vista* — que é precisamente o trabalho dela.
///
/// ⇒ o gate mede a soma **abaixo do branco** (com a radiância a um quinto), e a 2.ª metade afirma a
/// outra ponta: **acima dele a vista CORTA**, e é isso que impede alguém de ler a barra apertada da
/// 1.ª metade como *«a lei é linear em todo o lado»*.
#[test]
fn as_lampadas_somam() {
    let s = superficie();
    let t = texel([0.0, 0.0, 1.0]);
    let fraca = Lampada {
        para_a_luz: [0.0, 0.0, 1.0],
        radiancia: [0.2, 0.2, 0.2],
    };
    let uma = acende_texel(&s, &t, &[fraca], Ceu::chapado([0.0; 3]), Look::default());
    let duas = acende_texel(
        &s,
        &t,
        &[fraca, fraca],
        Ceu::chapado([0.0; 3]),
        Look::default(),
    );
    for c in 0..3 {
        assert!(
            duas[c] < 1.0,
            "premissa: as duas têm de caber abaixo do branco"
        );
        assert_eq!(
            duas[c],
            uma[c] * 2.0,
            "canal {c}: abaixo do branco, duas lâmpadas dão o dobro de uma, ao bit"
        );
    }

    // ⭐ **O CONTROLO**: uma lâmpada sozinha tem de mover o pixel, senão o dobro de zero passaria.
    assert_ne!(uma, t.albedo, "controlo: uma lâmpada tem de acender");

    // ⭐⭐ **A OUTRA PONTA**: acima do branco a VISTA corta, e a soma deixa de ser observável.
    let forte = Lampada {
        para_a_luz: [0.0, 0.0, 1.0],
        radiancia: [1.0; 3],
    };
    let a = acende_texel(&s, &t, &[forte], Ceu::chapado([0.0; 3]), Look::default());
    let b = acende_texel(
        &s,
        &t,
        &[forte, forte],
        Ceu::chapado([0.0; 3]),
        Look::default(),
    );
    assert!(
        b[0] < a[0] * 2.0 - 1e-3,
        "controlo: acima do branco a vista TEM de cortar ({} contra {})",
        b[0],
        a[0] * 2.0
    );
}

/// ⭐⭐⭐ **A LÂMPADA ANTI-PARALELA À VISTA NÃO ENVENENA A SOMA** — o achado desta crate.
///
/// Sem a cerca do [`meio_vector_degenera`] a lei do OpenPBR devolve `[NaN, NaN, NaN]` aqui, porque
/// o meio-vector `v + to_light` é o vector nulo. ⚠️ **No modelador isto tem medida nula** (a vista
/// é a do raio e varia por pixel); **num canvas 2D a [`VISTA`] é constante**, logo é uma
/// configuração que o artista escreve, e ela vale para a peça INTEIRA.
///
/// # ⛔⛔ Porque a régua NÃO pode ser `is_nan` sobre uma lâmpada sozinha
///
/// **A VISTA é uma REDE que esconde este defeito.** O `to_display` da [`ph2d_view_transform`]
/// sanitiza (*«luz sem sentido → luz nenhuma»*), logo um `NaN` que lhe chegue sai **preto** — e com
/// uma lâmpada degenerada sozinha e sem ambiente, preto é também o que a cerca produz. ⇒ a mutação
/// que apaga a cerca **SOBREVIVIA**, e o gate ficava verde sobre o defeito.
///
/// *A 1.ª redacção deste gate media exactamente isso, e só a prova de mutação o disse — depois de a
/// vista entrar. Uma régua escrita antes de a rede existir não sabe que passou a medir a rede.*
///
/// ⇒ a régua é a **CONTAMINAÇÃO**: uma lâmpada boa AO LADO da degenerada. Com a cerca, a boa
/// acende; sem ela, o `NaN` envenena a soma inteira e a peça fica preta.
#[test]
fn uma_lampada_por_tras_nao_envenena_a_soma() {
    let s = superficie();
    let t = texel([0.0, 0.0, 1.0]);
    let boa = lampada_de_frente();
    let tras = Lampada {
        para_a_luz: [0.0, 0.0, -1.0],
        radiancia: [1.0, 1.0, 1.0],
    };

    let so_a_boa = acende_texel(&s, &t, &[boa], Ceu::chapado([0.0; 3]), Look::default());
    let com_a_ma = acende_texel(
        &s,
        &t,
        &[boa, tras],
        Ceu::chapado([0.0; 3]),
        Look::default(),
    );

    // **A metade que a mutação mata:** a lâmpada boa tem de sobreviver à vizinha degenerada.
    assert_eq!(
        com_a_ma, so_a_boa,
        "a lâmpada anti-paralela tem de ser SALTADA, não somada — ela envenenou a soma"
    );
    for (c, &v) in com_a_ma.iter().enumerate() {
        assert!(!v.is_nan(), "canal {c}: saiu NaN");
    }

    // ⭐ **O CONTROLO da fixtura:** a lâmpada boa de facto acende — senão o `assert_eq` acima seria
    // entre dois pretos, e passaria com a cerca apagada.
    assert!(
        so_a_boa[0] > 0.05,
        "controlo: a lâmpada boa tem de acender ({})",
        so_a_boa[0]
    );

    // ⭐ **E a 2.ª ponta:** a `1e-3` do caso degenerado a lei já responde, logo a cerca corta um
    // PONTO e não um regime.
    let quase = Lampada {
        para_a_luz: normaliza([0.001, 0.0, -1.0]).unwrap(),
        radiancia: [1.0, 1.0, 1.0],
    };
    let com_a_quase = acende_texel(
        &s,
        &t,
        &[boa, quase],
        Ceu::chapado([0.0; 3]),
        Look::default(),
    );
    for (c, &v) in com_a_quase.iter().enumerate() {
        assert!(
            !v.is_nan(),
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
/// # ⭐⭐⭐ O que ele MEDIU (2026-09-20, `--release`, 32 núcleos, **87–98 % de CPU ociosa**)
///
/// ⚠️ **A ociosidade REAL, medida por `vmstat`, e não o `loadavg`** — ele mente a decair: a primeira
/// corrida desta tabela saiu com `load 8,9` sobre uma máquina que o `vmstat` lia a `97 %` ociosa,
/// e a leitura oposta (aceitar um `loadavg` baixo herdado) é a que esta casa já pagou.
///
/// ```text
///  lado    texels      ms   ms/Mtexel   par ms  ganho
///   256     65536     7,4      112,8      0,9    7,8x
///   512    262144    28,7      109,4      2,5   11,4x
///  1024   1048576   113,6      108,3      9,3–10,7  ~11x
///  2048   4194304   457,5      109,1     39,0     ~12x
///
///  a 1024², em paralelo, por numero de lampadas (tres corridas):
///   1 lampada    9,3 · 10,5 · 10,3 ms      3 lampadas  29,2 · 24,0 · 29,2 ms
///   2 lampadas  17,0 · 16,3 · 21,0 ms      4 lampadas  31,7 · 37,4 · 31,7 ms
/// ```
///
/// ⭐ **E a cura da cor do texel custou `~3 %`** (`105,9 → 108,3` ms/Mtexel): pôr o albedo como
/// `base_color` em vez de o multiplicar no fim é, sem verniz, uma troca de campo — o número
/// confirma o argumento em vez de o substituir.
///
/// ⭐⭐⭐ **O VEREDITO: o passe de dispositivo NÃO é optimização — é a condição de a re-acendida
/// continuar a ser um gesto contínuo.** A `1024²` a CPU paralela atravessa o orçamento de um quadro
/// (`16,7 ms`) **à SEGUNDA lâmpada**, e o rig permite quatro; a `2048²` ela estoura com uma só.
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
            soma +=
                f64::from(acende_texel(&s, t, &[l], Ceu::chapado([0.1; 3]), Look::default())[0]);
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
                            .map(|t| {
                                f64::from(
                                    acende_texel(
                                        &s,
                                        t,
                                        &[l],
                                        Ceu::chapado([0.1; 3]),
                                        Look::default(),
                                    )[0],
                                )
                            })
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
                            .map(|t| {
                                f64::from(
                                    acende_texel(
                                        &s,
                                        t,
                                        lampadas,
                                        Ceu::chapado([0.1; 3]),
                                        Look::default(),
                                    )[0],
                                )
                            })
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
/// ⛔⛔⛔ **A SONDA QUE REFUTOU A 1.ª REDACÇÃO DESTA LEI** — multiplicar o albedo no fim TINGE o
/// destaque especular.
///
/// O `compose` do OpenPBR **não é linear no `base_color`**: só o lóbulo difuso escala com ele, e o
/// especular não escala nada. ⇒ `albedo × direct(…)` dá a um plástico vermelho um destaque
/// **vermelho**, que é o que um METAL faz.
///
/// ```text
///   albedo              (A) multiplicar depois      (B) base_color = albedo    razao R/B
///   [0,80 0,10 0,10]   [0,5096 0,0637 0,0637]   [0,6370 0,4251 0,4251]   A 8,00  B 1,50
///   [0,10 0,80 0,10]   [0,0637 0,5096 0,0637]   [0,4251 0,6370 0,4251]   A 1,00  B 1,00
///   [0,05 0,05 0,90]   [0,0319 0,0319 0,5733]   [0,4099 0,4099 0,6673]   A 0,06  B 0,61
///   [0,80 0,80 0,80]   [0,5096 0,5096 0,5096]   [0,6370 0,6370 0,6370]   A 1,00  B 1,00
/// ```
///
/// ⚠️ **A linha CINZENTA é o controlo**, e ela diz porque o defeito passou despercebido: num albedo
/// sem matiz as duas leis dão a MESMA razão entre canais. *Uma régua corrida só em cinzento aprova
/// as duas.*
///
/// ⇒ a cura é [`ph2d_material::Surface::at_base_color`], e o gate que a guarda é o
/// [`a_optica_e_a_da_crate_da_lei_ao_bit`], cuja premissa esta sonda matou.
#[test]
#[ignore = "diagnostico: a medicao que escolheu a lei; corre a' mao"]
fn diag_o_albedo_multiplicado_contra_o_albedo_como_base_color() {
    use ph2d_material::OpenPbr;
    const VISTA: [f32; 3] = [0.0, 0.0, 1.0];
    // Uma lâmpada BRANCA perto do espelho — é lá que o destaque vive.
    let luz = {
        let v = [0.25f32, 0.0, 0.968_246_3];
        let q = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / q, v[1] / q, v[2] / q]
    };
    let n = [0.13f32, 0.0, 0.991_5]; // quase a bissectriz ⇒ destaque forte
    let padrao = OpenPbr::default().prepare();

    println!(
        "  albedo                 (A) albedo x direct        (B) base_color = albedo      razao R/B"
    );
    for albedo in [
        [0.8f32, 0.1, 0.1],
        [0.1, 0.8, 0.1],
        [0.05, 0.05, 0.9],
        [0.8, 0.8, 0.8],
    ] {
        let d = padrao.direct(n, VISTA, luz, [1.0; 3]);
        let a = [albedo[0] * d[0], albedo[1] * d[1], albedo[2] * d[2]];
        let b = OpenPbr {
            base_color: albedo,
            ..OpenPbr::default()
        }
        .prepare()
        .direct(n, VISTA, luz, [1.0; 3]);
        println!(
            "  [{:.2} {:.2} {:.2}]   [{:.4} {:.4} {:.4}]   [{:.4} {:.4} {:.4}]   A {:.2}  B {:.2}",
            albedo[0],
            albedo[1],
            albedo[2],
            a[0],
            a[1],
            a[2],
            b[0],
            b[1],
            b[2],
            a[0] / a[2].max(1e-9),
            b[0] / b[2].max(1e-9),
        );
    }
}
