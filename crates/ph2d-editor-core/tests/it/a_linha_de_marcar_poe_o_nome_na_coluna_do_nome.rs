//! ⭐⭐⭐ **O NOME DE UMA LINHA DE MARCAR VIVE NA COLUNA DO NOME — como o das linhas de número.**
//!
//! ⛔⛔ Até 2026-09-15 ele era pintado **encostado à esquerda da faixa**, a correr até à marca, com
//! um orçamento próprio. Num formulário que alterna linhas de número com linhas de marcar isso dá
//! **duas colunas de nome**, alternando linha sim linha não: as de número acabam no meio da linha,
//! alinhadas à direita (ordem do dono, 2026-09-14: *«as labels alinhadas todas à direita»*), e as de
//! marcar começavam na margem esquerda.
//!
//! *É a mesma queixa do rótulo por cima do campo, meia volta adiante* — e o widget é o mais usado do
//! app (**81** sítios fora desta crate).
//!
//! # ⚠️ A marca NÃO se move, e isso é deliberado
//!
//! Ela partilha a coluna do valor com o número, e é isso que dá ao formulário **uma** margem direita
//! (`the_form_has_one_right_margin`, que continua verde). ⇒ entre o nome e a marca fica um vão, e
//! ele é o mesmo em todas as linhas de marcar.
//!
//! # ⚠️⚠️ A régua é a CONTAGEM DE GLIFOS, e porquê
//!
//! Três formas foram tentadas, e as duas primeiras **não viam o sujeito**:
//!
//! | régua | porque falhou |
//! |---|---|
//! | `path_data` / `n_path_segments` | o Vello encaminha texto por `draw_glyphs` — **nenhum glifo entra na contagem de caminhos** (lição já paga no L-System e no `a_label_that_fits_is_never_painted_with_dots`) |
//! | `draw_data` das duas cenas | duas cenas iguais davam bytes diferentes, e o **controlo** (`tinta == tinta`) reprovou primeiro — *o instrumento media-se a si mesmo* |
//! | **glifos por corrida** | ✅ determinística, e mede o que o artista vê: *o nome foi cortado ou não* |
//!
//! ⛔ E comparar o `x` do rótulo com `colunas_da_linha(..).label` seria a mesma expressão dos dois
//! lados — vácua (a lição que o `the_lone_field_of_a_row_spans_what_the_pair_spans` pagou com uma
//! mutação **sobrevivente**).

use ph2d_a11y::NodeId;
use ph2d_editor_core::widget::{Checkbox, CheckboxState, CheckboxValue, Seccao, paint_checkbox};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// ⚠️ Estas leis são do REDESENHO, que é opcional (`PH2D_UI_NEW`) — o clássico pinta a linha de
/// sempre. ⛔ Sem isto o gate mediria o clássico com o nome do redesenho: verde, e sobre outra coisa.
fn redesign() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
}

const ID: NodeId = NodeId(1);

/// ⭐ **Um nome REAL do app, e dos mais longos** (a §12 do Inspector) — `144 px` a `Sm`.
///
/// ⚠️ **Ele tem de passar da metade da faixa abaixo**, senão as duas secções dão a mesma coluna e o
/// gate fica verde a medir o caso fácil. A asserção de enquadramento afirma-o.
const ROTULO: &str = "Show anchors at runtime";

/// A faixa: larga o suficiente para a marca caber com folga, estreita o suficiente para a **metade**
/// dela cortar o [`ROTULO`].
const FAIXA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 260.0,
    h: 22.0,
};

/// Quantos glifos a linha pousou na cena.
fn glifos_da_cena(scene: &VectorScene) -> usize {
    scene
        .inner()
        .encoding()
        .resources
        .glyph_runs
        .iter()
        .map(|r| r.glyphs.end - r.glyphs.start)
        .sum()
}

/// Pinta uma linha de marcar com a secção que o chamador declara, e conta os glifos.
fn glifos(ts: &mut TextSystem, sec: Option<Seccao>) -> usize {
    let mut scene = VectorScene::new();
    let mut cb = Checkbox::new(ID, ROTULO)
        .state(CheckboxState::Normal)
        .value(CheckboxValue::Unchecked);
    cb.seccao = sec;
    paint_checkbox(&cb, FAIXA, &mut scene, ts, Theme::Forge);
    glifos_da_cena(&scene)
}

/// As duas declarações: a secção que **não enumera** nomes (coluna = a metade da linha) e a que
/// declara este nome (coluna = o que ele precisa — o empréstimo da spec §6).
fn duas_seccoes(ts: &mut TextSystem) -> (Seccao, Seccao) {
    (Seccao::apenas_campos(1), Seccao::medida(ts, 1, &[ROTULO]))
}

#[test]
fn a_linha_de_marcar_poe_o_nome_na_coluna_do_nome() {
    redesign();
    let mut ts = TextSystem::new();
    let (sem_nomes, com_o_nome) = duas_seccoes(&mut ts);

    // ⭐⭐ **O ENQUADRAMENTO**: o nome tem de passar da metade, senão as duas colunas coincidem e o
    //     que se mede a seguir é o caso fácil — o gate ficaria verde com o defeito dentro.
    let metade = FAIXA.w * 0.5 - ph2d_tokens::Spacing::Md.px();
    let quer = ts.prefix_width(ROTULO, ph2d_tokens::TypeToken::Sm.px());
    assert!(
        quer > metade,
        "o rotulo mede {quer:.1} e a metade da faixa da' {metade:.1} — as duas seccoes dao a \
         mesma coluna e este gate deixou de distinguir alguma coisa"
    );

    // ⭐ **A LEI**: a coluna que a SECÇÃO declara decide se o nome é cortado.
    let cortado = glifos(&mut ts, Some(sem_nomes));
    let inteiro = glifos(&mut ts, Some(com_o_nome));
    assert!(
        cortado < inteiro,
        "a linha de marcar pintou {cortado} glifos com a coluna estreita e {inteiro} com a larga: \
         o nome nao esta' na coluna do nome — ele esta' encostado a' ESQUERDA da faixa, e o \
         formulario volta a ter duas colunas de nome"
    );

    // ⭐⭐ **O CONTROLO da medição**: a mesma entrada duas vezes dá o mesmo número.
    assert_eq!(
        glifos(&mut ts, Some(sem_nomes)),
        cortado,
        "a contagem de glifos nao e' determinista — a assercao acima nao prova nada"
    );
}

/// ⚠️ **A segunda metade: a pele de canvas NÃO segue a coluna de uma secção que ali não existe.**
///
/// ⛔ Ela é o único sítio do app onde a moldura é o que o **artista** desenhou. Se a lei de cima lhe
/// chegasse, o nome que ele pôs saltaria para a coluna de um formulário que ele não está a ver.
#[test]
fn fora_do_formulario_o_nome_fica_onde_o_artista_o_pos() {
    redesign();
    let mut ts = TextSystem::new();
    let (_, com_o_nome) = duas_seccoes(&mut ts);
    let sem = glifos(&mut ts, None);
    // Declarar a secção e **depois** sair do formulário tem de dar a mesma contagem.
    let mut scene = VectorScene::new();
    let cb = Checkbox::new(ID, ROTULO)
        .seccao(com_o_nome)
        .fora_do_formulario();
    paint_checkbox(&cb, FAIXA, &mut scene, &mut ts, Theme::Forge);
    assert_eq!(
        sem,
        glifos_da_cena(&scene),
        "a pele de canvas passou a seguir a coluna de uma seccao"
    );
}

/// ⭐⭐⭐ **A MARCA VIVE DENTRO DE UMA CAIXA que ocupa a coluna do controlo.**
///
/// ⛔⛔ **Ordem do dono, 2026-09-15, com a foto do inspector do Godot:** *«coloca um box em todo o
/// lado direito da linha e dentro do box o checkbox alinhado à esquerda. Vamos adotar essa
/// aparência»*.
///
/// ⭐⭐ Com ela a linha de marcar deixa de ser a **excepção do §3** do manual: a caixa começa no meio
/// da linha e acaba na margem direita, exactamente como a caixa de um número — *a «uma margem
/// direita» passa a sair da própria caixa*.
///
/// # A régua
///
/// Aqui os **caminhos** servem (ao contrário do rótulo, que é glifo): a superfície do campo é um
/// rectângulo arredondado mais uma moldura, e isso são segmentos.
///
/// 1. **a caixa existe** — a linha de formulário desenha mais caminhos que a mesma marca fora dele;
/// 2. **a caixa é a coluna do CONTROLO** — trocar a secção move-a, e a tinta muda.
///
/// **Mutação que deve sangrar:** apagar a chamada à `paint_field_surface` no `paint_boolean_mark`.
#[test]
fn a_marca_vive_dentro_de_uma_caixa_que_ocupa_a_coluna_do_controlo() {
    redesign();
    let mut ts = TextSystem::new();
    let (sem_nomes, com_o_nome) = duas_seccoes(&mut ts);

    // Sem rótulo: o que se mede é a CAIXA, não o texto.
    let mut caminhos = |sec: Option<Seccao>| -> (u32, Vec<u32>) {
        let mut scene = VectorScene::new();
        let mut cb = Checkbox::new(ID, "")
            .state(CheckboxState::Normal)
            .value(CheckboxValue::Unchecked);
        cb.seccao = sec;
        paint_checkbox(&cb, FAIXA, &mut scene, &mut ts, Theme::Forge);
        let e = scene.inner().encoding();
        (e.n_path_segments, e.path_data.clone())
    };

    let (com_caixa, tinta_estreita) = caminhos(Some(sem_nomes));
    let (sem_caixa, _) = caminhos(None);
    assert!(
        com_caixa > sem_caixa,
        "a linha de formulario desenhou {com_caixa} segmentos e a marca sozinha {sem_caixa}:          a CAIXA nao esta' la'"
    );

    let (_, tinta_larga) = caminhos(Some(com_o_nome));
    assert_ne!(
        tinta_estreita, tinta_larga,
        "a caixa nao se moveu quando a coluna da seccao mudou: ela nao e' a coluna do CONTROLO"
    );
}

/// ⭐⭐⭐ **A PALAVRA DO VALOR VIVE DENTRO DA CAIXA, à direita da marca.**
///
/// ⛔⛔ **Ordem do dono, 2026-09-15, com a foto do inspector do Godot:** *«Godot define a checkbox
/// como uma palavra (On) à direita. Isso me parece bom»*.
///
/// ⚠️ **Ela não muda com o valor, e é assim no alvo:** na foto dele três linhas dizem *On* e só uma
/// está marcada. A palavra nomeia o que a marca LIGA — como o rótulo de um interruptor de parede —,
/// e quem diz se está ligado é a marca. ⛔ Eu tinha-a deixado de fora por achar que ela mentia, e
/// devolvi-lhe a decisão; **ele decidiu, e a decisão é dele**.
///
/// # A régua
///
/// Com o rótulo **VAZIO**, o único texto que a linha pode pousar é a palavra. ⇒ numa linha de
/// formulário há glifos; fora dela (a pele de canvas) há **zero**.
///
/// **Mutação que deve sangrar:** apagar o `paint_text` da palavra — as duas contagens dão `0`.
#[test]
fn a_palavra_do_valor_vive_dentro_da_caixa() {
    redesign();
    let mut ts = TextSystem::new();
    let conta = |ts: &mut TextSystem, sec: Option<Seccao>| -> usize {
        let mut scene = VectorScene::new();
        // ⚠️ Rótulo VAZIO de propósito: assim o único texto possível é a PALAVRA.
        let mut cb = Checkbox::new(ID, "")
            .state(CheckboxState::Normal)
            .value(CheckboxValue::Unchecked);
        cb.seccao = sec;
        paint_checkbox(&cb, FAIXA, &mut scene, ts, Theme::Forge);
        glifos_da_cena(&scene)
    };
    let com = conta(&mut ts, Some(Seccao::apenas_campos(1)));
    let sem = conta(&mut ts, None);
    assert!(
        com > 0,
        "a linha de formulario nao pousou glifo nenhum com o rotulo vazio: a PALAVRA nao esta' la'"
    );
    assert_eq!(
        sem, 0,
        "a pele de canvas pousou {sem} glifo(s) com o rotulo vazio — a palavra saiu da linha de \
         formulario e foi parar onde o artista desenha"
    );
}
