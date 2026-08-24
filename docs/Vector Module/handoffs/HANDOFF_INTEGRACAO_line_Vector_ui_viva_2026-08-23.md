# HANDOFF DE INTEGRAÇÃO — `line/Vector` · a booleana nos ESTADOS + o fecho do estudo de UI VIVA (2026-08-23)

> **Leitor:** o agente **integrador**, munido de todos os handoffs da jornada (DIRETRIZ §1.5.3–1.5.4).
> A linha está **fechada e parada**: nada foi pushado, nada foi integrado, `foundational-integrate.sh`
> **não** foi rodado ([`CLAUDE.md §0.7`](../../../CLAUDE.md)).

---

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/Vector` |
| **HEAD** | `af40484bb37455da7f19be70197bc9f1c1afd1fc` |
| **Merge-base com `main`** | `35f937cb2a42b28aeeaf685afb5ad185df28fd18` |
| **Commits** | **19** |
| **Arquivos** | **116** (`+8 347 / −1 495`) |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |

---

## 2. O que a linha entrega (para ler o diff, não para repetir a narrativa)

Seis waves, todas com **«smoke OK» do Enio**. O *mecanismo* de cada uma está no doc que ela cita.

| # | Wave | Onde vive | Doc |
|---|---|---|---|
| 1 | **Os 4 chips da booleana estavam MORTOS** — a fileira nunca era pintada (o sujeito é o **primário**), e faltava o registro no `populate_ops` | `ph2d-panel-vector` | [`27_um_verbo_por_forma.md`](../27_um_verbo_por_forma.md) |
| 2 | **A booleana viva ANIMA nos estados de UI** — o verbo troca e o resultado **morfa**, com os operandos a mover-se (pos/scale/rot) | `ph2d-ui-state`, `shells/desktop` | [`28_plano_booleana_viva_nos_estados.md`](../28_plano_booleana_viva_nos_estados.md) |
| 3 | **Auditoria de 2 lentes** da wave 2 — **7 achados**, os dois graves invisíveis a toda a suíte; todos corrigidos | idem | idem §auditoria |
| 4 | **O realce de proveniência (C2)** — o objecto sob o ponteiro acende a linha da hierarquia **e** ganha contorno, por **uma fonte só**; estendido a **todos os objectos, em todos os modos** | `shells/desktop`, `ph2d-vec-render` | [estudo §6.2-bis](../Estudos/ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md) |
| 5 | **O PIE MENU (E4)** — segurar `P` põe as ferramentas em **oito direcções** sob o cursor | `ph2d-editor-core` | [estudo §6.5](../Estudos/ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md) |
| 6 | **O SOM DE UI (D1)** — quatro vozes sintetizadas, **desligadas por omissão** | `shells/desktop` | [`ui_sound.rs`](../../../shells/desktop/src/ui_sound.rs) |
| — | **BUG #27** — o traço virava **caneta elíptica** sob Scale não-uniforme | `ph2d-vec-render` | [`BUGS_vector.md`](../BUGS_vector.md) |

---

## 3. Foundational / compartilhado tocado — e por quê

⚠️ Esta linha é **larga em foundational**. Leia esta secção inteira antes de escolher a ordem de fusão.

### 3.1 `crates/ph2d-editor-core` — ⛔ **UMA QUEBRA DE API PÚBLICA**

⛔⛔ **`flat_button_surface` DEIXOU DE SER RE-EXPORTADO** de `widget/`.

```diff
-pub use button::{Button, ButtonKind, ButtonState, ICON_BUTTON_SIZE_PX, flat_button_surface, paint_button};
+pub use button::{Button, ButtonKind, ButtonState, ICON_BUTTON_SIZE_PX, paint_button};
+pub use button_surface::{chip_axis_color, chip_axis_t, flat_button_surface_color};
```

O mapa duro `(ButtonState) -> ColorToken` passou a ser **privado** dentro de
`widget/button_surface.rs`, e a porta pública é `flat_button_surface_color(v, theme) -> Color`,
que já lê o **relógio do eixo de hover**. Foi deliberado: *quatro pintores do mesmo quadrado, e um
deles amaciava* — tornar o mapa privado é o que faz o **compilador enumerar** os sítios de
migração em vez de os deixar à convenção.

⇒ **Se outra linha chamar `flat_button_surface`, ela NÃO compila depois da fusão.**
A migração é mecânica (passar `(state, t)` e o tema); o gate
`crates/ph2d-editor-core/tests/the_chip_axis_has_one_door.rs` recusa uma segunda porta.

**Quem denuncia a colisão é o COMPILADOR**, e é para isso que o mapa ficou privado — um `cargo
check -p <crate-da-outra-linha>` depois da fusão é a prova, não um grep. ⚠️ O grep abaixo é só
pré-leitura e **acerta em comentários** (verificado 2026-08-23: as ocorrências restantes na árvore
são 3 comentários e a agulha de um gate; **nenhuma chamada**):

```bash
grep -rn "flat_button_surface\b" --include="*.rs" . | grep -v flat_button_surface_color
```

### 3.2 Restante em `editor-core` — **aditivo**

| Arquivo | O quê |
|---|---|
| `src/widget/mod.rs` | ⚠️ **bloco `mod` GERADO** (`ph2d-widget-sync`): entraram `button_surface` e `radial_menu`. Se o merge o tocar, **re-rode `cargo run -p ph2d-widget-sync`** em vez de o editar |
| `src/widget/radial_menu.rs` | **NOVO** — o widget do pie menu (`MAX_SECTORS = 8`, testes inline) |
| `src/widget/button_surface.rs` | **NOVO** — §3.1 |
| `src/screens/hero.rs` | campo `pub ui_sound: bool` **apendado**; `pub mod radial;` |
| `src/screens/hero/radial.rs` (+ `radial_tests.rs`) | **NOVO** — o modelo do menu, derivado da secção do meio da tool-rail |
| `src/interaction/state/{mod,store_core}.rs` | `mod radial_ops;` + campo `pub(super) radial: Option<RadialOpen>` |
| `src/interaction/state/radial_ops.rs` | **NOVO** — `open_radial` / `close_radial` / `radial_point` |
| `src/widget/{button,tool_rail/paint,tool_rail/tests}.rs`, `screens/hero/{paint,fixture,topbar/cluster_painter}.rs` | migração para a porta única de §3.1 |
| `tests/architecture_panel_loc_cap.rs` | ⚠️ **allowance BAIXADA** `281 -> 267` (`paint_hierarchy_row`, pago por extracção). *Ela encolhe, nunca cresce* |
| `tests/{architecture_widget_showcase_coverage,hr12_widgets_a11y}.rs` | o widget novo entra nos censos |
| `tests/the_chip_axis_has_one_door.rs`, `tests/the_flat_surface_reads_the_clock.rs` | **NOVOS** arch-gates de §3.1 |

### 3.3 `shells/desktop` — largo, e com **duas** notas de fusão

| Arquivo | O quê | Risco |
|---|---|---|
| `src/project_schema.rs` | ⛔ **`PROJECT_SCHEMA` 89 -> 90** — vide §4 | **ALTO** |
| `src/project_schema_history.rs` | ⛔ **NOVO: a ESCADA foi PARTIDA outra vez** (teto de LOC) — vide §4 | **ALTO** |
| `src/project_schema_tests.rs` | a **tripla** do gate `(89,13,14) -> (90,13,14)` | **ALTO** |
| `src/render_loop/mod.rs` | o pick de hover por quadro + o dreno do som + `ui_host` (5 gestos) | médio |
| `src/app_state.rs` | 4 campos apendados: `hovered_object`, `hover_outline`, `pending_ui_sound`, `ui_bool_morphs` | baixo |
| `src/hover_highlight.rs` (+ `_tests.rs`) | **NOVO** — o pick composto, que **existia em triplicado** e passou a existir uma vez | médio |
| `src/input_dispatch.rs`, `input_dispatch/keyboard.rs`, `input_handlers.rs` | o `P` segurado; os consumidores do pick único | médio |
| `src/audio.rs` + `src/audio/ui_voice.rs` (**NOVO**) | `play_ui`; corte por teto de LOC | baixo |
| `src/ui_sound.rs` (+ `_tests.rs`) | **NOVO** — as 4 vozes e a guarda única | baixo |
| `src/prefs.rs`, `prefs_tests.rs`, `init.rs`, `main.rs` | 3.º eixo `ui_sound` no `~/.ph2d/prefs.txt` (formato **tolerante**, sem versão) | baixo |
| `src/radial_input.rs` (+ `_tests.rs`) | **NOVO** — ponteiro/commit do pie menu | baixo |
| `src/bool_live*.rs`, `vec_ui_state_*.rs`, `vec_bool_*.rs`, `ui_states_bool_smoke*.rs` | as waves 1–3 | baixo (pasta do módulo) |
| `src/build_smoke_router.rs` | **cena `=74`** — vide §4 | médio |
| `tests/the_highlight_has_one_source.rs` | **NOVO** — 8 gates da wave 4/6 | baixo |

### 3.4 `scripts/nextest-impacted.sh` — ⭐ **duas curas, e elas afectam TODAS as linhas**

⚠️ **As outras linhas que fecharem hoje ainda têm a versão CEGA.** Vale a pena integrar esta
primeiro, ou ao menos avisar quem correr o gate de fecho.

1. ⛔ **O trabalho NÃO COMMITADO era invisível.** `git diff A...` compara **commits**: um arquivo
   editado e por commitar não entrava no conjunto. **Medido 2026-08-23:** um fecho com quatro
   arquivos da `ph2d-ui-state` por commitar correu **10 418 testes sem correr um único** daquela
   crate — e saiu **verde**. O conjunto passou a ser a **união** do diff commitado com o
   `git status --porcelain`.
2. `NO_FAIL_FAST=1` passou a existir. O `CLAUDE.md §5.0` manda usar `--no-fail-fast` e **o script
   que executa a regra não a oferecia**: uma corrida que tropeça numa flake de relógio deixava
   ~7 000 testes por correr. *Uma regra fora do caminho de quem a executa não existe.*

### 3.5 Docs e memória

`CLAUDE.md` (**§5 Vector, linhas de estado/Aberto + a 5.ª flake no §5.0**) ·
`docs/Vector Module/{27,28,BUGS_vector}.md` · o **estudo de UI viva** (§6.2-bis, §6.5, **§6.6**) ·
`project-memory/` (**3 memórias novas**, 3 editadas — vide §4).

---

## 4. Símbolos que podem COLIDIR — a saída da sonda, colada

> ⚠️ **Referência, nunca evidência** (DIRETRIZ §1.5.9 item 3). Medida contra o `main` de
> **2026-08-23 22:50**. **Re-rode `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`
> dentro desta worktree imediatamente antes de fundir** — a divergência entre as duas leituras é
> ela própria um achado, e aponta a linha que integrou no meio.

```text
SUPERFÍCIE DE COLISÃO — line/Vector contra main
  merge-base 35f937cb2   ·   19 commit(s)   ·   116 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                         90   (base: 89)
  ⚠   └ tripla do gate               (90, 13, 14)   (base: (89, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES
    ph2d-ecs                               65   (base: 65)
    ph2d-render (espelho)                  66   (base: 66)
    ph2d-script (espelho)                  66   (base: 66)

▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR
    último no disco: 0162   próximo livre: 0163
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
```

### 4.1 ⛔⛔ `PROJECT_SCHEMA` 89 -> 90 — **e a escada MUDOU DE ARQUIVO**

O número **soma entre linhas, não se escolhe** (`CLAUDE.md §5.0`), e ⚠️ **a colisão passa MUDA
quando duas linhas escrevem o MESMO literal**. São **três** sítios:

| Sítio | Arquivo | Valor |
|---|---|---|
| a constante | [`project_schema.rs:161`](../../../shells/desktop/src/project_schema.rs) | `PROJECT_SCHEMA: u32 = 90` |
| o degrau | [`project_schema.rs:141`](../../../shells/desktop/src/project_schema.rs) (`/// v90 …`) | texto |
| a tripla | [`project_schema_tests.rs:393`](../../../shells/desktop/src/project_schema_tests.rs) | `(90, 13, 14)` |

⛔ **NOVO E IMPORTANTE:** o teto de LOC obrigou a partir o arquivo — os degraus **históricos**
mudaram-se para **`shells/desktop/src/project_schema_history.rs`** (459 linhas), e
`project_schema.rs` ficou com **161** (a constante, o degrau novo e a doutrina).
Isto é a **terceira** vez que este arquivo se parte (a `line/physics` partiu-o em 15/08), e é
exactamente o modo de falha que o `CLAUDE.md §5.0` nomeia: *um degrau escrito no arquivo errado
funde **limpo** e evapora*.

⇒ **Se outra linha também subir o schema:** o valor certo é **`base + nº de linhas que sobem`**,
contado, **nunca** um dos dois lados do conflito. O degrau dela tem de aterrar em
`project_schema.rs` (a ponta viva), não em `project_schema_history.rs`.

**Conteúdo do degrau v90:** `ph2d_ui_state::ObjectPose` ganhou **dois** campos apendados —
`bool_op: Option<u8>` (o verbo por forma) e `bool_group_op: Option<u8>` (o verbo do grupo).
O postcard é **posicional**, logo um save v89 lido como v90 leria além do fim do registo.

### 4.2 Outros literais que somam

| Símbolo | Valor desta linha | Onde |
|---|---|---|
| **cena de smoke** `PH2D_BUILD_SMOKE` | **`=74`** | [`build_smoke_router.rs`](../../../shells/desktop/src/build_smoke_router.rs) — ⚠️ **conte pelo roteador**, não por esta tabela; há gate `no_two_*_scenes_claim_the_same_level` |
| **chaves i18n** (5, todas com prefixo próprio) | `panel.vector.states.host`, `.host.unnamed`, `.nohost`, `.nohost.hint` | [`ph2d-i18n/src/vector.rs`](../../../crates/ph2d-i18n/src/vector.rs) |
| **allowance de LOC** | `paint_hierarchy_row` **281 -> 267** | `architecture_panel_loc_cap.rs` — ⚠️ se outra linha a **subir**, ganha o **menor** |
| **tokens de design** | **nenhum novo** | — |
| **ADR** | **nenhum** | — |
| **registos ECS** | **nenhum** | — |
| **memórias novas** | `feedback_a_silent_output_channel_can_be_muted_outside_the_process.md` · `feedback_an_inequality_accepts_a_whole_interval_only_an_oracle_accepts_an_answer.md` · `feedback_the_three_ui_seam_questions_miss_the_fourth_the_sequence.md` | `project-memory/` + **3 linhas no `MEMORY.md`** (arquivo compartilhado — conflito **textual** provável, resolução é **união**) |

### 4.3 Mesmo-símbolo por leitura humana (o que a sonda não vê)

- ⛔ `flat_button_surface` — §3.1. **É o único item desta linha que quebra compilação alheia.**
- `App::hovered_object` / `hover_outline` / `pending_ui_sound` / `ui_bool_morphs` — campos novos em
  `app_state.rs`; outra linha a apender ali dá conflito **textual**, resolução por **união**.
- `HeroScreen::ui_sound` — campo apendado; idem.
- `widget/mod.rs` — bloco **gerado**: resolva re-rodando `ph2d-widget-sync`.
- `CLAUDE.md §5` — **uma linha de texto** (DIRETRIZ §1.5.9 item 8). Conflito textual esperado com
  toda outra linha que feche hoje; resolução é **união**, nunca `--ours`.

---

## 5. Contratos congelados encostados (§6)

**NENHUM.** Provado pela sonda (`node.rs` e `tool.rs` **intocados**) e por grep:

```bash
git diff 35f937cb2..HEAD --name-only -- crates/ph2d-nodegraph/src/node.rs \
    crates/ph2d-editor-core/src/tool.rs crates/ph2d-vector-doc/ crates/ph2d-vector-traits/
# (vazio)
```

Nenhum ADR criado ⇒ a linha está **fora** de toda disputa pelo número 0163.

---

## 6. O que só o `ship.sh` pega (o gate de integração NÃO roda)

| Item | Estado desta linha |
|---|---|
| **`cargo fmt`** | ⚠️ **por conferir no ship** — a linha nunca correu `fmt --check` sobre a árvore combinada |
| **`typos`** | ⚠️ idem. Os docs desta linha são **densos e em português**, com nomes próprios (`Mergiraf`, `wireplumber`, `postcard`) — é o candidato mais provável a `✗` |
| **`cargo machete`** | **baixo risco**: `Cargo.lock` sem pacote externo novo; nenhuma dependência acrescentada |
| **`cargo deny` / `audit` (RUSTSEC)** | **sem exposição nova** — zero deps novas. Um `✗` aqui será **pré-fork** |
| **clippy latente** | o gate de fecho correu `--all-targets` sobre **as crates derivadas do diff** (§8). Crates **não tocadas** que compilem contra a API mudada de §3.1 só aparecem no ship |
| **fmt-skew** | ⚠️ os arquivos novos foram escritos com `Edit`, não gerados — mas `project_schema_history.rs` nasceu de um **corte com prova** (`doc-split` não se aplica a `.rs`; foi corte manual com `check` verde) |

---

## 7. Ordem, dependências e o que smokar

### 7.1 Ordem interna (os 19 commits são **sequenciais**, não independentes)

A cadeia de **código** (os 7 commits de docs/memória ficam intercalados e não têm dependência):

`26d3a9c0a` → `872107d73` (os chips) → `9cb694ba8` (a booleana nos estados) → `0c9738316` (a
auditoria que a corrige) → `860effeb4` → `b7302cc83` (a superfície plana e o eixo do chip, que
**preparam** a porta única de §3.1) → `e5f34af8e` → `eec095408` (o realce, e depois a
generalização) → `d652c8ea5` (o pie menu) → `5414ec040` (o bug #27) → `ddfcf5afe` → `998242650` →
`af40484bb` (o som).

⚠️ **`0c9738316` corrige `9cb694ba8`** — fundir o segundo sem o primeiro entrega a wave com os
**dois achados graves** que toda a suíte não via.
⚠️ **`860effeb4`+`b7302cc83` são o pré-requisito de §3.1** — a quebra de API não faz sentido sem
eles, e nenhum dos dois é opcional.

### 7.2 Ordem contra as outras linhas — **recomendação**

1. **`scripts/nextest-impacted.sh` (§3.4) o mais cedo possível** — as outras linhas fecham com a
   versão cega, que pode declarar verde um conjunto que nunca correu.
2. Depois desta linha, **qualquer linha que toque `editor-core/widget/`** paga a migração de §3.1.
3. O `PROJECT_SCHEMA` **conta-se** no fim, com todas as linhas na mão.

### 7.3 O que smokar (tudo abaixo já teve **«smoke OK» do Enio** nesta worktree)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_BUILD_SMOKE=74 cargo run -p ph2d-host-desktop --release
```
A booleana viva **dentro de um estado de UI**: nasce **pronta** (auto-play) e mostra a operação a
**morfar** com os operandos a mover-se. O rig 2 é o **controle** (material idêntico, sem pose).

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && cargo run -p ph2d-host-desktop --release
```
- **Realce de proveniência (C2):** passe o ponteiro sobre **qualquer** objecto, em **qualquer**
  modo — a linha da hierarquia acende **e** a forma ganha contorno, sempre o mesmo objecto.
- **Pie menu (E4):** **segure `P`**; as ferramentas aparecem em oito direcções sob o cursor. É
  **direcção, não distância**; a zona morta central **cancela**.
- **Bug #27:** desenhe uma forma com traço, **Scale não-uniforme** no modo Select — o traço engrossa
  **por igual nos dois eixos** (decisão do Enio) e não vira caneta elíptica.
- **Som de UI (D1):** ⛔ **desligado por omissão**. Ligue com `ui_sound=1` em `~/.ph2d/prefs.txt`.

### 7.4 ⛔ O que NÃO foi smokado / fica em aberto

- **D2 (partículas)** — o último item do estudo de UI viva, **não construído**.
- ⏸️ **Decisões do Enio, já devolvidas com os números:** o `n`/folga do *tether* e o
  `DRAG_RATE_X = 50`; abrir/fechar painel animado (as **secções** já animam — wave F4b).
- **O som de UI não tem interruptor na UI** — hoje só o `~/.ph2d/prefs.txt`. Follow-up nomeado,
  não construído.
- ⛔ **O tempo/ganho das 4 vozes (35–90 ms, 0,16–0,22) foi escrito SEM MEDIÇÃO** (viola
  `CLAUDE.md §0.0`). O gate põe **teto** de duração e **nunca pergunta se são audíveis**. Fica
  como dívida declarada: ou se mede, ou se pergunta ao Enio.

### 7.5 ⛔ A armadilha que custou uma sessão e **não é do código**

O primeiro smoke do D1 deu **silêncio total** com todos os elos internos verdes. O elo partido
estava **fora do processo**: um **mute por-aplicação do PipeWire**, gravado em
`~/.local/state/wireplumber/stream-properties` e indexado pelo **nome da aplicação** —
sobrevive a todo `cargo run`, a builds novos e a árvores novas.

```bash
pactl list sink-inputs | grep -E 'application.name|Mute:'   # com o app ABERTO
pactl set-sink-input-mute <id> 0
```

⚠️ **Já curado nesta máquina** (o ficheiro de estado diz `"mute":false`). Mecanismo completo no
cabeçalho de [`ui_sound.rs`](../../../shells/desktop/src/ui_sound.rs) e em
[`feedback_a_silent_output_channel_can_be_muted_outside_the_process.md`](../../../project-memory/feedback_a_silent_output_channel_can_be_muted_outside_the_process.md).
*Um gate verde sobre um canal mudo continua verde.*

---

## 8. Gate batched de fecho — **VERDE**

Corrido **1× sobre o diff acumulado** (`CLAUDE.md §2`), em **2026-08-23 22:5x**, com a árvore no
HEAD `af40484bb`.

### 8.1 Suíte

```text
CARGO_INCREMENTAL=0 NO_FAIL_FAST=1 bash scripts/nextest-impacted.sh

[nextest-impacted] changed: ph2d-editor-core ph2d-host-desktop ph2d-i18n ph2d-panel-hierarchy
                            ph2d-panel-painter-layers ph2d-panel-vector ph2d-ui-state
                            ph2d-vec-blend ph2d-vec-render
Starting 10617 tests across 785 binaries (1205 tests and 611 binaries skipped)
Summary [27.438s] 10617 tests run: 10617 passed, 1205 skipped
EXIT=0
```

⭐ **Zero falhas — nenhuma das cinco flakes de relógio do `CLAUDE.md §5.0` disparou**, porque a
corrida apanhou a máquina calma (`load` caiu de **25,8** para ~10 entre a sonda e o gate; duas
outras linhas fechavam em paralelo). ⚠️ Se o integrador as vir vermelhas na árvore combinada,
**re-rode-as sozinhas antes de suspeitar da fusão**.

⭐ **A corrida usou o `--no-fail-fast` de §3.4** — é a primeira desta linha que o pôde fazer, e é o
que garante que os 10 617 correram mesmo em vez de pararem na primeira flake.

### 8.2 Clippy

```text
CARGO_INCREMENTAL=0 cargo clippy --all-targets <as 9 crates>
EXIT=0   ·   0 erros   ·   0 avisos
```

⚠️ **O alvo foi DERIVADO do diff**, não escrito à mão
([memória](../../../project-memory/feedback_the_closing_clippy_must_cover_every_crate_the_line_touched.md)) —
e a lista bateu **exactamente** com o `changed:` que o `nextest-impacted` calculou por outro
caminho. Duas derivações independentes, o mesmo conjunto.

### 8.3 Auditoria — o que foi e o que **não** foi

| | |
|---|---|
| **Wave 2 (a booleana nos estados)** | ⭐ auditoria formal de **2 lentes** (CORRECÇÃO + COSTURA DE UI) — **7 achados**, os **dois graves invisíveis a toda a suíte**, todos corrigidos, cada um com o gate que faltava. É a que gerou as 2 memórias novas de método |
| **Waves 1, 4, 5, 6 e o bug #27** | revisão **inline** com gate red-first + prova de mutação por achado, **não** uma auditoria de 2 lentes separada |
| ⛔ **O que isto significa** | a linha **não** teve uma auditoria de 2 lentes sobre os **116 arquivos** de uma só vez. Se o Enio quiser essa cobertura, ela ainda não foi paga — e é honesto dizê-lo aqui em vez de a tabela sugerir o contrário |

### 8.4 Provas de mutação notáveis (o que elas apanharam)

Registadas porque cada uma corrigiu um gate que estava **verde por acidente**:

- `the_base_outlines_what_it_draws` era **tautológico** — num donut `Union` as duas regras
  candidatas dão a mesma área (400). Fixture trocada para `Subtract` (336 vs 400).
- Um gate de receita usava **desigualdade, não oráculo**: um mutante que devolvia a entrada crua
  sobrevivia porque 336 cai dentro da faixa. Passou a comparar com o cozido directo.
- `.is_some()` sobre o `publish` **não distinguia** secção cheia de face vazia. Passou a afirmar
  `published.host.is_some()`.
- Dois arch-gates estavam **ancorados na implementação** e expiraram no mesmo dia em que nasceram
  (`the_curve_is_written_to_the_single_host`, `both_consumers_read_the_one_field`) — re-ancorados
  no **nome da porta**.
- `the_ui_sound_never_follows_the_pointer` media **proximidade de palavras**: um mutante que armava
  som dentro do `pick_hovered_object` sobrevivia. Substituído por duas afirmações precisas.

---

## 9. Estado da worktree ao parar

- Árvore **limpa**, 19 commits à frente de `main`, **nada pushado**.
- `target/*/incremental` **reclamado** (DIRETRIZ §1.5.9 item 7).
- ⛔ A linha **não integra e não faz ship** — aguarda ordem explícita do Enio ([`CLAUDE.md §0.7`](../../../CLAUDE.md)).
