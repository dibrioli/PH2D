# HANDOFF de continuação — line/UIUX, onda F35 (memória da placa: o FX devolve; placa cheia avisa e volta), 26/10

> Linha NÃO integrada. Base: `main` c3662cd8e4 (não andou). Ponta: ver `git log -1` (F35 = os commits de
> `git log --oneline 0ad3df2369..HEAD`). **Smoke do dono 26/10: OK («parece ok. siga»).**
> Formato inalterado (`PROJECT_SCHEMA` 197). Shell sob o tecto (`the_shell_only_shrinks` verde;
> `present_title.rs` ganhou a fiação do aviso, e o seu comentário longo de «redraw contínuo» foi condensado).
> Plano e tabelas: [`F35_plano.md`](../F35_plano.md).

## 1 — Commits da F35

| commit | o quê |
|---|---|
| `c80056f8d1` | item 1: parado 120 quadros, o FX solta o que sobra da memória da placa |
| `b776f4077c` | item 2: placa cheia por outro programa — aviso no título + toast, o editor volta sozinho (item 3 do plano = só diagnóstico, §5) |
| `9f549c9c40` | `cargo fmt` nos ficheiros da onda |

## 2 — Item 1: o FX devolve a memória depois do zoom extremo

- `FxLive` conta quadros parados (`QUADROS_PARADOS = 120`); ao chegar solta o que `sobra` (um lado >
  `capacidade(preciso)`): texturas de trabalho do `FxStackPass` (`work_size`/`release_work`), `saida`, e o
  `VelloPass` de scratch (pool do Vello; `pico_lote`).
- Medido (driver amdgpu): cena 41 **1337 → 1081 MiB**; pior caso (drop shadow raio 2, janela 8192²)
  **4169 → 1344 MiB**; posse do FX **2316 → 12 MiB**.
- Preço da reconstrução (mínimo de 7): `VelloPass::new` ~6–10 ms, `FxStackPass::new` ~7–11 ms.
- Recusado: a 1.ª versão usava a folga ×2 do `serve` e deixava 4× a área (171 vs 61 MiB) — apertada.
- Gates: G12 CPU `what_the_extreme_zoom_allocated_is_left_over_back_at_the_opening_view`; G13 GPU
  `after_the_extreme_zoom_a_still_scene_gives_the_memory_back`. Sondas `fx_memoria_sonda`
  (`F35_SOMBRA_GRANDE=1` = pior caso) e `fx_memoria_sonda_do_preco`.
- Mutações M1–M7 (subagente): M1, M2, M3, M5, M7 sangram no G13; M6 no G12; M4 (`||`→`&&`) SOBREVIVEU até
  entrar um caso de um lado só no G12 (aí sangrou). G6 verde sob todas.

## 3 — Item 2: placa cheia (outro app) ⇒ aviso visível e volta sozinho

- Reproduzido ao vivo (ollama 13,7 GB, 946 MiB livres) ⇒ janela cinza para sempre.
- **Mecanismo MEDIDO**: o pool do Vello 0.10 (`wgpu_engine.rs` `ResourcePool`) guarda o buffer inválido
  (OOM) e o reusa a cada quadro. Sonda `sonda_da_placa_cheia` (`ph2d-render`, opt-in `F35_PLACA_CHEIA=1
  PH2D_GPU=1`, enche a placa principal ~1 s): sem refazer, liberada a memória o pixel segue `[0,0,0,0]`;
  refazendo, vermelho e `refeitos 1`.
- Cura:
  - `ph2d_gpu::memoria` (novo): `Vigia` (`anota`/`sem_memoria`/`mudou`, uma vez por episódio; `geracao()`
    sobe no FIM do episódio; `JANELA_MS` 3000; `sob_escopo_de_memoria`). O handler de erros não capturados
    anota o OOM; `PH2D_GPU_ERROS_TODOS=1` loga todo erro de device.
  - `VelloPass::entrega` = a ÚNICA entrega ao Vello, sob escopo de OOM; recusada ⇒ Err uma vez por
    episódio, depois espera `ESPERA_SEM_MEMORIA` 500 ms e `refaz` (novo `Renderer`, reaplica as texturas
    registradas pelos mesmos ids de `ImageData` via `registos`); `vello_fundo` agora registra pelo `VelloPass`.
  - Alvos do tamanho da superfície (GameRt, Tonemap, WorldRt, intermediário do `VelloPass`, MotionFx,
    Frost) esquecem o tamanho se criados sob recusa; no fim do episódio a shell põe `pending_resize` no
    mesmo tamanho (`present_title.rs`) ⇒ recriados + rebind do compositor. `FxLive` refaz (forget +
    release) quando `geracao` muda.
  - Aviso: prefixo do título da janela `shell.gpu_memory.title` (desenhado pelo SO, sempre visível) +
    toasts `shell.gpu_memory.out/back` (`crates/ph2d-editor-core/src/aviso_da_placa.rs`; tabela em inglês
    `crates/ph2d-i18n/src/shell.rs`).
- Reprodutor sem ollama: `crates/ph2d-gpu/examples/ocupa_a_placa.rs <livre_MiB> <segundos>`.
- Corrida no app com lastro (736 MiB livres, solto após 25 s): a UI voltou sozinha; essa corrida (antes do
  G16) ainda sem a forma do FX ⇒ curado pelo rebuild por `geracao` (G16).
- Censo de recursos inválidos sob placa cheia (`PH2D_GPU_ERROS_TODOS`): buffers `vello.*`, tex de
  `fx mostrada`/`fx_stack`, `ph2d-render individual texture` (36), `ph2d-render atlas`, intermediário do
  Vello, views do mipgen, clear-on-alloc.
- Gates: G14 CPU `the_out_of_memory_warning_comes_once_per_episode_and_so_does_its_end` (`ph2d-gpu`); G15
  GPU `a_renderer_rebuilt_after_out_of_memory_still_draws_the_registered_textures` (M8 sem re-aplicar ⇒
  vermelho; M9 sem refazer ⇒ vermelho); G16 GPU `the_end_of_an_out_of_memory_episode_rebuilds_the_fx_textures`
  (M10 sem forget ⇒ vermelho). O censo `every_hand_off_to_vello_goes_through_the_keepalive` foi atualizado
  para a entrega única (entregas ≥ 1, dois chamadores de `self.entrega`).

## 4 — Superfície de colisão (DELTA) e o que um merge pode partir

- Novos/alterados: `ph2d-gpu` (`memoria`, handler de erros, exemplo `ocupa_a_placa`), `ph2d-render`
  (`VelloPass`: `entrega`/`refaz`/`registos`; alvos da superfície; sonda), `ph2d-app-vec` (`FxLive`, testes
  G12/G13/G16), `ph2d-editor-core` (`aviso_da_placa.rs`), `ph2d-i18n` (`shell.rs`), `ph2d-host-desktop`
  (`present_title.rs`). Confira com `git diff --stat c3662cd8e4..HEAD`.
- Um merge parte: (a) outro chamador do Vello que contorne `VelloPass::entrega` (fora do censo e da
  proteção de OOM); (b) quem criar textura do tamanho da superfície sem esquecer o tamanho sob recusa
  (não renasce no fim do episódio); (c) `present_title.rs`: o fio do `pending_resize` + título é fácil de
  perder num conflito.
- Contador de cena: `PH2D_OBJECT_MODE_SMOKE` segue até **41** (esta onda não criou cena).

## 5 — Item 3 (só diagnóstico) e prova de fecho

- CodeEditor cena 3, sozinha 3×: vermelho a 30,33 s (load 31,5); verde 0,49 s duas vezes (load 47). É
  DEFEITO do CodeEditor (espera da lista do `luau-lsp`), não carga nem esta linha — já em
  [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md); plano §3.
- Fecho: `fmt` (9 ficheiros); clippy `-D warnings` limpo em `ph2d-gpu`, `ph2d-render`, `ph2d-editor-core`,
  `ph2d-i18n`, `ph2d-app-vec`, `ph2d-host-desktop`; testes CPU por nome verdes; gates GPU G9, G6-gesto
  (`a_gesture_never_drops`), G5, G13, G15, G16 e `o_mundo_chega_ao_byte` verdes na pista de teste (no NVIDIA
  principal FALHAM enquanto o ollama o enche — esperado, com a nova mensagem de OOM).
  `nextest-impacted` NÃO completou (morto; cadência). Suíte inteira: do integrador.
- Comandos: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && bash scripts/cargo-test-narrow.sh ph2d-gpu the_out_of_memory_warning`
  e `bash scripts/cargo-test-narrow.sh ph2d-app-vec what_the_extreme_zoom_allocated`. Os de placa
  (`#[ignore]`): `bash scripts/ph2d-run.sh` com `cargo test --profile smoke -p <crate> <nome> -- --ignored`
  (confira o módulo com `git grep`).

## 6 — Smoke do dono (OK) — passos

Fotos em `img_F35/`: [`cena41_placa_cheia.png`](img_F35/cena41_placa_cheia.png) (cinza, ollama presente),
[`cena41_volta_antes_do_G16.png`](img_F35/cena41_volta_antes_do_G16.png) (UI de volta, nuvem faltando —
pré-G16), [`cena41_antes_da_cura.png`](img_F35/cena41_antes_da_cura.png). A foto final da recuperação no app
NÃO foi retirada: o ollama segurou a placa > 30 min.

A) Pare o ollama. 1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_FX_PERF=1 PH2D_OBJECT_MODE_SMOKE=41 ./target/smoke/ph2d-host-desktop`
2. Em outro terminal: `watch -n1 nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv`.
3. Zoom ao máximo e volte; pare o mouse: ~2 s depois a memória cai. Errado: ela não cai.

B) Com o ollama carregado, abra o editor: o título começa com «Graphics card out of memory —». 1.
`ollama stop <nome>`. 2. Em ~3 s o editor desenha sozinho e aparece o aviso «The graphics card has memory
again». Errado: janela cinza sem aviso, ou sem voltar.

## 7 — ABERTO, por ordem

1. **Caches de CONTEÚDO criados sob recusa não se refazem**: atlas de sprites (`TextureAtlas`; refazer
   exige buscar no `AssetDb`/`atlas_asset_map` + tiles de partida de `atlas_loader` +
   `rebuild_material_bind_group`), `IndividualTextureStore` (fontes em vários donos, inclusive camadas do
   Painter preenchidas na GPU), views do mipgen. Outros caches com `VelloPass` ficam cobertos pela cura do
   `VelloPass`, mas as texturas próprias deles não foram auditadas. Sinal: `ph2d_gpu::memoria::geracao()`.
2. **«Desenhar mais leve» NÃO feito**: com ~1 GB livre o conjunto de trabalho do editor não cabe. MEÇA a
   pegada de VRAM por subsistema (`nvidia-smi` por processo, sem ollama) antes de desenhar um modo leve.
3. Foto real da recuperação no app da cena 41 depois do G16.
4. `CLAUDE.md` §5 (UIUX): o link «Último» troca só na integração; `PH2D_OBJECT_MODE_SMOKE` segue até 41.
5. Herdado: F34 §6 itens 4–5; F33 §6: 2, 3; F32 §6: 2, 3, 4.
6. **Integrar e push só por ordem EXPLÍCITA do Enio**; a suíte completa é do integrador.
