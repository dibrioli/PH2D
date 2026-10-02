# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-10-01 (AS FORMAS NA PLACA)

> **Para o agente INTEGRADOR** (só por ordem do Enio — `CLAUDE.md` §0.7). A linha está rebaseada
> sobre o `main`, passou o portão de fecho na árvore combinada (§7) e **não integra nem pusha
> sozinha.** Este documento SUPERSEDE, como documento de integração, o de
> [2026-09-24](HANDOFF_INTEGRACAO_line_motion_value_2026-09-24.md): aquele foi integrado na rodada 03
> e já está no `main`; o que segue são os **17 commits** feitos depois dele, mais os dois do fecho.
> O mecanismo inteiro vive no [doc 121](../121_as_formas_na_placa.md) — aqui fica só o que quem funde
> precisa de saber.

## §0 — IDENTIDADE

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |
| ramo | `line/motion-value` |
| base | `main` @ `912a9652e` — **0** commits do `main` por trazer |
| commits da linha | **17** + os dois do fecho (a cura do §7 e este documento) (2026-09-29 → 2026-10-01) · `85` ficheiros (+10 205 / −302) |
| integração | `--ff-only` possível no momento do fecho (a linha está POR CIMA do `main`) |

## §1 — O QUE A LINHA ENTREGA

**Um assunto: toda forma viva do Motion é desenhada pela PLACA, nítida e sem custo por cópia**
(ordem do dono de 29/09: *«não só as estrelas mas todas as shapes devem ser o mais otimizadas
possível»*, com a lei de que **nitidez não se troca por velocidade** — nada é assado em imagem).

| peça | o que é | doc 121 |
|---|---|---|
| **crate-folha nova `ph2d-shape-gpu`** | o passe de formas INSTANCIADO: cada geometria preparada uma vez, `N` cópias numa chamada, com a cobertura por área **portada do rasterizador fino do Vello** (`vello_shaders`, Apache-2.0 OR MIT — porta aberta, já dependência do repo) | §2, §6 |
| **rota da CPU** (W2) | as `VectorInstance` do `pump` vão ao passe, TUDO-OU-NADA por quadro; a camada é colada no acumulador entre o documento e o chrome | §7 |
| **rota do DISPOSITIVO** (W3) | o cozimento escreve as cópias de forma num buffer próprio; a ponte deixa de recusar pelo TIPO do nó e pergunta pelo CONTEÚDO | §8 |
| **o traço sob escala NÃO uniforme** (W4) | construído no ecrã a partir do EIXO, com a caneta redonda da casa (`w·√|det|`) e as juntas e pontas autoradas; só o traço TRACEJADO esticado fica no Vello | §9 |
| **cena `=127`** | estrelas esticadas com contorno sobre a galáxia da `=126`; arranjo LEGÍVEL (o contorno) e DENSO (`PH2D_TRACO_ESTICADO_DENSO=1`, o relógio) | §9.1 |
| **a faixa · o contorno calculado · as células e o fundo** | o custo por pixel desce em três degraus; no proxy de telemóvel a `=127` densa passa de `35,2` para **`18,76 ms`** de placa, **à frente do Vello (`19,42`)**; RTX `1,11` contra `2,03` | §9.3–§9.6 |

**Censo de rota:** `17` das `22` cenas do catálogo com forma vão à placa; as `5` que ficam têm a
recusa nomeada no `PH2D_MOTION_ROUTE_LOG` (`fx.glow` · passagem · colisor lido pelo grafo ×2 ·
traço tracejado).

**Smoke do dono:** ✅ `=127` aprovada em 30/09 (`raw 200` pela placa contra `100` sem ela) · ✅
`=127` densa aprovada em 01/10 depois das células.

**E um commit de sobras** (`a7ff80c43`): três vermelhos `#[ignore]` que o handoff de 24/09 deixou
abertos (§6 dele) — `write_the_rig_figures` (a corda tem `19` SEGMENTOS, derivado; figuras e PDF do
tutorial 9 regenerados), `measure_the_source_group` (exige `PH2D_MOTION_SO_COM_FORMA=0` e recusa-se
sem ele) e o gate da voz do dispositivo (lia só `//`; usa `ph2d_label_census::sem_comentarios`).

## §2 — SUPERFÍCIE DE COLISÃO, medida (`collision-surface.sh`, sobre o `main` @ `912a9652e`)

```
SCHEMAS             PROJECT_SCHEMA 176 (base 176) · tripla (176, 13, 22) igual
                    VEC_SCENE 22 · FLIP 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23 — todos iguais
REGISTRO            ph2d-ecs 108 · ph2d-render 109 · ph2d-script 109 — todos iguais
CONTRATO (§6)       crates/ph2d-nodegraph/src/node.rs intocado · crates/ph2d-editor-core/src/tool.rs intocado
ADR                 nenhum criado
Cargo.lock          um '+name': "ph2d-shape-gpu" (crate NOSSA, nova) — nenhum pacote EXTERNO novo
MARCADORES          nenhum
TETOS DE LOC        nenhum ficheiro da linha acima do tecto
```

⇒ **Zero contador partilhado se move.** Não há degrau de `PROJECT_SCHEMA` a recontar.
⚠️ `MAX_DEMO_LEVEL` do Motion `126 → 127` (a cena nova) — conte-o no roteador se outra linha
acrescentar uma cena de Motion na mesma rodada.

### §2.1 FOUNDATIONAL tocado — tudo ADITIVO menos UMA linha

| crate | mudança | aditiva? |
|---|---|---|
| `ph2d-render` | `BandSource::Formas` **apendado** (a posição é o índice do uniforme), `uniforms: [_; 3]` | ✅ — um `match` exaustivo sobre `BandSource` noutra linha deixa de compilar, alto |
| `ph2d-vector` | re-exporta `StrokeOpts`, `flatten`, `stroke as expand_stroke` do `kurbo` do Vello | ✅ |
| `ph2d-vec-render` | `mod placa` novo: `FormaParaAPlaca` + `forma_para_a_placa` | ✅ |
| `ph2d-gpu` | só `[dev-dependencies]` (`ph2d-label-census`) e o corpo de um teste | ✅ |
| `ph2d-gpu-cook` | `GpuCookError::FormaComMistura` (variante nova) · `lower_forma` + `formas` (módulos novos) · `read_formas` · `GpuCook::formas()` | ⚠️ ver abaixo |

**⚠️ NÃO aditivo — parte código de OUTRA linha que o use:**

| mudança | quem usa hoje (medido com `git grep` nesta árvore) |
|---|---|
| `ph2d_gpu_cook::lower::LOWER_COLUMNS` passa de `[&str; 8]` a `[&str; 9]` (`geometry_id` **no fim**), e `lower_module`/`lower_signature` recebem `present: [bool; 9]` | só a própria crate e os testes dela; um chamador novo de outra linha com `[bool; 8]` **não compila** (falha alta) |
| `GpuCookError` ganha `FormaComMistura` | só crates do Motion |

⚠️ **E uma mudança de COMPORTAMENTO de uma porta partilhada:** o baixamento das sprites no dispositivo
**cala** (tamanho, âncora e opacidade a zero) toda linha com `geometry_id > 0,5` — antes nenhuma
linha tinha a coluna, e sem ela a saída é **byte-idêntica** (gate `absent_columns_read_the_cpu_defaults`).

### §2.2 A shell — onde pode haver atrito com outra linha

`shells/desktop` tocada em **9** ficheiros (+132/−21), e o atrito real é a **ORDEM das fases do
quadro**, não os ficheiros:

| ficheiro | o que muda |
|---|---|
| `render_loop/fase_vector_bands.rs` | assinatura ganha `motion_tool_active` (1.º argumento) e decide se as formas vão à placa ANTES de o documento ser codificado |
| `render_loop/fase_hero_scene.rs` | o chamador da de cima |
| `draw_bands.rs` | `doc_bands_of(order, forcar: bool)` — com as formas na placa o documento vai às FAIXAS, para ficar por baixo delas |
| `render_loop/present.rs` | `plan.banded |= placa.ativa()` |
| `render_loop/present_chrome.rs` | cola a camada das formas no acumulador do mundo, por cima do documento e por baixo do vidro e do chrome; ⚠️ o `let tamanho` subiu umas linhas (é lido pela colagem) |
| `render_loop/fase_vector_overlays.rs` | com a placa activa a cena Vello NÃO encoda as formas do Motion |
| `render_loop/present_bands.rs` + `present_placa_tests.rs` | gate da fiação (`a_rota_do_dispositivo_chega_ao_quadro`) |
| `tests/it/the_gpu_cook_recusal_placement.rs` | a agulha passa de `graph_has_live_vector_source(` a `forma::formas_para_a_placa(` |

⇒ se outra linha da rodada mexer em `fase_vector_bands`/`present_chrome` (o vidro, as faixas do
documento, o `band_blit`), **leia as duas ordens antes de aceitar uma fusão textual limpa** — a
camada das formas tem de ficar entre o mundo e o chrome (doc 121 §3).

## §3 — ⚠️ Coisas que uma leitura rápida do diff entende ao contrário

1. **O passe NÃO é uma aproximação do Vello — é a conta DELE**, portada (`fill_path`, modo
   `AaConfig::Area`). A paridade vem por construção; os resíduos medidos são do aplanamento
   (`0,25 px`, a tolerância do próprio Vello) e estão nas barras dos gates.
2. **A faixa é MAIS exacta que o Vello nas quinas côncavas** (o Vello soma os dois rectângulos que se
   sobrepõem): é por isso que a paridade da estrela esticada SUBIU de alfa `40` para `73` e há um
   gate próprio contra a área verdadeira (`quina_exacta`, supersamostragem `64²`). Não é regressão.
3. **TUDO-OU-NADA por quadro, de propósito:** uma imagem, uma mistura, uma tinta própria ou um traço
   tracejado esticado devolvem o QUADRO inteiro ao Vello — partir a lista trocaria a ordem entre as
   duas metades.
4. **As cópias CONFORMES pequenas não vão às células** (`AREA_MINIMA_CONFORME = 1024 px²`): ir com
   todas regredia a escada de `32 768` (`9,70 → 13,53 ms`). A janela medida é `[256, 4096)`.
5. **A capacidade do cálculo é MEDIDA dois quadros depois** e, até lá, uma cópia que não cabe é
   desenhada pelo caminho de sempre **por cópia** — nunca um contorno truncado. No 1.º quadro de uma
   cena nova a mistura dos dois caminhos é normal e desenha a mesma imagem (gate).
6. **As portas `PH2D_CARIMBO_PREPARADO=0` e `PH2D_LOD_DA_FORMA=0` deixaram de tocar na rota de
   omissão** (são da rota da CPU); a porta que bissecta é `PH2D_FORMAS_NA_PLACA=0`, e o roteiro da
   `=126` foi reescrito à volta dela, com gate.
7. **A ponte deixou de recusar pelo TIPO do nó** (`graph_has_live_vector_source` e a recusa da
   instância condicional foram APAGADAS da `cook_gpu`) — a cerca é de CONTEÚDO e corre antes do plano.

## §4 — ⛔ Premissas minhas que a medição derrubou (nesta jornada)

- *«Um segmento horizontal contribui `dy = 0`, salta-se»* — verdade só no ECRÃ; no espaço local de
  uma cópia rodada abria o contorno (riscos horizontais). Nenhuma forma do gate da W1 tinha aresta
  horizontal.
- *«O `−1e-6` do Vello impede o `0/0`»* — verdade num ladrilho de `16 px`; relativo ao PIXEL as
  coordenadas chegam a centenas e o `NaN` pintava uma fileira inteira (a linha da foto do dono). O
  preenchimento tinha o mesmo defeito desde a W1, e quem o expôs foi uma mutação sobrevivente.
- *«A faixa nos pontos lisos basta»* — a sonda (estrelas arredondadas) melhorou `19 %` e o app (a
  `=127` tem estrelas de quinas vivas) não se mexeu: *uma sonda cuja forma não é a da cena mede
  outro programa*.
- *«O caminho novo para todas as cópias»* — regrediu a escada de estrelas pequenas conformes; daí a
  rota por área.
- *«A sonda de relógio diz a verdade»* — com o shader partido ela leu `0,005 ms` e saiu verde; hoje
  lê a camada de volta e reprova sem tinta.

**Recusas MEDIDAS** (no doc 121, cada uma com a tabela): o teste no espaço LOCAL (§9.2) · a árvore
de blocos no eixo (§9.3) · herdar a bissectriz (§9.4) · costurar as correntes por passo guloso
(§9.5) · partir as arestas longas em pedaços de `32 px` (§9.6).

## §5 — ⏳ O QUE FICA ABERTO (e de quem é)

| item | dono |
|---|---|
| **as estrelas GRANDES no proxy de telemóvel** perdem para o Vello (`2,32` contra `0,98 ms` esticadas; `1,35` contra `0,90` conformes) — o cálculo corre num fio por cópia e os pixels de borda lêem a célula inteira; a cura tem endereço (células construídas em paralelo por bloco + blocos partilhados na memória do grupo, como o rasterizador fino do Vello) | linha Motion |
| os **glifos** do `source.text` continuam no Vello | linha Motion |
| o **traço tracejado** sob escala não-uniforme fica no Vello (pede comprimento de arco por cópia) | linha Motion |
| a **W0/W5 como tabela única** em máquina calma: o A/B na mesma build está no §9.3 do doc 121 e é a partida; falta repeti-la depois das células com `load < 4` | linha Motion |
| `M6` do contorno calculado e `S6`/`S8` da faixa: sobreviventes NOMEADAS (relógio puro / cercas abaixo do ruído de aplanamento) | nomeado |
| a camada do `fx.glow` num quadro do dispositivo lê o `pump` do quadro anterior — **pré-existente** e inalcançável com formas (o glow com formas recusa a placa) | nomeado |

## §6 — Os gates que CRESCERAM nesta linha

- `ph2d-shape-gpu` (todos `#[ignore]`, com adaptador): `paridade_com_o_vello` (14 famílias, regime do
  produto: 3 quadros e todas as cópias no ecrã) · `contorno_calculado` (14 famílias pelos dois
  caminhos a alfa `≤ 1` + `so_as_copias_conformes_grandes_pagam_o_calculo`) · `quina_exacta` — mais
  os testes de unidade de `blocos`, `eixo`, `geometry`.
- `ph2d-gpu-cook`: `as_formas_no_dispositivo` (`#[ignore]`) · `lower_forma_tests` · a validação WGSL
  (`32` subconjuntos × `2` pivôs).
- `ph2d-app-motion`: `motion_shape_placa_tests` + `_gpu_tests` (paridade de PIXEL do produto pelas
  duas rotas) · `motion_bridge_gpu_formas_tests` · `motion_state_traco_esticado_demo_tests`.
- `shells/desktop`: `present_placa_tests` (a fiação) e a ordem das recusas.

**Provas de mutação** (arneses versionados em `docs/Motion Nodes/ferramentas/`): W3 `11/11` · W4
`15/15` · §9.2 `7/7` · §9.3 `11/11` · faixa `9/11` + 2 NOMEADAS · contorno `5/5` + 1 NOMEADA (re-ancorado e re-corrido no fecho, §7) ·
células `10/10`.

## §7 — A PROVA DE FECHO (corrida nesta árvore, sobre o `main` @ `912a9652e`)

⚠️ A máquina esteve a `load 20`–`70` a corrida inteira (outras linhas).

| portão | resultado |
|---|---|
| `nextest-impacted.sh` (`BASE` = merge-base, `ci-test`) | **16 734 / 16 737** — os três ✗ abaixo |
| ↳ `the_pass_aa_is_never_chosen_by_a_text_preference` (`ph2d-render`) | **DEFEITO DESTA LINHA, curado no fecho:** o teste de paridade montava um `vello::Renderer` próprio e nomeava o `AaConfig`, que pertence a quem possui o renderer. Hoje ele rasteriza pelo `ph2d_render::VelloPass` do PRODUTO (dev-dependency nova, sem ciclo) — verde, e a paridade com o Vello continua **5 / 5** na placa |
| ↳ `a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` (`ph2d-app-flip`) | membro NOMEADO da família de flakes de fan-out do §5.0 (`flip_smooth::resample_measurement::precisao::orcamento`); **3 de 3 verde sozinho a `load 26`–`41`**; zero linhas do diff naquela crate |
| ↳ `tres_bonecos_tres_amplitudes_e_o_rapido_acende_a_lampada` (`ph2d-app-components`) | ⚠️ **NÃO está na lista** do §5.0 e é para promover: reprovou com os três bonecos parados (`[0,0,0]`) a `load ~60`, passa **3 de 3 sozinho a `load 26`–`41`**, e a crate tem **zero** linhas do diff. Mecanismo provável: o prazo de um quadro por gancho do Luau (#16) mede RELÓGIO — sob fan-out o gancho estoura e o objecto pára |
| `cargo clippy -p <as 8 crates tocadas> --all-targets -- -D warnings` | ✅ |
| `cargo fmt --all -- --check` | ✅ (a 1.ª corrida apanhou `contorno_calculado.rs` por formatar — curado) |
| `censos-da-arvore-combinada.sh` (HR-15 + tectos, sobre o `main`) | ✅ **127 / 127**, controlo do filtro `12 de 12` |
| `doc-index.sh --check` | ✅ — e o índice manual da pasta `handoffs/` estava parado em 18/09: faltavam **três** handoffs (`09-19`, `09-20`, `09-24`), acrescentados com este |
| GPU com adaptador (RTX, `#[ignore]`) | ✅ `ph2d-shape-gpu` **5 / 5** (paridade · contorno calculado · quina exacta · NaN vertical · rota) · `ph2d-app-motion` paridade do produto **3 / 3** |
| pré-voo dos três arneses de mutação | ✅ `10/10` · `11/11` · `6/6` — ⚠️ o do §9.5 tinha **DUAS âncoras MORTAS** pela própria §9.6 (a verificação da reserva e o telescópio mudaram de sítio) e só abortava; re-ancoradas e o arnês **corrido de novo na placa: 6 de 6 como esperado** (5 sangram, a `M6` nomeada sobrevive) |

⚠️ **Um acidente de fecho, registado:** um laço meu chamou o arnês das células SEM o
`MUTA_SO_ANCORAS` e o da faixa (que usa `--so-ancoras`, outra bandeira) correu inteiro; os dois
foram mortos pelo PID a meio e deixaram uma mutação no `shape.wgsl`, **reposta do `HEAD` com
`touch`** (`git diff HEAD -- crates/ph2d-shape-gpu/src/` vazio) antes de qualquer medição seguinte.
Nenhuma linha da tabela correu com o shader mutado: as que vieram antes do acidente mediram o
`shape.wgsl` do `HEAD`, que é o que lá está hoje, e o arnês do §9.5 correu depois da reposição.

## §8 — OS SMOKES (o comando inteiro, copiável)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=127 PH2D_TRACO_ESTICADO_DENSO=1 cargo run -p ph2d-host-desktop --release
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=127 PH2D_TRACO_ESTICADO_DENSO=1 PH2D_FORMAS_NA_PLACA=0 cargo run -p ph2d-host-desktop --release
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=127 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=126 cargo run -p ph2d-host-desktop --release
```

(`=127` densa: o relógio — `raw` pela placa contra o Vello · `=127` legível: o contorno das estrelas
esticadas, que tem de ser redondo e igual nos lados compridos e curtos · `=126`: a galáxia de
estrelas, agora pela placa do dispositivo.) Diagnóstico: `PH2D_MOTION_ROUTE_LOG=1` diz a rota de
cada quadro; `PH2D_CONTORNO_CALCULADO=0` e `PH2D_AREA_MINIMA_CONFORME=<px²>` são instrumentos de
bissecção.

## §9 — A UMA LINHA proposta para o `CLAUDE.md` §5 (o integrador aplica)

> ⭐⭐⭐ **E AS FORMAS DO MOTION VÃO À PLACA (01/10, [doc 121](docs/Motion%20Nodes/121_as_formas_na_placa.md) + [handoff](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-01.md), smoke do dono aprovado):** crate nova `ph2d-shape-gpu` — um passe instanciado com a cobertura por área PORTADA do Vello, pelas rotas da CPU e do dispositivo, incluindo o traço sob escala não-uniforme (cena `=127`); no proxy de telemóvel a `=127` densa vai de `35,2` a **`18,76 ms`**, à frente do Vello; `PH2D_FORMAS_NA_PLACA=0` bissecta. ⚠️ `LOWER_COLUMNS` passa a `[_; 9]` (§2.1). ⏳ estrelas GRANDES no proxy, glifos e tracejado esticado ficam no Vello.
