# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, a DEMÃO idêntica ao Blender

**Data:** 2026-08-16 · **branch:** `line/sculpt3d` · **tip:** `b4bec8b88`
**Ordem que abriu a jornada:** *"a tarefa é tornar Layer Tool idêntico ao Blender. GO!!!!!!!!!"*
**Supersede**, apenas como *o que integrar agora*, o
[`HANDOFF_line_sculpt3d_LAYER_2026-08-16.md`](HANDOFF_line_sculpt3d_LAYER_2026-08-16.md)
— ⚠️ *aquele continua a ser a fonte do mecanismo do porte do `layer.cc` (§4.1, as
onze leis conferidas) e não foi copiado para cá.*

---

## §0 — O que integra, numa frase

O `Verb::Layer` tinha **uma** divergência real contra o `layer.cc`, e ela está
curada; as **outras duas** que o handoff anterior listava como abertas **caíram
por medição** e ficam escritas para ninguém as reabrir.

---

## §1 — O report, e o veredito de cada eixo

O Enio reportou dois eixos: *"se aumentar **hardness** ou **Auto Smooth**, Layer
fica muito ruim"*.

| o que o handoff anterior deixou | veredito de hoje | número |
|---|---|---|
| **§4.2 — o front-face é incondicional** (hipótese líder) | ✅ **REAL, curada** | mesa `0,7828` × rampa `0,3793` (1 dab, h=0,9) |
| **§3(a) — a altura colapsa com dureza alta** | ⛔ **REFUTADA** | ela **SOBE** `0,0735 → 0,0974`; esfregando fecha em **99,93%** da meta |
| **§3(b) — o pente é caráter de kernel** | ⛔ **REFUTADA** | o platô ondula **0,0093 de UMA aresta** — é a **parede** a escadear pela grade |
| **o `auto_smooth` aniquila a demão** | ✅ **já curado pelo porte da W8** | `0,00164 → 0,06975`, reconfirmado hoje |

⚠️ **A ordem em que isto foi feito é o que o tornou barato:** o handoff mandava
*"meça o FACING numa esfera, com dureza alta, comparando o PERFIL e não o pico"*
(§5.2b) e **prescrevia a cura** — *"não apague o `Continuous` do modo `B`; faça-o
virar o flag por-pincel que a fonte tem, com o default desligado"*. Foi
exactamente isso. Nada aqui foi inventado.

---

## §2 — A divergência, e por que ela é da FONTE

Em **cinco** arquivos do Blender (`layer.cc:149` · `clay_strips.cc` ·
`sculpt_cloth.cc` · `paint_color.cc` · `draw_face_sets.cc`) o facing corre atrás
de um `if`:

```c
if (brush.flag & BRUSH_FRONTFACE) {
  calc_front_face(cache.view_normal_symm, vert_normals, verts, factors);
}
```

⚠️ **O bit é o checkbox *"Front Faces Only"*** (`use_frontface`,
`properties_paint_common.py:1354`) e **nenhuma linha do Blender inteiro o LIGA** —
varrido: o único acerto fora de leitura é `use_front_face_ = brush_->flag &
BRUSH_FRONTFACE`, que também lê.

Aqui ele era **lei de MODO** (`FrontFace::Continuous` no `B`), e ⚠️ **a demão cai
no `B` por ACIDENTE**: `profile_s(Layer)` devolve `None`, o `for_verb` recua
(`ref_mode.rs:471`) e o `B` liga o facing. *A demão herdava um `cos` de um modo em
que ela nunca esteve.*

**A cura tem duas metades e as duas são da fonte:** a **LEI** continua a ser do
modo (o `calc_front_face` existe sempre) e o **INTERRUPTOR** passa a ser do
pincel (`Brush::front_faces_only`, default por VERBO, `Layer → false`).

⚠️ **⛔ Não apague o `Continuous` do `B`** — o handoff anterior já o dizia, e a
razão é que ele mudaria em silêncio os **outros** verbos que caem lá.

### §2.1 — O regime importa, e isto tem de estar escrito

Medido na esfera, dureza 0,9, borda/centro:

| dabs | desmarcado | marcado |
|---|---|---|
| 1 | **0,7828** (mesa) | **0,3793** (rampa) |
| 2 | 0,8186 | 0,4708 |
| 8 | 0,9162 | 0,7384 |
| **32** | **0,9831** | **0,9824** ← convergem |

⚠️ **O facing é uma TAXA nesta lei, não um perfil** — a demão SATURA
(`coat_step`), então esfregando o bastante os dois mundos chegam ao mesmo lugar.
A divergência é visível no **gesto rápido**, que é onde a foto do artista foi
tirada. *Um A/B que só esfregasse até saturar mediria zero e chamaria a lei de
inerte.*

⚠️ **Numa GRADE plana o facing é `1,0000` em toda parte** (medido) — e é por isso
que **nenhum dos 16 gates da W8 podia vê-lo**. A fixture nova é uma ESFERA com
raio de dab `0,9`, onde ele varre `0,5800 … 1,0000`; um dab pequeno no polo mal
sai de `1,0` e um A/B ali mede zero.

---

## §3 — As duas refutações, com o mecanismo

### §3.1 — A altura NÃO colapsa

A §3(a) dizia *"a nossa colapsa com dureza alta; a do Blender não — é a
divergência que se mede com um número"*. Medido pela porta do produto:

| hardness | relevo (demão) | relevo (Draw, controle) |
|---|---|---|
| 0,00 | 0,07354 | 0,08738 |
| 0,50 | 0,09105 | 0,11578 |
| 0,90 | **0,09737** | 0,13420 |

E **esfregando** (12 dabs, `layer_height = 0,1`): o platô fecha em **0,09993 =
99,93% da meta autorada**. A altura **sobe** com a dureza e chega onde o artista
mandou.

### §3.2 — O pente é a GRADE

A §3.1 do handoff a tornou falsificável: *"o período do pente tem de ser o passo
da grade"*. ⚠️ **A medição direta é mais barata que uma autocorrelação e separa as
duas hipóteses sem escolher um número:** com `hardness = h` o
`apply_hardness_to_distances` manda toda distância abaixo de `h` para **zero**, a
curva satura, e a lei do `layer.cc` leva todo vértice de mesmo `shape` à **mesma
altura absoluta** ⇒ se o platô ondula, é kernel.

| hardness | vértices no platô | altura média | ondulação / **aresta** |
|---|---|---|---|
| 0,25 | 221 | 0,09999 | **0,0012** |
| 0,50 | 885 | 0,09997 | **0,0041** |
| 0,75 | 2045 | 0,09995 | **0,0078** |
| 0,90 | 2973 | 0,09993 | **0,0093** |

⚠️ **A régua é a ARESTA da malha, nunca um épsilon escolhido** — uma ondulação
menor que o espaçamento de vértices não tem como aparecer na tela, e um limiar
absoluto envelheceria na primeira mudança de subdivisão.

⇒ **O platô é chato a 1% de uma aresta. O pente é a PAREDE a escadear pela grade
de quads**, e o handoff já avisava: *"a topologia do Blender na foto dele é
OUTRA"* e *"a do Blender TAMBÉM é feia"*. ⛔ **Persegui-lo é caçar o alvo errado.**

---

## §4 — A UI, e a porta que a torna alcançável

Caixa **`Front Faces Only`**, logo abaixo do Accumulate (a ordem do Blender).

⚠️ **Oferecida pela porta `Brush::offers_front_faces`**, o molde exacto do
`Verb::accumulates`: o **pintor** pergunta para OFERECER e o **roteador** pergunta
para HONRAR o clique — duas cópias divergiriam num controle que aparece e não move
um vértice. O **kernel não a chama**: ele lê o flag dentro do `match` sobre a lei,
onde o braço `Ignored` já o torna inalcançável.

⚠️ **Ela responde *"a lei existe"*, nunca *"o flag está ligado"*** — uma caixa que
se escondesse quando desmarcada seria uma caixa que ninguém consegue marcar, e o
default É desmarcado.

### §4.1 — A lei de re-arm tinha DUAS cópias, e elas já divergiam

O doc do `state::arm_verb_defaults` avisava: *"quatro `if` copiados no sítio de
uso é como o quinto campo nasce esquecido"* — e o atalho de teclado da shell
armava **dois** dos quatro, no mesmo dia em que o quinto ia nascer.

Os três que dependem **só do pincel** (`strength` · `accumulate` ·
`front_faces_only`) viraram **`Brush::arm_verb_defaults`** e os dois chamadores a
partilham. ⚠️ **O que sobra está NOMEADO, não esquecido:** o *falloff* (função de
verbo **e** modo, e o modo que entra tem de ser resolvido antes), a *referência*
(`mode_by_verb`) e o *raio* (pixels de tela) precisam do `Sculpt3dUi` ⇒ trocar de
verbo **pelo teclado** ainda não os arma. Divergência real, escrita nos dois
lados.

---

## §5 — A cena `=33` passa a exercitar o report

A §5 do handoff anterior: *"acrescente os dois passos que faltam — o Enio está a
produzir a evidência à mão porque a cena não a produz"*. Os oito passos viraram
**onze**:

- **(9) A DUREZA** — a demão vira uma **MESA com PAREDE**, não um domo mais alto,
  com o número ao lado; ⚠️ e a cena **avisa** que as listras na parede são a
  GRADE, *"o Blender escadeia igual na topologia dele, e a missão é ser idêntico a
  ele: não reporte a escada como defeito"*.
- **(10) O AUTO SMOOTH** — a demão **SOBREVIVE** (0,0735 → 0,0698) e o **CONTROLE
  ao lado** é o Draw a ser aniquilado (0,0874 → 0,0002), *"e isso está CERTO — ele
  é aditivo puro e não tem meta para onde voltar"*.
- **(11) FRONT FACES ONLY** — o interruptor novo, ⚠️ com o aviso do regime: *"é um
  dab, e o número de dabs importa; esfregue trinta vezes e as duas CONVERGEM"*.

---

## §6 — ⚠️ A sonda que estava MENTINDO, e ela era minha

A `measure_layer_front_face` ablacionava por **MODO** (`RefMode::S` × `B`). O
`kernel_for` é `for_verb(verb).kernel()`, o `for_verb` **RECUA para `B`** num
verbo que o modo não declara, e **o `S` não declara a demão**
(`ref_mode.rs:323`) ⇒ os dois braços corriam a MESMA lei e a sonda imprimia
**duas colunas idênticas** sob os rótulos *LIGADO* e *desligado*.

Eu descobri o vácuo durante a ablação, **troquei o método** (mutação do produto) e
**deixei o instrumento como estava**. Ele ficou a responder com confiança à
pergunta errada — a lição que o `PH2D_FLUID_PROFILE` do Painter já pagou três
vezes. Hoje a alavanca é o **flag do produto**, e o rodapé dela deixou de ser uma
hipótese condicional (*"se as duas coincidirem…"*) para ser a medição.

---

## §7 — Superfície de colisão, MEDIDA (não auto-relatada)

| item | estado |
|---|---|
| `PROJECT_SCHEMA` | **INTOCADO** — `git diff` vazio nos **três** sítios (`project.rs` · `project_schema.rs` · `project_schema_tests.rs`) |
| contrato congelado | **INTOCADO** — `git diff` vazio em `ph2d-nodegraph/src/node.rs` e `ph2d-core/src/tool.rs` |
| registro do `ph2d-ecs` + os **três** espelhos | **INTOCADO** |
| `*/Cargo.toml` · `Cargo.lock` | **ZERO** — nenhuma crate nova, nenhuma dep nova |
| ADR | **nenhum** ⇒ a linha fica **FORA de toda disputa de número** |
| `ph2d-i18n` | só o **irmão** `sculpt3d.rs` (+5). ⚠️ O `lib.rs` **não** é tocado ⇒ a cadeia `vector::tr(k).or_else(sculpt3d::tr)` da integração de 10/08 fica intacta |
| ids novos | **1**, e é `hash_node_id("sculpt3d.front_faces")` ⇒ fora de todo gate de contagem |
| scrollbar id | nenhum novo (o do painel segue **840**) |
| cenas de smoke | **nenhuma nova** — a `=33` já existia e ganhou três passos |

⚠️ **A interseção com o `main` é VAZIA e o `main` está a ZERO commits do fork —
mas esta caixa ENVELHECE entre o fechamento e a ordem.** Esta linha já a viu
envelhecer **duas** vezes (o `main` andou 142 commits numa integração e 298
noutra, com a interseção real a diferir da prevista nas duas). **Re-meça no dia.**

---

## §8 — O gate rodado

| gate | resultado |
|---|---|
| `cargo fmt --all` | aplicado, árvore limpa |
| `cargo check --workspace --all-targets` | **EXIT 0** |
| clippy nas 5 crates tocadas, `--all-targets` | **zero warnings** |
| `ph2d-sculpt3d` + `ph2d-panel-sculpt3d` + `ph2d-mesh` | **verdes** (298 no lib do motor, 50 no seam do painel) |
| `ph2d-editor-core` + `ph2d-host-desktop` | **3216 passed, 0 failed** (2ª corrida) |
| `architecture_workspace_file_loc_cap` · `node_id_collisions` | **verdes** |

⚠️ **A 1ª corrida da suíte da shell teve UMA falha, e ela NÃO é desta linha —
exonerada por QUATRO testemunhas:** `only_the_lower_row_breathes_and_it_moves_with_the_playhead`
(Motion Nodes) tem `git diff main...HEAD` **vazio** no arquivo, **passa isolado**,
o produtor dele **escreve uma fixture em DISCO** (`std::fs::write` —
a assinatura de corrida entre testes paralelos), e a **2ª corrida completa deu
3216/3216**.

**8 gates novos · 8 mutações, 8 sangram:**

| # | mutação | sangra |
|---|---|---|
| M1 | o front-face volta a ser incondicional | o perfil **e** o interruptor |
| M2 | `default_front_faces_only` devolve `true` sempre | o gate do default |
| M3 | …devolve `false` sempre | o default **e** a faixa (que o DERIVA) |
| M4 | `offers_front_faces` devolve `true` sempre | o CONTROLE do modo `Ignored` |
| M5 | o re-arm não arma o flag | o gate de troca de verbo |
| M6 | a dureza deixa de SATURAR | o platô chato **e** o perfil |
| M7 | o pintor nunca desenha a caixa | a presença no seam |
| M8 | o roteador alterna o VIZINHO | o flip no seam |

---

## §9 — Smoke

```text
env PH2D_SCULPT3D_SMOKE=33 cargo run -p ph2d-host-desktop --release
```

⚠️ **A cena imprime os onze passos; se a lista não aparecer, PARE.** Os passos
**1-8 são o CONTROLE** — eles foram aprovados antes e têm de continuar iguais. Os
**9-11 são a wave**, e o 11 é o único que precisa da esfera **curva**: deite a
demão sobre o flanco da peça, não no topo achatado, senão os dois estados
desenham a mesma coisa e o smoke não diz nada.

⚠️ **Rode a suíte do módulo também em DEBUG** — precedente registado nesta casa
(o `ph2d-flip-colorize` panicava só ali).

**As sondas, todas `#[ignore]`, imprimem e não afirmam:**

```text
cargo test -p ph2d-sculpt3d --release --test measure_layer_front_face -- --ignored --nocapture --test-threads=1
cargo test -p ph2d-sculpt3d --release --test measure_layer_comb        -- --ignored --nocapture --test-threads=1
cargo test -p ph2d-sculpt3d --release --test probe_layer_product       -- --ignored --nocapture --test-threads=1
```

⚠️ **`measure_brush_kernel` é kill de RELÓGIO** e já reprovou sob `load average
26`, passando isolado — *nenhuma leitura de relógio desta workstation significa
coisa nenhuma acima de `load ~5`*.

---

## §10 — Mudanças de comportamento, nomeadas

1. ⚠️ **O facing deixa de correr por omissão em TODO verbo que cai no modo `B`.**
   É a entrega, e alcança mais que a demão: qualquer verbo cujo modo declare
   `FrontFace::Continuous` passa a precisar da caixa marcada. **É o que a fonte
   faz** (o default do Blender é desligado), e o efeito satura com o esfregar.
2. **A caixa `Front Faces Only` aparece** abaixo do Accumulate, onde a lei existe.
3. **Trocar de verbo pelo teclado** passa a armar o `front_faces_only` (e continua
   a não armar falloff/referência/raio — §4.1).
4. A cena `=33` ganhou três passos.

---

## §11 — Aberto, com o preço ao lado

- ⛔ **NÃO persiga o pente** — está medido: 0,0093 de uma aresta. Ele é a parede
  na grade, e a referência escadeia igual.
- ⛔ **NÃO "melhore" a dureza alta.** A §3 do handoff anterior é explícita: *"a do
  Blender TAMBÉM é feia … o alvo é a violência dele; se o resultado ficar bonito e
  diferente, ele está errado."*
- **O falloff/referência/raio no atalho de teclado** (§4.1) — os três precisam do
  `Sculpt3dUi`, e trazê-los para o motor exigiria dar a esta crate um tipo de
  painel.
- **O `space attenuation`** do §5.1 continua não conferido; ele é **TAXA e não
  forma** (muda em quantos dabs a demão fecha, nunca a espessura final), então ele
  **não** explica nenhuma das fotos.
- **A W1 (os defaults do `B`) e o Draw Sharp** seguem **decisão de PRODUTO do
  Enio**, não dívida: o §7.0 mediu que os defaults por-tool do Blender moram num
  `.blend` **binário**, não no código.
