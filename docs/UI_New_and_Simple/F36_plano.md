# F36 — O que nasce sob a placa cheia, e quanto o editor ocupa nela

> Aberto pelo handoff F35 §7 (1, 2, 3). Uma família só (CLAUDE.md §0.10): **o que a placa recusou e
> não volta sozinho**. Os números deste plano são MEDIDOS (26/10, RTX 5060 Ti 16 GB, janela
> 1930×1040, `kwin --virtual`); o que ainda não está medido diz-se.

## 0 — A rodada de medição (uma só, três perguntas)

Reprodutor: `crates/ph2d-gpu/examples/ocupa_a_placa.rs <livre_MiB> <segundos>` (sem o `ollama`).
Fotos: `docs/Components/ferramentas/fotografa_cena.sh` em rajada (`FOTO_RAJADA=9 FOTO_INTERVALO=5`),
com `PH2D_GPU_ERROS_TODOS=1` (todo erro de device no log). Roteiros na pasta de rascunho da janela
(`rodada_volta.sh`, `rodada_pegada.sh`, `rodada_rotulos.sh`).

## 1 — O que segue inválido DEPOIS do fim do episódio (MEDIDO)

| corrida | livre | o que nasceu recusado | depois da volta definitiva |
|---|---|---|---|
| cena 41 (`ocupa_a_placa 700 25`) | 702 MiB | **átlas** (`ph2d-render atlas`, 514 erros), **1 textura individual**, buffers `vello.*`, tex do FX, intermédio do Vello | 0 erros no log — mas o átlas e a individual seguem INVÁLIDOS (a cena não tem sprites que os desenhem) |
| cena 1 (`ocupa_a_placa 700 25`) | 949 MiB | só buffers `vello.*` | 0 erros; o Canvas volta |

- Todas as 19 falhas de `Queue::write_texture` foram escritas em texturas JÁ inválidas (18 no átlas,
  1 na individual): nenhuma escrita numa textura sã falhou por falta de memória. ⇒ a recusa vê-se no
  NASCIMENTO da textura, e basta guardá-la aí.
- Os buffers do Vello, as texturas do FX e os alvos do tamanho da superfície renascem pela cura da F35
  (nenhum erro deles depois da volta).
- O episódio OSCILA enquanto o outro programa segura a placa: a cada `JANELA_MS` (3 s) sem recusa o
  vigia declara a volta, o `pending_resize` tenta, a placa recusa de novo. É o comportamento desenhado
  (cada volta tenta; nenhuma trava), não defeito.

### A cura (item 1)

- **Átlas** (`crates/ph2d-render/src/atlas/renasce.rs`): a textura nasce sob
  `sob_escopo_de_memoria`; recusada, o átlas guarda os bytes de cada região viva (`fonte`, só nesse
  caso — custo zero no caminho normal) e `refaz_se_recusada` refaz a textura com as MESMAS regiões no
  fim do episódio (as UVs guardadas pelos chamadores continuam certas). O `regrow_inplace` recusado
  re-insere tudo pelo funil `sobe`, logo também guarda.
- **Texturas individuais** (`IndividualTextureStore`): o mesmo, por entrada e com o MESMO id — os
  bytes de 8 ou 16 bits ficam guardados só para a entrada recusada; `replace_pixels` atualiza-os.
  A entrada vazia (`acquire_empty`, só o preview da pintura) renasce transparente.
- **Preview da pintura na placa** (`painter_gpu_preview`): no fim do episódio larga a sessão da placa
  e o slot e recompõe a tela inteira — a fonte são as camadas em memória comum.
- **Quem chama**: o `SpriteRenderer`, uma vez por quadro, quando `ph2d_gpu::memoria::geracao()` muda
  (o padrão do `FxLive`, F35); refeito o átlas, refaz o `material_bind_group` e os grupos por
  amostragem. A shell não cresce.

### A cura no app (fotos em `handoffs/img_F36/`)

| corrida (com a cura) | livre | nasceu recusado | depois da volta |
|---|---|---|---|
| cena 1 (`ocupa_a_placa 50 25`, dentro da vez: `PH2D_GPU=1 bash scripts/ph2d-run.sh bash rodada_volta.sh …`) | 282 MiB | átlas (543 erros + 3 098 desenhos com o grupo por amostragem) | **0 erros; o Canvas branco (sprite do átlas) de volta** — `cena1_volta_atlas_renascido.png` |
| cena 113 do Motion (`ocupa_a_placa 600 25`) | 749 MiB | átlas (530) | 0 erros — mas as formas do Motion não mostram o átlas à vista, logo não servem de prova visual |
| cena 41 (item 3, F35 §ABERTO 3) | 702 MiB | átlas, Vello, FX | a Nuvem volta desenhada — `cena41_volta_depois_do_G16.png` |

⚠️ Duas armadilhas da rodada, medidas: (1) o `fotografa_cena.sh` RECOMPILA antes de fotografar — com a
árvore suja a compilação comeu os 25 s da placa cheia e a foto saiu sem recusa nenhuma ⇒ compile ANTES
de encher a placa; (2) sem a vez, a foto esperou a suíte de outra linha enquanto o reprodutor já tinha
soltado a placa ⇒ a rodada inteira corre DENTRO da vez (`PH2D_GPU=1`). E o reprodutor enche em blocos
de 256 MiB: o alvo `600` parou em 795 MiB livres e o átlas coube.

### Gates e mutações (item 1)

| gate | crate | o que prova |
|---|---|---|
| G17 `an_atlas_refused_at_birth_comes_back_with_its_sprites_after_the_episode` | `ph2d-render` | átlas recusado (forçado: textura destruída) volta com os pixels da substituição, na mesma região |
| G18 `an_individual_texture_refused_at_birth_comes_back_with_the_same_id` | `ph2d-render` | 8 e 16 bits, escrita de região enquanto recusada, mesmo id e refcount |
| G19 `the_sprite_renderer_rebuilds_a_refused_atlas_and_its_material_group` | `ph2d-render` | por um DESENHO real: o 1.º quadro refaz o átlas e o grupo do material; sem mudança de geração nada |
| G20 `the_end_of_an_out_of_memory_episode_recomposes_the_gpu_preview` | `ph2d-app-painter` | fim de episódio ⇒ sessão e slot novos sem pincelada; quadro parado não refaz |

Mutações M1–M14 (subagente `mutacao`): **as 14 sangraram**, cada uma só no seu gate (M1 em G17 e G19).
A deteção REAL da recusa (`erro.is_some()` do escopo) não é alcançável por teste sem encher a placa: a
prova dela é a corrida no app (§1, tabela) — o censo das recusas no nascimento vem desse escopo.

## 2 — Quanto o editor ocupa na placa (MEDIDO)

`nvidia-smi` por processo, placa livre (sem `ollama`):

| processo | placa |
|---|---|
| dispositivo wgpu vazio (`ocupa_a_placa 999999 6`) | 198 MiB |
| editor sem cena | 831 MiB |
| editor, cena 1 (Canvas 512²) | 831 MiB |
| editor, cena 41 (Nuvem com FX) | 1 343 MiB |

Por rótulo (diagnóstico novo `PH2D_GPU_PEGADA=<s>[,<s>…]`, `crates/ph2d-gpu/src/pegada.rs`: o
relatório do `gpu-allocator`, cada alocação leva o rótulo do recurso), editor sem cena aos 12 s —
**alocado 570,1 MiB · reservado 789,3 MiB**:

| rótulo | MiB |
|---|---|
| `ph2d-render atlas` (8192² com mips, para 16 tiles de demo) | **341,3** |
| `vello.*` (lines 48 · segments 48 · ptcl 32 · seg_counts 16 · tile 16 · resto ~6) | ~166 |
| `game RT` + `motion-fx RT` (Rgba16Float) | 30,3 |
| intermédio do Vello + `game_rt_ldr` + `world RT` | 22,7 |
| `motion-fx mip` (6) | 5,1 |
| o resto (≈ 40 rótulos) | < 5 |

Cenas 1 e 41, mesma rodada, aos 12 s:

| cena | alocado | reservado | átlas | `vello.*` | FX (`fx_stack tex` + individual + intermédios) |
|---|---|---|---|---|---|
| sem cena | 570,1 | 789,3 | 341,3 | ~166 (1 renderizador) | — |
| 1 (Canvas) | 570,3 | 789,3 | 341,3 | ~166 (1) | — |
| 41 (Nuvem com FX) | 1 022,7 | 1 365,3 | 341,3 | **~500 (3 renderizadores)** | ~102 |


### Leitura

- **O átlas é 60 % do editor vazio**, e é custo FIXO: `ATLAS_DEFAULT_SIZE_PX = 8192` é pré-alocado
  para que nenhuma importação 4K tenha de crescer o átlas. Um átlas do tamanho do conteúdo (2048² com
  mips ≈ 21 MiB) devolveria ~320 MiB — o editor vazio passaria de 831 para ~510 MiB por processo.
- ⚠️ **Pré-condição medida no código**: três consumidores do arranque (`shells/desktop/src/init.rs`:
  Motion, partículas, bandos) GUARDAM a UV normalizada do tile branco (`motion_default_uv`). Hoje isso
  é seguro só porque um átlas de 8192 nunca cresce; um átlas que cresce (o `regrow_inplace` re-empacota
  e a UV normalizada divide pelo lado) deixá-los-ia a amostrar o texel errado. O empacotador
  (`rect_packer 0.2.1`) não sabe crescer no lugar.
- **O Vello custa ~166 MiB por RENDERIZADOR, com a cena vazia**: são os alocadores fixos por omissão
  do Vello 0.10 (`lines` 48 · `segments` 48 · `ptcl` 32 · `seg_counts` 16 · `tile` 16 MiB), que ficam no
  `ResourcePool` de cada `vello::Renderer` depois do 1.º desenho. Há pelo menos 8 donos de `VelloPass`
  próprio (o principal da shell, o `FxLive`, o filtro e os bakes do Motion, a Luz 2D, o Quadro); a cena
  41 acorda três ⇒ ~500 MiB.

### Conclusão do item 2 (com números)

No caso do dono (o `ollama` com 13,7 GB, **946 MiB livres**):

| cena | precisa (por processo) | cabe? | com átlas sob demanda (−~320) | + um só Vello (−~332) |
|---|---|---|---|---|
| sem cena / Canvas | 831 MiB | sim (folga 115) | ~510 (folga ~435) | ~510 |
| 41 (Nuvem com FX) | 1 343 MiB | **não (faltam ~397)** | ~1 020 (faltam ~77) | **~690 (folga ~255)** |

- Um «modo leve» que desenhe MENOS não é a alavanca: o peso medido não é desenho, são dois custos
  FIXOS — o átlas pré-alocado e um reservatório do Vello por dono. Tirá-los não muda um pixel e vale
  sempre, não só com a placa cheia.
- **(a) Átlas do tamanho do conteúdo**: nascer em 2048² (≈ 21 MiB com mips) e crescer pelo
  `regrow_inplace` que já existe. Pré-condição: a UV do tile branco é GUARDADA em quatro sítios
  (`shells/desktop/src/init.rs`: `default_uv_rect` do Motion, `p.uv` das partículas, `b.uv` dos
  bandos; `ph2d-app-motion/src/macro_track/inicial.rs`) — um átlas que cresce divide a UV normalizada
  pelo lado novo e eles passariam a amostrar o texel errado (o defeito «vermelho→grená» do
  `init_subsystems.rs`). As sprites leem a UV a cada quadro (`sim_extract_emit.rs`) e não têm o
  problema.
- **(b) Um só `vello::Renderer` (um reservatório) por placa**, partilhado pelos `VelloPass`:
  refatora a cura da F35 (`VelloPass::refaz` reaplica os `registos` de TODOS os donos) e toca oito
  donos de quatro famílias.
- Ambas são uma onda própria (F37), não um remendo no fim desta: (b) atravessa famílias de outras
  linhas e (a) exige mover o tile branco para uma leitura por quadro.


## 3 — Smoke do dono 27/10: o ARRANQUE caía com a placa cheia (curado)

O 1.º smoke caiu antes de abrir: `GpuContext::new: DeviceRequest("RequestDeviceError { inner:
Core(Device(OutOfMemory)) }")` em `shells/desktop/src/init.rs:110` — o pedido do DISPOSITIVO é a
primeira coisa que a placa pode recusar, e só era feito uma vez.

- Cura: `GpuContext::new_esperando_memoria` (o arranque do editor; a shell troca uma chamada, mesma
  contagem de linhas) — recusa por falta de memória ⇒ aviso no terminal, episódio anotado
  (`memoria::anota_falta_de_memoria`: o título e o aviso de volta da F35 aparecem ao abrir) e nova
  tentativa a cada `PAUSA_NO_ARRANQUE` (1 s) até `ESPERA_MAXIMA_NO_ARRANQUE` (600 s; o `ollama` segura
  um modelo 5 min por omissão). Outro erro falha na hora. `GpuContext::new` (testes, ferramentas)
  não muda.
- Gate G21 `the_editor_waits_for_the_graphics_card_instead_of_crashing` (CPU, `ph2d-gpu`): mutações
  sem reconhecer a falta, sem nova tentativa e sem teto — as 3 sangraram.
- No app (`ocupa_a_placa 0 20`, **189 MiB livres**): o editor esperou ~14 s (12 recusas), abriu quando
  a placa libertou, 0 erros de recurso — `handoffs/img_F36/arranque_com_189MiB_livres_abre.png`.
