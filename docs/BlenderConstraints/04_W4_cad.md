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
| **W4.3** a crate do solver | `ph2d-vec-constraints` (NOVA, pura): `RegraCad` (coincidente, H, V, paralelo, perpendicular, tangente recta↔círculo, igual, simétrico, medida de distância/ângulo/raio) por (path, id de vértice); tradução para `kcl_ezpz` (`=0.2.18`); arrasto = `Fixed` do ponto agarrado + palpite da solução anterior; `estado()` = livre/definido/conflito/REF; mede o custo de compilação da família `faer` — **feito 26/10** (§3.3) | `ph2d_vec_constraints::resolve(&Folha, &[RegraCad], Option<Arrasto>, palpite) -> Resultado` |
| **W4.4** no editor | onde guardar: as regras viajam no documento do vetor (`VecScene`, mesmo degrau 24); o arrasto chama `resolve` em `pen_drag.rs`/grab **depois** do snap (o snap dá o alvo, o solver o resto); inferência = a guia que estava acesa ao largar cria a regra; cores por token (zero hex); medida a mais nasce REF; conflito lista as regras de `estado()`; análise fora do fio do quadro; undo = 1 passo por gesto | `ph2d-app-vec` (nada na shell) |
| **W4.5** medida animada + cena 6 | a medida é um canal da família `CONSTRAINT = 8` (`campo` = valor, `i:j` = id do item); a pose resolvida parte SEMPRE do desenho AUTORADO em cada quadro (não do quadro anterior) ⇒ tocar = arrastar a timeline ao bit; chaves pela porta `ph2d_app_constraints::chaves::gravar` | `ph2d_vec_constraints::resolve` |

### 3.1 W4.2 em detalhe (desenhado 26/10; **feito 26/10** — o que mudou no caminho vai em negrito)

- **Feito:** `VertId` + `next_vert` (`ph2d-vec-scene/src/vert_id.rs`); o âmbito do `stroke_wire` generalizou-se de «é v22» para
  **a versão dos bytes** (`WireLayout::enter(ver)` / `with_wire_layout`, aninhados valem o MAIS ANTIGO) — um só âmbito, e não um
  `GeometryLayoutV23` irmão; o projeto escolhe-o por `fio_do_projeto(ver)` (`22` até ao `189`, `23` até ao `194`) e os blobs por
  `migra_fio(mundo, fio)` (era `migra_traco_v22`). `PROJECT_SCHEMA 195`, `VEC_SCENE 24`, tripla `(195, 13, 24)`. Literais: nas
  crates, `id: v.id` onde transformam UM vértice-fonte (afim, espelho, solda, achatar/suavizar, junção de B em A no `vec-edit`) e
  `NOVO` na geometria nova; na shell, `VecVertex::with_handles` (−42 linhas).

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

### 3.2 W4.3 — decisões tomadas lendo o código (26/10, antes do código)

- **O «círculo» é um PATH cuja GEOMETRIA é um círculo**, reconhecido pelos vértices e não pela receita: fechado, sem
  `subpaths`, ≥ 3 âncoras à mesma distância do circuncentro das três primeiras e cada alça tangente (⟂ ao raio) — é o
  que `shapes::ellipse(c, r, r)` e o `ellipse_sweep` de volta inteira (`round.rs:99`, 4 vértices `Smooth`) cozinham, e
  continua a sê-lo depois de *Convert to Curves* (a receita some, a geometria fica) e de uma rotação. **Porquê não a
  receita** (`VecShape::Param { kind: Ellipse, w == h }`, `shape_live.rs:36`): ela mora no ECS, que a crate pura não
  vê, e mente depois de um Convert; **porquê não elipse**: a ezpz não tem elipse — `Tangente`/`Raio` sobre uma elipse
  (`rx ≠ ry`, ou um afim de escala não-uniforme) ⇒ a regra fica **`SemAlvo`** (a cor diz porquê), nunca uma
  aproximação calada. Arco (Pie/sweep parcial) fica fora: a W4 dá recta↔círculo (§3).
- **Datum = centro + raio, os dois VARIÁVEIS** (`DatumCircle`), endereçados por `Ponto::Centro(path)` — o centro
  entra em qualquer regra de ponto (coincidente, medida, fixo), como o ponto do centro no Sketcher; `Raio` é a regra
  que o prende. **Escrever de volta:** cada vértice do círculo (âncora e alças) vai por
  `c' + (r'/r)·(p − c)` — guarda os ids, os `kind`, a fase da rotação e as alças KAPPA; actualizar a receita
  (`w = h = 2r'`, o `Transform`) é do editor (W4.4, `ph2d-app-vec`).
- **A porta é uma FOLHA de vários paths**, não um `&VecPath`: uma regra endereça `(VecPathId, VertId)` e a tangente
  liga uma recta de um path ao círculo de OUTRO. `Folha = [(&VecPath, Xform local→folha)]` (as formas vivas são
  centradas no local 0 com o centro no `Transform`, `shape_live.rs:43`); o resultado volta ao local pelo inverso do
  afim. Assinatura: `ph2d_vec_constraints::resolve(&Folha, &[RegraCad], Option<Arrasto>, palpite) -> Resultado`.
- **Alças acompanham a âncora** pelo mesmo delta local (o vértice move-se rígido; a forma da curva fica).
- **REF = posto do jacobiano NOSSO** (o da ezpz é `pub(crate)`, `constraints.rs:87,737`): resíduos assinados escritos
  uma vez, genéricos sobre um número dual (derivada exacta, sem diferenças finitas), posto em ordem de nascimento —
  a regra que não sobe o posto é REF. **Mudou no código:** Gram–Schmidt modificado 2× linha a linha
  (`posto.rs`), e não a `faer`: a QR de Householder SEM pivô inventa uma direcção numa coluna dependente e estraga
  o teste das seguintes; a pivotada responde «que posto», não «qual regra, por ordem». A crate não depende da `faer`.

### 3.3 W4.3 feita (26/10)

- `ph2d_vec_scene::cad` (o MODELO: `PontoCad` = `Vert(path, VertId)` | `Centro(path)`, `SegCad`, `Lei`, `RegraCad
  { lei, referencia }`) — mora na cena porque a W4.4 grava as regras na `VecScene`; o solver em `ph2d-vec-constraints`.
- Lei acrescentada ao §3: **`Fixo(ponto, xy)`** (o «bloquear» do Sketcher; as cenas da W4.0/W4.1 têm um em todas).
- Arrasto impossível (o agarrado está preso): o cursor desce para prioridade 1 e nenhuma regra parte — gate
  `dragging_a_fixed_point_breaks_no_rule`. Um vértice de um path citado como círculo não é ponto solto (`SemAlvo`).
- **Custo da família `faer`** (via `kcl-ezpz`, perfil `smoke`, `target` limpo, `loadavg` ~22 em 32 núcleos — teto
  pessimista): `ph2d-vec-scene` 4,4 s · **+ `ph2d-vec-constraints` com a árvore da ezpz 23,4 s**; `check` 25 s. Hoje
  nada da app depende dela; entra no caminho do `ph2d-app-vec` na W4.4 (re-medir lá o caminho crítico).
- **Mutação: 13/13 sangram** (1.ª ronda 8/13; sobreviventes M4 simétrico sem o 2.º resíduo, M9 palpite ignorado, M10
  sem a tangência das alças, M11 vértice de círculo como ponto solto, M12 sem a queda de prioridade → um gate cada,
  `5ce85c725f`, cada um visto vermelho sob a sua mutação).
- ⚠️ Não medido: o custo por passo de `resolve` (remonta o sistema a cada chamada). Mede-se na W4.4 no app, o pior
  passo do arrasto (o §5 pede-o antes da mensagem do smoke). → medido na §3.4.

### 3.4 W4.4 feita (26/10)

- **Onde moram:** `VecScene.regras` no FIM, degrau 24 (o v23 lê `vec![]`, `Deserialize` à mão em `cad.rs`, o padrão do
  `VecPath`). **Apagar um vértice ou um path tira as regras que o citam** em `VecScene::normaliza_ids_desde` — o
  assentamento dos ids que já corre antes de TODA captura de undo (`vec_tree_settle`) ⇒ mesmo passo, zero linha na
  shell (`VecScene::poda_regras`; um `Centro` só pede o path: o que deixou de ser círculo fica `SemAlvo`, não sai).
  `Lei::pontos()` = a porta única dos pontos citados (o solver largou a cópia `pontos_da`).
- **A guia diz QUAL vértice:** `SnapTargets.donos` (paralelo a `points`; o índice além do fim — os sprites — não tem
  dono) e `SnapAxis.dono`. Inferência (`ph2d-vec-edit/src/cad.rs` `leis_da_guia`): os dois eixos no MESMO vértice =
  coincidente; eixo X partilhado = vertical; Y = horizontal; grade, régua e curva não citam vértice.
- **A caneta:** `on_press_encaixe`/`on_drag_encaixe` embrulham as portas de sempre (a shell só trocou o nome e a
  closure devolve o `SnapResult`: +0 linhas). O vértice POSICIONADO ganha a regra no press; no arrasto de âncora, depois
  do snap, `resolve` sobre a folha dos paths citados (`Xform` publicado, palpite = o passo anterior) e `aplica`; a regra
  da guia é provisória no fim de `regras` e troca a cada passo — fica a do último passo (gate
  `a_drag_keeps_only_the_guide_lit_at_release`). Undo: o botão premido suprime a captura ⇒ um passo por gesto.
- **`junta_regra`** = a porta única de juntar: repetida (H/V/coincidente lidas nos dois sentidos) não entra; a mais pelo
  posto (`ph2d_vec_constraints::a_mais`, só o teste de posto, sem a análise da ezpz; gate contra a fixture do FreeCAD
  `the_rank_test_of_one_new_rule_agrees_with_the_oracle`): **medida nasce REF**, lei geométrica não entra.
- **Círculo vivo:** `ph2d_vec_entities::receita::assenta_circulos_vivos`, dentro do `settle_origins` — um
  `VecShape::Param` elipse `w = h` cuja geometria saiu do centro/diâmetro (só o solver a escreve assim; o Node congela a
  receita antes) ⇒ `Transform` ao centro e receita `2r`; ponto fixo (folga `1e-4·r + 1e-6·|centro|`, acima do `f32`).
  `gesture_paths` inclui `PenTool::cad_em_gesto()` (o `Xform` que a caneta leu vale até soltar). `resize_recipe` mudou-se
  para `ph2d-vec-entities` (o `ph2d-app-vec` re-exporta: uma cópia) e `circulo_de` para `ph2d_vec_scene::cad`.
- **Cores e conflito:** `ph2d_app_vec::cad_cores` corre `estado()` numa thread, só quando a ESTRUTURA muda (regras +
  ids dos vértices citados; posições fora da chave), uma de cada vez; o quadro nunca espera e nunca pinta a cor de
  outra estrutura. `cad_marks`: traço por regra (`Info` ok · `Text3` REF · `Warn` redundante · `Danger` conflito),
  losango por ponto (`Accent` livre · `Success` definido) e a ficha **«Conflict — delete one of: …»** com o nome de
  cada regra (`app.vec.cad.*`). `edit_marks::draw` recebe o `TextSystem` (shell +2 −2).
- ⚠️ **`ph2d-ui-testkit` não conduz o canvas** (só painéis): o seam `draw_with_a_guide_creates_the_rule` vive em
  `ph2d-vec-edit` com o MESMO snap que a shell corre (`collect_targets` + `snap`), e a ligação da shell é o gate de
  texto `the_pen_press_and_drag_take_the_snap_result` (`shells/desktop/tests/it`).

**Medido no editor** (`cad_medida_tests.rs`, perfil `smoke`, caneta × `resolve` cru intercalados, 7 rodadas, mínimo ·
mediana em ms; **`loadavg` 43 em 32 núcleos** — outras linhas a compilar: teto pessimista, o «A × B» vale pela
intercalação):

| cena | caneta (`on_drag_encaixe`) | `resolve` cru | juntar uma regra (posto) | análise das cores (`estado`) |
|---|---|---|---|---|
| escada 202 | **1,306** · 1,454 | 1,305 · 1,506 | 17,3 · 19,6 | 43,5 · 46,5 |
| misto 200 | **1,526** · 1,788 | 1,444 · 1,486 | 33,1 · 34,8 | 45,6 · 51,1 |

⇒ o editor não acrescenta custo ao solver (≤ 0,08 ms); **kill-criterion < 2 ms PASSA** mesmo a load 43 (a W4.0 mediu
0,68–0,81 ms do `resolve` a load 11–18). A análise das cores (~45 ms) está fora do quadro, como desenhado. Juntar uma regra (o posto,
só a COMPONENTE ligada, gate `another_shape_with_its_own_rules_does_not_change_the_rank_answer`) custava 17–33 ms a 200
pontos → **fechado na W6 (QR de Householder esparsa, 0,47–1,32 ms): [`07_W6_pontas.md` §2](07_W6_pontas.md#2-ponta-3--medido)**.

### 3.5 W4.5 feita (26/10) — a medida animada

- **Id durável da regra:** `RegraCad.id: RegraId(u32)` + `VecScene.next_regra`, apendados no MESMO degrau 24 (o v23
  lê `next_regra = 0`); `RegraId::NOVO = 0`. Quem dá é `VecScene::assenta_regras` (poda + ids, a lei do
  `normaliza_ids`: novo ou repetido ganha o próximo, nunca reusado), no assentamento que já precede todo undo. Gate
  `a_rule_keeps_its_id_when_an_earlier_rule_is_deleted`. `Lei::medida()`/`medida_mut()`/`dono()` (o path do 1.º ponto).
- **Família própria `CAD = 9`, e não a `CONSTRAINT = 8`** (o handoff anterior propunha a 8): o `ph2d-app-constraints`
  drena a 8 INTEIRA (`canais.rs:85`, a escrita sem `Constraints` é largada) e o `publicar` dele faz
  `esquecer(e, CONSTRAINT)` depois do do vetor (`fase_autokey.rs:47→53`) — a medida seria engolida e apagada. Cada
  família drena o seu (ADR-0075). `ph2d_ecs::canal_cad`: `MEDIDA = 1`, `i:j` = o `RegraId`, entidade = a do path dono;
  px, GRAUS num ângulo.
- **A porta:** `ph2d_vec_entities::medida` (uma `Folha` do layer — as duas famílias podem depender dela; o gate de
  camadas proíbe `app-vec ↔ app-constraints`): `campos` (publicar), `aplicar` (drena `CAD`, mede em `medida_mut`,
  resolve com `resolve(folha_AUTORADA, regras, None, None)`, escreve `aplica` sobre cópias do autorado) e a
  `ConducaoCad` (autorado + medidas autoradas; o que se escreveu; esquece num quadro SEM escrita da curva; uma cena que
  deixou de ser o que se escreveu = edição do artista = autorado novo). Só re-resolve quando a medida ou o autorado
  mudam (parado não custa). O `ph2d_app_vec::canais::{aplicar, publicar, ConducaoVetor}` chama-a: zero linha nova na
  shell (a `ConducaoVetor` leva-a dentro; `substituir` devolve `Vivo`).
- **Auto Key:** o vetor publica os cantos do AUTORADO para um path que a medida move (explorado: o autokey de um campo
  NÃO ligado grava quando o valor muda de um quadro para o outro com o Auto Key armado e parado — editar a medida
  gravaria chaves em todos os cantos). A medida publica-se viva (o eco da curva).
- **A cena 6** (`smoke_regua.rs`): perfil de 5 dentes, 20 cantos, 29 regras (H/V em todo lado, topos e folgas iguais,
  um `Fixo`, a altura do dente do meio `0,8 → 1,6 → 0,8` nos quadros 0/24/48, cúbica). `encenar(world, 6)` deixa o
  recurso `Semear`; o `ph2d_app_constraints::quadro` (agora `&mut VecScene`, shell +0) semeia; o `step` grava as três
  chaves por `chaves::gravar` (um passo) quando a entidade nasce, escolhe a régua e toca em laço.
- **Gate** `an_animated_dimension_plays_and_scrubs_to_the_same_bit` (`ph2d-app-constraints/tests/it/a_regua_de_desenho.rs`):
  49 quadros a tocar, todos com violação ≤ 1e-6 (unidades do mundo = 1e-4 px a 100 px/u; régua nova
  `ph2d_vec_constraints::violacao`, o resíduo menos o alvo), a altura medida nos CANTOS = a chave em 0/24/48, e
  0→40→12→40 numa cena nova = os mesmos bits. **Visto vermelho** sob a mutação «resolver a partir da cena (o quadro
  anterior)»: «arrastar até 40 ≠ tocar até 40».

- **Depois da 1.ª foto:** o pé do dente afundava (o solver repartia o esticão) ⇒ o chão preso (2.º `Fixo` + H entre as
  folgas, 33 regras) e o gate exige o pé em 0. A secção do Inspector `DRAWING RULES` (`cad_inspector.rs`, `MAX_REGRAS
  = 32`) ordena conflito → medidas → resto e mostra a medida em px/graus do projecto (`b8ca424171`).

**Medido** (`tests/it/medida_regua.rs`, `#[ignore]`, perfil `smoke`, intercalado, 7 rodadas, mínimo · mediana, ms):

| cena 6 (20 cantos) | pior passo do arrasto pela caneta (60 passos) | pior quadro da medida animada (49) |
|---|---|---|
| 29 regras, `loadavg` 21 | **0,053** · 0,097 | **0,046** · 0,094 |
| 33 regras, `loadavg` 20 | **0,066** · 0,069 | **0,048** · 0,056 |

### 3.6 ⛔ Smoke do dono 26/10 — o MOTOR passa, a SUPERFÍCIE é recusada

Funcionou (tocar, arrastar a timeline, × e Ctrl+Z), mas: *«Milhões de regras no painel, nomes repetidos, contra
intuitivo»* · *«não existe nada que possa ser entendido, apenas alguns pontos de cores diferentes»* · **«que seja feito
de forma muito bem desenhada e explícita como os gizmos das Shapes e por meio de uma nova TOOL na barra lateral
esquerda, não com um painel no Inspector»** (memória `project_drawing_rules_are_a_tool_with_explicit_gizmos_2026_10_26`).

| recusado (medido no smoke) | porquê |
|---|---|
| a lista no Inspector como superfície das regras | 33 linhas, «Vertical» ×10 sem dizer QUAL lado; 1 regra escondida pelo tecto 32 |
| traço colorido + losango (`cad_marks`) como única marca no canvas | não diz QUE regra é; o dono só viu «pontos de cores diferentes» |
| criar regra só pela guia da caneta | Equal/Fixed/Distance/Angle/Radius não se criam ⇒ a medida animada fica fora do alcance |
| cena de smoke com 33 regras | afoga a única coisa nova (a medida) |

Visto na foto do dono e **não** é defeito: o lado esquerdo inclinado = a 1.ª «Vertical» apagada no passo do × (é a do
lado 0); a ponta curva num topo = alça de Bézier puxada (as regras citam âncoras, não alças — a W4.6 decide se a tool
mostra/prende alças).

**W4.6 (feito 26/10 → §3.7/§3.8) — a tool «Rules»** (o desenho completo abre a próxima janela, red-first):
1. **Tool nova na barra esquerda do Vetor**, como as das Shapes/QuickShape (`ph2d-app-vec/src/vector_mode*.rs`,
   `quickshape_frame.rs` — ler com o `explorador` antes): seleccionar 1–2 pontos/lados mostra SÓ as regras possíveis
   (barra flutuante do mesmo idioma da da imagem); `junta_regra` continua a porta única.
2. **Gizmos explícitos** desenhados como os das Shapes (tokens, zero hex): selo por regra junto ao lado/ponto (`|` `—`
   `=`n com o par numerado, ⊙ coincidente, 🔒 fixo), **cota** com setas e o número clicável (escrever valor / chave do
   Auto Key — o canal `CAD = 9` já existe), hover acende o que a regra liga, clicar escolhe, Delete apaga, cor da
   FORMA = livre/definida/conflito (convenção SolidWorks/Fusion). Selos só com a tool activa; cotas sempre.
3. **Inspector**: sai a lista (no máximo um resumo); a secção `INSP_LIVE_CAD_SECTION` não recebe mais investimento.
4. **Cena de smoke nova** com poucas regras, cada uma visível (proposta: caixa com tampa que abre por ângulo animado);
   a cena 6 pode ser substituída.
5. Abertos herdados: régua da timeline mostra a medida em METROS (o Inspector em px); Auto Key ao escrever a medida não
   provado de ponta a ponta; W4.4: juntar regra a 200 pontos 17–33 ms.

### 3.7 W4.6 — o desenho (lido no código 26/10, ANTES do código)

**Onde a tool entra (a coluna ① do Vetor, não o `left_rail` do editor):** `DrawMode::Rules` (`ph2d-tool-vector`
`params_mode.rs`) + id `VECTOR_MODE_RULES` (`ids/vector.rs`) + `tool_panel_event.rs` (clique → modo) + a tabela `TOOLS`
de `ph2d-panel-vector/src/bars.rs` (família `Build`, ícone novo `IconId::VectorRules` = `docs/design/icons/vector-rules.svg`,
desenho ORIGINAL — não há Lucide local) + `hint_key` + `populate_modes.rs` (registo: o `wiring_parity`) + i18n
`panel.vector.mode.rules` / `panel.vector.hint.rules`. Sem tecla (o Illustrator não a tem; o `R` é do Rotate). Entra no
`PRECISE` da ④ (a ④ dos verbos de caminho roubaria o clique).

**Os gizmos e o clique moram numa crate, a shell paga ZERO linha nova** (a catraca está a 1 de folga):
- `ph2d-app-vec/src/cad_tool.rs` (estado `CadTool` no `VecState`: a ESCOLHA de 1–2 alvos `Alvo::{Ponto, Lado, Circulo}`,
  a regra escolhida, o hover, o último PLANO desenhado) + `cad_tool_plano.rs` (o plano em px de ECRÃ: selos, cotas, cor
  dos lados, e os rects de acerto — **o rect que se pinta é o que se acerta**, uma lista) + `cad_tool_pinta.rs`.
- **Clique / Esc:** pelas portas que a shell JÁ chama em todo modo — `shape_handles_live::on::press` e `on::escape`
  desviam para o `cad_tool` com `DrawMode::Rules` (consome todo press no canvas, como o Trim: o vazio limpa a escolha).
- **Desenho:** `edit_marks::draw` (já chamado) troca o `cad_marks::pinta` pelo plano novo: SELOS só com a tool activa,
  COTAS sempre; o `cad_marks` sai inteiro (recusado no smoke).
- **Quadro:** a linha da shell que chamava `cad_inspector::frame` passa a chamar `cad_tool::frame` (hover pelo
  `last_pointer`, dreno dos pedidos, semente do número, publicação da ④/②); a do barramento (`ComponentEdit::Cad`)
  sai com a secção.
- **Delete:** o `vec_delete_selected_vertex_or_path` + `vec_delete_selected` da shell MUDAM-SE para a crate
  (`ph2d_app_vec::apaga`) com a regra escolhida à frente — a shell encolhe (a cura é mover).

**Criar à mão:** clicar ponto/lado/círculo junta-o à escolha (máx. 2; um 3.º recomeça). A ④ (âncora = a caixa da escolha
no ecrã) mostra SÓ as regras possíveis — 1 lado: Horizontal, Vertical, Distance · 2 lados: Equal, Angle (Paralelo/
Perpendicular ficam de fora: não estão na lista do dono) · 1 ponto: Fixed · 2 pontos: Coincident, Horizontal, Vertical,
Distance · 1 círculo: Radius. A medida nasce com o valor ACTUAL (juntar não mexe no desenho; o `junta_regra` decide REF/
recusa). Botão → fila do painel (`ph2d-panel-vector` `state_cad.rs`, como os `PEDIDOS` da secção) → `cad_tool::frame` →
`junta_regra`. Ícones dos 8 botões = os glifos dos selos (o botão e o selo são o MESMO desenho).

**Cota e número:** clicar o número escolhe a regra e foca o chip numérico da ② (`ToolRailEntry::Number`, semente
one-shot ao escolher, como os campos da forma); escrever → `Medida` → o mesmo `aplica` que a secção usava (px/graus do
projecto → mundo/radianos; REF não se escreve) → o diff do canal `CAD` grava a chave com o Auto Key armado. O losango
do chip mapeia para `PropKind::Field(CAD, MEDIDA, regra)` (`marcas.rs`, tabela dinâmica).

**Cor da forma:** sobre cada lado citado, livre = `Info`, definido = `Text1`, conflito = `Danger` (SolidWorks: azul/preto/
vermelho); a análise é a do `CadCores` (fora da thread do quadro). Hover = os alvos da regra em `Selection`; escolha em
`Accent`.

**Inspector:** a secção DRAWING RULES sai INTEIRA (ids, populate/event/sync/paint, `cad_edits`, `LIVE_SECTIONS`, i18n,
`seam_cad`) — o dono não a quer como superfície; um resumo não ensina nada que a tool não mostre.

**Cena 7 «Caixa com tampa»** (`ph2d-app-constraints`, `max_level` 6 → 7): corpo retângulo (2 H, 2 V, `Fixed` no canto,
largura e altura = 2 cotas), tampa = um lado preso na dobradiça (`Coincident`), `Equal` à largura, `Angle` animado 0° →
110° → 0° (quadros 0/24/48). ~10 regras, todas à vista. Abre com a tool Rules na mão.

**Gates red-first (por nome):** `the_rules_tool_sits_in_the_left_column` · `a_choice_offers_only_the_rules_it_allows` ·
`a_rule_made_by_hand_goes_through_junta_regra_and_keeps_the_drawing` (seam: press real nos lados → botão → regra) ·
`clicking_a_dimension_number_selects_it_and_typing_moves_the_drawing` · `delete_removes_the_chosen_rule_in_one_step` ·
`badges_only_with_the_tool_dimensions_always` · `the_hit_rect_is_the_painted_rect` · `hover_lights_what_the_rule_binds`
· `the_shape_colour_says_free_defined_conflict` · `typing_a_dimension_with_autokey_records_a_key` · a unidade da
timeline (px) · `the_lid_opens_and_scrubs_to_the_same_bit`.

### 3.8 W4.6 — feito (26/10), medido, aberto

**Commits:** `7cccaa982d` doc §3.7 · `865ac64d85` a tool (coluna ①, ④/② e fila no painel `ph2d-panel-vector/src/state_cad.rs`;
gizmos e press na crate `ph2d-app-vec/src/cad_tool*.rs`; a secção DRAWING RULES do Inspector saiu inteira; o Delete mudou-se
para `ph2d_app_vec::apaga`) · `889cb60276` gates do canvas (`circulo.rs`: alça recolhida = canto — um retângulo deixava de
ter lados no plano) · `541fc408f3` losango/canal do número da cota (`marcas::Contexto.regra`) · `bb029132e1` cena 7 «Caixa
com tampa» (`ph2d-app-constraints/src/smoke_caixa.rs`, `PH2D_CONSTRAINT_SMOKE=7`); a timeline mostra a medida em px e o
ângulo em graus (`canal_cad::ANGULO = 2`, `canal::unidade`, `medida::chave_da` porta única) · `83788bb61f` traço da tampa em
unidades do mundo · `abbdcbf555` bateria por nome (chaves i18n órfãs, isenção `POR_FUNCAO` com `VECTOR_CAD_VALUE`, rustfmt) ·
`055ffe0b28` + `c91abddc83` + `d4c74c0755` report do dono no smoke.

**Report do dono (26/10):** «gravar o quadro com distance e auto key não funcionou». Mecanismo MEDIDO no app real com sonda
temporária (cena 7 + `PH2D_AUTOKEY_LOG`): o pedido `Medida` que chega à tool grava a chave (quadros 12 e 36 OK para a medida
do corpo selecionado); a medida da TAMPA (path não selecionado) não gravava — a população do Auto Key
(`shells/desktop/src/render_loop/autokey_amostra.rs::populacao`) era seleção + mãos, e o diff `autokey_campos` precisa do
quadro ANTERIOR (base), por isso uma «mão» de um quadro não serve. **Cura:** `ph2d_ecs::AlvosDasFamilias` (o que a tool
aponta além da seleção; sem a lei da mão — a timeline continua a conduzir), publicado por `ph2d_app_vec::maos::publicar` a
partir de `CadTool.alvos` (`cad_tool::alvos`: com regra ou escolha na mão, todo path citado por regra + os da escolha),
lido na `populacao`. Gate `a_chosen_rule_points_the_autokey_at_every_path_the_sheet_can_move` (mutação: alvos vazios ⇒
vermelho). Provado no app: «Path 1 · Dimension 150 px» com chave no quadro 12.

**Segundo defeito (visto na foto):** numa linha aberta de 2 pontos o centro da forma cai no próprio lado ⇒ «selo dentro,
cota fora» empatava e o «= 1» tapava o número. **Cura:** `EMPATE_PX` (no empate a cota fica na normal, o selo do outro lado) +
`desencosta_selos` (nenhum selo cruza um número nem outro selo). A cota REF mostra-se entre parênteses «(200 px)» (i18n
`app.vec.cad.referencia`): na cena 7 tudo está definido, qualquer medida nova nasce REF e sem chip para escrever, o que
confundiu o smoke. Gate `no_badge_covers_a_number_or_another_badge_and_a_reference_reads_in_parentheses` (2 mutações ⇒
vermelho).

O const `SEM` de `autokey_campos_vetor_tests.rs` (shell) não compilava desde `541fc408f3` (faltava `regra: None`) —
corrigido; `autokey_pass::` 56/56 verdes.

**Números:** shell 189 040 linhas contra tecto 189 041 (folga 1 — a próxima linha da shell tem de MOVER código); cena 7:
violação ≤ 1e-6 nos 49 quadros, play × scrub = mesmo bit (`the_lid_opens_and_scrubs_to_the_same_bit`). **Smoke do dono: OK
em 26/10** (depois da cura).

**ABERTO (a próxima janela resolve num bloco, §0.10):**
1. o chip ② da cota não re-semeia enquanto focado (mostra o valor antigo se a medida mudar por outro caminho);
2. a coluna de nomes da timeline (40 px) elide «1234 px»;
3. `CANTO`, `TRACO_LARGURA` e `PARAM_*` sem unidade na timeline;
4. a ④ não foi fotografada com clique (a sessão virtual não clica);
5. Auto Key grava também os cantos que a medida mexeu (lei do dono «grava tudo» — não é defeito);
6. W4.4: juntar regra a 200 pontos 17–33 ms;
7. o gate `each_vertex_mutation_site_keeps_or_assigns_ids` da W4.2 está sem ✓ na tabela §4 — conferir.

**Fechado em 26–27/10 (janela W5.0)** — handoff [`W5.1`](handoffs/HANDOFF_CONTINUACAO_W5_1_2026-10-27.md):
1. chip ② re-semeia: `WidgetStore::set_number_value` re-semeia um campo FOCADO cujo buffer ninguém tocou; o `cad_tool`
   compara a marca (id, bits). Gates `number_input_set_value_reseeds_an_untouched_focused_buffer` ·
   `a_focused_untouched_chip_follows_a_measure_changed_elsewhere`.
2. «1234 p…»: `com_unidade` (`tracks_value.rs`) divide os 6 caracteres da fatia — cede a precisão, depois o espaço
   («1234px», «-123°»). Gate `a_length_channel_shows_in_the_project_unit`.
3. `canal::unidade` dá Comprimento a `CANTO`, `TRACO_LARGURA`, `PARAM_LARGURA`, `PARAM_ALTURA`. Gate
   `each_field_says_its_display_unit`. **`PARAM_VALOR` e o tracejado ficam nus**: a grandeza depende do tipo da forma;
   precisa do tipo publicado no snapshot — ABERTO.
7. `each_vertex_mutation_site_keeps_or_assigns_ids` é verde; o cabeçalho prometia sub-gates inexistentes e agora diz a
   verdade: os sítios de fora cobrem-se pela porta do assentar do undo (`VecScene::normaliza_ids_desde`,
   `shells/desktop/src/vec_tree_settle.rs`). Mutação: 4/4 sangraram.
- **Smoke do dono 27/10 («as linhas das shapes piscam»):** `CadCores` invalidava a análise a cada VALOR de regra; a tocar,
  a análise (9–12 ms) chegava sempre atrasada ⇒ sem lados. Agora mostra a última resposta da MESMA estrutura até a nova
  chegar. Gates `an_animated_value_keeps_the_colours_until_the_new_analysis_arrives` ·
  `a_new_rule_never_shows_the_colours_of_the_old_structure`. Smoke do dono OK.

Continuam como estavam: **4** (a sessão virtual não clica ⇒ foto com clique impossível), **5** (não é defeito), **6**
(W4.4, juntar regra a 200 pontos 17–33 ms, ABERTO).

## 4. Gates red-first

| gate | fenómeno na fixture | passo |
|---|---|---|
| `a_well_constrained_sketch_lands_where_freecad_puts_it` | cenas de solução única da W4.1, posições ao 1e-6 | W4.3 ✓ |
| `the_drag_keeps_every_rule_at_every_step` | `w4_cenas` escada 202 e misto 200, 60 passos, 1e-4 px | W4.3 ✓ |
| `the_drag_is_deterministic` | o mesmo arrasto 2× ⇒ o mesmo bit | W4.3 ✓ |
| `a_vertex_keeps_its_id_when_a_neighbour_is_deleted` | apagar o 3.º de 10: os outros ids iguais; a regra no 5.º continua no mesmo ponto | W4.2 ✓ |
| `a_v23_scene_opens_with_ids_in_order` | bytes v23 gravados ⇒ ids `1..=n` (`verts` → `subpaths`), ida e volta v24 | W4.2 ✓ |
| `a_v194_project_opens_with_vertex_ids_in_order` | `.ph2dproj` gravado pelo código v194 (cena + geometria de uma pose) | W4.2 ✓ |
| `a_v23_{envelope_child,skin}_source_is_rewritten_with_ids` | blobs v23 do Envelope e da pele regravados vivos | W4.2 ✓ |
| `copy_point_follows_its_vertex_after_an_insert_before_it` | inserir antes · ponto de buraco · apagar ⇒ `SemAlvo` | W4.2 ✓ |
| `each_vertex_mutation_site_keeps_or_assigns_ids` | censo dos ~20 sítios (§2), um caso por sítio | W4.2 |
| `an_extra_dimension_is_born_ref` (teste de posto) · `a_conflict_lists_the_rules_involved` (`unsatisfied`) | `conflito_*` da W4.1: REF = [6]; conflito = [1,3,4,6] e [1,2] | W4.3 ✓ (o «nasce REF» no editor é W4.4) |
| seam `draw_with_a_guide_creates_the_rule` (em `ph2d-vec-edit`, com o snap da shell — o testkit não conduz o canvas) · `a_drag_keeps_only_the_guide_lit_at_release` · `the_pen_press_and_drag_take_the_snap_result` (shell) | caneta com a guia H acesa ⇒ regra H gravada; no arrasto fica a guia do soltar | W4.4 ✓ |
| `deleting_a_vertex_drops_the_rules_that_cite_it_when_ids_settle` · `deleting_a_path_drops_every_rule_that_cites_it` · `a_scene_with_rules_round_trips` | a regra sai no assentamento que precede o undo; o v23 abre sem regras | W4.4 ✓ |
| `dragging_a_corner_keeps_the_rectangle` · `a_rule_the_drawing_already_implies_is_not_added_and_an_extra_dimension_is_born_ref` · `the_rank_test_of_one_new_rule_agrees_with_the_oracle` | o arrasto pela caneta obedece; a diagonal de um retângulo definido nasce REF; a H implícita não entra | W4.4 ✓ |
| `a_live_circle_the_solver_moved_gets_its_recipe_and_transform_back` (+ controlo) | receita `2r`, `Transform` no centro, a forma não se move, ponto fixo | W4.4 ✓ |
| `the_colours_come_from_an_analysis_off_the_frame_thread_only_when_the_structure_changes` · `a_conflict_writes_which_rules_to_delete` (+ controlo) | a análise só pela estrutura; a ficha nomeia as regras em conflito | W4.4 ✓ |
| `an_animated_dimension_plays_and_scrubs_to_the_same_bit` · `a_rule_keeps_its_id_when_an_earlier_rule_is_deleted` · `a_dimension_curve_moves_the_drawing_but_the_document_publishes_the_authored_corners` · `the_capture_sees_the_authored_drawing_and_an_artist_edit_becomes_it` | medida com 3 chaves, quadros 0→40→12→40; o id não anda com o índice; o undo vê o autorado | W4.5 ✓ |
| `the_rules_tool_sits_in_the_left_column` · `a_choice_offers_only_the_rules_it_allows` · `a_rule_made_by_hand_goes_through_junta_regra_and_keeps_the_drawing` · `clicking_a_dimension_number_selects_it_and_typing_moves_the_drawing` · `delete_removes_the_chosen_rule_in_one_step` · `typing_a_dimension_with_autokey_records_a_key` | a tool na coluna, só as regras possíveis, a mão passa por `junta_regra`, o número clicável move o desenho, Delete num passo, a medida escrita grava chave | W4.6 ✓ |
| `badges_only_with_the_tool_dimensions_always` · `the_hit_rect_is_the_painted_rect` · `hover_lights_what_the_rule_binds` · `the_shape_colour_says_free_defined_conflict` · `no_badge_covers_a_number_or_another_badge_and_a_reference_reads_in_parentheses` | gizmos do canvas; nenhum selo tapa número; REF entre parênteses | W4.6 ✓ |
| `a_chosen_rule_points_the_autokey_at_every_path_the_sheet_can_move` · `the_lid_opens_and_scrubs_to_the_same_bit` | o Auto Key vê o path não selecionado; cena 7 play × scrub = mesmo bit | W4.6 ✓ |
| prova de mutação | tirar o `Fixed` do arrastado / a migração / o REF ⇒ o gate certo sangra | W4.3 ✓ 13/13 (§3.3); W4.4 ✓ 17/18 (M12 = só perf, exacto; M7 era equivalente e o palpite saiu) |

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
