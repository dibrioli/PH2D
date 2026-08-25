//! **O QUE UMA FORÇA NÃO SABIA DIZER** — a cena `=95` (doc 89, folha 02).
//!
//! Quatro pares. ⚠️ **Esta cena ANDA — carregue Play**: as quatro fileiras são simulação.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `force.attractor` | a rampa de sempre | **o PERFIL** — janela, pico e inversão |
//! | `force.wind` | `Force` | **`Target Velocity`** — o vento SATURA |
//! | `force.vortex` | `Force` | **`Target Velocity`** — o rodamoinho estabiliza num anel |
//! | `force.buoyancy` | uma onda | **quatro** — o mar deixa de ser uma senoide |
//!
//! ⚠️ **As oito bandas montam a topologia do integrador**, que é a única que faz uma força
//! mover alguma coisa: a fonte entra em `rest`, e a cadeia de forças vive DENTRO do laço
//! `pre`. Ver [`sim_chain`].
//!
//! ⚠️ **A fileira do mar não é uma nuvem: é uma FILEIRA DE BOIAS**, e ela leva mais duas
//! coisas que as outras não levam — **gravidade** ([`GRAVITY`]) e um **arrasto derivado**
//! ([`SEA_TRAP_MARGIN`]). As duas nasceram do mesmo report do Enio (*«os exemplos do mar não
//! parecem mar mas sim partículas ao vento»*), e cada uma é metade da causa: sem gravidade o
//! empuxo LANÇA tudo; com pouco arrasto a boia ENCAIXA na cava e viaja com a onda. Os dois
//! defeitos leem-se igual — peças a atravessar a banda sem subir nem descer.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão entre as duas colunas e entre as quatro linhas.
const GAP_X: f32 = 5.2;
const GAP_Y: f32 = 3.8;
/// O perfil que o atrator da direita autora.
const PEAK: f32 = 2.2;
const REVERSE: f32 = 1.2;
/// A resistência do ar que os dois modos-alvo autoram.
const AIR: f32 = 3.0;
/// O espectro que o mar da direita autora.
const WAVES: f32 = 4.0;

/// **A GRAVIDADE do mar**, e ela é um `force.wind` a apontar para baixo.
///
/// ⚠️ **O `force.buoyancy` não tem param de gravidade de propósito**, pela mesma razão por
/// que não há nó `force.gravity`: uma força direcional constante já existe no catálogo. A
/// cena `=4` (o mar de 490 k partículas) declara-o no doc-comment dela — *«empuxo sozinho
/// LANÇARIA tudo para cima, e é a disputa entre `density` e `strength` que assenta o campo
/// em `submersão = strength/density`»*.
///
/// ⛔ **E a primeira versão desta fileira não tinha nenhuma**, com o resultado exacto que a
/// nota previa e que o smoke devolveu: *«não parece mar, parece partículas ao vento»*.
/// Medido na cena antiga — a média de `y` subia `0,58` por cada 25 tiques **sem abrandar**
/// (`−5,73 → −2,72` em 150 tiques) e a banda ia de `3,16` a `14,01` de largura, invadindo a
/// fileira de cima. *Uma cena que demonstra um nó tem de reproduzir as condições em que a
/// resposta dele quer dizer alguma coisa.*
const GRAVITY: f32 = 2.0;
/// A densidade da água contra a [`GRAVITY`]: `6` contra `2` ⇒ assenta a um terço do calado.
const SEA_DENSITY: f32 = 6.0;
/// **O calado.** Fundo de propósito: a rampa de submersão é o que espalha as boias numa
/// FAIXA em vez de as pregar todas a um fio de cabelo.
const SEA_DRAFT: f32 = 0.5;

/// **O ARRASTO, e ele é DERIVADO de uma armadilha** — o segundo defeito que o smoke do Enio
/// devolveu, e o mais interessante dos dois.
///
/// ⛔ Com a gravidade já corrigida, a fileira **continuava** a ler-se como partículas a
/// serem levadas. Medido no regime assentado: excursão vertical mediana **`0,0056`** e
/// horizontal **`4,92`** em 5 segundos, ou seja `0,98` da velocidade da onda. As boias não
/// subiam nem desciam: elas **ENCAIXAVAM na cava e viajavam com a vaga**, e o que se via era
/// uma linha ondulada RÍGIDA a deslizar de lado.
///
/// ⚠️ **O nó não está errado — ele diz isto no doc-comment dele** (*«a boia deriva para a
/// cava e cavalga a vaga, em vez de subir e descer no mesmo sítio»*). O que estava errado era
/// a cena escolher números dentro do regime onde esse comportamento come tudo o resto.
///
/// **A lei da armadilha**, e ela prevê as duas transições medidas: a boia escorrega até o
/// empurrão em declive igualar o arrasto, logo ela encaixa se existir declive que o faça à
/// velocidade da onda — `densidade · declive_máximo · inv_len ≥ arrasto · velocidade`.
/// ⚠️ **O espectro multiplica o declive pelo número de camadas** (cada oitava tem metade da
/// amplitude e metade do comprimento ⇒ o MESMO declive), então a fileira de 4 ondas manda.
///
/// Medido, com o limiar previsto ao lado — a transição cai exactamente onde a lei a põe:
///
/// | densidade | ondas | limiar | arrasto 6 | arrasto 11 |
/// |---|---|---|---|---|
/// | 12 | 1 | `6,38` | preso (deriva `4,92`) | livre |
/// | 12 | 4 | `11,15` | preso (deriva `4,92`) | livre (deriva `1,72`) |
/// | 6 | 4 | `5,57` | livre | livre (deriva **`0,025`**) |
///
/// ⚠️ **A margem é `2×` e ela CUSTA vida:** mais arrasto afasta da armadilha e ao mesmo tempo
/// abafa a boia (a `20` a excursão vertical cai para `0,29` da altura da vaga). `2×` é onde a
/// deriva já é desprezável (`0,07` em 5 s) e a boia ainda faz `0,82` da vaga.
const SEA_TRAP_MARGIN: f32 = 2.0;

/// O limiar da armadilha para um mar de `waves` camadas — ver [`SEA_TRAP_MARGIN`].
pub(crate) fn sea_trap_threshold(waves: f32) -> f32 {
    let slope = waves * std::f32::consts::TAU * SEA_STEEPNESS;
    SEA_DENSITY * slope / (1.0 + slope * slope).sqrt() / SEA_SPEED
}

/// O arrasto que a cena autora: acima do limiar da fileira mais exigente (a de 4 ondas).
pub(crate) fn sea_drag() -> f32 {
    SEA_TRAP_MARGIN * sea_trap_threshold(WAVES)
}

/// A largura da banda de boias, e quantas cristas ela mostra.
///
/// ⚠️ **O comprimento de onda é DERIVADO da largura da banda**, e não o contrário. A
/// primeira versão tinha uma nuvem de `3,2` de largura contra um comprimento de `3,0`:
/// **menos de uma onda inteira à vista** — e a fileira prometia ao olho uma comparação de
/// *espaçamento entre cristas* que não cabia no que estava desenhado.
const SEA_SPAN: f32 = 7.0;
const SEA_CRESTS: f32 = 3.0;
const SEA_WAVELENGTH: f32 = SEA_SPAN / SEA_CRESTS;

/// **A ESBELTEZA da vaga** — `amplitude / comprimento`, e é ela que decide se aquilo se lê
/// como água ou como serrote.
///
/// ⚠️ **Derivada, não escolhida.** Uma onda de água tem um limite FÍSICO de esbelteza: acima
/// de `H/λ = 1/7` (ou seja `a/λ = 1/14 ≈ 0,071`) ela quebra e deixa de ser uma onda. A
/// primeira versão desta fileira estava em **`0,19`, 2,7× para lá do limite** — uma forma
/// que a água não faz, e portanto uma forma que o olho não lê como água.
///
/// O número que fica é o da cena `=4` (o mar de 490 k partículas que esta casa já shipa):
/// `amplitude 2,0 / comprimento 20,0` ⇒ **`0,1`**. Ela está 1,4× acima do limite de quebra
/// por escolha artística, e *uma segunda cena de mar não é onde essa decisão se re-abre*.
const SEA_STEEPNESS: f32 = 0.1;
const SEA_AMPLITUDE: f32 = SEA_WAVELENGTH * SEA_STEEPNESS;
const SEA_SPEED: f32 = 1.0;

/// **Quantas BOIAS**, e o número vem de Nyquist.
///
/// ⚠️ **A onda mais fina do espectro é `λ/8`** (quatro camadas, cada uma de metade do
/// comprimento da anterior — ver `WAVE_LACUNARITY`). Com as `48` colunas da primeira versão
/// a banda tinha **1,96 boias por essa onda**: *abaixo de dois pontos por período um seno
/// não é sub-amostrado, ele é irreconhecível* — sairia como ruído, e uma fileira que promete
/// «cristas de tamanhos diferentes» mostraria pontinhos a tremer.
///
/// `128` colunas dão `0,055` de espaçamento ⇒ **5,3 boias na onda mais fina** e `10,6` na
/// seguinte. O custo é nada: a cena `=4` corre 490 000.
const SEA_COLS: f32 = 128.0;

/// A altura a que as boias nascem: elas **caem** para a água e assentam nela.
const SEA_DROP: f32 = 0.6;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .ok()
}

/// **O ARRASTO** que as fileiras de equilíbrio precisam.
///
/// ⚠️ **Um anel é um EQUILÍBRIO, e um equilíbrio precisa de dissipação.** Sem arrasto a
/// nuvem atravessa o centro com a energia que ganhou a cair para lá e volta a sair — medido:
/// o ponto mais perto do alvo ficava em `4,65` de um raio de influência de `4`, ou seja
/// TODA a nuvem já tinha escapado. *Uma cena que demonstra onde as coisas assentam tem de
/// deixá-las assentar.*
fn damped(g: &mut Graph, ey: f32, head: NodeId) -> Option<NodeId> {
    let d = g.add_node("force.drag");
    g.set_pos(d, Pos { x: 380.0, y: ey });
    g.set_param(d, "coefficient", 1.2);
    wire(g, head, 0, d, 0, false)?;
    Some(d)
}

/// **A CADEIA QUE SIMULA** — a fonte, o integrador, e a força DENTRO do laço `pre`.
///
/// ⚠️ Uma `force.*` é `Pure` e só acumula `accel`; **um** integrador a consome. Montada na
/// horizontal (`fonte → força → saída`) a cena fica parada **sem erro nenhum** — o app
/// conserta esse gesto sozinho quando o artista o faz (ADR-0155), e um documento montado em
/// código não passa por esse portão.
fn sim_chain(g: &mut Graph, ey: f32, src: NodeId, head: NodeId, tail: NodeId) -> Option<NodeId> {
    let integ = g.add_node("motion.integrate");
    g.set_pos(integ, Pos { x: 460.0, y: ey });
    wire(g, src, 0, integ, 0, false)?;
    wire(g, integ, 0, head, 0, true)?;
    wire(g, tail, 0, integ, 1, false)?;
    Some(integ)
}

/// Leva a banda ao quadrante, pinta-a e fecha.
fn finish(g: &mut Graph, head: NodeId, rgb: [f32; 3], at: [f32; 2], ey: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x: 700.0, y: ey });
    g.set_param(mv, "dx", at[0]);
    g.set_param(mv, "dy", at[1]);
    wire(g, head, 0, mv, 0, false)?;
    let tint = g.add_node("motion.tint");
    g.set_pos(tint, Pos { x: 840.0, y: ey });
    g.set_param(tint, "r", rgb[0]);
    g.set_param(tint, "g", rgb[1]);
    g.set_param(tint, "b", rgb[2]);
    wire(g, mv, 0, tint, 0, false)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 980.0, y: ey });
    wire(g, tint, 0, out, 0, false)?;
    Some(out)
}

/// A nuvem que as quatro fileiras partilham.
fn cloud(g: &mut Graph, ey: f32, seed: f32) -> NodeId {
    let n = g.add_node("motion.scatter");
    g.set_pos(n, Pos { x: 120.0, y: ey });
    g.set_param(n, "count", 90.0);
    // ⚠️ **A nuvem cabe DENTRO do raio de influência** (4,0): com ela a transbordar,
    // os cantos ficavam fora e nenhuma das duas fileiras do atrator mostrava o que a
    // força faz — media-se o que ela NÃO alcança.
    g.set_param(n, "width", 3.2);
    g.set_param(n, "height", 3.2);
    g.set_param(n, "seed", seed);
    n
}

/// **O ATRATOR** — a rampa de sempre, ou o perfil com pico e inversão.
fn attractor(g: &mut Graph, ey: f32, profiled: bool) -> NodeId {
    let a = g.add_node("force.attractor");
    g.set_pos(a, Pos { x: 300.0, y: ey });
    g.set_param(a, "strength", 6.0);
    g.set_param(a, "radius", 4.0);
    if profiled {
        g.set_param(a, ph2d_node_force_attractor::PEAK, PEAK);
        g.set_param(a, ph2d_node_force_attractor::REVERSE, REVERSE);
    }
    a
}

/// **O VENTO** — aceleração, ou velocidade-alvo.
fn wind(g: &mut Graph, ey: f32, target: bool) -> NodeId {
    let w = g.add_node("force.wind");
    g.set_pos(w, Pos { x: 300.0, y: ey });
    g.set_param(w, "strength", 2.0);
    g.set_param(w, "angle", 0.0);
    g.set_param(w, "gust", 0.0);
    if target {
        g.set_param(w, ph2d_node_force_wind::MODE, 1.0);
        g.set_param(w, ph2d_node_force_wind::AIR_RESIST, AIR);
    }
    w
}

/// **O VÓRTICE** — idem.
fn vortex(g: &mut Graph, ey: f32, target: bool) -> NodeId {
    let v = g.add_node("force.vortex");
    g.set_pos(v, Pos { x: 300.0, y: ey });
    g.set_param(v, "strength", 8.0);
    g.set_param(v, "radius", 4.0);
    if target {
        g.set_param(v, ph2d_node_force_vortex::MODE, 1.0);
        g.set_param(v, ph2d_node_force_vortex::AIR_RESIST, AIR);
    }
    v
}

/// **AS BOIAS** — uma fileira larga de flutuadores, e não uma nuvem.
///
/// ⚠️ **Um mar lê-se como uma SUPERFÍCIE, e uma superfície precisa de pontos ao longo dela.**
/// As outras três fileiras mostram uma força a agir sobre uma nuvem; esta mostra a forma da
/// água, e uma mancha quadrada de pontos não tem forma nenhuma para mostrar. Elas nascem
/// [`SEA_DROP`] acima da linha de água e **caem** para ela.
fn sea_floats(g: &mut Graph, ey: f32) -> Option<NodeId> {
    let n = g.add_node("motion.grid");
    g.set_pos(n, Pos { x: 120.0, y: ey });
    g.set_param(n, "rows", 2.0);
    g.set_param(n, "cols", SEA_COLS);
    g.set_param(n, "gap_x", SEA_SPAN / (SEA_COLS - 1.0));
    g.set_param(n, "gap_y", 0.3);
    let up = g.add_node("motion.move");
    g.set_pos(up, Pos { x: 200.0, y: ey });
    g.set_param(up, "dy", SEA_DROP);
    wire(g, n, 0, up, 0, false)?;
    Some(up)
}

/// **A GRAVIDADE** — ver [`GRAVITY`]: um vento constante a apontar para baixo (270°).
fn gravity(g: &mut Graph, ey: f32) -> NodeId {
    let w = g.add_node("force.wind");
    g.set_pos(w, Pos { x: 300.0, y: ey });
    g.set_param(w, "angle", 270.0);
    g.set_param(w, "strength", GRAVITY);
    g.set_param(w, "gust", 0.0);
    w
}

/// **O MAR** — uma onda, ou quatro.
fn buoyancy(g: &mut Graph, ey: f32, spectrum: bool) -> NodeId {
    let b = g.add_node("force.buoyancy");
    g.set_pos(b, Pos { x: 380.0, y: ey });
    g.set_param(b, "level", 0.0);
    g.set_param(b, "density", SEA_DENSITY);
    g.set_param(b, "depth", SEA_DRAFT);
    g.set_param(b, "drag", sea_drag());
    g.set_param(b, "wave_amplitude", SEA_AMPLITUDE);
    g.set_param(b, "wave_length", SEA_WAVELENGTH);
    g.set_param(b, "wave_speed", SEA_SPEED);
    if spectrum {
        g.set_param(b, ph2d_node_force_buoyancy::WAVES, WAVES);
    }
    b
}

/// A fonte e a cadeia de forças de uma fileira: `(fonte, PRIMEIRA força, ÚLTIMA força)`.
///
/// ⚠️ A cadeia vive dentro do laço `pre` — ver [`sim_chain`]. Duas fileiras têm mais de uma
/// força: a do atrator acrescenta ARRASTO no fim (senão nada assenta), e a do mar acrescenta
/// GRAVIDADE no princípio (senão o empuxo lança tudo — ver [`GRAVITY`]).
fn band(g: &mut Graph, ey: f32, row: usize, on: bool) -> Option<(NodeId, NodeId, NodeId)> {
    if row == 3 {
        let src = sea_floats(g, ey)?;
        let grav = gravity(g, ey);
        let sea = buoyancy(g, ey, on);
        wire(g, grav, 0, sea, 0, false)?;
        return Some((src, grav, sea));
    }
    let src = cloud(g, ey, 3.0 + row as f32);
    let force = match row {
        0 => attractor(g, ey, on),
        1 => wind(g, ey, on),
        _ => vortex(g, ey, on),
    };
    let tail = if row == 0 {
        damped(g, ey, force)?
    } else {
        force
    };
    Some((src, force, tail))
}

/// Monta a cena. Devolve os oito sinks, em pares.
pub(crate) fn build_forces_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let rgb = [
        [0.52, 0.76, 1.0],
        [1.0, 0.78, 0.4],
        [0.66, 1.0, 0.72],
        [0.95, 0.6, 0.78],
    ];
    let mut sinks = Vec::with_capacity(8);
    for (row, colour) in rgb.iter().enumerate() {
        for col in 0..2 {
            let ey = (row * 2 + col) as f32 * 260.0;
            let on = col == 1;
            let (src, first, tail) = band(g, ey, row, on)?;
            let head = sim_chain(g, ey, src, first, tail)?;
            sinks.push(finish(g, head, *colour, band_at(row * 2 + col), ey)?);
        }
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

/// Os rótulos das oito bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "ATRATOR de sempre -- puxa mais forte no centro, e tudo acaba num ponto",
        "ATRATOR com PERFIL -- ele EMPURRA de perto e puxa de longe: forma-se um ANEL",
        "VENTO Force -- a aceleracao nunca para, e as pecas saem do ecra",
        "VENTO Target Velocity -- elas alcancam a velocidade do vento e ficam nela",
        "VORTICE Force -- o giro acelera sem fim e a nuvem espalha-se",
        "VORTICE Target Velocity -- ela estabiliza num rodamoinho de raio constante",
        "MAR de UMA onda -- todas as cristas a mesma distancia, como um desenho",
        "MAR de QUATRO ondas -- cristas de tamanhos diferentes: le^-se como agua",
    ]
    .into_iter()
    .enumerate()
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let at = band_at(k);
            crate::motion_demo_legend::Caption::new([at[0], at[1] + GAP_Y * 0.44], short_of(label))
        })
        .collect()
}

/// A ficha curta: o que está ANTES do primeiro `--`.
fn short_of(label: &'static str) -> &'static str {
    match label.find(" --") {
        Some(i) => &label[..i],
        None => label,
    }
}

/// Os números que a mensagem do smoke cita.
pub(crate) fn authored() -> (f32, f32, f32) {
    (PEAK, AIR, WAVES)
}

/// **Onde a banda `k` é POUSADA** — a conta que a cena e as fichas partilham.
///
/// ⚠️ A simulação corre em coordenadas LOCAIS (a linha de água é `level = 0`) e só depois o
/// `finish` desloca a banda para o quadrante dela. Quem quiser comparar uma pose com a
/// superfície tem de desfazer este deslocamento primeiro.
pub(crate) fn band_at(k: usize) -> [f32; 2] {
    let (row, col) = (k / 2, k % 2);
    [
        if col == 0 { -GAP_X } else { GAP_X },
        GAP_Y * 1.5 - row as f32 * GAP_Y,
    ]
}

/// **O espaçamento entre boias vizinhas** — a resolução com que esta cena amostra o mar.
///
/// ⚠️ Só os gates a leem: o produto não precisa de saber a que resolução amostra, precisa de
/// amostrar bem. É o gate que compara este número com a onda mais fina.
#[cfg(test)]
pub(crate) fn float_spacing() -> f32 {
    SEA_SPAN / (SEA_COLS - 1.0)
}

/// O mar autorado: `(amplitude, comprimento, velocidade, calado, submersão de equilíbrio)`.
///
/// ⚠️ **A submersão de equilíbrio é DERIVADA** (`sub = gravidade / densidade`, do
/// doc-comment do nó) — mas ⛔ **ela é o repouso de um corpo PARADO, e um gate não a pode
/// exigir de um mar vivo.** Eu escrevi esse gate e ele passava a `0,8%`; passava **porque as
/// boias estavam presas na cava**. Assim que elas cavalgam a vaga passam a ser forçadas, e o
/// ponto delas deixa de ser o estático (medido: `0,29` contra `0,167`). Fica aqui como
/// referência de escala — quem a usar num gate afirma um mar morto.
#[cfg(test)]
pub(crate) fn sea_authored() -> (f32, f32, f32, f32, f32) {
    (
        SEA_AMPLITUDE,
        SEA_WAVELENGTH,
        SEA_SPEED,
        SEA_DRAFT,
        GRAVITY / SEA_DENSITY,
    )
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_forces_tests.rs"]
mod tests;
