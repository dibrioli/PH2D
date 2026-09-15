//! ⭐⭐⭐ **A GEOMETRIA de uma LINHA de formulário** — as colunas, o rótulo e a coluna de animação.
//!
//! ⚠️ **Irmão por RESPONSABILIDADE do [`super`], forçado pelo tecto de 500 LOC do widget**
//! (2026-09-14, ao alinhar o rótulo à direita): *a CAIXA é um widget — ela desenha-se, tem estados
//! e um nó de acessibilidade; a LINHA é a repartição de uma faixa entre um nome e um controlo, e
//! serve painéis que nunca pintam uma caixa.* Os dois crescem por motivos diferentes.
//!
//! ⛔ A re-exportação fica no [`super`], então nenhum chamador muda de caminho.

use super::{DECORATOR_W, surface_rect};
use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::VectorScene;

/// ⭐⭐⭐ **A COLUNA DE ANIMAÇÃO está LIGADA nas linhas de formulário** (Enio, 2026-09-03:
/// *«a bolinha de animação — só desenhá-la»*).
///
/// ⚠️ **É um INDICADOR, não um controlo** — e a diferença é deliberada, não um esquecimento. Ele
/// **não regista hit nenhum**, logo não há clique a cair no vazio: um ponto que se pintasse como
/// alvo e não pusesse chave nenhuma seria um **controlo morto**, a espécie que o `CLAUDE.md` §5.0
/// caça. Ele diz *«esta propriedade é animável»*, que é verdade para todas
/// (*«nessa engine vou querer animar tudo»*), e cala-se sobre o resto.
///
/// ⚠️ **Ele CUSTA 14 px de largura em cada linha do app**, e foi por isso que ficou desligado até
/// haver decisão. A decisão é do dono e está tomada.
///
/// ⏳ Os outros estados do Blender — losango cheio (chave neste quadro), losango vazio (chave
/// noutro), ícone de driver — precisam da **timeline**, não de desenho.
pub const FORM_ROWS_SHOW_DECORATOR: bool = true;

/// **ONDE cai a coluna de animação** — a lei, com dois leitores: a caixa única e a linha de
/// verificação. ⛔ Derivada da [`surface_rect`], para que reservar e desenhar não possam divergir.
#[must_use]
pub(crate) fn decorator_rect(rect: Rect) -> Rect {
    let s = surface_rect(rect, true);
    Rect::new(s.x + s.w, rect.y, DECORATOR_W, rect.h)
}

/// ⭐⭐⭐ **A PORTA de uma linha de formulário construída à mão** — devolve a largura que sobra para
/// os controlos e **onde** pôr o ponto da coluna de animação.
///
/// ⚠️ **Ela existe porque o app tem ~20 construtores de linha à mão** (só o Inspector), cada um com
/// a sua aritmética de larguras, e a alternativa era cada um subtrair `DECORATOR_W` por sua conta.
/// ⛔ *Vinte subtracções são vinte oportunidades de a coluna ficar com um `x` diferente* — e a
/// coluna só quer dizer alguma coisa se for **uma**.
///
/// Uso, em duas linhas:
/// ```ignore
/// let (control_w, dot) = form_row_columns(x, w, row_y, row_h);
/// // …desenhe os controlos dentro de `control_w`…
/// paint_decorator_dot(scene, theme, dot);
/// ```
///
/// ⚠️ **Um ponto por LINHA, nunca por campo:** um par `X`/`Y` é *uma* propriedade com duas
/// componentes, e dois pontos diriam que são duas.
#[must_use]
pub fn form_row_columns(x: f32, w: f32, row_y: f32, row_h: f32) -> (f32, Rect) {
    // ⛔⛔ **A APARÊNCIA decide, e decide AQUI.** Report do Enio, 2026-09-03: sem
    // `PH2D_UI_NEW=1` o app *«abriu o desenho novo»* — porque os três PINTORES perguntavam a
    // aparência e as **linhas construídas à mão** não: são 19 sítios só no Inspector, e nenhum
    // deles a consultava.
    //
    // ⚠️ **A cura vai na porta, nunca nos 19** — pela mesma razão que a porta existe. E vai
    // **também** no [`paint_decorator_dot`]: um chamador que ignore a largura devolvida ainda assim
    // não pode pintar o ponto. *Duas metades, para que esquecer uma não chegue.*
    if !crate::paint::ui_is_redesign() {
        return (w.max(1.0), Rect::new(x + w, row_y, 0.0, row_h));
    }
    let control_w = (w - DECORATOR_W).max(1.0);
    (
        control_w,
        Rect::new(x + w - DECORATOR_W, row_y, DECORATOR_W, row_h),
    )
}

/// ⭐⭐⭐ **A LINHA DE PROPRIEDADE — rótulo à ESQUERDA, controlo à direita, e é UMA lei.**
///
/// ⛔⛔ **Report do dono, 2026-09-14, com duas fotos:** *«Label acima do campo numérico! Muito
/// ruim!»* (a secção LEG do Platform Player) e *«número na frente da label»* (a Sprite Sheet).
/// Medido no mesmo dia: só o Inspector tinha **cinco** pintores de «rótulo + campo numérico» —
/// dois empilhavam o rótulo (`sections/rows::num_row`, `sections/visibility::number_row`) e três
/// punham-no à esquerda —, e a largura da coluna do rótulo tinha **SEIS** respostas em todo o app,
/// cada uma um literal com dispensa: `96` · `78` · `84` · `76` · `72` · `150`.
///
/// ⭐ **E o doc da [`form_row_columns`], aqui ao lado, já nomeava esta família como a que não tinha
/// porta:** *«a terceira — rótulo à esquerda + campos numéricos soltos … é construída à mão em cada
/// painel, com a sua própria aritmética de larguras»*. Esta é essa porta.
///
/// # ⛔ Uma largura FIXA está errada por construção, e a prova não é de gosto
///
/// A coluna docada é **arrastável** (`WidgetStore::DOCK_W_MIN`..`720`). Um rótulo de `96 px` numa
/// coluna aberta a `720` deixa o controlo com `600` — e na largura mínima ele come a linha inteira.
/// *Os seis literais não são seis gostos: são seis leituras da MESMA coluna à largura de omissão.*
/// ⇒ a coluna do rótulo é uma **fracção** da linha.
///
/// # A fracção é O MEIO DA LINHA — ordem do dono
///
/// ⚠️ **Ela já foi MEDIDA e deixou de o ser, e a troca é honesta.** Nasceu a reproduzir o literal
/// que a casa mais escrevia (`96 / 276 = 0,348`) — arqueologia correcta do produto de então. Em
/// 2026-09-14 o dono viu o resultado e decidiu: *«Melhor alinhar no meio do painel e as labels
/// alinhadas todas à direita»*. ⇒ `0,5`, e o rótulo acaba **um vão antes** do meio para o controlo
/// começar exactamente nele.
///
/// ⛔ *Uma medição diz o que o produto FAZ; ela nunca disse o que ele DEVIA fazer* — e a segunda
/// pergunta é do dono (`CLAUDE.md` §0.8).
///
/// # O tecto tem RECURSO, e o recurso é o CONTROLO
///
/// Um rótulo não pode crescer até o campo deixar de ser usável: o piso do controlo é
/// [`ph2d_tokens::ICON_BTN_SIZE_PX`] — a largura de um botão de ícone, que é o que a coluna do
/// *stepper* de um `NumberInput` ocupa — **mais um dígito**. Abaixo disso o campo deixa de ter onde
/// pôr o valor (§0.0: *um limite legítimo diz de que recurso ele é*).
///
/// ⚠️ **O vão horizontal `rótulo → controlo` é [`Spacing::Md`] (8) porque é o que a casa já
/// responde** em 3 dos sítios — e ⏳ **ele NÃO tem derivação**: é a única grandeza desta porta sem
/// origem no modelo, e fica NOMEADA como dívida em vez de fingir-se lei. *Um número que a porta
/// centraliza pode ser corrigido num sítio; um espalhado por seis, não.*
///
/// ⚠️ **O ponto da coluna de animação continua a sair da [`form_row_columns`]** — esta porta
/// COMPÕE com ela e não repete a subtracção do [`DECORATOR_W`]. *Duas portas a responder ao mesmo
/// `x` é a forma que esta casa acabou de pagar seis vezes.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PropertyRow {
    /// Onde o rótulo é pintado — alinhado ao centro vertical da linha, elidido se não couber.
    pub label: Rect,
    /// Onde o controlo (campo, chip, selector) é pintado.
    pub control: Rect,
    /// A coluna de animação, já reservada — passe-a ao [`paint_decorator_dot`].
    pub dot: Rect,
}

/// ⭐⭐⭐ **ONDE O CONTROLO COMEÇA: no MEIO da linha.**
///
/// ⛔⛔ **Ordem do dono, 2026-09-14, com foto dos cartões LEG/WALK/JUMP:** *«as caixas numéricas são
/// muito grandes. Maiores que as labels. Melhor alinhar no meio do painel e as labels alinhadas
/// todas à direita (no centro do painel)»*.
///
/// ⚠️ **Era `0,348` — a razão MEDIDA `96/276`**, isto é, o literal que a casa mais escrevia à
/// largura de omissão. Ela estava certa como *arqueologia* (foi assim que a coluna nasceu) e
/// errada como *desenho*: entregava ao campo quase o dobro do rótulo, e um campo com o dobro da
/// largura do nome dele lê-se como se o número fosse o assunto.
///
/// ⭐ **E ela fecha a pergunta que a wave da unidade deixou aberta:** com a coluna a `84,2 px`,
/// **12** rótulos da §14 eram elididos; ao meio da linha ela mede **120** e o mais comprido do app
/// (*«Swim Line (weights)»*, `113,9`) passa a caber. *A decisão de aparência do dono resolveu, de
/// graça, o item que eu lhe tinha devolvido como escolha.*
///
/// ⚠️ **A fracção é da LINHA, não do utilizável.** A coluna de animação sai do lado do CONTROLO, e
/// medir a metade sobre o utilizável poria a fronteira `7 px` à esquerda do meio — *«o meio do
/// painel» é o meio do que o artista vê*, não o meio do que sobra depois de uma reserva que ele
/// não sabe que existe.
const LABEL_COL_FRAC: f32 = 0.5; // LITERAL-PX-OK: nao e' px, e' A METADE da linha (ordem do dono, 2026-09-14)

/// ⭐⭐⭐ **A METADE HORIZONTAL da [`property_row_columns`] — a largura da coluna do rótulo.**
///
/// ⚠️⚠️ **Ela existe porque a largura NÃO DEPENDE DO VERTICAL, e isso não é um detalhe: era o preço
/// escrito na dívida.** Em 2026-09-14 o painel de vetor ficou fora da conversão com a razão *«33
/// sítios a usá-la como constante livre, **muitos sem `y`/`row_h` em alcance**»* — e a régua estava
/// na moeda errada. O `row_y`/`row_h` da porta grande só decidem **onde** os rects começam e que
/// **altura** têm; a coluna sai de `x` e `w` e de mais nada. *Um bloqueio afirmado sobre um
/// argumento que o resultado não lê é um palpite com cara de medição* — e os 33 sítios tinham todos
/// o `inner_x`/`inner_w` à mão.
///
/// ⛔ **Ela não é uma segunda lei: a [`property_row_columns`] CHAMA-A.** Não há aqui nada para um
/// gate comparar — e isso foi medido: a primeira redacção do gate punha as duas lado a lado e a
/// prova de mutação **sobreviveu**, porque apagar o tecto muda as duas ao mesmo tempo. *Um gate que
/// compara duas construções é cego a uma mutação partilhada.* Quem defende a lei é o
/// `a_property_row_never_starves_its_control`, que mede o **piso do controlo** contra um oráculo
/// fora dela (duas mutações, as duas mortas).
#[must_use]
pub fn property_label_col_w(x: f32, w: f32) -> f32 {
    property_label_col_w_for(x, w, None)
}

/// ⭐⭐⭐ **A mesma coluna, mas o rótulo pode PEDIR EMPRESTADO ao controlo o que ele não usa.**
///
/// ⛔⛔ **Report do dono, 2026-09-14:** *«3 pontos (…) sendo usados antes de ficar estreito»* — com
/// a foto de um painel em que os campos mostravam `2`, `65` e `0.100 s` com espaço de sobra
/// enquanto os nomes ao lado truncavam.
///
/// ⚠️⚠️ **E a medição desmontou a minha suposição: o painel dele NÃO está na largura de omissão.**
/// O `~/.ph2d/layout.txt` diz `dock_w_right = 220,9` (o default é `304`), o que dá uma coluna de
/// **`78,4`** e **16 dos 52** rótulos da §14 elididos. *O meu gate media a largura de OMISSÃO —
/// onde cortam zero — e por isso estava verde sobre o que ele via.*
///
/// ⇒ `desired` é o rótulo mais largo que o chamador vai pintar, e a coluna passa a ser
/// `clamp(desired, metade, o que o controlo pode ceder)`:
///
/// - **o piso é a METADE** — a ordem do dono sobre o alinhamento continua a valer, e numa coluna
///   larga nada muda (a `304` a metade já chega a todos);
/// - **o tecto é o CONTROLO** — ele nunca desce do piso nomeado (o *stepper* mais um dígito), que é
///   o recurso;
/// - **`None` = a metade**, que é o comportamento de quem não sabe o que vai pintar.
///
/// Medido a `220,9`: as elisões passam de **16 para 3** (`Corner Look-ahead`, `Weight on Ground` e
/// `Swim Line (weights)` continuam maiores do que a linha aguenta).
#[must_use]
pub fn property_label_col_w_for(x: f32, w: f32, desired: Option<f32>) -> f32 {
    // ⚠️ O vertical entra a zero **e é deitado fora**: o único uso que a [`form_row_columns`] lhe dá
    // é montar o rect do ponto, e aqui só queremos a largura utilizável (que já desconta a coluna
    // de animação, ou não desconta nada na aparência clássica — a guarda mora lá, uma vez).
    let (usable_w, _dot) = form_row_columns(x, w, 0.0, 0.0);
    let gap = Spacing::Md.px();
    // ⛔⛔ **O piso do CONTROLO é o que o CAMPO declara precisar — e isso é uma ORDEM DO DONO.**
    //
    // ⚠️⚠️ **A 1.ª redacção nomeava o recurso ERRADO, e o doc dela dizia-o em voz alta:** *«o piso
    // do controlo é `ICON_BTN_SIZE_PX` — a largura de um botão de ícone, que é o que a coluna do
    // stepper de um `NumberInput` ocupa — mais um dígito»*. Medido: a coluna do stepper é
    // [`super::super::number_input::stepper_width`], que dá `clamp(0,6 × altura, 16, 22)` — **nunca
    // 36**. Logo `36 + 12 = 48` não era «o stepper mais um dígito»: era um número sem dono.
    //
    // ⛔ E o campo **já tinha** o dele, com a ordem escrita ao lado (2026-05-24): *«não permita que
    // a caixa seja redimensionada para menor que isso»* ⇒ [`super::super::NUMBER_INPUT_MIN_W_PX`]
    // (`72` = ~3-4 dígitos em `Sm` + o recuo + a coluna do stepper). *Quando o recurso já tem dono,
    // o piso é o dele* — a mesma lei que a [`property_label_origin`] paga um bloco acima.
    //
    // ⚠️ **Medido no produto, ANTES da cura** (campo registado, `Float Height` da §14):
    //
    // | painel | campo pintado | piso declarado |
    // |---|---|---|
    // | `220` (o mínimo do dock) | `48,00` | `72` ⛔ |
    // | `245` | `59,38` | `72` ⛔ |
    // | `280` | `94,38` | `72` ✅ |
    //
    // ⇒ *toda a metade estreita do curso do dock pintava o campo abaixo do que o dono mandou.*
    let control_min = super::super::NUMBER_INPUT_MIN_W_PX;
    // ⭐ O rótulo acaba um VÃO antes do meio da linha, para o controlo começar EXACTAMENTE nele.
    let metade = w * LABEL_COL_FRAC - gap;
    let tecto = (usable_w - gap - control_min).max(0.0);
    desired.unwrap_or(metade).max(metade).min(tecto).max(0.0)
}

/// ⭐⭐⭐ **O RÓTULO de uma linha de propriedade — alinhado à DIREITA, encostado ao controlo.**
///
/// ⛔⛔ **Ordem do dono, 2026-09-14:** *«as labels alinhadas todas à direita (no centro do
/// painel)»*. Com o controlo a começar no meio da linha, um rótulo alinhado à esquerda deixa um
/// rio de espaço variável entre o nome e o campo dele — e *quanto mais curto o nome, mais longe do
/// valor que ele nomeia*. Encostado à direita, o par nome-valor lê-se como um par.
///
/// ⚠️ **Elidido, e MEDIDO no peso em que pinta.** A largura sai do [`crate::text_elide::fit`] e do
/// `prefix_width`, os dois em `Medium` — *medir num peso e pintar noutro corta curto*, que é um
/// defeito que esta casa já pagou duas vezes.
///
/// ⚠️ **Quando nem assim cabe, degrada para a ESQUERDA:** o `x` recuado nunca passa de `label.x`,
/// logo um rótulo maior que a coluna encosta ao princípio dela e corta no fim — nunca invade o
/// vão nem o controlo.
/// ⚠️ **A assinatura é a do [`crate::paint::paint_text_elided`], argumento a argumento** — `x` é a
/// borda ESQUERDA da coluna e `y` a linha de base já centrada. É de propósito: converter um sítio
/// passa a ser trocar o nome da função, e uma conversão que muda a forma da chamada em 30 sítios é
/// uma conversão que alguém faz pela metade.
#[allow(clippy::too_many_arguments)]
pub fn paint_property_label(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    col_w: f32,
    color: ph2d_vector::Color,
) {
    if col_w <= 0.0 {
        return;
    }
    let (cabe, recuo, largura) = property_label_origin(text_system, text, x, font_size, col_w);
    // ⛔⛔ **O orçamento é a largura MEDIDA do que já coube — nunca `x + col_w − recuo`.**
    // Aquela diferença cancela em `f32` e devolve um valor um ULP abaixo de `largura` em ~8,6 %
    // das posições de `x`, e o pintor voltava a cortar um rótulo que cabia: era isto que o dono
    // via como *«3 pontos mesmo com folga»*. Ver o doc da [`property_label_origin`].
    //
    // ⚠️ **O `min(col_w)` guarda o caso degenerado** — numa coluna mais estreita que a própria
    // reticência o `fit` devolve o texto CRU, e ali o orçamento tem de continuar a ser a coluna,
    // para o pintor recusar em vez de invadir o controlo.
    crate::paint::paint_text_elided(
        text_system,
        scene,
        &cabe,
        recuo,
        y,
        font_size,
        largura.min(col_w),
        color,
    );
}

/// ⭐⭐ **A DECISÃO do alinhamento, sozinha: o texto que cabe e o `x` em que ele começa.**
///
/// ⚠️ **Ela é uma porta separada porque a decisão tem de ser GATEÁVEL.** Dentro do pintor ela só é
/// observável por quem sabe ler glifos de uma cena — e *uma decisão que só o pintor conhece é uma
/// decisão que nenhuma mutação mata*. O mesmo motivo que tirou a escolha do quad desdobrado do fio
/// do Sprite Inspector.
///
/// Devolve `(texto já elidido, x de origem, **a largura medida desse texto**).
///
/// ⛔⛔ **A terceira componente existe por um defeito MEDIDO** (report do dono, 2026-09-14, foto do
/// cartão JUMP: *«… mesmo com folga»*). O pintor precisa da largura do que coube, e re-derivava-a
/// por `x + col_w − recuo` — que em `f32` **não devolve `largura`**: `recuo` é ele próprio
/// `x + (col_w − largura)`, e a soma-e-subtracção cancela com erro. Quando o resultado cai um ULP
/// abaixo, o pintor conclui que o texto já não cabe e **volta a cortá-lo**, pondo reticências num
/// rótulo com dezenas de píxeis de folga. Medido: **310 de 3 600** células (nove rótulos × 400
/// posições de `x`) numa coluna de `140 px` onde o mais largo mede `100,3`.
///
/// ⇒ *quem já mediu uma grandeza devolve-a; re-derivá-la por diferença é a segunda conta que
/// discorda da primeira* — a mesma lei que a [`super::surface_rect`] paga um nível acima.
#[must_use]
pub fn property_label_origin(
    text_system: &mut TextSystem,
    text: &str,
    x: f32,
    font_size: f32,
    col_w: f32,
) -> (String, f32, f32) {
    let cabe = crate::text_elide::fit(text_system, text, font_size, col_w);
    let largura = text_system.prefix_width(&cabe, font_size);
    // ⚠️ **O `max(0)` é o degrau para a ESQUERDA**: um texto maior que a coluna (quando nem a
    // reticência cabe, o `fit` devolve-o cru) encosta ao princípio dela em vez de recuar para fora.
    (cabe, x + (col_w - largura).max(0.0), largura)
}

/// ⭐⭐⭐ **A porta de uma linha de propriedade** — ver [`PropertyRow`].
#[must_use]
pub fn property_row_columns(x: f32, w: f32, row_y: f32, row_h: f32) -> PropertyRow {
    property_row_columns_for(x, w, row_y, row_h, None)
}

/// ⭐ **A mesma porta, com o rótulo mais largo que o chamador vai pintar** — ver
/// [`property_label_col_w_for`].
#[must_use]
pub fn property_row_columns_for(
    x: f32,
    w: f32,
    row_y: f32,
    row_h: f32,
    desired_label_w: Option<f32>,
) -> PropertyRow {
    let (usable_w, dot) = form_row_columns(x, w, row_y, row_h);
    let gap = Spacing::Md.px();
    let label_w = property_label_col_w_for(x, w, desired_label_w);
    let control_x = x + label_w + gap;
    let control_w = (usable_w - label_w - gap).max(1.0);
    PropertyRow {
        label: Rect::new(x, row_y, label_w, row_h),
        control: Rect::new(control_x, row_y, control_w, row_h),
        dot,
    }
}

/// ⭐⭐⭐ **QUANTOS CAMPOS CABEM LADO A LADO NA COLUNA DO CONTROLO — e o que não cabe DESCE.**
///
/// ⛔⛔ **Ela nasce de duas ordens do dono que se CONTRARIAM numa linha de várias componentes:**
/// *«Label acima do campo numérico! Muito ruim!»* (2026-09-14) põe o nome ao lado, o que entrega
/// ao controlo **metade** da linha; e *«não permita que a caixa seja redimencionada para menor que
/// isso»* (2026-05-24) põe um piso de [`super::super::NUMBER_INPUT_MIN_W_PX`] em cada caixa. Numa
/// row de `X`/`Y` à largura de omissão do Inspector as duas não cabem ao mesmo tempo: a coluna do
/// controlo mede `128 px` e dois campos ao piso pedem `148`.
///
/// ⚠️ **A saída NÃO é encolher a coluna do rótulo para esta linha.** A granularidade da coluna é a
/// **SECÇÃO** — foi isso que o dono pediu (*«as labels alinhadas todas à direita»*), e uma coluna
/// por linha devolve a coluna irregular que a wave anterior recusou.
///
/// ⭐ **A saída é a que a [`super::seg_row`] irmã já pratica há um dia:** *o controlo REFLUI e a
/// coluna do rótulo não.* As componentes que não cabem ao piso descem para a linha seguinte,
/// **dentro da coluna do controlo** — que é exactamente como o Blender desenha um vector num painel
/// estreito.
///
/// Devolve `(campos por linha, número de linhas, largura de cada CÉLULA)`. ⚠️ **A largura é a mesma
/// em todas as linhas** — a última fica curta em vez de esticar o campo que sobra, senão um `Bounds`
/// de quatro componentes acabaria com o `H` ao dobro da largura do `X`.
///
/// ⚠️ **Quando nem UM campo cabe ao piso, devolve-se a coluna inteira mesmo assim**: aí o recurso
/// que falta é o painel, e encolher mais só apagaria o número. É a mesma resposta que a
/// [`property_label_col_w_for`] dá ao seu próprio tecto.
///
/// # ⭐⭐ O `lead`: o que a célula gasta ANTES da caixa
///
/// ⛔⛔ **Ordem do dono, 2026-09-15:** *«a mesma formatação do Position X/Y que fez para Anchor vou
/// querer para todo o Transform»*. As linhas do Transform trazem uma **letra de eixo colorida**
/// (`X` / `Y`) à esquerda de cada caixa, e essa letra é parte da COMPONENTE, não do rótulo da linha.
///
/// ⇒ `lead` é a largura que a letra e o vão dela consomem. **O piso continua a ser o da CAIXA** — a
/// célula precisa de `lead + piso`, e a caixa que o chamador pinta mede `célula − lead`. ⛔ Somar o
/// `lead` ao piso e passar isso como «piso» pareceria igual e mentiria sobre o recurso: quem tem
/// dono é a caixa (`CLAUDE.md` §0.0).
///
/// Quem não tem decoração nenhuma passa `0,0` — e aí a célula **é** a caixa.
#[must_use]
pub fn property_fields_layout(
    control_w: f32,
    n: usize,
    gap: f32,
    lead: f32,
) -> (usize, usize, f32) {
    let n = n.max(1);
    let celula_min = lead + super::super::NUMBER_INPUT_MIN_W_PX;
    let mut por_linha = 1usize;
    while por_linha < n {
        let k = (por_linha + 1) as f32;
        if k * celula_min + (k - 1.0) * gap <= control_w {
            por_linha += 1;
        } else {
            break;
        }
    }
    let linhas = n.div_ceil(por_linha);
    let largura = ((control_w - gap * (por_linha as f32 - 1.0)) / por_linha as f32).max(1.0);
    (por_linha, linhas, largura)
}

/// ⭐ **A porta da coluna de animação para quem NÃO usa a caixa única.**
///
/// ⚠️ Ela existe porque o app tem **três** famílias de linha de formulário, e só duas passam por
/// aqui: a caixa única e a linha de verificação. A terceira — rótulo à esquerda + campos numéricos
/// soltos, o Transform do Inspector — é construída à mão em cada painel, com a sua própria
/// aritmética de larguras. ⛔ Sem esta porta cada uma delas desenharia o seu próprio ponto, e a
/// coluna que o dono pediu ficaria com um `x` por painel.
///
/// O chamador **reserva** a coluna (encolhendo a largura das suas colunas em [`DECORATOR_W`]) e
/// chama isto com o rect dela.
pub fn paint_decorator_dot(scene: &mut VectorScene, theme: Theme, r: Rect) {
    // ⛔ A segunda metade da guarda — ver [`form_row_columns`].
    if !crate::paint::ui_is_redesign() {
        return;
    }
    paint_decorator(scene, theme, r, false);
}

/// A coluna de animação.
///
/// ⚠️ Aqui ela é só o estado «animável, sem chave» (o ponto vazio). Os outros do Blender — losango
/// cheio (chave neste quadro), losango vazio (chave noutro), ícone de driver — são trabalho a
/// seguir e precisam da **timeline**, não de desenho.
pub(crate) fn paint_decorator(scene: &mut VectorScene, theme: Theme, r: Rect, disabled: bool) {
    let d = Spacing::Xs.px();
    let dot = Rect::new(r.x + (r.w - d) * 0.5, r.y + (r.h - d) * 0.5, d, d);
    let c = if disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text3
    };
    fill_rounded_rect(scene, dot, d * 0.5, resolve(c, theme));
}

/// Trunca o rótulo para caber, com reticências.
///
/// ⚠️ Devolve string VAZIA quando nem duas letras cabem — e isso é uma resposta, não uma falha: a
/// caixa fica só com o número, que é o degrau seguinte da escada do estreito (pesquisa §6.1).
///
/// ⏳ A alternativa é o **esbatimento** (`Scene::push_luminance_mask_layer`, zero consumidores
/// hoje): em vez de `…`, o rótulo desvanece nos últimos px. É mais bonito e não come letras —
/// nomeado na pesquisa §7.3, com o custo por medir.
///
/// ⚠️ **`pub(crate)` porque a lei tem um SEGUNDO leitor desde 2026-09-03: a caixa de verificação**
/// (o widget mais usado do app, 81 sítios). *Uma lei de truncagem copiada para o vizinho é a
/// primeira linha de um formulário em que metade das linhas cede e a outra metade transborda.*
pub(crate) fn fit_label(
    text_system: &mut TextSystem,
    label: &str,
    size: f32,
    budget: f32,
) -> String {
    if budget <= 0.0 {
        return String::new();
    }
    if text_system.layout(label, size, f32::INFINITY).width() <= budget {
        return label.to_string();
    }
    let ell = "\u{2026}";
    let mut chars: Vec<char> = label.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let cand: String = chars.iter().collect::<String>() + ell;
        if text_system.layout(&cand, size, f32::INFINITY).width() <= budget {
            return cand;
        }
    }
    String::new()
}
