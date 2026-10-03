//! Os gates da NAVEGAÇÃO na arena (plano 30, W8) — pelo mesmo arnês do quadro inteiro
//! ([`super::Jogo`]): os morcegos nascem da fábrica, a ponte conduz-os, e o recomeço varre-os.
//!
//! ⚠️ **«Pisar» é a DISTÂNCIA do centro ao rectângulo contra o raio** (a quina recuada é redonda) —
//! a régua que a W7 errou duas vezes com cantos vivos.

use super::*;
use ph2d_ecs::MasterRoot;
use ph2d_editor_core::nav_edits::{NavAlvoModo, NavFieldEdit};

/// O raio do corpo de um morcego.
const R_MORCEGO: f32 = LADO_DO_MORCEGO / 2.0;
/// Uma folga de medida: o deslize do mover encosta a `1e-3` da forma recuada.
const FOLGA: f32 = 0.02;

/// A distância de `p` ao rectângulo `centro ± meio` (zero dentro).
fn dist_rect(p: Vec2, centro: [f32; 2], meio: [f32; 2]) -> f32 {
    let dx = ((p.x - centro[0]).abs() - meio[0]).max(0.0);
    let dy = ((p.y - centro[1]).abs() - meio[1]).max(0.0);
    dx.hypot(dy)
}

fn dist_muro(p: Vec2) -> f32 {
    dist_rect(
        p,
        [MURO_X, (MURO_CIMA + MURO_BAIXO) / 2.0],
        [MURO_MEIO, (MURO_CIMA - MURO_BAIXO) / 2.0],
    )
}

fn dist_lava(p: Vec2) -> f32 {
    dist_rect(p, LAVA_XY, [LAVA_WH[0] / 2.0, LAVA_WH[1] / 2.0])
}

impl Jogo {
    fn pos(&self, e: Entity) -> Vec2 {
        self.sim
            .world()
            .get::<Transform>(e)
            .map_or(Vec2::new(f32::NAN, f32::NAN), |t| t.translation)
    }

    /// O MOLDE com este nome (o que a fábrica copia).
    fn molde(&mut self, nome: &str) -> Entity {
        let w = self.sim.world_mut();
        let mut q = w.query_filtered::<(Entity, &Name), (
            bevy_ecs::query::With<MasterRoot>,
            bevy_ecs::query::Without<Spawned>,
        )>();
        q.iter(w)
            .find(|(_, n)| n.as_str() == nome)
            .map(|(e, _)| e)
            .expect("o molde existe")
    }

    /// ⛔ **O CONTROLO**: o molde do morcego volta a ser o projéctil teleguiado de antes da W8.
    fn morcego_teleguiado(&mut self) {
        let m = self.molde("Morcego");
        let mut em = self.sim.world_mut().entity_mut(m);
        em.remove::<(NavAgent, TopDownPlayer)>();
        em.insert(ProjectileMotion::from_law(
            ProjectileLaw {
                initial_speed: 1.0,
                max_speed: MORCEGO_RAPIDEZ,
                homing_accel: 5.0,
                bounciness: 0.6,
                max_bounces: u8::MAX,
                ..ProjectileLaw::default()
            },
            stable_name_id(HEROI),
        ));
    }
}

/// O que um morcego fez até morder (ou até ao fim do prazo).
struct Voo {
    mordeu: bool,
    /// A menor distância do centro dele ao muro.
    rente_ao_muro: f32,
    /// Passou por BAIXO do muro (à altura dele em `x`, abaixo da ponta).
    por_baixo: bool,
    /// O `x` mais à direita a que chegou.
    x_max: f32,
}

fn voa_um_morcego(j: &mut Jogo, quadros: u32) -> Voo {
    j.quadro(&[SINAL_MORCEGO]);
    let m = *j.copias("Morcego").first().expect("o morcego não nasceu");
    let mut v = Voo {
        mordeu: false,
        rente_ao_muro: f32::INFINITY,
        por_baixo: false,
        x_max: f32::NEG_INFINITY,
    };
    for _ in 0..quadros {
        let evs = j.quadro(&[]).eventos;
        if danos(&evs, j.heroi).iter().any(|(s, _)| *s == m) {
            v.mordeu = true;
            break;
        }
        let p = j.pos(m);
        v.rente_ao_muro = v.rente_ao_muro.min(dist_muro(p));
        v.por_baixo |= (p.x - MURO_X).abs() <= MURO_MEIO + R_MORCEGO && p.y < MURO_BAIXO;
        v.x_max = v.x_max.max(p.x);
    }
    v
}

/// ⭐⭐⭐ **Um morcego DÁ A VOLTA ao muro e morde** — por baixo dele, sem nunca o atravessar; e o
/// CONTROLO, o teleguiado de antes, fica do lado de cá e não morde.
///
/// **Mutações que devem sangrar:** o morcego sem `NavAgent` (o molde de antes); a região sem as
/// paredes (`obstacle_layers = 0`).
#[test]
fn um_morcego_da_a_volta_ao_muro() {
    // ~11 m de caminho a 2,2 m/s; o prazo é o dobro.
    const PRAZO: u32 = 600;
    let mut j = Jogo::da_cena();
    let v = voa_um_morcego(&mut j, PRAZO);
    assert!(v.mordeu, "o morcego não mordeu em 10 s");
    assert!(v.por_baixo, "o morcego não passou por baixo do muro");
    assert!(
        v.rente_ao_muro >= R_MORCEGO - FOLGA,
        "o morcego entrou no muro: centro a {} m",
        v.rente_ao_muro
    );

    let mut c = Jogo::da_cena();
    c.morcego_teleguiado();
    let v = voa_um_morcego(&mut c, PRAZO);
    assert!(
        !v.mordeu && v.x_max < MURO_X,
        "o CONTROLO passou o muro — a cena não prova a navegação (x máx {})",
        v.x_max
    );
}

/// `quadros` com o herói na lava: devolve a menor distância de um morcego à lava, a menor distância
/// de um morcego ao herói, e as mordidas.
fn heroi_na_lava(j: &mut Jogo, quadros: u32) -> (f32, f32, usize) {
    let mut na_lava = f32::INFINITY;
    let mut ao_heroi = f32::INFINITY;
    let mut mordidas = 0;
    for _ in 0..quadros {
        let evs = j.quadro(&[]).eventos;
        let h = j.heroi;
        mordidas += danos(&evs, h).iter().filter(|(s, _)| *s != h).count();
        let ph = j.pos(h);
        for m in j.copias("Morcego") {
            let p = j.pos(m);
            na_lava = na_lava.min(dist_lava(p));
            ao_heroi = ao_heroi.min(p.distance(ph));
        }
    }
    (na_lava, ao_heroi, mordidas)
}

/// ⭐⭐⭐ **Com o herói DENTRO da lava, os morcegos esperam na BORDA** — vêm até ela (a régua de
/// que vieram é o CONTROLO de que não ficaram presos noutro sítio) e não a pisam, logo não mordem.
/// E sem o `Avoid Harm` no molde, um entra.
///
/// ⚠️ A lava fere o morcego (sem equipa, fogo, e ele não resiste): é o `Damage::magoa` que a faz furo
/// na malha DELE — a decisão do dono no plano 30 §11.1.
///
/// **Mutações que devem sangrar:** o `avoid_harm` do morcego desligado; a lava sem queimadura.
#[test]
fn os_morcegos_esperam_na_borda_da_lava() {
    // Um morcego a cada 3 s; o 1.º chega à borda em ~4 s.
    const QUADROS: u32 = 600;
    let mut j = Jogo::novo(LAVA_XY, 0.0);
    let (na_lava, _, mordidas) = heroi_na_lava(&mut j, QUADROS);
    assert!(
        na_lava >= R_MORCEGO - FOLGA,
        "um morcego pisou a lava: centro a {na_lava} m da borda"
    );
    assert!(
        na_lava <= R_MORCEGO + 0.3,
        "nenhum morcego chegou à borda (o mais perto a {na_lava} m)"
    );
    assert_eq!(mordidas, 0, "um morcego mordeu o herói dentro da lava");

    let mut c = Jogo::novo(LAVA_XY, 0.0);
    let m = c.molde("Morcego");
    c.sim
        .world_mut()
        .get_mut::<NavAgent>(m)
        .expect("o molde é um agente")
        .avoid_harm = false;
    let (na_lava, ao_heroi, _) = heroi_na_lava(&mut c, QUADROS);
    assert!(
        na_lava < R_MORCEGO - FOLGA && ao_heroi < 1.0,
        "sem Avoid Harm o morcego não entrou na lava atrás do herói ({na_lava} m · {ao_heroi} m)"
    );
}

/// ⭐⭐⭐ **O tutorial 03: a Salamandra posta a PERSEGUIR atravessa a lava** — o gesto do tutorial é
/// *Add Component → Nav Agent* (a cascata traz o mover e a semente desliga-lhe o teclado: o gate
/// `escolher_um_agente_na_paleta_entrega_um_mover_que_o_ouve` da shell) e o alvo *Object → Heroi*
/// pela porta do Inspector. Ela é imune ao fogo: a lava não a magoa e não é furo na malha dela.
///
/// **Mutações que devem sangrar:** a Salamandra estática (o mover não a move); a imunidade ao fogo
/// tirada (a lava passa a furo e ela espera na borda).
#[test]
fn a_salamandra_posta_a_perseguir_atravessa_a_lava() {
    let mut j = Jogo::novo(LAVA_XY, 0.0);
    j.corre(20);
    let sal = *j
        .copias("Salamandra")
        .first()
        .expect("a Salamandra não nasceu");
    {
        let mut mover = TopDownPlayer::default();
        mover.conduzido_pela_navegacao();
        j.sim
            .world_mut()
            .entity_mut(sal)
            .insert((mover, NavAgent::default()));
    }
    let bits = sal.to_bits();
    for edit in [
        NavFieldEdit::AlvoModo(NavAlvoModo::Objecto),
        NavFieldEdit::AlvoNome(HEROI.to_owned()),
    ] {
        assert!(crate::nav_inspector::apply_nav_edit(
            j.sim.world_mut(),
            bits,
            &edit
        ));
    }
    let mut na_lava = f32::INFINITY;
    let mut evs = Vec::new();
    for _ in 0..900 {
        evs.extend(j.quadro(&[]).eventos);
        na_lava = na_lava.min(dist_lava(j.pos(sal)));
    }
    assert!(
        na_lava == 0.0,
        "a Salamandra não entrou na lava (centro a {na_lava} m)"
    );
    assert!(danos(&evs, sal).is_empty(), "a lava feriu a Salamandra");
    assert!(
        j.pos(sal).distance(j.pos(j.heroi)) < 1.5,
        "a Salamandra não chegou ao herói"
    );
}
