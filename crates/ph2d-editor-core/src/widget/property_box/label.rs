//! ⭐⭐⭐ **O NOME de uma linha de propriedade** — como ele é medido, truncado e pousado.
//!
//! ⚠️ **Irmão por RESPONSABILIDADE do [`super::row`], forçado pelo tecto de 500 LOC do widget**
//! (2026-09-15, quando a coluna do nome aprendeu a CEDER ao controlo): aquele ficheiro responde
//! *como a faixa se reparte entre um nome e um controlo*; este responde *o que acontece ao texto
//! do nome dentro da coluna que lhe coube*. ⛔ Os dois crescem por motivos diferentes — o primeiro
//! por ordens de disposição do dono, o segundo por defeitos de medição de texto.
//!
//! ⛔ A re-exportação fica no [`super`], então nenhum chamador muda de caminho.

use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

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
