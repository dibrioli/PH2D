# Auditoria do sistema de widgets — o que fechou e o que ficou (2026-08-09)

> Linha `line/Vector`, sobre `e71bad551`. Este documento existe porque o inventário vivia no
> scrollback de uma sessão, e **um achado que só existe numa janela de conversa deixa de existir
> quando a janela fecha**. Ele não é um handoff de integração: é a lista de trabalho da linha.
>
> **ESTADO: a lista está FECHADA.** Os dezoito itens saíram como **catorze correções**, **três
> itens que DISSOLVERAM na medição** (§2 A5 · A6 · o bloco D inteiro) e **um resíduo NOMEADO e
> não-fechado** (§D-resíduo). O que sobrevive à lista são as lições, não as pendências.

## §0 — A forma que todos os achados têm

Dezoito achados, um mecanismo: **um fato com duas cópias que discordam.**

O índice da opção contra a contagem de opções · o painel do popover contra as rows dele · a régua
do pintor contra a cópia que o dispatch faz dos números dela · a moldura do cartão contra a caixa
que lhe deram · o NOME de um gate contra o que o corpo dele mede.

⚠️ **E em quatro dos casos fechados a resposta certa já existia noutro lugar do repo** — o popover
que rola (três painéis) · o recorte do `TextInput` · o idioma do `expect` no `seam_authored_popover`.
A correção foi *entrar na fila que já existe*, não inventar mecanismo — vale procurar isso primeiro
em qualquer achado desta família.

⚠️ **E TRÊS itens deste inventário dissolveram na medição** — o A5 e o A6 porque eu classifiquei
pela FORMA em vez do mecanismo, e o **bloco D inteiro** (a varredura do `Xl3`) porque os treze
sítios medem NÃO-DEFEITO. *Um item de auditoria também é uma afirmação, e ele se mede antes de
virar trabalho.*

⚠️ **E as duas varreduras que dissolveram acharam o que a lista NÃO tinha:** a do "golden"
inexistente achou o **gate de staleness que o cabeçalho do gerado prometia e não existia**, e a do
`Xl3` achou a **`PANEL_HEADER_CLOSE_RESERVE` 2 px curta na fábrica**, pondo a faixa de arrasto por
cima do botão de fechar num painel real. *O valor de varrer um item falso é o que se tropeça no
caminho.*

---

## §1 — Fechados (dezanove commits)

| Commit | O achado |
|---|---|
| `520158282` | A marca de SWATCH sobrevivia à troca de tipo — a `rgba` de uma row que deixou de ser cor. |
| `df43a5c08` | Abrir o picker ESCREVIA o documento, e achatava um gradiente para a cor plana do chip. |
| `85f0c2f02` | A borda de cima da primeira row era pintada pela faixa de arraste, não pela row. |
| `4cd41855b` | A família de LISTA entrou no pino dos códigos duráveis (`WidgetKind::from_code`). |
| `aa68663b9` | O load esquecia a posição autorada dos controles. |
| `6900a8283` | Uma row que MORREU deixava o valor dela para a próxima de mesmo nome. |
| `9ba26966f` | O thumb do toggle saía do corpo, e ON podia ficar à esquerda de OFF. |
| `41a95074e` | **A opção marcada podia não existir** — índice fora da contagem; a reconciliação entrou no `populate::adopt`. |
| `82966f7c3` | A lista longa saía da tela em vez de rolar (a infra de popover rolável já existia, com 3 consumidores). |
| `101c98b2a` | A `TextArea` tinha DUAS réguas — o dispatch copiava os números do pintor, e o caret caía na linha errada. |
| `876055b9b` | A `TextArea` não recortava o próprio texto (o irmão de uma linha já recortava). |
| `b4e7c2764` | Nada do cartão sai da caixa que lhe foi dada (cabeça `Xl3` + rodapé dentro de um host variável). |
| `dd6278dec` | **O gate do topo do Chroma media `M/M`** — empurrava com `oklch_set_channel` e lia com `oklch_norm_channels`, inversas exactas ⇒ `1.0` para qualquer máximo, **incluindo `0.001`, que é o bug reportado**. |
| `d9017f9a8` | Os quatro gates de roteamento podiam passar VAZIOS (`else { return; }` sobre a tabela GERADA). |
| `8474398bc` | **`SkinParam.options`/`selected` sem gate, e o doc do `selected` FALSO em metade da família** — três comportamentos saíam de um campo (Tabs/Segmented clampam ⇒ marcam a ÚLTIMA; Radio/Dropdown no-opam ⇒ nenhuma). |
| `db5cd9615` | Um slot de largura FIXA dentro de um host VARIÁVEL (o chip de unidade · o `X` da tag · **a ORDEM do clamp** no `anchor_below`). |
| `4928fcb86` | Cinco docs que descreviam um produto que não existe (`set_live_rows` · `duplicate_keys` · `authored_option_id` · `PillGroup` · o doc órfão do `is_control`). |
| `8770f7cae` | **A faixa de arrasto do título comia 2 px do botão de FECHAR** — a reserva era uma CÓPIA de `PANEL_HEAD_PAD + Xl2` e nasceu 2 px curta na FÁBRICA; num dos dezasseis painéis o X arrastava em vez de fechar. |
| `deb7c39d4` | A varredura do item 18 — **duas classes com curas OPOSTAS**, e quatro docs cuja derivação nomeava o token errado (seguir o comentário encolheria toda seção do app). |

⚠️ **O `41a95074e` e o `d9017f9a8` são a mesma lição por dois lados:** o primeiro põe a
reconciliação na **porta por-quadro** (`populate::adopt`, onde o tipo, a marca de secção, a marca de
picker e a morte da row já são estabelecidos) em vez de em cada sítio que lê; o segundo tira um
gate da dependência de **conteúdo autorado**, que o Enio pode reautorar a qualquer momento.

---

## §1b — E DUAS defesas que a mutação provou INERTES

Escritas aqui porque a próxima LLM as vai encontrar e tomar por load-bearing.

- **`skin::marked_option`** — substituí-la por `Some(param.selected)` deixa os **29 gates da pele
  VERDES**: os quatro braços honram a lei por acidentes downstream *diferentes* (o pintor de abas
  compara `i == selected`; os dois `select` no-opam num valor desconhecido). O que de facto conserta
  o defeito é **não passar pelo construtor `Tabs::selected`**, que clampa. Ela fica porque sem ela a
  lei não está escrita em lugar nenhum.
- **O `.max(0.0)` na altura da seleção/caret do `text_input`** — o `push_clip` logo acima já apara o
  retângulo invertido, então um gate escrito contra a CENA não distingue as duas versões. **O gate
  que eu tinha escrito passava com o defeito reinstalado e foi REMOVIDO em vez de shipado.**
- **O recorte por célula do `key_value_list`** — mesma razão, e ela vale para **todo** recorte desta
  camada: a cena **codifica** os glifos independentemente do clip, que só age na rasterização. ⚠️ Ele
  é load-bearing no PRODUTO (sem ele uma chave de uma linha mais larga que a célula pinta sobre a
  coluna do valor) e só seria gateável com rasterização. *A metade que se vê no encoding — o texto
  ser UMA linha — é a que tem gate.*

---

## §2 — A lista, por ordem de quanto fazia o resto valer menos (TODA fechada)

### A. Gates que não podem falhar pelo motivo que alegam

O lote mais caro: enquanto vivem, **a suíte inteira vale menos do que parece**.

1. ~~`SkinParam.options` / `selected` sem gate~~ — **FECHADO** (`8474398bc`).
2. ~~`segment_hover_state_is_read_from_the_store`~~ — **FECHADO** (`e46fb49c9`). Ele checava
   hit-registration; medido, a leitura do store **inteiramente deletada** o deixava verde.
3. ~~`preferred_height_sums_entries`~~ — **FECHADO** (`e46fb49c9`). `h > chip_px * 5.0` sobre onze
   entradas: apagar **todos** os gaps o deixava verde.
4. ~~`field_rects_partition_host`~~ — **FECHADO** (`1e3ae8764`). Media ordem e largura positiva;
   deixar **um quarto do host vazio** o deixava verde.
5. ⛔ **`without_an_override_the_track_is_the_panel_law` — NÃO era achado.** Eu o listei como
   auto-referente por ele escrever a expectativa com a fórmula do produto. **Medido: trocar `0.25`
   por `0.40` no produto o derruba.** Ele é um **PIN** — a fórmula no teste é um literal reescrito
   à mão, e o propósito declarado dele é fazer a lei custar duas edições. ⚠️ A diferença para o
   gate do Chroma é exacta: *aquele empurrava e lia pela mesma FUNÇÃO* (inversas, logo `M/M`);
   *este restata a lei*. **A forma se parece; o mecanismo não.**
6. ⛔ **"A comparação golden omite `options`" — NÃO EXISTE comparação golden.** Eu li *"o golden"*
   no doc do `populate.rs` como um teste de arquivo-golden; ali a palavra nomeia o próprio
   `generated/panel.rs`. O `golden/` da crate do gerador está **vazio e nem versionado**.
   ⚠️ **Mas a varredura que provou isso achou coisa MAIOR, e ela FECHOU** (`1e3ae8764`): o
   cabeçalho do gerado prometia um gate de staleness que **não existia** — nada chamava `emit()`
   fora dos testes de unidade do próprio gerador.

### B. Docs que mentem

Um comentário que contradiz o código shipado é pior que comentário nenhum.

7. ~~`PillGroup` diz…~~ — **FECHADO** (`4928fcb86`).
8. ~~`duplicate_keys()` promete…~~ — **FECHADO** (`4928fcb86`).
9. ~~`authored_option_id` diz…~~ — **FECHADO** (`4928fcb86`).
10. ~~A afirmação de cadência do `set_live_rows`…~~ — **FECHADO** (`4928fcb86`).

### C. Geometria: um slot fixo dentro de um host variável

A família do `b4e7c2764`, que fechou só o cartão. Todas de baixo impacto e custo de uma linha —
mas cada uma é uma tinta que sai da moldura, e a moldura é o que o gizmo do canvas abraça.

11. ~~`numeric_input_with_unit::unit_rect`~~ — **FECHADO** (`db5cd9615`).
12. ~~`tag`: o X derrapa~~ — **FECHADO** (`db5cd9615`).
13. ~~`text_input`: seleção e caret~~ — **FECHADO** (`db5cd9615`).
14. ~~`popover::anchor_below`~~ — **FECHADO** (`db5cd9615`).
15. ~~`key_value_list` não recorta~~ — **FECHADO** (`042b6cb95`). ⚠️ E o **mecanismo não era o que
    eu escrevi**: o transbordo é VERTICAL, não horizontal — o `paint_text` **quebra** no
    `max_width`, e medido, uma chave de 53 chars numa row de 24 px pintava glifos de `y=11` a
    `y=55`, sobre as duas rows seguintes.
16. ~~`list_item` usa a largura MEDIDA do texto sem teto~~ — **FECHADO** (`042b6cb95`). Medido: 48
    caracteres numa row de 200 px começavam **337 px à esquerda dela**.

### D. Medições que faltavam — **AMBAS FEITAS** (`8770f7cae` + o commit dos docs)

17. ~~A varredura do `Xl3` não está completa~~ — **FEITA, e a tabela inteira dá NÃO-DEFEITO.**
    Treze usos em produto; três já curados (`numeric_input_with_unit` · `icon_button` · `card`) e
    os dez restantes caem em quatro categorias, cada uma medida:

    | Sítio | Veredito |
    |---|---|
    | `selection.rs:94` (`badge_w`) | host é `tag_w = 220` **constante** — folga 116 px |
    | `grid_snap/paint.rs:142` | é `reserve_right`, e o `paint_panel_title` já clampa `.max(0.0)` |
    | `color_swatch.rs:46` (`Md => Xl3`) | o token **É** o tamanho do objeto |
    | `pill_group.rs:49` | é o *default* de um builder cujo `preferred_width/height` **DERIVA** dele — a direção certa |
    | `modal.rs:165` · `color_picker.rs:154` | ⚠️ **galeria-only: ZERO chamadores de produto** |
    | `showcase/*` (3) | galeria |
    | painter-layers `header_h` (4) | header vertical em painel que **ROLA** |

    ⚠️ **O que a varredura ACHOU não estava na lista:** o defeito da família não era um `Xl3` — era
    a `PANEL_HEADER_CLOSE_RESERVE`, uma **cópia** de `PANEL_HEAD_PAD + Spacing::Xl2` que nasceu
    **2 px curta na escala de fábrica** e punha a faixa de arrasto por cima do botão de fechar num
    dos dezasseis painéis. Fechado em `8770f7cae`.

18. ~~`Spacing::px()` devolve o valor AUTORADO~~ — **VARRIDO, e a varredura tem uma FRONTEIRA
    exata:** o `NumToken` cobre **três** famílias (`Spacing` · `Radius` · `Stroke`); a
    **tipografia é `const fn`** e não é autorável, então todo literal derivado de `TypeToken` é
    seguro por construção (é o caso do `PANEL_HEADER_H_DEFAULT`, cujo *"Lg.px(~18)"* é o
    `TypeToken::Lg` e está **certo**). E `is_a_length` é só `finito && >= 0` — **não há teto**,
    então onde a cópia existe ela é errada ao vivo por qualquer margem.

    ⚠️ **A varredura devolveu DUAS classes com curas OPOSTAS, e eu quase escrevi a regra errada**
    (*"troque o literal pelo token"*), que teria mudado o produto em metade dos casos:

    - **Classe A — o comentário nomeia o token ERRADO.** O número está certo, o doc mente, e a
      mentira convida à mudança: `SECTION_LABEL_TO_CONTROL_PX = 4.0` dizia `Xxs` (que vale **2**)
      e `SECTION_INNER_ROW_GAP_PX = 8.0` dizia `Sm` (que vale **6**) — seguir os comentários
      encolheria o gap de toda seção do app. Mais o doc do botão de fechar (*"32×32 com glifo de
      16×16"*, sempre foi `Xl2` = 24) e o `TOPBAR_INTER_CHIP_GAP` (*"`Spacing::Xxs` equivalent"*
      numa barra de topo **inteiramente literal-escalada** — 44/100/156: coincidência, não
      derivação). **Cura = corrigir o DOC.**
    - **Classe B — a derivação é real e a cópia deriva.** Um caso: a `PANEL_HEADER_CLOSE_RESERVE`.
      **Cura = a PORTA pergunta** (não a constante virar função: são 21 crates de painel).

    ⚠️ **E a `PANEL_HEADER_ADD_RESERVE` era uma terceira coisa** — *"2 × icon size + gap"* não
    fecha com número nenhum do produto; medido, ela sobra **32 px** sobre o Add da Hierarquia
    (um literal de 30). O número fica, a fórmula inventada sai.

### D-resíduo — o que a varredura NOMEIA e não fecha

O app mistura dimensões de chrome **literais** com dimensões de **token**, e desde a wave de UI/UX
a escala de token é autorável **sem teto**. Isso cria uma classe de descasamento latente que
**nenhuma correção pontual endereça** — um `Spacing::Xl3` autorado em 100 estoura o rodapé de 56
de um modal, o header de 56 de um painel, e assim por diante. Fica **nomeado**: a decisão é de
produto (limitar a faixa autorável? tornar o chrome derivado?), não uma dívida de engenharia com
cura óbvia.

---

## §3 — O que não é achado

- **Flakes de CARGA, não de código.** `the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` e
  `a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget`
  (`ph2d-host-desktop::flip_smooth`) e `the_cost_of_depth_is_linear_not_explosive`
  (`ph2d-timeline`, já registada no CLAUDE.md §5) falharam durante as varreduras com **load 52,94**
  e passam isoladas; `git diff HEAD` naquelas crates é vazio. A regra do repo: *nenhum smoke desta
  máquina significa nada com o load acima de ~5*. Varredura final desta jornada: **8544/8544** a
  load 27, e **8520/8520** a load 7,09.

## §4 — Dois erros de processo meus, registados porque a forma se repete

Os dois têm a mesma forma: **uma ferramenta minha falhou em silêncio e o resultado parecia bom.**
Os dois só foram apanhados porque havia um `assert` no caminho.

1. **As mutações do cartão não foram aplicadas.** Uma função de shell chamava
   `python3 -c "…"` sem encaminhar `$1`/`$2` ⇒ `IndexError` em `sys.argv`, **zero mutações**, três
   corridas verdes sobre o produto correto. Eu teria reportado *"3 mutações, 3 sangram"* sobre nada.
   Denunciado pelo `Traceback` a imprimir ANTES do `echo`.
2. **A regex do `event_tests` não casou.** Esperava `Some(a, b)` onde o código tem `Some((a, b))`.
   O `assert n == 4` disparou **antes** do `write_text` ⇒ o ficheiro ficou intacto e a suíte que
   correu a seguir era a original. Foi a asserção de CONTAGEM que impediu um commit vazio.

⇒ **Toda edição em massa leva uma asserção de contagem, e ela corre antes da escrita.**
