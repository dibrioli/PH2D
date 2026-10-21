use super::*;
use crate::canal_vetor as v;

fn k(campo: u16, i: u16, j: u16) -> FieldKey {
    FieldKey::new(familia::VETOR, campo, i, j)
}

/// ⛔ Os números são gravados no documento da timeline: mudar um é perder a animação de quem o
/// gravou. Este gate fixa-os; acrescentar é livre, renumerar reprova.
#[test]
fn o_vocabulario_do_vetor_nao_renumera() {
    assert_eq!(familia::VETOR, 1);
    let fixos = [
        (v::ANCORA_X, 1),
        (v::ANCORA_Y, 2),
        (v::ENTRADA_X, 3),
        (v::ENTRADA_Y, 4),
        (v::SAIDA_X, 5),
        (v::SAIDA_Y, 6),
        (v::CANTO, 7),
        (v::TIPO_VERTICE, 8),
        (v::FECHADO, 9),
        (v::REGRA, 10),
        (v::TINTA_PREENCH, 20),
        (v::TINTA_TRACO, 60),
        (v::OFF_EXISTE, 0),
        (v::OFF_TIPO, 1),
        (v::OFF_SOLIDA, 2),
        (v::OFF_PARAGEM, 6),
        (v::OFF_N_PARAGENS, 11),
        (v::OFF_LINEAR, 12),
        (v::OFF_RADIAL, 16),
        (v::OFF_PONTO, 19),
        (v::OFF_N_PONTOS, 27),
        (v::OFF_PADRAO, 28),
        (v::OFF_PADRAO_MODOS, 36),
        (v::OFF_PADRAO_DESFASE, 38),
        (v::TRACO_LARGURA, 100),
        (v::TRACO_PONTA, 101),
        (v::TRACO_JUNCAO, 102),
        (v::TRACO_ALINHAMENTO, 103),
        (v::TRACO_TRACEJADO, 104),
        (v::TRACO_TRACO, 105),
        (v::TRACO_VAO, 106),
        (v::TRACO_MARCA_INICIO, 107),
        (v::TRACO_MARCA_FIM, 108),
        (v::TRACO_ESCALA_INICIO, 109),
        (v::TRACO_ESCALA_FIM, 110),
        (v::TRACO_MARCA_REDONDA, 111),
        (v::MISTURA, 140),
        (v::PILHA_LIGADA, 150),
        (v::PILHA_OPACIDADE, 151),
        (v::PILHA_MISTURA, 152),
        (v::PILHA_DESLOCA_X, 153),
        (v::PILHA_DESLOCA_Y, 154),
        (v::PILHA_DILATA, 155),
        (v::PILHA_DILATA_JUNCAO, 156),
        (v::EFEITO_LIGADO, 170),
        (v::EFEITO_PARAM, 171),
        (v::PARAM_LARGURA, 180),
        (v::PARAM_ALTURA, 181),
        (v::PARAM_VALOR, 182),
        (v::TEXTO_TAMANHO, 190),
        (v::TEXTO_PESO, 191),
        (v::TEXTO_ENTRELINHA, 192),
        (v::TEXTO_TRACKING, 193),
        (v::TEXTO_ALINHAMENTO, 194),
        (v::TEXTO_QUEBRA, 195),
        (v::TEXTO_EIXO, 196),
        (v::CONTORNO_PASSOS, 200),
        (v::CONTORNO_DISTANCIA, 201),
        (v::CONTORNO_ACEL, 202),
        (v::CONTORNO_JUNCAO, 203),
        (v::CONTORNO_LADO, 204),
        (v::CONTORNO_COR, 205),
        (v::OFFSET_DISTANCIA, 210),
        (v::OFFSET_JUNCAO, 211),
        (v::OFFSET_LADO, 212),
        (v::PINCEL_ESPACO, 112),
        (v::PINCEL_ESPELHO, 116),
        (v::TEXTO_QUEBRA_FIXA, 197),
        (v::TEXTO_FAMILIA, 198),
        (v::TEXTO_CONTEUDO, 199),
        (v::FILTRO_TIPO, 220),
        (v::FILTRO_LIGADO, 221),
        (v::FILTRO_COR, 225),
        (v::FILTRO_COR_B, 229),
        (v::FILTRO_PARAGEM, 244),
        (v::PERFIL_EXISTE, 250),
        (v::PERFIL_MULT, 253),
        (v::PADRAO_CAMINHO_INICIO, 260),
        (v::PADRAO_CAMINHO_ROTACAO, 265),
        (v::TEXTO_CAMINHO_INICIO, 270),
        (v::BLEND_PASSOS, 280),
        (v::ENVELOPE_CANTO_X, 290),
        (v::ENVELOPE_PINO, 297),
        (v::ENVELOPE_CURVA, 301),
        (v::ANCORAS_EXISTE, 310),
        (v::ANCORA_BASE, 315),
        (v::MOLDURA_RECORTA, 320),
        (v::LAYOUT_DIR, 330),
        (v::LAYOUT_MARGEM, 333),
        (v::LAYOUT_ABSOLUTO, 349),
        (v::WIDGET_PELE, 350),
    ];
    for (n, (atual, gravado)) in fixos.iter().enumerate() {
        assert_eq!(
            atual, gravado,
            "a constante nº {n} do vocabulário foi renumerada"
        );
    }
    // Os blocos de tinta não se sobrepõem — provado ao COMPILAR.
    const { assert!(v::TINTA_PREENCH + v::TINTA_LARGURA <= v::TINTA_TRACO) };
    const { assert!(v::TINTA_TRACO + v::TINTA_LARGURA <= v::TRACO_LARGURA) };
}

/// Todo campo declarado tem um rótulo e um grupo; nenhum buraco no meio de um bloco de tinta.
#[test]
fn todo_campo_da_tinta_tem_informacao() {
    for base in [v::TINTA_PREENCH, v::TINTA_TRACO] {
        for off in 0..v::TINTA_LARGURA {
            let info = info(k(base + off, 0, 0)).expect("campo de tinta sem informação");
            assert_eq!(info.grupo, base, "a tinta inteira é UM grupo");
            assert!(info.por_i, "cada entrada da pilha tem a sua tinta");
        }
        assert!(info(k(base + v::TINTA_LARGURA, 0, 0)).is_none() || base == v::TINTA_PREENCH);
    }
    let tipo = info(k(v::TINTA_PREENCH + v::OFF_TIPO, 0, 0)).unwrap();
    assert!(
        tipo.discreto,
        "trocar sólido por gradiente não se interpola"
    );
    let r = info(k(v::TINTA_PREENCH + v::OFF_SOLIDA, 0, 0)).unwrap();
    assert!(
        !r.discreto,
        "a cor passa aos poucos (lei do AE/Rive, Godot medido)"
    );
}

/// A forma inteira é UM grupo, através dos contornos; a tinta parte-se por entrada da pilha.
#[test]
fn o_grupo_e_a_chave_inteira() {
    let a = k(v::ANCORA_X, 0, 3);
    assert!(
        a.mesmo_grupo(k(v::SAIDA_Y, 2, 0)),
        "outro contorno, mesma forma"
    );
    assert!(
        a.mesmo_grupo(k(v::TIPO_VERTICE, 0, 1)),
        "o tipo do vértice vai na chave da forma"
    );
    assert!(!a.mesmo_grupo(k(v::TRACO_LARGURA, 0, 0)));
    let base = k(v::TINTA_PREENCH + v::OFF_SOLIDA, 0, 0);
    assert!(base.mesmo_grupo(k(v::TINTA_PREENCH + v::OFF_SOLIDA + 3, 0, 0)));
    assert!(
        !base.mesmo_grupo(k(v::TINTA_PREENCH + v::OFF_SOLIDA, 1, 0)),
        "a entrada 0 da pilha"
    );
    assert!(
        !base.mesmo_grupo(k(v::TINTA_TRACO + v::OFF_SOLIDA, 0, 0)),
        "a cor do traço"
    );
}

#[test]
fn esquecer_tira_so_a_entidade() {
    let mut l = CanaisLidos::default();
    l.publicar(7, k(v::ANCORA_X, 0, 0), 1.0);
    l.publicar(7, k(v::ANCORA_X, 0, 1), 2.0);
    l.publicar(8, k(v::ANCORA_X, 0, 0), 3.0);
    l.publicar(u64::MAX, k(v::ANCORA_X, 0, 0), 4.0);
    assert_eq!(l.campos(7).count(), 2);
    let outra = FieldKey::new(familia::VETOR + 1, 1, 0, 0);
    l.publicar(7, outra, 5.0);
    l.esquecer(7, familia::VETOR);
    assert_eq!(
        l.campos(7).collect::<Vec<_>>(),
        [(outra, 5.0)],
        "as outras famílias ficam"
    );
    assert_eq!(l.valor(8, k(v::ANCORA_X, 0, 0)), Some(3.0));
    l.esquecer(u64::MAX, familia::VETOR);
    l.esquecer(7, familia::VETOR + 1);
    assert_eq!(l.len(), 1);
}

#[test]
fn drenar_deixa_as_outras_familias() {
    let mut e = CanaisEscritos::default();
    e.escrever(1, k(v::ANCORA_X, 0, 0), 5.0);
    e.escrever(1, FieldKey::new(99, 1, 0, 0), 6.0);
    assert_eq!(
        e.drenar(familia::VETOR),
        vec![(1, k(v::ANCORA_X, 0, 0), 5.0)]
    );
    assert!(
        !e.is_empty(),
        "a escrita da família 99 fica para quem a drena"
    );
}

/// ⭐⭐ O eco: a curva escreve 127,5 numa cor `u8`, o documento fica 128 — a leitura devolve 127,5
/// (nada mudou); a mão põe 130 — a leitura devolve 130 (mudou). Sem o eco a chave fantasma nascia.
#[test]
fn o_eco_esconde_a_quantizacao_e_mostra_a_mao() {
    let mut l = CanaisLidos::default();
    let cor = k(v::TINTA_PREENCH + v::OFF_SOLIDA, 0, 0);
    l.eco(3, cor, 127.5, 128.0);
    l.publicar(3, cor, 128.0);
    assert_eq!(
        l.valor(3, cor),
        Some(127.5),
        "o documento é a curva quantizada"
    );
    l.publicar(3, cor, 130.0);
    assert_eq!(l.valor(3, cor), Some(130.0), "a mão mudou o documento");
    l.publicar(4, cor, 128.0);
    assert_eq!(l.valor(4, cor), Some(128.0), "o eco é por entidade");
}

/// O texto entra no canal por um índice estável: o mesmo texto, o mesmo número; `0` é nenhum.
#[test]
fn o_texto_e_um_indice_estavel() {
    let mut t = TextosDosCanais::default();
    assert_eq!(t.indice(None), 0.0);
    let a = t.indice(Some("Inter"));
    let b = t.indice(Some("Lato"));
    assert_eq!((a, b), (1.0, 2.0));
    assert_eq!(t.indice(Some("Inter")), a);
    assert_eq!(t.texto(b), Some("Lato"));
    assert_eq!(t.texto(0.0), None);
    assert_eq!(t.texto(9.0), None);
    let gravada = TextosDosCanais::de(t.textos().to_vec());
    assert_eq!(gravada.texto(a), Some("Inter"));
    assert!(info(k(v::TEXTO_FAMILIA, 0, 0)).is_some_and(|c| c.texto && c.discreto));
    assert!(info(k(v::TEXTO_TAMANHO, 0, 0)).is_some_and(|c| !c.texto));
}
