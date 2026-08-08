# Handoff de integração — `line/Vector` · **W7r: o modo de PREVIEW**

*2026-08-07 · 4 commits de código: `319a62552` o modelo · `68e3b78bd` a fiação ·
`e467ced79` o bug do gizmo · `cb9768de0` o Move All States*

> **Estado:** a wave FECHOU. ⚠️ **PENDENTE DE SMOKE** — e de ordem explícita do Enio para
> integrar. A linha não integra nem pusha sozinha.

---

## §1 — O que esta wave entrega, numa frase

**A UI que o artista desenhou passa a responder ao rato — e só dentro de um modo que, ao
sair, devolve o mundo exactamente ao que era.**

Era o único item aberto do W7. A ponte dos estados escreveu, no dia em que nasceu, as duas
razões pelas quais o rato não dirigia nada, e **as duas continuam verdadeiras**:

1. um hover que animasse a forma **enquanto o artista trabalha** tornaria o editor
   inutilizável (é por isso que o Figma põe a interação num *modo de apresentação*);
2. o undo deste editor é por **DIFF do mundo**, então uma pose escrita por hover viraria um
   passo de undo **a cada passagem do rato**.

⇒ o modo resolve as duas de uma vez, e é por isso que ele é um MODO e não um interruptor.

---

## §2 — ⭐ A lei da wave

> **Sair RESTAURA o mundo que a preview encontrou — NUNCA o estado Default.**

A tentação barata é *"ao sair, vá para o Default"*, e o modo de falha dela é **silencioso**:
o Default é uma pose **GRAVADA**, e o artista pode ter movido a forma depois de a gravar.
Sair para o Default **moveria o desenho dele** — o documento mudaria por ele ter olhado.

O gate põe o Default gravado em `0` e o mundo em `7`, **dois números que não coincidem por
acidente** (`leaving_restores_the_world_it_found_not_the_default_state`).

E o conjunto capturado é **exactamente** o que a preview pode escrever, *por construção*: a
`Machine` só emite poses cujos ids aparecem nos estados autorados, então capturar a união
deles é completo — não é uma lista que envelhece. Há gate a **medir** a afirmação em vez de
a repetir.

---

## §3 — As decisões, e o porquê de cada uma

| Decisão | Porquê |
|---|---|
| **A autoria FECHA inteira com a preview ligada** (nem Rec/Show/Clear, nem a duração) | O mundo, em preview, é uma pose **DERIVADA**; gravar dali autoraria uma pose que o artista nunca fez. E o undo está **suprimido**, então toda edição feita ali perderia o passo dela. Fechar **remove** a armadilha em vez de a documentar. |
| **O `Down`/`Up` primário é CONSUMIDO; o `Move` NÃO** | Um Down abriria um arrasto de edição por baixo do modo; um movimento não abre nada, e consumi-lo mataria **pan e zoom** — que o Figma mantém vivos pela mesma razão: *olhar de perto não é editar*. |
| **A guarda precede TODA ferramenta**, não só as do Vector | Os picks armados (conta-gotas, eyedropper de joint) são modais **sobre o Vector**; este é modal **sobre o editor**. |
| **`over_canvas_or_gizmo` é a guarda** ⚠️ *(corrigido na 2ª rodada — ver §7b)* | Ele aceita o gizmo por cima e **continua a barrar painel**, que é o que mantém o próprio botão *Preview* clicável. A v1 usava `on_canvas`, que exige o `hit_index` **vazio** — e o gizmo do hospedeiro selecionado o torna **inalcançável**. |
| **O interruptor só é oferecido onde há pose autorada** (`preview: Option<bool>`) | Um botão sobre uma cena vazia é um clique que não faz nada e que o artista não tem como diagnosticar. A pergunta é a **MESMA** que o `enter` recusa. |
| **O *ligado* é o `ButtonKind`, não o `ButtonState`** | O `ButtonState` descreve o **rato** (hover, press); escrever *ligado* nele faria o aceso **desaparecer** no instante em que o cursor passasse por cima. |
| **`held_button == Some(Primary)`, não `is_some()`** | O `held_button` guarda **qualquer** botão entre Down e Up, então um pan de botão do meio sobre um controle o mostraria `Pressed`. |
| **O readout diz como SAIR** | Um modo que toma o rato e não anuncia a porta é um modo em que o artista fica preso. |

---

## §4 — ⚠️ O que a integração precisa de saber

### 4.1 — A cadeia de Escapes MUDOU DE ARQUIVO

O `keyboard.rs` cruzou o cap de **600 LOC** (609) com o braço novo. Corte por **ASSUNTO**:
`input_dispatch/keyboard_escapes.rs` — *quem consome o Esc (e o Enter), e em que ordem*.

⚠️ **A ORDEM entre eles É a lei**, e é por isso que viajam **juntos** em vez de por dono;
espalhá-los deixaria a ordem implícita numa lista de `mod`, que é onde ela se perde. O Enter
do Painter viaja com o Esc dele porque são as duas metades da **mesma** sessão de forma.

⚠️ **E "consumir" mudou de forma:** `return;` → `return true;` (a cadeia está atrás de uma
porta que devolve *"consumi?"*).

### 4.2 — TRÊS gates IRMÃOS foram reapontados, e nenhum por o produto estar errado

| Gate | O que ele afirmava | O que ele afirma agora |
|---|---|---|
| `escape_cancels_the_drawing_before_any_tool_scoped_escape` | *"o joint é o braço 0"* | o braço 0 é a **preview** (modal sobre o editor, um nível acima) e o joint vem antes de todo Escape **TOOL-SCOPED** — que é o que o **nome dele sempre prometeu** |
| `escape_gives_up_an_armed_pick` | o literal `return;` numa janela de 120 bytes | a propriedade *ele volta daqui* (`return`) |
| `the_frame_advances_the_ui_state_machines` | a expressão de atribuição **INTEIRA** | *o retorno do `dispatch` alimenta o `ui_state_live`* — os termos da preview são pinados pelo gate desta wave |

*Uma âncora em endereço ou literal é um proxy que expira.*

### 4.3 — E um gate MEU tinha anchor AMBÍGUO, achado por mutação

`find("if self.ui_preview.is_on()")` casa **PRIMEIRO** no handler de **movimento**, ~480
linhas antes do de botão ⇒ a mutação *"a guarda deixa de consumir o clique"* **passava**, com
o intervalo examinado a varrer meia dúzia de outros `return`. **Um gate que procura no
arquivo todo afirma sobre o arquivo todo** — agora ele escopa por FUNÇÃO (`body_of`).

### 4.4 — Duas afirmações do repo que esta wave tornou FALSAS, e que foram reescritas

- o **passo 13** da cena `PH2D_BUILD_SMOKE=61` dizia *"passar o rato não anima nada — a
  interação pede um modo de apresentação, que é outra wave"*;
- o cabeçalho de `ui_state_bridge.rs` dizia *"ligar o mouse exige um modo de preview com
  história própria — decisão de produto, não trabalho mecânico"*.

*Um comentário que contradiz o código shipado é pior que comentário nenhum.*

---

## §5 — Inventário de colisão

| Eixo | Valor |
|---|---|
| **`PROJECT_SCHEMA`** | **INTOCADO** — o `project.rs` não é tocado (a preview é estado de SESSÃO, não do documento) |
| **`VEC_SCENE_SCHEMA`** | intocado |
| **ADR** | **nenhum** ⇒ a linha fica **fora** de toda disputa de número |
| **Contrato congelado** | intacto (`architecture_contract_surface` 3/3 · `architecture_tool_contract_surface` 4/4, **rodados**) |
| **`Cargo.toml`** | **ZERO** — nenhuma dep, nenhuma crate |
| **Registro do `ph2d-ecs`** | intocado |
| **ids novos** | **dois**, os dois por hash de string ⇒ sem gate de contagem: `VECTOR_STATE_PREVIEW` e `VECTOR_STATE_MOVE_ALL` (2ª rodada) |
| **i18n** | 3 chaves (`…states.preview`, `…preview.on`, `…move_all`) |
| **Arquivo novo fora do módulo** | `input_dispatch/keyboard_escapes.rs` (o corte de LOC) |

---

## §6 — Gates e mutações

**19 gates** na 1ª rodada (6 modelo · 3 seam de painel · 1 publish · 5 arch-gate de shell + 4
metades) e **+8 na 2ª** (§7b). ⚠️ **Total da wave: 16 mutações, 16 sangram** — a tabela abaixo é
só a 1ª rodada.

| # | Mutação | Sangra |
|---|---|---|
| M1 | a supressão de undo perde os termos da preview | `the_undo_is_suppressed…` |
| M2 | o `Move` CONSOME o evento | `the_pointer_move_feeds…` |
| M3 | a guarda modal deixa de consumir o clique | `the_preview_consumes_the_click…` |
| M4 | o Esc da preview desce na cadeia | `escape_leaves_the_preview…` |
| M5 | o interruptor é oferecido sempre | `the_preview_switch_is_offered_only…` |
| M6 | a autoria fica aberta na preview | `the_preview_closes_authoring…` |
| M7 | sair vai para o Default gravado | ⭐ `leaving_restores_the_world…` |
| M8 | a preview liga sobre cena vazia | `the_preview_refuses_to_open…` |
| M9 | o hospedeiro que se deixa fica aceso | `moving_from_one_host_to_another…` |
| M10 | a cadeia de Escapes nunca é chamada | `escape_leaves_the_preview…` (a 2ª metade) |
| M11 | o retorno do `dispatch` é descartado | `the_frame_advances_the_ui_state_machines` |

⚠️ **M3 sobreviveu à 1ª rodada** e o defeito era do gate (§4.3), não do produto.

---

## §7 — O SMOKE

```
env PH2D_BUILD_SMOKE=61 cargo run -p ph2d-host-desktop --release
```

A cena é a mesma do W7 e o roteiro dela cresceu — **os passos 13-17 são esta wave**:

- **13** — aperte **Preview** no topo da secção States: ele ACENDE, a linha diz como sair, e
  a autoria inteira FECHA.
- **14** — passe o rato sobre Play e Card **sem clicar**: eles reagem com o tween autorado, e
  o que você **deixa** volta ao Default no mesmo gesto.
- **15** — **aperte e segure**: vai para `Pressed` se o papel estiver gravado. Apertar no
  vazio não prende ninguém.
- **16** — o clique não pinta, não seleciona e não arrasta; **pan e zoom continuam vivos**, e
  o painel continua clicável.
- **17** — ⭐ **a PROVA:** mova o Play com o gizmo **antes** de entrar (longe do Default que
  você gravou), entre, passe o rato, saia por **Esc**. Ele tem de voltar para **onde você o
  deixou** — e o `Ctrl+Z` seguinte tem de desfazer o **seu** move, nunca um passo que a
  preview inventou.

---

## §7b — A 2ª rodada de smoke (Enio, 2026-08-07)

Veredito: *"Funciona como é dito"* — mais **um bug** e **uma feature**.

### O BUG — a preview era inalcançável sob o próprio caso de uso

*"Permite que o filho seja selecionado … com a forma longe da posição inicial, tanto o pai como
o filho."*

⚠️ **A causa estava escrita no repo, e eu escolhi a guarda errada citando-a num doc-comment meu.**
O `over_canvas_or_gizmo` diz literalmente que o `on_canvas` *"exige o `hit_index` VAZIO, mas o
gizmo registra as alças (e o interior de translação) NELE"*. Entrar na preview **exige o
hospedeiro selecionado** ⇒ o gizmo está sempre lá ⇒ a guarda modal **nunca disparava na
configuração em que a feature roda**.

E o *"com a forma longe fica pior"* é o mesmo mecanismo a mostrar-se: fora da caixa do gizmo o
`hit_index` volta a estar vazio e a guarda acorda — o **mesmo gesto** funcionava ou não conforme
**onde a forma estava**.

**Duas metades, e elas respondem a perguntas diferentes** (*de quem é o CLIQUE?* × *o que se VÊ na
apresentação?*): a guarda passa a `over_canvas_or_gizmo`, e o gizmo **não é publicado** durante a
preview (a caixa é derivada da pose AUTORADA, então fica para trás enquanto a máquina move a
forma — a razão pela qual o ADR-0128 recusou cinco vezes um gizmo sobre geometria que se move).

⚠️ **E o meu arch-gate era o buraco:** as três asserções dele provavam que a guarda existe,
CONSOME e PRECEDE as ferramentas, e nenhuma perguntava ***ela chega a correr?*** — verdes sobre
uma guarda estruturalmente inalcançável. Ele pina o **predicado** agora.

### A FEATURE — "Move All States"

*"Um botão que quando checado permita mover o widget … de modo que todas as posições dos
componentes em todos os estados sejam movidas relativamente."*

⚠️ **O defeito por trás do pedido, medido:** um estado grava a sub-árvore, e o hospedeiro está
nela **sempre que ele próprio é uma forma desenhada** ⇒ a translação dele fica congelada em cada
estado, e relocar o widget faz o Show seguinte devolvê-lo ao lugar antigo. ⚠️ **Um hospedeiro que
seja um GRUPO puro nunca teve o problema** (o `members` não o inclui), e há gate a medir isso —
é ele que separa *"a feature é necessária"* de *"a arquitetura está errada"*.

**Só o HOSPEDEIRO se desloca**, e é isso que torna a operação correta: as poses dos filhos são
locais a ele, então mover o `Transform` do pai já as leva junto; deslocá-las também moveria tudo
**duas vezes**.

⚠️ **A guarda anti-realimentação tem DUAS metades e nenhuma basta sozinha** — a condição
`!ui_state_live` **e** o ancoradouro re-escrito em TODO quadro. Sem a segunda, um Show acumula
dívida e o primeiro quadro que aplicar a despeja de uma vez. ⚠️ **O gate dela sobreviveu à
mutação na 1ª rodada:** `anchor > apply` não distingue *depois* de *dentro do ramo* — a mesma
distinção que o arch-gate do memo do Painter pagou em 2026-07-22.

**Opt-in de propósito:** desmarcado, mover re-autora só a pose atual, que é o que se quer quando
a intenção é corrigir UM estado.

**Smoke:** os passos **18-20** — o 18 **MOSTRA o defeito** antes de o 19 o curar, e o 20 é o
controle com a caixa desmarcada.

**Inventário desta rodada:** id novo `VECTOR_STATE_MOVE_ALL` (hash de string) · 1 chave i18n ·
`PROJECT_SCHEMA` **INTOCADO** (o toggle é transiente — ele qualifica o próximo ARRASTO, não o
documento; a classe do `BakeChannels` da física) · **zero `Cargo.toml`** · **nenhum ADR**.
**Total da wave: 16 mutações, 16 sangram.**

---

## §8 — Aberto, nomeado

- **Um grupo inteiro como hospedeiro** funciona (a sub-árvore é a unidade), mas **um
  hospedeiro dentro de outro** não foi exercitado — o `host_under` devolve o **primeiro** da
  lista de picks que pertence a algum hospedeiro, e com aninhamento a resposta certa é
  decisão de produto (o de dentro? o de fora?).
- **`Disabled` não tem gatilho** — os outros três papéis derivam dos dois fatos do rato, e
  *desabilitado* é um fato do DOCUMENTO. Ligá-lo exige decidir onde esse fato mora.
- **A preview não tem indicador fora do painel** — com a secção States fechada ou o painel
  escondido, o único sinal é a cena responder. O Esc continua a sair.
