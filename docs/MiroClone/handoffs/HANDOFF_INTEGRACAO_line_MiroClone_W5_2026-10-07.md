# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W5 «Organizar» → `main` (2026-10-07)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra
> nem pusha. Antecessores: [integração W0–W4](HANDOFF_INTEGRACAO_line_MiroClone_2026-10-07.md) ·
> [briefing W5](HANDOFF_CONTINUACAO_W5_2026-10-07.md). Plano: [`../02_plano.md`](../02_plano.md) §3 W5.
> Leis e fontes: [`../ferramentas/miro_organizar_notas.md`](../ferramentas/miro_organizar_notas.md) (o help
> do Miro, o produto) e o README do oráculo Excalidraw §W5 (o número, 51 casos).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| HEAD | `git rev-parse line/MiroClone` (o último commit é o deste handoff) |
| base (merge-base) | `2a925859d` — **o `main` não andou** desde a reabertura (`git rev-parse main` = `2a925859d` no fecho) ⇒ `--ff-only` se ninguém aterrar antes |
| commits | 12 (`git log --oneline 2a925859d..line/MiroClone`) |
| diff | 163 ficheiros, +25 594 / −421 (antes deste handoff; o grosso são as 51 fixtures JSON do oráculo em `ferramentas/excalidraw_oracle/saidas/`) |
| smoke | **por fazer pelo dono** (cena 6, §9) |

O que a W5 trouxe ao Quadro: **frames** (levam os filhos, nome e cor, aninháveis, conteúdo recortado na
borda, sempre atrás, painel de Frames), **grupos** (clique apanha o de fora, duplo-clique entra),
**travar**, **ordem** (`PgUp`/`PgDn`), **alinhar/distribuir**, **minimapa** (`M`), **busca** (`Ctrl+F`)
com **etiquetas** nas notas e filtro por elas, **links** entre elementos (`Ctrl+Alt+K`), **enquadrar**
(`Alt+1`/`Alt+2`) e o **nível de detalhe** de longe (notas = blocos de cor, frames = só o título).

## 2. Foundational / partilhado tocado — e porque é aditivo

| onde | o quê | aditivo? |
|---|---|---|
| `ph2d-board-model` | **`FORMAT_VERSION 4 → 5`** (o blob próprio dos quadros, dentro do `ProjectFile.boards` — **o `PROJECT_SCHEMA` NÃO muda**, 187 = main): `ElementKind::Frame` no FIM do enum; `Element::frame` + `Element::org: Option<Box<Org>>` (grupos, travar, link, etiquetas); `BoardDoc::frames` (índice `serde(skip)`, refeito ao ler); `legacy::read_v4` com a fixture `format_v4.bin` GRAVADA pelo build do formato 4 (`90856427c`); `Element::aabb` sem trigonometria a ângulo 0; `Camera::fit`; `BoardDoc::extent` | sim — lê 1–4, escreve 5 |
| `ph2d-board-edit`, `-render`, `ph2d-app-board` | as leis (`organize.rs`), o desenho (`frames.rs`, `tags.rs`, `note.rs`), a cena 6 | sim — da família |
| `ph2d-editor-core` | `documents.rs` (+`BoardLive::{notice, nav}`, `documents_nav.rs`), `ids/chrome/documents.rs` (+3 ids de campo), `screens/hero.rs` (+`mod board_nav`), `paint.rs` (+1 linha: `board_nav::paint`), `document_tabs.rs` (+2: populate/apply), `board_view.rs` (o minimapa antes do editor), `board_keys.rs` (`PageUp`/`PageDown`, teclas de organizar, a navegação antes), `board_bar*.rs` (+`board_bar_layout.rs`, `board_bar_organize.rs`), `board_nav*.rs` novos | sim — tudo só actua com uma aba de QUADRO activa |
| `shells/desktop` | `input_dispatch/keyboard_board.rs` **+2 linhas** (`PageUp`/`PageDown` → `BoardKey`) | sim — a catraca `the_shell_only_shrinks` verde |
| `ph2d-i18n` | `board.rs` (+~60 chaves `board.org.*`, `board.nav.*`, `board.notice.*`, `board.smoke.org.*`) | sim |

Nenhuma mudança de comportamento FORA do quadro.

## 3. Símbolos que podem colidir — `bash scripts/collision-surface.sh` (07/10)

```text

SUPERFÍCIE DE COLISÃO — line/MiroClone contra main
  merge-base 2a925859d   ·   11 commit(s)   ·   163 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        187   (base: 187)
      └ tripla do gate               (187, 13, 22)   (base: (187, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              105   (base: 105)
    ph2d-render (espelho)                 106   (base: 106)
    ph2d-script (espelho)                 106   (base: 106)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0180   próximo livre: 0181
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
    nenhum arquivo da linha passa do teto
```

- O que mais pode chocar num rebase: `ph2d-editor-core/src/documents.rs` (`BoardLive`, campos no fim),
  `screens/hero.rs` (lista de `mod`), `screens/hero/paint.rs` (a linha do `board_nav::paint` antes do
  `board_bar::paint`), `document_tabs.rs` (cadeias de `populate`/`apply_event`), `ids/chrome/documents.rs`,
  `ph2d-i18n/src/board.rs` (só desta família) e o `keyboard_board.rs` da shell (2 linhas no `match`).
- Ids derivados novos (`board_nav.rs`): saltos `0x4e61_{10,20,30,40}…` — separados nos bits 40–47 dos
  da barra (`0xb0a2_d7ab_5ba2_0000`) e das abas; os descodificadores exigem os bits de cima a zero
  (medido: saltos que só diferiam nos bits 32–33 colidiam — §6).

## 4. Contratos congelados

**Nenhum encostado.** `collision-surface.sh`: `node.rs` e `tool.rs` intocados; `git diff 2a925859d..HEAD --stat -- crates/ph2d-vector-doc crates/ph2d-nodegraph crates/ph2d-editor-core/src/tool.rs` **vazio**; gates à parte no fecho: nós (`ph2d-nodegraph`, `contract`) **3/3**, ferramentas (`architecture_tool_contract_surface`) **4/4**, vector (`ph2d-vector-doc`, `contract`) **11/11**.

## 5. Portão de fecho (batched, 1× sobre `2a925859d..HEAD`)

| portão | resultado |
|---|---|
| `PH2D_GPU=1 BASE=2a925859d NO_FAIL_FAST=1 bash scripts/nextest-impacted.sh` | **16 962 / 16 962** (2.ª passagem, árvore parada em `8912a1f8b`; a 1.ª, de descoberta, deu 5 vermelhos — abaixo) |
| clippy do `ship.sh` (`--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`) | limpo (1.ª: `needless_lifetimes`; 2.ª: `manual_range_contains` — curados, re-corrida com saída 0) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | limpo |
| `cargo fmt --all -- --check` | limpo (1.ª passagem: 27 ficheiros) |
| shell: `file_loc_caps`, `fn_loc_caps`, `arch*` (`ph2d-host-desktop --test it`) | 22 / 22 |
| `ph2d-editor-core --test it` inteiro (`arch_safe_clamp_only`, `architecture_workspace_file_loc_cap`, `the_shell_only_shrinks`, censos HR-15) | 536 / 536 (12 ignorados de nascença) |
| `bash scripts/censos-da-arvore-combinada.sh` | 114 / 114, os 12 censos com controlo |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` · `typos` · `cargo machete` | verdes (10 crates · 32 nomes) · nada · nenhuma dependência morta |
| GPU: `frames_paint` (pixels) | 2 / 2, sem «sem adaptador» |
| código `#[cfg(target_os` escrito/movido | nenhum |
| **auditoria (2 lentes)** | **correcção**: `same_pose` ignorava a casa (largar no mesmo sítio noutro frame não se desfazia) — curado + teste; travado não se EDITA — 12 caminhos de estilo/ordem/etiqueta passam por `Editor::editable` + teste. **costura**: o `M` com uma nota seleccionada abria o minimapa em vez de escrever nela (a lente deu o teclado como limpo; achado meu) — `would_type_into_note` + teste; minimapa com zero células não subtrai. |
| **prova de mutação** (22 leis: pertença ao frame, ordem, travar, alinhar/distribuir, grupos, índice dos frames, `read_v4`, recorte e blocos pela placa, a tecla `M`) | **22 / 22** — 19 à 1.ª; M17 (`same_pose` sem a casa) era buraco real, M8 (o `movable` sem o filtro) e M6 no editor (frame mais exterior) eram testes que não chegavam lá: tapados (`7ef67556e`) e re-mutados — sangram |

A 1.ª passagem (vermelha) apanhou: o ciclo `documents → screens` (o estado da navegação vivia no
ecrã → `documents::nav`), 59 números de glifo sem `LITERAL-PX-OK` (o `rustfmt` separava a marca do
número → constantes com nome, uma por linha), `0xffff_ffff`, «Minimap (M)» lido como unidade, e as
setas `←→↑↓` que a letra da interface não tem (tofu) → palavras.

## 6. Números MEDIDOS (`--release`, RTX 5060 Ti, variantes no MESMO processo intercaladas; mínimo)

**Nível de detalhe** (régua nova `ph2d-board-render/tests/it/measure_detail_cost.rs`, load 4,8):

| quadro | variante | CPU ms/imagem | placa (parede) ms |
|---|---|---|---|
| 10 mil notas de longe | sombra + riscas | 1,270 | 0,979 |
| 10 mil notas de longe | **blocos** (de nascença) | **0,572** | **0,508** |
| 100 frames × 100 formas | conteúdo | 5,372 | 1,671 |
| 100 frames × 100 formas | **só título < 96 px** (de nascença) | **0,229** | **0,466** |
| 1 000 frames × 100 formas | conteúdo | 13,913 | 4,213 |
| 1 000 frames × 100 formas | só título < 96 px | 4,813 | 1,628 |

O limiar 96 px: entre 48 (não apanha os frames de 89 px da cena) e 192 (mesmo ganho) — um frame de 96 px
com 10×10 formas mostra-as a ~7 px, ilegíveis; o ganho é de o limiar APANHAR os frames, não do número.

**A regressão que a régua apanhou e a atribuição sem perfilador** (`perf_event_paranoid` fecha o `perf`
e o `samply` a quem não é root): 100 mil rectângulos à vista iam de ~6,0 ms (binário do formato 4) a
~9,9 ms. Experiências no mesmo processo: o andaime dos frames custava ~0,8 ms e recortar pela vista
ANTES de ordenar +1,4–3 ms com tudo à vista (−1,2 de perto) — recusado. O resto: o binário do formato 4
com **+81 bytes de enchimento** no `Element` deu 7,9–8,6 ms, tão lento como o novo ⇒ era o TAMANHO do
elemento. Cura: grupos/travar/link/etiquetas em `Option<Box<Org>>` (+24 bytes em vez de +81) e o índice
dos frames no documento; depois, alternando processos: antigo 5,77–6,49 · novo 6,36–6,53 ms (+0,05…0,5).

**Rascunho e 1.ª imagem** (régua `measure_encode_cost`, load 4,2): 10 mil formas de longe em rascunho
11,1 ms de CPU (final 6,0); a 1.ª imagem de 10 mil formas 42,8 ms (rascunho 61,5 — a geração do traço à
mão) — o §10.4 do handoff anterior fica medido. ⚠️ Não atacado nesta onda: o rascunho de 10 mil formas
a ~8 px fica acima de 8 ms (ver §10).

**Pixels pela placa** (`frames_paint`, corre com `PH2D_GPU=1`): o filho que atravessa a borda corta-se
nela; o frame de 80 px mostra só o fundo; a nota de longe é um bloco sem sombra.

## 7. O que só o `ship.sh` / o CI pegaria

- Nada pendente: o clippy, o `deny warnings`, o fmt, o typos, o machete, os opcionais e os pacotes dos
  workflows correram no fecho (§5).
- ⚠️ O CI corre ~26 pacotes; as suítes do quadro vivem nas crates `ph2d-board-*` e em `tests/it` — o
  portão do integrador sobre a árvore combinada é quem as volta a correr.

## 8. Ordem e dependências

Uma linha só, sem dependência de outra. Se o `main` andou: rebase; o choque mais provável é o da §3.

## 9. Smoke (binário `smoke` já compilado na worktree)

```bash
cd ~/Documentos/Projetos/PH2D/Worktrees/line-MiroClone
PH2D_BOARD_SMOKE=6 ./target/smoke/ph2d-host-desktop   # organizar: frames, minimapa, painel de Frames, busca, etiquetas, link, grupo, travado
PH2D_BOARD_SMOKE=1 ./target/smoke/ph2d-host-desktop   # (1–5 continuam: abas, formas, setas, notas, caneta)
```

Fotografada antes de mandar (`fotografa_cena.sh`, 1930×1040): de longe (os quatro frames só em
título, o gigante em blocos de cor, minimapa e painel abertos), a 22 % (blocos, grupo em riscas, o
ícone de link) e a 75 % sobre a Pesquisa (as fitas «client», o texto das notas) — as duas últimas
com a câmara mudada só para a foto. O que a foto apanhou está no commit `bbb17addd`.

## 10. O que fica ABERTO

1. **Rascunho de quadro grande**: 10 mil formas a ~8 px custam 11,1 ms de CPU por imagem (final 6,0) —
   candidato: abaixo de N px no ecrã o traço à mão é invisível ⇒ desenhar o final (MEDIR o N, a régua
   `measure_encode_cost` já tem a cena `Sketch`).
2. **O chão das cenas de 100 mil**: ordenar os vivos a cada imagem (~1,3 ms) e percorrer o `BTreeMap`
   (~0,5 ms/passagem); o índice de z mantido pelas ops continua RECUSADO (plano §6, medido pior).
3. Por medir numa captura do dono: o desenho das etiquetas na nota (o help do Miro cala-o), o cadeado
   no canto do travado, o recorte dos frames no quadro (o Miro cala-o; adoptado o do Excalidraw).
4. Divergências ESCOLHIDAS do oráculo (testes documentam): o passo à frente/atrás passa o primeiro que
   TOCA (o Miro) e não o vizinho da lista (Excalidraw D1/D2); apagar um frame leva o conteúdo (a lei
   do briefing; o Excalidraw A7 solta-o); frames aninhados movem-se juntos (o Excalidraw recusa
   aninhar; no Miro é uma queixa aberta).
5. Painel de Frames: reordenar por setas (o Miro arrasta); renomear também no título do frame no quadro.
6. Herdados (W2–W4): pressão da mesa (`winit`), capturas do Miro por medir (P/G, sombra), ponta curta do
   Excalidraw, laço/smart drawing, cotovelo arrastável, rótulo ao longo da linha, abas que transbordam,
   desfazer das abas, trackpad, menu do botão direito.

## 10-bis. A máquina

`target/*/incremental` reclamado (39 G + 4,6 G); binário `smoke` quente — 2.ª corrida:
`Finished \`smoke\` profile [optimized] target(s) in 0.23s`; `pgrep -af 'ph2d|cargo|rustc'` sem
processos da linha (os que aparecem são de `line-motion-value` e de um slot do pincel).
