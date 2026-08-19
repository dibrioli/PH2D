# HANDOFF DE INTEGRAÇÃO (MESTRE) — `line/sculpt3d`

**Status:** FECHADO 2026-08-17 · no `main` em `77c7db4f9` (o commit que trouxe este arquivo).

**Data:** 2026-08-17 · **branch:** `line/sculpt3d` · **tip:** `75df717c4`
**Base:** `c1bf582dc` · **39 commits** · **115 arquivos** · **+17.890/−1.542**

**Supersede**, *apenas como o que integrar agora*, os dois handoffs anteriores da
linha:

- [`HANDOFF_INTEGRACAO_line_sculpt3d_LAYER_2026-08-16.md`](HANDOFF_INTEGRACAO_line_sculpt3d_LAYER_2026-08-16.md)
- [`HANDOFF_line_sculpt3d_LAYER_2026-08-16.md`](HANDOFF_line_sculpt3d_LAYER_2026-08-16.md)

⚠️ **Os dois continuam a ser a FONTE do mecanismo e não foram copiados para cá** —
o primeiro traz o porte do `layer.cc` (as onze leis conferidas uma a uma, a
divergência do front-face e as duas refutações), o segundo o diagnóstico que o
produziu. Este documento acrescenta a **última wave** (os cinco pedidos do Enio) e
**re-mede** a superfície de colisão contra o `main` de hoje, porque a caixa do
handoff anterior avisava que ela envelhece — e esta linha já a viu envelhecer duas
vezes.

---

## §0 — O que integra, numa frase

O módulo de escultura ganhou **quatro blocos**: o alisamento que não encolhe (W4),
o plano que virou superfície (W7), a **DEMÃO portada do `layer.cc`** (W8, com a
auditoria inteira que ela puxou) e a wave dos **cinco pedidos do Enio** — na qual
duas coisas foram construídas, **duas foram medidas e refutadas**, e **uma cura
está desenhada e NÃO construída de propósito**, porque construí-la contraria a
ordem permanente *"idêntico ao Blender"*.

---

## §1 — A jornada em quatro blocos

| bloco | commits | o que entrega |
|---|---|---|
| **W4** — o alisamento | `3e51db828`..`2bdf23ff4` (5) | **Slide Relax** (redistribui a malha sem mudar a forma, **71× menos** deformação) · **Surface Smooth (HC)**, o alisamento que devolve o que tirou · o **laplaciano por COTANGENTES** · e o `lambda` do Taubin, que era um palpite (o `l-mode` alisa **2,2×** mais) |
| **W7** — a superfície | `23913321b` (1) | o plano vira uma **SUPERFÍCIE**, e o `offset` era um **knob morto** |
| **W8** — a DEMÃO | `dd2b24fd4`..`b7b277fd9` (30) | o `Verb::Layer` portado do `layer.cc`, **mais a auditoria que ele puxou** (§1.1) |
| **os CINCO pedidos** | `1e03095b1`..`75df717c4` (3) | §2 |

### §1.1 — ⚠️ A W8 puxou uma auditoria maior que ela, e é a parte que um integrador estranha

Portar a demão obrigou a ler o catálogo inteiro, e **oito defeitos vivos caíram no
caminho** — todos anteriores a esta wave, cada um medido:

- **SETE dos 23 verbos nasciam com a força da referência ERRADA** (`914bf9c1a`) — e
  o preço não era o chip apagado, era a **forma do barro**: `profile(S, Layer)` é
  `None` ⇒ o slider virava o peso **cru** onde o `layer.cc` o **eleva ao quadrado**
  (`0,5000` contra `0,2500`, o dobro da taxa).
- **O catálogo de falloff dizia o nome do Blender sobre outra curva** (`313bc4a25`),
  e o `Sharper` virou um **CONE** (`f7139b090`) — as duas curvas de domo voltaram
  ao catálogo **com nome próprio** (`275368aaf`).
- **O `auto_smooth` era a força de um lerp e é um ORÇAMENTO de passadas**
  (`958435dd3`), com a atribuição a **ABSOLVER** hardness e falloff (`e0915819d`).
- **O Möller-Trumbore do PICK não era estanque** (`6e4907b72`, na `ph2d-mesh`), e a
  justificativa escrita era falsa.
- **O shell tomava METADE do par do SculptGL** (`7f4c2b6f5`): a âncora apagava os
  dabs que o `break` pulou — e a lei da âncora **REFUTOU a minha hipótese**
  (`3c782d5ce`), porque ela **AVANÇA num miss**, que é o que as duas referências
  dizem e o oposto do que eu tinha escrito (`0c305c519`).
- **TRÊS verbos shiparam sem ninguém decidir como o artista os pega** (`d7ed141e7`)
  — e o gate que devia pegar isso **só nomeava um por corrida**.
- **O arch-gate do transform PANICAVA sobre produto correto** (`c7c61f70f`).

⚠️ **Nada disso é escopo inventado:** cada um foi encontrado *dentro* do caminho de
tornar a demão idêntica à fonte, e deixá-los de pé faria a demão ser conferida
contra um catálogo que mente.

---

## §2 — Os cinco pedidos, item a item

> *"Temos bom resultado para Layer com Strength 0.7, Hardness 0.4 e Auto Smooth
> 0.0. Esses devem ser o valor padrão para o tool layer. Uma observação: quanto
> mais se aproxima do objeto (zoom), pior o resultado. Tente resolver isso. Veja
> se nosso algoritmo da incidência da ferramenta é tão bom quanto SculptGL e
> Blender, que usam as normais e mesmo nas laterais de um objeto esférico
> conseguem bom resultado. O gizmo da tool deve ter a direção das normais onde
> incide (a nossa o gizmo da tool permanece na direção da tela). As configurações
> dos parâmetros de cada tool não devem se propagar para outra tool."*

| # | pedido | veredito | commit |
|---|---|---|---|
| 1 | defaults do Layer | ✅ **construído** | `1e03095b1` |
| 2 | o zoom piora | ⚠️ **REAL, e HERDADO** — medido, cercado, **cura NÃO construída** | `75df717c4` |
| 3 | a incidência contra a referência | ⛔ **nossa lei já É a do Blender** — refutado por medição | (medição; sem código) |
| 4 | o gizmo segue a normal | ✅ **construído** | `db1f494fa` |
| 5 | params não propagam entre tools | ✅ **construído** | `1e03095b1` |

### §2.1 — (1) e (5): cada ferramenta lembra a própria afinação

⚠️ **A lei antiga só podia acertar enquanto o artista não mexesse em nada.** O
`arm_verb_defaults` levava o pincel **VIVO** para o verbo novo e re-armava campo a
campo sob *"arma se o artista ainda não tocou"* — uma heurística sobre intenção.
Agora a tabela guarda o **`Brush` INTEIRO por verbo** (`VerbSlot`, mais o
`radius_px`, que é a régua da TELA), e trocar de ferramenta é **salvar o slot que
sai e carregar o que entra**. Nada mais. *O slot sabe.*

O **Layer** nasce com `0.7 / 0.4 / 0.0`. ⚠️ **A tabela de referência é SILENCIOSA
para ele** (`profile(RefMode::S)` devolve `None` — o SculptGL não tem essa
ferramenta), então a fábrica dele é **nossa**, e é por isso que os números do smoke
têm onde morar sem divergir de fonte nenhuma. O gate os nomeia listando **o que
NÃO é o genérico**.

⚠️ **E um arch-gate apanhou um defeito que ESTA wave introduziu.** Enquanto a
tabela guardava só o modo, nada por-verbo podia envelhecer; com o pincel inteiro
no slot, carimbar uma referência deixava o `falloff` daquele verbo resolvido
contra a **ANTERIOR** — o chip diria *Blender* e o kernel rodaria a quártica do
SculptGL. Uma lei (`reconcile_mode`), **dois chamadores** (o chip, sobre o pincel
VIVO; o *apply to all*, sobre o pincel de **cada slot**).

**15 gates · 7 mutações, 7 sangram.**

### §2.2 — (4) o cursor deita na superfície

⚠️ **A cerca anterior media a coisa errada, e a distinção é GEOMÉTRICA.** O
módulo-doc do cursor afirmava que *"um círculo de raio `radius_px` no plano da tela
**é** a figura dele"*. A pegada é uma **BOLA de mundo** cujo raio deriva dos pixels
**na profundidade do acerto**; o círculo de tela é a **SILHUETA** dessa bola — e
quem recebe tinta é a **interseção dela com a superfície**, que num plano inclinado
de `θ` projeta uma elipse de eixo menor `r·cos θ`, **inscrita** na silhueta.

Medido:

| inclinação | eixo menor, círculo antigo | eixo menor, anel conformado | erro do antigo |
|---|---|---|---|
| **0°** | 80,00 px | **80,00 px** | — (coincidem ao centésimo) |
| 60° | 80,00 px | **37,98 px** | **2,1×** |
| 85° | 80,00 px | **6,57 px** | **12×** |

⚠️ **É a linha do 0° que torna a troca segura:** no caso comum as duas figuras são
a mesma, então o cursor **não pisca** quando a superfície encara a câmera.

⚠️ **A normal vem das normais SUAVES** (`Mesh::normals()`), que são o `base_nrm`
que o dab consome ⇒ *o cursor concorda com o kernel por construção*. E **NÃO do
`Hit::normal`**, que carrega uma lacuna nomeada (quad "gravata" → `[0,0,0]`) cujo
gatilho declarado é *"o primeiro leitor de produto"* — este seria ele, e adotá-lo
importaria a lacuna para dentro da feature.

⚠️ **O círculo de tela FICA como recuo honesto** para *"não sei a orientação"*:
normal degenerada, ou uma amostra do anel atrás do olho. **O segundo caso é
ALCANÇÁVEL** e a fixture o mede — ela precisou de superfície **de perfil (89°)** e
raio de mundo ~2× a distância do olho; ⚠️ a primeira versão da sonda dizia
*"inalcançável"* porque mirava em `tilt = 0`, onde o anel fica a **profundidade
constante** e nem 100.000 px projetam para fora.

**5 gates · 4 mutações, 4 sangram.**

---

## §3 — ⛔ O que foi MEDIDO e NÃO construído (para ninguém reconstruir)

### §3.1 — (2) o espigão do zoom é a lei da REFERÊNCIA

O report é **real**: pela porta do produto, a esbeltez `altura / raio` vai de
**0,135** com a câmera longe a **5,857** a 4× de zoom. O raio nasce dos **pixels** e
encolhe; a altura é de **MUNDO** e não encolhe ⇒ a demão vira espigão.

⚠️ **E ele é HERDADO, em três fatos do fonte:**

1. `rna_brush.cc:3230` declara `height` como **`PROP_DISTANCE`** — uma distância de
   mundo, não uma fração.
2. `layer.cc:101` a multiplica **crua**.
3. O `cache.radius` de lá **também** sai dos pixels.

⚠️ **O default do Blender é `0.5` contra o nosso `0.1`** — no mesmo zoom, o Layer
dele espiga **cinco vezes mais** que o nosso.

⛔ **A cura que achataria a curva é a lei do SculptGL** (`Brush.js:62`, o
deslocamento escalando com o raio) e **DIVERGE da referência**, contra a ordem
permanente *"idêntico ao Blender"*. **Não foi construída**: é decisão do Enio, com
o número na mão. A cerca é executável (`the_coat_height_is_a_world_distance_and_does_not_follow_the_radius`),
com mutação que sangra — para ninguém a "consertar" em silêncio.

⚠️ **E a SEGUNDA causa não é de lei nenhuma:** a 4× de zoom o pincel cobre **1,45
arestas medianas**. Nenhuma lei de deslocamento conserta um pincel mais estreito
que a malha — quem conserta é **subdividir**, e o gesto já shipa. *Confundir as
duas faria trocar a lei e continuar sem resolução.*

### §3.2 — (3) a incidência: a nossa lei JÁ É a do Blender

O pedido supunha um vão. **Medido, não há:**

- A normal que o dab consome é `base_nrm[s]` — a normal **congelada por vértice** no
  pen-down (`stroke_target.rs:130` e `:498`), que é **exatamente** o
  `orig_normals[i]` do `layer.cc:101`.
- A pegada é uma **esfera de mundo** (`PAINT_FALLOFF_SHAPE_SPHERE`), não um cilindro
  de tela — que é o que faz o flanco de uma esfera receber tinta correta.
- O vazamento para o lado de trás começa em **85-90°**, e ⚠️ **o Blender tem o
  mesmo**: lá o front-face é **opt-in** (`BRUSH_FRONTFACE`, e nenhuma linha do
  Blender inteiro o liga por omissão), o que a wave anterior já tinha portado como
  flag por-pincel.

⇒ **Nada a construir.** Os cinco gates de front-face (`verb_layer_front_face_tests.rs`)
passam, e o que sobra do report é o **item 2**, que é outro eixo.

---

## §4 — Superfície de colisão, MEDIDA hoje (não auto-relatada)

| item | estado |
|---|---|
| **`PROJECT_SCHEMA`** | **84 INTOCADO** — `git diff main...HEAD` **vazio** nos **quatro** sítios: `project.rs` · `project_schema.rs` · `project_load.rs` · `project_schema_tests.rs` |
| contrato congelado | **INTOCADO** — `git diff` vazio em `crates/ph2d-nodegraph/` e `crates/ph2d-core/src/tool.rs` |
| registro do `ph2d-ecs` | **INTOCADO** ⇒ os **três** espelhos (`ph2d-render` · `ph2d-script`) também |
| **`*/Cargo.toml` · `Cargo.lock`** | **ZERO** — nenhuma crate nova, **nenhuma dep externa nova**, nenhuma aresta interna |
| **ADR** | **nenhum** ⇒ a linha fica **FORA de toda disputa de número** (o próximo livre no `main` é **0160**) |
| `ph2d-i18n` | só o **irmão** `sculpt3d.rs` (**+5**). ⚠️ O `lib.rs` **não** é tocado ⇒ a cadeia `vector::tr(k).or_else(sculpt3d::tr)` que a integração de 10/08 instalou fica **intacta** |
| ids novos | **9**, e **todos `hash_node_id`** ⇒ fora de todo gate de contagem numérica |
| `SCULPT3D_VERB` | **20 → 23** — ⚠️ o tamanho é o do `Verb::ALL` (**23**, conferido) e há gate que os compara |
| scrollbar id | **nenhum novo** (o do painel segue **840**) |
| cenas de smoke | **2 novas** (`=32` superfície · `=33` demão). Censo: **1..33 sem duplicata** ⇒ **próxima livre: 34** |
| `rayon` | **nenhum uso novo** ⇒ nenhum ADR de exceção |

⚠️ **As cinco crates de GPU têm diff VAZIO** (`ph2d-mesh-render` · `ph2d-render` ·
`ph2d-gpu-cook` · `ph2d-paint-gpu` · `ph2d-flip-render`) ⇒ **esta linha não alcança
os gates de adapter**; eles foram verificados no `main` e não precisam de nova
corrida por causa dela.

⚠️ **A `ph2d-mesh` é a única crate tocada FORA do módulo, e a mudança é ADITIVA:**
o módulo novo `cotangent` (`RingWeights` · `ring_weights_at` ·
`mean_curvature_normal_at` · `cotangent_ring_average_at` · `curvature_normal_dir_at`
· `curvature_normals_of`), o `Mesh::tri_at` e três fixtures de malha
(`uv_sphere_shuffled` · `uv_sphere_noisy` · `open_disc`). **Nenhuma assinatura
existente muda** — o `triangles` aparece no diff apenas porque mudou de posição no
arquivo. Os cinco consumidores dela (`ph2d-mesh-render` · `ph2d-panel-sculpt3d` ·
`ph2d-sculpt3d` · `ph2d-sdf` · `shells/desktop`) compilam sem edição.

⚠️ **O `main` está a ZERO commits do fork neste momento** (`git rev-list --count
$(merge-base)..main` = 0) ⇒ o merge é um fast-forward trivial **hoje**. **Esta
caixa ENVELHECE entre o fechamento e a ordem** — esta linha já a viu envelhecer
duas vezes (o `main` andou **142** commits numa integração e **298** noutra, com a
interseção real a diferir da prevista nas duas). **Re-meça no dia.**

### §4.1 — ⚠️ O ponto de merge sensível é um CORTE, e ele é da própria linha

O `crates/ph2d-panel-sculpt3d/src/state.rs` **cruzou o teto de 600 LOC** e foi
partido: a memória por-verbo (o `VerbSlot` e as suas leis) mudou-se para o irmão
**novo** `slots.rs`.

⚠️ **O CAMINHO não mudou, de propósito.** O `state.rs` **re-exporta** a superfície
movida:

```rust
pub use crate::slots::{
    VerbSlot, arm_mode_defaults, reconcile_mode, switch_verb, switch_verb_parts, verb_index,
};
```

A razão é que **um arch-gate da shell lê o FONTE do nascimento** procurando
`VerbSlot::for_verb`, e o shell endereça `state::switch_verb` em vários sítios; o
re-export mantém os dois honestos sem churn. **Uma linha que edite `state.rs`
procurando o `VerbSlot` funde limpo contra um arquivo de onde ele saiu** — o modo
de falha que o corte do `project.rs` já produziu duas vezes neste repo.

⚠️ **E o gate de LOC que passou não provava nada:** o
`architecture_workspace_file_loc_cap` (700) **exclui `ph2d-panel-*`**; quem é dono
daquele arquivo é o `architecture_panel_loc_cap` (600), e era **ele** que estava
vermelho. *Um teto verde do gate errado é a família do vermelho-latente.*

---

## §5 — O gate rodado

Rodado **hoje**, no tip `75df717c4`:

| gate | resultado |
|---|---|
| `cargo fmt --all -- --check` | **EXIT 0** |
| `cargo check --workspace --all-targets` | **EXIT 0** |
| clippy nas 5 crates tocadas, `--all-targets` | **EXIT 0, zero warnings** |
| `ph2d-sculpt3d` | **299** no lib + as suítes de integração, **0 failed** |
| `ph2d-panel-sculpt3d` | **50** no seam + `verb_slots` (8) + `state_tests` (10), **0 failed** |
| `ph2d-host-desktop` | **2799 passed, 0 failed, 183 ignored** |
| `architecture_panel_loc_cap` (600) | **3 passed** |
| `architecture_workspace_file_loc_cap` (700) | **2 passed** |
| `node_id_collisions` | **7 passed** |

**Total da última wave: 21 gates · 12 mutações, 12 sangram.**

| # | mutação | sangra |
|---|---|---|
| M1 | o slot não guarda o pincel (volta ao re-arm por campo) | os gates de propagação |
| M2..M6 | os cinco campos, um a um | o gate do campo correspondente |
| **M7** | `reconcile_mode` não re-resolve o `falloff` | **só** o gate novo do carimbo sobre slot não-vivo |
| M8 | o cursor ignora a normal | 2 gates |
| M9 | `unit` aceita o vetor zero | o recuo degenerado |
| M10 | uma amostra que falha é **pulada** (anel parcial) | *o anel é inteiro ou ausente* |
| M11 | a fiação devolve `None` | o arch-gate |
| **M12** | a lei do zoom adota o `Brush.js` (escala com o raio) | a cerca da §3.1 |

⚠️ **A M10 existe porque a alternativa é pior que o defeito:** um anel a que falta
um pedaço **parece** um cursor a funcionar, e o artista não tem como saber que a
parte que falta é a que estava atrás do olho.

⚠️ **`measure_brush_kernel` é kill de RELÓGIO** e já reprovou nesta linha sob `load
average 26`, passando isolado — *nenhuma leitura de relógio desta workstation
significa coisa nenhuma acima de `load ~5`*.

---

## §6 — Mudanças de comportamento, nomeadas

1. ⚠️ **Cada ferramenta lembra a própria afinação.** Trocar de verbo **não** carrega
   mais força/curva/dureza/raio da anterior. É a entrega do item 5, e alcança os
   **23** verbos — quem estava habituado a afinar o Draw e encontrar o mesmo número
   no Clay vai encontrar o do Clay.
2. **O Layer nasce em `0.7 / 0.4 / 0.0`** (era o genérico `0.5 / 0.0 / 0.0`).
3. ⚠️ **O cursor deixa de ser um círculo de tela.** A 0° é indistinguível do
   anterior; em superfície inclinada ele **encolhe para o que a tinta de facto
   cobre** — a diferença é grande no flanco (12× a 85°), e é a entrega do item 4.
4. **O facing deixa de correr por omissão** em todo verbo cujo modo declare
   `FrontFace::Continuous` (herdado da wave anterior, §2 do handoff de 16/08) —
   **é o que a fonte faz**, e o efeito satura com o esfregar.
5. **Trocar de verbo pelo teclado** passa a armar o `front_faces_only`.
6. As cenas **`=32`** e **`=33`** são novas; a `=33` ganhou onze passos.

---

## §7 — Smoke

```text
env PH2D_SCULPT3D_SMOKE=33 cargo run -p ph2d-host-desktop --release   # a DEMÃO
env PH2D_SCULPT3D_SMOKE=32 cargo run -p ph2d-host-desktop --release   # a SUPERFÍCIE (W7)
```

⚠️ **A `=33` imprime os onze passos; se a lista não aparecer, PARE.** Os passos
**1-8 são o CONTROLE** (foram aprovados antes e têm de continuar iguais); os
**9-11** são a wave da demão, e o **11** é o único que precisa da esfera **curva** —
deite a demão sobre o **flanco**, não no topo achatado, senão os dois estados
desenham a mesma coisa.

**As duas perguntas desta última wave são de OLHO e não têm cena própria** (elas
alcançam toda ferramenta, não uma):

- **O cursor:** aponte para o **flanco** de uma esfera. O anel tem de **deitar na
  superfície** e encolher; encarando a câmera ele tem de ficar **igual ao de
  sempre** (se ele piscar ao atravessar a curvatura, a fiação regrediu).
- **A propagação:** afine o **Draw** (força e curva), pegue o **Clay**, afine
  diferente, **volte ao Draw**. Ele tem de estar como você o deixou.

**As sondas, todas `#[ignore]`, imprimem e não afirmam:**

```text
cargo test -p ph2d-sculpt3d --release --test measure_layer_zoom_and_flank -- --ignored --nocapture --test-threads=1
cargo test -p ph2d-sculpt3d --release --test measure_layer_front_face     -- --ignored --nocapture --test-threads=1
cargo test -p ph2d-sculpt3d --release --test probe_layer_product          -- --ignored --nocapture --test-threads=1
```

⚠️ **Rode a suíte do módulo também em DEBUG** — precedente registado nesta casa (o
`ph2d-flip-colorize` panicava só ali).

---

## §8 — Aberto, com o preço ao lado

**Decisões do Enio (entregues com número, deliberadamente não construídas):**

- ⛔ **A lei do zoom** (§3.1). Adotar o `Brush.js:62` achata a curva e **diverge da
  referência**. O número está na cerca.
- ⛔ **A W1 (os defaults do `B`) e o Draw Sharp** seguem decisão de **PRODUTO**, não
  dívida: os defaults por-tool do Blender moram num `.blend` **binário**, não no
  código.
- ⛔ **NÃO persiga o pente** da demão — medido, **0,0093 de UMA aresta**: é a parede
  a escadear pela grade, e a referência escadeia igual.
- ⛔ **NÃO "melhore" a dureza alta.** *A do Blender também é feia; o alvo é a
  violência dele. Se o resultado ficar bonito e diferente, ele está errado.*

**Trabalho nomeado:**

- O **falloff, a referência e o raio no atalho de teclado** — os três precisam do
  `Sculpt3dUi`, e trazê-los para o motor exigiria dar a esta crate um tipo de
  painel. Divergência real, escrita nos dois lados.
- O **`space attenuation`** continua não conferido; ele é **TAXA e não forma** (muda
  em quantos dabs a demão fecha, nunca a espessura final).
- A lacuna do **`Hit::normal`** (quad "gravata" → `[0,0,0]`) segue nomeada e sem
  leitor de produto — o cursor **não** a adotou, de propósito.
- ⚠️ **O roteador de cenas do sculpt3d NÃO é um `match`** — cada arquivo testa a env
  var por conta própria (`== Some("33")`), então um número repetido **não** é
  `unreachable pattern` do compilador: ele é uma cena inalcançável **em silêncio**.
  Hoje o censo dá **1..33 sem duplicata**; ⚠️ **conte a próxima lendo os arquivos,
  nunca esta nota**.
- **W9 (Mesh Filter)** segue o item mais barato da lista, pelo precedente do *Filter
  Layer* do Painter: **não há kernel novo**.

---

## §9 — Protocolo

⛔ **A linha NÃO integra e NÃO faz push.** Este handoff fecha a wave; a integração é
por **ordem explícita do Enio**, via agente integrador dedicado (CLAUDE.md §0.7).
