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

/// Um quadro de 60 fps, em microssegundos — o RECURSO de que o orçamento da pele é uma fatia.
const QUADRO_60FPS_US: usize = 16_667;

/// ⚠️ **A fatia do quadro que a pele pode gastar — e é a única ESCOLHA desta constante.** A pele é
/// uma coisa entre muitas no quadro (a arte do documento pelo Vello, o chrome, os passes de luz, o
/// resto das sprites); `1/10` deixa-lhe uma fatia visível sem lhe dar o quadro. Quem quiser medir
/// outra fatia tem o `PH2D_SKIN_PIECES`.
const FATIA_DA_PELE: usize = 10;

/// O custo MEDIDO de uma peça ENTREGUE com `Smooth`, em nanossegundos (a tabela do
/// [`SKIN_FRAME_PIECES`]).
const CUSTO_POR_PECA_NS: usize = 353;

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
/// ⭐ **O que UMA peça custa, MEDIDO OUTRA VEZ em 2026-09-16** (a lei do refinamento mudou, logo o
/// número tinha de ser remedido; `load 5,6`–`6,1`, o MÍNIMO de 40/60 corridas, **três** corridas com
/// leituras entre `0,328` e `0,355` — as sondas são
/// [`tests::custo::measure_the_cpu_cost_of_a_skinned_frame`] e a
/// `ph2d-render::sprite_mesh_gpu::measure_the_frame_cost_of_a_mesh_sprite`):
///
/// | o que o quadro faz por peça | µs |
/// |---|---:|
/// | descodificar a malha guardada (postcard, **por quadro**) | `0,014` |
/// | `Fast`: descodificar + deformar + montar o `SpriteMesh` | `0,025` |
/// | recolher + costurar a tira + enviar + DESENHAR (marginal, GPU esperada) | `0,013` |
/// | `Smooth` **uniforme**: o quadro inteiro, por peça entregue | `0,087` |
/// | **`Smooth` ADAPTATIVO: o quadro inteiro, por peça ENTREGUE** | **`0,340`** |
///
/// ⇒ `16,667 ms ÷ 10 ÷ 0,353 µs` = **`4 721` peças**.
///
/// ⛔⛔⛔ **E O NÚMERO DE ANTES (`1 080 ns` ⇒ `1 543` peças) ERA DE UM CAMINHO QUE O PRODUTO NUNCA
/// CORREU.** Ele foi medido em 2026-09-13 sobre um `Smooth` que **refinava**; desde então mediu-se
/// que a lei uniforme é **inerte** acima de `orçamento / 4` peças, logo o que o produto de facto
/// pagava era o `Fast` mais o custo de decidir. *Um custo medido sobre um caminho que não corre é
/// um orçamento que mente nos dois sentidos* — e este mentia para BAIXO, o que fazia a malha de
/// bind de `2 430` peças disparar o [`avisa_malhas_acima_do_orcamento`] em toda a execução.
///
/// ⚠️⚠️ **A lei nova é `3,9×` mais cara POR PEÇA** (`0,340` contra `0,087`), e isso é o preço de
/// ela decidir: ela mede o desvio de cada aresta, mantém o livro de donos e escolhe. *O que ela
/// compra em troca é entregar alguma coisa* — a uniforme era barata porque não fazia nada.
///
/// ⭐ **E o livro de contas já foi medido e cortado uma vez:** a 1.ª redacção usava DOIS mapas e um
/// `Vec` por aresta, e lia `0,52 µs`; com um mapa só e os donos num par fixo desceu a `0,34`
/// (`−35 %`). Ver [`ph2d_poly2d::refine_adaptive`].
pub const SKIN_FRAME_PIECES: usize = QUADRO_60FPS_US * 1_000 / FATIA_DA_PELE / CUSTO_POR_PECA_NS;

/// ⛔⛔ **A OUTRA PONTA DO TECTO, verificada na COMPILAÇÃO.** Um tecto apertado de mais deixa de
/// refinar uma malha comum, e a barra sai da malha que o produto **de facto** guarda: a do bind de
/// uma arte normal mede `~2 430` peças (medido 2026-09-15), então um orçamento abaixo disso não
/// deixa a lei adaptativa partir **uma** aresta que seja. Uma fatia mais fina (ou um custo por peça
/// maior, medido outra vez) tem de PARAR a build aqui, e não passar em silêncio.
///
/// ⚠️ A barra anterior era `1 000` e vinha de `k = 2` sobre uma malha de `~200` peças — a aritmética
/// da lei uniforme, que já não é a do produto.
const _: () = assert!(
    SKIN_FRAME_PIECES >= 2_500,
    "o orcamento da pele nao chega para refinar a malha que o bind de facto guarda"
);

/// ⭐⭐⭐ **OS NÚMEROS DO `Smooth`**: a tolerância da `ph2d-poly2d`, o orçamento do QUADRO e a LEI.
///
/// ⚠️ **`PH2D_SKIN_PIECES=<n>` afina o orçamento do QUADRO**, e não de uma imagem — ele existe desde
/// o report *«Smooth bugado quebrando a forma»* (dono, 2026-09-10), cuja causa se mediu depois
/// (a porta crua enchia o atlas, F6-e). *Um smoke que MEDE vale mais que um smoke que pergunta.*
///
/// ⭐⭐⭐ **`PH2D_SKIN_REFINE=uniforme` volta à lei do `k` global**, para bissecar. Ela é o que o
/// `Smooth` usou até 2026-09-16 e é **provadamente inerte** acima de `orçamento / 4` peças — ver
/// [`ph2d_poly2d::refine_adaptive`].
#[must_use]
pub fn refine_options() -> RefineOptions {
    static PECAS: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let escolhido = *PECAS.get_or_init(|| {
        std::env::var("PH2D_SKIN_PIECES")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|n| *n > 0)
    });
    static LEI: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let adaptativo = *LEI.get_or_init(|| {
        !std::env::var("PH2D_SKIN_REFINE").is_ok_and(|v| v.trim().eq_ignore_ascii_case("uniforme"))
    });
    RefineOptions {
        max_pieces: escolhido.unwrap_or(SKIN_FRAME_PIECES),
        adaptativo,
        ..RefineOptions::default()
    }
}

/// ⭐ **A parte do orçamento do quadro que cabe a uma imagem** — proporcional à malha que ela guarda,
/// e nunca abaixo dela (a malha guardada é o desenho mínimo; não há como desenhá-la com menos).
///
/// ⚠️ **Proporcional dá a todas o MESMO factor de crescimento**, logo a mesma qualidade relativa:
/// cada imagem pode chegar a `orçamento × (peças dela / peças de todas)`, e essa razão é a mesma
/// para todas. *Repartir por igual daria à imagem pequena um luxo que a grande não tem.*
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
/// ⭐⭐⭐ **E `pesos` é a tabela do PADRÃO-OURO guardada no bind** — vazia ⇒ a lei derivada.
///
/// ⚠️⚠️ **Ela viaja pelo REFINAMENTO, e é isso que a torna utilizável no `Smooth`:** um vértice que
/// a subdivisão inventa não tem peso guardado, e a única resposta certa é o baricêntrico do
/// triângulo que o gerou ([`ph2d_poly2d::refine_posed_attrs`]). ⛔ Localizar o ponto na malha seria
/// `O(n)` por ponto para chegar à mesma resposta que a proveniência já sabe de graça.
#[must_use]
pub fn posed_sprite_mesh(
    mesh: Mesh2d,
    p2l: Xform,
    pele: &ph2d_skeleton::Skin,
    pesos: &[f64],
    anchor: [f32; 2],
    size: [f32; 2],
    refine: Option<RefineOptions>,
) -> Option<(SpriteMesh, ph2d_poly2d::RefineReport)> {
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
    let (mesh, posed, relatorio) = match refine {
        None => {
            let posed: Vec<[f64; 2]> = mesh
                .rest
                .iter()
                .enumerate()
                .map(|(v, &q)| {
                    let w = pesos.get(v * ossos..(v + 1) * ossos).unwrap_or(&[]);
                    campo(q, w)
                })
                .collect();
            let pecas = mesh.tris.len();
            (
                mesh,
                posed,
                ph2d_poly2d::RefineReport {
                    pecas,
                    // ⚠️ O `Fast` não MEDE desvio nenhum — ele não refina, logo não pergunta. `None`
                    // di-lo; um `0.0` aqui seria um número a afirmar que a malha está perfeita.
                    desvio: None,
                    travado_pelo_orcamento: false,
                    lei: ph2d_poly2d::RefineLaw::Uniform { k: 1 },
                },
            )
        }
        Some(o) => {
            let (m, p, _, r) = ph2d_poly2d::refine_posed_attrs(&mesh, pesos, ossos, &mut campo, o);
            (m, p, r)
        }
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
        relatorio,
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
/// ⭐⭐⭐ **`suspensa` é a sprite que NÃO recebe malha neste quadro** — hoje, aquela que o Painter
/// está a editar (ordem do dono, 2026-09-15: *«se o usuário entrar no modo Painter em imagem
/// deformada por ossos a imagem deixa a deformação para ser pintada; ao sair, ela retorna»*).
///
/// ⚠️ **Esta folha não sabe o que é um Painter, e é assim que tem de ser:** quem decide é quem
/// chama ([`ph2d_app_painter::skin_suspend::sprite_achatada`]), e o parâmetro diz **o quê**, nunca
/// **porquê**. ⭐ O filtro é o PRIMEIRO da cadeia, logo a malha suspensa nem chega a ser
/// descodificada — a suspensão é mais barata que a deformação, não mais cara.
///
/// ⛔ **O regresso é por construção:** isto corre a cada quadro, então deixar de suspender devolve
/// a deformação sozinho. *Não há estado a repor, logo não há como ficar preso achatado.*
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
    suspensa: Option<u64>,
) -> usize {
    let presas: Vec<(Entity, SkinnedMesh)> = sim
        .world()
        .iter_entities()
        .filter(|er| is_skinned_image(sim.world(), er.id()) && Some(er.id().to_bits()) != suspensa)
        .filter_map(|er| Some((er.id(), skinned_mesh_of(sim, er.id())?)))
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
    let vivas: Vec<(Entity, Entity, SkinnedMesh)> = presas
        .into_iter()
        .filter_map(|(e, m)| Some((e, *instancias.get(&e)?, m)))
        .collect();
    let guardadas: usize = vivas.iter().map(|(_, _, m)| m.mesh.tris.len()).sum();
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
        let Some((p2l, pele)) = deform_field(sim, e, mesh.mesh.size, pixels_per_meter) else {
            continue;
        };
        let antes = mesh.mesh.tris.len();
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
                // ⚠️ A LEI vem de quem chamou (o `refine_options`) — só a tolerância e o orçamento
                // é que são factos DESTE quadro.
                adaptativo: o.adaptativo,
            }
        });
        let SkinnedMesh { mesh, pesos } = mesh;
        let Some((malha, rel)) =
            posed_sprite_mesh(mesh, p2l, &pele, &pesos, inst.anchor, inst.size, refine)
        else {
            continue;
        };
        // ⚠️ **O diagnóstico da família** (`PH2D_BONE_LOG=1`): sem ele um report de *«partiu»* não
        // distingue *quantas peças* de *que dobra* — e foi a CONTAGEM que se revelou a grandeza que
        // importa.
        if let (Some(o), true) = (refine, std::env::var_os("PH2D_BONE_LOG").is_some()) {
            eprintln!(
                "[bone] pele suave: {antes} -> {} pecas ({:?}, desvio {}, de um orcamento de \
                 quadro {})",
                malha.tris.len(),
                rel.lei,
                rel.desvio
                    .map_or_else(|| "nao medido".to_owned(), |d| format!("{d:.3} px locais")),
                o.max_pieces
            );
            // ⭐⭐⭐ **A LINHA QUE FALTAVA: «não refinou» tem DUAS causas e elas são opostas.**
            //
            // ⛔⛔ Ou a dobra não pediu refinamento nenhum (tudo bem), ou o ORÇAMENTO o proibiu —
            // e nesse caso o `Smooth` é o `Fast` **ao bit**, com o painel a dizer que está ligado.
            // Medido 2026-09-15 na cena do osso, com a lei UNIFORME: `780` peças com orçamento
            // `1 543` ⇒ `780 × 2² = 3 120 > 1 543` ⇒ `max_split = 1`, e a tolerância nunca decidia
            // nada. *Um diagnóstico que imprime o mesmo número para «não precisou» e para «não
            // pôde» cala exactamente a pergunta que o report do dono fazia.*
            if rel.travado_pelo_orcamento {
                eprintln!(
                    "[bone]   ⚠ o REFINAMENTO PAROU NO ORCAMENTO: {antes} -> {} pecas de um tecto \
                     de {} -- a tolerancia pedida nao foi alcancada",
                    malha.tris.len(),
                    o.max_pieces
                );
            }
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
