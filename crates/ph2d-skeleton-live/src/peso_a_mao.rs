//! **O PESO À MÃO** — o gesto e o olho da [`ph2d_skeleton_ecs::CorreccaoDePeso`].
//!
//! A lei já vive na [`ph2d_skeleton::Skin`] e o armazenamento na [`ph2d_skeleton_ecs::SkinBind`];
//! o que falta é o que só quem tem o MUNDO na mão pode responder: **onde, no repouso, está o barro
//! que o dedo aponta**, e **quanto pesa ali o osso em foco**.
//!
//! ⭐⭐⭐ **A MANCHA É PINTADA SOBRE A POSE E GUARDADA NO REPOUSO, e essa tradução é a razão de este
//! módulo existir.** O artista vê a arte deformada — é lá que ele vê o defeito — e a correcção tem
//! de ser ancorada na geometria de repouso, senão ela andava com a pose e corrigia o sítio errado
//! no quadro seguinte. ⇒ o dedo escolhe o ponto **POSADO** mais perto, e o que se guarda é o
//! **repouso desse mesmo ponto**.
//!
//! ⛔⛔ **Não é uma segunda lei de pesos.** Quem calcula o peso aqui é a MESMA porta que o quadro
//! corre ([`ph2d_skeleton::Skin::weights_corrected`], com as correcções já aplicadas): o olho vê o
//! que a arte tem, e não o que ela teria sem o que o artista já pintou. *Uma pré-visualização que
//! ignora o trabalho feito é uma segunda resposta à mesma pergunta.*
//!
//! ⚠️ **As duas mídias respondem pela mesma porta** ([`repousos`]) e a diferença entre elas é uma
//! só: *o que é um ponto aqui*. Numa forma são as três metades de cada vértice (a âncora e as duas
//! alças, que é o que a [`ph2d_vec_skin`] deforma); numa imagem são os vértices da malha do bind,
//! levados de pixels ao local pelo mesmo afim que a desenha.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton::Xform;
use ph2d_skeleton_ecs::{CorreccaoDePeso, SkinBind};

/// **Um ponto da pele** — onde ele está no repouso, onde está agora, e quanto o osso perguntado
/// manda nele.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PontoDaPele {
    /// No espaço da coisa no bind — o mesmo da [`ph2d_skeleton_ecs::CorreccaoDePeso::centro`].
    pub repouso: [f64; 2],
    /// Onde ele está AGORA, em mundo. É o que o dedo aponta e o que o olho vê.
    pub mundo: [f64; 2],
    /// O peso do osso perguntado, **já com as correcções** — `0.0` quando ele não manda nada ali.
    pub peso: f64,
}

/// **O que uma pincelada fez** — e, quando não fez nada, PORQUÊ.
///
/// ⚠️ **As três recusas são distintas de propósito.** *«A ferramenta não faz nada»* é o report que
/// esta casa já pagou muitas vezes, e a cura é sempre a mesma: o gesto diz qual das entradas lhe
/// falta em vez de ficar calado. Aqui são três, e a cura de cada uma é outra — prender o desenho,
/// escolher um osso, apontar para a arte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pincelada {
    /// A coisa apontada não está presa a esqueleto nenhum. Cura: *Bind*.
    SemPele,
    /// O osso em foco não é um dos tendões desta pele. Cura: escolher um que seja.
    OssoDeFora,
    /// O cursor não caiu sobre a arte — não há ponto de repouso para ancorar a mancha.
    ForaDaArte,
    /// Pintou. `manchas` é quantas a pele guarda depois desta.
    Pintada { manchas: usize },
}

/// ⭐⭐ **QUANTAS MANCHAS UMA PELE GUARDA** — o tecto, e o recurso dele é o **RELÓGIO DO QUADRO**.
///
/// A correcção é `O(pontos × manchas)` dentro do recook, que corre **todo quadro**. Medido
/// (`--release`, em `o_tecto_de_manchas_custa_uma_razao_e_nao_uma_ordem_de_grandeza`):
///
/// | pele | sem manchas | com o tecto cheio | |
/// |---|---|---|---|
/// | estrela (30 pontos), pelo recook do produto | `0,96 µs` | `2,80 µs` | `2,9×` |
/// | **2 000 pontos** (o pior que esta casa produz — a malha graduada de uma imagem) | — | **`205 µs`** | `1,2 %` de um quadro de 60 Hz |
///
/// ⇒ o tecto é **generoso e não apertado**: ele está `8×` abaixo do décimo de quadro que o gate
/// exige, e quem o quiser subir tem a tabela para o fazer. O que ele impede é o caso degenerado —
/// um arrasto de minutos a somar uma mancha por evento de ponteiro.
///
/// ⛔ **Não é «um número razoável»**: passar dele não corrompe nada — a mancha mais antiga do mesmo
/// osso é que cede o lugar, que é a leitura certa de *«continuei a pintar»*.
pub const MANCHAS_MAX: usize = 128;

/// ⭐ **QUANDO DUAS PINCELADAS SÃO A MESMA** — a fracção do raio abaixo da qual a nova FUNDE-SE na
/// que já lá estava, somando o `delta`.
///
/// ⚠️ **Sem ela um arrasto de três segundos deixaria uma mancha por evento de ponteiro** (~60/s), e
/// o tecto acima seria gasto num gesto. Com ela, o que o artista paga é a DISTÂNCIA percorrida, que
/// é o que ele vê.
///
/// ⚠️ **E a fusão é o que faz insistir no mesmo sítio EMPURRAR MAIS** — sem ela, a segunda passagem
/// sobre o mesmo ponto escreveria uma mancha idêntica ao lado e o efeito somaria na mesma, mas o
/// custo cresceria sem fim.
pub const FUSAO: f64 = 0.5;

/// O afim local→mundo da coisa, e o factor de escala UNIFORME dele.
///
/// ⚠️ **`√|det|` é a lei da casa para «quanto isto aumenta»**, e não é uma invenção deste módulo:
/// é o factor que o bug #27 fixou para a caneta sob escala não-uniforme (a média geométrica — para
/// escala uniforme é a própria escala, e é invariante à rotação).
fn mundo_e_escala(sim: &SimWorld, alvo: Entity) -> (Xform, f64) {
    let x = crate::skin_live::world_of(sim, alvo);
    let [a, b, c, d, _, _] = x.0;
    let det = d.mul_add(a, -(b * c));
    (x, det.abs().sqrt())
}

/// ⭐⭐⭐ **OS PONTOS DE REPOUSO DE UMA PELE — a porta única das duas mídias.**
///
/// `ppm` é o `pixels_per_meter` do projecto, e só a imagem o lê (é o mesmo que o
/// [`crate::skin_image::pixel_to_local`] pede, e o mesmo que o bind usou).
///
/// ⛔ **A ORDEM é a que a lei percorre**, e tem de ser: numa forma, `3k`, `3k+1`, `3k+2` são a
/// âncora e as duas alças do vértice `k` — exactamente o que a [`ph2d_vec_skin::aplica_corrigido`]
/// escreve. Uma segunda ordem aqui daria pesos plausíveis sobre os pontos errados.
#[must_use]
pub fn repousos(sim: &SimWorld, alvo: Entity, ppm: f32) -> Vec<[f64; 2]> {
    let Some(skin) = sim.world().get::<SkinBind>(alvo) else {
        return Vec::new();
    };
    if let Ok(g) = postcard::from_bytes::<crate::skinned_mesh::SkinnedPath>(&skin.source) {
        let mut out = Vec::new();
        let mut caminho = g.path;
        caminho.for_each_vert_mut(|v| {
            out.push(v.anchor);
            out.push(v.in_handle);
            out.push(v.out_handle);
        });
        return out;
    }
    let Ok(g) = postcard::from_bytes::<crate::skinned_mesh::SkinnedMesh>(&skin.source) else {
        return Vec::new();
    };
    let Some(sprite) = sim.world().get::<ph2d_render::Sprite>(alvo) else {
        return Vec::new();
    };
    let Some(p2l) = crate::skin_image::pixel_to_local(sprite, g.mesh.size, ppm) else {
        return Vec::new();
    };
    g.mesh.rest.iter().map(|&q| p2l.apply(q)).collect()
}

/// ⭐⭐⭐ **QUE ARTE PRESA ESTÁ SOB O CURSOR** — a pergunta do pen-down do pincel de peso.
///
/// ⚠️ **Ela pergunta à PELE e não à cena vectorial**, e é o que a faz valer nas duas mídias: uma
/// imagem presa não tem `VecPath` nenhum, logo um `path_at` responderia *«não há nada aqui»* sobre
/// a arte que o artista está a ver. ⛔ Duas perguntas — uma por mídia — divergiriam no primeiro
/// ajuste da tolerância, e o sintoma seria o pincel a funcionar num desenho e não no outro.
///
/// Devolve a pele cujo ponto POSADO mais perto cai dentro de `raio_mundo`; entre duas, a mais
/// perto. `None` quando o dedo não caiu em arte nenhuma.
#[must_use]
pub fn pele_sob_o_cursor(
    sim: &SimWorld,
    ppm: f32,
    mundo: [f64; 2],
    raio_mundo: f64,
) -> Option<Entity> {
    let mut melhor: Option<(f64, Entity)> = None;
    let peles: Vec<Entity> = sim
        .world()
        .iter_entities()
        .filter(|er| er.get::<SkinBind>().is_some())
        .map(|er| er.id())
        .collect();
    for e in peles {
        let (x, _) = mundo_e_escala(sim, e);
        let Some(pele) = crate::skin_live::skin_of(sim, e) else {
            continue;
        };
        let mut w = pele.scratch();
        let d = repousos(sim, e, ppm)
            .into_iter()
            .map(|p| d2(x.apply(pele.point_corrected(p, None, &mut w, &[])), mundo))
            .fold(f64::INFINITY, f64::min);
        if d <= raio_mundo * raio_mundo && melhor.is_none_or(|(b, _)| d < b) {
            melhor = Some((d, e));
        }
    }
    melhor.map(|(_, e)| e)
}

/// **Os pontos da pele, com o peso do osso perguntado.** Vazio quando a coisa não tem pele.
///
/// ⚠️ **O peso é lido pela porta do PRODUTO** ([`ph2d_skeleton::Skin::weights_corrected`]) — o mesmo cálculo que o
/// recook faz, com as correcções já dentro. *Uma pré-visualização com a lei crua mostraria a arte
/// que o artista já corrigiu como se ele não a tivesse tocado.*
#[must_use]
pub fn pontos_da_pele(sim: &SimWorld, alvo: Entity, osso: Entity, ppm: f32) -> Vec<PontoDaPele> {
    let Some(skin) = sim.world().get::<SkinBind>(alvo).cloned() else {
        return Vec::new();
    };
    let Some(pele) = crate::skin_live::skin_of(sim, alvo) else {
        return Vec::new();
    };
    let Some(tendao) = tendao_de(sim, &skin, osso) else {
        return Vec::new();
    };
    let repousos = repousos(sim, alvo, ppm);
    let guardados = pesos_guardados(&skin, repousos.len());
    let n = pele.len();
    let correcoes = skin.correcoes_resolvidas();
    let (x, _) = mundo_e_escala(sim, alvo);
    let mut w = pele.scratch();
    repousos
        .iter()
        .enumerate()
        .map(|(k, &p)| {
            let fatia = guardados
                .as_ref()
                .and_then(|g| g.get(k * n..(k + 1) * n))
                .filter(|_| n != 0);
            pele.weights_corrected(p, fatia, &mut w, &correcoes);
            let peso = w.get(tendao).copied().unwrap_or(0.0);
            PontoDaPele {
                repouso: p,
                mundo: x.apply(pele.point_corrected(p, fatia, &mut w, &correcoes)),
                peso,
            }
        })
        .collect()
}

/// ⭐⭐⭐ **O QUE O INDICADOR MOSTRA** — os pontos da arte que ESTE traço governa, já com o peso.
///
/// ⚠️ **Ela existe porque a escolha do alvo é uma LEI e vivia no laço de desenho da shell**, onde
/// teste nenhum lhe chega: *o traço pertence à arte em que começou* (`preso`, congelado no
/// pen-down) e, fora do traço, à arte sob o dedo — a MESMA pergunta que o pen-down faz.
/// ⛔ Perguntar à SELECÇÃO seria uma segunda resposta: o pincel escolhe pela ponta do dedo, e
/// *mostrar os pesos de outra arte é pior que não mostrar nenhum*.
///
/// Vazio sem osso em foco (não há de quem mostrar peso) e vazio sem alvo (o dedo está no vão).
#[must_use]
pub fn pontos_do_indicador(
    sim: &SimWorld,
    ppm: f32,
    osso: Option<Entity>,
    preso: Option<Entity>,
    cursor: Option<[f64; 2]>,
    raio_mundo: f64,
) -> Vec<([f64; 2], f64)> {
    let Some(osso) = osso else {
        return Vec::new();
    };
    let Some(alvo) =
        preso.or_else(|| cursor.and_then(|w| pele_sob_o_cursor(sim, ppm, w, raio_mundo)))
    else {
        return Vec::new();
    };
    pontos_da_pele(sim, alvo, osso, ppm)
        .into_iter()
        .map(|p| (p.mundo, p.peso))
        .collect()
}

/// A tabela do padrão-ouro, quando ela fecha com a contagem de pontos desta pele.
fn pesos_guardados(skin: &SkinBind, pontos: usize) -> Option<Vec<f64>> {
    if let Ok(g) = postcard::from_bytes::<crate::skinned_mesh::SkinnedPath>(&skin.source) {
        let fecha: &[f64] = if g.valida() { &g.pesos } else { &[] };
        let p = skin.pesos_do_quadro(fecha);
        return (!p.is_empty() && p.len().is_multiple_of(pontos.max(1))).then(|| p.to_vec());
    }
    let g = postcard::from_bytes::<crate::skinned_mesh::SkinnedMesh>(&skin.source).ok()?;
    let fecha: &[f64] = if g.valida() { &g.pesos } else { &[] };
    let p = skin.pesos_do_quadro(fecha);
    (!p.is_empty() && p.len().is_multiple_of(pontos.max(1))).then(|| p.to_vec())
}

/// **O tendão deste osso nesta pele** — a MESMA lei que o
/// [`SkinBind::correcoes_resolvidas`] aplica, e por isso lida de uma porta só.
fn tendao_de(sim: &SimWorld, skin: &SkinBind, osso: Entity) -> Option<usize> {
    let id = *sim.world().get::<ph2d_ecs::StableId>(osso)?;
    skin.tendons.iter().position(|t| t.bone == id)
}

/// **O ponto de repouso cujo POSADO está mais perto do cursor** — a tradução que o gesto precisa.
///
/// ⛔ **Sem tecto de distância, de propósito:** quem decide se o dedo caiu na arte é o
/// [`pinta`], e ele mede-o contra o RAIO do pincel — uma segunda régua aqui divergiria dela.
#[must_use]
pub fn ponto_sob_o_cursor(
    sim: &SimWorld,
    alvo: Entity,
    osso: Entity,
    ppm: f32,
    mundo: [f64; 2],
) -> Option<PontoDaPele> {
    pontos_da_pele(sim, alvo, osso, ppm)
        .into_iter()
        .filter(|p| p.mundo[0].is_finite() && p.mundo[1].is_finite())
        .min_by(|a, b| {
            d2(a.mundo, mundo)
                .partial_cmp(&d2(b.mundo, mundo))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn d2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dy.mul_add(dy, dx * dx)
}

/// ⭐⭐⭐ **PINTA UMA MANCHA** — o gesto inteiro, numa porta.
///
/// `raio_mundo` e `mundo` são o que o artista vê; o que fica guardado é o repouso, e o raio
/// convertido pela escala da coisa.
///
/// ⚠️ **O centro NÃO é o cursor — é o ponto da pele mais perto dele.** Ancorar no cursor cru
/// poria a mancha no vazio quando o dedo passa ao lado da arte, e ela deixaria de corrigir
/// exactamente quando o artista pensa que a pôs.
pub fn pinta(
    sim: &mut SimWorld,
    alvo: Entity,
    osso: Entity,
    ppm: f32,
    mundo: [f64; 2],
    raio_mundo: f64,
    delta: f64,
) -> Pincelada {
    if sim.world().get::<SkinBind>(alvo).is_none() {
        return Pincelada::SemPele;
    }
    let Some(id) = sim.world().get::<ph2d_ecs::StableId>(osso).copied() else {
        return Pincelada::OssoDeFora;
    };
    {
        let skin = sim
            .world()
            .get::<SkinBind>(alvo)
            .expect("a pele foi lida uma linha acima");
        if !skin.tendons.iter().any(|t| t.bone == id) {
            return Pincelada::OssoDeFora;
        }
    }
    let (_, escala) = mundo_e_escala(sim, alvo);
    if !(escala.is_finite() && escala > 0.0) {
        return Pincelada::ForaDaArte;
    }
    let Some(ponto) = ponto_sob_o_cursor(sim, alvo, osso, ppm, mundo) else {
        return Pincelada::ForaDaArte;
    };
    // ⛔ **O dedo tem de cair DENTRO do pincel.** Sem esta linha um clique do outro lado da tela
    // ancorava a mancha no ponto de arte mais próximo — longe, e sem o artista o pedir.
    if d2(ponto.mundo, mundo) > raio_mundo * raio_mundo {
        return Pincelada::ForaDaArte;
    }
    let raio = raio_mundo / escala;
    let mut skin = sim
        .world()
        .get::<SkinBind>(alvo)
        .expect("a pele foi lida acima")
        .clone();
    funde(&mut skin.correcoes, id, ponto.repouso, raio, delta);
    let manchas = skin.correcoes.len();
    sim.world_mut().entity_mut(alvo).insert(skin);
    Pincelada::Pintada { manchas }
}

/// A lei da fusão e do tecto — ver [`FUSAO`] e [`MANCHAS_MAX`].
fn funde(
    lista: &mut Vec<CorreccaoDePeso>,
    bone: ph2d_ecs::StableId,
    centro: [f64; 2],
    raio: f64,
    delta: f64,
) {
    let perto = FUSAO * raio;
    if let Some(c) = lista
        .iter_mut()
        .find(|c| c.bone == bone && d2(c.centro, centro) <= perto * perto)
    {
        // ⚠️ **O `delta` SOMA e satura em ±1**: o peso vive em `0..1`, logo empurrar duas vezes
        // para lá do topo não pode guardar um número que a lei depois corta — ele mentiria sobre
        // quanto falta para desfazer.
        c.delta = (c.delta + delta).clamp(-1.0, 1.0);
        c.raio = raio;
        return;
    }
    lista.push(CorreccaoDePeso {
        bone,
        centro,
        raio,
        delta,
    });
    if lista.len() > MANCHAS_MAX {
        // A mais ANTIGA do mesmo osso cede o lugar — ver [`MANCHAS_MAX`].
        if let Some(i) = lista.iter().position(|c| c.bone == bone) {
            lista.remove(i);
        } else {
            lista.remove(0);
        }
    }
}

#[cfg(test)]
#[path = "peso_a_mao_tests.rs"]
mod peso_a_mao_tests;
