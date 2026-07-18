# HANDOFF de INTEGRAÇÃO — `line/Vector`, sessão de 2026-07-18 (a PILHA de efeitos)

**Para:** o agente integrador (DIRETRIZ §1.5.3–1.5.4), quando o Enio mandar.
**Estado:** ✅ linha **fechada e verde**, 4 commits sobre a `main`. **NÃO integrei e NÃO pushei** —
a linha fecha, entrega o handoff e para (CLAUDE.md §0.7).

> ⚠️ **Pendente de SMOKE do Enio** — `PH2D_BUILD_SMOKE=13`. Ver §5.

---

## §1 — O que entra

| SHA | O quê |
|---|---|
| `19383f48` | **ADR-0132** — a decisão de arquitetura do LPE |
| `e5e40aa6` | **fix**: a alça de raio pergunta se a geometria é DERIVADA (bug vivo, 3 objetos) |
| `db50c236` | **feat**: a PILHA + o motor de arco + o **Trim Path** |
| `6f599cf1` | **feat**: a cena de smoke `PH2D_BUILD_SMOKE=13` |

**Base:** `02382568` (a linha estava a 2 commits de docs sobre a `main` `389676f9`).

---

## §2 — A decisão, em três frases

A fila pedia *"Live Path Effects como NÓS"*. **O contrato congelado foi medido e não bloqueia**
(`CookValue::Opaque` + `Domain::Vector` + `input_any`/`emit_any` já carregam geometria em aresta —
padrão Houdini/USD; param não-`f32` tem o canal de TEXT PARAM e a convenção de discriminante `f32`),
então **não houve PARE nem ADR de contrato** — e é justamente por a escolha ser livre que ela teve de
se defender pelo desenho.

**O desenho é PILHA, não grafo**: LPE é uma *lista no objeto selecionado* em toda ferramenta shipada;
não existe grafo POR objeto (o Motion Nodes tem UM para a cena inteira); o `Cow::Borrowed` que
sustenta o `cooked()` é trivial numa lista e deixaria de ser um `if` sob chave de memo; e é o desenho
que já funcionou 4× nesta linha. **O caminho do nó fica aberto de graça** — cada efeito é função pura
numa crate/módulo, então um nó o *embrulha* em vez de o reimplementar (ADR-0132 §4).

---

## §3 — Onde a pilha mora, e por que não podia morar noutro sítio

`VecPath.effects` é dado de documento; o `cooked()` roda a pilha **logo depois do estágio da quina**.
Nenhum consumidor mudou — o funil já era o `cooked()`.

⚠️ **Duas restrições de camada decidiram isto, e quem for mexer precisa de as saber:**

1. **O `cooked()` é chamado de DENTRO do próprio `ph2d-vec-scene`** (`inside.rs`, `boundary.rs`,
   `path_ops.rs`, `space.rs`). Avaliar a pilha noutro crate deixaria o **hit-test e a bbox** vendo
   geometria sem efeito — *"o que se vê"* divergindo de *"o que se aponta"*.
2. **`ph2d-vec-scene` é sem-kurbo por decisão declarada no `Cargo.toml`.** Por isso o motor de arco
   (`arclen.rs`) nasceu aqui em vez de vir do `kurbo::inv_arclen`: arrastar a stack Linebender para
   dentro do modelo de documento por 40 linhas de quadratura seria pagar caro por uma cerca decidida.

**A quina é o estágio ZERO e não entra na pilha** (ADR-0132 §3): o raio mora no vértice **autorado** e
arredondar **divide** um vértice; todo efeito a jusante **resampleia**, então a contagem de vértices é
*saída* dele. Os 4 sítios que escrevem `corner_radius: 0.0` em `ph2d-vec-envelope`/`ph2d-vec-blend`
**estão certos** — não os "conserte".

---

## §4 — Os invariantes que não podem morrer (e os gates que os seguram)

| Invariante | Por quê | Gate |
|---|---|---|
| Pilha vazia = **mesmo ponteiro** | foi o que permitiu ligar o `cooked()` em todo consumidor sem mudar comportamento | `an_empty_stack_still_borrows_the_source` |
| Pilha **neutra** também empresta | abrir a seção Effects e não configurar nada não pode custar uma alocação/frame | `a_stack_of_neutral_effects_still_borrows` |
| Cozinhar 2× == 1× | a saída sai com a pilha **vazia**, espelhando o `corner_radius: 0.0` do `corner_live`; sem isso a forma encolhe a cada passagem, **sem erro nenhum** | `cooking_the_cooked_path_changes_nothing` |
| A **ordem** importa | é o que faz "reordenar por arrastar" ser feature | `the_order_of_the_stack_changes_the_geometry` |
| Trim mede por **ARCO** | a versão ingênua (fatiar por `t`) *parece certa numa reta* | `asking_for_a_fraction_returns_that_fraction_of_the_length` |

**20 gates novos** (6 arco · 6 pilha · 8 trim) + **7** da alça de raio + **2** arch-gates.
Os do arco usam **oráculo externo** (reta em forma fechada · amostragem densa de 200k cordas), nunca
a mesma quadratura a concordar consigo mesma.

**Mutações:** 3 na pilha/trim (3 gates distintos, um cada — inclusive a implementação ingênua da
pesquisa) · 3 no guard da alça (**conjuntos distintos**: esquecer conector+morph derruba esses 2; não
subir a cadeia derruba o envelope; recusar demais derruba os 2 controles de presença) · 1 no
arch-gate.

---

## §5 — SMOKE (o que o Enio tem de ver)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  PH2D_BUILD_SMOKE=13 cargo run -p ph2d-host-desktop --release
```

- **A elipse desenha-se sozinha** em ~3 s. ⚠️ **O que importa não é que ela apareça — é que a ponta
  ande a velocidade CONSTANTE.** Se ela acelerar e frear conforme a curvatura, a medida voltou a ser
  por `t` e o gate da fração ficou para trás.
- **A estrela**: a janela de ¼ do caminho **gira** à volta da forma e atravessa a emenda sem
  tropeçar.

**Também vale conferir o fix da alça de raio** (`e5e40aa6`), que é independente: num **filho de
envelope** (`PH2D_BUILD_SMOKE=11`), no modo **Node**, as alças de raio **não devem mais aparecer**.
Antes apareciam, funcionavam, e o raio sumia no frame seguinte.

---

## §6 — Gates rodados nesta árvore

- `cargo check --workspace --all-targets` ✅ (o campo novo em `VecPath` muda o layout postcard de tudo)
- `cargo test -p ph2d-vec-scene` ✅ **202**
- `cargo test -p ph2d-host-desktop --bins --tests` ✅ **780** + os arch-gates de arquivo (⚠️ estes
  **não** rodam com `cargo test -p` de outro crate — a linha já pagou esse pedágio duas vezes)
- `cargo clippy --all-targets` ✅ **0 warnings** · `cargo fmt` aplicado **antes** de medir LOC

**Schema:** `VEC_SCENE_SCHEMA_VERSION` 8→9 · `PROJECT_SCHEMA` **18→19**, com a tripla do gate de
acoplamento atualizada para `(19, 8, 9)`. ⚠️ **Se outra linha bumpar o `PROJECT_SCHEMA` na mesma
jornada, o valor certo não está em nenhum dos dois lados do conflito: ele se CONTA**
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). De passagem, a narrativa do gate ganhou
o **v18** que ninguém tinha acrescentado (a UNIDADE do `width` do Flip, `cb42c9a2`).

---

## §7 — O que fica ABERTO, e por que parei aqui

**A seção *Effects* no painel vetorial.** A pilha é dado de documento e hoje só a cena de smoke a
escreve — não há como o artista adicionar um Trim pela UI.

**Parei de propósito, e a razão é a DIRETIVA:** a costura de painel são ~8 arquivos (ids em
`ph2d-editor-core` · i18n · `populate_effects` · `paint_effects` · registro da seção · `event.rs` ·
o snapshot em `state.rs` · a ponte no `vector_bridge`) e **costura não-testada é uma das 4 causas
nomeadas da semana perdida no Painter**. Meio-costurada seria pior que ausente.

O padrão a copiar é o do Envelope: `populate_envelope.rs` (59 linhas) + `paint_envelope.rs`, com o
gate `architecture_panel_wiring_parity` a cobrar a correspondência entre os dois. **Registrar é o que
torna clicável** — pintar + hit-rect não basta.

`PathEffect::label()` já existe e mora **no motor**, de propósito: uma segunda lista dos efeitos no
painel divergiria da primeira assim que alguém acrescentasse um.

**Resto da fila** (com §4.2 já corrigido no handoff de continuação — o **morph vivo JÁ ESTÁ FEITO**,
`244e546e`): chamfer (quase de graça) · texto em caminho (agora barato — o `arclen` existe) ·
repeater · largura variável · blend em cadeia.

---

## §8 — Uma coisa que encontrei e não corrigi

**`CLAUDE.md` §5 diz que o `PROJECT_SCHEMA` é 13.** Ele estava em **18** quando comecei (agora 19).
Não o mexi porque o `CLAUDE.md` é território partilhado por todas as linhas e uma edição minha ali
colide com todas elas na integração — mas alguém tem de o corrigir, e o integrador é quem está em
posição de o fazer sem colisão.
