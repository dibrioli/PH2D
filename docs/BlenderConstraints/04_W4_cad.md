# W4 — CAD no vetor (26/10/2026)

> Plano: [`01_plano.md`](01_plano.md) §3 (schema do vetor), §5, §7 W4 e o kill-criterion. Estado: **W4.0 e W4.1 medidas;
> o dono disse «pode seguir» (26/10) à ezpz** — o código abre pela W4.2 (§3).
> Bancada: [`ferramentas/bancada_solvers/`](ferramentas/bancada_solvers/) (`mede.sh`; uma cena por processo) · entradas
> NOSSAS: [`fixtures/w4_cenas/`](fixtures/w4_cenas/) · oráculo: [`ferramentas/oraculo_freecad.py`](ferramentas/oraculo_freecad.py)
> → [`fixtures/freecad_1_1_4_sketch_arrasto.json`](fixtures/freecad_1_1_4_sketch_arrasto.json) · números brutos:
> [`reports/w4_medicao/`](reports/w4_medicao/).

## 0. O que o artista vai conseguir fazer

No modo Edit do Vetor: prender pontos e segmentos com regras de desenho técnico (coincidente, horizontal/vertical,
paralelo, perpendicular, tangente, igual, simétrico, medida) e **arrastar** um ponto com o resto da forma a obedecer;
as regras nascem sozinhas ao desenhar com as guias; a cor diz o que está livre, o que está definido e o que está em
conflito; uma medida a mais nasce **REF** (só mostra); um conflito lista «apague uma destas»; a medida **anima-se**.

## 1. Medido antes de desenhar (W4.0)

**Método** ([`MEDIR_VELOCIDADE.md`](../DevOps/MEDIR_VELOCIDADE.md)): as variantes vivem no MESMO processo, aquecidas
fora da régua; **7 rodadas** de blocos intercalados com a ordem rodada; o bloco é um **arrasto de 60 passos** do mesmo
ponto num círculo de 40 px (≈ 4,2 px por passo, um rato real a 60 Hz), cada passo a partir da solução anterior; resumo
= **mínimo** das rodadas com a mediana ao lado. Perfil `release` da bancada = cópia do `smoke` do workspace. A régua de
cumprimento é NOSSA e a mesma para todos (`violacao` em px; tolerância `1e-4` px). `loadavg` 11–18 em 32 núcleos
durante as corridas (outras linhas a compilar): o número absoluto é teto pessimista; o «A × B» é justo pela intercalação.

**Cenas** (geradas por `bancada_solvers -- cenas`, verificadas a violação 0 na pose de partida):
- `escada_N` — perfil rectilíneo fechado de dentes: H/V em todo segmento, larguras iguais, alturas cotadas a cada 3.º
  dente, 1 ponto fixo; sub-restringido (o estado normal enquanto se desenha). Arrasta-se o canto de um dente do meio.
- `misto_N` — módulos de 10 pontos (rectângulo rodado por paralelo/perpendicular/medida, ponto sobre aresta, par
  simétrico, recta tangente a um círculo, coincidência) encadeados por paralelo e igual: arrastar um canto **roda todos
  os módulos** — o caso acoplado, o pior para decompor.

### Tabela (pior passo do arrasto, ms — mínimo · mediana de 7 rodadas)

| cena | ezpz | fiksi LM | fiksi LM passagem única | FreeCAD 1.1.4 (referência) |
|---|---|---|---|---|
| escada 50 pts | **0,155** · 0,164 | 0,511 ✗ | 0,369 ✗ | 0,683 |
| escada **202** pts | **0,684** · 0,704 (2.ª corrida 0,781) | 2,110 ✗ | 1,655 ✗ | 9,54 |
| escada 802 pts | **10,8** · 10,9 | 6,49 ✗ | 5,87 ✗ | 389 |
| misto 50 pts | **0,220** · 0,228 | 0,167 ✗ | 0,171 ✗ | 4,67 |
| misto **200** pts | **0,710** · 0,720 (2.ª corrida 0,812) | 1,388 ✗ | 0,931 ✗ | 187 |
| misto 800 pts | **2,51** · 4,44 | 28,2 ✗ | 10,8 ✗ | 13 679 |

✗ = **não cumpre as regras** (60 de 60 passos acima de 1e-4 px; §1.1). ezpz: pior violação ≤ **8,9e-10 px** em todas,
0 passos fora, **determinista** (as 7 rodadas dão as mesmas posições ao bit). FreeCAD: ≤ 4,5e-7 px; o tempo dele é
limite SUPERIOR (`SketchObject.moveGeometry` reinicia o arrasto a cada chamada) — serve de escala, não de concorrente;
duas corridas dele dão as MESMAS posições ao bit (a fixture guarda a 2.ª; os tempos variam ±15 %), e ele não marca
nenhuma regra redundante nem em conflito nas seis cenas.

**Kill-criterion (plano §7: «200 pontos em < 2 ms no arrasto»): PASSA com ezpz** — 0,68–0,81 ms no pior passo nas
duas cenas de 200, em duas corridas separadas. O teto medido (sem `MAX_*` inventado): 800 pontos acoplados = 2,5–4,4 ms;
800 rectilíneos = 10,8 ms (o custo cresce mais que linear na escada). Um limite de pontos, se um dia for preciso, sai
DESTA tabela.

**Custos fora do arrasto** (só quando a ESTRUTURA muda — juntar/apagar uma regra), mínimo de 3:

| cena | ezpz: montar + 1.º passo | ezpz: análise de graus de liberdade (`solve_analysis`) |
|---|---|---|
| 200 pts (escada · misto) | 0,20 · 1,78 ms | 9,1–9,9 · 11,5–11,9 ms |
| 800 pts (escada · misto) | 1,67 · 7,59 ms | 456 · 572 ms |

⇒ a análise (as cores livre/definido) **não** corre no arrasto: corre ao largar o gesto ou ao mudar a estrutura; a 800
pontos (½ s) sai do caminho do quadro (a fazer em W4.4: calcular fora do fio do quadro e pintar quando chega).

**Distância à solução do FreeCAD** (passos 15/30/45/60): ezpz 0,08–1,2 px na escada, ≈ 7 px no misto — as duas cumprem
as regras ao 1e-7; um sketch **sub-restringido** tem infinitas soluções e cada solver escolhe a sua. ⇒ a posição do
FreeCAD só é gate onde a solução é **única** (§4, cenas bem-restringidas a gerar em W4.1).

### 1.1 ⛔ Recusas MEDIDAS

| recusado | medição | porquê |
|---|---|---|
| **fiksi** (todas as variantes) | depois de um passo de arrasto: pior violação 0,13 px (misto 50) → 47 px (escada 202) → 97 px (escada 802); **60/60 passos** fora de 1e-4 px | o adaptador está certo: partindo da pose exacta, sem perturbar, a violação é **2e-13 px** (`diag`); com a perturbação por omissão (`perturb: true`) já sai dela (3e-2 px) sem nada mexer. Chamar o `solve` **10× por passo** dá as MESMAS posições (`fiksi_lm_10x`, 4–7 ms): ela pára num ponto que julga resolvido. `SolvingOptions` não tem tolerância. Hipótese NÃO medida: o critério de paragem é relativo à escala da cena. Um CAD que deixa 4 px de folga num H é pior que não ter regra (memória `feature_worse_than_not_existing`) |
| fiksi `Decomposer::SinglePass` | `determinista = false` nas 6 cenas | o mesmo arrasto dá posições diferentes de rodada para rodada ⇒ arrastar a timeline não daria o mesmo bit |
| fiksi `Optimizer::LBfgs` | escada 802: 56,5 ms no pior passo; **misto 50: uma só chamada `solve` > 25 min** (morta pelo prazo) | pendura; tirada das cenas misto em `mede.sh` |
| fiksi `Decomposer::RecursiveAssembly` | não medido | a doc dela diz que não trata pontos fixos ao referencial — todo sketch nosso tem um |
| FreeCAD/planegcs como motor | 9,5 ms (escada 202), 187 ms (misto 200), 13,7 s (misto 800) por passo | LGPL e C++; é o **oráculo**, corrido, nunca lido nem ligado |
| gate «posições = FreeCAD» num sketch sub-restringido | Δ 0,08–7 px com as duas exactas | a resposta não é única; o gate é «cumpre + não salta» |

### 1.2 Triagem de licença (por artefacto, `cargo metadata` da bancada + `pacman -Qi`)

| artefacto | licença | árvore | nota |
|---|---|---|---|
| `kcl-ezpz 0.2.18` (crates.io, KittyCAD) | MIT | 76 pacotes: MIT, MIT/Apache, Unlicense/MIT, Zlib, BSD-2, Unicode-3.0 — **todas** na lista `allow` do `deny.toml` | ~30 crates novas para o workspace (família `faer 0.23` + `gemm 0.18` + `nano-gemm`, `qd`, `dyn-stack`, `equator`, `generativity`, `mutants`); o custo de compilação mede-se em W4.3 |
| `fiksi` @ `ff48505` (git, endoli) | Apache-2.0 OR MIT | 27 pacotes; `colamd_rs` BSD-3; `kurbo 0.13.1` = o do workspace | recusada por medição (§1.1), não por licença |
| FreeCAD 1.1.4 (pacman) | LGPL-2.0-only | — | corrido por script; o fonte não foi aberto |

### 1.3 O que a ezpz NÃO tem (o desenho tem de cobrir)

- Tangência **orientada** (`LineTangentToCircle` distingue o lado): a regra grava o lado em que nasceu — escolher a
  ordem da recta que cumpre na pose de partida (a bancada faz assim, `tangente_orientada`).
- Ponto sobre recta = `PointLineDistance(_, _, 0)` **com sinal** (não troca de lado ao arrastar — bom; é a «salto de
  ramo» que o relatório aponta como a dor do CAD).
- Sem tangência a curva Bézier: a W4 dá tangência a **recta↔arco/círculo** e a **continuidade de alça** (G1) entre
  segmentos Bézier é a alça Smooth que o vetor já tem (`VertexKind::Smooth`, `ph2d-vec-scene/src/lib.rs:391`) — medir
  em W4.3 se uma regra «tangente» entre dois segmentos curvos é só isso.
- A API é sem estado (`solve(regras, palpites, config)`): o sistema remonta-se a cada passo — está DENTRO dos 0,7 ms.

### 1.4 W4.1 — solução ÚNICA e conflito (oráculo antes do código, 26/10)

Cenas `bem_*` (0 graus de liberdade — **contados pelo FreeCAD**, `DoF = 0` em todas): retângulo H/V cotado, retângulo
rodado por paralelo/perpendicular, recta tangente a um círculo, par simétrico, composto (sobre a recta + coincidente +
igual + perpendicular) e um perfil de 12 dentes com 98 regras. Cada uma em dois casos: partir de uma pose **errada**
(±3 px em cada ponto livre, `__partida`) e mudar a 1.ª medida em +10 % (`__medida`, o caso da medida animada). Cenas
`conflito_*`: uma medida a mais coerente, uma incoerente, e H+V+medida no mesmo segmento. `bancada -- bem` →
[`reports/w4_medicao/bem.json`](reports/w4_medicao/bem.json); a fixture guarda `bem.<cena>` com posições, `DoF`, e os
redundantes/conflitos do Sketcher **traduzidos para índices das nossas regras**.

| caso | FreeCAD 1.1.4 | ezpz 0.2.18 |
|---|---|---|
| 11 casos de solução única | violação ≤ 1,1e-13 px | violação ≤ 3e-9 px · **Δ posição ≤ 1,1e-8 px** do FreeCAD |
| `conflito_redundante` (medida a mais, coerente) | `solve = -2`, **redundante = [regra 6]** | cumpre tudo, **não avisa** (sem aviso, sem lista) |
| `conflito_inconsistente` (+10 px na medida a mais) | `-4`, conflito = **[1, 3, 4, 6]** | `unsatisfied` = **[1, 3, 4, 6]** — a MESMA lista |
| `conflito_hv` | `-4`, conflito = **[1, 2]** (V e a medida) | `unsatisfied` = **[1, 2]** — a MESMA lista |

⇒ **«apague uma destas»** = a lista `unsatisfied` da ezpz (igual à do FreeCAD nos dois casos). ⇒ **REF** não vem da
ezpz (a `FreedomAnalysis` só dá variáveis sub-restringidas, não o posto): é um teste de **posto** do jacobiano nosso — a
regra nova não sobe o posto ⇒ nasce REF — com a álgebra `faer` que a ezpz já traz; oráculo = `redundantes_regras`. ⇒ As
posições do FreeCAD **são gate** nas 11 cenas de solução única.

## 2. O lado do vetor (lido no código, 26/10)

- `VecVertex` (`crates/ph2d-vec-scene/src/lib.rs:403`) **não tem id**: é endereçado pelo índice em `VecPath.verts`
  (`:476`). O único id estável é `VecPathId = u64` (`:462`), contador `next_id` na `VecScene` (`:581`, `push_path`
  `:603`).
- `VEC_SCENE_SCHEMA_VERSION = 23` (`schema.rs:90`); postcard posicional com a versão à frente (`lib.rs:676–691`); o
  único degrau antigo legível é o v22 (`stroke_wire.rs:26`, teste `tests/it/stroke_v22_fixture.rs:64`).
- ~20 sítios inserem ou removem vértices: pen (`ph2d-vec-edit/src/lib.rs:357`), inserir no segmento (`node_hit.rs:105`
  → `geometry.rs:239`), apagar (`selection.rs:594`), média/juntar/inverter (`node_ops.rs:54,117,143`), simplificar
  (`reshape.rs:73`), soldar/partir (`weld.rs:52`), booleanas (`ph2d-vec-boolean`: `cut.rs:129`, `esporao.rs:231`,
  `overlap.rs:128–295`) e as substituições inteiras (`shape.rs:212`, `pencil.rs:328`, `ph2d-app-vec`: `weld.rs:237`,
  `shape_live.rs:102`, `morph_live.rs:213`, `connector_live.rs:220`, `bucket.rs:404`, `blend_live.rs:224`,
  `ph2d-vec-envelope/src/lib.rs:94`).
- O Copy Point da W3 (`ph2d-constraints/src/poderosas.rs:127`, `PontoDeForma { indice }`) lê `Pontos[id][indice]`
  (`ph2d-constraints-ecs/src/avalia.rs:180`), produzido de `p.verts` (`ph2d-app-constraints/src/lib.rs:89`) — **parte
  ao inserir um ponto antes dele**.
- Arrasto de âncora: `ph2d-vec-edit/src/pen_drag.rs:17` (`on_drag(.., snap: &mut dyn FnMut)`), grab em `lib.rs:675`;
  guias inteligentes em `snap.rs:329`, ponte `ph2d-app-vec/src/snap.rs:148`.

## 3. O desenho e a ORDEM (a fazer — cada passo abre com o gate vermelho)

| passo | o quê | porta única |
|---|---|---|
| ~~**W4.1** oráculo antes do código~~ | **feito 26/10** (§1.4) | `oraculo_freecad.py` (`W4_SO_BEM=1`) |
| **W4.2** id estável por vértice | `VecVertex.id: u32` + contador por path (nunca reusado); **+1 → `VEC_SCENE_SCHEMA_VERSION = 24`** com leitor do v23 que numera `0..n` pela ordem; cada um dos ~20 sítios: inserir = id novo, partir = id novo, apagar = o id some e as regras que o citam saem no MESMO passo de undo; booleana/substituição inteira = path novo, ids novos, regras do path antigo saem com aviso. **Migra o Copy Point** para (alvo, id do vértice) — fecha também o ABERTO «prender a ponto de forma por índice» da W3 | `ph2d_vec_scene::VecPath::vert_by_id` (nova) |
| **W4.3** a crate do solver | `ph2d-vec-constraints` (NOVA, pura): `RegraCad` (coincidente, H, V, paralelo, perpendicular, tangente recta↔círculo, igual, simétrico, medida de distância/ângulo/raio) por (path, id de vértice); tradução para `kcl_ezpz` (`=0.2.18`); arrasto = `Fixed` do ponto agarrado + palpite da solução anterior; `estado()` = livre/definido/conflito/REF; mede o custo de compilação da família `faer` | `ph2d_vec_constraints::resolve(&VecPath, &[RegraCad], arrasto) -> Resultado` |
| **W4.4** no editor | onde guardar: as regras viajam no documento do vetor (`VecScene`, mesmo degrau 24); o arrasto chama `resolve` em `pen_drag.rs`/grab **depois** do snap (o snap dá o alvo, o solver o resto); inferência = a guia que estava acesa ao largar cria a regra; cores por token (zero hex); medida a mais nasce REF; conflito lista as regras de `estado()`; análise fora do fio do quadro; undo = 1 passo por gesto | `ph2d-app-vec` (nada na shell) |
| **W4.5** medida animada + cena 6 | a medida é um canal da família `CONSTRAINT = 8` (`campo` = valor, `i:j` = id do item); a pose resolvida parte SEMPRE do desenho AUTORADO em cada quadro (não do quadro anterior) ⇒ tocar = arrastar a timeline ao bit; chaves pela porta `ph2d_app_constraints::chaves::gravar` | `ph2d_vec_constraints::resolve` |

### 3.1 W4.2 em detalhe (desenhado 26/10, lido no código — o código ainda NÃO abriu)

- **Onde mora o id:** `pub id: VertId` **dentro** do `VecVertex`, apendado ao FIM (postcard posicional) — a mesma lei do
  `corner_radius` (`ph2d-vec-scene/src/lib.rs:405-427`): inserir, apagar, inverter, soldar e partir levam-no de graça,
  sem tocar nos ~20 sítios do §2. `VertId(u32)`, `VertId::NOVO = 0` = «ainda sem id» (o que todo construtor e literal
  dá). `VecPath.next_vert: u32` apendado ao FIM do `VecPath` (contador por path, monotónico, nunca reusado; os
  contornos de `subpaths` partilham o espaço de ids do path).
- **Quem dá o id:** `VecPath::normaliza_ids()` — dá `next_vert` a cada `NOVO` e à 2.ª ocorrência de um id repetido
  (um vértice duplicado por uma operação), pela ordem `verts` → `subpaths`; idempotente. `VecScene::normaliza_ids()` em
  todos. Chamadores: `VecScene::push_path` e o gancho que JÁ corre antes de toda captura de undo —
  `settle_tree_before_capture` (`shells/desktop/src/vec_tree_settle.rs:63`, chamado de `undo_app.rs:448,538` antes de
  `capture_project` `:86`): toda edição do vetor vira passo de undo ali (`post_frame_undo`, `undo_app.rs:390`). A
  `VecScene` é UMA por projeto (`app_state_gfx.rs:158`). Um ficheiro antigo ganha ids `1..n` no 1.º assentamento.
- **Literais:** ~198 `VecVertex {` em 95 ficheiros (24 na shell). Na shell trocam-se pelos construtores
  (`VecVertex::corner`/`::smooth` + um `::with_handles(anchor, in, out, kind)` novo) — a shell ENCOLHE e paga a linha do
  `normaliza_ids` no gancho; nas crates junta-se `id: VertId::NOVO`.
- **Ler o antigo** (padrão do `stroke_wire.rs`, F16c): âmbito por thread `GeometryLayoutV23` com `Deserialize` à mão do
  `VecVertex` e do `VecPath` (espelhos V23/V24 + gate `the_live_layout_round_trips`); `VEC_SCENE_SCHEMA_VERSION 23 → 24`;
  `from_bytes`: v24 vivo · v23 = âmbito geometria · v22 = traço **e** geometria. Projeto: `PROJECT_SCHEMA 194 → 195`
  (reconta-se na integração), `project_load_read.rs:57` abre o âmbito da geometria para `ver <= 194`, e as fontes
  guardadas DENTRO de componentes regravam-se no load como o `migra_traco_v22` já faz (`ph2d-app-vec/src/envelope_live.rs:505`,
  `ph2d-skeleton-live/src/skinned_mesh.rs:205`) — generalizar as duas para o âmbito composto. ⚠️ **1.º passo da W4.2:
  censo de TODO portador gravado de `VecPath`/`VecVertex`** (poses de UI, `.ph2dmacro`, símbolos, área de transferência
  e o que mais houver) — cada um precisa de leitor antigo ou de regravação.
- **Copy Point:** a W1–W3 ainda **não está no main** ⇒ o `PontoDeForma { indice }` (`ph2d-constraints/src/poderosas.rs:127`)
  troca para `{ vert: VertId }` SEM migração; o recurso `Pontos` (`ph2d-app-constraints/src/lib.rs:89`) passa a
  `(VertId, âncora)` e cobre também os `subpaths`; id que sumiu ⇒ `Estado::SemAlvo`.

## 4. Gates red-first

| gate | fenómeno na fixture | passo |
|---|---|---|
| `a_well_constrained_sketch_lands_where_freecad_puts_it` | cenas de solução única da W4.1, posições ao 1e-6 | W4.3 |
| `the_drag_keeps_every_rule_at_every_step` | `w4_cenas` escada 202 e misto 200, 60 passos, 1e-4 px | W4.3 |
| `the_drag_is_deterministic` | o mesmo arrasto 2× ⇒ o mesmo bit | W4.3 |
| `a_vertex_keeps_its_id_when_a_neighbour_is_deleted` | apagar o 3.º de 10: os outros ids iguais; a regra no 5.º continua no mesmo ponto | W4.2 |
| `a_v23_scene_opens_with_ids_in_order` | bytes v23 gravados ⇒ ids `0..n`, ida e volta v24 | W4.2 |
| `copy_point_follows_its_vertex_after_an_insert_before_it` | hoje **vermelho** (índice) | W4.2 |
| `each_vertex_mutation_site_keeps_or_assigns_ids` | censo dos ~20 sítios (§2), um caso por sítio | W4.2 |
| `an_extra_dimension_is_born_ref` (teste de posto) · `a_conflict_lists_the_rules_involved` (`unsatisfied`) | `conflito_*` da W4.1: REF = [6]; conflito = [1,3,4,6] e [1,2] | W4.4 |
| seam `draw_with_a_guide_creates_the_rule` (`ph2d-ui-testkit`) | caneta com a guia H acesa ⇒ regra H gravada, undo 1 passo | W4.4 |
| `an_animated_dimension_plays_and_scrubs_to_the_same_bit` | medida com 3 chaves, quadros 0→40→12→40 | W4.5 |
| prova de mutação | tirar o `Fixed` do arrastado / a migração / o REF ⇒ o gate certo sangra | fecho |

A velocidade **não** vira teste (relógio sob carga mente): reconfere-se com `mede.sh` no fecho da onda, a tabela ao lado.

## 5. Cena 6 (conte: o roteador `PH2D_CONSTRAINT_SMOKE` responde até **5**, `ph2d-app-constraints/src/smoke.rs:23`)

**6 — «Régua de desenho»**: uma forma vetorial (perfil com dentes, como `escada_50`) com H/V, larguras iguais e uma
medida animada. O dono arrasta um canto no modo Edit e o resto obedece; dá Play e a medida estica o dente sem desfazer
as outras regras; a cor mostra o que ainda está livre. Números antes da mensagem: violação ≤ 1e-4 px em todos os
quadros, play × scrub = 0, pior passo do arrasto no app. Foto antes da mensagem.

## 6. Armadilhas desta onda

- A bancada vive FORA do workspace (o seu `Cargo.toml` tem `[workspace]` vazio) e compila para `target/bancada_solvers`
  desta worktree — nunca partilhar o `target` com outra worktree.
- Um solver pode pendurar DENTRO de uma chamada: o limite por bloco não o pára; por isso `mede.sh` corre uma cena por
  processo com `timeout`.
- `SketchObject.movePoint` não existe no FreeCAD 1.x (é `moveGeometry`, que devolve `None` — o sucesso prova-se pela
  violação); o sketcher só tem geometria depois do 1.º `solve()`.
