# Plano — a **máquina de estados do Morph**, autorada no canvas

> **Fila F1** ([doc 29](29_fila_morph_state_machine_e_texture_pattern.md)) · pesquisa em
> [doc 31](31_pesquisa_maquinas_de_estado.md) · o estado medido na reabertura está no §
> *RECONFERÊNCIA* do doc 29.
>
> Enio, 2026-08-24: *"um tipo de state machine específico para o tool Morph (…) entre múltiplas
> formas, de forma não destrutiva e funcional no runtime do game"*, com as setas *"no próprio canvas
> 2D (…) e nas setas colocaremos condições"*, e a configuração *"à seção states do módulo vector
> utilizando um modo preview"*. Em 2026-08-25, com as duas opções na mão: **"as setas devem ser
> desenhadas no canvas onde as formas foram desenhadas"**.

## §0 — O modelo, numa frase

⭐ **Um ESTADO é uma FORMA DESENHADA.** O artista desenha A, B e C no canvas e liga-as com setas; o
estado da máquina é *em qual delas ela está*, e a seta é *como se vai de uma para a outra*.

É a decisão que **apaga o caso especial**: nada de novo a nomear, nada de novo a gravar, e o que ele
vê **é** o modelo. Foi ela que a frase do Enio escolheu — *"setas de uma forma para outra"*.

### ⚠️ Isto NÃO é o `ph2d-ui-state`, e a correcção fica escrita

Na reabertura eu afirmei ao Enio que *"a máquina que contém a outra é a `ph2d-ui-state`"*. **Estava
errado**, e a medição que o mostra é sobre o produto, não sobre o código: aquela máquina interpola
**poses de N objectos** (translação, tinta, traço, geometria) entre papéis **fixos** de UI — hover,
pressed. Um estado dela é uma **gravação**, não uma forma no canvas. Aqui o assunto é **duas formas
e um `t`**.

> *Quem decide qual subsistema serve é o que o ARTISTA desenha, não o código que está por perto.*

O que de facto se reusa: o **motor** (`ph2d-spring`, `ph2d-anim::Easing` — os dois já folhas), o
**renderizador do par** (`VecMorph`, intocado), a **separação de undo** (`Driver::MorphT`, que já
existe) e o **vocabulário das condições** (as acções do Input Map, de ontem).

## §1 — A porta ÚNICA de cada pergunta

| Pergunta | A porta | ⚠️ |
|---|---|---|
| *que formas, que setas, que condições?* | `MorphGraph` (autorado, no documento) | as formas são **derivadas** das setas (`shapes()`) — uma lista à parte teria um estado que nenhuma seta alcança |
| *em que forma estou?* | `MorphMachine::current` | salta **no lançamento**, nunca na chegada nem na fila (§2) |
| *que par a cena mostra?* | `MorphMachine::pair` → `VecMorph::sources` | fica o par do ÚLTIMO voo com `t=1`; ⛔ **não** `(current, current)` |
| *onde no caminho?* | `MorphMachine::t` → `VecMorph::t` | e o undo não o vê — `Driver::MorphT` já existe |
| *a acção X aconteceu?* | `MorphMachine::fire(&str)` | ⛔ a crate **nunca** resolve o nome |
| *o que fazer daqui?* | `MorphMachine::live_actions` | a cura do medo do Animator (doc 31) |
| *ver a seta sem lhe dar nome* | `MorphMachine::travel(ix)` | a porta da pré-visualização |

## §2 — As leis, e por que cada uma é assim

1. ⭐ **Só as setas do estado CORRENTE disparam** (`MorphGraph::from`) — a correcção nº 1 da
   pesquisa (o *State Tree* do Unreal). Um varrimento global é a teia do Animator, construída por
   acidente.
2. ⛔ **Uma seta sem condição nunca dispara sozinha.** Ela nasce sem nome quando o artista a
   desenha; sem esta guarda, toda seta recém-desenhada responderia a uma acção de nome vazio.
3. ⭐ **O `current` salta no LANÇAMENTO** — a meio de `A→B` as setas oferecidas são as de `B`.
4. ⛔⛔ **Mas ele NÃO salta num pedido em fila**, e o gate nasceu vermelho a provar porquê: com o
   salto na fila, o segundo pedido era lido a partir de um estado onde a máquina ainda não está —
   ou não casava com seta nenhuma (*o input do jogador desaparecia em silêncio*), ou casava com uma
   seta cujo `from` não é onde ela aterra, e o par **saltava um estado inteiro**.
5. **Um pedido a meio do voo ESPERA a chegada** (*input buffer* de UM). ⛔ As duas alternativas
   estão fechadas: **ignorar** perde o input; **saltar** não é exprimível, porque o `VecMorph`
   guarda **um par** e sair do meio de `(A,B)` para `(B,C)` precisaria de uma mistura de três.
6. **O mais novo ganha e a fila não cresce** — uma fila funda reproduziria teclas que o jogador já
   esqueceu. Seguro **por construção** graças à lei 4: todo candidato parte do mesmo sítio.
7. **A fila é RECONFERIDA na chegada** — o artista pode ter apagado a seta durante o voo.
8. ⭐ **Chegar não troca o par.** O cache de `Plan` do `morph_live` é chaveado pela geometria em
   **mundo** das duas fontes, e a busca de fase custa os **5,9 ms** que o `Plan` foi inventado para
   matar; `t=1` em `(A,B)` já **é** a forma B.

## §3 — Onde encosta

* **Contrato congelado (§6):** nenhum. A crate é nova e o `VecMorph` **não se mexe**.
* **Schema:** o grafo é conteúdo autorado ⇒ `PROJECT_SCHEMA` sobe (medido **97** em 2026-08-25;
  ⛔ conte-o contra o `main` do dia, nos **três** sítios).
* **Registro de componentes:** o componente do grafo é um registo novo — espelhos em **71**, e o
  número **soma entre linhas**.

## §4 — As waves

| | | estado |
|---|---|---|
| **W1** | **A LEI**, folha (`ph2d-morph-machine`): grafo · setas · condições · fila · mola/curva | ✅ **2026-08-25** — 13 gates, **8 mutações, 8 sangraram** |
| **W2** | O componente + a persistência (`PROJECT_SCHEMA`) | ⏳ |
| **W3** | **O CANVAS**: desenhar as setas entre as formas, e autorá-las por arrasto | ⏳ |
| **W4** | A secção **States** do painel: a lista de setas, a condição (lê as acções do Input Map), o ritmo | ⏳ |
| **W5** | O **modo preview** + o ledger de undo (⚠️ o `Driven::MorphT` cobre o `t`, **não** o `sources`) | ⏳ |
| **W6** | A cena de smoke, com números MEDIDOS | ⏳ |

⚠️ **O que a W5 vai encontrar, e está medido de antemão:** o ledger de pré-visualização
(`preview_drive.rs`) já tem `Driver::MorphT` / `Driven::MorphT(f32)` — construído em 23/08 para a
curva de `Morph` da timeline. Ele guarda **o `t` e só o `t`**. A máquina escreve **também o
`sources`**, e esse facto **não tem dono no ledger** ⇒ sem o acrescentar, mudar de par durante a
pré-visualização entra no undo.

> ⚠️ **Meça cada linha deste plano antes de a honrar.** Escrito em 2026-08-25.
