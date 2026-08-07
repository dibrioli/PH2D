# HANDOFF DE INTEGRAÇÃO — `line/Vector`, W4c.4: os tokens de ESCALA chegam ao DOCUMENTO

> **Data:** 2026-08-06 · **Branch:** `line/Vector` · **Commit:** `ba37d0725`
> **Estado:** fechada, **pendente de smoke** e de ordem de integração.
> Waves anteriores desta jornada: [W4c.1](HANDOFF_INTEGRACAO_line_Vector_W4c1_2026-08-06.md) ·
> [W4c.2](HANDOFF_INTEGRACAO_line_Vector_W4c2_2026-08-06.md) ·
> [W4c.3](HANDOFF_INTEGRACAO_line_Vector_W4c3_2026-08-06.md).

---

## 1. O que a wave entrega, numa frase

A **espessura de um traço** e o **vão de um auto layout** deixam de ser literais e passam a poder
**SEGUIR um token numérico**, exactamente como a cor já seguia — e mudar o token move a arte, ao
vivo, no modo vigente.

---

## 2. ⚠️ A MEDIÇÃO que reescreveu a wave antes de uma linha ser escrita

A fila dizia: *"Escala — cai **de graça** no (1)+(2): escala **é** um token numérico. Se custar
mais que fiação, o (1) foi feito estreito demais."*

**A frase está errada, e o (1) não tem culpa.** Os três alvos que o plano nomeia vivem em
**unidades de MUNDO** — `StrokeSpec::width`, `VecLayout::gap` (o doc dele diz *"em unidades de
MUNDO (as do documento)"*), `VecVertex::corner_radius` — e um `NumToken` vale **PIXELS**. Uma cor é
adimensional e atravessa a fronteira sem conversão; um comprimento não.

**O número que mata a leitura ingênua:**

| | valor do token | lido como MUNDO | fração de uma moldura de telefone |
|---|---|---|---|
| `stroke.default` | 1,5 px | 1,5 unidade | **19% da altura** (`frames::LONG_SIDE = 8`) |
| `radius.full` | 999 px | 999 unidades | **125 molduras** |

⚠️ **A régua existe e já tem um dono declarado**, então a wave não a escolheu:
`ProjectSettings::pixels_per_meter` (ADR-0131 D4 — *"a única px→m é a do PROJETO; um 2º
`PIXELS_PER_METER` seria a segunda porta que diverge"*). Com o default de 100, `stroke.default`
vale **0,015 unidade = 1,58 pt** naquela moldura — o cabelo que o token promete.

⚠️ **E ela NÃO é o `px_to_world` da câmera**, embora a row *Width* do painel fale nele: aquele
número é px de **TELA** no zoom do momento (`vector_bridge.rs`: *"a largura viaja em px de tela na
tool e em MUNDO no documento"*), então resolver por ele faria o valor **salvo** depender de onde o
artista estava a olhar.

---

## 3. O diff, por assunto

### 3.1 O modelo (`ph2d-ecs`)

- `BoundProp` += **`StrokeWidth`(2)** · **`LayoutGapMain`(3)** · **`LayoutGapCross`(4)**,
  append-only, discriminantes pinados por gate.
- `BoundProp::from_code(u16)` — a inversa do discriminante, que é o que liga o clique do picker ao
  alvo sem um `match` paralelo na shell.
- ⚠️ **`CornerRadius` fica FORA, e o motivo MUDOU.** A nota antiga dizia que os três esperavam *"o
  canal que os resolve"*, e o canal chegou (a camada numérica da W4c.1). O que falta ao raio é
  outra coisa: ele é **por-VÉRTICE** (autorado pela alça do modo Node e pela ferramenta Fillet) e o
  painel **não tem um controle por-FORMA** para ele. Um binding é por-forma ⇒ hoje ele seria um
  alvo que nada preenche. A frase é a mesma; o vão é outro, e está reescrita no código.
- ⚠️ **`TokenFamily` NÃO ficou aqui.** *De que tabela um alvo se serve* é pergunta de UI, e o
  painel **não depende do `ph2d-ecs`** — a resposta mora onde os dois lados chegam (§3.3).

### 3.2 A régua e a resolução (shell)

- **`vec_bindings::token_world(key, tok)`** — a porta ÚNICA px↔mundo, irmã do `token_color`.
  Chave desconhecida devolve `None` (o literal vale); régua zero devolve `None` (o campo é público
  e uma espessura infinita pinta a tela inteira).
- **`vec_bindings::TokenCtx { theme, pixels_per_meter }`** — par nomeado, e não dois escalares
  soltos: o passe de DESENHO e o de AUTO LAYOUT resolvem tokens em sítios diferentes, e um
  `Theme`/`f32` soltos numa lista de argumentos são o par que alguém troca de ordem sem o
  compilador reclamar. `TokenCtx::factory()` é `#[cfg(test)]` — um `Default` alcançável do produto
  seria a porta por onde alguém resolveria um comprimento com a régua errada.
- **`vec_bindings::bound_gap(sim, frame, tok)`** — o vão resolvido por eixo, pela MESMA porta de
  conversão da largura.

### 3.3 A tabela ÚNICA de slots (`ph2d-editor-core::ids`)

`TOKEN_SLOTS: &[TokenSlot { code, chip, table }]` + `TokenTable::{Colour, Length}` (com `len` /
`key` / `position`) + `token_slot(code)` / `token_slot_of(chip)`.

⚠️ **Antes desta wave, CINCO consumidores traziam o seu próprio `[(0, Fill), (1, StrokeColor)]`
escrito à mão:** o `populate`, o `paint`, o `selected_row`, o roteamento de `Click` do painel e o
`token_choice` da shell. Com uma segunda **família** a entrar, o que um deles esquecesse viraria
**um chip pintado, com hit-rect, e morto sob o mouse** — o defeito que este painel já pagou quatro
vezes. Hoje os cinco percorrem a mesma lista.

⚠️ **O `code` É o discriminante do `BoundProp`**, e não um numerador paralelo — a shell o converte
de volta por `from_code`.

### 3.4 O desenho (`ph2d-vec-scene`)

- **`BoundPaint` → `BoundStyle`** (rename mecânico, 7 arquivos): uma largura não é tinta, e um
  `BoundPaint { width }` seria um nome a mentir em todo sítio de uso.
- `BoundStyle.width: Option<f64>` (em MUNDO, **já convertido** — esta crate não conhece régua, como
  já não conhece tema) e `VecPath::painted` a aplica.
- ⚠️ **A espessura segue a MESMA lei da cor do traço, e pela mesma metade que falta:** um token de
  largura numa forma **sem traço** teria de inventar a COR ⇒ ela engrossa o traço que existe e
  nunca cria um. O early-out do `Cow` a considera, então uma forma sem traço continua a devolver
  `Borrowed` — byte-idêntica.
- ⚠️ **`Eq` saiu do derive** porque `f64` não é `Eq`. Ninguém compara duas entradas por igualdade
  total; o que se perderia era uma promessa falsa.

### 3.5 O layout (`layout_live`)

`frame_style(l, gap: [Option<f64>; 2])` — o vão chega **resolvido** ao motor, no sítio ÚNICO onde
ele entra. `recook` ganhou o `TokenCtx`.

### 3.6 A UI (`ph2d-panel-vector`)

- Row de **Token** sob o slider *Width* (seção Stroke) e sob os campos de *Gap* (seção Layout).
- **A rachura** sobre o controle que o token cobre: o chip do Width vem de
  `widget::slider_with_chip_chip_rect` — a porta do widget, e não uma 2ª conta da posição, que
  divergiria quando a row empilha num painel estreito. `number_cell` passou a devolver o `Rect` que
  pintou, pela mesma razão.
- ⚠️ **As rows seguem as MESMAS cercas dos campos que anotam:** a da largura só existe com traço
  (o `stroke_exists`), a do vão só sobre uma moldura que flui (o `flows`, gêmeo novo), e a do vão
  **transversal** só no `Wrap` — exactamente onde o campo dele já é oferecido.
- `TokenBindings::of_slot(code)` — porta única de *"que chave está presa neste slot?"*, para o
  rótulo do chip e a linha destacada do picker não poderem divergir.
- `token_row` passou a tomar o **CHIP** e derivar o código: o par viajava solto, e nada impedia
  pintar o chip de um slot com o código de outro.

### 3.7 O *detach* (autorar o número solta o token)

- **Largura:** pelo canal one-shot (o `note_authored` genérico, que virou **máscara** sobre o
  discriminante em vez do par `(bool, bool)` — ele teria de virar tripla nesta wave).
- **Vão:** direto no `apply_layout_field`, que já tem mundo e seleção na mão. ⚠️ O canal one-shot
  da cor existe só porque quem SABE que uma cor foi autorada é o tool, que não tem nenhum dos dois.

---

## 4. Schema

| | antes | depois |
|---|---|---|
| `PROJECT_SCHEMA` | 59 | **60** ⚠️ **PROVISÓRIO** |
| `VEC_SCENE_SCHEMA_VERSION` | 14 | **14** (intacto) |
| `FLIP_SCHEMA_VERSION` | 13 | **13** (intacto) |

Tripla do pin: **`(60, 13, 14)`**.

⚠️ **O 60 se CONTA contra o `main` do dia da integração, não se escolhe**
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]) — a `line/FLIP` e a `line/physics` já
colidiram três vezes por escolher, e uma delas passou **muda** porque os dois lados escreveram o
mesmo literal e o git não tem opinião sobre o que o número significa.

⚠️ **Por que bumpar, já que apender variante não move nada:** `Fill`(0) e `StrokeColor`(1) ficam
onde estão, então **todo binding já salvo continua a ler**. O bump é pelo caminho **INVERSO** — um
build antigo a ler um arquivo novo bateria num índice de variante que ele não tem, e o número
transforma isso num erro de VERSÃO em vez de num postcard a falhar longe da causa. É o raciocínio
do `JointKind::Weld` (v28) e do `Cap::Square` (v48).

⚠️ **`VEC_SCENE_SCHEMA` fica quieto** numa feature de ESTILO porque o binding é **tabela LATERAL no
ECS**: nenhum campo foi apendado a `Paint`, a `StrokeSpec` ou a `VecShape`. É a decisão inteira do
`vec_bindings`, e é ela que mantém todo save de vetor com a mesma forma.

---

## 5. Superfície e colisão

| Eixo | Estado |
|---|---|
| **ADR** | **nenhum novo** ⇒ a linha fica **fora** da disputa de número desta janela |
| **Contrato congelado** | **intacto** — `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` e `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` verdes (rodados, não auto-relatados) |
| **`Cargo.toml`** | **zero tocado** — nenhuma dep, nenhuma crate nova |
| **Registro do `ph2d-ecs`** | **54, intocado** (nenhum componente novo: `VecBindings` já existia) |
| **ids novos** | `VECTOR_TOKEN_WIDTH` · `VECTOR_TOKEN_GAP_MAIN` · `VECTOR_TOKEN_GAP_CROSS` (consts, hash de slug) |
| **Arquivos novos** | `shells/desktop/src/render_loop/vector_bridge_vocab.rs` |
| **i18n** | nenhuma chave nova (a row reusa `panel.vector.token`) |

⚠️ **Ponto de merge sensível:** o rename `BoundPaint` → `BoundStyle` toca **7 arquivos** em 3
crates. Uma linha vizinha que tenha acrescentado um consumidor de `BoundPaint` vai conflitar por
NOME — o conserto é mecânico (o tipo é o mesmo, mais um campo `width: None`).

⚠️ **E `TOKEN_SLOTS` substituiu cinco listas escritas à mão.** Um merge textual limpo pode
**perder** um slot que outra linha tenha acrescentado a uma das listas antigas; quem pega isso é o
`every_token_slot_is_alive_and_lists_its_own_table`.

---

## 6. Gates e mutações

**9 mutações, 9 sangram** — ⚠️ e **QUATRO** delas só passaram a sangrar depois de o buraco que
expuseram ganhar gate:

| # | Mutação | Veredito |
|---|---|---|
| M1 | a régua some (`px` cru como mundo) | sangra 2 |
| M2 | a largura resolvida não chega ao traço | sangra 1 |
| M3 | o `frame_style` ignora o vão resolvido | **SOBREVIVEU** → gate novo (§6.1) |
| M4 | o `populate` conta pela tabela curta | sangra 2 |
| M5 | todo picker lista a tabela de COR | **SOBREVIVEU** → gate novo (§6.2) |
| M6 | digitar um vão não solta o token | **SOBREVIVEU** → gate novo (§6.3) |
| M7 | digitar uma largura não solta o token | **SOBREVIVEU** → gate novo (§6.4) |
| M8 | o `token_choice` só enumera os slots de cor | sangra 2 |
| M9 | a rachura da largura sem guarda | sangra 1 |

### 6.1 `a_gap_token_spaces_the_flow_and_the_literal_is_ignored_while_it_is_bound`
`bound_gap` podia estar perfeito e o `frame_style` continuar a ler o literal. O oráculo é a
**POSIÇÃO do segundo filho**, e não o número que a régua devolve — o número seria o espelho do
`token_world`, e o que se quer saber é se a moldura de facto espaça por ele. A fixture afirma
primeiro que o token vale um número **diferente** do literal, senão o gate não distingue as fontes.

### 6.2 `every_pickers_painted_list_is_its_own_table` (painel) — irmão de
`every_token_slot_paints_its_own_table` (shell)
⚠️ **Os dois não são redundantes**, e é a lição da wave: aquele prova que o **CLIQUE** decodifica
para a tabela certa, este que a **LISTA pintada** é a certa. Com só o primeiro, o picker da
espessura podia oferecer nomes de cor — o artista escolheria `"accent"`, o índice dele seria
decodificado contra a tabela de COMPRIMENTO, e a espessura pousaria num token que ele nunca viu.

### 6.3 `typing_a_gap_detaches_that_axis_token_and_only_that_one`
Com CONTROLE: editar o **recuo** não pode soltar o token de um **vão** — a lei não alcança mais do
que diz.

### 6.4 `authoring_a_width_arms_the_detach_in_the_bridge`
Arch-gate, porque a decisão mora no `vector_bridge::dispatch`, que exige janela e tool.

⚠️ **As âncoras dos arch-gates desta wave são ADJACÊNCIA DE LINHA** (*a nota/rachura é a primeira
linha da guarda dela*), nunca uma distância em bytes. A 1ª versão do gate da rachura usava uma
janela de 160 bytes e **expirou dentro da própria wave**, quando o `let chip = …` entrou no meio
([[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]]).

### 6.5 Dois defeitos de gate PRÉ-EXISTENTES que a wave encontrou

- **`the_gesture_reads_what_the_frame_drew`** ancorava na literal `".layout_live\n                
  .recook("` — a indentação escrita à mão. A chamada ganhou um argumento, o `rustfmt` a
  re-quebrou, e a agulha deixou de casar com **código correto**. ⚠️ O doc do próprio gate já
  registava esta doença e **a metade de cima já tinha sido curada** (ela lê o fonte sem espaço em
  branco); a de baixo ficou para trás. *Curar só uma das metades é como ela voltou a morder.*
- **`node_id_collisions`** não listava `VECTOR_TOKEN_FILL`/`_STROKE` — eles existiam desde a W4a e
  **nunca participaram da varredura de unicidade**. Os cinco entraram.

---

## 7. Como rodar

```
# a cena de PRENDER ao token (a arte que segue a tabela) — é onde a wave nova se julga
env PH2D_BUILD_SMOKE=51 cargo run -p ph2d-host-desktop --release

# a cena do PAINEL (autorar a tabela) — a irmã, para mexer nos números
env PH2D_BUILD_SMOKE=59 cargo run -p ph2d-host-desktop --release

# a bateria de fechamento
bash scripts/nextest-impacted.sh
cargo clippy --workspace --all-targets
cargo fmt --all -- --check
cargo machete
```

**Fechamento medido nesta árvore:** `nextest-impacted` **8899/8899** · clippy limpo · fmt limpo ·
machete limpo · LOC (fn, arquivo de painel, arquivo de shell, workspace) verdes · contratos
congelados verdes.

---

## 8. O smoke — o que julgar

Cena **`=51`**. Ela **não binda nada**: quem prende é o artista (a cicatriz que o `impasto_smoke`
prega). Os passos 1–11 são as waves anteriores; **os 12–16 são esta.**

1. **Passo 12 — a ESPESSURA.** Selecione o fundo do card da esquerda (ele tem traço). Na seção
   Stroke, logo abaixo do slider *Width*, há uma row **Token** nova. Escolha `stroke.heavy`.
   ⚠️ O contorno **engrossa**, e o chip do Width ganha a **rachura**: o número que ele mostra é a
   largura autorada, e o token a cobre.
2. **Passo 13 — a pergunta.** Tecla `T` (painel de Tokens) → seção *Scale (px)* → mexa em
   `stroke.heavy`. **O contorno da ARTE segue junto** — e o app também, porque é a mesma tabela.
3. **Passo 14 — o detach.** Digite um número no campo *Width*: o token volta para *None* sozinho.
4. **Passo 15 — o VÃO.** Selecione a barra de baixo (a moldura com três quadrados). Seção Layout →
   direção **Row** → na row **Token** abaixo do campo *Gap*, escolha `spacing.4xl`.
   ⚠️ **A fila se abre.** Mexa em `spacing.4xl` no painel de tokens: o espaçamento segue.
5. **Passo 16 — o CONTROLE da régua.** Se algum traço sair com ~19% da altura da moldura, a régua
   se perdeu — é exactamente o absurdo que a §2 mede.

⚠️ **E o controle de sempre:** o card da direita **não se mexe** em passo nenhum.

---

## 9. Aberto, nomeado

- **`CornerRadius`** — falta um **controle por-FORMA** para o raio (§3.1). É wave de produto (um
  campo *Radius* na seção Vertex, ou por-vértice bindável), não fiação.
- **O recuo (`pad`) do auto layout** não é bindável. Os quatro lados + o modo *All* são quatro (ou
  cinco) slots novos, e a pergunta *"o `All` binda os quatro?"* é de produto.
- **Multi-seleção** continua fora, pela razão de sempre: *"e se elas discordarem?"*.
- **W4c.5 — DTCG** é o que resta da fila. ⚠️ `color.rs:276` já nota que a nomenclatura das chaves
  casa com o que o DTCG fala, e agora as chaves numéricas são **pontuadas** (`spacing.md`) — confira
  antes de inventar um mapeamento.

---

⚠️ **A linha NÃO integra e NÃO pusha sozinha** (CLAUDE.md §0.7): ela fecha, entrega este handoff e
espera ordem explícita do Enio, executada por um agente integrador dedicado.
