//! Value → transform-channel plumbing for `motion.drive`, including the **one
//! broadcast rule** the whole value domain hangs on (doc 12): a value field of
//! length 1 is HELD (broadcast) across every instance; a length-N field applies
//! element-wise; any other length is a mismatch (`debug_assert`, then a lenient
//! element-wise fallback — no silent no-op). This is the TouchDesigner
//! "held constant" / Houdini "detail→point" rule, restricted to `1→N` only so
//! the strict substrate stays honest.
//!
//! Self-contained per drop-crate (like every behaviour's `channel`/`falloff`
//! helpers). Combine modes lerp the RESULT toward the driven value by the
//! multiplicative `falloff` field, so a focus mask limits which instances the
//! value drives — consistent with the rest of the family.

use ph2d_nodegraph::attr::{Column, Stream};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
pub(crate) fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The value driving instance `i`, applying the broadcast rule: a length-1
/// field broadcasts (the constant is held at every instance); a length-N field
/// is read element-wise. A length that is neither 1 nor `n` is a mismatch —
/// `debug_assert`ed loudly, then read leniently (element-wise, `0.0` past the
/// end) so a release build degrades rather than panics.
pub(crate) fn value_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 0.0,
        1 => vals[0], // broadcast: one value → every instance (the 1→N rule)
        _ => vals.get(i).copied().unwrap_or(0.0),
    }
}

/// How the driven value combines with the existing channel.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Combine {
    /// `channel + value` — the additive default (matches `motion.step`).
    Add,
    /// `value` — overwrite the channel with the driven value.
    Set,
    /// `channel * value` — scale the existing channel by the value.
    Multiply,
}

impl Combine {
    pub(crate) fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Combine::Set,
            2 => Combine::Multiply,
            _ => Combine::Add,
        }
    }
    fn apply(self, channel: f32, value: f32) -> f32 {
        match self {
            Combine::Add => channel + value,
            Combine::Set => value,
            Combine::Multiply => channel * value,
        }
    }
}

/// The channel index of Opacity — the alpha of the `tint` column.
pub(crate) const CH_OPACITY: i32 = 4;

/// The channel index of **Falloff** — the MASK itself, and the reason this channel exists.
///
/// The `falloff` column is the vocabulary five families share (the `field.*`, the `force.*`, the
/// deformers, the transforms, the stylistics all READ it), and until now it could only be
/// DERIVED FROM GEOMETRY: nothing computed could become a mask. That is the wall doc 89 §10.0
/// measured — a noise field, a per-instance density, force-over-life and luma-as-mask were all
/// inexpressible for the same one reason.
///
/// ⚠️ It does NOT self-mask, and the machinery is what decides that rather than taste: every
/// other variant binds `falloff` as a READ to blend with, so a variant whose TARGET is `falloff`
/// binds it once as `ReadWrite` and the common read is simply not there. Masking the writing of
/// the mask by the mask makes `Set` non-idempotent and means nothing to an artist.
///
/// ⚠️ And it is NOT clamped to `[0, 1]`, deliberately: a NEGATIVE weight inverts the force that
/// consumes it — a capability the conference found we have and C4D/Cavalry do not, because their
/// falloffs are `[0,1]` by construction. Clamping here would delete it before anyone used it.
pub(crate) const CH_FALLOFF: i32 = 5;

/// Os três canais de **COR sobre a cor que já está lá** — matiz, saturação e valor do
/// `tint`, apendados depois do `CH_FALLOFF` para nenhum grafo já autorado mudar de canal.
///
/// ⚠️ **Eles são a metade de ESCRITA do laço de cor**, e a razão de existirem está medida na
/// §0 do doc 89 fam. 9: *nada no catálogo escrevia R/G/B a partir de um valor* — o único
/// canal de cor daqui era o `CH_OPACITY`, que escreve o ALFA. Com o `motion.luminance`
/// lendo os oito canais (a metade de LEITURA, fechada na wave anterior), o laço
/// `luminance(Hue) → value.math → drive(Hue)` passa a existir.
///
/// ⚠️ **E os modos de combinação já eram o vocabulário da referência**, sem um param novo:
/// o *Master Hue* do AE / o `Hue` do Blender é um **deslocamento** (`Add`), e
/// *Saturation*/*Lightness* são **escalas** (`Multiply`) — `hue += 0`, `sat *= 1`,
/// `val *= 1` é a identidade que o doc pedia como default que reduz.
pub(crate) const CH_HUE: i32 = 6;
/// Ver [`CH_HUE`].
pub(crate) const CH_SAT: i32 = 7;
/// Ver [`CH_HUE`].
pub(crate) const CH_VAL: i32 = 8;

/// **O canal CUSTOM — o ESCRITOR DE COLUNA NOMEADA.**
///
/// Ele fecha a metade que faltava da §10.0 do plano 89: o `value.attribute` **LÊ**
/// qualquer coluna por nome (widget `Channels` + escape *"Custom…"*) e este nó só
/// **ESCREVIA** nas nove que o enum conhece — a assimetria que seis famílias
/// citaram, e que o doc 63 marcava como *"TEMOS"* apontando para o LEITOR.
///
/// ⚠️ **Ele é o ÚLTIMO índice, e isso não é arrumação:** o `channel` é um param
/// que o **documento GUARDA**, então renumerar o que já existe re-aponta em
/// silêncio todo grafo salvo. O espaço positivo cresce para cima, como o
/// `CH_HUE` fez.
///
/// ⚠️ **Ele RECUSA o device** (`applicable`), e o motivo é estrutural e não
/// preguiça: uma `ColumnBinding` carrega o nome da coluna como `&'static str` —
/// o sequenciador precisa dele **antes** de cozer, para montar o bind group —, e
/// um nome que o artista digita só existe em tempo de cook. O precedente é a
/// `Median` do `value.reduce`, que recua pela mesma porta. *Um caminho mais
/// lento é honesto; um nome escrito no buffer errado não é.*
pub(crate) const CH_CUSTOM: i32 = 9;

/// **UM EIXO do tamanho** — o *Scale + S.XYZ* do C4D, contra o *Uniform Scale* que o
/// canal `Size` (3) sempre foi.
///
/// ⚠️ **Metade do pedido já era exprimível e a outra metade não, e a medição separou as
/// duas** (`measure_size_axes`): `drive(Size) → motion.scale(uniform=0, amount≠amount_y)`
/// dá **anisotropia FIXA sobre uma magnitude DIRIGIDA** (medido, pior `|x−y| = 1,500000`
/// com razão 4:1 em todo elemento) — que é o squash-and-stretch inteiro. O que ela **não**
/// dá é **DOIS campos independentes**, um por eixo: o `motion.scale` escala com um PARAM,
/// que é o mesmo número para todas as peças. E o `Custom…` também não — escrever um
/// `Scalar` por cima de um `Vec2` mudaria o TIPO da coluna, então ele **recusa** e devolve
/// a entrada verbatim (medido: `|x−y| = 0,000000`).
///
/// ⚠️ **Apendados, nunca inseridos** — o `channel` é um param que o **documento GUARDA**,
/// e o `CH_CUSTOM` já pagou essa lição: renumerar re-aponta em silêncio todo grafo salvo.
pub(crate) const CH_SIZE_X: i32 = 10;
/// Ver [`CH_SIZE_X`].
pub(crate) const CH_SIZE_Y: i32 = 11;

/// A chave do text param que carrega o NOME da coluna — o espelho exacto do
/// `ATTR_KEY` do `value.attribute`, que é o leitor deste escritor.
pub const DRIVE_COL_KEY: &str = "column";

/// The stream column a channel index writes to: X/Y → `P`, Rotation → `rot`,
/// Opacity → `tint` (its alpha), Hue/Saturation/Value → `tint` (its colour),
/// Size (or any out-of-range value) → `size`.
pub(crate) fn channel_column(channel: i32) -> &'static str {
    match channel {
        0 | 1 => "P",
        2 => "rot",
        CH_OPACITY | CH_HUE | CH_SAT | CH_VAL => "tint",
        CH_FALLOFF => "falloff",
        _ => "size",
    }
}

/// **Escreve uma coluna ESCALAR nomeada** — a lei do [`drive_channel`] com o
/// alvo dado pelo artista em vez de pelo enum.
///
/// ⚠️ **Ela partilha os helpers e não a estrutura, de propósito:** o `blend`, o
/// `value_at` e o `falloff_at` são os MESMOS, então a mistura por `falloff` e o
/// broadcast do campo de valor não podem divergir do resto do nó; o que difere é
/// só *onde o número pousa*.
///
/// **Duas recusas, e as duas devolvem a entrada verbatim:**
/// - **nome vazio** — não há coluna a escrever, e cunhar uma chamada `""` seria
///   uma coluna que ninguém consegue nomear de volta;
/// - **a coluna existente não é ESCALAR** — ⚠️ e esta é a que importa: escrever
///   um `Scalar` por cima de um `P` (`Vec2`) ou de um `tint` (`Vec4`) **muda o
///   TIPO** da coluna, e quem a lê a jusante passa a ler outra coisa. A pergunta
///   é feita ao STREAM e não a uma lista de nomes proibidos — *uma lista
///   apodrece no dia em que nasce a décima convenção* (`texture_id`,
///   `geometry_id`, `uv_rect`, `nrm` … esta casa já as tem).
pub(crate) fn drive_named(
    input: &Stream,
    name: &str,
    vals: &[f32],
    scale: f32,
    mode: Combine,
) -> Stream {
    let n = input.count();
    // ⚠️ **DUAS recusas, e elas perguntam coisas diferentes.** A do TIPO impede
    // que um `Scalar` mude a FORMA de uma coluna que já existe (escrever sobre
    // `P` faria todo leitor a jusante ler outra coisa). A da ESCRITURAÇÃO impede
    // que ele acerte uma coluna cuja FORMA está certa e cujo PAPEL não é dado —
    // `id` e `sim_t` são escalares, então o teste de tipo os deixa passar, e um
    // `sim_t` no futuro faz `dt` clampar em zero e **congela a simulação**, sem
    // erro e sem badge. A porta é do substrato (`ph2d_nodegraph::attr`) porque
    // uma crate-nó não pode depender de outra (ADR-0075).
    if name.is_empty()
        || ph2d_nodegraph::attr::is_bookkeeping_column(name)
        || !matches!(input.get(name), None | Some(Column::Scalar(_)))
    {
        return input.clone();
    }
    let mut out = Stream::new(n);
    for (col_name, col) in input.columns() {
        if col_name != name {
            out.set(col_name.clone(), col.clone());
        }
    }
    let blend = |orig: f32, driven: f32, f: f32| orig + (driven - orig) * f.clamp(0.0, 1.0);
    // ⚠️ Coluna AUSENTE nasce em zero, e é o mesmo que o `base_vec2`/`base_vec4`
    // fazem para os canais do enum: o identity de um escalar dirigido é `0`, e
    // com `Combine::Add` isso faz o primeiro `drive` escrever o valor cru.
    let mut v = match input.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, 0.0);
    for (i, x) in v.iter_mut().enumerate() {
        let driven = mode.apply(*x, value_at(vals, i) * scale);
        *x = blend(*x, driven, falloff_at(input, i));
    }
    out.set(name.to_string(), Column::Scalar(v));
    out
}

fn base_vec2(input: &Stream, name: &str, n: usize, identity: [f32; 2]) -> Vec<[f32; 2]> {
    let mut v = match input.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, identity);
    v
}
fn base_vec4(input: &Stream, name: &str, n: usize, identity: [f32; 4]) -> Vec<[f32; 4]> {
    let mut v = match input.get(name) {
        Some(Column::Vec4(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, identity);
    v
}
/// A scalar column, materialised at full length from `identity` where it is absent.
/// ⚠️ The identity is a PARAMETER because it is not always `0`: an absent `falloff` means
/// `1.0` to every reader in the library (`falloff_at`'s own fallback), and a writer that
/// started it from `0` would disagree with every consumer about what "no mask" means.
fn base_scalar(input: &Stream, name: &str, n: usize, identity: f32) -> Vec<f32> {
    let mut v = match input.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, identity);
    v
}

/// Drive the selected `channel` of `input` from the value field `vals`
/// (scaled by `scale`, combined by `mode`), broadcast per the rule above and
/// masked per-instance by `falloff`. Returns a new stream with that column
/// rewritten and every other column copied through. Size drives both
/// components (uniform); an absent target column starts from its identity
/// (`0` for P/rot, unit `[1,1]` for size).
pub(crate) fn drive_channel(
    input: &Stream,
    channel: i32,
    vals: &[f32],
    scale: f32,
    mode: Combine,
) -> Stream {
    let n = input.count();
    debug_assert!(
        vals.is_empty() || vals.len() == 1 || vals.len() == n,
        "motion.drive: value field len {} matches neither 1 (broadcast) nor {n} (element-wise)",
        vals.len()
    );
    let target = channel_column(channel);
    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != target {
            out.set(name.clone(), col.clone());
        }
    }
    // Lerp the combined result toward the original by falloff: `falloff = 0`
    // leaves the channel untouched, `1` takes the full drive.
    let blend = |orig: f32, driven: f32, f: f32| orig + (driven - orig) * f.clamp(0.0, 1.0);
    match channel {
        0 | 1 => {
            let comp = channel as usize; // 0 = X, 1 = Y
            let mut p = base_vec2(input, "P", n, [0.0, 0.0]);
            for (i, pi) in p.iter_mut().enumerate() {
                let driven = mode.apply(pi[comp], value_at(vals, i) * scale);
                pi[comp] = blend(pi[comp], driven, falloff_at(input, i));
            }
            out.set("P", Column::Vec2(p));
        }
        2 => {
            let mut r = base_scalar(input, "rot", n, 0.0);
            for (i, ri) in r.iter_mut().enumerate() {
                let driven = mode.apply(*ri, value_at(vals, i) * scale);
                *ri = blend(*ri, driven, falloff_at(input, i));
            }
            out.set("rot", Column::Scalar(r));
        }
        // **Opacity** — the ALPHA of the tint, and the reason a particle can fade out at all
        // (doc 51). An element with no tint starts from opaque white, so driving the opacity of
        // an uncoloured stream does exactly what it says instead of silently doing nothing.
        //
        // Clamped to `[0, 1]`: the renderer alpha-blends, and an alpha of 1.4 or -0.2 is not a
        // brighter or a darker particle — it is a particle that reads as a bug.
        CH_OPACITY => {
            let mut t = base_vec4(input, "tint", n, [1.0, 1.0, 1.0, 1.0]);
            for (i, ti) in t.iter_mut().enumerate() {
                let driven = mode.apply(ti[3], value_at(vals, i) * scale);
                ti[3] = blend(ti[3], driven, falloff_at(input, i)).clamp(0.0, 1.0); // CLAMP-OK: alpha
            }
            out.set("tint", Column::Vec4(t));
        }
        // **A COR que já está lá** — ver [`CH_HUE`]. A instância entra em HSV, UM componente
        // é dirigido, e ela volta; o alfa atravessa intocado (quem o dirige é o `CH_OPACITY`).
        //
        // ⚠️ **O matiz NÃO é envolvido aqui, de propósito:** `hsv_to_rgba` já faz
        // `rem_euclid(1.0)` na entrada, então envolver antes seria a segunda resposta à mesma
        // pergunta — e a errada, porque o `blend` tem de ver o valor CONTÍNUO. Com `Add` (o
        // default, e o *Master Hue* da referência) o blend vira `orig + delta·f`, ou seja
        // *escale o DESLOCAMENTO*, que não tem ambiguidade de arco nenhuma.
        //
        // ⚠️ **A saturação é clampada porque é ESTRUTURAL, não gosto:** com `s > 1` o
        // `hsv_to_rgba` calcula `p = v·(1−s)` e devolve canais NEGATIVOS. O valor tem só o
        // piso em zero — um teto aqui seria uma regra que os outros produtores desta coluna
        // não obedecem (a interpolação `Cardinal` da rampa passa de 1 por construção, e o
        // doc dela diz isso).
        CH_HUE | CH_SAT | CH_VAL => {
            let mut t = base_vec4(input, "tint", n, [1.0, 1.0, 1.0, 1.0]);
            for (i, ti) in t.iter_mut().enumerate() {
                let (h, s, v) = ph2d_color::rgb_to_hsv(*ti);
                let f = falloff_at(input, i);
                let cur = match channel {
                    CH_HUE => h,
                    CH_SAT => s,
                    _ => v,
                };
                let driven = mode.apply(cur, value_at(vals, i) * scale);
                let next = blend(cur, driven, f);
                let (h, s, v) = match channel {
                    CH_HUE => (next, s, v),
                    CH_SAT => (h, next.clamp(0.0, 1.0), v), // CLAMP-OK: s>1 => canais negativos
                    _ => (h, s, next.max(0.0)),
                };
                *ti = ph2d_color::hsv_to_rgba(h, s, v, ti[3]);
            }
            out.set("tint", Column::Vec4(t));
        }
        // **The mask itself** — see [`CH_FALLOFF`]. No blend (nothing sensible masks the
        // writing of the mask) and no clamp (a negative weight inverts the force downstream).
        CH_FALLOFF => {
            let mut f = base_scalar(input, "falloff", n, 1.0);
            for (i, fi) in f.iter_mut().enumerate() {
                *fi = mode.apply(*fi, value_at(vals, i) * scale);
            }
            out.set("falloff", Column::Scalar(f));
        }
        // **UM eixo do tamanho** ([`CH_SIZE_X`]) — o espelho EXACTO do braço `0 | 1` do
        // `P`: a mesma lei, a mesma mistura, a mesma identidade unitária do braço `Size`
        // logo abaixo; o que muda é **qual componente** recebe.
        CH_SIZE_X | CH_SIZE_Y => {
            let comp = (channel - CH_SIZE_X) as usize; // 0 = X, 1 = Y
            let mut s = base_vec2(input, "size", n, [1.0, 1.0]);
            for (i, si) in s.iter_mut().enumerate() {
                let driven = mode.apply(si[comp], value_at(vals, i) * scale);
                si[comp] = blend(si[comp], driven, falloff_at(input, i));
            }
            out.set("size", Column::Vec2(s));
        }
        _ => {
            let mut s = base_vec2(input, "size", n, [1.0, 1.0]);
            for (i, si) in s.iter_mut().enumerate() {
                let f = falloff_at(input, i);
                let v = value_at(vals, i) * scale;
                si[0] = blend(si[0], mode.apply(si[0], v), f);
                si[1] = blend(si[1], mode.apply(si[1], v), f);
            }
            out.set("size", Column::Vec2(s));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Todo canal que o nó ROTEIA tem um chip no menu.**
    ///
    /// ⚠️ **Achado por mutação em 2026-08-18:** apagar um rótulo da lista deixa a suíte
    /// inteira VERDE e o canal do fim **inalcançável** — ele continua a existir no
    /// `channel_column`, no braço da CPU e no `variant_by_param`, e nenhuma superfície o
    /// oferece. ⚠️ E o `max` do hint **não** é quem o guarda: medido, a row de enum do
    /// painel clampa por `labels.len()` e **ignora `min`/`max`**, então para um enum
    /// aquela faixa é decorativa — um gate sobre ela pinaria uma lei que o produto não
    /// consome (foi escrito, medido e descartado, e ele acusava três nós corretos).
    ///
    /// A régua é o **ÚLTIMO índice implementado**, o único número que cresce quando um
    /// canal nasce.
    #[test]
    fn every_channel_the_node_routes_has_a_chip_in_the_menu() {
        let hint = crate::PARAM_HINTS
            .iter()
            .find(|h| h.param == "channel")
            .expect("o canal tem hint");
        let ph2d_node_registry::ParamWidget::Enum { labels } = hint.widget else {
            panic!("o canal e' um enum");
        };
        // CONTROLE: a varredura achou a lista do CANAL, e nao uma vazia.
        assert!(labels.contains(&"Custom…"), "os rotulos sao os do canal");
        assert_eq!(
            labels.len() as i32,
            CH_SIZE_Y + 1,
            "a lista de chips tem de cobrir 0..={}; ela oferece {}",
            CH_SIZE_Y,
            labels.len()
        );
    }

    fn sizes(out: &Stream) -> Vec<[f32; 2]> {
        match out.get("size").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("size e' Vec2"),
        }
    }

    /// Uma fileira com tamanhos NÃO-uniformes e NÃO-unitários de propósito: com
    /// `[1,1]` o eixo intocado seria indistinguível da identidade que o
    /// `base_vec2` inventa, e com `x == y` a troca de eixo passaria.
    fn sized_row() -> Stream {
        Stream::new(3).with(
            "size",
            Column::Vec2(vec![[0.5, 2.0], [1.5, 0.25], [3.0, 0.75]]),
        )
    }

    /// **Dirigir UM eixo deixa o outro exactamente onde estava.**
    #[test]
    fn driving_one_size_axis_leaves_the_other_alone() {
        let input = sized_row();
        let before = sizes(&input);

        let x = sizes(&drive_channel(&input, CH_SIZE_X, &[7.0], 1.0, Combine::Set));
        for (i, s) in x.iter().enumerate() {
            assert_eq!(s[0], 7.0, "o eixo X recebe (elemento {i})");
            assert_eq!(
                s[1].to_bits(),
                before[i][1].to_bits(),
                "o eixo Y fica AO BIT (elemento {i})"
            );
        }

        let y = sizes(&drive_channel(&input, CH_SIZE_Y, &[7.0], 1.0, Combine::Set));
        for (i, s) in y.iter().enumerate() {
            assert_eq!(s[1], 7.0, "o eixo Y recebe (elemento {i})");
            assert_eq!(
                s[0].to_bits(),
                before[i][0].to_bits(),
                "o eixo X fica AO BIT (elemento {i})"
            );
        }
    }

    /// **O canal `Size` de sempre continua a escrever os DOIS**, ao bit.
    ///
    /// ⚠️ Este é o CONTROLE que impede a wave de mudar o mundo que já shipava: sem ele,
    /// um braço novo que roubasse o `Size` passaria pelos gates dos eixos.
    #[test]
    fn the_uniform_size_channel_still_writes_both_axes() {
        let out = sizes(&drive_channel(&sized_row(), 3, &[7.0], 1.0, Combine::Set));
        for (i, s) in out.iter().enumerate() {
            assert_eq!((s[0], s[1]), (7.0, 7.0), "os dois eixos (elemento {i})");
        }
    }

    /// **DOIS campos independentes, um por eixo — o que a composição NÃO dava.**
    ///
    /// Medido antes de construir (`measure_size_axes`): `drive(Size) → motion.scale`
    /// dá anisotropia com razão FIXA (o `motion.scale` escala com um PARAM, igual para
    /// toda peça) e o `Custom…` **recusa** escrever um escalar sobre um `Vec2`. Aqui
    /// os dois eixos recebem campos que **não são múltiplos um do outro**, e nenhuma
    /// razão constante reproduz isso.
    #[test]
    fn two_independent_fields_can_drive_the_two_axes() {
        let input = Stream::new(3).with("size", Column::Vec2(vec![[1.0, 1.0]; 3]));
        let step1 = drive_channel(&input, CH_SIZE_X, &[1.0, 2.0, 3.0], 1.0, Combine::Set);
        let out = sizes(&drive_channel(
            &step1,
            CH_SIZE_Y,
            &[3.0, 1.0, 2.0],
            1.0,
            Combine::Set,
        ));
        assert_eq!(out, vec![[1.0, 3.0], [2.0, 1.0], [3.0, 2.0]]);
        // O discriminante: a razão x/y MUDA de peça para peça. Uma anisotropia fixa
        // (o que o `motion.scale` dá) teria a MESMA razão nas três.
        let r: Vec<f32> = out.iter().map(|s| s[0] / s[1]).collect();
        assert!(
            (r[0] - r[1]).abs() > 0.1 && (r[1] - r[2]).abs() > 0.1,
            "as razoes tem de diferir entre pecas, e medem {r:?}"
        );
    }

    /// **A máscara alcança o eixo** — `falloff = 0` deixa o tamanho onde estava, ao bit.
    #[test]
    fn a_masked_element_keeps_its_size_axis() {
        let input = Stream::new(2)
            .with("size", Column::Vec2(vec![[0.5, 2.0], [1.5, 0.25]]))
            .with("falloff", Column::Scalar(vec![1.0, 0.0]));
        let out = sizes(&drive_channel(&input, CH_SIZE_X, &[9.0], 1.0, Combine::Set));
        assert_eq!(out[0][0], 9.0, "falloff 1 leva o drive inteiro");
        assert_eq!(
            out[1][0].to_bits(),
            1.5f32.to_bits(),
            "falloff 0 nao deixa o drive tocar o eixo"
        );
    }

    #[test]
    fn a_length_one_value_broadcasts_to_every_instance() {
        // Three instances, ONE value (2.0) → all three shift by 2 in X.
        let input =
            Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]));
        let out = drive_channel(&input, 0, &[2.0], 1.0, Combine::Add);
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[2.0, 0.0], [3.0, 0.0], [4.0, 0.0]]),
            _ => panic!(),
        }
    }

    #[test]
    fn a_length_n_value_applies_element_wise() {
        let input = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3]));
        let out = drive_channel(&input, 0, &[1.0, 2.0, 3.0], 1.0, Combine::Add);
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
            _ => panic!(),
        }
    }

    #[test]
    fn set_and_multiply_combine_against_the_existing_channel() {
        let input = Stream::new(1).with("P", Column::Vec2(vec![[5.0, 0.0]]));
        let set = drive_channel(&input, 0, &[2.0], 1.0, Combine::Set);
        let mul = drive_channel(&input, 0, &[2.0], 1.0, Combine::Multiply);
        assert_eq!(px(&set), 2.0, "set overwrites");
        assert_eq!(px(&mul), 10.0, "multiply scales the existing 5");
    }

    #[test]
    fn falloff_zero_leaves_the_channel_untouched() {
        let input = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]))
            .with("falloff", Column::Scalar(vec![1.0, 0.0]));
        let out = drive_channel(&input, 0, &[3.0], 1.0, Combine::Add);
        match out.get("P").unwrap() {
            Column::Vec2(v) => {
                assert_eq!(v[0], [3.0, 0.0], "focused instance driven");
                assert_eq!(v[1], [0.0, 0.0], "masked instance untouched");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn size_channel_drives_both_components_from_unit_identity() {
        // A bare P-only stream driven on Size (multiply by 2) → unit×2 on both.
        let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
        let out = drive_channel(&input, 3, &[2.0], 1.0, Combine::Multiply);
        match out.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [2.0, 2.0], "unit identity × 2"),
            _ => panic!(),
        }
    }

    fn px(s: &Stream) -> f32 {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v[0][0],
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod falloff_tests {
    use super::*;

    /// **A computed number becomes a MASK.** The wall five families hit at once (doc 89 §10.0):
    /// the `falloff` column is what the `field.*`, the `force.*`, the deformers, the transforms
    /// and the stylistics all read, and it could only ever be DERIVED FROM GEOMETRY. Nothing
    /// computed — a noise, a luma, an age — could become one.
    #[test]
    fn a_value_field_becomes_the_mask_itself() {
        let input = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3]));
        let out = drive_channel(&input, CH_FALLOFF, &[0.25, 0.5, 1.0], 1.0, Combine::Set);
        match out
            .get("falloff")
            .expect("the mask is now a column anyone can write")
        {
            Column::Scalar(v) => assert_eq!(v, &vec![0.25, 0.5, 1.0]),
            _ => panic!("the mask is a scalar column"),
        }
    }

    /// **An absent mask is `1.0`, not `0.0`** — every reader in the library falls back to full
    /// effect (`falloff_at`), so a writer that started from zero would disagree with all of them
    /// about what "no mask" means, and `Add` would silently halve the world.
    #[test]
    fn an_absent_mask_starts_at_full_effect() {
        let input = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0]; 2]));
        let out = drive_channel(&input, CH_FALLOFF, &[0.0], 1.0, Combine::Add);
        match out.get("falloff").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![1.0, 1.0], "1.0 + 0.0, not 0.0 + 0.0"),
            _ => panic!(),
        }
    }

    /// **The mask does not mask ITSELF.** Every other channel lerps its result toward the
    /// original by `falloff`; doing that here would make `Set` non-idempotent and would mean
    /// nothing to an artist. ⚠️ On the device this is not a choice at all — a variant whose
    /// target is `falloff` binds it once as `ReadWrite`, so the common read is absent and the
    /// self-mask is *inexpressible*.
    #[test]
    fn the_mask_does_not_mask_itself() {
        let input = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0]; 2]))
            .with("falloff", Column::Scalar(vec![0.0, 0.5]));
        let out = drive_channel(&input, CH_FALLOFF, &[1.0], 1.0, Combine::Set);
        match out.get("falloff").unwrap() {
            // Self-masking would have pinned the first element at 0.0 forever — a mask you
            // could never turn back on.
            Column::Scalar(v) => assert_eq!(v, &vec![1.0, 1.0]),
            _ => panic!(),
        }
    }

    /// **A NEGATIVE weight survives** — and it is not an oversight. A negative `falloff` inverts
    /// the force that consumes it, which the conference found is ours alone (C4D and Cavalry are
    /// `[0,1]` by construction). Clamping here would delete the capability before anyone used it.
    #[test]
    fn a_negative_mask_is_not_clamped_away() {
        let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
        let out = drive_channel(&input, CH_FALLOFF, &[-1.0], 1.0, Combine::Set);
        match out.get("falloff").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![-1.0]),
            _ => panic!(),
        }
    }

    /// **The five channels that shipped are untouched** — the new one is a sixth index, so
    /// already-authored art cannot move.
    #[test]
    fn the_channels_that_shipped_are_untouched() {
        let input = Stream::new(1).with("P", Column::Vec2(vec![[3.0, 7.0]]));
        let out = drive_channel(&input, 0, &[1.0], 1.0, Combine::Add);
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[4.0, 7.0]]),
            _ => panic!(),
        }
        assert!(out.get("falloff").is_none(), "driving X invents no mask");
    }
}
