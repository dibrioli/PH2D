# HANDOFF DE INTEGRAÇÃO — `line/3DModeling`, 2026-09-06

> Para o **agente integrador**. Ordem do Enio: *«escreva handoff para integrar essa linha ao main»*.
> ⚠️ Esta linha **não integra e não pusha** (`CLAUDE.md` §0.7) — fecha, entrega isto e PARA.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/3DModeling` |
| HEAD | `4f87b14e7` |
| merge-base com `main` | `53832c884` |
| commits | **17** |
| ficheiros | **111** |

```
4f87b14e7 perf(3d-modeling): a auditoria da W128 — o divisor corria por LADRILHO (642×)
ba7634bc4 feat(3d-modeling): W128 — a SUPERFORMULA de Gielis
f17a4e05c docs(3d-modeling): a superquadratica sai da fila
ce8b3a47f feat(3d-modeling): W127 — a SUPERQUADRATICA
46e94b6a8 fix(3d-modeling): o ecrã em branco — o report apontava 31 pares, não um
4223c77ed docs(3d-modeling): o placar do levantamento
f232ea1a0 feat(3d-modeling): W125 — o cilindro com bojo, e a ESCADA recusada
006f419fc feat(field): a MOLA e o GYROID
b9dd85526 docs(3dmodeling): o levantamento de formas por FÓRMULA
73c6215b7 feat(field): a ESPIRAL e o DOCUMENTO por fórmula
5f028acf4 feat(field): o LOTE 3 do fluxograma
2b80dfd14 fix(field): a nuvem, e o gate que media toda forma só no ponto em que ela NASCE
4749ab2fe feat(3d): W120 — o LOTE DOS SÍMBOLOS
f121e3174 feat(3d): W119 — o LOTE DA SETA
45851891a fix(undo): a coluna que NUNCA foi vigiada
3c6557a1d fix(gates): os TRÊS vermelhos que as duas curas fizeram nascer
fd009ddc0 fix(3dmodeling): o undo devolve a MÃO, e o espelho deixa de ser um controlo morto
```

---

## 2. ⚠️ OS NÚMEROS QUE SE CONTAM — não os copie, re-conte contra o `main` do dia

Saída de `bash scripts/collision-surface.sh` **em 06/09** (referência, **nunca** evidência — se
outra linha integrar no meio, toda a coluna «base» muda):

```
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        115   (base: 114)
  ⚠   └ tripla do gate               (115, 13, 18)   (base: (114, 13, 18))
    VEC_SCENE_SCHEMA / FLIP_SCHEMA / DOC_VERSION       intocados
▸ REGISTRO DE COMPONENTES        80 / 80   (base: 80 / 80)   intocado
▸ CONTRATO CONGELADO (§6)        nodes e tools INTOCADOS
▸ ADR                            esta linha não cria ADR
▸ Cargo.lock                     nenhum pacote externo novo
▸ MARCADORES DE CONFLITO         nenhum
▸ TETOS DE LOC                   nenhum ficheiro da linha passa do teto
```

### Os quatro números desta linha, com o sítio onde se lêem

| número | valor aqui | onde se lê (⛔ não copie daqui) |
|---|---:|---|
| `PROJECT_SCHEMA` | `114 → 115` | [`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) **e a tripla** em `project_schema_tests.rs` — **três** sítios |
| `FIELD_DOC_VERSION` | `16 → 17` | [`ph2d-field/src/lib.rs`](../../../crates/ph2d-field/src/lib.rs) |
| `PrimitiveKind::ALL` | `48 → 54` | `primitive_kind.rs` (o tamanho do array é literal e **tem de casar**) |
| `CENAS` do smoke | `19 → 25` | `field3d_smoke_scene_tests.rs`, **e as duas notas** em `field3d_smoke.rs` e `main.rs` |

⛔⛔ **O `PROJECT_SCHEMA` sobe por ARRASTO e não é aditivo.** As três variantes de espelho da
`ph2d_field::Unary` passaram de **unidade** (zero bytes) a ter `offset: f32`, e a pilha de
modificadores viaja **posicionalmente** dentro de um `ComponentBlob` opaco. Um `Mirror` gravado num
`v114` lido num `v115` **come os bytes do que vinha a seguir, sem erro nenhum** — é para isso que o
número existe. Quem defende os bytes é o gate
`the_shape_of_a_saved_modifier_stack_is_pinned` (`82 → 94`).

⚠️ **Se outra linha também subir o `PROJECT_SCHEMA`, o valor certo não é nenhum dos dois lados** — e
a colisão **passa MUDA** se as duas escreverem o mesmo literal (`CLAUDE.md` §5.0).

---

## 3. Foundational / partilhado tocado, e por quê

| ficheiro | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-field/src/dims.rs` | **duas variantes novas de `Span`**: `Floor(f32)` e `Range{min,max}` | ✅ aditivo, mas ⚠️ **todo `match` sobre `Span` passa a ter de as cobrir** — são 5 sítios, todos exaustivos de propósito |
| `crates/ph2d-field/src/primitive.rs` | **6 primitivas novas** no enum | ✅ aditivo no fim |
| `crates/ph2d-i18n/src/model3d.rs` | 11 chaves novas | ✅ aditivo — ⚠️ é o ficheiro que outra linha do módulo 3D também tocaria |
| `crates/ph2d-ecs/src/scene/incremental.rs` | ⚠️ **fora do módulo**: a cura do cache incremental que resolvia a lista uma vez com a cena vazia (`45851891a`) | não-aditivo (uma linha de lógica) |
| `shells/desktop/src/render_loop/mod.rs` | uma linha, na mesma cura | não-aditivo |
| `shells/desktop/src/project_schema.rs` | o degrau `114 → 115` | ver §2 |

⛔ **Contratos congelados (§6): NENHUM encostado.** `node.rs` e `tool.rs` intocados.

### Ficheiros PARTIDOS por teto de LOC (o integrador vê-os como renomeação parcial)

| nasceu | cortado de | porquê |
|---|---|---|
| `ph2d-field/src/dims_write_formula.rs` | `dims_write.rs` (721 → 666) | as arms de escrita das formas por fórmula |
| `ph2d-field/src/primitive_family.rs` | `primitive.rs` (706 → 643) | o `match` forma → família |
| `ph2d-field-eval/src/primitive_tree_formula.rs` | `primitive_tree.rs` (723 → 682) | o despacho das formas por fórmula |
| `ph2d-field/src/dims_clamp.rs` | novo | a porta que repõe as invariantes |
| `ph2d-field/src/dims_scale_signs.rs` | `dims_scale.rs` | (W124) |
| `shells/desktop/src/field3d_smoke_scenes_shapes.rs` | `field3d_smoke_scenes.rs` | (W122) |

⚠️ **Se outra linha editou o corpo de um desses ficheiros, o hunk dela cai no lado errado do corte.**

---

## 4. Símbolos novos que podem COLIDIR

- `Span::Floor`, `Span::Range` — variantes de enum **foundational**.
- `PrimitiveKind::{Parallelogram, Delay, Display, OffPage, Document, Spiral, Helix, Gyroid, RoundedCylinder, Superquadric, Superformula}` e as `Primitive::` correspondentes.
- Constantes: `MAX_PARALLELOGRAM_SKEW`, `DELAY_SPAN_OVER_WIDTH`, `MAX_DISPLAY_POINT`, `MAX_OFFPAGE_POINT`, `MAX_SPIRAL_TURNS`, `MAX_SPIRAL_FILL`, `MAX_DOCUMENT_WAVE`, `MIN_GYROID_CELLS`, `MAX_GYROID_FILL`, `MIN/MAX_SUPERQUADRIC_EXPONENT`, `MIN/MAX_SUPERFORMULA_{SYMMETRY,N1,N}`.
- Chaves i18n: `panel.model3d.add.{parallelogram,delay,display,offpage,document,spiral,helix,gyroid,rounded_cylinder,superquadric,superformula}` e `field.dim.{bulge,cell,exponent_top,exponent_side,top_symmetry,top_n1,top_n2,top_n3,side_symmetry,side_n1,side_n2,side_n3}`.
- Cenas de smoke **20–25** (`PH2D_FIELD_SMOKE`).
- Contador público `ph2d_field_eval::ops_gielis::SCANS` (instrumento de gate).

---

## 5. O que SÓ o `ship.sh` apanha (o gate de integração não roda)

- **`typos`** — os docs desta linha têm muito português com acento e nomes próprios (*Gielis*, *squircle*, *Minkowski*). Não passaram pelo `typos` local.
- **`machete`** — **zero dependências novas**, então não deve acusar.
- **`deny` / `audit`** — nada novo no `Cargo.lock`.
- **clippy latente** — corrido `--all-targets --all-features` nas 7 crates que a linha toca; **não** na workspace inteira.
- **`fmt`** — verde na árvore inteira (`cargo fmt --all -- --check`).

---

## 6. Ordem, dependências e o que RE-SMOKAR

Os 17 commits são **sequenciais e dependentes** (cada wave assume a anterior): integre a branch
inteira, não cherry-picks.

### ✅ Smokado pelo Enio (aprovado)

| cena / gesto | veredito |
|---|---|
| lote do fluxograma, espiral, documento, mola, gyroid | *«smoke ok»* |
| `Rounded Cylinder` + o bug do ecrã em branco | *«smoke ok»* |
| `Superquadric` e a cena `=24` | *«smoke OK. Muito bom»* |
| `Superformula` e a cena `=25` | *«smoke OK»*, com o report de performance — **curado depois** (§7) |

### ⚠️ NÃO smokado — re-smokar depois de integrar

1. **A cura da performance da W128** (`4f87b14e7`) chegou **depois** do smoke dele: o memo e os
   atalhos exactos mudam o **preço**, nunca o valor (há gate a prová-lo). ⛔ **E ele já a smokou e
   NÃO viu diferença — ver §6-bis, que é o item ABERTO desta linha.**
2. **O `PROJECT_SCHEMA 115` com um ficheiro gravado antes**: abrir um `.ph2dproj` v114 tem de dizer
   *«Project migrated from format 114 to N»*.
3. **A `Span::Floor` no painel**: as formas afectadas são `RoundCone`, `Drop` e `BentArrow` — o
   slider delas passou a ter piso, e o piso **move-se** com as irmãs.

---

## 6-bis. ⛔⛔⛔ ABERTO — a minha régua diz `−42 %` e o OLHO DO DONO diz que não

> **Enio, 06/09, depois da cura:** *«não houve melhora significativa. mas vamos deixar a revisão
> para depois da integração»*.

⚠️ **Isto NÃO está resolvido, e não deve ser lido como se estivesse.** A cura é real e está gateada
(`3 852 → 0` varreduras por quadro, e o atalho exacto dos expoentes `1` e `2` é álgebra, não
aproximação) — mas *uma melhoria que o dono não vê é uma melhoria que ainda não chegou ao produto*,
e é a mesma lição que o quad remesh desta casa pagou quatro vezes: **uma barra calibrada sem o lado
que o dono aprovou mede os nossos defeitos, não os dele**.

### As hipóteses, por ordem de probabilidade — a próxima janela MEDE, não escolhe

1. ⭐⭐ **A régua mediu a cena errada.** As minhas leituras são `640×360` num arnês de teste; ele corre
   o **pill MODEL** numa janela cheia, com a câmara a mexer. A `1920×1080` são `9×` os pixels: `3,5`
   e `5,8 ms` viram `~32` e `~52` — **os dois muito acima do quadro de `16,7 ms`**, e a diferença
   entre duas coisas lentas não atravessa o limiar do que se **sente**. ⚠️ *Uma sonda que arma o
   módulo por env var mede outro programa que o pill* — esta linha já tem essa memória.
2. ⭐ **O que ele sente pode não ser a marcha.** O quadro de MOVIMENTO tem duas metades (a marcha e o
   contorno), e a W71 mediu a marcha em `80 %` de um quadro **de extrusão**. Numa superfórmula sem
   perfil nenhum a repartição pode ser outra. **Meça as duas metades separadas antes de optimizar
   qualquer uma.**
3. **O tecto pode não ser desta forma.** A base do módulo já está declarada acima do orçamento
   (doc 06 §13.0: o quadro de movimento custa `26,7 ms` contra `16,7`), e uma forma cara em cima de
   uma base cara move pouco a percepção.

### O que a próxima janela tem de fazer ANTES de tocar em código

- ⛔ **Medir no PILL, na resolução dele, e não no arnês** — com a mesma peça e a mesma câmara, antes
  e depois de `4f87b14e7` (`git stash` não; use duas árvores ou `git checkout` do commit anterior).
- ⛔ **Perguntar-lhe o que «significativa» quer dizer aqui** — se é *«deixa de engasgar ao rodar»*, a
  régua é o **quadro de movimento**, não o assente; se é *«a peça aparece mais depressa»*, é o
  refinamento.
- ⚠️ **E aceitar a resposta se ela for «a forma é assim»** — o preço tem uma escada medida
  (`caixa 0,7× · esfera 1,0× · superfórmula 1,6×`), e o mecanismo é a trigonometria por amostra.
  *Se o tecto for esse, a cura é outra coisa e não esta forma.*

---

## 7. ⚠️ SETE coisas que uma leitura rápida do diff entende AO CONTRÁRIO

1. **O bug do «ecrã em branco» não era da forma nova.** O report do Enio apontava o
   `RoundedCylinder`, e o censo achou **31 pares (forma, linha)** em ~20 formas, a maioria
   pré-existente. A cura é uma porta (`clamp_dims`) **derivada da tabela de faixas**, não uma lista.
2. **`the_fillet_reaches_every_edge_of_every_shape` ganhou uma entrada, e não foi afrouxado.** A
   `superformula` entrou no `APEX_EXCEPTION` **com o vinco localizado** (`0` de `791` pontos nos
   polos; todos num meridiano) e o censo de obsolescência dela passou a saber medir uma forma **sem
   filete**.
3. **A ESCADA foi construída QUATRO vezes e RECUSADA** — não é trabalho pendente. O filete dela é
   neutro em volume (`20 139 = 20 139`) e o gate que o exige está certo (doc 06 §126.4).
4. **O `MAX_SUPERFORMULA_N = 4` não é conservadorismo:** o custo é uma **bacia** com mínimo em `2` e
   sobe para os **dois** lados.
5. **O memo do `ops_gielis` não é o cache que a W53 recusou.** Aquele guardava um resumo do
   **documento** (o undo podia envenená-lo); este guarda o valor de uma **função pura**.
6. **A `superformula` é mais barata que a `superquadric`** depois da cura (`0,9×`) — o atalho exacto
   dos expoentes `1` e `2` tira o `atan2` inteiro da curva neutra.
7. **`ops_gielis::SCANS` é público de propósito** — é o instrumento de um gate de **custo**, e um
   defeito só de custo é invisível a todo gate de imagem.

---

## 8. Vermelhos conhecidos e NÃO causados por esta linha

| teste | evidência |
|---|---|
| `an_abandoned_march_returns_nothing_and_returns_fast` | flake de carga **já declarada** no `CLAUDE.md` §5.0 · **zero linhas** do diff em `ph2d-field-render/src/` · **3 de 3** verde sozinha a `load 1,1` |
| `only_the_lower_row_breathes_and_it_moves_with_the_playhead` | idem (demos de áudio) · **3 de 3** verde sozinha a `load 10,2` |

---

## 9. Estado do gate de fecho

| | |
|---|---|
| suítes verdes | **293** em 6 crates (`ph2d-field`, `-eval`, `-ecs`, `-render`, `ph2d-i18n`, `ph2d-host-desktop`) |
| censo de primitivas | 26 gates, **54** formas |
| `cargo fmt --all -- --check` | ✅ |
| `clippy --all-targets --all-features` | ✅ nas 7 crates tocadas |
| tetos de LOC | ✅ (três ficheiros **partidos** para lá chegar — §3) |
| binário do smoke | **compilado** em `target/release/ph2d-host-desktop` |

---

## 10. A UMA LINHA para o `CLAUDE.md` §5

> Substituir a linha **Aberto** do módulo *3D Modeling* por (e **não** acrescentar parágrafo):

```
⭐⭐⭐ **A paleta foi de 47 para 63 entradas** (`PrimitiveKind::ALL` = 54) com as famílias por
FÓRMULA — fluxograma, espiral, documento, mola, gyroid, cilindro com bojo, superquadrática e a
**superfórmula de Gielis** —, e o vocabulário de faixas ganhou `Span::Floor` e `Span::Range`.
⛔ **O ecrã em branco ao encolher um número está CURADO e era uma FAMÍLIA de 31 pares** (a porta
repõe as invariantes, derivada da tabela). ⛔ **Recusas medidas: o OVO e a ESCADA** (doc 06 §126).
⏳ **Faltam 10 formas** do levantamento — o placar conta-se no
[doc 08 §7.6](docs/3DModeling/08_formas_por_formula.md), nunca de memória.
Mecanismo: [handoff de 06/09](docs/3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_2026-09-06.md).
```
