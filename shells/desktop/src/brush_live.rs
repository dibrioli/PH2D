//! ⭐⭐⭐ **A ARTE dos pincéis da cena, resolvida por quadro** (plano 36, W3) — irmão do
//! [`crate::texture_pattern_live`], e pela mesma razão que ele existe.
//!
//! # Porque a resolução mora na shell
//!
//! A `ph2d-vec-render` não alcança a cena. Ir buscar a forma-fonte lá dentro poria o **guarda de
//! ciclo**, a **geometria viva** e o **cozimento** num sítio que não os pode medir — e é a mesma
//! escolha que o ladrilho do padrão já fez.
//!
//! # ⛔ Uma forma não pode ser o próprio pincel
//!
//! Desenhá-la exigiria as cópias, as cópias exigiriam a arte, e a arte seria ela. ⚠️ **O sintoma
//! não seria um erro**: seria o app a parar. A recusa é PURA e vive aqui, como a do padrão.
//!
//! # ⚠️ Sem memo, e o número decidiu-o
//!
//! O ladrilho do padrão é memoizado porque **assar** custa (1,047 ms medidos na W1 do plano 33).
//! Aqui não se assa nada: resolve-se um `VecPath` e clona-se. As **cópias** é que custam, e elas são
//! recalculadas no desenho — `0,423 ms` para 200 (plano 36 §7.4), medido. *Um memo sobre uma
//! resposta barata é estado derivado à espera de envenenar o undo.*

use ph2d_vec_render::BrushArts;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

/// A arte de cada pincel da cena, pela forma HOSPEDEIRA.
///
/// Vazio quando não há pincel nenhum — uma cena sem pincéis não paga uma alocação.
#[must_use]
pub(crate) fn resolve(
    scene: &VecScene,
    object_of: &dyn Fn(VecPathId) -> Vec<VecPathId>,
) -> BrushArts {
    let mut out = BrushArts::new();
    for path in scene.paths() {
        let Some(b) = path
            .stroke
            .as_ref()
            .and_then(ph2d_vec_scene::StrokeSpec::brush)
        else {
            continue;
        };
        let Some(alvo) = b.art else {
            // ⚠️ Um pincel SEM arte escolhida: nada a resolver, e a linha pinta a cor de recurso.
            continue;
        };
        if let Some(art) = art_of(scene, path.id, alvo, object_of) {
            out.insert(path.id, art);
        }
    }
    out
}

/// As formas que a arte `art` nomeia, **cozidas**, ou `None`.
///
/// ⭐⭐⭐ **UM GRUPO PODE SER A ARTE DE UM PINCEL** (report do Enio, 2026-08-30 — ele pediu-o para a
/// estampa, e o pincel é a mesma metade noutra tinta). O documento endereça a arte por um
/// `VecPathId`, e um grupo **não tem um**: o que muda é a **resolução** — o id passa a nomear o
/// OBJECTO a que aquele caminho pertence, que é a lei de selecção que o app já tem.
///
/// ⛔ **O guarda de ciclo é a primeira linha**, e não uma verificação a jusante: a forma não pode
/// ser o próprio pincel. ⚠️ E ele passou a ser sobre **PERTENÇA**, pela mesma porta que a estampa
/// usa ([`crate::texture_pattern_live::art_members`]): com um grupo, o anfitrião pode ser um
/// **membro** da arte, e aí desenhá-lo exigiria as cópias, as cópias exigiriam a arte, e a arte
/// seria ele. *O sintoma não seria um erro: seria o app a parar.*
///
/// ⚠️ **COZIDAS**, e não as fontes autoradas: um motivo com quina viva ou com uma pilha de efeitos
/// tem de se repetir como ele **parece**, não como foi digitado — a mesma lei que a arte-forma de um
/// padrão já obedece.
#[must_use]
fn art_of(
    scene: &VecScene,
    host: VecPathId,
    art: VecPathId,
    object_of: &dyn Fn(VecPathId) -> Vec<VecPathId>,
) -> Option<Vec<VecPath>> {
    let membros = crate::texture_pattern_live::art_members(host, art, object_of);
    let out: Vec<VecPath> = membros
        .iter()
        .filter_map(|m| scene.path(*m).map(|p| p.cooked().into_owned()))
        .collect();
    (!out.is_empty()).then_some(out)
}

#[cfg(test)]
#[path = "brush_live_tests.rs"]
mod tests;
