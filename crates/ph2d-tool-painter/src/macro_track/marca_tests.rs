use super::*;

#[test]
fn o_fnv1a_e_o_publicado() {
    // Os vectores de referência do FNV-1a de 64 bits.
    assert_eq!(fnv1a(FNV_INICIO, b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a(FNV_INICIO, b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(fnv1a(FNV_INICIO, b"foobar"), 0x8594_4171_f739_67e8);
}

#[test]
fn o_disco_cobre_os_centros_dentro_do_raio_e_a_caixa_contem_os() {
    let mut m = MarcaDoArtista::nova(16, 10, 7);
    assert!(m.vazia());
    let v0 = m.versao();
    m.marca_disco(4.0, 4.0, 2.0);
    let dentro: Vec<usize> = (0..160).filter(|&i| m.marcado(i)).collect();
    // A régua à parte: centro do texel a ≤ 2 de (4, 4).
    let esperado: Vec<usize> = (0..160)
        .filter(|&i| {
            let (x, y) = ((i % 16) as f32 + 0.5, (i / 16) as f32 + 0.5);
            (x - 4.0).powi(2) + (y - 4.0).powi(2) <= 4.0
        })
        .collect();
    assert_eq!(dentro, esperado);
    assert!(
        dentro.iter().all(|&i| m.cobertura(i) == CHEIA),
        "o disco é a imagem atual"
    );
    assert_eq!(m.caixa(), Some((2, 2, 4, 4)));
    assert_ne!(m.versao(), v0, "marcar muda a versão");
    // Fora da tela: nada.
    let v1 = m.versao();
    m.marca_disco(-9.0, -9.0, 2.0);
    assert_eq!(m.versao(), v1);
}

/// ⭐ **A cobertura compõe-se como a tinta e o que estava por baixo é o do 1.º toque** (W10): dois passos
/// de força ½ dão ¾; o 2.º não troca o `B`; um texel noutro ladrilho nasce com o seu.
#[test]
fn a_cobertura_compoe_por_cima_e_o_por_baixo_e_o_do_primeiro_toque() {
    let mut m = MarcaDoArtista::nova(200, 100, 1);
    let (i, j) = (5 * 200 + 7, 70 * 200 + 130);
    m.compoe_muitos([(i, 0.5, [10, 20, 30, 255]), (j, 0.25, [1, 2, 3, 255])]);
    assert_eq!(m.cobertura(i), compoe(0, 0.5));
    m.compoe_muitos([(i, 0.5, [99, 99, 99, 255])]);
    let tres_quartos = f32::from(m.cobertura(i)) / f32::from(CHEIA);
    assert!((tres_quartos - 0.75).abs() < 1e-4, "{tres_quartos}");
    assert_eq!(
        m.por_baixo(i),
        [10, 20, 30, 255],
        "o 2.º toque trocou o que estava por baixo"
    );
    assert_eq!(m.por_baixo(j), [1, 2, 3, 255]);
    assert_eq!(m.quantos(), 2);
    assert_eq!(m.caixa(), Some((7, 5, 124, 66)));
    assert_eq!(m.ladrilhos().len(), 2, "só os ladrilhos tocados vivem");
    // Força nula não toca; saturar fica cheio.
    m.compoe_muitos([(0, 0.0, [0; 4])]);
    assert!(!m.marcado(0));
    let mut bits = vec![0u64; palavras_para(200, 100)];
    liga(&mut bits, i);
    m.satura_bits(&bits);
    assert_eq!(m.cobertura(i), CHEIA);
}

#[test]
fn o_delta_devolve_os_ladrilhos_de_antes_e_o_refazer_os_de_depois() {
    let mut m = MarcaDoArtista::nova(130, 8, 3);
    m.marca_disco(2.0, 2.0, 1.5);
    let antes = Arc::clone(m.ladrilhos());
    m.compoe_muitos([(9, 0.5, [7; 4]), (2 * 130 + 100, 0.3, [8; 4])]);
    let d = DeltaDaMarca::entre(3, &antes, m.ladrilhos());
    assert_eq!(d.ladrilhos.len(), 2, "os dois ladrilhos tocados, e só eles");
    let depois = m.ladrilhos().as_ref().clone();
    m.aplica(&d, false);
    assert_eq!(*m.ladrilhos(), antes, "desfazer devolve a marca de antes");
    m.aplica(&d, true);
    assert_eq!(*m.ladrilhos().as_ref(), depois);
    // O passo coalescido vai do 1.º antes ao último depois.
    let meio = Arc::clone(m.ladrilhos());
    m.compoe_muitos([(9, 0.5, [0; 4])]);
    let mut junto = d.clone();
    junto.junta(&DeltaDaMarca::entre(3, &meio, m.ladrilhos()));
    let fim = m.ladrilhos().as_ref().clone();
    m.aplica(&junto, false);
    assert_eq!(*m.ladrilhos(), antes, "o run desfaz-se de uma vez");
    m.aplica(&junto, true);
    assert_eq!(*m.ladrilhos().as_ref(), fim);
    // Outra época não se aplica.
    let mut outra = MarcaDoArtista::nova(130, 8, 4);
    outra.aplica(&d, true);
    assert!(outra.vazia());
}

#[test]
fn o_formato_gravado_volta_igual_e_o_da_w9_abre_cheio() {
    let mut m = MarcaDoArtista::nova(300, 200, 0xdead_beef);
    m.marca_disco(10.0, 10.0, 6.0);
    m.compoe_muitos([
        (180 * 300 + 250, 0.4, [1, 2, 3, 200]),
        (181 * 300 + 250, 0.9, [4; 4]),
    ]);
    let b = m.codifica();
    let l = MarcaDoArtista::descodifica(&b).expect("lê-se");
    assert_eq!(l.ladrilhos(), m.ladrilhos());
    assert_eq!((l.tamanho(), l.epoca()), (m.tamanho(), m.epoca()));
    assert_eq!(l.caixa(), m.caixa());
    assert!(MarcaDoArtista::descodifica(&b[..b.len() - 3]).is_none());
    // Um cabeçalho forjado não aloca: acima do tecto do raster é recusado.
    let mut forjado = vec![FORMATO];
    forjado.extend_from_slice(&u32::MAX.to_le_bytes());
    forjado.extend_from_slice(&u32::MAX.to_le_bytes());
    forjado.extend_from_slice(&0u64.to_le_bytes());
    assert!(MarcaDoArtista::descodifica(&forjado).is_none());
    let vazia = MarcaDoArtista::nova(1024, 1024, 1).codifica();
    assert_eq!(
        MarcaDoArtista::descodifica(&vazia).map(|m| m.vazia()),
        Some(true)
    );
    // O FORMATO 1 (W9): um bit por texel, as corridas de zeros contadas — cada bit é `c` cheia.
    let mut w9 = vec![FORMATO_DOS_BITS];
    w9.extend_from_slice(&100u32.to_le_bytes());
    w9.extend_from_slice(&2u32.to_le_bytes());
    w9.extend_from_slice(&9u64.to_le_bytes());
    w9.extend_from_slice(&1u32.to_le_bytes()); // uma palavra de zeros
    w9.extend_from_slice(&1u32.to_le_bytes()); // uma cheia
    w9.extend_from_slice(&0b101u64.to_le_bytes());
    let l = MarcaDoArtista::descodifica(&w9).expect("o da W9 abre");
    let marcados: Vec<usize> = (0..200).filter(|&i| l.marcado(i)).collect();
    assert_eq!(marcados, [64, 66]);
    assert!(marcados.iter().all(|&i| l.cobertura(i) == CHEIA));
    assert_eq!(l.epoca(), 9);
}
