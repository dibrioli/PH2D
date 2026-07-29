//! **A cena do ELEVADOR** (`PH2D_PHYSICS_SMOKE=58`, W-Pulley).
//!
//! Uma polia é o primeiro vínculo do kit que **não é um joint do rapier** — ele
//! não tem polia e não oferece gancho de restrição, então ela é imposta de fora,
//! por um passe de impulso por sub-passo. O que se vê na tela, porém, tem de ser
//! simplesmente *uma corda que passa por roldanas*.
//!
//! Dois rigs, lado a lado:
//!
//! - **VERDE** — o elevador com contrapeso, duas roldanas de tamanhos
//!   DIFERENTES. O que um lado desce o outro sobe, e a corda passa na
//!   SUPERFÍCIE de cada roda, não pelo centro dela.
//! - **ÂMBAR** — a MESMA corda por **quatro** roldanas em ziguezague. O caminho
//!   é mais longo e o comportamento é o mesmo: numa corda única a tensão é
//!   uniforme, e mais roldanas não dão vantagem mecânica nenhuma.
//!
//! ⚠️ **A versão anterior desta cena vendia uma TALHA que não existia.** Ela
//! tinha um `ratio` (`l1 + razão·l2 ≤ L0`) e mostrava um contrapeso de 1 kg
//! erguendo 3 kg — mas aquela lei descreve uma talha **diferencial**, dois
//! tambores de raios diferentes no mesmo eixo, *sem os tambores*. Com roldanas
//! de verdade a vantagem mecânica volta por onde ela vem no mundo: uma roldana
//! montada num corpo que se MOVE (W3) ou um tambor DIRIGIDO (W2).
//!
//! Os números da mensagem saem da sonda `probe_smoke_58`, rodada sobre ESTAS
//! peças antes de a mensagem ser escrita.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, PulleyWheel, RigidBody, WrapSide,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

pub(crate) const GROUND: [f32; 4] = [0.42, 0.42, 0.46, 1.0];
const SIMPLE: [f32; 4] = [0.45, 0.9, 0.45, 1.0];
const ZIGZAG: [f32; 4] = [0.95, 0.75, 0.30, 1.0];
const WEIGHT: [f32; 4] = [0.30, 0.32, 0.40, 1.0];

/// Onde os dois corpos de cada rig nascem.
const START_Y: f32 = 3.0;
/// A distância entre a carga e o contrapeso de um rig.
const SPAN: f32 = 3.0;
/// A que altura acima dos corpos as roldanas ficam.
///
/// ⚠️ **Alto o bastante para o contrapeso NÃO alcançar a própria roldana** antes
/// de a carga pousar no chão. Um corpo que passa da roldana dele inverte o ramo —
/// a corda passa a puxar do outro lado — e a cena dá um tranco que não ensina
/// nada sobre polias (medido com `LIFT = 2`: o contrapeso passava a roda no tick
/// 56 e o rig inteiro voltava ao começo). O chão é o freio da cena, e a altura é
/// escolhida para que ele chegue primeiro.
const LIFT: f32 = 6.0;

/// **Quanto a carga do rig VERDE desce em 3 s**, metros. A carga é 3× o
/// contrapeso, então ela ganha e desce — até pousar no chão.
pub(crate) const MEASURED_SIMPLE_LOAD_DROP: f32 = 4.17;
/// **Quanto o contrapeso VERDE sobe no mesmo tempo.** A corda é inextensível, e
/// é isso que estes dois números dizem juntos.
pub(crate) const MEASURED_SIMPLE_CW_RISE: f32 = 4.15;
/// **Quanto a carga do rig ÂMBAR desce**, com o MESMO par de massas por uma
/// corda quatro vezes mais dobrada. Se ele diferisse do verde, a cena estaria
/// mostrando uma vantagem mecânica que a física não tem.
pub(crate) const MEASURED_ZIGZAG_LOAD_DROP: f32 = 4.18;

/// A massa da carga e a do contrapeso de cada rig, em kg. O par é o MESMO nos
/// dois — o que muda é só o caminho da corda, que é o ponto da cena.
const LOAD_KG: f32 = 3.0;
const COUNTERWEIGHT_KG: f32 = 1.0;
pub(crate) const R: f32 = 0.25;

pub(crate) fn ball(world: &mut World, name: &str, x: f32, mass: f32, rgba: [f32; 4], half: f32) {
    world.spawn((
        Name::new(name),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: half },
            density: mass / (std::f32::consts::PI * half * half),
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [half * 2.0, half * 2.0], rgba),
        Transform::from_translation(Vec2::new(x, START_Y)),
    ));
}

/// Uma roldana: uma ENTIDADE com centro (`Transform`) e raio.
fn wheel(world: &mut World, rope: &str, order: u16, x: f32, y: f32, radius: f32) {
    world.spawn((
        Name::new(format!("{rope} Wheel {}", order + 1)),
        PulleyWheel {
            rope: stable_name_id(rope),
            order,
            radius,
            wrap: WrapSide::Auto,
            motor_speed: 0.0,
            break_enabled: false,
            break_force: PulleyWheel::DEFAULT_BREAK_FORCE,
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(x, y)),
    ));
}

/// Um rig: carga + contrapeso + a corda que os une + as roldanas dela.
fn rig(world: &mut World, tag: &str, centre: f32, rgba: [f32; 4], zigzag: bool) {
    let load = format!("{tag} Load");
    let cw = format!("{tag} Counterweight");
    let rope = format!("{tag} Rope");
    ball(world, &load, centre - SPAN / 2.0, LOAD_KG, rgba, R);
    ball(
        world,
        &cw,
        centre + SPAN / 2.0,
        COUNTERWEIGHT_KG,
        WEIGHT,
        R * 0.7,
    );
    world.spawn((
        Name::new(rope.clone()),
        PhysicsJoint {
            body_a: stable_name_id(&load),
            body_b: stable_name_id(&cw),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(centre - SPAN / 2.0, START_Y)),
    ));
    let top = START_Y + LIFT;
    if zigzag {
        // Quatro roldanas, alternando de lado: o caminho dobra quatro vezes e a
        // mecânica não muda.
        for (i, (x, y, r)) in [
            (centre - SPAN / 2.0, top, 0.30),
            (centre - SPAN / 6.0, top + 0.9, 0.20),
            (centre + SPAN / 6.0, top, 0.20),
            (centre + SPAN / 2.0, top + 0.9, 0.30),
        ]
        .into_iter()
        .enumerate()
        {
            wheel(
                world,
                &rope,
                u16::try_from(i).expect("quatro roldanas"),
                x,
                y,
                r,
            );
        }
    } else {
        // Duas roldanas de tamanhos DIFERENTES — é o diâmetro que se vê.
        wheel(world, &rope, 0, centre - SPAN / 2.0, top, 0.45);
        wheel(world, &rope, 1, centre + SPAN / 2.0, top, 0.18);
    }
}

/// Monta a cena inteira.
pub(crate) fn build(world: &mut World) {
    world.spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 14.0,
                half_y: 0.3,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [28.0, 0.6], GROUND),
        Transform::from_translation(Vec2::new(0.0, -2.0)),
    ));
    rig(world, "Simple", -4.5, SIMPLE, false);
    rig(world, "Zigzag", 4.5, ZIGZAG, true);
}

impl crate::App {
    /// **Cena 58 (W-Pulley).** Dois elevadores, mesmo par de massas, e a única
    /// diferença é o caminho da corda.
    /// **Cena 59 (W-Pulley W2).** Quatro guinchos: um que ergue, um que baixa, e
    /// um PAR que só difere no diâmetro do tambor.
    ///
    /// Os números saem da sonda `probe_smoke_59`, rodada sobre ESTA cena.
    pub(crate) fn physics_smoke_winch(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_winch(gfx.sim.world_mut());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 59] O GUINCHO -- uma roldana com MOTOR.\n  \
               O contorno JA ESTA LIGADO (B o alterna). De PLAY para os passos 1-4 e 6\n  \
               (o toggle Physics ja esta armado); o passo 5 pede o relogio PARADO.\n\n  \
               1. AZUL (esquerda) -- o tambor recolhe e o gancho SOBE, sem ninguem\n     \
                  puxar nada. A corda encurta a `w*r`: 60 graus/s num tambor de 0,45 m\n     \
                  sao 0,47 m/s de corda. (medido em 2 s: subiu {hoist:.2} m.)\n  \
               2. VERMELHO -- o MESMO tambor com o motor NEGATIVO: ele paga corda e a\n     \
                  carga desce. Quem a baixa e a GRAVIDADE -- a corda so deixa de\n     \
                  segurar. Ela nunca EMPURRA. (desceu {lower:.2} m.)\n  \
               3. ROXO e VERDE (direita) -- o CAMBIO, e e o coracao desta wave. Os dois\n     \
                  motores giram na MESMA velocidade (60 graus/s); o que muda e o\n     \
                  DIAMETRO do tambor: 0,60 contra 0,20. O roxo sobe {ratio:.2}x mais rapido,\n     \
                  que e exatamente a razao dos raios. Era isto que o antigo campo\n     \
                  'Ratio' prometia e nao entregava -- agora ele e uma PECA na cena.\n  \
               4. SELECIONE um tambor na Hierarquia ('Hoist Rope Wheel 1'). No Inspector,\n     \
                  na secao 'Pulley Wheel', ha uma row nova: Motor (graus/s). Mude o numero\n     \
                  COM O RELOGIO ANDANDO -- o guincho responde na hora. Ponha negativo e\n     \
                  ele inverte; ponha zero e a roldana volta a ser uma roldana livre.\n  \
               5. PAUSE e ARRASTE o ponto do ARO para mudar o RAIO -- alca de ponto so\n     \
                  existe em REPOUSO (tocando, o overlay desenha a geometria do SOLVER).\n     \
                  Volte a tocar: a mesma rotacao passa a recolher mais (ou menos) corda.\n     \
                  O diametro e o cambio, e da para senti-lo com o dedo.\n  \
               6. SCRUB a regua para tras: o guincho REBOBINA junto com o mundo, porque\n     \
                  o quanto ele ja recolheu viaja no checkpoint. Reset devolve tudo ao\n     \
                  comeco.\n  \
               ⚠️ NAO deixe um guincho recolhendo ate o gancho ALCANCAR o tambor: nao ha\n     \
                  colisor na roldana, entao a carga passa por ela e a corda a sacode. O\n     \
                  teto interno limita a violencia (sem ele a carga sai de quadro a\n     \
                  milhares de m/s), mas o lugar certo de parar e antes.",
            hoist = MEASURED_HOIST_RISE,
            lower = MEASURED_LOWER_DROP,
            ratio = MEASURED_GEAR_RATIO,
        );
    }

    pub(crate) fn physics_smoke_pulley(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build(gfx.sim.world_mut());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 58] A POLIA -- uma corda por roldanas de verdade.\n  \
               A cena nasce PARADA e o contorno JA ESTA LIGADO -- B o ALTERNA, entao\n  \
               aperta-lo aqui o DESLIGA e as alcas somem junto. Faca o passo das ALCAS\n  \
               primeiro; o PLAY vem por ultimo (o toggle Physics ja esta armado).\n\n  \
               1. OLHE AS RODAS, e depois a corda. Cada roldana e um CIRCULO do tamanho\n     \
                  que ela tem, com um raio-guia; a corda toca a SUPERFICIE dela e nao passa\n     \
                  pelo centro. No verde as duas rodas tem tamanhos MUITO diferentes de\n     \
                  proposito -- de PLAY e olhe os raios-guia girando: a roda GRANDE gira mais\n     \
                  devagar, porque a mesma corda por um raio maior a faz dar menos voltas.\n  \
               2. VERDE (esquerda) -- o elevador. A carga de 3 kg ganha do contrapeso de\n     \
                  1 kg e desce; o que um lado desce o outro sobe, porque a corda nao\n     \
                  estica. (medido em 3 s: desceu {simple_drop:.2} m ate o chao e o contrapeso\n     \
                  subiu {simple_rise:.2} m -- o MESMO numero.)\n  \
               3. AMBAR (direita) -- a MESMA corda por QUATRO roldanas em ziguezague, e o\n     \
                  MESMO par de massas. A carga desce {zig_drop:.2} m: identico ao verde.\n     \
                  Isso NAO e um bug -- numa corda unica a tensao e uniforme, entao mais\n     \
                  roldanas so alongam o caminho. (A vantagem mecanica de verdade vem de\n     \
                  uma roldana montada num corpo que se MOVE, ou de um tambor dirigido:\n     \
                  sao as duas waves seguintes.)\n  \
               4. SELECIONE uma roldana na Hierarquia ('Simple Rope Wheel 1'). Aparecem\n     \
                  DOIS pontos ambar: o do CENTRO (arraste e ela se muda de lugar) e o do\n     \
                  ARO, a direita (arraste e ela muda de TAMANHO). Em qualquer um dos dois\n     \
                  a corda re-roteia na hora, e Ctrl+Z desfaz.\n  \
               5. E no Inspector abre a secao 'Pulley Wheel', com tres coisas:\n     \
                  Radius (o MESMO numero que a alca do aro -- dois gestos, um valor),\n     \
                  Order (a posicao dela ao longo da corda, contando de 1) e Wrap.\n     \
                  EXPERIMENTE: selecione 'Zigzag Rope Wheel 2' e marque 'Under' --\n     \
                  a corda salta para o outro lado daquela roldana (medido: o lado\n     \
                  resolvido vai de -1 para +1). 'Auto' e o algoritmo decidindo;\n     \
                  Over/Under sao o escape manual, e ate esta secao eles nao tinham\n     \
                  gesto nenhum -- eram estado salvo em arquivo que nada alcancava.\n  \
               6. ACRESCENTE uma roldana: selecione a corda ('Simple Rope') na Hierarquia\n     \
                  e clique 'Add Wheel' no fim da secao do joint. Ela nasce SOBRE a corda\n     \
                  (no meio do ultimo trecho, para nao dar um puxao) e vira um objeto na\n     \
                  Hierarquia -- arraste o dot dela para onde a corda tem de passar.\n     \
                  Para tirar uma, delete o objeto: uma roldana e um objeto como outro\n     \
                  qualquer, entao Delete e Ctrl+Z ja funcionam.\n  \
               7. E o gesto de CRIAR, pelas DUAS rotas:\n     \
                  (a) selecione dois corpos, escolha 'Pulley' no seletor 'Join As' da\n     \
                      secao Physics Body e clique em Join;\n     \
                  (b) OU aperte sobre um corpo no canvas, arraste e solte sobre o outro.\n     \
                  Nas duas, DUAS roldanas nascem como objetos na Hierarquia, acima de cada\n     \
                  corpo, e a corda ja nasce esticada.\n  \
               8. E o que NAO esta mais la: o campo 'Ratio'. Ele vendia uma talha que a\n     \
                  fisica nao tem -- ver o item 3.",
            simple_drop = MEASURED_SIMPLE_LOAD_DROP,
            simple_rise = MEASURED_SIMPLE_CW_RISE,
            zig_drop = MEASURED_ZIGZAG_LOAD_DROP,
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_pulley_tests.rs"]
mod tests;

// ---------------------------------------------------------------------------
// Cena 59 — O GUINCHO (W-Pulley W2).
// ---------------------------------------------------------------------------

const HOIST: [f32; 4] = [0.45, 0.80, 0.95, 1.0];
const LOWER: [f32; 4] = [0.95, 0.55, 0.45, 1.0];
const GEAR_BIG: [f32; 4] = [0.70, 0.55, 0.95, 1.0];
const GEAR_SMALL: [f32; 4] = [0.55, 0.95, 0.80, 1.0];
pub(crate) const POST: [f32; 4] = [0.35, 0.36, 0.40, 1.0];

/// A altura do braço do guindaste — onde os tambores ficam.
pub(crate) const BOOM_Y: f32 = 7.0;
/// Onde as cargas nascem.
pub(crate) const HOOK_Y: f32 = 1.0;

/// **Um guincho:** um poste ESTÁTICO amarra uma ponta da corda, a outra segura a
/// carga, e a roldana no alto é um TAMBOR.
///
/// O poste é estático porque é isso que um guincho é: quem recolhe está preso, e
/// quem sobe é a carga. Com os dois lados dinâmicos o recolhimento é dividido
/// entre eles, o que é outra máquina (e é o elevador da cena 58).
fn winch(world: &mut World, tag: &str, x: f32, radius: f32, omega_deg: f32, rgba: [f32; 4]) {
    let post = format!("{tag} Post");
    let load = format!("{tag} Load");
    let rope = format!("{tag} Rope");
    world.spawn((
        Name::new(post.clone()),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.15,
                half_y: 0.15,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], POST),
        Transform::from_translation(Vec2::new(x - 2.2, BOOM_Y)),
    ));
    ball(world, &load, x, 2.0, rgba, R);
    {
        // A carga nasce embaixo, não em `START_Y`.
        let mut q = world.query::<(&Name, &mut Transform)>();
        for (n, mut t) in q.iter_mut(world) {
            if n.as_str() == load {
                t.translation.y = HOOK_Y;
            }
        }
    }
    world.spawn((
        Name::new(rope.clone()),
        PhysicsJoint {
            body_a: stable_name_id(&post),
            body_b: stable_name_id(&load),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(x - 2.2, BOOM_Y)),
    ));
    world.spawn((
        Name::new(format!("{rope} Wheel 1")),
        PulleyWheel {
            rope: stable_name_id(&rope),
            order: 0,
            radius,
            wrap: WrapSide::Auto,
            motor_speed: omega_deg.to_radians(),
            break_enabled: false,
            break_force: PulleyWheel::DEFAULT_BREAK_FORCE,
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(x, BOOM_Y)),
    ));
}

/// **Quanto o gancho AZUL sobe em 2 s**, metros — `ω·r` com `ω = 60°/s` e
/// `r = 0,45` dá 0,942 m/s de corda, e a carga parte do repouso.
pub(crate) const MEASURED_HOIST_RISE: f32 = 0.93;
/// **Quanto o VERMELHO desce**, com o mesmo tambor girando ao contrário.
pub(crate) const MEASURED_LOWER_DROP: f32 = 0.95;
/// **A razão entre os dois tambores do par ROXO/VERDE** — o câmbio. Os raios
/// estão em 0,60 e 0,20, então a razão TEM de ser 3, e ela é: 1,245 contra
/// 0,414 m em 2 s.
pub(crate) const MEASURED_GEAR_RATIO: f32 = 3.01;

/// Monta a cena 59.
pub(crate) fn build_winch(world: &mut World) {
    world.spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 16.0,
                half_y: 0.3,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [32.0, 0.6], GROUND),
        Transform::from_translation(Vec2::new(0.0, -0.6)),
    ));
    // Recolhe · paga corda · e o par que mostra o CÂMBIO: mesmo motor, tambores
    // de raios diferentes.
    winch(world, "Hoist", -9.0, 0.45, 60.0, HOIST);
    winch(world, "Lower", -3.0, 0.45, -60.0, LOWER);
    winch(world, "Gear Big", 3.0, 0.60, 60.0, GEAR_BIG);
    winch(world, "Gear Small", 9.0, 0.20, 60.0, GEAR_SMALL);
}

/// **O que a cena spawna FORA do quadro** — `None` quando tudo cabe.
///
/// ⚠️ **Esta porta nasceu de um report:** *"selecionar Geared Rope Drum não mostra
/// três alças âmbar"*. A medição inocentou o publicador de alças (ele devolvia as
/// três) e a causa era o ENQUADRAMENTO — a câmera padrão mostra `y ∈ [−5, +5]`
/// (`center = (0,0)`, `height_world = 10`; o `main.rs` o diz na definição do
/// `WORLD_HALF`) e os tambores destas cenas moram entre **7 e 12 metros**. As
/// alças eram desenhadas metros acima do topo da tela, e a feature parecia morta.
///
/// **A lei que a porta carrega:** *uma cena de smoke enquadra o que ela pede que
/// se olhe.* As alturas de um rig são escolhidas por motivo FÍSICO (a pista de que
/// um contrapeso precisa) e um número desses não sabe nada sobre o que cabe na
/// tela — as duas coisas têm de ser reconciliadas por alguém, e um gate é quem
/// lembra.
///
/// Só o eixo Y: a largura visível depende do aspecto da janela, que a cena não
/// conhece. `skip` são os nomes cujo `Transform` é o CENTRO de algo feito para ser
/// atravessado (o chão), não para ser visto inteiro.
///
/// ⚠️ **`cfg(test)`:** os dois chamadores são gates. Um `pub(crate)` sem chamador
/// de produto não é código morto silencioso — é uma **segunda resposta** esperando
/// alguém chamá-la (a lição do `warp_axis` e do `serial_side`).
#[cfg(test)]
#[must_use]
pub(crate) fn outside_frame(
    world: &mut World,
    centre: [f32; 2],
    height: f32,
    skip: &[&str],
) -> Option<(String, f32)> {
    let (top, bottom) = (centre[1] + height * 0.5, centre[1] - height * 0.5);
    let mut q = world.query::<(&Name, &Transform)>();
    let mut worst = None;
    for (n, t) in q.iter(world) {
        if skip.contains(&n.as_str()) {
            continue;
        }
        let y = t.translation.y;
        if y > top || y < bottom {
            worst = Some((n.as_str().to_string(), y));
        }
    }
    worst
}
