//! ⭐⭐⭐ **A RECEITA SOBE AO PALCO — ela vem ao artista, e não o artista a ela** (Enio, 2026-09-07:
//! *«o canvas busca a posição inicial do prefab. não deve ser assim. O prefab deve aparecer na
//! posição central do canvas onde o canvas está»*).
//!
//! # O que a versão anterior fazia, e porque era a resposta errada
//!
//! Ela movia a **CÂMERA** até à receita. Cumpre a letra (*«o prefab no centro»*) e falha o
//! pedido: o artista estava a olhar para um sítio, e abrir uma receita levava-lhe a vista para
//! outro. ⇒ o que se move é a **receita**.
//!
//! # Porque isso deixou de ser uma edição
//!
//! A recusa que shipou na wave anterior — *«mover a receita seria uma edição de verdade: entra no
//! undo e viaja no ficheiro»* — assentava numa premissa que **duas medições derrubaram**:
//!
//! 1. **O undo tem o conceito que faltava.** O [`ph2d_preview_drive`] separa *documento* de
//!    *pré-visualização*: o motor escreve no mundo e a **captura** repõe o valor autorado. ⇒ pôr a
//!    receita no palco não regista passo nenhum e não entra no ficheiro.
//! 2. **A posição de mundo de uma receita é BASTIDOR.** Ela é invisível fora do modo de edição
//!    (`MasterPiece` sem `MasterEditing` sai da cena **e** da Hierarquia), então *onde ela está*
//!    nunca foi uma escolha que o artista visse. O palco toma-a emprestada e devolve-a ao fechar.
//!
//! ⚠️⚠️ **E as cópias não se mexem, por CONSTRUÇÃO** — não por cuidado meu: o `Transform` da RAIZ
//! está na lista `ROOT_IS_ITS_OWN` do [`crate::instance_sync`] (*«o `Transform` de uma peça
//! propaga; o da raiz é onde o artista a largou»*), logo ele nunca alcança uma cópia. É a mesma
//! razão pela qual o motor vectorial remove só a **translação** do mestre ao compor uma instância.
//!
//! # O que ele NÃO faz
//!
//! ⛔ **Não mexe na câmera** — nem no centro nem no zoom. Uma receita maior que a área aparece
//! grande, exactamente como qualquer outro objecto: o artista já tem roda para isso, e movê-la por
//! ele é o defeito que este módulo veio corrigir.
//!
//! ⛔ **Não segue o pan.** O palco é montado **uma vez**, quando a receita abre; depois disso ela é
//! um objecto do mundo como outro qualquer. Colá-la ao ecrã tiraria ao artista a única forma de a
//! ver em contexto — e brigaria com o dedo dele ao arrastar as peças.

use ph2d_ecs::{Entity, MasterEditing, SimWorld, Transform};
use ph2d_editor_core::zones::Rect;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

use ph2d_preview_drive::{Driven, PreviewDrive};

/// ⭐ **Como a sessão acabou** — o alias do tipo que a barra publica, para a shell não ter de
/// nomear a crate do chrome em cada sítio.
pub use ph2d_editor_core::screens::hero::prefab_bar::PrefabExit as Exit;

/// Uma receita em cena: a identidade dela, a entidade viva, a pose de **bastidor** e o que o palco
/// escreveu por último.
///
/// ⛔⛔ **A identidade é o `StableId`; os bits são só o ENDEREÇO de hoje.** Um `Ctrl+Z` dentro da
/// sessão respawna tudo com bits novos — guardar só os bits fazia o palco desmontar-se ao desfazer,
/// e a receita saltava de volta para o bastidor (fora do ecrã) com a sessão ainda aberta.
#[derive(Clone, Copy, Debug)]
pub struct Stage {
    /// A identidade DURÁVEL da receita.
    pub(super) id: u64,
    /// A entidade de agora — re-resolvida a cada quadro pelo [`hold`].
    pub(super) root: u64,
    /// Para onde ela volta quando o palco fecha.
    pub(super) authored: Transform,
    /// A referência que o ledger usa para saber que **fui eu** — e não outra mão — a escrever.
    pub(super) last_written: Transform,
}

/// ⭐ **A LEI, sem o app** — quanto é preciso deslocar a caixa `min..max` (mundo) para o centro dela
/// cair no centro de `area` (pixels da janela), **sem tocar na câmera**.
///
/// ⚠️ O ponto de destino sai de [`Camera2d::screen_to_world`] — a MESMA porta que o rato usa para
/// saber onde clicou —, então a receita aterra debaixo do pixel que o artista vê no meio, por
/// construção.
pub(crate) fn delta_to_centre(
    cam: &Camera2d,
    window: WindowSize,
    area: Rect,
    min: [f32; 2],
    max: [f32; 2],
) -> [f32; 2] {
    let area = usable(area, window);
    let target = cam.screen_to_world((area.x + area.w * 0.5, area.y + area.h * 0.5), window);
    let centre = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    [target[0] - centre[0], target[1] - centre[1]]
}

/// A área utilizável — a publicada, ou a janela quando ela ainda não existe.
///
/// ⚠️ **O primeiro quadro publica uma área degenerada** (o painel ainda não pintou), e centrar numa
/// caixa de lado zero poria a receita num canto.
fn usable(area: Rect, window: WindowSize) -> Rect {
    if area.w > 1.0 && area.h > 1.0 {
        area
    } else {
        Rect::new(
            0.0,
            0.0,
            window.width.max(1) as f32,
            window.height.max(1) as f32,
        )
    }
}

/// **A porta do quadro** — monta o palco quando uma receita abre, mantém-no declarado enquanto ela
/// estiver aberta, e desmonta-o quando ela fecha.
///
/// ⚠️ **As três metades são obrigatórias.** Sem a primeira a receita nunca vem; sem a segunda o
/// [`PreviewDrive::settle`] esquece a condução no quadro seguinte e a posição de palco **vira
/// documento** (um passo de undo e bytes no ficheiro por um gesto de *ver*); sem a terceira a
/// receita fica onde o palco a deixou para sempre.
/// ⚠️ Oito argumentos, e uma engrenagem em struct seria pior aqui: cada um é uma FONTE com dono
/// próprio no quadro (o pedido, o palco, a tela, a janela, a câmera, o mundo, o ledger), e um
/// struct-de-argumentos convidaria alguém a guardá-lo entre quadros — que é exactamente como um
/// deles ficaria velho sem que nada dissesse.
#[allow(clippy::too_many_arguments)]
pub fn run(
    pending: &mut Option<u64>,
    stage: &mut Option<Stage>,
    hero: &ph2d_editor_core::screens::hero::HeroScreen,
    viewport: Rect,
    window: WindowSize,
    camera: &Camera2d,
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
) {
    hold(stage, sim, drive);
    raise(pending, stage, hero, viewport, window, camera, sim, drive);
}

/// ⭐⭐ **FECHAR e SEGURAR** — as duas metades que não precisam da tela, e por isso as duas que têm
/// gate headless.
///
/// ⚠️ **Segurar é obrigatório em TODO quadro:** a [`PreviewDrive::settle`] esquece quem não foi
/// declarado, e no quadro seguinte a pose de palco **viraria documento** — um passo de undo e bytes
/// no ficheiro por um gesto de *ver*.
///
/// ⚠️ **O `before` é o que EU escrevi, e não o que está no mundo** — é assim que o ledger sabe que
/// não houve outra mão e mantém o autorado no bastidor. Se o artista arrastar a receita pelo palco,
/// o movimento dele é visto (e é a pose viva), mas continua a ser pré-visualização: ao fechar, ela
/// volta ao bastidor. *A posição de mundo de uma receita nunca foi uma escolha visível.*
pub(crate) fn hold(stage: &mut Option<Stage>, sim: &mut SimWorld, drive: &mut PreviewDrive) {
    let Some(st) = stage.as_mut() else {
        return;
    };
    // ⭐ **O endereço re-resolve-se pela identidade a cada quadro** — ver o doc do [`Stage`].
    let Some(bits) = crate::instance_verbs_walk::entity_for_stable_id(sim, st.id) else {
        // A receita deixou de existir (apagada, ou um restauro que não a trouxe): não há onde
        // repor, e insistir seria escrever numa entidade morta.
        *stage = None;
        return;
    };
    let root = Entity::from_bits(bits);
    if !is_open(sim, bits) {
        Driven::StagePose(st.authored).write(sim, root);
        *stage = None;
        return;
    }
    // ⭐⭐⭐ **O palco REMONTA-SE depois de um `Ctrl+Z`.** O restauro trouxe a receita com a pose de
    // BASTIDOR (a captura substitui a pré-visualização pelo autorado), e numa entidade nova — logo
    // a entrada do ledger também é nova, e a primeira declaração dela tem de carregar o autorado.
    // ⛔ Sem isto a receita saltava para fora do ecrã a meio da sessão, e o passo seguinte gravava
    // a pose de palco como documento.
    if bits != st.root {
        Driven::StagePose(st.last_written).write(sim, root);
        drive.driven(
            root,
            Driven::StagePose(st.authored),
            Driven::StagePose(st.last_written),
        );
        st.root = bits;
        return;
    }
    if let Some(now) = sim.world().get::<Transform>(root).copied() {
        drive.driven(
            root,
            Driven::StagePose(st.last_written),
            Driven::StagePose(now),
        );
        st.last_written = now;
    }
}

/// **LEVANTAR** — o pedido espera pela caixa da receita, e só é servido quando ela existe.
#[allow(clippy::too_many_arguments)]
fn raise(
    pending: &mut Option<u64>,
    stage: &mut Option<Stage>,
    hero: &ph2d_editor_core::screens::hero::HeroScreen,
    viewport: Rect,
    window: WindowSize,
    camera: &Camera2d,
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
) {
    if stage.is_none()
        && let Some(bits) = *pending
        && hero.gizmo.selection == Some(bits)
        && let Some(view) = hero.gizmo.view.as_ref()
    {
        let root = Entity::from_bits(bits);
        let Some(authored) = sim.world().get::<Transform>(root).copied() else {
            // Sem pose não há palco — e o pedido morre, senão ele tentaria em todo quadro.
            *pending = None;
            return;
        };
        let d = delta_to_centre(
            camera,
            window,
            ph2d_app_host::canvas_area::visible(hero, viewport),
            view.bbox_min_world,
            view.bbox_max_world,
        );
        let mut placed = authored;
        placed.translation.x += d[0];
        placed.translation.y += d[1];
        Driven::StagePose(placed).write(sim, root);
        // ⚠️ **A PRIMEIRA declaração é a que fixa o autorado** — o `before` dela é a pose de
        // bastidor, e é ela que a captura repõe enquanto o palco durar.
        drive.driven(root, Driven::StagePose(authored), Driven::StagePose(placed));
        let Some(id) = sim.world().get::<ph2d_ecs::StableId>(root).map(|s| s.0) else {
            // Sem identidade durável não há palco: ele não sobreviveria ao primeiro `Ctrl+Z`, e um
            // palco que desmonta sozinho é pior que nenhum.
            *pending = None;
            return;
        };
        *stage = Some(Stage {
            id,
            root: bits,
            authored,
            last_written: placed,
        });
        *pending = None;
    }
}

/// A receita ainda está aberta? (A entidade pode ter desaparecido — um restore do undo respawna
/// tudo com bits novos.)
fn is_open(sim: &SimWorld, root: u64) -> bool {
    let e = Entity::from_bits(root);
    sim.world().get_entity(e).is_ok() && sim.world().get::<MasterEditing>(e).is_some()
}

#[cfg(test)]
#[path = "prefab_stage_tests.rs"]
mod tests;
