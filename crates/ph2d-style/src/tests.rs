//! Os gates da camada de estilo.
//!
//! ⚠️ **A régua nº 1 é a IDENTIDADE AO BIT no ponto de fábrica**, medida por varredura sobre luzes e
//! geometrias — e ⛔ ela **não** prova a FORMA das expressões, coisa que esta linha chegou a
//! escrever no cabeçalho da crate e uma mutação sobrevivente refutou.
//!
//! ⚠️ **A régua nº 2 é a do §5.0 do `CLAUDE.md`** — *nenhum instrumento deste repo pergunta se o
//! VALOR chega a um consumidor*. Aqui ela é barata e obrigatória: **cada botão tem um gate que o
//! move e mede a diferença**, porque um knob desta crate que não mudasse a saída seria um controlo
//! morto com a lei toda escrita à volta dele.

use super::*;

/// Uma varredura de luzes de cena que cobre o preto, o miolo e o HDR.
const LUZES: [[f32; 3]; 7] = [
    [0.0, 0.0, 0.0],
    [0.18, 0.18, 0.18],
    [1.0, 1.0, 1.0],
    [0.02, 0.5, 0.97],
    [7.5, 0.3, 0.05],
    [0.0, 0.0, 12.0],
    [120.0, 80.0, 40.0],
];

/// Geometrias que cobrem a silhueta, a frente, a aresta, a cova e o plano.
const PONTOS: [Point; 9] = [
    Point {
        facing: 0.0,
        curvature: 0.0,
    },
    Point {
        facing: 0.5,
        curvature: 0.0,
    },
    Point {
        facing: 1.0,
        curvature: 0.0,
    },
    Point {
        facing: 0.25,
        curvature: 1.0,
    },
    Point {
        facing: 0.25,
        curvature: -1.0,
    },
    Point {
        facing: 0.8,
        curvature: 12.0,
    },
    Point {
        facing: 0.8,
        curvature: -12.0,
    },
    Point {
        facing: 0.13,
        curvature: 0.37,
    },
    Point {
        facing: 0.97,
        curvature: -0.04,
    },
];

pub(crate) fn bits(v: [f32; 3]) -> [u32; 3] {
    v.map(f32::to_bits)
}

/// ⭐⭐⭐ **O PONTO DE FÁBRICA É A IDENTIDADE, AO BIT** — a afirmação de que todo o resto depende.
///
/// ⚠️ **Por VARREDURA e não por um caso** — a afirmação é sobre toda luz e toda geometria, e uma
/// perda de bits que só apareça numa combinação passaria num gate de um caso só.
///
/// ⚠️⚠️ **E ela NÃO prova a forma das expressões** — duas mutações que trocavam a redacção das
/// tintas sobreviveram-lhe, e o que ficou no lugar dessa promessa é o
/// [`a_forma_que_reconstroi_por_diferenca_e_a_que_perde_bits`], com a varredura que refutou a
/// premissa e o controlo positivo do perigo verdadeiro.
#[test]
fn a_fabrica_e_a_identidade_ao_bit() {
    let s = Style::default();
    for luz in LUZES {
        for p in PONTOS {
            assert_eq!(
                bits(s.apply(luz, p)),
                bits(luz),
                "apply mexeu no ponto de fábrica: {luz:?} em {p:?}"
            );
        }
        assert_eq!(
            bits(s.saturate_indirect(luz)),
            bits(luz),
            "saturate_indirect mexeu no ponto de fábrica: {luz:?}"
        );
    }
}

/// ⭐⭐ **E ela sobrevive à PORTA** — senão o saneamento seria a segunda maneira de a perder.
#[test]
fn o_ponto_de_fabrica_e_ponto_fixo_da_porta() {
    assert_eq!(Style::default().sanitized(), Style::default());
}

/// A porta é idempotente — aplicá-la duas vezes é aplicá-la uma.
#[test]
fn a_porta_e_idempotente() {
    let sujo = Style {
        rim: Rim {
            color: [f32::NAN, 2.0, -1.0],
            strength: f32::INFINITY,
            width: 900.0,
        },
        curvature: Curvature {
            convex: [0.2, f32::NEG_INFINITY, 3.0],
            concave: [1.0, 1.0, f32::NAN],
            sharpness: f32::NAN,
        },
        zones: Zones {
            shadow: [0.4, 0.5, 0.6],
            highlight: [f32::NAN; 3],
            pivot: -5.0,
        },
        indirect_saturation: f32::NAN,
    };
    let uma = sujo.sanitized();
    assert_eq!(uma, uma.sanitized());
    // ⚠️ **A metade que diz o que a lei É**, e não só que ela estabiliza: o fallback é a FÁBRICA.
    let d = Style::default();
    assert!(
        (uma.rim.color[0] - d.rim.color[0]).abs() < f32::EPSILON,
        "cor ilegível → fábrica"
    );
    assert!(
        (uma.rim.strength - d.rim.strength).abs() < f32::EPSILON,
        "força ilegível → fábrica"
    );
    assert!(
        (uma.curvature.sharpness - d.curvature.sharpness).abs() < f32::EPSILON,
        "nitidez ilegível → fábrica"
    );
    assert!(
        (uma.indirect_saturation - d.indirect_saturation).abs() < f32::EPSILON,
        "saturação ilegível → fábrica"
    );
    // ⚠️ E os dois com DOMÍNIO próprio são apertados nele, não devolvidos à fábrica.
    assert!(
        (uma.rim.width - Rim::MAX_WIDTH).abs() < f32::EPSILON,
        "largura acima do tecto"
    );
    assert!(
        (uma.zones.pivot - Zones::MIN_PIVOT).abs() < f32::EPSILON,
        "pivô abaixo do piso"
    );
    // ⚠️ E o que é finito e legal ATRAVESSA — sem isto a porta podia limpar tudo e passar.
    assert!(
        (uma.zones.shadow[1] - 0.5).abs() < f32::EPSILON,
        "um número são passa intacto"
    );
    assert!(
        (uma.curvature.convex[0] - 0.2).abs() < f32::EPSILON,
        "uma tinta sã passa intacta"
    );
}

/// ⭐⭐⭐ **NADA DE `NaN` SAI DAQUI COM BOTÕES ILEGÍVEIS** — a lei da porta, medida na SAÍDA.
///
/// ⚠️ E ela mede a saída porque *uma porta que limpa a struct e uma lei que sobrevive ao lixo leem-se
/// igual num gate sobre a struct*.
#[test]
fn com_os_botoes_ilegiveis_a_saida_continua_finita() {
    let sujo = Style {
        rim: Rim {
            color: [f32::NAN; 3],
            strength: f32::NAN,
            width: f32::NEG_INFINITY,
        },
        curvature: Curvature {
            convex: [f32::INFINITY; 3],
            concave: [f32::NAN; 3],
            sharpness: f32::INFINITY,
        },
        zones: Zones {
            shadow: [f32::NAN; 3],
            highlight: [f32::NAN; 3],
            pivot: f32::NAN,
        },
        indirect_saturation: f32::NAN,
    };
    let s = sujo.sanitized();
    for luz in LUZES {
        for p in PONTOS {
            let out = s.apply(luz, p);
            assert!(
                out.iter().all(|c| c.is_finite()),
                "apply devolveu {out:?} de {luz:?}"
            );
        }
        assert!(s.saturate_indirect(luz).iter().all(|c| c.is_finite()));
    }
    // ⚠️ **E a GEOMETRIA por pixel é a outra metade** — ela não passa pela porta, e uma normal
    // degenerada é a forma normal de ela chegar suja.
    let podre = Point {
        facing: f32::NAN,
        curvature: f32::NAN,
    };
    let out = Style::default().apply([0.5; 3], podre);
    assert_eq!(
        bits(out),
        bits([0.5; 3]),
        "uma geometria ilegível não pode mover a fábrica"
    );
}

/// Os pesos da luminância são os da porta canónica da casa — ver [`LUMA`].
///
/// ⚠️ Esta crate é FOLHA e declara-os; o gate é o que impede as duas cópias de divergirem.
#[test]
fn os_pesos_da_luminancia_sao_os_da_porta_canonica() {
    let canal = |i: usize| {
        let mut c = [0.0f32; 3];
        c[i] = 1.0;
        ph2d_color::linear::LinearRgba::opaque(c[0], c[1], c[2]).luminance()
    };
    for (i, peso) in LUMA.iter().enumerate() {
        assert!(
            (peso - canal(i)).abs() < f32::EPSILON,
            "o peso {i} diverge da ph2d_color: {peso} contra {}",
            canal(i)
        );
    }
}

/// ⭐⭐ **A ARRUMAÇÃO fecha nos dois sentidos** — ver [`wgsl::pack`].
///
/// ⚠️ A volta compara contra o estilo **SANEADO** e não contra o original, porque a ida é a porta do
/// dispositivo: *uma ida-e-volta que ignorasse isso estaria a afirmar que o `pack` não saneia.*
#[test]
fn a_arrumacao_fecha_nos_dois_sentidos() {
    let s = Style {
        rim: Rim {
            color: [0.1, 0.2, 0.3],
            strength: 0.44,
            width: 5.5,
        },
        curvature: Curvature {
            convex: [0.6, 0.7, 0.8],
            concave: [0.9, 1.1, 1.2],
            sharpness: 1.3,
        },
        zones: Zones {
            shadow: [1.4, 1.5, 1.6],
            highlight: [1.7, 1.8, 1.9],
            pivot: 2.0,
        },
        indirect_saturation: 2.1,
    };
    assert_eq!(wgsl::unpack(&wgsl::pack(&s)), s.sanitized());
    // ⚠️⚠️ **A metade SUJA, e ela nasceu de uma mutação SOBREVIVENTE:** com a fixtura acima (toda
    // ela sã) o `sanitized` do `pack` é a identidade, logo apagá-lo não movia um bit e o gate ficava
    // verde sobre um dispositivo a receber `NaN`. *Uma ida-e-volta medida só sobre entradas limpas
    // não afirma nada sobre a porta que limpa.*
    let sujo = Style {
        indirect_saturation: f32::NAN,
        ..s
    };
    let v = wgsl::pack(&sujo);
    assert!(
        v.iter().all(|c| c.is_finite()),
        "o pack deixou lixo chegar ao dispositivo: {v:?}"
    );
    assert_eq!(
        wgsl::unpack(&v),
        sujo.sanitized(),
        "o pack é a porta do dispositivo"
    );
    // ⚠️ **Os vinte números são DISTINTOS de propósito** — com dois iguais, uma troca de campos
    // ficaria invisível.
    let v = wgsl::pack(&s);
    for i in 0..v.len() {
        for j in (i + 1)..v.len() {
            assert!(
                (v[i] - v[j]).abs() > 1e-6,
                "a fixtura tem dois números iguais ({i}, {j}): ela não discrimina uma troca"
            );
        }
    }
    assert_eq!(v.len(), wgsl::PACKED);
}

/// ⭐ **O shader sai com as constantes SUBSTITUÍDAS** — uma marca por preencher compila a lixo.
#[test]
fn o_shader_nao_sai_com_marcas_por_preencher() {
    let src = wgsl::source();
    // ⚠️⚠️ **A 1.ª redacção era `!src.contains('{')` e reprovou de imediato — sobre um shader
    // CERTO.** WGSL tem chavetas em todo o corpo de função, logo aquilo media *«isto é código»* e
    // não *«ficou uma marca»*. A forma de uma marca é `{MAIÚSCULAS}`, e é ela que se varre.
    let marcas: Vec<&str> = src
        .match_indices('{')
        .filter_map(|(i, _)| {
            let resto = &src[i + 1..];
            let fim = resto.find('}')?;
            let dentro = &resto[..fim];
            let e_marca = !dentro.is_empty()
                && dentro
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit());
            e_marca.then(|| &src[i..=i + 1 + fim])
        })
        .collect();
    assert!(
        marcas.is_empty(),
        "ficaram marcas por substituir no WGSL: {marcas:?}"
    );
    for w in LUMA {
        assert!(
            src.contains(&format!("{w:?}")),
            "o peso {w} não chegou ao shader"
        );
    }
    // ⚠️ **E as duas entradas existem com o nome que o consumidor chama** — um `source()` que
    // compilasse sem elas seria um shader sem lei.
    assert!(src.contains("fn st_apply("), "falta a entrada st_apply");
    assert!(
        src.contains("fn st_saturate_indirect("),
        "falta a entrada st_saturate_indirect"
    );
}

/// ⛔⛔ **A LEI DO `mul_add`** — ver o cabeçalho da crate.
///
/// ⚠️ **Ela varre o FICHEIRO e não a API**, porque o que ela proíbe é uma forma de escrita: um
/// `mul_add` fundido de um lado e solto no WGSL divergiria num byte, um dia, num pixel — e nenhum
/// gate de valor o apanharia, porque os dois motores não correm no mesmo teste.
#[test]
fn nenhuma_conta_desta_crate_e_fundida() {
    for (nome, fonte) in [
        ("lib.rs", include_str!("lib.rs")),
        ("wgsl.rs", include_str!("wgsl.rs")),
    ] {
        // ⚠️ O cabeçalho NOMEIA a lei, logo a varredura tem de saltar a prosa — senão ela acusa o
        // texto que a explica, que é a forma nº 1 de um censo textual mentir.
        let codigo: String = fonte
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//") && !t.starts_with("/*") && !t.starts_with('*')
            })
            .collect();
        assert!(
            !codigo.contains("mul_add") && !codigo.contains("fma("),
            "{nome} tem uma conta FUNDIDA — ver a lei no cabeçalho da crate"
        );
    }
}

/// ⛔⛔⛔ **A PREMISSA DESTA CRATE FOI REFUTADA POR UMA MUTAÇÃO SOBREVIVENTE, e este gate é o que
/// ficou no lugar dela.**
///
/// O cabeçalho dizia que a forma ingénua `a·(1−w) + b·w` **não serve** porque `(1−w) + w` não é `1`
/// para todo `w` em `f32`. ⇒ **duas mutações que a instalavam nas tintas SOBREVIVERAM**, e a
/// varredura diz porquê: em **`2 044 824`** amostras de `f32` em `[0, 1]` e em `200 000` logo acima
/// de `1`, `(1−w) + w` deu **`1,0` exactamente, sempre** — como tinha de dar, porque o erro daquela
/// soma é no máximo **meia ULP** de `1` e o desempate é **para o par**, que é o próprio `1,0`.
///
/// ⭐⭐⭐ **O perigo é OUTRO, e a mutação que sangrou é quem o nomeia: é reconstruir `b` a partir de
/// `a + (b − a)` quando `a ≠ b`.** Nas tintas os dois extremos são iguais no ponto de fábrica (as
/// duas brancas), logo o parêntesis é **zero** e as duas formas são exactas. Na saturação da
/// indirecta os extremos são a **luminância** e o **rgb**, que são diferentes — e ali `l + (rgb − l)`
/// perde bits por cancelamento.
///
/// ⇒ a lei que fica é **multiplicar o valor que se quer de volta, nunca reconstruí-lo por
/// diferença** — e as tintas continuam escritas na forma robusta por ela ser exacta *seja qual for*
/// a faixa do peso, que é uma propriedade e não uma coincidência da faixa de hoje.
#[test]
fn a_forma_que_reconstroi_por_diferenca_e_a_que_perde_bits() {
    // (a) a varredura que refutou a premissa: com os extremos IGUAIS as duas formas são exactas.
    let (mut vistos, mut maus) = (0u64, 0u64);
    let mut bits_w = 0u32;
    while f32::from_bits(bits_w) <= 1.0 && bits_w <= 0x3f80_0000 {
        let w = f32::from_bits(bits_w);
        vistos += 1;
        if (1.0f32 - w) + w != 1.0 {
            maus += 1;
        }
        bits_w = bits_w.saturating_add(521);
    }
    assert!(
        vistos > 1_000_000,
        "a varredura tem de ter população: {vistos}"
    );
    assert_eq!(
        maus, 0,
        "a premissa do cabeçalho voltou a ser verdade — reescreva-o"
    );

    // (b) o CONTROLO POSITIVO: com os extremos DIFERENTES, reconstruir por diferença perde bits.
    // ⚠️ Sem esta metade o gate diria «as duas formas são iguais» e mandaria escrever a errada.
    let mut perdeu = 0u32;
    for i in 1..2000u32 {
        let rgb = 0.1 + f64::from(i) as f32 * 0.001_37;
        let l = rgb * 0.213_7 + 0.011;
        if l + (rgb - l) != rgb {
            perdeu += 1;
        }
    }
    assert!(
        perdeu > 0,
        "reconstruir por diferença tem de perder bits nalgum sítio — senão a lei não tem sujeito"
    );
}
