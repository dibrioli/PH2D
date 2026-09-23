//! **How a render sink DRAWS** — o estilo do sink (doc 89, folha 17).
//!
//! Split out of `lib.rs` at the HR-18 LOC cap along the seam that was already
//! there: `lib.rs` runs the CLOCK (`MotionCookPump`) and `lower.rs` answers *what
//! does a cooked stream look like on screen*; this file answers the one question
//! that is neither — *in what STYLE does this sink draw?* — and is the door both
//! render routes ask.
//!
//! ⭐ E desde o doc 115 §16 ele responde a segunda metade da MESMA pergunta — *o que o sink
//! DESENHA* (`o_que_o_sink_desenha`), que é a corrente já com o passe do fim aplicado. As duas
//! moram juntas porque as duas se lêem dos params `collide` do sink, e porque o `lib.rs` estava
//! no tecto de LOC: o corte é pela responsabilidade que já estava escrita neste cabeçalho.
//!
//! The reference is unanimous and it decided the shape: Niagara puts blend on the
//! Sprite Renderer's material, Cavalry on the layer/shader, AE and Stardust on the
//! layer. Blend belongs to the RENDERER, not to a particle — so it is a param of
//! `motion.output` and a scalar of the lowering, never a per-element column.
//!
//! ⚠️ **E a MESMA leitura decide os outros três** (pivô · filtro · ordem): as três
//! referências põem cada um deles no renderer / no material / na camada, nunca na
//! partícula. É por isso que este ficheiro devolve um [`SinkStyle`] e não quatro
//! respostas soltas — *quatro perguntas com a mesma resposta estrutural são uma
//! pergunta*, e um segundo leitor de qualquer uma delas seria livre de arredondar
//! diferente.
//!
//! ⚠️ **The tag cannot travel as a stream column, and the reason is structural.**
//! On the device `motion.output` is `GpuKernel::PASSTHROUGH`: the sequencer emits
//! no pass for it, so anything its `eval` wrote would never reach the device
//! lowering. Both lowerings take the style as an argument instead — the CPU pump
//! asks this door inside its sink loop, the shell asks it for the single sink the
//! GPU route accepts, and `ph2d-gpu-cook` receives it (that crate keeps
//! `ph2d-eval-motion` a DEV dependency on purpose, so [`SinkStyle`] lives in
//! `ph2d-render` — the crate that owns the struct every field of it belongs to).

use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_render::{RenderInstance, SinkStyle};

/// The sink param that names a render sink's blend mode.
///
/// It is the SAME string `ph2d-node-motion-output` declares as `BLEND_PARAM`.
/// Neither crate may depend on the other (both are leaves of the node system), so
/// the agreement is pinned by a gate in the shell — the one place that sees both.
/// This substrate already knows a vocabulary of well-known NAMES (`"P"`, `"size"`,
/// `"rot"`, `"tint"`, `"uv_rect"`, `"texture_id"`, `"geometry_id"`); this joins it
/// as the first well-known *param* name rather than column.
pub const SINK_BLEND_PARAM: &str = "blend";

/// Os outros três params do sink, com os mesmos nomes que
/// `ph2d-node-motion-output` declara (e o mesmo gate da shell a pinar que
/// concordam — nenhuma das duas folhas alcança a outra).
pub const SINK_PIVOT_X_PARAM: &str = "pivot_x";
/// Ver [`SINK_PIVOT_X_PARAM`].
pub const SINK_PIVOT_Y_PARAM: &str = "pivot_y";
/// Ver [`SINK_PIVOT_X_PARAM`].
pub const SINK_FILTER_PARAM: &str = "filter";
/// Ver [`SINK_PIVOT_X_PARAM`].
pub const SINK_SORT_PARAM: &str = "sort";

/// ⭐⭐⭐ **COM QUEM a mistura acontece** — o mesmo nome que `ph2d-node-motion-output` declara como
/// `BLEND_WITH_PARAM` (o gate da shell pina-o). A lei e as três arrumações estão no doc 118 §1.
pub const SINK_BLEND_WITH_PARAM: &str = "blend_with";

/// ⭐⭐⭐ **O ALCANCE da mistura de um sink** — ver [`SINK_BLEND_WITH_PARAM`].
///
/// ⚠️ **Ele NÃO vive no [`SinkStyle`], e o motivo é de PASSE:** o estilo é o que o LOWERING lê
/// (uma instância de cada vez), e o alcance é uma decisão sobre o GRUPO — quem o lê é o passe que
/// arruma as camadas, depois de todas as cópias existirem. Pô-lo no estilo dava ao lowering um campo
/// que ele não tem como honrar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BlendWith {
    /// Cada cópia com TUDO o que está por baixo — o que as imagens sempre fizeram (tag `0`).
    #[default]
    Everything,
    /// Só entre as cópias; o grupo pousa sobre o cenário em `Normal` (tag `1`).
    Copies,
    /// As cópias juntam-se em `Normal`, e o GRUPO mistura-se com o cenário (tag `2`).
    Scene,
}

impl BlendWith {
    /// Os três, na ordem dos tags — a ordem é o contrato do `.ph2dproj`.
    pub const ALL: [BlendWith; 3] = [BlendWith::Everything, BlendWith::Copies, BlendWith::Scene];
}

/// ⭐⭐⭐ **A MISTURA de uma cópia vectorial e o GRUPO a que ela pertence** (doc 118 W1).
///
/// ⚠️ **O lowering NÃO a conhece** — ele cria toda instância com o [`Default`] — e quem a CARIMBA é
/// o pump, depois de o sink inteiro ter sido baixado: é essa a razão de o [`BlendWith`] não viver no
/// [`ph2d_render::SinkStyle`]. O passe que ARRUMA as camadas lê-a instância a instância.
///
/// ⭐ **Ela viaja NA instância e não numa lista de intervalos ao lado**, e o motivo é medido no
/// código: dois passes de LOD (`motion_bridge_objects_lod`, `motion_shape_lod`) fazem `retain`
/// sobre a lista de instâncias DEPOIS do cozimento — um intervalo `[a, b)` guardado ao lado passaria
/// a apontar para outras cópias em silêncio. O `retain` preserva a ORDEM e os campos, logo um carimbo
/// sobrevive-lhe por construção.
///
/// ⚠️ **`sink` separa os grupos, e não os dois outros campos:** dois sinks com o MESMO modo e o MESMO
/// alcance são dois grupos — no `Copies` cada um isola-se sozinho. É o índice do sink no laço do
/// quadro, que é tudo o que o passe precisa (ele só compara vizinhos).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MisturaDoSink {
    /// O tag de mistura do sink — a MESMA leitura que o [`ph2d_render::SinkStyle::blend`] faz
    /// ([`sink_blend_tag`]); `0` é o `Mix`, que o passe desenha sem camada nenhuma.
    pub blend: u8,
    /// Com quem a mistura acontece — [`BlendWith`].
    pub com: BlendWith,
    /// O índice do sink que produziu esta cópia, neste quadro.
    pub sink: u32,
}

/// O alcance de um sink, lido pela MESMA leitura arredondada e presa dos outros tags.
#[must_use]
pub fn sink_blend_with(graph: &Graph, sink: NodeId) -> BlendWith {
    let top = (BlendWith::ALL.len() - 1) as u8;
    BlendWith::ALL[usize::from(tag(graph, sink, SINK_BLEND_WITH_PARAM, top))]
}

/// Quão longe do centro o pivô pode ir, em fracções do tamanho — o mesmo número
/// que o nó publica como `PIVOT_LIMIT`, e o gate da shell pina que são iguais.
pub const SINK_PIVOT_LIMIT: f32 = 1.0;

/// ⭐⭐⭐ **O interruptor do PASSE de separação** (doc 115 W5) — o mesmo nome que o nó declara como
/// `COLLIDE_PARAM`, com o mesmo gate da shell a pinar que concordam.
pub const SINK_COLLIDE_PARAM: &str = "collide";
/// Ver [`SINK_COLLIDE_PARAM`].
pub const SINK_COLLIDE_ITERATIONS_PARAM: &str = "collide_iterations";
/// O default das varreduras — **o número que o `motion.collide` já ship**, herdado de propósito
/// para uma cena migrada não mudar de qualidade em silêncio.
pub const SINK_COLLIDE_ITERATIONS_DEFAULT: f32 = 8.0;
/// **O tecto das varreduras — o MESMO que a folha do nó declara**, e há gate da shell a pina-lo.
///
/// ⛔⛔ **Ele era `64` por HERANÇA e subiu para `4096` por MEDIÇÃO** (doc 115 §18, ordem do dono):
/// uma cadeia converge sempre, e o que ela pede é `~n²` varreduras — `64` segura **quatro** peças,
/// `1024` segura `16` (a `18,5 %` de um quadro) e `4096` segura `32` (a `157 %`). A tabela inteira,
/// com o relógio, vive no [`ph2d_node_motion_output::COLLIDE_ITERATIONS_MAX`].
///
/// ⚠️ **O número que o artista escreve é o que corre** — ⛔ este `clamp` corta o que está ACIMA do
/// tecto declarado, e o tecto declarado é o que a caixa do cartão oferece: *um tecto que aceita e
/// entrega outra coisa é o «aceita e mente» que este repo já pagou três vezes.*
pub const SINK_COLLIDE_ITERATIONS_MAX: f32 = 4096.0;

/// O valor de um param do sink, ou `0.0` se ele não foi autorado.
///
/// ⚠️ **`NaN`/`inf` caem para `0.0` e não para o clamp**: um documento corrompido
/// desenha como o de sempre em vez de escolher um extremo que ninguém autorou.
fn param(graph: &Graph, sink: NodeId, name: &str) -> f32 {
    let v = graph
        .node_param_overrides(sink)
        .and_then(|p| p.get(name))
        .copied()
        .unwrap_or(0.0);
    if v.is_finite() { v } else { 0.0 }
}

/// Um param que é um TAG: arredondado meio-para-longe-de-zero e **clampado** para
/// dentro da faixa, nunca embrulhado.
fn tag(graph: &Graph, sink: NodeId, name: &str, top: u8) -> u8 {
    param(graph, sink, name).round().clamp(0.0, f32::from(top)) as u8
}

/// **O estilo com que um sink desenha.**
///
/// **A porta única.** O pump da CPU pergunta-a por sink (ele percorre muitos); a
/// shell pergunta-a pelo único sink que a rota da GPU aceita. Dois chamadores, uma
/// resposta — um segundo leitor seria livre de arredondar ou clampar de outra
/// maneira, e as duas rotas desenhariam o mesmo documento de maneiras diferentes,
/// que nenhum gate a olhar para uma rota consegue ver.
///
/// Um nó sem overrides — e todo nó que não é sink — devolve [`SinkStyle::PLAIN`],
/// que é exactamente o que os dois lowerings cravavam antes destes params
/// existirem.
/// ⭐⭐⭐ **A PORTA DO PRODUTO da lei [`ph2d_render::SinkStyle::so_com_forma`]** — `PH2D_MOTION_SO_COM_FORMA=1`.
///
/// > **Ordem do dono, 2026-09-17, reaberta em 19/09 como report:** *«Prefiro que toda visualização
/// > passe pelo Duplicator e que nós como Grid, rope, etc, não passem de posições do espaço, sem
/// > nenhuma capacidade de gerar pixels na tela»* · *«sem o duplicator só aparece um gizmo de osso
/// > ou segmento de corda (ou outro tipo de segmento) que não renderiza em runtime»*.
///
/// ⭐⭐⭐ **Ela nasce LIGADA desde 2026-09-19, e a mudança veio com a saída que a torna barata.**
/// A redacção anterior dizia *«ela nasce DESLIGADA, e o número é a razão: ligada, 111 das 123
/// cenas deixam de desenhar o que desenham hoje»* — e isso continua verdade **sobre a lei que ela
/// descrevia**, que era *não desenhar nada*. O dono reabriu o report (*«o grid continua desenhando
/// quadrados. A ordem foi não desenhar nada. se quiser coloque apenas pontos nas posições»*), e a
/// segunda frase é o que ship: as `111` cenas continuam a desenhar, **como MARCAS**, e o custo que
/// mantinha a lei desligada deixou de existir.
///
/// ⚠️ **`PH2D_MOTION_SO_COM_FORMA=0` devolve os quads de omissão** — o caminho de bissecção, e a
/// única maneira de ver o que uma cena desenhava antes desta wave.
///
/// ⚠️ **LIDA UMA VEZ POR CORRIDA e NUM SÍTIO SÓ.** Este é o único leitor do ambiente desta lei —
/// os dois lowerings recebem-na como DADO, dentro do estilo, e é isso que os torna gateáveis sem
/// o ambiente (a auditoria do doc 115 §31: *«um gate que lê o ambiente mede a máquina»*).
/// Um `OnceLock` porque `sink_style` corre por sink e por quadro.
#[must_use]
pub fn so_com_forma_por_ordem() -> bool {
    static PORTA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *PORTA.get_or_init(|| ordem_de(std::env::var("PH2D_MOTION_SO_COM_FORMA").ok().as_deref()))
}

/// A leitura de uma ordem do ambiente, **pura**, para o gate a poder medir sem mexer no processo.
///
/// ⚠️⚠️ **A escada está INVERTIDA em relação às outras portas da casa, e é de propósito:** aqui a
/// ausência quer dizer *«a lei do dono»* e só um `0` explícito a desliga. As irmãs (`PH2D_GPU_COOK`
/// e companhia) armam uma feature nova e por isso leem a ausência como desligado; esta **já não é
/// nova** — ela é o produto, e o que a variável oferece é a saída de bissecção.
#[must_use]
pub fn ordem_de(v: Option<&str>) -> bool {
    !matches!(v, Some("0"))
}

#[must_use]
pub fn sink_style(graph: &Graph, sink: NodeId) -> SinkStyle {
    // O tecto do blend é o array de pipelines, lido DO RENDERER — um literal `5`
    // aqui continuaria a compilar no dia em que um sexto modo aterrasse e
    // recusaria silenciosamente escolhê-lo.
    let top = (ph2d_render::pipeline::BLEND_PIPELINE_COUNT - 1) as u8;
    let pivot_lim = SINK_PIVOT_LIMIT;
    SinkStyle {
        blend: tag(graph, sink, SINK_BLEND_PARAM, top),
        pivot: [
            param(graph, sink, SINK_PIVOT_X_PARAM).clamp(-pivot_lim, pivot_lim),
            param(graph, sink, SINK_PIVOT_Y_PARAM).clamp(-pivot_lim, pivot_lim),
        ],
        // ⚠️ O `repeat` fica em `Inherit` de propósito: com o `uv_xform` na
        // identidade as três leis de wrap concordam dentro de `[0,1]`, então um
        // knob de repetição neste sink seria **morto** até alguém escrever a
        // coluna `uv_cell` — e é o `motion.sub_uv` que a escreve, com o wrap dele.
        sampling: RenderInstance::pack_sampling(
            tag(
                graph,
                sink,
                SINK_FILTER_PARAM,
                ph2d_render::image_filter::FILTER_TAG_MAX,
            ),
            0,
        ),
        stream_order: param(graph, sink, SINK_SORT_PARAM) >= 0.5,
        // ⭐⭐⭐ **A LEI DO DONO, e ela NÃO sai de um param** — ver [`so_com_forma_por_ordem`].
        so_com_forma: so_com_forma_por_ordem(),
        // ⚠️ **O ladrilho da marca NÃO se lê do grafo — ele vem do ÁTLAS**, que esta função não
        // conhece. O valor de omissão é o átlas inteiro, e quem o tem (a bomba, pelo
        // [`crate::Pump::define_o_ladrilho_do_ponto`], e a rota do dispositivo) sobrescreve-o.
        // *Um sítio que não pode saber a resposta devolve a omissão e nomeia quem sabe.*
    }
}

/// O tag de blend de um sink — o atalho que os chamadores de sempre usam.
#[must_use]
pub fn sink_blend_tag(graph: &Graph, sink: NodeId) -> u8 {
    sink_style(graph, sink).blend
}

/// ⭐⭐⭐ **QUANTAS VARREDURAS este sink pede ao passe de separação** (doc 115 W5) — `0` quer dizer
/// *«o passe não corre»*, e é o que todo documento que já existe devolve.
///
/// # ⛔⛔ Porque ela NÃO vive no [`SinkStyle`], e o motivo é de MOTOR
///
/// Os cinco params deste sink são lidos no fim, por quem baixa a corrente — mas os quatro do
/// `SinkStyle` são **ESTILO** (o que a peça parece) e este muda **POSIÇÕES**. O `SinkStyle` viaja
/// para as duas rotas de lowering, e a do dispositivo ignoraria uma grandeza que não sabe honrar:
/// isso daria a MESMA cena separada na CPU e sobreposta na placa, sem erro nenhum — a espécie de
/// divergência que a cerca do doc 115 W1 existe para impedir. ⇒ porta própria, e a cerca do shell
/// recusa o dispositivo enquanto ela devolver `> 0`.
///
/// # ⚠️⚠️ O DEFAULT não vem do manifesto, e isso quase shipou um botão mudo
///
/// O [`param`] acima lê o **override** do documento e devolve `0.0` quando não há — ele nunca
/// consulta o `ParamSpec::default`. Os quatro params antigos deste sink têm todos default `0`, logo
/// ninguém tinha reparado; o `collide_iterations` é **o primeiro com default ≠ 0** desta casa, e
/// lido pela porta de sempre ele valeria `0` num documento acabado de criar. *O artista ligava o
/// interruptor e nada acontecia.* ⇒ a ausência de override lê-se aqui como
/// [`SINK_COLLIDE_ITERATIONS_DEFAULT`], que é o número que o nó declara.
#[must_use]
pub fn sink_collide_sweeps(graph: &Graph, sink: NodeId) -> usize {
    if param(graph, sink, SINK_COLLIDE_PARAM) < 0.5 {
        return 0;
    }
    let autorado = graph
        .node_param_overrides(sink)
        .and_then(|p| p.get(SINK_COLLIDE_ITERATIONS_PARAM))
        .copied()
        .filter(|v| v.is_finite())
        .unwrap_or(SINK_COLLIDE_ITERATIONS_DEFAULT);
    autorado.round().clamp(1.0, SINK_COLLIDE_ITERATIONS_MAX) as usize
}

/// ⭐⭐⭐ **O QUE UM SINK DESENHA** — a corrente cozida com o passe do fim aplicado (doc 115 §16).
///
/// `None` quando nada há a separar (a corrente não declara colisor, o interruptor está desarmado,
/// ou ninguém se mexeu) — e é isso que mantém toda cena de hoje **byte-idêntica**, sem clonar.
///
/// ⭐ O segundo membro é o [`ph2d_contact::passe::Relatorio`] — varreduras, peças e candidatos —,
/// que é o que dá ao artista o preço do knob sem ele ter de o ler no relógio de parede (report do
/// dono, 2026-09-18). ⚠️ *Os três números existem porque o relógio sozinho não distingue «muitas
/// peças» de «uma pilha apertada», e as curas são diferentes.*
///
/// # ⛔⛔ Porque isto é uma PORTA e não duas linhas repetidas
///
/// A W5 pôs o passe **dentro** do braço que faz o lowering, e a TOMADA (`tap_streams`, de que o
/// gizmo do colisor vive) cozinha por conta própria — logo ela continuou a guardar a corrente
/// **CRUA**. O dono viu-o na primeira foto: *«a colisão está correta … mas o gizmo do collider se
/// separa de sua shape e interpenetra»*. As formas estavam nas posições de DEPOIS e o contorno azul
/// nas de ANTES.
///
/// ⚠️ *Duas respostas à mesma pergunta — «onde estão as peças?» — e o artista vê as duas ao mesmo
/// tempo.* Com uma porta, um terceiro consumidor herda a resposta certa por construção.
#[must_use]
pub fn o_que_o_sink_desenha(
    graph: &Graph,
    sink: NodeId,
    cozido: &ph2d_nodegraph::attr::Stream,
) -> (
    Option<ph2d_nodegraph::attr::Stream>,
    ph2d_contact::passe::Relatorio,
) {
    #[cfg(test)]
    PASSAGENS.with(|c| c.set(c.get() + 1));
    ph2d_contact::passe::separa_com_relatorio(cozido, sink_collide_sweeps(graph, sink))
}

#[cfg(test)]
thread_local! {
    /// ⭐⭐⭐ **Quantas vezes esta porta correu** — o instrumento do report de 2026-09-18 (*«com 1024
    /// FPS cai para 7»*).
    ///
    /// ⚠️⚠️ **Ele existe porque a duplicação era INVISÍVEL a toda régua de valor.** O gizmo do colisor
    /// pede o próprio sink como tomada, e a tomada cozinhava-o outra vez; o 2.º cozimento bate no memo,
    /// mas o passe do fim **não é memoizado** e corria duas vezes. As duas passagens entregam a MESMA
    /// corrente — *logo nenhum gate de igualdade, de bits ou de pixel podia vê-las*, e o que sobra para
    /// observar é a CONTA.
    ///
    /// ⛔ `#[cfg(test)]`: o produto não paga um contador por quadro para se medir a si próprio.
    ///
    /// ⚠️⚠️ **E ele é POR THREAD, não global — a 1.ª redacção era um átomo e o gate REPROVOU na suíte
    /// enquanto passava sozinho.** Os testes correm em paralelo e há mais de um a cozinhar um sink com
    /// o passe armado: o contador partilhado somava as passagens de TODOS eles. *Um censo que partilha
    /// estado com os vizinhos mede os vizinhos* — e a cura é a régua ser do sítio onde a pergunta é
    /// feita, que aqui é a thread do teste.
    pub(crate) static PASSAGENS: core::cell::Cell<usize> = const { core::cell::Cell::new(0) };
}

#[cfg(test)]
#[path = "sink_style_tests.rs"]
mod tests;
