> ⚠️ SUBSTITUÍDO pelo [handoff W11](HANDOFF_INTEGRACAO_line_MiroClone_W11_2026-10-10.md) — integre por ele.

# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W5 … W10 → `main` (2026-10-10)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem
> pusha. **Substitui** o [handoff W9](HANDOFF_INTEGRACAO_line_MiroClone_W9_2026-10-10.md) como o último da
> linha. W5 … W9 (smoke do dono OK em todas) não foram integradas e a W10 vai por cima, na mesma branch —
> **integra-se tudo de uma vez**; para as anteriores valem os handoffs delas (o da W9 tem o portão e a
> tabela do foundational da W9). Briefing: [continuação W10](HANDOFF_CONTINUACAO_W10_2026-10-10.md).
> Estudo: [`../07_fidelidade_excalidraw.md`](../07_fidelidade_excalidraw.md) (§2 XV7, §3 as recusas da
> W10, §5 a linha W10). Oráculo: [README §W10](../ferramentas/excalidraw_oracle/README.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| base (merge-base) | `2a925859d` — **o `main` não andou** desde a W9 ⇒ `--ff-only` se ninguém aterrar antes |
| commits da W10 | `83138a1df` (3/3 a letra) · `0747a4a16` (oráculo) · `69a40b461` (1/3 o legado do 0.18.1) · `702b8a2f5` (2/3 o realce) · `4fd466d2a` (curas do portão e da auditoria) · `202b389fc` (mutação) · `e11be3aa8` (portão 2.ª) · `156d43c0a` (foto) + este handoff — `git log --oneline d34b0b5fb..HEAD` |
| diff da W10 | `git diff --shortstat d34b0b5fb..HEAD -- crates/`: 16 ficheiros, +1 019 / −54; o resto são as entradas e saídas do oráculo (a varredura `w10_F_*` compacta: 3,1 MB de saídas + 3,3 MB de entrada) e a foto |
| smoke | **OK do dono (10/10)** — §12 |

O que a W10 trouxe — as três pontas da W9 (handoff W9 §9), escolha do dono 10/10:

1. **O nosso `.excalidraw` no Excalidraw 0.18.1 depois de MEXER uma forma.** A lei do legado `{focus,
   gap}` foi MEDIDA (estudo XV7: 34 casos, ~3 600 pontas re-ligadas) e o escritor grava o par que faz o
   0.18.1 pôr a ponta onde a escrevemos: antes saltavam até **106 px**; agora rectângulo e elipse ficam a
   **≤ 0,044 px** (o resto é o próprio 0.18.1, que re-liga as duas pontas uma de cada vez).
2. **O realce ao arrastar uma ponta** é o do Excalidraw (medido no next a meio do arrasto): o contorno
   realçado e a seta já parada onde vai ficar — **sem** o ponto no foco que o nosso desenhava.
3. **A letra do texto importado segue o botão «Rascunho ↔ Final»** (decisão do dono 10/10, «troca junto
   com o botão»): os elementos que o botão alcança passam a `Letter::Auto`, no mesmo passo de desfazer; um
   texto aberto fecha-se antes. Cena 11 ganhou a lição 5 (a própria caixa).

## 2. Foundational / partilhado tocado — e porque é aditivo

| onde | o quê | porque é aditivo |
|---|---|---|
| `ph2d-board-excalidraw` (crate da linha) | `legacy_focus.rs` (novo): a lei XV7 e o modelo `landing` (`#[cfg(test)]`, só o gate o chama); `write.rs::binding` recebe o ponto vizinho e a ponta e grava `{focus, gap}` por ela (o `mode`/`fixedPoint` do next não mudam); testes `fidelity_w10_tests.rs` | o leitor (`bind.rs`) não mudou: lê `mode`/`fixedPoint` primeiro e ignora o valor do legado (W9) — os nossos ficheiros W9 relêem-se iguais |
| `ph2d-board-edit` | `toggle_sketch`: + `look.letter = Auto` no alcançado e `commit_text` antes; `overlay::Target` **perde o campo `fixed`**; `wire.rs::target` | `Target` só tem consumidores na família do quadro (`ph2d-board-render/src/overlay.rs` e testes) — `grep -rn 'Target {' crates/` |
| `ph2d-board-render` | `overlay.rs`: o ponto no foco sai (o contorno fica) | família do quadro |
| `ph2d-app-board` | cena 11: a caixa da lição 5 (`Lessons::letter`) + gate `lesson_5_*` | família do quadro |
| `ph2d-i18n` | `board_fidelity.rs`: a chave `board.smoke.fidelity.l5` | chave nova |
| `ph2d-editor-core` | só um teste (`board_bar_pen_tests.rs`) | — |
| shell · `Cargo.toml` · `Cargo.lock` · `.typos.toml` | **nada** | — |

**Contratos congelados (§6):** nenhum — `git diff --name-only d34b0b5fb..HEAD | grep -E
'ph2d-nodegraph|ph2d-tool|ph2d-vec|ph2d-vector|ph2d-imageio|schema|project\.rs|Cargo'` → vazio, e o diff
não contém `PROJECT_SCHEMA`, `NodeOp`, `VectorOp` nem `FORMAT_VERSION` (o formato do quadro continua 9).

## 3. Números MEDIDOS

- **A lei XV7** (oráculo 0.18.1, `saidas/w10_F_*.0181.json`, `mede_w10.py`): |Δfocus| **0,00000** em todos
  os casos (0,00003 no losango redondo; foco além de ±1 aceite); o vão: rectângulo de cantos vivos =
  Chebyshev (0,0000, também rodado, alto, `sw 4`, negativo), elipse = semi-eixos + gap (0,0000, também
  «redonda», rodada, −10/−40/−47); **`gap 0` põe a ponta DENTRO** (no foco) — o escritor nunca grava menos
  de 0,01 em módulo; **rectas paralelas aos eixos**: dois cantos empatam e ele espelha (48 pontas) — o
  escritor escreve `|focus|·(1+1e-6)`; losango e rectângulo redondo: o vão dele não é uma distância
  constante (losango g=6: vértice x a 4,43, y a 5,39) — **sem lei**.
- **O nosso ficheiro** (`X10_nosso`, 13 setas, 27 pontas presas; forma direita, rodada, redonda; órbita,
  centro, ponto fixo; recta, curva dele, curva do Miro, paralelas aos eixos), depois de o 0.18.1 re-ligar:

  | | antes da W10 | W10 |
  |---|---|---|
  | rectângulo / elipse, órbita e centro | até 85,3 | **≤ 0,044** (a lei prevê o 0.18.1 a ≤ 0,044 em todas) |
  | losango (órbita) · rect. redondo (centro) | até 106,0 · — | **3,875** · **3,513** (sem lei) |
  | ponto fixo dentro, do lado de LÁ da recta (não se exprime: ele pára na entrada) | — | 68,4 · 70,1 (rect) · 50,9 · 24,7 · 9,8 (elipse) — o par cuja ponta cai mais perto |
  | ponto fixo dentro do losango · do rect. redondo | — | 4,70 · 85,6 (sem lei) |

  No **next** (o do dono) o mesmo ficheiro: rectângulo/elipse ≤ 0,05 salvo a curva do Miro (15 pontos) a
  0,278 px; o losango 1,43 (a recusa W1 do ⌊w/2⌋+1).
- **O realce** (next a meio do arrasto, `saidas/w10_R_*.{0181,next}.meio.png`): só o contorno azul e a seta
  parada a `g`; os pontos que ele mostra (cinza, azul ao encaixar) são o encaixe no MEIO DO LADO.
- **Determinismo do oráculo:** duas corridas iguais — 34/34 `F_*`, 18/18 `R_*`, 2/2 `X10`.

## 4. Portão batched (1× sobre `2a925859d..HEAD`, duas passagens)

| portão | resultado |
|---|---|
| `nextest-impacted.sh` (`BASE=2a925859d`, `PH2D_GPU=1`, `NO_FAIL_FAST=1`) | ✓ **17 567/17 567** (2.ª passagem; a 1.ª: 1 vermelho, o `xv7` com `==` exacto nas posições — 1 ULP entre perfis) |
| `cargo check --workspace --all-targets` com `CARGO_BUILD_WARNINGS=deny` | ✓ (depois de `landing` → `#[cfg(test)]`: o `-D dead-code` apanhou-o na 2.ª) |
| clippy do `ship.sh` (`--features ph2d-spike/bevy_ecs`, `--keep-going`, `-D warnings`) | ✓ 0 avisos (1.ª: `needless_range_loop` no teste; 2.ª: o `dead-code`) |
| `cargo fmt --all -- --check` | ✓ (1.ª: 5 ficheiros) |
| `ph2d-host-desktop --test it` (com `file_loc_caps`) · `ph2d-editor-core --test it` | ✓ 791 · ✓ 536 |
| `censos-da-arvore-combinada.sh` | ✓ 114/114, 12/12 censos |
| `typos` · `cargo machete` · `cargo deny --all-features check` | ✓ · ✓ · ✓ |
| LOC | ✓ o maior tocado é `ph2d-board-edit/src/lib.rs` (613); `tests.rs` (969) é isento |
| gates nomeados (`arch_safe_clamp_only`, `clamp_calls_in_ui_are_safe_or_justified`, `no_magic_numeric_*`, `no_hex_color_*`, `the_shell_only_shrinks`, `architecture_*contract_surface`, `workspace_src_files_under_loc_cap`) | ✓ dentro do nextest impactado |

Depois da 2.ª passagem só mudaram o `#[cfg(test)]` de `landing` e o texto da lição 5: re-corridos à
parte o check/clippy (verdes), `ph2d-board-excalidraw` (27), `ph2d-app-board` (23), `ph2d-i18n` (23).

## 5. Auditoria de 2 lentes — curadas com teste (`4fd466d2a`)

| achado | lente · peso | cura | o teste que o via |
|---|---|---|---|
| um ponto fixo DENTRO do lado de LÁ da recta: o 0.18.1 move-o até 161 px (pára na entrada do nível) | 1 · alta | não se exprime: `legacy` escreve o par cuja ponta (`landing`, a lei) cai mais perto — a entrada ou o ponto do foco do `gap 0`; números no §3 e no estudo §3 | `xv7_the_0181_moves_no_tip_when_it_rebinds_our_arrows` (+ 7 pontos fixos na amostra) |
| ponto fixo dentro do losango/rect. redondo recebia o vão POSITIVO (a ponta saía para fora) | 1 · média | menos a distância ao contorno | o mesmo (losango 4,70) |
| elipse funda junto ao eixo maior: o nível degenerava (semi-eixo 6e-11, 9,8 px de erro dele) | 1 · baixa | o nível com semi-eixo < 1 px recusa-se | `a_level_thinner_than_a_pixel_is_refused` |
| o gate não ligava o oráculo gravado ao escritor de HOJE (o `UNTIE` sem gate) | 1 · média | o `xv7` confronta o `{focus, gap}` corrido com o escrito | o mesmo |
| o botão com um texto ABERTO: o cursor media-se na letra velha e o 1.º desfazer levava a escrita e a troca | 2 · média-baixa | `toggle_sketch` fecha o texto antes | `the_sketch_button_closes_the_open_text_first` |
| a lição 5 pedia «nada seleccionado» e a lição 4 deixa a seta seleccionada | 2 · baixa | a lição manda o Esc; o gate conduz a lição 4 primeiro | `lesson_5_*` |

Não curado, para o DONO (§9): uma **nota** importada com a letra fixa nunca volta ao modo (o botão não
alcança notas — são papel por desenho).

## 6. Prova de mutação

17 mutações (o agente `mutacao`, os quatro controlos; cada ficheiro restaurado com `touch`, árvore limpa):
**15 sangraram**. As duas sobreviventes — o filtro do botão só pela `sketch` (uma forma JÁ à mão com a
letra fixa ficava fixa) e o `THINNEST` a 0 — foram tapadas com `the_sketch_button_reaches_a_sketched_element_with_a_fixed_letter`
e `a_level_thinner_than_a_pixel_is_refused` e **re-mutadas: sangram** (`202b389fc`). ⚠️ O `THINNEST` (1 px)
é uma guarda: o 0.18.1 só está medido a falhar com 6e-11.

## 7. Fixtures, instrumentos e o que vive versionado

- `corre_fidelidade.mjs`: `--prefixo`, o passo `captura` (PNG + appState antes de largar), `guardar:
  false`, `compacto`. `entradas/gera_w10_casos.py` → `w10_focus_casos.json` (34 `F_*`) e
  `w10_realce_casos.json` (9 `R_*`); `mede_w10.py` tira as tabelas.
- **As três entradas NOSSAS regravadas e re-corridas** (o escritor mudou): W8 `w8_ph2d_quadro.*` (o
  `w8_sondas.json` fica o da W8 — só sorteados mudavam), W9 `w9_X_nosso.*`, W10 `w10_X10_nosso.*`. Como se
  regrava: README §W10.
- `docs/MiroClone/ferramentas/cena11_w10_foto.png` (a cena 11 pelo `fotografa_cena.sh`, tela virtual; a 1.ª
  foto apanhou a lição 5 a dizer «top bar» — o botão é o da linha tremida na barra da ESQUERDA).

## 8. Smoke (o comando exacto)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=11 ./target/smoke/ph2d-host-desktop
```

A cena 11 continua a da W9 (lições 1–4) mais a caixa da **lição 5** à direita, a meio. Para o dono:
(1) o comando acima; (2) `Esc` e o botão da linha tremida na barra da esquerda («Rascunho ↔ Final»);
(3) o texto da caixa 5 passa à letra à mão, como as formas e as setas; outra vez, volta à letra comum;
`Ctrl+Z` desfaz num passo; (4) deu errado se a caixa 5 continua na letra comum depois do botão. O realce:
(2) tecla `A`, arrastar de «Start here» para perto da borda de «Drop inside» — (3) a caixa acende inteira e
a seta já aparece a parar antes dela, sem ponto nenhum no contorno; (4) errado se aparecer um ponto
azul no contorno. A ponta 1 (0.18.1) não tem passo de dono (ele usa o excalidraw.com).

## 9. O que fica ABERTO

- **O foco do gesto na faixa por fora** (medido W10, `w10_R_ponta_banda_diag.next`): o next guarda o
  próprio CURSOR como foco; o nosso `anchor_for` projecta-o no contorno — numa aproximação diagonal a 3 px da
  borda a ponta difere ~4,3 px. Fica a lei da W9 (smoke OK do dono); registado no estudo §3.
- **O encaixe no meio do lado do next** (os pontos cinza/azul a meio do arrasto, `isMidpointSnappingEnabled`):
  não o temos; o nosso `anchor_for` já cola ao meio do lado dentro da faixa, sem o mostrar.
- **No 0.18.1 (baixo para o dono):** o losango e o rectângulo redondo sem lei do vão (≤ 3,9 px em órbita;
  4,7 e 85,6 num ponto fixo dentro); o ponto fixo do lado de lá não se exprime (§3).
- **Pergunta de produto para o dono:** uma NOTA importada com a letra do ficheiro fica com ela (o botão não
  alcança notas).
- No next, a curva do Miro re-liga a 0,278 px (ele põe a ponta na recta do ponto vizinho ao foco; a nossa
  sai da porta) — sub-pixel, não curado.
- Da tabela do estudo §4, inalterados (de onda própria): `line` fechada, `link`, `labelPosition`,
  `customData` de terceiros, imagem/embeddable, `zigzag` desenhado às riscas.

## 10. Para o integrador

- Ordem: a linha é uma só (W5 … W10 na mesma branch); `main` = merge-base ⇒ `--ff-only`.
- O formato do QUADRO continua 9 (a W10 não o tocou); nada soma com o `PROJECT_SCHEMA`.
- `Cargo.lock` · `.typos.toml`: a W10 não os tocou (os da W8/W9 valem — handoff W9 §10).
- CLAUDE.md §5.1 (Quadro) aponta este handoff.

## 11. A máquina

Nenhum processo desta linha (`pgrep -af 'ph2d|cargo|rustc' | grep line-MiroClone` vazio). `rm -rf
target/*/incremental`: **31 GB** recuperados (debug 27 G, smoke 4,1 G); disco 47 %.

## 11-bis. O binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, duas vezes; a 2.ª:

```text
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

## 12. Smoke do dono — OK (10/10)

*«smoke ok.»* — a lição 5 (a letra segue o botão) e o realce sem o ponto no foco. A pergunta das notas
importadas com a letra fixa (§9) ficou sem resposta.
