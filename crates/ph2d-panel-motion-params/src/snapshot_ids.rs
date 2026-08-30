//! **Quantos widgets uma row tem, e como eles se CHAMAM** — os tetos e a derivação de id do
//! painel de params (irmão de `snapshot.rs`, que responde a outra pergunta: *o que uma row É*).
//!
//! O corte é por responsabilidade e nasceu de um teto de LOC: os tipos de row crescem quando o
//! vocabulário de params cresce, e os ids crescem quando um EDITOR novo chega (curva, gradiente,
//! paleta) — duas taxas diferentes no mesmo arquivo.
//!
//! ⚠️ Todo id sai de `fnv_id(&format!(…))`, nunca de um array — é por isso que um slot a mais
//! custa uma string e o teto de linhas é sobre o `populate`, não sobre o espaço de ids.

use ph2d_a11y::NodeId;

/// Max param rows the pooled slider/chip widgets support.
///
/// ⚠️ **É um teto de RECURSO, e o recurso é o `populate`:** ele registra
/// **`7 + 2 × MAX_ENUM_OPTIONS` widgets por slot** — sete avulsos (slider · chip · number ·
/// reroll · **reset** · text · checkbox) e as duas fileiras de botão de opção — de uma vez,
/// para todos os slots, então o número multiplica direto. Não é o espaço de ids — eles saem
/// de `fnv_id(&format!(…))`, e um slot a mais custa uma string a mais.
///
/// ⚠️ **A conta é escrita como FÓRMULA porque o literal já apodreceu:** ela dizia *"21
/// widgets (… 2×8 botões de enum)"* e a lista tinha **seis** avulsos — o `reset` chegou
/// depois e ninguém recontou —, então o número certo naquele dia era 23 e o teto de opções
/// já não era 8 quando isto foi lido.
///
/// ⚠️ **E ele é MEDIDO, não escolhido** (§0). O valor anterior era **8**, com a justificativa
/// *"grid/transform/clone têm 3; o teto cobre os nós de fan-out"* — verdade quando foi escrita
/// e **falsa hoje**: o censo (`the_panel_shows_every_param_of_every_node`, na shell) mede o
/// pior nó do registry, e o `field.remap` sozinho declara **12** `ParamSpec`. Acima do teto o
/// `.take()` do `paint_rows` **não desenha nada e não registra nada** — o param existe no
/// modelo, o cook o lê, e o artista não tem como alcançá-lo: o modo de falha SILENCIOSO que
/// as quatro condições de UI existem para proibir.
///
/// O número é o pior caso medido mais folga de uma família; o gate é quem o mantém honesto —
/// o nó que passar deste teto deixa a suíte VERMELHA em vez de esconder um botão.
///
/// ⚠️ **`16 → 20` em 2026-08-19**, e a segunda vez que este teto é movido por medição e não
/// por gosto. O `motion.emitter` ganhou o `probability` (doc 89 folha 01) e passou a **17**
/// linhas — o gate reprovou com o nome do nó e a contagem, antes de o defeito chegar ao ecrã.
/// O censo do dia, do próprio gate: `motion.emitter` **17** · `motion.spline_wrap` 14 ·
/// `field.remap` / `value.noise` / `motion.boids` 13 · `motion.noise` 12. Os `20` são o pior
/// caso mais três de folga, e a metade oposta do gate (`teto ≤ 2 × pior`) mantém a folga
/// honesta: `20 ≤ 34`. **O preço são 4 slots × 21 registros = 84 registros a mais por
/// `populate`** — e ⚠️ ele MULTIPLICA com o [`MAX_ENUM_OPTIONS`] (48) nos slots que pintam um
/// selector, que é o que impede este número de ser «só mais um pouco».
///
/// ⚠️ **`20 → 24` em 2026-08-23, a TERCEIRA vez por medição.** O `motion.bezier_warp` (doc 89
/// folha 04, P1) declara **24** params — 4 cantos + 8 tangentes, cada um `(x, y)`, que é a
/// superfície do *Bezier Warp* da referência e não inchaço: um lado sem as duas tangentes
/// deixa de ser uma cúbica. O gate reprovou nomeando o nó e a contagem, antes de o defeito
/// chegar ao ecrã. Censo do dia: `motion.bezier_warp` **24** · `motion.spline_wrap` /
/// `motion.emitter` 15 · `motion.boids` 14 · `motion.noise` 13.
///
/// ⚠️ **E a FOLGA foi retirada de propósito** — os movimentos anteriores davam *pior + 3*, e
/// este dá **exactamente o pior**. A razão é o parágrafo acima: cada slot custa 21 registros
/// **e multiplica com o [`MAX_ENUM_OPTIONS`]** (48) nos slots que pintam selector, ou seja
/// `2 × 48` botões a mais por slot de folga. Com o pior caso a saltar de 17 para 24, três
/// slots de folga deixaram de ser arredondamento. *Um teto colado no pior caso faz o gate
/// avisar no dia seguinte ao nó crescer — que é para isso que ele existe.*
///
/// ⚠️ **`24 → 25` em 2026-08-30, e o gate avisou no dia seguinte, como prometido.** O
/// `source.lsystem` ganhou as três letras que plantam um objecto (`Leaf (J)`/`(K)`/`(M)`) e
/// passou a 25 no pior caso (guiado + `Branches` + molde `Custom`). O preço, medido pela mesma
/// porta e com o mesmo método do movimento anterior (`MockPanelHost::with_panel`, 200
/// construções, `--release`, TRÊS leituras por lado porque uma leitura abaixo da linha de base é
/// ruído e não ganho):
///
/// | `MAX_PARAM_ROWS` | leituras (µs/construção) |
/// |---|---|
/// | 24 | 186,8 · 188,9 · 180,9 |
/// | 25 | 205,0 · 198,1 · 198,3 |
///
/// ⚠️ **`25 → 30` em 2026-08-30, a QUARTA vez por medição.** O `source.lsystem` ganhou os cinco
/// controlos de folha que o 2.º smoke pediu (nível mínimo · ângulo · abertura · fracção à frente
/// · efeitos) e o pior caso VISÍVEL dele passou a **30** (guiado + `Branches` + molde `Custom`).
/// Medido pela mesma porta, agora com `500` construções e o **MÍNIMO de 5 leituras** — a
/// workstation estava a `load 15+` e uma média teria medido a carga:
///
/// | `MAX_PARAM_ROWS` | mínimo (µs/construção) |
/// |---|---|
/// | 25 | 202,8 |
/// | **30** | 253,2 |
///
/// ⇒ **~10 µs por slot**, o mesmo número que a medição de 2026-08-23 deu (~11). A medição
/// reproduz-se com `cargo test -p ph2d-panel-motion-params --release measure_screen_build --
/// --ignored --nocapture` (ver [`crate::measure_row_cap`]).
///
/// ⇒ **~11 µs por slot**, uma vez por construção de tela e **não por quadro** — a mesma unidade
/// em que o salto do [`MAX_ENUM_OPTIONS`] foi aceite (22,0 → 107,2 µs).
///
/// ⚠️ **E continua colado ao pior caso**: `25` é exactamente o que o pior nó pede hoje, sem
/// folga, pela razão do parágrafo acima.
pub const MAX_PARAM_ROWS: usize = 30;

/// Max named options a single segmented selector paints — an `Enum` row's `labels`, a
/// `Channels` row's channels **plus its trailing "Custom…"**, and the live-column chips.
///
/// ⚠️ **É um teto de RECURSO, e o recurso é o mesmo `populate` do [`MAX_PARAM_ROWS`]:** ele
/// registra `2 × MAX_ENUM_OPTIONS` botões por slot (a fileira de segmentos + a de chips), e
/// os dois tetos se MULTIPLICAM — subir este custa `2 × MAX_PARAM_ROWS` registros por opção.
///
/// ⚠️ **E acima dele a falha é SILENCIOSA, exatamente como a do teto de linhas:** o
/// `.min(MAX_ENUM_OPTIONS)` do `rows_paint_kinds` não desenha nem registra a opção excedente
/// — o valor existe no modelo, o cook o lê, e o artista não tem gesto que o alcance. O censo
/// que o mantém honesto é o `the_panel_paints_every_option_of_every_selector` (na shell, ao
/// lado do censo de linhas): nenhum gate por-nó o veria, porque cada um usa a fixture do seu
/// próprio nó.
///
/// ⚠️ **E ele é MEDIDO, não escolhido** (§0). O valor anterior era **8**, com a justificativa
/// *"cobre os conjuntos de canal / onda / easing com folga"* — uma frase sobre o que ele
/// cobria naquele dia, não sobre de que RECURSO ele é, e **falsa no dia em que foi lida**: o
/// censo mede o `source.shape`, que declara as **43** formas do catálogo fillável, e o painel
/// pintava **8**. Trinta e cinco formas existiam no modelo, o cook as cozinhava, e o artista
/// não tinha gesto que as alcançasse — com o gate do próprio nó (`KIND_LABELS.len() == 43`)
/// **verde**, porque ele pergunta sobre o NÓ e não sobre o painel.
///
/// O número é o pior caso medido (**43**) mais folga de uma família, arredondado ao múltiplo
/// da grade de 4 colunas do seletor. O preço, medido pela porta do painel
/// (`MockPanelHost::with_panel`, 200 construções): o `populate` sai de **22,0 µs** para
/// **107,2 µs** — uma vez por construção de tela, não por quadro.
///
/// ⚠️ **Uma lista de 43 num seletor SEGMENTADO é o widget errado a longo prazo** — o painel
/// já quebra em 4 colunas e o corpo dele rola, então a lista é *alcançável*, que é
/// estritamente melhor que 35 opções invisíveis; o widget certo para lista longa é um
/// dropdown (`widget/dropdown` já existe), e isso é `ParamWidget` novo, não uma constante.
pub const MAX_ENUM_OPTIONS: usize = 48;

/// Option-id base for a [`ChannelsRow`]'s live-column chips (the Custom picker).
///
/// ⚠️ **DERIVADO do teto, nunca um literal ao lado dele.** Um segmento curado usa
/// `param_enum_id(slot, opt)` com `opt < MAX_ENUM_OPTIONS`, então a base tem de começar
/// exatamente onde eles acabam. Escrito como número solto (era **32**, "bem acima do teto")
/// ele vira uma afirmação que **envelhece em silêncio**: subir o teto para 48 o faria mentir,
/// e a colisão só apareceria num picker com mais de 32 canais — ids iguais, dois widgets,
/// nenhum erro.
pub(crate) const CHANNELS_EXTRA_BASE: usize = MAX_ENUM_OPTIONS;

/// Max control points a single Curve row's editor supports (matches the field.remap
/// text param's practical ceiling; a handful of points shape any transfer). The
/// per-point `CurvePoint` widgets are pooled positionally like the enum options.
pub(crate) const MAX_CURVE_POINTS: usize = 8;

/// **Máximo de paradas que o editor de gradiente oferece — MEDIDO** (doc 85; bloco Z, doc 91).
///
/// O modelo (`ph2d_color::MAX_RAMP_STOPS`) admite **32**; o `+` recusa acima daqui.
///
/// ⚠️ **O número estava certo e a RAZÃO não existia**, que é o defeito que a folha 09 da
/// conferência acusou: dizia-se *"o painel é estreito e a faixa tem de ficar legível"* — uma
/// frase, não um recurso (`CLAUDE.md` §0.0). A derivação é esta, e o gate
/// `the_gradient_stop_ceiling_is_the_narrowest_panel_divided_by_a_pointer_target` refá-la a cada
/// corrida:
///
/// | grandeza | de onde vem | px |
/// |---|---|---|
/// | painel mais estreito | `ph2d_tokens::PANEL_MIN_W_PX` (o piso do arrasto de redimensionar) | 220 |
/// | recuo, dos dois lados | `ph2d_tokens::PANEL_HEAD_PAD_PX` × 2 | 36 |
/// | **faixa útil** | | **184** |
/// | alvo de ponteiro | `GRAB_R × 2` — a caixa de agarrar que este mesmo editor declara | 18 |
/// | folga da célula | `pad × 2` do strip de amostras | 4 |
/// | **por parada** | | **22** |
///
/// `184 / 22 = 8,36` ⇒ **8**.
///
/// ⚠️ **O recurso não é a legibilidade, é o ALVO DE PONTEIRO** — e a distinção decide o número.
/// Uma amostra de 14 px lê-se perfeitamente; o que ela deixa de ser é *clicável*, e cada amostra
/// abre o seletor de cor. A régua é a própria caixa de agarrar que este editor já declara para
/// os marcadores: uma amostra mais estreita que o alvo dos marcadores ao lado dela é um alvo
/// que a lei da casa já chama de pequeno demais.
///
/// ⚠️ **Contra o painel MAIS ESTREITO, não contra o de hoje**: um teto que só vale na largura
/// confortável parte-se quando o artista aperta a janela — a lei do pior caso que o
/// `motion.spring` já aplica ao relógio.
///
/// Os marcadores por-parada são registados a cada pintura, como as alças do Curve.
pub(crate) const MAX_GRADIENT_STOPS: usize = 8;

/// Stable widget id for the `slot`-th param row's slider (pooled, positional —
/// row `i` of whichever node is selected uses slot `i`).
pub(crate) fn param_slider_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/slider/{slot}"))
}

/// Stable widget id for the `slot`-th param row's numeric chip.
pub(crate) fn param_chip_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/chip/{slot}"))
}

/// Stable widget id for a colour-swatch row, keyed by its **anchor channel**
/// param name (unique within a node) — NOT positional like the slider/chip pool,
/// so the shell bridge computes the same id from the node's hints without
/// agreeing on row order. `pub` for the bridge's picker read-back / seeding.
pub fn param_swatch_id(anchor: &str) -> NodeId {
    fnv_id(&format!("motion_param/swatch/{anchor}"))
}

/// Stable widget id for the `slot`-th param row's checkbox (Toggle rows).
pub(crate) fn param_checkbox_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/check/{slot}"))
}

/// Stable widget id for option `opt` of the `slot`-th param row's segmented
/// selector (Enum rows).
pub(crate) fn param_enum_id(slot: usize, opt: usize) -> NodeId {
    fnv_id(&format!("motion_param/enum/{slot}/{opt}"))
}

/// Stable widget id for the `slot`-th param row's standalone numeric box — the
/// app-standard `NumberInput` (Angle + Seed rows; Scalar rows use the slider's
/// own chip instead).
pub(crate) fn param_number_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/number/{slot}"))
}

/// Stable widget id for the `slot`-th Seed row's re-roll button.
pub(crate) fn param_reroll_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/reroll/{slot}"))
}

/// Stable widget id for the `slot`-th Text row's `TextInput` field (formula editor).
///
/// ⚠️ **A [`crate::FileRow`] reuses it for its path field** — deliberately, and it is the
/// same reuse a `Channels` row's *Custom…* field and a `Source` row's field already make:
/// one text field per slot, whoever the row is. `on_text_commit` is the single place that
/// decides which param the buffer belongs to, so a fourth reader is a `match` arm rather
/// than a second pool nobody registers.
pub(crate) fn param_text_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/text/{slot}"))
}

/// The `slot`-th File row's **Browse…** button — the one that opens the OS dialog.
pub(crate) fn param_file_browse_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/file/{slot}/browse"))
}

/// The `slot`-th Curve row's **editor parent** id — the `CurvePoint.parent` every
/// handle carries, so `apply_event` routes the drained drag to the right row (the
/// dispatch emits `ValueChanged(parent)` on a handle drag).
pub(crate) fn param_curve_editor_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}"))
}

/// The `slot`-th Curve row's `point`-th draggable control-point handle.
pub(crate) fn param_curve_point_id(slot: usize, point: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/pt/{point}"))
}

/// The `slot`-th Curve row's **add-point** button.
pub(crate) fn param_curve_add_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/add"))
}

/// The `slot`-th Curve row's **remove-point** button.
pub(crate) fn param_curve_remove_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/remove"))
}

/// The `slot`-th Curve row's **interp** button — cycles the selected point's
/// segment interpolation (Linear → Smooth → Hold).
pub(crate) fn param_curve_interp_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/interp"))
}

/// The `slot`-th Gradient row's **editor parent** id — the `CurvePoint.parent` every
/// position marker carries, so `apply_event` routes the drained drag to the right row.
pub(crate) fn param_grad_editor_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}"))
}

/// The `slot`-th Gradient row's `stop`-th draggable position marker.
pub(crate) fn param_grad_stop_id(slot: usize, stop: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/stop/{stop}"))
}

/// Stable widget id for a Gradient row's `stop`-th colour swatch, keyed by the text-param
/// **name** + index (NOT positional) — so the shell bridge computes the same id from the
/// node's hints to seed the swatch colour and read the OKLCH pick back into the string.
/// `pub` for the bridge, exactly like [`param_swatch_id`].
pub fn param_grad_swatch_id(name: &str, stop: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad_swatch/{name}/{stop}"))
}

/// Stable widget id for a Palette row's `i`-th colour swatch, keyed by the text-param
/// **name** + index (NOT positional), so the shell bridge computes the same id to seed the
/// swatch and read the OKLCH pick back. `pub` for the bridge, exactly like
/// [`param_grad_swatch_id`].
///
/// ⚠️ **Derived from a string, so there is no pool and no cap** — the 200th swatch has an
/// id as readily as the 2nd, which is what lets the row have no length limit.
pub fn param_pal_swatch_id(name: &str, i: usize) -> NodeId {
    fnv_id(&format!("motion_param/pal_swatch/{name}/{i}"))
}

/// The `slot`-th Palette row's **add-colour** button.
pub(crate) fn param_pal_add_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/pal/{slot}/add"))
}

/// The `slot`-th Palette row's **remove-colour** button (drops the LAST colour).
pub(crate) fn param_pal_remove_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/pal/{slot}/remove"))
}

/// The `slot`-th Gradient row's **add-stop** button.
pub(crate) fn param_grad_add_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/add"))
}

/// The `slot`-th Gradient row's **remove-stop** button.
pub(crate) fn param_grad_remove_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/remove"))
}

/// The `slot`-th Gradient row's **interp** button — cycles the ramp's global
/// interpolation (Linear → Ease → Constant → Cardinal → B-Spline).
pub(crate) fn param_grad_interp_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/interp"))
}

/// O botão de **ESPAÇO** da `slot`-ésima row de Gradiente — cicla RGB → HSV → HSL, o
/// espaço em que dois stops vizinhos são interpolados (`RampColorMode`).
pub(crate) fn param_grad_space_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/space"))
}

/// O botão de **MATIZ** da `slot`-ésima row de Gradiente — cicla Near → Far → CW → CCW.
///
/// ⚠️ Ele só é PINTADO fora do RGB, onde o caminho de matiz de fato decide alguma coisa
/// (o braço `Rgb` do `mix2` nunca chama `lerp_hue`); em RGB seria um botão que gira e não
/// muda um pixel. O id existe sempre — quem decide é o pintor.
pub(crate) fn param_grad_hue_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/hue"))
}

/// The `slot`-th Gradient row's `p`-th **preset seed** chip (Rainbow / Heat / Ice /
/// Grayscale) — clicking it LOADS that preset's stops into the editable ramp (doc 85).
pub(crate) fn param_grad_preset_id(slot: usize, p: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/preset/{p}"))
}

/// FNV-1a-64 of `key` (same scheme as the graph panel's dynamic hit ids).
fn fnv_id(key: &str) -> NodeId {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    NodeId(h)
}

/// Id do botão de **reverter ao default** da `slot`-ésima row (pooled, posicional como o
/// slider/chip). Existe só nas rows que carregam override — um botão que sempre aparece e às
/// vezes não faz nada é o botão-morto que este codebase persegue.
pub(crate) fn param_reset_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/reset/{slot}"))
}
