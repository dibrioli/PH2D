# HANDOFF DE INTEGRAÇÃO — `line/3DModeling`, 2026-09-13 (W148)

> **DIRETRIZ §1.5.9.** A linha fecha aqui e **PARA**. Não integra, não pusha, não faz ship — isso é
> ordem explícita do Enio, por um agente integrador dedicado (§0.7).

## 1. Identidade

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling` |
| ramo | `line/3DModeling` |
| base (merge-base) | `1d43da737` — o `main` enviado de 13/09 (CI verde), sobre o qual a worktree foi recriada |
| commits à frente | ⏳ *(preencher no commit final)* |
| handoff anterior | [2026-09-10](HANDOFF_INTEGRACAO_line_3DModeling_2026-09-10.md) · a W2 (o piloto) está no [de 11/09](HANDOFF_INTEGRACAO_line_app_host_2026-09-11.md) |
| o mecanismo, as tabelas e as recusas | [`docs/3DModeling/12_a_cache_contra_o_casco.md`](../12_a_cache_contra_o_casco.md) |

**A wave numa frase:** o item ⏳ §83.9 do doc 06 (*a cache de fitas contra o CASCO*) — medido antes de
construir, o prémio era `~2×` da marcha e não `1,11×`, o desenho que a nota prescrevia perdia, e o
que shipa é o casco crescido por uma **distância** tirada do braço da órbita.

## 2. ⭐ NENHUM contador partilhado se move

`collision-surface.sh` (caminho absoluto do primário), corrido sobre a árvore da linha:

```
PROJECT_SCHEMA      128 (base: 128)   · tripla (128, 13, 22)
VEC_SCENE_SCHEMA     22 (base: 22)
FLIP_SCHEMA          13 (base: 13)
DOC_VERSION (tl)     18 (base: 18)
FIELD_DOC_VERSION    22 (base: 22)
ph2d-ecs 85 · ph2d-render (espelho) 86 · ph2d-script (espelho) 86   (todos = base)
contrato congelado: node.rs intocado · tool.rs intocado
ADR: nenhum · Cargo.lock: nenhum pacote novo · marcadores: nenhum · tectos de LOC: nenhum estoura
```

⭐ **A `shells/desktop` não é tocada** (delta `+0 −0`). Nenhum ficheiro fora de `crates/ph2d-field-eval`,
`crates/ph2d-field-render` e `docs/3DModeling`.

## 3. ⚠️ API partilhada TOCADA — leia antes de fundir

As duas crates são deste módulo, mas são **dependências** do `ph2d-app-field3d` e do
`ph2d-app-sculpt3d` (por transitividade, via `ph2d-viewport3d`).

**`ph2d-field-eval`** — só **apêndice**:

| item | o quê |
|---|---|
| `RegionHulls` (novo, `pub`) | os cascos de uma região por folha, com `contains` |
| `RegionCompiler::hulls` / `compile_hulled` (novos) | os cascos de uma região; a compilação com eles já feitos |
| `RegionCompiler::compile_at` | ⭐ **assinatura e resultado iguais** — passa a ser `compile_hulled(…, &hulls(…))`, a mesma conta mudada de sítio |
| `affine::local_maps` (crate-privado) | o mapa mundo→local por nó sobe da compilação; `RegionCompiler` guarda-o (era composto por região) |
| `hull::in_convex` (crate-privado) | o teste ponto-em-convexo com nome; `probe_in_hull` delega |

**`ph2d-field-render`** — ⚠️ **uma assinatura MUDA**:

| item | antes | depois |
|---|---|---|
| `TapeCache::get` | `(lo, hi)` | `(lo, hi, query: Option<&RegionHulls>)` |
| `TapeCache::insert` | `(lo, hi, tape)` | `(lo, hi, hulls: Option<RegionHulls>, tape)` |
| `TapeCache::inflate_of` | existia (`#[doc(hidden)]`) | ⛔ **apagada** — `growth_of()` / `uses_hulls()` |
| novos | — | `Growth`, `PAD_OF_REACH`, `with_growth`, `with_pad_of_reach`, `grow`, `reach`, `pad_phased` |

⭐ **Censo dos chamadores, medido:** fora das duas crates, a `TapeCache` só é construída por
`TapeCache::new()` (`ph2d-app-field3d/src/{smoke_state,preview_cost_tests}.rs`) — nenhum chama `get`,
`insert` ou `inflate_of`. Uma linha paralela que acrescente um chamador directo dessas funções parte a
compilar (alto), não em silêncio.

⚠️ **Variável de ambiente nova:** `PH2D_FIELD_TAPE_BOX=1` volta à cache da W82 (a caixa). É bissecção,
não configuração.

## 3-bis. ⛔⛔ UM DEFEITO ANTERIOR, achado na auditoria e CURADO — a folha debaixo de uma operação que dobra

**Visível ao artista:** um contorno desenhado (extrusão, polígono, torno) debaixo de uma **operação**
com espelho, matriz, radial, afunilamento, torção ou dobra saía com pixels errados na
pré-visualização — medido **`84`** (espelho) e **`586`** (matriz) contra `0` do gémeo sem o
modificador, e `0,496` de desacordo no documento dentro da região. A W56 escreveu a especialização por
região, a W79 fez do espelho na operação um gesto do produto, e **nenhum gate os juntava**: o da W56 põe
a matriz na própria folha, que ali é a raiz.

**A cura** (`affine::local_maps`, `bef750696`): a descida pára debaixo de um nó com modificador que
remapeia; os filhos ficam sem mapa, e a especialização e os cascos da cache desistem neles. ⚠️ **Preço
declarado:** essas folhas passam a ser certas e **não especializadas** (mais lentas), como já eram debaixo
de um modificador próprio. Gates novos: `the_specialisation_gives_up_under_a_remapping_ancestor`
(`ph2d-field-eval`) e `the_folded_leaf_draws_like_the_row_march` (`ph2d-field-render/tests/it`), os dois
vistos **vermelhos antes** da cura. Detalhe: doc 12 §12.9-bis.

## 4. Contratos congelados encostados

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`
intocados; zero ADR.

## 5. Gate batched do fecho

⏳ *(preencher: nextest-impacted com `BASE=merge-base` · `CARGO_BUILD_WARNINGS=deny cargo check
--workspace --all-targets` · clippy da linha do `ship.sh` · `cargo machete` · `check-standalone-optional`
· `check-workflow-packages` · `fmt --check` · `doc-index --check` — com a carga ao lado)*

Já corrido durante a wave: as suítes das três crates do campo (`ph2d-field-eval`, `ph2d-field-render`,
`ph2d-app-field3d`) — **728 / 728**, `51 s`.

## 6. ⚠️ Coisas que uma leitura rápida do diff entende ao contrário

1. **A caixa da W82 NÃO é código morto.** `Growth::Box` é o caminho de bissecção e o lado A de todo
   relógio do módulo — as duas políticas têm de poder correr no mesmo processo. Apagá-la «por arrumação»
   tira a régua da decisão que a substituiu.
2. **`compile_at` não mudou de comportamento.** O casco que ele consome é calculado pela mesma função,
   agora chamada de `hulls`; a diferença é que o valor pode ser guardado e comparado.
3. **`inflate_phased` sai byte-idêntico.** O misturador da fase subiu para `phase_u`, com dois leitores
   (a caixa e a folga), nas mesmas operações e na mesma ordem.
4. **As sondas `how_many_frames_to_keep` e `cohort_dispersion` passaram a medir o CASCO** — elas constroem
   a cache por `TapeCache::new()`. As tabelas nos doc-comments delas (e do `FRAMES_KEPT`/`PHASE`) são as da
   caixa. ⏳ reconferência nomeada no doc 12 §12.11.
5. **Os três gates de imagem não prendem a política.** Uma mutação que servia fitas de casco só pela caixa
   sobreviveu a todos (§7) — quem a prende é `a_hull_tape_is_never_served_to_a_region_its_hulls_do_not_contain`.
6. **O gate por folha `a_posed_leaf_region_holds_what_the_march_evaluates` não prende um mapa de pose
   errado** — o oráculo dele usa o mesmo mapa. Quem prende é a paridade da W56 (M5 em §7).
7. **`PAD_OF_REACH = 0,08` foi escolhido pelas CONTAGENS** (a opção que não perde para a caixa em
   gesto nenhum); ⏳ o relógio entre `0,06` e `0,08` está em §8.

## 7. Provas de mutação (instrumento versionado: `docs/3DModeling/ferramentas/w148_provas_de_mutacao.sh`)

| mutação | veredicto |
|---|---|
| M1 `contains` só por caixa | MORTA — `the_containment_of_hulls_is_sound_in_any_pose` |
| M2 servir fita de casco só por caixa | ⛔ **SOBREVIVEU aos 3 gates de imagem** · MORTA pelo gate de propriedade novo |
| M3 consulta sem cascos | MORTA — `a_drag_stops_recompiling_…` + gate da peça com poses |
| M4 folga sem o lado de baixo | MORTA — `the_padded_region_still_contains_its_query` |
| M5 mapa sem a pose do pai | MORTA — só `the_specialised_document_agrees_inside_its_region` (W56) |

## 8. O relógio

⏳ *(preencher: `measure_what_the_hull_cache_buys_on_the_clock`, a `load < 5`)*

## 9. As premissas que a MEDIÇÃO derrubou

1. ⛔ *«a cache contra o casco deixa `1,11×` na mesa»* (doc 06 §83.9) — **a mesa tinha até `~2×`**: a
   `TILE = 24` a caixa servia `1,9×`–`2,4×` as arestas do caminho sem cache. O `1,11×` era de quando o
   ladrilho era `64`, e a W88 moveu o número sem reconferir a nota.
2. ⛔ *«o teste em dois níveis com o casco inflado»* — com o casco **escalado** por `f` o acerto cai para
   `48`–`70 %` e as compilações triplicam. A folga tinha de ser uma distância.
3. ⛔ *(minha)* *«a caixa pesa porque serve a fita mais velha»* — servir a mais nova ou a mais apertada
   muda as arestas em `< 5 %`.
4. ⛔ *(minha)* *«a folga normaliza-se pelo ladrilho»* — pelo ladrilho, a mesma folga dá `81,5 %` e
   `91,8 %` em duas células; pelo **alcance** a partir do alvo, as nove células ficam na mesma faixa.
5. ⛔ *(minha)* *«os gates de imagem prendem a política de serviço»* — a M2 sobreviveu a todos.
6. ⛔ *«a especialização desiste debaixo de todo modificador que remapeia — há gate»* — o gate da W56 põe
   o modificador **na própria folha**; debaixo de uma **operação** que dobra, a folha era especializada
   no espaço errado (§3-bis).
7. ⛔ *(minha)* *«esperar por `load < 5` dá uma máquina calma»* — o laço disparou a `4,63` no segundo em
   que a minha própria suíte arrancou, e a 1.ª corrida do relógio mediu a `37`–`60` (doc 12 §12.10.1).

## 10. ⏳ O que fica ABERTO

- ⏳ `0,06` contra `0,08` do alcance — o relógio (§8).
- ⏳ `FRAMES_KEPT = 3` e `PHASE = 0,3` foram medidos com a caixa; a política do casco serve mais regiões
  com menos fitas e pede reconferência.
- ⏳ O cálculo dos cascos da CONSULTA (no `tiles.rs`) não tem contador próprio; o `GET_NS` só conta o
  teste de contenção.
- ⏳ **A pré-imagem de cada dobra** — uma folha de perfil debaixo de uma operação que remapeia deixou de
  ser especializada (§3-bis): certa e mais lenta. Especializá-la de novo é calcular a pré-imagem do
  espelho, da matriz, do radial, do afunilamento, da torção e da dobra — a wave que a W56 já nomeava, e
  sem preço medido ainda.

Os outros abertos do módulo não mudaram (doc 06 §13.0): a marcha, a mistura N-ária, o tecto de `round`
da estrela, o `SLABS`.

## 11. ⚠️ DUAS notas do `CLAUDE.md §5` (3D Modeling) OBSOLETAS — para o integrador

Elas mandam reconstruir trabalho já pago:

1. *«⏳ **O filete só é um ARCO a 90°** … hoje compensa-se só nas quinas AGUDAS, e as duas curas gerais
   estão medidas e rejeitadas»* — **a W107 (02/09) fechou-o**: o recuo passa a `r·(1/sin α − 1)` em todo
   ângulo (doc 06 §13.0, linha `✅ O FILETE É UM ARCO EM QUALQUER QUINA`, §108).
2. *«⛔ **A BASE FICA:** o quadro de movimento custa `26,7 ms` contra um orçamento de `16,7`»* — o doc 06
   §13.0 regista `✅ A pré-visualização ALCANÇA 60 Hz — mediana ~12 ms` (§90.4, §91.1), com o arrasto real
   a `10`–`27 ms`. O `26,7` é anterior à W82/W88.

## 12. O que SMOKAR

⭐ **Esta wave não muda a imagem, de propósito** — a cache escolhe que fita serve, e os gates dizem
que a forma não mexe um pixel. O que se confere é que **nada regrediu** e que a mão **não pesa mais**:

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && cargo run -p ph2d-host-desktop --profile smoke`
2. Clicar a pill **MODEL** no topo; criar duas ou três formas pela paleta (`A` ou *+ Add shape…*),
   escolher uma forma **desenhada** (`+ Extrude` sobre um contorno do editor vetorial) se houver.
3. Girar a peça com o botão esquerdo, deslocar com o do meio, aproximar e afastar com a roda — devagar
   e depressa.
4. **Tem de acontecer:** a peça sai **igual à de antes** e a vista acompanha a mão.
5. **Deu errado se:** aparecer um buraco, um risco ou uma face a piscar durante o movimento que some
   ao parar — isso é uma fita servida onde não vale. Para comparar com o comportamento antigo, o mesmo
   comando com `env PH2D_FIELD_TAPE_BOX=1` antes do `cargo` volta à cache de antes.

⏳ *(o binário deixado compilado, com a 2.ª saída colada, fica no §13)*

## 13. Higiene do fecho

- [ ] gate batched verde (§5)
- [x] handoff escrito (este ficheiro) · doc 12 · linha do §13.0 do doc 06 · índices da porta e dos handoffs
- [ ] `rm -rf target/*/incremental`
- [ ] binário do smoke compilado, 2.ª saída colada
- [ ] **UMA LINHA** no `CLAUDE.md §5` — ⚠️ escrita **na integração**, não aqui

## 14. A UMA LINHA para o `CLAUDE.md §5` (3D Modeling)

⏳ *(preencher depois do relógio)*
