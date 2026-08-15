//! **OS SENSORES do player** — o teto da quina, o headroom do agachar e a
//! parede.
//!
//! ⚠️ **Corte por RESPONSABILIDADE:** o `bridge::player` responde *"o que a lei
//! decidiu neste tique?"* — o cast do chão, a chamada da lei, o motor, a reação —
//! e estes três respondem a pergunta anterior: *"o que há à volta?"*. Nenhum
//! deles tem política: a decisão de castar (`*_probe_wanted`) é da LEI, e é ela
//! que os chama.
//!
//! ⚠️ **Dois leem por RAIOS e um VARRE o corpo**, e a diferença não é estilo:
//!
//! - o da **quina** é um PERFIL (65 amostras que dizem *onde* há teto, para a
//!   lei escolher para que lado escapar) — uma varredura devolve um contacto e
//!   não sabe responder isso;
//! - o da **parede** entrega o flanco inteiro e a lei reduz com a régua da perna
//!   (`max_slope`), porque *qual superfície* e *se é parede* são a mesma
//!   pergunta;
//! - o do **agachar** só alguma vez foi perguntado *"cabe?"*, e essa é
//!   literalmente a pergunta que uma varredura responde. Ele foi convertido na
//!   `W-ShapeCast`; os outros dois não, com o motivo escrito em cada um.
//!
//! # ⚠️ A GEOMETRIA de cada sensor é uma porta, e o desenho chama a MESMA
//!
//! Desde a `W-Probes` este módulo não computa origens dentro do laço de cast:
//! cada sensor tem uma porta que devolve **onde ele olha** ([`ground_ray`] ·
//! [`wall_rays`] · [`corner_geom`] · [`headroom_offset`]), e quem casta e quem
//! **desenha** chamam a mesma. Um segundo cálculo do lado do overlay seria uma
//! segunda resposta a *"onde este raio nasce?"* — e ela divergiria no primeiro
//! dia em que alguém mexesse numa das duas, com o sintoma a ser um desenho que
//! mente sobre o que o produto mede.
//!
//! Módulo FILHO por `#[path]`, então `super::*` continua a alcançar o que o pai
//! não exporta.

use super::*;

/// **Onde um sensor olha** — a geometria, sem a resposta.
///
/// ⚠️ Uma struct e não uma tupla: `origin` e `dir` são os dois `[f32; 2]` do
/// mesmo tamanho, e trocá-los compila, roda, e desenha um sensor que aponta para
/// o lugar errado.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct ProbeRay {
    pub origin: [f32; 2],
    /// Unitária.
    pub dir: [f32; 2],
    pub reach: f32,
    /// **Quanto do alcance está DENTRO do corpo** — a parte que o cast tem de
    /// percorrer para o `exclude_body` significar alguma coisa, e que o artista
    /// **não** está a afinar.
    ///
    /// ⚠️ **Um fato, DOIS consumidores com necessidades opostas:** o cast precisa
    /// nascer no CENTRO (senão a origem cai fora do corpo e o `exclude_body` não
    /// tem o que excluir) e o DESENHO precisa começar na BORDA — medido, um raio
    /// de parede mede 35 px na tela e **20 deles ficam por baixo do contorno do
    /// collider**, deixando um toco de 15 px como tudo o que o artista vê do
    /// número que ele está a mexer. Quem sabe a resposta é esta porta, então ela
    /// carrega as duas; derivá-la do lado do desenho seria a segunda resposta a
    /// *"onde acaba este corpo?"*.
    pub skin: f32,
}

/// **A PERNA** — um raio para baixo, do centro do corpo.
///
/// ⚠️ **O alcance é DERIVADO de propósito** (`float_height + cling_distance`) e
/// não ganha knob próprio: seria a segunda porta para *"até onde a perna
/// alcança"*, e as duas divergiriam. O que ele precisava era de ser **visto**, e
/// é isso que esta porta serve.
///
/// ⚠️ E ele é o alcance que a LEI considera chão, nem um milímetro além:
/// perguntar mais longe faria o cast achar coisas que a lei descartaria, ao
/// preço de descer mais no BVH.
pub(super) fn ground_ray(origin: [f32; 2], cfg: &PlayerConfig) -> ProbeRay {
    ProbeRay {
        origin,
        dir: [0.0, -1.0],
        reach: cfg.ride.float_height + cfg.ride.cling_distance,
        // ⚠️ ZERO, e não a meia-altura do corpo: o `float_height` é medido do
        // CENTRO (é a altura a que o centro cavalga), então o alcance inteiro
        // desta perna é o número autorado. Descontar o corpo aqui desenharia
        // menos do que o artista escreveu.
        skin: 0.0,
    }
}

/// **A PERNA INTEIRA** — os N raios que ela casta, lado a lado (`W-Probes2`).
///
/// ⚠️ **A perna era UM raio no centro, e o preço está medido**
/// (`measure_what_a_single_ground_ray_costs_over_a_gap`): parado sobre uma fenda
/// de 10 cm, num corpo de 40 cm que as bordas suportam fisicamente, ele afunda
/// **0,411 m — 46% do `float_height`**. É a mesma doença que o flanco teve na
/// W13, e a mesma cura.
///
/// As posições saem de [`wall_offsets`] — a MESMA porta do flanco, aplicada ao
/// outro eixo. Uma segunda função com a mesma aritmética divergiria no dia em
/// que a lei do meio-primeiro mudasse num dos dois.
pub(super) fn ground_rays(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    cfg: &PlayerConfig,
    origin: [f32; 2],
) -> ([ProbeRay; MAX_WALL_SAMPLES], usize) {
    let base = ground_ray(origin, cfg);
    let n = odd_samples(cfg.ride.samples, MAX_WALL_SAMPLES);
    let mut out = [base; MAX_WALL_SAMPLES];
    // Sem caixa (corpo recém-nascido) só há o do centro — e é o que sempre houve.
    let Some((mins, maxs)) = world.body_aabb(handle) else {
        return (out, 1);
    };
    let half_width = (maxs[0] - mins[0]) * 0.5;
    if !half_width.is_finite() || half_width <= 0.0 {
        return (out, 1);
    }
    let offs = wall_offsets(half_width, n, cfg.ride.spread);
    for (r, off) in out.iter_mut().zip(offs.iter()).take(n) {
        r.origin = [origin[0] + off, origin[1]];
    }
    (out, n)
}

/// **O FLANCO** — os três raios do sensor lateral, na direção em que o jogador
/// empurra.
///
/// ⚠️ **Do centro, e o alcance inclui a meia-largura**, porque a origem tem de
/// estar DENTRO do corpo para o `exclude_body` significar alguma coisa; o que a
/// lei quer saber é quanto há de parede **além** da borda, e é por isso que quem
/// consome desconta.
///
/// As alturas vêm de [`wall_offsets`] — a porta da LEI, nunca uma aritmética
/// local.
pub(super) fn wall_rays(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    cfg: &PlayerConfig,
    drive: f32,
) -> Option<([ProbeRay; MAX_WALL_SAMPLES], usize)> {
    let side = if drive > 0.0 { 1.0 } else { -1.0 };
    let (mins, maxs) = world.body_aabb(handle)?;
    let half_width = (maxs[0] - mins[0]) * 0.5;
    let half_height = (maxs[1] - mins[1]) * 0.5;
    if !half_width.is_finite() || half_width <= 0.0 || !half_height.is_finite() {
        return None;
    }
    let cx = (maxs[0] + mins[0]) * 0.5;
    let cy = (maxs[1] + mins[1]) * 0.5;
    let reach = half_width + cfg.wall.reach.max(0.0);
    let n = odd_samples(cfg.wall.samples, MAX_WALL_SAMPLES);
    let offs = wall_offsets(half_height.max(0.0), n, cfg.wall.spread);
    let mut out = [ProbeRay {
        origin: [cx, cy],
        dir: [side, 0.0],
        reach,
        skin: half_width,
    }; MAX_WALL_SAMPLES];
    for (r, off) in out.iter_mut().zip(offs.iter()).take(n) {
        r.origin = [cx, cy + off];
    }
    Some((out, n))
}

/// **A BEIRADA** — um raio para BAIXO, à frente da cabeça (`W-Ledge`).
///
/// ⚠️ **Um raio, e não uma varredura**, e a §4.3 do plano 08 já o dizia: *onde o
/// chão acaba* é uma pergunta de **perfil**, e uma varredura devolve **um**
/// contacto sem saber a que altura ele está. O que a beirada precisa é
/// exactamente a altura.
///
/// ⚠️ **A origem nasce `reach_y` ACIMA da cabeça e `meia-largura + grab` à
/// FRENTE**, e o alcance é `2·reach_y` — as duas soleiras do
/// [`ph2d_platformer::LedgeProbe`] saem daí, e o `x` em que ele bate **é** o
/// ponto do patamar em que o corpo vai pousar.
///
/// # ⚠️ Um LEQUE, e o molde é o [`wall_rays`] logo acima
///
/// Com [`ph2d_platformer::LedgeConfig::span`] em zero são **uma** amostra, na
/// posição exacta do raio de antes da `W-LedgeSensor` — é isso que mantém o
/// mundo aprovado byte-idêntico. Acima disso o sensor é um SEGMENTO centrado no
/// `grab`, e as amostras saem em ordem **CRESCENTE de afastamento**, porque o
/// consumidor quer a beirada mais PERTO do corpo e um laço que percorre por
/// ordem responde isso sem ordenar nada.
///
/// ⚠️ **O offset é aparado em zero**, não em algum épsilon: `off = 0` põe o raio
/// rente à face do corpo, que é uma pergunta legítima (um patamar encostado
/// nele) — e o cast já exclui o próprio corpo.
pub(super) fn ledge_rays(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    cfg: &PlayerConfig,
    side: f32,
) -> Option<([ProbeRay; ph2d_platformer::LEDGE_SPAN_SAMPLES], usize)> {
    let grab = cfg.ledge.grab;
    let reach_y = cfg.ledge.reach_y;
    if !grab.is_finite() || grab <= 0.0 || !reach_y.is_finite() || reach_y <= 0.0 {
        return None;
    }
    let (mins, maxs) = world.body_aabb(handle)?;
    let half_width = (maxs[0] - mins[0]) * 0.5;
    if !half_width.is_finite() || half_width <= 0.0 {
        return None;
    }
    let cx = (maxs[0] + mins[0]) * 0.5;
    let span = if cfg.ledge.span.is_finite() {
        cfg.ledge.span.max(0.0)
    } else {
        0.0
    };
    let n = if span > 0.0 {
        ph2d_platformer::LEDGE_SPAN_SAMPLES
    } else {
        1
    };
    let base = ProbeRay {
        origin: [cx + side * half_width, maxs[1] + ledge_origin_rise(cfg)],
        dir: [0.0, -1.0],
        reach: 2.0 * reach_y,
        // ⚠️ **ZERO, e não a meia-altura:** este raio nasce FORA do corpo, à
        // frente dele — não há nada dele a descontar do desenho.
        skin: 0.0,
    };
    let mut out = [base; ph2d_platformer::LEDGE_SPAN_SAMPLES];
    for (i, r) in out.iter_mut().enumerate().take(n) {
        r.origin[0] = cx + side * (half_width + ledge_offset(grab, span, n, i));
    }
    Some((out, n))
}

/// **Quanto acima do topo do corpo a origem do leque nasce**, metros —
/// `reach_y + offset_y`, a PORTA ÚNICA da geometria vertical.
///
/// ⚠️ **Três consumidores** (a origem do raio · o `lip_rise` que a lei lê · a
/// distância que o overlay desenha), e uma cópia a mais divergiria no primeiro
/// dia em que alguém mexesse num dos dois números: o desenho mostraria um
/// alcance e o solver usaria outro.
///
/// ⚠️ **Com `offset_y = 0` reduz LITERALMENTE ao `reach_y`** — o mundo de antes
/// do quarto controlo, ao bit.
pub(super) fn ledge_origin_rise(cfg: &PlayerConfig) -> f32 {
    let off = if cfg.ledge.offset_y.is_finite() {
        cfg.ledge.offset_y
    } else {
        0.0
    };
    cfg.ledge.reach_y + off
}

/// **O afastamento da amostra `i`**, medido a partir da face do corpo.
///
/// ⚠️ Com `n == 1` devolve o `grab` NU — a redução literal que torna o mundo de
/// antes desta wave byte-idêntico, sem um caso especial escrito à mão.
pub(super) fn ledge_offset(grab: f32, span: f32, n: usize, i: usize) -> f32 {
    if n <= 1 {
        return grab.max(0.0);
    }
    let t = i as f32 / (n - 1) as f32;
    (grab - span * 0.5 + span * t).max(0.0)
}

/// **O que há por cima da beirada** (`W-Ledge`) — a metade do sensor que este
/// módulo possui, e nada de política.
///
/// ⚠️ **`distance == 0` é uma RECUSA, e é o achado que a medição deu:** a origem
/// caiu **dentro** da geometria, ou seja a parede continua acima da cabeça e não
/// há beirada nenhuma. Sem esta linha o cast devolve o contrato de penetração
/// (`distance == 0`, publicado pelo `cast_ray`) e a lei leria um lábio à altura
/// exacta da origem — medido, com o corpo 0,4 m abaixo do lábio e uma janela de
/// 0,2 m ele reportava um patamar em `y = 3,3`, **onde não há superfície
/// nenhuma**. É esta a metade *"livre acima da cabeça"* que os motores de
/// referência pagam com um segundo raio; aqui ela sai do mesmo.
///
/// ⚠️ **E com EXTENSÃO ela deixa de ser grátis, então é feita à mão** — era
/// grátis porque o sensor era um PONTO. Num leque, **uma amostra dentro da
/// geometria recusa o leque INTEIRO**: se a parede continua acima da cabeça
/// junto ao corpo, não há beirada a apanhar por mais livre que esteja uma
/// amostra lá à frente. É o traço de folga que o mantle do Unreal paga em
/// separado, aqui devolvido à mesma varredura.
///
/// ⚠️ **Vence o acerto mais PERTO do corpo** — aproximando-se de um patamar as
/// amostras de dentro caem no vazio e as de fora batem no topo, logo o acerto
/// mais próximo **é a beirada**. A varredura tem de ver TODAS as amostras
/// (a recusa acima é sobre qualquer uma), então o vencedor é o primeiro acerto
/// em ordem de afastamento, e não um `?` no meio do laço.
pub(super) fn probe_ledge(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    layer: u8,
    cfg: &PlayerConfig,
    drive: f32,
) -> Option<ph2d_platformer::LedgeProbe> {
    let side = if drive > 0.0 { 1.0 } else { -1.0 };
    let (rays, n) = ledge_rays(world, handle, cfg, side)?;
    let (mins, maxs) = world.body_aabb(handle)?;
    let half_width = (maxs[0] - mins[0]) * 0.5;
    let half_height = (maxs[1] - mins[1]) * 0.5;
    let span = if cfg.ledge.span.is_finite() {
        cfg.ledge.span.max(0.0)
    } else {
        0.0
    };
    let mut won: Option<(f32, f32)> = None;
    for (i, r) in rays.iter().enumerate().take(n) {
        let Some(hit) = world.cast_ray(r.origin, r.dir, r.reach, Some(handle), layer) else {
            continue;
        };
        if hit.distance <= 0.0 {
            return None;
        }
        if won.is_none() {
            won = Some((hit.distance, ledge_offset(cfg.ledge.grab, span, n, i)));
        }
    }
    let (distance, off) = won?;
    let lip_rise = ledge_origin_rise(cfg) - distance;
    Some(ph2d_platformer::LedgeProbe {
        lip_rise,
        side,
        // ⚠️ **A borda de DENTRO do corpo aterra no `x` que o raio provou ser
        // patamar** — é isso que torna o alvo do mantle medido em vez de
        // suposto, e é por isso que ele não precisa de saber onde está a face
        // da parede. Com extensão, o `x` é o da amostra VENCEDORA.
        across: 2.0 * half_width + off,
        // ⚠️ **A MESMA medição, projetada para o outro consumidor** — ver o doc
        // de [`ph2d_platformer::LedgeProbe::rise`].
        rise: lip_rise + half_height + cfg.ride.float_height,
    })
}

/// **Onde o perfil do teto nasce** — a geometria que o leque e as duas laterais
/// partilham.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct CornerGeom {
    /// O meio do TOPO da caixa do corpo, em mundo.
    pub top: [f32; 2],
    /// A meia-altura do corpo — de onde saem os raios laterais.
    pub mid_y: f32,
    pub half_width: f32,
    /// O alcance lateral autorado (`corner_reach`).
    pub reach: f32,
    /// Quanto cada raio do leque sobe.
    pub rise: f32,
}

/// **A QUINA** — o perfil sobre a cabeça e a folga dos lados.
///
/// O leque mede `rel_up · dt · [`CORNER_LOOKAHEAD`]` — a distância que a cabeça
/// sobe no PRÓXIMO tique. É isso, e só isso, que torna a assistência preditiva:
/// o que o perfil descreve é um contato que ainda não aconteceu.
///
/// ⚠️ **`rise == 0` é ACEITE**, e é o desenho: um sensor que não está a subir
/// não tem leque nenhum, mas continua a ter o **vão lateral** que o artista
/// afina (`corner_reach`). Recusar aqui deixaria a `W-Probes` sem nada a mostrar
/// em todo tique em que o personagem não sobe — que são quase todos. Quem casta
/// é guardado pelo `corner_probe_wanted`, que já exige `rel_up > 0`.
pub(super) fn corner_geom(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    cfg: &PlayerConfig,
    rise: f32,
) -> Option<CornerGeom> {
    if !rise.is_finite() || rise < 0.0 {
        return None;
    }
    let (mins, maxs) = world.body_aabb(handle)?;
    let half_width = (maxs[0] - mins[0]) * 0.5;
    if !half_width.is_finite() || half_width <= 0.0 {
        return None;
    }
    Some(CornerGeom {
        top: [(maxs[0] + mins[0]) * 0.5, maxs[1]],
        mid_y: (maxs[1] + mins[1]) * 0.5,
        half_width,
        reach: cfg.jump.corner_reach,
        rise,
    })
}

/// **Quanto o leque do teto sobe** — a distância que a cabeça percorre no
/// PRÓXIMO tique.
///
/// ⚠️ Porta, e não uma multiplicação repetida: quem casta e quem **desenha**
/// precisam do mesmo número, e um leque desenhado com outra altura mostraria uma
/// antecipação que a lei não tem.
pub(super) fn corner_rise(rel_up: f32, dt: f32, cfg: &PlayerConfig) -> f32 {
    rel_up * dt * cfg.jump.corner_lookahead
}

/// **O AGACHAR** — para onde o corpo é varrido, e quanto.
///
/// ⚠️ Um deslocamento e não um raio: o que este sensor pergunta é *"o corpo cabe
/// se subir isto?"*, e a forma que responde é o corpo inteiro no destino. `None`
/// quando não há subida nenhuma a fazer.
pub(super) fn headroom_offset(cfg: &PlayerConfig) -> Option<[f32; 2]> {
    let rise = cfg.crouch.rise(&cfg.ride);
    (rise.is_finite() && rise > 0.0).then_some([0.0, rise])
}

/// **O perfil do teto acima da cabeça** (W10) — a metade do sensor que este
/// módulo possui, e nada de política.
///
/// ⚠️ **A grade de deslocamentos vem da lei** ([`corner_offsets`]), nunca de uma
/// aritmética local. Duas cópias deslocariam o perfil de meia célula em relação
/// ao corpo, e o sintoma seria um personagem empurrado para dentro da quina de
/// que ele fugia.
///
/// ⚠️ **A folga lateral é medida do CENTRO e descontada a meia-largura**, porque
/// o raio nasce no centro (a origem tem de estar dentro do corpo para o
/// `exclude_body` fazer sentido) e o que a lei quer saber é quanto há de espaço
/// **além** da borda. Saturada no próprio alcance: para esta decisão, *livre até
/// onde eu poderia querer ir* e *livre até o infinito* são a mesma coisa.
pub(super) fn probe_ceiling(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    layer: u8,
    cfg: &PlayerConfig,
    rel_up: f32,
    dt: f32,
    assist: bool,
) -> Option<CeilingProbe> {
    let g = corner_geom(world, handle, cfg, corner_rise(rel_up, dt, cfg))?;

    // ── O FATO (W-Ceiling) ───────────────────────────────────────────────────
    // ⚠️ **A régua é o que a cabeça percorre NESTE tique**, sem o
    // `corner_lookahead` que o leque acima usa: a antecipação pertence à
    // assistência, e um fato que a herdasse diria *"bateu"* um tique antes de
    // bater. É por isso que o número sai daqui e não do `corner_rise`.
    //
    // ⚠️ **Varredura da FORMA, não um raio** — o corpo pode ser composto, e um
    // raio pelo centro atravessaria uma marquise que só o ombro encosta. É o
    // mesmo primitivo do `probe_headroom`, pela mesma razão.
    let head_blocked = world
        .sweep_body(handle, [0.0, 1.0], (rel_up * dt).max(0.0), layer)
        .is_some();

    // ⚠️ **O leque é da ASSISTÊNCIA e só é varrido sob o knob dela.** Com o
    // alcance em zero o `samples` fica `0` — que o campo já documenta como *"não
    // perguntei"* — e o `corner_escape` nunca é alcançado, porque quem o gateia
    // é o `corner_probe_wanted`, que exige o mesmo alcance. Varrer aqui de
    // qualquer maneira faria uma ajuda desligada custar N raios.
    let n = if assist {
        odd_samples(cfg.jump.corner_samples, MAX_CORNER_SAMPLES)
    } else {
        0
    };
    let mut blocked = [false; MAX_CORNER_SAMPLES];
    for (slot, off) in blocked
        .iter_mut()
        .zip(corner_offsets(g.half_width, g.reach, n.max(1)).iter())
        .take(n)
    {
        *slot = world
            .cast_ray(
                [g.top[0] + off, g.top[1]],
                [0.0, 1.0],
                g.rise,
                Some(handle),
                layer,
            )
            .is_some();
    }

    let free = |dir: f32| {
        world
            .cast_ray(
                [g.top[0], g.mid_y],
                [dir, 0.0],
                g.half_width + g.reach,
                Some(handle),
                layer,
            )
            .map_or(g.reach, |h| (h.distance - g.half_width).clamp(0.0, g.reach))
    };

    // ⚠️ Os dois raios laterais são do ESCAPE (para onde livrar a cabeça), não do
    // fato — com a assistência desarmada eles seriam custo por uma resposta que
    // ninguém lê.
    let side_clear = if assist {
        [free(-1.0), free(1.0)]
    } else {
        [0.0, 0.0]
    };

    Some(CeilingProbe {
        half_width: g.half_width,
        blocked,
        side_clear,
        samples: n,
        head_blocked,
    })
}

/// **O que há sobre a cabeça** (W15) — a varredura que decide se ele pode
/// levantar-se de um agachar.
///
/// O corpo é varrido para CIMA por exactamente
/// [`ph2d_platformer::CrouchConfig::rise`] — *quanto ele SOBE ao levantar-se*.
/// Nem um milímetro além: perguntar mais longe recusaria o gesto por causa de um
/// teto que o personagem, de pé, não alcança.
///
/// # ⚠️ Eram TRÊS RAIOS, e a `W-ShapeCast` trocou-os por UMA varredura
///
/// A pergunta é *"o corpo cabe se subir `rise`?"*, e uma grade de linhas só sabe
/// respondê-la por amostragem. O preço estava medido
/// (`measure_the_gap_between_rays`): um pilar de 8 cm entre duas amostras é
/// invisível, a cabeça chega a **1,267** contra pedra em **1,25** e o solver
/// segura-a lá dentro.
///
/// ⚠️ **E some com ele a caixa envolvente:** a varredura usa a forma REAL do
/// collider, então a ressalva *"um teto que toque só a quina da caixa recusa o
/// levantar"* deixa de existir — não é uma tolerância que alguém afrouxou, é uma
/// pergunta que passou a ser feita sobre o corpo em vez de sobre a caixa dele.
///
/// ⚠️ **A dívida de gate que isto PAGA:** o doc anterior nomeava uma metade sem
/// gate (*"o laço percorre os TRÊS deslocamentos" só é observável sob um teto
/// parcial … trocar o laço por um raio central sobrevive à suíte*). Não há laço
/// a percorrer: a varredura não tem amostras para alguém esquecer.
///
/// ⚠️ **`sweep_body` já exclui o próprio corpo**, então não há aqui a aritmética
/// do "nasce no centro e desconta meia-largura" que os raios precisavam.
pub(super) fn probe_headroom(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    layer: u8,
    cfg: &PlayerConfig,
) -> Option<Headroom> {
    let up = headroom_offset(cfg)?;
    Some(Headroom {
        blocked: world.sweep_body(handle, [0.0, 1.0], up[1], layer).is_some(),
    })
}

/// **A parede ao lado** (W13) — a metade do sensor que este módulo possui, e
/// nada de política.
///
/// Os raios saem de [`wall_rays`]; aqui só se castam e se guarda o que voltou.
///
/// ⚠️ **O FLANCO INTEIRO, não só a cintura** — a altura de cada raio vem de
/// [`wall_offsets`], que é a porta única (o `crouch` e a quina têm as suas, pela
/// mesma razão). A versão de um raio só media a meia-altura, e o preço disso
/// está MEDIDO no `measure_wall_flank`: uma parede com fresta de 0,75 m, num
/// corpo de 1,0 m, **recusava o pulo de parede por inteiro** com 12,5 cm de pé e
/// de ombro ainda encostados.
///
/// ⚠️ **Fica o hit mais PRÓXIMO, e o meio desempata** (é o primeiro da lista):
/// numa parede plana os três raios medem a mesma distância, então a resposta é a
/// de sempre — o que muda é só onde o meio sozinho não via nada. E *mais
/// próximo* é o que um corpo empurrado de lado de facto encosta primeiro.
///
/// ⚠️ **Este módulo NÃO decide se aquilo é parede, NEM qual das amostras vale.**
/// Ele entrega o array inteiro e a lei escolhe (`ph2d_platformer::cling`) — o
/// padrão exato dos outros dois sensores multi-amostra desta ponte
/// ([`ph2d_platformer::Headroom`], [`ph2d_platformer::CeilingProbe`]). Reduzir
/// aqui seria uma segunda régua ao lado da que a perna já usa, e as duas
/// divergiriam no dia em que o `max_slope` autorado se movesse.
pub(super) fn probe_wall(
    world: &ph2d_physics::PhysicsWorld,
    handle: rapier2d_handle::Handle,
    layer: u8,
    cfg: &PlayerConfig,
    drive: f32,
) -> Option<WallProbe> {
    let (rays, n) = wall_rays(world, handle, cfg, drive)?;
    let mut hits = [None; MAX_WALL_SAMPLES];
    for (slot, r) in hits.iter_mut().zip(rays.iter()).take(n) {
        *slot = world
            .cast_ray(r.origin, r.dir, r.reach, Some(handle), layer)
            .map(|hit| WallHit {
                distance: hit.distance,
                normal: hit.normal,
            });
    }
    Some(WallProbe {
        side: rays[0].dir[0],
        hits,
        samples: n,
    })
}
