use super::*;

#[test]
fn o_fnv1a_e_o_publicado() {
    // Os vectores de referência do FNV-1a de 64 bits.
    assert_eq!(fnv1a(FNV_INICIO, b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a(FNV_INICIO, b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(fnv1a(FNV_INICIO, b"foobar"), 0x8594_4171_f739_67e8);
}

#[test]
fn o_disco_liga_os_centros_dentro_do_raio_e_a_caixa_contem_os() {
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
    assert_eq!(m.caixa(), Some((2, 2, 4, 4)));
    assert_ne!(m.versao(), v0, "marcar muda a versão");
    // Fora da tela: nada.
    let v1 = m.versao();
    m.marca_disco(-9.0, -9.0, 2.0);
    assert_eq!(m.versao(), v1);
}

#[test]
fn o_delta_liga_so_o_que_era_novo_e_o_desfazer_devolve_a_marca_de_antes() {
    let mut m = MarcaDoArtista::nova(8, 8, 3);
    m.marca_disco(2.0, 2.0, 1.5);
    let antes = m.palavras().as_ref().clone();
    let mut bits = vec![0u64; 1];
    for i in [0usize, 9, 40, 63] {
        liga(&mut bits, i);
    }
    let d = m.une_bits(&bits);
    for (i, b) in d.palavras.iter().map(|p| p.1).enumerate() {
        assert_eq!(b & antes[i], 0, "o delta só leva bits que estavam desligados");
    }
    m.aplica(&d, false);
    assert_eq!(*m.palavras().as_ref(), antes, "desfazer devolve a marca de antes");
    m.aplica(&d, true);
    assert!(m.marcado(63) && m.marcado(40));
    // Outra época não se aplica.
    let mut outra = MarcaDoArtista::nova(8, 8, 4);
    outra.aplica(&d, true);
    assert!(outra.vazia());
}

#[test]
fn o_formato_gravado_volta_igual() {
    let mut m = MarcaDoArtista::nova(300, 200, 0xdead_beef);
    m.marca_disco(10.0, 10.0, 6.0);
    m.marca_disco(250.0, 180.0, 9.0);
    let b = m.codifica();
    let l = MarcaDoArtista::descodifica(&b).expect("lê-se");
    assert_eq!(l.palavras(), m.palavras());
    assert_eq!((l.tamanho(), l.epoca()), (m.tamanho(), m.epoca()));
    assert_eq!(l.caixa(), m.caixa());
    let cru = m.palavras().len() * 8;
    assert!(b.len() * 8 < cru, "{} de {cru} bytes: as corridas de zeros não se gravam", b.len());
    assert!(MarcaDoArtista::descodifica(&b[..b.len() - 3]).is_none());
    let vazia = MarcaDoArtista::nova(1024, 1024, 1).codifica();
    assert_eq!(MarcaDoArtista::descodifica(&vazia).map(|m| m.vazia()), Some(true));
}
