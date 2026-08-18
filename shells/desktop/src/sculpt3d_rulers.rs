//! **AS RÉGUAS DO GESTO** — quanto um pixel de arrasto vale, e onde uma
//! grandeza deixa de existir.
//!
//! Filho (`#[path]`) de [`super`], cortado por ASSUNTO do que sobrou lá: o pai
//! diz *o que a CENA é* (o `Sculpt3dScene`, os módulos, o `Drag`) e este arquivo
//! diz *com que régua a mão fala com ela*.
//!
//! ⚠️ **Quase todo número aqui é decisão de SMOKE, não teto de recurso** — e a
//! distinção é a do §0 do `CLAUDE.md`: um limite legítimo diz **de que recurso
//! ele é** e traz a medição ao lado. Os que TÊM medição a carregam no próprio
//! doc-comment ([`DEFAULT_RADIUS_PX`], [`RADIUS_MIN_PX`],
//! [`RADIUS_MAX_FRAC_OF_HEIGHT`]); os outros dizem, em vez de fingir, que quem
//! os escolheu foi o olho do Enio.
//!
//! O pai re-exporta tudo (`use rulers::*`), então os filhos seguem lendo
//! `super::ORBIT_RAD_PER_PX` como sempre leram — o corte é de arquivo, nunca de
//! caminho.

/// Quantos radianos um pixel de arrasto vale.
///
/// Decisão de **smoke**, como a tolerância do RDP do Flip: 0,01 dá meia volta a
/// cada ~314 px, que é uma varredura confortável de trackpad. Não é um teto de
/// recurso, então não tem tabela de medição ao lado — tem o olho do Enio.
pub(super) const ORBIT_RAD_PER_PX: f32 = 0.01;

/// O raio do pincel, em **pixels de tela**.
///
/// ⚠️ **Pixels, não fração do modelo** — o raio de MUNDO é derivado por dab
/// (`Camera3d::world_radius_for_screen_px`), então o pincel mantém o tamanho
/// aparente quando a câmera aproxima. É o `computeWorldRadius2` do SculptGL, e
/// é o que Blender e ZBrush entregam: aproximar É como se alcança detalhe fino,
/// e um raio ancorado no modelo tornava isso impossível (o pincel crescia junto
/// com a imagem).
///
/// **50 px é MEDIDO, não escolhido:** é o que reproduz o tamanho aparente do
/// default anterior (0,12 do span) na cena do smoke a 720p — ver
/// `ph2d-mesh-render/tests/measure_screen_radius.rs`.
pub(super) const DEFAULT_RADIUS_PX: f32 = 50.0;

/// Passo das teclas de LUZ (`Q`/`E` giram, `R`/`F` sobem e descem), em graus
/// inteiros — que é a unidade em que o rig é autorado. Quinze graus porque o
/// gesto é *"ver a forma reacender"*, não afinar: um passo de 1° pediria vinte
/// toques para a mudança ficar óbvia.
pub(super) const LIGHT_STEP_DEG: u16 = 15;

/// Passo do `[` / `]`. Multiplicativo pelo motivo do `dolly`: o gesto tem o
/// mesmo efeito *aparente* com pincel grande e pequeno.
pub(super) const RADIUS_STEP: f32 = 1.15;

/// Quantos passos um clique de blur/sharpen dá.
///
/// ⚠️ **Número de SMOKE, não teto de recurso.** Um passo é pequeno de propósito
/// (`BLUR_MIX = 0,5`, para o gesto não apagar a própria borda de uma vez), então
/// o clique precisa de vários para o artista ver a diferença — e clicar de novo
/// borra mais, que é o que o gesto significa.
pub(super) const MASK_OP_PASSES: u32 = 6;

/// O piso do raio, em pixels. **Quem aperta é a TELA, não a malha** — e isso é
/// medição, não herança: a régua antiga dizia *"menor que uma aresta não pega
/// vértice"*, e na cena do smoke um disco de **0,5 px já pega um vértice**
/// (`measure_screen_radius.rs`), porque a malha é densa. O que de fato quebra
/// abaixo de um pixel é o artista **ver onde está mirando**.
///
/// ⚠️ **Este doc-comment estava ORFANADO:** ele tinha escorregado para cima do
/// [`MASK_OP_PASSES`] (que passou a abrir descrevendo o piso do raio) e a const
/// ficara NUA — a mesma família que o split do `paint.rs` do Painter já pagou em
/// 2026-07-19. Um `mod` novo entre uma doc e o item dela não dá erro: ela apenas
/// passa a documentar o vizinho.
pub(super) const RADIUS_MIN_PX: f32 = 1.0;

/// O teto do raio, em fração da ALTURA do viewport.
///
/// ⚠️ **Fração da tela, e não um número fixo de pixels, porque um teto fixo muda
/// de SIGNIFICADO com a resolução:** medido, 160 px cobre **91% da altura do
/// modelo a 1280×720 e 45% a 2560×1440**. `0,125` é a mesma promessa do teto
/// antigo (*acima de meio modelo o "pincel" é um deformador global, que é outra
/// ferramenta*) escrita no recurso que de fato aperta — com o enquadramento
/// padrão o modelo ocupa 49% da altura, então 1/8 de tela é meio modelo.
pub(super) const RADIUS_MAX_FRAC_OF_HEIGHT: f32 = 0.125;

/// A zona morta do **Twist**, em pixels de tela.
///
/// ⚠️ **Ela não é conforto, é a fronteira onde a grandeza deixa de existir:**
/// perto da âncora a direção *âncora → cursor* é RUÍDO, e um tremor de um pixel
/// a um pixel de distância vale meio radiano. Trinta é o número do SculptGL
/// (`Twist.js:92`), e como todo número de gesto deste arquivo ele é decisão de
/// **smoke** — não é teto de recurso nenhum.
pub(super) const TWIST_DEADZONE_PX: f32 = 30.0;

/// Quanto de escala vale um pixel de arrasto horizontal no **Local Scale**
/// (`+1` dobra o raio da pegada). Cem pixels dobram; decisão de smoke, como o
/// [`ORBIT_RAD_PER_PX`].
pub(super) const SCALE_PER_PX: f32 = 0.01;

/// Quanta força um pixel de arrasto vale no **FILTRO** —
/// `sculpt_filter_mesh.cc:2301` da referência.
///
/// ⚠️ **Não é um teto nem um ajuste de gosto: é a régua da referência.** Ela
/// decide quantos pixels o artista percorre para atravessar a faixa útil de cada
/// lei (`FilterKind::range`), e mudá-la muda o quanto a mão anda para o mesmo
/// resultado — nunca o resultado que uma dada força produz.
///
/// Mora aqui, e não no [`super::filter`], porque é **a mesma espécie** dos
/// vizinhos acima: pixel de arrasto → grandeza do gesto. O módulo do filtro
/// guarda o que o gesto FAZ.
pub(super) const FILTER_DRAG_PER_PX: f32 = 0.001;
