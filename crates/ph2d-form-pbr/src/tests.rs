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
