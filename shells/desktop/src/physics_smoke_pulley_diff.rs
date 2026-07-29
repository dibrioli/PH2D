//! **A cena do TAMBOR DIFERENCIAL** (`PH2D_PHYSICS_SMOKE=62`, W-Pulley W4).
//!
//! Irmã de `physics_smoke_pulley{,_tackle,_break}.rs` pelo cap de 600 LOC, e o
//! corte é o assunto: lá moram o elevador, o guincho, a ruptura e a talha; aqui,
//! a vantagem mecânica **CONTÍNUA** — a que não é 2, 4 ou 8, e sim o quociente de
//! dois diâmetros que o artista dimensiona.
//!
//! Os números da mensagem saem da sonda `probe_smoke_62`, rodada sobre ESTAS
//! constantes.

use super::physics_smoke_pulley::{GROUND, ball};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, PulleyWheel, RigidBody, WrapSide,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const GEARED: [f32; 4] = [0.45, 0.90, 0.55, 1.0];
const PLAIN: [f32; 4] = [0.95, 0.45, 0.45, 1.0];
const COUNTER: [f32; 4] = [0.30, 0.32, 0.40, 1.0];

/// **A carga**, kg. Os DOIS rigs a carregam.
const LOAD_MASS: f32 = 3.0;
/// **O contrapeso**, kg — e os dois rigs têm o mesmo.
const COUNTER_MASS: f32 = 1.0;
/// O raio por onde a corda ENTRA no tambor.
const R_IN: f32 = 0.5;
/// O raio por onde ela SAI — só o rig da esquerda o tem.
///
/// `R_IN / R_OUT` = **4**, então um contrapeso de 1 kg segura até 4 kg. A carga é
/// 3 kg de propósito: acima do que a roldana comum segura (1 kg) e abaixo do que
/// o tambor segura (4 kg), que é o que faz os dois rigs discordarem.
const R_OUT: f32 = 0.125;
/// Meia-distância entre os dois corpos pendurados.
const SPAN: f32 = 1.2;
/// Onde os dois corpos nascem.
const HANG_Y: f32 = 5.0;
/// O topo do chão.
///
/// ⚠️ **Alto**, e o motivo é a ENERGIA: no rig comum os dois lados andam ~1:1, e
/// quanto mais longe a carga cai mais depressa o contrapeso leve chega ao tambor.
/// Com o chão lá embaixo ele passava VOANDO por cima dele (y=12,5 contra um tambor
/// a 12,0), por INÉRCIA, depois de a carga já ter pousado. Encurtar a queda é o que
/// limita a velocidade — subir o tambor só adia.
const FLOOR_TOP: f32 = 2.0;
/// A altura do tambor.
///
/// ⚠️ **Alto o bastante para o contrapeso NUNCA o alcançar**, e a medição escolheu
/// o número: no rig comum os dois lados andam ~1:1, então a carga cai os 5 m até o
/// chão e o contrapeso sobe outro tanto. Com o tambor no `BOOM_Y` de 7,0 dos
/// irmãos ele chegava a **6,92** — encostado —, a rota degenerava (o caso que o W1
/// nomeou: um corpo que passa da própria roldana não tem tangente) e a carga
/// passava a cair LIVRE, atravessando o chão até y=−1,12. A 12,0 sobra folga.
const DRUM_Y: f32 = 12.0;

/// Um sarilho: o contrapeso à esquerda, a carga à direita, um tambor no teto.
///
/// `geared = false` é o **CONTROLE**: a mesma montagem com uma roldana COMUM, onde
/// a vantagem é 1 porque a tensão de uma corda que desliza é uniforme — e é
/// exatamente por isso que o `ratio` do W-Pulley saiu (§3 do plano).
fn windlass(world: &mut World, tag: &str, x: f32, geared: bool, rgba: [f32; 4]) {
    let counter = format!("{tag} Counterweight");
    let load = format!("{tag} Load");
    let rope = format!("{tag} Rope");
    ball(world, &counter, x - SPAN, COUNTER_MASS, COUNTER, 0.22);
    ball(world, &load, x + SPAN, LOAD_MASS, rgba, 0.3);
    for (name, y) in [(&counter, HANG_Y), (&load, HANG_Y)] {
        let mut q = world.query::<(&Name, &mut Transform)>();
        for (n, mut t) in q.iter_mut(world) {
            if n.as_str() == *name {
                t.translation.y = y;
            }
        }
    }
    world.spawn((
        Name::new(rope.clone()),
        PhysicsJoint {
            // O contrapeso é a ponta A: é o lado por onde a corda ENTRA no
            // tambor, e a engrenagem conta a partir dele.
            body_a: stable_name_id(&counter),
            body_b: stable_name_id(&load),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        // ⚠️ **O `Transform` de uma corda é a ÂNCORA EM A**, não o lugar do
        // tambor: ele é convertido para o frame local de A e vira o ponto onde a
        // corda se amarra. Pô-lo no tambor amarra a corda a um ponto rígido a 1,2 m
        // do contrapeso — um mastro invisível —, e a primeira versão desta cena fez
        // exatamente isso: os dois corpos foram arremessados (contrapeso a y=20,5 e
        // a y=−48,0) e as duas cargas SUBIRAM.
        Transform::from_translation(Vec2::new(x - SPAN, HANG_Y)),
    ));
    world.spawn((
        Name::new(format!("{rope} Drum")),
        PulleyWheel {
            rope: stable_name_id(&rope),
            order: 0,
            radius: R_IN,
            // **O segundo diâmetro, e é só isto que difere entre os dois rigs.**
            radius_out: if geared { R_OUT } else { 0.0 },
            wrap: WrapSide::Auto,
            ..PulleyWheel::default()
        },
        Transform::from_translation(Vec2::new(x, DRUM_Y)),
    ));
}

/// **Quanto a carga do rig ENGRENADO SOBE em 2 s**, metros.
///
/// Ela sobe, e não apenas fica parada: 3 kg está ABAIXO dos 4 kg que o tambor
/// 0,50 → 0,125 segura, então o contrapeso de 1 kg desce e a LEVANTA. É a
/// demonstração inteira num gesto — a mesma massa que o rig ao lado deixa cair.
pub(crate) const MEASURED_GEARED_RISE: f32 = 0.64;
/// **Quanto a carga do CONTROLE cai em 2 s**, metros — ela para aí porque
/// encontra o CHÃO, não porque a corda a pegou.
pub(crate) const MEASURED_PLAIN_DROP: f32 = 2.70;

/// **O ENQUADRAMENTO desta cena** — ver `physics_smoke_pulley::outside_frame`
/// para a lei e o report que a produziu.
///
/// ⚠️ **A mensagem desta cena manda selecionar o tambor e OLHAR o segundo anel
/// aparecer** — e com a câmera padrão (`y ∈ [−5, +5]`) o tambor a 12 m nunca
/// esteve na tela. A instrução era inverificável, e ninguém notou porque o que a
/// cena demonstra (as cargas indo para lados opostos) acontece embaixo.
pub(crate) const CAMERA_CENTRE: [f32; 2] = [0.0, 6.5];
/// A altura de mundo que cobre do chão ao topo dos tambores com folga.
pub(crate) const CAMERA_HEIGHT: f32 = 15.0;

/// ⚠️ **Enquadrar não é AFASTAR a câmera até tudo caber** — o modo de "consertar"
/// um gate de fit que o esvazia. Compile-time de propósito: mexer na const acima
/// para além disto quebra o BUILD, não um teste que alguém pode filtrar.
const _: () = assert!(CAMERA_HEIGHT < 20.0);

/// Monta a cena 62.
pub(crate) fn build_differential(world: &mut World) {
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
        Transform::from_translation(Vec2::new(0.0, FLOOR_TOP - 0.3)),
    ));
    windlass(world, "Diff", -4.5, true, GEARED);
    windlass(world, "Plain", 4.5, false, PLAIN);
}

#[cfg(test)]
#[path = "physics_smoke_pulley_diff_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 62 (W-Pulley W4).** Dois sarilhos com a MESMA carga e o MESMO
    /// contrapeso; a única diferença é o SEGUNDO diâmetro do tambor.
    pub(crate) fn physics_smoke_differential(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_differential(gfx.sim.world_mut());
        // Sem isto o tambor — que a mensagem manda selecionar — nasce 7 m acima
        // do topo da tela. Ver o doc de `CAMERA_CENTRE`.
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 62] O TAMBOR DIFERENCIAL -- a vantagem mecanica CONTINUA.\n  \
               Aperte B para ver os vinculos, e depois PLAY (o toggle Physics ja esta armado).\n\n  \
               Os DOIS sarilhos tem a MESMA carga ({load:.0} kg) e o MESMO contrapeso\n  \
               ({counter:.0} kg). A unica diferenca e o SEGUNDO diametro do tambor -- e eles\n  \
               andam para lados OPOSTOS.\n\n  \
               1. VERDE (esquerda) -- o tambor tem DOIS raios: a corda entra em {r_in:.2} m e\n     \
                  sai em {r_out:.3} m. Girar o eixo recolhe de um lado e paga do outro em\n     \
                  proporcoes diferentes, entao a vantagem e {gear:.0}x: 1 kg segura ate {hold:.0} kg.\n     \
                  O contrapeso DESCE e LEVANTA os {load:.0} kg (+{geared:.2} m em 2 s).\n     \
                  Repare nos DOIS aneis concentricos: e deles que o numero sai.\n  \
               2. VERMELHO (direita) -- o mesmo tambor com UM raio so. Numa corda que\n     \
                  DESLIZA a tensao e uniforme, logo a vantagem e 1: os {load:.0} kg vencem\n     \
                  1 kg, CAEM ({plain:.2} m) e batem no chao.\n\n  \
               (!) Ninguem digitou um \"{gear:.0}\" em lugar nenhum -- ele e o quociente de duas\n     \
               circunferencias que estao DESENHADAS. Foi por nao ter peca na cena que o\n     \
               campo `ratio` do W-Pulley saiu.\n\n  \
               AUTORE VOCE MESMO: selecione 'Plain Rope Drum' (o tambor da direita) na\n  \
               Hierarquia. Na secao 'Pulley Wheel' a row 'Out Radius (m)' esta em 0 --\n  \
               uma roldana comum. Digite {r_out:.3}: o segundo anel aparece e, no Play, a\n  \
               carga da direita para de cair. Voltar a 0 a solta.",
            load = LOAD_MASS,
            counter = COUNTER_MASS,
            r_in = R_IN,
            r_out = R_OUT,
            gear = R_IN / R_OUT,
            hold = COUNTER_MASS * R_IN / R_OUT,
            geared = MEASURED_GEARED_RISE,
            plain = MEASURED_PLAIN_DROP,
        );
    }
}
