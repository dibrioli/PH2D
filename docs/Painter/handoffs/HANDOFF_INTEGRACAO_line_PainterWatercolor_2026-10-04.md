# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, «cada controlo funciona em cada meio» (2026-10-02 → 2026-10-04)

> **Este documento é para o AGENTE INTEGRADOR** (que só funde por ordem explícita do dono). Ele diz o
> que a linha toca, onde um merge pode doer, a prova de fecho e os smokes. Cobre **todos** os commits
> desde `1ad60a1ce` — nenhum foi integrado. O *mecanismo* de cada passo vive nos docs, e este aponta
> para eles:
>
> * o **censo** [`45_censo_dos_controlos.md`](../45_censo_dos_controlos.md) e o **plano**
>   [`46_plano_cada_controlo_em_cada_meio.md`](../46_plano_cada_controlo_em_cada_meio.md) (as decisões do
>   dono verbatim e o estado de cada item);
> * os **BUGS** [#30–#34](../BUGS_painter.md) (o índice dos fechados, uma linha com o mecanismo);
> * os dois handoffs de continuação de 04/10
>   ([1.ª onda](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-04.md) ·
>   [2.ª onda](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-04_ONDA2.md)) — o instrumento do censo e
>   o que custou tempo.

---

## §1 — Coordenadas

| | |
|---|---|
| ramo | `line/PainterWatercolor` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` |
| base | merge-base **`1ad60a1ce`** = o `main` de hoje (não andou desde o fork) ⇒ **nada para rebasear** |
| commits | **29** (28 de trabalho + `b11273456`, os avisos do clippy do fecho) + o deste handoff |
| diff | `138` ficheiros, `+7 988 / −435` (antes deste handoff) |
| crates novas · pacotes EXTERNOS novos · ADR novo | **nenhum · nenhum · nenhum** |
| dependência INTERNA nova | `ph2d-panel-painter-layers` → dev-deps `ph2d-tool-painter` (feature `test-support`) e `ph2d-tool-registry` (o censo) — `Cargo.lock` +1 linha |

**O que a linha fez, por onda:**

| onda | commits | o quê |
|---|---|---|
| 02/10 — papel e borda da aquarela | `b03bbfba2`…`3045ce716` (10) | o Tooth assenta nos vales e vai a 2; o **Mapping** do Paper sai (BUGS #30) · o Ragged Edge ganha **Flow** (o Classic + 8 padrões, Size, Angle, pré-visualização) e **Paper Edge** (BUGS #31) · o Tooth morde no **Wet Paint** (BUGS #32) · **«Use as Flow»** no menu da Hierarchy |
| 03–04/10 — o censo, 1.ª onda | `9dbf23ba2`…`bbe258153` (7) | Blow/Smear da água com um carimbo fora da grade não rebentam · **o censo** (cada controlo × cada meio, medido pela tinta) · **Reset** de uma secção repõe o pincel do MODO · **Accumulate/Space Attenuation** só onde o meio os oferece (no Wet Paint apagavam o traço) · o **esmaecido** (o controlo dependente desenha-se no tom desabilitado, com a dica) |
| 04/10 — 2.ª onda | `53a04d728`…`e39cefcfb` (7) | **Dry Time por poça** (cada traço seca no seu tempo; a 1.ª versão reprovou no smoke do dono e a cura ao texel foi aprovada: *«está ok»*) · **o gate permanente do censo** + BUGS #33 · Shape Color Ramp sai do Wet Paint · Spread medido · **Blend no Wet Paint** |
| 04/10 — 3.ª onda | `36ab5fb6a`…`e3fe733f6` (4) | **BUGS #34** — a corda do Solid apagava o traço no Impasto e em `Strength < 1` · na **Aquarela**: a Shape Color Ramp, os **fios** (Sketchy · Wire · degraus do Ribbon) e o **Solid** |

---

## §2 — Superfície de colisão (colada do `collision-surface.sh`, 2026-10-04)

```
SUPERFÍCIE DE COLISÃO — line/PainterWatercolor contra main
  merge-base 1ad60a1ce   ·   28 commit(s)   ·   138 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        178   (base: 178)
      └ tripla do gate               (178, 13, 22)   (base: (178, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      23   (base: 23)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              108   (base: 108)
    ph2d-render (espelho)                 109   (base: 109)
    ph2d-script (espelho)                 109   (base: 109)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0175   próximo livre: 0176
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
      662 / 700   crates/ph2d-app-painter/src/painter_bridge.rs  (tem marcador/allowlist — confira o valor congelado)
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.

(corrido sobre os 28 commits de trabalho; o 29.º, `b11273456`, só troca `chunks_exact` por
`as_chunks` e um `%` por `is_multiple_of` em seis ficheiros já listados)
```

---

## §3 — O que está fora da pasta do módulo (e porquê) — tudo ADITIVO

| onde | o quê | porquê |
|---|---|---|
| `ph2d-editor-core` (`ids/menus_hierarchy.rs`, `action_bus_hier.rs`, `screens/hero/{menu_tables,pre_populate}.rs`) | id `CTX_MENU_HIER_USE_AS_FLOW = hash_node_id("ctx_menu_hier_use_as_flow")` · variante `HierRequest::UseAsFlow { row }` · a linha no menu da Hierarchy | «Use as Flow» (BUGS #31) — irmão do `USE_AS_GRANULATION`, append-only |
| `ph2d-panel-hierarchy` (`event.rs` + gate) | o clique da opção nova empurra a variante | idem |
| `shells/desktop/src/render_loop/` (`fase_use_as_paper.rs`, `fase_bus_hierarchy.rs`, `fase_bus_drain_out.rs`, `fase_hero_commits.rs`) | o despacho do «Use as Flow» (`UsoDaCamada::Fluxo`) | idem — **a shell ENCOLHE 10 linhas** (`+22 / −32`) |
| `ph2d-app-painter` (`painter_bridge.rs` → `painter_bridge_previews.rs`) | as pré-visualizações do Flow, num irmão | o `painter_bridge.rs` perto do tecto de LOC |
| `ph2d-i18n` (`painter_layers.rs`, `chrome_menus.rs`, `shell_media.rs`) | chaves novas: `panel.painter_layers.inerte.*` (13 dicas do esmaecido), `panel.painter_layers.watercolor.{flow, flow_classic, flow_size, flow_angle, paper_edge}`, `chrome.menu.use_as_flow`, `shell.fase_use_as_paper.{flow, watercolor_flow_set}` | texto novo pela porta do i18n |
| `ph2d-painter-brush` (`spec.rs`, `spec/queries.rs`, `spec_default.rs`, `texture/patterns/specs.rs`) | campos `BrushSpec::{edge_flow, paper_edge}` · consts `PAPER_TOOTH_MAX = 2.0`, `FLOW_SIZE_MIN = 0.25`, `FLOW_SIZE_MAX = 4.0`, `FLOW_KINDS` (9) · consultas `space_attenuation_reaches`, `jitter_unit_matters`, `dash_length_matters`, `grain_samples` · `param_inerte` | o Flow e as perguntas do esmaecido/censo. `BrushSpec` **não** é serializado (`#[derive(Clone, Copy, Debug, PartialEq)]`, sem `serde`) |
| `ph2d-wet-paint` (`painter.rs`, `painter/{doors,knobs}.rs`, `paper.rs`, `tools.rs` + gate) | `Engine::set_paper_tooth`, `paper::dente` · `rect_around_nao_vazio` | BUGS #32 · o pânico da Blow/Smear |
| `ph2d-panel-registry-init` (`tests/it/nenhum_rotulo_do_app_pinta_nada.rs`) | a catraca do degrau estreito **5 → 4** | a linha Mapping do Paper saiu (medido por eliminação) |
| `ph2d-tool-painter/Cargo.toml` | feature `test-support` (só os gates a ligam: a água num relógio FIXO) | o censo |

**Lista/catraca BAIXADA:** a do degrau estreito (5 → 4) acima. **Item partilhado cujos usos se
apagaram:** nenhum.

---

## §4 — Contratos congelados e schemas

- **Contratos congelados (`CLAUDE.md` §6): nenhum encostado.** Nodes, Tools e Vector não aparecem no
  diff; os gates `architecture_*_contract_surface` correm no `nextest-impacted` (§6).
- **`PROJECT_SCHEMA` e os outros schemas: intocados** — a §2 mostra cada um igual à base; nenhum
  ficheiro de projecto, de `ph2d-core` ou de schema está no diff.

---

## §5 — O que só o `ship.sh` pega — corrido no fecho

Todos corridos 1× sobre o diff acumulado, `load` entre 6 e 36:

| régua | resultado |
|---|---|
| `cargo fmt --all -- --check` | verde |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| clippy `--all-targets --all-features -D warnings` nos 9 pacotes tocados | **verde depois de `b11273456`** — a 1.ª corrida achou 8 avisos do clippy 1.98 (`chunks_exact_to_as_chunks` ×7, `manual_is_multiple_of` ×1), todos em código desta linha; o 8.º só apareceu depois de curados os 7, porque o erro no lib escondia o pacote dependente |
| `cargo machete` | verde |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `#[cfg(target_os` novo ou movido | nenhum (não há o que cruzar para macOS) |

### §5-bis — A árvore combinada

`censos-da-arvore-combinada.sh`: **127/127**, *«12 de 12 censos correram»*. O `main` não andou, então
esta árvore É a combinada.

---

## §6 — Prova de fecho

- **`nextest-impacted` (base `1ad60a1ce`): 19 673 / 19 673**, `Summary [151.739s] 19673 tests run: 19673
  passed (1 slow), 11268 skipped`. O flake conhecido `the_mask_stroke_cost_does_not_follow_the_canvas` não
  apareceu.
- **Os gates de LOC e de fonte:** `file_loc_caps` (shell), `architecture_workspace_file_loc_cap`,
  `the_shell_only_shrinks`, `arch_safe_clamp_only` — 9/9 verdes.
- **Os três da `ph2d-panel-registry-init` no âmbito do WORKSPACE** (`nenhum_rotulo_do_app_pinta_nada` ·
  `a_marca_tem_a_altura_da_linha` · `nenhuma_escolha_do_app_e_montada_a_mao`; com `-p` falham por falta
  de painéis): **16/16**.
- **Os dois censos do painter** (`o_censo_dos_controlos_so_encolhe`, `a_linha_esmaecida_e_a_que_nao_age`):
  verdes, também depois de `b11273456`, com os 163 gates das zonas que ele tocou.
- **O smoke compilado** (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`,
  depois do `rm -rf target/*/incremental` — 70 GB), a 2.ª corrida, sem um único `Compiling`:

```
▸ linha line_painterwatercolor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.20s
```

**Provas de mutação** (o detalhe no corpo de cada commit): `36ab5fb6a` **5/5** · `b1b262be7` **4/4**
(a 4.ª sobreviveu à 1.ª versão dos gates e deu o gate do 1.º carimbo de ponta de imagem) ·
`cd31caf1d` **5/5** · `e3fe733f6` **3/3** · e as das ondas anteriores, nos corpos delas.

**Auditoria (DIRETIVA §3), duas lentes sobre o que a 3.ª onda mudou de comportamento:**

| | lente CORREÇÃO — o rascunho em todos os canais | lente COSTURA — a aguada recebe o que não é dab |
|---|---|---|
| CLAIM | uma escrita transitória por evento (a corda do Solid; a mancha na aguada) não deixa estado no traço depois de desfeita | o fio e a mancha do Solid entram nos acumuladores da sessão molhada e chegam ao composite do quadro e ao bake do pen-up |
| TRAÇO | `stamp_dabs` (bracket) → `peel_drag_preview` (+ `peel_mancha_na_aguada`) → dabs → `park_stroke` → `stamp_solid_preview` → `stamp_corda_em_rascunho` (devolve `stroke_mask`, `per_layer_stroke.cov`, `tex_rng`; `corpo_suspenso`) · na aquarela `mancha_na_aguada(…, true)` guarda o recorte dos 6 planos · `paint_end` → `assenta_o_corpo_da_corda` → `commit_drag_preview` larga o registo | `park_stroke` → `stamp_threads` → `fios_na_aguada` · `stamp_solid_preview` → `mancha_na_aguada` — os dois por `PlanosDaAguada::deposita` e pelas três janelas sujas → `paint_tick` → `apply_watercolor(false)`; `paint_end` → `apply_watercolor(true)` |
| ASSERÇÃO-VERMELHA | `a_corda_nao_deixa_rasto_nos_acumuladores_do_traco` · `a_corda_nao_gasta_o_sorteio_do_grao` · `a_mancha_provisoria_nao_deixa_resto_na_aguada` | `o_sketchy_tinge_a_aguada_alem_do_rastro` · `o_wire_sem_linha_deixa_so_o_arame_na_aguada` (é o único que vê as janelas sujas) · `os_degraus_do_ribbon_chegam_a_aguada` · `o_solid_enche_a_regiao_cercada_na_aguada` · `uma_elipse_em_solid_e_um_disco_na_aguada` |
| NÃO-CHECADO | o «pisca» do smoke (§9); um acumulador do traço que nasça amanhã e a corda não devolva — a lista mora num sítio só (`stamp_corda_em_rascunho`), sem gate que a enumere | a mistura do Mixer/Pigment num fio ou numa mancha (por desenho: cor do pincel por `over`) |
| LOC LIDAS | ~1 400 (`solid_deposit`, `stamp_route`, `stamp_preview`, `stroke_lifecycle`, `stroke_cover`, `impasto` cabeçalho, `relief_state`) | ~1 100 (`watercolor_accum`, `watercolor_accum_cor`, `watercolor_rim` cabeçalho, `thread_deposit`, `stroke_multi`) |

**Olhado (DIRETIVA §4), não só medido:** a rampa na aguada × no digital; os fios na aguada (a 1.ª
lei, cobertura por `max`, fundia a teia densa numa poça e riscava o miolo de preto — reprovada pela
imagem antes de qualquer gate); as três formas do Solid na aguada.

---

## §7 — Smokes

**Aprovados pelo dono nesta jornada:** o Dry Time por poça ao texel (*«está ok»*); o Solid na
aquarela (*«funciona»* — com o defeito aberto da §9).

**Por smokar** (não voltou smoke do dono): a onda de 02/10 (Flow, Paper Edge, Tooth no Wet Paint,
«Use as Flow»), o esmaecido, o Reset por meio, o Blend no Wet Paint, a rampa e os fios na aquarela, e
o Solid no Impasto (BUGS #34).

Depois da fusão é o `main` que se smoka:

```
cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --profile smoke
```

O binário desta árvore ficou compilado (§6, a 2.ª corrida, `0.20s`). No painel do pincel: **Watercolor**
→ **Solid** → um laço à mão livre (uma aguada cheia, borda escura); **Line Type → Wire** com
*Connection Line* desligado (só o arame); **Shape → Color Ramp** com duas cores (a aguada pinta-se da
rampa). **Impasto** → **Solid** → um laço (o traço não perde tinta).

---

## §8 — O que uma leitura rápida do diff entende ao contrário

1. **O «Sketchy no Impasto só age com Solid» do censo não se curou na porta dos fios.** A porta
   nunca excluiu o Impasto; o Solid é que estragava o traço e os fios viam-se por cima do estrago
   (BUGS #34). Na fábrica do Impasto o fio cai no rastro OPACO da mesma cor e não muda um byte — fica
   na lista do censo com esse motivo (e o mesmo na Aquarela: dentro da tinta molhada o fio só deposita
   pigmento).
2. **Na aguada o fio só ESTENDE a cobertura sobre papel seco** (`Cobertura::SoNoSeco`). Não é
   timidez: com `max` o aro riscava o miolo (14 de 693 texels) e a teia densa virava uma poça.
3. **Uma rampa de duas cores ao longo de um traço sai acinzentada no miolo — nos dois meios.** É a
   lei do digital (cada carimbo deposita a cauda clara por cima do centro do anterior), não um
   defeito da aguada.
4. **O tom (B&W) da Shape Color Ramp na aguada só age com uma Shape**, como no digital (`stamp.rs`
   remapeia o valor CRU da silhueta); sem Shape é um no-op de propósito.
5. **`ManchaNaAguada` mora na `PainterTool` e não no `PaintState`**: o `state.rs` está no tecto de 700
   linhas. Ao lado da `WashCadence`, pela mesma razão (bookkeeping de ESCREVER a tela).
6. **`ReliefState::corpo_suspenso` tem o sentido de SUSPENDER** — o `Default` (`false`) é o caminho de
   sempre; só a corda do Solid num quadro intermédio o arma.
7. **A pegada do rascunho de forma na aguada cresce pela caixa do Solid** (`stamp_drag_preview_watercolor`)
   e nenhuma mutação a fere: o contorno da figura já contém a região. Fica porque o irmão digital
   (`stamp_drag_preview`) faz a mesma união — duas regras para a mesma pegada divergiriam.

---

## §9 — Aberto

- **O Solid na aquarela PISCA esporadicamente** (smoke do dono, 2026-10-04: *«funciona. contudo
  esporadicamente a área de preenchimento pisca. Depois corrigiremos isso.»*). **Não investigado.** O
  mecanismo da mancha provisória é: descasca no 1.º lote de cada evento (`stamp_dabs` → `peel_drag_preview`
  → `peel_mancha_na_aguada`) e volta a depositar no `park_stroke` do mesmo evento; o composite é um por
  quadro. Um quadro que componha ENTRE o descasque e o depósito mostraria a aguada sem a mancha — a
  primeira coisa a medir é se algum sítio do ciclo de traço carimba lotes sem chamar o `park_stroke`
  (o tique do airbrush/estabilizador). ⚠️ Hipótese, não medição.
- **Wet Paint — Composite Brush (plano 46 item 8) e Solid/fios (item 9): NÃO feitos, NÃO medidos.** O
  kill-criterion ficou escrito antes de qualquer build: o quadro da pilha cheia no Wet Paint não pode
  passar o do Composite no Digital; a teia densa de carimbos de água por quadro, idem. A máquina
  esteve entre `load 18` e `load 106` a sessão inteira — nenhum número de desempenho vale ali
  (`CLAUDE.md` §5.0). As linhas do censo dos dois continuam na lista, com «a fazer».
- **Secar DENTRO de uma poça** (doc 14 #12b) — fora, com o porquê no plano 46 item 5.
- **A procura completa do censo** (`CENSO_ARMAR=2`) não cabe nos 30 min do `ph2d-run` sob carga no
  Impasto e no Wet Paint.

---

## §10 — Para o integrador

- **`CLAUDE.md` §5.1, módulo Painter:** esta linha trocou o link do último handoff para este (a frase
  do que o módulo é não mudou).
- **Flakes:** nenhuma nova a promover; nenhum vermelho de relógio nesta corrida.
- **Memória:** a linha não escreveu ficheiros em `project-memory/`.

### O perfil do agente (`agent-loop-profile.sh`)

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
──────────────────────────────────────────────────────────────────────────────
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                246   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                740 : 250   alvo: <= 1,0  razao 3.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1765 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         487 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
──────────────────────────────────────────────────────────────────────────────
  As leis moram no CLAUDE.md §2 (sempre carregado); a DIRETIVA_IMPLEMENTACAO aponta pra la'.
  ⚠️ Rode com poucas sessoes para ver o HABITO recente; 'all' e' o baseline historico.
```
