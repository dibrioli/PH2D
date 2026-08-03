//! **UMA MOLDURA REDIMENSIONA; ela não ESCALA** — o que a alça do gizmo significa quando o sujeito
//! é uma moldura (plano UI/UX, corolário do W3).
//!
//! # O defeito, e o mecanismo dele
//!
//! O gizmo escreve a **pose** (`Transform.scale` da entidade) e a pose de um pai é herdada por
//! todo descendente — é isso que um grafo de cena É. Numa forma-folha isso é exactamente o certo:
//! escalar um desenho é escalar o desenho. Numa MOLDURA é o oposto do que o artista pede: os
//! filhos esticam junto, a tipografia achata, e a regra de âncora que ele acabou de armar nunca
//! chega a correr, porque a moldura não mudou de **caixa** — ela mudou de **escala**.
//!
//! O painel já fazia o certo: `W`/`H` reescrevem a GEOMETRIA da moldura
//! ([`crate::input_dispatch::apply_vec_transform`]) e deixam a pose em paz. Este módulo é o que
//! leva a alça do gizmo à mesma porta, para que as duas metades da interface não respondam
//! diferente à mesma pergunta.
//!
//! # A lei numa frase
//!
//! **O SUJEITO decide.** Moldura ⇒ a alça muda a caixa e os filhos respondem pelas regras deles.
//! Qualquer outra coisa ⇒ escala, como sempre. É a divisão do Figma (a alça de uma frame
//! redimensiona; escalar tudo é a ferramenta *Scale*), do Unity (Rect tool × Scale tool) e do
//! Rive (redimensionar um artboard não escala o conteúdo).
//!
//! ⚠️ **Não é um checkbox no objeto**, e a razão é que a pergunta não é sobre o OBJETO — é sobre a
//! intenção do GESTO. A mesma moldura merece as duas respostas em momentos diferentes, e um
//! interruptor persistente obriga a *marcar → gesto → desmarcar*: esquecer de desmarcar torna o
//! **próximo** gesto silenciosamente errado, meses depois, num ficheiro que já foi salvo.
//!
//! ⚠️ **E a regra é da moldura-idade, não da filiação.** Um path solto dentro de uma moldura é um
//! desenho-folha: escalá-lo continua a ser escalá-lo. Uma moldura ANINHADA é apanhada porque *é*
//! moldura, não porque é filha.
//!
//! # Escalar a moldura inteira continua possível
//!
//! Seleccione o **conteúdo** e arraste: uma multi-selecção escala pelo caminho de sempre, e o
//! `Transform` de cada membro é escrito. Outra selecção, outro verbo — sem estado novo no
//! ficheiro, e escolhido no momento em vez de dois painéis atrás.

use ph2d_ecs::{Entity, SimWorld, VecFrame, VecPathRef};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

/// A menor razão de escala que um redimensionamento aceita.
///
/// ⚠️ Positiva, e o espelho não passa — o mesmo clamp do `W`/`H` do painel (`target.max(1e-4)`).
/// Arrastar uma alça para lá do pivô inverteria o winding da moldura enquanto os filhos ficavam
/// onde estão: uma caixa do avesso com o conteúdo do lado direito. Uma largura não é negativa.
const MIN_RATIO: f64 = 1e-4;

/// **O que a moldura era quando o arrasto começou.**
///
/// ⚠️ Guardar a geometria de partida — em vez de aplicar a razão INCREMENTAL a cada movimento — é
/// o que torna o resultado um facto do **gesto** e não da taxa de amostragem do rato. O gizmo
/// recomputa uma transformação ABSOLUTA contra o `start_transform` a cada `CursorMoved`; compor
/// isso sobre a geometria já escalada multiplicaria a razão uma vez por evento. É a mesma lei que
/// o depósito do Painter e o solver da física seguem, e pela mesma razão.
#[derive(Clone)]
pub(crate) struct FrameResizeStart {
    /// A entidade arrastada. O arrasto seguinte é de outra pessoa; ver [`Self::is_for`].
    entity_bits: u64,
    id: VecPathId,
    /// A geometria da moldura no instante do pen-down.
    path: VecPath,
    /// O ponto FIXO do gesto, em coordenadas LOCAIS da moldura.
    anchor: [f64; 2],
}

impl FrameResizeStart {
    /// Este instantâneo é deste arrasto? Um arrasto novo sobre outra entidade tem de o refazer.
    #[must_use]
    pub(crate) fn is_for(&self, entity_bits: u64) -> bool {
        self.entity_bits == entity_bits
    }
}

/// **A moldura que este arrasto redimensiona** — `None` para tudo o mais.
///
/// ⚠️ Porta única, perguntada pelo braço do gizmo. Uma segunda pergunta (*"o pai é moldura?"*, por
/// exemplo) faria o gesto e a regra discordarem sobre quem é contêiner.
#[must_use]
pub(crate) fn resizable_frame(sim: &SimWorld, entity: Entity) -> Option<VecPathId> {
    let w = sim.world();
    w.get::<VecFrame>(entity)?;
    Some(w.get::<VecPathRef>(entity)?.0)
}

/// A razão de uma medida de mundo para a local. Degenerada vira `1`: converter um deslocamento
/// dentro de algo sem tamanho não quer dizer nada.
fn ratio(world: f64, local: f64) -> f64 {
    if local.abs() > 1e-9 {
        world / local
    } else {
        1.0
    }
}

/// **Arma o gesto**: fotografa a geometria e traduz o pivô de mundo para o local.
///
/// ⚠️ O ponto fixo é DERIVADO do pivô, e não escolhido por análise de qual alça foi pegada. Uma
/// tabela `canto arrastado → canto fixo` seria uma segunda resposta à pergunta que o gizmo já
/// respondeu no pen-down (`anchor_pivot_world`), e divergiria no primeiro caso que ela não
/// enumerasse — que é precisamente o CTRL, que ancora no CENTRO. Aqui o centro cai fora de graça.
///
/// ⚠️ A conversão é exacta enquanto o mapa local→mundo for alinhado aos eixos, e aproximada sob
/// ROTAÇÃO — a mesma aproximação declarada que a caixa do gizmo e o passe de âncoras carregam.
#[must_use]
pub(crate) fn begin(
    scene: &VecScene,
    xforms: &ph2d_vec_scene::VecXforms,
    entity_bits: u64,
    id: VecPathId,
    pivot_world: [f32; 2],
) -> Option<FrameResizeStart> {
    let path = scene.paths().iter().find(|p| p.id == id)?.clone();
    let (llo, lhi) = scene.path_curve_bbox(id)?;
    let (wlo, whi) = scene.path_world_curve_bbox(xforms, id)?;
    let mut anchor = [0.0_f64; 2];
    for i in 0..2 {
        let to_local = ratio(lhi[i] - llo[i], whi[i] - wlo[i]);
        anchor[i] = llo[i] + (f64::from(pivot_world[i]) - wlo[i]) * to_local;
    }
    Some(FrameResizeStart {
        entity_bits,
        id,
        path,
        anchor,
    })
}

/// **Aplica a razão ABSOLUTA do gesto**: repõe a geometria de partida e escala-a UMA vez.
///
/// Devolve `false` se a moldura já não está na cena (apagada a meio do arrasto). Razão `1,1` repõe
/// o instantâneo e sai — arrastar de volta ao ponto de partida devolve a moldura exactamente ao
/// que era, **ao bit**, e não a uma vizinhança dela.
pub(crate) fn apply(scene: &mut VecScene, start: &FrameResizeStart, sx: f64, sy: f64) -> bool {
    let Some(p) = scene.path_mut(start.id) else {
        return false;
    };
    *p = start.path.clone();
    let (sx, sy) = (clamp_ratio(sx), clamp_ratio(sy));
    if (sx - 1.0).abs() > 1e-9 || (sy - 1.0).abs() > 1e-9 {
        scene.scale_path(start.id, sx, sy, start.anchor);
    }
    true
}

/// Uma razão utilizável: positiva e não-degenerada. Ver [`MIN_RATIO`].
fn clamp_ratio(s: f64) -> f64 {
    if s.is_finite() { s.max(MIN_RATIO) } else { 1.0 }
}

impl crate::App {
    /// Fotografa a moldura arrastada — `None` se o sujeito do arrasto não é uma.
    ///
    /// A cola de shell mora aqui, e não no `advance_gizmo_drag`: o resto deste módulo é puro e
    /// dirigível headless, e é isso que faz os gates medirem o produto em vez de uma cópia dele.
    #[must_use]
    pub(crate) fn begin_frame_resize(
        &self,
        entity_bits: u64,
        pivot_world: [f32; 2],
    ) -> Option<FrameResizeStart> {
        let gfx = self.gfx.as_ref()?;
        let entity = Entity::from_bits(entity_bits);
        let id = resizable_frame(&gfx.sim, entity)?;
        let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
        begin(&gfx.vec_scene, &xf, entity_bits, id, pivot_world)
    }
}

#[cfg(test)]
#[path = "vec_frame_resize_tests.rs"]
mod tests;
