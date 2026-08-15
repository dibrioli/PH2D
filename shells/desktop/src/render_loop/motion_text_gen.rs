//! **A metade do shell do `source.text`** (doc 89, folha 14 §3 item 1) — monta o
//! bloco de texto, interna **um `VecPath` por glifo** e publica um stream de **uma
//! instância por caractere** no canal externo que o nó lê.
//!
//! Irmão exacto do [`motion_shape_gen`](super::motion_shape_gen), e de propósito:
//! um nó recebe params, entradas e o playhead — nada mais —, então quem alcança a
//! fonte e a biblioteca de vetor é o shell. O que muda em relação à forma é a
//! CONTAGEM: uma forma publica UMA linha, um texto publica uma por letra, e é isso
//! que faz a biblioteca `motion.*` inteira agir por caractere sem um nó novo.
//!
//! ## O layout não é feito aqui
//!
//! Ele vem de [`crate::vec_glyph::walk_glyphs`], a MESMA porta que o texto do
//! editor Vector usa. Um segundo laço aqui responderia *"onde cai cada letra?"*
//! uma segunda vez, e as duas respostas divergiriam no dia em que alguém mexesse
//! no alinhamento — com o texto do canvas e o texto do grafo a desenhar coisas
//! diferentes a partir dos mesmos números.
//!
//! ## O pivô sai do próprio laço, sem estado nenhum
//!
//! [`Pivot::Center`] desloca a geometria `−adv/2` e o `P` `+adv/2`, com o avanço
//! que o laço já tem na mão. ⚠️ Nada é lembrado entre quadros para isso: o
//! deslocamento entra na CHAVE do glifo, então o cache endereçado por conteúdo
//! resolve-o sozinho — e a soma `geometria + P` é a mesma nos dois pivôs, que é o
//! que torna a escolha invisível em repouso (gate).

use ph2d_node_source_text::{
    FONT_KEY, MANIFEST, Pivot, TEXT_KEY, TextParams, font_of, text_key, text_of,
};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_tool_vector::TextAlign;
use ph2d_vec_scene::VecPath;
use ph2d_vec_scene::text_path::GlyphFrame;
use ph2d_vector_font::{AxisTag, GlyphId, VariableFont};

use crate::motion_state::MotionState;
use crate::vec_glyph::{TextLayout, TextPlacement, walk_glyphs};
use crate::vec_glyph_build::glyph_to_vec_path;

/// O default do manifesto para um param — o fallback que o `ctx.param` do nó toma
/// quando não há override. Ler pelo mesmo caminho dos dois lados é o que faz a
/// chave do shell e a do nó serem os mesmos bits.
fn manifest_default(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|s| s.name == name)
        .map(|s| s.default)
        .unwrap_or(0.0)
}

/// **A chave de conteúdo de UM GLIFO** — o que decide se dois carimbos partilham
/// geometria. As três letras de `"AAA"` internam uma vez.
///
/// ⚠️ Ela carrega o DESLOCAMENTO do pivô e não o modo: é o deslocamento que entra
/// na construção, e uma chave que nomeasse a intenção em vez da entrada voltaria a
/// dar a geometria errada no dia em que o cálculo do pivô mudasse.
fn glyph_key(font: &str, size: f32, weight: f32, shift: f64, ch: char) -> String {
    format!(
        "glyph:{}:{font}:{}:{}:{}:{ch}",
        font.len(),
        size.to_bits(),
        weight.to_bits(),
        shift.to_bits()
    )
}

/// A geometria de UM glifo, na origem dele (deslocada por `−shift` em x).
///
/// ⚠️ **`fill` e `stroke` ficam em `None`, e isso é a convenção do renderer, não
/// omissão**: um primitivo sem tinta autorada é preenchido com o `tint` da
/// INSTÂNCIA (`tessellate_shape_instance`), que é o que deixa um `motion.tint`
/// colorir as letras a jusante. Pôr um `Paint` aqui tornaria o texto imune a ele.
fn glyph_path(
    font: &VariableFont,
    gid: GlyphId,
    scale: f64,
    shift: f64,
    axes: &[(AxisTag, f32)],
) -> Option<VecPath> {
    let frame = GlyphFrame {
        origin: [-shift, 0.0],
        x_axis: [1.0, 0.0],
        y_axis: [0.0, 1.0],
    };
    let outline = font.outline(gid, axes).ok()?;
    glyph_to_vec_path(&outline, scale, &frame, None, None)
}

/// Monta o stream de um bloco: uma linha por glifo COM contorno.
///
/// ⚠️ Um glifo sem contorno (o espaço) **não vira linha** — uma instância sem
/// geometria desenharia nada e ainda assim contaria como elemento, então um
/// `motion.stagger` atrasaria pelos espaços e a onda ficaria com buracos que o
/// artista não escreveu. O pen avança na porta de layout, antes desta decisão, e é
/// por isso que tirar o espaço da CONTAGEM não o tira do ESPAÇAMENTO.
pub(crate) fn build_stream(
    store: &mut super::motion_shape_gen::VecPathStore,
    p: &TextParams,
    font_name: &str,
    text: &str,
) -> Stream {
    let font = crate::vec_font::resolve((!font_name.is_empty()).then_some(font_name));
    let axes = [(AxisTag::WEIGHT, p.weight)];
    let layout = TextLayout {
        size: f64::from(p.size),
        line_height: f64::from(p.line_height),
        tracking: f64::from(p.tracking),
        align: match p.align {
            ph2d_node_source_text::Align::Left => TextAlign::Left,
            ph2d_node_source_text::Align::Center => TextAlign::Center,
            ph2d_node_source_text::Align::Right => TextAlign::Right,
        },
        // Sem caixa: o refluxo é a wave seguinte (folha 14 §3), e uma caixa que
        // ninguém autora seria um número inventado a decidir onde a linha parte.
        wrap_width: None,
    };
    let scale = layout.size / f64::from(font.units_per_em().max(1));
    let placement = TextPlacement::At([0.0, 0.0]);
    let mut pos: Vec<[f32; 2]> = Vec::new();
    let mut geo: Vec<f32> = Vec::new();
    walk_glyphs(
        &font,
        text,
        &layout,
        &axes,
        &placement,
        |ch, gid, pen, advance| {
            let shift = match p.pivot {
                Pivot::Pen => 0.0,
                Pivot::Center => advance / 2.0,
            };
            let key = glyph_key(font_name, p.size, p.weight, shift, ch);
            let handle = match store.handle_for(&key) {
                Some(h) => h,
                None => match glyph_path(&font, gid, scale, shift, &axes) {
                    Some(path) => store.intern(&key, || path),
                    None => return, // espaço / contorno vazio: nada a desenhar
                },
            };
            #[expect(
                clippy::cast_possible_truncation,
                reason = "mundo em f32 — a mesma fronteira que todo P do stream atravessa"
            )]
            pos.push([(pen[0] + shift) as f32, pen[1] as f32]);
            geo.push(handle as f32);
        },
    );
    Stream::new(pos.len())
        .with("P", Column::Vec2(pos))
        .with("geometry_id", Column::Scalar(geo))
}

/// Publica a geometria de todo nó `source.text` no cook.
///
/// ⚠️ **Chamada POST-dreno, PRÉ-cook**, ao lado da irmã das formas e pela mesma
/// razão: publicar antes do dreno mintaria a chave PRÉ-edição enquanto o cook lê a
/// PÓS-edição ⇒ o nó clona um externo vazio por um quadro ⇒ **o texto pisca ao
/// editar**.
pub(crate) fn publish(motion: &mut MotionState) {
    // Junta os trabalhos primeiro para o empréstimo do grafo morrer antes de
    // mutarmos o store e o cook (três campos disjuntos do `MotionState`).
    let graph = &motion.doc.graph;
    let jobs: Vec<(String, TextParams, String, String)> = graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == MANIFEST.name)
        .map(|n| {
            let ov = graph.node_param_overrides(n.id);
            let get = |name: &str| {
                ov.and_then(|m| m.get(name).copied())
                    .unwrap_or_else(|| manifest_default(name))
            };
            let tov = graph.node_text_params().get(&n.id);
            let text = text_of(tov.and_then(|m| m.get(TEXT_KEY)).map(String::as_str)).to_string();
            let font = font_of(tov.and_then(|m| m.get(FONT_KEY)).map(String::as_str)).to_string();
            (
                text_key(get, &font, &text),
                TextParams::read(get),
                font,
                text,
            )
        })
        .collect();
    for (key, p, font, text) in jobs {
        let stream = build_stream(&mut motion.shape_store, &p, &font, &text);
        motion.pump.cook.set_external(key, stream);
    }
}

#[cfg(test)]
#[path = "motion_text_gen_tests.rs"]
mod tests;
