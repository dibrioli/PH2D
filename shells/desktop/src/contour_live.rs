//! **O Contour VIVO na shell** — o cozimento do [`ph2d_ecs::VecContour`] (pesquisa `20_*` item #9).
//!
//! Generalização direta do [`crate::offset_live`]: onde ele produz UM caminho offsetado, este
//! produz **N anéis concêntricos com uma rampa de cor**. A forma fonte nunca é tocada — é ela que
//! o modo Node edita —, e os anéis são desenho derivado, re-cozidos aqui e desenhados por
//! [`ph2d_vec_render::dispatch`] no z dela.
//!
//! # A ordem de desenho é UMA regra: do maior para o menor
//!
//! Com `d > 0` os anéis são MAIORES que a fonte e têm de ficar atrás dela; com `d < 0` são
//! MENORES e têm de ficar à frente, senão a fonte os tapa. As duas frases são a mesma: **desenhe
//! do maior para o menor**, e a fonte entra na posição que lhe cabe nessa ordem. Um `if` por sinal
//! seria duas respostas para uma pergunta, e a segunda envelheceria quando aparecesse o modo
//! `Both`.
//!
//! # O memo é POR ANEL, e é o que torna o slider de passos barato
//!
//! O anel `k` está em `d · k^accel` — uma função só de `k` e dos parâmetros, **nunca da contagem**
//! (vide o § do `d` no [`ph2d_ecs::VecContour`]). Então acrescentar um passo não invalida os
//! anteriores: o memo guarda o `Vec` de anéis já cozidos e só coze o que falta. Medido
//! (`probe_contour_cost`): um anel custa 0,4–1,4 ms conforme forma e quina, então re-cozer 16 por
//! frame arrastando os passos seria 5–11 ms — o gesto mais comum do efeito, e o mais caro, se o
//! memo não existisse.
//!
//! Quem **invalida** o memo inteiro é o que move todos os anéis de uma vez: a geometria de mundo,
//! o `d`, a quina, o lado e a aceleração. É a mesma chave do `offset_live` — o que ENTROU —, não
//! um contador de versão que alguém esqueceria de bumpar.

use std::collections::BTreeMap;

use ph2d_color::{LinearRgba, OklabColor, SrgbRgba};
use ph2d_ecs::{Entity, SimWorld, VecContour};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, VecScene, VecXforms, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// O que ENTROU (a geometria de mundo + tudo o que move TODOS os anéis) e os anéis que saíram.
///
/// `steps` fica **fora** da chave de propósito: ele não move anel nenhum, só diz quantos existem —
/// e é isso que deixa o memo reusar o prefixo quando o artista arrasta a contagem.
struct Memo {
    world: VecPath,
    d: f64,
    join: u8,
    side: u8,
    accel: f32,
    /// Os anéis já cozidos, do mais próximo (k=1) ao mais distante. Cada entrada é o que o
    /// `offset_path` devolveu para aquele `k` — pode ser mais de um caminho (um donut offsetado
    /// para dentro se parte em vários) ou nenhum (o anel morreu).
    rings: Vec<Vec<VecPath>>,
}

impl Memo {
    /// `true` se este memo descreve as MESMAS entradas que movem todos os anéis.
    fn matches(&self, world: &VecPath, spec: &VecContour) -> bool {
        self.d == spec.d
            && self.join == spec.join
            && self.side == spec.side
            && self.accel == spec.accel
            && &self.world == world
    }
}

/// O cozimento vivo de todos os contours da cena.
#[derive(Default)]
pub(crate) struct ContourLive {
    live: LiveGeometry,
    memo: BTreeMap<VecPathId, Memo>,
}

impl ContourLive {
    /// A geometria derivada deste frame — o que o `dispatch` desenha no lugar da fonte.
    pub(crate) fn live(&self) -> &LiveGeometry {
        &self.live
    }

    /// Re-coza todos os contours. Chamado uma vez por frame, DEPOIS do `sync` (senão uma forma
    /// recém-criada ainda não teria entidade e o componente não seria encontrado).
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
    ) {
        self.live.clear();
        for path in scene.paths() {
            let Some(spec) = spec_of(sim, map, path.id) else {
                continue;
            };
            if spec.steps == 0 {
                continue;
            }
            // A fonte entra em MUNDO (a pose assada), como no `offset_live`: o offset é uma
            // distância de mundo, então cozer em local daria anéis de espessura errada sob escala.
            let mut world = path.cooked().into_owned();
            bake_xform(&mut world, &xform_of(xforms, path.id));

            let fresh = self
                .memo
                .get(&path.id)
                .is_some_and(|m| m.matches(&world, &spec));
            if !fresh {
                self.memo.insert(
                    path.id,
                    Memo {
                        world: world.clone(),
                        d: spec.d,
                        join: spec.join,
                        side: spec.side,
                        accel: spec.accel,
                        rings: Vec::new(),
                    },
                );
            }
            let Some(memo) = self.memo.get_mut(&path.id) else {
                continue;
            };
            // Só coze o que FALTA — o prefixo sobrevive a mexer na contagem (§ do módulo).
            while u16::try_from(memo.rings.len()).unwrap_or(u16::MAX) < spec.steps {
                let k = u16::try_from(memo.rings.len()).unwrap_or(u16::MAX) + 1;
                let dist = spec.ring_distance(k);
                memo.rings.push(ph2d_vec_boolean::offset_path(
                    &world,
                    dist,
                    crate::vec_expand::join_of_code(spec.join),
                    crate::vec_expand::side_of_code(spec.side),
                ));
            }

            let out = assemble(&world, memo, &spec);
            // ⚠️ Inserido MESMO com um anel vazio: vazio é a ANIQUILAÇÃO (o offset comeu a forma),
            // e desenhar o que sobrou é a resposta certa. A entrada AUSENTE significaria "desenhe a
            // fonte", e os anéis mortos ressuscitariam como a forma inteira.
            self.live.insert(path.id, out);
        }
        // O memo não pode sobreviver ao componente: uma forma que perdeu o contour (Apply, undo,
        // detach) tem de voltar a desenhar-se, e um memo órfão a manteria coberta de anéis.
        self.memo
            .retain(|id, _| spec_of(sim, map, *id).is_some_and(|s| s.steps > 0));
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira, e os
    /// `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.live.clear();
        self.memo.clear();
    }
}

/// Monta a lista final: os anéis pintados pela rampa + a fonte, **do maior para o menor**.
///
/// A fonte entra com a tinta DELA (é a forma do artista, não um passo da rampa) e o anel `k` com a
/// cor interpolada em **Oklab** entre a fonte e o alvo, na fração `ramp_t(k)`.
///
/// ⚠️ `ramp_t` **não** é a função que decidiu a distância daquele anel (`ring_distance`) — as duas
/// divergem de propósito, e o porquê está no `accel` do [`VecContour`]. O que as mantém coerentes
/// é serem MONÓTONAS na mesma direção: o anel mais distante é sempre o mais próximo do alvo.
fn assemble(world: &VecPath, memo: &Memo, spec: &VecContour) -> Vec<VecPath> {
    let from = source_rgba(world);
    let mut out: Vec<VecPath> = Vec::new();
    let n = spec
        .steps
        .min(u16::try_from(memo.rings.len()).unwrap_or(u16::MAX));
    // Do mais distante ao mais próximo; a fonte por último = por cima (a regra do § do módulo).
    for k in (1..=n).rev() {
        let t = spec.ramp_t(k);
        let paint = Paint::solid(ramp(from, spec.to, t));
        for ring in &memo.rings[usize::from(k - 1)] {
            let mut p = ring.clone();
            p.fill = Some(paint.clone());
            // O traço do anel é descartado: um contour é um empilhamento de PREENCHIMENTOS, e
            // herdar o traço da fonte desenharia N contornos por cima da rampa.
            p.stroke = None;
            out.push(p);
        }
    }
    out.push(world.clone());
    // `d < 0` faz os anéis ENCOLHEREM, então a ordem "maior primeiro" é a inversa desta.
    if spec.d < 0.0 {
        out.reverse();
    }
    out
}

/// A cor de onde a rampa parte: a do preenchimento da fonte, ou branco opaco se ela não tem um
/// (um contour sobre forma sem fill ainda tem de ter um extremo de onde sair).
fn source_rgba(world: &VecPath) -> [u8; 4] {
    match &world.fill {
        Some(Paint::Solid(c)) => [c.r, c.g, c.b, c.a],
        _ => [255, 255, 255, 255],
    }
}

/// Interpola `from → to` em **Oklab** e devolve sRGB.
///
/// Oklab, e não sRGB cru, porque é o meio da rampa que denuncia: dois tons saturados interpolados
/// em sRGB passam por um cinza lamacento que ninguém autorou. O módulo já mostra ao artista um
/// picker OKLCH — a rampa tem de viver no mesmo espaço que o seletor.
///
/// ⚠️ O **alfa** viaja LINEAR e fora do Oklab: ele não é cor, é cobertura, e passá-lo pelo espaço
/// perceptual não quer dizer nada.
fn ramp(from: [u8; 4], to: [u8; 4], t: f64) -> Rgba8 {
    #[allow(clippy::cast_possible_truncation)]
    let tf = t.clamp(0.0, 1.0) as f32;
    let a = OklabColor::from_linear(SrgbRgba::new(from[0], from[1], from[2], from[3]).to_linear());
    let b = OklabColor::from_linear(SrgbRgba::new(to[0], to[1], to[2], to[3]).to_linear());
    let mixed: LinearRgba = a.lerp(b, tf).to_linear();
    let srgb = mixed.to_srgb();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let alpha = (f32::from(from[3]).mul_add(1.0 - tf, f32::from(to[3]) * tf) + 0.5) as u8;
    Rgba8::new(srgb.0[0], srgb.0[1], srgb.0[2], alpha)
}

/// O contour de `id`, se houver. Porta única: o cozimento, o painel e o Apply perguntam AQUI.
#[must_use]
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecContour> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecContour>(Entity::from_bits(bits))
        .copied()
}

#[cfg(test)]
#[path = "contour_live_tests.rs"]
mod tests;
