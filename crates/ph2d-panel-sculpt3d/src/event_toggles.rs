//! **OS SEIS INTERRUPTORES DO PAINEL, EM TABELA** — irmão (`#[path]`) do
//! [`super::event`], cortado por ASSUNTO.
//!
//! Cada um responde às MESMAS três perguntas — *que id sou eu*, *a lei que eu
//! ligo existe com este pincel na mão*, e *que campo eu viro* —, e enquanto elas
//! viviam em seis braços de `match` cada braço repetia as quatro linhas do corpo
//! (`seam_reset_button` · clonar a UI · virar o campo · `push_intent`).
//!
//! ⚠️ **A segunda pergunta é a que não pode faltar, e é por isso que ela está na
//! tabela e não num `if` do pintor:** um clique sintético — ou um id que
//! sobreviveu a uma troca de verbo no mesmo quadro — armaria um flag que nenhum
//! dab lê, e o painel voltaria a mostrá-lo marcado no próximo pincel que oferece
//! a lei. *O pintor não pinta a caixa onde a lei não existe; esta tabela recusa
//! o clique pela mesma razão, e as duas perguntam à mesma porta do motor.*
//!
//! ⚠️ **O gate de LOC foi o gatilho, não a razão.** O `apply_event` cruzou os 200
//! do `architecture_panel_loc_cap` quando o tecido ganhou o *Pin Simulation
//! Boundary*, e o que saiu foi a metade com fronteira própria — não a última
//! coisa que alguém escreveu.

use ph2d_a11y::NodeId;
use ph2d_sculpt3d::{ClothFilterKind, Verb};

use crate::state::Sculpt3dUi;

/// **UM INTERRUPTOR**: o id, a pergunta que decide se ele existe agora, e o
/// campo que ele vira.
pub(crate) type Toggle = (NodeId, fn(&Sculpt3dUi) -> bool, fn(&mut Sculpt3dUi));

/// `(o id · a lei existe? · o que virar)`.
///
/// ⚠️ **A lei é perguntada ao MOTOR** (`Verb::accumulates`,
/// `Brush::offers_front_faces`, `ClothArea::offers_pin`), nunca a uma lista de
/// nomes aqui — o pintor faz a mesma pergunta para decidir se desenha a caixa, e
/// duas cópias divergiriam num interruptor que aparece e não muda um vértice.
pub(crate) const TOGGLES: [Toggle; 17] = [
    (
        crate::ids::SCULPT3D_ACCUMULATE,
        |u| u.brush.verb.accumulates(),
        |u| u.brush.accumulate = !u.brush.accumulate,
    ),
    (
        crate::ids::SCULPT3D_FRONT_FACES,
        |u| u.brush.offers_front_faces(),
        |u| u.brush.front_faces_only = !u.brush.front_faces_only,
    ),
    (
        crate::ids::SCULPT3D_GRAB_ANCHOR,
        |u| u.brush.offers_grab_anchor(),
        |u| u.brush.grab_active_vertex = !u.brush.grab_active_vertex,
    ),
    (
        crate::ids::SCULPT3D_SURFACE_ONLY,
        |u| u.brush.offers_surface_only(),
        |u| u.brush.surface_only = !u.brush.surface_only,
    ),
    // ⚠️ **PROCURAR TAMBÉM PARA TRÁS** — a caixa existe porque sem ela um alvo
    // do lado errado deixa o pincel **inerte**, e isso é a resposta certa e não
    // um defeito (espec §6.3.3: não há «projectar para o infinito»).
    (
        crate::ids::SCULPT3D_PROJECT_BIDIR,
        |u| u.brush.offers_project_controls(),
        |u| u.brush.project_bidirectional = !u.brush.project_bidirectional,
    ),
    // ── Os DOIS interruptores do pincel de POSE ─────────────────────────────
    (
        crate::ids::SCULPT3D_POSE_ANCHORED,
        |u| u.brush.offers_pose_controls(),
        |u| u.brush.pose.ancorado = !u.brush.pose.ancorado,
    ),
    (
        crate::ids::SCULPT3D_POSE_ROT_LOCK,
        // ⭐ A pergunta é mais estreita que a do irmão, de propósito: ver
        // [`ph2d_sculpt3d::Brush::offers_pose_rotation_lock`].
        |u| u.brush.offers_pose_rotation_lock(),
        |u| u.brush.pose.trava_rotacao = !u.brush.pose.trava_rotacao,
    ),
    (
        crate::ids::SCULPT3D_SCRAPE_DYNAMIC,
        |u| u.brush.verb == Verb::MultiplaneScrape,
        |u| u.brush.scrape_dynamic = !u.brush.scrape_dynamic,
    ),
    (
        crate::ids::SCULPT3D_CLOTH_PIN,
        |u| u.brush.verb == Verb::Cloth && u.brush.cloth_area.offers_pin(),
        |u| u.brush.cloth_pin = !u.brush.cloth_pin,
    ),
    (
        crate::ids::SCULPT3D_CLOTH_PERSISTENT,
        |u| u.brush.verb == Verb::Cloth,
        |u| u.brush.cloth_persistent = !u.brush.cloth_persistent,
    ),
    (
        crate::ids::SCULPT3D_CLOTH_COLLISIONS,
        |u| u.brush.verb == Verb::Cloth,
        |u| u.brush.cloth_collisions = !u.brush.cloth_collisions,
    ),
    // ⭐⭐ **AS COLISÕES DO FILTRO** — a espec §7 diz que ele as tem («idem §5.6,
    // opção nasce desligada») e nós passávamos-lhe uma lista vazia.
    (
        crate::ids::SCULPT3D_CFILTER_COLLISIONS,
        |u| u.filter_law.is_cloth(),
        |u| u.cloth_filter.collisions = !u.cloth_filter.collisions,
    ),
    // ⭐⭐ **OS TRÊS EIXOS DO *Force Axis*** — e quem diz se eles existem agora é o
    // MOTOR (`ClothFilterKind::le_os_eixos`), nunca uma lista de nomes aqui.
    //
    // ⚠️ **Só a Escala os lê.** Pintá-los para os outros quatro tipos seria
    // exactamente o knob morto que esta casa varre a cada wave — o artista
    // clica, nada muda, e ele conclui que o app tem um defeito que não tem.
    (
        crate::ids::SCULPT3D_CFILTER_AXIS[0],
        |u| {
            u.filter_law
                .cloth()
                .is_some_and(ClothFilterKind::le_os_eixos)
        },
        |u| u.cloth_filter_axes[0] = !u.cloth_filter_axes[0],
    ),
    (
        crate::ids::SCULPT3D_CFILTER_AXIS[1],
        |u| {
            u.filter_law
                .cloth()
                .is_some_and(ClothFilterKind::le_os_eixos)
        },
        |u| u.cloth_filter_axes[1] = !u.cloth_filter_axes[1],
    ),
    (
        crate::ids::SCULPT3D_CFILTER_AXIS[2],
        |u| {
            u.filter_law
                .cloth()
                .is_some_and(ClothFilterKind::le_os_eixos)
        },
        |u| u.cloth_filter_axes[2] = !u.cloth_filter_axes[2],
    ),
    (
        crate::ids::SCULPT3D_ALPHA_PREVIEW,
        |u| u.brush.alpha.is_some(),
        |u| u.alpha_preview = !u.alpha_preview,
    ),
    // ⚠️ O arame **não pergunta nada**: ele é da VISTA, e toda ferramenta o tem.
    (
        crate::ids::SCULPT3D_WIREFRAME,
        |_| true,
        |u| u.wireframe = !u.wireframe,
    ),
];

/// **Este id é um interruptor que a lei OFERECE agora?** — a porta única das duas
/// perguntas, e o guard do braço de `match` que os despacha.
pub(super) fn oferecido(ui: &Sculpt3dUi, id: NodeId) -> bool {
    TOGGLES.iter().any(|(tid, lei, _)| *tid == id && lei(ui))
}

/// Vira o campo deste id. ⚠️ Só é chamada depois de [`oferecido`] — o `find` que
/// falha aqui é um caminho que o guard já recusou.
pub(super) fn virar(ui: &mut Sculpt3dUi, id: NodeId) {
    if let Some((_, _, vira)) = TOGGLES.iter().find(|(tid, _, _)| *tid == id) {
        vira(ui);
    }
}
