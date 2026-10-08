# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W5 + W5b «Guias e alinhamento de objectos rodados» → `main` (2026-10-08)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra
> nem pusha. **Substitui** o [handoff W5](HANDOFF_INTEGRACAO_line_MiroClone_W5_2026-10-07.md) como o
> último da linha: a W5 (aprovada no smoke do dono, §12 dele) não foi integrada e a W5b continua por
> cima, na mesma branch — integra-se tudo de uma vez. Briefing:
> [continuação W5b](HANDOFF_CONTINUACAO_W5b_2026-10-08.md). O estudo, as leis com fonte e número e as
> recusas: [`../03_guias_e_alinhamento.md`](../03_guias_e_alinhamento.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| HEAD | `git rev-parse line/MiroClone` (o último commit é o deste handoff) |
| base (merge-base) | `2a925859d` — **o `main` não andou** (`git rev-parse main` = `2a925859d` no fecho) ⇒ `--ff-only` se ninguém aterrar antes |
| commits | 25 antes deste handoff (`git log --oneline 2a925859d..line/MiroClone`): 18 da W5 + o briefing + 7 da W5b (`e5a98d979`..`8f07d7719` e o do `.typos.toml`) |
| diff da W5b | `d95880f6b..HEAD`: 83 ficheiros, +74 941 / −107 — o grosso são as fixtures JSON dos dois oráculos (`ferramentas/{excalidraw,tldraw}_oracle/saidas/`); código e testes: 17 ficheiros, +2 116 / −107 |
| smoke | **por fazer pelo dono** (cena 7, §9) |

O que a W5b trouxe ao Quadro: guias nos **eixos próprios** de um objecto rodado (desenhadas inclinadas)
e dentro de um **frame rodado**, o **encaixe do ângulo** a rodar (no de outro objecto, mod 90°),
**frames** como alvo e a alinhar a caixa DELES, **espaçamento igual**, alinhamento pelos **cantos
rodados** (o que os dois oráculos fazem), a casa do PONTEIRO (um filho só alinha com os irmãos e o
frame) e a tolerância **8 px de ecrã** (medida igual no Excalidraw e no tldraw).

## 2. Foundational / partilhado tocado — e porque é aditivo

| onde | o quê | aditivo? |
|---|---|---|
| `ph2d-board-edit` | o encaixe sai de `gesture.rs` (698 → 675 linhas) para `guides.rs` (novo: `Snapper`, famílias de eixos, casa, ângulo) e `snap.rs` (`Stops` com os cantos de dentro, máscara de eixos, espaçamento igual); `resize::turned`; `Overlay.guides` documentado | sim — da família; o `Overlay` não muda de forma |
| `ph2d-editor-core` | `board_view::metrics`: `snap` `Spacing::Sm` (6) → `Spacing::Md` (8) | sim — só o quadro lê estas medidas |
| `ph2d-app-board` | cena 7 (`smoke_guides.rs` + gate), `ROUTERS.max_level` 6 → 7, `smoke_organize::frame` passa a `pub(super)`; dev-dependência `ph2d-text` | sim |
| `ph2d-i18n` | `board.rs` +15 chaves `board.smoke.guide.*` | sim |
| `ph2d-board-render` | só testes: `frames_paint` (+1, a guia rodada pela placa), régua `measure_snap_cost` (`#[ignore]`) | sim |
| `.typos.toml` | exclui `docs/MiroClone/ferramentas/tldraw_oracle/saidas/**` (ids aleatórios do tldraw, o idioma das do Excalidraw) | sim |

**`ph2d-board-model` intocado pela W5b** (`git diff d95880f6b..HEAD -- crates/ph2d-board-model` vazio):
`FORMAT_VERSION` continua **5** (o da W5), nenhum campo novo no documento. **`PROJECT_SCHEMA` 187 = main.**
A shell (`shells/desktop`) não é tocada pela W5b. Nenhuma mudança fora do quadro.

## 3. Símbolos que podem colidir — `bash scripts/collision-surface.sh` (08/10)

```text
SUPERFÍCIE DE COLISÃO — line/MiroClone contra main
  merge-base 2a925859d   ·   25 commit(s)   ·   241 arquivo(s)
▸ SCHEMAS            PROJECT_SCHEMA 187 (base 187) · tripla (187, 13, 22) · VEC_SCENE 22 · FLIP 13 · DOC_VERSION 18
▸ REGISTRO           ph2d-ecs 105 · ph2d-render 106 · ph2d-script 106 (= base)
▸ CONTRATO CONGELADO node.rs intocado · tool.rs intocado
▸ ADR                esta linha não cria ADR
▸ Cargo.lock         nenhum '+name' novo (a aresta ph2d-app-board → ph2d-text é interna)
▸ MARCADORES         nenhum
▸ TETOS DE LOC       nenhum arquivo da linha passa do teto
```

O que pode chocar num rebase além do da W5 (handoff W5 §3): `board_view.rs` (1 linha, `snap:`),
`ph2d-i18n/src/board.rs` (chaves só desta família, no fim do `match`), `smoke.rs` (tabela e `match`).

## 4. Contratos congelados

**Nenhum encostado.** `node.rs` e `tool.rs` intocados (`collision-surface.sh`); `git diff 2a925859d..HEAD
-- crates/ph2d-vector-doc crates/ph2d-nodegraph crates/ph2d-editor-core/src/tool.rs` vazio.

## 5. Portão de fecho (batched, 1× sobre `2a925859d..HEAD`, árvore parada em `8f07d7719`)

| portão | resultado |
|---|---|
| `PH2D_GPU=1 BASE=2a925859d NO_FAIL_FAST=1 bash scripts/nextest-impacted.sh` | **16 997 / 16 997** (9 813 fora do impactado), 126 s |
| clippy do `ship.sh` (`--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`) | saída 0 |
| `cargo fmt --all -- --check` | limpo |
| `ph2d-editor-core --test it` inteiro (`arch_safe_clamp_only`, `architecture_workspace_file_loc_cap`, censos HR-15) | 536 / 536 (12 ignorados de nascença) |
| shell `ph2d-host-desktop --test it` (`file_loc_caps`, `fn_loc_caps`, `arch*`, `the_shell_only_shrinks`) | 791 / 791 (7 ignorados) |
| `bash scripts/censos-da-arvore-combinada.sh` | 114 / 114, os 12 censos |
| `ph2d-i18n` · `cargo machete` · `typos` (o do `ship.sh`, projecto inteiro) | verdes · nada morto · 1.ª passagem: 2 ids do tldraw nas fixtures → exclusão no `.typos.toml`, re-corrida limpa |
| `ph2d-board-edit` · `ph2d-board-render` · `ph2d-app-board` | 121 lib · 12 lib + 6 it (7 réguas ignoradas) · 1 (o gate da cena) |
| GPU `frames_paint` (pixels) | 6 / 6, sem «sem adaptador» |
| **auditoria (2 lentes)** | **leis** — 9 achados, todos curados com teste (`90ab1a02e`): o espaçamento igual corria sobre um PONTO (criar/redimensionar mostravam vãos de uma caixa que não existe); a pega de um lado encaixava os dois eixos (e num rodado ficava aquém da guia); rodar VÁRIOS encaixava a volta e não o ângulo deles; `Alt` não deixava os originais como alvo; o encaixe destravava o eixo do `Shift`; o arco do ângulo sem chão (a r = 10 a janela chegava a 45°); a casa fixada no carregar (o largar decide pelo ponteiro); `Shift`/nota a criar e redimensionar em proporção desenhavam guias que a forma não segue; o par do vão sem partilhar a fila. **costura** — a cena 7 ensinava o contrário (o 1.º encaixe era o centro, guias direitas, o canto de um vizinho; «I stick to 30°, 45°» encaixava também nos 20° do frame rodado): refeita com uma lição por frame a 40 e o gate `every_lesson_teaches_its_own_alignment` (a zoom 1 e ao do enquadrar) — que mediu mais uma lei: ao MESMO ângulo, só os eixos próprios (o filho saltava 0,97 para um alinhamento de ecrã); os testes usavam 6 px e o produto 8 |
| **prova de mutação** (20 leis: eixos próprios, cantos de dentro, casa, seguir o ponteiro, `Alt`, caixa do frame, frame alvo, ângulo, chão do arco, ângulo comum, espaçamento igual na lib e na cena, ponto sem vãos, par na fila, `Shift`, eixo da pega, quadrado, só-próprios ao mesmo ângulo, redimensionar nos eixos próprios, pixels da guia rodada) | **20 / 20** — 18 à 1.ª; o quadrado (o alvo estava longe do ponteiro) e o espaçamento na cena (o arrasto acabava no sítio exacto, sem precisar do encaixe) eram testes que não chegavam lá: tapados (`8f07d7719`) e re-mutados — sangram |

## 6. Números MEDIDOS

**Oráculos corridos** (§0.9, por script, entradas nossas; duas corridas byte a byte iguais):

| | Excalidraw 0.18.1 (MIT) | tldraw 5.5.2 (licença tldraw: uso de desenvolvimento — triada pelo `LICENSE.md` instalado) |
|---|---|---|
| corrida | `ferramentas/excalidraw_oracle/corre_guias.mjs`, 15 casos, 142 sondas, **30/30** iguais | `ferramentas/tldraw_oracle/corre_guias.mjs`, 8 casos, **16/16** iguais |
| rodado alinha por | 4 cantos rodados + centro (canto de dentro a x 300 exacto, caixa a 274,1 · 250 · 239,8 · 229,3) | 4 cantos rodados + centro (8/8) |
| guias oblíquas | 0 em 142 | 0 |
| encaixe do ângulo | nenhum; `Shift` 15° | nenhum (arredonda ao grau ao largar); `Shift` 15° |
| tolerância | **8 px** de ecrã, zoom 1 e 2 | **8 px** de ecrã, zoom 1 e 2 |
| filho de frame | alinha também com o de fora | **nunca** com o de fora (adoptado) |

**Régua do encaixe** (`ph2d-board-render/tests/it/measure_snap_cost.rs`, `--release`, mínimo de 7 rodadas
intercaladas, `load` 9,0 — tecto): a 100 mil à vista, mexer **1,61 ms** (direito, med 2,1) / **1,94 ms**
(rodado, med 2,4) por movimento; carregar 11,7 ms uma vez por gesto (o `Ctrl`, que não encaixa, paga o
mesmo). A 10 mil: 0,16 / 0,19 ms. ⇒ sem índice espacial nem tecto de alvos (§0.0).

## 7. O que só o `ship.sh` / o CI pegaria

O `typos` (corrido aqui, curado). Nada de `#[cfg(target_os`. Nenhuma feature nova.

## 8. Ordem e dependências

Independente das outras linhas; o `main` não andou ⇒ fast-forward. Se outra linha aterrar antes e tocar
em `ph2d-i18n/src/board.rs` ou `board_view.rs`, o conflito é por linhas vizinhas (aditivo).

## 9. Smoke (binário `smoke` já compilado na worktree — §10-bis)

Ao dono, em passos:

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=7 ./target/smoke/ph2d-host-desktop`
2. Abre o quadro com cinco caixas brancas (lições): «Tilted shapes», «Rotate», «Tilted frame»,
   «Equal spacing», «Frame A»/«Frame B». Em cada uma, a forma que diz *«drag me»*, *«slide me»* ou
   *«rotate me»* é a que se mexe.
3. O que tem de acontecer:
   - «Tilted shapes»: arrastar a caixa amarela de baixo devagar na direcção da inclinação, para cima
     e para a esquerda — a borda esquerda dela cola na linha da de cima e aparece uma linha vermelha
     **inclinada**;
   - «Rotate»: pegar na bolinha de rodar da caixa rosa do meio e rodar — ela «prende» ao chegar a 30°,
     a 45° e a direito (aparecem duas linhas no ângulo); com `Shift` roda de 15 em 15°; com `Ctrl` roda
     livre;
   - «Tilted frame»: a caixa azul da direita, arrastada para a esquerda, prende quando os vãos ficam
     iguais; a de baixo, arrastada para a direita, prende com a esquerda alinhada à de cima — as linhas
     seguem a inclinação do frame;
   - «Equal spacing»: a 3.ª caixa verde, arrastada para a esquerda, prende com os dois vãos iguais;
   - «Frame B»: agarrado pelo branco vazio e arrastado para cima, prende com o topo alinhado ao do A.
4. Deu errado se: uma linha aparece direita onde a lição promete inclinada; uma forma salta para outro
   sítio antes de chegar perto do alinhamento; rodar não prende em 30°/45°/direito, ou prende noutro
   ângulo.

Fotografada (`fotografa_cena.sh`, 1930×1040, 08/10): as cinco lições cabem no ecrã, os textos lêem-se.
Os arrastos não se fotografam (a sessão virtual ignora eventos sintéticos); o gate
`every_lesson_teaches_its_own_alignment` conduz cada um pelo editor e as linhas pela placa
(`a_rotated_guide_paints_along_the_rotated_edges`).

## 10. O que fica ABERTO

1. ⚠️ **O enquadrar das cenas** (achado na foto, já era assim antes): o enquadrar corre no 1.º quadro e
   centra no quadro INTEIRO — os painéis laterais (Hierarchy, Inspector) ficam por cima dele, por isso o
   que fica debaixo deles não se vê. `Alt+1` depois de abrir recentra. A cena 7 foi desenhada para caber
   no meio visível.
2. Recolher só a família que o gesto usa (rodar não precisa das caixas; mover não precisa dos ângulos):
   recusado por agora, medido (§6) — o custo é percorrer os vivos.
3. Por medir numa captura do dono: o que o Miro faz com um rodado (o help cala-se; os frames dele não
   rodam). Se o produto dele mandar outra coisa, as leis L3/L4 são as que mudam (03 §2).
4. Herdados: o §10 do handoff W5 (rascunho de quadro grande, chão das cenas de 100 mil, capturas do
   Miro, painel de Frames, os de W2–W4).

## 10-bis. A máquina

(preenchido no fecho: `target/*/incremental` reclamado e a 2.ª compilação do `smoke`.)
