//! `PH2D_ORDEM_SMOKE=1|2` — a Hierarchy manda no desenho e no clique, em TODA família (GEF 19): sete objectos de
//! sete tipos, sobrepostos; `=1` a ordem da tabela, `=2` invertida. Junta-se a `motion_objeto_smoke`, que cria a
//! «Pile» e o «Swarm»; as outras famílias nascem pela shell.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, RootOrder, SimWorld, Transform, World};
use ph2d_render::Sprite;

/// A variável e os níveis.
pub const ROUTER: ph2d_app_host::SmokeRouter = ph2d_app_host::SmokeRouter {
    env: "PH2D_ORDEM_SMOKE",
    max_level: 2,
};

/// O nível pedido em `get` (`1` ou `2`).
#[must_use]
pub fn nivel_in(get: impl Fn(&str) -> Option<String>) -> Option<u32> {
    get(ROUTER.env)?
        .trim()
        .parse()
        .ok()
        .filter(|n| (1..=ROUTER.max_level).contains(n))
}

#[must_use]
pub fn nivel() -> Option<u32> {
    static NIVEL: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *NIVEL.get_or_init(|| nivel_in(|k| std::env::var(k).ok()))
}

static MONTADA: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A cena monta uma vez por processo: `true` na primeira pergunta.
#[must_use]
pub fn monta_agora() -> bool {
    !MONTADA.swap(true, std::sync::atomic::Ordering::Relaxed)
}

/// Uma peça da cena.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Peca {
    Imagem,
    Pilha,
    Traco,
    Rosto,
    Enxame,
    Macro,
    Camera,
}

/// A ordem do `=1`, de cima para baixo na Hierarchy (o 1.º desenha por baixo de todos).
pub const ORDEM: [Peca; 7] = [
    Peca::Imagem,
    Peca::Pilha,
    Peca::Traco,
    Peca::Rosto,
    Peca::Enxame,
    Peca::Macro,
    Peca::Camera,
];

/// O nome das peças que esta cena põe (a «Pile» e o «Swarm» são do smoke do Motion).
#[must_use]
pub fn nome(p: Peca) -> &'static str {
    match p {
        Peca::Imagem => "Card",
        Peca::Pilha => "Pile",
        Peca::Traco => "Ink",
        Peca::Rosto => "Face",
        Peca::Enxame => "Swarm",
        Peca::Macro => "Macro",
        Peca::Camera => "Camera",
    }
}

/// A pose (m) e a escala das peças que esta cena arruma — todas sobre a «Pile» e o «Swarm».
#[must_use]
pub fn pose(p: Peca) -> Option<([f32; 2], [f32; 2])> {
    match p {
        Peca::Imagem => Some(([1.5, 1.2], [5.0, 3.2])),
        Peca::Traco => Some(([1.2, 1.0], [1.6, 1.6])),
        Peca::Rosto => Some(([2.6, 1.0], [6.0, 6.0])),
        Peca::Macro => Some(([0.4, 2.2], [1.0, 1.0])),
        Peca::Camera => Some(([1.5, 1.4], [1.0, 1.0])),
        // As taças da cena 114 ficam em y ∈ [−3, −1,4] m locais: sobem para dentro da vista, sobre o cartão.
        Peca::Pilha => Some(([1.5, 3.5], [1.0, 1.0])),
        Peca::Enxame => None,
    }
}

/// A cor da imagem: um cartão azul-esverdeado, opaco.
const CARTAO: [f32; 4] = [0.18, 0.55, 0.62, 1.0];

/// A imagem, pelo menu Add ▸ Game ▸ Sprite (a porta do produto), tingida.
pub fn imagem(sim: &mut SimWorld, portas: crate::object_add::Portas<'_>) -> Option<u64> {
    let bits = crate::object_add::add(crate::object_add::SPRITE, sim, portas)?.ok()?;
    if let Some(mut s) = sim.world_mut().get_mut::<Sprite>(Entity::from_bits(bits)) {
        s.tint = CARTAO;
    }
    Some(bits)
}

/// Nome e pose da peça (se esta cena a arruma).
pub fn arruma(world: &mut World, bits: u64, p: Peca) {
    let Ok(mut e) = world.get_entity_mut(Entity::from_bits(bits)) else {
        return;
    };
    e.insert(Name::new(nome(p)));
    if let (Some((at, escala)), Some(mut t)) = (pose(p), e.get_mut::<Transform>()) {
        t.translation = Vec2::new(at[0], at[1]);
        t.scale = Vec2::new(escala[0], escala[1]);
    }
}

/// A peça que tem o nome dela — para a «Pile» e o «Swarm», que o smoke do Motion criou.
#[must_use]
pub fn por_nome(world: &mut World, p: Peca) -> Option<u64> {
    let mut q = world.query::<(Entity, &Name)>();
    q.iter(world)
        .find(|(_, n)| n.as_str() == nome(p))
        .map(|(e, _)| e.to_bits())
}

/// O lugar de cada peça na Hierarchy: a tabela no `=1`, invertida no `=2`.
#[must_use]
pub fn lugar(p: Peca, nivel: u32) -> u32 {
    let i = ORDEM.iter().position(|q| *q == p).unwrap_or(0);
    let i = if nivel == 2 { ORDEM.len() - 1 - i } else { i };
    u32::try_from(i).unwrap_or(u32::MAX)
}

/// Dá o lugar a cada peça, por baixo das raízes que já havia (que não mudam de ordem entre si).
pub fn ordena(world: &mut World, nivel: u32, pecas: &[(Peca, u64)]) {
    let nossas: Vec<Entity> = pecas.iter().map(|(_, b)| Entity::from_bits(*b)).collect();
    for &e in &nossas {
        if let Ok(mut e) = world.get_entity_mut(e) {
            e.remove::<RootOrder>();
        }
    }
    let _ = ph2d_ecs::assign_missing_root_order(world);
    let base = world
        .query::<(Entity, &RootOrder)>()
        .iter(world)
        .filter(|(e, _)| !nossas.contains(e))
        .map(|(_, r)| r.0.saturating_add(1))
        .max()
        .unwrap_or(0);
    for &(p, bits) in pecas {
        if let Ok(mut e) = world.get_entity_mut(Entity::from_bits(bits)) {
            e.insert(RootOrder(base.saturating_add(lugar(p, nivel))));
        }
    }
}

/// O que a cena diz no terminal.
#[must_use]
pub fn mensagem(nivel: u32, pecas: usize) -> String {
    let ordem = if nivel == 2 {
        "REVERSED: Camera on top of the Hierarchy, Card at the bottom — now the Card covers everything"
    } else {
        "Card on top of the Hierarchy (drawn at the BACK), Camera at the bottom"
    };
    format!(
        "[ordem-smoke ={nivel}] {pecas}/7 objects of seven kinds overlap: Card (image), Pile (Motion \
         shapes falling — the simulation), Ink (a Flip stroke), Face (a painted Pixel Sprite), Swarm \
         (Motion copies pulled by force fields), Macro and Camera (icons). Order: {ordem}. Higher in \
         the Hierarchy = drawn BELOW. Click where they overlap: the first click takes what you SEE \
         on top; each click again goes one step down."
    )
}

#[cfg(test)]
#[path = "ordem_smoke_tests.rs"]
mod tests;
