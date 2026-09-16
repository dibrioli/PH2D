//! ⭐⭐⭐ **UMA MARCA NUNCA PINTA A COR DA CAIXA EM QUE ASSENTA.**
//!
//! ⛔⛔ **Report do dono, 2026-09-15, com foto:** *«O Checkbox desmarcado é invisível»* — a linha
//! *Autoplay* mostrava a palavra *On* e mais nada.
//!
//! # ⚠️ É a MESMA queixa da véspera, um degrau mais fundo — e quem moveu o degrau fui eu
//!
//! Em 2026-09-14 o dono reportou *«Checkbox invisível»*: a marca desmarcada enchia `Bg1`, a cor do
//! CARTÃO em que ela assentava, e num tema moderno não há moldura de repouso. A cura foi a
//! [`ph2d_editor_core::paint::body_fill`] — *um degrau abaixo da superfície mais funda*, que é o
//! [`ph2d_tokens::visuals::Chrome::field_fill`].
//!
//! Em 2026-09-15 a marca **mudou-se para dentro de uma CAIXA DE CAMPO** (spec §6-quinquies, ordem
//! do dono com a foto do Godot). O fundo dessa caixa é exactamente `field_fill`. Medido:
//!
//! | tema | a caixa | a marca desmarcada | antes | depois |
//! |---|---|---|---|---|
//! | Dark | `#090909` | `#292929` | **`0`/255** ⛔ | `32` |
//! | Gray | `#121212` | `#3D3D3D` | **`0`/255** ⛔ | `43` |
//! | Light | `#DBDBDB` | `#E6E6E6` | **`0`/255** ⛔ | `11` |
//!
//! ⚠️⚠️ **A lei que isto escreve** (`CLAUDE.md` §0.0): *uma cura de contraste é calibrada contra
//! uma SUPERFÍCIE, e quem move a peça para outra superfície tem de reconferir a nota.* A de 14/09
//! estava escrita como se fosse absoluta.
//!
//! # ⛔ A cláusula da MOLDURA, e porque não é uma isenção
//!
//! Onde o tema traça moldura de repouso (a família clássica, e o OLED com *Draw Extra Borders*) a
//! marca lê-se pela borda e a distância de tom pode ser zero — a **mesma** cláusula que o
//! `a_field_is_never_the_colour_of_what_it_sits_on` do `ph2d-tokens` já declara por escrito para a
//! relação campo-contra-cartão. ⚠️ Ali as superfícies são três e **a caixa de campo não era uma
//! delas**: até 15/09 nenhum controlo assentava num campo.

use ph2d_editor_core::paint::{body_fill, on_field_fill};
use ph2d_tokens::derive::SURFACE_STEP;
use ph2d_tokens::visuals::{Chrome, Feel};
use ph2d_tokens::{Color, ColorToken, Theme};

/// A distância entre dois tons, no pior canal, em unidades de `255` — a mesma régua do gate irmão.
fn dist(a: Color, b: Color) -> f32 {
    let f = |x: u8, y: u8| (f32::from(x) - f32::from(y)).abs();
    f(a.r, b.r).max(f(a.g, b.g)).max(f(a.b, b.b))
}

/// Os três estados em que a marca **não** é `Accent` — os únicos em que ela pode desaparecer.
/// ⚠️ A MARCADA é acento cheio e lê-se em qualquer tema; é por isso que o report do dono foi
/// sempre sobre **metade** das caixas do painel.
fn estados_sem_acento() -> [(&'static str, Feel, ColorToken); 3] {
    [
        ("repouso", Feel::Rest, ColorToken::Bg1),
        ("sob o rato", Feel::Hovered, ColorToken::Bg2),
        ("parada", Feel::Disabled, ColorToken::Bg2),
    ]
}

#[test]
fn a_marca_nunca_e_a_cor_da_caixa_em_que_assenta() {
    let barra = SURFACE_STEP * 255.0;
    let mut sem_moldura = 0;
    let mut medidos = 0;
    for theme in Theme::ALL {
        let chrome = Chrome::of(theme);
        if chrome.field_border.is_visible() {
            // A marca lê-se pela borda — ver o cabeçalho.
            continue;
        }
        sem_moldura += 1;
        for (nome, feel, classic) in estados_sem_acento() {
            let marca = on_field_fill(theme, feel, classic);
            let d = dist(marca, chrome.field_fill);
            assert!(
                d + 0.51 >= barra,
                "{theme:?}: a marca {nome} fica a {d:.0}/255 da CAIXA em que assenta (barra \
                 {barra:.0}) — sem moldura e sem degrau de tom NAO HA MARCA. Foi este o report do \
                 dono em 2026-09-15, com a marca a `0/255` da caixa."
            );
            medidos += 1;
        }
    }
    // ⚠️ **Piso de população:** um gate que salte todos os temas passa trivialmente. Os modernos
    // sem *Draw Extra Borders* são três (`Dark`, `Gray`, `Light`) — o OLED tem-nas.
    assert!(
        sem_moldura >= 3 && medidos >= 9,
        "o gate mediu {medidos} par(es) em {sem_moldura} tema(s) sem moldura — a varredura morreu"
    );
}

/// ⭐⭐ **O CONTROLO: a porta do CARTÃO seria invisível aqui, e é por isso que há duas.**
///
/// ⛔ Sem este teste, alguém que unificasse as duas portas «porque fazem quase o mesmo» apagaria a
/// cura sem nenhum gate reprovar: o `a_marca_nunca_e_a_cor_da_caixa_em_que_assenta` mede a porta
/// NOVA, e a antiga continuaria certa **para o cartão**, que é o que ela responde.
#[test]
fn e_a_porta_do_cartao_seria_exactamente_a_cor_da_caixa() {
    let mut vistos = 0;
    for theme in Theme::ALL {
        let chrome = Chrome::of(theme);
        if chrome.field_border.is_visible() {
            continue;
        }
        let no_cartao = body_fill(theme, Feel::Rest, ColorToken::Bg1);
        assert!(
            dist(no_cartao, chrome.field_fill) < 0.51,
            "{theme:?}: a `body_fill` deixou de devolver a cor do campo — se ela mudou, este \
             controlo deixou de descrever o defeito de 2026-09-15 e o doc das DUAS portas tem de \
             ser reconferido"
        );
        vistos += 1;
    }
    assert!(vistos >= 3, "o controlo mediu {vistos} tema(s)");
}

/// ⭐⭐⭐ **E O PINTOR USA A PORTA CERTA — medido na CENA, não no código.**
///
/// ⛔⛔ Sem esta metade, a cura fica **desligada e o gate verde**: os dois testes acima medem a
/// PORTA, e uma porta certa que ninguém chama produz exactamente o app do report. *É a família
/// «uma porta sem chamador e uma lei ausente produzem o mesmo app»*, que este repo já pagou.
///
/// A régua é a TINTA que o pintor de facto pousou (`encoding().draw_data`): numa linha de marcar
/// desmarcada, a cena tem de conter a cor da CAIXA **e** a da MARCA, e elas têm de ser duas.
///
/// **Mutação que sangra:** trocar o `corpo(..)` do `checkbox/mark.rs` de volta pela
/// `body_fill(..)` ⇒ a cor da marca desaparece da cena (a caixa e a marca passam a ser o mesmo
/// `u32`).
#[test]
fn e_o_pintor_da_linha_de_marcar_usa_essa_porta() {
    use ph2d_editor_core::widget::{Checkbox, CheckboxValue, paint_checkbox};
    use ph2d_editor_core::zones::Rect;

    let mut vistos = 0;
    for theme in Theme::ALL {
        let chrome = Chrome::of(theme);
        if chrome.field_border.is_visible() {
            continue;
        }
        let mut scene = ph2d_vector::VectorScene::new();
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        let cb = Checkbox::new(ph2d_a11y::NodeId(1), "Autoplay")
            .value(CheckboxValue::Unchecked)
            .seccao(ph2d_editor_core::property_row::Seccao::apenas_campos(1));
        paint_checkbox(
            &cb,
            // ⚠️ Larga o bastante para a caixa existir: numa linha estreita o controlo reflui e
            //    esta régua mediria outra coisa.
            Rect::new(0.0, 0.0, 256.0, ph2d_tokens::ROW_H_PX),
            &mut scene,
            &mut text,
            theme,
        );
        let tintas = scene.inner().encoding().draw_data.clone();
        let argb = |c: Color| {
            (u32::from(c.a) << 24) | (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b)
        };
        let caixa = argb(chrome.field_fill);
        let marca = argb(on_field_fill(theme, Feel::Rest, ColorToken::Bg1));
        assert!(
            caixa != marca,
            "{theme:?}: a porta devolve a cor da caixa — o teste de cima ja' o disse"
        );
        assert!(
            tintas.contains(&caixa),
            "{theme:?}: a CAIXA do campo nao foi pintada ({tintas:08X?}) — esta regua deixou de \
             medir uma linha de marcar de formulario"
        );
        assert!(
            tintas.contains(&marca),
            "{theme:?}: a marca desmarcada NAO usa a porta do campo ({tintas:08X?}) — ela foi \
             pintada com a cor da caixa e o artista nao a ve. Report do dono de 2026-09-15."
        );
        vistos += 1;
    }
    assert!(vistos >= 3, "a varredura mediu {vistos} tema(s)");
}
