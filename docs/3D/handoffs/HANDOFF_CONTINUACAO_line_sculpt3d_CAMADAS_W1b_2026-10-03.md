# HANDOFF de CONTINUAÇÃO — `line/sculpt3d` · etapa 4, CAMADAS na peça: a W1b fechada (2026-10-03)

> **Para quem é:** o agente que assume a linha para a onda seguinte — pelo bloco do
> [MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).
> **SUPERSEDE** o [de W3](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_W3_2026-10-03.md). Não é handoff de
> integração: o de integração em vigor é o [A_INCLINACAO](HANDOFF_INTEGRACAO_line_sculpt3d_A_INCLINACAO_2026-10-02.md);
> o da etapa 4 escreve-se no fecho da linha. O mecanismo inteiro está no
> [doc 30 §13](../30_plano_camadas_e_efeitos_na_peca.md) (13 = a 1.ª tentativa e a decisão; 13.1 = o
> entregue; 13.2 = os degraus de topo).

## §0. Estado

| | |
|---|---|
| ramo · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d` · merge-base `1ad60a1ce` (= `main` a 03/10) |
| commits da W1b | `11dd22f45` (a medição, parada no critério) · `8f399be47` (o produto) · `d0f1f4d59` (os gates das leis sem dono) · `c0f4fdd32` (a mutação) · `9f08a1ec6` (a casa do tradutor) · `1c06af521` (a dobra larga) · este handoff |
| decisão do dono (03/10) | **opção 1**: a placa mostra a cor a ±1 degrau de sRGB8; gravar/exportar/doar continua a CPU exacta |
| `SCULPT_DOC_VERSION` · `PROJECT_SCHEMA` | **`6 → 7`** (o FUNDO da pilha; o v6 congelado em `doc_migracao.rs`, abre com a cor por vértice gravada) · intocado |
| contratos §6 | intocados (`Tool=12`, `PanelEvent=4`) |
| a shell (`shells/desktop/src`) | **0** linhas; só o censo `tests/it/the_sculpt_mesh_edits_are_wired.rs:819` segue a assinatura nova do `sync_mesh` |

**Foundational tocado (e porquê):**

| crate | o quê |
|---|---|
| **`ph2d-layer-ops` (NOVA, folha sem deps)** | `LayerOp` · `LayerMask`, saídos de `ph2d-render/src/layer_compositor/ops.rs` (que os reexporta: os caminhos `ph2d_render::layer_compositor::LayerOp` continuam) |
| `ph2d-tool-painter` | `compositor/gpu_ops.rs` = o `flatten_for_gpu` (veio de `ph2d-app-painter/src/painter_gpu_flatten.rs`, com os testes) · `pub use flatten_for_gpu` na raiz · dep `ph2d-layer-ops` |
| `ph2d-render` | dep `ph2d-layer-ops`; `ops.rs` perdeu as duas definições |
| `ph2d-app-painter` | o módulo `painter_gpu_flatten` **SAIU** (3 chamadores apontam para `ph2d_tool_painter::flatten_for_gpu`) |
| `ph2d-mesh-render` | `tinta_achata.rs` + `shaders/tinta_achata.wgsl` (`AchataDaTinta`, `MeshRenderer::achata_tinta_at`, `le_tinta_at` para os gates, `grade`) · o buffer `amostras` ganhou `COPY_SRC` |

⚠️ **Porquê esta casa** (o gate `architecture_no_dependency_climbs_a_layer` reprovou a 1.ª): de uma
FERRAMENTA (`ph2d-tool-painter`, dona da `LayerStack`) só dependem painéis, famílias, a composição e os
registos, e uma família não depende de outra ⇒ a tradução pilha → operações só pode morar NA
ferramenta, e ela não pode depender do motor (`ph2d-render`) ⇒ o vocabulário desceu a uma folha.

## §1. O que a W1b deixou

- **A recomposição do painel, do balde e dos desfazeres deles é da PLACA.** `recompoe` compõe na CPU
  só o prefixo dos vértices (`PilhaDaPeca::por_vertice`, ao bit) e marca `compor_na_placa`; o
  `sync_mesh` (agora `fn sync_mesh(&mut self, gpu: &GpuContext)`) compõe a pilha no compositor do
  Painter (`CompostosDaCena`, um `LayerCompositor` por peça) e achata-a no buffer do plano do slot.
  O `tinta.wgsl` NÃO mudou.
- **O plano da CPU pode ficar ATRASADO** (`PilhaDaPeca::atrasada`, sessão). Quem lê a peça inteira na
  CPU pede `tinta_da_peca::pilha::para_ler` (o `assa` da exportação/doação); o resto dos leitores só
  usa o prefixo dos vértices ou o comprimento. Subida inteira com a CPU atrasada ⇒ a placa volta a
  compor. Pilha que a placa não exprime (os seis ajustes sem código) ⇒ a CPU compõe e sobe como antes.
- **O FUNDO é fixado quando a pilha nasce** (`pilha_da_peca_fundo.rs`) e viaja no ficheiro (v7). Cura
  um defeito das W1–W3: lia-se da cor por vértice VIVA, que a recomposição reescreve — uma pilha
  translúcida ANDAVA a cada passo de arrasto/desfazer/balde (`50`, `37`, `27` degraus de sRGB8).
- **Cada camada tem versão e linhas sujas** (`NaPlaca`): o compositor só sobe a camada cuja versão
  mudou, e só as linhas. Arrastar a opacidade não sobe píxel nenhum.
- **A dobra alarga acima de `8 192` linhas** (`ALTURA_MAX_DA_DOBRA`): `1 024` até `64x`, `2 048` a
  `128x`, `8 192` a `256x` — a placa recusava esses dois degraus e a peça caía em silêncio na CPU.
- O desfazer do balde usa `recompoe_o_plano` (repõe a cor por vértice ele mesmo, ao bit).

## §2. Medido (perfil `smoke`, `load < 4`, a régua `painel::diag_o_preco_de_arrastar_a_opacidade_de_ponta_a_ponta`, agora `3..=8`)

| degrau | um passo do arrasto: CPU · `sync_mesh` (medianas) | W3 |
|---|---|---|
| `8x` | `0,04` · `0,06 ms` | `2,44` · `0,10` |
| `64x` | **`0,04` · `0,26 ms`** (pior `0,45`) | `36,4` · `2,85` |
| `128x` | `0,04` · `0,82 ms` (pior `2,24`) | — (CPU: `153 ms`) |
| `256x` | `0,08` · `5,7 ms` (pior `12,6`) | — (CPU: `646 ms`) |

Paridade placa/CPU na pilha rica: `23` de `188 424` bytes a um degrau (`0,012 %`); nunca mais de um.

## §3. Gates (todos verdes no HEAD)

| | |
|---|---|
| sem placa | `composto_na_placa_tests` (3: todo escritor muda a versão · a cor por vértice é o prefixo da peça · a dobra cabe e é a mais estreita) · `uma_pilha_translucida_recomposta_fica` · `um_v6_abre_com_o_fundo_da_cor_por_vertice` · `a_forma_da_pilha_gravada_e_pinada` re-pinado `83`/`3 590` (a conta fecha à mão) · `ph2d-mesh-render::a_grade_cobre_cada_amostra_uma_vez` |
| com placa | `tinta_no_produto_tests::placa::` (3: a pilha rica a um degrau, `8x..32x` + `128x`, metadado e faixa suja · o painel muda a pilha e a placa tem a peça + subida inteira atrasada · a pilha que a placa recusa) — produto inteiro `--ignored tinta_no_produto --skip diag_`: **33/33** |
| mutação | `docs/3D/ferramentas/muta_a_pilha_na_placa.sh` **19/19** (a 1.ª corrida deixou 3 vivas: o gate da deriva comparava recomposições entre si; a grade do despacho só tem 2.ª linha acima de `16,7 M`; «outra dobra, outro compositor» repetia o `ensure_array` e SAIU) · W7 do `muta_a_pilha_da_peca.sh` re-ancorada · pré-voo dos 19 `muta_*.sh` limpo |
| fecho batched (HEAD `bc53d7271` + o fmt e a M2 em `bc78dfb17`) | `nextest-impacted` **19 403/19 403** · `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` · clippy `-D warnings` · `machete` · `check-standalone-optional` · `check-workflow-packages` · `architecture_*` **102/102** · `fmt --check` · pré-voo dos 19 `muta_*.sh` — limpos. A 1.ª corrida apanhou `architecture_no_dependency_climbs_a_layer` (a 1.ª casa do tradutor) e o censo do `sync_mesh` da shell — curados em `9f08a1ec6` |

## §4. Para o integrador

- **Crate nova** `ph2d-layer-ops` (membro por glob, sem edição central). `Cargo.lock` muda.
- **Item partilhado apagado:** `ph2d_app_painter::painter_gpu_flatten` (público) — se outra linha o
  chama, troque por `ph2d_tool_painter::flatten_for_gpu`. Os tipos `LayerOp`/`LayerMask` mudaram de
  crate mas o `ph2d-render` reexporta-os: código que os nomeia pelo `ph2d_render` continua.
- **`ops.rs` do `ph2d-render`** perdeu ~95 linhas do topo: uma linha que lá acrescente uma variante
  ao `LayerOp` colide — a variante vai para `crates/ph2d-layer-ops/src/lib.rs`.
- O censo da shell `the_sculpt_mesh_edits_are_wired.rs:819` passou a procurar `sync_mesh(&mut self, gpu`.
- Premissas do briefing que a medição derrubou: (1) o `tinta.wgsl` lê o composto — não, o plano da
  placa é ESCRITO por compute e o shader fica intocado; (2) «paridade ao bit» — é ±1 (a divisão do
  WGSL), decisão do dono; (3) o tradutor numa crate própria — sobe uma camada, vive na ferramenta;
  (4) o critério media só `64x` — o chip vai a `256x`, e dois degraus caíam na CPU.

## §5. O que fica (nomeado)

- ⏳ **O tecto de camadas a `256x`:** o orçamento de cache do compositor é `1 GiB` (política do Painter
  2D, `layer_cache_budget`) e uma camada lá são `193 MB` ⇒ **5** camadas (máscaras contam); acima, a
  peça compõe na CPU (`~650 ms`). A `128x` cabem `21`. Decisão de memória de placa, quando o dono a pedir.
- O traço continua a compor as sujas na CPU (`0,04 ms`); com a base translúcida ele semeia o fundo
  inteiro em cada quadro (`fundo_semeado` na `compoe_amostras`) — cachear o fundo por amostra se pesar.
- ⭐ **Report do dono no smoke (03/10): o traço numa camada nova sai serrilhado e o da base liso.**
  MEDIDO (`camadas_painel::diag_o_traco_numa_camada_e_o_da_base`): não é defeito da pilha — o pincel
  do Painter mistura em sRGB codificado e o compositor em luz (resíduo `0,002` contra a previsão em
  luz; até `0,286` contra a base). **Decisão do dono: opção 1 — as camadas juntam-se em tons de ecrã.**
  Plano: [`docs/Painter/45`](../../Painter/45_plano_as_camadas_juntam_se_em_tons_de_ecra.md) — é a
  **onda seguinte**, numa janela nova, antes da W4.
- Depois, pela ordem do doc 30 §7: **W4** (relevo por camada), W5, W6, W7.

## §6. Fecho, smoke e binário

Binário compilado nesta worktree no HEAD, depois de `rm -rf target/*/incremental` (`29 G` + `4,2 G`);
a 1.ª build foi de `29,0 s`:

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

Smoke ao dono (cena `52`):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=52 cargo run -p ph2d-host-desktop --profile smoke
```

1. `Paint Detail` → `64x`; `IMG` → `PNTR`; no painel do Painter, o separador `Layers`; `+` → `Layer 2`;
   pinte um traço vermelho na bola.
2. Arraste o slider de opacidade da `Layer 2` de um lado para o outro: o vermelho acompanha o slider
   sem atraso (antes, a `64x`, cada passo levava dois quadros e meio).
3. Escolha a `Layer 1` e ponha a opacidade dela a meio: a bola clareia. Depois arraste a opacidade da
   `Layer 2` várias vezes: FORA do vermelho a cor da bola não pode mudar (antes escorregava a cada
   movimento).
4. `Ctrl+Z` várias vezes: cada passo volta atrás.
5. `Paint Detail` → `256x` e repita o 2: continua a acompanhar.
6. Deu errado se: o slider arrasta com atraso a `64x`, a cor fora do vermelho muda no passo 3, ou o
   `Ctrl+Z` não devolve a bola.

`bash scripts/agent-loop-profile.sh` (20 sessões): paralelismo `1,16` · respostas/sessão `184` ·
`test:check` `2,0` · edições pela `Edit` `37 %` · contexto por passo `483 mil` · início `63 mil`.
