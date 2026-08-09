# Auditoria do sistema de widgets — o que fechou e o que ficou (2026-08-09)

> Linha `line/Vector`, sobre `e71bad551`. Este documento existe porque o inventário vivia no
> scrollback de uma sessão, e **um achado que só existe numa janela de conversa deixa de existir
> quando a janela fecha**. Ele não é um handoff de integração: é a lista de trabalho da linha.

## §0 — A forma que todos os achados têm

Dezoito achados, um mecanismo: **um fato com duas cópias que discordam.**

O índice da opção contra a contagem de opções · o painel do popover contra as rows dele · a régua
do pintor contra a cópia que o dispatch faz dos números dela · a moldura do cartão contra a caixa
que lhe deram · o NOME de um gate contra o que o corpo dele mede.

⚠️ **E em quatro dos casos fechados a resposta certa já existia noutro lugar do repo** — o popover
que rola (três painéis) · o recorte do `TextInput` · o idioma do `expect` no `seam_authored_popover`.
A correção foi *entrar na fila que já existe*, não inventar mecanismo. Vale procurar isso primeiro
em cada item aberto abaixo.

---

## §1 — Fechados (catorze commits)

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

⚠️ **O `41a95074e` e o `d9017f9a8` são a mesma lição por dois lados:** o primeiro põe a
reconciliação na **porta por-quadro** (`populate::adopt`, onde o tipo, a marca de secção, a marca de
picker e a morte da row já são estabelecidos) em vez de em cada sítio que lê; o segundo tira um
gate da dependência de **conteúdo autorado**, que o Enio pode reautorar a qualquer momento.

---

## §2 — Abertos, por ordem de quanto fazem o resto valer menos

### A. Gates que não podem falhar pelo motivo que alegam

O lote mais caro: enquanto vivem, **a suíte inteira vale menos do que parece**.

1. **`SkinParam.options` / `SkinParam.selected` — canal público, ZERO gates.**
   `crates/ph2d-editor-core/src/widget/skin.rs:141,143` · gates em `skin/param_tests.rs`.
   Os dois canais gateados (`rgba`, `icon`) têm cada um um **par** — *chega ao braço que o declara*
   e *é inerte em todos os outros*. A família de LISTA não tem nem um nem outro, e é consumida por
   quatro braços (`Tabs`, `RadioGroup`, `SegmentedAdaptive`, `Dropdown`).
   ⚠️ **E o doc do campo promete uma lei que três dos quatro não honram:** ele diz *"Fora do alcance
   ⇒ nenhuma"*, mas `Tabs::selected` **CLAMPA** (`tabs.rs:63`, `idx.min(len-1)`) ⇒ marca a ÚLTIMA;
   `RadioGroup::select` (`radio_group.rs:85`) e `Dropdown::select` (`dropdown/mod.rs:139`) **no-opam
   em silêncio** ⇒ não marcam nenhuma. Três comportamentos, um campo.
   *Alcançável hoje?* Não pelos dois consumidores atuais (o canvas passa `selected: 0` fixo,
   `widget_live.rs:200`; o painel clampa desde o `41a95074e`). **É o próximo consumidor que herda a
   promessa falsa** — e o canal existe precisamente para que consumidores novos entrem barato.

2. **`segment_hover_state_is_read_from_the_store`** — `widget/segmented_adaptive.rs:299`.
3. **`preferred_height_sums_entries`** — `widget/tool_rail/tests.rs:24`.
4. **`field_rects_partition_host`** — `widget/vector3_editor.rs:195` (e o irmão
   `field_rects_partition_host_left_to_right`, `rect2_editor.rs:334`).
5. **`without_an_override_the_track_is_the_panel_law`** — `widget/slider.rs:229`.
6. **A comparação golden omite `options`** — `panel-authored/src/populate.rs:205` nomeia o
   mecanismo (*"era invisível porque as chaves COINCIDIAM: o golden foi gerado da mesma cena do
   smoke"*); a comparação do golden compilado não confere a lista de opções.

### B. Docs que mentem

Um comentário que contradiz o código shipado é pior que comentário nenhum.

7. **`PillGroup` diz *"Used 5x in the editor TopBar"* e tem ZERO chamadores.**
   `widget/pill_group.rs` — medido: só `mod`/`pub use` em `widget/mod.rs:37,101` e uma menção de
   token em `ph2d-tokens/src/chrome.rs:63`. Nenhum painel, nenhuma shell.
8. **`duplicate_keys()` promete um readout que o painel não pinta.**
   `panel-authored/src/rows.rs:374` — o doc diz *"o readout que o painel mostra em vez de
   desempatar"* e *"o painel **diz** que há repetidos"*. Único chamador: `rows_tests.rs:141`.
   Ou o painel passa a dizer, ou o doc para de prometer. **Repetidos partilham id**, então o defeito
   que ele descreve (duas rows como um controle) é real e invisível.
9. **`authored_option_id` diz que só o dropdown precisa dela.**
   `ids/chrome/authored.rs` (doc do `authored_option_id`): *"Só quem esconde as opções precisa dela
   … nas abas, no rádio e na segmentada quem regista os segmentos é o pintor do catálogo"*. Falso —
   `panel-authored/src/paint.rs:296` regista a família INLINE por `inline_option_rect` +
   `authored_option_id`, e é assim que uma aba fica clicável no painel compilado.
10. **A afirmação de cadência do `set_live_rows`** — a nota diz *"por quadro"* onde a fixture e o
    produto discordam sobre quando a tabela viva é publicada e devolvida.

### C. Geometria: um slot fixo dentro de um host variável

A família do `b4e7c2764`, que fechou só o cartão. Todas de baixo impacto e custo de uma linha —
mas cada uma é uma tinta que sai da moldura, e a moldura é o que o gizmo do canvas abraça.

11. **`numeric_input_with_unit::unit_rect`** — `widget/numeric_input_with_unit.rs:99,112`.
    `chip_w` é `Spacing::Xl3` (32) e `unit_rect` é `host.x + host.w - chip`: num host mais estreito
    que 32 o chip começa **à esquerda do host**. O irmão `input_rect` já tem `.max(0.0)`.
12. **`tag`: o X derrapa para a esquerda** — `widget/tag.rs:101`
    (`host.x + host.w - pad_x - close_size`, sem piso). O rótulo ao lado já tem `.max(0.0)`.
13. **`text_input`: seleção e caret com altura NEGATIVA** — `widget/text_input.rs:201` e `:236-240`
    (`rect.h - pad_y * 2.0`, sem `.max(0.0)`).
14. **`popover::anchor_below` pode devolver `x` fora do viewport** — `widget/popover.rs:66`:
    `center_x.max(viewport.x).min(viewport.x + viewport.w - w)`. Com `w > viewport.w` o `.min()`
    vence e a superfície começa à esquerda da tela. **A ordem do clamp é a lei**, e aqui ela está
    invertida em relação ao `popover_rect_clamped` que o dropdown usa.
15. **`key_value_list` não recorta** — `widget/key_value_list.rs`, zero `push_clip`. A mesma
    correção do `876055b9b` (`TextArea`), um widget adiante.
16. **`list_item` usa a largura MEDIDA do texto sem teto** — o rótulo empurra o resto para fora da
    row em vez de ser cortado.

### D. Medições que faltam

17. **A varredura do `Xl3` não está completa.** Os usos em `color_swatch.rs:46`,
    `color_picker.rs:154`, `showcase/*` e `modal.rs:19` **não foram medidos** — alguns são
    tamanhos naturais legítimos (o token *é* o tamanho do objeto), outros podem ser o defeito do
    item 11. O `icon_button.rs:94` **já foi curado** e é o precedente escrito: *clamp, e não um
    número novo*.
18. **`Spacing::px()` devolve o valor AUTORADO** (`num_runtime::live` → `num_overrides`), não a
    constante de fábrica — a escala de token virou editável na wave de UI/UX. ⚠️ Toda cópia de um
    token como literal é **errada ao vivo**, não meramente latente. Não há varredura que prove que
    não existem outras.

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
