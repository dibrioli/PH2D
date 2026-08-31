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
//! # ⛔⛔ COM memo — e a nota que dizia o contrário estava certa sobre a wave errada
//!
//! Esta secção dizia *"aqui não se assa nada: resolve-se um `VecPath` e clona-se"*, e concluía que
//! um memo seria *"estado derivado à espera de envenenar o undo"*. **Medido em 2026-08-30, e o que
//! ela não via era o `cooked()`:**
//!
//! | arte | um `cooked()` |
//! |---|---|
//! | simples (`Cow::Borrowed`) | **46,6 ns** |
//! | VIVA (raio de quina + pilha de efeitos) | **17 529 ns** — `376×` |
//!
//! A lei sai limpa: `custo ≈ P × G × cooked(arte)` — o `cooked()` é **95–98%** do total, e a
//! expansão de objecto que a auditoria suspeitava é `0,18%`. Com arte viva, numa cena de 200
//! caminhos:
//!
//! | pincéis × membros | por quadro | % de 16,7 ms |
//! |---|---|---|
//! | `1 × 16` | `0,300 ms` | **1,80%** |
//! | `10 × 16` | `2,860 ms` | **17,1%** |
//! | `50 × 16` | `14,278 ms` | **85,5%** |
//!
//! ⚠️ **A wave do grupo NÃO criou a dívida — multiplicou-a por `G`.** A `G = 1` a rota nova e a
//! velha dão o mesmo relógio (`1,00×`), e `50 × 1` já custava `0,887 ms` (**5,34% de um quadro**)
//! antes dela. *Uma acusação de perf a uma wave pode estar a apontar para o que ela ampliou.*
//!
//! ⚠️ **E é `O(P × G × S)`, não `O(P × G)`:** o `subtree_paths` filtra `scene.paths()` e o
//! `VecScene::path` é um `find` — os dois lineares na CENA. A `S = 6400` a expansão passa a ser
//! **93%** do total. A `S = 200` ela é ruído, e é por isso que a acusação se lê correcta ali.
//!
//! ⭐ O memo paga-se **881–1484×** (comparar a chave: `12,9 ns` a `G=1`, `349,6 ns` a `G=16`).
//! Instrumento: [`crate::brush_live_cost_probe`].

use ph2d_vec_render::BrushArts;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};
use std::collections::BTreeMap;

/// **Com o que a arte de um pincel foi resolvida** — as CINCO coisas que a [`resolve`] lê.
///
/// ⚠️⚠️ **A lição que esta jornada pagou três vezes:** *um memo cuja chave não contém tudo o que o
/// produtor lê não é um memo — é um congelamento.* Cada campo aqui é uma coisa que muda o desenho
/// **sem tocar** nos outros:
///
/// 1. o **CONTEÚDO da lista expandida** — e ele responde por **três** perguntas de uma vez, porque
///    um [`VecPath`] carrega o **próprio id**: *qual é a arte* (trocar o `art` autorado muda a
///    lista), *quais são os membros* (agrupar, desagrupar, reordenar, apagar ou reparentar mudam-na
///    **sem tocar num vértice**, e a ordem é a de **z**) e *como cada um é desenhado*;
/// 2. a **POSE** de cada membro — num grupo ela **É** a disposição, e a ausência dela produziu o
///    report de 30/08 na estampa.
///
/// A **hospedeira** não é campo: ela é a chave do mapa. E a recusa de ciclo não precisa de entrar —
/// quando ela morde, a lista sai vazia e a entrada nem chega a existir.
///
/// ⛔⛔ **TRÊS CAMPOS FORAM APAGADOS POR MUTAÇÃO SOBREVIVENTE** (`art`, `membros`, e a redundância
/// entre eles): as mutações que os tornavam constantes **não mataram o gate**, e a causa não era um
/// gate fraco — era código **inerte**. *Um campo de chave que nenhuma mutação mata é peso que a
/// próxima pessoa vai tentar manter em sincronia com o resto.*
#[derive(Clone, PartialEq)]
struct Key {
    conteudo: Vec<VecPath>,
    pose: Vec<[f64; 6]>,
}

/// O memo da arte dos pincéis — irmão do [`crate::texture_pattern_live::TexturePatternLive`].
#[derive(Default)]
pub(crate) struct BrushLive {
    arts: BrushArts,
    keys: BTreeMap<VecPathId, Key>,
}

impl BrushLive {
    /// Re-resolve **só o que mudou**, e devolve o mapa do quadro.
    ///
    /// ⚠️ A chave é montada **sem** cozinhar (o `cooked()` é 95–98% do custo): ela lê o `VecPath`
    /// autorado, que é o que o `cooked()` consome. Cozinhar para decidir se é preciso cozinhar seria
    /// o memo a pagar exactamente o que ele existe para evitar.
    pub(crate) fn resolve(
        &mut self,
        scene: &VecScene,
        object_of: &dyn Fn(VecPathId) -> Vec<VecPathId>,
        xforms: &ph2d_vec_scene::VecXforms,
    ) -> &BrushArts {
        let mut vivos = std::collections::BTreeSet::new();
        for path in scene.paths() {
            let Some(alvo) = path
                .stroke
                .as_ref()
                .and_then(ph2d_vec_scene::StrokeSpec::brush)
                .and_then(|b| b.art)
            else {
                continue;
            };
            let membros = crate::texture_pattern_live::art_members(path.id, alvo, object_of);
            if membros.is_empty() {
                continue;
            }
            let conteudo: Vec<VecPath> = membros
                .iter()
                .filter_map(|m| scene.path(*m).cloned())
                .collect();
            if conteudo.len() != membros.len() {
                continue;
            }
            let key = Key {
                pose: membros
                    .iter()
                    .map(|m| xforms.get(m).copied().unwrap_or_default().0)
                    .collect(),
                conteudo,
            };
            vivos.insert(path.id);
            if self.keys.get(&path.id) == Some(&key) {
                continue;
            }
            if let Some(art) = art_of(scene, path.id, alvo, object_of, xforms) {
                self.arts.insert(path.id, art);
                self.keys.insert(path.id, key);
            }
        }
        // ⚠️ **As duas metades da varredura.** Marcar sem desmarcar deixaria a arte de um traço que
        // deixou de ser pincel a ser desenhada para sempre — a mesma lei do memo da estampa.
        self.arts.retain(|k, _| vivos.contains(k));
        self.keys.retain(|k, _| vivos.contains(k));
        &self.arts
    }
}

/// A arte de cada pincel da cena, pela forma HOSPEDEIRA — **sem memo**.
///
/// ⚠️ Fica para quem resolve **uma vez** (o assado do Motion, o re-cook do FX): ali um memo não tem
/// onde viver, e a resposta é pedida uma vez por gesto, não por quadro.
#[must_use]
pub(crate) fn resolve(
    scene: &VecScene,
    object_of: &dyn Fn(VecPathId) -> Vec<VecPathId>,
    xforms: &ph2d_vec_scene::VecXforms,
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
        if let Some(art) = art_of(scene, path.id, alvo, object_of, xforms) {
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
    xforms: &ph2d_vec_scene::VecXforms,
) -> Option<Vec<VecPath>> {
    let membros = crate::texture_pattern_live::art_members(host, art, object_of);
    let out: Vec<VecPath> = membros
        .iter()
        .filter_map(|m| {
            scene.path(*m).map(|p| {
                let mut c = p.cooked().into_owned();
                // ⛔⛔ **A POSE ENTRA AQUI, e a ausência dela era o report de 30/08 noutra tinta.**
                //
                // Desde o ADR-0110 a geometria é **local** e quem a põe no mundo é o `Xform`. Sem
                // esta linha, o arranjo dos membros era o que a geometria AUTORADA dizia, e não onde
                // o artista os pôs: uma dupla desenhada e agrupada funcionava por acidente, e
                // **mexer** num membro depois não movia nada.
                //
                // ⚠️ Com arte de UMA forma isto não tem sujeito (o motivo é re-enquadrado na guia de
                // qualquer maneira, e o `art_at_height` re-centra) — *a wave do grupo não criou o
                // defeito, tornou-o alcançável: num GRUPO, a pose **é** a disposição.*
                if let Some(xf) = xforms.get(m) {
                    ph2d_vec_scene::bake_xform(&mut c, xf);
                }
                c
            })
        })
        .collect();
    (!out.is_empty()).then_some(out)
}

#[cfg(test)]
#[path = "brush_live_tests.rs"]
mod tests;
