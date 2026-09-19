//! ⭐⭐⭐ **A CENA DO BRILHO e o que ela PROVA no produto** — a metade que mede PÍXEIS.
//!
//! ⚠️ **Irmão por RESPONSABILIDADE do [`super::tests`], e o corte foi forçado pelo tecto de LOC —
//! que escolheu a fronteira certa:** aquele responde *«a fileira certa escreve no número certo, e
//! diz a razão certa quando está apagada»*; este responde *«e na cena do dono, o halo aparece?»*.
//! As duas perguntas têm curas opostas — uma arruma uma tabela, a outra muda a lei ou a cena.

use super::*;

fn ligado() -> Bloom {
    Bloom {
        enabled: true,
        ..Bloom::default()
    }
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

    let donos = tabela.owners.as_ref().expect("quatro folhas pedem um dono");
    let barra_idx = folhas.len() - 1;

    // ⭐⭐⭐ **(1-bis) A BARRA É ESCURA, e a régua pergunta POR FOLHA** — medido no caminho do RENDER.
    //
    // ⚠️⚠️ **A FOTO não podia dizer isto:** a cena abre em **Matcap**, e o matcap ignora a cor do
    // material (ele é a luz do olho). A primeira foto mostrou a barra do mesmo rosa das bolas e eu
    // quase a li como defeito — *o que a foto mostra ali é o MODO, não o material*.
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
        "a régua não achou as duas populações ({n_barra} píxeis de barra, {n_bolas} de bolas)"
    );
    #[allow(clippy::cast_precision_loss)]
    let (mb, mbo) = (
        luz_barra as f64 / n_barra as f64,
        luz_bolas as f64 / n_bolas as f64,
    );
    assert!(
        mb < mbo * 0.5,
        "o roteiro chama-lhe «a barra ESCURA» e ela lê {mb:.0} contra {mbo:.0} das bolas"
    );

    // ⭐⭐⭐ **(1-ter) A ESCADA DAS LUZES separa-as, e isso é o que faz o limiar ENSINAR.**
    //
    // ⛔ Uma mutação que baixasse as três forças NÃO mata o halo — **o céu de estúdio já põe uma
    // peça clara acima do limiar**, logo o que faz uma coisa brilhar aqui é ser CLARA, e emitir é
    // uma maneira de o ser. *A barra não brilha por ser escura, e não por «não emitir».*
    for par in crate::smoke::scenes::edge::BRILHOS_DA_CENA.windows(2) {
        assert!(
            par[1] >= par[0] * 2.9,
            "a escada das luzes é {par:?} — com passos curtos o limiar apaga-as todas de uma vez"
        );
    }
    assert!(
        crate::smoke::scenes::edge::BRILHOS_DA_CENA[0]
            > ph2d_field_render::BloomParams::default().threshold,
        "a luz mais fraca tem de começar ACIMA do limiar de fábrica, senão ela nunca acende"
    );
    // ⭐⭐⭐ **E A MAIS FORTE TEM DE CABER DEBAIXO DO SLIDER DO LIMIAR** — senão o passo do roteiro
    // («suba o Threshold e elas apagam-se uma de cada vez») promete o que o painel não deixa fazer.
    let teto = rows(ligado(), true)
        .into_iter()
        .find(|r| r.param == Param::Bloom(1))
        .map(|r| r.bound.value())
        .expect("o tecto do limiar");
    let mais_forte = crate::smoke::scenes::edge::BRILHOS_DA_CENA[2];
    assert!(
        teto > mais_forte,
        "o slider do limiar pára em {teto} e a luz mais forte da cena vale {mais_forte} — \
         ela seria INAPAGÁVEL, e o roteiro manda apagá-la"
    );

    // ⭐⭐⭐ **A REGIÃO É A PONTA DA BARRA, e as duas redacções anteriores mediram a JANELA.**
    //
    // ⛔ A 1.ª contou a metade de BAIXO do ecrã e leu `49,7 %` numa cena certa; a 2.ª comparou-a com
    // a barra acesa e leu `22 628` contra `25 106` (`1,11×`) — ⚠️ **e a causa da segunda foi a cadeia
    // da REFERÊNCIA**, que espalha o halo das bolas muito mais longe do que a minha espalhava, logo
    // ele inunda a metade de baixo por construção. *Uma régua calibrada contra um motor pior mede a
    // janela quando o motor melhora.*
    //
    // ⇒ a região é **derivada da geometria da cena**: a barra é mais COMPRIDA que a fileira de
    // bolas, logo as pontas dela têm fundo que não é vizinho de bola nenhuma. É lá que a pergunta
    // *«a barra brilha?»* tem resposta.
    let caixa = |quem: &dyn Fn(usize) -> bool| {
        let (mut x0, mut x1, mut y0, mut y1) = (usize::MAX, 0usize, usize::MAX, 0usize);
        for y in 0..h as usize {
            for x in 0..w as usize {
                let i = y * w as usize + x;
                if g.hit[i] && donos.at(g.point[i]).is_some_and(quem) {
                    x0 = x0.min(x);
                    x1 = x1.max(x);
                    y0 = y0.min(y);
                    y1 = y1.max(y);
                }
            }
        }
        (x0, x1, y0, y1)
    };
    let (bx0, bx1, by0, by1) = caixa(&|k| k == barra_idx);
    let (ex0, ex1, _, _) = caixa(&|k| k != barra_idx);
    assert!(
        bx0 < ex0 && bx1 > ex1,
        "a barra tem de ser mais comprida que a fileira de bolas para haver ponta: \
         barra {bx0}..{bx1}, bolas {ex0}..{ex1}"
    );
    let na_ponta = |x: usize, y: usize| {
        (x < ex0 || x > ex1) && (by0..=by1).contains(&y) && x >= bx0 && x <= bx1
    };
    // ⚠️⚠️ **A régua é a SOMA da luz acrescentada, e não QUANTOS píxeis mudaram** — a contagem
    // leu `30` contra `30` porque naquela região **todos** os píxeis já tinham mudado: ela satura.
    // *Uma régua que conta quantos não vê quanto*, e é a terceira forma desta mesma armadilha nesta
    // jornada (a meia-altura, o alcance, e agora a contagem).
    let conta = |img: &[u8]| {
        let mut soma = 0i64;
        for y in by0..=by1 {
            for x in bx0..=bx1 {
                let i = y * w as usize + x;
                if !g.hit[i] && na_ponta(x, y) {
                    for c in 0..3 {
                        soma += i64::from(img[i * 4 + c]) - i64::from(sem[i * 4 + c]);
                    }
                }
            }
        }
        soma
    };
    let na_ponta_escura = conta(&com_brilho);
    let na_ponta_acesa = conta(&com_barra_acesa);

    // (1) As luzes derramam — medido no fundo INTEIRO.
    let em_cima = (0..(w * h) as usize)
        .filter(|&i| !g.hit[i] && sem[i * 4..i * 4 + 3] != com_brilho[i * 4..i * 4 + 3])
        .count();
    assert!(
        em_cima > 300,
        "as luzes não derramam: só {em_cima} píxeis de fundo acenderam"
    );
    // (2) E a BARRA não é quem acende: nas pontas dela, acendê-la muda TUDO.
    assert!(
        na_ponta_acesa > na_ponta_escura * 3 / 2,
        "o controlo não separa as duas cenas: nas pontas da barra a ESCURA acendeu \
         {na_ponta_escura} e a ACESA {na_ponta_acesa} — se a barra já brilhasse, os dois números \
         seriam parecidos"
    );
}

/// ⭐⭐⭐ **O HALO CHEGA AO ECRÃ NA CENA A SÉRIO — com o FUNDO do módulo, que é TRANSPARENTE.**
///
/// # ⛔⛔⛔ O report do dono de 2026-09-19, e o que ele expôs sobre as réguas desta wave
///
/// *«Talvez devido a total falta de atmosfera não se possa perceber o efeito ao redor das
/// esferas»* — com a foto. O halo estava a ser **calculado e somado** (medido no app a correr:
/// `2 738 165` de `5 215 704` canais mudados, saltos até `255`) e a tela ficava igual, porque ele
/// mora onde a peça não está e ali o quadro tem **cobertura zero** — e o que se vê é este quadro
/// **composto** sobre o canvas.
///
/// ⚠️⚠️ **Nenhuma régua desta wave podia vê-lo, e são DUAS cegueiras somadas:** os gates do passe
/// pintavam sobre um fundo **opaco** (ali o alfa é `255` em todo o lado e nunca é a grandeza que
/// decide), e a única coisa que alguém OLHOU foi a [`despeja_o_halo`] — que escreve **PPM**, um
/// formato **sem canal alfa**. *Um despejo que deita fora um canal não pode auditar esse canal.*
///
/// ⭐ Este gate fecha o furo onde ele existe: a **cena do produto**, com o
/// [`crate::smoke::BACKGROUND`] **lido** e não repetido — se alguém o tornar opaco, é este gate que
/// diz que a lei mudou de sujeito.
#[test]
fn o_halo_da_cena_chega_com_cobertura_sobre_o_fundo_do_modulo() {
    use crate::render_light::{StudioSky, lamps};
    let (w, h) = (320u32, 240u32);
    let doc = crate::smoke::scenes::scene(36);
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let mut sim = ph2d_ecs::SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let folhas: Vec<bevy_ecs::entity::Entity> = sim
        .world()
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("filhos")
        .iter()
        .copied()
        .collect();
    for (e, m) in folhas
        .iter()
        .zip(crate::smoke::scenes::materiais_da_cena(36).expect("material"))
    {
        sim.world_mut().entity_mut(*e).insert(m);
    }
    let t = crate::materials::Table::build(sim.world(), root, cam.half_extent, w as f32);
    let rig = ph2d_light::LightRig::default();
    let lam = lamps(&rig);
    let pinta = |b: ph2d_field_render::Bloom| {
        ph2d_field_render::shade_render(
            &g,
            &cam,
            &t.surfaces_for(),
            &ph2d_field_render::Lighting {
                lamps: &lam,
                points: &[],
                sky: &StudioSky,
                shadows: None,
            },
            &ph2d_field_render::Presentation {
                bloom: b,
                ..ph2d_field_render::Presentation::of(crate::shading::OPENING_LOOK)
            },
            // ⭐ **LIDO do módulo** — ver a nota acima.
            crate::smoke::BACKGROUND,
        )
    };
    let sem = pinta(ph2d_field_render::Bloom::default());
    let com = pinta(ph2d_field_render::Bloom {
        enabled: true,
        ..ph2d_field_render::Bloom::default()
    });
    assert_eq!(
        crate::smoke::BACKGROUND[3],
        0,
        "o fundo do módulo deixou de ser transparente: esta lei mudou de sujeito"
    );
    // ⚠️ **A SILHUETA não é fundo**: um pixel de borda leva cobertura PARCIAL das sub-amostras que
    // acertaram (medido: `64` num deles), e sem esta cerca o controlo acusa produto correcto.
    let mut borda = vec![false; (w * h) as usize];
    for e in &g.edges {
        borda[e.pixel as usize] = true;
    }
    let (mut acesos, mut sem_cobertura) = (0usize, 0usize);
    for i in 0..(w * h) as usize {
        if g.hit[i] || borda[i] {
            continue;
        }
        // ⚠️ **O CONTROLO**: sem brilho o fundo desta cena é transparente, logo o que a metade de
        // baixo mede é o HALO e não a peça.
        assert_eq!(sem[i * 4 + 3], 0, "o controlo falhou no pixel {i}");
        let luz = com[i * 4].max(com[i * 4 + 1]).max(com[i * 4 + 2]);
        if luz > 0 {
            acesos += 1;
            sem_cobertura += usize::from(com[i * 4 + 3] == 0);
        }
    }
    assert!(
        acesos > 1000,
        "a cena não contém o fenómeno: só {acesos} píxeis de fundo acenderam"
    );
    assert_eq!(
        sem_cobertura, 0,
        "{sem_cobertura} de {acesos} píxeis do halo saem com cobertura ZERO — o canvas apaga-os"
    );
}
