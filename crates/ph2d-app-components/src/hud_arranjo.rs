//! ⭐ **O HUD que se arruma, vivo** — o passe que corre DEPOIS das âncoras e põe os irmãos de um canvas onde as
//! relações do [`UiArranjo`] mandam (ADR-0194, `docs/BlenderConstraints/06_W5_resto.md` §3.3).
//!
//! # A largura do rótulo é a caixa VIVA dele
//!
//! O rótulo já foi re-cozido com o texto DERIVADO (`hud_label_live`, antes do layout) e ancorado: a caixa que o
//! passe lê é a que se desenha. Uma segunda medição (moldar o texto outra vez) seria outra resposta para a mesma
//! caixa — a tinta tem lados que o avanço não tem. Por isso o conteúdo de uma `LarguraDoConteudo` é a largura
//! viva, e encolher abaixo dela escala o rótulo POR IGUAL (o texto não se espreme).
//!
//! # Preview, nunca documento
//!
//! O passe só DEVOLVE caixas; quem as publica é o layout vivo da shell, como faz com as âncoras.
//!
//! # O solver vive entre quadros
//!
//! Um [`Arranjo`] por canvas, remontado só quando as relações ou os irmãos mudam; as caixas e a vista entram
//! como edições (`atualiza_elementos`). Unidades: as LOCAIS do canvas (as do artista; a grelha de `1/256` é
//! dela), y para cima, a origem no centro da caixa de referência.

use std::collections::BTreeMap;

use ph2d_ecs::hud_arranjo::{FilhoVisto, Forca, Relacao, UiArranjo, Visto};
use ph2d_ecs::{ChildOf, Children, Entity, SimWorld, StableId, Transform, UiCanvas, VecShape};
use ph2d_hud_layout::{Arranjo, Caixa, Elemento};
use ph2d_vec_entities::entities::VecEntityMap;

use crate::hud_bridge::{View, anchor_frame_of};

/// Uma caixa de MUNDO: `(min, max)`.
pub type Caixa2 = ([f64; 2], [f64; 2]);

/// Um irmão que o arranjo muda de sítio neste quadro.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Movido {
    pub kid: Entity,
    pub de: Caixa2,
    pub para: Caixa2,
}

struct Vivo {
    lacos: Vec<(Relacao<StableId>, Forca)>,
    ids: Vec<StableId>,
    arranjo: Arranjo,
}

/// Os solvers vivos, um por canvas com arranjo, e o retrato do último quadro (para a ferramenta).
#[derive(Default)]
pub struct Arranjos {
    vivos: BTreeMap<Entity, Vivo>,
    visto: Vec<Visto>,
}

/// O canvas no mundo: a origem e a escala local → mundo.
#[derive(Clone, Copy)]
struct Moldura {
    t: [f64; 2],
    s: [f64; 2],
}

impl Moldura {
    fn local(self, (lo, hi): Caixa2) -> Caixa {
        let (x0, y0) = (
            (lo[0] - self.t[0]) / self.s[0],
            (lo[1] - self.t[1]) / self.s[1],
        );
        let (x1, y1) = (
            (hi[0] - self.t[0]) / self.s[0],
            (hi[1] - self.t[1]) / self.s[1],
        );
        Caixa {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        }
    }

    fn mundo(self, c: Caixa) -> Caixa2 {
        let p = |x: f64, y: f64| [self.t[0] + x * self.s[0], self.t[1] + y * self.s[1]];
        (p(c.x, c.y), p(c.x + c.w, c.y + c.h))
    }
}

/// Os canvas, pelos filhos que desenham (sem repetidos, ordem estável) — com ou sem arranjo: a ferramenta
/// precisa de ver o canvas antes da primeira seta.
fn canvases(sim: &SimWorld, map: &VecEntityMap) -> Vec<Entity> {
    let w = sim.world();
    let mut out: Vec<Entity> = Vec::new();
    for &bits in map.values() {
        let Some(pai) = w
            .get::<ChildOf>(Entity::from_bits(bits))
            .map(ChildOf::parent)
        else {
            continue;
        };
        if w.get::<UiCanvas>(pai).is_some() && !out.contains(&pai) {
            out.push(pai);
        }
    }
    out.sort_unstable();
    out
}

impl Arranjos {
    /// ⭐ **Um quadro.** `caixa_de(kid)` é a caixa de mundo que o filho desenha AGORA (depois das âncoras).
    /// Devolve só quem anda: um arranjo sem efeito não paga a cópia da geometria.
    pub fn arranja(
        &mut self,
        sim: &SimWorld,
        map: &VecEntityMap,
        vista: Option<View>,
        mut caixa_de: impl FnMut(Entity) -> Option<Caixa2>,
    ) -> Vec<Movido> {
        let w = sim.world();
        let todos = canvases(sim, map);
        self.vivos.retain(|e, _| todos.contains(e));
        let mut out = Vec::new();
        self.visto.clear();
        let vazio = UiArranjo::default();
        for canvas in todos {
            let (Some(cfg), Some(tf)) = (w.get::<UiCanvas>(canvas), w.get::<Transform>(canvas))
            else {
                continue;
            };
            let arr = w.get::<UiArranjo>(canvas).unwrap_or(&vazio);
            // Com câmera de jogo, a caixa EFECTIVA (a que as âncoras leem); sem ela, a de referência onde está.
            let (r, h) = (f64::from(cfg.ref_w) / 2.0, f64::from(cfg.ref_h) / 2.0);
            let now = vista
                .and_then(|v| anchor_frame_of(*cfg, v))
                .map_or([-r, -h, r, h], |(b, _)| b);
            let m = Moldura {
                t: [f64::from(tf.translation.x), f64::from(tf.translation.y)],
                s: [f64::from(tf.scale.x), f64::from(tf.scale.y)],
            };
            if m.s[0].abs() < 1e-12 || m.s[1].abs() < 1e-12 {
                continue;
            }
            let tela = Caixa {
                x: now[0],
                y: now[1],
                w: now[2] - now[0],
                h: now[3] - now[1],
            };
            // Os irmãos que desenham e têm id, pela ordem dos filhos.
            let mut kids: Vec<(Entity, StableId, Caixa2)> = Vec::new();
            for &k in w.get::<Children>(canvas).map(|c| &c[..]).unwrap_or(&[]) {
                if let (Some(&id), Some(b)) = (w.get::<StableId>(k), caixa_de(k)) {
                    kids.push((k, id, b));
                }
            }
            let ids: Vec<StableId> = kids.iter().map(|k| k.1).collect();
            let lacos = &arr.lacos;
            let texto = |k: Entity| matches!(w.get::<VecShape>(k), Some(VecShape::Text(_)));
            let mut visto = Visto {
                canvas,
                escala: m.s,
                lacos: lacos.clone(),
                filhos: kids
                    .iter()
                    .map(|&(kid, id, caixa)| FilhoVisto {
                        kid,
                        id,
                        caixa,
                        texto: texto(kid),
                    })
                    .collect(),
            };
            if lacos.is_empty() {
                self.vivos.remove(&canvas);
                self.visto.push(visto);
                continue;
            }
            let indice = |id: StableId| ids.iter().position(|&x| x == id);
            let com_largura: Vec<bool> = (0..ids.len())
                .map(|i| lacos.iter().any(|(r, _)| matches!(r, Relacao::LarguraDoConteudo { a } if indice(*a) == Some(i))))
                .collect();
            let els: Vec<Elemento> = kids
                .iter()
                .zip(&com_largura)
                .map(|(k, &larg)| {
                    let caixa = m.local(k.2);
                    Elemento {
                        caixa,
                        conteudo: larg.then_some(caixa.w),
                    }
                })
                .collect();
            let vivo = match self.vivos.get_mut(&canvas) {
                Some(v) if v.lacos == *lacos && v.ids == ids => v,
                _ => {
                    let rel: Vec<_> = lacos
                        .iter()
                        .filter_map(|(r, f)| Some((r.mapeia(indice)?, *f)))
                        .collect();
                    let arranjo = Arranjo::new(tela, &els, &rel);
                    self.vivos.insert(
                        canvas,
                        Vivo {
                            lacos: lacos.clone(),
                            ids: ids.clone(),
                            arranjo,
                        },
                    );
                    self.vivos.get_mut(&canvas).expect("acabado de pôr")
                }
            };
            let caixas = vivo.arranjo.atualiza_elementos(tela, &els);
            for (((k, e), (c, &larg)), f) in kids
                .iter()
                .zip(&els)
                .zip(caixas.into_iter().zip(&com_largura))
                .zip(&mut visto.filhos)
            {
                if parada(e.caixa, c) {
                    continue;
                }
                let c = if larg { por_igual(e.caixa, c) } else { c };
                f.caixa = m.mundo(c);
                out.push(Movido {
                    kid: k.0,
                    de: k.2,
                    para: f.caixa,
                });
            }
            self.visto.push(visto);
        }
        out
    }

    /// O retrato do último quadro: cada canvas e os filhos como se desenham.
    #[must_use]
    pub fn visto(&self) -> &[Visto] {
        &self.visto
    }
}

/// Dentro de [`ph2d_hud_layout::PARADA`] nada andou (o solver devolve a caixa na grelha).
fn parada(a: Caixa, b: Caixa) -> bool {
    let d = ph2d_hud_layout::PARADA;
    [(a.x, b.x), (a.y, b.y), (a.w, b.w), (a.h, b.h)]
        .iter()
        .all(|(p, q)| (p - q).abs() <= d)
}

/// O rótulo encolhido escala POR IGUAL: a altura acompanha a largura, centrada na faixa que o solver deu.
fn por_igual(de: Caixa, para: Caixa) -> Caixa {
    if de.w <= 0.0 || para.w >= de.w {
        return para;
    }
    let h = de.h * para.w / de.w;
    Caixa {
        y: para.y + (para.h - h) / 2.0,
        h,
        ..para
    }
}

#[cfg(test)]
#[path = "hud_arranjo_tests.rs"]
mod tests;
