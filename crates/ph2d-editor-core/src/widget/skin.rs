//! **A PELE POR-WIDGET** — o degrau 2 do §2 do plano UI/UX: uma forma autorada no canvas é
//! pintada pelo **pintor REAL** do catálogo, no lugar dela.
//!
//! # A pergunta desta wave, e a única resposta que não bifurca
//!
//! O §2 do plano recusou as duas respostas fáceis (*o desenho VIRA o widget* — reimplementar
//! drag/foco/teclado no canvas; *o widget VIRA desenho* — trocar 44 widgets testados por um
//! interpretador) e escolheu a terceira: **o desenho é a PELE, o widget é o COMPORTAMENTO, o
//! token é a PONTE**. Este módulo é a ponte.
//!
//! ⚠️ **O canvas chama o PINTOR REAL, nunca uma cópia.** Uma prévia que redesenhasse o botão à mão
//! seria uma segunda resposta a *"que aparência tem este widget?"*, e a divergência entre as duas
//! só apareceria numa screenshot — o modo de falha mais caro que este repo conhece, porque
//! ninguém lê número em screenshot. Daí a forma deste arquivo: uma função, um `match`, e cada
//! braço termina no `paint_*` que o painel nativo já chama.
//!
//! # A APARÊNCIA vem dos TOKENS, não do desenho — e isso é decisão, não omissão
//!
//! O plano falava em *"cantos, sombra, gradiente, cores por estado"* saindo do desenho. Medido: os
//! painéis **não aceitam** nada disso — a assinatura deles é `(dados, rect, scene, text, theme)` e
//! toda cor/raio sai de `ph2d_tokens`. Dar-lhes um canal de estilo por-widget seria 44 assinaturas
//! novas **e** uma segunda porta para a aparência, no dia seguinte ao da W6.1 ter feito a tabela de
//! cor autorável.
//!
//! ⇒ **O desenho responde ONDE e O QUÊ** (o retângulo e o tipo); **os tokens respondem COMO**.
//!
//! ⚠️ **E o dia que esta nota previa CHEGOU** (2026-08-09). Ela dizia que um mapeamento por-tipo
//! — *"o preenchimento da forma vira a cor da swatch"* — era *"uma tabela de 44 casos especiais,
//! deliberadamente NÃO construída"*, e que ele nasceria *"no dia em que um tipo precisar de um
//! parâmetro que o token não exprime"*. Medido: são **2 de 16**, não 44, e eles são [`SkinParam`].
//! Todo o resto continua respondido pelo retângulo, pelo rótulo, pelos tokens e pelo estado vivo.
//!
//! # O RÓTULO é o `Name` da entidade
//!
//! Não há campo de rótulo no componente. O nome que o artista já digita na Hierarquia **é** o
//! rótulo — a mesma lei que a W5c usou para os eixos de variant, e pelo mesmo motivo: um segundo
//! lugar para digitar o nome de uma coisa é um segundo lugar que discorda do primeiro.
//!
//! # O ESTADO é `Normal`, e a fronteira tem dono
//!
//! *idle / hover / press / disabled* é a **W7** (a máquina de estados + Smart Animate), que o plano
//! escalona depois desta. Um widget que respondesse ao mouse aqui seria comportamento no canvas —
//! exactamente o que o §2 recusou.

use crate::interaction::InteractiveState;
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::widget::{
    Button, ButtonState, Card, Checkbox, ColorSwatch, Divider, IconButtonStyle, IconGlyph,
    LevelMeter, ListItem, NumberInput, ProgressBar, SectionHeader, Slider, Spinner, Tag, TextInput,
    Toggle, paint_button, paint_card, paint_checkbox, paint_color_swatch, paint_divider,
    paint_icon_button, paint_level_meter, paint_list_item, paint_number_input_with_buffer,
    paint_progress_bar, paint_section_header, paint_slider, paint_spinner, paint_tag,
    paint_text_input, paint_toggle,
};
use crate::widget::{
    Dropdown, DropdownOption, RadioGroup, RadioOption, RadioOrientation, SegmentedAdaptive,
    SegmentedOption, TabItem, Tabs, paint_dropdown_chip, paint_radio_group_with_labels, paint_tabs,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;

use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme};
use ph2d_vector::{BezPath, VectorScene};

/// O valor que um controle contínuo mostra na prévia.
///
/// ⚠️ Um slider em `0` ou em `1` é indistinguível de uma trilha quebrada — a barra preenchida
/// desaparece ou come a trilha inteira, e o artista não consegue julgar nem uma nem outra. O meio
/// é o único ponto em que as **duas** metades do controle são visíveis ao mesmo tempo. Ele não é
/// um limite (não há recurso do qual seja): é o que a prévia mostra até a W7 ligar o valor a uma
/// fonte.
const PREVIEW_VALUE: f32 = 0.5;

/// O pico que a prévia do medidor segura, acima do RMS.
///
/// ⚠️ **A razão do [`PREVIEW_VALUE`] vale aqui com força maior:** um medidor em `0` é uma coluna
/// escura indistinguível de uma coluna quebrada, e um medidor em `1` come a escala inteira. E ele
/// tem uma terceira parte que os outros controles não têm — o **marcador de pico** —, que só é
/// visível se estiver ACIMA do preenchimento. Um pico igual ao RMS desenharia a marca dentro da
/// barra, onde ela não se lê.
///
/// Não é um limite (não há recurso do qual seja): é o que a prévia mostra até uma fonte de áudio
/// real alimentar o widget.
const PREVIEW_PEAK: f32 = 0.72; // LITERAL-PX-OK: o pico da PRÉVIA, acima do RMS para a marca se ver

/// O id que os pintores recebem na prévia.
///
/// ⚠️ **Nenhum pintor do catálogo lê o próprio `id` ao pintar** (conferido: eles recebem o estado
/// por argumento) — o id existe para a a11y e para o `WidgetStore`, e uma pele de canvas não tem
/// nem uma nem outro. Um id REAL aqui seria pior que inútil: ele colidiria com o widget homônimo
/// do painel nativo no store que a shell possui.
const PREVIEW_ID: NodeId = NodeId(0);

mod param;
pub use param::SkinParam;
use param::{mark_selected_tab, marked_option};

/// **A PELE PREENCHE A MOLDURA — uma frase, doze tipos** (BUGS_vector #26).
///
/// Dez dos doze já faziam isso. Os outros dois desenhavam o widget **centrado dentro** dela — a
/// caixa do checkbox a 64,3% da altura, a trilha do slider a 25% —, e o resto era o *padding* de
/// uma LINHA de painel. Numa linha isso está certo: ali toda linha tem a mesma altura, todo
/// checkbox tem o mesmo tamanho, e é isso que faz um formulário ler como formulário.
///
/// ⚠️ **No canvas não há linha, e é isso que o torna um defeito visível:** o gizmo abraça o
/// RETÂNGULO, então uma pele que ocupa 25% dele deixa uma folga vertical que o artista vê a cada
/// gesto — e que faz o **snap encaixar no lugar errado**, porque o que encaixa é a moldura e o
/// que se vê é a tinta. Enio, 2026-08-05: *"por que o gizmo não se ajusta perfeitamente na
/// vertical … seria interessante se ajustar para fins de snap perfeito e também manter o padrão
/// dos outros widgets"*.
///
/// ⚠️ **A primeira correção parou a meio caminho** e a foto do report media exactamente a lei
/// dela: 64,3% e 25,0% previstos, `~67%` e `~24%` medidos na tela. Ela curou *o widget não cresce
/// com a moldura*; esta cura *o widget não é a moldura*. O padding de linha não é do canvas.
///
/// ⛔ **O que isto continua NÃO sendo:** não mexe em `CHECKBOX_BOX_PX` nem no teto do painel. Eles
/// governam TODOS os painéis do app, e movê-los para agradar ao canvas re-dimensionaria a
/// interface inteira — mover o número do consumidor errado. `None` continua a ser a lei de todo
/// painel, **ao bit**, e a mutação que a remove derruba três gates.
///
/// ⚠️ **O token não deixa de significar: ele passa a ser o TAMANHO NATURAL do objeto.** Uma
/// moldura da altura do token pinta exactamente a tinta que o app pinta numa linha — o que muda é
/// que a moldura passou a medir o *widget* em vez da *linha que o hospeda*.
fn skin_checkbox_box_px(rect: Rect) -> f32 {
    // A caixa é QUADRADA, então preencher a altura só é possível enquanto ela couber na largura.
    // ⚠️ O `min` da largura mora AQUI e não no pintor: o pintor limita pela altura (a lei que todo
    // painel usa e que uma linha estreita nunca exercita), e alargá-la mudaria a rota do painel
    // para agradar ao canvas — exactamente o que a nota acima recusa.
    rect.h.min(rect.w)
}

/// A espessura da trilha do slider numa moldura de canvas: a moldura inteira. Irmã de
/// [`skin_checkbox_box_px`] e a mesma frase — o que sobrava era o padding da linha.
fn skin_slider_track_px(rect: Rect) -> f32 {
    rect.h
}

mod geometry;
pub use geometry::inline_option_rect;
mod kind;
pub use kind::WidgetKind;

/// **A PORTA ÚNICA**: pinta a pele de `kind` em `rect`, pelo pintor real do catálogo.
///
/// `label` é o `Name` da entidade — o rótulo que o widget mostra.
///
/// `param` é o que o retângulo, o rótulo e os tokens **não conseguem** responder — a cor de uma
/// [`WidgetKind::ColorSwatch`] e o glifo de um [`WidgetKind::IconButton`] (ver [`SkinParam`]).
/// Todo outro tipo passa o neutro.
///
/// ⚠️ Quem chama isto tem de ser quem chamaria o painel nativo: mesma `VectorScene`, mesmo
/// `TextSystem`, mesmo `Theme`. É essa igualdade de argumentos que torna o gate de bytes possível
/// — a prévia e o painel não podem divergir se percorrem a MESMA função com a MESMA entrada.
pub fn paint_widget_skin(
    kind: WidgetKind,
    label: &str,
    param: SkinParam,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_widget_skin_with(
        kind,
        label,
        PREVIEW_ID,
        None,
        param,
        rect,
        scene,
        text_system,
        theme,
    );
}

/// **A MESMA porta, com o que o widget É agora** — a pele viva (plano UI/UX W8b.2).
///
/// `live` é a fatia do [`WidgetStore`](crate::interaction::WidgetStore) daquele id **e o `t` do
/// hover dele**: o valor de um slider, o on/off de um toggle, o texto de um campo, o *hot/active*
/// que o ponteiro escreve — e quanto da transição já passou.
///
/// ⚠️ **O par vem INTEIRO, e é isso que o torna impossível de descasar.** A versão anterior tomava
/// só o `&InteractiveState`, então cada arm copiava o estado DISCRETO e deixava o `hover_t` no
/// default `1.0` — o valor que faz o `Button::bg_color` cair no token duro. O painel gerado
/// **reagia e SALTAVA** enquanto os vinte e três painéis escritos à mão amaciavam: metade do app
/// numa lei, metade noutra, que é precisamente o sintoma que a F2 existe para curar. Quem
/// constrói o par é a porta [`WidgetStore::skin_live`](crate::interaction::WidgetStore::skin_live),
/// que recebe o id **uma vez**.
///
/// ⚠️ **`Some((estado, SETTLED))` é BYTE-IDÊNTICO ao mundo pré-par** — era esse o default que os
/// arms herdavam —, e é por isso que o gate da prévia continua a ser o mesmo teste.
///
/// ⚠️ **Um painel gerado e a prévia do canvas percorrem ESTA função, e é isso que os impede de
/// divergir.** Um segundo `match` sobre os doze tipos seria a segunda resposta a *"que aparência
/// tem um Slider?"*, e a divergência entre as duas só apareceria numa screenshot — o modo de falha
/// mais caro que este repo conhece. A prévia é literalmente **esta função sem estado**: `None` cai
/// nos valores de prévia, que é o que uma pele de canvas tem para mostrar.
///
/// ⚠️ **Estado que não casa com o tipo cai no valor de prévia, e nunca entra em pânico.** É
/// alcançável (um `populate` que declare o tipo errado), e o produto certo é desenhar o widget com
/// a aparência neutra — uma tela que explode por causa de um registro trocado troca um controle
/// errado por nenhum app.
#[allow(clippy::too_many_arguments)]
pub fn paint_widget_skin_with(
    kind: WidgetKind,
    label: &str,
    id: NodeId,
    live: Option<(&InteractiveState, f32)>,
    param: SkinParam,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let _ = id;
    match kind {
        WidgetKind::Button => {
            let mut b = Button::new(id, label);
            if let Some((InteractiveState::Button { state }, t)) = live {
                b = b.visual((*state, t));
            }
            paint_button(&b, rect, scene, text_system, theme);
        }
        WidgetKind::Toggle => {
            let mut t = Toggle::new(id, label);
            if let Some((InteractiveState::Toggle { state, on }, hover_t)) = live {
                t = t.visual((*state, hover_t));
                t.on = *on;
            }
            paint_toggle(&t, rect, scene, theme);
        }
        WidgetKind::Checkbox => {
            let mut c = Checkbox::new(id, label);
            c.box_px = Some(skin_checkbox_box_px(rect));
            if let Some((InteractiveState::Checkbox { state, value }, t)) = live {
                c = c.visual((*state, t));
                c.value = *value;
            }
            paint_checkbox(&c, rect, scene, text_system, theme);
        }
        WidgetKind::Slider => {
            let mut s = Slider::new(id, label);
            s.value = PREVIEW_VALUE;
            s.track_px = Some(skin_slider_track_px(rect));
            if let Some((InteractiveState::Slider { state, value, .. }, t)) = live {
                s = s.visual((*state, t));
                // ⚠️ Pela porta do widget, nunca pelo campo: `set_value` é quem limita a `0..=1`,
                // e escrever o campo cru pintaria uma barra fora da trilha.
                s.set_value(*value);
            }
            paint_slider(&s, rect, scene, theme);
        }
        WidgetKind::ProgressBar => {
            let mut b = ProgressBar::new(id, label);
            // ⚠️ `Indeterminate` (o default) pinta uma lasca fixa cuja POSIÇÃO a shell anima — na
            // prévia estática ela leria como uma barra quebrada. A prévia é determinada.
            b.mode = crate::widget::ProgressMode::Determinate(PREVIEW_VALUE);
            paint_progress_bar(&b, rect, scene, text_system, theme);
        }
        WidgetKind::Tag => {
            let mut t = Tag::new(id, label);
            // ⚠️ **O `Tag` não tem `hover_t`, então não há o que vestir** — o fade dele é wave
            // própria (acrescentar o campo), e continua o adiamento que o `tag.rs` já nomeia.
            if let Some((InteractiveState::Tag { state }, _)) = live {
                t.state = *state;
            }
            paint_tag(&t, rect, scene, text_system, theme);
        }
        WidgetKind::TextInput => {
            let mut i = TextInput::new(id, label);
            i.value = label.to_string();
            if let Some((
                InteractiveState::TextInput {
                    state, text, caret, ..
                },
                t,
            )) = live
            {
                i = i.visual((*state, t));
                i.value = text.clone();
                i.caret_byte = *caret;
            }
            paint_text_input(&i, rect, scene, text_system, theme);
        }
        WidgetKind::Card => {
            let mut c = Card::new(id);
            c.title = Some(label.to_string());
            paint_card(&c, rect, scene, text_system, theme);
        }
        WidgetKind::SectionHeader => paint_section_header(
            &SectionHeader::new(id, label),
            rect,
            scene,
            text_system,
            theme,
        ),
        WidgetKind::ListItem => {
            let mut l = ListItem::new(id, label);
            // ⛔ **O `ListItem` NÃO amacia, e a cerca é honrada por CONSTRUÇÃO** (ele não tem
            // `hover_t`): oito rows a descer em 200 ms com um assentamento de 0,22 s ficariam
            // meio-acesas ao mesmo tempo — *o realce de menu obedece ao cursor*. Estudo §6.5.
            if let Some((InteractiveState::ListItem { state, selected }, _)) = live {
                l.state = *state;
                l.selected = *selected;
            }
            paint_list_item(&l, rect, scene, text_system, theme);
        }
        WidgetKind::Spinner => paint_spinner(&Spinner::new(id, label), rect, scene, theme),
        WidgetKind::Divider => paint_divider(&Divider::new(id), rect, scene, theme),
        WidgetKind::NumberInput => {
            // ⚠️ `step`/`min`/`max` ficam nos defaults do construtor de propósito: eles governam o
            // que um GESTO produz, e o gesto é do painel — a pele responde *que aparência tem um
            // campo numérico*, e nenhum dos três a muda. Autorá-los seria um parâmetro por-tipo
            // sem consumidor (ver o doc do `ALL`).
            let mut n = NumberInput::new(id, label, f64::from(PREVIEW_VALUE));
            // O buffer do estado vivo é o que o artista está DIGITANDO; sem estado, o pintor
            // formata o `value` sozinho — é o que o `None` significa aqui.
            let mut buf: Option<&str> = None;
            let (mut caret, mut anchor) = (0usize, None);
            if let Some((
                InteractiveState::NumberInput {
                    state,
                    value,
                    buffer,
                    caret: c,
                    selection_anchor,
                    ..
                },
                t,
            )) = live
            {
                n = n.visual((*state, t));
                n.value = *value;
                buf = Some(buffer.as_str());
                caret = *c;
                anchor = *selection_anchor;
            }
            paint_number_input_with_buffer(&n, buf, caret, anchor, rect, scene, text_system, theme);
        }
        WidgetKind::ColorSwatch => {
            // ⚠️ **Sem parâmetro, a swatch mostra o XADREZ** — e isso não é um fallback de
            // conveniência: `[0,0,0,0]` é alfa zero, o pintor desenha o tabuleiro de
            // transparência, e *"nenhuma cor escolhida"* é exactamente o que ele significa em
            // toda UI de cor que existe. Inventar um cinza aqui seria pintar uma escolha que o
            // artista não fez.
            let rgba = param.rgba.unwrap_or([0, 0, 0, 0]);
            paint_color_swatch(&ColorSwatch::new(id, label, rgba), rect, scene, theme);
        }
        // **A FAMÍLIA DE LISTA** — N opções, uma marcada, e os rótulos vêm dos FILHOS que o
        // artista desenhou (ver `WidgetKind::takes_options`). Os três terminam no pintor real do
        // catálogo, como todos os irmãos; o que muda entre eles é só o desenho.
        //
        // ⚠️ **Os ids das opções são o `PREVIEW_ID`**, e é a mesma razão dos outros braços: os
        // pintores não leem o próprio id ao pintar, e um id real aqui colidiria com o widget
        // homónimo do painel nativo no store que a shell possui. Quem precisa de id por opção é o
        // painel COMPILADO, que os deriva da chave — não a pele do canvas.
        // ⚠️ **Um controle de lista SEM opções ainda tem de aparecer.** Com zero itens o pintor de
        // abas não desenha nada — e um controle invisível é pior que um errado: o artista não
        // consegue nem selecioná-lo para lhe dar os filhos que o tornariam visível. A moldura
        // vazia é a afordância honesta de *este controle existe e ainda não tem opções*.
        WidgetKind::Tabs | WidgetKind::RadioGroup | WidgetKind::SegmentedAdaptive
            if param.options.is_empty() =>
        {
            fill_rounded_rect(
                scene,
                rect,
                Radius::Md.px(),
                resolve(ColorToken::Bg2, theme),
            );
            stroke_rounded_rect(
                scene,
                rect,
                Radius::Md.px(),
                StrokeToken::Default.px(),
                resolve(ColorToken::Border, theme),
            );
        }
        WidgetKind::Tabs => {
            let items: Vec<TabItem> = param
                .options
                .iter()
                .map(|o| TabItem::new(PREVIEW_ID, o.clone()))
                .collect();
            let mut t = Tabs::new(id, label, items);
            mark_selected_tab(&mut t, &param);
            paint_tabs(&t, rect, scene, text_system, theme);
        }
        WidgetKind::RadioGroup => {
            let opts: Vec<RadioOption<usize>> = param
                .options
                .iter()
                .enumerate()
                .map(|(i, o)| RadioOption::new(PREVIEW_ID, i, o.clone()))
                .collect();
            let mut g = RadioGroup::new(id, label, opts).orientation(RadioOrientation::Horizontal);
            if let Some(i) = marked_option(&param) {
                g = g.selected(i);
            }
            paint_radio_group_with_labels(&g, rect, scene, text_system, theme);
        }
        WidgetKind::SegmentedAdaptive => {
            let opts: Vec<SegmentedOption> = param
                .options
                .iter()
                .map(|o| SegmentedOption::new(PREVIEW_ID, o.clone()))
                .collect();
            let seg = SegmentedAdaptive::new(id, label, opts);
            // ⚠️ O pintor adaptativo pede `store` e `hit_index` porque ele REGISTA os segmentos.
            // A pele do canvas não tem nem um nem outro (e não deve ter: comportamento no canvas
            // é o que o §2 do plano recusou), então aqui ele é desenhado pelo irmão de abas na
            // variante segmentada — o MESMO desenho, sem a metade que regista.
            let items: Vec<TabItem> = seg
                .options
                .iter()
                .map(|o| TabItem::new(PREVIEW_ID, o.label.clone()))
                .collect();
            let mut t = Tabs::new(id, label, items).variant(crate::widget::TabsVariant::Segmented);
            mark_selected_tab(&mut t, &param);
            paint_tabs(&t, rect, scene, text_system, theme);
        }
        // ⚠️ **O chip, e NUNCA a lista** — ver [`WidgetKind::defers_a_popover`]. Chamar o
        // `paint_dropdown` de conveniência aqui desenharia a lista aberta dentro do recorte do
        // corpo do painel (cortada) e por baixo das rows seguintes (que são pintadas depois). A
        // segunda superfície é do passe diferido de quem tem `hit_index`.
        //
        // ⚠️ **E ele NÃO precisa do guard de lista vazia** que os três irmãos acima têm, porque o
        // chip não itera opções: ele desenha a moldura, o rótulo e o galo. Um dropdown sem opções
        // desenha-se sozinho como *"nada escolhido"* — que é o `placeholder`, e é honesto.
        WidgetKind::Dropdown => {
            let opts: Vec<DropdownOption<usize>> = param
                .options
                .iter()
                .enumerate()
                .map(|(i, o)| DropdownOption::new(PREVIEW_ID, i, o.clone()))
                .collect();
            let mut dd = Dropdown::new(id, label, opts);
            if let Some(i) = marked_option(&param) {
                dd.select(i);
            }
            paint_dropdown_chip(&dd, rect, scene, text_system, theme);
        }
        WidgetKind::IconButton => {
            // ⚠️ **`Compact`, e não `Chip`** — o `Chip` é a PÍLULA da TopBar (`Radius::Xl`, *"hero
            // cards, splash"*), e o próprio doc do estilo nomeia o defeito: numa caixa densa ele
            // arredonda numa lozenga *"não importa em que preset de raio o tema esteja"*. Uma row
            // de painel é densa por definição.
            //
            // ⚠️ **Sem glifo, o botão desenha a MOLDURA e mais nada** — o irmão exacto do xadrez
            // da swatch. Um `IconId` de reserva seria uma escolha que o artista não fez, e ele a
            // leria como sua; um retângulo vazio é *nenhum ícone escolhido*, que é o que é.
            let empty = BezPath::new();
            let glyph = param.icon.unwrap_or(IconGlyph::Path(&empty));
            // ⚠️ **A premissa que aqui estava era VERDADE quando foi escrita, e esta wave
            //    tornou-a falsa.** Ela dizia *"um botão vestido não tem track no relógio da tela —
            //    o neutro é o que o mantém byte a byte"*, e o neutro era a única resposta possível
            //    enquanto o `t` não chegava a esta função. Ele chega: o par entra inteiro, e a
            //    prévia (`None`) continua a cair no neutro pelo `unwrap_or`.
            let visual = match live {
                Some((InteractiveState::Button { state }, t)) => (*state, t),
                _ => (ButtonState::Normal, crate::motion::SETTLED),
            };
            paint_icon_button(rect, glyph, IconButtonStyle::Compact, visual, scene, theme);
        }
        WidgetKind::LevelMeter => {
            // ⚠️ Um medidor é READOUT: ele não tem estado de interação (não há
            // `InteractiveState::LevelMeter`, e a `Row::is_control` o deixa de fora), então a
            // prévia é tudo o que ele mostra até uma fonte de áudio o alimentar. É o molde exato
            // do `ProgressBar` acima.
            let mut m = LevelMeter::new(id, label);
            m.rms = [PREVIEW_VALUE; 2];
            m.peak_hold = [PREVIEW_PEAK; 2];
            paint_level_meter(&m, rect, scene, theme);
        }
    }
}

#[cfg(test)]
// ⚠️ Em `skin/tests.rs`, e NÃO num `skin_tests.rs` solto: o `ph2d-widget-sync` varre os `*.rs`
// do topo desta pasta para gerar o bloco de `mod`, então um irmão solto vira um "WIDGET" — sem
// showcase, sem a11y e sem opt-out, com o gate de staleness a sangrar. O dir e o arquivo dedupam
// para um `mod` só; é o precedente do `tool_rail/tests.rs` e a cicatriz do `command_palette`.
#[path = "skin/tests.rs"]
mod tests;
