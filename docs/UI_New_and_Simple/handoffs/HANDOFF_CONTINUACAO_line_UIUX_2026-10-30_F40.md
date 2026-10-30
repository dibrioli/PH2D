# HANDOFF de continuação — line/UIUX, F40 (30/10) → F41

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD `b097edfe0a`, base = main `422fec2027`
(stack de 30/10: Vello 0.11 + wgpu 30). **Nada integrado, nada enviado.** O dono pediu este handoff
ANTES do smoke da F40 (30/10, «prompt para nova janela resolver tudo») — o smoke F37–F40 foi
corrido pelo agente e está verde (§3), mas o dono ainda não o fez.

## 1 — Commits da linha (merge-base..HEAD)

| sha | o quê |
|---|---|
| `05b196c6fc` | fixture REAL `vec_scene_v24_pilha_cad.bin` (gravada pelo `5228501c26`), braço V24 do `from_bytes` visto vermelho por 2 mutações; planos F30–F32 renumerados (LISTA 10 · MUNDO 11 · MESA 12 · degraus 203→207 · VEC 24→25) |
| `db1480ac66` | sonda A/B/C do Zig Zag no vale (removida no `b097edfe0a`; fotos em `img_F40/zigzag_*`) |
| `72fb5974a7` | 16 cores cruas → tokens em `ph2d-vec-render` (`DIVIDA` vazia); guias/seleção/laço = destaque do tema |
| `ac80f341aa` | família de canal OSSO = 13 (`ph2d-ecs/src/canal_osso.rs`, `ph2d-app-skeleton/src/canais_osso.rs`): Length/Strength/Segments com losango |
| `36d77788d3` | comentários do `ph2d-app-skeleton` fechados |
| `adde7f1586` | curva do osso = pré-visualização POR CAMPO (`ph2d-timeline-preview::campo_do_osso`); losango em Mix/Softness/Chain/Offset |
| `8776714ebe` | `sob_escopo_de_memoria` ganha escopo de validação por fora (G29) |
| `b2c88ebe55` | pilha de FX sob escopo de memória, substituta 1×1 (G30/G31) |
| `c531ef677c` | intermédia do Vello recusada nunca instalada; textura do FX idem (G32/G33) |
| `faffb5cacc` | fotos da rodada F37/F38/F39 |
| `0ba108818a` | `View::Toggle` no canal: 0 = desligada, 1 = ligada; `DOC_VERSION` 24→25 (`doc_v24.rs`, v → 1 − v); fixture `timeline_doc_v24.bin` |
| `c03c9df537` `e37bd7ba11` | cena `PH2D_VEC_BONE_SMOKE=8` (+ `PH2D_VEC_BONE_SMOKE_TOCA=1`) |
| `75d231b556` | fmt |
| `b097edfe0a` | **lei C do dono (30/10): o nó do Zig Zag num vale é cortado**, e em cada passada do Sketch — `ph2d-vec-scene/src/fx_zigzag_corta.rs` (≤ ¼ do contorno, Newton + De Casteljau, quina no cruzamento); G34/G35 + 3 mutações vermelhas |

## 2 — ABERTO (a F41 resolve tudo — ordem do dono 30/10)

1. **⚠️ «Follow Value» ligado a um Toggle lê ao contrário depois do `0ba108818a`.** A regra das
   Constraints guarda/lê o valor do canal do Toggle na convenção antiga (0 = ligada). Curar muda o
   formato gravado da família Constraints (degrau de `PROJECT_SCHEMA`, contado por
   `python3 scripts/schema-recount.py`, com migração v→1−v só onde a fonte é um Toggle) + fixture real
   gravada pelo código de ANTES. **Bloqueia a integração da linha.** Nenhuma linha Constraints viva
   (`git worktree list` 30/10) — o dono autorizou tocar.
2. **Length do osso animado não empurra o filho nem estica o desenho** (mão e curva fazem o mesmo:
   ambos por `knobs::apply`). **Decisão do DONO pendente** — perguntar primeiro, com foto da cena
   `PH2D_VEC_BONE_SMOKE=8`. Referência: no Blender/Spine o comprimento (escala Y em pose) leva os
   filhos na cauda e deforma a malha.
3. **F30 §6 item 6c — matriz de camadas da física não animável** (família Física; nenhuma linha
   viva). Pôr os bits da matriz num canal (família nova ou MUNDO 11), losango no painel.
4. **F30 §6 item 8 — regras do mundo e da mesa fora do Desfazer**: vivem em `gfx.physics`, fora do
   mundo que o `ProjectState::capture` fotografa. Curar = o Desfazer passa a levá-las (diff), gate de
   ida-e-volta.
5. Teste que falta: a seleção/guias/laço do vetor pintam com `ColorToken::Accent` do tema (hoje só
   `o_overlay_vetorial_nao_tem_cor_crua` prova a ausência de cor crua).
6. Teste que falta: clique no losango de **Segments** do painel do esqueleto grava a chave (o de
   clique usa Length).

Fora de escopo (ordem do dono): o recuo da Luz 2D sob blocos pequenos (`docs/Luz2D/BUGS_luz2d.md` §8).

## 3 — Smoke F37–F40 (corrido pelo agente, 30/10, `[gpu-erro]` = 0 em todas)

Fotos em `img_F40/smoke_30/`. F37 cena 41 com ~1 015 MB livres: forma + «Bevel». F38 cena 41 com
~484 MB livres: janela cinzenta (533 MiB reservados não cabem), volta sozinha em 75 s. F39 cena 40:
seis nuvens. F40 `PH2D_VEC_BONE_SMOKE=8`: losango cheio em Length; a tocar, Length 2.933 e losango
vazado. Zig Zag C: `img_F40/zigzag_C_antes_depois.png`. Binário `target/smoke` compilado no `b097edfe0a`.

## 4 — Números contados (não escolha)

Canais: CONSTRAINT 8 · CAD 9 · LISTA 10 · MUNDO 11 · MESA 12 · OSSO 13. `PROJECT_SCHEMA` da linha
203→207. `VEC_SCENE_SCHEMA` 25. `TimelineDoc DOC_VERSION` 25. Gates da linha até G35.
