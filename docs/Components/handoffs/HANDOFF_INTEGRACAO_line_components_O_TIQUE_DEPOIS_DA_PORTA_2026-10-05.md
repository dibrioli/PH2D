# HANDOFF DE INTEGRAÇÃO — `line/components`: O TIQUE DEPOIS DA PORTA, COM LAMA (navegação W15) — 2026-10-05

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §23 (kill-criterion escrito antes do código,
> §23.3 e §23.4; a medição e o que ela corrigiu, §23.5; recusas, §23.6; a prova, §23.7; o aberto, §23.8). Este
> documento é o que o integrador precisa. A W5–W14 já está no `main` (rodada de 04/10, handoff
> [`A_VELOCIDADE_EM_CENAS_GRANDES`](HANDOFF_INTEGRACAO_line_components_A_VELOCIDADE_EM_CENAS_GRANDES_2026-10-04.md)).

## §0 — O `--ff-only` deve passar limpo

- base `5d596eaaf` (o `main` depois da poda do 3D) · seis commits por cima: `34f11deec` `f0a691f4e` (o plano e os
  kill-criteria, ANTES do código) · `2c90e0c68` (a lei: a procura em fatias) · `023893231` (a ponte) · `9c9242009`
  (docs, ADR, mutação) · `d750d5b21` (o que o gate de fecho apanhou) — e o deste handoff.
- ⚠️ Há uma worktree nova no mesmo módulo, `line-Components2` (aberta em `5d596eaaf`, sem commits quando esta
  fechou): se ela tocar a navegação, a colisão é com os ficheiros do §1.

## §1 — Superfície de colisão

**`ph2d-nav`, `ph2d-navmesh`, `ph2d-physics-ecs` (a ponte da navegação).** Zero crate nova, zero pacote novo no
`Cargo.lock` (uma linha: o `rayon` passa a ser dependência da `ph2d-physics-ecs` — já estava resolvido), zero
linha em `shells/desktop`, zero i18n, zero componente registado, `PROJECT_SCHEMA` intocado (o `AgentRuntime` é
memória de corrida, vai no anel e não no projeto). Prova: `git diff --name-only 5d596eaaf HEAD` não toca
`ph2d-nodegraph`, `ph2d-tool-*`, `ph2d-vector*`, `ph2d-imageio`, registos ou esquemas (§6 do CLAUDE.md).

| ficheiro | o quê |
|---|---|
| `ph2d-nav/src/polyanya_fatias.rs` **NOVO** · `polyanya.rs` · `polyanya_custo.rs` | a procura em `begin`/`resume` (pára DEPOIS do `pop` em que o trabalho passou o tecto); a `find_path_costs` em fases (`Custos`); sem tecto é a de sempre, ao bit |
| `ph2d-nav/src/plano.rs` **NOVO** · `link.rs` | `Plano` (o plano do agente em fatias, trabalho desde zero) · `Atalhos` (o Dijkstra dos atalhos em fatias); **`plan_with_links` saiu** (era `pub(crate)`) |
| `ph2d-nav/src/agent.rs` | **API nova:** `step_in_turn` + `Vez` · `AMeio` · `advance_mid` · `fatia_depois_de`; `AgentRuntime` ganha `a_meio` e `recomecos`; `step_with` é `step_in_turn` sem tecto, ao bit |
| `ph2d-nav/src/refresh.rs` | **API pública mudada:** `serve(…) -> usize` saiu, `reparte(…) -> Vec<u64>` (a reserva de cada um) |
| `ph2d-nav/src/polyanya_trabalho.rs` | `Stats::soma` |
| `ph2d-navmesh/src/tiles.rs` | `TiledMesh::assinatura()` (o CONTEÚDO: os mosaicos e a assinatura de cada um) |
| `ph2d-physics-ecs/src/bridge/nav.rs` · `nav_fila.rs` · `nav_malha.rs` · `tape.rs` · `Cargo.toml` | o passo em paralelo (`PROCURAS_EM_PARALELO = 16`); a vez; «mudou» pelo conteúdo; **`ControllerMemory` ganha `nav_sinais`** (o anel); `set_nav_slices` · `set_nav_parallel` · `nav_search_work` · `nav_search_critical_work` |
| testes | `ph2d-navmesh` `tests/it/fatias.rs` **NOVO** (+ `dominancia::cena` passa a `pub(super)`) · `ph2d-physics-ecs` `tests/it/nav_fatias.rs` **NOVO**, `nav_mundo.rs`, `nav_nascer.rs` |
| `ph2d-app-physics/src/bridge/dispatch.rs` · `tape_second_life_tests.rs` **NOVO** | (§4b) a fita grava todo tique devido |
| `ph2d-nav/src/agent.rs` (`AgentConfig::radius`, `alcance_de_atalho`) · `ph2d-app-components/src/nav_smoke_lava_tests.rs` | (§4b) a entrada de um atalho a um raio |
| docs | plano 30 §23–§24; `docs/Physics/BUGS_physics.md` Bug #10; **ADR-0180** (o `rayon` na ponte); `ferramentas/mutacao_navegacao_w15_2026-10-05.py`; `examples/medir_replaneio.rs` (as versões no mesmo processo) |

### ⚠️ O que um merge pode partir

1. **O número do ADR SOMA entre linhas:** `0180` foi contado contra o máximo do `main` e das worktrees do dia
   (`0179`) — reconte-o contra o `main` na integração (o nome do ficheiro, o título, as citações no plano §23.5, no
   `Cargo.toml` da `ph2d-physics-ecs` e no doc de `PROCURAS_EM_PARALELO`) e corra `bash scripts/adr-index.sh`.
2. Quem chamar `refresh::serve`, construir um `refresh::Owed` à mão ou um `AgentRuntime` por campos não compila / tem
   campos novos (`a_meio`, `recomecos`) — `..Default::default()` serve.
3. Um terceiro assunto no anel dos controladores passa pelo `ControllerMemory` (o tipo obriga).
4. Tectos: `ph2d-nav` `polyanya.rs` `603 / 700` · `agent.rs` `~555` · ponte `nav.rs` `693 / 700` (⚠️ quase no tecto:
   a cura de quem crescer é MOVER, como os comandos da sonda foram para o `nav_fila.rs`).

## §2 — O que a W15 traz

**O defeito:** com muita lama, o tique depois de uma porta chegava a `40–90 ms` — UMA procura passava o orçamento
sozinha, e os replaneios que não passavam pela fila (o alvo andou, saiu do corredor, preso) faziam `1–7` procuras num
tique mesmo sem porta (W14 §22.8).

**A cura:** toda procura paga de um orçamento por tique e PÁRA a meio quando ele acaba (o idioma do Detour
`updateSlicedFindPath`), continuando no tique seguinte; no anel vai só a descrição dela (o scrub refá-la até ao mesmo
`pop`). As procuras a meio avançam em PARALELO (16, `rayon`, ADR-0180), a mais adiantada primeiro; o caminho crítico do
tique é UM orçamento. Quem espera anda o caminho que tem.

**De passagem, um defeito de determinismo da W9:** a fila decidia «a malha mudou» pelo que a actualização dizia, e
depois de um scrub as malhas são as do FIM da corrida — um scrub para antes de uma porta mudar via uma mudança que a
corrida não viu. Agora «mudou» é o conteúdo contra a assinatura que o anel guardou.

| `150` lamas a peso 4 (o mínimo de 7 intercaladas, load `22–37`) | antes (A) | **depois** |
|---|---|---|
| pior tique depois da porta, 10 · 50 · 200 agentes | `48 · 59 · 66 ms` | **`5,9 · 7,5 · 12,9`** |
| CONTROLO sem porta | `1,2 · 37 · 88` | `6,4 · 6,0 · 11,1` |
| trabalho do caminho crítico num tique | até `968 mil` | **`40 mil`** |
| o que falta andar no fim (a régua de que a vez não atrasa) | — | `+0,6 · +1,2 · +2,2 %` |
| sem caminho no fim, a 200 agentes | `0` | **`0`** (em série: `189`) |

## §3 — ⏳ O que fica ABERTO

- **A mutação `S5` sobrevive** (a guarda `sem_zona`): a resposta dela é a mesma da verificação parcial da corrida por
  construção, e uma fixtura que a separe pede duas portas com mudanças em zonas diferentes entre o âncora, o agora e o
  fim (plano §23.7).
- **O CONTROLO com lama a 10 agentes** fica em `6,4 ms` (`0,7` sem lama): a versão de antes fazia o trabalho todo no
  tique da porta (`48 ms`) e ficava vazia; a de agora reparte-o a um orçamento por tique (§23.5, o critério 3 tal como
  escrito não se cumpre).
- Com a fila cheia, quem replaneia por motivo próprio espera um tique pela vez; o «sempre pelo menos um» da W9 saiu.
- De antes: a barreira larga e lenta (`2,9×`, §22.6) e a dominância da procura ponderada (§22.4).

## §4 — A prova de fecho

- **Gate batched** (sobre `5d596eaaf..HEAD`): `nextest-impacted` `15 765 / 15 766` — o vermelho foi o tecto de LOC
  (`bridge/nav.rs` `719 / 700`), curado em `d750d5b21` movendo os comandos da sonda; depois disso o clippy
  `--workspace --all-targets -D warnings` limpo (apanhou `result_large_err` em `Plano::begin`, curado) e `ph2d-nav` ·
  `ph2d-navmesh` · `ph2d-orca` · `ph2d-physics-ecs` · `ph2d-app-components` · `ph2d-editor-core` `3 731 / 3 731`
  (incluindo `workspace_src_files_under_loc_cap`). Load `20–45` (outras linhas a compilar).
- **Gates novos** (11): ver o plano §23.7.
- **Mutação `18 / 19`**, zero defeitos de arnês, árvore igual antes e depois. A 1.ª corrida deu `13 / 22`: quatro eram
  código MORTO (cortado), três gates estavam cegos ou faltavam (um scrub com a fixtura vazia — substituído; três gates
  novos), e um observador faltava (a dominância).
- **A medição** (`medir_replaneio`, a régua do dono de 05/10): as versões no MESMO processo (`set_nav_slices`,
  `set_nav_parallel`), 7 rodadas intercaladas com a ordem rodada, o mínimo e a mediana, uma compilação `smoke`.

## §4b — Os dois reports do smoke do dono (05/10), curados na mesma linha

1. **Cena `=4`: dois inimigos iguais presos no mesmo portal** (pré-existente, W5 + W7 — igual com as fatias
   desligadas). O teletransporte disparava no CENTRO do agente; dois corpos encostados um de cada lado do
   ponto nunca lá chegavam. Cura: a entrada de um atalho alcança-se quando fica DENTRO do corpo
   (`ph2d_nav::agent::alcance_de_atalho`; **`AgentConfig` ganha `radius`** — quem o constrói por campos tem
   de o pôr; `0` = o comportamento de antes). Plano 30 §24 (a tabela do limiar), gate
   `dois_inimigos_iguais_pelo_mesmo_portal_nao_se_prendem`, três mutações à mão a sangrar.
2. **Cena VIDA `=4`: na 2.ª vida o herói andava e rodava sozinho** (pré-existente, a fita da W7/W17). A
   gravação regravava só o tique ALVO de cada quadro; depois do recomeço, num quadro que devia dois tiques, o
   do meio ficava com o dedo da vida anterior. Cura: gravar todo tique devido
   (`ph2d-app-physics/src/bridge/dispatch.rs` — ⚠️ **ficheiro da família da Física**, tocado por esta linha).
   `docs/Physics/BUGS_physics.md` **Bug #10**, gate `na_segunda_vida_sem_tecla_o_heroi_nao_anda_nem_roda`.

3. **Cena `=4` (2.º report): quem sai do portal em cima do herói prende-o.** O salto não perguntava quem
   estava na saída (aterrava a `2 cm` do centro do herói). Cura: com a saída ocupada, espera em cima da
   entrada (**`Vez` ganha `saida_livre`** — a ponte responde pela forma de cada corpo). Plano 30 §24.1, gate
   `ninguem_sai_do_portal_dentro_de_outro_corpo`.

## §5 — O smoke

Nenhuma cena nova: numa cena pequena o orçamento nunca se esgota e a condução é a MESMA (o kill-criterion 1); o que
mudou só a bateria mostra (a sonda com `150` lamas). As cenas de sempre, como regressão:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=3 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
```

Fotografadas no ecrã virtual antes de ir ao dono (`target/prova/w15/ph2dnavsmoke3.png`, `ph2dnavsmoke4.png`,
`ph2dvidasmoke4.png`, `59–61 fps`): na `=3` o guarda patrulha (*«Moving · 0.84 m to go»*); na `=4` o vermelho deu a
volta à lava e *«Arrived»* ao herói; na arena os morcegos andam (*«Signal: morcego»*, *«Signal: coracao»*).

### O smoke compilado (a 2.ª corrida, colada)

Depois de `rm -rf target/*/incremental` (`11 G` do `debug` + `3,6 G` do `smoke`), a 2.ª corrida de
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` — zero linhas `Compiling`:

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 3600s
    Finished `smoke` profile [optimized] target(s) in 0.45s
```

(Refeito depois das curas do §4b, de novo após `rm -rf target/*/incremental`; as cenas `=4` da navegação e da
arena fotografadas outra vez — `target/prova/w15/*-curas.png`, `59–60 fps`. O gate batched sobre o diff
acumulado: `nextest-impacted` `15 766 / 15 768`, os dois vermelhos são membros catalogados da família de flakes
de carga — `the_cost_of_sampling_a_path_is_flat_in_its_anchors` e
`the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`, `FLAKES_DE_CARGA.md` — e passam sozinhos a load
`35`; clippy `--workspace --all-targets -D warnings` limpo.)

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (8% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                296   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                769 : 239   alvo: <= 1,0  razao 3.2x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1843 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         462 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
