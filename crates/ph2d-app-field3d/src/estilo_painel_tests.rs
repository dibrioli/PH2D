//! Os gates das fileiras do estilo.

use super::*;

/// ⭐⭐⭐ **A IDA E A VOLTA FECHAM POR CADA LINHA** — o que o painel mostra é o que a escrita põe.
///
/// ⚠️ **Por VARREDURA sobre as dez**, e não por um caso: a arrumação é uma tabela de posições, e um
/// `slot` trocado entre duas linhas é exactamente o defeito que um caso só não vê.
#[test]
fn cada_linha_le_o_que_ela_propria_escreve() {
    for l in &LINHAS {
        // ⚠️⚠️ **A sonda é DERIVADA do tecto da própria fileira, e não um `0,375` para todas.**
        // Desde que a suavidade da curvatura ganhou cercas de RECURSO (`[MIN_SOFTNESS ..
        // MAX_SOFTNESS]`, ver `ph2d_style::Curvature`), um valor fixo cai **fora** do domínio dela e
        // a porta aperta-o — e a ida-e-volta lia isso como um defeito de arrumação.
        // ⇒ *uma sonda partilhada por fileiras com domínios diferentes mede a cerca, não a tabela.*
        let sonda = l.teto.map_or(0.375, |t| t * 0.375);
        let escrito = if l.teto.is_some() {
            with_number(Style::default(), l.slot, sonda)
        } else {
            with_colour(Style::default(), l.slot, [200, 40, 90])
        };
        let linha = rows(escrito, true)
            .into_iter()
            .find(|r| r.param == Param::Style(l.slot))
            .expect("a linha que acabou de ser escrita");
        if l.teto.is_some() {
            assert!(
                (linha.value - sonda).abs() < 1e-6,
                "{}: escrevi {sonda} e a linha lê {}",
                l.key,
                linha.value
            );
        } else {
            assert_eq!(
                linha.swatch,
                Some([200, 40, 90]),
                "{}: a amostra não devolve a cor escrita",
                l.key
            );
        }
    }
}

/// ⭐⭐⭐ **ESCREVER NUMA LINHA NÃO TOCA NAS OUTRAS.**
///
/// ⛔ É a metade que um gate de ida-e-volta sozinho não tem: um `slot` errado devolve o valor certo
/// **e** estraga o vizinho, e a primeira metade fica verde.
#[test]
fn escrever_numa_linha_nao_toca_nas_outras() {
    for l in &LINHAS {
        // ⚠️ A mesma sonda derivada do irmão acima — ver lá porquê.
        let escrito = if let Some(t) = l.teto {
            with_number(Style::default(), l.slot, t * 0.375)
        } else {
            with_colour(Style::default(), l.slot, [200, 40, 90])
        };
        let antes = wgsl::pack(&Style::default());
        let depois = wgsl::pack(&escrito);
        let largura = if l.teto.is_some() { 1 } else { 3 };
        for (i, (a, b)) in antes.iter().zip(&depois).enumerate() {
            let meu = (l.slot as usize..l.slot as usize + largura).contains(&i);
            assert!(
                meu || (a - b).abs() < f32::EPSILON,
                "{}: escrever no slot {} mexeu no slot {i}",
                l.key,
                l.slot
            );
        }
    }
}

/// ⭐⭐ **AS DEZ LINHAS COBREM OS VINTE NÚMEROS, e cada um UMA vez.**
///
/// ⛔ Uma posição que nenhuma linha alcance é um knob **vivo e inalcançável** — a espécie que o
/// `CLAUDE.md` §5.0 separa do knob morto, e cuja cura é OPOSTA.
#[test]
fn as_linhas_cobrem_a_arrumacao_inteira_e_sem_repetir() {
    let mut visto = [0u8; wgsl::PACKED];
    for l in &LINHAS {
        let largura = if l.teto.is_some() { 1 } else { 3 };
        for v in visto.iter_mut().skip(l.slot as usize).take(largura) {
            *v += 1;
        }
    }
    // ⚠️⚠️ **A RESERVA sai da varredura, e a exclusão é DERIVADA de [`wgsl::RESERVADAS`]** — a
    // arrumação tem `5` cores e `7` escalares (`22` floats) e o alinhamento de `vec4` pede `24`.
    // ⛔ **E ela vem com a metade que a impede de ser uma licença:** uma posição reservada não pode
    // ser reclamada por fileira nenhuma. *Sem isso, «excluir da varredura» passaria a esconder um
    // botão que aterrou na reserva e é inalcançável.*
    for r in wgsl::RESERVADAS {
        assert_eq!(
            visto[r], 0,
            "a posição {r} é RESERVA e uma fileira reclamou-a"
        );
    }
    // ⭐⭐⭐ **E há uma TERCEIRA categoria, que não é fileira nem reserva:** a posição do PASSO da
    // curvatura do estilo ([`wgsl::EPS_DO_ESTILO`]), cujo dono é a **montagem do dispositivo** e não
    // o `Style` — ela precisa do raio da PEÇA, que o estilo não tem.
    //
    // ⛔ **E a excepção vem com a metade que a impede de ser uma licença:** o [`wgsl::pack`] tem de
    // a deixar a ZERO. *Se ele a preenchesse, haveria dois escritores para a mesma posição, e o
    // último a correr ganharia sem ninguém saber qual é.*
    assert_eq!(
        visto[wgsl::EPS_DO_ESTILO],
        0,
        "o passo do estilo é da MONTAGEM e uma fileira reclamou-o"
    );
    assert!(
        (wgsl::pack(&Style::default())[wgsl::EPS_DO_ESTILO] - wgsl::RESERVA).abs() < f32::EPSILON,
        "o `pack` preencheu a posição que a montagem escreve: dois escritores, um valor"
    );
    let orfas: Vec<usize> = (0..wgsl::PACKED)
        .filter(|&i| visto[i] == 0)
        .filter(|i| !wgsl::RESERVADAS.contains(i) && *i != wgsl::EPS_DO_ESTILO)
        .collect();
    assert!(orfas.is_empty(), "posições sem linha nenhuma: {orfas:?}");
    let repetidas: Vec<usize> = (0..wgsl::PACKED).filter(|&i| visto[i] > 1).collect();
    assert!(
        repetidas.is_empty(),
        "posições com DUAS linhas — duas superfícies sobre um valor: {repetidas:?}"
    );
}

/// ⭐⭐⭐ **FORA DO RENDER NÃO HÁ FILEIRA** — *uma affordance que não pode ser honrada é pior do que
/// nenhuma*, e no matcap o estilo não corre.
#[test]
fn no_matcap_a_seccao_nao_e_oferecida() {
    assert!(
        rows(Style::default(), false).is_empty(),
        "o matcap oferece estilo"
    );
    assert_eq!(rows(Style::default(), true).len(), LINHAS.len());
    // ⚠️ E a secção abre-se UMA vez — duas cabeçalhos seriam duas secções com o mesmo nome.
    let cabecalhos = rows(Style::default(), true)
        .iter()
        .filter(|r| r.section == Some(SECCAO))
        .count();
    assert_eq!(cabecalhos, 1, "a secção abre {cabecalhos} vezes");
}

/// ⭐ **O tecto de cada número é o que a tabela declara** — e o da largura sai da LEI, não daqui.
///
/// ⚠️ *Um tecto escrito duas vezes é a segunda resposta que envelhece*: a largura do contorno tem
/// dono (`ph2d_style::Rim::MAX_WIDTH`), e este gate prende as duas pontas.
#[test]
fn o_tecto_da_largura_sai_da_lei() {
    let largura = LINHAS
        .iter()
        .find(|l| l.key == "panel.model3d.style.rim_width")
        .expect("a linha da largura");
    assert!(
        (largura.teto.expect("é um número") - ph2d_style::Rim::MAX_WIDTH).abs() < f32::EPSILON,
        "o tecto da largura no painel não é o da lei"
    );
}

/// ⭐⭐⭐ **CADA COR DO ESTILO TEM UMA AMOSTRA SÓ SUA** (report do Enio, 2026-09-19: *«se modifico
/// qualquer cor em style, todas mudam ao mesmo tempo»*).
///
/// # ⛔⛔ O defeito, e porque os cinco gates acima ficaram VERDES por cima dele
///
/// O selector de cor da casa é **um** e flutua; um painel entra nele registando o `NodeId` da
/// amostra. O id era cunhado `(entidade, campo)`, e as cinco cores do estilo têm `entity = 0` (o
/// estilo não é de entidade nenhuma) **e** caíam no braço final do `match`, que respondia `campo =
/// 0` ⇒ as cinco partilhavam `hash("model3d.color.swatch.0.0")`. Com o selector aberto numa, as
/// cinco liam *«aberto em mim»* e as cinco pediam a escrita.
///
/// ⚠️⚠️ **Os gates acima medem a LEI e o DRENO, e o defeito vive ENTRE os dois** — na identidade com
/// que a fileira é pintada. O gate da costura alimenta o dreno com a âncora já certa, logo ele entra
/// **abaixo** da rotura: é a lei que o `CLAUDE.md` §5.0 escreve como *«nenhum instrumento pergunta
/// se o VALOR chega a um consumidor»*, aqui na forma *«nenhum perguntava se duas fileiras são o
/// MESMO controlo»*.
#[test]
fn cada_cor_do_estilo_tem_uma_amostra_so_sua() {
    let fileiras = rows(Style::default(), true);
    let cores: Vec<_> = fileiras.iter().filter(|r| r.swatch.is_some()).collect();
    // ⚠️ **Piso de população**: sem ele, uma varredura que deixasse de achar cor nenhuma ficaria
    // trivialmente verde — e a secção tem cinco.
    assert_eq!(
        cores.len(),
        LINHAS.iter().filter(|l| l.teto.is_none()).count(),
        "a varredura não achou as cores do estilo"
    );
    let mut vistos: Vec<(u64, &'static str)> = Vec::new();
    for c in &cores {
        let id = ph2d_panel_model3d::swatch_id(c)
            .unwrap_or_else(|| panic!("{}: uma cor sem amostra é uma cor inalcançável", c.key))
            .0;
        if let Some((_, quem)) = vistos.iter().find(|(v, _)| *v == id) {
            panic!(
                "{} e {} são o MESMO controlo (id {id}) — mexer numa mexe na outra",
                quem, c.key
            );
        }
        vistos.push((id, c.key));
    }
}

/// ⭐⭐ **E a amostra do estilo NUNCA é a de um objecto** — a não-colisão entre as duas famílias.
///
/// ⛔ Sem este gate, a segurança do sentinela `entity = 0` dependeria do acidente de
/// `Entity::to_bits()` nunca valer zero. *Uma propriedade que vale por acidente é a que cai no dia
/// em que outra crate muda uma representação* — aqui ela é inexprimível: os dois nomes diferem.
#[test]
fn a_amostra_do_estilo_nao_colide_com_a_de_um_objecto() {
    let alheias: Vec<ph2d_panel_model3d::ParamRow> = [0u8, 4, 8, 12, 16]
        .into_iter()
        .flat_map(|k| [Param::Material(k), Param::Light(k)])
        .map(|param| ph2d_panel_model3d::ParamRow {
            entity: 0,
            param,
            key: "x",
            value: 0.0,
            lo: 0.0,
            bound: Bound::Soft(1.0),
            inert: None,
            integral: false,
            choices: &[],
            section: None,
            swatch: Some([0, 0, 0]),
            subject: None,
        })
        .collect();
    let deles: Vec<u64> = alheias
        .iter()
        .filter_map(|r| ph2d_panel_model3d::swatch_id(r).map(|i| i.0))
        .collect();
    assert_eq!(deles.len(), alheias.len(), "uma fileira de objecto sem id");
    for c in rows(Style::default(), true)
        .iter()
        .filter(|r| r.swatch.is_some())
    {
        let meu = ph2d_panel_model3d::swatch_id(c)
            .expect("a amostra do estilo")
            .0;
        assert!(
            !deles.contains(&meu),
            "{}: a amostra do estilo tem o id de uma cor de objecto",
            c.key
        );
    }
}

/// ⭐⭐⭐ **O CLIQUE CHEGA AO ESTILO DA CENA — a costura, pelo DRENO do produto.**
///
/// # ⛔⛔ Porque este gate existe, e porque ele mede o dreno e não a função
///
/// *Um controlo nunca pintado e um MORTO SOB O DEDO dão o mesmo report*, e esta casa pagou essa
/// lição sete vezes só no módulo da escultura. Os gates acima medem a TABELA — eles ficariam verdes
/// com o braço do `match` a faltar, ou posto **depois** do genérico, que é o mesmo defeito escrito
/// de outra maneira.
///
/// ⚠️ **A ordem dos braços é load-bearing** e é ela que este gate prende: o braço genérico do
/// `SetColor` casa com QUALQUER âncora, e um braço de estilo a seguir a ele seria inalcançável.
#[test]
fn o_dreno_do_produto_escreve_no_estilo_da_cena() {
    // ⚠️ **O módulo tem de estar ARMADO** — o `with_smoke` devolve `None` a um módulo fechado, e
    // um gate que ignorasse esse `None` mediria o nada. (A 1.ª redacção deste gate estoirou aqui, o
    // que é o modo de falha bom.)
    crate::smoke::set_armed_by_panel(true);
    crate::smoke::with_smoke(|s| s.set_style(Style::default()));
    let mut world = bevy_ecs::world::World::new();

    // (1) um NÚMERO — a nitidez da ARESTA (a `slot 7`; a da cova é a `20`).
    ph2d_panel_model3d::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetParam {
        entity: 0,
        param: Param::Style(7),
        value: 3.5,
    });
    crate::scene::apply_intents_for_test(&mut world, &[]);
    let lido = crate::smoke::with_smoke(|s| s.style).expect("a cena");
    assert!(
        (lido.curvature.edge_sharpness - 3.5).abs() < 1e-6,
        "o clique num número não chegou ao estilo: {}",
        lido.curvature.edge_sharpness
    );

    // (2) e uma COR — a tinta da aresta, pela âncora.
    ph2d_panel_model3d::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetColor {
        entity: 0,
        anchor: Param::Style(4),
        srgb: [255, 0, 0],
    });
    crate::scene::apply_intents_for_test(&mut world, &[]);
    let lido = crate::smoke::with_smoke(|s| s.style).expect("a cena");
    assert!(
        lido.curvature.convex[0] > lido.curvature.convex[1],
        "o clique numa cor não chegou ao estilo: {:?}",
        lido.curvature.convex
    );
    // ⚠️ **E o número escrito antes SOBREVIVEU** — sem esta metade, um dreno que reescrevesse o
    // estilo inteiro a cada intent passaria.
    assert!(
        (lido.curvature.edge_sharpness - 3.5).abs() < 1e-6,
        "a escrita da cor apagou o número que estava lá"
    );

    // ⚠️⚠️ **CONTROLO:** o mesmo dreno com um param de OUTRA família não pode tocar no estilo.
    //
    // ⛔ **E ele leva uma entidade DE VERDADE**, não o `0` das linhas de estilo: a 1.ª redacção
    // passou `0` e o `Entity::from_bits(0)` fez **pânico** dentro do braço genérico. *É o modo de
    // falha bom* — e diz uma coisa sobre o desenho: o `0` das linhas de estilo só é seguro porque
    // os braços delas vêm ANTES, e no dia em que alguém os mover o produto **estoura em vez de
    // escrever no sítio errado em silêncio**.
    let alheia = world.spawn_empty().id();
    ph2d_panel_model3d::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetParam {
        entity: alheia.to_bits(),
        param: Param::Material(0),
        value: 0.123,
    });
    crate::scene::apply_intents_for_test(&mut world, &[]);
    let depois = crate::smoke::with_smoke(|s| s.style).expect("a cena");
    assert_eq!(depois, lido, "um param de outra família mexeu no estilo");
    crate::smoke::with_smoke(|s| s.set_style(Style::default()));
}

/// ⭐⭐⭐ **UMA FILEIRA QUE NÃO PODE FAZER NADA DIZ PORQUÊ** (report do dono, 2026-09-19: *«Zone
/// pivot não sei para que serve mas parece morto»*).
///
/// # ⛔⛔ Ele estava certo, e a lista era maior do que ele viu
///
/// Na configuração em que o painel ABRE, quatro fileiras desta secção são inertes **por
/// construção** — e as dez shipavam `inert: None`. O `Zone Pivot` é o caso literal: a lei é
/// `sombra + (luz − sombra)·h`, e com as duas tintas brancas o parêntesis é **exactamente zero**,
/// *a mesma lei que faz a omissão ser a identidade ao bit*.
///
/// ⚠️ **É a decisão do dono de 2026-09-18**, que o cabeçalho deste ficheiro citava e não cumpria.
///
/// **Mutação que deve sangrar:** `inert: None` de volta no construtor da `ParamRow`.
#[test]
fn uma_fileira_inerte_diz_porque_esta_apagada() {
    // (1) ⭐ O ESTADO EM QUE O PAINEL ABRE — e a lista é **derivada**, nunca escrita à mão.
    let fabrica = rows(Style::default(), true);
    let apagadas: Vec<&str> = fabrica
        .iter()
        .filter(|r| r.inert.is_some())
        .map(|r| r.key)
        .collect();
    assert!(
        apagadas.len() >= 4,
        "de fábrica esperava pelo menos quatro fileiras apagadas, achei {apagadas:?}"
    );
    // ⚠️ **Piso de população do outro lado**: se TODAS estivessem apagadas a secção não teria por
    // onde se começar a usar, e a asserção acima ficaria trivialmente verdadeira.
    assert!(
        apagadas.len() < fabrica.len(),
        "de fábrica a secção inteira está apagada: não há por onde começar"
    );
    // ⭐ E o pivô é uma delas — é o report, à letra.
    assert!(
        apagadas.contains(&"panel.model3d.style.pivot"),
        "o Zone Pivot abre vivo e ele é inerte de fábrica: {apagadas:?}"
    );

    // (2) ⭐⭐ E CADA UMA ACORDA COM O GESTO QUE A RAZÃO NOMEIA — a metade que impede a primeira de
    //     ser uma licença para apagar tudo.
    let acordado = Style {
        rim: ph2d_style::Rim {
            strength: 1.0,
            ..ph2d_style::Rim::default()
        },
        curvature: ph2d_style::Curvature {
            convex: [1.0, 0.5, 0.5],
            concave: [0.5, 0.5, 1.0],
            ..ph2d_style::Curvature::default()
        },
        zones: ph2d_style::Zones {
            highlight: [1.0, 0.9, 0.7],
            ..ph2d_style::Zones::default()
        },
        ..Style::default()
    };
    let vivas = rows(acordado, true);
    let ainda: Vec<&str> = vivas
        .iter()
        .filter(|r| r.inert.is_some())
        .map(|r| r.key)
        .collect();
    assert!(
        ainda.is_empty(),
        "com os quatro gestos feitos ainda há fileiras apagadas: {ainda:?}"
    );

    // (3) ⭐ A RAZÃO é uma CHAVE que a tabela de textos conhece — uma chave órfã pinta o nome dela.
    for r in &fabrica {
        if let Some(k) = r.inert {
            let frase = ph2d_i18n::tr(k);
            assert_ne!(frase, k, "a razão «{k}» não está na tabela de textos");
            assert!(
                frase.starts_with("Inactive:"),
                "a razão de «{}» não começa por «Inactive:»: {frase}",
                r.key
            );
        }
    }
}

/// ⭐⭐⭐ **O ROTEIRO DA CENA NOMEIA CONTROLOS QUE EXISTEM** — a lei que esta jornada pagou.
///
/// # ⛔⛔⛔ Porque ele nasceu: a FOTO, e não a suíte
///
/// A wave de 2026-09-19 renomeou `Rim Width` para **`Rim Falloff`** (o nome estava ao contrário:
/// subir «Width» ESTREITAVA) e partiu `Curvature Sharpness` em **`Edge Sharpness`** e
/// **`Cavity Sharpness`**. O roteiro da `=35` continuou a mandar o dono carregar nos **três nomes
/// antigos**, e a suíte inteira fechou verde: *um `println!` não é compilado contra nada*.
///
/// ⚠️ **Quem o apanhou foi a fotografia da cena** (`fotografa_cena.sh`), que imprime o roteiro ao
/// lado da imagem — a mesma régua que o `#16` e o `#18` do TOP-20 já tinham pago. ⇒ *um passo que
/// nomeia um controlo AFIRMA que ele está na tela, e o dono aprova o smoke com o passo impossível
/// dentro.*
///
/// # ⭐⭐ A régua é DERIVADA, e a lista de excepções é NOMEADA
///
/// Os rótulos legítimos saem da **mesma tabela que pinta as fileiras** ([`LINHAS`]) — nunca de uma
/// segunda lista, que divergiria no dia seguinte. O que o roteiro pode dizer além deles é uma lista
/// **explícita** de palavras que não são controlos desta secção (o modo, o pill, o nome da secção).
///
/// ⚠️ **O piso de população é obrigatório:** sem ele, um extractor partido colhe zero candidatos e
/// o gate fica verde a medir nada — a falha muda que o `CLAUDE.md` §5.0 nomeia.
#[test]
fn o_roteiro_da_cena_nomeia_controlos_que_existem() {
    // ⛔⛔⛔ **A JANELA DA 1.ª REDACÇÃO ESCAPAVA PARA O FIM DO FICHEIRO, e isso passou despercebido
    // por um dia inteiro.** Ela era `fonte[i..]` cortada no *primeiro* `pub fn cena_35` **a seguir**
    // ao `println!` — e aquela assinatura vem ANTES dele, logo o `find` devolvia `None` e o
    // `map_or(fonte.len(), …)` lia do roteiro **até ao fim do ficheiro**.
    //
    // ⚠️ *Enquanto a `=35` foi a última cena do ficheiro isso não se via*: a janela errada e a certa
    // continham o mesmo texto. Quem o expôs foi a cena `=36`, cujos doc-comments a janela passou a
    // engolir — e o gate acusou `Curvature Sharpness` (um nome ANTIGO que um comentário guarda de
    // propósito) e `Com 2  8  32`. *Uma janela que se estende até ao fim do ficheiro e uma janela
    // certa leem-se iguais enquanto o ficheiro acabar ali.*
    //
    // ⇒ a colheita passa a ser **derivada e de TODOS os roteiros deste ficheiro**: as linhas dentro
    // de um `println!` que anuncia um passo, e nada mais. ⛔ Um doc-comment não entra — ele guarda a
    // história, e citar um nome morto é o trabalho dele.
    let fonte = include_str!("smoke_scenes_edge.rs");
    let mut roteiro = String::new();
    let mut dentro = false;
    for linha in fonte.lines() {
        let podada = linha.trim_start();
        // ⛔ Um comentário nunca entra, nem quando repete a marca.
        if podada.starts_with("//") {
            continue;
        }
        if podada.contains("[field-smoke]") {
            dentro = true;
        }
        if dentro {
            roteiro.push_str(linha);
            roteiro.push(' ');
            if podada.contains(");") {
                dentro = false;
            }
        }
    }
    // ⚠️ Piso de população: sem ele, uma marca renomeada faria a colheita devolver o vazio e o gate
    // ficaria verde a medir nada.
    assert!(
        roteiro.len() > 2000,
        "a colheita dos roteiros devolveu {} bytes — ela partiu-se",
        roteiro.len()
    );
    let roteiro = roteiro.as_str();

    // ⭐⭐⭐ **A POPULAÇÃO É TODA SECÇÃO DA CENA, e a premissa «só há o Style» MORREU em 2026-09-19.**
    //
    // ⛔ A 1.ª redacção lia só a tabela deste módulo, e reprovou no dia em que o roteiro da `=36`
    // passou a nomear *Bloom*, *Threshold* e *Size 1*. ⚠️ **A cura barata era acrescentá-los ao
    // `FORA`** — e isso é exactamente como uma lista de excepções cresce até não medir nada: ela
    // passaria a aprovar um roteiro que nomeasse um controlo do brilho que NÃO existe.
    //
    // ⇒ os rótulos vivos são a UNIÃO das secções que o [`crate::scene_panel`] apende ao retrato, e
    // cada uma entra pela **própria** tabela. *Uma terceira secção reprova este gate até alguém a
    // juntar aqui, que é a diferença entre uma lista que alguém tem de se lembrar de estender e uma
    // que não fica verde sem a extensão.*
    let mut vivos: std::collections::BTreeSet<String> = LINHAS
        .iter()
        .map(|l| ph2d_i18n::tr(l.key).to_string())
        .collect();
    vivos.extend(
        crate::brilho_painel::rows(ph2d_field_render::Bloom::default(), true, false)
            .iter()
            .map(|r| ph2d_i18n::tr(r.key).to_string()),
    );
    // ⭐⭐⭐ **E OS DO MATERIAL**, que a janela corrigida passou a alcançar: com ela certa, este gate
    // deixou de ler só o roteiro da `=35` e passou a ler os de TODAS as cenas deste ficheiro — e a
    // `=33` manda carregar em *Thin Walled* e *Subsurface Radius*, que são fileiras do material.
    //
    // ⭐ **A porta já existia e o doc dela declara este uso por escrito**
    // ([`ph2d_field_ecs::material_key`]: *«ela existe para um CENSO poder nomear o que mede sem
    // escrever uma segunda lista»*). ⛔ Uma lista à mão aqui seria essa segunda lista.
    for k in 0..ph2d_field::MATERIAL_FIELDS {
        if let Some(chave) = ph2d_field_ecs::material_key(k) {
            vivos.insert(ph2d_i18n::tr(chave).to_string());
        }
    }
    // ⚠️ E as ESCOLHAS de uma fileira também são rótulos que um roteiro pode nomear.
    for chave in ["field.dim.thin_walled_no", "field.dim.thin_walled_yes"] {
        vivos.insert(ph2d_i18n::tr(chave).to_string());
    }
    assert!(
        vivos.len() >= 10 + 11 + 30,
        "a população encolheu: {} rótulos entre as secções deste painel",
        vivos.len()
    );

    // ⛔ **As palavras do roteiro que NÃO são controlos desta secção**, uma a uma e com o porquê.
    const FORA: &[&str] = &[
        "Shading", // o modo de sombreamento, não um controlo da secção
        "Render",  // idem
        "Matcap",  // idem
        "Model",   // o separador do painel, no canto superior direito
    ];

    // ⭐⭐⭐ **A RÉGUA É A CORRIDA CONTER UM RÓTULO — e a 1.ª redacção comparava a corrida INTEIRA.**
    //
    // ⛔ Ela colhia pares de `Palavra Palavra` em Maiúscula Inicial e exigia que o PAR fosse um
    // rótulo. Isso partiu-se em 2026-09-19 sobre um roteiro CERTO: a frase *«Suba Size 5 …»* forma
    // o par `Suba Size`, porque o extractor recusa dígitos e o rótulo real é **`Size 5`**.
    //
    // ⚠️⚠️ **A cura barata era pôr `Suba` no [`FORA`]**, e é assim que uma lista de excepções cresce
    // até não medir nada: com ela lá dentro, um roteiro que mandasse carregar num `Suba Tamanho`
    // inexistente passaria. ⇒ a régua passa a ser *«esta corrida CONTÉM um rótulo vivo»*, que é
    // estritamente mais forte — ela continua a apanhar o defeito de origem (um `Rim Width` cujo
    // rótulo virou `Rim Falloff` não contém rótulo nenhum) e deixa de acusar o português à volta.
    let e_rotulo_ish = |w: &str| {
        // ⛔⛔ **O VAZIO tem de sair PRIMEIRO, e a 1.ª redacção não o tirava:** `"".chars().all(…)`
        // é **vacuamente verdadeiro**, logo toda a pontuação — que o `trim_matches` abaixo reduz a
        // uma string vazia — entrava nas corridas e colava-as umas às outras. *Um `all` sobre um
        // conjunto vazio é a forma mais silenciosa de um predicado dizer que sim.*
        if w.is_empty() {
            return false;
        }
        let mut c = w.chars();
        let inicial = c.next().is_some_and(char::is_uppercase) && c.clone().all(char::is_lowercase);
        (inicial && w.chars().all(char::is_alphabetic) && w.len() > 2)
            // ⭐ Os DÍGITOS entram, porque um rótulo pode tê-los (`Size 5`) — e era exactamente
            // recusá-los que fazia o extractor inventar um par que o texto não contém.
            || w.chars().all(|c| c.is_ascii_digit())
    };
    // As corridas MÁXIMAS de palavras assim.
    let mut corridas: Vec<Vec<&str>> = Vec::new();
    let mut corrente: Vec<&str> = Vec::new();
    for w in roteiro.split_whitespace() {
        // ⚠️ A pontuação encosta na palavra (`Render.`, `STYLE,`): tira-se para a decidir, e o que
        // entra na corrida é a palavra limpa.
        let limpa = w.trim_matches(|c: char| !c.is_alphanumeric());
        if e_rotulo_ish(limpa) {
            corrente.push(limpa);
        } else {
            if corrente.len() >= 2 {
                corridas.push(std::mem::take(&mut corrente));
            } else {
                corrente.clear();
            }
        }
    }
    if corrente.len() >= 2 {
        corridas.push(corrente);
    }
    assert!(
        corridas.len() >= 6,
        "o extractor colheu {} corridas — ele partiu-se e o gate mediria o nada",
        corridas.len()
    );

    // ⚠️ Um rótulo pode ter parêntesis (`Size 1 (finest)`), e o roteiro nunca os escreve: a
    // comparação é pelas PALAVRAS do rótulo até ao 1.º parêntesis.
    let nucleo = |s: &str| -> Vec<String> {
        s.split_whitespace()
            .take_while(|w| !w.starts_with('('))
            .map(str::to_string)
            .collect()
    };
    let nucleos: Vec<Vec<String>> = vivos.iter().map(|v| nucleo(v)).collect();
    let contem_rotulo = |c: &[&str]| {
        nucleos.iter().any(|n| {
            !n.is_empty()
                && c.windows(n.len())
                    .any(|j| j.iter().zip(n).all(|(a, b)| a == b))
        })
    };

    let orfaos: Vec<String> = corridas
        .iter()
        // ⛔ **Uma corrida só de DÍGITOS nunca é um rótulo** — é prosa («as luzes valem 2, 6 e 18»).
        // Os dígitos entraram no extractor porque um rótulo pode tê-los (`Size 5`), e sem esta
        // linha ele colhia `2 6` e acusava o roteiro de nomear um controlo inexistente.
        // ⚠️ *A cura barata era pôr «2 6» no [`FORA`]*, e essa lista passaria a crescer com cada
        // número que alguém escrevesse num roteiro.
        .filter(|c| c.iter().any(|w| w.chars().any(char::is_alphabetic)))
        .filter(|c| !contem_rotulo(c))
        .filter(|c| !c.iter().any(|w| FORA.contains(w)))
        .map(|c| c.join(" "))
        .collect();
    assert!(
        orfaos.is_empty(),
        "o roteiro manda carregar em controlos que a cena não tem: {orfaos:?}\n  vivos: {vivos:?}"
    );
}
