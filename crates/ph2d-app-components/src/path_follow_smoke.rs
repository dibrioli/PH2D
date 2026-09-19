//! ⭐⭐⭐ **Smoke do SEGUIDOR DE CAMINHO** (suplente #23). `PH2D_PATHFOLLOW_SMOKE=1`.
//!
//! # A cena: UMA pista desenhada, e quatro coisas a percorrê-la
//!
//! | quadrado | a diferença | o que se vê |
//! |---|---|---|
//! | **Ronda** | nada (o valor de fábrica) | dá a volta e recomeça |
//! | **Vai-e-volta** | `Cycle = Ping-Pong` | vai até ao fim e **volta pela mesma curva** |
//! | **Faixa de fora** | `Side Offset = 0,45 m` | a mesma pista, **uma faixa ao lado** |
//! | **CONTROLO** (cinzento) | ⛔ nenhum seguidor — **dois tweens de pose** | vai em LINHA RECTA, por fora da curva |
//!
//! ⭐⭐⭐ **O CONTROLO é a §5.0 posta na tela.** A sonda mediu que dois tweens de pose saem da pista
//! em **`2,000000`** numa meia circunferência de raio `2` — e é esse o quadrado cinzento: ele parte
//! e chega com os outros, no mesmo relógio, e **corta pelo meio**. *Sem ele, a cena mostra quatro
//! objectos a andar e não mostra porque é que o componente existe.*
//!
//! ⚠️ **A pista é DESENHADA e fica visível**, com um traço grosso: o seguidor anda sobre uma forma
//! que o artista pode pegar, renomear e re-desenhar — e é isso que a secção nomeia pelo NOME.
//!
//! ⚠️ Se a linha `[pathfollow-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, PathFollow, Timer, Timers, Transform, Tweens, Visibility};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_tween::{Canal, Ciclo, Tween};
use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do `match` do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 1;

/// O NOME da pista — a referência durável, e o que o artista escreve na secção.
///
/// ⚠️ **Lido nos DOIS sítios pela mesma const**: aqui, para nomear a forma, e no componente de cada
/// seguidor. Escrito duas vezes, um erro de letra daria uma cena em que nada anda e o painel
/// diria *«no object has that name»* sobre uma forma que está à vista.
pub const PISTA: &str = "Trilho";

/// O período da volta. ⚠️ **O MESMO para os quatro**, de propósito: com ritmos diferentes o olho
/// compara velocidades em vez de comparar **percursos**.
const PERIODO_US: u64 = 2_400_000;

/// O raio da meia circunferência — a mesma fixtura da sonda do §5.0, e é o que torna o desvio do
/// CONTROLO um número previsto (`2,0 m`) em vez de uma impressão.
const RAIO: f64 = 2.0;
/// `4/3·tan(π/8)`, a constante clássica do quarto de círculo por cúbica, já escalada ao raio.
const K: f64 = 0.552_284_749_830_793_4 * RAIO;

const CHAO_RGBA: [f32; 4] = [0.14, 0.16, 0.20, 1.0];
const PECA_RGBA: [f32; 4] = [0.45, 0.68, 0.92, 1.0];
const CTRL_RGBA: [f32; 4] = [0.62, 0.64, 0.68, 1.0];
const LADO: f32 = 0.5;

/// O que a cena montou — quem nasce **escolhido**, para o Inspector abrir com sujeito.
pub struct Montada {
    /// ⚠️ **Em BITS**, como a irmã do tween: é isso que a selecção do gizmo guarda.
    pub escolhido: u64,
}

/// ⭐ **A PISTA** — meia circunferência aberta, de `(−2, 0)` a `(2, 0)`, a passar por `(0, 2)`.
///
/// ⚠️ Ela é **ABERTA** de propósito: num contorno fechado o `Ping-Pong` e o `Restart` acabam no
/// mesmo sítio, e a coluna do vai-e-volta ficava indistinguível da primeira.
fn pista() -> VecPath {
    VecPath {
        verts: vec![
            VecVertex {
                anchor: [-RAIO, 0.0],
                in_handle: [-RAIO, 0.0],
                out_handle: [-RAIO, K],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
            VecVertex {
                anchor: [0.0, RAIO],
                in_handle: [-K, RAIO],
                out_handle: [K, RAIO],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
            VecVertex {
                anchor: [RAIO, 0.0],
                in_handle: [RAIO, K],
                out_handle: [RAIO, 0.0],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
        ],
        closed: false,
        // ⚠️ **Traço e NENHUM preenchimento** — um contorno aberto preenchido fecha-se sozinho no
        // desenho, e o que ficava na tela era uma meia-lua em vez de uma pista.
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(90, 100, 120, 255), 0.09)),
        ..VecPath::default()
    }
}

/// Um relógio que repete para sempre e **não fala** — a lei do `#2`: um nome aqui poria um aviso na
/// tela a cada volta.
fn laco() -> Timer {
    Timer {
        name: "volta".into(),
        duration_us: PERIODO_US,
        repeat: true,
        autostart: true,
        signal: String::new(),
    }
}

fn seguidor(world: &mut ph2d_ecs::World, nome: &str, pf: PathFollow, cor: [f32; 4]) -> Entity {
    world
        .spawn((
            Name::new(nome),
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [LADO, LADO], cor),
            pf,
            Timers(vec![laco()]),
        ))
        .id()
}

/// Monta a cena. `None` = o nível não existe.
pub fn montar(cx: &mut crate::scene_ctx::SceneCtx, nivel: u32) -> Option<Montada> {
    monta_em(cx.sim, cx.vec_scene, cx.vec_entities, nivel)
}

/// ⭐⭐⭐ **A mesma montagem, escrita nos TRÊS empréstimos que ela de facto usa.**
///
/// ⚠️ Ela existe porque um gate não consegue construir um [`crate::scene_ctx::SceneCtx`] — ele pede
/// um ecrã, um relógio e uma árvore de tags que esta cena não toca. *Escrito em tipos, o pedido
/// cabe em três referências*, e é isso que torna a cena medível pelo caminho do PRODUTO em vez de
/// por uma cópia dela.
pub fn monta_em(
    sim: &mut ph2d_ecs::SimWorld,
    cena: &mut ph2d_vec_scene::VecScene,
    mapa: &mut ph2d_vec_entities::entities::VecEntityMap,
    nivel: u32,
) -> Option<Montada> {
    match nivel {
        1 => Some(cena_um(sim, cena, mapa)),
        _ => None,
    }
}

fn cena_um(
    sim: &mut ph2d_ecs::SimWorld,
    cena: &mut ph2d_vec_scene::VecScene,
    mapa: &mut ph2d_vec_entities::entities::VecEntityMap,
) -> Montada {
    // ⚠️⚠️ **O CHÃO PRIMEIRO** — a `assign_missing_root_order` congela a ordem das raízes pela
    // ordem de criação, e um fundo criado por último desenha **por cima de tudo** (o report do
    // dono no #13).
    sim.world_mut().spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [20.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));

    // ⭐⭐⭐ **A pista é desenhada, e a entidade dela nasce AQUI** — a `sync` é o passe que adopta
    // uma forma nova numa entidade da Hierarquia, e chamá-la agora é o que torna esta cena
    // **auto-contida**: sem ela o nome só existiria no quadro seguinte, e os seguidores abriam a
    // cena com a queixa *«no object in the scene has that name»* à vista.
    let id = cena.push_path(pista());
    ph2d_vec_entities::entities::sync(sim, cena, mapa);
    if let Some(&bits) = mapa.get(&id) {
        let e = Entity::from_bits(bits);
        if let Some(mut n) = sim.world_mut().get_mut::<Name>(e) {
            n.0 = PISTA.to_owned();
        }
    }

    let base = PathFollow {
        caminho: PISTA.to_owned(),
        ..PathFollow::default()
    };
    // ⭐ **RONDA** — o valor de fábrica, e é a que nasce escolhida (o passo (1) do roteiro).
    let ronda = seguidor(sim.world_mut(), "Ronda", base.clone(), PECA_RGBA);
    // ⭐ **VAI-E-VOLTA** — UMA diferença: o ciclo. A dobra da W8, segundo consumidor.
    seguidor(
        sim.world_mut(),
        "Vai-e-volta (Ping-Pong)",
        PathFollow {
            ciclo: Ciclo::PingPong,
            ..base.clone()
        },
        [0.95, 0.76, 0.30, 1.0],
    );
    // ⭐ **FAIXA DE FORA** — UMA diferença: o deslocamento perpendicular.
    seguidor(
        sim.world_mut(),
        "Faixa de fora (Side)",
        PathFollow { lado: 0.45, ..base },
        [0.55, 0.85, 0.60, 1.0],
    );

    // ⭐⭐⭐ **O CONTROLO — dois tweens de pose, no MESMO relógio.**
    //
    // ⚠️ Ele NÃO tem `PathFollow`, e é essa a lição: o percurso dele é a CORDA da curva, e o
    // desvio é o raio inteiro (`2 m`). *Esta é a medição do §5.0 posta na tela.*
    sim.world_mut().spawn((
        Name::new("Controlo (dois tweens)"),
        Transform::from_translation(Vec2::new(-2.0, 0.0)),
        Visibility::visible(),
        Sprite::atlas(WHITE_TILE_KEY, [LADO, LADO], CTRL_RGBA),
        Tweens(vec![
            #[allow(clippy::cast_possible_truncation)]
            Tween::linear(Canal::PositionX, -(RAIO as f32), RAIO as f32),
            Tween::linear(Canal::PositionY, 0.0, 0.0),
        ]),
        // ⚠️ **DOIS relógios iguais** — a lei do módulo é *o tween `i` corre no timer `i`*, e um
        // só deixaria o segundo tween inerte e calado.
        Timers(vec![laco(), laco()]),
    ));

    Montada {
        escolhido: ronda.to_bits(),
    }
}

/// O roteiro, impresso. ⚠️ **Auto-contido**: cada passo chega ao seguinte sem depender do estado
/// que a sessão anterior deixou.
pub fn anuncia() {
    eprintln!("[pathfollow-smoke] O SEGUIDOR DE CAMINHO — desenhe a patrulha com a caneta");
    eprintln!("  (1) A pista cinzenta e' uma FORMA DESENHADA chamada «{PISTA}». Os tres quadrados");
    eprintln!("      de cor andam SOBRE ela; o cinzento nao — ele corta pelo meio.");
    eprintln!("  (2) O quadrado azul «Ronda» ja' nasce escolhido: no painel da direita, a seccao");
    eprintln!("      PATH FOLLOW diz «{PISTA}» no campo Path, e o relogio dela e' o Timer 1.");
    eprintln!("  (3) Apague o nome no campo Path. O quadrado PARA, e o painel diz porque —");
    eprintln!("      «Type the name of a drawn shape». Escreva «{PISTA}» outra vez e ele volta.");
    eprintln!("  (4) Escolha o quadrado AMARELO: o Cycle dele e' «Ping-Pong», e e' a UNICA");
    eprintln!("      diferenca — ele volta pela mesma curva em vez de recomecar.");
    eprintln!("  (5) Escolha o VERDE: o «Side Offset» poe-no numa faixa ao lado da mesma pista.");
    eprintln!("      Mude o numero e veja-o mudar de faixa sem sair do percurso.");
    eprintln!("  (6) Desligue «Face Path» num deles: ele continua a andar e deixa de virar.");
    eprintln!(
        "  (7) Na Hierarquia, escolha «{PISTA}» e ARRASTE-A. Os tres seguidores vao com ela."
    );
    eprintln!("  DEU ERRADO se: algum quadrado de cor sair da pista · o cinzento NAO cortar pelo");
    eprintln!("      meio · o painel nao disser porque e' que um seguidor sem nome esta' parado.");
}

#[cfg(test)]
#[path = "path_follow_smoke_tests.rs"]
mod tests;
