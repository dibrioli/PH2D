# HANDOFF DE INTEGRAÇÃO — `line/motion-value` · 2026-08-22

> Entregável que fecha a linha ([DIRETRIZ §1.5.9](../../IntegracaoMultiAgente/DIRETRIZ.md)).
> ⛔ **A linha NÃO integra e NÃO pusha.** Isto é para o **agente integrador**, por ordem do Enio.

---

## 1 — Identidade

| | |
|---|---|
| Branch | `line/motion-value` |
| HEAD | `b47b3b45b` |
| Base do fork (merge-base com `main`) | `ee1432203` |
| Commits | **68** |
| Diff | 347 arquivos · +45.850 / −5.342 |

---

## 2 — Foundational / compartilhado tocado

⚠️ **A esmagadora maioria é ADITIVA** (arquivo novo). O que importa ao integrador é o que foi
**MODIFICADO**, porque é só aí que há conflito de texto.

| área | novos | modificados | natureza |
|---|---|---|---|
| `crates/ph2d-node-*` | **2 crates novas** (`ph2d-node-field-shape`, `ph2d-node-value-cursor`) | ~40 crates de nó | a crate de um nó é **drop-crate** (ADR-0075): risco de colisão ~zero, exceto pelo registry gerado (item 3) |
| `shells/desktop/src` | **70 arquivos** | ~30 | as cenas de smoke e os `*_tests.rs`; os modificados são os **roteadores** (item 3) |
| `crates/ph2d-fbm` | — | 2 | `natural_range` + `gain_offset_for_range` — **API nova, aditiva**; nenhum chamador existente muda |
| `crates/ph2d-eval-motion`, `ph2d-gpu-cook` | — | 4 | as **duas rotas de lowering** aprenderam a coluna `blend` (item 5) |
| `crates/ph2d-motion-diagnose` | — | 3 | diagnósticos novos, aditivos |
| `crates/ph2d-vec-*`, `ph2d-render` | — | ~13 | correções pontuais (o assador de tiles que vazava textura; a rosca cortada) |

⚠️ **Nenhum arquivo da linha passa do teto de LOC** (verificado por `collision-surface.sh`).

---

## 3 — Símbolos que podem COLIDIR (o que grepar)

### 3.1 ⚠️ **O maior risco desta linha: 23 NÚMEROS DE CENA**

A base do fork tem o roteador de smoke até a cena **59**. Esta linha reclama **60 a 82**, e sobe
`MAX_DEMO_LEVEL` de 59 para **82** em
[`motion_state_demo_router.rs`](../../../shells/desktop/src/motion_state_demo_router.rs).

⚠️ **É o caso-tipo do «número que soma entre linhas»** (CLAUDE.md §5.0): o roteador é uma lista de
braços e **o PRIMEIRO vence** — dois braços com o mesmo número deixam o segundo inalcançável **em
silêncio**, que foi como a cena dos tokens da `line/Vector` sumiu em 2026-08-02. O git não sabe o
que o número significa: se outra linha também reclamou `=60`, o merge sai **limpo e errado**.

**O que fazer:** `git grep -nE 'Some\("(6[0-9]|7[0-9]|8[0-2])"\)' -- shells/desktop/src/motion_state_demo_router.rs`
em CADA worktree antes de fundir. O gate `no_two_smoke_scenes_claim_the_same_level` apanha o
duplicado **depois** de fundido; grepar antes evita a renumeração em cascata.

### 3.2 O registry de nós é GERADO — duas entradas novas

`ph2d-node-field-shape` e `ph2d-node-value-cursor` entram em
`crates/ph2d-node-registry-init/src/lib.rs` e no `Cargo.toml` dele, **entre marcadores**, por
`cargo run -p ph2d-node-sync`. O gate `staleness` reprova se a lista divergir da pasta.

⚠️ **Se outra linha também criou crate de nó, o bloco entre marcadores conflita.** A cura **não é
resolver o texto à mão** — é fundir as duas listas e **re-rodar o `node-sync`**, que é a fonte.

### 3.3 `Cargo.lock`

**2 pacotes novos, os DOIS internos** (as crates acima). ⚠️ **Nenhuma dependência externa nova** —
o que baixa muito o risco de `machete`/`deny`/`audit` no ship (item 5).

### 3.4 Saída literal do `collision-surface.sh`

```
SUPERFÍCIE DE COLISÃO — line/motion-value contra main
  merge-base ee1432203   ·   68 commit(s)   ·   347 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                         84   (base: 84)
      └ tripla do gate               (84, 13, 14)   (base: (84, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                               57   (base: 57)
    ph2d-render (espelho)                  58   (base: 58)
    ph2d-script (espelho)                  58   (base: 58)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR
    último no disco: 0159   próximo livre: 0160
    esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock
  ⚠ 2 pacote(s) '+name' novo(s):  "ph2d-node-field-shape"  "ph2d-node-value-cursor"
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⚠️ **PRAZO DE VALIDADE:** esta tabela mede a linha contra o `main` de **2026-08-22**. Se outra
linha fundir antes desta, **toda a coluna «base» muda e a tabela não reclama**. O integrador
**RE-RODA** `collision-surface.sh` em cada worktree imediatamente antes de fundir; a divergência
entre as duas leituras é ela própria um achado.

---

## 4 — Contratos congelados encostados

**NENHUM.** Verificado pelo `collision-surface.sh`: `ph2d-nodegraph/src/node.rs` e
`ph2d-editor-core/src/tool.rs` **intocados**.

⚠️ **E isso é uma escolha de desenho, não sorte.** Todo canal novo desta linha —
`ParamGate`/`ParamGateAbove`, `ParamHardMax`, `Coupling`, os hints — é **side-metadata no
REGISTRY**, registado por uma chamada `reg.register_*` na `fn register` da própria crate. O
`NodeManifest` continua nos 8 campos congelados (§6). **Nenhum ADR é necessário.**

---

## 5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

| risco | leitura desta linha |
|---|---|
| `machete` / `deny` / `audit` | **baixo** — zero dependências externas novas |
| `typos` | não corrido nesta linha; os docs novos são densos em português com termos técnicos |
| `clippy --all-targets --all-features` | corrido **limpo** em `ph2d-host-desktop` e `ph2d-node-registry-init`; ⚠️ **não** corrido na workspace inteira |
| `fmt` | `cargo fmt --all` corrido no fecho |
| drift pré-fork | a base é de **2026-08-22 de manhã**; quanto mais tarde a integração, mais latente ([[project_integration_prefork_lines_ship_drift]]) |

⚠️ **Orce 2–4 iterações de ship** — é a mediana medida deste repositório
([[project_integrator_ship_catches_latents_budget_iterations]]).

---

## 6 — Ordem, dependências e o que SMOKAR

### 6.1 Ordem

Os 68 commits são **sequenciais e independentes de outras linhas**. Nenhum depende de trabalho
fora daqui. ⚠️ Os **quatro últimos** formam uma unidade e devem viajar juntos:

| commit | o quê |
|---|---|
| `c466a415c` | os dois instrumentos da caça aos knobs mortos |
| `02b0da869` | os **19 knobs curados** + as duas provas |
| `28f14b9eb` | a cena `=82` (o smoke da cura) |
| `b47b3b45b` | a conferida das 99 aplicada às folhas (só docs) |

### 6.2 Smokado pelo Enio ✅

`=76` · `=77` · `=78` · `=79` · `=80` · `=81` · `=82` — **todos com veredito OK**.
⚠️ O `=80` teve um defeito **encontrado por ele** (*"as duas últimas fileiras não se movem"*), já
curado e com o gate refeito: a versão anterior media um TOTAL e passava sobre uma cena morta.

### 6.3 ⚠️ NÃO smokado (o que o integrador deve olhar)

- **Nenhuma cena foi re-rodada depois do rebase/integração** — por definição.
- **Os 4 nós curados que a cena `=82` NÃO desenha**: `motion.emitter` (shape_w/h),
  `motion.boids` (avoid_radius/lookahead), `force.wind` (lacunarity/roughness),
  `fx.rgb_split` (strength/x/y). Eles seguem a mesma lei e estão cobertos pelos dois gates
  (`param_gates_are_exact` no kernel, `params_visible_tests` no painel), mas **nenhum olho os viu**.
- A suíte de GPU (`--features gpu -- --ignored`) **não foi corrida nesta janela**.

---

## 7 — Estado dos gates no fecho

| gate | resultado |
|---|---|
| `cargo test -p ph2d-host-desktop --bin` | **2882 ✓ · 0 ✗** (191 ignorados) |
| `cargo test -p ph2d-node-registry-init` | ✓ (inclui `param_gates_are_exact`, 3) |
| As 11 crates curadas | 229 ✓ |
| `clippy --all-targets` (shell + registry-init) | **0** |
| `cargo fmt --all` | limpo |
| LOC — `file_loc_caps` (shell, 600) | ✓ |
| LOC — `architecture_workspace_file_loc_cap` (700) | ✓ |
| `no_conference_scene_ships_a_setup_hole` | ✓ |
| `doc-index.sh --check` | ✓ (14 índices) |
| `conferencia_vs_manifesto.py` | ✓ **verde pela primeira vez** — 127 nós, nenhuma contagem discorda |
| `placar_conferencia.py` | P0 0 · P1 4 · P2 91 · ✅ 214 · ⛔ 124 |

---

## 8 — ⚠️ AMBIENTE: a armadilha que custou 40 minutos nesta janela

O build entrou num laço invisível — `ld.mold` morrendo em **SIGBUS** a cada tentativa de link, ~10
min por retry, sem nada no output a dizer o que estava errado.

⚠️ **O meu diagnóstico foi ERRADO e o Enio corrigiu-o na mesma noite.** Eu li «o disco encheu»; o
`df` tinha **526 GB livres**. A causa real é outra e está medida em
[[project_btrfs_metadata_starved_not_disk_full_2026_08_22]]:

1. **Metadata do btrfs faminta** — 937,85 GiB dos 950 alocados a blocos de DADOS, **0 byte
   não-alocado** ⇒ `ENOSPC` com meio terabyte livre. Cura é **root**: `btrfs balance`, não `rm -rf`.
2. **O `target/` do primário vivia em tmpfs sobre zram** ⇒ swap 32/32 GiB com 61 GiB de RAM livre.
3. **`csum failed` em artefatos NOVOS**, correlacionado com o **kernel 7.2.0** — 0 coredumps nos 5
   dias anteriores, 33 depois.

**A assinatura reconhecível, que é o que serve ao próximo:** o linker a **0% de CPU** com
`ps -o wchan=` a dizer **`vfs_coredump`**. Isso não é lentidão sob carga — é crash em laço.
**Cura imediata: `cargo clean -p <crate>`** (aqui: 99,4 GB / 638.939 ficheiros). Instrumento:
`bash scripts/btrfs-health.sh`; runbook: `docs/DevOps/BTRFS_METADATA_E_SWAP.md`.

⚠️ **E o erro de método que o mascarou:** corri `cargo test … | tail -3`, e **o pipe destruiu o
exit code** — quando matei o processo por engano, o pipeline devolveu `0` e eu li «passou»
([[feedback_pipe_masks_script_exit_code]]). *Redirecione para ficheiro e leia o `$status`; nunca
canalize um comando cujo veredito importa.*

---

## 9 — O que fica ABERTO (para o §5 e para a próxima linha)

- **91 controles** da conferência dos nós (doc 89), dos quais **~20 são obras** (peça nova de
  motor), não knobs. **4 P1**, todos estruturais — cada um pede uma porta ou um nó, não um param.
- ⚠️ **Os 5 defeitos de painel que a caça achou e que NÃO foram curados** por serem de outra
  natureza: estão em [doc 90 §5](../90_caca_aos_knobs_mortos.md) — a porta por-elemento do
  `motion.spline_wrap` lida no elemento 0, o `motion.wave` que descarta o sinal da altura, o
  `motion.kaleidoscope` que ignora o `falloff`, os knobs vivos e **inalcançáveis pela UI**.
- ⚠️ **Dois pontos cegos da sonda de knobs mortos ficam por curar** (doc 90 §4): o efeito que não
  está nas COLUNAS (um `fx.*` de raster) e o nó que precisa de uma CENA. Eles saem como
  `BANCADA-SUSPEITA` e **não acusam nada** — o que é honesto, mas cego.

---

## 10 — Higiene do fecho

✅ `rm -rf target/*/incremental` — executado (o `cargo clean -p ph2d-host-desktop` do item 8 já
reclamou 99,4 GB; o `incremental` foi a zero antes disso).

---

**Resumo:** linha `line/motion-value` pronta (HEAD `b47b3b45b`, 68 commits, base `ee1432203`).
Nenhum contrato congelado encostado; nenhuma dependência externa nova; **o risco de colisão é um
só e é nominal — os 23 números de cena (60–82) e o `MAX_DEMO_LEVEL`**. Aguardo ordem de integração.
