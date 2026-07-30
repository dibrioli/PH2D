//! **O WIDTH TOOL** — as alças de largura na curva (plano 25 §5, ADR-0145).
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

/// **Onde cada alça está**, em MUNDO — uma por parada, deslocada pela NORMAL à distância que a
/// fita tem ali. Vazio sem traço (uma forma sem tinta não tem largura a editar).
///
/// A normal é a esquerda da tangente, sempre o mesmo lado: a fita é simétrica, então duas alças
/// por parada seriam dois controles para um número.
#[must_use]
pub(crate) fn handles(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    id: VecPathId,
) -> Vec<[f64; 2]> {
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
            [p[0] + n[0] * d, p[1] + n[1] * d]
        })
        .collect()
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
    let hs = handles(sim, scene, map, id);
    if let Some(k) = hs
        .iter()
        .position(|h| (h[0] - world_pt[0]).hypot(h[1] - world_pt[1]) <= radius)
    {
        // ⚠️ **Agarrar NÃO escreve nada**, e a tentação de armar aqui morre na própria lei do
        // módulo: o perfil de uma forma virgem é o NEUTRO, que é uniforme, e `arm` REMOVE um
        // perfil uniforme (o neutro-é-ausência). Armar no press seria escrever e apagar no mesmo
        // gesto — e o arrasto seguinte não acharia lista nenhuma. Quem lê o neutro é o `drag`.
        return Some(Grab {
            path: id,
            stop: k,
            created: false,
        });
    }
    // Não há alça sob o cursor: se ele está SOBRE a curva, nasce uma parada ali.
    let arc = arc_of(scene, map, sim, id)?;
    let total = arc.total();
    if total <= 0.0 {
        return None;
    }
    let s = arc.closest_arc(world_pt);
    let (p, _) = arc.frame_at(s);
    if (p[0] - world_pt[0]).hypot(p[1] - world_pt[1]) > radius {
        return None;
    }
    let pos = (s / total).clamp(0.0, 1.0);
    let cur = profile_or_neutral(sim, map, id);
    let mult = cur.at(pos);
    let mut v: Vec<WidthStop> = cur.as_slice().to_vec();
    v.push(WidthStop { pos, mult });
    let stops = WidthStops::new(v);
    let k = stops
        .as_slice()
        .iter()
        .position(|st| (st.pos - pos).abs() < 1e-12 && (st.mult - mult).abs() < 1e-12)?;
    arm_stops(sim, map, id, &stops);
    Some(Grab {
        path: id,
        stop: k,
        created: true,
    })
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
    let cur = profile_or_neutral(sim, map, grab.path);
    let mut v: Vec<WidthStop> = cur.as_slice().to_vec();
    let Some(st) = v.get_mut(grab.stop) else {
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
    let hs = handles(sim, scene, map, id);
    let Some(k) = hs
        .iter()
        .position(|h| (h[0] - world_pt[0]).hypot(h[1] - world_pt[1]) <= radius)
    else {
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
