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
//!
//! ⭐⭐⭐ **E o PESO é do NÓ** (ordem do dono, 2026-09-19): numa forma as três metades de um vértice
//! movem-se pelos pesos da ÂNCORA, e quem diz isso é UMA porta — [`ph2d_vec_skin::dono_do_peso`],
//! a mesma que o laço da deformação exprime. ⛔ Escrever `k % 3` aqui seria a segunda resposta à
//! pergunta de que mídia é o ponto, e o sintoma seria o indicador a pintar um peso que a arte não
//! tem.

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
    if let Some(g) = crate::skinned_mesh::le(&skin.source) {
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

/// ⭐⭐ **DE QUE PONTO SAI O PESO DO PONTO `k`** — a lei da mídia, num sítio só.
///
/// Numa forma é a ÂNCORA da tripla ([`ph2d_vec_skin::dono_do_peso`]); numa imagem cada vértice é o
/// dono do peso dele. ⚠️ **Os três leitores desta crate passam por aqui** — o instantâneo posado, os
/// pontos com peso e a escolha do ponto sob o cursor —, porque *três cópias de `k % 3` divergiriam
/// no dia em que uma mídia nova chegasse*.
const fn dono_do_peso(caminho: bool, k: usize) -> usize {
    if caminho {
        ph2d_vec_skin::dono_do_peso(k)
    } else {
        k
    }
}

/// **As posições POSADAS da arte**, na mesma ordem dos [`repousos`] — o que o artista está a ver.
///
/// ⚠️ Ela existe para a [`pele_sob_o_cursor`] poder perguntar *«o cursor está DENTRO desta arte?»*,
/// que é uma pergunta sobre a SILHUETA e não sobre os pontos dela.
#[must_use]
fn posados(sim: &SimWorld, alvo: Entity, ppm: f32) -> Vec<[f64; 2]> {
    let Some(skin) = sim.world().get::<SkinBind>(alvo).cloned() else {
        return Vec::new();
    };
    let Some(pele) = crate::skin_live::skin_of(sim, alvo) else {
        return Vec::new();
    };
    let repousos = repousos(sim, alvo, ppm);
    let guardados = pesos_guardados(&skin, repousos.len());
    let n = pele.len();
    let correcoes = skin.correcoes_resolvidas();
    let (x, _) = mundo_e_escala(sim, alvo);
    let mut w = pele.scratch();
    let cam = e_caminho(sim, alvo);
    repousos
        .iter()
        .enumerate()
        .map(|(k, &p)| {
            // ⚠️ **O peso sai do DONO e a mistura move ESTE ponto** — se o instantâneo posado
            // usasse o peso do próprio ponto, o hit-test mediria uma silhueta que o desenho não
            // tem, e o dedo tocaria num sítio com a arte noutro.
            let d = dono_do_peso(cam, k);
            let base = repousos.get(d).copied().unwrap_or(p);
            let fatia = guardados
                .as_ref()
                .and_then(|g| g.get(d * n..(d + 1) * n))
                .filter(|_| n != 0);
            pele.weights_corrected(base, fatia, &mut w, &correcoes);
            x.apply(pele.blend(p, &w))
        })
        .collect()
}

/// **O contorno POSADO de uma forma, achatado em segmentos.**
///
/// ⚠️ **As triplas são `(âncora, alça de entrada, alça de saída)`** — ver [`repousos`] —, logo o
/// troço entre o vértice `k` e o `k+1` é a cúbica `(a_k, out_k, in_{k+1}, a_{k+1})`.
///
/// ⛔ **Achatada à mão e não pela `kurbo`:** esta crate é uma folha e não depende de geometria
/// vectorial. `SEGMENTOS` é grosseiro de propósito — o consumidor é um teste de ponto, não um
/// rasterizador.
fn contorno_posado(pts: &[[f64; 2]]) -> Vec<[f64; 2]> {
    const SEGMENTOS: usize = 8;
    let n = pts.len() / 3;
    if n < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(n * SEGMENTOS);
    for k in 0..n {
        let (a, o) = (pts[3 * k], pts[3 * k + 2]);
        let j = (k + 1) % n;
        let (i, b) = (pts[3 * j + 1], pts[3 * j]);
        for s in 0..SEGMENTOS {
            let t = s as f64 / SEGMENTOS as f64;
            let u = 1.0 - t;
            let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            out.push([
                w0 * a[0] + w1 * o[0] + w2 * i[0] + w3 * b[0],
                w0 * a[1] + w1 * o[1] + w2 * i[1] + w3 * b[1],
            ]);
        }
    }
    out
}

/// **O ponto está dentro do polígono?** — regra do número de voltas por cruzamentos (par/ímpar).
fn dentro(poli: &[[f64; 2]], q: [f64; 2]) -> bool {
    let mut dentro = false;
    let n = poli.len();
    for i in 0..n {
        let (a, b) = (poli[i], poli[(i + 1) % n]);
        if (a[1] > q[1]) != (b[1] > q[1]) {
            let t = (q[1] - a[1]) / (b[1] - a[1]);
            if q[0] < a[0] + t * (b[0] - a[0]) {
                dentro = !dentro;
            }
        }
    }
    dentro
}

/// A distância ao segmento `ab` — o que cobre um caminho ABERTO, que não tem interior.
fn d2_segmento(a: [f64; 2], b: [f64; 2], q: [f64; 2]) -> f64 {
    let (vx, vy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = vx * vx + vy * vy;
    if l2 <= f64::EPSILON {
        return d2(a, q);
    }
    let t = (((q[0] - a[0]) * vx + (q[1] - a[1]) * vy) / l2).clamp(0.0, 1.0);
    d2([a[0] + t * vx, a[1] + t * vy], q)
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
    // ⭐ Quem CONTÉM o cursor ganha de quem apenas está perto: com duas artes sobrepostas, o dedo
    // está numa delas, e a outra só passa por ali. Entre dois que contêm, o mais perto do contorno.
    let mut dentro_de: Option<(f64, Entity)> = None;
    let mut perto_de: Option<(f64, Entity)> = None;
    let peles: Vec<Entity> = sim
        .world()
        .iter_entities()
        .filter(|er| er.get::<SkinBind>().is_some())
        .map(|er| er.id())
        .collect();
    for e in peles {
        let pts = posados(sim, e, ppm);
        if pts.is_empty() {
            continue;
        }
        // ⛔⛔ **A pergunta é ao que a PELE GUARDA, e nunca «tem `Sprite`?»** — a 1.ª redacção
        // perguntava o segundo, e no app (ao contrário da fixtura de unidade) uma FORMA também
        // carrega um `Sprite`: toda arte vectorial ia pelo ramo da imagem, onde o `SkinnedMesh`
        // não parseia, a porta devolvia «não achei» e a tela ficava sem pontos — *o mesmo sintoma
        // do report, com outra causa por baixo, e só a FOTOGRAFIA o mostrou*.
        //
        // ⚠️ E a porta certa já existia neste ficheiro (a [`e_caminho`], escrita para o indicador):
        // *duas respostas à mesma pergunta divergem, e estas divergiram em duas horas*.
        let (contem, d) = if e_caminho(sim, e) {
            let poli = contorno_posado(&pts);
            if poli.is_empty() {
                (
                    false,
                    pts.iter()
                        .map(|&p| d2(p, mundo))
                        .fold(f64::INFINITY, f64::min),
                )
            } else {
                let d = (0..poli.len())
                    .map(|i| d2_segmento(poli[i], poli[(i + 1) % poli.len()], mundo))
                    .fold(f64::INFINITY, f64::min);
                (dentro(&poli, mundo), d)
            }
        } else {
            malha_sob(sim, e, ppm, &pts, mundo)
        };
        if contem {
            if dentro_de.is_none_or(|(b, _)| d < b) {
                dentro_de = Some((d, e));
            }
        } else if d <= raio_mundo * raio_mundo && perto_de.is_none_or(|(b, _)| d < b) {
            perto_de = Some((d, e));
        }
    }
    dentro_de.or(perto_de).map(|(_, e)| e)
}

/// **A malha de uma IMAGEM sob o cursor** — dentro de um triângulo posado, ou perto de uma aresta.
///
/// ⚠️ **Ela é separada do caminho porque a geometria é outra:** uma imagem não tem contorno de
/// bézier, tem triângulos — e perguntar a silhueta dela pelo casco seria uma terceira resposta.
fn malha_sob(
    sim: &SimWorld,
    alvo: Entity,
    _ppm: f32,
    posados: &[[f64; 2]],
    mundo: [f64; 2],
) -> (bool, f64) {
    let Some(skin) = sim.world().get::<SkinBind>(alvo) else {
        return (false, f64::INFINITY);
    };
    let Ok(g) = postcard::from_bytes::<crate::skinned_mesh::SkinnedMesh>(&skin.source) else {
        return (false, f64::INFINITY);
    };
    let mut dentro = false;
    let mut d = f64::INFINITY;
    for t in &g.mesh.tris {
        let Some(a) = posados.get(t[0] as usize) else {
            continue;
        };
        let (Some(b), Some(c)) = (posados.get(t[1] as usize), posados.get(t[2] as usize)) else {
            continue;
        };
        let (a, b, c) = (*a, *b, *c);
        // Sinal dos três produtos cruzados: dentro quando eles concordam.
        let cruz = |p: [f64; 2], q: [f64; 2]| {
            (q[0] - p[0]) * (mundo[1] - p[1]) - (q[1] - p[1]) * (mundo[0] - p[0])
        };
        let (s1, s2, s3) = (cruz(a, b), cruz(b, c), cruz(c, a));
        if (s1 >= 0.0 && s2 >= 0.0 && s3 >= 0.0) || (s1 <= 0.0 && s2 <= 0.0 && s3 <= 0.0) {
            dentro = true;
        }
        d = d
            .min(d2_segmento(a, b, mundo))
            .min(d2_segmento(b, c, mundo))
            .min(d2_segmento(c, a, mundo));
    }
    (dentro, d)
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
    let cam = e_caminho(sim, alvo);
    repousos
        .iter()
        .enumerate()
        .map(|(k, &p)| {
            // ⚠️ **Uma conta só, e o peso é do DONO** — a 1.ª redacção chamava `weights_corrected`
            // e a seguir `point_corrected`, que refaz a mesma conta: o peso lido e o peso que moveu
            // o ponto eram derivados duas vezes, e bastava uma das duas ganhar uma cerca para o
            // indicador passar a mostrar um número que a arte não usa.
            let d = dono_do_peso(cam, k);
            let base = repousos.get(d).copied().unwrap_or(p);
            let fatia = guardados
                .as_ref()
                .and_then(|g| g.get(d * n..(d + 1) * n))
                .filter(|_| n != 0);
            pele.weights_corrected(base, fatia, &mut w, &correcoes);
            PontoDaPele {
                repouso: p,
                mundo: x.apply(pele.blend(p, &w)),
                peso: w.get(tendao).copied().unwrap_or(0.0),
            }
        })
        .collect()
}

/// ⭐⭐⭐ **OS PONTOS QUE CARREGAM PESO** — os que o pincel pode pousar e o indicador pode pintar.
///
/// Numa imagem são todos; numa forma são os **NÓS**, porque é neles que o peso vive
/// ([`ph2d_vec_skin::dono_do_peso`]).
///
/// ⚠️ **Ela tem DOIS consumidores e é por isso que existe:** o indicador (o que se vê) e o
/// [`ponto_sob_o_cursor`] (onde a mancha é ancorada). ⛔ Com a escolha escrita só no indicador, o
/// pincel continuava a ancorar manchas em ALÇAS — o centro da correcção cairia a meio caminho de um
/// nó, e o artista via a cor mudar num sítio onde não carregou.
#[must_use]
pub fn pontos_de_peso(sim: &SimWorld, alvo: Entity, osso: Entity, ppm: f32) -> Vec<PontoDaPele> {
    let cam = e_caminho(sim, alvo);
    pontos_da_pele(sim, alvo, osso, ppm)
        .into_iter()
        .enumerate()
        .filter(|(k, _)| !cam || ph2d_vec_skin::e_no(*k))
        .map(|(_, p)| p)
        .collect()
}

/// ⭐⭐⭐ **O QUE O INDICADOR MOSTRA** — os pontos de TODA a arte que este osso governa, com o peso.
///
/// ⛔⛔⛔ **Report do dono (2026-09-19): *«As cores só aparecem se o mouse estiver sobre a forma»*** —
/// e ele tem razão. A redacção anterior perguntava *«que arte está debaixo do dedo?»* e devolvia
/// **vazio** no vão entre as formas, logo a tela acendia e apagava ao passar o rato. ⚠️ **A
/// justificação que estava escrita aqui — *«o pincel escolhe a arte pela ponta do dedo, logo a
/// pré-visualização tem de responder à MESMA pergunta»* — confundia DUAS perguntas:** *onde o traço
/// vai pintar* (que é do dedo, e continua a ser) e *o que o osso governa* (que é do OSSO, e não tem
/// nada que ver com onde o rato está). ⇒ a premissa morreu e a morte está à vista neste diff.
///
/// ⭐ **A população é EXACTA e não uma escolha:** as peles cujos tendões contêm este osso. Uma que
/// não o tenha é a que o [`pinta`] recusa com [`Pincelada::OssoDeFora`] — *pintá-la de azul
/// prometeria um pincel que a porta ao lado recusa*.
///
/// ⛔⛔ **E quem responde a isso é o [`tendao_de`], UMA camada abaixo** — esta função varre todas as
/// peles e não filtra nenhuma. A 1.ª redacção filtrava aqui por `tendons.iter().any(...)`, e uma
/// **mutação SOBREVIVENTE** mostrou que aquilo era a segunda resposta à mesma pergunta: apagá-la
/// não mudava um único ponto, porque o [`pontos_da_pele`] já devolve vazio (e devolve-o **antes**
/// de resolver os repousos, logo nem o custo era o que eu supunha). *Duas respostas à mesma
/// pergunta não se mantêm por serem baratas — tira-se a que não é a lei.*
///
/// Vazio sem osso em foco: não há de quem mostrar peso.
#[must_use]
pub fn pontos_do_indicador(sim: &SimWorld, ppm: f32, osso: Option<Entity>) -> Vec<([f64; 2], f64)> {
    let Some(osso) = osso else {
        return Vec::new();
    };
    // ⚠️ **A varredura junta-se ANTES de perguntar os pesos** — o `iter_entities` empresta o mundo,
    // e o [`pontos_de_peso`] volta a lê-lo.
    let peles: Vec<Entity> = sim
        .world()
        .iter_entities()
        .filter(|er| er.get::<SkinBind>().is_some())
        .map(|er| er.id())
        .collect();
    let mut fora = Vec::new();
    for alvo in peles {
        fora.extend(
            pontos_de_peso(sim, alvo, osso, ppm)
                .into_iter()
                .map(|q| (q.mundo, q.peso)),
        );
    }
    fora
}

/// **Esta arte é um CAMINHO?** (a alternativa é uma imagem, que não tem alças).
///
/// ⚠️ Ela pergunta ao que a pele GUARDA e não ao mundo — é a mesma fonte que os [`repousos`] leem
/// para decidir se emitem triplas, e duas respostas divergiriam no primeiro formato novo.
#[must_use]
fn e_caminho(sim: &SimWorld, alvo: Entity) -> bool {
    sim.world()
        .get::<SkinBind>(alvo)
        .is_some_and(|skin| crate::skinned_mesh::le(&skin.source).is_some())
}

/// A tabela do padrão-ouro, quando ela fecha com a contagem de pontos desta pele.
fn pesos_guardados(skin: &SkinBind, pontos: usize) -> Option<Vec<f64>> {
    if let Some(g) = crate::skinned_mesh::le(&skin.source) {
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
///
/// ⛔⛔ **E ela escolhe entre os pontos que CARREGAM peso** ([`pontos_de_peso`]), nunca entre todos:
/// ancorar a mancha numa ALÇA punha o centro da correcção num sítio que não pesa nada, e o peso do
/// nó ao lado subia por tabela. *É a segunda metade do report de 2026-09-19 — «pesos em alças» não
/// era só o desenho, era também onde o gesto pousava.*
#[must_use]
pub fn ponto_sob_o_cursor(
    sim: &SimWorld,
    alvo: Entity,
    osso: Entity,
    ppm: f32,
    mundo: [f64; 2],
) -> Option<PontoDaPele> {
    pontos_de_peso(sim, alvo, osso, ppm)
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

/// ⭐ Os gates do INDICADOR — irmãos dos de cima pelo tecto de LOC, cortados por assunto: aqueles
/// medem o que uma pincelada FAZ, estes o que o app MOSTRA antes de ela acontecer.
#[cfg(test)]
#[path = "peso_a_mao_indicador_tests.rs"]
mod peso_a_mao_indicador_tests;
