//! ⭐⭐⭐ **UMA SPRITE DESENHADA COMO MALHA** — o primitivo que põe a imagem presa ao esqueleto
//! DENTRO do passe de sprites (plano `docs/Skeleton/03_plano_a_pele_no_passe_de_sprites.md`).
//!
//! # Porque não há pipeline nova
//!
//! O `vs_main` do `sprite.wgsl` calcula tudo a partir de dois atributos por vértice e da
//! instância: `local = anchor + quad_pos · size`, `world = world_pos + basis · local`, e a UV sai
//! do `quad_uv` (espelhado, repetido, `uv_xform`, `mix(atlas_uv)`). ⇒ um vértice da malha leva o
//! `quad_uv` DE REPOUSO e o `quad_pos` que devolve a posição POSADA, e a malha herda tinta,
//! opacidade, mistura, pré-multiplicação, repetição e o shader de marca do recorte — tudo o que
//! uma sprite tem, sem uma linha de WGSL.
//!
//! As 10 pipelines são `TriangleStrip` sem culling ⇒ `N` triângulos entram como UMA tira com
//! degenerados de ligação: `5N − 2` vértices.
//!
//! # ⭐⭐ Porque não há costuras
//!
//! Dois triângulos que partilham uma aresta são rasterizados pela regra de canto: cada centro de
//! pixel pertence a UM deles. Não há anti-aliasing por aresta a compor `1 − a·b`, nem faixa
//! desenhada duas vezes — que é o que o caminho do Vello (um recorte por triângulo) não conseguia
//! (`tests/it/skin_pieces_gpu_cost.rs`).
//!
//! # ⭐⭐ Quem mais precisa de saber que uma sprite é uma malha (W3)
//!
//! A malha vive num componente AO LADO da instância, então todo consumidor que COPIA a instância ou
//! lê o quad dela via a sprite em repouso: o vidro do prefab e o emissivo copiam-na
//! ([`LiftedInstances`]), e o picking e as caixas lêem o quad (`crate::picking`). ⇒ as duas perguntas
//! *«isto desenha-se como malha?»* ([`drawn_mesh`]) e *«que texel está debaixo deste ponto?»*
//! ([`uv_under`]) moram AQUI, uma vez, para o desenho e para quem aponta.
//!
//! # ⚠️ Divergência DECLARADA: a tinta por canto
//!
//! O shader calcula a tinta por canto POR VÉRTICE (bilinear sobre o `quad_uv`) e interpola-a. O
//! quad tem 4 vértices e interpola-a em dois triângulos; uma malha fina aproxima-a do bilinear
//! verdadeiro. Com a tinta por canto uniforme — quase toda sprite — os dois coincidem.

use crate::sprite::{QuadVertex, RenderInstance};
use ph2d_ecs::{Entity, PresentWorld, SimRef};
use ph2d_gpu::GpuContext;

/// ⭐ **A malha de uma sprite, posada** — componente de APRESENTAÇÃO, na mesma entidade da
/// [`RenderInstance`] que ela substitui.
///
/// - `local`: a posição POSADA de cada vértice, em metros no espaço LOCAL da sprite (o espaço em que
///   o quad de repouso é `anchor ± size/2`).
/// - `uv`: a coordenada DE REPOUSO de cada vértice na imagem, `0..1`, com `v = 0` em cima (a
///   convenção do [`QuadVertex::QUAD_STRIP`]).
/// - `tris`: triângulos por índice em `local`/`uv`.
///
/// ⚠️ **Se a malha não puder ser desenhada** (comprimentos diferentes, `size` zero, nenhum triângulo
/// válido), a instância desenha o QUAD de repouso — visível, nunca calada. A pergunta é
/// [`drawn_mesh`].
#[derive(bevy_ecs::component::Component, Debug, Default, PartialEq)]
pub struct SpriteMesh {
    pub local: Vec<[f32; 2]>,
    pub uv: Vec<[f32; 2]>,
    pub tris: Vec<[u32; 3]>,
    /// ⭐⭐⭐ **A PELE, quando a PLACA é que posa** ([`SpriteMeshSkin`], F9 W2).
    ///
    /// ⚠️⚠️ **Ela muda o que o [`Self::local`] SIGNIFICA, e é a coisa mais importante deste
    /// componente:** com `None` ele traz as posições **POSADAS** (o caminho de sempre, e o de toda
    /// malha que não é uma pele); com `Some`, o **REPOUSO** — quem pousa é o shader, e quem
    /// responde na CPU é o [`Self::posado`]. ⛔ Um leitor que espere posições posadas e receba
    /// repouso não falha: ele desenha o anel onde a arte **não** está. É por isso que a pergunta
    /// tem uma porta só.
    pub skin: Option<crate::sprite_mesh_skin::SpriteMeshSkin>,
}

/// ⚠️ **`Clone` à mão por causa do `clone_from`:** o derivado recria os três `Vec` a cada cópia, e o
/// [`LiftedInstances`] copia malhas A CADA QUADRO para buffers reusados (HR-3).
impl Clone for SpriteMesh {
    fn clone(&self) -> Self {
        Self {
            local: self.local.clone(),
            uv: self.uv.clone(),
            tris: self.tris.clone(),
            skin: self.skin.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.local.clone_from(&source.local);
        self.uv.clone_from(&source.uv);
        self.tris.clone_from(&source.tris);
        self.skin.clone_from(&source.skin);
    }
}

impl SpriteMesh {
    /// ⭐⭐⭐ **ESTA MALHA, COM AS POSIÇÕES ONDE A ARTE DE FACTO ESTÁ** — a porta ÚNICA das dez
    /// costuras (F9 W3).
    ///
    /// ⛔⛔ **Sem pele ela devolve `self` EMPRESTADO — zero cópia e zero aritmética**, que é o que
    /// mantém o caminho de toda malha que não é uma pele byte-idêntico *por construção*. Com pele
    /// ela posa pela [`SpriteMeshSkin::posa`], que é a **mesma** lei que o shader corre.
    ///
    /// ⚠️ **O custo mudou de sítio, e é esse o ponto da wave:** antes a CPU posava **todo quadro,
    /// para desenhar**; agora posa **só quando alguém pergunta** — o ponteiro sobre a arte, a caixa
    /// do gizmo, os fantasmas do onion. Num quadro em que ninguém pergunta, ela não corre.
    ///
    /// ⚠️ **A malha devolvida não tem pele** (`skin: None`): ela JÁ está posada, e deixar lá a
    /// tabela convidaria alguém a posá-la duas vezes.
    #[must_use]
    pub fn posado(&self) -> std::borrow::Cow<'_, Self> {
        let Some(skin) = self.skin.as_ref().filter(|s| s.valida(self.local.len())) else {
            return std::borrow::Cow::Borrowed(self);
        };
        std::borrow::Cow::Owned(Self {
            local: self
                .local
                .iter()
                .enumerate()
                .map(|(v, &p)| skin.posa(v, p))
                .collect(),
            uv: self.uv.clone(),
            tris: self.tris.clone(),
            skin: None,
        })
    }

    /// ⭐ **A UV de um vértice que, EM REPOUSO, está no ponto local `local`** — a do QUAD naquele ponto.
    ///
    /// É a inversa da lei do shader (`local = anchor + quad_pos · size`) com `uv = (qx + ½, ½ − qy)`,
    /// a convenção do [`QuadVertex::QUAD_STRIP`]. Um vértice cuja UV sai daqui lê, em repouso, o mesmo
    /// texel que o quad leria ali — e o espelho, a repetição e o `uv_xform`, que o shader aplica a
    /// esta UV, tratam a malha e o quad da mesma maneira.
    ///
    /// ⚠️ **Mora aqui, na crate do shader, e não em quem prende a imagem:** uma segunda convenção de
    /// `v` escrita noutra crate seria a segunda resposta à mesma pergunta.
    ///
    /// `None` com um lado do `size` nulo ou não finito.
    #[must_use]
    pub fn uv_at(local: [f32; 2], anchor: [f32; 2], size: [f32; 2]) -> Option<[f32; 2]> {
        quad_pos(local, anchor, size).map(|q| [q[0] + 0.5, 0.5 - q[1]])
    }

    /// Os triângulos que o passe DESENHA, em local — os de índices dentro da malha, pela ordem da
    /// tira (a mesma regra do [`stitch`], que salta os outros).
    pub(crate) fn triangles(&self) -> impl DoubleEndedIterator<Item = [usize; 3]> + '_ {
        let n = self.local.len().min(self.uv.len());
        self.tris
            .iter()
            .map(|t| [t[0] as usize, t[1] as usize, t[2] as usize])
            .filter(move |t| t.iter().all(|&i| i < n))
    }
}

/// ⭐⭐ **A malha que o passe de sprites DESENHA no lugar do quad desta instância — ou `None`, e aí
/// desenha-se o quad.**
///
/// ⚠️ **Uma pergunta, dois consumidores:** o [`MeshFrame::push`] (o desenho) e o `crate::picking`
/// (quem aponta). Uma malha que o picking lesse e o passe recusasse seria apontável onde não se vê.
#[must_use]
pub(crate) fn drawn_mesh(
    mesh: Option<&SpriteMesh>,
    size: [f32; 2],
) -> Option<std::borrow::Cow<'_, SpriteMesh>> {
    desenhavel(mesh, size).map(SpriteMesh::posado)
}

/// ⭐⭐⭐ **A MESMA pergunta, SEM posar** — o que o desenho precisa e o picking não.
///
/// ⚠️⚠️ **A separação nasceu com a F9 W2 e ela é a coisa que impede a wave de se pagar a si mesma:**
/// o [`MeshFrame::push`] só quer saber *«isto é desenhável?»* e entrega o REPOUSO à placa. Se ele
/// passasse pelo [`drawn_mesh`], a CPU posaria a malha inteira **por quadro** para deitar fora o
/// resultado — exactamente o custo que esta wave existe para remover.
#[must_use]
pub(crate) fn desenhavel(mesh: Option<&SpriteMesh>, size: [f32; 2]) -> Option<&SpriteMesh> {
    let m = mesh?;
    let quad_ok = quad_pos([0.0, 0.0], [0.0, 0.0], size).is_some();
    (quad_ok && m.local.len() == m.uv.len() && m.triangles().next().is_some()).then_some(m)
}

/// A margem de arredondamento de um peso baricêntrico em `f32` — duas subtracções e uma divisão, a
/// poucos ULP. Sem ela um ponto SOBRE a aresta partilhada por dois triângulos podia não pertencer a
/// nenhum dos dois, e o picking teria uma fenda que o desenho não tem.
const BORDA_BARICENTRICA: f32 = 8.0 * f32::EPSILON;

/// Os pesos baricêntricos de `p` no triângulo `t`, com a borda INCLUÍDA; `None` fora dele ou num
/// triângulo sem área (que o rasterizador não desenha).
pub(crate) fn barycentric(p: [f32; 2], t: [[f32; 2]; 3]) -> Option<[f32; 3]> {
    let w = barycentric_raw(p, t)?;
    w.iter().all(|&x| x >= -BORDA_BARICENTRICA).then_some(w)
}

/// Os mesmos pesos **sem a cerca**: fora do triângulo eles saem negativos, e é isso que EXTRAPOLA
/// o afim dele para um ponto de fora. ⚠️ Só quem já sabe qual é o triângulo certo pode usá-los —
/// quem PROCURA um triângulo usa a [`barycentric`], que os recusa.
pub(crate) fn barycentric_raw(p: [f32; 2], t: [[f32; 2]; 3]) -> Option<[f32; 3]> {
    let [a, b, c] = t;
    let menos = |x: [f32; 2], y: [f32; 2]| [x[0] - y[0], x[1] - y[1]];
    let cruz = |u: [f32; 2], v: [f32; 2]| u[0] * v[1] - u[1] * v[0];
    let area = cruz(menos(b, a), menos(c, a));
    if area.is_nan() || area.abs() < f32::MIN_POSITIVE {
        return None;
    }
    let w1 = cruz(menos(p, a), menos(c, a)) / area;
    let w2 = cruz(menos(b, a), menos(p, a)) / area;
    Some([1.0 - w1 - w2, w1, w2])
}

/// ⭐ **Os três vértices LOCAIS de um triângulo da malha** (os índices já vieram de
/// [`SpriteMesh::triangles`]).
pub(crate) fn corners(mesh: &SpriteMesh, t: [usize; 3]) -> [[f32; 2]; 3] {
    [mesh.local[t[0]], mesh.local[t[1]], mesh.local[t[2]]]
}

/// ⭐ **Os três vértices do mesmo triângulo em UV DE REPOUSO** — o gémeo de [`corners`], e os dois
/// juntos são o afim que aquele triângulo aplica.
pub(crate) fn uv_corners(mesh: &SpriteMesh, t: [usize; 3]) -> [[f32; 2]; 3] {
    [mesh.uv[t[0]], mesh.uv[t[1]], mesh.uv[t[2]]]
}

/// ⭐⭐ **O ponto LOCAL `p` cai sobre a malha posada?**
#[must_use]
pub(crate) fn covers(mesh: &SpriteMesh, p: [f32; 2]) -> bool {
    mesh.triangles()
        .any(|t| barycentric(p, corners(mesh, t)).is_some())
}

/// ⭐⭐⭐ **A DEFORMAÇÃO LOCAL de um triângulo** — a `2×2` adimensional que leva um deslocamento na
/// textura ao deslocamento que ele ocupa no ECRÃ, e **a identidade em repouso**.
///
/// ⛔⛔ É o que faltava ao pincel (report do dono, 2026-09-14, com foto): a posição já era a certa e
/// a FORMA não — onde o leque comprime a arte, um disco de textura chega ao ecrã como uma lasca.
/// Quem consome isto é a [`ph2d_painter_brush::canvas_warp`], que a inverte para saber que elipse
/// pintar; quem escolhe o triângulo (ou COMPÕE vários) é a [`crate::sprite_mesh_warp::warp_over`].
///
/// A conta é o triângulo: com `A = [P₁−P₀, P₂−P₀]` (local) e `B = [U₁−U₀, U₂−U₀]` (uv), o jacobiano
/// `d(local)/d(uv)` é `A·B⁻¹`; dividido pelo do QUAD de repouso (`diag(sw, −sh)`, que é o que a lei
/// do quad usa) sobra a deformação pura. ⚠️ **A divisão pelo quad é o que a torna adimensional** —
/// sem ela o número carregaria o tamanho da sprite e o pincel mudaria de forma ao redimensioná-la.
///
/// ⚠️ **UMA conta, e é esta:** ela viveu emparelhada com a busca do triângulo até 2026-09-14, e a
/// wave do footprint precisou de a aplicar a triângulos que não são o do ponto. Duas cópias desta
/// álgebra seriam duas respostas à mesma pergunta — e a W11b mostrou o que custa errar a base numa
/// delas. `None` num triângulo degenerado (`det(B) ≈ 0`) ou com `size` não positivo.
#[must_use]
pub(crate) fn warp_of(mesh: &SpriteMesh, t: [usize; 3], size: [f32; 2]) -> Option<[[f32; 2]; 2]> {
    if size[0] <= 0.0 || size[1] <= 0.0 {
        return None;
    }
    let c = corners(mesh, t);
    let uv = uv_corners(mesh, t);
    let b = [
        [uv[1][0] - uv[0][0], uv[2][0] - uv[0][0]],
        [uv[1][1] - uv[0][1], uv[2][1] - uv[0][1]],
    ];
    let det = b[0][0] * b[1][1] - b[0][1] * b[1][0];
    if !det.is_finite() || det.abs() < 1e-12 {
        return None;
    }
    let a = [
        [c[1][0] - c[0][0], c[2][0] - c[0][0]],
        [c[1][1] - c[0][1], c[2][1] - c[0][1]],
    ];
    // `A · B⁻¹`
    let inv = [
        [b[1][1] / det, -b[0][1] / det],
        [-b[1][0] / det, b[0][0] / det],
    ];
    let j = [
        [
            a[0][0] * inv[0][0] + a[0][1] * inv[1][0],
            a[0][0] * inv[0][1] + a[0][1] * inv[1][1],
        ],
        [
            a[1][0] * inv[0][0] + a[1][1] * inv[1][0],
            a[1][0] * inv[0][1] + a[1][1] * inv[1][1],
        ],
    ];
    // ⚠️⚠️ **As DUAS pontas na mesma base, e é aqui que a 1.ª redacção errou:** as LINHAS de `j`
    // estão em coordenadas locais (`y` para CIMA) e as COLUNAS em `uv` (`v` para BAIXO). A
    // resposta tem de estar em coordenadas de ECRÃ nos dois lados — senão a matriz nasce numa
    // base MISTA, que nega os termos fora da diagonal: uma arte RODADA recebia a elipse
    // espelhada, esticada na diagonal errada (report do dono: *«sem melhorias»*).
    // ⛔ E as fixturas alinhadas aos eixos **não o viam**: ali os termos fora da diagonal são
    // zero. ⇒ `D · j · diag(1/sw, 1/sh)`, com `D` a espelhar a linha do `y`.
    Some([
        [j[0][0] / size[0], j[0][1] / size[1]],
        [-j[1][0] / size[0], -j[1][1] / size[1]],
    ])
}

/// ⭐⭐ **A UV DE REPOUSO debaixo do ponto LOCAL `p`** — interpolada no triângulo posado que o
/// contém; `None` fora da malha.
///
/// ⚠️ **Numa dobra que sobrepõe a malha a si mesma, ganha o triângulo desenhado POR ÚLTIMO** — é o
/// que fica à vista, porque a tira desenha-os pela ordem e o de cima cobre o de baixo.
#[must_use]
pub(crate) fn uv_under(mesh: &SpriteMesh, p: [f32; 2]) -> Option<[f32; 2]> {
    mesh.triangles().rev().find_map(|t| {
        let w = barycentric(p, corners(mesh, t))?;
        let uv = [mesh.uv[t[0]], mesh.uv[t[1]], mesh.uv[t[2]]];
        Some([
            w[0] * uv[0][0] + w[1] * uv[1][0] + w[2] * uv[2][0],
            w[0] * uv[0][1] + w[1] * uv[1][1] + w[2] * uv[2][1],
        ])
    })
}

/// ⭐⭐⭐ **O PONTO LOCAL onde a malha posada desenha o texel de UV de repouso `uv`** — o gémeo de
/// [`uv_under`], na direcção contrária.
///
/// ⛔⛔ **Ele nasceu porque quem desenha POR CIMA da arte não sabia onde a arte está** (2026-09-15):
/// a [`crate::mesh_uv`] curou quem APONTA, e as alças do editor de curva do Painter continuavam a
/// ser pintadas pelo afim do quad de REPOUSO — logo o artista clicava num sítio e a marca aparecia
/// noutro, *ou* via o ponto de controlo longe da tinta que ele próprio acabara de pousar. ⚠️ **Uma
/// lei escrita só na direcção de quem aponta ainda não é uma lei:** um controlo que se desenha por
/// um mapa e se agarra por outro é um controlo morto sob o dedo.
///
/// ⭐ **Aqui NÃO há ambiguidade de dobra, e é por construção:** a malha de repouso é uma PARTIÇÃO
/// da imagem (dois triângulos não partilham texel), então um `uv` cai em UM triângulo. É a pergunta
/// inversa — *que texel está debaixo deste ponto?* — que tem de escolher o desenhado por último.
///
/// `None` fora da malha: o chamador fica com a lei do quad, que é o que a [`crate::MeshUv`] já faz
/// do outro lado.
#[must_use]
pub(crate) fn local_at_uv(mesh: &SpriteMesh, uv: [f32; 2]) -> Option<[f32; 2]> {
    mesh.triangles().find_map(|t| {
        let w = barycentric(uv, uv_corners(mesh, t))?;
        let c = corners(mesh, t);
        Some([
            w[0] * c[0][0] + w[1] * c[1][0] + w[2] * c[2][0],
            w[0] * c[0][1] + w[1] * c[1][1] + w[2] * c[2][1],
        ])
    })
}

/// ⭐⭐ **INSTÂNCIAS COPIADAS DA CENA, cada uma com a malha que tinha** — o que o vidro do prefab e o
/// emissivo re-desenham em isolamento ([`crate::SpriteRenderer::render_lifted_instances`]).
///
/// ⛔ **Uma cópia do `RenderInstance` sozinha perde a malha**, que vive noutro componente da mesma
/// entidade: até à W3 do plano 03 uma imagem presa ao esqueleto aparecia DEFORMADA na cena e em
/// REPOUSO no halo emissivo e por cima do vidro da receita aberta.
///
/// ⚠️ **Reusado entre quadros** (HR-3): o `clear` guarda a capacidade, e as malhas copiam-se por
/// `clone_from` para os `Vec` do quadro anterior.
#[derive(Default)]
pub struct LiftedInstances {
    instances: Vec<RenderInstance>,
    /// `(índice em instances, malha)`; só as primeiras `n_meshes` são deste quadro.
    meshes: Vec<(usize, SpriteMesh)>,
    n_meshes: usize,
}

impl LiftedInstances {
    pub fn clear(&mut self) {
        self.instances.clear();
        self.n_meshes = 0;
    }

    /// Acrescenta uma instância, com a malha dela quando ela tem uma.
    pub fn push(&mut self, inst: RenderInstance, mesh: Option<&SpriteMesh>) {
        if let Some(m) = mesh {
            let i = self.instances.len();
            match self.meshes.get_mut(self.n_meshes) {
                Some(slot) => {
                    slot.0 = i;
                    slot.1.clone_from(m);
                }
                None => self.meshes.push((i, m.clone())),
            }
            self.n_meshes += 1;
        }
        self.instances.push(inst);
    }

    /// ⭐⭐ **A porta dos consumidores:** limpa, e recolhe da cena cada instância que `keep` aceita —
    /// `keep` recebe a entidade da SIMULAÇÃO e pode alterar a cópia (o emissivo multiplica a tinta) —,
    /// COM a malha dela. Um consumidor que passe por aqui não pode esquecer a malha.
    pub fn collect_from(
        &mut self,
        present: &mut PresentWorld,
        mut keep: impl FnMut(Entity, &mut RenderInstance) -> bool,
    ) {
        self.clear();
        let mut q = present
            .world_mut()
            .query::<(&RenderInstance, &SimRef, Option<&SpriteMesh>)>();
        for (inst, sim_ref, malha) in q.iter(present.world()) {
            let mut copia = *inst;
            if keep(sim_ref.0, &mut copia) {
                self.push(copia, malha);
            }
        }
    }

    #[must_use]
    pub fn instances(&self) -> &[RenderInstance] {
        &self.instances
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// A malha que a instância `index` levou, se levou.
    #[must_use]
    pub fn mesh_of(&self, index: usize) -> Option<&SpriteMesh> {
        self.meshes()
            .iter()
            .find(|(i, _)| *i == index)
            .map(|(_, m)| m)
    }

    pub(crate) fn meshes(&self) -> &[(usize, SpriteMesh)] {
        &self.meshes[..self.n_meshes]
    }
}

/// ⭐ **Marca cada instância de uma fatia isolada com a malha que ela levou, e limpa a marca das
/// outras** — ANTES da ordenação, porque os índices são os da fatia.
///
/// ⛔ Uma fatia crua (`meshes` vazio, o glow do Motion) sai toda sem marca: uma marca herdada
/// indexaria as malhas de OUTRA chamada de render.
pub(crate) fn tag_lifted(
    scratch: &mut [RenderInstance],
    frame: &mut MeshFrame,
    meshes: &[(usize, SpriteMesh)],
) {
    frame.clear();
    for inst in scratch.iter_mut() {
        clear_mesh_tag(inst);
    }
    for (i, m) in meshes {
        if let Some(inst) = scratch.get_mut(*i) {
            inst.flip_uv |= frame.push(m, inst.anchor, inst.size) << RenderInstance::MESH_SHIFT;
        }
    }
}

/// As malhas de UMA chamada de render: os vértices costurados e o intervalo de cada uma.
#[derive(Default)]
pub(crate) struct MeshFrame {
    pub(crate) vertices: Vec<QuadVertex>,
    pub(crate) ranges: Vec<(u32, u32)>,
    /// Scratch dos vértices de UMA malha antes da costura (reutilizado entre malhas).
    work: Vec<QuadVertex>,
    /// ⭐⭐⭐ **A PELE, PARALELA ao [`Self::vertices`]** (F9 W2) — o vértice `j` da tira lê
    /// `pesos[j]`/`ossos[j]`, que é o que o `@builtin(vertex_index)` dá de graça numa chamada
    /// não-indexada. ⛔ Um `@location` novo era inexprimível: os `0..15` do dispositivo estão cheios.
    pub(crate) pesos: Vec<[f32; crate::sprite_mesh_skin::OSSOS_POR_VERTICE]>,
    pub(crate) ossos: Vec<[u32; crate::sprite_mesh_skin::OSSOS_POR_VERTICE]>,
    /// Os afins de TODAS as malhas deste quadro, concatenados e **já conjugados para o quad** de
    /// cada instância — ver [`conjuga_para_o_quad`].
    pub(crate) afins: Vec<crate::sprite_mesh_skin_gpu::SkinAfimGpu>,
    /// As tabelas `n × n` de juntas de TODAS as malhas deste quadro, concatenadas e **já
    /// conjugadas para o quad** de cada instância — ver [`conjuga_ponto_para_o_quad`].
    ///
    /// ⚠️ **Cada registo de osso diz onde a tabela DELE começa** (`info.z`), porque a concatenação
    /// não é uniforme: duas malhas do quadro podem ter contagens de osso diferentes.
    pub(crate) juntas: Vec<[f32; 2]>,
}

impl MeshFrame {
    pub(crate) fn clear(&mut self) {
        self.vertices.clear();
        self.ranges.clear();
        self.pesos.clear();
        self.ossos.clear();
        self.afins.clear();
        self.juntas.clear();
    }

    /// Acumula `malha` convertida para o quad desta instância e devolve a marca (`1..`), ou `0`
    /// se ela não puder ser desenhada.
    pub(crate) fn push(&mut self, malha: &SpriteMesh, anchor: [f32; 2], size: [f32; 2]) -> u32 {
        if desenhavel(Some(malha), size).is_none() {
            return 0;
        }
        self.work.clear();
        for (l, uv) in malha.local.iter().zip(&malha.uv) {
            let Some(pos) = quad_pos(*l, anchor, size) else {
                return 0;
            };
            self.work.push(QuadVertex { pos, uv: *uv });
        }
        let antes = self.vertices.len();
        let antes_pele = (
            self.pesos.len(),
            self.ossos.len(),
            self.afins.len(),
            self.juntas.len(),
        );
        let Some(intervalo) = stitch(&mut self.vertices, &self.work, &malha.tris) else {
            self.vertices.truncate(antes);
            return 0;
        };
        // ⭐⭐⭐ **A PELE percorre a MESMA costura** ([`caminho_da_tira`]) — não um segundo laço.
        self.costura_da_pele(malha, anchor, size);
        debug_assert_eq!(
            self.pesos.len(),
            self.vertices.len(),
            "a tabela da pele deixou de ser PARALELA ao buffer de vertices — o peso do vertice j \
             passaria a outro, e isso nao estoura: desenha a arte torcida no sitio errado"
        );
        let marca = self.ranges.len() + 1;
        let cabe = u32::try_from(marca)
            .ok()
            .filter(|m| *m <= RenderInstance::MESH_MASK >> RenderInstance::MESH_SHIFT);
        let Some(marca) = cabe else {
            self.vertices.truncate(antes);
            self.pesos.truncate(antes_pele.0);
            self.ossos.truncate(antes_pele.1);
            self.afins.truncate(antes_pele.2);
            self.juntas.truncate(antes_pele.3);
            return 0;
        };
        self.ranges.push(intervalo);
        marca
    }
}

/// ⭐ **`local → quad_pos`** — a inversa da lei do shader `local = anchor + quad_pos · size`.
///
/// `None` quando o `size` tem um lado nulo ou não finito: ali não há quad a que referir o vértice.
#[must_use]
pub(crate) fn quad_pos(local: [f32; 2], anchor: [f32; 2], size: [f32; 2]) -> Option<[f32; 2]> {
    let ok = |s: f32| s.is_finite() && s != 0.0;
    if !(ok(size[0]) && ok(size[1])) {
        return None;
    }
    Some([
        (local[0] - anchor[0]) / size[0],
        (local[1] - anchor[1]) / size[1],
    ])
}

/// ⭐ **Costura uma lista de triângulos numa tira** com degenerados de ligação, acrescentando a `out`.
///
/// Para cada triângulo depois do primeiro acrescenta `(último, a)` e depois `a b c`: as quatro
/// janelas de 3 que atravessam a ligação têm dois vértices iguais (área zero, nenhum fragmento).
/// Triângulos com índices fora de `verts` são **saltados**. `None` se nenhum triângulo for válido.
pub(crate) fn stitch(
    out: &mut Vec<QuadVertex>,
    verts: &[QuadVertex],
    tris: &[[u32; 3]],
) -> Option<(u32, u32)> {
    let start = out.len();
    let mut algum = false;
    caminho_da_tira(tris, verts.len(), |v| {
        out.push(verts[v]);
        algum = true;
    });
    if !algum {
        return None;
    }
    Some((u32::try_from(start).ok()?, u32::try_from(out.len()).ok()?))
}

/// ⭐⭐⭐ **A SEQUÊNCIA DE ÍNDICES DE ORIGEM QUE A TIRA PERCORRE** — a lei da costura, num sítio só.
///
/// ⚠️⚠️ **Ela existe porque a costura passou a ter DOIS consumidores** (F9 W2): os vértices e a
/// tabela da PELE, que é um vector paralelo ao buffer deles. ⛔ Escrever a caminhada duas vezes
/// poria o peso do vértice `j` no vértice `j+2` no dia em que alguém mexesse num dos dois laços —
/// e isso não estoura: desenha a arte a torcer-se no sítio errado. *Uma lei escrita em dois sítios
/// ainda não é uma lei.*
///
/// Para cada triângulo depois do primeiro emite `(último, a)` e depois `a b c`: as quatro janelas
/// de 3 que atravessam a ligação têm dois vértices iguais (área zero, nenhum fragmento).
/// Triângulos com índices fora de `n` são **saltados**.
pub(crate) fn caminho_da_tira(tris: &[[u32; 3]], n: usize, mut emite: impl FnMut(usize)) {
    let mut ultimo: Option<usize> = None;
    for t in tris {
        let [a, b, c] = [t[0] as usize, t[1] as usize, t[2] as usize];
        if a >= n || b >= n || c >= n {
            continue;
        }
        if let Some(u) = ultimo {
            emite(u);
            emite(a);
        }
        emite(a);
        emite(b);
        emite(c);
        ultimo = Some(c);
    }
}

/// Limpa a marca de malha de uma instância — o que toda instância que NÃO veio da recolha precisa.
pub(crate) fn clear_mesh_tag(inst: &mut RenderInstance) {
    inst.flip_uv &= !RenderInstance::MESH_MASK;
}

/// ⭐ **Desenha um run**: o quad instanciado de sempre, ou — se o run é de uma malha — o intervalo
/// dela, trocando o buffer do slot 0 e repondo o quad a seguir.
///
/// ⚠️ **Uma porta para os TRÊS passes** (normal, recorte, máscara): uma malha que desenhasse num e
/// não nos outros perderia o recorte ou a máscara sem erro nenhum.
pub(crate) fn draw_run(
    pass: &mut wgpu::RenderPass<'_>,
    run: &crate::renderer::DrawRun,
    quad: &wgpu::Buffer,
    mesh: &wgpu::Buffer,
    ranges: &[(u32, u32)],
) {
    if run.mesh == 0 {
        pass.draw(0..4, run.start..run.end);
        return;
    }
    let Some(&(a, b)) = ranges.get(run.mesh as usize - 1) else {
        return;
    };
    pass.set_vertex_buffer(0, mesh.slice(..));
    pass.draw(a..b, run.start..run.end);
    pass.set_vertex_buffer(0, quad.slice(..));
}

/// O buffer de vértices das malhas de uma chamada — o gémeo do [`crate::InstanceBuffer`].
pub(crate) struct MeshVertexBuffer {
    buffer: wgpu::Buffer,
    capacity: u32,
}

impl MeshVertexBuffer {
    pub(crate) fn new(gpu: &GpuContext) -> Self {
        Self {
            buffer: Self::allocate(gpu, 1),
            capacity: 1,
        }
    }

    fn allocate(gpu: &GpuContext, capacity: u32) -> wgpu::Buffer {
        gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render sprite mesh vbo"),
            size: u64::from(capacity) * std::mem::size_of::<QuadVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub(crate) fn upload(&mut self, gpu: &GpuContext, vertices: &[QuadVertex]) {
        let Ok(needed) = u32::try_from(vertices.len()) else {
            return;
        };
        if needed > self.capacity {
            let mut cap = self.capacity.max(1);
            while cap < needed {
                cap = cap.saturating_mul(2);
            }
            self.buffer = Self::allocate(gpu, cap);
            self.capacity = cap;
        }
        if needed > 0 {
            gpu.queue
                .write_buffer(&self.buffer, 0, bytemuck::cast_slice(vertices));
        }
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

#[cfg(test)]
#[path = "sprite_mesh_tests.rs"]
mod tests;
