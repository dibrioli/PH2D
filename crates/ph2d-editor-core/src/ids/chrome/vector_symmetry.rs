//! **Os ids da seção SYMMETRY** — a simetria de desenho (plano 25 §9, W6.3) — irmão de `vector`
//! pelo teto de 700 LOC, e o corte é por responsabilidade: aqui moram os controles de um MODO de
//! desenho (o outro lado do traço é derivado enquanto ele está ligado), e não os do documento.
//!
//! ⚠️ **Isto nasceu como um `PathEffect` da pilha e foi REPROVADO** (Enio, 2026-08-01: *"funciona
//! bem mas não é legal como um efeito; melhor como uma opção para as tools de desenho exatamente
//! como o modo painter, morando em uma seção específica para isso"*). Os ids de efeito
//! desapareceram com ele; estes são de uma seção própria.

use ph2d_a11y::NodeId;

use super::hash_node_id;

/// **A seção SYMMETRY** — o cabeçalho colapsável.
pub const VECTOR_SECTION_SYMMETRY: NodeId = hash_node_id("vector.section.symmetry");

/// **Enable** — o par exclusivo que arma o modo (`Off` / `On`).
///
/// ⚠️ Ele fica no TOPO e **gateia toda a seção**, que é a lei que o Enio estabeleceu no painel do
/// impasto (*"é quem habilita esse modo de pintura … esse card só aparece se enable estiver
/// checado"*). Desarmado, os controles abaixo editariam o estilo de um espelho que não existe.
pub const VECTOR_SYM_OFF: NodeId = hash_node_id("vector.sym.off");
/// Ver [`VECTOR_SYM_OFF`].
pub const VECTOR_SYM_ON: NodeId = hash_node_id("vector.sym.on");

/// **Mirror X** — reflecte esquerda↔direita numa linha vertical.
pub const VECTOR_SYM_KIND_X: NodeId = hash_node_id("vector.sym.kind_x");
/// **Mirror Y** — reflecte cima↔baixo numa linha horizontal.
pub const VECTOR_SYM_KIND_Y: NodeId = hash_node_id("vector.sym.kind_y");
/// **Custom** — reflecte na linha que o artista desenhou.
pub const VECTOR_SYM_KIND_CUSTOM: NodeId = hash_node_id("vector.sym.kind_custom");
/// **Radial** — `segments` cópias em rotação (a *circular* que o Enio pediu explicitamente).
pub const VECTOR_SYM_KIND_RADIAL: NodeId = hash_node_id("vector.sym.kind_radial");

/// **Segments** — quantas cópias a rosácea tem. Só no Radial: nos espelhos a contagem é dois por
/// definição, e um slider preso em 2 é um controle morto.
pub const VECTOR_SYM_SEGMENTS: NodeId = hash_node_id("vector.sym.segments");
/// O chip numérico ligado ao [`VECTOR_SYM_SEGMENTS`].
pub const VECTOR_SYM_SEGMENTS_NUM: NodeId = hash_node_id("vector.sym.segments.num");

/// **Fuse** — solda as duas metades num contorno fechado quando as pontas pousam no eixo (o *Fuse
/// paths* do Inkscape / o *Merge* do Blender). É ele que faz do meio-perfil um vaso.
///
/// ⚠️ Só nos ESPELHOS: no Radial não há costura a fechar, e o kernel o ignora — oferecê-lo ali
/// seriam dois chips que não fazem nada.
pub const VECTOR_SYM_FUSE_OFF: NodeId = hash_node_id("vector.sym.fuse_off");
/// Ver [`VECTOR_SYM_FUSE_OFF`].
pub const VECTOR_SYM_FUSE_ON: NodeId = hash_node_id("vector.sym.fuse_on");

/// **Apply** — consolida as cópias em geometria de documento e desarma a simetria.
///
/// ⚠️ Oferecido só quando há simetria VIVA na seleção: sem ela não há o que consolidar, e um botão
/// que não faz nada é pior que botão que falta.
pub const VECTOR_SYM_APPLY: NodeId = hash_node_id("vector.sym.apply");
