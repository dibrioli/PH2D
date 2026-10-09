# Handoff de continuação — Luz 2D, W3 → W4 (2026-10-09)

- **Linha:** `line/Luz2D` · worktree `Worktrees/line-Luz2D` · merge-base `2a925859d` (sem rebase pendente).
- **Estado:** EM CURSO. A secção «Desenho» foi escrita ANTES do código (pedido da W3); o resto fecha no fim.

## Desenho — onde o solver entra no quadro e quem o possui (escrito antes do código)

Leitura do quadro de hoje (`shells/desktop/src/render_loop/`): `present.rs::present_world_sprites` →
[faixas de baixo] → `SpriteRenderer::render_with_streams` (`ph2d-render/src/renderer_draw.rs`, encoder e
submit próprios) → Flip (`present.rs`) → emissivo/glow (`present_fx.rs`, re-desenha sprites ISOLADAS pelo
MESMO pipeline num RT próprio) → `present_chrome.rs`: tonemap (hoje `BYPASS_LUT` = `clamp(hdr, 0, 1)`,
`tonemap.wgsl:146`) → faixas de cima → Vello (chrome **e** documento na MESMA cena, salvo em faixas) →
compositor. O documento só sai da cena do chrome no modo FAIXAS (ADR-0154): `doc_bands_of(ordem, forçar)`
codifica-o por faixa e `BandBlit` cola cada faixa no acumulador `world_rt` (`BandSource::Vector`).

| peça | onde | dono |
|---|---|---|
| coleta + guarda + grade ancorada + `Solver` | uma fase do quadro ANTES de `fase_vector_bands` decide `luz_ativa`; o solver é gravado no INÍCIO de `present_world_sprites`, encoder e submit próprios (a fila ordena-o antes do passe de sprite) | `ph2d-app-light2d::LuzDoQuadro` (crate nova da família); a shell guarda UM campo e chama três portas |
| `receber.wgsl` (`luz_em`) | `ph2d-render/src/shaders/` — os DOIS consumidores (sprite e colagem do vetor) vivem nesta crate | `ph2d-render` (foundational, ADR-0107) |
| sprite recebe | `@group(3)` em TODAS as pipelines de sprite, ligado uma vez por passe — o idioma do grupo da PELE (`pipeline.rs`): um grupo NEUTRO (`ligada = 0`, ramo não tomado) quando não há luz | `SpriteRenderer::receber_luz` |
| vetor recebe | luz acesa ⇒ o quadro vai às FAIXAS (`doc_bands_of(.., forçar)`, `plan.banded |=`), e a colagem `BandSource::Vector`/`Formas` faz `linear(albedo) × E → a MESMA função de tonemap → sRGB`; a grade do chrome (`grid_behind`) NÃO recebe | `BandBlit::receber_luz` |
| «não recebe luz» | bit 31 do `flip_uv` (tirado do índice de malha: 23 bits = 8 M malhas), posto na coleta das instâncias pelo `SimRef`, como as RETIDAS do vidro | `ph2d-render::sprite_collect` |
| glow/emissivo | o passe isolado do `present_fx` liga o grupo NEUTRO — a emissão é luz, não recebe luz | `SpriteRenderer` |

**Guarda (G6) — decisão desta onda:** a cena acende **sse tem ao menos uma `Luz`** (Ponto ou Ambiente).
O plano dizia «uma `Luz` **ou** um emissor ativo»; medido: a cena de smoke `emissive_smoke.rs` e todo
projeto com `SpriteEmissive` ficariam ESCUROS no dia em que isto entrasse (sem `Luz`, `E = 0` fora do
brilho). Com a guarda pela `Luz`, todo projeto anterior é o quadro de hoje, e a sprite que já brilha
ilumina assim que a cena tem luz (a promessa «sem migração» do §2.3 continua de pé). A escuridão
ambiente é a `Luz` Ambiente — o papel do `CanvasModulate` do Godot.
