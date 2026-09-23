//! **QUE FERRAMENTA ESTÁ NA MÃO, E QUE REFERÊNCIA ELA SEGUE** — irmão do
//! `body.rs`, cortado por ASSUNTO.
//!
//! As duas rows viajam juntas porque respondem à MESMA pergunta em dois níveis:
//! *qual verbo* e *segundo quem*. A `Reference` fica logo abaixo da lista de
//! ferramentas por isso, e não por caber ali.
//!
//! O resto do corpo do painel (pincel, espelho, topologia, sombreamento, cena,
//! entrega) fica no `body.rs`.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_sculpt3d::{
    ClothFilterKind, ClothFilterOrientation, FilterKind, RefMode, kelvinlet::Scales,
};
use ph2d_tokens::Spacing;

use super::widgets::{self, command, header, labelled_seg, toggle};
use crate::state::{Sculpt3dSnapshot, UiLevel};

/// **A FERRAMENTA** — hoje **um botão** cuja face é o pincel na mão.
///
/// # ⛔⛔ A faixa que REFLUÍA saiu daqui, e a nota que a defendia tinha a premissa EXPIRADA
///
/// Este doc dizia: *«um grupo segmentado com muitas opções quebra em linhas, e a alternativa (um
/// dropdown) esconde todas menos uma atrás de um clique»* — e a frase a seguir admitia que a
/// contagem crescera sem ninguém a reconferir (ela foi escrita para **~10** verbos, depois **16**,
/// depois **23**). Hoje são **38**, e é o `CLAUDE.md` §0.0: *quem move o número que tornava algo
/// inalcançável tem de reconferir a nota.*
///
/// **Medido** (`quantas_entradas_tem_cada_painel.rs`, 2026-09-20): o painel mede `2 373 px` num
/// encaixe de `880` e o artista vê **2 das 7** secções; esta secção come `614 px` — `70 %` do que
/// ele vê —, dos quais `276` eram as fichas. Os botões que ele gira enquanto esculpe começavam em
/// `y = 722`, **onde a tela acaba**. ⇒ ordem do dono, 2026-09-20: *«sim»*.
///
/// ⚠️ **E o destino NÃO é um dropdown** — a objecção original continua de pé. É a **paleta**
/// ([`crate::brush_palette`]), que é o contentor desta casa para um catálogo: modal centrado, com
/// rolagem própria e **busca**. ⛔ Um menu de área foi medido e recusado: o `preferred_height` do
/// menu de contexto **soma sem tecto e não rola**, logo `38 × 22 ≈ 836 px` pintariam para fora de
/// um tablet de `1 024` — *o mesmo defeito noutro sítio*.
///
/// ⚠️ **A CONTAGEM continua a não ser citada** — a lista é a fonte. Os números acima são de uma
/// medição datada, que é outra coisa: eles descrevem o dia em que a decisão foi tomada.
pub(super) fn paint_tool(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (fold, mut y) = header(
        ctx,
        crate::ids::SCULPT3D_SEC_TOOL,
        tr("panel.sculpt3d.section.tool"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    // ⭐⭐⭐ **UM BOTÃO, e a face dele é o pincel na mão** — as 38 fichas foram para a paleta
    //    ([`crate::brush_palette`]), por ordem do dono de 2026-09-20. Ver o cabeçalho desta função
    //    para a medição e para a nota que isto reconfere.
    //
    // ⚠️ **O rótulo é o do verbo e mais nada** (HR-15): compor `"{} …"` aqui seria texto pintado
    //    por este ficheiro, e o que o botão significa — *«é este o pincel; clique para trocar»* —
    //    é o que a posição dele no cabeçalho da secção `Tool` já diz.
    y = command(
        ctx,
        crate::ids::SCULPT3D_OPEN_BRUSHES,
        tr(snap.ui.brush.verb.label_key()),
        x,
        w,
        y,
    );
    y = paint_reference_row(ctx, snap, x, w, y);
    y = paint_filter_row(ctx, snap, x, w, y);
    widgets::end_fold(ctx, fold, y + Spacing::Md.px())
}

/// **O FILTRO** — uma LEI na malha INTEIRA, com o arrasto a dar a força.
///
/// ⚠️ **A PREMISSA DESTA ROW MUDOU com a W9b, e a mudança é visível.** Ela era
/// oferecida só a `Verb::filters_mesh`, porque a lei era DERIVADA do verbo em
/// mãos; três das sete leis não têm verbo nenhum (não existe pincel de Scale,
/// de Sphere nem de Random), então esse critério as tornava inalcançáveis por
/// qualquer gesto. O filtro passa a ser oferecido **sempre**, e o verbo em mãos
/// deixa de decidir — ele apenas SEMEIA a escolha ao armar.
///
/// ⚠️ **E *visível ⇔ vivo* continua de pé por outra via:** antes o arm morria
/// com o verbo, e a razão escrita era que um arm aceso e invisível pararia o
/// botão esquerdo sem nada na tela dizer por quê. Com a row sempre pintada, um
/// arm aceso é sempre **visível** — a preocupação some, e o mecanismo que a
/// resolvia sai com ela.
///
/// ⚠️ **BASIC, e não Pro:** o filtro não é afinação do verbo — ele é uma lei
/// aplicada de uma vez, e o precedente é o *Filter Layer* do Painter, que vive
/// no card do próprio Sculpt.
fn paint_filter_row(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let y = widgets::toggle(
        ctx,
        crate::ids::SCULPT3D_FILTER,
        tr("panel.sculpt3d.filter"),
        snap.filter_armed,
        x,
        w,
        y,
    );
    if !snap.filter_armed {
        // ⚠️ **O selector só existe ARMADO, e isto NÃO é esconder um
        // controle** — é o inverso: desarmado, escolher a lei não muda coisa
        // nenhuma, e sete chips que não produzem nada são a definição do botão
        // que o artista descobre vazio clicando. Armado, cada um muda o que o
        // próximo arrasto vai fazer.
        return y;
    }
    // ⚠️ **UMA convenção: o id é a POSIÇÃO no `FilterKind::ALL`**, e nunca o
    // discriminante. O roteador (`event.rs`) devolve `ALL[i]` a partir do
    // índice do id, então indexar aqui por `*k as usize` seria uma segunda
    // convenção que **coincide com a primeira só enquanto o `ALL` estiver em
    // ordem de discriminante** — reordená-lo faria um chip rotulado `Sphere`
    // escrever `Relax`, com a fileira pintada, viva sob o mouse e mentindo.
    let (kind_ids, labels): (Vec<_>, Vec<&str>) = FilterKind::ALL
        .iter()
        .enumerate()
        .map(|(i, k)| (crate::ids::SCULPT3D_FILTER_KIND[i], tr(k.label_key())))
        .unzip();
    // ⚠️ **`None` quando a lei escolhida é de TECIDO** — e é assim que as duas
    // fileiras dizem a verdade ao mesmo tempo: só uma tem chip aceso. ⛔ Cair no
    // `0` faria a fileira de malha mostrar `Smooth` aceso com o filtro a correr
    // gravidade, que é um controlo a mentir sobre o que o arrasto vai fazer.
    // ⭐ **`usize::MAX` = nenhum chip aceso**, e a propriedade é do widget, com o
    // doc dele a declará-la: *«out-of-range clamps to no selection»*. ⛔ Uma
    // função nova ao lado seria a segunda resposta a uma pergunta já respondida.
    let selected = snap
        .ui
        .filter_law
        .mesh()
        .and_then(|k| FilterKind::ALL.iter().position(|&x| x == k))
        .unwrap_or(usize::MAX);
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.filter_kind"),
        &kind_ids,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // ⭐⭐⭐ **A SEGUNDA FAMÍLIA** (espec §7) — o mesmo gesto, o mesmo undo, e uma
    // lei que ACUMULA em vez de refazer um passo.
    //
    // ⚠️ **Fileira própria, e não catorze chips numa só:** *Inflate* e *Scale*
    // existem nos dois lados e são leis diferentes — *um chip cujo rótulo não
    // distingue a lei precisa da fileira para o fazer.*
    let (cloth_ids, cloth_labels): (Vec<_>, Vec<&str>) = ClothFilterKind::ALL
        .iter()
        .enumerate()
        .map(|(i, k)| (crate::ids::SCULPT3D_CLOTH_FILTER_KIND[i], tr(k.label_key())))
        .unzip();
    let cloth_sel = snap
        .ui
        .filter_law
        .cloth()
        .and_then(|k| ClothFilterKind::ALL.iter().position(|&x| x == k))
        .unwrap_or(usize::MAX);
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.cloth_filter_kind"),
        &cloth_ids,
        &cloth_labels,
        cloth_sel,
        x,
        w,
        y,
    );
    // ⚠️ **O REFERENCIAL só existe com uma lei de TECIDO escolhida** — é a mesma
    // lei do selector acima do toggle: com uma lei de malha em mãos, escolher a
    // orientação não muda um vértice, e dois chips que não produzem nada são a
    // definição do botão que o artista descobre vazio clicando.
    if !snap.ui.filter_law.is_cloth() {
        return y;
    }
    // ⚠️ **DOIS chips e não os três da espec** — o `World` está fora por MEDIÇÃO
    // (a pose de uma escultura não tem rotação, logo ele daria os eixos do
    // `Local`). A fileira é a lista OFERECIDA, e o índice é a posição nela.
    let oferecidos = ClothFilterOrientation::offered();
    let (orient_ids, orient_labels): (Vec<_>, Vec<&str>) = oferecidos
        .iter()
        .enumerate()
        .map(|(i, o)| {
            (
                crate::ids::SCULPT3D_CLOTH_FILTER_ORIENT[i],
                tr(o.label_key()),
            )
        })
        .unzip();
    let orient_sel = oferecidos
        .iter()
        .position(|&o| o == snap.ui.cloth_filter_orientation)
        .unwrap_or(usize::MAX);
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.cloth_filter_orient"),
        &orient_ids,
        &orient_labels,
        orient_sel,
        x,
        w,
        y,
    )
}

/// **A REFERÊNCIA que este verbo segue** — a row `S` · `B` · `L`.
///
/// ⚠️ **Ela pinta só os modos OFERECIDOS** (`RefMode::offered_for`), e o id de
/// cada chip vem da posição no `RefMode::ALL` — nunca da posição na fileira. É a
/// lei anti-chip-morto do §3 do plano: hoje o `L` não declara nada de seu, e um
/// chip que produz o que o vizinho produz é um botão que o artista descobre
/// vazio clicando.
///
/// ⚠️ **E ela fica no BASIC**, com o seletor de ferramenta: o doc 20 mede que a
/// escolha muda o pincel em `1,08× a 1,44×` ao longo do raio e a lei do kernel em
/// `1,7e-3` — é o achado mais consequente do estudo, e escondê-lo atrás de um
/// interruptor Pro seria esconder a decisão que mais importa.
fn paint_reference_row(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let verb = snap.ui.brush.verb;
    let offered: Vec<RefMode> = RefMode::offered_for(verb).collect();
    if offered.len() < 2 {
        // Um modo só não é uma escolha; pintar a fileira seria um rádio de um
        // botão. Não acontece hoje e o gate cobra os dois.
        return y;
    }
    let mode_ids: Vec<_> = offered
        .iter()
        .map(|m| crate::ids::SCULPT3D_REF_MODE[*m as usize])
        .collect();
    let labels: Vec<&str> = offered.iter().map(|m| tr(m.label_key())).collect();
    let selected = offered
        .iter()
        .position(|&m| m == snap.ui.brush.mode)
        .unwrap_or(0);
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.reference"),
        &mode_ids,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // ⭐⭐ **AS COLISÕES DO FILTRO** — o pano bate nas outras peças da cena.
    //
    // ⚠️ **Nasce desligada**, e o preço é a razão: `2,6×` a `6,1×` o custo de um
    // dab, e aqui a peça INTEIRA é o pior caso — não há banda a limitar quem
    // colide.
    let y = toggle(
        ctx,
        crate::ids::SCULPT3D_CFILTER_COLLISIONS,
        tr("panel.sculpt3d.cfilter_collisions"),
        snap.ui.cloth_filter.collisions,
        x,
        w,
        y,
    );
    // ⭐⭐ **O *Force Axis*** (espec §7) — e ele aparece **só na Escala**, que é o
    // único tipo que o lê.
    //
    // ⚠️ **Quem responde é o MOTOR** (`ClothFilterKind::le_os_eixos`), nunca uma
    // lista de nomes aqui: pintá-lo nos outros quatro tipos seria o knob morto
    // que esta casa varre a cada wave — o artista clica, nada muda, e conclui
    // que o app tem um defeito que não tem.
    //
    // ⛔ **Ele era um controlo que NÃO EXISTIA**, e a distinção com um knob
    // morto é a cura: a de um morto é ligar o braço, a de um ausente é criá-lo.
    // O motor já o honrava desde 07/09 (gate `escala_eixox`).
    let y = if snap
        .ui
        .filter_law
        .cloth()
        .is_some_and(ph2d_sculpt3d::ClothFilterKind::le_os_eixos)
    {
        let mut yy = y;
        for (i, eixo) in ["X", "Y", "Z"].iter().enumerate() {
            yy = toggle(
                ctx,
                crate::ids::SCULPT3D_CFILTER_AXIS[i],
                // ⚠️ O rótulo é composto porque a chave nomeia a FAMÍLIA e o eixo
                // é o índice — três chaves de i18n para `X`/`Y`/`Z` seriam três
                // traduções da mesma letra.
                &format!("{} {eixo}", tr("panel.sculpt3d.cfilter_axis")),
                snap.ui.cloth_filter_axes[i],
                x,
                w,
                yy,
            );
        }
        yy
    } else {
        y
    };

    let y = command(
        ctx,
        crate::ids::SCULPT3D_REF_MODE_ALL,
        tr("panel.sculpt3d.reference_all"),
        x,
        w,
        y,
    );
    paint_elastic_scales_row(ctx, snap, x, w, y)
}

/// **QUÃO LARGO é o campo elástico** — a família de escalas do kernel.
///
/// ⚠️ **A row é oferecida pela MESMA porta que o motor pergunta**
/// (`RefMode::field(verb)`), e não por uma lista de verbos ao lado: o
/// `stroke_target` consome o kernel exatamente onde essa porta devolve `Some`,
/// então uma fileira desenhada em qualquer outro lugar seriam três chips que não
/// movem um vértice. Duas cópias da mesma pergunta é como a próxima nasce
/// desalinhada.
///
/// ⚠️ **PRO, pela regra do [`UiLevel`]:** o valor foi ARMADO — a `Tight` é a
/// família que a medição do resíduo de borda escolheu —, então em Basic o
/// artista está com a largura que o kernel escolheu por ele, não com um vazio.
fn paint_elastic_scales_row(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if !snap.ui.ui_level.shows(UiLevel::Pro) {
        return y;
    }
    let verb = snap.ui.brush.verb;
    if snap.ui.brush.mode.field(verb).is_none() {
        return y;
    }
    let labels: Vec<&str> = Scales::ALL.iter().map(|s| tr(s.label_key())).collect();
    let selected = Scales::ALL
        .iter()
        .position(|&s| s == snap.ui.brush.elastic_scales)
        .unwrap_or(0);
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.elastic_scales"),
        &crate::ids::SCULPT3D_ELASTIC_SCALES,
        &labels,
        selected,
        x,
        w,
        y,
    )
}
