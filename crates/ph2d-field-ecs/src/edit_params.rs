//! ⭐ **AS PERGUNTAS E OS NÚMEROS de um nó** — o que a Hierarquia enumera e o que o painel edita.
//!
//! ⚠️ Nenhuma regra é inventada aqui: o raio muda por [`ph2d_field::set_shape_radius`] e o teto sai
//! de [`ph2d_field::round_limit`] / [`ph2d_field::characteristic_size`] — as **mesmas** funções que
//! a validação do documento cozido usa. Um painel que calculasse o próprio teto ofereceria valores
//! que a peça recusa, e o artista veria o controle parar sem explicação.

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::world::World;
use ph2d_field::{
    Bound, Dim, NodeShape, Param, Span, Unary, Xform, characteristic_size, round_limit,
};

use crate::{FieldMods, FieldNode, FieldPose};

/// ⭐ **PORQUE É QUE UM NÚMERO DO MATERIAL NÃO CHEGA AO PIXEL** — ver o cabeçalho do módulo.
///
/// ⚠️ Ele saiu deste ficheiro em 18/09 ao ganhar a **razão** e a **partição**: a lei passou a ser
/// uma resposta que outros querem fazer directamente, e este ficheiro passou dos `700`.
#[path = "edit_params_inerte.rs"]
mod inerte;

/// **A árvore em pré-ordem**, com a profundidade de cada nó — a mesma ordem e o mesmo aninhamento
/// que a Hierarquia mostra.
///
/// ⭐ É a ordem certa para o painel: uma lista que discordasse da Hierarquia obrigaria o artista a
/// manter dois mapas na cabeça da mesma peça.
#[must_use]
pub fn walk(world: &World, root: Entity) -> Vec<(Entity, u8)> {
    let mut out = Vec::new();
    let mut stack = vec![(root, 0u8)];
    while let Some((e, depth)) = stack.pop() {
        if world.get::<FieldNode>(e).is_none() {
            continue;
        }
        out.push((e, depth));
        if let Some(children) = world.get::<Children>(e) {
            // Invertido para o `pop` sair na ordem de `Children`.
            for c in children
                .iter()
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
            {
                stack.push((c, depth.saturating_add(1)));
            }
        }
    }
    out
}

/// O raio editável deste nó, ou `None` quando ele não tem nenhum.
#[must_use]
pub fn radius_of(world: &World, entity: Entity) -> Option<f32> {
    world.get::<FieldNode>(entity)?.shape.radius()
}

/// Até onde esse raio pode ir, e **de que natureza é o limite**.
///
/// - Numa **primitiva** é uma parede ([`Bound::Hard`]): acima dela a forma deixa de existir e
///   o campo deixa de ser uma distância.
/// - Numa **operação** não há limite de validade nenhum ([`Bound::Soft`]) — o campo continua
///   correto com qualquer raio. O que existe é *escala*: um filete maior do que a menor peça que ele
///   junta engole-a. O número vem daí.
#[must_use]
pub fn radius_bound(world: &World, entity: Entity) -> Option<Bound> {
    match &world.get::<FieldNode>(entity)?.shape {
        NodeShape::Leaf(p) => round_limit(p).map(Bound::Hard),
        NodeShape::Combine(_) => Some(Bound::Soft(subtree_scale(world, entity))),
        // Uma escultura não tem raio autorado: a aresta dela é a malha.
        NodeShape::Sampled { .. } => None,
    }
}

/// ⭐⭐⭐ **A FAIXA DE UMA LINHA DE JUNTA fecha-se pela PEÇA, e não pela vista** (report do Enio,
/// 2026-09-09: *«os sliders das joints vão de 0 a 16 quando só precisa de 0 a 1»*).
///
/// ⚠️ **[`Span::Positive`] entrega o tecto ao enquadramento**, que é o da **cena inteira** — numa
/// fileira de seis peças ele abre `0..16` para um número cujo valor útil vive abaixo de `0,1`, e
/// todo o curso do dedo cabe num pixel. Isso está certo para uma largura (que pode crescer até ao
/// quadro) e errado para um raio de junta, que é **local**.
///
/// ⭐ **O número já existia e não era usado:** a [`radius_bound`] devolve `Bound::Soft(escala)` para
/// uma operação desde a W10, e o comentário da linha do painel dizia por extenso que trocá-lo *«é
/// número do Enio, com a peça à frente»*. ⇒ *uma nota que espera um veredito precisa de ser
/// perguntada; esta esperou até ele tropeçar nela.*
///
/// ⚠️ **Só a [`Span::Positive`] é trocada.** O desequilíbrio de um chanfro é uma **razão**, não um
/// comprimento, e a escala da peça não o limita — ele traz a faixa dele de casa
/// ([`ph2d_field::Blend::second`]).
/// ⭐⭐ **O CURSO DO SEGUNDO NÚMERO de uma junta** — e ele nem sempre é um comprimento.
///
/// A meia-largura de um friso é uma medida da peça e fecha-se pela [`subtree_scale`]. O
/// **desequilíbrio** de um chanfro é uma **razão**: ele multiplica o recuo do lado longo, e o que o
/// limita é o ponto em que esse recuo alcança a peça — `escala / raio`. ⇒ *o mesmo raciocínio, na
/// unidade certa de cada um.*
///
/// ⚠️ **Com o raio a zero não há razão nenhuma que signifique alguma coisa**, e o piso devolve a
/// escala crua em vez de infinito: um slider de curso infinito é o defeito do report noutra escala.
fn curso_do_segundo(blend: ph2d_field::Blend, escala: f32) -> f32 {
    match blend {
        // ⚠️ **O `Chamfer` entra aqui junto com o `Bevel`**, e esquecê-lo era o defeito: a linha do
        // desequilíbrio nasce sobre um chanfro SIMÉTRICO (com `1,0` dentro), e a promoção a `Bevel`
        // só acontece quando alguém lhe escreve. *A faixa tem de estar certa antes do primeiro
        // arrasto, não depois dele.*
        ph2d_field::Blend::Bevel { radius, .. } | ph2d_field::Blend::Chamfer { radius }
            if radius > 1.0e-6 =>
        {
            escala / radius
        }
        _ => escala,
    }
}

fn fecha_pela_peca(d: Dim, escala: f32) -> Dim {
    if d.span == Span::Positive {
        Dim {
            span: Span::SoftFromZero(escala),
            ..d
        }
    } else {
        d
    }
}

/// A menor peça sob um nó, **com a escala da cadeia acumulada**.
///
/// ⚠️ A escala acumula de propósito: um cilindro de 0,1 dentro de um grupo escalado 3× mede 0,3 na
/// peça, e é esse o número que dá sentido a um raio de mistura. Usar só a escala do próprio nó
/// (como a versão de arena fazia, onde não havia cadeia) subestimaria a peça em cada nível de
/// agrupamento.
fn subtree_scale(world: &World, root: Entity) -> f32 {
    let mut best = f32::INFINITY;
    let mut stack = vec![(root, 1.0f32)];
    while let Some((e, acc)) = stack.pop() {
        let Some(node) = world.get::<FieldNode>(e) else {
            continue;
        };
        let acc = acc * world.get::<FieldPose>(e).map_or(1.0, |p| p.xform.scale);
        match &node.shape {
            NodeShape::Leaf(p) => best = best.min(characteristic_size(p) * acc),
            // ⚠️ A escala característica de uma escultura é a caixa dela, e a caixa vive no campo
            // amostrado, que o mundo não conhece. Não contribuir é a resposta certa quando há outra
            // peça a dar a escala — e o item aberto quando ela for a única.
            NodeShape::Sampled { .. } => {}
            NodeShape::Combine(_) => {
                if let Some(children) = world.get::<Children>(e) {
                    for c in children.iter().copied().collect::<Vec<_>>() {
                        stack.push((c, acc));
                    }
                }
            }
        }
    }
    if best.is_finite() && best > 0.0 {
        best
    } else {
        1.0
    }
}

/// ⭐⭐⭐ **O que CONTÉM a subárvore, eixo a eixo** — a grandeza OPOSTA à [`subtree_scale`], e a que
/// o plano de um espelho precisa.
///
/// ⚠️ **As duas respondem a perguntas opostas, e emprestar uma à outra é um defeito medido:** a
/// escala é o que é *fino* na peça (a espessura de uma casca nasce dela, e por isso ela é o
/// **mínimo**); o alcance é o que a *contém* (um plano posto lá fora nunca a corta, e por isso ele
/// é o **máximo**). Com a escala no lugar do alcance, um espelho posto numa peça alongada nasceria
/// com o plano **dentro** dela — visível, mas não o gesto que o botão promete.
///
/// ⚠️ **Por EIXO, e não um raio**: o espelho nomeia o eixo dele
/// ([`ph2d_field::UnaryKind::Mirror`] e irmãos), então o plano pode nascer exactamente na face da
/// peça naquele eixo — que é o «as duas encostadas» do [`ph2d_field::Unary::Array`]. Um raio único
/// poria o gémeo longe do corpo numa peça achatada.
///
/// ⚠️ **A pose do PRÓPRIO nó não conta** — a pilha de modificadores corre no espaço local dele, e a
/// pose é aplicada depois. Só as poses dos FILHOS entram.
fn subtree_reach(world: &World, root: Entity) -> [f32; 3] {
    let mut best = [0.0f32; 3];
    let mut stack = vec![(root, 1.0f32, [0.0f32; 3])];
    while let Some((e, acc, off)) = stack.pop() {
        let Some(node) = world.get::<FieldNode>(e) else {
            continue;
        };
        match &node.shape {
            NodeShape::Leaf(p) => {
                let h = ph2d_field::bounding_half_extents(p);
                for i in 0..3 {
                    best[i] = best[i].max(off[i].abs() + h[i] * acc);
                }
            }
            // A caixa de uma escultura vive no campo amostrado, que o mundo não conhece — a mesma
            // ausência declarada que a [`subtree_scale`] já nomeia.
            NodeShape::Sampled { .. } => {}
            NodeShape::Combine(_) => {
                if let Some(children) = world.get::<Children>(e) {
                    for c in children.iter().copied().collect::<Vec<_>>() {
                        let (s, t) = world
                            .get::<FieldPose>(c)
                            .map_or((1.0, [0.0; 3]), |p| (p.xform.scale, p.xform.translation));
                        let mut d = off;
                        for i in 0..3 {
                            d[i] += t[i] * acc;
                        }
                        stack.push((c, acc * s, d));
                    }
                }
            }
        }
    }
    best.map(|v| if v > 0.0 { v } else { 1.0 })
}
// ⭐ **As chaves vivem com quem LÊ, e o irmão que escreve importa-as.** Elas são o
// VOCABULÁRIO do painel: o `params_of` constrói as linhas por elas e o
// `edit_params_write::set_param` resolve-as de volta. Duplicá-las seria a segunda
// verdade clássica — e o sintoma seria uma linha que se pinta e não se escreve.
/// As chaves i18n dos três eixos da posição.
pub(super) const POS_KEYS: [&str; 3] = ["field.dim.pos_x", "field.dim.pos_y", "field.dim.pos_z"];

/// ⭐⭐⭐ **As chaves i18n dos números do MATERIAL**, na ordem de [`ph2d_field::Param::Material`].
///
/// ⚠️ **A contagem é [`ph2d_field::MATERIAL_FIELDS`]**, e há gate: uma tabela mais curta do que a
/// lei daria um `params_of` a entrar em pânico no índice, e uma mais longa pintaria uma linha que a
/// escrita recusa.
pub(super) const MATERIAL_KEYS: [&str; ph2d_field::MATERIAL_FIELDS as usize] = [
    "field.dim.base_weight",
    "field.dim.base_r",
    "field.dim.base_g",
    "field.dim.base_b",
    "field.dim.base_diffuse_roughness",
    "field.dim.metalness",
    "field.dim.specular_weight",
    "field.dim.specular_r",
    "field.dim.specular_g",
    "field.dim.specular_b",
    "field.dim.roughness",
    "field.dim.specular_ior",
    "field.dim.coat",
    "field.dim.coat_r",
    "field.dim.coat_g",
    "field.dim.coat_b",
    "field.dim.coat_roughness",
    "field.dim.coat_ior",
    "field.dim.coat_darkening",
    "field.dim.emission",
    "field.dim.emission_r",
    "field.dim.emission_g",
    "field.dim.emission_b",
    "field.dim.subsurface_weight",
    "field.dim.subsurface_r",
    "field.dim.subsurface_g",
    "field.dim.subsurface_b",
    "field.dim.subsurface_radius",
    "field.dim.subsurface_scale_r",
    "field.dim.subsurface_scale_g",
    "field.dim.subsurface_scale_b",
    "field.dim.subsurface_anisotropy",
    "field.dim.thin_walled",
];

/// As chaves i18n das linhas de uma luz — **a mesma ordem do [`crate::FieldLight::get`]**.
pub(super) const LIGHT_KEYS: [&str; ph2d_field::LIGHT_FIELDS as usize] = [
    "field.dim.light_intensity",
    "field.dim.light_color",
    "field.dim.light_color_g",
    "field.dim.light_color_b",
];

/// ⭐⭐⭐ **AS LINHAS DE UMA LUZ** — a pose, a força e a cor.
///
/// # ⛔ O que ela NÃO oferece, e porquê
///
/// **A rotação.** Um ponto não tem orientação: os três ângulos seriam três sliders que não movem um
/// pixel, que é o controlo morto que a W34 proíbe por escrito. ⚠️ E a **escala** também não — uma
/// lâmpada não tem tamanho nesta lei (o raio dela é o piso da singularidade, e é uma const do
/// renderizador, não um knob).
fn light_params(world: &World, entity: Entity, l: crate::FieldLight) -> Vec<(Param, Dim)> {
    let pose = world
        .get::<FieldPose>(entity)
        .map_or(Xform::IDENTITY, |p| p.xform);
    (0..3u8)
        .map(|k| {
            (
                Param::Pos(k),
                Dim {
                    key: POS_KEYS[k as usize],
                    value: pose.translation[k as usize],
                    span: Span::Free,
                },
            )
        })
        .chain((0..ph2d_field::LIGHT_FIELDS).filter_map(|k| {
            Some((
                Param::Light(k),
                Dim {
                    key: LIGHT_KEYS[k as usize],
                    value: l.get(k)?,
                    // ⚠️ **Tecto MOLE, como o material**: a faixa do slider é `0..1` e o campo
                    // aceita mais — uma luz de força `4` é uma luz a quatro unidades de distância,
                    // e recusá-la seria inventar um limite que a física não tem.
                    span: Span::SoftFromZero(1.0),
                },
            ))
        }))
        .collect()
}

/// ⭐⭐⭐ **A FAIXA de um número do material** — e **quinze dos dezasseis são a mesma**.
///
/// ⭐ **Do zero a um, e as duas pontas são do MODELO**: uma rugosidade acima de `1` não é mais
/// áspera, um metal a `2` não é mais metal, e um peso de verniz também não. ⚠️ `SoftFromZero` e não
/// `Wall`: o campo numérico continua **sem tecto** (uma cor base em HDR é uma afirmação legítima
/// sobre a peça, e um brilho próprio ainda mais), e o que este número fecha é o **curso do slider**.
///
/// # ⚠️⚠️ A excepção é o IOR do verniz, e a faixa dele é FÍSICA
///
/// Ele é a única grandeza deste material que **não é uma fracção**, e a régua que respondeu pelas
/// outras não responde por ele: varrido até `20`, o quadro **nunca pára de se mexer**
/// (`docs/Render3d/05` §21), logo *«onde deixa de ser observável»* não tem resposta aqui.
///
/// ⇒ o recurso é o **material de que uma película transparente pode ser feita**: o piso é `1`
/// (a luz não atravessa nada mais depressa do que o vácuo) e o tecto é `2,5`, logo acima do
/// diamante (`2,42`), que é o mais alto dos transparentes conhecidos.
///
/// ⛔ **`Range` e não `SoftFromZero`**, isto é: as duas pontas são **duras**, e digitar fora delas
/// clampa. *Um IOR abaixo de `1` não é um valor raro — é um valor que o modelo não admite*, e um
/// slider que começasse em `0` ofereceria metade do curso a sítios inexistentes.
#[must_use]
fn material_span(field: u8) -> Span {
    match field {
        11 | 17 => Span::Range { min: 1.0, max: 2.5 },
        // ⭐ A FASE do espalhamento vive em `[-1, 1]` — para trás e para a frente. `Hard`, porque
        // as duas pontas são do modelo e não da vista.
        31 => Span::Range {
            min: -1.0,
            max: 1.0,
        },
        // ⚠️ **A PAREDE FINA é um booleano guardado como número** — uma escolha de dois, e é a
        // `Span::Choice` que faz o arrasto ser inteiro em vez de contínuo. ⏳ Que ela se pinte como
        // uma CAIXA e não como uma escolha fica nomeado (`docs/Render3d/10`).
        32 => Span::Choice(&["field.dim.thin_walled_no", "field.dim.thin_walled_yes"]),
        _ => Span::SoftFromZero(1.0),
    }
}

/// As chaves i18n dos três ângulos.
pub(super) const ROT_KEYS: [&str; 3] = ["field.dim.rot_x", "field.dim.rot_y", "field.dim.rot_z"];

/// ⭐ **A faixa canónica de cada posição do trio, em graus** — e ela **não é a mesma nas três**.
///
/// ⚠️ Não é uma escolha de UI: é o alcance da própria representação. Num XYZ Euler o ângulo do
/// **meio** vive em `[−90°, 90°]` e os de fora em `(−180°, 180°]`, e é por isso que a linha do meio
/// tem metade do curso. Dar-lhe 180 foi o defeito da primeira versão: o slider oferecia sítios que a
/// leitura seguinte **renomeava**, e num arrasto isso vira um ciclo de dois (ver
/// [`ph2d_field::xform::set_rotation_degree`], que é onde a lei e a medição estão).
///
/// ⚠️ **Prender o do meio não perde orientação nenhuma** — toda orientação tem um trio canónico com
/// `|β| ≤ 90°`. Perde-se o *nome*: «Y = 120» escreve-se `X = 180 · Y = 60 · Z = 180`.
pub(super) const ROT_SPAN_DEG: [f32; 3] = [180.0, 90.0, 180.0];

/// ⭐ **Todos os números autorados de um nó**, na ordem em que o painel os mostra.
///
/// Posição · rotação · escala (só onde ela não compete com nada) · dimensões da forma.
///
/// ⚠️ **A ordem é a do Inspector de objeto que todo modelador tem** (posição, rotação, escala, e só
/// depois o que a forma mede). Ela não é decorativa: os três primeiros trios existem em **todo** nó,
/// então uma peça inteira lê-se com o olho no mesmo sítio de linha para linha.
///
/// ⚠️ **A escala aparece só numa OPERAÇÃO.** Numa folha, o tamanho visível são as dimensões — e
/// mostrar as duas coisas daria ao artista dois controles para a mesma coisa, sem forma de saber
/// qual o próximo gesto mexe. Ver [`ph2d_field::scale_primitive`], que é o outro lado da mesma
/// decisão.
#[must_use]
pub fn params_of(world: &World, entity: Entity) -> Vec<(Param, Dim)> {
    // ⭐⭐⭐ **UMA LUZ NÃO É UM NÓ** (ordem do dono, 14/09) — ela tem pose e lâmpada, e mais nada. A
    // pergunta vem ANTES do `FieldNode` porque uma luz não o tem: sem esta linha o painel de uma
    // luz escolhida sairia **vazio**, que é o que ele fazia até esta wave.
    if let Some(l) = world.get::<crate::FieldLight>(entity) {
        return light_params(world, entity, *l);
    }
    let Some(node) = world.get::<FieldNode>(entity) else {
        return Vec::new();
    };
    let pose = world
        .get::<FieldPose>(entity)
        .map_or(Xform::IDENTITY, |p| p.xform);
    let degrees = ph2d_field::xform::rotation_degrees(pose);
    let mut out: Vec<(Param, Dim)> = (0..3u8)
        .map(|k| {
            (
                Param::Pos(k),
                Dim {
                    key: POS_KEYS[k as usize],
                    value: pose.translation[k as usize],
                    // ⚠️ Uma posição não tem parede **nem piso**: a origem não é um canto do mundo.
                    // Quem fecha as duas pontas é a vista (ver `Span::Free`).
                    span: Span::Free,
                },
            )
        })
        .chain((0..3u8).map(|k| {
            (
                Param::Rot(k),
                Dim {
                    key: ROT_KEYS[k as usize],
                    value: degrees[k as usize],
                    // ⚠️ **A mesma porta que recusa a escrita** decide se há faixa: um eixo que a
                    // trava de cardan tirou do mapa não é um slider curto, é um facto sem controle.
                    //
                    // ⭐ **E ele DIZ porquê desde 18/09** (ver [`Span::Locked`]): esta trava era
                    // muda, e um eixo apagado sem razão à vista lê-se como o painel avariado — a
                    // razão nomeia o gesto que o destranca (mexer no ângulo do MEIO), que é a lei
                    // do `shape_palette::why_not`.
                    span: if ph2d_field::xform::rotation_axis_is_free(pose, k) {
                        Span::Turn(ROT_SPAN_DEG[k as usize])
                    } else {
                        Span::Locked("field.inert.gimbal_axis")
                    },
                },
            )
        }))
        .collect();
    match &node.shape {
        // Uma escultura tem pose e mais nada: o que ela é vive na malha.
        NodeShape::Sampled { .. } => {}
        NodeShape::Combine(op) => {
            out.push((
                Param::Scale,
                Dim {
                    key: "field.dim.scale",
                    value: pose.scale,
                    span: Span::Positive,
                },
            ));
            // A única dimensão de uma operação é o raio da mistura, e ela entra pela porta de
            // sempre — `Dim(0)`, que o `set_param` reencaminha.
            if let Some(v) = node.shape.radius() {
                let escala = subtree_scale(world, entity);
                out.push((
                    Param::Dim(0),
                    fecha_pela_peca(
                        Dim {
                            // ⭐ **"Joint" e não "Fillet" desde a W98**, e é o rótulo a apanhar o modelo:
                            // depois do verbo por forma, o raio de um grupo é o **raio de junção
                            // padrão** — o que as formas caladas usam. É a mesma grandeza que a linha
                            // [`Param::Joint`] de cada filho escreve, e duas palavras para uma grandeza
                            // é o que faz o artista pensar que são duas.
                            key: "field.dim.joint",
                            value: v,
                            // ⚠️ **Uma mistura não tem parede**: o campo continua a ser uma distância com
                            // qualquer raio ([`radius_bound`] devolve sempre `Soft` aqui).
                            //
                            // ⏸️ O `radius_bound` sabe um alcance mais **apertado** do que o da vista — a
                            // menor peça sob o nó, que é o raio a partir do qual a mistura a engole.
                            // Trocá-lo pelo da vista muda o **tato** do arrasto desta linha e de mais
                            // nenhuma, e isso é número do Enio, com a peça à frente.
                            span: Span::Positive,
                        },
                        escala,
                    ),
                ));
            }
            // ⭐⭐ **O SEGUNDO NÚMERO da mistura padrão deste grupo** (W145) — a meia-largura de um
            // sulco, o desequilíbrio de um chanfro. ⚠️ **Derivado do [`ph2d_field::Blend::second`]**,
            // que traz a chave e a faixa consigo: um carácter novo com dois números aparece aqui sem
            // uma linha de mudança, e um sem eles não oferece controle nenhum.
            if let Some(d) = op.blend().second() {
                let escala = subtree_scale(world, entity);
                out.push((
                    Param::Seam(0),
                    fecha_pela_peca(d, curso_do_segundo(op.blend(), escala)),
                ));
            }
        }
        NodeShape::Leaf(p) => out.extend(
            ph2d_field::dims(p)
                .into_iter()
                .enumerate()
                .map(|(i, d)| (Param::Dim(i as u16), d)),
        ),
    }
    // ⭐⭐⭐ **O RAIO DA JUNÇÃO desta forma** (W98) — logo depois do que ela mede, porque é sobre o
    // **encontro** dela com o resto e não sobre o que ela é.
    //
    // ⚠️ **A pergunta é feita ao PAPEL, e não à forma** ([`crate::verb_role`]): a base não se junta
    // a nada, e a raiz da peça também não. Oferecer a linha ali seria um controle que escreve num
    // verbo que ninguém lê — a affordance que mente, e a mesma lei da W34 que a fileira do painel
    // já honra.
    //
    // ⭐ **E ela aparece também para quem HERDA**, com o valor herdado: *"quero a boca deste furo
    // mais macia"* não pode exigir que o artista entenda o modelo do verbo primeiro. Escrever
    // materializa (ver [`Param::Joint`]), e o chip `Inherit` apaga-se à vista.
    if let Some(op) = crate::verb_role(world, entity).and_then(|r| r.op()) {
        let escala = subtree_scale(world, entity);
        out.push((
            Param::Joint,
            fecha_pela_peca(
                Dim {
                    key: "field.dim.joint",
                    value: op.blend().amount(),
                    // ⚠️ **Sem parede**: o campo continua a ser uma distância com qualquer raio
                    // de mistura. Quem fecha o **curso do slider** é a PEÇA — ver
                    // [`fecha_pela_peca`] e o report de 09/09.
                    span: Span::Positive,
                },
                escala,
            ),
        ));
        // ⭐⭐ **E o segundo número DELE** (W145), logo abaixo — ver [`Param::Seam`] para por que os
        // dois slots não podem ser um.
        if let Some(d) = op.blend().second() {
            out.push((
                Param::Seam(1),
                fecha_pela_peca(d, curso_do_segundo(op.blend(), escala)),
            ));
        }
    }
    // ⭐⭐⭐ **O MATERIAL, e só numa FOLHA** (`docs/Render3d/05`) — o que a forma mede à LUZ, depois
    // do que ela mede à régua.
    //
    // ⚠️ **A pergunta é feita à FORMA e não ao nó:** quem o traçado sabe nomear por pixel é a folha
    // (`ph2d_field_eval::owners`), então um material num grupo seria um valor que nenhum pixel
    // consegue ir buscar — a affordance que mente, a mesma lei que o raio de junção acima honra.
    //
    // ⚠️ **A ausência do componente É o material de omissão**, e é por isso que as linhas aparecem
    // na mesma: sem elas o artista teria de saber que precisa de «acrescentar um material» antes de
    // poder escolher uma cor. Escrever materializa (ver [`ph2d_field::Param::Material`]).
    if matches!(node.shape, NodeShape::Leaf(_)) {
        let m = world
            .get::<crate::FieldMaterial>(entity)
            .copied()
            .unwrap_or_default();
        // ⭐⭐⭐ **UM NÚMERO INERTE NÃO DESAPARECE — ELE FICA TRAVADO** (ordem do Enio, 2026-09-14:
        // *«os slideres que só aparecem sob uma condição específica não devem desaparecer, mas
        // apenas serem inativados, mas sempre visíveis»*).
        //
        // ⛔⛔ **A lei já estava escrita nesta casa, e eu apliquei a OUTRA.** O [`Span::Locked`] diz,
        // por extenso: *«é diferente de "não aparece" — o valor continua a ser um facto que o
        // artista precisa de ler, e esconder a linha faria o painel saltar de tamanho a cada
        // travessia»*. A W34 (*o painel oferece exactamente o que o gesto faz*) proíbe **pintar um
        // controlo** que não pode ser honrado; ela **não** manda apagar a linha. *Duas leis que se
        // leem parecidas, e a diferença entre elas é o painel a saltar debaixo do dedo.*
        //
        // ⭐ **QUAIS posições, e porquê cada uma, vive no [`inerte::razao_inerte`]** — ela
        // saiu daqui em 18/09 ao ganhar a RAZÃO e a PARTIÇÃO, e o corte é por responsabilidade: a
        // lei passou a ser uma resposta que o censo quer fazer **sem montar um painel**.
        out.extend((0..ph2d_field::MATERIAL_FIELDS).filter_map(|k| {
            Some((
                Param::Material(k),
                Dim {
                    key: MATERIAL_KEYS[k as usize],
                    value: m.get(k)?,
                    span: match inerte::razao_inerte(&m, k) {
                        Some(razao) => Span::Locked(razao),
                        None => material_span(k),
                    },
                },
            ))
        }));
    }
    // ⭐⭐ **A RESOLUÇÃO do contorno vivo** (W55) — logo depois do que a forma mede, e antes do que
    // se fez a ela.
    //
    // ⚠️ **A pergunta é feita ao VÍNCULO, não à forma.** Um `Extrude` cujo desenho foi largado é uma
    // extrusão normal, com as mesmas dimensões e sem esta linha — e é o componente ausente que o
    // diz. Perguntar `matches!(shape, Extrude | Revolve)` ofereceria um controle que não tem onde
    // escrever.
    if let Some(src) = world.get::<crate::FieldProfileSource>(entity) {
        out.push((
            Param::Resolution,
            Dim {
                key: "field.dim.resolution",
                value: src.level as f32,
                // ⭐ **Uma CONTAGEM**: o passo do arrasto é 1, não há meio nível, e as duas pontas
                // são do documento — o piso é o joelho que a W54 mediu e o teto é o custo do
                // traçado assente (ver [`ph2d_field::MAX_PROFILE_RESOLUTION`]).
                span: Span::Count {
                    min: 1,
                    max: ph2d_field::MAX_PROFILE_RESOLUTION,
                },
            },
        ));
    }
    // ⭐ **Os modificadores vêm por ÚLTIMO**, e é a ordem em que eles correm: primeiro o que a forma
    // é, depois o que se fez a ela. Uma linha de casca acima da largura da caixa leria como se a
    // parede fosse uma propriedade da caixa, e ela é uma operação sobre o resultado.
    // ⚠️ **Um modificador pode ter VÁRIOS números** (uma matriz tem contagem e espaçamento) — e
    // pode não ter nenhum (o espelho). O `flat_map` é o que exprime as duas pontas sem um caso
    // especial para cada.
    out.extend(
        mods_of(world, entity)
            .into_iter()
            .enumerate()
            .flat_map(|(slot, m)| {
                m.dims().into_iter().enumerate().map(move |(field, d)| {
                    (
                        Param::Mod {
                            slot: slot as u16,
                            field: field as u8,
                        },
                        d,
                    )
                })
            }),
    );
    out
}

/// ⭐ **A pilha de modificadores deste nó** — vazia quando ele não tem nenhum.
#[must_use]
pub fn mods_of(world: &World, entity: Entity) -> Vec<Unary> {
    world
        .get::<FieldMods>(entity)
        .map(|m| m.stack.clone())
        .unwrap_or_default()
}

/// ⭐ **Acrescenta um modificador ao nó**, no ponto neutro da natureza dele.
///
/// ⚠️ **O tamanho de nascimento vem da PEÇA**, não de uma constante: uma casca é uma fração da
/// menor peça sob o nó ([`subtree_scale`]), porque só quem vê a peça sabe o que é fino nela. Um
/// número absoluto seria invisível numa peça grande e engoliria uma pequena — nos dois casos o
/// artista conclui que o botão não fez nada.
///
/// Devolve `false` sem escrever nada quando a entidade não é um nó, ou é uma **escultura** (ver
/// abaixo) — e é o `false` que deixa quem chamou dizê-lo ao artista.
pub fn add_mod(world: &mut World, entity: Entity, kind: ph2d_field::UnaryKind) -> bool {
    let Some(node) = world.get::<FieldNode>(entity) else {
        return false;
    };
    // ⭐ **Uma escultura NÃO aceita modificadores, e a recusa é aqui** (W25).
    //
    // ⚠️ **A regra já existia — no documento** ([`ph2d_field::FieldError::ModsOnSampled`]) — e
    // ninguém a consultava antes de escrever. O componente entrava no mundo, o cozimento do quadro
    // seguinte recusava o documento inteiro, e a peça **inteira** desaparecia da tela com a
    // Hierarquia intacta. *Uma invariante que só o validador conhece é uma invariante que a UI
    // descobre partindo-se.*
    if matches!(node.shape, NodeShape::Sampled { .. }) {
        return false;
    }
    let born = Unary::born(
        kind,
        subtree_scale(world, entity),
        subtree_reach(world, entity),
    );
    let mut e = world.entity_mut(entity);
    if let Some(mut m) = e.get_mut::<FieldMods>() {
        m.stack.push(born);
    } else {
        e.insert(FieldMods { stack: vec![born] });
    }
    true
}

/// ⭐ **Tira do nó o PRIMEIRO modificador daquela natureza**, e diz se tirou algum.
///
/// ⚠️ O primeiro, e não todos: a pilha é ordenada, e apagar em bloco tiraria um que o artista pôs
/// de propósito depois de outro. Devolve `false` quando não havia nenhum — é o que faz o botão ser
/// um interruptor honesto.
pub fn remove_mod(world: &mut World, entity: Entity, kind: ph2d_field::UnaryKind) -> bool {
    let Some(mut m) = world.get_mut::<FieldMods>(entity) else {
        return false;
    };
    let Some(i) = m.stack.iter().position(|u| u.kind() == kind) else {
        return false;
    };
    m.stack.remove(i);
    // ⚠️ **A pilha vazia sai do nó.** Um componente presente e vazio não muda a forma, mas muda os
    // BYTES — e o undo compara bytes: acrescentar e tirar um modificador deixaria a peça diferente
    // de si mesma, e o desfazer teria um passo a mais do que o artista fez.
    let empty = m.stack.is_empty();
    if empty {
        world.entity_mut(entity).remove::<FieldMods>();
    }
    true
}
