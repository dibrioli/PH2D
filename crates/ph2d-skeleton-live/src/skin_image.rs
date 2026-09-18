//! ⭐⭐⭐ **UMA IMAGEM OBEDECE AO ESQUELETO** — a segunda mídia (ordem do dono, 2026-09-09).
//!
//! Pesquisa e recusas medidas: [`docs/Skeleton/02_pesquisa_a_malha_sobre_a_imagem.md`]. O desenho de
//! hoje: [`docs/Skeleton/03_plano_a_pele_no_passe_de_sprites.md`].
//!
//! # As quatro perguntas, e onde cada uma é respondida
//!
//! ```text
//! onde a tinta acaba?      ph2d_poly2d::grid_mesh_of               (uma vez, ao PRENDER)
//! que peso cada ponto tem? ph2d_skin_weights::bounded_biharmonic   (uma vez, ao PRENDER — guardado)
//! onde o ponto vai parar?  ph2d_skeleton::Skin::point_with          (por quadro, com o peso guardado)
//! como se desenha isso?    ph2d_render::SpriteMesh                 (ESTE módulo põe-na, por quadro)
//! ```
//!
//! ⚠️ **Esta tabela envelheceu duas vezes, e a redacção de 2026-09-09 fica como contraste:** a malha
//! era o CONTORNO triangulado (`ph2d_poly2d::mesh_of`) até a grelha graduada da F6-b (2026-09-10), e
//! o peso era *«derivado por distância ao osso e não guardado»* até o padrão-ouro (2026-09-15),
//! que é resolvido uma vez sobre a arte e guardado com a malha
//! ([`crate::skinned_mesh::SkinnedMesh`]).
//!
//! ⭐⭐ **A metade difícil já estava feita no dia em que a mídia nasceu:** a
//! [`ph2d_skeleton_ecs::SkinBind`] já nascera agnóstica de mídia — o doc dela dizia, por escrito,
//! *«serve um `VecPath` hoje e uma malha raster amanhã sem uma variante nova nem um schema por
//! mídia»*.
//!
//! # ⭐⭐⭐ A imagem presa é a PRÓPRIA SPRITE, desenhada como MALHA (plano 03, W2, 2026-09-13)
//!
//! Ela passa pelo extract como qualquer sprite — rank, olho da Hierarquia, tinta, opacidade, mistura,
//! recorte — e [`attach_skin_meshes`] põe na instância dela o [`SpriteMesh`] posado. O passe de
//! sprites troca o quad pela malha, sem pipeline nova.
//!
//! ⛔⛔ **Até esse dia ela era uma CAMADA do Vello por cima do quadro** — um recorte e um afim por
//! triângulo —, e isso tinha cinco defeitos com uma causa só (fila F6-h): fora da ORDEM, desenhada
//! com o olho FECHADO, as propriedades da sprite IGNORADAS, COSTURAS de anti-aliasing entre peças
//! vizinhas (`1 − a·b`), e a régua da imagem a ler a âncora CRUA. *Não se remenda uma camada para
//! entrar numa ordem de que ela não faz parte.*
//!
//! ⛔ **A rota de UMA INSTÂNCIA DE QUAD POR CÉLULA foi medida e recusada:** a instância carrega um
//! basis 2×2, logo cada célula sairia **paralelogramo**, e duas células vizinhas de um warp real não
//! partilham aresta — a costura abre fenda. A malha é UMA instância com os vértices dela.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, PresentWorld, SimRef, SimWorld, With};
use ph2d_poly2d::Mesh2d;

use crate::skinned_mesh::SkinnedMesh;
use ph2d_render::nine_slice::SlicePatchMirror;
use ph2d_render::{RenderInstance, Sprite, SpriteMesh};
use ph2d_skeleton::Xform;

/// ⭐⭐⭐ **PIXEL DA IMAGEM → PONTO LOCAL DA SPRITE** — a lei que ata as duas réguas.
///
/// A sprite ocupa `size` metros LOCAIS com o centro no [`Sprite::resolve_anchor`] — o MESMO número
/// que o extract carimba em `RenderInstance.anchor` —, e a textura é lida com `v = 0` **em cima**.
/// ⇒ sem espelho, o pixel `(0, 0)` é o canto **superior esquerdo** do quad e o `(w, h)` o inferior
/// direito.
///
/// ⚠️⚠️ **O `y` VIRA.** Uma imagem tem o `y` a crescer para baixo e o mundo tem-no a crescer para
/// cima; esquecê-lo desenha o personagem **de cabeça para baixo**, e é o tipo de defeito que passa
/// por todo gate de geometria (a malha está certa, a deformação está certa, e a imagem está ao
/// contrário).
///
/// ⛔⛔ **A âncora é a RESOLVIDA, e não a crua** (o quinto defeito da F6-h): com *Centered* desligado
/// ou um *Offset*, o quad desenha-se deslocado do `anchor` cru, e uma régua que lesse o cru prendia a
/// malha onde a imagem não está — a pele deformava-se meio quad ao lado da arte.
///
/// ⚠️ **O espelho entra AQUI, na POSIÇÃO, e a UV não o repete.** O shader espelha a UV do quad
/// (`flip_x`/`flip_y`): no canto esquerdo de um quad espelhado lê-se a coluna direita da imagem. ⇒ o
/// pixel `x` de uma sprite espelhada mora do outro lado do centro, e a UV de cada vértice é a do
/// QUAD naquele ponto ([`SpriteMesh::uv_at`]) — o shader espelha-a e volta ao pixel `x`. Espelhar só
/// a UV desenharia a tinta fora da silhueta; espelhar as duas não espelharia nada.
///
/// `pixels_per_meter` é o do projecto, o mesmo que o extract passa ao `resolve_anchor` — dois valores
/// dariam uma âncora a cada um. `None` quando a imagem tem lado zero — ali não há régua.
#[must_use]
pub fn pixel_to_local(sprite: &Sprite, size_px: [u32; 2], pixels_per_meter: f32) -> Option<Xform> {
    rect_to_quad(
        sprite,
        [0.0, 0.0, f64::from(size_px[0]), f64::from(size_px[1])],
        sprite.resolve_anchor(pixels_per_meter),
        sprite.size,
    )
}

/// ⭐⭐⭐ **A RÉGUA GERAL: um RECTÂNGULO da malha esticado no QUAD de uma instância.**
///
/// `rect` é `[x0, y0, x1, y1]` em pixels da malha; `anchor`/`size` são os da INSTÂNCIA — que num
/// 9-slice é o quad do pedaço, e numa sprite comum é o quad dela.
///
/// ⭐ **A [`pixel_to_local`] é o caso particular** em que o rectângulo é a malha inteira e o quad é o
/// da sprite, e ela DELEGA aqui — *duas aritméticas para o mesmo mapa divergiriam no primeiro
/// ajuste, e o sintoma seria a tinta do pedaço do meio meio pixel ao lado da dos cantos*.
///
/// ⚠️⚠️ **O `y` vira e o espelho entra na POSIÇÃO** — as duas leis são as do doc da [`pixel_to_local`],
/// e valem igual para um pedaço: o shader espelha a UV do quad do PEDAÇO, e a posição tem de o
/// acompanhar.
///
/// `None` com um lado nulo — ali não há régua.
#[must_use]
pub fn rect_to_quad(
    sprite: &Sprite,
    rect: [f64; 4],
    anchor: [f32; 2],
    size: [f32; 2],
) -> Option<Xform> {
    let (w, h) = (rect[2] - rect[0], rect[3] - rect[1]);
    if !(w > 0.0 && h > 0.0) {
        return None;
    }
    let lado = |s: f32, espelhado: bool| {
        if espelhado {
            -f64::from(s)
        } else {
            f64::from(s)
        }
    };
    let (sx, sy) = (lado(size[0], sprite.flip_x), lado(size[1], sprite.flip_y));
    let (ax, ay) = (f64::from(anchor[0]), f64::from(anchor[1]));
    let (mx, my) = (sx / w, -sy / h);
    Some(Xform([
        mx,
        0.0,
        0.0,
        my,
        ax - sx / 2.0 - mx * rect[0],
        ay + sy / 2.0 - my * rect[1],
    ]))
}

/// ⭐ **DE QUE PIXELS A MALHA NASCE** — o irmão por responsabilidade (e pelo tecto de LOC).
///
/// ⚠️ O `pub use` é deliberado: os consumidores escrevem `skin_image::mesh_from_rgba`, e mudar o
/// endereço de uma porta por causa de um tecto de linhas seria o tecto a mandar na API.
#[path = "skin_image_tinta.rs"]
mod tinta;
pub use tinta::{cell_alpha, mesh_from_alpha, mesh_from_rgba};

/// ⭐⭐⭐ **AS ARTICULAÇÕES DESTE ESQUELETO, em PIXELS DA IMAGEM** — o que gradua a malha.
///
/// ⚠️ **Duas conversões, e a ordem importa:** mundo → local da sprite (o inverso da pose dela) →
/// pixel da imagem (o inverso da [`pixel_to_local`]). Trocá-las põe as articulações no sítio certo
/// de um espaço errado, e o adensamento cai onde não há dobra nenhuma — *um defeito que não estoura
/// e não se vê num gate de geometria, só numa malha que fica fina no sítio errado*.
///
/// ⚠️ **A ponta e a raiz de cada osso entram as DUAS.** Numa corrente contínua elas coincidem e o
/// duplicado é inofensivo (a marcha dos cortes usa o mais próximo); numa ponta de corrente, a
/// ponta é uma dobra a sério — é lá que a mão do personagem gira.
#[must_use]
pub fn joints_in_image(
    sim: &SimWorld,
    e: Entity,
    ossos: &[Entity],
    size_px: [u32; 2],
    pixels_per_meter: f32,
) -> Vec<[f64; 2]> {
    let Some(sprite) = sim.world().get::<Sprite>(e) else {
        return Vec::new();
    };
    let (Some(p2l), Some(mundo)) = (
        pixel_to_local(sprite, size_px, pixels_per_meter),
        ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(sim, e),
        )
        .inverse(),
    ) else {
        return Vec::new();
    };
    let Some(l2p) = p2l.inverse() else {
        return Vec::new();
    };
    let segs = crate::skin_live::bone_segments(sim);
    ossos
        .iter()
        .filter_map(|b| segs.iter().find(|(x, _, _)| *x == b.to_bits()))
        .flat_map(|&(_, a, t)| [a, t])
        .map(|p| l2p.apply(mundo.apply(p)))
        .collect()
}

/// ⭐⭐⭐ **OS PESOS DE PELE DESTA IMAGEM, PELO PADRÃO-OURO** — resolvidos uma vez, ao prender.
///
/// `eixos` são as extremidades de cada osso **no espaço da forma**, na MESMA ordem dos tendões
/// (ver `skin_live::tendons_and_axes` — é uma lista só, de propósito). Devolve a tabela achatada
/// `pesos[v * ossos + j]`, ou **vazia** quando o solver não tem resposta.
///
/// ⚠️⚠️ **A régua `local → pixel` entra AQUI, e a direcção é a que se lê ao contrário:** os eixos
/// chegam em unidades da forma e a malha vive em **pixels da imagem**, então o que se aplica é a
/// INVERSA da [`pixel_to_local`]. *Resolver no espaço da forma daria um resultado que depende da
/// escala da sprite* — a mesma arte importada duas vezes com tamanhos diferentes prenderia
/// conjuntos de vértices diferentes, e o artista veria dois rigs a partir de um desenho.
///
/// ⛔ **Vazia é uma resposta honesta**, e o consumidor sabe lê-la ([`ph2d_skeleton::Skin::point`]
/// continua a derivar pela lei euclidiana). Ela acontece quando a sprite não tem régua (lado zero)
/// ou quando a malha não admite laplaciano — nos dois casos *não sei* é melhor que uma tabela
/// inventada.
///
/// ⚠️ **O custo é do gesto de PRENDER e está medido** (bancada da `ph2d-skin-weights`, release):
/// `720` triângulos ⇒ `16 ms` · `1 584` ⇒ `135 ms` · `2 880` ⇒ `454 ms` · `5 120` ⇒ `1,5 s`. A
/// malha de uma sprite típica cai na primeira linha; o `PH2D_BONE_LOG=1` imprime o relógio e a
/// contagem para quem quiser medir a dele.
#[must_use]
pub fn weights_for_mesh(
    sim: &SimWorld,
    e: Entity,
    mesh: &Mesh2d,
    eixos: &[crate::skin_live::OssoPreso],
    pixels_per_meter: f32,
) -> Vec<f64> {
    if eixos.is_empty() {
        return Vec::new();
    }
    let Some(l2p) = sim
        .world()
        .get::<Sprite>(e)
        .and_then(|s| pixel_to_local(s, mesh.size, pixels_per_meter))
        .and_then(|p2l| p2l.inverse())
    else {
        return Vec::new();
    };
    let handles: Vec<ph2d_skin_weights::Handle> = eixos
        .iter()
        .map(|o| ph2d_skin_weights::Handle {
            a: l2p.apply(o.a),
            b: l2p.apply(o.b),
        })
        .collect();
    let comeco = std::time::Instant::now();
    let Some(w) = ph2d_skin_weights::bounded_biharmonic(
        mesh,
        &handles,
        ph2d_skin_weights::Options::default(),
    ) else {
        eprintln!(
            "[bone] os pesos do padrao-ouro NAO resolveram ({} vertices, {} ossos) — esta imagem \
             cai na lei derivada",
            mesh.rest.len(),
            handles.len()
        );
        return Vec::new();
    };
    if std::env::var_os("PH2D_BONE_LOG").is_some() {
        let r = w.report;
        eprintln!(
            "[bone] pesos (BBW): {} vertices x {} ossos, {} presos, {} rondas, residuo {:.2e}, \
             soma_pior {:.2e}, fora_de_banda {:.2e}, convergiu={} — {:?}",
            r.vertices,
            r.ossos,
            r.presos,
            r.rondas,
            r.residuo,
            r.soma_pior,
            r.fora_de_banda,
            r.convergiu,
            comeco.elapsed()
        );
    }
    if !w.report.convergiu {
        eprintln!(
            "[bone] ⚠ os pesos do padrao-ouro nao CONVERGIRAM em {} rondas (residuo {:.2e}) — a \
             solucao e' admissivel e pode nao ser o minimo",
            w.report.rondas, w.report.residuo
        );
    }
    w.por_vertice.into_iter().flatten().collect()
}

/// ⭐⭐ **O ORÇAMENTO DO QUADRO mudou de ficheiro, NÃO de endereço** (tecto de LOC, 2026-09-16).
///
/// ⚠️ O `pub use` é deliberado, pela mesma razão do [`crate::bend_live`]: os consumidores escrevem
/// `skin_image::SKIN_FRAME_PIECES` e `skin_image::refine_options`.
pub use crate::skin_budget::{SKIN_FRAME_PIECES, refine_options};

/// ⛔ **Uma vez por processo, e nunca calado:** a instância desta sprite não é o quad DELA — um
/// 9-slice desenha-se em patches, uma folha sob pré-visualização desdobra o quad —, e a malha só
/// sabe o quad da sprite. Ela é desenhada SEM deformar, que é visível e não mente. Lacuna NOMEADA no
/// plano 03.
fn avisa_quad_que_nao_e_o_da_sprite() {
    static AVISADO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !AVISADO.swap(true, std::sync::atomic::Ordering::Relaxed) {
        eprintln!(
            "[bone] uma imagem presa desenha-se com um quad que nao e' o dela (9-slice, ou a folha \
             desdobrada de uma pre-visualizacao) — ela aparece SEM deformar: a malha so' conhece o \
             quad da sprite"
        );
    }
}

/// ⭐⭐⭐ **O CAMPO DE DEFORMAÇÃO desta imagem** — a régua da imagem mais a pele.
///
/// ⚠️ **Ele está definido em TODO ponto da imagem, e não só nos vértices da malha**, porque os
/// pesos são **derivados** e não guardados. É essa a lei inteira: *a malha não é a deformação, ela
/// é uma amostragem dela* — quem quiser mais pontos pede-os e eles existem.
///
/// ⛔ Uma segunda porta que compusesse a régua e a pele à mão divergiria desta na primeira
/// ramificação, e a imagem passaria a responder a uma lei diferente da forma vectorial.
#[must_use]
pub fn deform_field(
    sim: &SimWorld,
    e: Entity,
    size_px: [u32; 2],
    pixels_per_meter: f32,
) -> Option<(Xform, ph2d_skeleton::Skin)> {
    let p2l = pixel_to_local(sim.world().get::<Sprite>(e)?, size_px, pixels_per_meter)?;
    Some((p2l, crate::skin_live::skin_of(sim, e)?))
}

/// ⭐⭐⭐ **[`deform_field`] NUM INSTANTE** — a mesma régua, com a pele resolvida das poses que
/// `poses` der ([`crate::skin_live::skin_of_with`]).
///
/// ⚠️ **A RÉGUA `pixel → local` não depende do instante**, e é correcto: ela é a geometria do quad
/// da sprite (tamanho, âncora resolvida, espelho), não uma pose. O que o tempo muda é a PELE.
///
/// ⚠️ **O índice de ossos vem de fora** ([`crate::skin_live::bone_index`]): o consumidor é um LAÇO
/// (`N` artes × `M` instantes), e reconstruí-lo por chamada seria uma varredura do mundo inteiro por
/// fantasma — a lei que o doc do `skin_of` já escreve.
#[must_use]
pub fn deform_field_with(
    sim: &SimWorld,
    e: Entity,
    size_px: [u32; 2],
    pixels_per_meter: f32,
    index: &crate::skin_live::BoneIndex,
    poses: &impl Fn(Entity) -> Xform,
) -> Option<(Xform, ph2d_skeleton::Skin)> {
    let p2l = pixel_to_local(sim.world().get::<Sprite>(e)?, size_px, pixels_per_meter)?;
    Some((p2l, crate::skin_live::skin_of_in(sim, e, index, poses)?))
}

/// ⭐⭐⭐ **A MALHA POSADA de uma imagem presa — a porta ÚNICA do que se desenha.**
///
/// Recebe a malha de repouso (em pixels da imagem), a régua `pixel → local` ([`pixel_to_local`]), a
/// pele já resolvida e o quad da instância; devolve a [`SpriteMesh`] que o passe de sprites desenha,
/// mais o `k` do refinamento (`1` = não refinou).
///
/// ⚠️ **`refine` chega JÁ em unidades LOCAIS e já com a fatia do orçamento** — a conversão de pixels
/// de ecrã e a repartição do orçamento são factos do QUADRO, e ficam em quem tem o quadro na mão.
/// *Uma porta que recebesse a câmara passaria a ter duas respostas para «quanto é meio pixel».*
///
/// ⭐⭐ **Ela existe porque há DOIS consumidores** (2026-09-13): o quadro vivo
/// ([`attach_skin_meshes`]) e os **fantasmas do onion**, que posam a MESMA malha com as poses de
/// `t ± k`. ⛔ Copiada, as duas divergiriam no primeiro ajuste da UV — e o sintoma seria um fantasma
/// com a textura deslocada, que se lê como um defeito da própria pele.
///
/// `None` quando um vértice de repouso cai fora do quad da sprite (a UV não existe) — o mesmo
/// critério de sempre, numa porta só.
/// ⭐⭐⭐ **E `pesos` é a tabela do PADRÃO-OURO guardada no bind** — vazia ⇒ a lei derivada.
///
/// ⚠️⚠️ **Ela viaja pelo REFINAMENTO, e é isso que a torna utilizável:** um vértice que
/// a subdivisão inventa não tem peso guardado — ele nasce da aresta que o gerou, pela lei de
/// [`crate::skin_refine`]. ⛔⛔ **A 1.ª redacção desta nota dizia que a única resposta certa era o
/// baricêntrico**, e foi essa leitura em linha recta que pôs um vinco em cada aresta do bind (smoke
/// do dono, 2026-09-16: *«micro irregularidades»*). ⛔ Localizar o ponto na malha seria `O(n)` por
/// ponto para chegar à mesma resposta que a proveniência já sabe de graça.
#[must_use]
pub fn posed_sprite_mesh(
    mesh: Mesh2d,
    p2l: Xform,
    pele: &ph2d_skeleton::Skin,
    pesos: &[f64],
    anchor: [f32; 2],
    size: [f32; 2],
) -> Option<SpriteMesh> {
    let mut escrever = pele.scratch();
    // ⭐ **UMA porta por lei, escolhida UMA vez** — e não um `if` por vértice: a tabela ou existe
    // para esta malha ou não existe, e isso é um facto do bind, não de um ponto.
    let ossos = if mesh.rest.is_empty() {
        0
    } else {
        pesos.len() / mesh.rest.len()
    };
    let mut campo = |q: [f64; 2], w: &[f64]| {
        let p = p2l.apply(q);
        if ossos == 0 {
            pele.point(p, &mut escrever)
        } else {
            pele.point_with(p, w, &mut escrever)
        }
    };
    // ⭐⭐⭐ **UMA LEI, e é a de sempre: um afim por triângulo da malha que chegou.** O refinamento
    // POR QUADRO morreu em 2026-09-17 com a fileira `Deform` (ordem do dono) — a densidade é uma
    // decisão do BIND, e quem a toma é o [`crate::skin_bake_cache`].
    let posed: Vec<[f64; 2]> = mesh
        .rest
        .iter()
        .enumerate()
        .map(|(v, &q)| {
            let w = pesos.get(v * ossos..(v + 1) * ossos).unwrap_or(&[]);
            campo(q, w)
        })
        .collect();
    // ⭐ A UV de cada vértice é a do QUAD no ponto de REPOUSO dele — ver [`pixel_to_local`].
    let uv = mesh
        .rest
        .iter()
        .map(|&q| SpriteMesh::uv_at(f32_de(p2l.apply(q)), anchor, size))
        .collect::<Option<Vec<[f32; 2]>>>()?;
    Some(SpriteMesh {
        local: posed.into_iter().map(f32_de).collect(),
        uv,
        tris: mesh.tris,
    })
}

/// ⭐ **ESTA ENTIDADE É UMA IMAGEM PRESA AO ESQUELETO?** — uma `Sprite` com `SkinBind`.
///
/// ⚠️ **DERIVADA, nunca guardada:** uma sprite com pele é uma sprite que o esqueleto deforma, e a
/// pergunta responde-se olhando os dois componentes. Um terceiro componente a dizê-lo seria uma
/// fonte de verdade que pode discordar dos outros dois.
///
/// ⚠️ **Mora aqui desde 2026-09-13** (vivia no extract da shell, que a usava para NÃO emitir a
/// instância da imagem): quem a pergunta hoje é o painel do esqueleto (há uma imagem presa na
/// selecção?) e o [`attach_skin_meshes`] — e o extract deixou de precisar de saber o que é uma pele.
#[must_use]
pub fn is_skinned_image(world: &ph2d_ecs::World, e: Entity) -> bool {
    world.get::<Sprite>(e).is_some() && world.get::<ph2d_skeleton_ecs::SkinBind>(e).is_some()
}

/// ⭐⭐ **SOLTA UMA IMAGEM DO ESQUELETO** — tira-lhe a pele; devolve se havia pele a tirar.
///
/// ⚠️ **Existe pela regra F6-s do dono** (2026-09-16): uma ferramenta que muda o TAMANHO ou a
/// MARGEM da imagem, ao ser aplicada, quebra a ligação com os ossos — a malha do bind foi traçada
/// sobre a moldura de ANTES, e lê-la sobre a nova poria cada texel no sítio errado. Quem decide
/// QUANDO é a porta das ferramentas (`ph2d_app_painter::skin_suspend`); esta só sabe O QUÊ.
///
/// ⭐ **O regresso é o Ctrl+Z de sempre:** a pele é um componente registado, então tirá-la no mesmo
/// quadro do Apply põe a imagem nova e a ligação perdida no MESMO passo de desfazer.
///
/// ⛔ Só uma imagem presa (uma `Sprite` com pele) é tocada — uma forma vectorial presa solta-se pela
/// `skin_live::release`, que sabe devolver a geometria autorada.
pub fn release_image(sim: &mut SimWorld, bits: u64) -> bool {
    let Some(e) = Entity::try_from_bits(bits) else {
        return false;
    };
    if !is_skinned_image(sim.world(), e) {
        return false;
    }
    sim.world_mut()
        .entity_mut(e)
        .remove::<ph2d_skeleton_ecs::SkinBind>();
    true
}

fn f32_de(q: [f64; 2]) -> [f32; 2] {
    [q[0] as f32, q[1] as f32]
}

/// ⭐⭐⭐ **PÕE A MALHA POSADA NA INSTÂNCIA DE CADA IMAGEM PRESA** — a metade que se vê.
///
/// Corre DEPOIS do extract (as instâncias do quadro existem; o `present` é refeito por quadro) e
/// antes do passe de sprites. Devolve quantas malhas pôs; `0` é *«nenhuma imagem presa está a ser
/// desenhada»*, o caminho de toda cena que não usa a 2.ª mídia.
///
/// ⭐⭐⭐ **A sujeita é a INSTÂNCIA que o extract emitiu, e não a entidade da cena.** Uma sprite
/// escondida pelo olho, fora das camadas da câmera ou sem textura carregada não recebe malha porque
/// não tem instância — *perguntar a visibilidade aqui outra vez seria a segunda resposta que diverge*.
///
/// ⚠️ **Só a instância BASE** (`Without<SlicePatchMirror>`: as células fantasma da folha aberta e os
/// patches do 9-slice levam a MESMA `SimRef` com outra geometria), e só se ela for o quad da SPRITE
/// (o `size` e a âncora resolvida dela); fora disso a sprite desenha-se sem deformar e o aviso diz
/// porquê.
///
/// ⭐⭐⭐ **`suspensas` são as sprites que NÃO recebem malha neste quadro** — as que uma ferramenta de
/// pixels ou de moldura está a editar (ordem do dono, 2026-09-15, e a regra F6-s de 2026-09-16:
/// *«ao usar a ferramenta a imagem fica sem deformação até o fim da operação»*). ⚠️ **Um CONJUNTO, e
/// não uma:** o Padding, o Upscale e o Equalize Sizes editam a SELECÇÃO inteira.
///
/// ⚠️ **Esta folha não sabe o que é um Painter, e é assim que tem de ser:** quem decide é quem
/// chama (`ph2d_app_painter::skin_suspend::sprites_achatadas`), e o parâmetro diz **o quê**, nunca
/// **porquê**. ⭐ O filtro é o PRIMEIRO da cadeia, logo a malha suspensa nem chega a ser
/// descodificada — a suspensão é mais barata que a deformação, não mais cara.
///
/// ⛔ **O regresso é por construção:** isto corre a cada quadro, então deixar de suspender devolve
/// a deformação sozinho. *Não há estado a repor, logo não há como ficar preso achatado.*
///
/// ⛔⛔ **A ESCOLHA `Fast`/`Smooth` MORREU em 2026-09-17, por ordem do dono** — o que sai é sempre a
/// malha ASSADA no bind ([`crate::skin_bake_cache`]). ⚠️ **A premissa do botão tinha morrido antes
/// dele:** ele nasceu em 10/09 para *«arestas rectas ao dobrar»*, e a grelha graduada pelas
/// articulações (10/09) mais a lei de pesos de Hermite (16/09) curaram o facetado **na própria
/// malha de bind**. Medido em pixels de ECRÃ na dobra e no zoom que a cena ship, as duas leis punham
/// cada texel a `0,0435 px` uma da outra na mediana e `0,335 px` no pior ponto — *o dono reportou
/// quatro vezes que não via diferença, e não havia*.
///
/// ⚠️ **Toda régua desta linha media a grandeza ERRADA para aquela pergunta** (o desvio ao CAMPO, em
/// pixels da ARTE, que é uma propriedade da APROXIMAÇÃO). A régua de ecrã, e a metade que exige que
/// a separação EXISTA em dobra forte: `ph2d_app_vec::smoke_bone_paint_pixels_tests`.
pub fn attach_skin_meshes(
    sim: &SimWorld,
    present: &mut PresentWorld,
    pixels_per_meter: f32,
    suspensas: &[u64],
) -> usize {
    let presas: Vec<(Entity, SkinnedMesh)> = sim
        .world()
        .iter_entities()
        .filter(|er| {
            is_skinned_image(sim.world(), er.id()) && !suspensas.contains(&er.id().to_bits())
        })
        .filter_map(|er| Some((er.id(), skinned_mesh_of(sim, er.id())?)))
        .collect();
    if presas.is_empty() {
        return 0;
    }
    // ⭐⭐⭐ **QUEM RECEBE MALHA: a instância BASE e todo PEDAÇO que diga que fracção mostra** (F11).
    //
    // ⛔ Os fantasmas da folha aberta e a pré-visualização animada levam `SlicePatchMirror` e **não**
    // levam `SlicePatchSource` — eles ficam de fora, e é desenho e não esquecimento: o quad deles é
    // a folha DESDOBRADA, que não é um pedaço da arte desta sprite.
    let alvos: Vec<(Entity, Entity, Option<[f32; 4]>)> = {
        let mut q = present.world_mut().query_filtered::<(
            Entity,
            &SimRef,
            Option<&ph2d_render::nine_slice::SlicePatchSource>,
            ph2d_ecs::Has<SlicePatchMirror>,
        ), With<RenderInstance>>();
        q.iter(present.world())
            .filter(|(_, r, fonte, espelho)| {
                presas.iter().any(|(e, _)| *e == r.0) && (fonte.is_some() || !espelho)
            })
            .map(|(p, r, fonte, _)| (r.0, p, fonte.map(|f| f.frac)))
            .collect()
    };
    // ⭐⭐⭐ **DESENHA-SE A MALHA ASSADA, SEMPRE** (F9 W2b): a densidade é decisão do BIND — não do
    // quadro —, memoizada por [`crate::skin_bake_cache`], e a partir de 2026-09-17 ela é a ÚNICA
    // lei (ver o doc desta função).
    //
    // ⚠️ **A assadura é a que erra MENOS e custa MENOS**, e as duas metades foram medidas: ela
    // persegue a curvatura do peso no BIND, onde a pose é a de repouso e há tempo, e entrega
    // `2,7 ×` mais peças por `5,6 ×` menos relógio — refinar custa `~0,32 µs` por peça e desenhar
    // uma peça já fina `~0,017 µs`. *O caro era decidir por quadro, nunca a peça.*
    //
    // ⛔ **`unwrap_or(m)` não é um fallback silencioso:** a assadura devolve `None` quando o campo
    // de pesos é LINEAR, e ali partir uma aresta não muda um bit — a malha do bind já é a resposta.
    // ⚠️ **A assadura é por SPRITE e não por instância**: um 9-slice tem nove alvos e UMA arte, e
    // assar nove vezes pagaria nove vezes o que o memo existe para cobrar uma.
    let assadas: BTreeMap<Entity, SkinnedMesh> = presas
        .into_iter()
        .map(|(e, m)| {
            let a = crate::skin_bake_cache::assada_da_arte(sim, e, &m).unwrap_or(m);
            (e, a)
        })
        .collect();
    let mut feitas = 0;
    for (e, p, frac) in alvos {
        let (Some(sprite), Some(inst), Some(assada)) = (
            sim.world().get::<Sprite>(e),
            present.world().get::<RenderInstance>(p).copied(),
            assadas.get(&e),
        ) else {
            continue;
        };
        // ⛔ **O aviso fica para quem NÃO diz que pedaço é** — a folha desdobrada de uma
        // pré-visualização. Um pedaço de 9-slice traz a fracção, e por isso já não cai aqui.
        if frac.is_none()
            && (inst.size != sprite.size || inst.anchor != sprite.resolve_anchor(pixels_per_meter))
        {
            avisa_quad_que_nao_e_o_da_sprite();
            continue;
        }
        let antes = assada.mesh.tris.len();
        // ⭐⭐⭐ **O PEDAÇO: a malha cortada na fracção que este quad mostra**, e o mapa é o do quad
        // DELE. ⚠️ Cortar no espaço da IMAGEM (e não recortar o quad) é a lei toda do 9-slice: o
        // pedaço do meio mostra a faixa central ESTICADA, logo a tinta que o mapa de repouso poria
        // ali não é a que se vê.
        let (mesh, pesos, rect) = match frac {
            Some(f) if f != [0.0, 0.0, 1.0, 1.0] => {
                let (w, h) = (
                    f64::from(assada.mesh.size[0]),
                    f64::from(assada.mesh.size[1]),
                );
                let r = [
                    f64::from(f[0]) * w,
                    f64::from(f[1]) * h,
                    f64::from(f[2]) * w,
                    f64::from(f[3]) * h,
                ];
                let ossos = if assada.mesh.rest.is_empty() {
                    0
                } else {
                    assada.pesos.len() / assada.mesh.rest.len()
                };
                let Some((m, w)) =
                    ph2d_poly2d::submesh_in_rect(&assada.mesh, &assada.pesos, ossos, r)
                else {
                    continue;
                };
                (m, w, r)
            }
            _ => (
                assada.mesh.clone(),
                assada.pesos.clone(),
                [
                    0.0,
                    0.0,
                    f64::from(assada.mesh.size[0]),
                    f64::from(assada.mesh.size[1]),
                ],
            ),
        };
        let Some(p2l) = rect_to_quad(sprite, rect, inst.anchor, inst.size) else {
            continue;
        };
        let Some(pele) = crate::skin_live::skin_of(sim, e) else {
            continue;
        };
        let Some(malha) = posed_sprite_mesh(mesh, p2l, &pele, &pesos, inst.anchor, inst.size)
        else {
            continue;
        };
        // ⚠️ **O diagnóstico da família** (`PH2D_BONE_LOG=1`): sem ele um report de *«facetou»* não
        // distingue *quantas peças* de *que dobra* — e foi a CONTAGEM que se revelou a grandeza que
        // importa. ⛔ Ele diz agora UMA coisa, porque há UMA lei: quantas peças esta imagem desenha.
        if std::env::var_os("PH2D_BONE_LOG").is_some() {
            eprintln!(
                "[bone] pele: {antes} -> {} pecas desenhadas",
                malha.tris.len()
            );
        }
        present.world_mut().entity_mut(p).insert(malha);
        feitas += 1;
    }
    feitas
}

/// ⭐⭐ **A malha GUARDADA desta imagem, com os pesos dentro** — a porta única do que o quadro lê.
///
/// ⚠️ **O `SkinBind::source` é opaco de propósito**, e quem sabe decodificá-lo é quem sabe o que a
/// coisa É: uma entidade com `Sprite` guarda uma [`SkinnedMesh`], uma com `VecPathRef` guarda um
/// `VecPath`. ⛔ Um discriminante guardado ao lado seria uma segunda fonte de verdade sobre a
/// mídia, e ela poderia discordar da entidade.
///
/// ⛔ **Uma tabela que não fecha com a malha é RECUSADA** ([`SkinnedMesh::valida`]) — ler uma
/// tabela deslocada por um vértice entrega pesos plausíveis e arte errada.
#[must_use]
pub fn skinned_mesh_of(sim: &SimWorld, e: Entity) -> Option<SkinnedMesh> {
    let skin = sim.world().get::<ph2d_skeleton_ecs::SkinBind>(e)?;
    let m: SkinnedMesh = postcard::from_bytes(&skin.source).ok()?;
    m.valida().then_some(m)
}

/// A malha guardada, sem os pesos — para quem só pergunta pela geometria.
///
/// ⚠️ **Ela é uma DOBRA da [`skinned_mesh_of`], nunca uma segunda descodificação**: dois leitores
/// dos mesmos bytes divergem no primeiro que alguém mexer.
#[must_use]
pub fn mesh_of(sim: &SimWorld, e: Entity) -> Option<Mesh2d> {
    skinned_mesh_of(sim, e).map(|m| m.mesh)
}

#[cfg(test)]
#[path = "skin_image_tests.rs"]
mod tests;
