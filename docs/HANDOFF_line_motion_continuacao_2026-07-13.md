# HANDOFF — `line/motion-value`: continuação dos Motion Nodes (2026-07-13)

> **Para o agente que assume a linha.** A jornada anterior fechou e **foi integrada ao `main`**
> (35 commits, 17 crates-nó novas). Este documento é o seu briefing completo: como se trabalha
> aqui, o que existe de verdade, e a fila.
>
> **Leia antes de tocar em qualquer coisa:**
> - [`GUIA_JORNADA_MODO_L.md`](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) — como a jornada roda
> - [`DIRETRIZ.md §1.5`](IntegracaoMultiAgente/DIRETRIZ.md) — o protocolo que **você** segue
> - [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) — a cada passo
> - [`CLAUDE.md`](../CLAUDE.md) — os 7 inegociáveis

---

# PARTE I — O MODO DE TRABALHAR (Modo L)

## 1. O que é, em uma frase

Você é **uma linha autônoma** numa worktree própria. Trabalha sozinho, fecha o gate, escreve
o handoff e **PARA**. Não há Coordenador. **Integração e ship são ordem EXPLÍCITA do Enio**,
executados por um agente integrador dedicado — nunca por você.

## 2. As 6 regras que, se você quebrar, quebrou o protocolo

1. **Tudo dentro da worktree.** Ler, editar, `git`, `cargo` — tudo em
   `Worktrees/line-motion-value/`. O **mesmo caminho relativo existe no repo primário**, e
   editar lá é editar a árvore ERRADA. **Mutação de arquivo = SEMPRE caminho absoluto.** Na
   dúvida: `pwd`. (Isto me pegou nesta linha. Um `sed -i` relativo escreve no primário.)
2. **Foundational você PODE tocar** (ADR-0107) — mas **contrato congelado ([CLAUDE.md §6])
   = PARE e reporte ao Enio** (exige ADR). Os congelados que te cercam:
   `NodeOp=2` / `OpResolver=1` / `NodeManifest=8`. Gate: `architecture_contract_surface`.
   *(O `EvalCtx` NÃO faz parte da superfície congelada — dá para estender.)*
3. **Ao CRIAR foundational, projete para ISOLAMENTO** — módulo irmão, ponto de extensão
   append-only. Anote todo id/const/variante novo no handoff.
4. **`git commit --no-verify`. NUNCA `push`. NUNCA `--force`. NUNCA `git add -A`.**
5. **Conflito em `Cargo.lock` ou `ph2d-node-registry-init`: REGENERE**, nunca resolva à mão:
   `cargo run -p ph2d-node-sync`.
6. **Você NÃO integra e NÃO shipa.** Fecha a fatia → gate → handoff → **PARA**.

## 3. Como você abre a sua linha

O repo primário está em `main`, limpo. **A worktree JÁ EXISTE** e está **limpa, 0 à frente /
104 atrás** do `main`. Reaproveite (o `target/` está quente — build incremental, muito mais
barato que do zero):

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value
git fetch origin 2>/dev/null; git checkout line/motion-value
git reset --hard main          # 0 commits seus a perder — a linha foi INTEGRADA
git log --oneline -1           # deve casar com o main
```

Confirme que está no lugar certo antes do primeiro `cargo`:
```bash
pwd   # .../Worktrees/line-motion-value  ← se disser outra coisa, PARE
```

## 4. O ciclo de trabalho

```bash
# INNER LOOP — só isto, o dia inteiro:
cargo check -p <crate>

# 1× NO FECHAMENTO da fatia (não por task):
cargo test -p <crates tocadas>
cargo clippy -p <crates> --all-targets -- -D warnings
rustup run 1.95 cargo fmt -p <crates>          # o toolchain PINADO, não o plain
```

Fast mode: `git commit --no-verify` (instantâneo), **zero push, zero CI**.

## 5. Como se prova uma coisa aqui (NÃO é negociável)

**Verde-de-compilação é velocidade; no audit vale ZERO.** O método que o Enio ratificou nesta
linha, e que você deve seguir em cada fatia:

1. **Pesquise o algoritmo padrão-ouro ANTES de codar** — e cite-o pelo nome (Schneider, Wang,
   Bridson, Ephraim-Malah…). Não invente o que a indústria já resolveu.
2. **HR-5:** zero transcendental em produção (`sqrt` é permitido).
3. **Demo auto-tocável no documento de boot** (`shells/desktop/src/motion_demo_strobe.rs`) —
   o Enio não deve ter que montar um grafo para ver a sua feature.
4. **Guards FALSIFICÁVEIS**, provando a cadeia inteira. Depois **MUTE o código** e confirme
   que o guard fica **VERMELHO**. Um guard que não sabe falhar não prova nada.
5. **Nota-ADR numerada** em `docs/Motion Nodes/` (a próxima é a **57**).
6. Gate em lote → commit → **PARA**. O Enio faz o smoke.

**Meça pelo EXIT CODE, nunca pelo texto.** `cmd | grep …` faz o `$?` virar o do `grep`:
```bash
cargo fmt --all -- --check > /dev/null 2>&1; echo "exit: $?"   # 0 = limpo
```
Isto me mordeu **nesta linha, no fechamento**: um `&& echo "fmt OK"` imprimiu OK enquanto o
fmt falhava.

## 6. ⚠️ As armadilhas que já custaram caro NESTA linha

| Armadilha | O que aconteceu |
|---|---|
| **`cargo check -p` verde por 33 commits** | escondeu duas bombas que só o gate de fechamento viu: uma crate **desformatada** e um arquivo em **604 > 600 LOC**. Ambas teriam feito o CI vermelho na mão do integrador. |
| **Dois caps de LOC diferentes** | o do **workspace** é 700 (`architecture_workspace_file_loc_cap`); o do **shell** é **600** (`shells/desktop/tests/file_loc_caps.rs`). Rode os DOIS. LOC estourado = **split em módulo irmão**, NUNCA allowlist. |
| **Um filtro dentro de um gate é um buraco nele** | o guard `every_row_range_contains_its_value_for_every_node_and_param` estava **verde** porque filtrava `.starts_with("motion.")` — e os nós novos são `sim.*`. O nome prometia "every node". Pergunte sempre **sobre o quê** um gate está verde. |
| **`python` `str.replace()` que não casa é no-op SILENCIOSO** | o `fmt` reflowa o texto entre edições e a âncora deixa de casar; o teste antigo passa e você comemora. Use `assert old in s`, ou o `Edit` tool. |
| **`perl`/`sed` com literal acentuado** | corrompe o arquivo em mojibake. Texto em pt-BR só via `Edit`. |
| **`git checkout -- <arquivo>`** | reverteu edições minhas NÃO commitadas. Para desfazer mutação de teste, use `cp` de um backup. |

## 7. Comunicação com o Enio

pt-BR, direto, **decisão primeiro**. Ele é o dono/decisor; a LLM é o único dev. Ele **não lê
código** — ele faz o **smoke visual**. Então:

- Toda fatia termina com o comando pronto, **com o `cd` junto**:
  ```
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
  ```
- **Decida, não pergunte.** Escolha o padrão-ouro e execute; reporte a decisão.
- Quando ele diz *"difícil de ajustar"* / *"não faz direito"* → **é bug de DESIGN**, não de
  calibração. Questione o modelo, não os números.
- Cadência: ele diz **"próxima"** e **você escolhe** a próxima fatia da fila.

---

# PARTE II — O ESTADO REAL (verificado agora, não de memória)

## 8. Onde está o quê

| | |
|---|---|
| `main` | `2c25d716` — contém TUDO da jornada anterior (rebase; SHAs mudaram) |
| Worktree | `Worktrees/line-motion-value` — limpa, 104 atrás. `git reset --hard main` |
| Nós | **88 crates** `ph2d-node-*` |
| Docs | `docs/Motion Nodes/00..56` — a próxima nota-ADR é a **57** |
| Plano-mestre | [`docs/Motion Nodes/01_plano_modulo_motion_nodes.md`](Motion%20Nodes/01_plano_modulo_motion_nodes.md) §3 (roadmap M0..M5) |
| Suíte | `cargo test -p ph2d-host-desktop --bins` → **427 verdes** no `main` |

> ⚠️ **O `target/` do repo PRIMÁRIO está quebrado** (`Not a directory`). Se precisar rodar algo
> no primário, exporte `CARGO_TARGET_DIR` para um diretório temporário. Na worktree é normal.

## 9. O que os Motion Nodes JÁ são (não reconstrua nada)

- **Editor de nós completo (F2/F3):** backdrops (grupos que carregam o que emolduram), faca,
  probe + sparkline, smart-connect, Ctrl+D, waypoints/reroute, readouts inline, véu nos nós
  inertes, marcha nos fios vivos, largura ∝ massa, *postage stamp* por card, add-menu com
  scroll e barra arrastável. Pan = botão do MEIO; box-select = esquerdo.
- **Zona de Simulação (O4) fechada:** `sim.zone` (estado vivo entre ticks) · `sim.step` ·
  `sim.spawn` · `sim.lifetime` · `sim.collide`. O doc de boot é a **chuva** — nasce, acelera,
  envelhece, desvanece, colide, assenta.
- **M4:** rig (`skeleton`/`fk`/`ik_2bone`/`fabrik`/`rubber_hose`/`skin_deformer`) + FX
  por-instância (`fx.rgb_split`, `fx.drop_shadow`, `motion.mirror`).
- **Persistência:** o grafo entra no projeto (Ctrl+S / Ctrl+O). `PROJECT_SCHEMA = 3`.
- **Perf:** o custo da Vello é **por objeto de desenho**, não por vértice (~5000 → ~250/quadro).

## 10. 🔴 O que MUDOU debaixo de você na integração (leia isto)

O integrador fundiu a linha `anim`, que trouxe o **W4.T7 — relógio único**:

> **O `MotionTransport` MORREU.** O Motion não tem mais relógio próprio: ele cozinha no tick
> do **`ph2d_core::Playhead`** global (`motion_bridge::ticks_owed` / `motion_tick`).

Consequências práticas que você **vai** encontrar:

- `MotionState::install()` (o load de projeto) **não reseta mais o transporte** — não há um.
  Está correto: o pump novo tem `last_cooked_tick = None`, o playhead global manda, e a sim
  se re-deriva pelo caminho de **scrub** (M2.N2). O guard
  `a_loaded_document_cooks_exactly_like_a_freshly_booted_one` cobre isso e **passa**.
- Qualquer teste seu que queira rodar a sim precisa **construir um `Playhead`**, não um
  transporte. O molde já está pronto em `motion_state_tests.rs::run()` — **copie de lá.**
- Duas ratoeiras que o helper de teste comeu antes de virar guard, e que continuam valendo:
  1. **O shell cozinha o tick 0 ANTES de avançar** (o cook de catch-up do quadro pausado).
     Avançar primeiro joga o pump no caminho de scrub com o anel vazio e **não sai nada**.
  2. **O relógio nasce PAUSADO** e `advance` é no-op enquanto estiver. Sem `play()`, o tick
     nunca sai de 0 e você mede um cook vazio.

## 11. Pendências de smoke (o Enio ainda não validou)

- **`d9cbc10b` — fix dos sliders** (a régua era função do valor: chegava a bilhões). Confira
  no **Collider**: onde havia um slider numérico agora devem estar 3 botões (Floor/Disc/Bowl).
- **`36bdb80a` — persistência do grafo** (Ctrl+S / Ctrl+O).

Se ele reportar problema em algum, **isso vira a FILA 0** e passa na frente de tudo.

---

# PARTE III — A FILA DE IMPLEMENTAÇÃO

> Derivada do plano-mestre §3 cruzado com o que **de fato existe** no repo (não de memória).
> A ordem é minha recomendação; **o Enio pode reordenar** — ele manda.

### FILA 1 — 🟢 **Subgrafo + breadcrumb** (M4 editor) — *comece por aqui*

**Não existe** (`grep` por `Subgraph`/`breadcrumb`: zero). É o maior buraco de **usabilidade**
do editor hoje: com **88 nós** na biblioteca e a chuva já gastando 19 cards, um grafo real vira
uma parede. Todo editor de nós sério resolve isto com nesting, e nós temos **zero**.

- Um grupo de nós colapsa num **card único**; duplo-clique entra; **breadcrumb** no topo sai.
- **Padrão-ouro a pesquisar antes de codar:** Houdini (subnet/HDA) · Nuke (Group/Gizmo) ·
  Blender (Node Group + interface de sockets). Decida **onde o subgrafo mora**: um `Graph`
  aninhado no `MotionDoc`, ou um nó cujo `eval` cozinha um sub-grafo?
- ⚠️ **O contrato de nós é CONGELADO.** Se a sua solução exigir mexer em `NodeManifest`/`NodeOp`
  → **PARE e reporte ao Enio** (exige ADR). Antes disso, procure o caminho que **não** exige:
  o precedente vivo é o **canal de TEXT PARAM** (doc 32), que deu a `motion.expression` sem
  tocar o contrato — os params moram no `Graph`, não no `NodeManifest`. **Esse é o padrão.**
- O `[backdrop]` já existe no doc e no formato textual: é um precedente de como estender o
  `MotionDoc` de forma aditiva.

### FILA 2 — 🟢 **Promoção param → socket** (M4 editor)

Também **não existe**. Hoje um param é ou um literal (o slider) ou nada — não dá para dizer
*"este raio é dirigido por aquele `value.lfo`"* sem um nó adaptador. A promoção transforma o
param numa **porta de entrada dinâmica**.

- Mesma restrição de contrato da FILA 1 — e a mesma saída: procure o caminho aditivo primeiro.
- Semântica de precedência a decidir e **documentar**: `socket > keyframe > literal` (o plano
  já antecipa isso em M4.N1, pensando na timeline futura).

### FILA 3 — 🟡 **FX passes no compositor HDR** (M4, o resto)

Faltam `fx-glow` · `fx-bloom` · `fx-blur` (dual-Kawase) · `fx-vignette` · `fx-levels` ·
`fx-hue-shift`. **Atenção — estes NÃO são nós por-instância** como o `rgb_split`: o plano diz
*"no compositor HDR; `layer_fx` no documento"*. É outra arquitetura.

- **Reuso obrigatório:** o Painter **já tem** um compositor GPU 22-modos + adjustment layers
  (`ph2d-painter-effects`). Antes de escrever um shader, vá ver o que dá para reaproveitar.
  Escrever um segundo bloom é dívida, não feature.

### FILA 4 — 🟡 **Os 4 nós que faltam do plano**

`motion-delay` · `motion-buoyancy` · `motion-distribute-poisson` (Bridson) ·
`motion-path` (integra `vector.*`).

Fan-out puro, baixo risco, cada um é uma fatia pequena. **`motion-path` é o mais valioso**
(casa Motion com o módulo Vector). Bom trabalho de aquecimento se você quiser calibrar o ciclo
antes de encarar a FILA 1.

### FILA 5 — 🔵 **W4.T4 — dock da timeline no `motion_timeline_slot`**

O slot **existe** em `screens/layout.rs` mas nasce com altura 0 (encaixe deferido). O
bloqueio era *"espere a linha Motion fechar"* — **ela fechou**, e o relógio agora é único
(§10), que era o pré-requisito real.

⚠️ **Coordene com o Enio antes**: isto encosta na **linha da timeline**. Duas linhas no mesmo
módulo é proibido (GUIA §1). Pode ser que ele prefira dar isso à linha `anim`.

### 🚫 FORA DA SUA LINHA (não pegue)

- **GPU / M5** (`ph2d-motion-gpu`, CookPlan, kernel fusion) — é uma **linha foundational
  dedicada**, com [plano próprio](plans/2026-07-gpu-resident-node-pipeline.md). **NUNCA
  enxertada numa linha de fan-out.** Se você tocar nisso, quebrou o protocolo.
- **Keyframes do Motion** — **deferidos** até a integração da timeline (decisão registrada em
  [memória](../project-memory/project_motion_keyframes_deferred_timeline_integration.md)).
  A pesquisa pré-implementação está preservada; não a refaça.
- **Contrato congelado** (`NodeManifest`/`NodeOp`/`OpResolver`) — só com ADR, ordem do Enio.

---

## 12. Quando você fechar

1. Gate em lote (os DOIS caps de LOC, clippy `-D warnings`, fmt com o toolchain **pinado**).
2. Nota-ADR em `docs/Motion Nodes/57_*.md`.
3. **Handoff de integração** (DIRETRIZ §1.5.9): branch/HEAD/base · **foundational tocado** ·
   crates novas · superfícies de colisão · o que só o `ship.sh` pega.
4. **PARE.** Reporte "linha pronta + handoff" e espere a ordem do Enio.

**Conte com 2–4 iterações de vermelho no `ship.sh`** (não é você: o gate per-linha não roda
`machete`, `deny`, `audit` nem `typos` — e o `typos` tem allowlist pt-BR própria).
