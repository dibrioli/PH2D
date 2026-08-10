# Handoff — Impasto: uma casa só para as dez ferramentas (`line/Painter`, 2026-07-19)

> **NÃO integrado, NÃO pushado.** Linha fechada, handoff escrito, PARADO (CLAUDE.md §0.7).

## §0 — O pedido

Enio, 2026-07-19:

1. *"o checkbox Adjust Last Stroke deve ser desmarcado por padrão."*
2. *"Perceba que os tools de Impasto estão espalhados em 3 lugares: no painel brush, no smear e Sculpt.
   Vamos unificar e organizar tudo num único lugar no painter. As tools todas devem ser organizadas
   logo abaixo de Adjust Last Stroke."*
3. *"seja um profissional no layout dos botões e propriedades das tools."*

## §1 — O que a leitura do código achou antes de mexer

As três casas não eram só espalhamento — **cada uma existia em exatamente um modo de pintura**:

| onde | alcançável em | o que você tinha |
|---|---|---|
| a seção Impasto | `Paint` só (`impasto_applies`) | Body · Material · **Lighting** |
| o Plow | `Smear` só (`impasto_plow_applies`) | Plow, e mais nada |
| o card Sculpt | `Sculpt` só | os 8 verbos, num card no TOPO do painel |

Lendo a coluna da direita para baixo: **o card Lighting era alcançável no Brush e em lugar nenhum
mais.** Ou seja — você entrava em Sculpt, o modo cuja razão de existir é *moldar relevo*, e perdia os
controles que tornam relevo **visível**. Idem no Smear. Não é "três lugares para um assunto": é dois
dos três lugares sem como enxergar o que estão fazendo.

Isso decidiu o desenho.

## §2 — O desenho

Uma seção, e a ordem descendo o painel é a ordem das perguntas:

```
▼ IMPASTO                                   [•] [↺]
  [ ] Adjust Last Stroke        ← governa TODO slider abaixo ⇒ não pertence a caixa nenhuma
  ┌ TOOL ─────────────────────────────────┐
  │ Deposit  Knife  Smooth  Sharpen       │  ← as dez, logo abaixo do checkbox
  │ Flatten  Scrape  Fill  Chisel         │
  │ Layer    Inflate                      │
  └───────────────────────────────────────┘
  ┌ <as propriedades da ferramenta selecionada, e só dela> ┐
  ┌ Material ─────────────────────────────┐  ← só no Deposit
  ┌ Lighting ─────────────────────────────┐  ← toda ferramenta (mas só com Enable ON — §5d)
```

**Material estreita, Lighting não.** Material é per-BRUSH e é assado no canvas *com o depósito*; cada
modo tem slot de pincel próprio, então um Shine sob a Faca ou sob um verbo editaria um slot que nada
lê — knob morto, na seção que mais já pagou por eles. Lighting é o oposto: é do CANVAS, e não poder
acender o relevo que você está moldando **era o bug**.

**Enable desceu para dentro do card do Deposit.** Ele é o interruptor do depósito ("este pincel deposita
corpo"), não da seção. Ele gateava a seção inteira — o que também levava embora o Lighting, então "este
pincel não deposita" significava em silêncio "e você não pode acender o de mais ninguém".

### ⚠️ A consequência de arquitetura

**Escolher uma ferramenta USA ela** ⇒ o seletor troca o `PaintMode`. Logo esta lista e os chips do rail
esquerdo são **duas VISTAS de um rádio, não dois rádios**. Por isso o rádio do rail passou a ser
**derivado do modo publicado** (`sync_from_mode` ← `paint_mode_wire()`) em vez de escrito por quem foi
clicado por último. Sem isso, escolher Chisel aqui deixaria o rail destacando "Brush" enquanto você
esculpe: duas respostas para *"que ferramenta estou segurando?"*, e a errada na tela.

O seletor roteia pelas **portas que já existiam** (`set_paint_tool_mode` + `set_sculpt_mode`), nunca
escrevendo `paint_mode` direto — aquelas portas commitam um fill vivo, trocam o slot de pincel e encerram
as sessões de warp/deform.

**Um id por ferramenta:** os 8 verbos **reusam** `PAINTER_SCULPT_MODE_IDS`. Um segundo conjunto teria de
ser mantido em passo com o primeiro e responderia a mesma pergunta — que é como dois viram divergentes.
O router do sculpt agora **delega** para `set_impasto_tool`.

## §3 — Arquivos

**Novos:** `ph2d-tool-painter/src/tool/paint/impasto_tool.rs` (as dez, o predicado, a porta) ·
`ph2d-panel-painter-layers/src/paint_impasto_tool.rs` (o card TOOL + o despacho por-ferramenta) ·
`tests/seam_impasto_tool.rs` (6 gates).

**Splits por LOC cap:** `brush_settings.rs` 694→489 + `brush_core_settings.rs` (214) — o bloco
`impl PainterTool` de falloff/size/spacing/jitter/dash. `paint.rs` bateu 702 e voltou a **700 exatos**
reancorando dois doc-comments que minhas linhas `mod` tinham **órfãnado** (o doc de `brush_settings`
tinha passado a descrever o módulo novo).

**Tocados:** `paint_impasto.rs` (corpo reescrito; `paint_plow_only` → `paint_knife_card`) ·
`paint_sculpt.rs` (o `mode_card` saiu; `paint_sculpt_section` → `paint_sculpt_rows`) · `paint_brush.rs`
(chamada do topo removida) · `snapshot.rs` · `brush_fallback.rs` · `sculpt_panel.rs` ·
`impasto_settings.rs` · `ids/chrome/painter_impasto.rs` · `rail_painter_tools.rs` · `painter_bridge.rs`.

## §4 — Item 1: o default (commit `b29cfabb`)

`impasto_live_edit` nasce `false`. Gate novo `a_fresh_brush_does_not_adjust_the_last_stroke` pina o
**COMPORTAMENTO** que o artista encontra (traço → Depth+Shine → canvas byte-idêntico), não o booleano.

**O achado do caminho: TREZE fixtures herdavam o default em silêncio** — as duas do próprio toggle, a do
material/undo, as três de live-knob e as sete de material/lâmpada. Todas testam a **capacidade**
live-edit, nunca o default. Agora declaram a premissa (`t.paint.impasto_live_edit = true`) em vez de
chegar nela por *toggle*: uma fixture que alcança seu estado togglando **inverte de sentido** no dia em
que o default se move, e continua verde testando a afirmação oposta. Corrigido em 8 sítios.

## §5 — Gates e mutações

**6 gates novos** (`seam_impasto_tool.rs`, dirigidos por PONTEIRO) + **2** no rail (`editor-core`).

| mutação | o que sangra |
|---|---|
| 1. `impasto_section_applies` de volta a `Paint` só | **4 de 6** |
| 2. escolher verbo não entra em Sculpt | 2 de 6 |
| 3. `sync_from_mode` retorna cedo | o gate do rail, e só ele |
| 4. Material vaza para a Faca | o gate do material, e só ele |
| 5. tirar o guard de mesmo-modo | **SOBREVIVE — ver abaixo** |

### ⚠️ Dois defeitos de gate, os dois meus

**(a) O gate "reachable by a pointer" nasceu VERDE provando a coisa errada.** Ele achava os dez
retângulos e passava — enquanto os chips estavam **mortos sob o mouse**, porque a fixture montava o host
com `MockPanelHost::new()`, que **pula o `populate`**. Pintado, hit-registrado e inerte: exatamente a
falha que o cabeçalho daquele arquivo descreve, reproduzida pelo gate feito para pegá-la. Agora ele
**clica** os dez.

**(b) Eu ia shipar uma afirmação FALSA num doc-comment.** Escrevi que o guard de mesmo-modo era
"correctness, não otimização" — que re-entrar no modo atual encerraria a sessão de sculpt que a troca de
verbo vai re-carimbar. A mutação 5 sobreviveu à workspace inteira, então fui **ler** o
`set_paint_tool_mode`: todos os três encerramentos de sessão dele já são gateados em `old != new`
(`stencil.rs`, sair do Smear / do Deform / do Sculpt). Chamada com o modo atual **não encerra nada**. O
guard é higiene (evita re-commitar um fill que não roda e um round-trip do slot a cada clique de verbo).
Doc corrigido; o guard **fica**, porque torna este call site robusto contra a forma de guard que aquela
função já carrega três vezes — o próximo escrito sem `old != new` dispararia numa troca de verbo, e *esse*
seria o bug que o parágrafo original descrevia.

## §5b — Segunda rodada (mesmo dia, pós-screenshot do Enio)

**(1) `Enable` subiu para o TOPO da seção.** Enio: *"já que ele é quem habilita esse modo de pintura"*, e
*"esse card só aparece se enable de Impasto estiver checado"*. Ele agora é a primeira linha e gateia tudo
abaixo — **o Lighting incluído** (ver §5d: eu tinha isentado a luz, o Enio reverteu).

**(2) O bug que o gating expôs, MEDIDO antes de curar.** Cada modo tem `BrushSpec` próprio, então o
Enable era por-slot. Com ele gateando a lista: tique no Deposit → clique **Knife** → `switch_brush_slot`
carrega o `impasto` do Smear (`false`) → a seção colapsa **e leva embora a lista de onde você acabou de
clicar**. Medido: `Deposit true → Knife false → Chisel false`. Cura: `toggle_brush_impasto` escreve nos
**três slots de relevo** — os três tools são UM assunto, e "estou trabalhando com corpo" não pode ser
verdade do pincel e falso da faca na mesma mão. Depois: `true → true → true`. Modos sem verbo na lista
ficam intocados (é o master do assunto, não um global).

**(3) O wrap dos botões era END-DEMOTION.** Screenshot do Enio: `Deposit | Knife | Smooth` na primeira
linha e **sete botões empilhados um por linha**, cada um esticado de borda a borda. A regra do widget
compartilhado era *"cabe o prefixo na primeira linha, cada sobra ganha uma linha inteira"* — e ela
**concorda com flow-wrap sempre que há 0 ou 1 sobra**, que é todo grupo de 2-4 opções do app. Por isso a
diferença ficou invisível até chegar uma lista de **dez**. Agora é flow greedy (`segmented_row_counts`),
e ⚠️ **paint e measure passaram a CHAMAR a mesma função** em vez de implementarem a regra duas vezes —
container medido por uma regra e preenchido por outra é exatamente como a próxima seção pinta por cima
dos botões e mata os hit targets. Gates: o pack (`[3,3,3,1]`, contra `1+9` da regra velha) + a
**equivalência dos grupos curtos**, que é o que torna seguro mexer num widget que 8 painéis usam.

⚠️ **Um oráculo meu nasceu errado:** o gate do pack exigia `n > 1` em toda linha após a primeira — o que
um flow CORRETO reprova sempre que o resto é um (10 a 3 por linha = `3+3+3+1`). O layout estava certo e o
gate estava errado; a afirmação virou *"toda linha menos a ÚLTIMA está cheia"*.

## §5c — A FACA virou modo próprio (`PaintMode::Knife`)

Enio: *"o modo Smear do Impasto (knife) deve ser único e não compartilhado com o smear dos outros tipos de
pintura já que ele afeta o Volume do impasto. Smear com botão no painel lateral é o smear dos outros modos
de pintura."*

Rodavam como **um** `PaintMode`, logo **um slot de `BrushSpec`**: o Plow que faz de uma faca uma faca
estava também no smear comum, e mexer num movia o outro. Agora `PaintMode::Knife` (slot 11), wire
`"knife"`, mesmo caminho de motor e **slot próprio** ⇒ Plow/Size/Spacing são dela.

⚠️ **Isto reabre um default de propósito.** Enquanto a faca ERA o Smear, `impasto_plow` passou a nascer
`1.0` (*"a faca leva a massa"*, 2026-07-18) — a medição continua válida, mas o racional é da **FACA**.
Separados: Knife nasce `1.0`, Smear do rail volta a `0.0` (arrasta cor, deixa o corpo onde está).

⚠️ **`PaintMode::smears()` é a porta única** — todo sítio que despacha o campo de smear pergunta a ela em
vez de testar `== Smear`. Uma *enumeração* daqueles sítios é o que apodrece quando um segundo membro entra
na família: o motor rodaria o warp para um e não para o outro, e a faca simplesmente não faria nada.

⚠️ **A Faca não tem botão no rail**, de propósito (o rail é do smear comum). Então `sync_from_mode("knife")`
deixa o rail com **nada** aceso — o artista segura uma ferramenta que aquela tira não oferece, e acender o
parente mais próximo seria o rail nomeando a ferramenta errada.

⚠️ **Achado: eu tinha criado uma segunda porta e não tinha visto.** Meu `paint_mode_wire()` duplicava o
**`active_paint_mode_id()` que já existia** em `stencil.rs` — a mesma pergunta, duas respostas, no mesmo
dia em que escrevi três comentários contra isso. Deletado; o shell usa a original, que ainda é melhor
(devolve `"eraser"`, que o rail tem).

E `impasto_section_applies` virou **Paint | Knife | Sculpt** — o Smear comum saiu da lista, porque não tem
verbo nela.

## §5d — O card Lighting hide com o Enable (reversão de uma isenção minha)

Enio: *"o card Lighting é próprio de Impasto. só deve aparecer se impasto estiver ativo."*

Na §5b(1) eu tinha **isentado** o Lighting do gate do Enable, com o raciocínio de que os controles de luz
são do CANVAS, não do pincel, então desligar o depósito não deveria tirá-los. O Enio decidiu o contrário:
o card Lighting é parte do assunto Impasto e some com ele. Agora o branch `!brush.impasto` retorna sem
pintar nada além do próprio Enable.

⚠️ **O passe de luz NÃO foi tocado.** `impasto_visible()` lê `impasto_show` + "existe relevo", **nunca**
`brush.impasto` — então o relevo já pintado **continua aceso** com o Enable off; só os CONTROLES da luz
somem até religar o Impasto. Esconder o card ≠ apagar a luz, e o motor mantém os dois separados. O gate
`enable_off_hides_the_whole_section_but_leaves_the_way_back_on` (ex-`…but_never_the_light`) inverteu a
afirmação e agora exige que `Show`/`Angle` sumam com o resto; a mutação que pinta Lighting no branch off
sangra. Os gates irmãos (`the_light_switch_is_reachable_from_every_mode_that_shapes_relief`,
`material_is_the_deposits_and_lighting_is_everyones`) testam com Enable ON e seguem válidos.

## §6 — Fora de escopo, nomeado em vez de contrabandeado

**O "Affect Relief" do Deform é uma QUARTA casa.** O Enio nomeou três; esta não entrou. O Deform é
mode-exclusive (early return em `paint_brush.rs:52`), então puxá-lo para cá exige reestruturar aquele
early return — decisão própria, não carona. Recomendação: se for unificar, o Deform vira a 11ª
ferramenta da lista e o corpo dele passa a hospedar a seção.

**O relevo por-camada** (slider + chip Add/Level em `paint_rows_relief.rs`) mora na aba **Layers** e lê
`Layer`, não `BrushSettings` — outra fonte de dados, outra aba. Fica.

## §7 — Smoke

1. **A queixa original:** abra o painel Brush, seção **Impasto**. Uma lista **TOOL** com dez chips logo
   abaixo de *Adjust Last Stroke*; abaixo dela, só os knobs da ferramenta escolhida.
2. **O bug que isto conserta:** escolha **Chisel**. O card **Lighting** continua na tela (antes sumia).
   Escolha **Knife**: idem. Mexa em Angle/Elevation e veja o relevo acender nos três.
3. **O rádio do rail segue:** ao escolher Chisel, o chip **SCULP** do rail esquerdo acende sozinho; ao
   escolher Deposit, volta para **Brush**. Nunca dois acesos.
4. **Sem knob morto:** no Deposit há Depth e não há Plow; na Faca há Plow e não há Depth; num verbo não
   há nenhum dos dois.
5. **Material:** presente no Deposit, ausente na Faca e nos verbos (é assado com o depósito).
6. **Item 1:** pinte um traço, mexa em Depth. O traço pronto **não muda** (antes mudava). Marque
   *Adjust Last Stroke* e mexa de novo: agora alcança.
7. **Regressão:** o card do Sculpt **não está mais no topo** do painel. Ele é a lista TOOL agora.

## §8 — Estado

`line/Painter`, commits desta jornada: `b29cfabb` (item 1) + o desta reorganização.
`cargo fmt` · clippy 0 · LOC caps verdes. **NÃO integrei, NÃO pushei** — Modo L, ordem explícita do Enio.
