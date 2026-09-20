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

/// ⭐⭐⭐ **As duas peças de baixo PRENDEM-SE aos cantos OPOSTOS** — o item que o handoff do #20
/// deixou aberto (*«as âncoras não estão ligadas ao canvas»*).
///
/// ⚠️ A régua é o TEXTO da cena, e o que ela afirma é a coisa que um artista vê: **os dois cantos
/// são opostos em `x`**. Um gate que só contasse *«há duas âncoras»* ficaria verde com as duas
/// presas ao MESMO canto — e as duas peças empilhavam-se uma em cima da outra ao alargar a janela.
///
/// **Mutação que deve sangrar:** pôr `min: [0.0, 0.0]` nas duas.
#[test]
fn as_duas_pecas_de_baixo_prendem_se_a_cantos_opostos() {
    // ⚠️ **A fonte é a CRATE DA FAMÍLIA, e o endereço já se mudou uma vez:** a regra vivia em
    // `shells/desktop/src/hud_smoke_anchors.rs` e saiu para a crate em 2026-09-19, quando a
    // catraca da shell a mandou para casa. *Uma agulha que nomeia um endereço falha ALTO no dia da
    // mudança, que é a espécie barata.*
    let src = include_str!("../../../../crates/ph2d-app-components/src/hud_smoke_anchors.rs");
    let resta = src
        .find("world.entity_mut(contagem).insert(VecAnchors {")
        .expect("a contagem deixou de se prender ao canto");
    let pontos = src
        .find("world.entity_mut(pontos).insert(VecAnchors {")
        .expect("os pontos deixaram de se prender ao canto");

    // A fracção em `x` de cada um, lida do bloco que começa em cada `find`.
    let x_de = |i: usize| -> f64 {
        // ⚠️ A fatia SATURA: o ficheiro é curto, e `i + 200` passava do fim dele — a 1.ª
        // redacção rebentava no `slice index out of range` em vez de medir.
        let bloco = &src[i..src.len().min(i + 200)];
        let m = bloco.find("min: [").expect("a regra tem um `min`");
        bloco[m + 6..]
            .split(',')
            .next()
            .expect("a fraccao em x")
            .trim()
            .parse()
            .expect("um numero")
    };
    let (a, b) = (x_de(resta), x_de(pontos));
    assert!(
        (a - b).abs() > 0.5,
        "as duas pecas prendem-se ao MESMO lado em x ({a} e {b}) — ao alargar a janela elas \
         empilham-se uma sobre a outra"
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
