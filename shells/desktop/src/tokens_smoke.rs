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

use crate::smoke_script::Step;
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

/// Os passos que o artista executa. ⚠️ Eles vivem numa CONST para o gate de largura os poder
/// medir sem abrir uma janela — um roteiro que quebra no terminal é o defeito que a porta
/// [`crate::smoke_script`] existe para impedir.
const STEPS: &[Step] = &[
    Step {
        verb: "O PAINEL",
        lines: &[
            "Ele já está aberto à direita. A tecla T abre e fecha.",
            "A 1ª linha diz o modo vigente e quantos tokens estão autorados nele.",
        ],
    },
    Step {
        verb: "A COR RE-VESTE O APP INTEIRO",
        lines: &[
            "Clique a swatch de 'accent' e escolha um verde no picker.",
            "A janela toda muda enquanto você arrasta: realces, foco, botões.",
            "Não é uma prévia — é o app que você está a usar.",
        ],
    },
    Step {
        verb: "O RESET DA LINHA",
        lines: &[
            "A linha autorada fica com o nome realçado e ganha um Reset.",
            "Carregue nele: a cor volta à de fábrica e o Reset desaparece.",
        ],
    },
    Step {
        verb: "O MODO",
        lines: &[
            "Autore 'bg-0' (o fundo) e aperte M para ciclar o tema.",
            "O Forge fica com a sua cor; os outros três seguem de fábrica —",
            "o override é do PAR (modo, token). Volte ao Forge com mais três M.",
        ],
    },
    Step {
        verb: "RESET THIS MODE",
        lines: &[
            "Devolve o modo vigente inteiro, e SÓ ele — cor E escala.",
            "Autore algo noutro modo antes, e confirme que aquilo sobrevive.",
        ],
    },
    Step {
        verb: "O ELO — um token SEGUE outro",
        lines: &[
            "Cada linha tem um botão de corrente à direita.",
            "Clique o da linha que quer MUDAR: ela arma.",
            "Clique o da linha que ela deve SEGUIR: o elo fecha.",
            "É a ordem em que se fala — 'o border segue o accent'.",
            "O rótulo passa a dizer 'border-emph - accent'.",
            "Agora autore o accent noutra cor: quem o segue muda junto, na hora.",
            "Clicar de novo a linha armada desarma (não existe auto-elo).",
        ],
    },
    Step {
        verb: "O CONTRASTE — o aviso da WCAG",
        lines: &[
            "Pegue 'text-1' e escolha uma cor quase igual à do fundo.",
            "Nasce um bloco de aviso no topo: qual par quebrou, a razão medida",
            "e a que a norma exige. E as DUAS linhas do par ganham a marca —",
            "o texto E o fundo, porque escurecer o fundo quebra o texto.",
            "Nada disso é clicável, de propósito: consertar é escolher outra cor.",
            "Dê Reset no text-1 e o bloco tem de sumir inteiro.",
        ],
    },
    Step {
        verb: "A ESCALA (px) — a família numérica",
        lines: &[
            "Role até 'Scale (px)': spacing, radius e stroke, uma linha cada.",
            "O campo é um número: digite, ou arraste-o para varrer.",
            "Reset e corrente funcionam igual aos da cor (px pode seguir px:",
            "'radius.md' pode seguir 'spacing.md' — é a mesma unidade).",
            "Um valor negativo é RECUSADO com um toast, e o campo volta.",
        ],
    },
    Step {
        verb: "⚠️ A ESCALA MOVE O APP — é ESTA a wave (W4c.2)",
        lines: &[
            "Suba 'spacing.lg' de 12 para ~40: o app RE-ESPAÇA na hora —",
            "o painel, os cards, o rail. Baixe para 2 e ele aperta.",
            "⚠️ Se a janela NÃO se mexer, PARE: a tabela de runtime não está",
            "a ser publicada, e o resto do roteiro não diz nada.",
            "'stroke.default' engrossa as linhas; 'radius.*' arredonda as quinas.",
            "Reset devolve na hora. E o *Reset This Mode* continua alcançável",
            "mesmo num valor absurdo — role até ele (foi medido: até 65536 px).",
        ],
    },
    Step {
        verb: "A FÓRMULA — é ESTA a wave (W4c.3)",
        lines: &[
            "Numa linha de escala, clique o botão de FÓRMULA (à esquerda do elo)",
            "e escreva:   {spacing.md} * 2",
            "Enter. A linha passa a valer o dobro do 'spacing.md' — e o número",
            "à esquerda mostra QUANTO deu.",
            "⚠️ Agora mude o 'spacing.md': a linha da fórmula segue JUNTO.",
            "É essa a diferença entre uma fórmula e um número que por acaso bate.",
            "⚠️ Numa linha COM fórmula o chip de digitar SOME — um valor tem UM",
            "editor, e os dois na tela deixariam o chip apagar a fórmula em silêncio.",
            "Escreva  {spacing.enormous}  e o app RECUSA, dizendo a chave.",
            "Escreva  {spacing.md} + gap  e ele recusa 'gap' — um nome que não é",
            "um token valeria ZERO em silêncio, e é o que a recusa impede.",
            "Apague o campo e dê Enter: a linha volta à fábrica.",
        ],
    },
    Step {
        verb: "O ARQUIVO",
        lines: &[
            "Autore duas cores, um elo, um número E uma fórmula. Ctrl+S, feche,",
            "reabra, Ctrl+O. Os quatro voltam — e a fórmula volta como FÓRMULA",
            "(o campo mostra o texto), não como o número que ela deu.",
            "Depois abra um projeto de fábrica: o app tem de voltar ao de",
            "fábrica (o load ESQUECE o documento anterior — os dois).",
        ],
    },
    Step {
        verb: "O DTCG — é ESTA a wave (W9)",
        lines: &[
            "No topo há 'Export DTCG...' e 'Import DTCG...', lado a lado.",
            "Autore uma cor, um elo e uma fórmula. Export: escolha um lugar.",
            "Abra o arquivo num editor de texto — ele é para ser LIDO:",
            "cada token com $type e $value, o elo como  {accent} , e a",
            "fórmula no $extensions (o $value dela é o número que ela deu).",
            "Agora *Reset This Mode* e Import do mesmo arquivo: os três voltam,",
            "e a fórmula volta como FÓRMULA. O toast diz quantos entraram.",
            "⚠️ Import de um arquivo de fábrica autora ZERO — o toast diz",
            "'already at factory'. Se ele autorar ~80, PARE: a tabela de",
            "fábrica ficou inalcançável e re-editar tokens.json não chega ao app.",
            "Aperte M antes do Import e ele re-veste o modo que está NA TELA,",
            "não o que o arquivo diz no $description.",
        ],
    },
    Step {
        verb: "O CONTROLE",
        lines: &[
            "As três formas do canvas não mudam de cor em passo nenhum.",
            "Re-vestir o chrome não toca no documento.",
        ],
    },
];

fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let theme = gfx
        .hero_screen
        .as_ref()
        .map_or_else(ph2d_tokens::Theme::default, |h| h.theme);
    eprintln!(
        "[tokens] painel ABERTO: {} tokens de cor + {} de escala (px) no modo vigente, \
         {} autorado(s); {} formas de controle na cena.",
        ph2d_tokens::ColorToken::ALL.len(),
        ph2d_tokens::NumToken::ALL.len(),
        ph2d_tokens::overrides::overridden_count(theme)
            + ph2d_tokens::num_overrides::num_overridden_count(theme),
        gfx.vec_scene.paths().len()
    );
    crate::smoke_script::script("tokens", "o painel já está aberto", STEPS);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O ROTEIRO cabe no terminal** — o defeito medido que originou a porta `smoke_script`.
    ///
    /// ⚠️ Sem ele o roteiro volta a quebrar sozinho: as linhas desta cena tinham 81-82 colunas
    /// e o teto é 75, então a continuação perdia a indentação e lia-se como um passo novo.
    #[test]
    fn the_script_fits_in_a_terminal() {
        crate::smoke_script::assert_fits("tokens", STEPS);
    }

    /// **A sonda que MOSTRA o roteiro** — o oráculo de legibilidade é o olho, não um assert.
    ///
    /// `cargo test -p ph2d-host-desktop --bins show_the_script -- --ignored --nocapture`
    ///
    /// ⚠️ O gate ao lado prova que as linhas CABEM; nenhum assert prova que elas se LEEM. Quem
    /// escrever a próxima cena roda isto antes de decidir que o roteiro está pronto.
    #[test]
    #[ignore = "sonda: imprime o roteiro para ser lido"]
    fn show_the_script() {
        crate::smoke_script::script("tokens", "o painel já está aberto", STEPS);
    }

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
            "a tabela de cor tem {} tokens — o painel não teria o que listar",
            ph2d_tokens::ColorToken::ALL.len()
        );
        assert!(
            ph2d_tokens::NumToken::ALL.len() > 10,
            "a tabela de escala tem {} tokens — o passo da família numérica \
             falaria sobre nada",
            ph2d_tokens::NumToken::ALL.len()
        );
    }
}
