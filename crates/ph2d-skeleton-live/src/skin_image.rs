//! ⭐⭐⭐ **UMA IMAGEM OBEDECE AO ESQUELETO** — a segunda mídia (ordem do dono, 2026-09-09).
//!
//! Pesquisa e recusas medidas: [`docs/Skeleton/02_pesquisa_a_malha_sobre_a_imagem.md`]. O desenho de
//! hoje: [`docs/Skeleton/03_plano_a_pele_no_passe_de_sprites.md`].
//!
//! # As quatro perguntas, e onde cada uma é respondida
//!
//! ```text
//! onde a tinta acaba?      ph2d_poly2d::mesh_of          (uma vez, ao PRENDER)
//! que peso cada ponto tem? Skeleton::weights_at          (já existia)
//! onde o ponto vai parar?  Skeleton::deform_points       (já existia)
//! como se desenha isso?    ph2d_render::SpriteMesh       (ESTE módulo põe-na, por quadro)
//! ```
//!
//! ⭐⭐ **A metade difícil já estava feita.** O peso é derivado por distância ao osso e não é
//! guardado, e a [`ph2d_skeleton_ecs::SkinBind`] já nascera agnóstica de mídia — o doc dela dizia,
//! por escrito, *«serve um `VecPath` hoje e uma malha raster amanhã sem uma variante nova nem um
//! schema por mídia»*. ⇒ o que esta mídia acrescenta é a **malha** e o **desenho**, e nada mais.
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

use ph2d_ecs::{Entity, PresentWorld, SimRef, SimWorld, With, Without};
use ph2d_poly2d::{Mesh2d, RefineOptions};
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
    let (w, h) = (f64::from(size_px[0]), f64::from(size_px[1]));
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
    let (sx, sy) = (
        lado(sprite.size[0], sprite.flip_x),
        lado(sprite.size[1], sprite.flip_y),
    );
    let a = sprite.resolve_anchor(pixels_per_meter);
    let (ax, ay) = (f64::from(a[0]), f64::from(a[1]));
    Some(Xform([
        sx / w,
        0.0,
        0.0,
        -sy / h,
        ax - sx / 2.0,
        ay + sy / 2.0,
    ]))
}

/// ⭐⭐ **A MALHA DESTA IMAGEM, a partir dos pixels dela** — a porta do gesto de prender.
///
/// ⚠️ **Só o canal ALFA entra.** A cobertura é o que decide a silhueta, e passar as três cores
/// junto seria dar ao traçador três respostas para a mesma pergunta.
///
/// ⭐⭐⭐ **`focos` são as ARTICULAÇÕES, em pixels da imagem** (report do dono, 2026-09-10:
/// *«deveria ser um quadmesh inteligente com maior densidade nas áreas das articulações»*). Elas
/// entram porque só quem prende sabe onde a dobra vai acontecer — o leaf da geometria não sabe o
/// que é um osso, e não devia saber.
#[must_use]
pub fn mesh_from_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
    focos: &[[f64; 2]],
    opts: ph2d_poly2d::GridOptions,
) -> Option<Mesh2d> {
    let alfa: Vec<u8> = rgba.iter().skip(3).step_by(4).copied().collect();
    ph2d_poly2d::grid_mesh_of(&alfa, width, height, focos, opts)
}

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

/// Um quadro de 60 fps, em microssegundos — o RECURSO de que o orçamento da pele é uma fatia.
const QUADRO_60FPS_US: usize = 16_667;

/// ⚠️ **A fatia do quadro que a pele pode gastar — e é a única ESCOLHA desta constante.** A pele é
/// uma coisa entre muitas no quadro (a arte do documento pelo Vello, o chrome, os passes de luz, o
/// resto das sprites); `1/10` deixa-lhe uma fatia visível sem lhe dar o quadro. Quem quiser medir
/// outra fatia tem o `PH2D_SKIN_PIECES`.
const FATIA_DA_PELE: usize = 10;

/// O custo MEDIDO de uma peça ENTREGUE com `Smooth`, em nanossegundos (a tabela do
/// [`SKIN_FRAME_PIECES`]).
const CUSTO_POR_PECA_NS: usize = 1_080;

/// ⭐⭐⭐ **O ORÇAMENTO DE PEÇAS DA PELE DE IMAGEM, POR QUADRO** — derivado do recurso deste caminho:
/// o TEMPO do quadro.
///
/// ⚠️⚠️ **O número de antes era do VELLO, e descrevia outro caminho.** Até 2026-09-13 a pele era uma
/// camada do Vello e o tecto saía do buffer fixo de informação por desenho dele (`1 << 18` palavras,
/// `11` + bins por peça ⇒ metade dele dava `8 738` peças, e passar do buffer deixava o quadro
/// **em branco**). Desde a W2 do plano 03 a pele é uma malha no passe de sprites: aquele buffer já
/// não é gasto por ela, e `8 738` peças custariam hoje **`9,4 ms`** — mais de metade de um quadro de
/// 60 fps. *§0.0: o número de um caminho morto não limita o vivo.*
///
/// ⭐ **O que UMA peça custa, medido** (W4, 2026-09-13; `load 3,7`–`3,9`, o MÍNIMO de 40/60 corridas
/// — as sondas são [`tests::measure_the_cpu_cost_of_a_skinned_frame`] e a
/// `ph2d-render::sprite_mesh_gpu::measure_the_frame_cost_of_a_mesh_sprite`):
///
/// | o que o quadro faz por peça | µs |
/// |---|---:|
/// | descodificar a malha guardada (postcard, **por quadro**) | `0,134` |
/// | `Fast`: descodificar + deformar + montar o `SpriteMesh` | `0,200` |
/// | recolher + costurar a tira + enviar + DESENHAR (GPU esperada) | `0,039` |
/// | **`Smooth`: o quadro inteiro, por peça ENTREGUE** | **`1,08`** |
///
/// ⇒ `16,667 ms ÷ 10 ÷ 1,08 µs` = **`1 543` peças**.
///
/// ⛔⛔⛔ **E ESTA NOTA AFIRMAVA UMA COISA FALSA ATÉ 2026-09-15:** *«este é o tecto; quem decide o
/// refinamento é a TOLERÂNCIA — dentro dele o `Smooth` refina só o que a dobra pedir»*. Ele **não**
/// refina só o que a dobra pede: o `k` do [`ph2d_poly2d::refine_posed`] é **GLOBAL** (cada triângulo
/// é partido `k × k`), e o tecto dele é `max_split = ⌊√(orçamento / peças)⌋`. ⇒ **uma malha base
/// acima de `orçamento / 4` peças só admite `k = 1`, e o `Smooth` fica byte-idêntico ao `Fast`, com
/// a tolerância a não decidir nada.**
///
/// ⚠️ Com `1 543` de orçamento isso é toda malha acima de **`385`** triângulos — que é quase toda
/// arte real. Medido na cena do osso (`PH2D_VEC_BONE_PAINT_SMOKE`, 2026-09-15): `780` peças ⇒
/// `k = 1` ⇒ desvio de **`14,24 px`** de ecrã contra uma tolerância que promete `0,5`.
///
/// ⭐ **O caminho está medido e é o ADAPTATIVO:** com `k` por triângulo (só onde o desvio pede) a
/// mesma cena custa `3 034` peças contra as `28 080` do `k = 6` global — **`9×` mais barato** —, e
/// numa cena bem autorada cai para `1 059`, **dentro** deste orçamento. Ele não está construído; o
/// mecanismo, os números e a armadilha (as arestas pendentes) estão no handoff §13 de 2026-09-15.
/// ⚠️ **O `PH2D_BONE_LOG=1` diz agora quando o refinamento está desligado por esta aritmética** —
/// antes «não precisou» e «não pôde» imprimiam a mesma linha.
pub const SKIN_FRAME_PIECES: usize = QUADRO_60FPS_US * 1_000 / FATIA_DA_PELE / CUSTO_POR_PECA_NS;

/// ⛔⛔ **A OUTRA PONTA DO TECTO, verificada na COMPILAÇÃO.** Um tecto apertado de mais deixa de
/// refinar uma malha comum: a malha guardada do smoke tem `~200` peças e um `k = 2` entrega `~800`
/// — abaixo de `1 000` o `Smooth` fica inerte, que é a doença do tecto de `1 024` POR IMAGEM que
/// este número substituiu. Uma fatia mais fina (ou um custo por peça maior, medido outra vez) tem
/// de PARAR a build aqui, e não passar em silêncio.
const _: () = assert!(
    SKIN_FRAME_PIECES >= 1_000,
    "o orcamento da pele nao chega para refinar uma malha comum (k = 2)"
);

/// ⭐⭐⭐ **OS NÚMEROS DO `Smooth`**: a tolerância da `ph2d-poly2d` e o orçamento do QUADRO.
///
/// ⚠️ **`PH2D_SKIN_PIECES=<n>` afina o orçamento do QUADRO**, e não de uma imagem — ele existe desde
/// o report *«Smooth bugado quebrando a forma»* (dono, 2026-09-10), cuja causa se mediu depois
/// (a porta crua enchia o atlas, F6-e). *Um smoke que MEDE vale mais que um smoke que pergunta.*
#[must_use]
pub fn refine_options() -> RefineOptions {
    static PECAS: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let escolhido = *PECAS.get_or_init(|| {
        std::env::var("PH2D_SKIN_PIECES")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|n| *n > 0)
    });
    RefineOptions {
        max_pieces: escolhido.unwrap_or(SKIN_FRAME_PIECES),
        ..RefineOptions::default()
    }
}

/// ⭐ **A parte do orçamento do quadro que cabe a uma imagem** — proporcional à malha que ela guarda,
/// e nunca abaixo dela (a malha guardada é o desenho mínimo; não há como desenhá-la com menos).
///
/// ⚠️ **Proporcional dá o MESMO `k` a todas**, logo a mesma qualidade: o `k` sai de
/// `√(parte / triângulos)`, e a razão é a mesma para todas as imagens do quadro.
fn parte_do_orcamento(triangulos: usize, guardadas: usize, orcamento: usize) -> usize {
    let parte = orcamento.saturating_mul(triangulos) / guardadas.max(1);
    parte.max(triangulos)
}

/// ⛔ **Uma vez por processo**, e nunca calado (DIRETIVA §2: zero no-op silencioso): as malhas
/// GUARDADAS das imagens deste quadro já passam do orçamento, e o `Smooth` não tem refinamento a
/// cortar — cada imagem é desenhada com a malha que guardou.
fn avisa_malhas_acima_do_orcamento(guardadas: usize, orcamento: usize) {
    static AVISADO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !AVISADO.swap(true, std::sync::atomic::Ordering::Relaxed) {
        eprintln!(
            "[bone] as imagens presas deste quadro guardam {guardadas} pecas e o orcamento do \
             quadro e' {orcamento} (PH2D_SKIN_PIECES) — o Smooth nao refina nenhuma: cada uma e' \
             desenhada com a malha guardada"
        );
    }
}

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
/// pesos são **derivados** e não guardados. É essa a lei inteira do `Smooth`: *a malha não é a
/// deformação, ela é uma amostragem dela* — quem quiser mais pontos pede-os e eles existem.
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
#[must_use]
pub fn posed_sprite_mesh(
    mesh: Mesh2d,
    p2l: Xform,
    pele: &ph2d_skeleton::Skin,
    anchor: [f32; 2],
    size: [f32; 2],
    refine: Option<RefineOptions>,
) -> Option<(SpriteMesh, u32)> {
    let mut escrever = pele.scratch();
    let mut campo = |q: [f64; 2]| pele.point(p2l.apply(q), &mut escrever);
    let (mesh, posed, k) = match refine {
        None => {
            let posed: Vec<[f64; 2]> = mesh.rest.iter().map(|&q| campo(q)).collect();
            (mesh, posed, 1)
        }
        Some(o) => ph2d_poly2d::refine_posed(&mesh, &mut campo, o),
    };
    // ⭐ A UV de cada vértice é a do QUAD no ponto de REPOUSO dele — ver [`pixel_to_local`].
    let uv = mesh
        .rest
        .iter()
        .map(|&q| SpriteMesh::uv_at(f32_de(p2l.apply(q)), anchor, size))
        .collect::<Option<Vec<[f32; 2]>>>()?;
    Some((
        SpriteMesh {
            local: posed.into_iter().map(f32_de).collect(),
            uv,
            tris: mesh.tris,
        },
        k,
    ))
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
/// ⭐⭐ **`Smooth`** (`smooth = Some`): a tolerância é em pixels de ECRÃ, e a escala `local → ecrã` é
/// a base de mundo que a instância leva para a GPU (`basis`) vezes `px_per_world`, a da câmera — a
/// que a GPU aplica, e não uma segunda conta a partir do `Transform`. O orçamento é do QUADRO,
/// repartido pelas imagens que DESENHAM: uma escondida não gasta a parte de ninguém.
pub fn attach_skin_meshes(
    sim: &SimWorld,
    present: &mut PresentWorld,
    pixels_per_meter: f32,
    smooth: Option<RefineOptions>,
    px_per_world: f64,
) -> usize {
    let presas: Vec<(Entity, Mesh2d)> = sim
        .world()
        .iter_entities()
        .filter(|er| is_skinned_image(sim.world(), er.id()))
        .filter_map(|er| Some((er.id(), mesh_of(sim, er.id())?)))
        .collect();
    if presas.is_empty() {
        return 0;
    }
    let instancias: BTreeMap<Entity, Entity> = {
        let mut q = present
            .world_mut()
            .query_filtered::<(Entity, &SimRef), (With<RenderInstance>, Without<SlicePatchMirror>)>(
            );
        q.iter(present.world())
            .filter(|(_, r)| presas.iter().any(|(e, _)| *e == r.0))
            .map(|(p, r)| (r.0, p))
            .collect()
    };
    let vivas: Vec<(Entity, Entity, Mesh2d)> = presas
        .into_iter()
        .filter_map(|(e, m)| Some((e, *instancias.get(&e)?, m)))
        .collect();
    let guardadas: usize = vivas.iter().map(|(_, _, m)| m.tris.len()).sum();
    if let Some(o) = smooth
        && guardadas > o.max_pieces
    {
        avisa_malhas_acima_do_orcamento(guardadas, o.max_pieces);
    }
    let mut feitas = 0;
    for (e, p, mesh) in vivas {
        let (Some(sprite), Some(inst)) = (
            sim.world().get::<Sprite>(e),
            present.world().get::<RenderInstance>(p).copied(),
        ) else {
            continue;
        };
        if inst.size != sprite.size || inst.anchor != sprite.resolve_anchor(pixels_per_meter) {
            avisa_quad_que_nao_e_o_da_sprite();
            continue;
        }
        let Some((p2l, pele)) = deform_field(sim, e, mesh.size, pixels_per_meter) else {
            continue;
        };
        let antes = mesh.tris.len();
        // ⚠️⚠️ **A tolerância é em pixels de ECRÃ:** meia unidade local é meio pixel a zoom `1` e
        // **quatro** a zoom `8`. *A suavidade que o olho vê é um facto de espaço de ecrã* — uma
        // tolerância em unidades locais afinaria a malha para o zoom em que o artista não está.
        // ⚠️ A conversão (e a fatia do orçamento) moram AQUI, que é quem tem o quadro na mão; a
        // porta que posa recebe-as já resolvidas.
        let refine = smooth.map(|o| {
            let b = inst.basis;
            let det = f64::from(b[0]) * f64::from(b[3]) - f64::from(b[2]) * f64::from(b[1]);
            let escala = (px_per_world * det.abs().sqrt()).max(f64::MIN_POSITIVE);
            RefineOptions {
                tolerance_px: o.tolerance_px / escala,
                max_pieces: parte_do_orcamento(antes, guardadas, o.max_pieces),
            }
        });
        let Some((malha, k)) = posed_sprite_mesh(mesh, p2l, &pele, inst.anchor, inst.size, refine)
        else {
            continue;
        };
        // ⚠️ **O diagnóstico da família** (`PH2D_BONE_LOG=1`): sem ele um report de *«partiu»* não
        // distingue *quantas peças* de *que dobra* — e foi a CONTAGEM que se revelou a grandeza que
        // importa.
        if let (Some(o), true) = (refine, std::env::var_os("PH2D_BONE_LOG").is_some()) {
            eprintln!(
                "[bone] pele suave: {antes} -> {} pecas (k={k}, de um orcamento de quadro {})",
                malha.tris.len(),
                o.max_pieces
            );
            // ⭐⭐⭐ **A LINHA QUE FALTAVA: «k=1» tem DUAS causas e elas são opostas.**
            //
            // ⛔⛔ Ou a dobra não pediu refinamento nenhum (tudo bem), ou o ORÇAMENTO o proibiu —
            // e nesse caso o `Smooth` é o `Fast` **ao bit**, com o painel a dizer que está ligado.
            // Medido 2026-09-15 na cena do osso: `780` peças com orçamento `1 543` ⇒
            // `780 × 2² = 3 120 > 1 543` ⇒ `max_split = 1`, e a tolerância nunca decide nada.
            // *Um diagnóstico que imprime o mesmo número para «não precisou» e para «não pôde»
            // cala exactamente a pergunta que o report do dono fazia.*
            if ph2d_poly2d::max_split(antes, o) == 1 {
                eprintln!(
                    "[bone]   ⚠ o REFINAMENTO esta' DESLIGADO POR ARITMETICA: {antes} x 2² = {} > \
                     {} -- aqui o Smooth E' o Fast, e a tolerancia nao decide nada",
                    antes * 4,
                    o.max_pieces
                );
            }
        }
        present.world_mut().entity_mut(p).insert(malha);
        feitas += 1;
    }
    feitas
}

/// A malha guardada nos bytes opacos da pele desta entidade.
///
/// ⚠️ **O `SkinBind::source` é opaco de propósito**, e quem sabe decodificá-lo é quem sabe o que a
/// coisa É: uma entidade com `Sprite` guarda uma [`Mesh2d`], uma com `VecPathRef` guarda um
/// `VecPath`. ⛔ Um discriminante guardado ao lado seria uma segunda fonte de verdade sobre a
/// mídia, e ela poderia discordar da entidade.
#[must_use]
pub fn mesh_of(sim: &SimWorld, e: Entity) -> Option<Mesh2d> {
    let skin = sim.world().get::<ph2d_skeleton_ecs::SkinBind>(e)?;
    postcard::from_bytes(&skin.source).ok()
}

#[cfg(test)]
#[path = "skin_image_tests.rs"]
mod tests;
