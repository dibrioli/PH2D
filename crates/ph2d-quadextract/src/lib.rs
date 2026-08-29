//! ⭐⭐⭐ **A EXTRACÇÃO DE MALHA QUAD a partir de um MAPA DE GRADE INTEIRA.**
//!
//! **Entra:** uma malha de triângulos **remalhada isotropicamente** mais um mapa de
//! grade inteira — uma parametrização **por triângulo** `f_t : t → R²` em que as
//! **isolinhas inteiras**, trazidas de volta à superfície, *são* a malha quad.
//! **Sai:** uma malha de **quads puros**, com posições em `R³`.
//!
//! ⛔⛔ **FASE ZERO, obrigatória e fora desta crate: remalhar isotropicamente.**
//! Medido em 2026-08-24, com tudo o resto igual (mesma superfície, mesmo campo,
//! mesma extracção, mesma densidade):
//!
//! | triangulação de entrada | enviesamento p50 | faces com canto `>60°` | quads |
//! |---|---|---|---|
//! | ⛔ leque sobre uma malha de quadriláteros | `10,4°` · `12,5°` | `7` · `7` | `99,9%` |
//! | ⭐ **remalhada isotropicamente** | **`5,1°`** · **`5,5°`** | **`0`** · `3` | **`100%`** |
//!
//! ⇒ *o dobro do enviesamento, sem uma linha de algoritmo mudar.* Triângulos
//! compridos com viés diagonal contaminam a parametrização, e a extracção herda-o.
//! O passe vive em `ph2d-remesh-iso`; **não meça esta crate sem ele.**
//!
//! # ⭐⭐ A tese do método, e por que ela nos serve
//!
//! ⛔ A maioria dos remalhadores gasta a maior parte do tempo a **impedir dobras** —
//! endurecendo o sistema e re-resolvendo até não haver nenhuma. ⭐ **Este método
//! aceita a dobra e extrai à mesma**, e por isso o solver pode parar mais cedo e um
//! mapa antes considerado defeituoso passa a ser utilizável.
//!
//! | espécie de dobra | quem a trata |
//! |---|---|
//! | contida no interior de uma célula da grade | ⭐ **ninguém** — o traço nunca a atravessa |
//! | atravessa uma isolinha sem tocar num ponto de grade | o traço ([`walk`]) |
//! | **contém** um ponto de grade | as saídas + as células + a fusão |
//! | leque com menos de meia volta | ⚠️ detectado e contado ([`ExtractReport::collapsed_fans`]) |
//!
//! # As fases, que são a ordem de dependência de dados
//!
//! | fase | onde |
//! |---|---|
//! | sanear a entrada (colapso · transições · precisão · pontos fixos) | [`ingest`] + [`sanitize`] |
//! | os nós | [`nodes`] |
//! | as saídas, e a ordem delas | [`ports`] |
//! | traçar cada saída até à parceira | [`walk`] |
//! | fechar as células, fundir, montar | [`cells`] |
//!
//! # ⛔ O PRÉ-REQUISITO que esta crate não resolve
//!
//! A extracção **assume** que as translações das funções de transição são
//! **inteiras**. Um mapa em que não sejam tem as grades de duas cartas desalinhadas,
//! e o saneamento apenas *arredonda o erro para dentro*. Quem o garante é o
//! arredondamento misto-inteiro do solver (`ph2d_gridmap::round`); quem o **mede** é
//! [`ExtractReport::shift_residual`], e ele é o gate.

#![forbid(unsafe_code)]

pub mod exact;
pub mod mapa;

mod cells;
mod doublets;
pub use doublets::repair_doublets;
mod fan;
mod ingest;
mod nodes;
mod ports;
mod sanitize;
pub(crate) mod walk;

use ph2d_mesh::{Mesh, MeshError};

/// **O MAPA DE GRADE INTEIRA**, como esta crate o consome.
#[derive(Clone, Copy, Debug)]
pub struct CornerMap<'a> {
    /// Os vértices da superfície, em `R³`.
    pub pos: &'a [[f32; 3]],
    /// Os triângulos.
    pub tris: &'a [[u32; 3]],
    /// ⭐ Por face, por canto, a imagem no domínio. **Por CANTO** — é isso que dá a
    /// cada triângulo a sua carta.
    pub uv: &'a [[[f64; 2]; 3]],
}

/// O que impede um mapa de ser extraído.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractError {
    /// Uma coordenada do domínio é `NaN` ou infinita.
    NotFinite,
    /// O mapa não tem uma única coordenada não-nula.
    EmptyDomain,
    /// ⚠️ O domínio não cabe na grade exacta — ver [`exact`].
    DomainTooLarge,
    /// Os triângulos e as imagens têm comprimentos diferentes.
    Mismatched,
    /// A malha de saída não se deixou montar.
    Mesh(MeshError),
}

impl core::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotFinite => write!(f, "o domínio tem uma coordenada não finita"),
            Self::EmptyDomain => write!(f, "o domínio é todo zero"),
            Self::DomainTooLarge => write!(f, "o domínio não cabe na grade exacta"),
            Self::Mismatched => write!(f, "triângulos e imagens têm comprimentos diferentes"),
            Self::Mesh(e) => write!(f, "a malha de saída não se montou: {e:?}"),
        }
    }
}

impl core::error::Error for ExtractError {}

impl From<MeshError> for ExtractError {
    fn from(e: MeshError) -> Self {
        Self::Mesh(e)
    }
}

/// ⭐ **O QUE A EXTRACÇÃO MEDIU DE SI PRÓPRIA.**
///
/// ⚠️ **Contagens ao lado de cada grandeza, sempre** — um balde que ninguém enche
/// lê-se como perfeito, e três defeitos desta família já foram pagos nesta linha.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExtractReport {
    /// ⭐ **O EXPOENTE DA GRADE COMUM** (`Q`) — o passo interno é `2^-Q` de célula.
    ///
    /// ⚠️ Está aqui porque é o número de que toda a exactidão desta crate depende:
    /// com ele, e só com ele, as coordenadas do domínio saneado são inteiros de
    /// menos de `2^52` e o predicado de orientação é um `i128` exacto.
    pub grid_exponent: u32,
    // ── entrada
    /// Arestas cuja imagem no domínio degenerou e foram colapsadas (§2.1).
    pub collapsed_edges: usize,
    /// Arestas que degeneraram **depois** do saneamento (§2.5).
    pub late_collapsed: usize,
    /// Faces mortas no colapso.
    pub dead_faces: usize,
    /// Arestas interiores · de bordo · não-manifold.
    pub interior_edges: usize,
    /// Arestas com uma face só.
    pub boundary_edges: usize,
    /// ⛔ Arestas com mais de duas faces.
    pub non_manifold_edges: usize,
    // ── as duas réguas do mapa
    /// O pior resíduo da **rotação** lido das cartas, em quartos de volta.
    pub rot_residual: f64,
    /// ⭐⭐⭐ O pior resíduo da **translação**, em células. ⛔ A extracção assume `0`.
    pub shift_residual: f64,
    /// A **mediana** do mesmo resíduo. ⚠️ Um máximo de meia célula com mediana `0` é
    /// *uma* costura má; com mediana `0,03` é o mapa inteiro a não fechar.
    pub shift_residual_p50: f64,
    /// ⭐⭐⭐ Quantas transições ficaram fraccionárias — a contagem, não o extremo.
    pub shift_fractional: usize,
    /// ⭐⭐⭐ O pior desencontro relativo de comprimento nas transições fraccionárias.
    pub seam_length_gap: f64,
    /// ⭐⭐⭐ Lados cuja imagem é mais curta que `1/100` de célula.
    pub tiny_edges: usize,
    /// Lados mais curtos que `1/10` de célula.
    pub short_edges: usize,
    /// ⛔ Transições que não se deixaram reler **exactamente** dos valores saneados.
    pub inexact_transitions: usize,
    /// ⛔⛔⛔ Recursos de transição que **não aproximam** — ver
    /// [`crate::sanitize::SanitizeReport::far_fallbacks`]. `> 0` é vermelho.
    pub far_fallbacks: usize,
    // ── saneamento
    /// Vértices pregados no ponto fixo de uma holonomia com rotação.
    pub pinned_fixed: usize,
    /// Singularidades de rotação nula pregadas no ponto inteiro.
    pub pinned_integer: usize,
    /// Vértices de bordo (leque aberto).
    pub open_fans: usize,
    /// ⛔ Leques regulares cuja holonomia não fechou.
    pub holonomy_broken: usize,
    /// A distribuição de valências, indexada por valência.
    pub valence: [usize; 16],
    /// ⭐ Quantas vezes a contagem por **ângulo** e a **holonomia** discordaram sobre
    /// a valência de um vértice — e a holonomia, que é exacta, decidiu.
    ///
    /// ⚠️ Acontece à volta de uma **dobra**: ali o domínio percorre a cunha ao
    /// contrário, e o ângulo sozinho lê `4` sobre um vértice singular.
    pub valence_adjusted: usize,
    // ── nós
    /// Nós de vértice · de aresta · de face.
    pub vertex_nodes: usize,
    /// Nós de aresta.
    pub edge_nodes: usize,
    /// Nós de face.
    pub face_nodes: usize,
    /// Faces de área zero no domínio.
    pub degenerate_faces: usize,
    /// ⭐ Faces **dobradas** (área negativa) — não são um erro.
    pub folded_faces: usize,
    // ── saídas e traço
    /// Saídas emitidas.
    pub ports: usize,
    /// Saídas emparelhadas.
    pub linked: usize,
    /// Saídas pendentes por **bordo**.
    pub pending_boundary: usize,
    /// ⛔ Saídas que chegaram e não acharam parceira.
    pub orphan: usize,
    /// ⛔⛔ A metade das órfãs que chegou ao ponto e não achou parceira — ver
    /// [`crate::walk::WalkStats::orphan_no_partner`].
    pub orphan_no_partner: usize,
    /// ⛔⛔ A metade que não achou por onde sair do triângulo (carta dobrada).
    pub orphan_no_exit: usize,
    /// ⭐⭐⭐ Das «sem parceira», quantas chegaram a um ponto que **tem nó** — falta-lhe só
    /// a cardinal de volta.
    pub orphan_no_partner_node_exists: usize,
    /// ⭐⭐⭐ Das «sem parceira», quantas caíram sobre uma **aresta** do triângulo.
    pub orphan_no_partner_on_edge: usize,
    /// ⭐⭐⭐ Quantas órfãs o **resgate pela face gémea** salvou — ver
    /// [`crate::walk::WalkStats::orphan_rescued_across_edge`].
    pub orphan_rescued_across_edge: usize,
    /// ⛔⛔⛔ **Porque o resgate pela gémea NÃO disparou** — `(sem aresta, sem gémea,
    /// sem chave, chave com outra direcção, a própria porta)`. Ver
    /// [`crate::walk::WalkStats::rescue_no_side`].
    pub rescue_why: (usize, usize, usize, usize, usize),
    /// ⭐⭐⭐ Quantas seriam resgatadas por cada convenção — ver
    /// [`crate::walk::WalkStats::rescue_would`].
    pub rescue_would: [usize; 4],
    /// ⭐⭐⭐ Ver [`crate::walk::WalkStats::rescue_by_fold`].
    pub rescue_by_fold: [usize; 8],
    /// Ver [`crate::walk::WalkStats::rescue_ambiguous`].
    pub rescue_ambiguous: usize,
    /// ⭐⭐⭐ `(pares ligados pelo passe MÚTUO, candidatas sem correspondência do outro lado)`.
    pub rescue_mutual: (usize, usize),
    /// ⭐⭐⭐ Ver [`crate::walk::WalkStats::rescue_offset`].
    pub rescue_offset: [usize; 4],
    /// ⭐⭐⭐ `(sem porta nenhuma, destas com a gémea degenerada, com a face de cá degenerada)`.
    pub rescue_no_port: (usize, usize, usize),
    /// ⭐⭐⭐ `(destas, quantas num CANTO da gémea, quantas com a gémea a ter outras portas)`.
    pub rescue_no_port_where: (usize, usize),
    /// ⭐⭐⭐ `(cantos com o nó MUDO, cantos com portas noutras faces)`.
    pub rescue_corner: (usize, usize),
    /// ⭐⭐⭐ `(órfãs de canto com leque AMBÍGUO, candidatas que ele registou)`.
    pub fan_ambiguous: (usize, usize),
    /// ⭐ Das «sem parceira», quantas caíram num **canto** — ver
    /// [`crate::walk::WalkStats::orphan_on_corner`].
    pub orphan_on_corner: usize,
    /// ⭐⭐⭐ Quantas o resgate pelo **leque** salvou.
    pub orphan_rescued_in_fan: usize,
    /// ⛔⛔⛔ Destas, quantas morreram num triângulo de área ZERO no domínio.
    pub orphan_no_exit_flat: usize,
    /// ⛔⛔ Destas, quantas tinham a origem já **fora** do triângulo.
    pub orphan_no_exit_o_outside: usize,
    /// ⛔⛔ Destas, quantas teriam saída pelo lado de **entrada**.
    pub orphan_no_exit_entry_only: usize,
    /// ⭐⭐⭐ A que distância o segmento passa do triângulo, em CÉLULAS de grade.
    pub orphan_miss_cells_p50: f32,
    /// ⭐ O diâmetro desse triângulo, em células — a régua da linha de cima.
    pub orphan_tri_cells_p50: f32,
    /// ⭐⭐⭐ **ONDE as órfãs morrem**, em raios normalizados — o sintoma MAIS A MONTANTE
    /// da cadeia que produz um furo.
    pub orphan_radius_p50: f32,
    /// O `p99` do raio normalizado da peça — a régua da linha de cima.
    pub piece_radius_p99: f32,
    /// ⛔ Traços que estouraram o tecto de passos.
    pub runaway: usize,
    /// ⛔ Traços que chegaram a uma parceira **já emparelhada com outra** — onde duas
    /// cartas se sobrepõem. A ligação alheia é preservada e esta saída fica pendente.
    pub contested: usize,
    /// Passos de traço, somados.
    pub walk_steps: usize,
    /// Vezes que um traço atravessou uma mudança de orientação.
    pub walk_flips: usize,
    // ── células
    /// Células fechadas.
    pub cells_closed: usize,
    /// Percursos abandonados numa saída pendente.
    /// ⭐⭐⭐ **ONDE os percursos falharam**, em raios normalizados — ver
    /// [`crate::cells::CellStats::failed_radius_p50`].
    ///
    /// ⛔ Um total não responde a *«furos nas pontas»*; uma coordenada responde.
    pub cells_failed_radius_p50: f32,
    /// A régua da linha de cima: o `p99` do raio normalizado de todos os nós.
    pub node_radius_p99: f32,
    pub cells_abandoned: usize,
    /// ⛔ Percursos que não fecharam.
    pub cells_unclosed: usize,
    /// Grupos de nós que a fusão colapsou.
    pub merged_groups: usize,
    /// ⚠️ Nós com o leque colapsado abaixo de meia volta (§6.4) — **detectado**.
    pub collapsed_fans: usize,
    /// ⭐⭐⭐ **A ORDEM DAS SAÍDAS, medida**: quantos passos consecutivos entre
    /// saídas do mesmo nó avançam `0`, `1`, `2` ou `3` quartos de volta.
    ///
    /// ⚠️ **A lista é horária, logo todo passo são desce um quarto** — um mapa
    /// saneado põe **tudo** no balde `3`. É a prova executável da propriedade de que
    /// a extracção de células depende (*«virar à esquerda» = a saída seguinte*), e
    /// ela é cega a qualquer outra coisa.
    ///
    /// ⚠️ O par que **dá a volta** fica de fora: é onde a holonomia entra, e num nó
    /// de valência 5 ele lê igual **por ser uma singularidade**.
    ///
    /// ⚠️ E **só conta os pares em que as duas cartas são não-dobradas** — numa
    /// carta dobrada o horário da superfície é o anti-horário do domínio, e o passo
    /// lê `1` por estar certo. Esses vão para [`Self::port_step_folded`].
    pub port_step: [usize; 4],
    /// O histograma dos pares que tocam uma carta **dobrada** — aí o balde certo é
    /// o `1`.
    pub port_step_folded: [usize; 4],
    /// ⭐ Quantos **lados** cada percurso fechado teve. Uma grade sã põe tudo no
    /// balde `4`; perto de uma dobra as cartas sobrepõem-se e a órbita cresce.
    pub ring_len: [usize; 17],
    /// ⭐ Quantos **cantos distintos** sobram depois da fusão. O teorema diz `4`,
    /// `2` ou `1`; um `3` exigiria uma ligação diagonal, que a fusão não cria.
    pub ring_distinct: [usize; 17],
    // ── saída
    /// Quads emitidos.
    pub quads: usize,
    /// Bígonos e monógonos, que colapsam.
    pub degenerate_cells: usize,
    /// ⛔ Células com três cantos — o teorema diz que não existem.
    pub triangles: usize,
    /// ⭐⭐⭐ **Células ESPELHADAS que caíram** — ver
    /// [`crate::cells::CellStats::mirrored_cells`]. Uma dobra do mapa cobre a mesma região
    /// duas vezes com orientações opostas, e o par sai como uma **almofada solta**: dois
    /// quads coincidentes, casco fechado, `χ = 2`, invisível a toda outra régua.
    pub mirrored_cells: usize,
    /// ⭐⭐⭐ **Doublets dissolvidos** — ver [`crate::cells::CellStats::doublets`]. Um vértice
    /// interior com duas arestas e duas faces: a mordida das pontas finas, e ela
    /// **realimenta-se** quando a saída volta a entrar na cadeia.
    pub doublets: usize,
}

/// ⭐⭐⭐ **A EXTRACÇÃO.**
///
/// `valence` é a valência por vértice, quando o produtor do mapa a souber (o índice
/// por-vértice de um campo cruzado é um facto dele). ⚠️ **Só é preciso distinguir
/// `4` de `≥ 8`**; `None` faz a crate contá-la ela própria, grosseiramente, e isso
/// **serve**.
///
/// # Errors
/// Ver [`ExtractError`].
pub fn extract(
    map: &CornerMap<'_>,
    valence: Option<&[u8]>,
) -> Result<(Mesh, ExtractReport), ExtractError> {
    let (mut topo, ing) = ingest::ingest(map)?;
    let san = sanitize::sanitize(&mut topo, valence);
    let (nodes, ns) = nodes::find_nodes(&topo);
    let mut ports = ports::emit(&topo, &nodes);
    let ws = walk::trace_all(&topo, &mut ports);
    let (mesh, cs) = cells::build(&topo, &nodes, &ports)?;
    let report = ExtractReport {
        grid_exponent: ing.grid_exponent,
        collapsed_edges: ing.collapsed,
        late_collapsed: san.late_collapsed,
        dead_faces: ing.dead_faces,
        interior_edges: ing.interior_edges,
        boundary_edges: ing.boundary_edges,
        non_manifold_edges: ing.non_manifold,
        rot_residual: ing.rot_residual,
        shift_residual: ing.shift_residual,
        shift_residual_p50: ing.shift_residual_p50,
        shift_fractional: ing.shift_fractional,
        seam_length_gap: ing.seam_length_gap,
        tiny_edges: ing.tiny_edges,
        short_edges: ing.short_edges,
        inexact_transitions: san.inexact_transitions,
        far_fallbacks: san.far_fallbacks,
        pinned_fixed: san.pinned_fixed,
        pinned_integer: san.pinned_integer,
        open_fans: san.open_fans,
        holonomy_broken: san.holonomy_broken,
        valence: san.valence,
        valence_adjusted: san.valence_adjusted,
        vertex_nodes: ns.vertex,
        edge_nodes: ns.edge,
        face_nodes: ns.face,
        degenerate_faces: ns.degenerate_faces,
        folded_faces: ns.folded_faces,
        ports: ports.ports.len(),
        linked: ws.linked,
        pending_boundary: ws.boundary,
        orphan: ws.orphan,
        orphan_no_partner: ws.orphan_no_partner,
        orphan_no_partner_node_exists: ws.orphan_no_partner_node_exists,
        orphan_no_partner_on_edge: ws.orphan_no_partner_on_edge,
        orphan_rescued_across_edge: ws.orphan_rescued_across_edge,
        rescue_why: (
            ws.rescue_no_side,
            ws.rescue_no_twin,
            ws.rescue_no_key,
            ws.rescue_wrong_dir,
            ws.rescue_self,
        ),
        rescue_would: ws.rescue_would,
        rescue_by_fold: ws.rescue_by_fold,
        rescue_ambiguous: ws.rescue_ambiguous,
        rescue_mutual: (ws.rescue_mutual, ws.rescue_not_mutual),
        rescue_offset: ws.rescue_offset,
        rescue_no_port: (
            ws.rescue_no_port,
            ws.rescue_no_port_flat,
            ws.rescue_no_port_here_flat,
        ),
        rescue_no_port_where: (ws.rescue_no_port_corner, ws.rescue_no_port_face_has_others),
        rescue_corner: (ws.rescue_corner_node_mute, ws.rescue_corner_other_faces),
        fan_ambiguous: (ws.fan_ambiguous, ws.fan_candidate),
        orphan_on_corner: ws.orphan_on_corner,
        orphan_rescued_in_fan: ws.orphan_rescued_in_fan,
        orphan_no_exit: ws.orphan_no_exit,
        orphan_no_exit_flat: ws.orphan_no_exit_flat,
        orphan_no_exit_o_outside: ws.orphan_no_exit_o_outside,
        orphan_no_exit_entry_only: ws.orphan_no_exit_entry_only,
        orphan_miss_cells_p50: ws.orphan_miss_cells_p50,
        orphan_tri_cells_p50: ws.orphan_tri_cells_p50,
        orphan_radius_p50: ws.orphan_radius_p50,
        piece_radius_p99: ws.piece_radius_p99,
        runaway: ws.runaway,
        contested: ws.contested,
        walk_steps: ws.steps,
        walk_flips: ws.flips,
        cells_closed: cs.closed,
        cells_failed_radius_p50: cs.failed_radius_p50,
        node_radius_p99: cs.node_radius_p99,
        cells_abandoned: cs.abandoned,
        cells_unclosed: cs.unclosed,
        merged_groups: cs.merged_groups,
        collapsed_fans: cs.collapsed_fans,
        port_step: cs.port_step,
        port_step_folded: cs.port_step_folded,
        ring_len: cs.ring_len,
        ring_distinct: cs.ring_distinct,
        quads: cs.quads,
        degenerate_cells: cs.degenerate_cells,
        triangles: cs.triangles,
        mirrored_cells: cs.mirrored_cells,
        doublets: cs.doublets,
    };
    Ok((mesh, report))
}

/// ⭐ **A CARACTERÍSTICA DE EULER** de uma malha — `V − E + F`.
///
/// ⚠️ **Ela é a régua que apanha a asa perdida**, e esta linha já pagou por ela: um
/// toro que sai com `χ = 2` passa em **todas** as outras réguas (100 % quads, zero
/// bordo, zero não-manifold) e perdeu a alça.
#[must_use]
pub fn euler_characteristic(mesh: &Mesh) -> i64 {
    use std::collections::BTreeSet;
    let mut edges: BTreeSet<(u32, u32)> = BTreeSet::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            edges.insert(if a < b { (a, b) } else { (b, a) });
        }
    }
    i64::try_from(mesh.vert_count()).unwrap_or(i64::MAX) - i64::try_from(edges.len()).unwrap_or(0)
        + i64::try_from(mesh.face_count()).unwrap_or(0)
}
