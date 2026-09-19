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
    assert!(rows(ligado(), false, false).is_empty());
    assert_eq!(rows(ligado(), true, false).len(), LINHAS.len());
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
            threshold: limiar,
            ..ligado()
        };
        let joelho = rows(b, true, false)
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

/// ⭐⭐⭐ **AS TRÊS RAZÕES, e a ordem delas é a lei** — da mais geral para a mais específica.
///
/// ⚠️ *Dizer «o brilho está desligado» a quem também está no caminho do dispositivo é mandá-lo
/// resolver a metade errada* — e é por isso que o gate mede a ORDEM e não só a presença.
#[test]
fn a_razao_certa_para_o_facto_certo() {
    // (1) No dispositivo: TODAS apagadas, e com a razão do MOTOR — mesmo a do interruptor.
    let no_dev = rows(ligado(), true, true);
    assert_eq!(no_dev.len(), LINHAS.len());
    assert!(
        no_dev
            .iter()
            .all(|r| r.inert == Some("field.inert.bloom_runs_on_the_reference_path")),
        "no caminho do dispositivo todas as fileiras têm de dizer o motor"
    );

    // ⭐⭐⭐ **(1-bis) A CÉLULA QUE DISCRIMINA A ORDEM: no dispositivo E com o brilho DESLIGADO.**
    //
    // ⛔ Ela nasceu de uma mutação SOBREVIVENTE: trocar os dois blocos do [`super::apagada`] deixava
    // este gate verde, porque as células acima têm o brilho **LIGADO** — e com ele ligado o ramo do
    // interruptor não dispara, logo a ordem não é observável. *Um corpus que não contém as duas
    // condições ao mesmo tempo não pode testar qual delas ganha.*
    //
    // ⚠️ E a resposta certa é a do MOTOR, com mecanismo: ligar o brilho aqui **não acende nada**,
    // logo mandar o artista ligá-lo é mandá-lo resolver a metade errada.
    let dev_e_off = rows(Bloom::default(), true, true);
    assert!(
        dev_e_off
            .iter()
            .all(|r| r.inert == Some("field.inert.bloom_runs_on_the_reference_path")),
        "com as DUAS razões em jogo tem de ganhar a do motor: {:?}",
        dev_e_off.iter().filter_map(|r| r.inert).collect::<Vec<_>>()
    );

    // (2) Na referência com o brilho DESLIGADO: só o interruptor é vivo.
    let desligado = rows(Bloom::default(), true, false);
    assert_eq!(
        desligado.iter().filter(|r| r.inert.is_none()).count(),
        1,
        "com o brilho desligado a única fileira viva é o interruptor"
    );
    assert!(
        desligado[0].inert.is_none(),
        "o interruptor tem de ser vivo"
    );

    // (3) Na referência com ele LIGADO: nenhuma apagada — a metade NEGATIVA, sem a qual «apagar»
    // vira licença.
    let vivo = rows(ligado(), true, false);
    assert!(
        vivo.iter().all(|r| r.inert.is_none()),
        "com o brilho ligado e na referência nenhuma fileira pode estar apagada: {:?}",
        vivo.iter().filter_map(|r| r.inert).collect::<Vec<_>>()
    );

    // (4) E o joelho com o limiar em ZERO — a razão específica, que só aparece quando a geral não
    // se aplica.
    let sem_limiar = rows(
        Bloom {
            threshold: 0.0,
            ..ligado()
        },
        true,
        false,
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
    let r = rows(ligado(), true, false);
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
    assert_eq!(
        vistas,
        (0..u8::try_from(Bloom::SLOTS).expect("cabe")).collect::<Vec<_>>(),
        "as posições da arrumação e as das fileiras têm de ser as mesmas"
    );
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
    // ⚠️ E a terceira resposta (o motor) tem de CHEGAR lá: sem ela as fileiras nasciam vivas no
    // caminho do dispositivo, onde mexer nelas não muda um pixel.
    let i = painel
        .find("brilho_painel::rows(")
        .expect("a chamada que o assert acima acabou de encontrar");
    assert!(
        painel[i..i + 300].contains("gpu_frame::enabled()"),
        "a chamada não leva a resposta do MOTOR, e as fileiras nasceriam vivas no dispositivo"
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
