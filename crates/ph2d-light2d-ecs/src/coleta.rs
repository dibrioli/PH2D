//! **A coleta** — `World → CenaDeLuz`. Pura, CPU, ordem estável.
//!
//! - `Luz` `Ponto`: disco de raio `tamanho_m` com radiância `cor·intensidade/(2·tamanho_m)`, ou
//!   `Emissor::Ponto` (intensidade) quando `tamanho_m == 0`.
//! - `Luz` `Ambiente`: soma `cor·intensidade/(2π)` no céu.
//! - `Sprite` + `SpriteEmissive > 0`: o QUAD do sprite, radiância `tint∘self_tint` (rgb·a) ×
//!   intensidade. ⚠️ Os texels não estão no `World` (só o `AssetId`); a cor do quad é a do tint.
//!   A captura por texel é da C0 (W2).
//! - `Sprite` + `BloqueiaLuz`: o quad, com a opacidade. Um sprite que emite NÃO entra também como
//!   oclusor (o emissor já é opaco).
//!
//! Fora: objetos escondidos na árvore; valores não finitos (um NaN apaga, nunca envenena — a lei do
//! `SpriteEmissive::clamped`); intensidade, raio ou alcance que não iluminam nada.

use std::collections::BTreeMap;

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_ecs::{GlobalTransform, SpriteEmissive, StableId, is_hidden_in_tree};
use ph2d_render::Sprite;

use crate::cena::{CenaDeLuz, Emissor, Oclusor, Rgb};
use crate::componentes::{BloqueiaLuz, Luz, TipoDeLuz};

/// Chave de ordem: o `StableId` (sobrevive a undo e save), depois a `Entity`.
type Chave = (Option<u64>, Entity);

fn chave(world: &World, e: Entity) -> Chave {
    (world.get::<StableId>(e).map(|s| s.0), e)
}

/// Os cantos do quad no mundo, na ordem do quad do shader de sprite.
fn quad(g: &GlobalTransform, sprite: &Sprite) -> Vec<[f32; 2]> {
    let [ax, ay, bx, by, tx, ty] = g.affine();
    [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
        .iter()
        .map(|q: &[f32; 2]| {
            let lx = sprite.anchor[0] + q[0] * sprite.size[0];
            let ly = sprite.anchor[1] + q[1] * sprite.size[1];
            [lx * ax + ly * bx + tx, lx * ay + ly * by + ty]
        })
        .collect()
}

fn finito(c: Rgb) -> bool {
    c.iter().all(|x| x.is_finite())
}

fn acende(c: Rgb) -> bool {
    finito(c) && c.iter().all(|&x| x >= 0.0) && c.iter().any(|&x| x > 0.0)
}

/// O alcance útil: `> 0` ou `+∞`; NaN e `≤ 0` apagam a luz.
fn alcance(a: f32) -> Option<f32> {
    (a > 0.0).then_some(a)
}

/// A ÚNICA porta por onde emissores e oclusores chegam a um solver.
#[must_use]
pub fn cena_de_luz(world: &World) -> CenaDeLuz {
    let mut emissores: BTreeMap<Chave, Emissor> = BTreeMap::new();
    let mut oclusores: BTreeMap<Chave, Oclusor> = BTreeMap::new();
    let mut ceu = [0.0_f32; 3];

    if let Some(mut q) = world.try_query::<(Entity, &Luz, Option<&GlobalTransform>)>() {
        for (e, luz, g) in q.iter(world) {
            if is_hidden_in_tree(world, e) {
                continue;
            }
            let fluencia = luz.cor.map(|c| c * luz.intensidade);
            if !acende(fluencia) {
                continue;
            }
            if luz.tipo == TipoDeLuz::Ambiente {
                for (c, f) in ceu.iter_mut().zip(fluencia) {
                    *c += f / std::f32::consts::TAU;
                }
                continue;
            }
            let (Some(alcance_m), Some(g)) = (alcance(luz.alcance_m), g) else {
                continue;
            };
            let t = g.translation();
            let centro = [t.x, t.y];
            let emissor = if luz.tamanho_m == 0.0 {
                Emissor::Ponto {
                    centro,
                    intensidade: fluencia,
                    alcance_m,
                }
            } else if luz.tamanho_m > 0.0 && luz.tamanho_m.is_finite() {
                Emissor::Disco {
                    centro,
                    raio: luz.tamanho_m,
                    radiancia: fluencia.map(|f| f / (2.0 * luz.tamanho_m)),
                    alcance_m,
                }
            } else {
                continue;
            };
            emissores.insert(chave(world, e), emissor);
        }
    }

    if let Some(mut q) = world.try_query::<(Entity, &Sprite, &GlobalTransform)>() {
        for (e, sprite, g) in q.iter(world) {
            if is_hidden_in_tree(world, e) {
                continue;
            }
            let k = world.get::<SpriteEmissive>(e).map_or(0.0, |s| s.clamped());
            if k > 0.0 {
                let (t, s) = (sprite.tint, sprite.self_tint);
                let alfa = t[3] * s[3];
                let radiancia = [0, 1, 2].map(|i| t[i] * s[i] * alfa * k);
                if acende(radiancia) {
                    emissores.insert(
                        chave(world, e),
                        Emissor::Poligono {
                            vertices: quad(g, sprite),
                            radiancia,
                            alcance_m: f32::INFINITY,
                        },
                    );
                    continue;
                }
            }
            if let Some(b) = world.get::<BloqueiaLuz>(e)
                && b.opacidade > 0.0
            {
                oclusores.insert(
                    chave(world, e),
                    Oclusor {
                        vertices: quad(g, sprite),
                        opacidade: b.opacidade.min(1.0),
                    },
                );
            }
        }
    }

    CenaDeLuz {
        emissores: emissores.into_values().collect(),
        oclusores: oclusores.into_values().collect(),
        ceu,
    }
}
