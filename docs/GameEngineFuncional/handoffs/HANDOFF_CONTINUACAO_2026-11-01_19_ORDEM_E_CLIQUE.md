# Handoff de continuação — GEF onda 19 (ordem do desenho e do clique), 01/11 (2.ª janela)

Linha `line/GameEngineFuncional`, worktree `Worktrees/line-GameEngineFuncional`, sobre o main `422fec2027`
(o main não andou até 01/11). Plano e decisões: [19_ordem_e_clique_plano.md](../19_ordem_e_clique_plano.md);
decisões do dono em `project-memory/project_hierarchy_order_rules_drawing_and_click_2026_11_01.md`.
A onda 18c (miniatura) foi aprovada no smoke de 31/10; o dono mandou NÃO fechar a linha ainda.
**Dono 01/11:** no modo Draw do Flip a coluna da direita continua a abrir na aba Flip (pergunta fechada).

## Feito (não reconstruir)

| etapa | commits | o quê |
|---|---|---|
| E1 | `7d703c2d79` | clique pela pilha do desenho (`FrameOrder::stack_key`), ícones à frente sem retorno cedo, trava em toda família, `pick_order::cycle_step` (1 clique/degrau) |
| E2 | `c2d460075d` | `Family::Pixel`; arte do Pixel na cena da faixa |
| E3 | `06494dff4e`, `cc02d5e412` | `Family::Motion`; placa colada por faixa (`motion_shape_placa_faixa.rs`) |
| E5 | `b1c99a7bbf`, `1db4cf3d50` | Z Index em todo tipo do Add (gate) |
| E6 | `4bf8ac8efe` | ícone da Macro no canvas (`ph2d_app_macros::icone_no_canvas`) |
| E7b | `26c6e5c2b3` | repetição de camada escalada |
| item 0 | `2371198ee8` | o `AtlasFull {4,4}` era HERDADO do main (F37 `9617f571ff`: o ladrilho branco 64² nasce na origem ⇒ átlas de 64 cheio), não do stack nem desta linha. Cura: átlas 128 em 5 sítios (3 relatados + 2 em `motion_filter_imagens_gpu_tests.rs`); 9 testes de placa verdes |
| E4 | `9ba9527c26` | `Family::Flip` (old_stack logo acima dos sprites = o `game_rt` de sempre); `sim_extract::flip_participant`; `FramePlan.main = (última de sprites, +Flip logo acima)` — sem sprites, a última Flip; baixo `take(main.0)`, cima `skip(main.1)`; faixa Flip fora do main: `GameRt::clear` → `pass::render(objects)` → tonemap → colagem `Sprites`. `ph2d_app_flip::pass_quadro::Quadro` (fantasmas/peek saíram da shell); `FlipState.preview_do_quadro`. **Também curou** o duplo desenho das faixas de cena sem sprites (`[P, V]`, `[M, V]`: as duas metades cobriam tudo). O olho da Hierarchy passa a esconder o Flip; o clique conhece o Flip pelo rank |
| E3 resto b | `3a1957bdf0` | camada filtrada + lente do Motion no sítio da faixa do TOPO também abaixo da última de sprites: as duas metades chamam `faixa::cola`; `ColaMotion` recebe o `SpriteRenderer`; `Topo::JaColada` saiu |
| E3 resto a | `68154a91f0` | `Family::Palco`: o palco de uma macro pousa no rank da entidade Macro; `FrameOrder::adormece_palcos` (Macro sem palco a tocar dá o rank ao vizinho — sem faixa vazia); `PalcosDasMacros::{pousa, lentes}` com filtro por entidade; fora da ordem (Macro escondida) pousa no fim, como antes |
| E7a | — | **decidido, sem código:** fora do Play a paralaxe é conduzida pela câmera DO JOGO (parada no editor) ⇒ os fundos aparecem na pose do início do jogo — coerente com «pose autorada». Foto: `fotos_19/e7a_game_smoke_2_editor.png` |

## ABERTO, por ordem

0. **⚠️ CONFERIR PRIMEIRO:** na foto `fotos_19/e7a_game_smoke_2_editor.png` (`PH2D_GAME_SMOKE=2`, editor, Player
   seleccionado) o **jogador não aparece no canvas** (moedas, chão, fundos sim). Fotografar a MESMA cena com o
   binário do main (worktree do main, `--profile smoke`) e com esta linha: se o main mostra o jogador, bissectar
   `422fec2027..HEAD` (suspeitos: E2/E4 — `FramePlan.main`/`rank_window`, ou o Player a cair fora da janela de
   rank). Se o main também não mostra, é posição/câmera — registar e seguir.
1. **O clique nas formas do Motion** (medido por leitura: NÃO existe hit-test de Motion no canvas;
   `hover_highlight.rs:85` não tem termo de Motion). Rota: CPU — no clique, para cada cópia de
   `motion_shape_luz::donos` (cópias da `PlacaDeFormas`: `pos`, `size`, `basis`, `anchor`, `geometry`), ponto →
   local pela inversa e teste contra a caixa da geometria (`VecPathStore`, `path_bounds_under`); dispositivo
   (`do_dispositivo: Some(n)`) — UM readback no clique do buffer de instâncias (`n × 64 B`), donos por
   `gpu_cook.faixas_dos_sinks()`. Ler antes a semântica de `anchor` e o que `prepara(path)` guarda. Entra em
   `pick_objects_at` e ordena-se pelo `stack_key` (que já conhece o Motion). Palcos das macros idem (dono = Macro).
2. **E3 resto c — imagens de grupo** (instâncias `geometry_id == 0` numa corrida): vão sempre com a faixa Motion
   do TOPO (`faixa::corta`, `imagens: topo`). Mecanismo: a imagem não tem `fonte` (o dono vem de
   `pump.faixas[f].vetores` em espaço de cópias; um sink só de imagens dá faixa vazia). Cura: guardar por imagem o
   índice `k` em `insts` (`motion_shape_placa_monta.rs:~77`), dono = sink cuja `vetores` contém `k`, e partir
   `c.imagens` pela faixa; na rota do dispositivo o equivalente de `faixas_dos_sinks`. Só aparece com ≥2 objectos
   Motion intercalados com outra família.
3. **E7c** — gate dos limites do nível 1 (`smoke.rs:285-325`; limites = paredes, medido no código).
4. **Trava × grupo:** um filho de `GroupedChildren` deixou de se pegar em TODA família com a E1; avaliar se o
   clique num filho agrupado deve pegar o grupo.
5. **Perf a medir (E4):** com faixas Flip intercaladas, o `LayerCompositor` do Flip cresce a capacidade pela
   chamada (não pelo quadro) ⇒ o LRU pode despejar as fatias da outra faixa e re-rasterizar todo quadro
   (`PH2D_FLIP_STATS=1`: «rasterizada(s)/pulada(s)»). Correcto, só caro; a cura é reservar a capacidade pelo
   total de camadas visíveis do quadro.
6. **A cena do smoke (UM bloco):** cena nova (env livre: `PH2D_ORDEM_SMOKE=1|2`, `=2` = RootOrder invertido) com
   imagem + Flip com traço + Pixel Sprite pintado + objecto Motion com FORMAS e simulação com campo + câmera +
   Macro, sobrepostos. Moldes (explorador 01/11): `motion_object_smoke_sink.rs` (`PH2D_MOTION_OBJ_SMOKE=9`, já
   mistura vetor+Flip+sprite); Flip: `ph2d-app-flip/src/object_add.rs` + `ph2d-app-light2d/src/smoke_flip.rs:72-110`;
   Pixel: `ph2d-app-pixel/src/smoke.rs:~86-98`; Motion: `motion_objeto_smoke.rs:80-140`; Macro:
   `ph2d-app-macros/src/object_add.rs:21-32`; câmera: `ph2d-app-components/src/object_add.rs:13`. Registo por
   `SmokeRouter` (`ph2d-app-host/src/family.rs:10`). Fotografar as duas ordens (`fotografa_cena.sh`) ANTES do
   smoke, mais o clique (`smoke_mao.rs`) e o ícone da Macro.
7. **Provas de mutação em lote** (`mutacao`): `sort_by_key` do pick, `cycle_step`, `stack_key`, `cola_a_faixa`,
   o arm Motion do passe de baixo, e AGORA: `FramePlan.main` (`i + 1` sem o Flip ⇒ Flip duas vezes/nenhuma),
   `adormece_palcos` (vizinho), `pass::so_estes` (ordem), o filtro `fora` dos palcos no fim.
8. **Testes de placa por correr** (`PH2D_GPU=1 … --profile smoke -- --ignored`, por nome): `ph2d-app-flip`
   `pass_luz_tests` (assinatura de `pass::render` mudou), `ph2d-app-motion` `faixa`/`palcos` ignorados, e os 4
   gates da shell da placa/palcos/miniatura.
9. **Smoke do dono** (passos numerados, a cena do item 6 nas duas ordens) → só por ordem dele, fecho (§1.5.9):
   gate batched sobre `422fec2027..HEAD`, fmt+clippy das crates tocadas (`PH2D_SUITE=fecho`; tocadas:
   ph2d-render, ph2d-app-flip, ph2d-app-motion, ph2d-app-macros, ph2d-app-pixel, ph2d-host-desktop, …),
   **ph2d-app-macros inteira** (`motivo: deps`), `agent-loop-profile.sh`, `rm -rf target/*/incremental`, handoff de
   INTEGRAÇÃO das ondas 18, 18b, 18c e 19.

## Medido / riscos

- Tecto da shell (`the_shell_only_shrinks`): 188 704 contra 189 041 depois da E4 (+~30 com os restos) — folga ~300.
- `ph2d-app-flip` 28 542 / 28 960.
- Byte-idêntico: cenas sem intercalação continuam no caminho de sempre; o que muda de propósito: Flip escondido
  pelo olho some; vários objectos Flip seguem a Hierarchy (antes a ordem do `FlipDoc`); faixas de cena sem
  sprites deixam de se desenhar duas vezes.
- Uma faixa Motion partida repinta a camada uma vez por segmento por faixa; com uma faixa só, os mesmos bytes.
- O Pixel nas faixas recebe a Luz 2D (antes, no chrome, não recebia).
