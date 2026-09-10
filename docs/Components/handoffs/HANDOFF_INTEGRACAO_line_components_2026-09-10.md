# HANDOFF DE INTEGRAÇÃO — `line/components`, 2026-09-10

> **A linha FECHA aqui e PARA** ([`CLAUDE.md §0.7`](../../../CLAUDE.md)). Integrar e shipar são
> ordem explícita do Enio, por um **agente integrador dedicado** (DIRETRIZ §1.5.3–1.5.4).

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/components` |
| HEAD | `d74686586` |
| merge-base com `main` | `39d48cd76` |
| commits | **33** |
| ficheiros | **226** (`+30 004 / −7 129`) |
| handoff anterior | [`…PREFAB_ABERTO_2026-09-07`](HANDOFF_INTEGRACAO_line_components_PREFAB_ABERTO_2026-09-07.md) |

**O que a jornada entregou, por bloco** (a narrativa fica aqui; o `§5` recebe UMA linha):

1. **F4.6c + F8** — o segundo motor de instância do vetor SAI (`−5 170` LOC líquidas) e o
   `FlipDoc` passa a ser partilhado.
2. **A REDE do undo** — os verbos tardios produziam um passo fantasma; a cura corre onde a
   fotografia é tirada, e a peça «mais cara» deixou de ter tecto.
3. **TOP-20 #2 `Timer`**, **#5 `SignalActions`**, **#4 `AudioSource2D`/`AudioListener2D`**,
   **#7 `GameCamera`/`CameraFollow`/`CameraLimits`** — cada um com lei pura + ponte + painel.
4. **O selector de VERBO** e a cura do popover (clamp, rolagem, *light-dismiss*).
5. **Dois diagnósticos de áudio** e o `scripts/audio-mudo.sh`.

---

## §2 — Foundational / partilhado tocado, e porquê

| onde | o quê | aditivo? |
|---|---|---|
| `ph2d-ecs` | `timer.rs`, `signal_actions.rs`, `audio_2d.rs`, **`camera_2d.rs`** — módulos IRMÃOS, append-only | ✅ |
| `ph2d-ecs/scene/registry.rs` | **+5 líquidos** contra o `main` (ver §3) | ✅ |
| `ph2d-component-desc` | `catalog/logic.rs`, `catalog/audio.rs`, **`catalog/camera.rs`** + 3 categorias | ✅ |
| `ph2d-editor-core` | `ids/inspector_{timer,action,audio,camera}.rs` · `screens/hero/inspector_model_*` · `action_bus` (+4 variantes) | ✅ |
| `ph2d-panel-inspector` | 4 secções novas + `sync_sections` + o corte do `paint_optional`/`state_popovers`/`popovers` | ⚠️ o corte **move** funções |
| `ph2d-render` / `ph2d-script` | **só o número do espelho do registo** (`80 → 85` contra o `main`) | ✅ |
| `ph2d-ui-testkit` | `set_collapsed` novo + **corte** de `sowing.rs` (235 linhas saem do `lib.rs`) | ⚠️ corte |
| `shells/desktop` | `render_loop/{timer_tick,signal_actions,audio_2d,camera_2d}.rs` + `inspector_{timer,action,audio,camera}.rs` + 4 cenas de smoke | ✅ |
| `scripts/` | **`audio-mudo.sh`** (ficheiro NOVO, nada colide) | ✅ |

⚠️ **Os DOIS cortes são o que uma fusão textual entende pior**: `ph2d-ui-testkit/src/lib.rs` e
`ph2d-panel-inspector/src/paint*.rs` tiveram blocos **movidos** para irmãos. Uma linha que edite
uma dessas funções no sítio antigo funde **limpo** e a edição fica no ficheiro errado, ou evapora.
*Grepe pelo NOME da função, não pelo ficheiro.*

---

## §3 — Símbolos que podem COLIDIR (saída do `collision-surface.sh`, 2026-09-10)

```
SUPERFÍCIE DE COLISÃO — line/components contra main
  merge-base 39d48cd76   ·   32 commit(s)   ·   226 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        124   (base: 123)
  ⚠   └ tripla do gate               (124, 13, 22)   (base: (123, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                              —   (base: —)
  ⚠ ph2d-render (espelho)                  78   (base: 80)     ← LER A NOTA ABAIXO
  ⚠ ph2d-script (espelho)                  78   (base: 80)     ← LER A NOTA ABAIXO
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR   último no disco: 0169   próximo livre: 0170
    esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⚠️⚠️ **A tabela acima foi tirada ANTES do commit `d74686586` e a linha dos espelhos já está
CORRIGIDA na árvore.** Ela lia `78` porque os espelhos estavam **vermelhos** — ver §6. Os valores
que o integrador deve contar são:

⚠️⚠️ **Toda a coluna «base» abaixo saiu de `git show main:<ficheiro>`, e NÃO das notas dentro dos
ficheiros.** A 1.ª redacção deste handoff tirou-a das notas — e **três dos seis deltas saíram
errados**, porque uma nota ao lado de uma conta descreve a população que ela tinha *quando foi
escrita*, e esta linha mexeu nos mesmos contadores duas vezes (a F4.6c **baixou-os** antes de as
quatro waves os subirem). *Escrevi a tabela de memória dentro da secção que existe para avisar
disso.*

| contador | esta linha | base (`main`) | delta | composição |
|---|---:|---:|---:|---|
| `PROJECT_SCHEMA` | **124** | 123 | **+1** | F8 |
| `ph2d-ecs` `reg.len()` | **84** | **79** | **+5** | `−2` F4.6c · `+1` Timers · `+1` SignalActions · `+2` Audio · `+3` Camera |
| espelhos `ph2d-render` / `ph2d-script` | **85** | **80** | **+5** | a mesma conta, `+1` da Sprite nos dois lados |
| `LIVE_SECTIONS` | **20** | 16 | **+4** | Timer · Action · Audio · Camera |
| `ComponentCategory::ALL` | **16** | **13** | **+3** | `Logic` · `Audio` · `Camera` |
| `any_live_section` | `[bool; 15]` | `[bool; 11]` | **+4** | uma por secção nova |

⛔ **CONTE O DELTA, NUNCA O LITERAL.** Duas linhas que registem um componente cada e escrevam o
mesmo número fundem **mudas** — o git não sabe o que a conta significa.

⚠️ **E `+5` não é `+7`**: as notas dentro do `ph2d-ecs` e dos espelhos falam de `77`/`78`, que é o
estado **intermédio** desta linha depois da F4.6c. Contra o `main` a conta é `+5`.

**Símbolos novos que uma outra linha pode ter escolhido:**
`ph2d::ecs::{Timers, TimerRuntime, SignalActions, AudioSource2D, AudioListener2D, GameCamera,
CameraFollow, CameraLimits, CameraRuntime}` · `ComponentCategory::{Logic, Audio, Camera}` ·
`EditorAction::Inspector{Timer,Action,Audio,Camera}Edit` · `SignalVerb::{PlaySound, StopSound}`
(⚠️ **apendados no fim** — a posição É a tag) · os ids `INSP_{TIMER,ACTION,AUDIO,CAMERA}_*`.

---

## §4 — Contratos congelados

**NENHUM.** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` intocados (confirmado
pelo `collision-surface.sh`). Nenhum ADR criado ⇒ fora de toda disputa de número.

---

## §5 — O que só o `ship.sh` apanha

- **`scripts/audio-mudo.sh` é ficheiro NOVO** — nunca passou por `shellcheck`/`typos` do CI.
- **Zero dependências externas novas** (`Cargo.lock` sem `+name`) ⇒ nada para o `machete`/`deny`.
- **45 fixturas `.csv` novas** em `ph2d-ecs/tests/fixtures/camera2d_godot/` (**540 KB**) mais o
  `oraculo.gd` e um `README.md`. ⚠️ O `README.md` **não** entra no `doc-index.sh` (não é um dos 14
  directórios), mas o `typos` lê-o.
- **`fmt` corrido em toda a árvore** no fecho; clippy `--all-targets` a zero nas crates tocadas.

---

## §6 — ⛔⛔ O VERMELHO QUE O FECHO APANHOU, e ele estava vivo há TRÊS waves

O `collision-surface.sh` imprimiu **`espelho 78 (base: 80)`** — uma coluna a **DESCER** numa linha
que, no líquido, ACRESCENTOU cinco componentes. Corrido, o gate deu `left: 85 · right: 78`.

Os TOP-20 **#2**, **#4** e **#7** entraram no registo do `ph2d-ecs` sem que ninguém contasse nos
espelhos do `ph2d-render` e do `ph2d-script`.

⛔ **A causa é de PROCESSO:** cada um dos três fechos correu `cargo test -p` sobre **as crates que
a wave editou**, e este gate vive em duas que nenhuma das três tocou. *Um portão que só corre o que
a linha editou é cego a todo espelho.* — já registado na memória
([`-p <crate> sozinho`](../../../project-memory/feedback_testing_a_crate_alone_hides_every_defect_in_a_feature_the_shell_enables.md),
[`filtro de nome`](../../../project-memory/feedback_a_closing_run_with_a_name_filter_never_reaches_a_tree_scanning_gate.md)),
e esta é a **terceira** ocorrência.

⚠️ **E o `85` não é `84 + 1` por acaso:** os espelhos contam o registo do ECS **mais a Sprite**
(`registers_sprite_alongside_ecs`), e a base `78` era `77 + 1` pela mesma conta.

---

## §7 — Ordem, dependências e o que SMOKAR

**Os commits são sequenciais e não têm ordem alternativa** — cada wave (lei → ponte → painel)
depende da anterior, e as duas curas de popover (`864ebc83c`, `43c7bb6a9`, `fc5f133e6`) entram
entre o painel do `SignalActions` e o do áudio porque foram um report do dono a meio.

**Já smokado e APROVADO pelo Enio** (nesta ordem, cada um com o veredito dele):

| cena | comando | veredito |
|---|---|---|
| `SignalActions` | `PH2D_SIGNAL_ACTION_SMOKE=1` | ✅ |
| som de cena | `PH2D_AUDIO_2D_SMOKE=1` | ✅ (⚠️ ver §8) |
| câmera de jogo | `PH2D_GAME_CAMERA_SMOKE=1` | ✅ (4 reports, todos curados) |
| o painel da câmera | idem, clicando em **Camera** | ✅ |
| o *lookahead* | idem, `Lookahead = 0,5` | ✅ |

**⏳ O que NÃO foi smokado:**
- **`PH2D_TIMER_SMOKE=1`** — a cena existe e o dono nunca a correu isolada (ele viu o `Timer` a
  funcionar **dentro** da cena do som, que é onde o relógio dispara a sirene).
- **A máscara de camadas da câmera** (`Cull Mask`) — pintada, registada e com gate de alcance, e
  **sem consumidor medido no produto**: o `cull_mask` chega à `Camera2d` da shell, mas nenhuma cena
  desta linha tem duas `VisibilityLayer` distintas para o provar a olho.
- **Carregar um `.ph2dproj` gravado antes desta linha** — o `PROJECT_SCHEMA` subiu `123 → 124`.

---

## §8 — ⚠️ SETE coisas que uma leitura rápida do diff entende ao contrário

1. **`CameraRuntime` não estar no registo NÃO é esquecimento — é a cerca.** Ele não deriva
   `Serialize`, então não *cabe* no registo, e há gate a afirmar a ausência. Registá-lo faria cada
   quadro com clique, numa cena a seguir o jogador, virar um passo de undo.
2. **`aim_at` perdeu o parâmetro `dt`, e isso é a CURA, não uma simplificação.** Ele fazia dois
   trabalhos — estimar velocidade e aplicá-la — e o `dt` do primeiro **não era** o do segundo. Hoje
   são três funções (`sample_velocity` · `smooth_velocity` · `aim_at`).
3. **A constante de tempo da suavização é `1/lookahead` e NÃO é um número inventado**: perguntar
   «onde estará daqui a `L`?» e estimar numa janela mais curta que `L` é medir ruído.
4. **O herói da cena de smoke anda pelo TECLADO e não pelo rato de propósito.** Um arrasto ancora o
   objecto na posição de mundo sob o cursor, que deriva da câmera ⇒ com uma câmera que o segue, a
   cadeia fecha-se sobre si mesma. Arrastar **ainda trepida**, e isso é **declarado**.
5. **`inspector_queue_dirty` era `audio_commit`.** Desde a secção CAMERA a bandeira serve duas
   secções; o nome antigo descrevia metade do que ela significa.
6. **A `Preview`/`Look Through` viaja pelo `EditorAction` e NÃO escreve no documento.** Ela é estado
   de VISTA (o molde é o «Show sheet on canvas»); viaja por ali porque é ali que o painel fala com
   a shell, e a shell é quem tem a vista.
7. **Os `.csv` do `camera2d_godot` não são golden nossos: são saída do ORÁCULO.** Regerá-los exige
   o Godot instalado (`oraculo.gd` ao lado, com o comando no `README`), e o gate confere o
   **cabeçalho** contra a tabela do teste — um corpus regerado com outros parâmetros reprova em vez
   de testar outra lei em silêncio.

---

## §9 — ⚠️ SEIS premissas minhas que a medição derrubou

1. *«O oráculo é uma referência a ler.»* — **O Godot 4.7.2 é MIT e está instalado**, então a
   triagem parou na porta aberta: porta-se **correndo**, e cada corrida virou gate. A 1.ª sonda deu
   `75,33` de trilho onde a verdade é `115,2`, porque o `make_current()` falhava em silêncio.
2. *«A zona morta trabalha sobre o centro suavizado.»* — **Falso, e são DOIS estados.** Com um só,
   o degrau e a rampa passam e o **vaivém** erra `1,066284 px`: o defeito aparece exactamente
   quando o alvo inverte.
3. *«O `f32::clamp` chega para os limites.»* — Ele **entra em pânico** com `min > max`, que é
   precisamente uma sala mais estreita que o ecrã. O oráculo fixa no **centro da caixa** (`350,000000`).
4. *«O amortecimento do oráculo é seguro.»* — Ele **oscila** a `speed·dt = 2` e **diverge** a `3,33`.
   Divergência declarada, com a tabela dele dentro.
5. *«O `wall_dt` serve para mover o herói.»* — **Não**: ele e a câmera correm em relógios
   diferentes, e a diferença entre dois relógios é DESENHADA como tremor (`~14 px` a 60 Hz).
6. *«A antecipação estava só com um pico.»* — Eram **três** defeitos, e o terceiro (um quadro sem
   tique a zerar a velocidade, `0,78 m/s` sobre `8,00`) **mascarava** o primeiro: a mutação do
   `sample_dt` **sobreviveu** enquanto ele esteve vivo.

---

## §10 — ⏳ O que fica ABERTO (para a linha do §5)

- **`Preview` do áudio não tem indicador de «está a tocar»**.
- **A lista de delegação de a11y não tem censo de obsolescência** — um `rename` órfão foi apanhado
  por acaso; uma **remoção** apodreceria calada.
- **`AudioSource2D.sound` é um CAMINHO** — mover o ficheiro parte o som, e o projecto não embute o
  áudio. A cura é o áudio no índice de assets, wave própria. Não há `FieldKind::File`.
- **O relógio do som é o de PAREDE** — um scrub da timeline não rebobina um som.
- **`Cull Mask` sem consumidor medido** (ver §7).
- **A antecipação não tem suavização SEPARÁVEL** — a constante de tempo é derivada do `lookahead`,
  e não há knob. Se algum dia o dono quiser as duas coisas independentes, é um campo novo.
- **TOP-20 seguinte: #9 `Tags`.**

---

## §11 — A linha do `CLAUDE.md §5`, PRONTA A COLAR

⚠️ **Esta linha NÃO edita o `CLAUDE.md`**, e é deliberado: o próprio §5 escreve que *«ele só se
edita na integração»*, e cinco linhas a fechar no mesmo dia colidiriam ali com certeza. O
integrador cola isto na entrada **Componentes / instâncias**, na posição `Aberto:`:

> ⭐⭐⭐ **O TOP-20 anda até ao #7** (10/09): **#2 `Timer`**, **#5 `SignalActions`**, **#4
> `AudioSource2D` + `AudioListener2D`** e **#7 `GameCamera` + `CameraFollow` + `CameraLimits`**,
> cada um com lei pura, ponte e **painel** — mais a **F4.6c** (o 2.º motor de instância do vetor
> sai, `−5 170` LOC) e a **F8** (o `FlipDoc` partilhado). ⭐ **A lei da câmera é PORTADA do Godot
> 4.7.2 (MIT) corrido como oráculo sem interface** — 45 fixturas com cabeçalho, paridade de
> `0,000061 px`, com a **divergência declarada** onde ele oscila e diverge. ⚠️ **Três leis que só a
> medição deu:** a ordem *zona morta → amortecimento* · a zona morta ter **acumulador próprio** (um
> estado só erra `1,07 px` em toda inversão do alvo) · e uma cerca mais estreita que a janela
> **fixar no centro dela** (onde o `f32::clamp` entra em pânico). ⛔⛔ **E a auditoria do
> *lookahead* achou TRÊS defeitos, com o terceiro a MASCARAR o primeiro** — o `dt` da amostra ≠ o
> passo da lei (a mira dobrava em toda moldura de 2 tiques) · parar colapsava a mira em `4 m` num
> quadro · e **um quadro sem tique zerava a velocidade** (`0,78 m/s` sobre `8,00`, dez vezes
> fraca). ⚠️ **E o painel não era semeado do snapshot — nem o do ÁUDIO, desde a wave dele**:
> mostrava os defaults do `populate` com números plausíveis. ⏳ **ABERTO** e as **seis** premissas
> refutadas: [handoff de 10/09](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_2026-09-10.md).
> **Smokes:** `PH2D_TIMER_SMOKE=1` · `PH2D_SIGNAL_ACTION_SMOKE=1` · `PH2D_AUDIO_2D_SMOKE=1` ·
> `PH2D_GAME_CAMERA_SMOKE=1`. ⚠️ *«meters mexem e não ouço»* é o **sistema**, não o app:
> `bash scripts/audio-mudo.sh`.

---

## §12 — Portão de fecho (batched, 1× sobre o diff acumulado)

| passo | resultado |
|---|---|
| `cargo fmt --all` | árvore limpa |
| `cargo clippy --all-targets` nas crates tocadas | **zero** avisos |
| **`scripts/nextest-impacted.sh`** sobre o diff acumulado | ⭐ **`14 537` testes, `14 537` verdes, `1 373` saltados, `exit 0`** (53,7 s) |
| `ph2d-render` + `ph2d-script` corridas **explicitamente** | verdes (§6) |
| `target/*/incremental` | reclamado (DIRETRIZ §1.5.9 item 7) |

⚠️ **A corrida do `nextest-impacted` foi feita com a máquina a `load 12,6`** — irrelevante para
gates de correcção, e a nota fica porque **é** relevante para os de razão: se algum da família de
flakes de recurso reprovar na árvore combinada, re-corra-o sozinho **com o `/proc/loadavg` ao
lado** antes de olhar para este diff (`CLAUDE.md §5.0`).
