# HANDOFF de continuação — line/UIUX, onda F34 (zoom perto numa forma com filtro), 26/10

> Linha NÃO integrada. Base: `main` c3662cd8e4 (não andou). Ponta: ver `git log -1` (F34 = os 4 commits
> depois de `f0218f1736`). **Smoke do dono 26/10: OK («Curou!»)**, depois de 3 rodadas de correção (§4).
> Formato inalterado (`PROJECT_SCHEMA` 197, tripla `(197, 13, 24)`). Shell **189040** (tecto 189041).

## 1 — Commits da F34 (`git log --oneline c3662cd8e4..HEAD`, só os da onda)

| commit | o quê |
|---|---|
| `f87bd91f1e` | **a cura**: o job do FX recorta-se à VISTA; segmentos recortados à janela; ruído ancorado à forma; WGSL `round`→`arredonda`; sonda/gates `fx_live_zoom_tests.rs`; plano `F34_plano.md` |
| `bb434ff1a1` | 1.º smoke («com o máximo de zoom ainda pisca»): textura de saída com CAPACIDADE (cresce por degraus) + `texels` em `FxImage` — o gesto não a re-registra a cada quadro |
| `2308bb7912` | 2.º smoke («no zoom máximo o objeto é cortado»): a vista do FX é a janela CHEIA (`fase_vector_fx_recook.rs`; `scene_window_wh` cortava a 55 % sob `CenterSplit`) |
| `faaf066212` | 3.º smoke («ainda piscando e sumindo», Bevel Depth 1.4): Bevel com geometria não tem margem de leitura e a sua luz não é deslocamento (`ph2d-render/src/fx_stack_plan.rs`); só a VISTA vai ao atlas do Vello (textura `saida` + `copia_o_que_mostra`) |

## 2 — Mecanismo (detalhe e tabelas: [`F34_plano.md`](../F34_plano.md))

- Defeito: o job do FX era dimensionado pela forma INTEIRA no ecrã e cortado em `MAX_FX_SIDE = 8192` a
  partir do canto superior-esquerdo; a imagem de FX toma o lugar do desenho ⇒ cobertura 0 % fora do canto
  (a forma SOME) e custo `O(texels × segmentos)` (recook ~234 ms). Valia para todo filtro.
- Cura, em ordem do pipeline: `fx_live_memo.rs` `job_for` + `cabe_no_lado` recortam o job à janela (campo
  `mostra`); `silhouette_segments_within` (`ph2d-vec-render/src/silhouette.rs`) semeia só os segmentos que
  alcançam a janela; ruído ancorado à forma (`Origem::Ponto`, não ao canto do job); WGSL em
  `fx_stack_field.rs`: `round` → `arredonda` (`floor(v+0.5)`, sem depender do modo do driver);
  `fx_live.rs`: scratch com capacidade + `copia_o_que_mostra`.
- **Recook ~234 ms → ≤ 1 ms** (régua `fx_zoom_regua_*` no plano).
- Recusado: `campo_le` (alargar as leituras por cadeia de saltos do JFA) — a mutação SOBREVIVEU, ou seja o
  gate não o distinguia; revertido, não reconstruir sem um gate que o separe.

## 3 — Superfície de colisão (DELTA) e o que um merge pode partir

- `ph2d-app-vec`: `fx_live_memo.rs`, `fx_live.rs`, `fase_vector_fx_recook.rs`, `fx_live_zoom_tests.rs`
  (novo) e `fx_live_zoom_gpu_tests.rs` (novo, `#[ignore]`, placa), cena 41 `vector_mode_filtro_zoom.rs`,
  `scenes.rs` 9..=41. `ph2d-vec-render/src/silhouette.rs` (+`silhouette_segments_within`).
  `ph2d-render`: `fx_stack_plan.rs`, `fx_stack_field.rs` (WGSL), `fx_stack_tests`.
- Contador de cena: `PH2D_OBJECT_MODE_SMOKE` vai até **41** (quem integra outra linha com cena nova
  RECONTA, não escolhe). Shell 189040 (tecto 189041), intocada além do contador.
- Um merge parte: (a) o censo `every_recook_tells_vello_the_pixels_changed` — a agulha agora é
  `stack.run_from_com_origem(`; um recook novo que chame outra porta fica fora do censo; (b) quem mexer em
  `FxImage` sem o campo `texels` volta a re-registrar a textura a cada quadro de gesto (G6/G7).

## 4 — Smoke do dono (26/10, OK)

Rodadas: «com o máximo de zoom ainda pisca» → `bb434ff1a1`; «no zoom máximo o objeto é cortado» →
`2308bb7912`; «ainda piscando e sumindo» (Bevel Depth 1.4) → `faaf066212`. Depois veio «Tela preta»:
**NÃO é defeito nosso** — o `ollama` (devstral-64k) segurava 13,7 de 16 GB de VRAM e o Vello deu OOM no
`lines_buf` do 1.º quadro; liberada a placa, o dono confirmou **«Curou!»**.

Passos (cena 41, Nuvem verde gigante com Bevel + Drop Shadow; foto [`img_F34/cena41.png`](img_F34/cena41.png)):
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_FX_PERF=1 PH2D_OBJECT_MODE_SMOKE=41 ./target/smoke/ph2d-host-desktop`
2. Selecione a Nuvem; no filtro Bevel ponha Depth 1.4.
3. Aproxime o zoom até o máximo; arraste (pan) e gire a roda.
4. Deve: a forma ficar inteira, sem piscar, sumir ou ser cortada; o FPS não cair. Deu errado: some/pisca/
   corta, ou `PH2D_FX_PERF` mostra recook de centenas de ms. Tela preta + `[gpu-erro]` no stderr = VRAM
   cheia por outro app (ABERTO 2), não esta onda.

## 5 — Gates e provas (todos verdes, cada um vermelho sob mutação)

- `fx_live_zoom_tests.rs`: G1, G2, G3, G4, G6, G10, G11. `fx_live_zoom_gpu_tests.rs` (ignored, placa):
  G5, G7, G8, G9. `ph2d-render` `fx_stack_tests`:
  `the_bevel_light_is_not_a_read_and_its_band_is_not_a_margin_with_geometry`. Censo acima.
- Mutações M1–M17 (script no scratchpad da sessão, não versionado). ⚠️ Lição: restaurar o ficheiro com o
  mtime ANTIGO fez o cargo reusar binários mutados — restaure com `os.utime` para «agora».
- Comandos: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && bash scripts/cargo-test-narrow.sh ph2d-app-vec fx_live_zoom`
  e `bash scripts/cargo-test-narrow.sh ph2d-render the_bevel_light_is_not_a_read`. Os de placa:
  `bash scripts/ph2d-run.sh` com `cargo test --profile smoke -p ph2d-app-vec fx_live_zoom_gpu -- --ignored` (confira o nome do módulo com `git grep`).
- Suíte inteira antes de integrar (corrida prévia à integração): **22331/22332**.
  O 1 vermelho: `ph2d-host-code-scenes::it a_cena_3_do_editor_conduzida::a_cena_3_conduzida_passo_a_passo`
  (CodeEditor, a F34 não o toca) — confirmar SOZINHO com o `loadavg` ao lado
  ([FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md)); na F33 já tinha dado o mesmo, verde a load 7,9.

## 6 — ABERTO, por ordem

1. **Texturas de trabalho do FX só crescem** (capacidade por degraus, nunca encolhem): depois de uma visita
   a zoom extremo podem reter até ~2 GiB de VRAM. Cura: encolher após N quadros ociosos (medir N).
2. **VRAM esgotada (outro app) ⇒ editor em branco**, só o log `[gpu-erro]` no stderr. Deve degradar/avisar
   o usuário de forma visível (banner), não calar.
3. Confirmar o vermelho da `cena_3` do CodeEditor sozinho (§5).
4. `CLAUDE.md` §5 (UIUX): o smoke lista `…6..41` (feito neste worktree); o link «Último» troca só na
   integração. `PH2D_AUTOKEY_SMOKE` segue `1..33` na linha (F32 §6: recontar na integração).
5. Da F33 §6: 2 (seleção da cena 40 não acende o painel), 3 (Zig Zag/Sketch no vale côncavo — decisão do
   dono), e da F32 §6: 2, 3, 4.
6. **Integrar e push só por ordem EXPLÍCITA do Enio**; a suíte completa é do integrador.
