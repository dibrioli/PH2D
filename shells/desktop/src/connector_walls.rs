//! **O que uma linha ENXERGA** — de que ela desvia, e a que ela se prende.
//!
//! Módulo filho do [`crate::connector_live`] (teto de 600 LOC por arquivo da shell) e o lugar das
//! duas únicas perguntas que ligam o DOCUMENTO à rota: *o que barra a linha?* (as paredes) e *a
//! que uma ponta se prende?* (o alvo). Estar num arquivo só não é arrumação — as duas respostas
//! são a mesma regra vista de dois lados, e é essa regra que fecha o laço:
//!
//! > **A rota de um conector NÃO PODE depender da pose de um rótulo.**
//!
//! Se depender — por qualquer das duas arestas —, o sistema tem realimentação. O rótulo de um
//! conector mora **em cima da rota dele** (é onde ele nasce, no meio por comprimento de arco);
//! então a rota reage ao rótulo, o rótulo se re-centra na rota nova, a rota reage de novo. Foi o
//! que o Enio viu: *"linha e texto pulando sem parar"*.
//!
//! Um laço de realimentação não se amortece nem se clampa: **corta-se**. As duas arestas morrem
//! aqui — [`is_annotation`] (o rótulo não é parede) e [`anchor_target`] (o rótulo não é alvo).

use ph2d_ecs::{Entity, SimWorld, VecConnector, VecLabel, VecShape};
use ph2d_vec_connect::Aabb;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms};

use crate::vec_entities::VecEntityMap;

/// Quantos vínculos de rótulo seguir ao resolver um alvo. Um rótulo de um rótulo é absurdo, mas
/// o `path_at` do duplo-clique pega o que está no TOPO — e o topo pode ser outro rótulo. O teto
/// existe para a resolução ser **total**: um ciclo de vínculos não pode travar o frame.
const MAX_LABEL_HOPS: usize = 4;

/// **É anotação?** — um conector, ou TEXTO (rotulado ou não). Nada disto é estrutura do diagrama:
/// não barra uma linha, e não serve de âncora para uma ponta.
///
/// # Por que `VecShape::Text`, e não só `VecLabel`
///
/// Porque **o vínculo é pendurado por um passe que roda DEPOIS de quem pergunta.** Um rótulo nasce
/// vazio: sem geometria não há path, sem path não há entidade, e o `label_live::upkeep_pending` só
/// consegue pendurar o `VecLabel` quando a 1ª letra materializa o objeto — e ele roda **depois** do
/// `connector_live::recook`, que é quem monta as paredes.
///
/// Perguntar pelo `VecLabel` responde **não** exatamente no frame em que o rótulo aparece. A linha
/// desvia da própria legenda; o passe do rótulo, logo abaixo **no mesmo frame**, arrasta o texto
/// para cima da rota desviada; no frame seguinte a isenção finalmente vale, a rota volta a ser
/// reta e o texto volta com ela. **Os dois pulam.** E basta um frame em que o vínculo não esteja
/// lá — o texto que o usuário criou com a ferramenta T e largou sobre a linha nunca tem vínculo
/// nenhum — para o laço voltar a existir.
///
/// É a MESMA armadilha que o `a_fresh_connector_never_enters_the_xform_map` já documenta para o
/// `settle_origins`: *só se pula o que se ENXERGA*. Uma isenção que depende de um componente
/// pendurado mais tarde no mesmo frame não é uma isenção — é uma corrida.
///
/// `VecShape::Text` não tem esse defeito: o `vec_text::upsert_text_shape` o pendura **no começo do
/// frame**, antes de qualquer leitor. E é a propriedade CERTA, não um proxy: *texto é anotação, não
/// estrutura* — um conector não desvia de uma legenda, tenha ela dono ou não. O `VecLabel` fica
/// como cinto (um rótulo convertido em curvas perde o `VecShape` e continua sendo um rótulo).
#[must_use]
pub(crate) fn is_annotation(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> bool {
    map.get(&id).is_some_and(|&bits| {
        let e = Entity::from_bits(bits);
        sim.world().get::<VecConnector>(e).is_some()
            || sim.world().get::<VecLabel>(e).is_some()
            || matches!(sim.world().get::<VecShape>(e), Some(VecShape::Text(_)))
    })
}

/// O hospedeiro de `id`, se `id` for um rótulo.
fn host_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecPathId> {
    let &bits = map.get(&id)?;
    Some(sim.world().get::<VecLabel>(Entity::from_bits(bits))?.host)
}

/// **O objeto a que uma ponta REALMENTE se prende.** `None` ⇒ não há alvo legítimo, e o chamador
/// trata a ponta como órfã (ela CONGELA onde está — o mesmo caminho de um alvo apagado).
///
/// Um rótulo nunca é o alvo: a linha se prende ao que ele **rotula**. E isso não é preciosismo —
/// é a segunda aresta do laço, e a que não perdoa.
///
/// # Como uma ponta vai parar num rótulo
///
/// O gesto que cria o vínculo (`connector_gesture::shape_under_cursor`) pega a forma do TOPO sob o
/// cursor e filtra os **conectores** — mas não os rótulos. E um rótulo nasce **centrado no
/// hospedeiro**: ele está, literalmente, por cima da caixa em que o usuário está mirando. Mirar no
/// meio de uma caixa rotulada e prender a linha no TEXTO dela é o caminho comum, não o exótico.
///
/// # E aí o laço não oscila — ele FOGE
///
/// Se o texto em que a ponta se prendeu for o rótulo do **próprio** conector, a rota passa a
/// depender da bbox do rótulo, e o rótulo se põe no meio da rota: a ponta corre para o meio, o que
/// encurta a linha, o que traz o meio para mais perto, … Medido no gate: o comprimento **cai pela
/// metade a cada frame**, para sempre. É "a linha e o texto pulando sem parar", na versão que nunca
/// assenta.
///
/// Resolver o rótulo para o hospedeiro mata os dois casos de uma vez: a ponta se prende à CAIXA
/// (que é o que o usuário mirou) e a rota deixa de depender de qualquer pose de rótulo. Um alvo que
/// continue sendo anotação depois de resolvido (um conector, um texto solto) **não é alvo**.
#[must_use]
pub(crate) fn anchor_target(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    target: VecPathId,
) -> Option<VecPathId> {
    let mut at = target;
    for _ in 0..MAX_LABEL_HOPS {
        match host_of(sim, map, at) {
            Some(host) if host != at => at = host,
            _ => break,
        }
    }
    // O alvo resolvido tem de EXISTIR e ser uma forma. Um conector (ligar linha em linha é
    // proibido pelo gesto — e realimentaria a rota) ou um texto solto não ancoram nada.
    let alive = scene.paths().iter().any(|p| p.id == at);
    (alive && !is_annotation(sim, map, at)).then_some(at)
}

/// **Toda forma que pode barrar uma linha**: a caixa de MUNDO de cada path com contorno fechado
/// que não seja [`anotação`](is_annotation).
///
/// Os filtros são a definição de "parede". Um **conector** não é obstáculo — se fosse, o primeiro
/// cruzamento entre duas linhas empurraria todas as outras, e o diagrama desmancharia sozinho a
/// cada traço novo. **Texto** não é obstáculo (ver [`is_annotation`]). E um **contorno aberto** (um
/// traço da caneta, uma aresta interna de um cubo) não tem interior: não há o que atravessar. É o
/// mesmo critério do `boundary_hit`, e não por acaso — a borda em que a linha encosta e a parede de
/// que ela desvia têm de ser a MESMA borda.
///
/// Calculado uma vez por frame, não uma vez por conector.
#[must_use]
pub(crate) fn shape_boxes(
    sim: &SimWorld,
    scene: &VecScene,
    xforms: &VecXforms,
    map: &VecEntityMap,
) -> Vec<Aabb> {
    let has_interior = |p: &VecPath| {
        (0..p.contour_count()).any(|c| matches!(p.contour(c), Some((v, true)) if v.len() >= 2))
    };
    scene
        .paths()
        .iter()
        .filter(|p| has_interior(p) && !is_annotation(sim, map, p.id))
        .filter_map(|p| {
            let (lo, hi) = scene.path_world_curve_bbox(xforms, p.id)?;
            Some(Aabb::new(lo, hi))
        })
        .collect()
}

/// **Os obstáculos que ESTA rota precisa enxergar.**
///
/// Passar o documento inteiro seria correto e caro: o grafo de visibilidade tem `(2n+3)²` nós,
/// então cada forma do desenho encareceria TODA rota — inclusive a que liga duas caixas vizinhas no
/// canto oposto da tela. A poda por região devolve o custo ao tamanho do problema: numa rota curta
/// sobram duas ou três caixas, e o grafo volta a ter dezenas de nós.
///
/// # A região CRESCE — e é por isso que isto é um ponto fixo, não um filtro
///
/// O filtro ingênuo (pegue quem cruza o corredor entre as duas pontas) tem um furo: uma forma que
/// atravessa a borda do corredor obriga a linha a **contorná-la**, e o contorno vai até a borda
/// OPOSTA dessa forma — que está fora do corredor original, possivelmente colada em OUTRA forma,
/// que o filtro descartou. A linha atravessaria essa segunda forma, e o desvio pareceria
/// simplesmente quebrado.
///
/// Então a seleção é iterativa: quem cruza a região entra, a região engole a caixa de quem entrou,
/// repete. Termina sempre (o conjunto só cresce, e é finito) e converge em uma ou duas passadas num
/// diagrama de verdade. Num diagrama denso ela pega tudo — que é a resposta **certa**, e o preço de
/// estar certo.
#[must_use]
pub(crate) fn obstacles_in_play(shapes: &[Aabb], a: Aabb, b: Aabb, pad: f64) -> Vec<Aabb> {
    let mut roi = a.union(b).inflate(pad);
    let mut taken = vec![false; shapes.len()];
    loop {
        let mut grew = false;
        for (i, s) in shapes.iter().enumerate() {
            if !taken[i] && roi.overlaps(*s) {
                taken[i] = true;
                roi = roi.union(s.inflate(pad));
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    shapes
        .iter()
        .zip(&taken)
        .filter_map(|(s, &t)| t.then_some(*s))
        .collect()
}

#[cfg(test)]
#[path = "connector_walls_tests.rs"]
mod tests;
