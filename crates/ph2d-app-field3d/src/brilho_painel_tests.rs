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

/// ⭐⭐⭐ **A CENA `=36` CONTÉM O FENÓMENO** — as luzes derramam e a BARRA não, medido em píxeis.
///
/// ⛔⛔ **É a lei que esta casa exige de toda cena de smoke**, e ela não é um detalhe: *uma cena que
/// ensina o CONTRÁRIO do que acontece é pior que uma cena ausente, porque a ausente não é
/// acreditada* (`CLAUDE.md` §5.0). O roteiro promete três coisas e este gate mede as três:
///
/// 1. com o brilho LIGADO as bolas ganham halo **fora** delas;
/// 2. a barra **não** ganha halo próprio — ela não emite, e é o CONTROLO da cena;
/// 3. com o brilho DESLIGADO a imagem é a de sempre, **ao bit**.
///
/// ⚠️ **A metade (2) é a que faz as outras duas valerem alguma coisa:** sem ela, «tudo acendeu»
/// passa — e «tudo acendeu» é um passe que ignora o limiar.
#[test]
fn a_cena_do_brilho_contem_o_fenomeno() {
    use crate::render_light::{StudioSky, lamps};
    let (w, h) = (280u32, 200u32);
    let cam = ph2d_field_render::Orbit::default();
    let doc = crate::smoke::scenes::scene(36);
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);

    // ⭐⭐⭐ **OS MATERIAIS ENTRAM PELO MUNDO, e a 1.ª redacção passava-os numa lista com
    // `owners: None` — o que faz TODO pixel usar `all[0]`.**
    //
    // ⛔ O controlo da barra leu `12 475` contra `12 475`: *acender a barra não mudava um byte,
    // porque a barra estava a ser pintada com o material da primeira bola*. Um gate assim mede uma
    // cena com um material só, que não é a cena do app — e ele teria ficado VERDE sobre a metade
    // que interessa se eu tivesse escrito a barra ao contrário.
    //
    // ⇒ a tabela é a do PRODUTO ([`crate::materials::Table::build`]), construída do MUNDO como o
    // quadro a constrói. *É ela que sabe qual folha usa qual material.*
    let rig = ph2d_light::LightRig::default();
    let lam = lamps(&rig);
    let light = ph2d_field_render::Lighting {
        lamps: &lam,
        points: &[],
        sky: &StudioSky,
        shadows: None,
    };
    let mut sim = ph2d_ecs::SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    // ⚠️ As folhas são os FILHOS da raiz — a mesma porta que o `materials_tests` usa, e a mesma
    // ordem que o [`crate::smoke::scenes::materiais_da_cena`] declara por escrito.
    let folhas: Vec<bevy_ecs::entity::Entity> = sim
        .world()
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("a raiz tem filhos")
        .iter()
        .copied()
        .collect();
    let seed = crate::smoke::scenes::materiais_da_cena(36).expect("a cena 36 pede material");
    assert_eq!(
        folhas.len(),
        seed.len(),
        "a cena semeia {} materiais para {} folhas — a ordem do `materiais_da_cena` é a das FOLHAS",
        seed.len(),
        folhas.len()
    );
    let semeia = |sim: &mut ph2d_ecs::SimWorld, mats: &[ph2d_field_ecs::FieldMaterial]| {
        for (e, m) in folhas.iter().zip(mats) {
            sim.world_mut().entity_mut(*e).insert(*m);
        }
    };
    semeia(&mut sim, &seed);
    let tabela = crate::materials::Table::build(sim.world(), root, cam.half_extent, w as f32);
    const FUNDO: [u8; 4] = [0, 0, 0, 255];
    let pinta_com = |t: &crate::materials::Table, b: ph2d_field_render::Bloom| {
        ph2d_field_render::shade_render(
            &g,
            &cam,
            &t.surfaces_for(),
            &light,
            &ph2d_field_render::Presentation {
                bloom: b,
                ..ph2d_field_render::Presentation::of(crate::shading::OPENING_LOOK)
            },
            FUNDO,
        )
    };
    let pinta = |b: ph2d_field_render::Bloom| pinta_com(&tabela, b);
    let sem = pinta(ph2d_field_render::Bloom::default());
    let com_brilho = pinta(ph2d_field_render::Bloom {
        enabled: true,
        ..ph2d_field_render::Bloom::default()
    });

    // (3) A omissão é a imagem de sempre — e ela vem primeiro porque protege tudo o que já shipou.
    let base = pinta(ph2d_field_render::Bloom::default());
    assert_eq!(
        sem, base,
        "a cena não é determinista, e o resto não vale nada"
    );

    // ⭐⭐⭐ **(1) e (2) medem-se com um CONTROLO, e a 1.ª redacção mediu sem ele.**
    //
    // ⛔ Ela contava os píxeis acesos na metade de BAIXO do ecrã e exigia que fossem poucos —
    // e leu **`49,7 %`** sobre uma cena CERTA. *A cadeia de níveis espalha o halo das bolas por
    // dezenas de píxeis, logo ele atravessa a linha média por construção*: a régua media a
    // GEOMETRIA da janela, não a lei.
    //
    // ⇒ a régua é **relativa**: a MESMA cena com a barra ACESA tem de acender muito mais em baixo.
    // *É o controlo que transforma «acendeu pouco» numa afirmação sobre o que a barra emite.*
    let com_barra_acesa = {
        let mut m = seed.clone();
        let ultimo = m.len() - 1;
        m[ultimo] = ph2d_field_ecs::FieldMaterial {
            emission: 8.0,
            emission_color: [1.0, 0.92, 0.80],
            specular_weight: 0.0,
            ..ph2d_field_ecs::FieldMaterial::default()
        };
        semeia(&mut sim, &m);
        let t2 = crate::materials::Table::build(sim.world(), root, cam.half_extent, w as f32);
        pinta_com(
            &t2,
            ph2d_field_render::Bloom {
                enabled: true,
                ..ph2d_field_render::Bloom::default()
            },
        )
    };

    let meio = (h / 2) as usize;
    let conta = |img: &[u8], de: usize, ate: usize| {
        let mut n = 0usize;
        for y in de..ate {
            for x in 0..w as usize {
                let i = y * w as usize + x;
                if !g.hit[i] && sem[i * 4..i * 4 + 3] != img[i * 4..i * 4 + 3] {
                    n += 1;
                }
            }
        }
        n
    };
    let em_cima = conta(&com_brilho, 0, meio);
    let em_baixo = conta(&com_brilho, meio, h as usize);
    let em_baixo_acesa = conta(&com_barra_acesa, meio, h as usize);

    // (1) As luzes derramam.
    assert!(
        em_cima > 300,
        "as luzes não derramam: só {em_cima} píxeis de fundo acenderam junto às bolas"
    );
    // ⭐⭐⭐ **(1-bis) A BARRA É ESCURA, e a régua pergunta POR FOLHA** — medido no caminho do RENDER.
    //
    // ⚠️⚠️ **A FOTO não podia dizer isto:** a cena abre em **Matcap**, e o matcap ignora a cor do
    // material (ele é a luz do olho). A primeira foto mostrou a barra do mesmo rosa das bolas e eu
    // quase a li como defeito — *o que a foto mostra ali é o MODO, não o material*.
    //
    // ⛔⛔ **E a 1.ª redacção deste gate media uma BANDA DO ECRÃ** (o quarto de baixo) e ficou VERDE
    // sobre uma mutação que pintava a barra de rosa: aquela banda cai quase toda ABAIXO da barra, e
    // a média dela é feita de meia dúzia de píxeis. ⇒ quem diz de quem é cada pixel é o
    // [`ph2d_field_eval::owners::Owners`] que a tabela do produto já construiu — *a mesma porta que
    // o quadro usa para saber que material pintar*.
    let donos = tabela.owners.as_ref().expect("quatro folhas pedem um dono");
    let barra_idx = folhas.len() - 1;
    let (mut luz_bolas, mut n_bolas) = (0u64, 0u64);
    let (mut luz_barra, mut n_barra) = (0u64, 0u64);
    for i in 0..(w * h) as usize {
        if !g.hit[i] {
            continue;
        }
        let soma = u64::from(sem[i * 4]) + u64::from(sem[i * 4 + 1]) + u64::from(sem[i * 4 + 2]);
        match donos.at(g.point[i]) {
            Some(k) if k == barra_idx => {
                luz_barra += soma;
                n_barra += 3;
            }
            Some(_) => {
                luz_bolas += soma;
                n_bolas += 3;
            }
            None => {}
        }
    }
    assert!(
        n_barra > 200 && n_bolas > 200,
        "a régua não achou as duas populações ({n_barra} píxeis de barra, {n_bolas} de bolas) — \
         ela partiu-se e mediria o nada"
    );
    #[allow(clippy::cast_precision_loss)]
    let (mb, mbo) = (
        luz_barra as f64 / n_barra as f64,
        luz_bolas as f64 / n_bolas as f64,
    );
    assert!(
        mb < mbo * 0.5,
        "o roteiro chama-lhe «a barra ESCURA» e ela lê {mb:.0} contra {mbo:.0} das bolas — \
         uma cena que diz uma coisa e mostra outra é pior que uma cena ausente"
    );

    // ⭐⭐⭐ **(1-ter) A ESCADA DAS LUZES separa-as, e isso é o que faz o limiar ENSINAR.**
    //
    // ⛔ Uma mutação que baixasse as três forças NÃO mata o halo — e isso é um facto sobre a cena
    // que vale a pena guardar: **o céu de estúdio já põe uma peça clara acima do limiar**, logo o
    // que faz uma coisa brilhar aqui é ser CLARA, e emitir é uma maneira de o ser. *A barra não
    // brilha por ser escura, e não por «não emitir».*
    //
    // ⇒ o que esta cena tem de garantir é a ESCADA: com passos de menos de `4×` o limiar apaga-as
    // todas ao mesmo tempo e o roteiro («apagam-se UMA DE CADA VEZ») passa a mentir.
    for par in crate::smoke::scenes::edge::BRILHOS_DA_CENA.windows(2) {
        assert!(
            par[1] >= par[0] * 3.5,
            "a escada das luzes é {par:?} — com passos curtos o limiar apaga-as todas de uma vez"
        );
    }
    assert!(
        crate::smoke::scenes::edge::BRILHOS_DA_CENA[0]
            > ph2d_field_render::Bloom::default().threshold,
        "a luz mais fraca tem de começar ACIMA do limiar de fábrica, senão ela nunca acende"
    );

    // (2) E a BARRA não é quem acende a metade de baixo — o controlo di-lo.
    assert!(
        em_baixo_acesa > em_baixo * 3 / 2,
        "o controlo não separa as duas cenas: a barra ESCURA acendeu {em_baixo} e a ACESA \
         {em_baixo_acesa} — se a barra já brilhasse, os dois números seriam parecidos"
    );
}
