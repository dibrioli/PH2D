//! ⭐⭐⭐⭐ **A CURVATURA DE UM OSSO, viva** — irmão do [`crate::skin_live`] pelo tecto de LOC, com o
//! corte por RESPONSABILIDADE: ali mora *a pele de uma coisa, re-cozida por quadro*, aqui **de onde
//! vêm as duas alças que arqueiam o corpo e onde elas estão**.
//!
//! ⚠️ **O endereço não mudou:** o [`crate::skin_live`] re-exporta as quatro coisas públicas daqui,
//! porque os consumidores escrevem `skin_live::…` e reescrevê-los trocaria um corte por um mapa de
//! excepções (HOWTO §1.2/§1.4). *O ficheiro é uma unidade de manutenção; o caminho é um contrato.*

use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton::Xform;
use ph2d_skeleton_ecs::Bone;

use crate::skin_live::world_of;

/// **As duas alças de curvatura e as duas extremidades de um osso**, em MUNDO.
///
/// ⚠️⚠️ **Existe um alias GÉMEO na `ph2d_skeleton_render`, e as duas crates NÃO se conhecem —
/// de propósito.** O desenho do osso nunca importou a lei (ele é `ph2d-vector` + tokens, e é isso
/// que o deixa servir qualquer mídia), e criar a aresta só para partilhar um nome trocaria uma
/// duplicação de **quatro palavras** por uma dependência entre duas famílias. *A forma é a mesma
/// estruturalmente, logo o valor atravessa sem conversão nenhuma.*
pub type BendHandles = ([[f64; 2]; 2], [[f64; 2]; 2]);

/// ⭐⭐⭐⭐ **O OSSO COMO ELE DE FACTO DOBRA — a porta ÚNICA que resolve o `Handles::Auto`.**
///
/// ⛔⛔ **Ninguém nesta árvore lê `Bone::spec()` directamente a partir daqui**, e a razão é a lição
/// que este módulo já pagou três vezes: *um controlo DESENHADO por um mapa e DEFORMADO por outro é
/// um controlo morto sob o dedo*. O corpo que se pinta, a alça que se agarra e a pele que se
/// deforma têm de ler o MESMO osso — e com o `Auto` o osso efectivo já não é o que está guardado.
///
/// # ⭐⭐⭐ As tangentes saem da transformação RELATIVA, nunca do mundo
///
/// O vizinho anterior é o PAI e o seguinte é o FILHO, e a pose de um filho em relação ao pai **é**
/// o `Transform` dele — não é preciso ir ao mundo e voltar. ⚠️ **É isso que torna o ponto neutro
/// EXACTO:** numa corrente recta o pai fica em `(−L_pai, 0)` no local deste osso (o inverso de uma
/// translação pura é exacto) e a ponta do filho em `(L + L_filho, 0)`, logo as duas tangentes são
/// `(1, 0)` **ao bit** e o [`ph2d_skeleton::bend::auto_bend`] devolve `Bend::STRAIGHT`. *Uma volta
/// pelo mundo teria deixado `y ≈ 1e-17`, e ligar o Auto num rig recto arquearia tudo um bocadinho.*
///
/// ⚠️ **Um osso com DOIS filhos-osso não tem «o seguinte»** — ali a corrente ramifica, e escolher um
/// deles seria um sorteio. Nesse caso (e na ponta de uma corrente) a tangente é a do próprio eixo,
/// que deixa aquele lado recto.
#[must_use]
pub fn effective_spec(sim: &SimWorld, e: Entity) -> Option<ph2d_skeleton::bend::BoneSpec> {
    let osso = sim.world().get::<Bone>(e)?;
    let spec = osso.spec();
    if osso.handles != ph2d_skeleton::bend::Handles::Auto {
        return Some(spec);
    }
    let eixo = [spec.length, 0.0];
    // A tangente da RAIZ: da raiz do osso ANTERIOR até à ponta deste. Sem anterior, o próprio eixo.
    let t_raiz = pai_osso(sim, e)
        .and_then(|_| local_de(sim, e).inverse())
        .map_or(eixo, |inv| {
            let pai_raiz = inv.apply([0.0, 0.0]);
            [eixo[0] - pai_raiz[0], eixo[1] - pai_raiz[1]]
        });
    // A tangente da PONTA: da raiz deste osso até à ponta de QUEM MANDA. Sem ninguém, o próprio eixo.
    let t_ponta = manda_na_ponta(sim, e, osso.curve_tip).map_or(eixo, |(f, comp)| {
        let x = local_de(sim, f);
        x.apply([comp, 0.0])
    });
    Some(ph2d_skeleton::bend::BoneSpec {
        curve: ph2d_skeleton::bend::auto_bend(versor(t_raiz), versor(t_ponta)),
        ..spec
    })
}

/// O afim LOCAL de uma entidade — a pose dela em relação ao pai.
fn local_de(sim: &SimWorld, e: Entity) -> Xform {
    ph2d_vec_entities::transform::xform_of_transform(
        sim.world()
            .get::<ph2d_ecs::Transform>(e)
            .copied()
            .unwrap_or_default(),
    )
}

/// O versor de `v`; `(1, 0)` quando ele é degenerado.
///
/// ⚠️ **Sobre um vector já no eixo (`y == 0`, `x > 0`) ele devolve `(1, 0)` AO BIT** — `hypot(x, 0)`
/// é `|x|` exacto e `x / x` é `1.0` exacto. É essa propriedade que faz o ponto neutro do `Auto`
/// não ser uma tolerância.
fn versor(v: [f64; 2]) -> [f64; 2] {
    let n = v[0].hypot(v[1]);
    if n > 0.0 {
        [v[0] / n, v[1] / n]
    } else {
        [1.0, 0.0]
    }
}

/// O pai deste osso, se o pai for um osso.
fn pai_osso(sim: &SimWorld, e: Entity) -> Option<Entity> {
    let p = sim.world().get::<ph2d_ecs::ChildOf>(e)?.parent();
    sim.world().get::<Bone>(p).map(|_| p)
}

/// ⭐⭐⭐ **QUEM MANDA NA PONTA, e com que comprimento** — a resolução do
/// [`ph2d_skeleton_ecs::CurveTip`], e a única porta que a responde.
///
/// - [`CurveTip::Chain`]: o ÚNICO filho-osso; com zero ou mais de um, **ninguém** (a lei de sempre,
///   e a razão é que ali a corrente ramifica — escolher seria um sorteio que muda com a ordem de
///   varredura);
/// - [`CurveTip::Straight`]: **ninguém**, mesmo havendo um filho só (o *«modo actual»* que o dono
///   mandou manter como escolha);
/// - [`CurveTip::Bone`]: o filho com aquele [`ph2d_ecs::StableId`] — e **só** se ele for mesmo
///   filho-osso deste osso. *Uma referência que não se resolve fica recta; ela não inventa um
///   sorteio.*
fn manda_na_ponta(
    sim: &SimWorld,
    e: Entity,
    tip: ph2d_skeleton_ecs::CurveTip,
) -> Option<(Entity, f64)> {
    use ph2d_skeleton_ecs::CurveTip;
    if tip == CurveTip::Straight {
        return None;
    }
    let mut achado: Option<(Entity, f64)> = None;
    for er in sim.world().iter_entities() {
        let filho_de_e = er
            .get::<ph2d_ecs::ChildOf>()
            .is_some_and(|c| c.parent() == e);
        if !filho_de_e {
            continue;
        }
        let Some(b) = er.get::<Bone>() else {
            continue;
        };
        match tip {
            // ⭐ O ESCOLHIDO: o primeiro (e único) filho com este id — a comparação é entre os
            // FILHOS, nunca uma busca global, logo um osso de outro esqueleto não pode mandar aqui.
            CurveTip::Bone(id) => {
                if er.get::<ph2d_ecs::StableId>() == Some(&id) {
                    return Some((er.id(), b.length));
                }
            }
            // A CORRENTE: um filho manda; dois ou mais não têm desempate.
            CurveTip::Chain => {
                if achado.is_some() {
                    return None;
                }
                achado = Some((er.id(), b.length));
            }
            CurveTip::Straight => unreachable!("a saída de cima já devolveu"),
        }
    }
    // ⚠️ Com `Bone(id)` chegar aqui é *o escolhido já não é meu filho* ⇒ recto, e não o primeiro
    // que aparecer.
    achado.filter(|_| tip == CurveTip::Chain)
}

/// ⭐⭐⭐ **AS DUAS ALÇAS DE CURVATURA de um osso, em MUNDO**, mais as duas extremidades dele — a
/// porta ÚNICA do desenho e do dedo (`ph2d_skeleton_render::draw_bend` e o `bone_pick`).
///
/// ⛔⛔ **`None` quando a curvatura é INERTE**, isto é, quando o osso produz um sub-osso só
/// (`segments <= 1`). Pintar a alça ali prometeria um verbo que o arrasto não executa — a espécie
/// de controlo morto que este módulo acabou de pagar no botão `Smooth`. *E a pergunta é feita pela
/// porta do produto ([`ph2d_skeleton::bend::segments_of`]), nunca por um `> 1` escrito aqui.*
///
/// ⚠️ **As extremidades saem da POLILINHA e não do `spec`**: num osso já arqueado a ponta desenhada
/// é o último nó da curva, e é dela que a haste da alça tem de sair para o desenho fechar.
#[must_use]
pub fn bend_handles(sim: &SimWorld, bits: u64) -> Option<BendHandles> {
    let e = Entity::try_from_bits(bits)?;
    let spec = effective_spec(sim, e)?;
    if ph2d_skeleton::bend::segments_of(spec.segments) <= 1 {
        return None;
    }
    // ⛔⛔ **EM `Auto` NÃO HÁ ALÇA PARA AGARRAR, e é isso que a torna honesta:** as duas são
    // DERIVADAS da corrente, então arrastar uma seria escrever num valor que o quadro seguinte
    // recalcula — *o artista veria o gizmo voltar debaixo do dedo*, que é o defeito que a âncora
    // de IK deste módulo já pagou. ⇒ ali elas não se pintam nem se pegam, e o painel esconde os
    // quatro números pela mesma razão.
    if sim.world().get::<Bone>(e)?.handles == ph2d_skeleton::bend::Handles::Auto {
        return None;
    }
    let x = world_of(sim, e);
    let [inn, out] = ph2d_skeleton::bend::handles(spec.length, spec.curve);
    let pts = ph2d_skeleton::bend::polyline(spec);
    let (a, b) = (
        *pts.first().expect("a polilinha tem ao menos dois nos"),
        *pts.last().expect("a polilinha tem ao menos dois nos"),
    );
    Some(([x.apply(inn), x.apply(out)], [x.apply(a), x.apply(b)]))
}

/// ⭐⭐⭐ **ESCREVE A CURVATURA a partir de um ponto de MUNDO** — o inverso do [`bend_handles`], e o
/// verbo do arrasto.
///
/// `ponta = false` move a alça da raiz, `true` a da ponta. Devolve `false` quando não há onde
/// escrever (não é osso, é rígido, ou a pose é singular).
///
/// ⚠️ **A conversão mundo → local passa pelo INVERSO da mesma pose que o desenho usa**, e não por
/// uma cadeia escrita à mão: um osso filho herda a pose do pai, e reconstruí-la aqui poria a alça
/// onde ela não está desenhada — o defeito que o `sprite_image_to_screen_affine` deste repo já
/// pagou por escrito.
pub fn set_bend_handle(sim: &mut SimWorld, bits: u64, world: [f64; 2], ponta: bool) -> bool {
    let Some(e) = Entity::try_from_bits(bits) else {
        return false;
    };
    let Some(osso) = sim.world().get::<Bone>(e).copied() else {
        return false;
    };
    let spec = osso.spec();
    // ⚠️ As MESMAS duas recusas do [`bend_handles`], e pela mesma razão — escrever onde não há alça
    // pintada seria o gesto a mandar num valor que ninguém vê.
    if ph2d_skeleton::bend::segments_of(spec.segments) <= 1
        || osso.handles == ph2d_skeleton::bend::Handles::Auto
    {
        return false;
    }
    let Some(inv) = world_of(sim, e).inverse() else {
        return false;
    };
    let Some(v) = ph2d_skeleton::bend::bend_from_handle(spec.length, inv.apply(world), ponta)
    else {
        return false;
    };
    let Some(mut osso) = sim.world_mut().get_mut::<Bone>(e) else {
        return false;
    };
    // ⛔ **Sem tecto, e é §0.0:** não há recurso nenhum a limitar quanto um osso arqueia — o que
    // limita é o olho do artista. O ponto NEUTRO, esse, é alcançável (largar a alça no terço).
    if ponta {
        osso.curve.out = v;
    } else {
        osso.curve.inn = v;
    }
    true
}
