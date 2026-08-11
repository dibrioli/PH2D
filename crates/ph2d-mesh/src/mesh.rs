//! A malha residente: os buffers, a adjacência, as normais e o índice espacial
//! — a estrutura que o ADR-0150 chama de **representação primária**.
//!
//! Layout SoA (*struct of arrays*), adaptado de
//! `reference/sculptgl/src/mesh/MeshData.js`, MIT — ver `LICENSES/sculptgl-MIT.txt`.
//! SoA e não AoS porque é assim que os buffers sobem para a GPU e porque um dab
//! toca **posição** de milhares de vértices sem ler a cor de nenhum.
//!
//! ⚠️ **Cor e máscara são PREGUIÇOSAS**, o padrão dos planos do impasto no
//! Painter (`heights`/`covers`/`mats`, alocados só quando a camada é tocada):
//! uma malha importada que ninguém pintou não paga 12 B/vértice de cor nem
//! 4 B/vértice de máscara. `None` significa *o default uniforme*, e a
//! `measure_memory` mede as duas situações em vez de eu afirmar qual importa.
//!
//! ⚠️ **O que NÃO é guardado, e por quê:** centro e caixa por face. O SculptGL
//! os mantém (24 B/face — a 5 M faces, 120 MB) para atualizar o octree
//! incrementalmente. Os dois são O(1) a partir de posição+face, então aqui são
//! computados no build e descartados; quando a atualização incremental chegar
//! (W2), ela recomputa os da PEGADA, que é trabalho limitado pelo pincel.

use crate::aabb::Aabb;
use crate::adjacency::Adjacency;
use crate::edges::Edges;
use crate::face::Face;
use crate::normals;
use crate::octree::{Octree, RefitScratch};

/// A malha de triângulos/quads residente na CPU.
/// As portas que mudam a topologia — ver o módulo.
#[path = "mesh_splice.rs"]
mod splice;

/// A porta que ENCOLHE a topologia — ver o módulo.
#[path = "mesh_shrink.rs"]
mod shrink;

/// OS PLANOS OPCIONAIS (cor, máscara, AO) e a validade do AO — ver o módulo.
#[path = "mesh_planes.rs"]
mod planes;

/// A CONTABILIDADE DE BYTES (malha e os dois scratches) — ver o módulo.
#[path = "mesh_memory.rs"]
mod memory;

/// OS CANAIS AUTORADOS ATRAVESSANDO UMA TROCA DE TOPOLOGIA — ver o módulo.
#[path = "mesh_transfer.rs"]
mod transfer;

pub use transfer::transfer_authored;

pub use shrink::VertexMerge;
pub use splice::VertexAppend;

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    /// **A curvatura por vértice** (`crate::curvature`) — DERIVADA, como a
    /// normal, e não autorada, como a máscara.
    ///
    /// ⚠️ É essa classificação que decide o resto: ela não é preguiçosa (todo
    /// vértice tem uma no instante em que tem normal), não entra no undo por
    /// conta própria (o que o undo guarda é posição, e ela sai de posição), e
    /// não é serializada (`persist` reconstrói pelo `rebuild`). O que ela **tem**
    /// de fazer é viajar na compactação, junto das normais — ver
    /// [`Self::shrink_topology`].
    curvatures: Vec<f32>,
    /// **A curvatura em unidades de MUNDO** (`κ`, em `1/comprimento`) — a irmã
    /// da de cima, do MESMO gather, com o MESMO sinal, e de dimensão diferente.
    ///
    /// ⚠️ **Ela existe porque as duas perguntas são diferentes, e a medição é
    /// quem separa** (`tests/measure_curvature_units.rs`): escalar a peça 2×
    /// deixa `curvatures` intacta e **divide esta pela metade**; dobrar a
    /// tesselação faz o oposto. O Cavity quer a invariante de escala (uma ruga de
    /// uma aresta lê igual em qualquer tamanho); o **SSS pré-integrado** quer
    /// esta, porque a difusão tem escala física e a LUT é indexada pelo produto
    /// `raio_de_espalhamento × κ`.
    ///
    /// Mesma classificação da irmã: **DERIVADA** — recomputada pelas quatro
    /// portas, fora do undo, fora da serialização.
    curv_world: Vec<f32>,
    colors: Option<Vec<[f32; 3]>>,
    masks: Option<Vec<f32>>,
    /// **O AO assado por vértice** (`ph2d_sdf::bake_ao`) — quanto do céu cada
    /// vértice enxerga, `1` aberto e `0` enterrado.
    ///
    /// ⚠️ **Ele não é da família de nenhum dos dois vizinhos acima, e é isso que
    /// decide o resto.** A curvatura é **DERIVADA**: as portas a recomputam, e
    /// ela nunca fica velha. A máscara é **AUTORADA**: as portas a carregam, e
    /// ela continua válida seja qual for a forma. O AO é **MEDIDO DA FORMA** —
    /// então mexer na forma não o apaga nem o mantém: deixa-o **ERRADO**.
    ///
    /// E errado de um jeito que ninguém reporta: uma fresta clara demais lê como
    /// escolha de iluminação, não como número velho. Daí o [`Self::ao_is_stale`]
    /// existir ao lado — o canal carrega a própria data de validade, porque a
    /// obsolescência aqui é **inerente ao desenho e tem de ser DITA**.
    ///
    /// `None` = nunca assado (e não paga 4 B/vértice por isso).
    ao: Option<Vec<f32>>,
    /// **A ESPESSURA assada por vértice** (`crate::thickness::bake`) — quanto de
    /// matéria há atrás de cada vértice, medido pelo raio que entra por ele.
    ///
    /// ⚠️ **Mesma família do [`Self::ao`], e é por isso que ela divide a
    /// validade dele em vez de trazer uma segunda:** os dois são MEDIDOS DA
    /// FORMA pelo mesmo gesto, contra a mesma malha, e ficam errados no mesmo
    /// instante — no dab seguinte. Duas flags seriam duas respostas a *"o que
    /// eu medi ainda descreve esta peça?"*, e a segunda é a que alguém esquece
    /// de marcar.
    ///
    /// ⚠️ **E ela NÃO é `2/|κ|`.** O proxy pela curvatura é grátis e sai exato
    /// numa esfera — e erra **420% num toro e 511% numa chapa**
    /// (`ph2d-sdf/tests/measure_thickness.rs`), que é justamente a forma pela
    /// qual a luz atravessa. Um proxy que só acerta na fixture que o validaria
    /// não é uma medição.
    ///
    /// `None` = nunca assada (e não paga 4 B/vértice por isso).
    thickness: Option<Vec<f32>>,
    /// A forma mudou desde o último bake dos planos MEDIDOS? Ver os dois campos
    /// acima — a validade é uma só porque o gesto que os produz é um só.
    baked_stale: bool,
    faces: Vec<Face>,
    face_normals: Vec<[f32; 3]>,
    adjacency: Adjacency,
    octree: Octree,
    bounds: Aabb,
}

/// Cor de um vértice que a malha ainda não pintou (branco).
pub const DEFAULT_COLOR: [f32; 3] = [1.0, 1.0, 1.0];
/// Máscara de um vértice que ninguém mascarou (0 = totalmente esculpível).
pub const DEFAULT_MASK: f32 = 0.0;
/// PREVIEW de um vértice que ninguém previu (0 = o barro não é tingido).
///
/// ⚠️ **O preview NÃO é um plano desta malha** — ele é derivado do pincel vivo e
/// mora fora dela, precisamente para não atravessar a subdivisão, o remesh, o
/// fechamento de buraco, a fusão e o documento. A CONSTANTE mora aqui porque
/// dois crates que não se conhecem precisam do mesmo número: o kernel escreve
/// isto para *"nada aqui"* e o renderizador sobe isto para *"ninguém armou"*, e
/// se os dois divergirem o barro nasce tingido sem que ninguém tenha pedido.
pub const DEFAULT_PREVIEW: f32 = 0.0;
/// AO de um vértice que ninguém assou (1 = céu aberto).
///
/// ⚠️ **O default é o que NÃO escurece.** Um canal ausente tem de ser
/// invisível; se a ausência escurecesse, toda malha nasceria suja e o
/// artista iria procurar a sujeira no shader.
pub const DEFAULT_AO: f32 = 1.0;

impl Mesh {
    /// Constrói a partir de posições e faces cruas: valida os índices, deriva
    /// normais, adjacência e octree.
    ///
    /// **Valida**, e isso não é zelo: um índice fora de alcance vindo de um OBJ
    /// de terceiro vira, sem checagem, uma leitura errada em cada kernel que
    /// tocar aquele vértice — e o sintoma aparece a três waves de distância,
    /// numa normal torta.
    pub fn from_parts(positions: Vec<[f32; 3]>, faces: Vec<Face>) -> Result<Self, MeshError> {
        let n = positions.len() as u32;
        for (fi, f) in faces.iter().enumerate() {
            for &v in f.verts() {
                if v >= n {
                    return Err(MeshError::VertexOutOfRange {
                        face: fi,
                        vertex: v,
                        vert_count: positions.len(),
                    });
                }
            }
        }
        let mut mesh = Self {
            normals: vec![[0.0, 1.0, 0.0]; positions.len()],
            curvatures: vec![0.0; positions.len()],
            curv_world: vec![0.0; positions.len()],
            positions,
            faces,
            ..Self::default()
        };
        mesh.rebuild();
        Ok(mesh)
    }

    /// Recomputa tudo que é DERIVADO de posições e faces: normais de face,
    /// normais de vértice, adjacência, octree e a caixa do mundo.
    ///
    /// Porta única de propósito. Uma wave que reconstrói só metade disto deixa
    /// o sistema **estável e errado** — o octree apontando para onde a malha
    /// estava —, e esse é o modo de falha que não levanta erro nenhum.
    pub fn rebuild(&mut self) {
        normals::recompute_face_normals(&self.positions, &self.faces, &mut self.face_normals);
        self.adjacency = Adjacency::build(self.positions.len(), &self.faces);
        normals::recompute_vertex_normals(
            &self.face_normals,
            &self.adjacency.vert_faces,
            &mut self.normals,
            None,
        );
        // ⚠️ DEPOIS das normais, sempre — ela as lê.
        crate::curvature::recompute_curvature(
            &self.positions,
            &self.normals,
            &self.adjacency.vert_verts,
            &mut self.curvatures,
            &mut self.curv_world,
        );
        self.octree = Octree::build(&self.positions, &self.faces);
        self.bounds = Aabb::from_points(&self.positions);
    }

    #[must_use]
    pub fn vert_count(&self) -> usize {
        self.positions.len()
    }

    #[must_use]
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Quantos triângulos a rasterização vê — o número que os tetos por tier
    /// (ADR-0104) falam, e que **não** é a contagem de faces quando há quads.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.faces.iter().map(Face::tri_count).sum()
    }

    #[must_use]
    pub fn positions(&self) -> &[[f32; 3]] {
        &self.positions
    }

    #[must_use]
    pub fn normals(&self) -> &[[f32; 3]] {
        &self.normals
    }

    /// **A curvatura por vértice**, adimensional e com sinal: `> 0` côncavo,
    /// `< 0` convexo. Ver [`crate::curvature`] para a lei e a escala.
    #[must_use]
    pub fn curvatures(&self) -> &[f32] {
        &self.curvatures
    }

    /// **A curvatura em unidades de MUNDO** (`1/comprimento`), mesmo sinal da
    /// irmã. É o eixo que a LUT do SSS pré-integrado indexa — ver o campo.
    #[must_use]
    pub fn curv_world(&self) -> &[f32] {
        &self.curv_world
    }

    #[must_use]
    pub fn faces(&self) -> &[Face] {
        &self.faces
    }

    #[must_use]
    pub fn face_normals(&self) -> &[[f32; 3]] {
        &self.face_normals
    }

    #[must_use]
    pub fn adjacency(&self) -> &Adjacency {
        &self.adjacency
    }

    /// **O grafo de arestas desta malha**, construído AGORA.
    ///
    /// ⚠️ Ele não é um campo, e a [`crate::Edges`] explica por quê: hoje o único
    /// consumidor é a subdivisão, que roda quando o artista aperta um botão.
    /// Guardá-lo cobraria a construção em todo `rebuild` — inclusive o de cada
    /// undo — para responder uma pergunta que ninguém faz naquele instante.
    /// Quem precisa dele duas vezes seguidas guarda o retorno.
    #[must_use]
    pub fn edges(&self) -> Edges {
        Edges::build(&self.faces, &self.adjacency)
    }

    #[must_use]
    pub fn octree(&self) -> &Octree {
        &self.octree
    }

    #[must_use]
    pub fn bounds(&self) -> Aabb {
        self.bounds
    }

    /// **Descarta os vértices e faces do FIM** — a inversa exata de uma
    /// operação que só ACRESCENTA (hoje: fechar buraco).
    ///
    /// ⚠️ **Ela VALIDA em vez de confiar, e a validação é o motivo de ela
    /// existir como porta.** Um `verts` que corte abaixo do que as faces
    /// sobreviventes referenciam deixa índices pendurados — a mesma classe de
    /// defeito que o [`Self::from_parts`] recusa na entrada, e que sem checagem
    /// vira leitura errada em cada kernel que tocar aquele vértice.
    ///
    /// ⚠️ **Não é um `remove` genérico.** Ela só sabe desfazer um *append*, e
    /// quem a usar para outra coisa vai descobrir que os índices do meio não se
    /// deslocam — por isso o nome diz o fim, e por isso o único chamador é o
    /// desfazer do preenchimento.
    pub fn truncate(&mut self, verts: usize, faces: usize) -> Result<(), MeshError> {
        if verts > self.positions.len() || faces > self.faces.len() {
            return Err(MeshError::VertexOutOfRange {
                face: faces,
                vertex: verts as u32,
                vert_count: self.positions.len(),
            });
        }
        let n = verts as u32;
        for (fi, f) in self.faces[..faces].iter().enumerate() {
            for &v in f.verts() {
                if v >= n {
                    return Err(MeshError::VertexOutOfRange {
                        face: fi,
                        vertex: v,
                        vert_count: verts,
                    });
                }
            }
        }
        self.faces.truncate(faces);
        self.positions.truncate(verts);
        self.normals.truncate(verts);
        if let Some(c) = self.colors.as_mut() {
            c.truncate(verts);
        }
        if let Some(m) = self.masks.as_mut() {
            m.truncate(verts);
        }
        if let Some(a) = self.ao.as_mut() {
            a.truncate(verts);
            self.baked_stale = true;
        }
        if let Some(t) = self.thickness.as_mut() {
            t.truncate(verts);
            self.baked_stale = true;
        }
        self.rebuild();
        Ok(())
    }

    /// **Todo quad vira dois triângulos.** Devolve quantas faces nasceram.
    ///
    /// ⚠️ **É o preço de entrar em topologia dinâmica, e não é escolha nossa:**
    /// o `MeshDynamic` do SculptGL é *"triangles only"* na primeira linha, e o
    /// dyntopo do Blender triangula ao ser ligado pelo mesmo motivo — partir
    /// uma aresta de um quad não produz dois quads, produz um triângulo e um
    /// pentágono, e a partir daí a lei do split precisaria de um caso por
    /// forma de face.
    ///
    /// ⚠️ **Ela não mexe em vértice nenhum** — nem posição, nem cor, nem
    /// máscara —, então uma malha já triangulada sai **byte-idêntica** e chamar
    /// duas vezes custa uma varredura e nada mais. É isso que a torna segura de
    /// pôr num arm que o artista pode apertar sem querer.
    ///
    /// A diagonal é `[0,2]` (o corte `v0-v1-v2` + `v0-v2-v3`), a MESMA que o
    /// [`Face::tris`] usa para desenhar — um quad que se vê partido de um jeito
    /// e se torna partido de outro muda a silhueta no instante do arm.
    pub fn triangulate(&mut self) -> usize {
        if self.faces.iter().all(Face::is_tri) {
            return 0;
        }
        let mut out = Vec::with_capacity(self.faces.len() * 2);
        for f in &self.faces {
            let v = f.verts();
            if f.is_tri() {
                out.push(*f);
            } else {
                out.push(Face::tri(v[0], v[1], v[2]));
                out.push(Face::tri(v[0], v[2], v[3]));
            }
        }
        let added = out.len() - self.faces.len();
        self.faces = out;
        self.rebuild();
        added
    }

    /// As posições para escrita — o que um kernel de pincel move.
    ///
    /// Quem escreve aqui **fica devendo** um `refresh_region`: a normal, o
    /// octree e a caixa passam a descrever a malha de antes. É deliberado que a
    /// dívida seja explícita em vez de a porta reconstruir tudo sozinha — um
    /// dab que reconstruísse o octree inteiro seria O(malha) num gesto que é
    /// O(pegada), que é precisamente o que este índice existe para evitar.
    pub fn positions_mut(&mut self) -> &mut [[f32; 3]] {
        // ⚠️ **E fica devendo mais uma coisa: o AO passa a descrever a forma de
        // antes.** Marcar aqui é o mais perto de uma porta ÚNICA que este plano
        // consegue — `positions` é privado e esta é a única saída `&mut` dele,
        // então todo kernel de pincel do módulo passa por aqui. As outras duas
        // que mexem na forma são as de TOPOLOGIA, e elas marcam por conta.
        self.baked_stale = true;
        &mut self.positions
    }

    /// **Põe a origem local no centro da caixa** e devolve o deslocamento que
    /// foi retirado.
    ///
    /// ⚠️ **Isto NÃO é cosmético, e não dá para fazer pela `Pose`.** O espelho
    /// da escultura reflete NEGANDO uma coordenada
    /// (`ph2d_sculpt3d::Symmetry::signs`), então **o plano de simetria É a
    /// origem local** — uma malha que o autor deixou a dez unidades do zero
    /// espelha em torno de um plano que não passa por ela, e o gesto vira lixo.
    /// A `Pose` move o objeto no MUNDO; ela não move a origem local em relação
    /// à geometria, que é exatamente a grandeza aqui.
    ///
    /// ⚠️ **A escala NÃO é tocada, e a assimetria tem razão:** centrar é exigido
    /// por um mecanismo, redimensionar não é exigido por nenhum (o pincel mede
    /// PIXELS DE TELA desde a W4, e a câmera enquadra). O tamanho é assunto da
    /// `Pose`, onde não custa reescrever os números do autor.
    ///
    /// Devolve `[0, 0, 0]` numa malha já centrada — e nesse caso não escreve um
    /// vértice sequer.
    pub fn recenter(&mut self) -> [f32; 3] {
        let c = self.bounds.center();
        if c == [0.0, 0.0, 0.0] {
            return c;
        }
        for p in &mut self.positions {
            p[0] -= c[0];
            p[1] -= c[1];
            p[2] -= c[2];
        }
        // ⚠️ Só a CAIXA e o octree — as normais e a adjacência são invariantes
        // por translação, e um `rebuild` inteiro aqui pagaria por recomputá-las
        // sem que um número mudasse.
        self.bounds = Aabb::from_points(&self.positions);
        self.octree = Octree::build(&self.positions, &self.faces);
        c
    }

    /// Os vértices dentro da esfera — a consulta que um dab faz.
    ///
    /// O octree responde com as faces das folhas tocadas (conservador); aqui
    /// entra o filtro EXATO por distância e a deduplicação, porque um vértice
    /// aparece em todas as faces do anel dele.
    pub fn verts_in_sphere(
        &self,
        center: [f32; 3],
        radius: f32,
        scratch: &mut QueryScratch,
        out: &mut Vec<u32>,
    ) {
        out.clear();
        self.octree
            .faces_in_sphere(center, radius, &mut scratch.faces);
        if scratch.seen.len() != self.positions.len() {
            scratch.seen = vec![0u32; self.positions.len()];
            scratch.epoch = 0;
        }
        scratch.epoch = scratch.epoch.wrapping_add(1);
        // O carimbo 0 é o "nunca visto" do vetor recém-criado, então a época
        // nunca pode valer 0 — sem isto a primeira consulta após um wrap
        // devolveria vazio, uma vez a cada 4 bilhões e impossível de reproduzir.
        if scratch.epoch == 0 {
            scratch.epoch = 1;
            scratch.seen.fill(0);
        }
        let r2 = radius * radius;
        for &fi in &scratch.faces {
            for &v in self.faces[fi as usize].verts() {
                if scratch.seen[v as usize] == scratch.epoch {
                    continue;
                }
                scratch.seen[v as usize] = scratch.epoch;
                let p = self.positions[v as usize];
                let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
                if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r2 {
                    out.push(v);
                }
            }
        }
    }

    /// Recalcula as normais afetadas por um deslocamento em `moved`.
    ///
    /// É a metade *limitada pela pegada* do `rebuild`: as normais de face das
    /// faces que tocam `moved`, e depois as normais de vértice de TODOS os
    /// vértices dessas faces — não só os que se moveram. Um vizinho parado ao
    /// lado de uma face que girou tem a normal mudada, e esquecê-lo deixa uma
    /// costura visível exatamente na borda do pincel, que é onde o artista está
    /// olhando.
    ///
    /// ⚠️ **O octree é re-ajustado aqui** (W2), pela mesma lista de faces que as
    /// normais usam. A W1 o deixava velho de propósito — *"enquanto o dab move
    /// menos que a folga das caixas frouxas isso é invisível"* —, e o preço
    /// daquela frase é um traço forte empurrando um vértice para fora da caixa
    /// da folha dele: ele some da consulta e o pincel deixa um BURACO, sem erro
    /// e sem aviso. `Octree::refit` custa a PEGADA, não a malha, então a razão
    /// original para adiar (não transformar `O(pegada)` em `O(malha)`) continua
    /// honrada.
    pub fn refresh_region(&mut self, moved: &[u32], scratch: &mut RegionScratch) {
        if moved.is_empty() {
            scratch.forget();
            return;
        }
        scratch.reset(self.faces.len(), self.positions.len());

        scratch.faces.clear();
        for &v in moved {
            for &fi in self.adjacency.vert_faces.neighbours(v as usize) {
                if !scratch.face_seen[fi as usize] {
                    scratch.face_seen[fi as usize] = true;
                    scratch.faces.push(fi);
                }
            }
        }
        scratch.verts.clear();
        for &fi in &scratch.faces {
            for &v in self.faces[fi as usize].verts() {
                if !scratch.vert_seen[v as usize] {
                    scratch.vert_seen[v as usize] = true;
                    scratch.verts.push(v);
                }
            }
        }
        // Calcula para um vetor CONTÍGUO e espalha — é o que deixa a leitura
        // pura e permite o `rayon`. Escrever direto nos índices esparsos seria
        // escrita concorrente sobre o mesmo `Vec`.
        normals::face_normals_of(
            &self.positions,
            &self.faces,
            &scratch.faces,
            &mut scratch.tmp,
        );
        for (&fi, n) in scratch.faces.iter().zip(&scratch.tmp) {
            self.face_normals[fi as usize] = *n;
        }
        normals::vertex_normals_of(
            &self.face_normals,
            &self.adjacency.vert_faces,
            &scratch.verts,
            &mut scratch.tmp,
        );
        for (&v, n) in scratch.verts.iter().zip(&scratch.tmp) {
            self.normals[v as usize] = *n;
        }
        // **A CURVATURA, e a MESMA lista serve — isto é um fato sobre o alcance,
        // não uma economia.** A curvatura de `u` é função de `p(u)`, `n(u)` e das
        // posições do anel de `u`. Um vértice FORA de `scratch.verts` não tem
        // nenhum dos três mexido: ele não se moveu; nenhuma face dele toca um
        // movido (senão ele estaria na lista, que é justamente como ela é
        // construída), então a normal dele não mudou; e todo vizinho dele divide
        // uma face com ele, logo também não se moveu.
        //
        // ⚠️ **Vem DEPOIS do laço acima, e a ordem carrega peso:** ela lê
        // `self.normals`, e rodar antes daria a curvatura do vértice novo medida
        // contra a normal de antes de ele dobrar — o sinal sai invertido
        // exatamente na crista que o pincel acabou de levantar.
        crate::curvature::curvature_of(
            &self.positions,
            &self.normals,
            &self.adjacency.vert_verts,
            &scratch.verts,
            &mut scratch.tmp_k,
            &mut scratch.tmp_kw,
        );
        for ((&v, k), w) in scratch
            .verts
            .iter()
            .zip(&scratch.tmp_k)
            .zip(&scratch.tmp_kw)
        {
            self.curvatures[v as usize] = *k;
            self.curv_world[v as usize] = *w;
        }
        // As MESMAS faces que moveram as normais movem as caixas. Duas listas
        // divergiriam no dia em que uma delas ganhasse um filtro.
        self.octree.refit(
            &self.positions,
            &self.faces,
            &scratch.faces,
            &mut scratch.refit,
        );
        // A caixa do mundo é a raiz do octree — derivá-la de novo percorrendo
        // todas as posições seria o `O(malha)` entrando pela porta dos fundos.
        self.bounds = self.octree.bounds();
        // Limpa só o que sujou — zerar os vetores inteiros faria deste passe
        // `O(malha)` pela porta dos fundos.
        for &fi in &scratch.faces {
            scratch.face_seen[fi as usize] = false;
        }
        for &v in &scratch.verts {
            scratch.vert_seen[v as usize] = false;
        }
    }

    /// O buffer de índices que a GPU consome (quads triangulados).
    pub fn triangle_indices(&self, out: &mut Vec<[u32; 3]>) {
        out.clear();
        out.reserve(self.triangle_count());
        for f in &self.faces {
            f.triangles(out);
        }
    }
}

/// Buffers reutilizados entre consultas — a consulta é feita por movimento do
/// mouse, e alocar por dab é o que transforma um gesto em serrilhado.
#[derive(Clone, Debug, Default)]
pub struct QueryScratch {
    faces: Vec<u32>,
    seen: Vec<u32>,
    epoch: u32,
}

/// Buffers reutilizados pelo [`Mesh::refresh_region`].
///
/// Os `*_seen` são vetores do TAMANHO da malha, mas o passe só os toca onde
/// escreveu e os limpa na saída — é o que torna o custo função da pegada e não
/// da malha, e é a razão de eles viverem aqui em vez de nascerem por dab.
#[derive(Clone, Debug, Default)]
pub struct RegionScratch {
    faces: Vec<u32>,
    verts: Vec<u32>,
    face_seen: Vec<bool>,
    vert_seen: Vec<bool>,
    /// Saída contígua das portas paralelas, reusada pelas duas metades do passe.
    tmp: Vec<[f32; 3]>,
    /// A mesma coisa para a CURVATURA, que é escalar. ⚠️ Vetor próprio e não um
    /// reuso do [`Self::tmp`]: a curvatura roda **depois** das normais e as lê,
    /// então os dois estão vivos ao mesmo tempo.
    tmp_k: Vec<f32>,
    /// O irmão do `tmp_k` para a curvatura de MUNDO — mesma lista de vértices,
    /// mesmo gather, saída própria (o `curvature_of` devolve o par).
    tmp_kw: Vec<f32>,
    refit: RefitScratch,
}

impl RegionScratch {
    /// Os vértices cuja NORMAL o último `refresh_region` recomputou.
    ///
    /// ⚠️ **É um superconjunto de "quem se moveu", e a diferença é visível.** Um
    /// vizinho parado ao lado de uma face que girou tem a normal mudada; quem
    /// subir para a GPU só a lista de movidos deixa a normal velha exatamente na
    /// BORDA do pincel, que é onde o artista está olhando. Esta é a lista que o
    /// upload incremental consome.
    #[must_use]
    pub fn refreshed(&self) -> &[u32] {
        &self.verts
    }

    /// Declara que nada foi refrescado (um dab que não tocou geometria).
    ///
    /// Existe porque a alternativa é o chamador ler a lista do dab ANTERIOR e
    /// subir uma região que ninguém mexeu — barato, mas mentiroso, e a mentira
    /// vira um gate verde sobre um upload que não acompanha o produto.
    pub fn forget(&mut self) {
        self.verts.clear();
    }

    /// ⚠️ **`resize` e não `vec![]`, e a diferença aparece na topologia dinâmica:**
    /// a malha muda de tamanho a cada dab, e re-alocar dois vetores do tamanho
    /// dela por dab é `O(malha)` entrando pela porta dos fundos justamente na
    /// wave que existe para tirá-lo. Crescer preserva a capacidade e zera só a
    /// cauda — e as entradas antigas já são `false`, porque o passe limpa o que
    /// sujou antes de sair.
    fn reset(&mut self, faces: usize, verts: usize) {
        self.face_seen.resize(faces, false);
        self.vert_seen.resize(verts, false);
    }
}

/// O que impede uma malha inválida de existir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshError {
    VertexOutOfRange {
        face: usize,
        vertex: u32,
        vert_count: usize,
    },
}

impl core::fmt::Display for MeshError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VertexOutOfRange {
                face,
                vertex,
                vert_count,
            } => write!(
                f,
                "face {face} aponta para o vértice {vertex}, e a malha tem {vert_count}"
            ),
        }
    }
}

impl core::error::Error for MeshError {}

#[cfg(test)]
#[path = "mesh_tests.rs"]
mod tests;
