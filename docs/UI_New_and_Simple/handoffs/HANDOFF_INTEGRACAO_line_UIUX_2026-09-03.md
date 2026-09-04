# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-09-03

> **Para o agente integrador.** DIRETRIZ §1.5.9. A linha está fechada; **não integra nem shipa sem
> ordem explícita do Enio** (§0.7).

---

## §0 — ⛔⛔ LEIA ISTO ANTES DE TUDO: a UI nova entra DESLIGADA

Ordem do Enio, 2026-09-03, ao mandar escrever este handoff:

> *«essa nova UI ainda deve ficar desativada até que esteja concluída. Por enquanto permanece a
> antiga.»*

⇒ **o caminho de omissão do `main` é a UI de sempre.** O redesenho liga-se com **`PH2D_UI_NEW=1`**,
como o `PH2D_FLIP_NEW_ENGINE` e o `PH2D_RETOPO_EXTRACT` fazem nos módulos deles.

| | |
|---|---|
| a bandeira | `ph2d_tokens::UiLook` — `Classic` (omissão) · `Redesign` |
| publicada por | `paint::set_ui_look`, uma vez por quadro, a partir de `PH2D_UI_NEW` (`OnceLock`) |
| lida por | os **três** pintores que o redesenho mudou: a linha de propriedade, a caixa de verificação, o interruptor |
| gate | `by_default_the_app_wears_the_old_ui` |

⚠️ **O gate mede TINTA, não a bandeira.** Afirmar `UiLook::default() == Classic` seria uma tautologia
sobre uma linha de código; ele pinta os três widgets nas duas aparências, exige que **difiram**, e
exige que o que sai **sem ninguém escolher** seja byte a byte o clássico. ⛔ Sem a metade do
«difiram», o gate ficaria verde no dia em que alguém apagasse o clássico.

⚠️ **`PH2D_UI_NEW` só liga com `1`.** Vazio, `0`, `true`, `yes`, ` 1` → clássico. *Um interruptor que
liga com o que não percebe é um interruptor que se liga sozinho* — e este governa se o `main` mostra
um redesenho a meio.

### §0.1 — O que NÃO está atrás da bandeira, e porquê

⛔ As **correcções** ao caminho antigo ficam ligadas para toda a gente. *Um defeito curado não é uma
aparência nova, e escondê-lo atrás de um interruptor é deixá-lo por curar para quem não o liga.*

| correcção | porquê fica |
|---|---|
| a **deriva do cursor** (§14 da pesquisa) | o rect registado é o denominador do valor; o defeito existia no caminho antigo e a cura é a mesma lei |
| a **roda do rato** sobre a bancada | o painel publicava polegar de rolagem e a roda dava zoom na câmara |
| o **arrastar do corpo** de um painel para o rolar | é um GESTO que faltava, não uma aparência; num tablet os painéis eram não-roláveis |
| as **pílulas apagadas** | zero consumidores em todo o repo — não havia o que preservar |
| o **Widget Lab** | painel de estudo, atrás do menu *Window*; não muda a cara de nada |

---

## §1 — O que a linha entrega

**81 commits**, `268` ficheiros de código (`+20 576 / −3 008`). O registo com mecanismo é a pesquisa
[`07_o_redesenho_dos_widgets.md`](../pesquisa/07_o_redesenho_dos_widgets.md), §1–§21.

### 1.1 — O redesenho (atrás de `PH2D_UI_NEW=1`)

| obra | o que é | onde |
|---|---|---|
| **a caixa única** | uma linha de propriedade deixa de ser `rótulo \| trilha \| caixa` (`154 px` de cromo fixo) e passa a ser **uma caixa**: rótulo dentro à esquerda, valor dentro à direita, preenchimento a dizer a fracção | `widget/property_box.rs` |
| **os quatro desenhos** | `Underline` (o escolhido) · `Bar` · `Inset` · `Ghost`, mais raio e altura como preferências | `ph2d-tokens::SliderStyle` |
| **a caixa de verificação** | a **linha inteira** é o alvo e a marca vai à **direita**, na coluna do número; rótulo a `12 px` como as outras | `widget/checkbox/mark.rs` |
| **o interruptor** | funde-se na marca da caixa — **de pintura**, nunca de modelo | `widget/toggle.rs` |
| **a coluna de animação** | um ponto por linha, na margem; **indicador, não controlo** | `property_box::form_row_columns` |
| **a bancada** | `ph2d-panel-widget-lab` — os quatro desenhos, a régua de largura, os estados, as cores, e o widget antigo lado a lado | crate nova |

### 1.2 — As correcções (ligadas sempre)

- a **deriva do cursor** — o rect registado no `HitIndex` é também o **denominador** do valor;
- a **rolagem por arrasto** no corpo de um painel — `BodyScrollAnchor`, com a guarda `hit.is_none()`;
- a **roda** sobre a bancada;
- três `seam_*` do Vector que **clicavam** onde diziam **arrastar**.

### 1.3 — ⛔ O que foi construído e NÃO shipa

| | veredito |
|---|---|
| **barra de rolagem a `2 px`** | ⛔ **recusada por medição**: a cerca do `SCROLLBAR_W = 10` são as palavras do dono (*alvo táctil no iPad*), e **num tablet não há hover**. Ao verificá-la apareceu o buraco a sério — os painéis não tinham arrasto nenhum. |
| **4.º modo de fonte** (`font_embolden` do Vello 0.10) | ⛔ **construído, smokado pelo dono e REVERTIDO**: *«o cursor ficou lento e não se pode abrir nada mais. não vi diferença na font»*. Mecanismo na §18: o efeito reescreve a **geometria** de cada glifo e paga-se **por quadro**; nenhum cache o salva. |
| **os desenhos `Split` e `Stack`** | ⛔ controlos negativos do estudo — voltam a ter coluna fixa. |

---

## §2 — ⚠️ Superfície de colisão

| ficheiro | risco |
|---|---|
| `crates/ph2d-editor-core/src/widget/` | **12 ficheiros** tocados + 4 novos (`property_box.rs`, `checkbox/`, `slider_with_chip/`, `toggle_classic.rs`). ⚠️ O `checkbox.rs` e o `slider_with_chip.rs` **viraram pastas** por tecto de LOC — um merge textual sobre eles falha; é preciso re-aplicar por conteúdo. |
| `crates/ph2d-editor-core/src/paint.rs` | três thread-locais novos vizinhos (`SLIDER_STYLE`, `UI_LOOK`) |
| `crates/ph2d-editor-core/src/interaction/` | `drag.rs` (+`BodyScrollAnchor`), `state/mod.rs` (+1 campo), `state/panel_ops.rs`, os três braços de despacho |
| `crates/ph2d-editor-core/src/screens/hero/` | `paint.rs` (publica a aparência; `PANEL_Z_ORDER_FALLBACK` +1), `topbar/`, `ids/` |
| `crates/ph2d-panel-inspector/src/sections/` | `transform.rs` partido em dois; `ordering.rs`, `rows.rs`, `sampling.rs`, `anchors.rs` tocados |
| `shells/desktop/` | `Cargo.toml` (+`panel-widget-lab`), `forwarding.rs` (+`LAB_PANEL`) |
| `crates/ph2d-tokens/` | `slider_style.rs` (novo: `SliderStyle`, `UiLook`), `typography.rs` (+`TextRendering::ALL`) |

⚠️ **Números que somam entre linhas:** um `NodeId` de scrollbar (`LAB_SCROLLBAR_ID = 844`) e os ids
do painel novo. Se outra linha acrescentou ids, **conte**, não escolha.

---

## §3 — Gates novos (o que eles defendem)

| gate | afirma |
|---|---|
| `by_default_the_app_wears_the_old_ui` | ⛔ **a condição de integração**: sem escolher aparência, sai o clássico |
| `the_fill_lands_under_the_cursor` | a tinta acaba debaixo do dedo, **com controlo** que reproduz o registo estreito |
| `the_form_has_one_right_margin` | a marca segue a borda direita; o interruptor pinta a **mesma** marca; a coluna é desenhada |
| `the_animation_column_has_one_x` | as três famílias de linha põem o ponto no **mesmo `x`** |
| `every_form_row_reserves_the_animation_column` | ⭐ **catraca com censo de obsolescência**: 11 secções em dívida, e quem já reserva **sai** da lista |
| `a_panel_scrolls_by_dragging_its_body` | o gesto novo, **com controlo** de que ele nunca rouba uma pressão que um widget reclamou |
| `the_app_default_slider_style_is_the_one_the_owner_chose` | Underline / raio 4 / linha 22, por nome **e** por px |
| `the_inspector_is_open_when_the_app_opens` | pina uma decisão que só existia como um `true` numa tabela |
| `every_panel_the_registry_ships_reaches_the_binary` | ⭐ o painel novo estava **fora do binário** e o menu continuava a pintá-lo |

---

## §4 — ⏳ O que fica ABERTO (a razão de a bandeira existir)

1. **A coluna de animação em 11 secções** do Inspector — nomeadas na dívida do gate. Cada uma é
   uma edição de duas linhas pela porta `form_row_columns`.
2. **O ritmo das linhas** — as de marcar medem `18 px`, as de propriedade `22`. Um formulário
   alterna alturas. ⚠️ **Decisão do Enio pendente**: igualar é mais coerente e mais **alto**, e
   altura é o recurso escasso num tablet.
3. **O que o estudo §5.3 propôs e ninguém tocou** — e é o que mais muda a *cara*: os cantos dos
   painéis ainda são **16 px** (o estudo diz 4), as secções **não recolhem**, os cartões ainda
   desenham moldura, as caixas de texto têm moldura permanente, etiquetas e amostras de cor ainda
   são pílulas (`radius: 999`).
4. **O esbatimento do rótulo** (`push_luminance_mask_layer`) em vez de `…` — nomeado, por medir.
5. **A inércia** da rolagem por arrasto — o conteúdo pára com o dedo. É sensação; mede-se com a mão.

---

## §5 — ⚠️ Leis que esta linha pagou (leia antes de mexer aqui)

1. ⭐⭐⭐ **Um rect no `HitIndex` é também o DENOMINADOR.** Registá-lo mais estreito do que o que se
   pinta **não recorta: ESCALA** (`w/(w−84)` = 1,62×). Excluir uma sub-zona faz-se por **ordem de
   registo** — o índice resolve em `rev()` —, nunca por largura.
2. ⭐⭐ **Um censo por PINTOR não vê as linhas que ninguém pinta por um pintor.** Foi assim que o
   Transform ficou sem a coluna: ele é construído à mão dentro do painel.
3. ⭐⭐ **Um censo por NOME conta homónimos.** `paint_toggle` dava 29 sítios; os consumidores do
   widget eram **três** — os outros são funções locais que desenham outra coisa.
4. ⭐⭐ **Uma isenção de gate pode nomear um consumidor que não existe.** A da `pill_group` dizia
   *«o topbar pinta-as a cada quadro»* e o topbar nunca a chamou. Ninguém confere o **texto** de um
   opt-out.
5. ⭐⭐ **Verificar uma cerca pode revelar que o gesto que ela protege não existe.** A do
   `SCROLLBAR_W` levou ao achado de que um painel não se rolava com o dedo.
6. ⭐ **Uma contagem literal num gate faz cada feature nova editar o teste de alguém** — o
   `text_rendering_cycles_three_states` partiu-se só por existir um 4.º preset.
7. ⭐ **Um gate que fixa uma constante do produto mede a versão dele que já não corre** — aconteceu
   duas vezes no mesmo dia (a superfície da caixa, o ciclo dos presets).
8. ⭐ **Um parâmetro com dois papéis torna a chamada errada defensável** — derivar a coluna de
   `box_px.is_none()` partiu o contrato daquele campo.
9. ⭐ **Um efeito que reescreve a GEOMETRIA de cada glifo paga-se por quadro**, e nenhum cache o
   salva.
10. ⭐ **Uma closure grande esconde o modelo que os seus capturados formam** — catorze capturas do
    Transform eram *a geometria da secção*.

---

## §6 — Como smokar

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && cargo run -p ph2d-host-desktop --release
```

- **sem nada** → a UI de sempre. É isto que o `main` passa a mostrar.
- **com `env PH2D_UI_NEW=1`** → o redesenho: a caixa única, a marca à direita, a coluna de animação.
- `Window › Widget Lab` → a bancada, sempre no redesenho (é o estudo).

---

## §7 — Estado da árvore

- Gate de fecho: workspace verde, `clippy --all-targets` limpo nas crates tocadas, `fmt` limpo.
- ⚠️ **Flakes de carga conhecidas** que aparecem sob fan-out e passam sozinhas — as duas são da
  lista do `CLAUDE.md` §5.0: `the_cost_of_depth_is_linear_not_explosive` (Timeline) e a família
  `flip_smooth::resample_measurement::precisao::orcamento`.
- ⛔ **Nenhum push, nenhuma integração.** A linha fecha aqui.
