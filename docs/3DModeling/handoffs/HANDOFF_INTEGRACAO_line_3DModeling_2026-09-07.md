# HANDOFF DE INTEGRAÇÃO — `line/3DModeling`, 2026-09-07

> Para o **agente integrador**. Ordem do Enio: *«smoke OK. Antes de seguir vamos integrar ao main.
> Escreva handoff»*.
> ⚠️ Esta linha **não integra e não pusha** (`CLAUDE.md` §0.7) — fecha, entrega isto e PARA.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/3DModeling` |
| HEAD do **código** | `3890903cb` |
| HEAD real | o commit **deste handoff** (o mais recente do ramo), que traz também o degrau do `FIELD_DOC_VERSION`. ⚠️ **Sem hash de propósito:** ele é o próprio commit que se está a escrever, e um número aqui envelhece a cada `--amend` |
| merge-base com `main` | `815555aed` |
| commits | **10** de produto **+ 1** de fecho |
| ficheiros | **92** (+9 220 / −836), mais os 3 do fecho |

```
3890903cb feat(3d-modeling): W135 -- a ROSCA e o SERRILHADO, e a cunha INFINITA que ganhava o min dentro da peca
42561e9d9 fix(3d-modeling): W134b -- os TRES defeitos de forma do report, e o tecto que e' do MODELO
a96cb5197 feat(3d-modeling): W134 -- o NO DE TORO (p,q), e as tres reguas que eu corrigi antes do algoritmo
a1342f778 feat(3d-modeling): W133 -- os VERTICES no canvas, e o arrasto pela alca
d4306f9a9 docs(memoria): as quatro licoes da W132
4c47cdcd2 feat(3d-modeling): W132 -- o POLIGONO de N vertices, e as DUAS notas que a medicao derrubou
3c9fd46c1 docs(3d-modeling): o triangulo sai da fila -- faltam 9
d4f3f17e8 chore(3d-modeling): as tres queixas do clippy nas sondas da W131
5ef6ceeba feat(3d-modeling): W131 -- o TRIANGULO escaleno, e a lei da casa que eu violei
d881bc1e8 docs(3d-modeling): por que a cura de -42 % nao se SENTE -- a regua media outra resolucao
```

⚠️ **A ordem é sequencial e não há ramos** — `d881bc1e8` fecha a jornada anterior (a régua da
resolução), e daí em diante cada wave assenta na anterior. **Não reordene**: a W134b depende do
tecto que a W134 introduziu, e a W135 lê o `thread_round_limit` que só existe nela.

---

## 2. ⚠️ OS NÚMEROS QUE SE CONTAM — não os copie, re-conte contra o `main` do dia

Saída de `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` **em 07/09**
(referência, **nunca** evidência — se outra linha integrar no meio, toda a coluna «base» muda):

```
SUPERFÍCIE DE COLISÃO — line/3DModeling contra main
  merge-base 815555aed   ·   10 commit(s)   ·   92 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        121   (base: 121)
      └ tripla do gate               (121, 13, 22)   (base: (121, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)                  82   (base: 82)
    ph2d-script (espelho)                  82   (base: 82)
▸ CONTRATO CONGELADO (§6)                  nodes e tools INTOCADOS
▸ ADR                                      esta linha não cria ADR
▸ Cargo.lock                               nenhum '+name' novo
▸ MARCADORES DE CONFLITO                   nenhum
▸ TETOS DE LOC                             nenhum ficheiro da linha passa do teto
```

⭐ **Esta linha não move nenhum dos quatro schemas da sonda.** O `FieldDoc` é **cozido da cena a
cada quadro** e não é persistido; o que viaja no `.ph2dproj` são os componentes ECS, e nenhum deles
mudou de forma.

### ⛔⛔ E há um número que a sonda NÃO vê, e esta linha move-o

| número | aqui | base | onde se lê |
|---|---:|---:|---|
| **`FIELD_DOC_VERSION`** | **18** | 17 | [`crates/ph2d-field/src/lib.rs`](../../../crates/ph2d-field/src/lib.rs) |

⚠️⚠️ **O `collision-surface.sh` conta `PROJECT_SCHEMA`, `VEC_SCENE_SCHEMA`, `FLIP_SCHEMA` e o
`DOC_VERSION` da timeline — e o `FIELD_DOC_VERSION` não está na lista dele.** Duas linhas que o
subam em paralelo fundem-se **sem que a sonda diga nada**, e o git não sabe o que o número
significa (`CLAUDE.md` §5.0). ⇒ **conte-o à mão** antes de fundir, e considere acrescentá-lo à
sonda.

⚠️ **E o degrau estava em dívida há cinco waves:** as W131–W135 acrescentaram **quatro** variantes
ao `Primitive` e nenhuma o subiu. Foi achado no **fecho**, não nas waves. A subida é um degrau só
(as quatro variantes são **acrescentadas no fim**, nenhum índice existente se move, e nada persiste
um `FieldDoc`) — a razão de subir é a lei que o degrau **v7** desta mesma escada já escrevia.

### Os outros números desta linha, com o sítio onde se lêem

| número | aqui | base | onde se lê (⛔ não copie daqui) |
|---|---:|---:|---|
| `PrimitiveKind::ALL` | **58** | 54 | [`primitive_kind.rs`](../../../crates/ph2d-field/src/primitive_kind.rs) |
| entradas do catálogo | **68** | 63 | `grep -c 'key: "panel.model3d.add' shells/desktop/src/field3d_shapes.rs` |
| `CENAS` do smoke | **29** | 25 | [`field3d_smoke_scene_tests.rs`](../../../shells/desktop/src/field3d_smoke_scene_tests.rs) |

---

## 3. Símbolos novos que podem COLIDIR

**Variantes de `enum` (acrescentadas NO FIM, nunca no meio):**
`Primitive::Triangle` · `Primitive::Polygon` · `Primitive::TorusKnot` · `Primitive::Thread`, e os
`PrimitiveKind` homónimos.

**Módulos novos em `ph2d-field`:** `polygon` · `knot` · `thread`.
**Módulo novo em `ph2d-field-eval`:** `ops_triangle` · `ops_knot` · `ops_thread`.

**Constantes públicas novas** (todas em `ph2d_field`, todas re-exportadas no `lib.rs`):

```
MIN_POLYGON_VERTICES = 3     MAX_POLYGON_VERTICES = 27    POLYGON_TOLERANCE
MIN_KNOT_WINDS = 1           MAX_KNOT_WINDS = 8           KNOT_FOOTPRINT = 0.30
MIN_KNOT_LOOPS = 1           MAX_KNOT_LOOPS_OVER_WINDS = 4   KNOT_CORD_MARGIN = 0.85
MIN_THREAD_STARTS = 1        MAX_THREAD_STARTS = 128      THREAD_CORE_FLOOR = 0.25
MIN_THREAD_HANDS = 1         MAX_THREAD_HANDS = 2
MIN_THREAD_FLANK_DEG = 5.0   MAX_THREAD_FLANK_DEG = 75.0
funções: polygon_points · polygon_profile · with_one_more_vertex · with_one_fewer_vertex
         knot_cord_ceiling · max_knot_loops · thread_depth_ceiling · thread_round_limit
```

**Chaves de i18n novas** (todas em `crates/ph2d-i18n/src/model3d.rs`, e o ficheiro é **partilhado**):

```
panel.model3d.add.triangle · .polygon · .torus_knot · .thread · .knurl
field.dim.cord · .knot_p · .knot_q
field.dim.thread_depth · .thread_flank · .thread_starts · .thread_hands
field.dim.vertices · .ax .ay .bx .by .cx .cy · .v1x…v27y   (54 chaves do polígono)
```

⚠️ **As 54 chaves `vNx`/`vNy` são uma família derivada de `MAX_POLYGON_VERTICES`** — se outra linha
acrescentar chaves ao mesmo `match`, o conflito textual é resolúvel por concatenação (nenhuma
sobrepõe as outras), mas **confira que nenhuma das duas escreveu a mesma chave**.

**Cenas de smoke novas:** `PH2D_FIELD_SMOKE=26|27|28|29`. ⚠️ O número da próxima **conta-se** no
[`field3d_smoke_scenes.rs`](../../../shells/desktop/src/field3d_smoke_scenes.rs), nunca de memória.

---

## 4. Contratos congelados encostados

**NENHUM.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` estão intocados — a sonda confirma-o acima. Esta linha não cria ADR.

---

## 5. Foundational / partilhado tocado, e por quê

| ficheiro | o que mudou | aditivo? |
|---|---|---|
| `crates/ph2d-i18n/src/model3d.rs` | **+72 chaves** no `match` do módulo 3D | **sim** — só linhas novas no mesmo braço |
| `shells/desktop/src/main.rs` | a nota do smoke passou de `1..25` para `1..29` (duas palavras) | sim |
| `shells/desktop/src/field3d_*.rs` (27 ficheiros) | **são a shell DESTE módulo** — gizmo de vértices, catálogo, cenas | sim |

⭐ **Nada fora do módulo 3D e do `ph2d-i18n`.** ⛔ Nenhum ficheiro de `ph2d-editor-core`,
`ph2d-core`, `ph2d-ecs` ou `ph2d-tokens` foi tocado.

⚠️ **`crates/ph2d-field/src/profile.rs` recebeu prosa que veio do `primitive.rs`** (a lei do eixo Y
da revolução e a proibição de cruzar o eixo). É movimentação de comentário, **zero** mudança de
comportamento — mas se outra linha editou a mesma prosa no `primitive.rs`, o merge vai parecer uma
supressão. *Ela não foi apagada: mudou de ficheiro*, e o motivo está escrito nos dois sítios (o
`primitive.rs` estava no tecto de LOC e a W135 precisava das linhas).

---

## 6. O que SÓ o `ship.sh` apanha (o gate de integração não corre)

- **`typos`** — os docs desta linha trazem muito português técnico; nada foi verificado pelo
  dicionário do CI.
- **`machete` / `deny` / `audit`** — ⛔ **nenhuma dependência nova**, então o risco é o de deriva
  pré-fork, não desta linha.
- **`clippy --all-targets` da workspace INTEIRA** — aqui correu em `ph2d-field`, `-field-eval`,
  `-field-ecs`, `ph2d-i18n` e `ph2d-host-desktop`, todos limpos. Uma crate que esta linha não toca
  pode ter deriva vinda de outro sítio.
- **`fmt` da árvore** — limpo aqui; ⚠️ ele **re-expande** o que o `rustfmt` reordena, e esta linha
  já foi mordida por isso (o `pub mod thread` que o fmt moveu para a ordem alfabética).

---

## 7. Gate batched do fecho — o que correu, e com que carga

| | resultado |
|---|---|
| `the_census_of_every_primitive` | **28 / 28** (`load 0,38`, 275 s) |
| `measure_sharp_edges` | **10 / 10** (2 dos quais estavam VERMELHOS e foram curados nesta wave) |
| `the_thread_that_winds_a_cylinder` | **10 / 10** |
| `the_knot_that_two_counts_make` | **11 / 11** |
| `ph2d-field-eval` (suíte inteira, release) | **20 suites, 0 falhas** |
| `ph2d-field` (suíte inteira, release) | **53 / 53** (depois do degrau do `FIELD_DOC_VERSION`) |
| `ph2d-host-desktop` filtrado por `field3d` | **358 / 358** |
| `clippy --all-targets` (5 crates) · `fmt --check` | limpos |
| tectos de LOC | `primitive.rs` **700/700** · `field3d_shapes.rs` **600/600** — ambos NO tecto |

⚠️⚠️ **Os dois ficheiros no tecto exacto são uma armadilha para a próxima wave:** uma linha a mais
em qualquer um deles é vermelho. O `primitive.rs` tem a lei escrita no próprio doc — *toda primitiva
nova paga a entrada dela mudando a prosa de uma variante antiga para o módulo do mecanismo* —, e a
W135 pagou-a movendo o `Revolve` para o `profile.rs`.

---

## 8. ⚠️ SEIS coisas que uma leitura rápida do diff entende ao contrário

1. **`FIELD_DOC_VERSION 17 → 18` não é uma mudança de formato desta wave** — é o degrau que **cinco**
   waves deviam. As quatro variantes vão no **fim** do enum e nenhum documento se lê errado; o degrau
   sobe pela lei, não por necessidade.
2. **O `Thread` NÃO é a `Helix` com um cilindro à volta.** A mola mede a distância a uma **CURVA**;
   a rosca é um **perfil varrido por movimento de parafuso**, e é isso que faz o factor da recta
   tangente fechar em forma fechada (`k = 1/√(1 + β²n_w²)`) em vez de pedir a correcção de curvatura
   que o nó pagou.
3. **`starts` não acrescenta um ramo à árvore** — ao contrário do `p` do nó. Ele engorda o divisor, e
   quem paga é a **marcha**. ⛔ Uma sonda que contasse nós diria que ele é grátis.
4. **O `.max(-dr)` no `ops_thread.rs` não é uma guarda defensiva** — sem ele o `max` dos dois flancos
   é uma cunha **infinita** que ganha o `min` a `ρ = 0,013`, onde `‖∇f‖` lê **2,4562**. E **nenhuma
   régua de forma o via**: a secção, o volume e a silhueta lêem `0,000 %` nos dois lados.
5. **`e_raiz` e `e_cruz` levam `round` e `chamfer = 0,0`, e isso é deliberado.** Um chanfro numa quina
   **côncava** não corta — enche. Com duas raízes por crista ele somava o dobro do que tirava
   (`50 290 → 50 358`).
6. **As duas entradas do `APEX_EXCEPTION`/`TANGENT_JOIN_EXCEPTION` da rosca não são licenças** — cada
   uma traz a cura **construída e medida**, e o número que a recusou (`‖∇f‖ = 3,71` contra `1`, e o
   `slab_and_walls` a dizer por escrito que a cura dele só serve perfis que são **intersecção**).

---

## 9. As premissas que a MEDIÇÃO derrubou nesta linha

| eu escrevi | o que a medição deu |
|---|---|
| *«a volta mais próxima do nó sai de um `round()`»* (plano 09) | ⛔ são `p` fios a cortar cada plano meridiano ⇒ um **`min` sobre `p` ramos** |
| *«o minorante sai de dividir pelo gradiente máximo»* | ⛔⛔ isso **ENGORDA** a peça: o zero de `m·c − corda` está em `m = corda/c`. A folga multiplica o campo **inteiro** |
| *«a rosca é a hélice varrida num cilindro»* (plano 09) | ⛔ é um **perfil**, e a intersecção que o plano previa é uma **união** |
| *«a parede do cilindro é ortogonal à tampa»* (comentário meu, no código) | ⚠️ certo sobre o cilindro e **cego ao arranque**, que encontra a tampa a `30°` |
| *«cada recuo sozinho já está dentro do balde»* ([`edge_shrink`], pré-existente) | ⛔ refutado com número: `3,71` num diedro agudo. A constante foi calibrada num corpus cujo pior caso era `120°` |
| *o zero das contagens é COAGIDO para o mínimo* (1.ª redacção de um gate meu) | ⛔ a faixa `Count` não o oferece e a porta **recusa** — a régua estava errada, não o produto |

---

## 10. O que SMOKAR depois de integrar (e o que o Enio já aprovou)

✅ **Aprovado por ele nesta jornada:** o *Triangle*, o *Polygon(N)* com os vértices no canvas, o
*Torus Knot* (duas rondas — a 2.ª curou os três defeitos de forma que ele fotografou), e a *Thread*
com o *Knurled Grip*.

⚠️ **O que a integração pode partir e que só um smoke vê:**

```
cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --release
```

1. **MODEL** → **A** → a paleta tem de listar **68** entradas, com *Thread* e *Knurled Grip* na
   família **Round** e *Torus Knot* na **Rings**.
2. Cada uma das cinco formas novas tem de nascer **com filete** (a lei
   `every_new_shape_that_can_round_is_born_round`) — ⛔ se alguma nascer de aresta viva, uma fusão
   comeu o construtor dela.
3. `PH2D_FIELD_SMOKE=26`, `=27`, `=28` e `=29` — as quatro cenas novas. A `=29` põe quatro roscas
   lado a lado e imprime uma linha a descrevê-las: **se a linha não aparecer, PARE**.
4. As linhas do painel de cada forma nova têm de ter **rótulo**, não a chave crua (uma chave sem
   tradução pinta o identificador e vaza por quadro).

---

## 11. Estado da fila, e a UMA LINHA para o `CLAUDE.md §5`

A fila de formas fecha em **6** (era 10 no início da jornada anterior): saíram o *Triangle*, o
*Polygon(N)*, o *Nó de toro* e a *Rosca*. ⭐ **A §7.4 do [doc 08](../08_formas_por_formula.md) —
as famílias fora de catálogo — fecha INTEIRA**: o que fica lá são os dois **modificadores** (grade
hexagonal, metabolas) e os **fractais**, e nenhum dos três é uma forma.

**Ficam na fila (6):** Bezier · Parabola · Circle Wave (2D) · Plane · Death Star · Vesica Segment.
⚠️ O **Plane** não é uma forma a construir — é a bola de recorte a admitir uma peça **infinita**.

⏳ **ABERTO e nomeado nesta linha:**
- a `sd_helix` usa a **corda crua** e engorda o tubo `1/c` (invisível só porque numa mola típica
  `c = 0,992`). ⭐ A ferramenta para a curar está agora escrita **duas** vezes (W134 e W135), e para
  uma curva ela é `hypot(dr, dz·sin β)`, que é **exactamente** 1-Lipschitz;
- o **arranque** da rosca fica afiado, com a cura medida e recusada (§8.6) — a saída de produto é um
  **chanfro de entrada na ponta**, que é outra feature;
- o `FIELD_DOC_VERSION` **não está no `collision-surface.sh`**;
- a revisão de performance da **superfórmula** continua aberta (§0-bis do plano 09);
- ⚠️ o [doc 06](../06_resultados_cena_e_gizmo.md) está em **~850 KB**, muito além do joelho de
  80–110 KB — um `python3 scripts/doc-split.py` é devido, e **não** foi feito nesta jornada.

**A linha do §5 (substituir a que lá está sobre o catálogo de formas):**

> ⭐⭐ **O catálogo tem 68 entradas sobre 58 primitivas** (W131–W135: *Triangle*, *Polygon(N)* com
> os vértices arrastáveis no canvas, *Torus Knot* `(p,q)`, *Thread* e *Knurled Grip*), e a fila de
> formas por fórmula fechou de 10 para **6** — [doc 09](docs/3DModeling/09_plano_das_dez_que_faltam.md).
> `FIELD_DOC_VERSION` **17 → 18**. Cenas **`=26`..`=29`**.

---

## 12. Higiene do fecho

- ✅ `rm -rf target/*/incremental` corrido **depois** do gate e deste handoff.
- ✅ **Binário de release compilado e a par do código** (`fontes mais novas que o binário: 0`) — o
  Enio não paga um build de release para smokar.
