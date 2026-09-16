# HANDOFF DE INTEGRAÇÃO — `line/3DModeling` · O ARCO NO PERFIL (2026-09-16)

**Branch:** `line/3DModeling` · **Base:** `git merge-base main HEAD`
**Ordem do dono:** *«o render do vaso deveria ser mais rápido»* → *«implemente a cura»*.

---

## §1 — O que mudou, em uma frase

Uma quina arredondada deixou de ser **oito segmentos rectos** na fita que a marcha avalia por pixel
e passou a ser **um arco exacto**. O vaso do dono: `77,13 → 25,6 ms` (**`2,95×`**); a cantoneira
`112,78 → 13,2 ms` (**`8,5×`**). Medições, réguas e recusas:
[`docs/Render3d/06_auditoria_do_vaso.md`](../../Render3d/06_auditoria_do_vaso.md).

## §2 — ⚠️ CONTADORES PARTILHADOS QUE SE MEXERAM — conte o DELTA, nunca o literal

| contador | aqui | delta |
|---|---:|---:|
| `PROJECT_SCHEMA` | `131 → 132` | **+1** |
| `ph2d_field::FIELD_DOC_VERSION` | `22 → 23` | **+1** |

⚠️⚠️ **O `FIELD_DOC_VERSION` NÃO está na tripla do `project_schema_tests`** e continua a subir à
mão; o instrumento que avisa é o `the_shape_of_a_saved_profile_is_pinned` da `ph2d-field`
(`90 → 92` bytes). ⚠️ E o `collision-surface.sh` **não o vê** — duas linhas que o subam em paralelo
fundem **mudas**.

⛔ Zero contratos congelados tocados. Zero ADR. Zero pacote externo novo.

## §3 — ⛔⛔ FOUNDATIONAL DE OUTRA LINHA: uma correcção em `ph2d-vec-scene`

`corner_live::fillet_handles` emitia `h = (4/3)·tan(α/4)·s_in` onde a lei do arco pede `·r`, e
`s_in = r·tan(α/2)` — um factor a mais, que vale `1` **só a `90°`**. O artista pedia raio `0,05` e
a `130°` recebia **`0,083`** (`110×` a tolerância de cozimento).

⚠️ **É product-visible em TODA quina arredondada do app fora de `90°`**, não só no modelador.
⚠️ **O gate que devia tê-lo apanhado corre sobre um QUADRADO** — quatro cantos a `90°`, o único
ângulo onde o defeito é invisível. Gate novo: `o_filete_vivo_e_um_arco_em_todo_angulo`, `30°`–`150°`.
⭐ A `ph2d-vec-scene` passa **`501/501`** com a cura dentro.

⇒ **Quem integrar avise a `line/Vector`**: goldens de imagem com quinas arredondadas fora de `90°`
mudam, e **a mudança é a cura**.

## §4 — Os ficheiros

| ficheiro | o quê |
|---|---|
| `ph2d-field/src/profile.rs` | campo `arcs`, porta `with_arcs`, `prim_count`, `arc_count`, `ProfileError::BulgeMismatch` |
| `ph2d-field/src/lib.rs` | `FIELD_DOC_VERSION` 22→23 com o degrau escrito |
| `ph2d-field-profile/src/lib.rs` | `bulge_do_cubico` (reconhece o arco pela GEOMETRIA) + `decomposicao_exacta` |
| `ph2d-field-eval/src/profile.rs` | o emissor da fita: distância por cunha + a correcção da meia-lua no sinal |
| `ph2d-field-eval/src/profile_arc_tests.rs` | **o gate que decide**: o campo com arcos == o campo da polilinha |
| `ph2d-vec-scene/src/corner_live.rs` | a correcção do alçapão (§3) |
| `ph2d-app-field3d/src/device_probes.rs` | as duas sondas da auditoria |
| `shells/desktop/src/project_schema*.rs` | degrau 131→132 + a tripla |

## §5 — ⚠️ SEIS coisas que uma leitura rápida do diff entende ao contrário

1. **`contours()` NÃO encolheu, e não podia.** Ela continua a ser a polilinha densa — **a FIGURA**.
   Os arcos são uma vista **adicional**. A 1.ª tentativa fez o contrário e dois gates que já existiam
   apanharam-na na primeira corrida.
2. **`segment_count()` deixou de ser «o número que manda no custo»** — é o `prim_count()`. Quem citar
   um custo a partir do primeiro cita a polilinha, não o que a marcha avalia.
3. **O reconhecimento é por GEOMETRIA, não por proveniência.** Não há bandeira «isto veio de um
   `corner_radius`», de propósito: um arco desenhado à caneta ou vindo de SVG é o mesmo facto.
4. ~~**Uma quina acima de `~140°` continua tesselada** e isso é correcto~~ — ⛔ **ESTA NOTA ESTAVA
   ERRADA** (ver o §10): a barra dividia-se pelo nível, e acima do nível 1 a quina ficava tesselada
   a partir de `~90°`, e um CÍRCULO nunca era arco. A cura não foi afrouxar a barra: foi dar-lhe a
   precisão de um quarto de círculo e partir as quinas acima de `90°` na emissão.
5. **O `bulge` positivo é «curva para a ESQUERDA de `a→b`»**, que **não** é a convenção do DXF para o
   sentido. O sentido de varrimento **deriva-se dos ângulos** (com `|bulge| < 1` o arco é o menor),
   nunca de uma convenção decorada — a 1.ª régua assumiu-a e leu `0,065` de erro sobre uma
   decomposição correcta.
6. **O cubo da cena `2` é o CONTROLO**: a fita dele é byte-idêntica (`28` ops, `558` B de WGSL). O
   relógio dele mexe `~12 %` entre corridas — *é essa a dispersão da máquina.*

## §6 — ⚠️ QUATRO premissas minhas que a medição derrubou

1. *«as quinas do vaso são arcos de círculo»* — **não eram**, e é o §3.
2. *«posso pôr os bulges paralelos à polilinha»* — a polilinha deixa de ser a figura (§5.1).
3. *«a régua ponto-a-ponto chega»* — ela acusou uma decomposição correcta com um erro **igual em
   todas as amostras**, que é assinatura da régua (ponto→VÉRTICE em vez de ponto→SEGMENTO).
4. *«o custo do arco tem de ser estimado»* — a auditoria estimou `3×`–`5×` e o medido foi `2,95×` no
   vaso e `8,5×` na cantoneira. *A estimativa acertou a ordem e errou a dispersão entre peças.*

## §7 — ⏳ ABERTO

- ~~A especialização por REGIÃO não usa arcos — ganho por colher, não defeito~~ — ⛔⛔ **ERA DEFEITO,
  e visível** (smoke do dono no mesmo dia, *«arestas ainda visíveis»*): o modo MODEL traça na CPU por
  essa especialização. **CURADO** — ver o §9 abaixo;
- ⏳ **O `select×555`** do vaso por auditar na fita nova;
- ~~⏳ **Quina `> ~140°`**: partir a cúbica em duas na emissão~~ — ✅ **feito** (§10).

## §8 — O smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke
```
A cena `5` é o vaso torneado. Comparar com `PH2D_FIELD_SMOKE=4` (a cantoneira, o maior ganho).

## §9 — ⛔⛔ ADENDA: as faixas que sobreviveram (smoke do dono, mesmo dia)

O modo **MODEL** traça na **CPU**, e a CPU especializa a árvore por região com um `ProfileIndex`
construído da **polilinha densa**: a cura acima só tinha chegado à placa (modo RENDER). Medido na cena
`5`, frente, `1920×1080`: **`12 196` picos de faceta na CPU → `0`**, placa `0` antes e depois.
Registo completo: [`docs/3DModeling/BUGS_3dmodeling.md`](../BUGS_3dmodeling.md) #1 ·
[`docs/Render3d/06`](../../Render3d/06_auditoria_do_vaso.md) Parte III.

**Ficheiros a mais:** `ph2d-field-eval/src/profile_arc.rs` (a lei do arco numa porta só) ·
`profile_winding.rs` (a aritmética do enrolamento, cortada do `profile_index.rs` pelo tecto de LOC:
`758 → 662`) · `profile_dist.rs` (os limites do corte) · `ph2d-app-field3d/src/vaso_sem_facetas_tests.rs`
(o gate do produto, CPU-only).

⚠️ **TRÊS coisas que uma leitura rápida entende ao contrário:**
1. **O `ProfileIndex` passou a indexar a DECOMPOSIÇÃO exacta**, não a polilinha — `edge(i)` devolve a
   CORDA de um arco, e `arco(i)` diz se é um. O corte espacial continua conservador pela flecha.
2. **O `inside` do índice parte de um ponto SEGURO por célula** (`start`), não do canto — defeito da
   W56 que existia para polilinhas também (quase nunca disparava).
3. **A árvore GLOBAL também mudou** (o empate sobre a corda): a Parte II tinha um defeito de sinal
   num conjunto de medida nula que nenhuma grelha apanhava.

**Gates:** `ph2d-field-eval` `118` (+6) · `ph2d-app-field3d` `395` (+1) · **8 mutações, 8 mortas** (duas
sobreviveram à 1.ª ronda e pediram o gate do corte) · arquitectura `378` · casca `815` ·
clippy `-D warnings` limpo.

## §10 — ⛔⛔ ADENDA 2: o arco que não sobrevivia ao botão (auditoria depois do smoke aprovado)

O smoke do §9 foi aprovado no nível de omissão do `Resolution`. Com o botão noutro sítio, **três**
mecanismos devolviam a polilinha, e um quarto defeito recusava figuras (registo:
[`BUGS_3dmodeling.md`](../BUGS_3dmodeling.md) #2 · [`docs/Render3d/06`](../../Render3d/06_auditoria_do_vaso.md) Parte IV):

1. **A barra do reconhecedor** passa a `max(tol, ERRO_DO_QUARTO·r)` — `ph2d-field-profile/src/lib.rs`.
   ⭐ **Um círculo passa a ser `4` arcos** (era `168` segmentos em todo nível).
2. ⛔⛔ **FOUNDATIONAL DE OUTRA LINHA, outra vez: `ph2d-vec-scene`.** `corners::circular_fillet` é a
   porta dos dois arredondadores (`corners::rounded_corner` e `corner_live::fillet_handles`): acima de
   `90°` o filete sai em **duas** cúbicas com um vértice `Smooth` no meio. **Até `90°` a saída é
   byte-idêntica** (a mesma conta, na mesma ordem). ⚠️ **Muda a contagem de vértices cozidos de toda
   quina aguda** — a estrela de 5 pontas com as pontas arredondadas passa de `15` a `20` (o gate em
   `ph2d-vec-edit/src/shape.rs` foi actualizado com a razão). *A `line/Vector` deve saber disto*; o
   desenho 2D fica mais fiel ao raio pedido.
3. ⛔⛔ **O preview (`ph2d-app-field3d/src/preview.rs::coarse_doc`) comparava `segment_count`** e
   trocava os arcos por uma polilinha `7×`–`83×` mais cara nos níveis altos — a mexer e parado.
   Hoje compara `prim_count`.
4. **`ph2d-field/src/profile.rs::Profile::with_arcs`** aceita **duas** primitivas quando uma é arco
   (a meia-lua da caneta) — antes recusava a peça inteira. Duas rectas continuam recusadas.

⚠️ **TRÊS coisas que uma leitura rápida entende ao contrário:**
1. **Nenhum contador partilhado se mexe.** O `Profile` serializado não mudou (a porta só ficou
   menos restritiva), e a geometria cozida do vector não é gravada — o `VecPath` guarda a fonte com o
   `corner_radius`.
2. **`ERRO_DO_QUARTO = 2,7254e-4` não é folga**: é o erro do quarto canónico medido, e o gate
   `o_quarto_canonico_define_a_barra_do_arco` segura as duas metades (cabe e ENCHE).
3. **A cena 5 passou a ser `smoke_scenes::vaso(nível)`** — o mesmo desenho, agora cozinhável noutro
   nível pelos gates; a cena é o nível de omissão.

**Ficheiros novos:** `ph2d-vec-scene/src/corner_split_tests.rs` · `ph2d-field-eval/src/profile_meia_lua_tests.rs`.

**Gates:** `12` mutações, `12` mortas. Impactados (`BASE=HEAD`): `15 236` / `15 237`, e o vermelho
era a contagem da estrela. Workspace inteira (`ci-test`): `22 885` / `22 886` — o vermelho é
`the_cost_of_sampling_a_path_is_flat_in_its_anchors` (família de flakes de carga: load `52`, `3/3`
verde sozinho a load `14–17`, zero linhas na `ph2d-timeline`). Clippy `-D warnings` limpo nas seis
crates tocadas.

⏳ **Os gates de PLACA (`#[ignore]`, `PH2D_GPU=1`) NÃO foram re-corridos nesta adenda:** às 19:00 a
placa estava com o smoke de outra linha (`line/sculpt3d`, cena `47`) e com o Blender, e a regra da
casa para a placa é exclusão ([`TETOS_DE_RECURSO_POR_LINHA.md`](../../DevOps/TETOS_DE_RECURSO_POR_LINHA.md)).
A última corrida deles foi a do §9 (`62/62`). O risco que sobra é baixo e está nomeado: a fita do
dispositivo recebe `12` arcos em vez de `10` no vaso, e agora arcos também nos níveis altos (antes
a polilinha engrossada) — a mesma lei que os `62` já provaram. ⇒ **quem integrar corre-os** com a
placa livre.

**Smoke:** o mesmo comando do §8; subir **Resolution** no painel do modelo e olhar o lábio de perto.

## §11 — ⛔ ADENDA 3: o smoke dirigido prendia o canvas (report do dono, mesmo dia)

*«o Modo model não está permitindo usar o modo Vector»* — com o app aberto por
`PH2D_FIELD_SMOKE=5`. A env armava o módulo sem olhar o painel, e a W42 só curou o caminho do pill.
Cura em `ph2d-app-field3d/src/smoke_requests.rs` (`scene_for`: a env escolhe a cena, o painel liga
e desliga; a abertura automática pergunta à env). Registo: [`BUGS_3dmodeling.md`](../BUGS_3dmodeling.md) #3.
Gate `mode_tests::the_directed_smoke_disarms_with_the_panel_too`, 4 mutações, 4 mortas; impactados
`2 656` / `2 656`.

⚠️ **Uma leitura rápida entende ao contrário:** o `com_env_do_smoke` é `#[cfg(test)]` e sobrepõe a
env **por thread** — o produto lê sempre a env real.

## §12 — ⛔ ADENDA 4: a forma acrescentada a uma raiz-forma não aparecia (report do dono, mesmo dia)

A paleta pendura a forma nova na raiz, e o `promote_leaf_hosts` (**`ph2d-field-ecs`, foundational do
módulo**) saltava a raiz com a nota *«um caso que não existe hoje»* — as cenas `2` e `5` nascem com a
raiz numa forma. Cura: `promote_root_in_place` (a raiz vira a união no mesmo sítio; a forma desce
como primeira filha com geometria, modificadores, verbo, material e vínculo). Registo:
[`BUGS_3dmodeling.md`](../BUGS_3dmodeling.md) #4. 4 mutações, 4 mortas.

⚠️ **Uma leitura rápida entende ao contrário:** a cena `5` **não** foi mudada — ela é a reprodução,
e é o produto que passa a tratar uma raiz-forma. Um projeto gravado com uma raiz-forma e filhos
abre agora com a peça promovida (a Hierarquia ganha uma linha: a forma dentro de «Model»).

⛔ **E dois gates da costura da importação estavam VERDES sobre a peça invisível** (cena `2`, raiz-forma):
contavam a arena do documento, onde o cozimento põe os filhos de uma forma soltos. Passaram a contar
formas e a procurar entre os nós que a raiz alcança (`import_seam_tests.rs`).

## §13 — ADENDA 5: smoke das adendas 3 e 4 APROVADO; a bateria de placa; duas curas de auditoria

**Smoke do dono: OK** (o Vector toma o canvas com o smoke dirigido aberto; o cilindro nasce dentro
de «Model», liso nos níveis altos).

**A bateria de placa (`PH2D_GPU=1 … --ignored`) que ficou devida no §10: `62` de `62`, em três
partes, com a razão de cada uma:**
- `59` passaram na bateria (`1 255 s`, load `14`–`52`, com outra linha a correr testes);
- ⚠️ **dois gates de RELÓGIO reprovaram na bateria e passam sozinhos** (load `12`–`14`, uma thread):
  `na_faixa_do_produto_a_placa_ganha_com_margem` (bateria `1,76×`, sozinho `3,95×`–`7,25×`) e
  `com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento` (bateria `3/18` com uma cena a
  `6 850 ms`; sozinho `13/18`). ⇒ **pedido ao integrador:** promovê-los à família de flakes de
  recurso do `CLAUDE.md` §5.0 (a linha pede, o integrador escreve). Zero linhas desta adenda na
  lei que eles medem;
- ⛔ **um gate de CONTAGEM reprovou por defeito do PRÓPRIO gate, curado:**
  `um_arrasto_nao_reenvia_a_escultura` lia o contador de envios do traçador **partilhado** por todos
  os gates de placa da crate — `[15, 18, 19, 20, 22, 24]` na bateria, `[1, 1, 1, 1, 1, 1]` sozinho
  (3/3). Hoje usa um traçador próprio: verde entre os cinco gates de escultura em paralelo, e a
  mutação que reenvia a grade em todo quadro (`G1`) lê `[1, 2, 3, 4, 5, 6]` e mata-o.

**Duas curas de auditoria em `ph2d-field-ecs`** (registo: `BUGS_3dmodeling.md` #4 e #5):
- **#5** — largar na Hierarquia uma forma sobre outra que não está na origem fazia-a **saltar**
  (`(0,1,0)` → `(−1,1,0)`): a promoção passava os filhos ao grupo sem compor a pose do anfitrião.
  O gate W31 tinha o anfitrião na origem. Gate
  `a_shape_dropped_onto_a_posed_shape_stays_where_it_was`, mutação F1 morta.
- **#4, a metade que ficou nomeada** — o `cook` já não desce aos filhos de uma forma: o documento só
  tem o que a raiz alcança. Gate `the_cooked_arena_holds_only_what_the_root_reaches`, mutação F2
  morta.

Impactados: `2 899` / `2 899`. Clippy limpo.
