//! **A cena dos TOKENS** — `PH2D_BUILD_SMOKE=59` (plano UI/UX W6, degrau 1).
//!
//! # A pergunta desta cena é de olho, e ela é sobre o APP, não sobre um desenho
//!
//! *Eu mudo uma cor no painel e a janela inteira — a barra, os painéis, o rail, os botões — muda
//! junto, ao vivo.*
//!
//! É por isso que ela **não monta geometria nenhuma**: o sujeito do smoke é o próprio editor. O
//! que a cena dá é o painel ABERTO (a tecla `T` é o gesto, e ela é provada por um arch-gate; abrir
//! aqui poupa o artista de o descobrir) e algumas formas para ele ver que a arte **não** muda —
//! re-vestir o chrome não pode tocar no documento.
//!
//! ⚠️ **E ela imprime o número que a torna válida:** quantos tokens o painel vai listar. Se for
//! zero, PARE — a tabela não chegou, e o resto do roteiro não diz nada.

use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// Três formas quaisquer: o CONTROLE de que a arte não é chrome.
const ART: [[f64; 4]; 3] = [
    [-4.0, 0.4, -2.4, 2.0],
    [-1.8, 0.0, -0.2, 1.6],
    [0.4, 0.8, 2.0, 2.4],
];

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        6 => open(app),
        7 => announce(app),
        _ => {}
    }
}

fn paths() -> Vec<VecPath> {
    ART.iter()
        .enumerate()
        .map(|(i, r)| {
            let rgb = [[214, 82, 82], [82, 184, 120], [148, 110, 214]][i];
            let mut p = rectangle([r[0], r[1]], [r[2], r[3]]);
            p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
            p
        })
        .collect()
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for p in paths() {
        gfx.vec_scene.push_path(p);
    }
}

/// Abre o painel. ⚠️ O gesto REAL é a tecla `T` (e há arch-gate sobre ela) — abrir aqui é poupar
/// o artista de a descobrir, não substituir a costura.
fn open(app: &mut crate::App) {
    if let Some(gfx) = app.gfx.as_mut()
        && let Some(hero) = gfx.hero_screen.as_mut()
    {
        hero.panel_visibility.insert("tokens", true);
    }
}

fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let theme = gfx
        .hero_screen
        .as_ref()
        .map_or_else(ph2d_tokens::Theme::default, |h| h.theme);
    eprintln!(
        "[tokens] painel ABERTO com {} tokens de cor no modo vigente; {} autorado(s). A cena tem \
         {} formas, e elas sao o CONTROLE.",
        ph2d_tokens::ColorToken::ALL.len(),
        ph2d_tokens::overrides::overridden_count(theme),
        gfx.vec_scene.paths().len()
    );
    eprintln!("[tokens] o roteiro:");
    eprintln!("  1. O painel **Tokens** esta' aberto a' direita (a tecla **T** abre e fecha).");
    eprintln!("     A 1a linha diz o MODO vigente e quantos tokens estao autorados nele.");
    eprintln!("  2. ⚠️ **A PROVA DA WAVE**: clique na swatch de **accent** e escolha um verde no");
    eprintln!("     picker. A janela INTEIRA muda junto — realces, foco, botoes — enquanto voce");
    eprintln!("     arrasta. Nao e' uma previa: e' o app que voce esta' a usar.");
    eprintln!(
        "  3. A linha autorada fica com o NOME realcado e ganha um **Reset**. Carregue nele:"
    );
    eprintln!("     a cor volta a' de fabrica e o Reset desaparece (um Reset sobre um token de");
    eprintln!("     fabrica seria um clique que nao faz nada).");
    eprintln!("  4. ⚠️ **O MODO**: autore **bg-0** (o fundo) e aperte **M** para ciclar o tema. O");
    eprintln!("     Forge fica com a sua cor; os outros tres continuam de fabrica — o override e'");
    eprintln!("     do PAR (modo, token). Volte ao Forge com mais tres **M**.");
    eprintln!("  5. **Reset This Mode** devolve o modo vigente inteiro. ⚠️ SO' o vigente: autore");
    eprintln!("     algo noutro modo antes, e confirme que ele sobrevive.");
    eprintln!(
        "  6. ⚠️ **O ELO** (W4b.1): cada linha tem um botao de CORRENTE a' direita. Clique o"
    );
    eprintln!("     da linha que voce quer MUDAR (ela arma) e depois o da linha que ela deve");
    eprintln!("     SEGUIR — a ordem em que se fala: 'o border segue o accent'. O rotulo passa a");
    eprintln!("     dizer 'border-emph  -  accent' e a swatch mostra a cor do ALVO.");
    eprintln!("     Agora autore o **accent** noutra cor: quem o segue muda JUNTO, na hora.");
    eprintln!("     Clicar a propria linha armada DESARMA (nao existe auto-elo).");
    eprintln!("  7. ⚠️ **O CONTRASTE** (W4b.2): pegue **text-1** e escolha uma cor quase igual a'");
    eprintln!("     do fundo. Aparece um bloco de AVISO no topo do painel dizendo qual par");
    eprintln!("     quebrou, com a razao medida e a que a WCAG exige — e as DUAS linhas do par");
    eprintln!("     (o texto E o fundo) ganham a marca de aviso.");
    eprintln!("     ⚠️ Nada disso e' clicavel, de proposito: consertar e' escolher outra cor.");
    eprintln!("     Devolva o **text-1** com o Reset e o bloco tem de SUMIR inteiro.");
    eprintln!(
        "  8. ⚠️ **ELA SOBREVIVE AO ARQUIVO**: autore duas cores E um elo, **Ctrl+S**, feche"
    );
    eprintln!(
        "     o app, reabra e **Ctrl+O**. As cores E o elo voltam. Depois abra um projeto de"
    );
    eprintln!("     FABRICA: o app tem de voltar as cores de fabrica (o load ESQUECE o anterior).");
    eprintln!("  9. ⚠️ **O CONTROLE**: as tres formas do canvas nao mudam de cor em passo nenhum.");
    eprintln!("     Re-vestir o chrome nao toca no documento.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena tem o que o passo do CONTROLE manda olhar** — três formas de cores distintas.
    ///
    /// ⚠️ Ele não cita o NÚMERO do passo de propósito: o roteiro cresce a cada wave (o controle
    /// já foi o 7 e é o 9), e um doc que numera o passo mente na primeira linha inserida acima.
    #[test]
    fn the_control_art_exists_and_is_distinguishable() {
        let fills: Vec<_> = paths().iter().map(|p| p.fill.clone()).collect();
        assert_eq!(fills.len(), 3);
        assert!(fills.iter().all(Option::is_some));
        for i in 0..fills.len() {
            for j in i + 1..fills.len() {
                assert_ne!(fills[i], fills[j], "as formas {i} e {j} tem a mesma cor");
            }
        }
    }

    /// **A tabela que o painel vai listar não está vazia** — o número que o `announce` imprime.
    ///
    /// ⚠️ Um painel de zero linhas é indistinguível de um painel quebrado, e o roteiro inteiro
    /// falaria sobre nada.
    #[test]
    fn the_table_the_panel_lists_is_not_empty() {
        assert!(
            ph2d_tokens::ColorToken::ALL.len() > 20,
            "a tabela de cor tem {} tokens — o painel nao teria o que listar",
            ph2d_tokens::ColorToken::ALL.len()
        );
    }
}
