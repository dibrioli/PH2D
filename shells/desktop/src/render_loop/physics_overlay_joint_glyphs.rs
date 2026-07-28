//! **Os GLIFOS de um joint — uma figura por tipo (W-J1).**
//!
//! Um joint não tem geometria: é uma relação. Até esta wave os quatro tipos
//! desenhavam a MESMA figura (segmento + dois anéis), então o canvas dizia
//! *"há um joint aqui"* e nada mais — nem qual, nem com que alcance, nem se a
//! restrição está trabalhando. Godot desenha uma cruz de 20 px, Fyrox não
//! desenha nada, Unreal desenha um cone que ninguém arrasta; o RUBE desenha
//! TODO fato como geometria, e é dele que este vocabulário vem.
//!
//! ## Duas réguas, e a diferença é FÍSICA, não gosto
//!
//! - **Comprimento é comprimento** ⇒ o anel de repouso/máximo é construído em
//!   MUNDO e projetado: dar zoom dobra o raio dele, como dobra a bola que ele
//!   descreve.
//! - **Ângulo e ornamento não são comprimento** ⇒ o arco de limite, o quadrado
//!   do weld, a amplitude do zigue-zague e as pontas são pixels de TELA
//!   constantes. É a mesma lei que o [`super::physics_overlay_annotations`]
//!   enuncia para a seta de força.
//!
//! ⚠️ **Toda figura angular é construída numa BASE derivada da câmera**
//! (`û` = onde o +x de mundo cai na tela, `ŵ` = o +y), nunca em ângulos de tela
//! escritos à mão: assim um limite anti-horário desenha o lado para o qual o
//! corpo VISIVELMENTE gira, sob qualquer y-flip — a cicatriz que o glifo de
//! torque da zona já pagou.

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

/// Cordas num arco — o mesmo número que o glifo de torque usa, pela mesma razão
/// (270° têm de ler como curva, não como polígono).
///
/// ⚠️ Compartilhado com o arco da CORDA na roldana (`physics_overlay_pulley`):
/// os dois desenham o mesmo círculo, e um arco mais grosseiro que o aro dele
/// apareceria como a corda descolando da roda.
pub(super) const ARC_SEGS: u32 = 24;

/// Raio do anel do PINO, px de tela. Maior que a ponta (`JOINT_DOT_PX`) porque
/// ele é o GLIFO — a figura que diz *dobradiça* — e não a marca de uma ponta.
pub(super) const PIN_RING_PX: f64 = 6.5; // LITERAL-PX-OK: chrome de overlay

/// Meia-diagonal do quadrado do WELD, px de tela.
pub(super) const WELD_HALF_PX: f64 = 5.5; // LITERAL-PX-OK: chrome de overlay

/// Meia-largura da BARRA, px de tela. Menor que o anel do pino (`PIN_RING_PX`)
/// para as duas paralelas passarem POR DENTRO dos anéis das pontas em vez de
/// cruzá-los — a barra sai dos olhais, que é o que um elo faz.
pub(super) const ROD_HALF_PX: f64 = 2.5; // LITERAL-PX-OK: chrome de overlay

/// Raio do arco de LIMITE, px de tela — fora do anel do pino, que ele cerca.
pub(super) const LIMIT_ARC_PX: f64 = 21.0; // LITERAL-PX-OK: chrome de overlay

/// Comprimento das marcas radiais nas pontas do arco de limite (as "paredes").
const LIMIT_TICK_PX: f64 = 6.0; // LITERAL-PX-OK: chrome de overlay

/// Amplitude do zigue-zague da MOLA, px de tela.
const SPRING_AMP_PX: f64 = 5.0; // LITERAL-PX-OK: chrome de overlay

/// Quantas meias-ondas o zigue-zague tem. Ímpar não fecha simétrico; par sim.
const SPRING_HUMPS: u32 = 8;

/// A projeção mundo→tela deste módulo, num lugar só.
pub(super) fn screen_of(camera: &Camera2d, window: WindowSize, w: [f32; 2]) -> Point {
    let (sx, sy) = camera.world_to_screen(w, window);
    Point::new(f64::from(sx), f64::from(sy))
}

/// A base de tela da câmera: `(û, ŵ)` normalizados, onde `û` é o +x de mundo e
/// `ŵ` o +y. Um ângulo de mundo `a` cai em `centro + R·(cos a·û + sen a·ŵ)`, e
/// `a` crescente traça anti-horário DE MUNDO na tela — seja qual for o flip.
fn screen_basis(
    camera: &Camera2d,
    window: WindowSize,
    centre: [f32; 2],
) -> (Point, (f64, f64), (f64, f64)) {
    let c = screen_of(camera, window, centre);
    let px = screen_of(camera, window, [centre[0] + 1.0, centre[1]]);
    let py = screen_of(camera, window, [centre[0], centre[1] + 1.0]);
    let norm = |vx: f64, vy: f64| {
        let l = vx.hypot(vy);
        if l < 1e-9 {
            (0.0, 0.0)
        } else {
            (vx / l, vy / l)
        }
    };
    (
        c,
        norm(px.x - c.x, px.y - c.y),
        norm(py.x - c.x, py.y - c.y),
    )
}

/// Um ponto sobre um arco, dada a base de tela já resolvida — o primitivo de
/// TODA figura angular deste módulo.
fn arc_point_in(c: Point, u: (f64, f64), v: (f64, f64), ang: f64, r: f64) -> Point {
    Point::new(
        c.x + r * (ang.cos() * u.0 + ang.sin() * v.0),
        c.y + r * (ang.cos() * u.1 + ang.sin() * v.1),
    )
}

/// **Onde uma parede do arco de limite está, em px de tela** (W-J3).
///
/// A MESMA função que [`limit_arc`] usa para desenhar a marca radial — o grip
/// que o artista arrasta e a parede que ele vê são um lugar só. Duas derivações
/// desta posição seriam duas respostas a *"onde termina o alcance?"*, e a que
/// discordasse seria justamente a invisível: o retângulo de hit.
pub(super) fn limit_end_screen(
    camera: &Camera2d,
    window: WindowSize,
    centre_w: [f32; 2],
    angle_a: f32,
    limit: f32,
) -> Point {
    let (c, u, v) = screen_basis(camera, window, centre_w);
    arc_point_in(c, u, v, f64::from(angle_a) + f64::from(limit), LIMIT_ARC_PX)
}

/// Meia-extensão do trilho de um Slider SEM limites, px de tela — ele diz só uma
/// DIREÇÃO, então o comprimento é do chrome. Um pouco maior que o arco de limite
/// do Pin, porque uma reta precisa de mais extensão que um arco para ler como
/// eixo em vez de risco.
const RAIL_PX: f64 = 26.0; // LITERAL-PX-OK: chrome de overlay

/// Meio-comprimento dos tracinhos de fim de curso, px de tela — irmãos dos
/// `LIMIT_TICK_PX` do arco (é a mesma figura: *o alcance termina aqui*), um
/// pouco maiores porque cruzam uma reta e não a ponta de um arco.
const RAIL_TICK_PX: f64 = 7.0; // LITERAL-PX-OK: chrome de overlay

/// **O TRILHO de um Slider** (W-J5) — o desenho canônico do prismatic: uma reta
/// pelo eixo, com tracinhos perpendiculares nos fins de curso.
///
/// ⚠️ **Os fins de curso são MUNDO, o resto é tela.** Um curso é uma distância em
/// metros, então os tracinhos têm de sentar onde o corpo vai de fato parar — eles
/// crescem com o zoom junto com a cena. Já a espessura, o comprimento dos
/// tracinhos e a extensão de um trilho ILIMITADO são chrome: sem limites o eixo
/// não tem comprimento nenhum a mostrar, e inventar um em metros seria desenhar
/// um número que não existe.
///
/// `axis_w` é a direção unitária de mundo que a ponte resolveu
/// (`JointView::axis`), nunca uma re-derivação daqui: duas respostas a *"para onde
/// este trilho aponta?"* desenhariam um eixo que o solver não usa.
pub(super) fn slider_rail(
    camera: &Camera2d,
    window: WindowSize,
    anchor_w: [f32; 2],
    axis_w: [f32; 2],
    limits: Option<[f32; 2]>,
) -> BezPath {
    let mut p = BezPath::new();
    let along = |t: f32| {
        screen_of(
            camera,
            window,
            [anchor_w[0] + axis_w[0] * t, anchor_w[1] + axis_w[1] * t],
        )
    };
    // A direção do eixo JÁ PROJETADA, de onde sai a perpendicular de tela. Sai de
    // dois pontos do próprio eixo, então honra qualquer flip da câmera sem saber
    // que ele existe.
    let c = along(0.0);
    let ahead = along(1.0);
    let (dx, dy) = (ahead.x - c.x, ahead.y - c.y);
    let len = dx.hypot(dy);
    if len < 1e-9 {
        return p; // câmera degenerada: nada honesto a desenhar
    }
    let (ux, uy) = (dx / len, dy / len);
    let (perp_x, perp_y) = (-uy, ux);
    let (from, to) = match limits {
        // Com curso: a reta vai de ponta a ponta do alcance REAL.
        Some([min, max]) => (along(min), along(max)),
        // Sem curso: só a direção, num comprimento de chrome.
        None => (
            Point::new(c.x - ux * RAIL_PX, c.y - uy * RAIL_PX),
            Point::new(c.x + ux * RAIL_PX, c.y + uy * RAIL_PX),
        ),
    };
    p.move_to(from);
    p.line_to(to);
    // Os tracinhos existem só quando há fim de curso — sem limites não há onde
    // parar, e um tracinho ali afirmaria um limite que não existe.
    if limits.is_some() {
        end_ticks([from, to], (perp_x, perp_y), &mut p);
    }
    p
}

/// **Os tracinhos de FIM DE CURSO**, perpendiculares ao eixo, em px de tela.
///
/// Porta única do trilho ([`slider_rail`]) e da suspensão ([`wheel_strut`]):
/// os dois dizem a MESMA coisa — *o alcance termina aqui* — e duas cópias
/// divergiriam no dia em que o comprimento do tracinho mudasse num só.
fn end_ticks(ends: [Point; 2], perp: (f64, f64), p: &mut BezPath) {
    for end in ends {
        p.move_to(Point::new(
            end.x - perp.0 * RAIL_TICK_PX,
            end.y - perp.1 * RAIL_TICK_PX,
        ));
        p.line_to(Point::new(
            end.x + perp.0 * RAIL_TICK_PX,
            end.y + perp.1 * RAIL_TICK_PX,
        ));
    }
}

/// **O glifo da RODA** — o cubo (um anel) mais a MOLA da suspensão ao longo do
/// eixo, e os batentes de curso quando eles existem.
///
/// ⚠️ **É COMPOSTO de propósito, e é isso que o torna legível:** o anel é o
/// mesmo do [`pin_glyph`] e significa a mesma coisa (*isto gira em torno deste
/// ponto*), o zigue-zague é o mesmo idioma do [`spring_zigzag`] e diz *isto
/// cede*, e os tracinhos são os do [`slider_rail`]. Uma roda é literalmente as
/// três frases juntas, então inventar uma quarta figura seria pedir ao artista
/// que decorasse um símbolo em vez de ler o que já sabe.
///
/// A distinção contra os vizinhos é de GEOMETRIA, não de nome (a cicatriz do
/// par `Layer`/`Layers`): contra o Pin, existe uma polilinha que oscila fora do
/// anel; contra a Spring, há um anel FECHADO e o zigue-zague não liga duas
/// âncoras (as duas coincidem numa roda); contra o Slider, o traço ao longo do
/// eixo não é reto.
///
/// Como o [`slider_rail`], o eixo vem da ponte (`JointView::axis`) e nunca é
/// re-derivado aqui, e os fins de curso são MUNDO enquanto o resto é chrome.
pub(super) fn wheel_strut(
    camera: &Camera2d,
    window: WindowSize,
    anchor_w: [f32; 2],
    axis_w: [f32; 2],
    limits: Option<[f32; 2]>,
) -> BezPath {
    let mut p = BezPath::new();
    let along = |t: f32| {
        screen_of(
            camera,
            window,
            [anchor_w[0] + axis_w[0] * t, anchor_w[1] + axis_w[1] * t],
        )
    };
    let c = along(0.0);
    let ahead = along(1.0);
    let (dx, dy) = (ahead.x - c.x, ahead.y - c.y);
    let len = dx.hypot(dy);
    if len < 1e-9 {
        return p; // câmera degenerada: nada honesto a desenhar
    }
    let (ux, uy) = (dx / len, dy / len);
    let (perp_x, perp_y) = (-uy, ux);
    // O CUBO: o mesmo anel do pino, porque é a mesma afirmação.
    ring_px(c, PIN_RING_PX, &mut p);
    // A MOLA: da borda do anel para fora, ao longo do eixo, num comprimento de
    // chrome — o curso de uma suspensão é um número em metros e ele já tem
    // desenho próprio (os tracinhos); esticar a mola até lá faria a figura
    // sumir num curso curto.
    let base = Point::new(c.x + ux * PIN_RING_PX, c.y + uy * PIN_RING_PX);
    let tip = Point::new(c.x + ux * RAIL_PX, c.y + uy * RAIL_PX);
    p.extend(spring_zigzag(base, tip).iter());
    // Os batentes: onde a suspensão de fato PARA, em metros. ⚠️ Comprimir é
    // POSITIVO neste eixo (o cubo sobe em direção ao chassi), então o `max` é o
    // batente de compressão e o `min` o de extensão — medido, porque uma faixa
    // simétrica não distingue as duas convenções. O par inteiro é o que um rod
    // não podia ter.
    if let Some([min, max]) = limits {
        end_ticks([along(min), along(max)], (perp_x, perp_y), &mut p);
    }
    p
}

/// **Onde o grip do anel de comprimento está, em MUNDO** (W-J3).
///
/// Sobre o anel, na direção de `toward` (a âncora B) — onde o artista já está
/// olhando, e onde o próprio anel responde *"B está no comprimento?"*. Mundo, e
/// não tela, porque um comprimento é um comprimento: o grip cresce com o zoom
/// junto com o anel que ele agarra.
///
/// Degenerado (B sobre A) cai no ponto onde [`ring_px`] começa a desenhar o anel,
/// que é o único lugar sem escolha arbitrária.
pub(super) fn length_handle_world(centre_w: [f32; 2], toward: [f32; 2], length: f32) -> [f32; 2] {
    let (dx, dy) = (toward[0] - centre_w[0], toward[1] - centre_w[1]);
    let d = dx.hypot(dy);
    let (ux, uy) = if d < 1e-6 {
        (1.0, 0.0)
    } else {
        (dx / d, dy / d)
    };
    [centre_w[0] + ux * length, centre_w[1] + uy * length]
}

/// Um anel de raio em PIXELS DE TELA, centrado num ponto de tela.
pub(super) fn ring_px(centre: Point, radius_px: f64, path: &mut BezPath) {
    path.move_to(Point::new(centre.x + radius_px, centre.y));
    for i in 1..=ARC_SEGS {
        let th = std::f64::consts::TAU * f64::from(i) / f64::from(ARC_SEGS);
        path.line_to(Point::new(
            centre.x + radius_px * th.cos(),
            centre.y + radius_px * th.sin(),
        ));
    }
}

/// **O glifo do PINO** — um anel: a dobradiça, a figura que todo editor 2D usa
/// para "estes dois giram em torno deste ponto".
pub(super) fn pin_glyph(centre: Point) -> BezPath {
    let mut p = BezPath::new();
    ring_px(centre, PIN_RING_PX, &mut p);
    p
}

/// **O glifo do WELD** — um quadrado, girado com o corpo A.
///
/// Quadrado e não anel porque um weld **não gira**: a figura tem quinas, e as
/// quinas seguem a rotação do corpo, então o artista vê o par travado virar
/// junto. ⚠️ A distinção Pin×Weld é de GEOMETRIA, nunca de identificador — a
/// cicatriz do par `Layer`/`Layers` da timeline, reprovado num smoke por serem
/// a mesma figura com nomes diferentes.
pub(super) fn weld_glyph(
    camera: &Camera2d,
    window: WindowSize,
    centre_w: [f32; 2],
    angle_a: f32,
) -> BezPath {
    let (c, u, v) = screen_basis(camera, window, centre_w);
    let a = f64::from(angle_a);
    let at = |ang: f64| arc_point_in(c, u, v, ang, WELD_HALF_PX);
    let mut p = BezPath::new();
    let corners = [0.25, 0.75, 1.25, 1.75].map(|k: f64| at(a + k * std::f64::consts::PI));
    p.move_to(corners[3]);
    for c in corners {
        p.line_to(c);
    }
    p
}

/// **O glifo da MOLA** — o zigue-zague entre as âncoras.
///
/// Ele É o span (não há figura separada de âncora), e é isso que o distingue da
/// corda: um zigue-zague inverte de direção a cada meia-onda, uma corda não
/// inverte nunca. Um gate mede exatamente essa propriedade.
pub(super) fn spring_zigzag(a: Point, b: Point) -> BezPath {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx.hypot(dy);
    let mut p = BezPath::new();
    p.move_to(a);
    if len < 1e-6 {
        // Mola de span zero: nada a serpentear. As pontas do chamador é que
        // mantêm o joint visível — a mesma razão dos anéis do desenho antigo.
        return p;
    }
    let (ux, uy) = (dx / len, dy / len);
    let (nx, ny) = (-uy, ux);
    for i in 1..SPRING_HUMPS {
        let t = f64::from(i) / f64::from(SPRING_HUMPS);
        let s = if i % 2 == 0 { -1.0 } else { 1.0 };
        p.line_to(Point::new(
            a.x + dx * t + nx * SPRING_AMP_PX * s,
            a.y + dy * t + ny * SPRING_AMP_PX * s,
        ));
    }
    p.line_to(b);
    p
}

/// **O glifo da CORDA** — reta quando TESA, curva quando FROUXA.
///
/// A flecha (`sag`) sai da aproximação parabólica de um fio pendurado: uma
/// parábola de flecha `s` sobre corda `d` tem comprimento `d·(1 + 8/3·(s/d)²)`,
/// então igualar isso ao comprimento `L` da corda dá `s = d·√(3/8·(L/d − 1))`.
/// É um READOUT da folga, não uma simulação: o rapier trata a corda como um
/// LIMITE de distância e não pendura fio nenhum. ⚠️ **A direção da barriga é a
/// da GRAVIDADE** — a mesma fonte que decide para onde é "para cima" no empuxo
/// (W-Buoyancy); sem gravidade a corda desenha reta, degenerado que a física
/// resolve sozinha.
pub(super) fn rope_span(
    a: Point,
    b: Point,
    slack_ratio: f64,
    gravity_screen: (f64, f64),
) -> BezPath {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let d = dx.hypot(dy);
    let mut p = BezPath::new();
    p.move_to(a);
    // Tesa (ou sem gravidade para pendurar): uma reta, que é o que uma corda
    // sob carga é.
    if slack_ratio <= 0.0 || d < 1e-6 || gravity_screen.0.hypot(gravity_screen.1) < 1e-9 {
        p.line_to(b);
        return p;
    }
    // `slack_ratio` = L/d − 1, já calculado pelo chamador (que é quem tem os
    // metros); o cap impede que uma corda muito frouxa desenhe uma barriga
    // maior que ela mesma.
    let sag = (d * (0.375 * slack_ratio).sqrt()).min(d * 0.6);
    let mid = Point::new(
        (a.x + b.x) * 0.5 + gravity_screen.0 * sag,
        (a.y + b.y) * 0.5 + gravity_screen.1 * sag,
    );
    p.quad_to(mid, b);
    p
}

/// **O glifo da BARRA** — duas linhas PARALELAS entre as âncoras.
///
/// A convenção de desenho mecânico para um elo rígido, e ela é escolhida por
/// GEOMETRIA e não por nome: uma corda tesa desenha uma reta que passa
/// exatamente sobre o segmento âncora→âncora, e uma barra desenha duas linhas
/// que **nunca** o tocam. É essa a diferença que um gate consegue medir — a
/// cicatriz do par `Layer`/`Layers` da timeline, reprovado num smoke por serem a
/// mesma figura com identificadores diferentes.
///
/// As pontas (os anéis) são do chamador, como no Spring e na Rope: um rod é
/// pinado nas duas extremidades, e é isso que os anéis dizem.
pub(super) fn rod_bar(a: Point, b: Point) -> BezPath {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx.hypot(dy);
    let mut p = BezPath::new();
    if len < 1e-6 {
        // Barra de span zero: as duas paralelas colapsariam numa só. Os anéis do
        // chamador mantêm o joint visível, exatamente como na mola.
        return p;
    }
    // A perpendicular unitária, em px de TELA — a espessura de uma barra
    // desenhada é chrome, não uma grandeza do mundo (a lição do traço de 1,5
    // unidades que virou 150 px no Shape Builder).
    let (nx, ny) = (-dy / len * ROD_HALF_PX, dx / len * ROD_HALF_PX);
    for s in [1.0, -1.0] {
        p.move_to(Point::new(a.x + nx * s, a.y + ny * s));
        p.line_to(Point::new(b.x + nx * s, b.y + ny * s));
    }
    p
}

/// **O arco de LIMITE de um pino** — as duas paredes e a agulha.
///
/// O limite do rapier é sobre o ângulo RELATIVO (`θb − θa`), então o arco é
/// desenhado no frame do corpo **A** (`angle_a + min ..= angle_a + max`) e a
/// agulha aponta para `angle_b`, que é onde o corpo B de fato está. Ler as duas
/// no mesmo desenho é o que responde *"quanto ainda posso girar?"* — a pergunta
/// que o Unreal responde com um cone e o Unity só depois de entrar num modo.
pub(super) fn limit_arc(
    camera: &Camera2d,
    window: WindowSize,
    centre_w: [f32; 2],
    angle_a: f32,
    limits: [f32; 2],
    angle_b: f32,
) -> BezPath {
    let (c, u, v) = screen_basis(camera, window, centre_w);
    let at = |ang: f64, r: f64| arc_point_in(c, u, v, ang, r);
    let a0 = f64::from(angle_a) + f64::from(limits[0]);
    let a1 = f64::from(angle_a) + f64::from(limits[1]);
    let mut p = BezPath::new();
    p.move_to(at(a0, LIMIT_ARC_PX));
    for i in 1..=ARC_SEGS {
        let t = f64::from(i) / f64::from(ARC_SEGS);
        p.line_to(at(a0 + (a1 - a0) * t, LIMIT_ARC_PX));
    }
    // As paredes: uma marca radial em cada extremo, para o fim do alcance ser
    // um LUGAR e não o ponto onde a curva simplesmente parou.
    for a in [a0, a1] {
        p.move_to(at(a, LIMIT_ARC_PX - LIMIT_TICK_PX));
        p.line_to(at(a, LIMIT_ARC_PX + LIMIT_TICK_PX));
    }
    // A agulha: onde o corpo B está AGORA, do centro até o arco.
    p.move_to(c);
    p.line_to(at(f64::from(angle_b), LIMIT_ARC_PX));
    p
}

/// **O anel de COMPRIMENTO** — repouso da mola, máximo da corda.
///
/// Construído em MUNDO e projetado (o raio é uma distância real), então a
/// âncora B pousada SOBRE o anel significa *no comprimento*, dentro dele
/// *comprimida/frouxa*, fora *esticada*. É o número do §12 virando lugar.
pub(super) fn length_ring(
    camera: &Camera2d,
    window: WindowSize,
    centre_w: [f32; 2],
    length: f32,
) -> BezPath {
    let c = screen_of(camera, window, centre_w);
    let edge = screen_of(camera, window, [centre_w[0] + length, centre_w[1]]);
    let r = (edge.x - c.x).hypot(edge.y - c.y);
    let mut p = BezPath::new();
    if r > 0.5 {
        ring_px(c, r, &mut p);
    }
    p
}
