//! **Arch-gate: na cena do HUD, o rótulo do botão é FILHO do botão** (TOP-20 #20).
//!
//! # A lei, e o defeito MEDIDO que ela impede
//!
//! O hit-test de objecto entrega *a forma mais ao topo que contém o ponto*. O `+10` é desenhado por
//! cima do corpo do botão ⇒ **o dedo aterra sempre no RÓTULO**, e é a subida da cadeia
//! ([`ph2d_ecs::hud::botao_de`], gateada na folha) que faz disso um clique no botão.
//!
//! ⚠️ **A lei da folha não basta:** ela responde *«de quem é esta forma?»* e precisa que a CADEIA
//! exista. Com o rótulo pendurado no canvas — como ele nasceu — a resposta é `None`, e carregar no
//! meio do botão não o pressiona. A auto-conferência da cena mediu-o à letra (`achou=Some(2)`,
//! `esperado=3`), e o veredito dela é um `eprintln!` que ninguém lê num portão. Este gate é a
//! metade que reprova.
//!
//! ⛔ **Textual, e a razão é a de sempre:** a montagem vive num método de `App` que exige janela,
//! GPU e superfície — nenhum teste a alcança. É a forma dos irmãos `architecture_*` da shell.

/// ⚠️ A PROSA sai antes de a lei ser aplicada — senão este mesmo cabeçalho satisfazia o `assert`,
/// e o gate passaria a medir a explicação em vez do código.
fn sem_comentarios(src: &str) -> String {
    src.lines()
        .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn o_rotulo_do_botao_do_hud_e_filho_do_botao() {
    let src = sem_comentarios(include_str!("../../src/hud_smoke.rs"));

    // Controlo positivo: a cena ainda monta as peças de que a lei fala. Sem isto, um ficheiro
    // renomeado ou esvaziado deixaria os dois `assert` abaixo trivialmente verdes.
    for agulha in [
        "let (Some(e_pontos), Some(e_resta), Some(e_botao), Some(e_rotulo))",
        "UiButton",
    ] {
        assert!(
            src.contains(agulha),
            "controlo positivo: a cena do HUD deixou de conter `{agulha}` — este gate passou a \
             medir o nada"
        );
    }

    assert!(
        src.contains("ChildOf(e_botao)"),
        "o rótulo do botão não é filho do BOTÃO na cena do HUD.\n\
         O dedo aterra no rótulo (o hit-test devolve a forma mais ao topo que contém o ponto), e a \
         subida da cadeia (`ph2d_ecs::hud::botao_de`) só encontra o botão se a cadeia existir. Sem \
         isto, carregar no MEIO do botão não o pressiona — medido em 2026-09-17."
    );

    // A outra metade: ele não pode estar TAMBÉM na lista que pendura tudo no canvas — duas poses
    // locais somam-se, e o rótulo sairia do ecrã por baixo.
    let parenta_no_canvas = src
        .split("ChildOf(canvas)")
        .next()
        .expect("split devolve sempre a 1.ª fatia");
    assert!(
        !parenta_no_canvas.contains("(e_rotulo, pend.rotulo.local)"),
        "o rótulo está na lista que pendura no CANVAS e também no botão — a pose local somaria \
         duas vezes o mesmo deslocamento"
    );
}

/// ⭐⭐⭐ **Cada peça de baixo é AUTORADA do lado a que ela se prende** — e a régua cruza as duas
/// metades, que é precisamente o que a redacção anterior deste gate não fazia.
///
/// # ⛔⛔ A premissa do gate anterior MORREU, e ela foi paga por um report
///
/// Ele lia as fracções `min` da crate da família e afirmava que os dois cantos eram **opostos em
/// `x`** — e estavam. O que ninguém media era ONDE cada peça está desenhada: a pontuação prendia-se
/// à aresta DIREITA e era autorada em `x = −7`, a contagem prendia-se à ESQUERDA e estava em
/// `x = +8`. Em `Keep` a caixa efectiva é a de referência, o delta é `0,0` e nada se move — logo
/// *«os demais modos OK»*; em `Expand` cada uma anda para a sua borda e a certa altura elas
/// **atravessam-se** (report do dono, 2026-09-20).
///
/// ⇒ *um gate que mede a REGRA e nunca a POSIÇÃO fica verde sobre uma cena que se cruza.*
///
/// # O que este afirma
///
/// A aritmética está provada na folha ([`ph2d_app_components::hud_smoke_anchors`], onde o SINAL da
/// posição é derivado da fracção da regra, com o controlo da autoria espelhada ao lado). O que só
/// se pode afirmar AQUI é o **EMPARELHAMENTO**: que a peça a que a cena dá a regra da esquerda é a
/// mesma que ela autora com [`Canto::Esquerda`].
///
/// **Mutação que deve sangrar:** trocar os dois argumentos do `prende_os_cantos`.
#[test]
fn cada_peca_de_baixo_e_autorada_do_lado_a_que_se_prende() {
    let src = sem_comentarios(include_str!("../../src/hud_smoke.rs"));

    // (a) Que CANTO cada peça usa para nascer.
    let canto_de = |peca: &str| -> String {
        let i = src
            .find(&format!("{peca}: Peca {{"))
            .unwrap_or_else(|| panic!("a cena do HUD deixou de montar a peca `{peca}`"));
        let bloco = &src[i..src.len().min(i + 200)];
        let k = bloco
            .find("local: Canto::")
            .unwrap_or_else(|| panic!("a peca `{peca}` deixou de nascer de um `Canto`"));
        bloco[k + "local: Canto::".len()..]
            .split('.')
            .next()
            .expect("o nome do canto")
            .trim()
            .to_owned()
    };

    // (b) Que peça está por trás de cada `Entity` — lido da tupla que as liga, nunca do nome.
    let i = src
        .find("let (Some(e_pontos), Some(e_resta)")
        .expect("a cena deixou de ligar as pecas a entidades");
    let bloco = &src[i..src.len().min(i + 400)];
    let nomes: Vec<&str> = bloco
        .split("Some(")
        .skip(1)
        .filter_map(|t| t.split(')').next())
        .collect();
    let pecas: Vec<&str> = bloco
        .split("ent(pend.")
        .skip(1)
        .filter_map(|t| t.split('.').next())
        .collect();
    assert!(
        nomes.len() >= 2 && pecas.len() >= 2,
        "controlo positivo: a extraccao leu {} entidades e {} pecas — este gate passou a medir o \
         nada",
        nomes.len(),
        pecas.len()
    );
    let peca_de = |entidade: &str| -> String {
        let k = nomes
            .iter()
            .position(|n| *n == entidade)
            .unwrap_or_else(|| panic!("a cena nao liga `{entidade}` a peca nenhuma"));
        (*pecas.get(k).expect("uma peca por entidade")).to_owned()
    };

    // (c) A quem a cena dá cada regra. A assinatura é `(world, contagem, pontos)`, e a contagem
    // prende-se à ESQUERDA (provado na folha).
    let i = src
        .find("prende_os_cantos(")
        .expect("a cena deixou de prender os cantos");
    let args: Vec<String> = src[i..]
        .split_once('(')
        .expect("a chamada tem argumentos")
        .1
        .split_once(')')
        .expect("a chamada fecha")
        .0
        .split(',')
        .map(|a| a.trim().to_owned())
        .collect();
    assert_eq!(args.len(), 3, "a chamada mudou de forma: {args:?}");

    for (arg, esperado) in [(&args[1], "Esquerda"), (&args[2], "Direita")] {
        let peca = peca_de(arg);
        let canto = canto_de(&peca);
        assert_eq!(
            canto, esperado,
            "a cena da' a regra do canto `{esperado}` a `{arg}` (a peca `{peca}`), que nasce em \
             `Canto::{canto}` — ela e' autorada do lado OPOSTO aquele a que se prende, e em \
             `Expand` as duas pecas atravessam-se"
        );
    }
}

/// ⛔⛔⛔ **A cena FECHA o painel da timeline, senão o HUD dela fica atrás dele.**
///
/// ⚠️ **Medido na foto de 2026-09-20** (janela `1930×1012`): com a timeline aberta o botão do HUD
/// é alcançável na caixa de ecrã `y 728..848` e o painel começa em `~720` ⇒ a sonda do produto lê
/// `on_canvas=false (painel=Some(true))` e o gesto do dono **nem chega ao ramo do HUD**. Fechado,
/// a mesma sonda lê `on_canvas=true` e o clique é consumido nos dois lados.
///
/// ⚠️⚠️ **`false` e não a AUSÊNCIA da linha:** a arrumação vive em `~/.ph2d/layout.txt`, fora do
/// repositório — com a timeline aberta de ontem, não abrir não fecha nada.
///
/// **Mutação que deve sangrar:** voltar a pôr `true`.
#[test]
fn a_cena_do_hud_fecha_o_painel_da_timeline() {
    let src = sem_comentarios(include_str!("../../src/hud_smoke.rs"));
    assert!(
        src.contains(r#"panel_visibility.insert("timeline", false)"#),
        "a cena do HUD deixou de FECHAR a timeline — com ela aberta as pecas de baixo caem atras \
         do painel, e nem se veem nem se clicam"
    );
    assert!(
        !src.contains(r#"panel_visibility.insert("timeline", true)"#),
        "a cena do HUD volta a ABRIR a timeline — as duas linhas nao podem conviver"
    );
}

/// ⛔⛔ **A cena abre em `Expand`, e sem isso ela não demonstra nada.**
///
/// ⚠️ Desde a correcção do oráculo (bloco L4) só o `Expand` cresce a caixa efectiva: com `Keep` os
/// dois cantos ficam na área segura e **arrastar a borda da janela não move um pixel**. O roteiro
/// manda arrastar ⇒ *uma cena que abre no modo errado ensina o CONTRÁRIO do que diz*, que é a
/// espécie que o `CLAUDE.md` §5.0 chama de pior que uma cena ausente.
///
/// **Mutação que deve sangrar:** `fit: Fit::Keep` na cena.
#[test]
fn a_cena_do_hud_abre_em_expand() {
    let src = include_str!("../../src/hud_smoke.rs");
    assert!(
        src.contains("fit: Fit::Expand,"),
        "a cena do HUD deixou de abrir em `Expand` — arrastar a borda nao move nada, e o roteiro \
         manda arrastar"
    );
}

/// ⛔⛔⛔ **O passo do roteiro que manda trocar o `Fit` NOMEIA a linha da Hierarquia, e não o
/// painel** — porque a linha `Fit` só existe com a RAIZ escolhida.
///
/// ⚠️⚠️ **Report do dono, 2026-09-20, com foto:** *«se a secção HUD que vc se refere é no
/// inspector, não tem as opções que vc mandou mudar»*. E ele tinha razão: a cena abre com o
/// **rótulo** escolhido, e o painter pinta as linhas do canvas só `if has_canvas` ⇒ com um filho
/// seleccionado a linha `Fit` **não está lá**. *Um passo que manda clicar numa linha AFIRMA que ela
/// está na tela, e o dono aprova o smoke com o passo impossível dentro.*
///
/// ⚠️ E a raiz **não se pega no canvas**: ela não tem `Sprite` nem `VecShape`, logo o
/// `pick_sprite_at_world` nunca a devolve — a Hierarquia é a única porta. É a mesma lei que a cena
/// do #15 pagou, e aqui a cura é o roteiro NOMEAR a porta certa.
///
/// **Mutação que deve sangrar:** tirar a palavra `HIERARQUIA` do roteiro.
#[test]
fn o_roteiro_manda_escolher_a_raiz_na_hierarquia_antes_do_fit() {
    let src = include_str!("../../src/hud_smoke.rs");
    // ⚠️ **A agulha cabe numa LINHA:** o roteiro é um `println!` com continuações `\`, e a 1.ª
    // redacção procurou «`Fit` de `Expand` para `Keep`» — que o corte de linha parte em duas.
    // *Uma agulha tem de sobreviver à forma como o texto está escrito.*
    let i = src
        .find("de `Expand` para")
        .expect("o roteiro deixou de mandar trocar o Fit");
    // O passo tem de nomear a HIERARQUIA **antes** de falar do `Fit`.
    let antes = &src[..i];
    let h = antes
        .rfind("HIERARQUIA")
        .expect("o roteiro manda trocar o `Fit` sem dizer onde escolher a RAIZ");
    assert!(
        i - h < 400,
        "a palavra HIERARQUIA esta' longe demais do passo do `Fit` — nao e' o mesmo passo"
    );
    // ⭐ E a raiz tem de ter NOME, senão a linha da Hierarquia não é nomeável.
    assert!(
        src.contains("Name::new(\"HUD\")"),
        "a raiz do canvas perdeu o nome — o roteiro manda clicar numa linha sem etiqueta"
    );
}
