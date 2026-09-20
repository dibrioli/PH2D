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
/// ⚠️ **As três pistas de cor só existem com o pincel de pintura em mãos**, e é
/// por isso que a régua as pergunta com ele armado: perguntá-las com o pincel de
/// fábrica leria *«não existem»* sobre um painel correcto.
#[test]
fn o_roteiro_nomeia_as_pistas_de_cor_que_o_painel_pinta() {
    use ph2d_panel_sculpt3d::rows::rows;
    use ph2d_panel_sculpt3d::state::{Sculpt3dUi, UiLevel};

    let mut ui = Sculpt3dUi {
        ui_level: UiLevel::Basic,
        ..Sculpt3dUi::default()
    };
    ui.brush.verb = Verb::Paint;
    for label in [
        "panel.sculpt3d.color_r",
        "panel.sculpt3d.color_g",
        "panel.sculpt3d.color_b",
    ] {
        let row = rows()
            .find(|r| r.label == label)
            .unwrap_or_else(|| panic!("o roteiro nomeia `{label}` e a tabela não o tem"));
        assert!(
            row.visible(&ui),
            "o roteiro manda escolher a cor no nível de fábrica e `{label}` não é \
             pintada ali — o passo (1) é impossível"
        );
    }
    // ⛔ **O CONTROLO, e sem ele o gate passa com três pistas pintadas SEMPRE:**
    // com um pincel que não deposita a cor do pincel, elas têm de sumir.
    ui.brush.verb = Verb::Blur;
    assert!(
        !rows()
            .filter(|r| r.label.starts_with("panel.sculpt3d.color_"))
            .any(|r| r.visible(&ui)),
        "as pistas de cor são pintadas com o `Blur` em mãos, que NÃO as lê — \
         três knobs mortos, a espécie que o dono reporta como «não vejo efeito»"
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
/// [`o_roteiro_nomeia_as_pistas_de_cor_que_o_painel_pinta`] olha a TABELA de
/// rows, e a fileira da luz é pintada à mão pelo `body.rs`, **fora** dela.
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
    const PINTOR: &str = include_str!("../../ph2d-panel-sculpt3d/src/paint/body.rs");
    for pedaco in PINTOR.split("tr(\"").skip(1) {
        if let Some(chave) = pedaco.split('"').next() {
            na_tela.push(tr(chave).to_string());
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
