# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-03 — a F3 do Sculpt (Sculpt ▸ Object · Sculpt · Paint)

> Leitor: o agente integrador, e a próxima janela desta linha. **Cita e NÃO substitui**
> [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md),
> [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md)
> e [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md):
> os commits da escala, do menu Add e da F2 continuam descritos lá. Este cobre os commits DEPOIS do
> `a58dab080` e refaz o gate sobre o diff ACUMULADO desde a base.
>
> **Ordem do dono (03/10):** a F3 do Sculpt nesta linha, sem integrar. O desenho foi mostrado antes
> de codar; duas escolhas dele: **o pill SCULPT sai** e **a peça nova nasce em Sculpt**.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `1ad60a1ce` · **HEAD** = o commit deste
  ficheiro. `git cherry main HEAD` = **41 `+`**, nenhum integrado. Rebase: no-op (o `main`
  não andou).

## §0 — Para o `CLAUDE.md` §5.1 (UI/UX)

Trocar o link do «Último:» por este ficheiro. A frase do módulo não muda.

## §1 — O que esta onda entrega

| commit | o quê |
|---|---|
| `9f242d740` | `ObjectMode::Sculpt`; `ModeFamily` vira trait (+ `follow`, `wants`); a família da escultura; a peça nascida entra em Sculpt; o `D` não entra no barro; as abas pedem o modo; o pill SCULPT sai; `PH2D_OBJECT_MODE_SMOKE=3` |
| `fa296168c` | o gizmo de transformação só existe em Object (`mode_drive::object_gizmo_shows`) — achado pela foto |
| `0c328ec3e` | os 5 gates que moravam no ficheiro do pill e NÃO eram dele voltam (`the_sculpt_frame_is_wired.rs`); cliques reais no seletor da peça e nas abas; fmt |
| `521fae151` | os dois sobreviventes da mutação fechados |
| `c82e56f0b` | fmt |

**O desenho** (spec/06 §4 F3, escrito lá com os desvios):
- `ph2d_editor_core::screens::hero::mode_drive::ModeFamily` é agora um **trait**: `modes`, `holds`,
  `enter`, `leave` (todos com a ENTIDADE), `follow(current)` (o módulo segue o modo que ficou — a
  outra metade da rede `still_holds`) e `wants()` (um objecto que nasceu num modo pede-o; o quadro
  selecciona-o e entra). A shell constrói as famílias **por quadro** com o que cada uma empresta
  (`render_loop/fase_object_mode.rs`). O quadro: 0 `wants` → 1 publicar → 2 rede → 3 pedido →
  4 `follow` → 5 seletor.
- `ph2d_app_painter::paint_mode::Family` (struct unitária; a lei não mudou).
- `ph2d_app_sculpt3d::sculpt_mode::Family<'a>` — `(Sculpt3D, Sculpt)`, `(Sculpt3D, Paint)`; empresta
  `Option<&mut Sculpt3dScene>` e o mapa peça↔entidade (`entities::world_map`). Leis puras `holds`
  (barro ∧ peça viva ∧ Sculpt⇒¬Painter / Paint⇒Painter) e `follow` (`Hold`/`Release`/`Wait`).
  - **Entrar** = `active` = a peça da entidade, `preso = Some(peça)`, barro na tela; Paint põe o
    Painter em mãos (ele prende a tela da vista: `painter_na_malha`, sem mudança).
  - **Sair/Release** = `preso = None`, barro → LUZ (`toggle_clay`), larga o Painter preso à peça.
  - **`Sculpt3dScene::preso`**: com ela, `aim` (a ÚNICA porta que movia o `active` no pen-down) só
    raia a peça presa (`pick_active`) — a outra fica intocada mesmo à frente.
  - **`Sculpt3dScene::pede_o_modo`**: nasce `true` em `Sculpt3dScene::new` e o menu Add marca-o ao
    acrescentar a uma cena existente ⇒ as cinco portas de nascer (Add, load, import, smoke, e a do
    pill que saiu) entram em Sculpt sem linha na shell.
- `cycle_role` (o `D`): do barro → luz; fora dele luz ⇄ desligada; **nunca** entra no barro.
- `slot_tabs_ferramenta::intencao_da_aba(panel, &ModeState, ferramenta)`: a aba da escultura em
  Paint pede `Enter(Sculpt)`; a do Painter em Sculpt pede `Enter(Paint)` — só se o activo declara o
  modo. `abas::abas_seguem_a_ferramenta` deixou de exigir o IMG.

## §2 — Foundational tocado (aditivo) e contratos

- `ph2d-editor-core`: `object_mode` (+ `Sculpt`), `ids/chrome/rail.rs` (+ `OBJECT_MODE_SCULPT`),
  `mode_drive` (trait + `object_gizmo_shows`), `slot_tabs_ferramenta`, e as remoções do pill (§3).
  `hero.rs` **699** · `interaction/state/mod.rs` **696** · `left_rail.rs` **693** · `hero/paint.rs`
  **700** — intocados. `action_bus.rs` **647** (−3).
- **Dependência nova:** `ph2d-app-sculpt3d → ph2d-component-desc` (vocabulário, zero deps — o mesmo
  que a F2 deu ao Painter). `Cargo.lock` regenerado pelo cargo.
- `chrome/mod.rs`: blocos gerados regenerados por `cargo run -p ph2d-chrome-sync` (44 handlers).
- Contratos congelados (§6): **nenhum encostado**.

## §3 — Superfície de colisão (para o integrador)

**Falha ALTO noutra linha (não compila):**
- `ModeFamily` deixou de ser struct (só esta linha a usa; a F2 ainda não está no `main`).
- **APAGADOS:** `ids::TOPBAR_SCULPT3D`, `chrome::sculpt3d_toggle`, `EditorAction::ToggleSculpt3d`,
  `Sculpt3dRequests::{toggle_request, take_toggle}`, `ph2d_app_sculpt3d::mode::{apply_toggle,
  sync_pill}` (+ o re-export `sync_pill`), `App::sculpt3d_apply_toggle`,
  `menu_bar::ModuleTruth::ShellOwned`. `ModuleTruth::resolve` devolve `bool` (era `Option<bool>`);
  `MODULE_TRUTHS` **22 → 21**.
- `ObjectMode` ganhou `Sculpt`: todo `match` exaustivo noutra linha parte.
- `Sculpt3dScene` ganhou `preso` e `pede_o_modo` (o literal mora só no `birth.rs`).
- `intencao_da_aba` mudou de assinatura.

**Funde LIMPO e REPROVA depois:**
- A fila de pills e o menu *Window* perderam a linha SCULPT: um censo de pills/linhas noutra linha
  que a conte reprova.
- i18n **APAGADAS:** `chrome.menu.sculpt_3d`, `chrome.topbar.name.sculpt_3d`,
  `chrome.topbar.pill.sculpt`, `chrome.topbar.tip.sculpt_3d_d_cycles`.
- Gate **APAGADO:** `shells/desktop/tests/it/the_sculpt_pill_enters_and_leaves_the_mode.rs` (e a
  linha em `tests/it/main.rs`); 3 testes de pill em `mode_tests.rs` e o `a_request_with_no_gpu…`.
- `the_shell_only_shrinks`: esta onda **+26 −429** na `shells/desktop` (líquido **−403**); acumulado
  desde a base **+690 −731** (líquido **−41**).

**Enums, ids e chaves (append-only):**
- Id NOVO: `OBJECT_MODE_SCULPT = hash_node_id("object_mode.row.sculpt")`.
- i18n NOVA: `object_mode.sculpt` = «Sculpt Mode».
- Env: `PH2D_OBJECT_MODE_SMOKE=3`.

**Muda comportamento:**
- O pill **SCULPT** e a linha *Window ▸ Sculpt 3D* **saíram**. Esculpir = seleccionar a peça +
  *Mode ▸ Sculpt Mode* (ou Tab), ou *Add ▸ 3D ▸ Esfera/Cubo/…* (nasce em Sculpt).
- Abrir um projecto com escultura, importar uma malha e as cenas `PH2D_SCULPT3D_SMOKE=<n>` entram
  em Sculpt sobre a peça activa (antes: o barro na tela sem modo).
- Fora de Sculpt/Paint o barro sai (a peça não se desenha — ADR-0150: barro e luz exclusivos).
- O `D` já não entra no barro.
- As abas Sculpt 3D / Painter trocam o MODO, e o IMG deixou de ser porta da pintura da peça.
- ⚠️ **O gizmo de transformação some em TODO modo de criação** — também no Image ▸ Paint da F2 (a
  caixa+alças por cima da imagem a pintar). A selecção fica armada.
- Em Sculpt/Paint, a mira só vê a peça presa.

## §4 — Fecho: gate batched, mutação, auditoria

**Gate batched** (verificador, 1× sobre o diff acumulado desde `1ad60a1ce`, `loadavg` ~25 — a
máquina é partilhada):

| portão | resultado |
|---|---|
| `nextest-impacted` (base = merge-base) | **19 670 / 19 671** — o vermelho foi `named_gates_census_tests::every_gate_the_sculpt_family_names_exists` (determinístico, reprova sozinho) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde, zero avisos |
| `clippy --workspace --all-targets -D warnings` | verde |
| `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `cargo fmt --check` | 5 ficheiros desta onda → formatados |

⛔ **O vermelho era um defeito MEU, e é a lição da onda:** o
`the_sculpt_pill_enters_and_leaves_the_mode.rs` tinha **9** gates e só **4** eram do pill. Apaguei o
ficheiro pelo NOME e levei 5 gates de outras leis (a cena nunca toma o clique da moldura · todo
fundo de moldura conhecido · a visibilidade do painel pela borda do barro · o padrão do pincel lê as
camadas vivas · a luz assada só re-autora quando a lâmpada mexe). O censo de gates citados da
escultura apanhou a citação órfã. Voltaram **verbatim** em
`shells/desktop/tests/it/the_sculpt_frame_is_wired.rs` (5/5 verdes). Depois das correcções:
clippy `--all-targets -D warnings` nas 4 crates tocadas, `fmt --check` e o censo — verdes.

**Mutação** (agente `mutacao`; controlos VERDES antes, filtro com população > 0 em cada nome).
Lado INDEPENDENTE: famílias FALSAS no editor-core (`mode_drive_tests`: `ImageFamily` + uma
`SculptFamily` falsa que declara Paint também), a família REAL na crate dela.

| # | mutação | gate | |
|---|---|---|---|
| M1 | `family()` procura só pelo modo | `paint_declared_by_two_types_opens_the_family_of_the_type` | sangrou |
| M2 | o passo 0 (`wants`) apagado | `a_born_object_is_selected_and_enters_its_mode_once` | sangrou |
| M3 | o laço `follow` apagado | `every_family_follows_the_mode_that_stayed` | sangrou |
| M4 | `object_gizmo_shows` → `true` | `the_object_gizmo_shows_only_in_object_mode` | sangrou |
| M5 | a aba da escultura pede Paint | `a_aba_escolhida_diz_que_modo_se_quer` | sangrou |
| M6 | a aba ignora os modos do activo | `a_aba_nao_pede_um_modo_que_o_activo_nao_tem` | sangrou |
| M7 | `holds` sem `h.piece` | `holds_needs_the_clay_the_piece_and_the_right_hand` | sangrou |
| M8 | `follow` sem o braço `Wait` | `the_clay_follows_the_mode` sangrou; o de placa **sobreviveu** → asserção do barro acrescentada → sangra |
| M9 | `enter(Sculpt)` não larga o Painter | **sobreviveu** (o `leave` antes cobria-o) → o gate entra com o Painter por outra porta → sangra |
| M10 | `wants` não consome a marca | `a_born_piece_asks_for_sculpt_once_…` | sangrou |
| M11 | `aim` sem a trava `preso` | `two_pieces_sculpt_on_one_never_aims_at_the_other` | sangrou |
| M12 | o `D` volta ao barro | `the_d_key_never_enters_the_clay` | sangrou |

Os gates de placa (`#[ignore]`): `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p
ph2d-app-sculpt3d --lib -- --include-ignored sculpt_mode` → **6/6**; `mode::tests` (o `D`) verde.

**Auditoria (DIRETIVA §3), três lentes** (formato curto; LOC lidas ~1 070 no total):

- **Correção — a outra peça intocada.** TRAÇO: `despacho_clique_prologo.rs:98` → `sculpt3d_pointer_down`
  (`sculpt3d_host.rs:82`) → os `aim` de `input_down.rs` e `painter_na_malha.rs:336` → `space.rs::aim`
  (com `preso`: `active` = a presa, raio só nela). VERMELHA: `two_pieces_sculpt_on_one_never_aims_at_the_other`
  (M11; controlo em Object toma B). NÃO-CHECADO: Shift+D/undo movem o `active` entre quadros — o
  `follow` e o `aim` repõem a presa (os dois lêem `preso`).
- **Fiação — do clique ao barro.** TRAÇO: linha do seletor / `slot_tabs::apply_event` →
  `EditorAction::ObjectMode` → `fase_bus_drain` → `fase_hero_frame.rs:54` → `fase_object_mode.rs` →
  `mode_drive::drive` → `sculpt_mode::Family::enter`. VERMELHAS: `the_sculpt_row_of_a_piece_asks_for_sculpt`
  e `the_sculpt_tabs_ask_for_the_mode` (cliques reais, todos os painéis), M1. NÃO-CHECADO: a fase pede
  `App`; a foto `=3` percorre-a (Add → `wants` → Enter → seletor).
- **Coerência barro ↔ modo.** As portas do papel: o `D` (nunca entra, M12), o MODEL (tira o barro →
  `holds` falso → Object), load/import/smoke/Add (`pede_o_modo` → `wants`; `Wait` até haver entidade) e
  o `follow` de todo quadro (Release, M8). Uma porta NOVA que ponha o barro sem modo é largada no
  quadro seguinte, por desenho.

## §5 — Premissas do briefing que a medição derrubou

- *«`Sculpt3dScene::active` liga-se à entidade seleccionada»* — só na MUDANÇA da selecção
  (`entities_sync`), e o `aim` do pen-down move-o a cada gesto (a única porta). Abrir «sobre a peça
  desta entidade» pedia a TRAVA (`preso`), não só a ligação.
- *«Um modo novo = uma `ModeFamily` na crate da família»* — as portas só recebiam o
  `ToolRegistry`; a escultura precisa da cena (no `AppGfx`) e do mundo. ⇒ trait por quadro.
- *«O caminho actual da pintura é IMG + a aba do Painter»* — o botão do Painter já tinha saído do
  IMG na F2; restava a aba, guardada pelo IMG.
- *«Nasce em Object»* (spec/06 §3.1) — fora do barro a peça não se desenha; a regra deixava a peça
  nova invisível. O dono escolheu Sculpt.
- A foto achou o gizmo do objecto por cima do barro (nenhum gate o via).
- *«O ficheiro do gate do pill é do pill»* — tinha 9 gates, 4 dele. ⛔ Apagar um ficheiro de testes
  pelo NOME é apagar cada gate que mora nele; liste-os (`grep -n '^fn '`) antes.

## §6 — Smoke do dono (o que ensaiar)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. Botão direito no canvas vazio → **Add Object** → grupo **3D** → **Sculpt Sphere**. Tem de aparecer a esfera
   e, no topo do canvas, o botão **Sculpt Mode**; à direita abre o painel **Sculpt 3D**. Errado: a
   esfera não aparece, ou o botão diz Object Mode.
3. Arrastar o rato sobre a esfera → ela deforma (esculpir). Não aparece o quadradinho azul de mover.
4. Clicar **Sculpt Mode** → a lista tem **Object Mode · Sculpt Mode · Paint Mode** → **Paint Mode** →
   abre o painel do Painter; pintar sobre a esfera. Errado: a lista não tem Paint, ou não pinta.
5. Clicar a aba **Sculpt 3D** (ao lado da aba Painter) → volta a esculpir; a aba **Painter** → volta a
   pintar. Errado: a aba só muda o painel e a mão continua a mesma.
6. **+** da Hierarchy → **Sculpt Cube** → nasce um cubo, já em Sculpt. Esculpir no cubo; a esfera não mexe
   mesmo passando o pincel por cima dela. Errado: a esfera deforma.
7. Em Sculpt, clicar a linha da esfera na Hierarchy → não troca, aviso *«Leave Sculpt Mode (Tab) to
   select another object»*.
8. **Tab** → **Object Mode**: a escultura some do canvas (é o comportamento de sempre ao sair do
   barro). **Tab** outra vez → volta a Sculpt.
9. A barra de cima já não tem o botão **SCULPT**, e o menu **Window** já não tem *Sculpt 3D*.
10. Com uma imagem (**Ctrl+N**) em **Paint Mode**, o quadradinho azul de mover também já não aparece
    por cima dela.

Fotos (tela virtual, `fotografa_cena.sh`): `PH2D_OBJECT_MODE_SMOKE=3` — a esfera em Sculpt com o
seletor aberto nas três faces; conferida antes e depois da cura do gizmo.

## §7 — O que fica para a próxima janela

- **F3 Flip** (o próximo pela medição: escreve sempre no 1.º desenho, `autokey.rs:54`) → Model (um
  por cena) → Vector (a partição do `DrawMode`).
- Em Sculpt, a barra da esquerda continua a mostrar MOVE/ROT/SCALE (as ferramentas do Object; a
  escultura não é uma `Tool`, ADR-0150) — pré-existente; a limpeza é da F4 (toggles que viraram
  modos).
- `Ctrl+Tab` (a lista de modos); Image ▸ Mask; a F4.
- O `Edit` da malha (D6: «Edit quando existir»).

## §8 — Perfil do loop (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.16/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                182   alvo: <= 800
  ✗ cargo test : cargo check                375 : 191   alvo: <= 1,0  razao 2.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  38%   alvo: >= 80%  (1388 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         479 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```

## §9 — Binário de smoke (último passo: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida)

@@BUILD@@
