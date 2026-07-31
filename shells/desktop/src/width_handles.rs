//! **O WIDTH TOOL** — as alças de largura na curva (plano 25 §5, ADR-0148).
//!
//! O W1c deu a representação (a lista de paradas) e o motor vivo; o W1d deu uma fonte automática
//! (o gesto do lápis). Este é o terceiro escritor do MESMO perfil: **a mão do artista, ponto a
//! ponto** — o *Width Tool* do Illustrator.
//!
//! # O gesto, e por que ele é um MODO
//!
//! Uma alça por parada, **fora da curva**, à distância que a fita de facto tem ali. Arrastar:
//! afastar da curva ENGROSSA, aproximar AFINA, e andar ao longo dela MOVE a parada. Clicar na
//! curva onde não há alça **acrescenta** uma parada ali; o botão direito sobre uma alça a
//! **apaga**.
//!
//! É um modo (o pill **Width**) e não uma alça escondida no Node pela razão que o Fillet/Chamfer
//! já pagou: no Node as alças competiriam com as âncoras — e uma parada de multiplicador pequeno
//! senta a milímetros da curva, ou seja em cima delas. O Illustrator faz o mesmo (Shift+W).
//!
//! # Nada aqui inventa uma segunda maneira de guardar a largura
//!
//! Toda escrita passa por [`crate::profile_live::arm`], a MESMA porta dos quatro sliders e do
//! lápis. O que muda é só quem calcula a lista — e é por isso que o preview, o Apply e as três
//! rotas de autoria não podem divergir: há um motor (`power_stroke_layers`) e um lugar onde o
//! perfil mora (o componente).
//!
//! # A primeira alça nasce do NEUTRO
//!
//! Numa forma sem perfil, o 1º clique cria a lista uniforme (duas paradas, `1.0` nas pontas) e
//! **só então** arrasta — o desenho não salta no toque. É a mesma lei do Fillet, que transforma um
//! ponto suave em quina antes de arredondá-lo: *a ferramenta faz, dentro do gesto, o passo que o
//! artista faria à mão*.

use ph2d_ecs::SimWorld;
use ph2d_vec_scene::{VecPathId, VecScene, WidthStop, WidthStops, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// A parada agarrada por um arrasto.
// Sem `Eq`: o `pos` é `f64`. `PartialEq` basta — ninguém usa isto como chave.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Grab {
    /// O caminho cuja largura está a ser editada.
    pub path: VecPathId,
    /// O índice da parada na lista.
    pub stop: usize,
    /// Esta parada NASCEU neste gesto (o press foi sobre a curva, não sobre uma alça) e o dedo
    /// **ainda não a moveu**. O 1º arrasto o derruba.
    ///
    /// ⚠️ **Existe por causa de um número medido, e é o que torna a inserção invisível.** O
    /// `smoothstep` liga paradas CONSECUTIVAS, então inserir uma re-parametriza os dois vãos que
    /// ela divide: o desvio máximo é **13,1% da faixa do perfil** — e é estrutural, o MESMO em
    /// todo perfil (é o máximo entre um smoothstep e dois meio-smoothsteps), não um acidente de
    /// dados. Trocar por interpolação LINEAR tornaria a inserção exata e poria um VINCO em cada
    /// parada, que é o que o `WidthProfile` recusa desde o 1º dia.
    ///
    /// A cura não é a interpolação, é o GESTO: com o Width Tool cria-se um ponto de largura
    /// **arrastando** a partir da curva (é o que o Illustrator faz), e um clique que não moveu
    /// nada não pediu nada — então ele é desfeito no release e o desenho não muda. Quem arrasta
    /// nunca vê os 13,1%, porque a espessura já está a mudar sob o dedo.
    pub created: bool,
    /// **Onde a parada estava quando o gesto começou**, em fração de arco.
    ///
    /// ⚠️ **Existe porque o índice sozinho MENTE numa forma virgem**, e o defeito era visível:
    /// a parada criada nasce com o multiplicador que o perfil já tem ali (para o desenho não
    /// saltar), então num perfil neutro a lista continua UNIFORME — e o `arm` remove um perfil
    /// uniforme (o neutro-é-ausência, a lei deste módulo). O `press` devolvia então um índice
    /// para uma lista que nunca foi guardada, e o `drag` seguinte relia o NEUTRO (duas paradas)
    /// e editava a de índice 1: **a ponta do traço**. MEDIDO: o 1º gesto do Width numa forma
    /// virgem levava `[(0, 1), (1, 1)]` a `[(0, 1), (0.241, 5)]` — o artista puxava no meio e o
    /// FIM do traço dele mudava de sítio, com a metade final a engrossar toda.
    pub pos: f64,
}

/// A meia-largura do traço de `id`, em unidades de MUNDO — a régua da alça.
///
/// ⚠️ **A largura NÃO sobe pelo afim**, e a alça tem de concordar com a fita: o `bake_xform`
/// transforma pontos e comprimentos de PATH (o raio de quina, o do gradiente) e deixa
/// `stroke.width` como está, então o `power_stroke` molda a fita na largura autorada mesmo sob
/// uma pose escalada. Uma alça que multiplicasse pela escala pousaria fora da tinta.
fn half_width(scene: &VecScene, id: VecPathId) -> Option<f64> {
    let w = scene.path(id)?.stroke?.width;
    (w > 0.0).then_some(w * 0.5)
}

/// O caminho de `id` em MUNDO, medido por arco — a mesma porta que o Pattern usa para as fichas
/// dele, então as duas famílias de alça não podem discordar sobre onde a curva está.
fn arc_of(
    scene: &VecScene,
    map: &VecEntityMap,
    sim: &SimWorld,
    id: VecPathId,
) -> Option<ph2d_vec_scene::arc_path::ArcPath> {
    let src = scene.path(id)?;
    let mut world = src.cooked().into_owned();
    let xf = crate::vec_transform::build(sim, map);
    bake_xform(&mut world, &xform_of(&xf, id));
    ph2d_vec_scene::arc_path::ArcPath::from_contour(&world.verts, world.closed)
}

/// O perfil de `id` — o vivo, ou o NEUTRO (duas paradas em `1.0`) quando ainda não há nenhum.
///
/// O neutro é o que a ferramenta mostra numa forma virgem: duas alças coladas à borda da fita, que
/// é onde a tinta de facto está. Sem isto o artista veria uma curva sem alça nenhuma e não teria
/// por onde começar.
fn profile_or_neutral(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> WidthStops {
    crate::profile_live::spec_of(sim, map, id).unwrap_or_else(|| {
        WidthStops::new(vec![
            WidthStop {
                pos: 0.0,
                mult: 1.0,
            },
            WidthStop {
                pos: 1.0,
                mult: 1.0,
            },
        ])
    })
}

/// **O braço de uma parada** — o que a mão agarra e o que ele mede.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct HandleView {
    /// A ficha agarrável, **SOBRE a curva**, no ponto da parada.
    pub at: [f64; 2],
    /// A ponta da haste: a borda da fita ali (`meia-largura × multiplicador` pela normal).
    pub tip: [f64; 2],
}

/// **Onde cada braço está**, em MUNDO — um por parada. Vazio sem traço (uma forma sem tinta não
/// tem largura a editar).
///
/// ⚠️ **A ficha fica na CURVA, e a haste é que vai até a borda da fita** (report do Enio,
/// 2026-07-30). A 1ª versão punha a ficha na borda — a manipulação direta de *onde a tinta
/// acaba* —, e a borda é `meia-largura × multiplicador` fora da curva: com o multiplicador alto
/// isso **atravessa a vizinhança**. MEDIDO num grampo de braços a `0,30`: um arrasto que produziu
/// multiplicador `3,75` sobre traço `0,16` pôs a ficha em `y = 0,30`, **exatamente sobre o outro
/// braço**. O artista clicava numa linha e a alça nascia na de ao lado, clicava outra vez, e
/// ficava com uma alça em cada segmento.
///
/// Com a ficha na curva, *"de que linha é esta alça?"* não tem resposta errada possível — e um
/// clique entre duas linhas próximas cria **uma** parada, na mais próxima do rato
/// ([`ph2d_vec_scene::arc_path::ArcPath::closest_arc`] já escolhia a certa; era o DESENHO que
/// mentia). É o *Width Tool* do Illustrator e os nós do *Power Stroke* do Inkscape.
///
/// A normal da haste é a esquerda da tangente, sempre o mesmo lado: a fita é simétrica, então
/// duas hastes por parada seriam dois controles para um número.
#[must_use]
pub(crate) fn handles(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    id: VecPathId,
) -> Vec<HandleView> {
    let (Some(hw), Some(arc)) = (half_width(scene, id), arc_of(scene, map, sim, id)) else {
        return Vec::new();
    };
    let total = arc.total();
    if total <= 0.0 {
        return Vec::new();
    }
    profile_or_neutral(sim, map, id)
        .as_slice()
        .iter()
        .map(|s| {
            let (p, t) = arc.frame_at(s.pos.clamp(0.0, 1.0) * total);
            let len = t[0].hypot(t[1]);
            let n = if len > 0.0 {
                [-t[1] / len, t[0] / len]
            } else {
                [0.0, 1.0]
            };
            let d = hw * s.mult;
            HandleView {
                at: p,
                tip: [p[0] + n[0] * d, p[1] + n[1] * d],
            }
        })
        .collect()
}

/// **Onde na curva o gesto pousou** — a resposta única de proximidade desta ferramenta.
struct Landing {
    /// A fração de arco do ponto do caminho mais próximo do rato.
    pos: f64,
    /// A parada que **já está ali**, se houver. `None` ⇒ o gesto cria uma.
    stop: Option<usize>,
}

/// **A proximidade é medida à LINHA, e a busca por parada corre AO LONGO dela** (Enio,
/// 2026-07-30: *"o melhor critério para escolher que segmento atuar é a proximidade do mouse em
/// relação à linha"*).
///
/// ⚠️ **A versão anterior perguntava no espaço LIVRE** — *existe alguma ficha a menos de 12 px do
/// rato?* — e num cruzamento isso é indecidível: as duas linhas passam a milímetros uma da outra,
/// então a alça da linha de CIMA ficava sempre dentro do raio de um clique dirigido à de BAIXO, e
/// engolia-o. Nenhum ajuste do raio salva: junto ao cruzamento a distância entre as linhas tende a
/// zero, e a resposta certa não é *"a ficha mais perto no plano"*.
///
/// Agora há **uma** pergunta de proximidade — [`ph2d_vec_scene::arc_path::ArcPath::closest_arc`],
/// que já escolhe o ramo mais próximo — e a segunda pergunta (*já há parada aqui?*) é feita em
/// **ARCO**, sobre o ramo que a primeira escolheu. Duas linhas que se cruzam estão a milímetros no
/// plano e a meio caminho uma da outra **ao longo do traço**: é isso que torna a escolha decidível,
/// e é a mesma grandeza em que a parada vive.
///
/// Porta única: o press agarra-ou-cria por aqui e o botão direito apaga por aqui. Duas buscas
/// divergiriam sobre o que é *"estar sob o cursor"*, e a do apagar acertaria uma alça diferente da
/// que o realce mostra.
fn landing(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    id: VecPathId,
    world_pt: [f64; 2],
    radius: f64,
) -> Option<Landing> {
    let arc = arc_of(scene, map, sim, id)?;
    let total = arc.total();
    if total <= 0.0 {
        return None;
    }
    let s = arc.closest_arc(world_pt);
    let (p, _) = arc.frame_at(s);
    // O gesto tem de estar SOBRE a linha. Fora do raio não há nem alça nem parada nova — é o
    // clique no vazio, e ele não pode acrescentar nada.
    if (p[0] - world_pt[0]).hypot(p[1] - world_pt[1]) > radius {
        return None;
    }
    let pos = (s / total).clamp(0.0, 1.0);
    // O mesmo raio, na unidade em que a parada vive: uma fração do arco total.
    let reach = radius / total;
    let stop = profile_or_neutral(sim, map, id)
        .as_slice()
        .iter()
        .enumerate()
        .map(|(k, st)| (k, (st.pos - pos).abs()))
        .filter(|&(_, d)| d <= reach)
        // A MAIS PRÓXIMA, não a primeira: com duas paradas dentro do alcance, a primeira da lista
        // é um acidente da ordenação, e agarrar a errada move um ponto que o artista não apontou.
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(k, _)| k);
    Some(Landing { pos, stop })
}

/// **A pressão**: agarra a alça sob o cursor, ou ACRESCENTA uma parada se o cursor está sobre a
/// curva. `None` quando o gesto não é deste caminho.
///
/// A parada nova nasce com o multiplicador que o perfil JÁ tem ali — o desenho não muda no toque,
/// e o arrasto seguinte é que o move. Uma parada que nascesse em `1.0` faria a fita saltar sob o
/// dedo antes de o artista pedir qualquer coisa.
#[must_use]
pub(crate) fn press(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    id: VecPathId,
    world_pt: [f64; 2],
    radius: f64,
) -> Option<Grab> {
    let l = landing(sim, scene, map, id, world_pt, radius)?;
    let cur = profile_or_neutral(sim, map, id);
    if let Some(k) = l.stop {
        // ⚠️ **Agarrar NÃO escreve nada**, e a tentação de armar aqui morre na própria lei do
        // módulo: o perfil de uma forma virgem é o NEUTRO, que é uniforme, e `arm` REMOVE um
        // perfil uniforme (o neutro-é-ausência). Armar no press seria escrever e apagar no mesmo
        // gesto — e o arrasto seguinte não acharia lista nenhuma. Quem lê o neutro é o `drag`.
        return Some(Grab {
            path: id,
            stop: k,
            created: false,
            pos: cur.as_slice().get(k).map_or(0.0, |s| s.pos),
        });
    }
    // Não há parada onde o dedo pousou: nasce uma ali.
    let pos = l.pos;
    let stops = WidthStops::new(insert_stop(cur.as_slice(), pos, cur.at(pos)));
    let k = stop_index(&stops, pos)?;
    arm_stops(sim, map, id, &stops);
    Some(Grab {
        path: id,
        stop: k,
        created: true,
        pos,
    })
}

/// A lista `base` com uma parada acrescentada em `pos` — **ordenada**, a mesma que o `press`
/// produz. Porta única: o press a escreve e o `drag` a REPRODUZ quando ela não chegou a ser
/// guardada (perfil uniforme), e duas construções divergiriam sobre o índice da parada nova.
fn insert_stop(base: &[WidthStop], pos: f64, mult: f64) -> Vec<WidthStop> {
    let mut v = base.to_vec();
    v.push(WidthStop { pos, mult });
    v
}

/// O índice da parada que senta em `pos` — a resposta que o [`Grab`] guarda.
fn stop_index(stops: &WidthStops, pos: f64) -> Option<usize> {
    stops
        .as_slice()
        .iter()
        .position(|st| (st.pos - pos).abs() < 1e-12)
}

/// **A lista que este arrasto edita, e o índice nela.**
///
/// Normalmente é o perfil vivo tal como está. O caso que exige esta porta é a forma VIRGEM: a
/// parada que o `press` criou não chegou a ser guardada (a lista ficou uniforme e o `arm` a
/// removeu), então reconstruí-la aqui é o que impede o arrasto de editar a parada errada — a
/// ponta do traço. Ver [`Grab::pos`] para o número que isso valia.
fn working_stops(
    sim: &SimWorld,
    map: &VecEntityMap,
    grab: Grab,
) -> Option<(Vec<WidthStop>, usize)> {
    if let Some(cur) = crate::profile_live::spec_of(sim, map, grab.path) {
        return Some((cur.as_slice().to_vec(), grab.stop));
    }
    let neutral = profile_or_neutral(sim, map, grab.path);
    if !grab.created {
        return Some((neutral.as_slice().to_vec(), grab.stop));
    }
    let stops = WidthStops::new(insert_stop(
        neutral.as_slice(),
        grab.pos,
        neutral.at(grab.pos),
    ));
    let k = stop_index(&stops, grab.pos)?;
    Some((stops.as_slice().to_vec(), k))
}

/// **O arrasto**: a distância à curva vira o multiplicador, a projeção na curva vira a posição.
///
/// As duas de uma vez, e não uma por eixo: o dedo aponta um LUGAR, e o lugar responde as duas
/// perguntas. Separá-las (uma tecla para cada) seria pedir ao artista que soubesse qual metade da
/// alça ele está a mover.
pub(crate) fn drag(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    grab: Grab,
    world_pt: [f64; 2],
) -> bool {
    let (Some(hw), Some(arc)) = (
        half_width(scene, grab.path),
        arc_of(scene, map, sim, grab.path),
    ) else {
        return false;
    };
    let total = arc.total();
    if total <= 0.0 {
        return false;
    }
    // O NEUTRO quando ainda não há perfil: é o que as alças mostram numa forma virgem, e é dele
    // que o 1º arrasto parte. Exigir um componente aqui tornaria a ferramenta inerte justamente
    // na forma em que ela é usada pela 1ª vez.
    let Some((mut v, k)) = working_stops(sim, map, grab) else {
        return false;
    };
    let Some(st) = v.get_mut(k) else {
        return false;
    };
    let s = arc.closest_arc(world_pt);
    let (p, _) = arc.frame_at(s);
    // ⚠️ A distância é ABSOLUTA (sem sinal): a alça vive de um lado só, e deixar o multiplicador
    // ficar negativo viraria a fita do avesso — uma largura negativa não é uma largura.
    let d = (world_pt[0] - p[0]).hypot(world_pt[1] - p[1]);
    st.pos = (s / total).clamp(0.0, 1.0);
    st.mult = (d / hw).clamp(0.0, MAX_MULT);
    let stops = WidthStops::new(v);
    arm_stops(sim, map, grab.path, &stops);
    true
}

/// **Desfaz uma parada que nasceu num clique e nunca foi movida** — ver [`Grab::created`]. No-op
/// para uma alça agarrada (ela já existia) ou para um gesto que arrastou.
pub(crate) fn discard_if_untouched(sim: &mut SimWorld, map: &VecEntityMap, grab: Grab) {
    if !grab.created {
        return;
    }
    let Some(cur) = crate::profile_live::spec_of(sim, map, grab.path) else {
        return;
    };
    let mut v: Vec<WidthStop> = cur.as_slice().to_vec();
    if grab.stop >= v.len() {
        return;
    }
    v.remove(grab.stop);
    let stops = if v.len() < 2 {
        WidthStops::default()
    } else {
        WidthStops::new(v)
    };
    arm_stops(sim, map, grab.path, &stops);
}

/// **Apagar** a parada sob o cursor (o botão direito). `true` se apagou.
///
/// Abaixo de duas paradas não sobra perfil: a lista é limpa e o traço volta a ser o uniforme de
/// sempre — o mesmo neutro-é-ausência das outras rotas, em vez de deixar uma parada solta a
/// governar a largura inteira por um caminho que ninguém mais usa.
pub(crate) fn remove(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    id: VecPathId,
    world_pt: [f64; 2],
    radius: f64,
) -> bool {
    let Some(k) = landing(sim, scene, map, id, world_pt, radius).and_then(|l| l.stop) else {
        return false;
    };
    let cur = profile_or_neutral(sim, map, id);
    let mut v: Vec<WidthStop> = cur.as_slice().to_vec();
    if k >= v.len() {
        return false;
    }
    v.remove(k);
    let stops = if v.len() < 2 {
        WidthStops::default()
    } else {
        WidthStops::new(v)
    };
    arm_stops(sim, map, id, &stops);
    true
}

/// O teto do multiplicador que uma alça alcança. **MEDIDO** contra o próprio motor: acima de
/// ~6× a fita de um traço fino deixa de ler como traço e vira uma mancha, e o `power_stroke` passa
/// a gastar o dobro do tempo no sweep (a fita auto-intersecta mais). O limite existe para que
/// arrastar a alça para fora da tela não produza uma forma que o artista não consegue desfazer
/// visualmente — o Ctrl+Z continua lá, mas a fita já cobriu o desenho todo.
const MAX_MULT: f64 = 6.0;

/// Escreve a lista pela porta única — a MESMA dos sliders e do lápis.
fn arm_stops(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId, stops: &WidthStops) {
    crate::profile_live::arm(sim, map, &[id], stops);
}

#[cfg(test)]
#[path = "width_handles_tests.rs"]
mod tests;
