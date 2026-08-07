//! **O painel da cena 3D** (`SCULPT3D_*`) — ADR-0150, W12.
//!
//! A família de ids do `ph2d-panel-sculpt3d`: a lista de ferramentas, os knobs
//! do pincel, o espelho, a topologia, o sombreamento e a lista de peças.
//!
//! Slug pontilhado (`sculpt3d.*`), como a família do painel de física. Hash de
//! string, então nenhum contador de id se move — o `node_id_collisions` varre
//! estas chaves como varre as outras.

use super::{NodeId, hash_node_id};

/// Retângulo externo do painel (para o `z_order` + a barreira de hit).
pub const SCULPT3D_PANEL: NodeId = hash_node_id("sculpt3d.panel");
/// Botão de fechar (X).
pub const SCULPT3D_CLOSE: NodeId = hash_node_id("sculpt3d.close");

// ── Cabeçalhos de seção ─────────────────────────────────────────────────────
// Dobráveis, e têm de ser: o `paint_section_header` SEMPRE pinta o chevron,
// então um cabeçalho sem id vivo desenha um "clique para dobrar" que não faz
// nada.
/// A ferramenta em mãos (os 16 verbos).
pub const SCULPT3D_SEC_TOOL: NodeId = hash_node_id("sculpt3d.sec.tool");
/// Os knobs do pincel (raio, força, falloff, e os dois condicionais).
pub const SCULPT3D_SEC_BRUSH: NodeId = hash_node_id("sculpt3d.sec.brush");
/// Os espelhos.
pub const SCULPT3D_SEC_SYMMETRY: NodeId = hash_node_id("sculpt3d.sec.symmetry");
/// A resolução da malha — topologia dinâmica, multires, remesh.
pub const SCULPT3D_SEC_TOPOLOGY: NodeId = hash_node_id("sculpt3d.sec.topology");
/// Como a forma é LIDA — cavidade e luz.
pub const SCULPT3D_SEC_SHADING: NodeId = hash_node_id("sculpt3d.sec.shading");
/// A lista de peças e os verbos que a mexem.
pub const SCULPT3D_SEC_SCENE: NodeId = hash_node_id("sculpt3d.sec.scene");

// ── A ferramenta ────────────────────────────────────────────────────────────
/// Os 16 verbos, na ordem de `ph2d_sculpt3d::Verb::ALL`.
///
/// ⚠️ **O tamanho é o do `Verb::ALL`, e o gate o compara** — um verbo novo sem
/// chip aqui é uma ferramenta que o artista não alcança, que é exatamente o que
/// aconteceu com o `Magnify` antes de ele ganhar a tecla `A`.
pub const SCULPT3D_VERB: [NodeId; 16] = [
    hash_node_id("sculpt3d.verb.0"),
    hash_node_id("sculpt3d.verb.1"),
    hash_node_id("sculpt3d.verb.2"),
    hash_node_id("sculpt3d.verb.3"),
    hash_node_id("sculpt3d.verb.4"),
    hash_node_id("sculpt3d.verb.5"),
    hash_node_id("sculpt3d.verb.6"),
    hash_node_id("sculpt3d.verb.7"),
    hash_node_id("sculpt3d.verb.8"),
    hash_node_id("sculpt3d.verb.9"),
    hash_node_id("sculpt3d.verb.10"),
    hash_node_id("sculpt3d.verb.11"),
    hash_node_id("sculpt3d.verb.12"),
    hash_node_id("sculpt3d.verb.13"),
    hash_node_id("sculpt3d.verb.14"),
    hash_node_id("sculpt3d.verb.15"),
];

// ── O pincel ────────────────────────────────────────────────────────────────
/// As 5 curvas de `ph2d_sculpt3d::Falloff::ALL`.
pub const SCULPT3D_FALLOFF: [NodeId; 5] = [
    hash_node_id("sculpt3d.falloff.0"),
    hash_node_id("sculpt3d.falloff.1"),
    hash_node_id("sculpt3d.falloff.2"),
    hash_node_id("sculpt3d.falloff.3"),
    hash_node_id("sculpt3d.falloff.4"),
];
/// Raio do pincel, em **pixels de tela**.
pub const SCULPT3D_RADIUS: NodeId = hash_node_id("sculpt3d.radius");
/// Chip ligado a [`SCULPT3D_RADIUS`].
pub const SCULPT3D_RADIUS_NUM: NodeId = hash_node_id("sculpt3d.radius_num");
/// Força do dab, em `[0, 1]`.
pub const SCULPT3D_STRENGTH: NodeId = hash_node_id("sculpt3d.strength");
/// Chip ligado a [`SCULPT3D_STRENGTH`].
pub const SCULPT3D_STRENGTH_NUM: NodeId = hash_node_id("sculpt3d.strength_num");
/// Deslocamento do plano, em fração do raio (só os verbos de plano o leem).
pub const SCULPT3D_PLANE_OFFSET: NodeId = hash_node_id("sculpt3d.plane_offset");
/// Chip ligado a [`SCULPT3D_PLANE_OFFSET`].
pub const SCULPT3D_PLANE_OFFSET_NUM: NodeId = hash_node_id("sculpt3d.plane_offset_num");
/// Quanto o Crease aperta lateralmente.
pub const SCULPT3D_PINCH: NodeId = hash_node_id("sculpt3d.pinch");
/// Chip ligado a [`SCULPT3D_PINCH`].
pub const SCULPT3D_PINCH_NUM: NodeId = hash_node_id("sculpt3d.pinch_num");

/// **O PADRÃO que decide onde, dentro da pegada, o verbo age** — a primeira
/// opção é NENHUM e as outras são os padrões de `ph2d_sculpt3d::Alpha::ALL`.
///
/// ⚠️ O tamanho é `Alpha::ALL.len() + 1`, e o `+ 1` é o pincel liso — que **não**
/// é um padrão. É a mesma aritmética do [`SCULPT3D_MATCAP`], e pelo mesmo
/// motivo: um chip a mais pinta uma opção que o motor não tem, um a menos deixa
/// um padrão inalcançável. Gateado.
pub const SCULPT3D_ALPHA: [NodeId; 7] = [
    hash_node_id("sculpt3d.alpha.none"),
    hash_node_id("sculpt3d.alpha.0"),
    hash_node_id("sculpt3d.alpha.1"),
    hash_node_id("sculpt3d.alpha.2"),
    hash_node_id("sculpt3d.alpha.3"),
    hash_node_id("sculpt3d.alpha.4"),
    hash_node_id("sculpt3d.alpha.5"),
];
/// Tamanho de uma feature do alpha, em unidades de objeto.
pub const SCULPT3D_ALPHA_SCALE: NodeId = hash_node_id("sculpt3d.alpha_scale");
/// Chip ligado a [`SCULPT3D_ALPHA_SCALE`].
pub const SCULPT3D_ALPHA_SCALE_NUM: NodeId = hash_node_id("sculpt3d.alpha_scale_num");

// ── O espelho ───────────────────────────────────────────────────────────────
// TRÊS botões e não um rádio: os eixos são independentes (o ZBrush espelha em
// dois ao mesmo tempo), e um segmented é *um de N* por construção.
/// Espelho em X.
pub const SCULPT3D_SYM_X: NodeId = hash_node_id("sculpt3d.sym.x");
/// Espelho em Y.
pub const SCULPT3D_SYM_Y: NodeId = hash_node_id("sculpt3d.sym.y");
/// Espelho em Z.
pub const SCULPT3D_SYM_Z: NodeId = hash_node_id("sculpt3d.sym.z");

// ── A topologia ─────────────────────────────────────────────────────────────
/// Liga/desliga a topologia dinâmica.
pub const SCULPT3D_DYNTOPO: NodeId = hash_node_id("sculpt3d.dyntopo");
/// Os 3 degraus de detalhe (grosso/médio/fino).
pub const SCULPT3D_DETAIL: [NodeId; 3] = [
    hash_node_id("sculpt3d.detail.0"),
    hash_node_id("sculpt3d.detail.1"),
    hash_node_id("sculpt3d.detail.2"),
];
/// Desce um nível de multiresolução.
pub const SCULPT3D_LEVEL_DOWN: NodeId = hash_node_id("sculpt3d.level_down");
/// Sobe um nível de multiresolução.
pub const SCULPT3D_LEVEL_UP: NodeId = hash_node_id("sculpt3d.level_up");
/// Subdivide (acrescenta um nível ACIMA).
pub const SCULPT3D_SUBDIVIDE: NodeId = hash_node_id("sculpt3d.subdivide");
/// Reverte (reconstrói um nível ABAIXO).
pub const SCULPT3D_REVERSE: NodeId = hash_node_id("sculpt3d.reverse");
/// Reconstrói a casca (voxel remesh).
pub const SCULPT3D_REMESH: NodeId = hash_node_id("sculpt3d.remesh");
/// Tapa os buracos.
pub const SCULPT3D_CLOSE_HOLES: NodeId = hash_node_id("sculpt3d.close_holes");

// ── O sombreamento ──────────────────────────────────────────────────────────
/// **A CAVIDADE** — quanto a curvatura escurece a fresta e clareia a crista.
pub const SCULPT3D_CAVITY: NodeId = hash_node_id("sculpt3d.cavity");
/// Chip ligado a [`SCULPT3D_CAVITY`].
pub const SCULPT3D_CAVITY_NUM: NodeId = hash_node_id("sculpt3d.cavity_num");

/// **QUANTO DO AO ASSADO ENTRA** — irmão da cavidade no painel, e o oposto dela
/// na origem: a cavidade é derivada e existe sempre, o AO só existe depois de um
/// bake explícito.
pub const SCULPT3D_AO: NodeId = hash_node_id("sculpt3d.ao");
/// Chip ligado a [`SCULPT3D_AO`].
pub const SCULPT3D_AO_NUM: NodeId = hash_node_id("sculpt3d.ao_num");
/// **QUANTO DO AO DE TELA ENTRA** — o irmão MEDIDO do de cima.
///
/// ⚠️ Dois knobs e não um, e a diferença não é gosto: o assado é exato, viaja no
/// arquivo e ENVELHECE a cada pincelada; este é medido todo frame, nunca fica
/// velho e só vê o que está na tela. Colapsá-los num knob só obrigaria o artista
/// a escolher entre a oclusão que ele VÊ enquanto trabalha e a que ele EXPORTA.
pub const SCULPT3D_SSAO: NodeId = hash_node_id("sculpt3d.ssao");
/// Chip ligado a [`SCULPT3D_SSAO`].
pub const SCULPT3D_SSAO_NUM: NodeId = hash_node_id("sculpt3d.ssao_num");
/// **Quanto do espalhamento sub-superficial entra** (`ph2d_mesh_render::sss`).
pub const SCULPT3D_SSS: NodeId = hash_node_id("sculpt3d.sss");
/// Chip ligado a [`SCULPT3D_SSS`].
pub const SCULPT3D_SSS_NUM: NodeId = hash_node_id("sculpt3d.sss_num");
/// **Até onde a luz viaja dentro do material**, como FRAÇÃO do maior lado da
/// peça — nunca um comprimento absoluto (ver a row).
pub const SCULPT3D_SSS_SCATTER: NodeId = hash_node_id("sculpt3d.sss_scatter");
/// Chip ligado a [`SCULPT3D_SSS_SCATTER`].
pub const SCULPT3D_SSS_SCATTER_NUM: NodeId = hash_node_id("sculpt3d.sss_scatter_num");

/// **ASSAR O AO** — o botão que mede quanto do céu cada vértice enxerga.
///
/// ⚠️ É um BOTÃO e não um passe automático porque o bake não cabe num pen-up:
/// ~338 ms na malha que a cena `=16` abre (`ph2d-sdf/tests/measure_ao.rs`).
pub const SCULPT3D_BAKE_AO: NodeId = hash_node_id("sculpt3d.bake_ao");

/// **COM QUE LUZ o barro é mostrado** — a primeira opção é o RIG DO ARTISTA e as
/// outras são os matcaps de [`ph2d_mesh_render::MATCAPS`].
///
/// ⚠️ O tamanho é `MATCAPS.len() + 1`, e o `+ 1` é o rig — que **não** é um
/// matcap. A igualdade das duas contagens é gateada: um chip a mais pinta uma
/// opção que o shader não tem, um a menos deixa um material inalcançável.
pub const SCULPT3D_MATCAP: [NodeId; 7] = [
    hash_node_id("sculpt3d.matcap.rig"),
    hash_node_id("sculpt3d.matcap.0"),
    hash_node_id("sculpt3d.matcap.1"),
    hash_node_id("sculpt3d.matcap.2"),
    hash_node_id("sculpt3d.matcap.3"),
    hash_node_id("sculpt3d.matcap.4"),
    hash_node_id("sculpt3d.matcap.5"),
];

/// **ACUMULAR na mesma pincelada** — o `BRUSH_ACCUMULATE` do Blender.
pub const SCULPT3D_ACCUMULATE: NodeId = hash_node_id("sculpt3d.accumulate");

/// A malha de arestas desenhada por cima da forma.
pub const SCULPT3D_WIREFRAME: NodeId = hash_node_id("sculpt3d.wireframe");
/// Azimute da lâmpada selecionada, em graus.
pub const SCULPT3D_LIGHT_AZ: NodeId = hash_node_id("sculpt3d.light_az");
/// Chip ligado a [`SCULPT3D_LIGHT_AZ`].
pub const SCULPT3D_LIGHT_AZ_NUM: NodeId = hash_node_id("sculpt3d.light_az_num");
/// Elevação da lâmpada selecionada, em graus.
pub const SCULPT3D_LIGHT_ELEV: NodeId = hash_node_id("sculpt3d.light_elev");
/// Chip ligado a [`SCULPT3D_LIGHT_ELEV`].
pub const SCULPT3D_LIGHT_ELEV_NUM: NodeId = hash_node_id("sculpt3d.light_elev_num");

// ── A cena ──────────────────────────────────────────────────────────────────
/// As 4 primitivas (esfera, cubo, cilindro, toro).
pub const SCULPT3D_ADD: [NodeId; 4] = [
    hash_node_id("sculpt3d.add.0"),
    hash_node_id("sculpt3d.add.1"),
    hash_node_id("sculpt3d.add.2"),
    hash_node_id("sculpt3d.add.3"),
];
/// Duplica a peça ativa.
pub const SCULPT3D_DUPLICATE: NodeId = hash_node_id("sculpt3d.duplicate");
/// Apaga a peça ativa.
pub const SCULPT3D_DELETE: NodeId = hash_node_id("sculpt3d.delete");
/// Isola a peça ativa (o *local view*).
pub const SCULPT3D_ISOLATE: NodeId = hash_node_id("sculpt3d.isolate");
/// Funde as peças à vista numa só.
pub const SCULPT3D_MERGE: NodeId = hash_node_id("sculpt3d.merge");

// ── A máscara ───────────────────────────────────────────────────────────────
/// As 4 operações de máscara (limpar, inverter, borrar, afiar).
///
/// ⚠️ Elas moram na seção do PINCEL, ao lado do verbo `Mask`, e não numa seção
/// própria: um artista que acabou de pintar máscara procura o que fazer com ela
/// onde ele a pintou.
pub const SCULPT3D_MASK_OP: [NodeId; 4] = [
    hash_node_id("sculpt3d.mask_op.0"),
    hash_node_id("sculpt3d.mask_op.1"),
    hash_node_id("sculpt3d.mask_op.2"),
    hash_node_id("sculpt3d.mask_op.3"),
];
