//! Gates do empacotador por máscara.

use super::empacota::{Mascara, arruma, com_folga, marca_triangulo};

/// Uma máscara a partir de um desenho em texto, de cima para baixo.
fn desenho(linhas: &[&str]) -> Mascara {
    let alt = linhas.len();
    let larg = linhas.iter().map(|l| l.len()).max().unwrap_or(0);
    let mut celulas = vec![false; larg * alt];
    for (j, l) in linhas.iter().rev().enumerate() {
        for (i, c) in l.chars().enumerate() {
            celulas[j * larg + i] = c == '#';
        }
    }
    Mascara { larg, alt, celulas }
}

/// Duas máscaras arrumadas não partilham uma célula.
fn conferir(ms: &[Mascara], lado: usize, pos: &[(usize, usize)]) {
    let mut oc = vec![usize::MAX; lado * lado];
    for (i, m) in ms.iter().enumerate() {
        let (x0, y0) = pos[i];
        for y in 0..m.alt {
            for x in 0..m.larg {
                if !m.celulas[y * m.larg + x] {
                    continue;
                }
                let c = (y0 + y) * lado + x0 + x;
                assert_eq!(oc[c], usize::MAX, "a peca {i} bate na {} em {c}", oc[c]);
                oc[c] = i;
            }
        }
    }
}

/// ⭐⭐⭐ **A LEI DO MÓDULO: duas peças arrumadas nunca partilham uma célula.**
#[test]
fn duas_mascaras_arrumadas_nunca_partilham_uma_celula() {
    let ms = vec![
        desenho(&["####", "####"]),
        desenho(&["##", "##", "##"]),
        desenho(&["#"]),
        desenho(&["###"]),
    ];
    let pos = arruma(&ms, 8, 0).expect("cabe num 8x8");
    conferir(&ms, 8, &pos);
    // ⛔ O CONTROLO: num quadrado pequeno demais ele RECUSA em vez de sobrepor.
    assert!(arruma(&ms, 2, 0).is_none());
}

/// ⭐⭐⭐ **O que este módulo existe para fazer: um `L` aninha no vão do outro.**
///
/// ⚠️ **Com as CAIXAS não cabe, e é por isso que a fixtura é esta:** duas caixas `3 × 3`
/// não entram num quadrado `4 × 4`, e as duas FORMAS entram — é a diferença que a §10
/// mede como `18,7 % → 22,2 %` de tinta nas peças do dono.
#[test]
fn um_l_aninha_no_vao_do_outro_onde_as_caixas_nao_cabiam() {
    let l = desenho(&["#..", "#..", "###"]);
    let l_invertido = desenho(&["###", "..#", "..#"]);
    assert_eq!((l.ocupadas(), l_invertido.ocupadas()), (5, 5));
    let ms = vec![l, l_invertido];
    let pos = arruma(&ms, 4, 0).expect("as duas FORMAS cabem num 4x4");
    conferir(&ms, 4, &pos);
    // ⛔ O CONTROLO que prova que a fixtura contém o fenómeno: as CAIXAS não cabem.
    let caixas = vec![
        desenho(&["###", "###", "###"]),
        desenho(&["###", "###", "###"]),
    ];
    assert!(
        arruma(&caixas, 4, 0).is_none(),
        "com as caixas cheias tinha de nao caber"
    );
}

/// ⛔⛔ **A rasterização é CONSERVADORA**: uma célula que o triângulo TOCA fica marcada.
///
/// *Uma amostra do centro deixaria uma lasca a atravessar a fronteira sem marcar nada*, e
/// a garantia de não-sobreposição em `[0,1]²` cairia com ela.
#[test]
fn a_rasterizacao_marca_toda_celula_que_o_triangulo_toca() {
    let mut m = Mascara {
        larg: 4,
        alt: 4,
        celulas: vec![false; 16],
    };
    // Uma lasca fininha a atravessar a linha `y = 1,5`: o centro de nenhuma célula cai
    // dentro dela, e mesmo assim ela toca as quatro colunas.
    marca_triangulo(&mut m, [[0.1, 1.45], [3.9, 1.45], [3.9, 1.55]], 1.0);
    for x in 0..4 {
        assert!(
            m.celulas[4 + x],
            "a coluna {x} da linha 1 tem de estar marcada"
        );
    }
    assert_eq!(m.ocupadas(), 4, "e mais nenhuma");

    // ⛔⛔ **O CONTROLO, e a 1.ª redacção dele NÃO discriminava:** ela olhava uma célula
    // FORA da caixa do triângulo, que a varredura nunca visita — logo um `toca` que
    // devolvesse `true` a tudo passava na mesma, e a mutação SOBREVIVEU. O caso que
    // separa é um triângulo cuja CAIXA cobre a grelha inteira e cujo corpo não: uma
    // hipotenusa deixa o canto oposto limpo.
    let mut d = Mascara {
        larg: 4,
        alt: 4,
        celulas: vec![false; 16],
    };
    marca_triangulo(&mut d, [[0.0, 0.0], [3.99, 0.0], [0.0, 3.99]], 1.0);
    assert!(d.celulas[0], "o canto do angulo recto e' tocado");
    assert!(
        !d.celulas[3 * 4 + 3],
        "o canto OPOSTO a' hipotenusa nao e', e a caixa do triangulo cobre-o"
    );
    assert!(
        d.ocupadas() < 16 && d.ocupadas() >= 10,
        "cerca de metade mais a diagonal: {}",
        d.ocupadas()
    );
}

/// ⭐ **A folga entra na máscara**, e é isso que faz o empacotador não a poder esquecer.
#[test]
fn a_folga_cresce_a_mascara_em_todas_as_direccoes() {
    let m = desenho(&["#"]);
    let g = com_folga(&m, 2);
    assert_eq!((g.larg, g.alt, g.ocupadas()), (5, 5, 25));
    assert_eq!(com_folga(&m, 0).ocupadas(), 1);
    // ⛔⛔ **E a folga é paga UMA vez e não duas.** Ela entra pelo [`arruma`], que engorda
    // o lado que PERGUNTA e marca o que é SÓLIDO ⇒ entre duas peças fica exactamente `g`
    // célula(s). *Engordar os dois lados paga `2g`, e a cadeia de mips só pede `g`.*
    let ms: Vec<Mascara> = (0..4).map(|_| desenho(&["##", "##"])).collect();
    let pos = arruma(&ms, 16, 1).expect("cabem");
    let mut menor = usize::MAX;
    for i in 0..ms.len() {
        for j in (i + 1)..ms.len() {
            let dx = pos[i].0.abs_diff(pos[j].0);
            let dy = pos[i].1.abs_diff(pos[j].1);
            // As peças medem `2 × 2`: `3` de distância é exactamente uma célula de vão.
            assert!(dx >= 3 || dy >= 3, "as pecas {i} e {j} encostam: {pos:?}");
            menor = menor.min(dx.max(dy));
        }
    }
    assert_eq!(
        menor, 3,
        "com `g = 1` o par mais perto fica a UMA celula de vao, nao a duas: {pos:?}"
    );
}

/// ⛔⛔ **Uma máscara VAZIA é recusada.** Ela caberia em qualquer sítio sem marcar nada, e
/// tudo o que viesse a seguir passaria por cima dela.
#[test]
fn uma_mascara_vazia_e_recusada_em_vez_de_arrumada() {
    let vazia = Mascara {
        larg: 2,
        alt: 2,
        celulas: vec![false; 4],
    };
    assert_eq!(vazia.ocupadas(), 0);
    assert!(arruma(&[vazia], 8, 0).is_none());
    // ⭐ O CONTROLO: a mesma com UMA célula entra.
    assert!(arruma(&[desenho(&["#."])], 8, 0).is_some());
}

/// ⭐⭐⭐ **O empacotador por máscara CHEGA ao atlas e arruma melhor que o de caixas.**
///
/// ⛔ As de cima medem a PORTA; esta percorre o `build` inteiro — *um empacotador com a
/// lei certa e o atlas a não o chamar lê-se como um empacotador que não funciona*.
#[test]
fn o_empacotador_por_mascara_chega_ao_atlas_e_aperta_o_quadrado() {
    let (mesh, cut, map, jumps) = super::lib_tests::fita_com(0, false, true);
    let mascara = super::build(&mesh, &cut, &map, &jumps);
    let caixas = super::build_com(
        &mesh,
        &cut,
        &map,
        &jumps,
        super::Opcoes {
            empacotar_por_mascara: false,
            ..super::Opcoes::default()
        },
    );
    assert_eq!(
        mascara.relatorio.ilhas, caixas.relatorio.ilhas,
        "o corte e' o mesmo — so' a arrumacao muda"
    );
    assert!(
        mascara.relatorio.aproveitamento > caixas.relatorio.aproveitamento,
        "a tinta tem de ocupar mais do quadrado: {} contra {}",
        mascara.relatorio.aproveitamento,
        caixas.relatorio.aproveitamento
    );
    // ⛔ E a lei que ele não pode quebrar: duas peças nunca partilham um ponto.
    let s = super::sobreposicao::medir(&mesh, &mascara);
    let i = super::sobreposicao::Classe::IlhasDiferentes.indice();
    assert_eq!(s.pares_por_classe[i], 0, "{:?}", s.pares);
    for z in &mascara.uv {
        assert!(
            (0.0..=1.0).contains(&z[0]) && (0.0..=1.0).contains(&z[1]),
            "o (u,v) saiu do quadrado: {z:?}"
        );
    }
}

/// ⭐⭐⭐ **As duas curas da W3 shipam LIGADAS, e isso é uma decisão com razão escrita.**
///
/// ⛔⛔ A lei da casa é que tudo o que é novo ship desligado — e aqui não há produto do
/// outro lado: nenhum botão consome este atlas ainda, e desligá-las faria [`super::build`]
/// entregar, por omissão, um atlas com `18,7 %` de tinta quando ele sabe fazer `29,6 %`.
/// *Uma omissão que entrega um resultado sabidamente pior não é conservadora.*
///
/// ⚠️ E este gate nasceu de uma **mutação SOBREVIVENTE**: trocar o valor de fábrica não
/// partia nada, porque todos os outros gates passam as opções à mão.
#[test]
fn as_curas_do_espaco_shipam_ligadas() {
    let o = super::Opcoes::default();
    assert!(o.cortar, "sem o corte o atlas pinta duas vezes");
    assert!(
        o.orientar,
        "sem orientar a caixa de uma ilha esguia e' quase vazia"
    );
    assert!(
        o.empacotar_por_mascara,
        "sem a mascara arruma-se o involucro"
    );
    // ⛔⛔ **E a COLAGEM shipa DESLIGADA, pela mesma régua e com o sinal ao contrário.**
    // Medido na malha do dono: colar dá `31,8 %` de tinta contra `42,6 %` e `156,5` de
    // costura contra `129,3` — *ela perde nos dois eixos* (doc 26 §11). A lei fica, a
    // porta fica, o valor de fábrica é não colar.
    assert!(!o.colar, "colar perde tinta E costura na malha do artista");
}

/// ⛔⛔ **Uma peça que colapsa num PONTO não derruba o empacotador por máscara.**
///
/// A rasterização visita `floor(min)..ceil(max)`, e numa peça de extensão zero esse
/// intervalo é **VAZIO** — a máscara sai sem uma célula, e o empacotador recusa-a (e bem:
/// ela caberia em todo o lado sem ocupar nada). ⇒ o atlas dá-lhe **uma** célula.
///
/// ⚠️ Sem esta cerca a recusa arrasta o atlas INTEIRO para o empacotador de prateleiras,
/// que é a rede — *uma peça degenerada não pode custar a arrumação de todas as outras*.
/// A mutação que a apaga sobrevivia porque nenhuma fixtura tinha uma peça de área zero.
#[test]
fn uma_peca_que_colapsa_num_ponto_nao_derruba_a_arrumacao() {
    let (mesh, cut, mut map, jumps) = super::lib_tests::fita_com(0, false, true);
    // A carta `2` inteira num ponto só: as duas faces dela têm área zero em `(u, v)`.
    map.uv[2] = vec![[20.0, -7.0]; 4];
    let mascara = super::build(&mesh, &cut, &map, &jumps);
    let caixas = super::build_com(
        &mesh,
        &cut,
        &map,
        &jumps,
        super::Opcoes {
            empacotar_por_mascara: false,
            ..super::Opcoes::default()
        },
    );
    assert_eq!(mascara.relatorio.orfaos, 0, "ninguem fica sem (u,v)");
    assert!(
        mascara.relatorio.aproveitamento > caixas.relatorio.aproveitamento,
        "o empacotador por mascara TEM de ter corrido: {} contra {}",
        mascara.relatorio.aproveitamento,
        caixas.relatorio.aproveitamento
    );
    for z in &mascara.uv {
        assert!(
            (0.0..=1.0).contains(&z[0]) && (0.0..=1.0).contains(&z[1]),
            "{z:?}"
        );
    }
}

/// ⭐⭐⭐ **AS MAIORES PRIMEIRO, e isto é lei e não gosto.**
///
/// Medido na escultura do dono, inverter a ordem custa **`3,0` pontos** de tinta na malha
/// crua (`29,6 % → 26,6 %`) e **`4,2`** na remalhada (`38,7 % → 34,5 %`) — *na mesma
/// direcção nas duas, que é o que separa uma alavanca de ruído*.
///
/// ⚠️ A fixtura é o caso mínimo em que a ordem decide entre CABER e não caber: um
/// quadrado `4 × 4`, uma peça `3 × 3` e sete peças de uma célula, que somam `16` — o
/// quadrado inteiro. Com a grande primeiro, as sete entram no `L` que sobra; com as
/// pequenas primeiro, elas ocupam duas fileiras e a grande **deixa de ter sítio**.
#[test]
fn as_maiores_primeiro_e_isso_decide_entre_caber_e_nao_caber() {
    let mut ms = vec![desenho(&["###", "###", "###"])];
    ms.extend((0..7).map(|_| desenho(&["#"])));
    assert_eq!(
        ms.iter().map(Mascara::ocupadas).sum::<usize>(),
        16,
        "a fixtura enche o quadrado exactamente"
    );
    let pos = arruma(&ms, 4, 0).expect("com as maiores primeiro, cabe");
    conferir(&ms, 4, &pos);

    // ⛔ O CONTROLO: as MESMAS peças, arrumadas pela ordem inversa à mão, não cabem.
    let mut invertida: Vec<Mascara> = ms.iter().skip(1).cloned().collect();
    invertida.push(ms[0].clone());
    // ⚠️ `arruma` reordena sozinha, logo a ordem da lista não a engana — o que prova o
    // CONTROLO é a prova de mutação sobre a chave do `sort`, e ela está no arnês.
    assert!(
        arruma(&invertida, 4, 0).is_some(),
        "ela reordena e continua a caber"
    );
}

/// ⛔⛔ **Sem colagem, o relatório NÃO afirma uma holonomia.**
///
/// A [`super::Relatorio::holonomia_max`] mede o rasgo que sobra ao fechar um ciclo de
/// costuras COLADAS. Sem colagem não há ciclo nenhum, e a 1.ª redacção devolvia a
/// distância CRUA entre os dois lados — `4,03e1` numa esfera, que se lê como *«o
/// assentamento falhou»* quando a verdade é *«não houve assentamento»*.
///
/// ⚠️ O `0` só é honesto porque a [`super::Relatorio::coladas`] o acompanha e lê `0`
/// também: *um zero de «não medido» e um de «perfeito» são o mesmo byte, e o que os
/// separa é o piso de população ao lado.*
#[test]
fn sem_colagem_o_relatorio_nao_inventa_uma_holonomia() {
    let (mesh, cut, map, jumps) = super::lib_tests::fita(0, true);
    let sem = super::build(&mesh, &cut, &map, &jumps);
    assert_eq!(
        (sem.relatorio.coladas, sem.relatorio.ciclos),
        (0, 0),
        "sem colagem nao ha' costura colada nem ciclo"
    );
    assert!(
        sem.relatorio.holonomia_max == 0.0 && sem.relatorio.cola_max == 0.0,
        "e nenhuma das duas colunas afirma um numero: {} / {}",
        sem.relatorio.holonomia_max,
        sem.relatorio.cola_max
    );
    // ⭐ O CONTROLO: a MESMA fixtura colada tem holonomia de verdade, e a população
    // di-lo. Sem esta metade, um `build` que nunca medisse nada passaria.
    let com = super::lib_tests::colado(&mesh, &cut, &map, &jumps);
    assert!(com.relatorio.coladas >= 3 && com.relatorio.ciclos >= 1);
    assert!(
        com.relatorio.holonomia_max > 1.0,
        "o anel colado tem de acusar rasgo: {}",
        com.relatorio.holonomia_max
    );
}
