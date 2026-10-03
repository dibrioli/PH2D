# HANDOFF de CONTINUAÇÃO — `line/sculpt3d` · etapa 4, CAMADAS na peça: a W3 fechada (2026-10-03)

> **Para quem é:** o agente que assume a linha para a onda seguinte — pelo bloco do
> [MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).
> **SUPERSEDE** o [de W2](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_W2_2026-10-03.md). Não é handoff de
> integração: o de integração em vigor é o [A_INCLINACAO](HANDOFF_INTEGRACAO_line_sculpt3d_A_INCLINACAO_2026-10-02.md);
> o da etapa 4 escreve-se no fecho da linha.

## §0. Estado

| | |
|---|---|
| ramo · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d` · merge-base `1ad60a1ce` (= `main` a 03/10) |
| commits da W3 | `c7500a6c2` (o código) · `8a7458f0e` (doc 27) · `0893e81b9` (as 2 mutações sobreviventes + arnês) · `233e8b77d` (doc 30 §12) · este handoff |
| plano | [`docs/3D/30`](../30_plano_camadas_e_efeitos_na_peca.md) — **W3 = §12** (desenho, portas, premissas derrubadas, medição, o que fica) |
| `SCULPT_DOC_VERSION` · `PROJECT_SCHEMA` | `6` (intocado na W3) · intocado |
| contratos §6 | intocados: nenhuma variante de `PanelEvent`/`Tool` — o painel fala pelo `PanelEvent` de sempre |
| fora da família (W3) | `ph2d-tool-painter` (`tool/layer_edit.rs`, `tool/piece_layers.rs`; curvas e arrasto de linha viraram funções puras partilhadas; `seed_user_adjustment`; `PieceLayerOp` público) · `ph2d-panel-painter-layers` (`peca.rs`; barra, linha de máscara, linha de relevo e menu de ajustes em modo peça) · `ph2d-editor-core` (`DropdownOption::disabled`, campo novo `false` em todo chamador) · `ph2d-app-painter` (a ponte publica `panel_layers`/`panel_selection`/modo peça/recusa) · `ph2d-i18n` (chaves novas abaixo) |
| a shell | não tocada |

## §1. O que a W3 deixou (detalhe no doc 30 §12)

- Com o Painter na peça, o painel de Layers mostra a **pilha da peça**: a escultura espelha-a no
  Painter a cada quadro (`sync_piece_layers`), cada gesto vira um `PieceLayerOp`, e
  `painter_na_malha::camadas::camadas_do_painel` aplica-os pela porta da `PilhaDaPeca` (cada um com
  `StrokeUndo::Camadas`; um arrasto = um passo), recompõe a peça UMA vez e devolve o espelho.
- A porta perdeu os `#[cfg(test)]` das operações; ganhou `troca_metadado` e `troca_estrutura`; a
  base fica no fundo e não se apaga; tudo recusa com traço aberto.
- O que a peça não oferece fica **apagado e não registado** no painel, e as frases do fundo dizem
  porquê (+ a última recusa, a vermelho). O pen-down do Painter com a activa ajuste/máscara RECUSA.
- Tecto de LOC (700/ficheiro), que a W1/W2 tinham passado: `doc.rs` 762 → 486 (formatos congelados
  para `doc_migracao.rs`), `tinta_da_peca.rs` 753 → 698 (a tabela do `NIVEL_MAX` foi para o doc 27
  §16.10; `Origem`/`Desfecho` para `tinta_da_peca_pilha.rs`), `painter_bridge.rs` 699.

## §2. Verde (todos sobre o HEAD da W3)

| corrida | resultado |
|---|---|
| `nextest-impacted` (BASE = merge-base) | **19 397 / 19 397** |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` · clippy `--workspace --all-targets -D warnings` · `machete` · `check-standalone-optional` · `check-workflow-packages` | limpos |
| produto com placa (`--ignored tinta_no_produto`) | **45 / 45** (os 42 de antes + `painel::` 2 + a sonda) |
| gates novos | `piece_layers_tests` 4 · `pilha_da_peca_porta_tests` 4 · `seam_peca` 2 · `tinta_no_produto_tests::painel` 2 (placa) |
| mutação W3 (`docs/3D/ferramentas/muta_a_pilha_da_peca.sh`, agora 43 âncoras) | **20/20** (P1–P19 + a M3 re-ancorada); 2 sobreviveram na 1.ª corrida e foram curadas no `0893e81b9` (o gate do arrasto fazia um pedido só; um filtro que nada decidia saiu) |
| pré-voo dos 18 `muta_*.sh` | limpos (P10 do `muta_o_plano_no_ficheiro.sh` re-apontada para `doc_migracao.rs`) |

## §3. Medido (doc 30 §12, `load 3,3`, perfil `smoke`)

Um passo do arrasto de opacidade, ponta a ponta: `8x` `2,4 ms` · `16x` `2,5` · `32x` `7,5` ·
**`64x` `36,4 ms` + `2,85 ms` a subir o plano** (`~39 ms`, dois quadros e meio). ⇒ **a W1b (compor
na placa) é a onda seguinte**, com o alvo do §7: o passo a `64x` abaixo de `4 ms`, paridade ao
bit-de-sRGB8.

## §4. O que a onda seguinte tem de fazer

1. **W1b** — as camadas dobradas sobem como texturas `1024×H`, o `ph2d-render::layer_compositor`
   compõe-nas, o `tinta.wgsl` lê o composto pelo índice; a CPU fica a referência ao bit. A porta a
   trocar é `tinta_da_peca::pilha::recompoe` (e a `compoe_amostras` do traço). Régua: as duas sondas
   do §3.
2. Depois, pela ordem do doc 30 §7: W4 (relevo por camada — a linha de relevo do painel volta na
   peça), W5, W6 (os ajustes apagados no menu acendem), W7.

## §5. Armadilhas pagas nesta onda

- O guarda do `ph2d-run` recusa qualquer comando cujo TEXTO contenha `cargo test` sem a porta —
  inclusive um heredoc `python3` que escreve um `.sh`: o script vai num ficheiro do scratchpad.
- `MockPanelHost::drag_at` faz Down/Move/Up num quadro: é UM `SetValue` (o Down repete o valor). Um
  gate de «um arrasto = um desfazer» tem de arrastar em DOIS quadros.
- O tecto de 700 linhas por ficheiro (`architecture_workspace_file_loc_cap`) só corre no
  `ph2d-editor-core`: a W1/W2 passaram-no sem o ver. Corra-o cedo quando um ficheiro crescer.
- `tinta_da_peca.rs` tem dezenas de âncoras de mutação e `include_str!` de censos: mover CÓDIGO dele
  parte gates; corta-se prosa (o porquê histórico vai para o doc).

## §6. Smoke e binário

Binário compilado nesta worktree no HEAD da W3, depois de `rm -rf target/*/incremental` (`24 G` +
`5,7 G`); a 1.ª build foi de `37,4 s`:

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

Smoke ao dono (cena `52`):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=52 cargo run -p ph2d-host-desktop --profile smoke
```

1. `Paint Detail` → `8x`; `IMG` → `PNTR`; no painel do Painter, o separador `Layers`: aparece
   `Layer 1` e, no fundo, a frase «Layers of the 3D piece. Not here yet: …».
2. O botão `+` (New layer): aparece `Layer 2`, escolhida. Pinte um traço vermelho na bola.
3. Arraste o slider de opacidade da `Layer 2` para a esquerda: o vermelho na bola fica mais fraco.
4. `Ctrl+Z`: o vermelho volta inteiro; `Ctrl+Z` outra vez: o traço sai; outra vez: a `Layer 2` sai.
5. `+ Adj`: `Gaussian Blur`, `Sharpen`, `Bloom`… aparecem apagados; escolha `Invert` — a bola inverte.
   Clique na linha do `Invert` e tente pintar: nada pinta, e o fundo do painel diz porquê, a vermelho.
6. Deu errado se: o `+` não cria camada, a opacidade não muda a bola, o `Ctrl+Z` não a devolve, um
   botão apagado responde ao clique, ou o traço pinta com o `Invert` escolhido.

`bash scripts/agent-loop-profile.sh` (20 sessões): paralelismo `1,15` · respostas/sessão `182` ·
`test:check` `2,3` · edições pela `Edit` `37 %` · contexto por passo `485 mil` · início `62 mil`.
