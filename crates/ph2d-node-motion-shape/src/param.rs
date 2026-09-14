//! **Os NOMES dos params f32 do `source.shape`** — a UMA lista de que o manifesto, as
//! hints de UI, os gates, o `eval` do nó e o leitor do shell tiram a chave (sem literal de
//! string a derivar entre eles).
//!
//! Módulo irmão pelo teto de LOC do HR-18: o `lib.rs` chegou ao teto ao ganhar o TRIM e o
//! TRACEJADO, e a lista de nomes é a metade que não tem nada a ver com o `NodeOp`.

use ph2d_nodegraph::node::ParamSpec;

pub const KIND: &str = "kind";
pub const SIZE: &str = "size";
pub const ASPECT: &str = "aspect";
pub const SIDES: &str = "sides";
pub const CORNER: &str = "corner";
pub const STAR_DEPTH: &str = "star_depth";
pub const CLEFT: &str = "cleft";
pub const TOOTH_DEPTH: &str = "tooth_depth";
pub const HOLE: &str = "hole";
/// A largura do TRAÇO em unidades de mundo. `0` = sem traço ⇒ a forma de
/// sempre, byte-idêntica (ver [`super::ShapeParams::stroke`]).
pub const STROKE_WIDTH: &str = "stroke_width";
pub const STROKE_R: &str = "stroke_r";
pub const STROKE_G: &str = "stroke_g";
pub const STROKE_B: &str = "stroke_b";
pub const STROKE_A: &str = "stroke_a";

/// **A ABERTURA do arco**, em graus (doc 89 folha 14, a linha do *sweep / start /
/// inner*). A família do círculo (`Ellipse`/`Circle`/`Pie`/`Segment`) é uma só forma na
/// biblioteca; o que a faz pizza, rosquinha ou anel parcial são estes três números, que
/// a receita passava **fixos**.
///
/// ⚠️ **`0` é SENTINELA: *"como a forma nasce"***, e não *"uma fatia de zero graus"*.
/// Sem ela o default não reduz: um círculo passa `0` hoje (a biblioteca lê isso como
/// volta inteira) mas uma `Pie` passa `k.defaults()`, o ângulo canónico dela — um
/// default único quebraria uma das duas. O que se perde é autorar uma fatia de
/// exactamente 0°, que não desenha nada.
pub const SWEEP: &str = "sweep";
/// **Onde o arco COMEÇA**, em graus. `0` é o default da biblioteca para as três formas,
/// então aqui ele é o valor neutro de verdade, não uma sentinela.
pub const START: &str = "start";
/// **O RAIO INTERNO** como fracção do externo (`0` = maciço, o default da biblioteca).
/// É o que leva a pizza a rosquinha e o arco a anel parcial.
///
/// ⚠️ Não vale para a `Segment`: a corda (`ellipse_chord`) não tem miolo — e o gate
/// `no_kind_hides_a_live_knob_or_shows_a_dead_one` é quem o prova, mexendo no número.
pub const INNER: &str = "inner";

/// **Os três DESVIOS de raio por canto** (doc 89 folha 14, a linha do *raio por canto*),
/// somados ao `corner` — `[TL, TR, BR, BL]` e o `corner` é o TL. `0` em todos ⇒ o
/// round-rect uniforme, e a `rounded_rect_corners` desvia literalmente para a
/// `rounded_rect` de sempre quando os quatro raios são iguais e a suavização é zero.
pub const CORNER_TR: &str = "corner_tr";
pub const CORNER_BR: &str = "corner_br";
pub const CORNER_BL: &str = "corner_bl";
/// **A SUAVIZAÇÃO do canto** (`0..1`, o *corner smoothing* do Figma / o squircle do
/// iOS). `0` = o arco circular de sempre.
pub const SMOOTHING: &str = "smoothing";

/// **Onde o trecho revelado COMEÇA**, em fração do comprimento total do contorno (doc 89
/// folha 14, a linha do *trim/dash*). É o *Trim Paths* do After Effects e o *Trim* do
/// Cavalry: keyar o [`TRIM_END`] de 0 a 1 **desenha** a forma.
///
/// ⚠️ **`{0, 1, 0}` é NEUTRO e o neutro é no-op byte-idêntico** — a pilha de efeitos
/// (ADR-0132) salta um efeito neutro por inteiro, então o default não custa uma alocação
/// nem move um bit da forma que sempre shipou.
///
/// ⚠️ **A célula da folha 14 apontava a função ERRADA.** Ela dizia *"`trim_path` existe
/// (`marker.rs:395`), falta a fiação"* — e aquela função recua as pontas em unidades de
/// MUNDO ao longo da poligonal das âncoras para dar lugar às setas, e **devolve o caminho
/// intocado se ele for fechado**. Medido em 2026-08-19: das 47 formas da biblioteca, 42
/// fecham, então ligá-la daria dois sliders inertes em 100% do catálogo fillável. O que
/// esta linha liga é o [`ph2d_vec_scene::fx_trim`] — arco exato, e **abre** o contorno.
pub const TRIM_START: &str = "trim_start";
/// **Onde o trecho revelado ACABA**, em fração do comprimento (`1` = o caminho inteiro).
pub const TRIM_END: &str = "trim_end";
/// **Gira o ponto de partida** ao longo do caminho, em frações. Num contorno fechado ele dá
/// a volta pela emenda — é o que faz um traço correr em torno de um círculo.
pub const TRIM_OFFSET: &str = "trim_offset";

/// **O TRACEJADO**: o comprimento do traço como MÚLTIPLO da largura (`0` = contínuo).
///
/// ⚠️ Múltiplo da largura, e não unidade de mundo, porque é o que o [`ph2d_vec_scene::StrokeSpec`]
/// já fala: engrossar o traço alonga traço e vão na proporção, então a projeção da ponta nunca
/// engole o vão.
pub const DASH: &str = "dash";
/// **O VÃO** entre dois traços, também em múltiplos da largura. Inerte enquanto [`DASH`] for `0`.
pub const DASH_GAP: &str = "dash_gap";

/// **TODOS eles, na ordem do manifesto.**
///
/// ⚠️ Ela existe para a CHAVE do cache ser derivada em vez de enumerada. A
/// `shape_key` listava os nove campos à mão, e uma chave que enumera as
/// entradas de um valor é como a próxima é esquecida — o param novo passa a
/// não mintar entrada nova, a forma antiga volta do cache, e o controle fica
/// **inerte depois da primeira vez** (foi o defeito do *Pattern Offset* do
/// sculpt3d, 2026-08-09). Um param acrescentado aqui entra na chave e no
/// manifesto de uma vez.
pub const ALL: &[&str] = &[
    KIND,
    SIZE,
    ASPECT,
    SIDES,
    CORNER,
    STAR_DEPTH,
    CLEFT,
    TOOTH_DEPTH,
    HOLE,
    STROKE_WIDTH,
    STROKE_R,
    STROKE_G,
    STROKE_B,
    STROKE_A,
    SWEEP,
    START,
    INNER,
    CORNER_TR,
    CORNER_BR,
    CORNER_BL,
    SMOOTHING,
    TRIM_START,
    TRIM_END,
    TRIM_OFFSET,
    DASH,
    DASH_GAP,
];

/// **A COR PRÓPRIA da forma** (doc 89 folha 14 — idem Cavalry / AE / Illustrator: um
/// primitivo desenhado tem preenchimento).
///
/// ⚠️ **A célula media a composição e ela FUNCIONA** — `source.shape → motion.tint` pinta o
/// primitivo, e o picker OKLCH já vive naquele nó. Isto entra na mesma pela razão que fechou
/// metade da folha 05: *um nó a mais para dizer de que cor é a coisa que este nó desenha*. Uma
/// forma é a única fonte do catálogo cujo produto é ELA PRÓPRIA — as outras emitem posições
/// para outra coisa pintar.
///
/// ⚠️ **É um TOGGLE e não uma sentinela na alfa.** O irmão `stroke_*` usa `stroke_width = 0`
/// como *"sem traço"*, e ali a sentinela é natural (uma largura de zero é a ausência). Uma cor
/// não tem essa grandeza: usar `fill_a = 0` faria arrastar a alfa até ao fim no picker
/// **trocar silenciosamente de modo**, em vez de dar uma forma transparente.
///
/// ⚠️ **Desligado, a coluna `tint` NÃO é tocada** — o que o shell publicou atravessa como
/// atravessava, byte a byte. É a lei estrutural do `follow_rotation` do `motion.spline_wrap`:
/// o default não é *"escrever o mesmo valor"*, é *"não escrever"*.
pub const FILL: &str = "fill";
pub const FILL_R: &str = "fill_r";
pub const FILL_G: &str = "fill_g";
pub const FILL_B: &str = "fill_b";
pub const FILL_A: &str = "fill_a";

/// **A ROTAÇÃO PRÓPRIA da forma**, em graus (doc 89 folha 14 — *"uma estrela apontando para
/// cima"*).
///
/// ⚠️ **ATRIBUI, não soma**, e é a lei da casa vista do lado da FONTE: o
/// `motion.distribute_curve` faz `set` no `rot` *porque é uma fonte e não há nada com que
/// compor*, enquanto o `motion.spline_wrap` SOMA por ser modificador. Este nó é fonte.
///
/// ⚠️ **`0` não escreve a coluna** (a mesma lei estrutural do [`FILL`]) — e `0` é também o
/// valor que a coluna teria, então a única diferença é entre *não haver `rot`* e *haver um
/// `rot` de zero*. A jusante isso importa: um `motion.rotate` a somar sobre uma coluna ausente
/// parte do `0` na mesma, mas um censo de colunas veria uma que ninguém autorou.
pub const ROTATION: &str = "rotation";

/// ⭐⭐ **O COLISOR QUE A FORMA DECLARA** (doc 109 — ordem do dono, 2026-09-13: *«colidem
/// sozinhas»*). `collide = 0` ⇒ nenhuma coluna de colisor é escrita ⇒ o stream de sempre.
///
/// ⚠️ **Os cinco ficam FORA de [`ALL`], de propósito:** não mudam a geometria, e entrar na chave
/// de conteúdo re-internaria um `VecPath` a cada clique na caixa — o custo que a normalização do
/// `size` existe para evitar.
pub const COLLIDE: &str = "collide";
/// **A FORMA do colisor** (doc 109 §5 — report do dono: *«no mínimo colliders circulares e
/// retangulares que tentam se adaptar às dimensões da shape»*): `0` Box · `1` Circle.
/// ⚠️ O índice é formato de arquivo — APPEND ONLY.
pub const COLLIDER_SHAPE: &str = "collider_shape";
/// `Box`: a LARGURA como fracção da largura da forma (`1` = a caixa envolvente do contorno).
pub const COLLIDER_WIDTH: &str = "collider_width";
/// `Box`: a ALTURA como fracção da altura da forma.
pub const COLLIDER_HEIGHT: &str = "collider_height";
/// `Circle`: o RAIO como fracção do círculo que toca os lados maiores da caixa envolvente.
pub const COLLIDER_RADIUS: &str = "collider_radius";
/// **VER o colisor no canvas** (report do dono, 2026-09-13: *«coloque um botão no nó shape: Ver
/// collider»*). Ligado, o contorno do colisor de cada peça desta forma é pintado POR CIMA da arte —
/// esteja o cartão seleccionado ou não. As ALÇAS continuam a ser da forma seleccionada, porque
/// arrastar edita os params DELA.
pub const SHOW_COLLIDER: &str = "show_collider";
/// **TRAVAR a rotação** (doc 109 §6 — report do dono, 2026-09-13: *«precisa destravar a rot. e
/// colocar outro botão para travar rotação»*). Desligado (o default) as peças TOMBAM no contacto;
/// ligado, elas só deslizam. Escreve a coluna `inv_inertia` a `0` — o *Freeze Rotation* de um corpo
/// rígido, dito no vocabulário desta casa.
pub const LOCK_ROTATION: &str = "lock_rotation";

/// ⭐⭐⭐ **O ATRITO da peça** (doc 109 §7 — report do dono, 2026-09-13: *«os círculos não rotacionam
/// com a colisão, talvez por falta de atrito. Precisamos de parâmetros do material»*).
///
/// `0` é gelo e `1` é lixa, a faixa de todo motor. ⚠️ **E ele não é só «quanto trava»: é a única
/// coisa que RODA um círculo** — a correcção de não-penetração empurra ao longo da normal, cuja
/// alavanca num disco é exactamente zero (a conta está em `ph2d_contact::atrito`).
///
/// ⚠️ **O default é `0,5`, e é o do Rapier** — o motor de corpo rígido que este repo já usa na
/// física. Um `0` faria o dono ter de descobrir o slider para ver a coisa que ele pediu.
pub const FRICTION: &str = "friction";
/// ⭐⭐ **O SALTO da peça** (doc 109 §7): quanto de um embate volta. `0` (o default, o do Rapier e
/// o do Box2D) é uma peça morta — a lei de sempre.
///
/// ⚠️ **O par combina-se pelo MAIOR dos dois**, então uma bola saltitante salta contra uma parede
/// morta. É a lei do Box2D, e é o que o artista espera de uma bola saltitante.
pub const BOUNCE: &str = "bounce";

/// **As duas colunas que o SHELL publica** com a geometria — a caixa envolvente do contorno de
/// preenchimento, em unidade de geometria — e que o nó **retira sempre**, depois de declarar.
/// Nenhum nó a jusante as vê.
pub const COLLIDER_FIT_CENTER_COL: &str = "collider_fit_center";
pub const COLLIDER_FIT_HALF_COL: &str = "collider_fit_half";

/// ⭐ **A DECLARAÇÃO de cada param — o nome E o default, no mesmo sítio.**
///
/// ⚠️ **Irmã de [`ALL`] pelo teto de LOC do HR-18** (o `lib.rs` chegou a `697` de `700` ao ganhar
/// a secção de colisão), e melhor por assunto: uma lista de nomes ao lado de uma lista de defaults
/// noutro ficheiro é como um param novo nasce sem valor de partida.
///
/// ⚠️ **APPEND ONLY, como toda lista lida posicionalmente por quem guarda um índice.**
pub const SPECS: &[ParamSpec] = &[
    ParamSpec {
        name: KIND,
        default: 0.0,
    },
    ParamSpec {
        name: SIZE,
        default: 1.0,
    },
    ParamSpec {
        name: ASPECT,
        default: 1.0,
    },
    ParamSpec {
        name: SIDES,
        default: 6.0,
    },
    ParamSpec {
        name: CORNER,
        default: 0.0,
    },
    ParamSpec {
        name: STAR_DEPTH,
        default: 0.45,
    },
    ParamSpec {
        name: CLEFT,
        default: 0.2,
    },
    ParamSpec {
        name: TOOTH_DEPTH,
        default: 0.35,
    },
    ParamSpec {
        name: HOLE,
        default: 0.45,
    },
    // ⚠️ **Apendados** (doc 89 folha 14, P0). `stroke_width = 0` ⇒ sem
    // `StrokeSpec` ⇒ a forma que sempre shipou, byte-idêntica.
    ParamSpec {
        name: STROKE_WIDTH,
        default: 0.0,
    },
    ParamSpec {
        name: STROKE_R,
        default: 0.0,
    },
    ParamSpec {
        name: STROKE_G,
        default: 0.0,
    },
    ParamSpec {
        name: STROKE_B,
        default: 0.0,
    },
    ParamSpec {
        name: STROKE_A,
        default: 1.0,
    },
    // ⚠️ **Apendados** (doc 89 folha 14, as duas últimas células). `fill = 0` e
    // `rotation = 0` ⇒ as colunas `tint`/`rot` do shell atravessam INTOCADAS.
    ParamSpec {
        name: FILL,
        default: 0.0,
    },
    ParamSpec {
        name: FILL_R,
        default: 1.0,
    },
    ParamSpec {
        name: FILL_G,
        default: 1.0,
    },
    ParamSpec {
        name: FILL_B,
        default: 1.0,
    },
    ParamSpec {
        name: FILL_A,
        default: 1.0,
    },
    ParamSpec {
        name: ROTATION,
        default: 0.0,
    },
    // ⚠️ **Apendados** (doc 89 folha 14, as linhas do *sweep/start/inner* e do *raio
    // por canto*). Todos neutros no default — o `sweep = 0` pela sentinela documentada
    // no [`param::SWEEP`], os outros seis porque `0` já é o valor que a biblioteca usa.
    ParamSpec {
        name: SWEEP,
        default: 0.0,
    },
    ParamSpec {
        name: START,
        default: 0.0,
    },
    ParamSpec {
        name: INNER,
        default: 0.0,
    },
    ParamSpec {
        name: CORNER_TR,
        default: 0.0,
    },
    ParamSpec {
        name: CORNER_BR,
        default: 0.0,
    },
    ParamSpec {
        name: CORNER_BL,
        default: 0.0,
    },
    ParamSpec {
        name: SMOOTHING,
        default: 0.0,
    },
    // ⚠️ **Apendados** (doc 89 folha 14, a linha do *trim/dash*). `{0, 1, 0}` é o
    // NEUTRO do `TrimSpec`, e a pilha de efeitos salta um efeito neutro por inteiro
    // ⇒ a forma que sempre shipou, byte-idêntica. O `dash_gap` nasce em `1` porque
    // um vão de zero seria um tracejado contínuo com custo — mas ele só existe
    // quando o `dash` sai do zero.
    ParamSpec {
        name: TRIM_START,
        default: 0.0,
    },
    ParamSpec {
        name: TRIM_END,
        default: 1.0,
    },
    ParamSpec {
        name: TRIM_OFFSET,
        default: 0.0,
    },
    ParamSpec {
        name: DASH,
        default: 0.0,
    },
    ParamSpec {
        name: DASH_GAP,
        default: 1.0,
    },
    // ⚠️ **Apendados** (doc 109). `collide = 0` ⇒ nenhuma coluna nova ⇒ o stream de sempre.
    ParamSpec {
        name: COLLIDE,
        default: 0.0,
    },
    ParamSpec {
        name: COLLIDER_SHAPE,
        default: 0.0,
    },
    ParamSpec {
        name: COLLIDER_WIDTH,
        default: 1.0,
    },
    ParamSpec {
        name: COLLIDER_HEIGHT,
        default: 1.0,
    },
    ParamSpec {
        name: COLLIDER_RADIUS,
        default: 1.0,
    },
    // **Ver o colisor** (report do dono, 2026-09-13) — LIGADO por omissão: quem liga o
    // `Collide` quer ver o que declarou, e o botão é para o DESLIGAR quando a cena enche.
    ParamSpec {
        name: SHOW_COLLIDER,
        default: 1.0,
    },
    // **Travar a rotação** (doc 109 §6) — DESLIGADO por omissão: o dono pediu a rotação
    // destravada, e este botão é o que a prende quando ela não é o que se quer.
    ParamSpec {
        name: LOCK_ROTATION,
        default: 0.0,
    },
    // ── O MATERIAL (doc 109 §7) ─────────────────────────────────────────────
    // APENDADOS, nunca inseridos: a lista é lida posicionalmente por quem guarda um índice.
    ParamSpec {
        name: FRICTION,
        default: 0.5,
    },
    ParamSpec {
        name: BOUNCE,
        default: 0.0,
    },
];
