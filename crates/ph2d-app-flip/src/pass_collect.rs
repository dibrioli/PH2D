//! As camadas activas AGORA, na ordem de composição — irmão de `pass.rs` por tecto de LOC.
//!
//! Corte mecânico: a função saiu inteira, verbatim; abriu-se só a visibilidade para o módulo pai.

use super::*;

/// As camadas ativas AGORA, na ordem de composição (objeto por objeto; dentro de
/// cada objeto, de baixo p/ cima = ordem do slice; só visíveis). Cada objeto
/// amostra pelo SEU FPS. NÃO empacota — isso é sob demanda no cache (`ensure_tess`),
/// pra troca de quadro barata.
///
/// O `preview` (traço em curso) é **dobrado na camada-alvo** — a do [`FlipTarget`]
/// (o desenho em edição; fallback: topo, igual ao bake) — para compor pelo blend/opacity dela
/// em tempo real, na posição z certa (inclusive uma camada ainda VAZIA, sintetizada
/// aqui). Devolve `(camadas, preview_não_atribuído)`: o 2º é `Some` só quando a
/// camada-alvo é oculta/inexistente (cai no overlay Normal — nunca desenhar às cegas).
pub(super) fn collect_layers<'a>(
    flip: &'a FlipDoc,
    playhead: &Playhead,
    preview: Option<&'a FlipGpuData>,
    target: FlipTarget,
    models: &[(FlipObjectId, Xform)],
    ghosts: Option<crate::pass_ghosts::GhostSources<'_>>,
    peek: Option<crate::peek::PeekDir>,
) -> (Vec<LayerRef<'a>>, Option<&'a FlipGpuData>) {
    // Camada-alvo do preview: a do alvo da autoria — a MESMA que o `bake_stroke`
    // usa. `None` sem preview ou sem desenho em edição.
    let resolved = target.resolve(flip).map(|(oid, lid)| (oid.0, lid));
    let target: Option<(u64, LayerId)> = preview.and(resolved);

    // A camada que o PEEK retima — a MESMA resolução do alvo do preview: é a camada
    // que o animador está folheando.
    let peek_target: Option<(u64, LayerId)> = peek.and(resolved);

    let mut out = Vec::new();
    let mut unfolded = preview; // vira None assim que o preview é dobrado
    for obj in flip.objects() {
        let frame = obj.frame_at(playhead);
        // O afim LOCAL→mundo do objeto (ADR-0111). Ausente = identidade (o objeto
        // nunca foi movido; a geometria ainda É mundo).
        let model = models
            .iter()
            .find(|(id, _)| *id == obj.id)
            .map_or(Xform::IDENTITY, |(_, x)| *x);
        for layer in obj.layers() {
            if !layer.visible {
                continue; // oculta não contribui (preview numa oculta cai no fallback)
            }
            let this_preview = if target == Some((obj.id.0, layer.id)) {
                unfolded.take()
            } else {
                None
            };
            // O PEEK: a camada-alvo amostra a folha VIZINHA (âncora = chave ativa;
            // `flip_peek::peek_frame`). As outras ficam no quadro — são o contexto
            // sobre o qual se folheia. Sem peek, `sample == frame` e nada muda.
            let sample = match peek {
                Some(dir) if peek_target == Some((obj.id.0, layer.id)) => {
                    crate::peek::peek_frame(layer, frame, dir)
                }
                _ => frame,
            };
            // **Pelo CICLO** (`drawing_at_cycled`), não pelo caminho cru: é aqui que
            // Loop/Ping-Pong existem. Amostrar cru fazia o último desenho segurar para
            // sempre e os ciclos não faziam NADA (o bug do 1º corte).
            let did = layer.drawing_at_cycled(sample);
            let drawing = did.and_then(|d| obj.drawing(d));
            let has_geo = drawing.is_some_and(|d| !d.strokes.is_empty());

            // Os fantasmas DESTA camada entram ANTES dela na pilha (mais para o
            // fundo) — e DEPOIS de todas as camadas de baixo. É isto que faz o
            // fantasma da camada de cima aparecer POR CIMA da arte da camada de
            // baixo: ele é uma fatia da pilha, não um passe por baixo de tudo.
            // Blend Normal + opacity 1: o fade/opacidade já estão no alpha do tint
            // (aplicá-los de novo pelo compositor os elevaria ao quadrado, e um
            // blend Multiply da camada tingiria o fantasma com a arte de baixo).
            if let Some(sources) = ghosts {
                // No quadro-FONTE (o do ciclo): os vizinhos do desenho que está NA
                // TELA. No quadro cru, um Loop na 2ª volta não teria vizinho nenhum.
                let src = layer.source_frame(frame);
                for g in crate::pass_ghosts::collect(obj, layer, src, playhead, sources) {
                    out.push(LayerRef {
                        key: ghost_key(obj.id.0, layer.id.0, g.delta),
                        obj: obj.id.0,
                        blend: ph2d_flip::BlendMode::default().to_u8(),
                        opacity: 1.0,
                        cache_key: (obj.id.0, g.drawing_id),
                        drawing: Some(g.drawing),
                        preview: None,
                        // **Cada fantasma na POSE DA SUA chave** (W7.2): o fantasma
                        // mostra onde o desenho ESTAVA — e "onde" inclui o lugar, não
                        // só a forma. Herdar a pose do quadro corrente empilharia todos
                        // os fantasmas em cima da arte de agora. O `shift` é o Shift &
                        // Trace (a folha deslizada) — identidade fora do modo.
                        model: ph2d_flip_entities::transform::art_to_world_traced(
                            &model,
                            layer.frame_pose(g.key),
                            g.shift,
                        ),
                        ghost: Some((g.tint, g.alpha)),
                        // O fantasma paralaxa NO plano da própria camada (um ghost de
                        // fundo mora no fundo), senão ele descolaria da arte que sombreia.
                        depth: layer.depth,
                    });
                }
            }

            if !has_geo && this_preview.is_none() {
                continue; // sem geometria e sem preview aqui = não compõe
            }
            out.push(LayerRef {
                key: layer_key(obj.id.0, layer.id.0),
                obj: obj.id.0,
                blend: layer.blend.to_u8(),
                opacity: layer.opacity,
                cache_key: (obj.id.0, did.map_or(u32::MAX, |d| d.0)),
                drawing: if has_geo { drawing } else { None },
                preview: this_preview,
                // A pose da chave EXIBIDA — amostrada pelo mesmo mapa do desenho
                // (`offset_at_cycled`), senão a arte da 2ª volta de um Loop sairia no
                // lugar de um quadro que não existe.
                //
                // O PREVIEW (o traço vivo) entra nesta MESMA fatia e por isso recebe a
                // mesma pose — e está certo: ele já foi convertido para o espaço do
                // DESENHO pelo funil de entrada (`flip_active_world_to_local`, que desfaz
                // a pose da chave ativa). As duas pontas usam a mesma transform, que é a
                // única forma de o preview não folgar do traço assado.
                model: art_to_world(&model, layer.pose_at_cycled(sample)),
                ghost: None,
                depth: layer.depth,
            });
        }
    }
    (out, unfolded)
}
