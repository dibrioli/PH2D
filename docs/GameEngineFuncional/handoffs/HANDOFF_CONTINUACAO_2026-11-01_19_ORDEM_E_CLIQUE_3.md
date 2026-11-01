# Handoff de continuação — GEF 19 (ordem do desenho e do clique), 01/11 (3.ª janela)

Linha `line/GameEngineFuncional`, worktree `Worktrees/line-GameEngineFuncional`, sobre o main `422fec2027`
(o main não andou). Antes: [2.ª janela](HANDOFF_CONTINUACAO_2026-11-01_19_ORDEM_E_CLIQUE.md) (E1–E7b, item 0 do
AtlasFull, E4, restos a/b da E3) e o [plano](../19_ordem_e_clique_plano.md). Nada disto foi ao dono ainda.

## Feito nesta janela (não reconstruir)

| item | commit | o quê |
|---|---|---|
| item 0 (jogador ausente) | — (na msg de `fad7ab4748`) | NÃO é defeito: `PH2D_GAME_SMOKE=2` no main dá a MESMA foto (`fotos_19/item0_game_smoke_2_main.png`); o jogador nasce em x=−8 m, à esquerda da vista inicial (atrás da Hierarchy) |
| 1 clique no Motion | `fad7ab4748` | `ph2d_app_motion::motion_pick` (`Alvo`, `CacheDoClique`): donos das cópias sob o ponto, a de cima primeiro. Dispositivo: `read_instances` + `read_imagens_do_grupo` + `read_formas`, donos por `faixas_dos_sinks`/`imagens_do_grupo().faixas()`; CPU: `pump` (`instancias`+`ladrilhos` por baixo, `vetores` por cima). Caixa local pela inversa de `P + basis·(anchor + q·size)`. Uma leitura por quadro (esquecida no `present`). `PickWorld.motion` nos 6 sítios |
| 2 resto c da E3 | `480282fb1d` | `PlacaDeFormas.fonte_imagens`; `luz::donos_das_imagens`; `faixa::corta` corta cópias E imagens pelos donos, pedaços emparelhados (corrida ORDENADA continua alinhada) |
| 3 E7c | `5680fbd6d1` | gate `os_limites_da_camara_do_nivel_1_sao_as_paredes` |
| 4 trava × grupo | `22e1c55130` | `ph2d_ecs::pick_target`: clique na arte de filho agrupado pega o grupo MAIS DE FORA; grupo com `Locked` não pega. Vetor (`view.locked` local ao pick só com o cadeado) e Flip (`pick_all_at_world`: escondido ou `Locked` próprio) devolvem o filho; marquee e gizmo continuam com `is_locked_for_edit`. **Decisão técnica derivada das leis do dono 26/05 + 01/11 — confirmar no smoke** |
| 6 cena do smoke | (este commit) | `PH2D_ORDEM_SMOKE=1|2` (`ph2d_app_game::ordem_smoke`: tabela `ORDEM`, `pose`, `lugar`, `ordena` por baixo das raízes de antes; router no `smoke::ROUTERS`). Card (imagem pelo Add ▸ Sprite) · Pile e Swarm (o `motion_objeto_smoke` arma também com esta env) · Ink (`ph2d_app_flip::ordem_smoke::traco`) · Face (`ph2d_app_pixel::smoke::rosto`, escala 6) · Macro · Camera. Shell: `ordem_smoke()` em `janela_do_jogo.rs`, chamado logo depois do `game_smoke` |

## ⚠️ O que a foto da cena MOSTROU (abrir a próxima janela por aqui)

Fotos: `fotos_19/ordem_smoke_1.png` e `ordem_smoke_2.png` (`fotografa_cena.sh … 1930 1040 … 10`).

A. **DEFEITO — as cópias do Swarm não obedecem à ordem:** no `=2` o Card (último ⇒ por cima) cobre o Ink e o
   Face, mas as cópias vermelhas do Swarm (molde «Ball» = SPRITE ⇒ linhas de `pump.instances`, passe de sprites
   do Motion) continuam por cima do Card. Suspeito: as instâncias de sprite do Motion não entram na faixa do
   rank do objecto Motion (a E3 partiu a PLACA de formas por faixa; as linhas de sprite do cozimento desenham
   noutro passe). Medir antes: onde o presente desenha `pump.instances`/o buffer de instâncias do dispositivo
   relativamente às faixas (`present.rs`, `draw_bands`), e com `PH2D_GPU_COOK=0` (CPU) × omissão.
B. **A Pile não aparece** em nenhuma das duas (só o anel do objecto na origem). Conferir se as formas da cena
   114 caíram para fora da vista/estão por baixo do Card (no `=1` a Pile está acima do Card, logo devia ver-se)
   ou se a placa não desenha fora da ferramenta Motion nesta montagem. Comparar com `PH2D_MOTION_OBJETO_SMOKE=1`
   sozinho (lá as duas desenham).
C. **O modo fica em Draw (do Pixel)** apesar do `ObjectMode::Enter(Object)` empurrado no fim: o Pixel nasce
   em edição e, provavelmente, o frame do Pixel re-entra. Ver `ph2d_app_pixel::object_add::spawn` /
   `frame_in(editing)` — talvez seja `editing` vindo do modo do objecto seleccionado (seleccionei o Card).
D. Medir ainda o **item 5** (perf do `LayerCompositor` do Flip com faixas intercaladas, `PH2D_FLIP_STATS=1`)
   nesta cena.

## ABERTO, por ordem

1. A, B, C acima (A é defeito do produto; B e C da cena). Fotografar de novo as duas ordens.
2. Item 5 (perf Flip intercalado) — medir nesta cena; cura só com número.
3. **Provas de mutação em lote** (`mutacao`): as da lista anterior (`sort_by_key` do pick, `cycle_step`,
   `stack_key`, `cola_a_faixa`, arm Motion do passe de baixo, `FramePlan.main`, `adormece_palcos`,
   `pass::so_estes`, filtro `fora` dos palcos) + NOVAS: `motion_pick::da_cpu` sem `filter(visible)`, formas
   antes das imagens; `donos_das_imagens` a ler `fonte`; `corta` sem emparelhar; limites `max: [17, 9]`;
   `pick_target` a devolver o filho; `lugar` sem o `nivel == 2`.
4. **Testes de placa por nome** (`PH2D_GPU=1 … --profile smoke -- --ignored`): `ph2d-app-flip pass_luz_tests`,
   `ph2d-app-motion` `faixa`/`palcos`/`motion_shape_luz` ignorados, os 4 gates da shell da placa/palcos/
   miniatura; e um gate de placa para `motion_pick::do_dispositivo` (cozer um objecto Motion, clicar numa cópia).
5. Smoke do dono (passos numerados, `PH2D_ORDEM_SMOKE=1` e `=2`; clique onde as peças se sobrepõem, e num
   filho de grupo) → só por ordem dele, fecho §1.5.9 (lista de crates tocadas: + ph2d-ecs, ph2d-app-pixel).

## Medido / riscos

- `the_shell_only_shrinks` verde depois do item 1 (+~38 linhas na shell); a cena acrescentou ~60 — reconferir.
- O realce do ponteiro passa a ler o Motion: uma leitura de placa por quadro enquanto o rato se move sobre o
  canvas com cozimento vivo (`n × 64 B` + o `poll` que espera a fila). Não medido — medir na cena antes do fecho.
- O `motion_objeto_smoke` agora também arma com `PH2D_ORDEM_SMOKE` (o `=1` dele).
