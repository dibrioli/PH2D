//! **O QUE A SEÇÃO DO PINCEL DESENHA ALÉM DOS KNOBS** — irmão do `body.rs` e do
//! `mask_tools.rs`, cortado por ASSUNTO.
//!
//! As rows contínuas saem da tabela e o `body.rs` as percorre genericamente;
//! aqui mora o que é do PINCEL e de mais ninguém — com que profundidade a seção
//! se mostra, a curva do peso, a família do padrão, o preview e o acumular.
//!
//! ⚠️ **A CABEÇA e a CAUDA da mesma seção viajam juntas**, e não é arrumação: as
//! duas são a moldura em que as rows genéricas caem, e separá-las poria *o que
//! vem antes* e *o que vem depois* em arquivos diferentes — a próxima pessoa a
//! mexer na ordem teria de descobrir isso.

use ph2d_editor_core::ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_sculpt3d::{Alpha, ClothArea, ClothMode, Falloff, Verb};
use ph2d_tokens::Spacing;

use super::body::paint_one_row;
use super::mask_tools::paint_mask_tools;
use super::widgets::{command, labelled_seg, toggle};

use crate::preview;
use crate::rows;
use crate::state::{Sculpt3dSnapshot, UiLevel};

/// **COM QUE PROFUNDIDADE OLHAR** — o par `Basic` · `Pro`, no TOPO da seção do
/// pincel (§2.3 do plano).
///
/// ⚠️ **Ele governa a seção do PINCEL e nada mais.** Sombreamento e topologia
/// descrevem *como a forma é lida* e *quão fino é o barro* — nenhum dos dois é
/// um knob que o verbo armou —, então um interruptor que os alcançasse esconderia
/// controles que ninguém escolheu por você, que é exatamente a linha que separa
/// divulgação progressiva de amputação.
///
/// ⚠️ **E ele fica DENTRO do cabeçalho dobrável**, não acima: quem fecha a seção
/// do pincel fechou o assunto inteiro, e um chip órfão pairando sobre uma seção
/// fechada seria um controle sem sujeito.
pub(super) fn paint_level_row(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let selected = UiLevel::ALL
        .iter()
        .position(|&l| l == snap.ui.ui_level)
        .unwrap_or(0);
    let labels: Vec<&str> = UiLevel::ALL.iter().map(|l| l.label()).collect();
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.ui_level"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_UI_LEVEL,
        &labels,
        selected,
        x,
        w,
        y,
    )
}

/// O que vem DEPOIS dos knobs do pincel: a curva e as operações de máscara.
pub(super) fn paint_brush_tail(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    // O falloff logo abaixo dos knobs: ele é a FORMA do peso, e a força é
    // quanto dele se aplica.
    //
    // ⚠️ **BASIC, e a premissa que ele era `Pro` foi REFUTADA pela referência.**
    // O argumento antigo era que a curva já vem escolhida a cada troca de
    // ferramenta (`VerbProfile::falloff`), logo esconder-lhe o acesso seria
    // divulgação progressiva. A regra do [`UiLevel`] admite isso — *só uma row
    // cujo valor a ferramenta já traz pode ser `Pro`* —, mas ser ADMISSÍVEL
    // não é ser certo, e a referência mede o contrário:
    //
    // * no `properties_paint_common.py` o `FalloffPanel` **não** é desenhado por
    //   `brush_settings_advanced` — ele é painel de primeira classe;
    // * no cabeçalho de ferramenta ele é um **popover sempre visível**
    //   (`layout.popover("VIEW3D_PT_tools_brush_falloff")`).
    //
    // Ou seja: no Blender a curva é *dobrada*, nunca *ausente* — o artista vê um
    // cabeçalho que diz que ela existe. O nosso `Pro` a tornava **invisível sem
    // rastro**, e é a diferença entre dobrar e amputar. Reportado no smoke da
    // demão: *"funciona corretamente mas não dá a opção de escolher o falloff"*.
    //
    // ⚠️ **E o que estava errado era a PREMISSA do Basic, não a lei dele:** o doc
    // do [`UiLevel::Basic`] diz *"o vocabulário do SculptGL"*, e o SculptGL **não
    // tem** seletor de curva — a dele é fixa. Herdar aquele vocabulário apagava
    // um controle que a nossa malha tem **doze** vezes.
    //
    // ⚠️ **Segue uma faixa que REFLUI, e não um dropdown**, pelo precedente que o
    // `tool.rs` já mediu para os vinte verbos: um dropdown esconde onze curvas
    // atrás de um clique para mostrar uma, e quem escolhe uma curva a escolhe
    // COMPARANDO. (O Blender troca para dropdown no painel estreito, mas o `seg`
    // desta casa reflui em vez de transbordar — a razão dele não se aplica.)
    // ⚠️ **A curva e' pintada SEMPRE, e isso e' uma CERCA — nao um esquecimento.**
    //
    // Em 2026-08-30 uma caca aos knobs mortos mediu, e a medicao esta' certa: com o verbo `Mask`
    // em maos o peso vem de `brush.mask_weight(t)` (que le^ **so'** a `mask_hardness`, nunca o
    // falloff), e com um campo elastico activo a curva inteira e' `kelvinlet::rim_landing`. Nesses
    // casos **este selector nao molda nada**.
    //
    // ⭐ **E a inercia deixou de ser uma AFIRMACAO e passou a ser MEDIDA**, pela porta do produto:
    // `ph2d-sculpt3d/tests/measure_where_the_curve_knobs_reach.rs` roda o MESMO gesto com duas
    // curvas e compara o barro (e o canal) **ao bit** nos tres regimes, com o controle positivo do
    // `Verb::Draw` ao lado. *Um comentario que diz «isto e' inerte» envelhece calado; um gate que o
    // mede sangra no dia em que deixar de ser verdade.*
    //
    // ⛔ **A CURA DE PRIMEIRA ESCOLHA — fazer o consumidor USAR o valor — foi TENTADA nos dois
    // regimes e MEDIDA a partir, nas duas vezes:**
    //
    // | regime | mutacao no produto | gate existente que sangrou | numero |
    // |---|---|---|---|
    // | `Verb::Mask` | `mask_weight(t) * falloff.weight(t)` | `the_mask_channel_reproduces_the_reference_kernel` | divergencia **1,201e-1** contra a barra de `1,192e-7` — 10⁶× |
    // | campo elastico | `rim_landing(t) * falloff.weight(shaped_distance(t))` | `the_stroke_delivers_what_the_kernel_promises` | o traco poe **−6,9e-6** onde o campo manda **−3,8e-4** (55× menos barro) |
    //
    // ⇒ No canal, o `Falloff` mediria *uma tool contra a curva de OUTRA* — a mascara tem curva
    // propria na referencia (`Masking.js:66-69`) e o `Verb::Mask` nasce com a quartica, que a
    // referencia nao aplica. No campo, o perfil **JA' E'** o falloff, e compor os dois o aplica
    // duas vezes: o agarre morre. As duas recusas sao de LEI, nao de gosto.
    //
    // ⛔ **Escondê-lo mesmo assim foi TENTADO no mesmo dia e REVERTIDO**, porque o gate
    // `the_basic_level_never_hides_the_curve_that_shapes_the_dab` o apanhou — e o doc dele carrega
    // a decisao, com referencia: no Blender o `FalloffPanel` e' painel de primeira classe e um
    // popover **sempre visivel**; *«ele e' dobrado, nunca ausente: o artista SEMPRE ve^ que existe
    // uma curva»*. ⭐ *Uma fileira inerte num estado nao e' o mesmo que uma fileira morta* — e a
    // diferenca entre as duas e' uma decisao de produto que ja' foi tomada, com argumento.
    //
    // ⚠️ **A row da DUREZA ao lado NAO herda esta cerca**, e desde 2026-08-30 ela segue a porta do
    // motor (`rows::shapes_the_distance`): a dureza e' inerte SO' sob campo elastico — no `Mask`
    // ela chega, porque o `shaped_distance` roda ANTES da curva do canal, tal como o
    // `apply_hardness_to_distances` roda antes do `BKE_brush_calc_curve_factors`. *Duas fileiras
    // vizinhas, a mesma aparencia, e regimes de morte diferentes: so' a medicao as separa.*
    //
    // ⚠️ **Quem quiser mexer nisto mexe no GATE primeiro**, e leva um argumento melhor que o do
    // Blender. A saida que nao viola a cerca e' desenha-la **desactivada** com a razao a' vista,
    // que e' desenho novo (e um rotulo i18n novo) e nao existe hoje.
    let selected = Falloff::ALL
        .iter()
        .position(|&f| f == snap.ui.brush.falloff)
        .unwrap_or(0);
    let labels: Vec<&str> = Falloff::ALL.iter().map(|f| f.label()).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.falloff"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_FALLOFF,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // **O PADRÃO**, logo abaixo do falloff — os dois moldam o MESMO peso: o
    // falloff diz como ele cai do centro à borda, o alpha diz onde ele age
    // dentro disso. A primeira opção é NENHUM, e o deslocamento de um é a mesma
    // aritmética que o seletor de matcap usa (o `event` a desfaz com
    // `checked_sub`; as duas metades vivem uma ao lado da outra de propósito).
    let mut labels: Vec<&str> = vec![tr("panel.sculpt3d.alpha.none")];
    labels.extend(Alpha::ALL.iter().map(|a| a.label()));
    // ⚠️ **O SLOT DE IMAGEM tem o nome do SPRITE**, e ele é o ÚLTIMO chip.
    //
    // ⚠️ **Isto REVOGA a decisão que estava escrita aqui.** A versão anterior
    // dizia que uma imagem nunca está na `ALL`, que o `position` devolve `None`
    // para ela e que cair em *nenhum* era honesto — *"o chip aceso não mente
    // sobre um padrão que aquela fileira não oferece"*. A premissa era que a
    // fileira não o oferecia; agora ela oferece, e o que sobrava era um painel
    // dizendo **None** com um padrão vivo e um preview desenhado logo abaixo:
    // o artista lia o painel como quebrado antes de olhar para a miniatura.
    //
    // ⚠️ **O nome vem do RETRATO, não do `Alpha`.** O motor guarda os pixels; de
    // onde eles vieram é proveniência da CENA. Um `label()` que devolvesse o
    // nome do sprite obrigaria o enum a carregar uma `String` que kernel nenhum
    // lê.
    if let Some(name) = snap.alpha_image_name.as_deref() {
        labels.push(name);
    }
    let selected = crate::state::alpha_chip_index(snap);
    let mut y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.alpha"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_ALPHA,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // **O ALPHA POR IMAGEM**, logo abaixo da fileira de nomes — e ele é um
    // BOTÃO, não um décimo chip. A fileira lista NOMES (as nove fórmulas); uma
    // imagem não é um nome, é uma coisa para a qual se aponta. Um chip "Image"
    // teria de existir antes de haver pixels, e é justamente esse estado que o
    // `Alpha::Image` torna inexprimível ao carregar a imagem dentro de si.
    //
    // ⚠️ **Sem sprite selecionado ele NÃO é pintado**, e não é dimming: um botão
    // que só pode falhar é como o artista aprende que ele não funciona. É a
    // mesma decisão do "Bake to Sprite" logo acima, que mostra uma dica no lugar.
    if snap.has_bake_target {
        y = command(
            ctx,
            ids::SCULPT3D_ALPHA_SPRITE,
            tr("panel.sculpt3d.alpha_sprite"),
            x,
            w,
            y,
        );
    }
    // ⚠️ **A pista de escala vem AQUI, colada nos chips que a governam** — e não
    // no bloco de knobs acima, que é onde ela nasceu e onde o smoke a perdeu:
    // lá ela aparecia do nada, separada do seletor pela fileira do Falloff, e o
    // artista lia um número sem saber de que ele era. A row continua na tabela
    // (ver `Row::place`); o que mudou é onde ela é desenhada.
    for row in rows::rows().filter(|r| r.place == rows::Place::AfterAlpha && r.visible(&snap.ui)) {
        y = paint_one_row(ctx, snap, row, x, w, y);
    }
    // **O PREVIEW**, logo ABAIXO das pistas que o mudam — e a posição é a mesma
    // decisão que moveu a pista de escala para cá: um controle e o que ele
    // governa têm de estar no campo de visão um do outro, senão o artista arrasta
    // um número olhando para outro lugar.
    // **O interruptor do preview NO BARRO**, entre as pistas e o quadro — os
    // dois mostram o mesmo padrão e a caixa governa o de FORA, então ela fica
    // onde o olho já está. ⚠️ Só com padrão armado, pela mesma razão da pista de
    // escala: sem padrão ele é um interruptor de coisa nenhuma.
    if snap.ui.brush.alpha.is_some() {
        y = toggle(
            ctx,
            ids::SCULPT3D_ALPHA_PREVIEW,
            tr("panel.sculpt3d.alpha_preview"),
            snap.ui.alpha_preview,
            x,
            w,
            y,
        );
    }
    y = preview::paint(ctx, snap, x, w, y);
    paint_per_verb_switches(ctx, snap, x, w, y)
}

/// **Os interruptores que so' existem para CERTOS verbos** — acumular, so'-as-faces-da-frente, e
/// a lamina a ler a superficie.
///
/// ⚠️ Irmao do [`paint_brush_tail`] por corte de RESPONSABILIDADE, e nao por tamanho: os tres
/// perguntam ao **motor** se a lei existe para o verbo em maos (`verb.accumulates()`,
/// `offers_front_faces()`, `verb == MultiplaneScrape`) em vez de a uma lista de nomes aqui — e
/// e' essa pergunta partilhada que os torna um assunto so'.
///
/// ⚠️ Nasceu em 2026-08-30 porque o `paint_brush_tail` passou o teto de 200 LOC ao ganhar o gate
/// do falloff. **Sexta vez nesta jornada que um teto e' estourado por comentario de medicao** — e,
/// como as cinco anteriores, curado por corte, nunca subindo o teto.
fn paint_per_verb_switches(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    // **ACUMULAR**, e só onde ele faz alguma coisa. ⚠️ A pergunta é feita à
    // PORTA do motor (`Verb::accumulates`) e não a uma lista de nomes aqui: o
    // aplicador pergunta à mesma para honrar o clique, e duas cópias divergiriam
    // num interruptor que aparece e não muda nada — quem tem âncora carrega o
    // gesto TOTAL desde o pen-down, e somar totais não significa nada.
    let y = if snap.ui.brush.verb.accumulates() {
        toggle(
            ctx,
            ids::SCULPT3D_ACCUMULATE,
            tr("panel.sculpt3d.accumulate"),
            snap.ui.brush.accumulate,
            x,
            w,
            y,
        ) + Spacing::Sm.px()
    } else {
        y
    };
    // **SÓ AS FACES DA FRENTE**, e só onde a lei existe. ⚠️ A pergunta é à
    // PORTA do motor (`Brush::offers_front_faces`) e não a uma lista de modos
    // aqui: o roteador pergunta à mesma para honrar o clique, e o kernel lê o
    // flag dentro do `match` sobre a lei. Um modo cuja lei é `Ignored` não tem
    // o que ligar, e a caixa lá seria um interruptor de coisa nenhuma.
    //
    // ⚠️ **Ela pergunta se a LEI existe, nunca se o flag está LIGADO** — o
    // default é desmarcado (é o do Blender, `DNA_brush_types.h:206`), e uma
    // caixa que se escondesse no default seria uma caixa que ninguém marca.
    let y = if snap.ui.brush.offers_front_faces() {
        toggle(
            ctx,
            ids::SCULPT3D_FRONT_FACES,
            tr("panel.sculpt3d.front_faces"),
            snap.ui.brush.front_faces_only,
            x,
            w,
            y,
        ) + Spacing::Sm.px()
    } else {
        y
    };
    // **A LÂMINA LÊ A SUPERFÍCIE**, e só onde há lâmina. ⚠️ A pergunta é ao
    // VERBO, a mesma que o motor faz antes de amostrar os dois lados — uma lista
    // paralela aqui seria um interruptor que aparece noutra ferramenta e não
    // muda um vértice.
    let y = if snap.ui.brush.verb == Verb::MultiplaneScrape {
        toggle(
            ctx,
            ids::SCULPT3D_SCRAPE_DYNAMIC,
            tr("panel.sculpt3d.scrape_dynamic"),
            snap.ui.brush.scrape_dynamic,
            x,
            w,
            y,
        ) + Spacing::Sm.px()
    } else {
        y
    };
    let y = paint_cloth_rows(ctx, snap, x, w, y);
    paint_mask_tools(ctx, snap, x, w, y)
}

/// **AS DUAS FILEIRAS DO PINCEL DE TECIDO** — *Deformation* e *Simulation Area*,
/// na ordem em que o painel da referência as põe (espec §8.4).
///
/// ⚠️ **Elas só existem com o verbo Cloth na mão**, e a pergunta é ao VERBO — a
/// mesma cerca da lâmina do `MultiplaneScrape` acima: uma lista paralela aqui
/// seria uma fileira que aparece noutra ferramenta e não move um vértice.
///
/// ⚠️ **Antes de 2026-09-06 os dois selectores eram VARIÁVEIS DE AMBIENTE.** O
/// motor respondia aos oito modos e às três áreas desde que a lei da referência
/// nasceu, e o artista chegava a UM. *Não era um botão morto: era um motor vivo
/// sem botão nenhum.*
fn paint_cloth_rows(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    if snap.ui.brush.verb != Verb::Cloth {
        return y;
    }
    let selected = ClothMode::ALL
        .iter()
        .position(|&m| m == snap.ui.brush.cloth_mode)
        .unwrap_or(0);
    let labels: Vec<&str> = ClothMode::ALL.iter().map(|m| m.label()).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.cloth_mode"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_CLOTH_MODE,
        &labels,
        selected,
        x,
        w,
        y,
    );
    let selected = ClothArea::ALL
        .iter()
        .position(|&a| a == snap.ui.brush.cloth_area)
        .unwrap_or(0);
    let labels: Vec<&str> = ClothArea::ALL.iter().map(|a| a.label()).collect();
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.cloth_area"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_CLOTH_AREA,
        &labels,
        selected,
        x,
        w,
        y,
    )
}
