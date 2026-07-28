//! **A silhueta RESOLVIDA das formas com traço** — o que o campo de distância dos FX precisa
//! saber sobre uma forma que o `ph2d-vec-render` sozinho não sabe responder.
//!
//! # O bug que este módulo fecha
//!
//! O campo exato do plano 24 é semeado pela GEOMETRIA (o pé da fronteira sai de um laço sobre os
//! segmentos da silhueta) e cai no JFA sobre o raster quando não há geometria. O caminho do raster
//! semeia em texels DISCRETOS, então a direção que ele devolve salta na fronteira de célula de
//! Voronoi e a distância escadeia — renderizado, o bevel sai com um **pente** de hachuras
//! diagonais finas. Foi o que o Enio fotografou.
//!
//! E toda forma com TRAÇO caía nesse caminho, porque `silhouette_segments` recusa (com razão) o
//! caminho autorado: numa forma traçada, a curva que o artista desenhou passa pelo MEIO da faixa
//! de tinta, e semeá-la poria a fronteira dentro da forma. A silhueta que se vê é
//! `preenchimento ∪ contorno-do-traço` — e só a booleana sabe uni-los.
//!
//! # Por que a união mora AQUI e não lá
//!
//! O `ph2d-vec-render` **não depende** do `ph2d-vec-boolean` (o `Cargo.toml` dele avisa do skew de
//! versão do kurbo), e essa cerca é boa: o desenhista não precisa saber resolver interseção. Então
//! a shell — que já conhece as duas — pergunta à booleana e entrega o resultado pronto, do mesmo
//! jeito que entrega a geometria derivada do offset/pattern/contour.
//!
//! # O memo é MEDIDO, e o que ele guarda é camera-INDEPENDENTE
//!
//! `silhouette_paths` custa, em release, **0,19–0,31 ms** numa estrela de 5 pontas (10 âncoras),
//! **0,46–0,67 ms** numa de 12 (24 âncoras) e **1,67–2,54 ms** numa de 40 (80 âncoras) — linear na
//! contagem de âncoras, ~0,02 ms cada. O re-cook dos FX roda a cada frame de um arrasto de zoom
//! (a pilha resolvida em pixels muda), então sem memo um zoom sobre uma forma complexa pagaria
//! isso por frame.
//!
//! A chave é o que de facto determina a resposta — a geometria de MUNDO que entra — e não um
//! contador de versão que alguém esqueceria de bumpar. É o mesmo desenho do [`crate::offset_live`],
//! e por o mundo ser função da POSE (nunca da câmera) o memo acerta durante todo pan e zoom: a
//! união é paga por EDIÇÃO, não por frame.

use std::collections::BTreeMap;

use ph2d_ecs::SimWorld;
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, VecScene, VecXforms, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// Uma entrada do memo: o que ENTROU (a geometria de mundo desenhada) e o que SAIU.
struct Memo {
    input: Vec<VecPath>,
    out: Vec<VecPath>,
}

/// A silhueta resolvida de cada forma TRAÇADA que carrega um filtro, com memo por caminho.
#[derive(Default)]
pub(crate) struct FxSilhouette {
    memo: BTreeMap<VecPathId, Memo>,
    live: LiveGeometry,
    /// Quantas vezes a booleana foi de facto chamada desde o começo — o instrumento do gate de
    /// custo.
    ///
    /// ⚠️ **O `ph2d_vec_boolean::__sweep_calls` NÃO serve aqui**, e a primeira versão do gate errou
    /// por isso: ele conta entradas em `offset_path`, e a união passa por `outline_stroke` +
    /// `apply_many`, que não o tocam. O gate media zero contra zero e **não podia falhar pelo
    /// motivo que alegava**. Um contador próprio mede exatamente a frase que ele afirma: *quantas
    /// vezes o caminho CARO disparou* (o padrão do ADR-0120).
    cooks: u64,
}

impl FxSilhouette {
    /// As regiões resolvidas deste frame, em espaço de MUNDO. Vazio = nenhuma forma traçada com
    /// filtro na cena, e o campo se comporta exatamente como antes deste módulo existir.
    pub(crate) fn live(&self) -> &LiveGeometry {
        &self.live
    }

    /// Quantas uniões foram cozidas até agora. Cresce só num MISS de memo — é a diferença entre
    /// pagar por edição e pagar por frame.
    #[cfg(test)]
    pub(crate) fn cooks(&self) -> u64 {
        self.cooks
    }

    /// Re-resolve o que mudou. Chamado uma vez por frame, DEPOIS dos cozimentos de geometria
    /// derivada (o campo tem de descrever o que o `dispatch` de facto desenha) e ANTES do
    /// `fx_live::recook`, que a consome.
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &LiveGeometry,
    ) {
        self.live.clear();
        for path in scene.paths() {
            // Só paga onde alguém lê: a união serve ao campo de distância, e sem filtro não há
            // campo. Uma forma traçada sem FX não custa um sweep.
            if crate::fx_live::spec_of(sim, map, path.id).is_none() {
                continue;
            }
            // O que o `dispatch` DESENHA — a derivada quando há uma, a fonte assada em mundo
            // quando não há. É a mesma pergunta que o `silhouette_segments` faz, e ela tem de ter
            // a mesma resposta nos dois lados ou o campo descreve outra forma.
            let input: Vec<VecPath> = match live.get(&path.id) {
                Some(items) => items.clone(),
                None => {
                    let mut world = path.cooked().into_owned();
                    bake_xform(&mut world, &xform_of(xforms, path.id));
                    vec![world]
                }
            };
            // Sem traço em nenhuma peça, o `silhouette_segments` já responde exato pela fonte —
            // resolver aqui seria uma SEGUNDA resposta à mesma pergunta.
            if input.iter().all(|p| p.stroke.is_none()) {
                continue;
            }
            let hit = self.memo.get(&path.id).is_some_and(|m| m.input == input);
            if !hit {
                self.cooks += 1;
                let out = input.iter().flat_map(regions_of).collect::<Vec<_>>();
                self.memo.insert(path.id, Memo { input, out });
            }
            if let Some(m) = self.memo.get(&path.id)
                && !m.out.is_empty()
            {
                self.live.insert(path.id, m.out.clone());
            }
        }
        // O memo não sobrevive ao que o gerou. ⚠️ **É HIGIENE, e a mutação que o remove sobrevive
        // de propósito:** uma entrada órfã nunca é LIDA (o `continue` do filtro/traço vem antes), e
        // se a forma voltasse a ser traçada a entrada velha erraria de qualquer jeito (o `input`
        // passa a incluir o traço). O que ele evita é o memo crescer sem fim numa sessão longa.
        self.memo.retain(|id, _| self.live.contains_key(id));
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira debaixo do memo,
    /// e os `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.memo.clear();
        self.live.clear();
    }
}

/// As regiões fechadas que UMA peça pinta, normalizadas ao que o consumidor exige: **sem traço**
/// (a faixa de tinta já foi absorvida) e **com preenchimento** (é uma região, não uma linha).
///
/// ⚠️ **A normalização é CINTO, não conserto — e eu ia shipar a afirmação contrária.** A mutação que
/// a remove sobreviveu à suíte inteira, e a medição diz por quê: hoje a booleana já devolve a união
/// com `stroke: None` e `fill: Some`. O que a mantém aqui é o modo de falha do outro lado — o
/// `push_path` do `silhouette_segments` recusa peça com traço **em silêncio**, então uma união
/// entregue crua faria a forma voltar ao raster com todos os gates verdes. O fato de hoje está
/// PINADO num gate (`the_boolean_already_hands_back_regions_and_this_normalisation_is_a_belt`), que
/// é o que impede o cinto de virar prosa se a booleana mudar de opinião.
fn regions_of(p: &VecPath) -> Vec<VecPath> {
    ph2d_vec_boolean::silhouette_paths(p)
        .into_iter()
        .map(|mut r| {
            r.stroke = None;
            r.fill = Some(Paint::Solid(Rgba8::new(255, 255, 255, 255)));
            r
        })
        .collect()
}

#[cfg(test)]
#[path = "fx_silhouette_tests.rs"]
mod tests;
