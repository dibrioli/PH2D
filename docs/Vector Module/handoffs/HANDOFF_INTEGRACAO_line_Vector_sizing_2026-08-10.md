# HANDOFF DE INTEGRAÇÃO — `line/Vector` · o SIZING (2026-08-10)

> Para o agente **integrador**, por ordem do Enio (DIRETRIZ §1.5.9). A linha está **fechada**;
> ela **não** integra e **não** faz ship.

---

## §0 — O que esta linha entrega, em três linhas

O **vocabulário de tamanho do Figma**, que era a única metade em falta do auto layout:
**Fixed · Hug · Fill · Min · Max · Absolute position**. Antes desta wave o tamanho de toda moldura
era digitado à mão, e um filho não tinha como sair do fluxo.

⚠️ Ela nasce de um **estudo medido** (`docs/Vector Module/Estudos/ESTUDO_containers_e_catalogo_minimo_de_UI_2026-08-10.md`),
que o Enio pediu para decidir se faltavam `VBoxContainer`/`HBoxContainer`/`Grid`. O veredito:
**não faltam** (são `LayoutDir::Row`/`Column`/`RowWrap`, já shipados), e o que faltava de verdade
era o sizing.

---

## §1 — Identidade

| | |
|---|---|
| Branch | `line/Vector` |
| HEAD | `bbed0e99d` |
| Base | `76788440a` (o `main` de 2026-08-10) |
| Commits | **9** (1 de troca + 1 de estudo + 4 de produto + 1 de handoff + **2 de smoke**) |
| Diff | 29 arquivos, +2309/−58 |

---

## §2 — ⚠️ Superfície de COLISÃO (o que o integrador tem de conferir)

| item | valor nesta linha | nota |
|---|---|---|
| **`PROJECT_SCHEMA`** | **INTOCADO** (`git diff` vazio em `project.rs`) | componente novo cunha `stable_type_id` próprio — o precedente do `VecFrame`/`VecBindings` |
| **`VEC_SCENE_SCHEMA`** | **INTOCADO** (14) | |
| **Registro do `ph2d-ecs`** | **55 → 57** | ⚠️ **e os DOIS espelhos 56 → 58** (`ph2d-render`, `ph2d-script`) — o contador é **TRÊS casas**, cada uma na suíte da própria crate |
| Componentes novos | `VecLayoutSize` · `VecLayoutAbsolute` | |
| **Contrato congelado** | **intacto** (`git diff` vazio em `ph2d-nodegraph` e `ph2d-core/src/tool.rs`) | |
| **ADR** | **nenhum novo** ⇒ a linha fica **FORA de toda disputa de número** | o ADR-0153 é **emendado**, não substituído (§4) |
| `Cargo.toml` / `Cargo.lock` | **ZERO** | nenhuma crate nova, nenhuma dep nova |
| Ids novos | **9**, todos `hash_node_id` | fora de todo contador |
| Cena de smoke | **`=66`** | era a próxima livre; o gate `no_two_smoke_scenes_claim_the_same_level` está verde |
| `MAX_FX_KINDS` · scrollbar id · `WidgetKind` | **intocados** | |

⚠️ **O ponto de merge sensível é UM:** `crates/ph2d-vec-layout/src/lib.rs` teve o campo
`Node::size` **trocado de tipo** (`[f64; 2]` → `[Len; 2]`) e ganhou `min`/`max`. Uma linha que
construa um `Node` não compila até acrescentar `..Node::default()`. Há **16** construções na
árvore, todas em testes desta crate, na sonda e na fatia da shell — todas já convertidas.

---

## §3 — O que foi construído, e a lei de cada peça

### 3.1 O motor (`ph2d-vec-layout`)

- **`Len::{Fixed(f64), Hug}`** — enum e não `f64` + `bool`, porque as duas respostas são
  **exclusivas**: um eixo que abraça não tem um número que alguém escreveu.
- **`min`/`max` por eixo.**
- **`LayoutError::HugWithoutFlow`** — uma FOLHA que pede para abraçar é **recusada**. ⚠️ Acomodar
  seria pior que falhar: sem filhos o conteúdo mede zero, e a forma **desapareceria sem erro**.
- O **espaço oferecido à raiz** passa a ser a pergunta do abraço (`MaxContent` × `Definite`).

### 3.2 O documento (`ph2d-ecs`)

- **`VecLayoutSize { size, min, max }`** — do **NÓ**, sobre si. ⚠️ O `LayoutSize::Fixed` do
  documento **não carrega número**, ao contrário do motor: o número de uma forma é a **geometria
  dela**, e guardá-lo aqui seria a segunda resposta a *"que tamanho tem esta moldura?"*.
- **`VecLayoutAbsolute`** — marcador; a presença é o booleano (o idioma do `Ccd` da física).

### 3.3 A fatia (`shells/desktop/src/layout_live.rs`)

- **`size_of`** é porta única, irmã do `frame_style`, e **recusa o abraço a quem não flui** (cai
  para o tamanho medido).
- **O fora-do-fluxo sai da FATIA, não do motor.** Um nó que o motor nunca vê fica com a pose
  autorada e continua a andar com o pai e a ser recortado por ele. Dizê-lo ao motor pediria um
  *inset* — quatro números derivados que ninguém autorou.
- **A RAIZ entra no laço de colocação**, porque um `Hug` muda o tamanho dela e o tamanho novo tem
  de ser **desenhado**. Sem abraço o afim sai identidade e o `is_identity` salta-a antes de
  qualquer cópia ⇒ **byte-intocado** para quem nunca pediu abraço.
- ✅ **O `clip` veio de graça:** o `frame_clip` já lê a `LiveGeometry`, então a moldura que encolhe
  leva o recorte junto.

### 3.4 A UI (`ph2d-panel-vector`)

**Width/Height: Fixed | Hug** (um par por eixo) · **Min/Max** (zero = ausência) · **Absolute
position**, que **esconde Grow/Shrink** — quem saiu do fluxo não reparte sobra nenhuma.

---

## §4 — ⚠️ O que o integrador deve levar aos DOCS (não é código)

O **ADR-0153 tem um argumento que a medição de hoje derruba**. Ele recusa a feature `grid` do
`taffy` dizendo que ela *"triplica o custo de build (0,20 → 0,63 s)"*; re-medido **costas-com-costas
com `cargo clean` entre corridas, máquina calma (`load 0,44`)**: **173-180 ms → 191-192 ms**, ou
**+11 ms (1,07×)**.

⇒ A recusa **continua defensável pelo OUTRO argumento** (ausência de consumidor), mas quem a ler
hoje decide com um número que não é mais verdade. *Quem move o número que tornava algo inalcançável
tem de reconferir a nota* (§0 do `CLAUDE.md`). O estudo já traz a emenda escrita; **o ADR ainda
não foi editado** — deixado ao integrador de propósito, porque um ADR é documento compartilhado e
a edição pode colidir.

---

## §4b — ⚠️ As DUAS rodadas de smoke do Enio, e o que elas acharam

As duas foram sobre o **mesmo controle** (*Absolute position*), por causas **diferentes** — a
segunda só ficou visível depois de a primeira ser curada.

**(1) *"Não achei Absolute Position no painel do quadrado âmbar"***. O produto estava certo: o
toggle é escondido quando o pai não empilha, e **escondê-lo em silêncio** deixava o artista a
olhar para um painel que não dizia o que faltava. `LayoutItem` ganhou `in_flow`, e o painel passou
a **escrever** *"Set the parent frame to Row or Column first"* (o precedente do Falloff dos Motion
Nodes: *inerte é dito, não omitido*).

⚠️ **A primeira tentativa de cura REMOVIA um caso que funcionava** — trocar o predicado do sujeito
de `VecLayout` para `VecFrame` parece a leitura óbvia e está errada: o passe de layout recolhe os
filhos de **qualquer** entidade com `VecLayout`, tenha ela moldura ou não. Dois gates sangraram na
hora; o predicado é a **união**.

**(2) *"O checkbox não aceita ser checado, pode não estar linkado na UI"***. O id **estava**
linkado em todas as seis pontas da costura (id · `populate` · `paint`+hit · `LAYOUT_CHIPS` ·
`forwards_plain_click` · o roteador da shell), e o gate de seam que o prova estava **verde**. O que
o matava era uma **ORDEM**: o `apply_layout_edit` abria com *"resolve a moldura, ou desiste"*, e
`frame_of_selection` **recusa um filho sozinho** por desenho (o doc-comment do `vec_frame_edit`
di-lo). Todo edit daquela porta é da MOLDURA — **menos este, que é do FILHO** — então o único edit
que é pedido com o filho selecionado era exatamente o único que o guard matava, e o braço que o
honrava vinha depois dele.

⚠️ **A função IRMÃ já fazia certo:** o `apply_layout_field` **não tem guard no topo** — cada braço
resolve o próprio sujeito (`Grow`/`Shrink` pelo filho, `Min`/`Max`/`Gap`/`Pad` pela moldura). Era o
`apply_layout_edit` que estava fora de linha, e é por isso que a cura é mover **um** caso, não
afrouxar o guard: abri-lo para todos faria um `Dir` pedido sobre um filho solto ligar fluxo na
entidade errada (gate irmão `a_frame_edit_asked_with_only_the_child_selected_does_nothing`).

---

## §5 — Gates e mutações

| onde | gates | mutações |
|---|---|---|
| motor | **19** (7 novos) | **5 / 5 sangram** |
| fatia | **57** (6 novos) | **6 / 6 sangram** |
| seam do painel | **16** (4 novos) | ponteiro REAL (Down+Up), nunca `Click` sintético |

⚠️ **Uma mutação sobreviveu e a cura foi um gate, não a barra:** trocar `MaxContent` por
`Definite(0.0)` no espaço da raiz passava por tudo. O gate que faltava
(`a_hugging_frame_that_wraps_does_not_wrap`) testa uma afirmação **minha** de doc-comment que
estava escrita sem nada a segurá-la.

⚠️ **E o gate de seam do toggle era VERDE sobre um controle MORTO** (§4b.2), o que nomeia o que
ele pode e o que ele **não** pode provar: *o clique chega ao barramento* é uma afirmação sobre o
PAINEL, e ela continua verdadeira quando quem morre é a porta do outro lado. O par que faltava
mede a outra ponta — **o componente aparece no filho** — e é ele que sangra com a ordem revertida.

---

## §6 — O que SÓ o `ship.sh` pega

- `clippy --all-targets` das crates **não impactadas** pelo meu filtro.
- `machete` / `deny` — **nada a temer**: zero `Cargo.toml` tocado.
- ⚠️ **Rode a suíte também em DEBUG.** Precedente registrado nesta linha (o pânico do
  `ph2d-flip-colorize` só aparecia ali).

---

## §7 — O smoke

**`env PH2D_BUILD_SMOKE=66 cargo run -p ph2d-host-desktop --release`**

⚠️ **Se a linha `[sizing] cena montada: …` não aparecer, PARE** — o resto do smoke não diz nada.

A cena dá o **material** e **não arma nada** (a cicatriz do `impasto_smoke`): as quatro molduras
nascem sem `VecLayout` e sem `VecLayoutSize`. O roteiro de 6 passos é impresso pela própria cena,
com os números **medidos** dela. A quarta moldura é o **CONTROLE** e não pode mudar em passo nenhum.

**Aprovar exige ver:** (1) as duas molduras abraçadas ficarem com **larguras diferentes uma da
outra**; (2) o fundo escuro **encolher junto** (é o que prova que o tamanho novo é desenhado, não
só calculado); (3) o selo âmbar **ficar onde está** enquanto os três azuis se arrumam.

⚠️ **O passo 5 tem uma PRÉ-CONDIÇÃO, e o roteiro impresso pela cena passou a dizê-la:** o
*Absolute position* só é oferecido quando o pai **empilha**, então a moldura SELO tem de receber
`Direction → Row` no passo 1 como as outras duas. Sem isso o painel escreve *"Set the parent frame
to Row or Column first"* — que é a resposta certa, e é o que a primeira rodada de smoke não tinha.

---

## §8 — Aberto, com o preço ao lado

- **Escalar deforma o raio.** O `fit` aplica um afim, então uma moldura arredondada que abraça sai
  com cantos ovais. ⚠️ **Não é regressão desta wave** — é o que o `grow` já fazia desde a W2; o que
  esta wave faz é tornar o caso mais comum. A cura é re-cozinhar a `VecShape::Param` no tamanho
  novo em vez de a escalar, e é **wave própria**.
- **Scroll numa moldura** (o item 3 do estudo). `Hug + Max` já cobrem *"cresce até um teto"*; o que
  falta é o que passa DO teto.
- **`Fill` continua exposto como número** (`grow`), e não com o nome do Figma. Renomeá-lo é
  decisão de produto: o número diz mais (repartição em razão), o nome diz melhor.
