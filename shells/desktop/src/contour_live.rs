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
    /// A ÚLTIMA geometria não-vazia de cada anel — a guarda de continuidade do arrasto.
    ///
    /// ⚠️ **Existe por causa de um defeito do motor de offset, e o número é MEDIDO** (report do
    /// Enio, *"o efeito não é contínuo, dá saltos como se piscasse"*): o `linesweeper` 0.3.0
    /// entra em pânico numa fração grande das distâncias — 47 de 400 num hexágono com quina
    /// Round, **278 de 400** numa estrela com Bevel — e o `offset_path` converte a queda em
    /// VAZIO (senão o app morria). Arrastar o slider varre distâncias, então cada anel some e
    /// volta: era exatamente o piscar.
    ///
    /// Quatro curas foram MEDIDAS E REFUTADAS antes desta (detalhe no handoff): descartar
    /// segmentos de comprimento zero antes do sweep (**zero efeito** — e a primeira medição que
    /// disse o contrário era artefato da minha própria sonda) · empurrar a distância por 1e-4
    /// (recupera ~50%, **12%** no pior caso) · offset ITERADO em passos pequenos (48→18 falhas
    /// mas **1311 ms/frame**, 400× o custo) · fundir os sweeps num só (falhas idênticas).
    ///
    /// Então isto **não é a cura, é a continuidade**: onde o sweep não consegue responder, o anel
    /// fica onde estava em vez de desaparecer. O artista vê um anel que atrasa, não um que
    /// pisca. A cura de verdade é do motor de offset e precisa de wave própria com ADR.
    last_good: Vec<Vec<VecPath>>,
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

            let memo = self.ensure(path.id, &world, &spec);
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

    /// O memo de `id`, com os anéis que faltam já cozidos. **Porta única do cozimento** — o
    /// `recook` (por frame) e o [`ContourLive::expand`] (no clique) passam por aqui, senão o
    /// Expand entregaria anéis cozidos por um segundo caminho e as formas SALTARIAM no clique
    /// que promete materializar o que está na tela.
    fn ensure(&mut self, id: VecPathId, world: &VecPath, spec: &VecContour) -> &Memo {
        if !self.memo.get(&id).is_some_and(|m| m.matches(world, spec)) {
            // ⚠️ Invalidar NÃO joga fora o `last_good`: é justamente durante o arrasto — quando
            // toda mudança de `d` invalida — que a continuidade importa. O que morre é a
            // geometria FRESCA; a última boa de cada anel atravessa para o cozimento seguinte.
            let last_good = self.memo.remove(&id).map_or_else(Vec::new, |m| m.last_good);
            self.memo.insert(
                id,
                Memo {
                    world: world.clone(),
                    d: spec.d,
                    join: spec.join,
                    side: spec.side,
                    accel: spec.accel,
                    rings: Vec::new(),
                    last_good,
                },
            );
        }
        let memo = self
            .memo
            .get_mut(&id)
            .expect("o memo acabou de ser inserido se faltava");
        // Só coze o que FALTA — o prefixo sobrevive a mexer na contagem (§ do módulo).
        while u16::try_from(memo.rings.len()).unwrap_or(u16::MAX) < spec.steps {
            let k = usize::from(u16::try_from(memo.rings.len()).unwrap_or(u16::MAX));
            let fresh = ph2d_vec_boolean::offset_path(
                world,
                spec.ring_distance(u16::try_from(k + 1).expect("k < steps <= MAX_CONTOUR_STEPS")),
                crate::vec_expand::join_of_code(spec.join),
                crate::vec_expand::side_of_code(spec.side),
            );
            if fresh.is_empty() {
                // O sweep não respondeu (§ do `last_good`): o anel FICA onde estava. Se nunca
                // houve geometria boa para ele, aí sim não há o que desenhar — um anel que
                // nunca existiu não pode "ficar".
                let held = memo.last_good.get(k).cloned().unwrap_or_default();
                memo.rings.push(held);
            } else {
                if memo.last_good.len() <= k {
                    memo.last_good.resize(k + 1, Vec::new());
                }
                memo.last_good[k].clone_from(&fresh);
                memo.rings.push(fresh);
            }
        }
        memo
    }

    /// **Expand Contour** — materializa os anéis em formas REAIS na cena e descarta o efeito.
    ///
    /// É o *Break Contour Apart* do Corel, e o irmão exato do Expand do Blend (ADR-0128 Fase D):
    /// o que estava na tela passa a ser geometria editável ponto a ponto. **A FONTE fica** (≠
    /// booleana, que consome os operandos) — o contour era desenho por cima dela, e apagá-la seria
    /// destruir a arte para entregar uma cópia.
    ///
    /// Os anéis saem da MESMA [`painted_rings`] que o `recook` desenha, sobre a MESMA [`ensure`]
    /// que ele coze: é o ponto todo, e é o que garante que nada mude de forma ou de cor no clique.
    /// Já vêm assados em MUNDO, então o path nasce sem pose (a entidade nasce na identidade no
    /// `sync`) e a geometria está no lugar certo.
    ///
    /// Devolve **uma sequência de z (fundo → topo) por contour expandido**, para o chamador
    /// enfileirar em `vec_restack`: quem manda no z é a ÁRVORE (ADR-0110), e as entidades dos
    /// anéis só nascem no `sync` seguinte.
    pub(crate) fn expand(
        &mut self,
        sim: &mut SimWorld,
        scene: &mut VecScene,
        map: &VecEntityMap,
        xforms: &VecXforms,
        ids: &[VecPathId],
    ) -> Vec<Vec<VecPathId>> {
        let mut runs: Vec<Vec<VecPathId>> = Vec::new();
        for &id in ids {
            let Some(spec) = spec_of(sim, map, id) else {
                continue;
            };
            let Some(src) = scene.paths().iter().find(|p| p.id == id) else {
                continue;
            };
            let mut world = src.cooked().into_owned();
            bake_xform(&mut world, &xform_of(xforms, id));
            let rings = painted_rings(&world, self.ensure(id, &world, &spec), &spec);
            if rings.is_empty() {
                continue; // contour que aniquilou a forma: não há o que materializar
            }
            // A posição é buscada agora: uma inserção anterior já moveu os índices.
            let at = scene
                .paths()
                .iter()
                .position(|p| p.id == id)
                .map_or(0, |z| z + 1);
            let new: Vec<VecPathId> = rings
                .into_iter()
                .enumerate()
                .map(|(k, p)| scene.insert_path(at + k, p))
                .collect();
            // O efeito foi consumido: a fonte deixa de carregar o componente, senão o frame
            // seguinte re-cozeria anéis por cima dos que acabaram de virar formas.
            if let Some(&bits) = map.get(&id)
                && let Ok(mut em) = sim.world_mut().get_entity_mut(Entity::from_bits(bits))
            {
                em.remove::<VecContour>();
            }
            self.memo.remove(&id);
            runs.push(stacked(new, id, spec.d));
        }
        runs
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira, e os
    /// `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.live.clear();
        self.memo.clear();
    }
}

/// Monta a lista final: os anéis pintados pela rampa + a fonte, **do maior para o menor**.
fn assemble(world: &VecPath, memo: &Memo, spec: &VecContour) -> Vec<VecPath> {
    stacked(painted_rings(world, memo, spec), world.clone(), spec.d)
}

/// **A ordem de desenho, fundo → topo** — os anéis (mais distante primeiro) e depois a FONTE.
///
/// Genérica sobre o que está a ser ordenado porque a regra tem **dois** consumidores que ordenam
/// coisas diferentes: o cozimento empilha `VecPath` para desenhar, e o [`expand`] empilha
/// `VecPathId` para o `vec_zorder::restack`. Escrita duas vezes, ela divergiria — e a divergência
/// seria *as formas saltarem de z no clique que promete entregar o que estava na tela*.
fn stacked<T>(mut rings: Vec<T>, source: T, d: f64) -> Vec<T> {
    rings.push(source);
    // `d < 0` faz os anéis ENCOLHEREM, então "maior primeiro" é a ordem inversa desta.
    if d < 0.0 {
        rings.reverse();
    }
    rings
}

/// Os anéis já pintados pela rampa, do mais DISTANTE ao mais próximo.
///
/// A fonte não entra aqui: ela é a forma do artista, não um passo da rampa. O anel `k` leva a cor
/// interpolada em **Oklab** entre a da fonte e o alvo, na fração `ramp_t(k)`.
///
/// ⚠️ `ramp_t` **não** é a função que decidiu a distância daquele anel (`ring_distance`) — as duas
/// divergem de propósito, e o porquê está no `accel` do [`VecContour`]. O que as mantém coerentes
/// é serem MONÓTONAS na mesma direção: o anel mais distante é sempre o mais próximo do alvo.
///
/// **Porta única do desenho de um anel**, e é por isso que o [`expand`] a chama em vez de recozer:
/// uma 2ª porta faria os anéis mudarem de cor no clique que promete materializá-los como estão.
fn painted_rings(world: &VecPath, memo: &Memo, spec: &VecContour) -> Vec<VecPath> {
    let from = source_rgba(world);
    let mut out: Vec<VecPath> = Vec::new();
    let n = spec
        .steps
        .min(u16::try_from(memo.rings.len()).unwrap_or(u16::MAX));
    for k in (1..=n).rev() {
        let paint = Paint::solid(ramp(from, spec.to, spec.ramp_t(k)));
        for ring in &memo.rings[usize::from(k - 1)] {
            let mut p = ring.clone();
            p.fill = Some(paint.clone());
            // O traço do anel é descartado: um contour é um empilhamento de PREENCHIMENTOS, e
            // herdar o traço da fonte desenharia N contornos por cima da rampa.
            p.stroke = None;
            out.push(p);
        }
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

/// O contour de `id`, se houver. Porta única: o cozimento, o painel e o Expand perguntam AQUI.
#[must_use]
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecContour> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecContour>(Entity::from_bits(bits))
        .copied()
}

/// Os três comandos da seção — o que o clique num botão pede à cena.
///
/// `Add` e `Remove` são as duas portas do MODELO (o componente passa a existir, ou deixa de);
/// `Expand` é a única que produz GEOMETRIA. Ficam num enum e não em três `bool` porque são
/// mutuamente exclusivos por construção: um clique é um clique.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ContourCmd {
    Add,
    Remove,
    Expand,
}

/// O código de quina que este id do painel endereça, se for um dos três chips de **Corner**.
///
/// Os códigos são os MESMOS do Expand (`0` Miter · `1` Round · `2` Bevel), resolvidos adiante pela
/// MESMA porta (`vec_expand::join_of_code`) — um segundo vocabulário para a mesma pergunta é como
/// o chip do artista passa a significar outra quina.
#[must_use]
pub(crate) fn join_code_of_id(id: ph2d_editor::ids::NodeId) -> Option<u8> {
    match id {
        i if i == ph2d_editor::ids::VECTOR_CONTOUR_JOIN_MITER => Some(0),
        i if i == ph2d_editor::ids::VECTOR_CONTOUR_JOIN_ROUND => Some(1),
        i if i == ph2d_editor::ids::VECTOR_CONTOUR_JOIN_BEVEL => Some(2),
        _ => None,
    }
}

/// O código de lado que este id endereça, se for um dos três chips de **Side** (`0` Outer ·
/// `1` Inner · `2` Both). Irmão do [`join_code_of_id`], mesmos códigos do Expand.
#[must_use]
pub(crate) fn side_code_of_id(id: ph2d_editor::ids::NodeId) -> Option<u8> {
    match id {
        i if i == ph2d_editor::ids::VECTOR_CONTOUR_SIDE_OUTER => Some(0),
        i if i == ph2d_editor::ids::VECTOR_CONTOUR_SIDE_INNER => Some(1),
        i if i == ph2d_editor::ids::VECTOR_CONTOUR_SIDE_BOTH => Some(2),
        _ => None,
    }
}

/// A distância por passo com que um contour NASCE, em fração do tamanho da forma.
///
/// ⚠️ **Não é zero, e a razão é o primeiro frame.** O slider de Offset é bipolar e o neutro dele
/// cai no centro do curso; armar no neutro daria N anéis exatamente sobre a forma, ou seja um
/// clique em *Add Contour* que **não muda um pixel** — o artista concluiria que o botão está
/// quebrado. 8% por passo × os 4 passos do default cresce a forma em ~32%: vê-se de imediato e
/// ainda cabe na tela.
const DEFAULT_D_FRAC: f64 = 0.08;

/// **Arma** um contour em cada caminho de `ids` que ainda não tenha um.
///
/// `scale` é o tamanho da seleção em unidades de MUNDO (a mesma `vec_expand::offset_scale` que o
/// Offset usa): o painel fala em FRAÇÃO — porque o mapa do store é estático e um rótulo em
/// unidades de mundo mentiria a cada troca de seleção — e o componente guarda MUNDO, porque tem
/// de sobreviver a essa troca. A conversão acontece aqui, na fronteira, e só aqui.
///
/// Uma forma que **já tem** contour é deixada como está: *Add* cria, não redefine — clicar duas
/// vezes não pode apagar os valores que o artista acabou de afinar.
///
/// Devolve quantas entidades ganharam o componente.
pub(crate) fn arm(sim: &mut SimWorld, map: &VecEntityMap, ids: &[VecPathId], scale: f64) -> usize {
    let mut n = 0;
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        if sim.world().get::<VecContour>(e).is_some() {
            continue;
        }
        let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
            continue;
        };
        em.insert(VecContour {
            d: DEFAULT_D_FRAC * scale,
            ..VecContour::default()
        });
        n += 1;
    }
    n
}

/// **Tira** o contour de `ids` — a forma volta a desenhar-se sozinha e nada é materializado.
pub(crate) fn remove(sim: &mut SimWorld, map: &VecEntityMap, ids: &[VecPathId]) -> usize {
    let mut n = 0;
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        if sim.world().get::<VecContour>(e).is_none() {
            continue;
        }
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.remove::<VecContour>();
            n += 1;
        }
    }
    n
}

/// Edita os contours VIVOS de `ids` — os sliders, os chips de Corner/Side e a cor-alvo.
///
/// **Não arma nada:** um caminho sem contour continua sem. É o que faz mexer num controle sem ter
/// clicado *Add Contour* não inventar um efeito de lugar nenhum — a mesma lei do `retune` do
/// Offset, e a razão pela qual a seção mostra o botão em vez dos controles enquanto não há um.
pub(crate) fn edit(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    f: impl Fn(&mut VecContour),
) {
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let Some(mut cur) = sim.world().get::<VecContour>(e).copied() else {
            continue;
        };
        f(&mut cur);
        cur.steps = cur.steps.clamp(1, ph2d_ecs::MAX_CONTOUR_STEPS);
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.insert(cur);
        }
    }
}

/// O contour que o painel deve mostrar: o do PRIMEIRO caminho selecionado que tenha um.
#[must_use]
pub(crate) fn current(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
) -> Option<(VecPathId, VecContour)> {
    selection
        .iter()
        .find_map(|&id| spec_of(sim, map, id).map(|s| (id, s)))
}

#[cfg(test)]
#[path = "contour_live_tests.rs"]
mod tests;
