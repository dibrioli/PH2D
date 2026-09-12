# `line/app-motion` — FASE B: **o corte está bloqueado, e o bloqueador está medido** (2026-09-11)

> **Veredito:** a família `motion` **não se corta nesta wave**. Ela é **UM** componente ligado de
> `410` ficheiros / `99 845` LOC, e move-se **tudo-ou-nada**. O que a prende não é a `App` (a Fase A
> resolveu isso) nem o trait de host (as 5 portas chegam): são **13 módulos residentes na shell,
> partilhados entre famílias** — dos quais **10 são folhas PURAS** cuja dona não é esta linha.
>
> ⇒ **PARE e reporte**, pela regra 2 do [§1 da Fase B](../../IntegracaoMultiAgente/BLOCOS_ABERTURA_W2_FASE_B_2026-09-11.md)
> e pela regra 5 (*não edite a árvore de outra linha*). É exactamente a categoria que a **batedora**
> nomeou — *«três folhas residentes na shell, partilhadas entre famílias … não é porta e não é sua:
> reporte, é linha própria»* — só que aqui ela **não é uma nota de rodapé: é o caminho crítico
> inteiro**.

---

## §1 — O que a Fase A já tinha resolvido (e por isso não é o bloqueio)

| grandeza | Fase A deixou | leitura |
|---|---:|---|
| blocos `impl App` na família | **0** | os 10 viraram funções livres |
| campos de `App` que a família toca | **6** | `gfx` (46) · `timeline` (4) · `vec_entities` (4) · `motion_shell` (2) · `flip_state` (1) · `playhead` (1) |
| o que as cenas pedem ao `gfx` | **5 tipos** | `motion` (59) · `tools` (19) · `sim` (16) · `vec_scene` (5) · `flip` (5) |

⭐ Pela regra 2 (*«antes de pedir porta, escreva o que a função PRECISA em tipos»*), esses 5 são
**assinatura, não porta**. ⛔ **Zero sextos métodos pedidos ao `AppHost`.**

---

## §2 — O bloqueio, medido quatro vezes

**A família é um componente único.** O grafo tem duas espécies de aresta, e a segunda é dura nos
dois sentidos (é a lei que a `line/app-vec` mediu primeiro):

- **mole** — `crate::motion::X` / `crate::render_loop::motion_X`;
- **dura** — `#[path]`: um pai que declara um filho acoplado **não sai**, e um filho que usa
  `super::` do pai também não.

| modelo | movem | ficam |
|---|---:|---:|
| só arestas moles | 210 / 46 757 | 200 / 53 088 |
| **+ `#[path]` dura** (a honesta) | **15 / 3 365** | 395 / 96 480 |
| + `crate::App` conta como âncora (⚠️ o buraco da régua, §3) | **7 / 1 765** | 403 / 98 080 |
| desenho *«a PONTE fica na shell + os 3 tipos partem-se»* | **9 / 1 991** | 401 / 97 854 |
| **se as 26 âncoras forem curadas** | **410 / 99 845** | **0** |

⭐⭐ **E o contrafactual é a parte que decide:** curar **3** âncoras não liberta nada; curar **7**
também não; **só curar as 26 liberta as 410**. Cada âncora sozinha arrasta **o mesmo** componente de
`384` ficheiros / `93 770` LOC. *Não existe fatia intermédia — a família é indivisível.*

### As 26 âncoras, e o que as prende

| módulo residente na shell | LOC | puro? | âncoras que prende | consumidores por família |
|---|---:|---|---:|---|
| **`vec_entities`** | 268 | **PURA** | **7** | shell 107 · **vec 62** · motion 13 |
| `audio` | 569 | **PURA** | 3 | shell 11 · motion 3 |
| `flip` | 17 931 | 376 refs | 2 | flip 5 · shell 9 · motion 2 |
| `field_gizmo` | 583 | 31 refs | 2 | shell 7 · motion 2 |
| `thumbnail` | 115 | **PURA** | 2 | shell 2 · motion 2 |
| `vec_glyph` · `vec_glyph_build` | 449 · 445 | **PURAS** | 1 · 1 | vec 5+1 · motion 1+1 |
| `brush_live` · `pan_diag` | 223 · 187 | **PURAS** | 1 · 1 | shell 5+2 · motion 1+1 |
| `picker_smoke` | 187 | 9 refs | 1 | **só motion** |
| `render_loop::{warp_gizmo, warp_gizmo_fixtures, flip_pass}` | 383 · 116 · 598 | **PURAS** | 1 · 1 · 1 | shell 3+3 · motion 2+1 |

⚠️ **`vec_entities` sozinho prende 7 das 26**, e ele é **62 % consumido pela família `vec`** contra
**13 %** por mim. Ele aparece como **tipo de PARÂMETRO** (`map: &crate::vec_entities::VecEntityMap`),
e **uma crate não pode nomear um tipo que vive no binário da shell** — logo não há cura por
assinatura: ou a folha se muda, ou eu inventaria uma abstracção através da fronteira de outra
família, que é precisamente o que o HOWTO §1.2 proíbe (*«duas famílias que partilham código
partilham uma FOLHA, nunca uma delas à outra»*).

⭐ **Dois blocos já não são problema** e mostram que o caminho existe: `modal` já é porta da
[`ph2d-app-host`](../../../crates/ph2d-app-host/) e `vec_font` já vive na
[`ph2d-app-vec`](../../../crates/ph2d-app-vec/). **Uma crate PODE depender de outra crate** — o que
é impossível é depender da shell. ⇒ *o bloqueio dissolve-se sozinho à medida que as folhas saem.*

---

## §3 — ⚠️ A régua que eu próprio parti (e o número que ela inflacionava)

A 1.ª medição desta linha deu **210 ficheiros / 46 757 LOC movíveis** e estava **errada em 30×**.
Dois defeitos, os dois na direcção cara (*dizer que dá para mover o que não dá*):

1. **`#[path]` modelado como aresta de um só sentido.** Ele é **duro nos dois**: `210 → 15`.
2. ⛔⛔ **O acoplamento por ACESSO A CAMPO era invisível.** As cenas alcançam a `MotionState` por
   `app.gfx.motion` — um **campo**, não um caminho `crate::`. A régua varria `crate::` e via zero.
   `15 → 7`. *Uma régua que procura um NOME não vê um acoplamento que viaja por um CAMPO.*

⭐ E a régua só ficou honesta quando parou de perguntar *«este ficheiro cita algo de fora?»* e passou
a perguntar **o fecho**: *«a partir daqui, que raízes da shell continuam alcançáveis?»* — que é a
mesma lição que a batedora escreveu no HOWTO §2.12 (*«medir o fecho em vez de reagir ao erro
seguinte»*), e que eu tive de pagar outra vez por não a ter aplicado à primeira.

⚠️ **Todas as contagens deste doc tiram comentários antes de contar** (HOWTO §2.12): sem isso, a
varredura acusa o próprio doc-comment que explica a cura.

---

## §4 — O que NÃO foi feito, e porquê (a decisão que fica para o dono)

⛔ **Não movi os 7 ficheiros livres (`1 765` LOC, 1,8 %).** Eles são o *motion path overlay*, o
*glow dirt* e o `motion_shell_state` — e os consumidores deles são o `input_dispatch.rs` e o
`app_state.rs`, que são **exactamente onde as seis linhas da W2 colidem**. Mover 1,8 % agora cria
superfície de conflito nos dois ficheiros mais disputados do repo, **e será refeito por inteiro**
quando a família sair toda. ⇒ *é churn com custo de merge e sem valor de tecto.*

⚠️ **É uma decisão de julgamento, não uma impossibilidade** — se o dono preferir a fatia, ela é uma
jornada curta e está medida aqui.

⛔ **Nenhum contador partilhado se mexeu**, nenhum `TETO_LOC` foi tocado (regra 1), e a shell está
**byte a byte como o `main`**: esta linha não produziu commits de produto.

---

## §4-bis — ⛔ DECISÃO DO INTEGRADOR (11/09): a linha de folhas NÃO abre agora

> *«Os seus três maiores bloqueadores são território da `line/app-vec`, que está a movê-los NESTE
> momento. Uma linha nova ali colidiria com o vec (54 ficheiros abertos) e com a sculpt3d (115).
> PARE aqui. Não mova produto. Quando o vec e a sculpt3d integrarem, o integrador remede o seu fecho
> e reabre-a.»*

A fatia de 1,8 % fica **de fora**, pela razão do §4. A linha **pára com zero produto movido**.

### ⚠️ Duas correcções para quem remedir (o §5 abaixo tinha-as por medir)

1. **`vec_transform` NÃO é bloqueador desta família** — `0` referências em `motion/` e
   `render_loop/motion_*`. Os três maiores desta família são **`vec_entities` (7 âncoras)**,
   **`audio` (3)** e **`flip`/`field_gizmo`/`thumbnail` (2 cada)**; `vec_glyph` e `vec_glyph_build`
   prendem **1** cada. ⇒ só **um** dos três maiores é território do vec.
2. ⛔⛔ **A lista de 26 NÃO encolhe sozinha quando o vec integrar — encolhe `3`.** Medido:

| | âncoras |
|---|---:|
| que o **vec** destranca sozinho (`vec_entities`, `vec_glyph*`) | **3** de 26 |
| com bloqueador **não-vec** (o resíduo) | **23** |

**O resíduo, por bloqueador:** `crate::App` **10** · `audio` 3 · `flip` 2 · `thumbnail` 2 ·
`field_gizmo` 2 · `flip_pass` · `brush_live` · `pan_diag` · `modal` · `picker_smoke` ·
`warp_gizmo` · `warp_gizmo_fixtures` (1 cada).

⭐ **E os `10` do `crate::App` são MEUS, não da linha de folhas:** pela regra 2 eles são
**assinatura** (as cenas pedem 5 tipos que o `AppGfx` segura), e curam-se dentro desta linha sem
substrato nenhum. ⇒ **o resíduo que justifica a linha de folhas é ~`13` âncoras sobre ~8 módulos**,
com o `audio` (3) à frente — e o `audio` **não** é território do vec, logo não cai com ele.

⚠️ **Como continua a valer o tudo-ou-nada** (§2), curar `3 + 10` não move um ficheiro. O que a
reabertura tem de medir é se o resíduo chegou a **zero**, não se encolheu.

---

## §5 — O que destrava isto (uma linha, não cinco)

Uma **linha das folhas partilhadas** que tire da shell, para crates-folha, os módulos **puros** que
mais famílias consomem. Por ordem de alavanca medida para a `motion`:

1. **`vec_entities`** (268 LOC, pura) — 7 âncoras minhas; e é **da `vec`** por consumo (62 %).
2. **`audio`** (569, pura) — 3 âncoras.
3. `thumbnail` (115) · `brush_live` (223) · `pan_diag` (187) · `warp_gizmo` (383) ·
   `warp_gizmo_fixtures` (116) · `flip_pass` (598) · `vec_glyph` (449) · `vec_glyph_build` (445) —
   todas **puras**, 1–2 âncoras cada.
4. `flip` (17 931) e `field_gizmo` (583) **não** são folhas puras — são território da `line/app-flip`
   e da shell, e pedem decisão própria.

⇒ Depois disso a `motion` corre **de uma vez** (410 ficheiros, `99 845` LOC), porque as 26 âncoras
caem juntas e o componente inteiro fica livre. ⚠️ **E é provável que o mesmo destranque a `sculpt3d`
e a `vec`**: as três partilham `vec_entities`, `field_gizmo` e `thumbnail`.

---

## §6 — A prova desta jornada

| | |
|---|---|
| baseline `nextest list` (na base, antes de mover) | **22 659** testes |
| ficheiros de produto movidos | **0** (ver §4) |
| `shells/desktop/src` | **417 195** LOC — inalterado |
| contadores partilhados | inalterados |
| `TETO_LOC` do `the_shell_only_shrinks` | **não tocado** (regra 1) |
| 6.º método no `AppHost` | **nenhum pedido** (regra 2) |

**O binário do smoke fica COMPILADO** (regra I):

```
$ cargo build -p ph2d-host-desktop --profile smoke     # 1.ª
    Finished `smoke` profile [optimized] target(s) in 23.89s
$ cargo build -p ph2d-host-desktop --profile smoke     # 2.ª — A PROVA
    Finished `smoke` profile [optimized] target(s) in 0.18s
    (linhas "Compiling": 0)
```

⚠️ **A prova do `nextest-list-diff` não se aplica**: ela compara duas listas à volta de um
movimento, e não houve movimento. A baseline fica capturada para quem retomar.
