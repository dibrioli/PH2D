//! Os gates do cartão de secção. ⚠️ **A régua é a CENA** (quantos caminhos foram emitidos e por
//! que ordem), não a aritmética do `Rect` — um cartão que se calcula certo e não se pinta lê-se
//! exactamente como um que se pinta.

use super::*;
use ph2d_tokens::Theme;

/// Quantos caminhos a cena emitiu.
fn path_count(scene: &VectorScene) -> usize {
    scene.inner().encoding().n_paths as usize
}

/// ⭐⭐⭐ **O corpo NÃO se perde ao ser estacionado.**
///
/// Esta é a metade que paga o mecanismo: se o `append` deixasse cair o que o corpo pintou, todo
/// painel convertido ficaria em branco — e nenhum gate de geometria o veria, porque os `Rect`
/// continuariam certos. *Uma faixa reservada não é uma faixa pintada.*
#[test]
fn the_parked_body_comes_back_whole_and_the_cards_come_under_it() {
    let theme = Theme::MODERN[0];

    // Quanto custa só o corpo, sem cartão nenhum.
    let mut plain = VectorScene::new();
    let mut y = 0.0;
    for _ in 0..3 {
        fill_rounded_rect(
            &mut plain,
            Rect::new(0.0, y, 100.0, 20.0),
            0.0,
            resolve(ColorToken::Text1, theme),
        );
        y += 30.0;
    }
    let body_paths = path_count(&plain);
    assert!(body_paths >= 3, "o corpo de controlo tem de pintar algo");

    // O mesmo corpo, dentro de cartões.
    let mut scene = VectorScene::new();
    let n = with_section_cards(&mut scene, theme, 0.0, |scene| {
        let mut y = 0.0;
        let mut n = 0;
        for _ in 0..3 {
            fill_rounded_rect(
                scene,
                Rect::new(0.0, y, 100.0, 20.0),
                0.0,
                resolve(ColorToken::Text1, theme),
            );
            y += 30.0;
            y = close_section(scene, theme, 0.0, 100.0, y);
            n += 1;
        }
        n
    });

    assert_eq!(n, 3, "o corpo devolve o que devolveria fora do cartão");
    assert_eq!(
        path_count(&scene),
        body_paths + 3,
        "a cena tem de ter o corpo INTEIRO mais um cartão por secção — se o `append` perdesse o \
         corpo isto leria o número dos cartões sozinhos"
    );
}

/// ⭐ **O cartão envolve o conteúdo, e por FORA** — nenhuma linha muda de sítio.
#[test]
fn the_card_is_an_outset_so_no_row_moves() {
    let theme = Theme::MODERN[0];
    let mut cards = SectionCards::new(theme, 10.0);
    let mut scene = VectorScene::new();
    let next = cards.close(&mut scene, 20.0, 100.0, 70.0);

    let (rect, depth) = cards.rects[0];
    assert_eq!(depth, CardDepth::Section);
    assert!(
        rect.y < 10.0 && rect.y + rect.h > 70.0 && rect.x < 20.0 && rect.x + rect.w > 120.0,
        "o cartão {rect:?} tem de conter o conteúdo (10..70 em y, 20..120 em x) com folga"
    );
    assert!(next > 70.0, "o cursor avança para lá do conteúdo fechado");
}

/// ⭐⭐ **Uma subsecção é um cartão de OUTRA cor** — é isso, e só isso, que a distingue.
#[test]
fn a_subsection_is_a_lighter_card_on_top_of_its_parent() {
    let theme = Theme::MODERN[0];
    let section = resolve(CardDepth::Section.token(), theme);
    let sub = resolve(CardDepth::Subsection.token(), theme);
    assert_ne!(
        section, sub,
        "o conteúdo de uma subsecção vive num container de cor DIFERENTE (pedido do dono)"
    );
    let lum = |c: ph2d_vector::Color| {
        let [r, g, b, _] = c.components;
        r + g + b
    };
    assert!(
        lum(sub) > lum(section),
        "o degrau é para CIMA: a subsecção assenta sobre o cartão do pai"
    );
}

/// ⛔ **O clássico continua a ser o clássico** — risco, nenhum cartão.
#[test]
fn the_classic_family_still_draws_the_rule_and_no_card() {
    for theme in Theme::CLASSIC {
        let mut scene = VectorScene::new();
        with_section_cards(&mut scene, theme, 0.0, |scene| {
            let y = close_section(scene, theme, 0.0, 100.0, 40.0);
            assert!(y > 40.0);
        });
        assert!(
            path_count(&scene) > 0,
            "o {theme:?} tem de continuar a desenhar o risco entre secções"
        );
    }
}

/// ⭐⭐ **Um corpo DENTRO de outro devolve o do pai** — a razão de o livro ser uma PILHA.
///
/// ⚠️ O Inspector pinta popovers e o showcase pinta notas; nada impede um corpo de abrir dentro
/// de outro. Com um slot único, o de dentro apagaria o de fora ao sair — e o painel de fora
/// perderia os cartões **e o conteúdo**, porque a cena estacionada dele mora ao lado do livro.
#[test]
fn a_body_inside_another_gives_the_parent_its_own_back() {
    let theme = Theme::MODERN[0];
    let mut scene = VectorScene::new();

    let n = with_section_cards(&mut scene, theme, 0.0, |scene| {
        fill_rounded_rect(
            scene,
            Rect::new(0.0, 0.0, 100.0, 20.0),
            0.0,
            resolve(ColorToken::Text1, theme),
        );
        let inner = with_section_cards(scene, theme, 40.0, |scene| {
            fill_rounded_rect(
                scene,
                Rect::new(0.0, 40.0, 100.0, 20.0),
                0.0,
                resolve(ColorToken::Text2, theme),
            );
            close_section(scene, theme, 0.0, 100.0, 60.0)
        });
        assert!(inner > 60.0, "o corpo de dentro fechou o cartão dele");
        // ⭐⭐ **A prova mede o INSTANTE, e é isso que a torna honesta.** Um cartão não pinta nada
        //    quando fecha — ele só é RECOLHIDO, e só chega à cena no `end`. Um risco pinta-se
        //    ali e já. ⇒ se o corpo de dentro tivesse apagado o livro do de fora, esta chamada
        //    cairia no braço clássico e a contagem de caminhos subiria **aqui**.
        //    ⚠️ A 1.ª redacção deste gate media a cena no FIM e uma mutação sobreviveu: com o
        //    livro perdido o risco desenha, a contagem final continua alta, e o `y` devolvido
        //    também avança. *Duas coisas diferentes que dão o mesmo número no fim.*
        let before = path_count(scene);
        let out = close_section(scene, theme, 0.0, 100.0, 30.0);
        assert_eq!(
            path_count(scene),
            before,
            "o corpo de fora perdeu o livro: isto desenhou um RISCO em vez de recolher um cartão"
        );
        out
    });

    assert!(n > 30.0, "o corpo de fora continuou a fechar cartões");
    assert!(
        path_count(&scene) >= 4,
        "os dois corpos e os dois cartões têm de estar todos na cena (medido {})",
        path_count(&scene)
    );
}

/// ⛔⛔ **UM CARTÃO DENTRO DE OUTRO DA MESMA COR NÃO É UM CARTÃO — é o mesmo cartão.**
///
/// Report do dono, 2026-09-06, com as duas telas lado a lado: *«em Audio Editor: Effects temos o
/// card. Já o card de Painter: Jitter não se vê mais.»*
///
/// ⚠️ **Ele não sumiu — foi ENGOLIDO.** O Painter já pintava um cartão próprio para o *Jitter* (e
/// mais onze) em `Bg1`, e a wave 12 pôs um cartão de SECÇÃO por trás dele, também `Bg1`. Dois
/// tons iguais encostados leem-se como uma superfície só.
///
/// ⚠️⚠️ **E nenhuma régua desta linha o via**, o que é o mais caro: a cena CRESCEU (os 9 cartões do
/// pincel estão lá, com 296 e 609 px de altura), o `close_section` foi chamado, o gate do cartão
/// ficou verde — *tudo o que se media respondia «sim, há cartão»*. O que faltava medir era o
/// CONTRASTE entre profundidades vizinhas.
#[test]
fn two_nested_depths_never_paint_the_same_tone() {
    // ⛔ **O OLED fica de fora, e a exceção é HERDADA, não inventada:** o gate irmão
    // `a_card_stands_off_its_panel` (ph2d-tokens) já a declara com a medição — *«base preta e
    // contraste 0 colapsam a família, e quem separa lá é a Draw Extra Borders, como no Godot»*.
    // ⚠️ Medido aqui em 2026-09-06: painel, secção e subsecção lêem **0, 0, 0** — no OLED nenhum
    // cartão do app é visível, o que é o preço do preto puro e **não** uma regressão desta wave.
    for theme in [Theme::Dark, Theme::Gray, Theme::Light] {
        let section = resolve(CardDepth::Section.token(), theme);
        let sub = resolve(CardDepth::Subsection.token(), theme);
        let d = |a: ph2d_vector::Color, b: ph2d_vector::Color| {
            let (x, y) = (a.components, b.components);
            (0..3).map(|i| (x[i] - y[i]).abs()).fold(0.0_f32, f32::max)
        };
        // ⚠️⚠️ **A barra é «DIFERENTES», e não um número — de propósito.** A tentação era herdar os
        // 12/255 que o dono aprovou em §7.7, mas aquele par é *cartão contra PAINEL*; este é
        // *subsecção contra SECÇÃO*, e a escada dá-lhe **10** (`#1f1f1f → #292929`). Pôr 12 aqui
        // seria calibrar uma pergunta com a resposta de outra — o defeito que esta linha já
        // registou duas vezes. *Se 10 chega ao olho é veredito do dono, e este gate defende o que
        // se pode provar sem ele: que as três superfícies são TRÊS.*
        let panel = resolve(ColorToken::PanelBg, theme);
        let lum = |c: ph2d_vector::Color| {
            let [r, g, b, _] = c.components;
            r + g + b
        };
        // ⚠️ **A direcção é do TEMA, não da lei** — num tema claro a escada DESCE (as superfícies
        // escurecem ao aninhar), e a 1.ª redacção deste gate exigia subida e reprovou o `Light`
        // sobre uma escada correcta. *Uma lei escrita na polaridade de um tema é uma lei sobre
        // aquele tema.* O que é invariante é a MONOTONIA: cada degrau afasta-se do anterior no
        // mesmo sentido.
        let (a, b, c) = (lum(panel), lum(section), lum(sub));
        assert!(
            (a < b && b < c) || (a > b && b > c),
            "{theme:?}: a escada de fundos deixou de ser monótona — painel {:.0}, secção {:.0}, \
             subsecção {:.0} (em 765)",
            a * 255.0,
            b * 255.0,
            c * 255.0
        );
        assert!(
            d(section, sub) > 0.0 && d(panel, section) > 0.0,
            "{theme:?}: dois degraus vizinhos pintam o MESMO tom — um cartão dentro do outro \
             desaparece, que foi exactamente o report do Painter/Jitter (seccao a subseccao: {:.1}/255)",
            d(section, sub) * 255.0
        );
    }
}
