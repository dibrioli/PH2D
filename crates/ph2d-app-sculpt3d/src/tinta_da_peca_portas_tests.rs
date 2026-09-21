//! ⭐⭐⭐⭐ **AS PORTAS QUE DECIDEM** — os gates das funções PURAS deste
//! módulo, cortados do irmão [`super::tests`] pelo tecto de LOC (`706` contra
//! `700`) e pelo ASSUNTO.
//!
//! Lá mora **o PLANO** — como ele nasce, como ele volta à peça, o que ele pesa,
//! onde está o tecto dele. Aqui moram **as decisões que não têm pixel nenhum**:
//! de onde o quadro lê o plano ([`crate::tinta_da_peca::rota`]), se este gesto
//! mexe na topologia, se o passe corre neste pen-down, e se basta subir as
//! amostras.
//!
//! ⚠️ **Elas são puras de propósito, e essa é a razão de este ficheiro existir
//! do lado dos testes normais:** cada uma dessas escolhas vive num laço que
//! pede um `wgpu::Device`, e um gate ali nasceria `#[ignore]` — *o CI nunca
//! corre um `#[ignore]`*. Tirá-las para funções puras é o que as torna
//! alcançáveis pela varredura que corre em toda máquina.

use super::*;

/// ⭐⭐⭐ **GATE — QUEM SEGURA O PLANO, e o que se reconcilia.**
///
/// ⛔⛔ **As duas leis desta porta têm modo de falha caro e INVISÍVEL a toda
/// régua de cor**, e é por isso que ela é pura:
///
/// 1. **durante um traço não se reconcilia nada** — ali o `Option` da peça está
///    VAZIO (o empréstimo é um `take`), logo um `garante` construiria um plano
///    BRANCO novo em cada quadro, que o `close_stroke` sobrescreveria. *Um
///    alocador de dezenas de MB a 60 Hz que nenhuma imagem acusa.*
/// 2. **só a peça ACTIVA ganha um plano novo** — armar o knob não pode
///    multiplicar `64` amostras por vértice por toda a cena.
///
/// ⚠️ **E a terceira metade é a que o artista vê:** com o traço a segurar, o
/// upload lê o GESTO. Lendo a peça ele subiria `armado = 0` e *a tinta fina
/// desapareceria no instante em que o artista começasse a pintar*.
#[test]
fn a_rota_diz_quem_segura_o_plano_e_o_que_se_reconcilia() {
    // (1) A activa com o traço a segurar: não se reconcilia, e lê-se o gesto.
    assert_eq!(rota(true, true, false, Some(2)), Rota::Emprestado);
    // ⚠️ **Mesmo que a peça AINDA tenha um plano** — o caso do quadro em que o
    // pen-down já correu mas a peça não foi tocada: a resposta é a mesma.
    assert_eq!(rota(true, true, true, Some(2)), Rota::Emprestado);

    // (2) Uma peça que NÃO é a activa não ganha plano, mesmo com o knob armado.
    assert_eq!(
        rota(false, false, false, Some(3)),
        Rota::DaPeca { pedir: None }
    );
    // ⛔ **O CONTROLO:** se ela já tem um, mantém-se no nível do knob — largá-lo
    //    faria a tinta de uma peça sumir por o artista ter escolhido outra.
    assert_eq!(
        rota(false, false, true, Some(3)),
        Rota::DaPeca { pedir: Some(3) }
    );

    // (3) A activa sem traço: reconcilia com o que o knob diz, nos dois lados.
    assert_eq!(
        rota(true, false, false, Some(1)),
        Rota::DaPeca { pedir: Some(1) }
    );
    assert_eq!(rota(true, false, true, None), Rota::DaPeca { pedir: None });

    // ⛔⛔ **E um traço NOUTRA peça não empresta nada a esta** — sem a cerca do
    //    `e_a_activa` o laço leria o plano do gesto para TODA peça do quadro, e
    //    o device desenharia a tinta da peça activa nas vizinhas.
    assert_eq!(
        rota(false, true, true, Some(2)),
        Rota::DaPeca { pedir: Some(2) }
    );
}

/// ⚠️⚠️ **GATE — o LAÇO DE UPLOAD percorre a porta, e não uma cópia da lei.**
///
/// ⛔ *Um gate que chama a função em vez de percorrer a rota afirma que a lei
/// existe, nunca que o quadro a usa* — a frase que o §24 desta linha pagou e
/// que ela própria violou uma wave depois. O laço pede um `wgpu::Device`, logo
/// a única régua que corre no CI é o TEXTO; ele é lido por [`include_str!`],
/// que **deixa de compilar** se o irmão mudar de ficheiro.
#[test]
fn o_laco_de_upload_pergunta_a_porta_de_onde_ler_o_plano() {
    const SLOTS: &str = include_str!("slots.rs");
    let codigo: Vec<&str> = SLOTS
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect();
    assert!(
        codigo.iter().any(|l| l.contains("fn sync_mesh(")),
        "a extracção de código partiu-se: o laço não está nas {} linhas colhidas",
        codigo.len()
    );
    for agulha in [
        "tinta_da_peca::rota(",
        "tinta_da_peca::Rota::DaPeca { pedir }",
        "tinta_da_peca::Rota::Emprestado",
        "upload_tinta_at(",
    ] {
        assert!(
            codigo.iter().any(|l| l.contains(agulha)),
            "o laço de upload deixou de conter `{agulha}` -- ou ele parou de \
             perguntar a' porta, ou parou de subir o plano"
        );
    }
    // ⛔ **E a metade NEGATIVA:** o laço não pode voltar a escrever a decisão à
    //    mão. Com a lei em dois sítios, o gate acima passa a medir a porta
    //    enquanto o produto corre a cópia.
    assert!(
        !codigo
            .iter()
            .any(|l| l.contains("i == self.active && self.stroke.tinta_fina")),
        "a decisão de quem segura o plano voltou a ser escrita dentro do laço: \
         uma lei escrita em dois sítios ainda não é uma lei"
    );
}

/// ⭐⭐⭐⭐ **GATE — COM O PLANO ARMADO, UM PINCEL DE COR DEIXA DE MEXER NA
/// TOPOLOGIA** — o report do dono de 2026-09-20 (*«traços posteriores estão
/// reduzindo a resolução dos traços em alta resolução anteriores»*).
///
/// ⚠️ **A população que a resposta muda é CONTADA e não escrita à mão**, e é
/// ela a lei: *o plano protege exactamente os verbos de COR*. Uma lista à mão
/// aqui seria a segunda resposta a [`ph2d_sculpt3d::Verb::paints_color`], e as
/// duas divergem no dia em que nascer o quarto pincel de cor.
///
/// ⛔ **Os TRÊS controlos, e cada um recusa uma cura barata:**
///
/// * **sem plano** os mesmos verbos continuam a refinar — senão a cura teria
///   sido *«pincel de cor nunca refina»*, que apaga a ordem do dono de 19/09;
/// * um verbo de **FORMA** refina com o plano armado — senão o plano teria
///   desarmado o passe inteiro, e o artista que esculpe deixaria de o ter;
/// * um verbo que **nunca** mexe na topologia continua a não mexer — senão a
///   porta estaria a inventar um `true` que a tabela do dyntopo não tem.
#[test]
fn com_o_plano_armado_um_pincel_de_cor_nao_mexe_na_topologia() {
    use ph2d_sculpt3d::Verb;

    let muda: Vec<Verb> = Verb::ALL
        .into_iter()
        .filter(|v| o_gesto_muda_a_topologia(*v, false) != o_gesto_muda_a_topologia(*v, true))
        .collect();
    let de_cor: Vec<Verb> = Verb::ALL.into_iter().filter(|v| v.paints_color()).collect();
    assert_eq!(
        muda, de_cor,
        "o plano muda a resposta de {muda:?} e os verbos de cor são {de_cor:?} -- \
         a lei é *o plano protege exactamente quem pinta*, e ela deixou de \
         descrever o produto"
    );
    assert!(
        !de_cor.is_empty(),
        "a população é VAZIA: este gate estaria verde a afirmar nada"
    );

    // ⭐ CONTROLO 1 — sem plano, quem pinta continua a refinar (ordem do dono
    //   de 2026-09-19, que esta wave NÃO revoga).
    for v in &de_cor {
        assert!(
            o_gesto_muda_a_topologia(*v, false),
            "{v:?} deixou de refinar mesmo SEM plano -- isso apaga a ordem de 19/09"
        );
        assert!(
            !o_gesto_muda_a_topologia(*v, true),
            "{v:?} ainda refina com o plano armado -- é o report de 20/09"
        );
    }

    // ⭐ CONTROLO 2 — um verbo de FORMA não é tocado pelo plano.
    assert!(
        o_gesto_muda_a_topologia(Verb::Draw, true) && o_gesto_muda_a_topologia(Verb::Draw, false),
        "o plano desarmou o passe para um verbo de FORMA: quem esculpe com \
         topologia dinâmica perdeu-a"
    );
    // ⭐ CONTROLO 3 — e a porta não inventa um `true` onde a tabela diz não.
    assert!(
        !o_gesto_muda_a_topologia(Verb::Smooth, true)
            && !o_gesto_muda_a_topologia(Verb::Smooth, false),
        "o alisador passou a mexer na topologia -- a porta está a decidir em vez \
         de delegar na tabela do dyntopo"
    );
}

/// ⭐⭐⭐⭐ **A PORTA DO PEN-DOWN, nas DUAS configurações em que as duas lentes
/// discordavam.**
///
/// ⛔⛔ Medido em 2026-09-21: a voz do pen-down perguntava só pelo INTERRUPTOR
/// e o consumidor ([`crate::Sculpt3dScene::open_dyntopo_stroke`]) corre com
/// `interruptor || verbo.corre_sem_o_interruptor()` **e** com a pilha por
/// montar. As duas células abaixo são, cada uma, um erro de sinal diferente —
/// *um falso negativo e um falso positivo na mesma condição*.
///
/// ⚠️ **A metade do CONTROLO é o que impede a cura de virar «responde sempre
/// sim»**: a configuração normal continua a responder o que respondia.
#[test]
fn o_passe_do_pen_down_tem_as_tres_metades() {
    use ph2d_sculpt3d::Verb;

    // (a) ⛔ O FALSO NEGATIVO: o `Density` corre SEM o interruptor.
    assert!(
        Verb::Density.corre_sem_o_interruptor(),
        "o CONTROLO desta célula: sem isto ela não mede nada"
    );
    assert!(
        o_passe_corre_no_pen_down(Verb::Density, false, 1, true),
        "com o interruptor DESLIGADO o Density mexe na topologia à mesma — \
         a voz que perguntasse só pelo interruptor ficaria calada"
    );

    // (b) ⛔ O FALSO POSITIVO: com a pilha montada os dois motores recusam.
    assert!(
        !o_passe_corre_no_pen_down(Verb::Draw, true, 2, true),
        "com uma pilha de multiresolução o passe NÃO corre — avisar aqui é \
         pôr o preço de um gesto que ninguém paga"
    );

    // (c) ⭐ O CONTROLO: a configuração normal não mudou de resposta.
    assert!(
        o_passe_corre_no_pen_down(Verb::Draw, true, 1, true),
        "o caso do produto: interruptor ligado, uma peça sem pilha, verbo de FORMA"
    );
    assert!(
        !o_passe_corre_no_pen_down(Verb::Draw, false, 1, true),
        "sem interruptor e sem ser o Density, um verbo de forma não corre o passe"
    );
    assert!(
        !o_passe_corre_no_pen_down(Verb::Paint, true, 1, true),
        "um verbo de COR com o plano armado não mexe na topologia (a cura de 20/09)"
    );
    assert!(
        o_passe_corre_no_pen_down(Verb::Paint, true, 1, false),
        "e SEM o plano armado ele volta a mexer — senão esta porta apagava a metade \
         que a cura de 20/09 existe para preservar"
    );
}

/// ⭐⭐⭐⭐ **O ATALHO DO UPLOAD tem QUATRO cercas, e a quarta nasceu de um
/// defeito MUDO.**
///
/// ⛔⛔ Medido em 2026-09-21: a cerca `!mexeu` não diz *«a topologia não
/// mudou»* — ela é `!dirty.is_empty()`, e o `mesh_rebuilt` (que TODA mudança
/// de topologia chama) faz `dirty.clear()`. ⇒ depois da triangulação do
/// pen-down de um verbo com ÂNCORA — que **não carimba**, logo não enche o
/// `dirty` — o atalho disparava, a `upload_tinta_at` nunca corria, e a tinta
/// ficava a descrever a malha de antes com a geometria já renovada.
///
/// ⚠️ **Cada asserção move UMA entrada e deixa as outras no valor que faz o
/// atalho disparar** — com duas a mexer ao mesmo tempo, uma cerca apagada
/// passa despercebida atrás da outra (a forma que a fixtura do payload pagou
/// nesta mesma jornada).
#[test]
fn o_atalho_do_upload_tem_quatro_cercas() {
    // ⭐ O CONTROLO: a configuração em que ele EXISTE para disparar — um traço
    // de cor a escrever amostras numa malha que o device já tem.
    assert!(
        so_as_amostras_bastam(true, false, false, false),
        "sem esta célula as quatro abaixo aprovavam um atalho que nunca dispara"
    );

    assert!(
        !so_as_amostras_bastam(false, false, false, false),
        "sem o plano EMPRESTADO não há traço a escrever amostras"
    );
    assert!(
        !so_as_amostras_bastam(true, true, false, false),
        "com a janela de vértices suja as POSIÇÕES mudaram, e elas viajam no plano"
    );
    assert!(
        !so_as_amostras_bastam(true, false, true, false),
        "com o plano sujo alguém pediu o plano INTEIRO"
    );
    assert!(
        !so_as_amostras_bastam(true, false, false, true),
        "⛔ a QUARTA: com a malha por subir o device NÃO tem esta malha — \
         e o `!mexeu` não o diz, porque o mesh_rebuilt limpa o próprio dirty"
    );
}

/// ⭐⭐⭐ **E o laço de upload LÊ a porta** — o elo que a torna mais do que uma
/// função certa sem chamador.
///
/// ⚠️ **Censo de TEXTO, e a razão é a de sempre nesta crate:** o laço pede um
/// `wgpu::Device`. ⛔ A prosa é cortada antes de se medir — *um doc-comment que
/// EXPLICA a cura contém o nome da porta, e um censo ingénuo lê-o como se fosse
/// a chamada*.
#[test]
fn o_laco_de_upload_pergunta_a_porta_se_bastam_as_amostras() {
    const SLOTS: &str = include_str!("slots.rs");
    let codigo: String = SLOTS
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let prosa: String = SLOTS
        .lines()
        .filter(|l| l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    // ⛔⛔⛔ **A AGULHA É A CHAMADA INTEIRA, e a 1.ª redacção não era.**
    // Ela perguntava só por `matches!(line.job, SlotJob::Full)` — e esse texto
    // aparece **duas** vezes neste ficheiro (a outra é a condição que decide se
    // vale a pena subir o plano) ⇒ a mutação que troca o ARGUMENTO por `false`
    // deixava o censo satisfeito pela OUTRA ocorrência, e **SOBREVIVEU**.
    // *Uma agulha que é um fragmento mede a presença do fragmento, não a da
    // chamada* — é a mesma lei que o arnês desta casa já escreve para as
    // âncoras de mutação (*«âncora = expressão inteira»*).
    let agulha = [
        "let so_as_amostras = crate::tinta_da_peca::so_as_amostras_bastam(",
        "                    emprestado,",
        "                    mexeu,",
        "                    self.objects[i].tinta_suja,",
        "                    matches!(line.job, SlotJob::Full),",
        "                )",
    ]
    .join("\n");
    assert!(
        codigo.contains(&agulha),
        "o laço de upload deixou de perguntar à porta com as QUATRO entradas.\n\
         Esperava, textualmente:\n{agulha}"
    );
    assert!(
        !prosa.contains(&agulha),
        "CONTROLO: a agulha foi encontrada na PROSA, logo este censo não mede fiação nenhuma"
    );
}
