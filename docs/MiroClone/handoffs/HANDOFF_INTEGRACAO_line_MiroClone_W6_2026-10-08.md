# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W5 + W5b + W6 «Planear: mapa mental e modelos» → `main` (2026-10-08)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra
> nem pusha. **Substitui** o [handoff W5b](HANDOFF_INTEGRACAO_line_MiroClone_W5b_2026-10-08.md) como o
> último da linha: W5 e W5b (smoke do dono OK nas duas, 08/10) não foram integradas e a W6 continua por
> cima, na mesma branch — **integra-se tudo de uma vez**. Para W5/W5b valem os handoffs deles (o que
> mudou, os portões, o aberto); este cobre a W6. Briefing: [continuação W6](HANDOFF_CONTINUACAO_W6_2026-10-08.md).
> Estudo, leis M1–M19 / T1–T6 com fonte e número, recusas e a régua:
> [`../04_planear_mapa_e_modelos.md`](../04_planear_mapa_e_modelos.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| base (merge-base) | `2a925859d` — **o `main` não andou** no fecho ⇒ `--ff-only` se ninguém aterrar antes |
| commits da W6 | `7eb48dd1c`..`78e2743b3` (10) + este handoff; o total da linha: `git log --oneline 2a925859d..line/MiroClone` |
| diff da linha em `crates/` | 82 ficheiros, +14 176 / −669 (W5+W5b+W6); a W6 sozinha: `git diff ca8f35143..HEAD --stat` |
| smoke | **por fazer pelo dono** (cena 8, §8) |

O que a W6 trouxe ao Quadro: **mapa mental** (ferramenta *Mind map*, `Tab` filho, `Enter` irmão, setas
pela árvore, `Delete` leva o ramo, recolher com o número, `+`, arrastar para outro pai, auto layout
desligável, horizontal/vertical) com o layout por FAIXAS medido na imagem oficial do Miro, e a galeria
de **seis modelos** (retrospectiva, brainwriting, kanban, mapa mental, fluxograma, mapa de jornada).

## 2. Foundational / partilhado tocado — e porque é aditivo

| onde | o quê | aditivo? |
|---|---|---|
| `ph2d-board-model` | `Org::mind: Option<Mind>` (no FIM do `Org`), **`FORMAT_VERSION` 5 → 6** com `legacy::read_v5` e `fixtures/format_v5.bin` **gravada pelo build `ca8f35143`** (teste temporário, retirado) — gate `a_format_5_file_still_opens_and_nothing_in_it_is_a_mind_node` | sim: lê 1–5 |
| `ph2d-board-model` | `BoardDoc::live()`/`live_in_z_order()`/`live_len()` **excluem os nós escondidos** por um ramo recolhido (a porta única do desenho, clique, laço, busca); `live_with_hidden()` novo para o que um frame leva (`descendants`, adopção) | só muda num quadro com um mapa recolhido; sem `Mind` nenhum elemento é escondido |
| `ph2d-board-model` | `apply_batch` arruma os mapas que o lote tocou (`mind::settle`, M17), no mesmo passo de desfazer; `BoardOp::id()` | num lote sem nós de mapa, só uma consulta por op |
| `ph2d-board-geom` | `width_for_text` (irmã de `height_for_text`) | sim |
| `ph2d-board-edit` | `Tool::MindMap`, `Command::Mind{Child,Sibling,Nav,Collapse,Auto,Vertical}`, `Template`, `Overlay::{mind_plus, mind_drop}`; módulos `mindmap.rs`, `mindmap_drop.rs`, `templates.rs`, `overlay.rs` (corte puro do `lib.rs`, 693 → 566); `with_descendants` leva o ramo de um nó; `Grow` recusa nós; um nó de mapa não se trava | enums com variantes novas: os `match` exaustivos do `ph2d-editor-core` foram actualizados na linha |
| `ph2d-editor-core` | barra curta 12 → **14** botões (`MIND_TOOL`, `TEMPLATES` logo a seguir à Nota), `Item::Mind`, `Selected::Mind`, `board_bar_mind.rs`, `BoardLive::templates_open`, teclas em `board_keys.rs` (`mind_key`), um clique no quadro fecha a galeria (`board_view.rs`) | sim |
| `ph2d-i18n/src/board.rs` | chaves `board.tool.{mindmap,templates}`, `board.mind.*`, `board.tpl.*`, `board.smoke.mind.*` | sim |
| `ph2d-board-render` | `mind.rs` (ramo antes de cada nó, no recorte do frame dele; selo depois); dev-dep `ph2d-vec-layout` (a régua) | sim |
| `ph2d-app-board` | cena 8 (`smoke_mind.rs`), `max_level` 7 → 8; `ph2d-text` passa de dev-dep a dep (a cena mede o texto dos nós) | sim |
| `.typos.toml` | exclui `docs/MiroClone/ferramentas/mindmap_oracle/saidas/**` (como as dos outros oráculos) | sim |

**Contratos congelados (CLAUDE.md §6): nenhum tocado** — a W6 (`git diff --name-only ca8f35143..HEAD -- crates shells`) toca só `ph2d-board-{model,edit,geom,render}`, `ph2d-app-board`, `ph2d-editor-core` (o quadro) e `ph2d-i18n`; nada em `ph2d-nodegraph`, `ph2d-tool-runtime`, `ph2d-vector`, `ph2d-imageio` nem na shell. **`PROJECT_SCHEMA` não tocado** (187): o formato do quadro tem versão própria.

## 3. Números MEDIDOS

- **Miro** (imagem oficial do modelo «Mind Map», 1440×960, ao píxel — estudo §1.3): passo **54** entre
  nós de uma linha, entre folhas E entre primos; pai no MEIO da faixa (não a média do 1.º e último);
  cada lado da raiz centrado nela; **122** de texto a texto a partir da borda de FORA do pai (⇒ caixa =
  texto + 9, vão **104**); a curva: braço **40,5 = 0,39 × 104** em 5 ramos (rms 0,32–0,97 px) — ⛔ não o
  `END_ARM` das setas; pílula da raiz 170 × 66; recolher **não rearruma** (GIFs do help, quadro a quadro).
- **Oráculos corridos** (`ferramentas/mindmap_oracle/`, MIT pelo pacote instalado, 39/39 iguais em
  duas corridas): ME/SMM por faixas (vãos 75/40/60 e 104/54/54), markmap compacto (recusado); teclas
  `Tab`/`Enter`/setas/`Delete` destiladas por app (estudo §1.2).
- **O mapa oficial refeito** cai a < 1 px dos centros do Miro (`the_miro_template_map_lands_where_the_miro_draws_it`).
- **Régua** (`ph2d-board-render/tests/it/measure_mind_layout_cost.rs`, `--release`, mínimo de 7, `load`
  64,9): arrumar 0,066 / 0,571 / **7,26 ms** a 100 / 1 000 / 10 000 nós; uma tecla `Tab` 0,172 / 0,889 /
  **9,91 ms**; o mesmo pelo `taffy` 0,291 / 2,197 / 27,96 ms ⇒ **nenhum tecto de nós**; o `taffy`
  exprime a lei (`taffy_expresses_the_mind_layout`, < 1e-3: ele conta em `f32`) a ~3,8× do custo ⇒ fica a
  passagem própria.

## 4. Portão batched (1× sobre `2a925859d..HEAD`, depois das curas)

| portão | resultado |
|---|---|
| `nextest-impacted` (`PH2D_GPU=1`) | **17 064 / 17 064** (a 1.ª corrida: 17 040/17 041 — `no_magic_numeric` em `board_bar_mind.rs`, curado com constantes nomeadas) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ |
| clippy do `ship.sh` (`--features ph2d-spike/bevy_ecs --keep-going -D warnings`) | ✓ (a 1.ª: `needless_range_loop` em `templates.rs` e `too_many_arguments` na régua — curados) |
| `check-standalone-optional` · `check-workflow-packages` · `fmt --check` · `machete` · `typos` | ✓ |
| `ph2d-editor-core --test it` | 536 ✓ |
| `ph2d-host-desktop --test it` (tectos da shell, arch gates) | 791 ✓ |
| `censos-da-arvore-combinada.sh` | 12/12 censos, 114/114 |

## 5. Auditoria de 2 lentes (antes da mutação) — 24 achados, todos curados com teste

**Lente A (leis), 12:** duplicar/colar um nó deixava DUAS chaves de ordem iguais e o `Enter` seguinte
entrava em pânico (`FracKey::between`) ⇒ chave própria logo depois do original; os frames não levavam os
nós escondidos (apagar deixava órfãos visíveis, copiar perdia filhos) ⇒ `live_with_hidden`; uma
selecção mista arrastada voltava toda ao sítio ⇒ o largar de ramo só quando só o ramo vai; arrasto
misto / redimensionar / rodar não arrumavam ⇒ `settle_moved`; o modelo de mapa morava no frame da
origem e centrava-se com os nós empilhados ⇒ casa no centro de inserção e layout antes de centrar; um
ramo largado não mudava de frame (M18); largar num recolhido escondia o largado ⇒ abre-o; colar um ramo
recolhido seleccionava escondidos; `Delete` num travado saltava a selecção e os órfãos de um ramo travado
⇒ nós de mapa não se travam (o Miro) e um ramo colado sem pai vira raiz; a cor repetia a do vizinho ⇒ a
seguinte à do último; o filho de um pai manual nascia por cima dele.
**Lente B (costura), 12:** `Ctrl+↓` num nó aberto clonava-o com uma seta (caía no `Grow`); a lição 4
fechava o mapa inteiro (o selo não seleccionava o nó); a galeria sobrepunha-se às janelas da Nota e da
Caneta e não fechava com um clique no quadro; a zoom > 1 o «+» tapava o número e roubava-lhe o clique;
os ramos ignoravam o recorte do frame (gate de píxeis na placa); o número do selo a branco lia-se a
1,4:1 ⇒ `readable_ink`; a dica de recolher não seguia o estado; setas presas a escondidos; empurrar nós
arrumados deixava um passo vazio; números de desenho soltos (portão); o cabeçalho do roteador da cena.

## 6. Prova de mutação — 31/31

31 mutações, uma por lei nova (layout por faixas, vão, borda de fora, lados, recolher sem rearrumar,
escondido, manual, vertical, `settle` no lote, braço da curva, lado mais leve, `Tab` último, `Enter`
logo depois, setas ao primo, casa do pai, cor seguinte, chave da cópia, `Delete` → pai, empurrar,
ramo no `with_descendants`, largar → pai, selo selecciona, selecção mista, largar abre, setas a
escondidos, descendentes do frame, fronteira do `Tab`, galeria, dica, recorte do ramo (GPU), `+` sem o
lado do selo). Baseline verde e filtros com ≥ 1 teste; restauro por `git checkout` + `touch`. A 1.ª
passada deixou **2 sobreviventes** (recolher um ramo de UMA faixa não prova nada; ninguém testava
`descendants` directamente) — tapados em `78e2743b3` e re-mutados: sangram.

## 7. Fixtures, instrumentos e o que vive versionado

`ferramentas/mindmap_oracle/` (instala, corre, entradas, `saidas/` — a 2.ª corrida é ignorada pelo
`.gitignore`), `ferramentas/planear_pesquisa_miro_e_apps.md`, `ferramentas/cena8_foto.png`,
`fixtures/format_v5.bin`. As réguas ao píxel da imagem do Miro foram scripts de scratchpad: os números
estão transcritos no estudo §1.3 e no cabeçalho de `mind_tests.rs`.

## 8. Smoke (o comando exacto)

```bash
cd ~/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=8 ./target/smoke/ph2d-host-desktop
```

Abre um mapa começado («Plan», «Ideas», «Risks» recolhido com o «3», «Next steps») e seis notas com as
lições, num frame. O gate `every_mind_lesson_does_what_it_says` conduz cada lição pelo editor a zoom 1 e
0,8; as teclas pela porta da shell estão em `board_bar_mind_tests.rs`.

## 9. O que fica ABERTO

1. ⚠️ **O enquadrar das cenas** (W5b §10.1, continua): o 1.º quadro enquadra o quadro inteiro na JANELA
   e ignora a câmara da cena; a cena 8 vive num frame com a margem do que os painéis tapam. A cura é no
   `ph2d-editor-core` (enquadrar a área visível) — não é da W6.
2. Letra do mapa pela raiz (o Miro: muda-se só na raiz e escala o mapa): o layout já escala pela letra
   da raiz, mas a barra de contexto de um nó só mostra recolher/auto layout/direcção — mudar a letra da
   raiz não re-mede os filhos.
3. No **rascunho**, os ramos saem lisos (a pílula sai à mão).
4. O «−» ao PAIRAR do Miro para recolher: aqui recolhe-se pelo botão da barra e por `Ctrl+↑`; o «+» só
   no nó seleccionado (o Miro mostra-o ao pairar).
5. O vão do layout VERTICAL é o do horizontal (o Miro não o publica; sem captura).
6. Nós de mapa rodam e agrupam (o Miro não deixa); só o travar foi recusado.
7. A barra curta com 14 botões mede ~400 px; nenhum gate verifica que cabe numa janela baixa.
8. A tinta do número do selo (`readable_ink`) está provada pela paleta (`every_branch_colour_has_a_readable_badge_ink`), não pelos píxeis do texto.
9. Herdados: o §10 do handoff W5b e o §10 do W5.

## 10. Para o integrador

- `CLAUDE.md` §5.1, entrada do Quadro: o link «Último» passa a **este handoff**; a frase passa a (≤ 300 car.):
  *«… organizar (frames, grupos, travar, minimapa, busca, nível de detalhe), guias de rodados e planear
  (mapa mental por faixas do Miro e seis modelos, [estudo](docs/MiroClone/04_planear_mapa_e_modelos.md))
  … Smoke `PH2D_BOARD_SMOKE=1..8`.»*
- Conflitos prováveis só se outra linha tocar em `ph2d-i18n/src/board.rs`, `board_bar.rs` ou
  `board_keys.rs` (aditivos, por linhas vizinhas).

## 11. A máquina

`agent-loop-profile.sh` (20 sessões): paralelismo 1,15/passo (alvo ≥ 1,5) · respostas/sessão 180 ✓ ·
test:check 1,7× (alvo ≤ 1) · edições pelo `Edit` 38 % (alvo ≥ 80 %: os cortes e as curas em bloco foram
por script com `assert` de contagem) · contexto relido 398 mil (alvo ≤ 250 mil) · início 63 mil ✓.
`target/*/incremental` reclamado e o binário `smoke` compilado 2× — a 2.ª saída em §11-bis.
