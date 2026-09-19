//! Os gates das fileiras do brilho.

use super::*;

fn ligado() -> Bloom {
    Bloom {
        enabled: true,
        ..Bloom::default()
    }
}

/// ⭐⭐⭐ **FORA DO RENDER NÃO HÁ SECÇÃO** — a mesma lei do estilo, e a ausência é a resposta.
#[test]
fn fora_do_render_nao_ha_seccao() {
    assert!(rows(ligado(), false).is_empty());
    assert_eq!(rows(ligado(), true).len(), LINHAS.len());
}

/// ⭐⭐⭐ **A ESCRITA VAI PARAR AO NÚMERO CERTO** — as onze posições, uma a uma.
///
/// ⚠️ **A régua é o PRODUTO da ida e da volta:** escrever na posição `n` e ler a posição `n` de
/// volta pela mesma arrumação. *Um gate que só escrevesse ficaria verde com um `match` que manda
/// tudo para o mesmo campo.*
#[test]
fn cada_fileira_escreve_no_seu_numero() {
    for l in &LINHAS {
        let antes = ligado();
        // ⚠️ Um valor que NÃO é o de fábrica de nenhum campo, senão a escrita certa e a errada
        // leem-se iguais.
        let novo = 0.375_f32;
        let depois = with_number(antes, l.slot, novo);
        let (a, d) = (antes.pack(), depois.pack());
        for (k, (x, y)) in a.iter().zip(&d).enumerate() {
            if k == l.slot as usize {
                continue;
            }
            assert_eq!(
                x.to_bits(),
                y.to_bits(),
                "escrever em {} mexeu na posição {k}",
                l.key
            );
        }
        // ⭐ O interruptor é a excepção DECLARADA: `0,375` não é `> 0,5`, logo ele lê `0`.
        let esperado = if l.slot == 0 { 0.0 } else { novo };
        assert_eq!(
            d[l.slot as usize], esperado,
            "{} não guardou o que lhe foi escrito",
            l.key
        );
    }
}

/// ⭐⭐⭐ **O TECTO DO JOELHO É O LIMIAR, e ele ANDA com ele** — o tecto derivado desta secção.
#[test]
fn o_tecto_do_joelho_segue_o_limiar() {
    for limiar in [0.5_f32, 1.0, 4.0, 11.25] {
        let b = Bloom {
            params: ph2d_field_render::BloomParams {
                threshold: limiar,
                ..ph2d_field_render::BloomParams::default()
            },
            ..ligado()
        };
        let joelho = rows(b, true)
            .into_iter()
            .find(|r| r.param == Param::Bloom(2))
            .expect("a fileira do joelho");
        assert_eq!(
            joelho.bound,
            Bound::Soft(limiar),
            "com o limiar em {limiar} o tecto do joelho tinha de o seguir"
        );
    }
}

/// ⭐⭐⭐ **AS TRÊS RAZÕES DA LEI — e a QUARTA, a do MOTOR, foi APAGADA em 2026-09-19.**
///
/// ⛔⛔ **A premissa deste gate morreu e a morte fica à vista.** Até àquele dia a primeira razão era
/// o MOTOR (*«o brilho corre no caminho de referência»*) e ela ganhava de todas as outras: no
/// caminho de omissão as onze fileiras nasciam apagadas, **o interruptor incluído** — ou seja, *o
/// artista não conseguia sequer LIGAR o efeito sem abrir o app com uma variável de ambiente*. A
/// célula que discriminava a ordem daquela razão nasceu de uma mutação sobrevivente e foi-se com
/// ela: com o gémeo em WGSL ([`ph2d_bloom::wgsl`]) o brilho corre nos DOIS motores.
///
/// ⚠️ O que fica são as três razões da **LEI**, que não dependem de motor nenhum — e a ordem delas
/// continua a ser medida, porque *dizer «o brilho está desligado» a quem tem o limiar em zero é
/// mandá-lo resolver a metade errada*.
#[test]
fn a_razao_certa_para_o_facto_certo() {
    // ⛔ **E a razão do motor não pode voltar por engano** — este censo é o que impede alguém de
    // reintroduzir um ramo por motor sem tocar no doc acima.
    assert!(
        !include_str!("brilho_painel.rs").contains("bloom_runs_on_the_reference_path"),
        "a razão do MOTOR voltou ao painel — o brilho corre nos dois desde 19/09"
    );

    // (2) Na referência com o brilho DESLIGADO: só o interruptor é vivo.
    let desligado = rows(Bloom::default(), true);
    assert_eq!(
        desligado.iter().filter(|r| r.inert.is_none()).count(),
        1,
        "com o brilho desligado a única fileira viva é o interruptor"
    );
    assert!(
        desligado[0].inert.is_none(),
        "o interruptor tem de ser vivo"
    );

    // (3) Na referência com ele LIGADO: **só o ÂNGULO fica apagado**, e a excepção é DECLARADA.
    //
    // ⛔ Este gate dizia *«nenhuma apagada»* e reprovou no dia em que o modelo do Godot saiu: o
    // nosso tem anamorfose, e um ângulo com o estiramento em `1` é inerte **por geometria** — um
    // círculo rodado é o mesmo círculo. *A premissa morreu e está reescrita com a morte à vista.*
    let vivo = rows(ligado(), true);
    let apagadas: Vec<&str> = vivo.iter().filter_map(|r| r.inert).collect();
    assert_eq!(
        apagadas,
        vec!["field.inert.bloom_is_round"],
        "com o brilho ligado a ÚNICA apagada é o ângulo, e só por o halo ser redondo"
    );
    // ⭐⭐ **E a metade que impede a excepção de virar licença: com a anamorfose ARMADA ele acende.**
    // *Sem isto, «o ângulo está sempre apagado» passaria — e seria um controlo morto com uma
    // desculpa ao lado.*
    let esticado = rows(
        Bloom {
            params: ph2d_field_render::BloomParams {
                stretch: 2.5,
                ..ph2d_field_render::BloomParams::default()
            },
            ..ligado()
        },
        true,
    );
    assert!(
        esticado.iter().all(|r| r.inert.is_none()),
        "com a anamorfose armada nenhuma fileira pode estar apagada: {:?}",
        esticado.iter().filter_map(|r| r.inert).collect::<Vec<_>>()
    );

    // (4) E o joelho com o limiar em ZERO — a razão específica, que só aparece quando a geral não
    // se aplica.
    let sem_limiar = rows(
        Bloom {
            params: ph2d_field_render::BloomParams {
                threshold: 0.0,
                ..ph2d_field_render::BloomParams::default()
            },
            ..ligado()
        },
        true,
    );
    assert_eq!(
        sem_limiar
            .iter()
            .find(|r| r.param == Param::Bloom(2))
            .and_then(|r| r.inert),
        Some("field.inert.bloom_threshold_is_zero"),
        "com o limiar em zero o joelho não tem passagem para suavizar"
    );
}

/// ⛔ **UMA SECÇÃO E UM CABEÇALHO** — a lei que o painel já assume, e que uma fileira a mais parte.
#[test]
fn ha_um_cabecalho_e_e_o_primeiro() {
    let r = rows(ligado(), true);
    assert_eq!(r.iter().filter(|r| r.section.is_some()).count(), 1);
    assert_eq!(r[0].section, Some(SECCAO));
}

/// ⭐⭐ **AS POSIÇÕES SÃO AS DA ARRUMAÇÃO, e são TODAS** — o censo que impede uma posição órfã.
///
/// ⚠️ ⛔ *Um número que a lei tem e o painel não oferece é um controlo inalcançável, e um que o
/// painel oferece duas vezes é o defeito das cinco cores* (`11` §9).
#[test]
fn as_onze_posicoes_estao_la_uma_vez_cada() {
    let mut vistas: Vec<u8> = LINHAS.iter().map(|l| l.slot).collect();
    vistas.sort_unstable();
    // ⚠️ **Uma AMOSTRA reclama TRÊS posições** (os três canais da tinta), logo a varredura conta a
    // largura de cada fileira — *contar fileiras onde a arrumação conta CANAIS deixaria dois slots
    // por reclamar e o gate leria isso como uma posição órfã*.
    let mut visto = [0u8; Bloom::SLOTS];
    for l in &LINHAS {
        let largura = if l.cor { 3 } else { 1 };
        for v in visto.iter_mut().skip(l.slot as usize).take(largura) {
            *v += 1;
        }
    }
    let orfas: Vec<usize> = (0..Bloom::SLOTS).filter(|&i| visto[i] == 0).collect();
    assert!(orfas.is_empty(), "posições sem fileira nenhuma: {orfas:?}");
    let repetidas: Vec<usize> = (0..Bloom::SLOTS).filter(|&i| visto[i] > 1).collect();
    assert!(
        repetidas.is_empty(),
        "posições com DUAS fileiras — duas superfícies sobre um valor: {repetidas:?}"
    );
    let _ = vistas;
}

/// ⭐⭐⭐ **A CENA USA ESTAS FILEIRAS, e não só a função existe** — a metade que os seis gates acima
/// não têm.
///
/// ⛔⛔ Todos eles chamam [`rows`] **directamente**, logo ficariam VERDES com o
/// [`crate::scene_panel`] a nunca as apender: *um gate que chama a função em vez de percorrer a
/// rota afirma que a peça certa existe, nunca que a cena a usa* — a lei que o módulo da escultura
/// pagou por mutação sobrevivente (`docs/3D` §24).
///
/// ⚠️ **E a ORDEM do dreno é load-bearing:** o braço do brilho tem de vir ANTES do genérico, que
/// casa por TUDO. Um braço depois dele seria um controlo pintado, registado e **morto sob o dedo** —
/// que é exactamente o report que esta casa já recebeu quatro vezes.
#[test]
fn a_cena_apende_as_fileiras_e_o_dreno_ouve_antes_do_generico() {
    let painel = include_str!("scene_panel.rs");
    // ⛔⛔ **É `rows.extend(…)` e não `…rows(`, e a diferença veio de uma MUTAÇÃO SOBREVIVENTE:**
    // trocar o `rows.extend(` por um `let _ = (` deixa a chamada no ficheiro, deixa este gate verde
    // e **larga as onze fileiras no chão**. *Um gate que pergunta «a função é chamada?» não pergunta
    // «o que ela devolve chega a alguém?»* — a mesma distinção que o §5.0 faz entre o clique chegar
    // à ferramenta e a escrita dela chegar a um efeito.
    assert!(
        painel.contains("rows.extend(crate::brilho_painel::rows("),
        "o retrato da cena não APENDE as fileiras do brilho — elas existem e ninguém as vê"
    );
    // ⛔⛔ **A metade que aqui estava MORREU em 2026-09-19, e a morte fica à vista:** ela exigia que
    // a chamada levasse a resposta do MOTOR (`gpu_frame::enabled()`), porque sem ela as fileiras
    // nasciam vivas no caminho do dispositivo — *onde mexer nelas não mudava um pixel*. Com o gémeo
    // em WGSL o brilho corre nos dois, e a resposta deixou de existir.
    //
    // ⚠️ **No lugar dela fica a metade contrária**, que é a que agora pode ficar errada: ninguém
    // pode voltar a passar um motor para aqui sem que este gate o diga.
    let i = painel
        .find("brilho_painel::rows(")
        .expect("a chamada que o assert acima acabou de encontrar");
    assert!(
        !painel[i..i + 300].contains("gpu_frame::enabled()"),
        "a chamada voltou a levar o MOTOR — o brilho corre nos dois desde 19/09"
    );

    // ⛔⛔ **A RÉGUA VARRE CÓDIGO, e a 1.ª redacção varria o FICHEIRO** — ela reprovou sobre uma
    // ordem CERTA, porque o doc-comment do braço do estilo **explica a lei** e diz, por extenso,
    // *«os de baixo casam por `Param::Material(_)`»*. O `find` acertou nesse comentário, que vem
    // **antes** do braço do brilho.
    //
    // ⚠️ É a armadilha que esta casa já nomeou uma vez (a `line/app-physics` Fase B: *«uma régua
    // textual a varrer `\bApp\b` lê o doc-comment que EXPLICA a cura e acusa 93 ficheiros de 133»*)
    // — e o que a torna perigosa é que ela acusa exactamente quem documentou melhor.
    let dreno: String = include_str!("scene_intents.rs")
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let bloom = dreno
        .find("ph2d_field::Param::Bloom(slot)")
        .expect("o braço do brilho no dreno");
    // O genérico do material, que casa por tudo o que sobra.
    let generico = dreno
        .find("Param::Material(_)")
        .or_else(|| dreno.find("ModelIntent::SetParam { param, value"))
        .expect("o braço genérico");
    assert!(
        bloom < generico,
        "o braço do brilho ({bloom}) vem DEPOIS do genérico ({generico}) — ele nunca casa, e as \
         onze fileiras ficam mortas sob o dedo"
    );
}
