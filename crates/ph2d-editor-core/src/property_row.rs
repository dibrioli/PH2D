//! ⭐⭐⭐ **O PINTOR de uma linha de propriedade — a porta que TODO painel usa.**
//!
//! ⛔⛔ **Ela mudou-se do `ph2d-panel-inspector` para cá em 2026-09-15, e o motivo é uma medição:**
//! sete painéis pintam campos numéricos (`flip-frames` · `grid-snap` · `motion-graph` ·
//! `motion-params` · `painter-layers` · `timeline` · `vector`) e **nenhum** passava pela porta —
//! cada um com a sua aritmética. Só o `painter-layers` tem **oito** larguras de coluna de rótulo
//! escritas à mão (`96` · `70` · `62` · `60` · `60` · `54` · `44` · `38`), e o cabeçalho do
//! `number_field.rs` dele declara-se *«the SAME number box as the Inspector's Transform (the app
//! standard) … X/Y pairs share one line with red X / green Y axis tags like the reference»*.
//!
//! ⚠️⚠️ **Isto é: ele é uma CÓPIA da linha do Transform, com os defeitos que o Transform acabou de
//! largar** — a largura fixa (que a spec §3 prova errada por construção: o dock é arrastável) e as
//! letras de eixo numa coluna própria (que custam a disposição que o dono aprovou). *Uma porta que
//! vive dentro de um painel é uma porta que o painel seguinte copia.*
//!
//! A lei que estas funções servem está em `docs/UI_New_and_Simple/spec/03_a_linha_de_propriedade.md`.

use crate::interaction::{HitIndex, WidgetStore};
// ⭐⭐ **A [`Seccao`] e as colunas vivem no `widget`, e são re-exportadas AQUI** — o módulo
//    `property_row` já depende do `widget`, e pôr a declaração do lado de cá faria o `widget`
//    depender dele: um CICLO, que o `architecture_the_foundation_modules_form_a_dag` recusa. *A
//    declaração de uma linha pertence a quem sabe repartir a linha.*
use crate::paint::resolve;
use crate::widget::showcase::read_number_input;
use crate::widget::{NumberInput, PropertyRow, Unit, paint_number_input_with_buffer};
pub use crate::widget::{Seccao, colunas_da_linha};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// ⭐⭐⭐ **O NOME de uma linha, pintado à esquerda — e devolve ONDE o controlo vai.**
///
/// ⛔⛔ Para as linhas cujo controlo o chamador CONSTRÓI: um segmentado com «nenhum aceso», uma
/// grelha, quatro amostras de cor.
///
/// ⚠️ **O chamador pinta o ponto** (`paint_decorator_dot(scene, theme, row.dot)`) — ele não pode ser
/// pintado aqui porque a altura do controlo só se sabe depois de o desenhar, e há controlos que
/// refluem.
#[allow(clippy::too_many_arguments)]
pub fn paint_label_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    h: f32,
    label: &str,
    seccao: Seccao,
) -> PropertyRow {
    let row = colunas_da_linha(x, w, y, h, seccao);
    let label_font = TypeToken::Sm.px();
    crate::widget::paint_property_label(
        text_system,
        scene,
        label,
        row.label.x,
        row.label.y + (row.label.h - label_font) * 0.5,
        label_font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    row
}

/// ⭐⭐⭐ **A GEOMETRIA de uma linha, e o NOME já pintado — UMA derivação, N pintores.**
///
/// ⛔⛔ **Ela existe porque há rotas que não pintam a partir do store.** O painel da Grelha desenha
/// linhas cujo valor vem do ESTADO e não do `WidgetStore` (um `NodeId` partilhado por vários tipos
/// de grelha espelha o campo do tipo ACTIVO), e o passe de pintura de um painel não tem a loja
/// mutável para lá escrever. ⇒ elas precisam da geometria e do nome, e pintam o campo por sua conta.
///
/// ⚠️⚠️ **Extrair isto é o oposto de duplicar:** sem ela, a segunda rota re-derivaria *«onde é que a
/// coluna do nome acaba»* — e esta linha já pagou duas vezes o preço de duas derivações da mesma
/// grandeza. *Um pintor a mais é barato; uma segunda conta da mesma coisa não.*
#[allow(clippy::too_many_arguments)]
fn row_and_layout(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    n_campos: usize,
    seccao: Seccao,
) -> (PropertyRow, usize, usize, f32) {
    let label_font = TypeToken::Sm.px();
    let gap = ph2d_tokens::control_gap_px();
    // ⛔⛔ **O nome vem da SECÇÃO, nunca desta linha** — ver [`Seccao`], com a foto e a tabela.
    //    Uma linha que tem MAIS componentes do que a secção declarou continua a ser servida por
    //    elas (o `max`), mas a COLUNA não muda: ela é da secção.
    let seccao = seccao.com_pelo_menos(n_campos);
    let row = colunas_da_linha(x, w, y, ROW_H_PX, seccao);
    crate::widget::paint_property_label(
        text_system,
        scene,
        label,
        row.label.x,
        row.label.y + (row.label.h - label_font) * 0.5,
        label_font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    let (por_linha, linhas, cw) =
        crate::widget::property_fields_layout(row.control.w, n_campos, gap, 0.0);
    (row, por_linha, linhas, cw)
}

/// ⭐⭐⭐ **A LINHA DE VÁRIAS COMPONENTES — o nome à ESQUERDA, as caixas na coluna do controlo.**
///
/// ⛔⛔ **Report do dono, 2026-09-14** (*«Label acima do campo numérico! Muito ruim!»*) e
/// **2026-09-15** (*«A disposição ficou diferente … Position X/Y Caixa Caixa»* · *«O painel ainda
/// largo com espaço à esquerda e as linhas já se quebram … Isso não pode acontecer»*). As três
/// respostas estão aqui dentro e em mais lado nenhum:
///
/// 1. o nome à esquerda, alinhado à direita, elidido (spec §3 e §4);
/// 2. o que não cabe ao piso do campo **desce** dentro da coluna do controlo (§6-bis);
/// 3. a coluna do nome **cede** ao controlo antes de deixar a linha quebrar (§6-ter).
///
/// ⚠️ **UM ponto por LINHA, nunca por campo:** um par `X`/`Y` é *uma* propriedade com duas
/// componentes, e dois pontos diriam que são duas.
///
/// ⚠️ **A [`Seccao`] é da SECÇÃO e não desta linha** — ver a §6-ter da spec e o doc dela: se cada
/// linha cedesse pelo que ELA precisa, a coluna sairia esfarrapada. Os `campos` são **a linha que a
/// secção não quer ver quebrar**, e ⛔ não o máximo mecânico: uma secção com uma row de 4 campos que
/// só cabe num painel de `~700` declara `2`, senão ela deixa de ceder e o par `X`/`Y` volta a
/// quebrar.
#[allow(clippy::too_many_arguments)]
fn paint_fields_row_inner(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    field_ids: &[NodeId],
    step: f64,
    unit: Option<Unit>,
    seccao: Seccao,
) -> (f32, Option<Rect>) {
    let (row, por_linha, linhas, cw) = row_and_layout(
        text_system,
        scene,
        theme,
        x,
        w,
        y,
        label,
        field_ids.len(),
        seccao,
    );
    let gap = ph2d_tokens::control_gap_px();
    let passo = ph2d_tokens::row_pitch_px();
    let mut primeiro = None;
    for (i, &id) in field_ids.iter().enumerate() {
        let rect = Rect::new(
            row.control.x + (cw + gap) * (i % por_linha) as f32,
            row.control.y + passo * (i / por_linha) as f32,
            cw,
            ROW_H_PX,
        );
        hit_index.register(id, rect);
        if i == 0 {
            primeiro = Some(rect);
        }
        let (state, value, buffer, caret, anchor) = read_number_input(store, id);
        let input = NumberInput::new(id, "", value)
            .step(step)
            .visual((state, store.hover_live(id)))
            .suffix(unit.map(Unit::suffix));
        paint_number_input_with_buffer(
            &input,
            Some(buffer),
            caret,
            anchor,
            rect,
            scene,
            text_system,
            theme,
        );
    }
    crate::widget::paint_decorator_dot(scene, theme, row.dot);
    (y + passo * linhas as f32, primeiro)
}

/// ⭐⭐⭐ **A LINHA DE VÁRIAS COMPONENTES** — ver [`paint_fields_row_inner`]. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub fn paint_fields_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    field_ids: &[NodeId],
    step: f64,
    unit: Option<Unit>,
    seccao: Seccao,
) -> f32 {
    paint_fields_row_inner(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label,
        field_ids,
        step,
        unit,
        seccao,
    )
    .0
}

/// ⭐⭐ **A mesma linha, com UM campo — e devolve ONDE ele ficou.**
///
/// ⛔⛔ **Ela existe porque há quem desenhe POR CIMA do campo:** o painel do vector risca a caixa
/// quando o valor está preso a um token (*«um token cobre este número»*), e para isso precisa do
/// rectângulo que o pintor de facto usou.
///
/// ⚠️⚠️ **É a MESMA implementação, não uma segunda conta** — as duas entradas chamam o
/// [`paint_fields_row_inner`]. *Uma segunda derivação de «onde é que o campo ficou» divergiria no
/// dia em que a coluna do nome mudasse de lei, e a risca apareceria ao lado do número em vez de
/// sobre ele* — que é literalmente o defeito que a `widget::surface_rect` existe para impedir, um
/// nível abaixo.
#[allow(clippy::too_many_arguments)]
pub fn paint_field_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id: NodeId,
    step: f64,
    unit: Option<Unit>,
    seccao: Seccao,
) -> (f32, Rect) {
    let (next_y, rect) = paint_fields_row_inner(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label,
        &[id],
        step,
        unit,
        seccao,
    );
    (
        next_y,
        rect.expect("uma linha de UM campo pinta sempre esse campo"),
    )
}

/// ⭐⭐⭐ **CABE uma linha de propriedade com este nome dentro de `w`?**
///
/// ⛔⛔ **Report do dono, 2026-09-15, com duas fotos (painel largo e estreito):** *«Em Grain:
/// Voronoi : Metric e Edges os nomes somem ao estreitar o painel. Melhor seria quebrar a linha»*.
///
/// Aquelas duas vivem **emparelhadas**, cada uma numa METADE da largura do painel. Ao estreitar, a
/// coluna do nome de uma metade fica menor do que a reticência e o
/// [`crate::widget::paint_property_label`] devolve **string vazia** — que é a resposta certa dele
/// (*«a caixa fica só com o número, que é o degrau seguinte da escada do estreito»*) e a **errada**
/// para quem escolheu emparelhar: *o degrau a seguir a «não cabe o nome» não é apagar o nome, é
/// deixar de emparelhar.*
///
/// ⇒ quem empareelha pergunta ANTES. ⚠️ **Ela pergunta à PORTA, não a uma segunda aritmética:** o
/// veredito sai da [`crate::widget::property_row_columns_for`], a mesma que vai desenhar. *Duas
/// contas para «isto cabe?» divergem no dia em que uma das leis muda.*
#[must_use]
pub fn property_row_fits(w: f32, label_w: f32) -> bool {
    let piso = crate::widget::NUMBER_INPUT_MIN_W_PX;
    let row =
        crate::widget::property_row_columns_for(0.0, w, 0.0, ROW_H_PX, Some(label_w), Some(piso));
    row.label.w >= label_w - 0.01 && row.control.w >= piso - 0.01
}

/// ⭐⭐⭐ **A linha de UM campo cujo VALOR o chamador traz** — devolve o `y` seguinte e o rect do campo.
///
/// ⛔⛔ **Ela existe para quem não pinta a partir do store.** O painel da Grelha desenha linhas cujo
/// valor vem do ESTADO (um `NodeId` partilhado por vários tipos de grelha espelha o campo do tipo
/// ACTIVO), e o passe de pintura de um painel recebe a loja por `&`, não por `&mut` — não há onde
/// espelhar. ⇒ o chamador traz `valor`, `buffer`, `caret`, `âncora` e o par visual.
///
/// ⚠️ **A geometria e o nome saem da MESMA [`row_and_layout`] que a irmã usa** — não há aqui uma
/// segunda conta de onde a coluna acaba.
#[allow(clippy::too_many_arguments)]
pub fn paint_field_row_value(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id: NodeId,
    value: f64,
    buffer: Option<&str>,
    caret: usize,
    anchor: Option<usize>,
    visual: (crate::widget::TextInputState, f32),
    unit: Option<Unit>,
    seccao: Seccao,
) -> (f32, Rect) {
    let (row, _, linhas, cw) = row_and_layout(text_system, scene, theme, x, w, y, label, 1, seccao);
    let rect = Rect::new(row.control.x, row.control.y, cw, ROW_H_PX);
    hit_index.register(id, rect);
    let input = NumberInput::new(id, "", value)
        .visual(visual)
        .suffix(unit.map(Unit::suffix));
    paint_number_input_with_buffer(
        &input,
        buffer,
        caret,
        anchor,
        rect,
        scene,
        text_system,
        theme,
    );
    crate::widget::paint_decorator_dot(scene, theme, row.dot);
    (y + ph2d_tokens::row_pitch_px() * linhas as f32, rect)
}

/// ⭐⭐⭐ **UMA LINHA DE MARCAR, pela porta** — o nome na coluna da secção, a caixa na do controlo.
///
/// ⛔⛔ Ela existe porque a linha de marcar é uma **linha de propriedade** (spec §6-quinquies) e
/// até 2026-09-15 cada painel montava a dele à mão: o rect, o `register`, o `checkbox_visual`, o
/// `bool → CheckboxValue`. São quatro passos, e **esquecer o terceiro deixa a caixa sem hover sem
/// nada deixar de compilar** — a família que o [`crate::widget::Checkbox::visual`] já nomeia.
///
/// ⚠️ **O valor é um `bool` do MODELO, nunca do store** — a lei que a §11 do Inspector escreveu
/// para a caixa *Playing*: ler o visual faria o primeiro clique depois de trocar de objecto mandar
/// o valor do objecto **anterior**.
#[allow(clippy::too_many_arguments)]
pub fn paint_check_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    caixa: (NodeId, &str, bool),
    seccao: Seccao,
) -> f32 {
    let (id, label, on) = caixa;
    let rect = Rect::new(x, y, w, ROW_H_PX);
    hit_index.register(id, rect);
    let cb = crate::widget::Checkbox::new(id, label)
        .visual(store.checkbox_visual(id))
        .value(if on {
            crate::widget::CheckboxValue::Checked
        } else {
            crate::widget::CheckboxValue::Unchecked
        })
        .seccao(seccao);
    crate::widget::paint_checkbox(&cb, rect, scene, text_system, theme);
    y + ph2d_tokens::row_pitch_px()
}

/// ⭐⭐⭐ **AS LINHAS DE MARCAR DE UMA SECÇÃO — emparelhadas quando CABEM, empilhadas quando não.**
///
/// ⛔⛔⛔ **Medido em 2026-09-15, depois de a marca ganhar CAIXA** (§6-quinquies): as **cinco**
/// fileiras emparelhadas de booleanos do Inspector (*Animation* · *Timers* · *Audio* · *Camera* ·
/// o *Flip H/V* da folha) ficaram com os **dez** nomes CORTADOS em todo o curso útil do dock:
///
/// | painel | METADE da linha | coluna do nome que sobra | os nomes medem |
/// |---|---|---|---|
/// | `220` (mínimo do dock) | `83,0` | **`0,0`** ⛔ (nome nenhum é pintado) | `28,5`–`79,8` |
/// | `273,3` (a do dono) | `109,6` | **`15,6`** ⛔ | idem |
/// | `304` (omissão) | `125,0` | **`31,0`** ⛔ | idem |
/// | `420` | `183,0` | `83,5` ✅ | idem |
///
/// ⚠️⚠️ **A causa não é o emparelhamento: é que uma METADE não tem onde pôr a caixa.** O piso do
/// campo é [`crate::widget::NUMBER_INPUT_MIN_W_PX`] (`72`, ordem do dono de 2026-05-24) e numa
/// metade de `109,6` ele come tudo menos `15,6` — *o tecto do §6 e a metade colidem, e quem perde é
/// o nome*. Antes da caixa a marca media `18` e sobrava linha para o nome; **a aparência que o dono
/// mandou adoptar não cabe em meia linha.**
///
/// ⇒ a lei do §6-quater aplica-se aqui **tal como está escrita**: *emparelhar é uma escolha do
/// painel; caber é uma medição da porta.* ⛔ E a pergunta vai à [`property_row_fits`], nunca a uma
/// segunda aritmética.
///
/// ⚠️ **Quando emparelham, a «secção» das duas é o PAR** — as duas metades partilham uma coluna
/// medida sobre os dois nomes, senão a da esquerda e a da direita caem em `x` diferentes dentro da
/// mesma fileira. Quando desemparelham, cada uma entra na coluna da **secção**, que é o que as
/// alinha com os números ao lado.
#[allow(clippy::too_many_arguments)]
pub fn paint_check_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    caixas: &[(NodeId, &str, bool)],
    seccao: Seccao,
) -> f32 {
    let gap = ph2d_tokens::Spacing::Sm.px();
    let meia = ((w - gap) * 0.5).max(1.0);
    let fonte = TypeToken::Sm.px();
    let mut cur_y = y;
    let mut i = 0;
    while i < caixas.len() {
        // ⚠️ **A medida é a do nome MAIS LARGO do par**, não a de cada um: as duas metades vão
        //    partilhar uma coluna só, logo é ela que tem de caber. *Perguntar por nome aprovaria um
        //    par cuja coluna comum não cabe em nenhuma das duas.*
        let par = (i + 1 < caixas.len()).then(|| {
            text_system
                .prefix_width(caixas[i].1, fonte)
                .max(text_system.prefix_width(caixas[i + 1].1, fonte))
        });
        match par {
            Some(quer) if property_row_fits(meia, quer) => {
                let sec = Seccao::medida(text_system, 1, &[caixas[i].1, caixas[i + 1].1]);
                paint_check_row(
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                    x,
                    meia,
                    cur_y,
                    caixas[i],
                    sec,
                );
                cur_y = paint_check_row(
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                    x + meia + gap,
                    meia,
                    cur_y,
                    caixas[i + 1],
                    sec,
                );
                i += 2;
            }
            _ => {
                cur_y = paint_check_row(
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                    x,
                    w,
                    cur_y,
                    caixas[i],
                    seccao,
                );
                i += 1;
            }
        }
    }
    cur_y
}

/// ⭐⭐⭐ **A LINHA DE COR — o nome à esquerda, a amostra a ENCHER a coluna do valor.**
///
/// ⛔⛔ **Report do dono, 2026-09-21, com uma foto do Inspector e um DESENHO ao lado:** *«os
/// seletores de cor de todo o app precisam ser padronizados»*. Ele desenhou a amostra como uma
/// **barra larga** na coluna do valor — como a caixa de marcar e o campo numérico — e não como o
/// quadradinho encostado à direita que o app pinta hoje.
///
/// ⛔ **MEDIDO no mesmo dia:** o app pinta **`109`** selectores de cor em **CINCO** larguras
/// diferentes — `18`, `24`, `32`, `120` e `268 px`. ⭐ E a forma que ele desenhou **já existia**,
/// num painel só: o `authored`, que é o painel gerado por TABELA (`268 px`, a coluna inteira). *O
/// padrão não foi inventado aqui — foi promovido a porta.*
///
/// ⚠️ **Ela não é uma segunda aritmética de colunas:** passa pela [`row_and_layout`], a mesma do
/// campo e da caixa, com **um** campo — e um campo só recebe a coluna do controlo inteira. É isso
/// que a alinha com as vizinhas, que é a outra metade do report (*«o alinhamento precisa melhorar
/// em todos os lugares»*).
///
/// ⚠️ **O hit é a BARRA INTEIRA**, não o quadradinho: com o alvo do tamanho do controlo, apontar a
/// cor deixa de ser pontaria.
#[allow(clippy::too_many_arguments)]
pub fn paint_color_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id: NodeId,
    fallback_rgba: [u8; 4],
    mixed: bool,
    seccao: Seccao,
) -> f32 {
    let (row, _por_linha, _linhas, cw) =
        row_and_layout(text_system, scene, theme, x, w, y, label, 1, seccao);
    let rect = Rect::new(row.control.x, row.control.y, cw, ROW_H_PX);
    // ⚠️ Quem resolve o valor é ESTE lado — ver o doc do `paint_swatch_or_mixed`.
    let rgba = (!mixed).then(|| store.widget_color(id).unwrap_or(fallback_rgba));
    crate::widget::paint_swatch_or_mixed(rect, id, rgba, scene, theme);
    hit_index.register(id, rect);
    crate::widget::paint_decorator_dot(scene, theme, row.dot);
    y + ph2d_tokens::row_pitch_px()
}
