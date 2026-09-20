//! **OS GATES DA CENA DA PINTURA** (`=51`) — irmão (`#[path]`) da [`super`].

use super::*;

/// ⭐⭐ **ELA ABRE COM O PINCEL DE PINTURA NA MÃO — as DUAS metades.**
///
/// O roteiro diz *«o pincel de pintura já está na sua mão»*, e essa frase é uma
/// AFIRMAÇÃO sobre o arranque. Ela parte-se em duas porque cada metade falha por
/// um motivo diferente:
///
/// 1. **a LEI** — com a cena armada, o verbo escolhido é o da pintura;
/// 2. **o FIO** — o prólogo da crate chama esta cena.
///
/// ⛔ **Sem a segunda, um gate que chama a função em vez de percorrer a rota
/// afirma que a lei existe e nunca que a cena a usa** (§24). E ela é lida por
/// [`include_str!`], que **deixa de compilar** se o irmão mudar de ficheiro —
/// em vez de ficar verde a medir menos.
#[test]
fn a_cena_da_pintura_abre_com_o_pincel_de_pintura() {
    // A LEI, pura: a cena escolhe a pintura, e só ela.
    assert_eq!(
        verbo_da_cena(true),
        Some(Verb::Paint),
        "a =51 não arma o pincel de pintura, e o roteiro promete-o na 1.ª linha"
    );
    assert_eq!(
        verbo_da_cena(false),
        None,
        "o prólogo desta cena mexe no pincel de OUTRA cena — um verbo trocado \
         debaixo de um roteiro que não o menciona"
    );
    // O FIO, em duas pontas: o prólogo chama esta cena, e ela pergunta pela env.
    const ROTEADOR: &str = include_str!("scenes.rs");
    const ESTA: &str = include_str!("scenes_pintura.rs");
    assert!(
        ROTEADOR.contains("pintura::arma(cena);"),
        "o prólogo da crate não chama a =51: a lei existe e ninguém a corre"
    );
    assert!(
        ESTA.contains("verbo_da_cena(pintura_scene())"),
        "o `arma` deixou de perguntar pela cena: ele passaria a armar a pintura \
         em TODA cena do módulo"
    );
}

/// ⭐⭐⭐ **O ROTEIRO NOMEIA CONTROLOS QUE EXISTEM, E NO NÍVEL QUE ELE PROMETE.**
///
/// ⛔ *Um passo que manda clicar numa linha AFIRMA que ela está na tela*, e o
/// dono aprova o smoke com o passo impossível dentro — foi o que aconteceu com o
/// `Auto-Smooth` da `=41`, que é `Pro` num painel que nasce `Basic`.
///
/// ⚠️ **A CAIXA DE COR só existe com o pincel de pintura em mãos**, e é por isso
/// que a régua a pergunta com ele armado: perguntá-la com o pincel de fábrica
/// leria *«não existe»* sobre um painel correcto.
///
/// # ⚠️ A premissa do NÍVEL morreu aqui, e a morte está à vista no diff
///
/// A redacção anterior varria as **três pistas** `Color R`/`G`/`B` e afirmava
/// que elas eram `Basic` — *«o roteiro manda escolher a cor no nível de fábrica
/// e a pista não é pintada ali»*. Em 2026-09-20 as três saíram, por ordem do
/// dono (*«troque os sliders de cor pelo seletor de Cor (caixa de cor)»*), e a
/// amostra que as substitui **não é uma `Row`**: ela não tem `level`, logo não
/// há nível em que ela desapareça. ⇒ a pergunta do nível deixou de existir, e o
/// que fica no lugar dela é a asserção que a torna verdadeira — *o pintor não
/// consulta o `ui_level`* —, lida do ficheiro para que ela não possa voltar em
/// silêncio.
#[test]
fn o_roteiro_nomeia_a_caixa_de_cor_que_o_painel_pinta() {
    // (a) Com o verbo que a cena arma, a caixa É pintada.
    assert!(
        Verb::Paint.deposita_a_cor_do_pincel(),
        "o verbo desta cena deixou de depositar a cor do pincel: a caixa não é \
         desenhada e o passo (1) do roteiro é impossível"
    );
    // ⛔ **O CONTROLO, e sem ele o gate passa com a caixa pintada SEMPRE:** um
    // pincel que puxa a cor da vizinhança não a lê, e oferecê-la seria o knob
    // morto que o dono reporta como «não vejo efeito».
    assert!(
        !Verb::Blur.deposita_a_cor_do_pincel(),
        "o `Blur` passou a declarar que deposita a cor do pincel — ele puxa-a do \
         ANEL, e a caixa ali seria um controlo morto"
    );
    // (b) E ela não tem cerca de NÍVEL, que é o que faz o passo (1) ser possível
    // em qualquer profundidade do painel. ⚠️ Lido do PINTOR: uma afirmação sobre
    // uma ausência tem de ser feita contra o ficheiro que a produz.
    const PINTOR: &str = include_str!("../../ph2d-panel-sculpt3d/src/paint/brush_cor.rs");
    assert!(
        !PINTOR.contains("ui_level"),
        "o pintor da caixa de cor passou a consultar o nível do painel — o passo \
         (1) do roteiro deixou de ser possível no nível em que o painel ABRE, e \
         esta cena manda escolher a cor antes de qualquer outra coisa"
    );
}

/// ⚠️ **A peça desta cena é a `96×144`, e o número é o que a torna legível** —
/// ver o cabeçalho do módulo. ⛔ E ela é MUITO mais leve que o default do módulo
/// de propósito: a §24 desta linha registou uma cena que fabricava a peça pesada
/// e fazia o dono reportar o pincel como lento.
#[test]
fn a_peca_da_cena_e_densa_e_muito_mais_leve_que_o_default() {
    let m = peca();
    // ⚠️ **A contagem NÃO é `LATITUDES × LONGITUDES`**, e a diferença é a
    // construção da esfera: os dois pólos são um vértice cada e a costura do
    // meridiano não se repete. *Escrever o produto aqui seria escrever de
    // memória um número que a porta calcula* — o gate lê-o da peça e afirma só
    // a ORDEM DE GRANDEZA, que é o que a cena precisa.
    assert!(
        m.vert_count() > LATITUDES * LONGITUDES * 9 / 10,
        "a peça tem {} vértices contra ~{} da grelha pedida: a porta mudou de \
         construção e a densidade desta cena deixou de ser a medida",
        m.vert_count(),
        LATITUDES * LONGITUDES
    );
    assert!(
        m.vert_count() > 10_000,
        "a peça tem {} vértices: abaixo disto uma marca de tinta lê-se como uma \
         mancha de vértices soltos e o passo (3) do roteiro não tem o que mostrar",
        m.vert_count()
    );
    assert!(
        m.vert_count() < 98_306 / 2,
        "a peça tem {} vértices, metade ou mais do default do módulo — esta cena \
         voltaria a medir o tamanho da peça em vez da ferramenta",
        m.vert_count()
    );
}

/// ⭐⭐⭐⭐ **TODO NOME QUE O ROTEIRO PÕE ENTRE CRASES É UM NOME QUE O DONO LÊ NA
/// TELA** — e o censo DERIVA-OS do roteiro, nunca de uma lista escrita à mão.
///
/// ⛔⛔ **O defeito que ele fecha nasceu e morreu no mesmo dia:** o passo (4-bis)
/// mandava escolher a fileira `Light` e o painel pinta **`Material`**
/// (`panel.sculpt3d.matcap`) — o dono seguiria o roteiro e procuraria uma linha
/// que não existe, que é a família que esta casa já pagou com o `Auto-Smooth`
/// da `=41`.
///
/// ⚠️⚠️ **E o gate irmão não o podia ver:** o
/// [`o_roteiro_nomeia_a_caixa_de_cor_que_o_painel_pinta`] pergunta a UM
/// controlo se ele é oferecido, e a fileira da luz é pintada à mão pelo
/// `body.rs`, fora de toda tabela.
/// *Um censo sobre UMA das populações lê-se, num relatório, como um censo sobre
/// todas* — e é por isso que este colhe as quatro.
#[test]
fn todo_nome_entre_crases_do_roteiro_existe_na_tela() {
    use ph2d_i18n::tr;
    use ph2d_panel_sculpt3d::rows::rows;
    use ph2d_panel_sculpt3d::state::{Sculpt3dUi, UiLevel};

    /// O que o roteiro nomeia e **não** é um rótulo de tela: teclas.
    ///
    /// ⚠️ Lista NOMEADA e curta de propósito — ela é a única saída do censo, e
    /// uma saída larga é a catraca a virar licença. O controlo abaixo proíbe
    /// que alguém estacione aqui um rótulo que o painel de facto pinta.
    const TECLAS: &[&str] = &["P", "Ctrl+Z"];

    // ── O ROTEIRO, colhido do fonte ─────────────────────────────────────────
    // ⛔⛔ **Ele NÃO é uma `const` de propósito** (ver o doc do `announce`): o
    // censo de TEXTO desta família isenta o que sai por `eprintln!`, e tirar o
    // roteiro de lá custaria uma linha nova de dívida no HR-15 só para tornar
    // este gate mais fácil de escrever.
    // ⚠️ A régua da colheita é o PREFIXO do módulo, que só as linhas do roteiro
    // trazem — e nunca um doc-comment, que é a prosa que EXPLICA a cura e já
    // enganou a metade de baixo deste mesmo gate.
    const ESTA_CENA: &str = include_str!("scenes_pintura.rs");
    let roteiro: Vec<&str> = ESTA_CENA
        .lines()
        .filter(|l| !l.trim_start().starts_with("//") && l.contains("[sculpt3d]"))
        .collect();
    // ⛔ **O PISO da colheita:** uma extracção partida devolve zero linhas, e
    // todo laço abaixo fica trivialmente verdadeiro sobre o vazio.
    assert!(
        roteiro.len() > 30,
        "a colheita do roteiro deu {} linhas: a extracção partiu-se",
        roteiro.len()
    );
    let roteiro = roteiro.join("\n");

    // ── A paridade das crases vem PRIMEIRO ──────────────────────────────────
    // ⛔ Uma crase solta faz o emparelhamento colher meia frase como se fosse o
    // nome de um controlo, e o censo passa a medir prosa — em silêncio, porque
    // uma frase nunca casa com rótulo nenhum e o erro lê-se como «o roteiro
    // nomeia algo que não existe».
    let crases = roteiro.matches('`').count();
    assert_eq!(
        crases % 2,
        0,
        "o roteiro tem {crases} crases, um número ÍMPAR: há uma solta, e o censo \
         abaixo deixa de saber onde acaba um nome"
    );

    // ── As quatro populações do que o painel MOSTRA ─────────────────────────
    let mut ui = Sculpt3dUi {
        ui_level: UiLevel::Basic,
        ..Sculpt3dUi::default()
    };
    // ⚠️ **Com o pincel da CENA em mãos**: as pistas de cor só existem com ele,
    // e perguntá-las com o pincel de fábrica leria «não existem» sobre um painel
    // correcto.
    ui.brush.verb = verbo_da_cena(true).expect("a cena declara um verbo");

    let mut na_tela: Vec<String> = Vec::new();
    // (a) as fileiras da TABELA, no nível em que o painel nasce;
    na_tela.extend(
        rows()
            .filter(|r| r.visible(&ui))
            .map(|r| tr(r.label).to_string()),
    );
    // (b) os chips de ferramenta, que são os verbos;
    na_tela.extend(
        ph2d_sculpt3d::Verb::ALL
            .iter()
            .map(|v| v.label().to_string()),
    );
    // (c) tudo o que o pintor do corpo do painel traduz — a população que a
    //     tabela de rows NÃO contém (a fileira da luz e os chips dela vivem
    //     aqui). ⚠️ Lido por `include_str!` de propósito: se o ficheiro mudar de
    //     sítio isto deixa de COMPILAR, em vez de ficar verde a medir menos.
    //     ⚠️⚠️ **E são DOIS pintores, não um — a 2.ª fonte nasceu em 20/09 e
    //     este gate ACUSOU a ausência dela.** A caixa de cor é pintada pelo
    //     `brush_cor.rs` e **não é uma `Row`**, logo a população (a) não a tem;
    //     com só o `body.rs` aqui, o passo (1) do roteiro (*«a fileira
    //     `Color`»*) lia-se como um nome que nada na tela usa. *Um censo sobre
    //     UMA das populações lê-se, num relatório, como um censo sobre todas* —
    //     a frase que este ficheiro já escrevia, cobrada a quem a escreveu.
    const PINTOR: &str = include_str!("../../ph2d-panel-sculpt3d/src/paint/body.rs");
    const PINTOR_COR: &str = include_str!("../../ph2d-panel-sculpt3d/src/paint/brush_cor.rs");
    for fonte in [PINTOR, PINTOR_COR] {
        for pedaco in fonte.split("tr(\"").skip(1) {
            if let Some(chave) = pedaco.split('"').next() {
                na_tela.push(tr(chave).to_string());
            }
        }
    }
    // (d) as teclas.
    na_tela.extend(TECLAS.iter().map(|s| (*s).to_string()));

    // ⛔ **O CONTROLO da saída**: nenhuma tecla pode ser também um rótulo
    // pintado — senão a lista de excepções vira o sítio onde um controlo morto
    // se esconde do censo.
    let pintados: Vec<&String> = na_tela.iter().take(na_tela.len() - TECLAS.len()).collect();
    for t in TECLAS {
        assert!(
            !pintados.iter().any(|p| p.as_str() == *t),
            "`{t}` está na lista de teclas E é um rótulo que o painel pinta: a \
             excepção deixou de ser uma excepção"
        );
    }
    // ⛔ **E o PISO de população**: uma extracção partida devolveria zero, e um
    // censo que não acha nada acusaria tudo — ou, pior, um `any` sobre uma lista
    // vazia ficaria verde noutra redacção deste mesmo gate.
    assert!(
        pintados.len() > 40,
        "o censo colheu só {} rótulos de tela: a extracção partiu-se",
        pintados.len()
    );

    // ── O veredito, nome a nome ─────────────────────────────────────────────
    let mut nomeados = 0usize;
    for (i, pedaco) in roteiro.split('`').enumerate() {
        if i % 2 == 0 {
            continue; // fora das crases
        }
        nomeados += 1;
        assert!(
            !pedaco.contains('\n'),
            "o roteiro parte um nome em duas linhas ({pedaco:?}): um rótulo tem de \
             caber numa linha, senão o dono não o reconhece na tela"
        );
        assert!(
            na_tela.iter().any(|n| n == pedaco),
            "o roteiro manda o dono procurar `{pedaco}` e NADA na tela se chama \
             assim — nem fileira, nem chip de ferramenta, nem rótulo do corpo do \
             painel, nem tecla. Ele vai procurar uma linha que não existe."
        );
    }
    // ⛔ E o piso do próprio sujeito: um roteiro que deixasse de nomear
    // controlos passaria este gate por vácuo.
    assert!(
        nomeados >= 8,
        "o roteiro nomeia só {nomeados} controlos: este censo ficou sem sujeito"
    );
}

/// ⭐⭐⭐ **A CENA ARMA O PINCEL PELA PORTA DO PRODUTO, NUNCA ESCREVENDO O CAMPO.**
///
/// ⛔⛔ `cena.brush.verb = v` põe o **RÓTULO** do pincel de pintura num pincel
/// que continua afinado como o `Draw`, e o que se perde tem número: a força fica
/// no default do verbo de fábrica onde o `Paint` declara o dele — menos tinta em
/// cada passagem, num pincel cuja razão de existir é depositar tinta.
///
/// ⚠️⚠️ **E era IRREVERSÍVEL dentro da cena:** o
/// [`ph2d_panel_sculpt3d::state::switch_verb_parts`] devolve cedo quando o verbo
/// que entra já é o que está em mãos ⇒ clicar no chip `Paint` seria um **no-op**
/// e a afinação declarada ficaria **inalcançável** — *a cena que arma pelo campo
/// cru tranca exactamente a afinação que ela quis escolher*.
///
/// ⚠️ **As duas metades falham por motivos diferentes:** a LEI (a porta entrega o
/// que o verbo declara) e o FIO (o `arma` percorre-a). Sem a segunda, alguém
/// reescreve o `arma` com o campo cru e este gate fica verde a medir a porta.
#[test]
fn a_cena_arma_o_pincel_pela_porta_e_nao_pelo_campo() {
    use ph2d_panel_sculpt3d::state::{VerbSlot, switch_verb_parts};

    // O MESMO arranjo que o `birth.rs` monta: um slot por verbo, cada um já com
    // o que aquele verbo declara.
    let mut slots: Vec<VerbSlot> = Verb::ALL.iter().map(|v| VerbSlot::for_verb(*v)).collect();
    let mut brush = ph2d_sculpt3d::Brush::default();
    let mut radius_px = 50.0_f32;
    let verbo = verbo_da_cena(true).expect("a cena declara um verbo");

    // ⛔ **O CONTROLO vem primeiro:** se o verbo de fábrica declarasse a MESMA
    // força, a asserção de baixo passaria com o campo cru e este gate não
    // afirmaria nada.
    let de_fabrica = brush.verb;
    assert_ne!(
        verbo.default_strength(),
        de_fabrica.default_strength(),
        "o `{verbo:?}` e o verbo de fábrica `{de_fabrica:?}` declaram a MESMA força: \
         esta fixtura deixou de conter o fenómeno"
    );

    switch_verb_parts(&mut slots, &mut brush, &mut radius_px, verbo);
    assert_eq!(
        brush.strength,
        verbo.default_strength(),
        "a porta entregou a força {} onde o `{verbo:?}` declara {} — o pincel tem o \
         nome certo e a afinação de outro",
        brush.strength,
        verbo.default_strength()
    );

    // O FIO, lido por `include_str!` — que deixa de COMPILAR se o irmão mudar de
    // ficheiro, em vez de ficar verde a medir menos.
    //
    // ⛔⛔ **E ele lê o CÓDIGO, nunca o ficheiro inteiro.** A 1.ª redacção
    // reprovou sobre a cura CERTA, porque o doc-comment do `arma` **cita o campo
    // cru para explicar porque ele saiu** — *uma régua textual que varre um
    // ficheiro inteiro lê a prosa que EXPLICA a cura e acusa-a de ser o defeito*
    // (a mesma forma que a Fase B da física registou ao varrer `\bApp\b`).
    const ESTA: &str = include_str!("scenes_pintura.rs");
    let codigo: Vec<&str> = ESTA
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect();
    // ⛔ **O PISO:** um filtro partido devolveria zero linhas, e `!contains` sobre
    // o vazio é trivialmente verdadeiro — o gate passaria a certificar o nada.
    assert!(
        codigo.iter().any(|l| l.contains("pub(crate) fn arma(")),
        "a extracção de código partiu-se: o `arma` não está nas {} linhas colhidas",
        codigo.len()
    );
    assert!(
        codigo.iter().any(|l| l.contains("switch_verb_parts(")),
        "o `arma` deixou de passar pela porta de troca de ferramenta"
    );
    assert!(
        !codigo.iter().any(|l| l.contains("cena.brush.verb =")),
        "o `arma` voltou a escrever o campo cru: o pincel nasce com o rótulo da \
         pintura e a afinação do verbo de fábrica"
    );
}
