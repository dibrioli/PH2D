//! ⭐⭐⭐ **UMA IMAGEM OBEDECE AO ESQUELETO** — a segunda mídia (ordem do dono, 2026-09-09).
//!
//! Pesquisa e recusas medidas: [`docs/Skeleton/02_pesquisa_a_malha_sobre_a_imagem.md`].
//!
//! # As quatro perguntas, e onde cada uma é respondida
//!
//! ```text
//! onde a tinta acaba?      ph2d_poly2d::mesh_of      (uma vez, ao PRENDER)
//! que peso cada ponto tem? Skeleton::weights_at      (já existia)
//! onde o ponto vai parar?  Skeleton::deform_points   (já existia)
//! como se desenha isso?    ESTE módulo               (por quadro)
//! ```
//!
//! ⭐⭐ **A metade difícil já estava feita.** O peso é derivado por distância ao osso e não é
//! guardado, e a [`ph2d_skeleton_ecs::SkinBind`] já nascera agnóstica de mídia — o doc dela dizia,
//! por escrito, *«serve um `VecPath` hoje e uma malha raster amanhã sem uma variante nova nem um
//! schema por mídia»*. ⇒ o que esta wave acrescenta é a **malha** e o **desenho**, e nada mais.
//!
//! # ⛔ Porque o desenho é UM AFIM POR TRIÂNGULO, e não um pipeline novo
//!
//! Um triângulo de repouso e um de destino determinam um afim exactamente
//! ([`ph2d_affine::Xform::from_triangle`]), então uma malha deformada desenha-se como *N* pedaços
//! da MESMA imagem, cada um recortado ao seu triângulo. As duas peças já existiam — o
//! [`ph2d_vector::VectorScene::push_clip`] e o desenho de imagem com afim — e o compositor põe
//! o Vello **por cima** do passe de sprites.
//!
//! ⛔⛔ **«A MESMA imagem» tem de ser o mesmo ID do atlas, e não só os mesmos bytes** — ver
//! [`SkinImageCache`]: pela porta crua cada peça era uma cópia inteira da imagem no atlas.
//!
//! ⛔ **A rota por INSTÂNCIA de sprite foi medida e recusada:** a instância carrega um basis 2×2,
//! logo cada célula sairia **paralelogramo**, e duas células vizinhas de um warp real não partilham
//! aresta — a costura abre fenda. ⏸️ Um pipeline de triângulos texturados é a optimização, **com
//! razão medida**, se a rota de hoje não couber no quadro.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_poly2d::Mesh2d;
use ph2d_render::Sprite;
use ph2d_skeleton::Xform;

/// ⭐⭐⭐ **PIXEL DA IMAGEM → PONTO LOCAL DA SPRITE** — a lei que ata as duas réguas.
///
/// A sprite ocupa `size` unidades de mundo, centrada no `anchor` (que é o vector do pivô ao centro
/// do quad, em metros locais), e a textura é lida com `v = 0` **em cima**. ⇒ o pixel `(0, 0)` é o
/// canto **superior esquerdo** do quad e o pixel `(w, h)` o inferior direito.
///
/// ⚠️⚠️ **O `y` VIRA, e é a única inversão de todo o módulo.** Uma imagem tem o `y` a crescer para
/// baixo e o mundo tem-no a crescer para cima; esquecê-la desenha o personagem **de cabeça para
/// baixo**, e é o tipo de defeito que passa por todo gate de geometria (a malha está certa, a
/// deformação está certa, e a imagem está ao contrário).
///
/// `None` quando a imagem tem lado zero — ali não há régua.
#[must_use]
pub fn pixel_to_local(sprite: &Sprite, size_px: [u32; 2]) -> Option<Xform> {
    let (w, h) = (f64::from(size_px[0]), f64::from(size_px[1]));
    if !(w > 0.0 && h > 0.0) {
        return None;
    }
    let (sx, sy) = (f64::from(sprite.size[0]), f64::from(sprite.size[1]));
    let (ax, ay) = (f64::from(sprite.anchor[0]), f64::from(sprite.anchor[1]));
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
) -> Vec<[f64; 2]> {
    let Some(sprite) = sim.world().get::<Sprite>(e) else {
        return Vec::new();
    };
    let (Some(p2l), Some(mundo)) = (
        pixel_to_local(sprite, size_px),
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

/// Palavras da DISTRIBUIÇÃO POR BINS por peça — o limite de cima do intervalo que a sonda de GPU
/// `ph2d-render::skin_pieces_gpu_cost` deu: numa cena só com a pele o quadro desenhou `17 496`
/// peças (sobram `≥ 3,98` palavras cada) e ficou em branco a `21 600` (sobravam `1,14`).
const BINNING_WORDS_PER_PIECE: u32 = 4;

/// ⭐⭐⭐ **O ORÇAMENTO DE PEÇAS DA PELE DE IMAGEM, POR QUADRO** — derivado do recurso, não escolhido.
///
/// O Vello guarda a informação de todo desenho de um quadro num buffer FIXO
/// ([`ph2d_vector::VELLO_BIN_DATA_WORDS`], `1 << 18`), e passar dele deixa **o quadro inteiro em
/// branco** em release (pânico em debug) — painéis incluídos. Uma peça gasta
/// [`ph2d_vector::CLIPPED_IMAGE_INFO_WORDS`] (`11`) mais a distribuição por bins (até `4`):
///
/// | quem gasta o buffer | medido (2026-09-13) |
/// |---|---|
/// | o chrome do editor, painéis de omissão | `109` palavras (`0,04 %`) |
/// | o chrome, TODOS os painéis abertos | `~1 190` (`0,45 %`) |
/// | uma peça da pele | `11` + bins `1,14`–`3,98` |
/// | uma cena só com a pele | desenha `17 496` peças · em branco a `21 600` |
///
/// ⇒ **METADE do buffer para a pele** (`8 738` peças); a outra metade fica para a arte do canvas,
/// que **não foi medida** — e é por isso que é metade, e não `99 %`.
///
/// ⚠️ **Este é o tecto; quem decide o refinamento é a TOLERÂNCIA.** Até 2026-09-13 havia um tecto
/// de `1 024` peças POR IMAGEM, escolhido *«do lado seguro»* de um intervalo que ninguém tinha
/// medido — e ele passava por cima dela: a `k = 2` o `Smooth` entregava `3,5 px` numa dobra forte
/// contra os `0,5 px` pedidos (§0.0: nunca deixe um palpite definir o produto).
pub const SKIN_FRAME_PIECES: usize = (ph2d_vector::VELLO_BIN_DATA_WORDS
    / 2
    / (ph2d_vector::CLIPPED_IMAGE_INFO_WORDS + BINNING_WORDS_PER_PIECE))
    as usize;

/// ⭐⭐⭐ **OS NÚMEROS DO `Smooth`**: a tolerância da `ph2d-poly2d` e o orçamento do QUADRO.
///
/// ⚠️ **`PH2D_SKIN_PIECES=<n>` afina o orçamento do QUADRO**, e não de uma imagem — ele existe desde
/// o report *«Smooth bugado quebrando a forma»* (dono, 2026-09-10), cuja causa se mediu depois
/// (a porta crua enchia o atlas, F6-e). *Um smoke que MEDE vale mais que um smoke que pergunta.*
#[must_use]
pub fn refine_options() -> ph2d_poly2d::RefineOptions {
    static PECAS: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let escolhido = *PECAS.get_or_init(|| {
        std::env::var("PH2D_SKIN_PIECES")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|n| *n > 0)
    });
    ph2d_poly2d::RefineOptions {
        max_pieces: escolhido.unwrap_or(SKIN_FRAME_PIECES),
        ..ph2d_poly2d::RefineOptions::default()
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
/// GUARDADAS deste quadro já passam do orçamento, e não há refinamento a cortar — o desenho mínimo
/// não cabe, e o Vello pode deixar o quadro em branco.
fn avisa_malhas_acima_do_orcamento(guardadas: usize, orcamento: usize) {
    static AVISADO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !AVISADO.swap(true, std::sync::atomic::Ordering::Relaxed) {
        eprintln!(
            "[bone] as imagens presas deste quadro guardam {guardadas} pecas e o orcamento do \
             quadro e' {orcamento} (PH2D_SKIN_PIECES) — acima dele o Vello pode deixar o quadro em \
             branco"
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
) -> Option<(Xform, ph2d_skeleton::Skin)> {
    let p2l = pixel_to_local(sim.world().get::<Sprite>(e)?, size_px)?;
    Some((p2l, crate::skin_live::skin_of(sim, e)?))
}

/// ⭐⭐⭐ **O AFIM DE UM TRIÂNGULO — de pixel da imagem a espaço LOCAL da sprite.**
///
/// ⚠️ **O repouso entra em PIXELS**, e não em local: assim o afim devolvido já compõe as duas
/// coisas (a régua da imagem e a deformação), e o desenho passa-o directamente ao
/// `draw_image_rgba_transformed`, que espera um mapa a partir do espaço da imagem. Compor as duas
/// à mão em cada chamada seria a segunda resposta à mesma pergunta.
#[must_use]
pub fn triangle_xform(
    mesh: &Mesh2d,
    posed: &[[f64; 2]],
    tri: [u32; 3],
) -> Option<(Xform, [[f64; 2]; 3])> {
    let i = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
    let rest = [
        *mesh.rest.get(i[0])?,
        *mesh.rest.get(i[1])?,
        *mesh.rest.get(i[2])?,
    ];
    let alvo = [*posed.get(i[0])?, *posed.get(i[1])?, *posed.get(i[2])?];
    Some((Xform::from_triangle(rest, alvo)?, alvo))
}

#[cfg(test)]
#[path = "skin_image_tests.rs"]
mod tests;

/// ⭐⭐⭐ **DESENHA AS IMAGENS PRESAS AO ESQUELETO** — a metade que se vê.
///
/// Devolve quantas desenhou; `0` quer dizer *«nenhuma imagem está presa»* e é o caminho de
/// omissão de toda cena que não usa a 2.ª mídia.
///
/// ⚠️ **Cada triângulo é um RECORTE mais um afim**, e nada mais: o `push_clip` limita o desenho ao
/// triângulo posado, e a imagem inteira é desenhada com o afim que leva o triângulo de repouso
/// àquele. Dois triângulos vizinhos concordam nos dois vértices que partilham, logo a aresta comum
/// é a mesma recta nos dois — **a continuidade é consequência, não tolerância**.
///
/// ⚠️ **Os pixels vêm do `AssetDb`, por CONTEÚDO, e passam por uma cache** — ⛔ nunca de uma
/// leitura da GPU por quadro, e nunca de uma cópia por quadro: o `Asset` entrega um `Cow`, e
/// convertê-lo a cada quadro copiaria a imagem inteira 60 vezes por segundo. *É a mesma lição que
/// o `field3d_smoke_state` já escreveu ao lado do slot dele.*
///
/// ⛔⛔⛔ **E a cache guarda a IMAGEM ESTÁVEL, não os bytes** (report *«Smooth bugado quebrando a
/// forma»*, medido 2026-09-13). A 1.ª redacção guardava o `Arc` e desenhava cada peça pela porta
/// crua, que cunha um id do atlas **por chamada** ⇒ uma cópia inteira da imagem **por triângulo**,
/// por quadro, contra um atlas que pára em `8192²`. Medido sem ecrã, no `Resolver` (a decisão do
/// atlas é da CPU):
///
/// | imagem | peças | porta crua: peças que NÃO aparecem | estável |
/// |---|---:|---:|---:|
/// | `320×96` (o smoke) | `216` | `0` — e `25,3 MB` reenviados por quadro | `0` |
/// | `320×96` | `7 776` (o report) | **`5 651`** | `0` |
/// | `1024×1024` | `216` (o `Fast`!) | **`152`** | `0` |
///
/// ⇒ o report era o ATLAS, e o modo `Fast` partia com arte de tamanho comum. O gate é
/// `a_skinned_image_is_one_atlas_resident_however_many_pieces_and_frames` (o que a cena EMITE).
pub fn draw_skinned_images(
    sim: &SimWorld,
    asset_db: &ph2d_asset::AssetDb,
    cache: &mut SkinImageCache,
    cam: ph2d_vector::Affine,
    scene: &mut ph2d_vector::VectorScene,
    smooth: Option<ph2d_poly2d::RefineOptions>,
) -> usize {
    let alvos: Vec<(Entity, ph2d_asset::AssetId)> = sim
        .world()
        .iter_entities()
        .filter(|er| er.get::<ph2d_skeleton_ecs::SkinBind>().is_some())
        .filter(|er| er.get::<Sprite>().is_some())
        .filter_map(|er| er.get::<ph2d_ecs::SpritePixels>().map(|p| (er.id(), p.0)))
        .collect();
    // ⭐⭐⭐ **As malhas PRIMEIRO: o orçamento é do QUADRO, e reparte-se pelo que cada imagem guarda.**
    // Um tecto por imagem deixaria N imagens presas multiplicá-lo até ao buffer fixo do Vello
    // (`SKIN_FRAME_PIECES`) — e ali o quadro inteiro fica em branco, painéis incluídos.
    let malhas: Vec<(Entity, ph2d_asset::AssetId, Mesh2d)> = alvos
        .into_iter()
        .filter_map(|(e, id)| Some((e, id, mesh_of(sim, e)?)))
        .collect();
    let guardadas: usize = malhas.iter().map(|(_, _, m)| m.tris.len()).sum();
    let orcamento = smooth.map_or(SKIN_FRAME_PIECES, |o| o.max_pieces);
    if guardadas > orcamento {
        avisa_malhas_acima_do_orcamento(guardadas, orcamento);
    }
    let mut feitas = 0;
    for (e, id, mesh) in malhas {
        let Some((p2l, pele)) = deform_field(sim, e, mesh.size) else {
            continue;
        };
        let Some(imagem) = stable_image(asset_db, cache, id) else {
            continue;
        };
        // `local → ecrã`: a pose de mundo da sprite, depois a câmara.
        let mundo = ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(sim, e),
        );
        let to_screen = cam * affine_of(mundo);
        // ⭐⭐⭐ **A ALTERNATIVA `Smooth`** (report do dono, 2026-09-10).
        //
        // ⚠️⚠️ **A tolerância é em pixels de ECRÃ, e é por isso que ela passa pela câmara aqui:**
        // meia unidade local é meio pixel a zoom `1` e **quatro** a zoom `8`. *A suavidade que o
        // olho vê é um facto de espaço de ecrã* — uma tolerância em unidades locais afinaria a
        // malha para o zoom em que o artista não está.
        let (mesh, posed) = {
            let mut escrever = pele.scratch();
            let mut campo = |p: [f64; 2]| pele.point(p2l.apply(p), &mut escrever);
            match smooth {
                None => {
                    let posed: Vec<[f64; 2]> = mesh.rest.iter().map(|&p| campo(p)).collect();
                    (mesh, posed)
                }
                Some(o) => {
                    let escala = to_screen.determinant().abs().sqrt().max(f64::MIN_POSITIVE);
                    let antes = mesh.tris.len();
                    let parte = parte_do_orcamento(antes, guardadas, o.max_pieces);
                    let (m, p, k) = ph2d_poly2d::refine_posed(
                        &mesh,
                        &mut campo,
                        ph2d_poly2d::RefineOptions {
                            tolerance_px: o.tolerance_px / escala,
                            max_pieces: parte,
                        },
                    );
                    // ⚠️ **O diagnóstico da família** (`PH2D_BONE_LOG=1`): sem ele um report de
                    // *«partiu»* não distingue *quantas peças* de *que dobra* — e foi a CONTAGEM
                    // que se revelou a grandeza que importa.
                    if std::env::var_os("PH2D_BONE_LOG").is_some() {
                        eprintln!(
                            "[bone] pele suave: {antes} -> {} pecas (k={k}, parte {parte} de um \
                             orcamento de quadro {})",
                            m.tris.len(),
                            o.max_pieces
                        );
                    }
                    (m, p)
                }
            }
        };
        for &tri in &mesh.tris {
            let Some((pixel_to_local_deformado, alvo)) = triangle_xform(&mesh, &posed, tri) else {
                continue;
            };
            let mut p = ph2d_vector::BezPath::new();
            let pt = |q: [f64; 2]| {
                let s = to_screen * ph2d_vector::Point::new(q[0], q[1]);
                ph2d_vector::Point::new(s.x, s.y)
            };
            p.move_to(pt(alvo[0]));
            p.line_to(pt(alvo[1]));
            p.line_to(pt(alvo[2]));
            p.close_path();
            scene.push_clip(&p);
            scene.draw_stable_image_transformed(
                &imagem,
                to_screen * affine_of(pixel_to_local_deformado),
                ph2d_vector::ImageQuality::Medium,
            );
            scene.pop_layer();
        }
        feitas += 1;
    }
    feitas
}

/// A imagem de um `AssetId` como recurso ESTÁVEL do Vello — construída uma vez, clonada por peça.
///
/// ⚠️ **O clone é refcount + o MESMO id**, e o id é a unidade do atlas: é isto que faz as N peças
/// de um quadro, e os quadros seguintes, serem **um** residente em vez de N cópias da imagem.
fn stable_image(
    asset_db: &ph2d_asset::AssetDb,
    cache: &mut SkinImageCache,
    id: ph2d_asset::AssetId,
) -> Option<ph2d_vector::StableImage> {
    if let Some(imagem) = cache.get(&id) {
        return Some(imagem.clone());
    }
    let asset = asset_db.get(&id)?;
    let (w, h, cow) = asset.image_rgba8()?;
    let imagem = ph2d_vector::StableImage::from_rgba(std::sync::Arc::new(cow.into_owned()), w, h)?;
    cache.insert(id, imagem.clone());
    Some(imagem)
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

/// **A cache das imagens presas**, por CONTEÚDO — ver [`draw_skinned_images`].
///
/// ⚠️ Tem nome desde 2026-09-12 porque passou a ser um campo de uma struct de OUTRA crate
/// (`ph2d_app_skeleton::state::SkeletonState`): escrita por extenso lá, a família precisaria de
/// depender da `ph2d-asset` só para nomear a chave.
///
/// ⛔⛔ **O valor é um [`ph2d_vector::StableImage`], e NÃO os bytes** (2026-09-13). Guardar o `Arc`
/// e desenhar pela porta crua poupava a CÓPIA de CPU e cunhava um **id por peça**: o atlas guardava
/// uma cópia inteira da imagem por triângulo, e o que não cabia em `8192²` não era desenhado.
pub type SkinImageCache = std::collections::BTreeMap<ph2d_asset::AssetId, ph2d_vector::StableImage>;

fn affine_of(x: Xform) -> ph2d_vector::Affine {
    ph2d_vector::Affine::new(x.0)
}
