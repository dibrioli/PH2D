# HANDOFF — TOP-20 #20, o HUD (`line/components`, 2026-09-17)

> **Ordem do dono:** *«escolha um e implemente»*, sobre os três abertos que restavam da lista.
> Escolhido o **#20 — placar, vida e menu na tela**, que é o que fecha o ciclo de uma demo jogável.

## §1 — O que a linha entrega

Um objecto pode agora carregar um **HUD**: uma raiz que se cola à vista da câmera do jogo, rótulos
cujo número o jogo muda, e botões que publicam um sinal para a tabela do #5.

| peça | o que é |
|---|---|
| [`ph2d-hud`](../../../crates/ph2d-hud/) | a crate-folha com a LEI: onde a caixa de referência cai na vista (`Keep` · `Stretch`), como um número vira texto, e quando um botão dispara |
| `UiCanvas` · `UiLabel` · `UiButton` · `Counter` | os componentes, no `ph2d-ecs` |
| [`hud_bridge`](../../../crates/ph2d-app-components/src/hud_bridge.rs) + `fase_hud` | a pose conduzida por quadro, pelo LEDGER |
| [`hud_label_live`](../../../shells/desktop/src/hud_label_live.rs) | o **10.º produtor** de `LiveGeometry` — os glyphs do número derivado |
| [`despacho_clique_hud`](../../../shells/desktop/src/input_dispatch/despacho_clique_hud.rs) | o clique que publica, só durante a corrida |
| secção **HUD** do Inspector + `catalog/hud.rs` | o que faz o HUD ser **autorável** e não só demonstrável |
| `PH2D_HUD_SMOKE=1` | a cena: o mundo rola, o HUD não |

## §2 — O ORÁCULO, e as TRÊS cegueiras que a sonda teve primeiro

Sonda versionada: [`godot_hud_probe.gd`](../ferramentas/godot_hud_probe.gd) (Godot 4.7.2, MIT,
`--headless`). ⛔⛔ **A 1.ª redacção mediu o NADA em dois dos três blocos, e foram os CONTROLOS que
o disseram** — as três estão no cabeçalho dela:

1. **L1** — o irmão no mundo lia a mesma posição com a câmera em três sítios ⇒ *um controlo que não
   reproduz o fenómeno torna o teste inteiro vácuo*;
2. **L2** — a escala vinha idêntica para cinco tamanhos de janela: `Window.size` não pega numa
   janela fantasma ⇒ **a experiência inverte-se** (janela fixa, referência variável);
3. **L3** — *«carregar dentro e largar fora»* **disparava**, porque faltava um MOUSE MOTION: o
   controlo do alvo nunca soube que o cursor tinha saído. *Um evento em falta mede outro programa.*

⛔ **Divergência DECLARADA:** o alvo arredonda a banda do letterbox a pixel inteiro e encolhe a
escala para caber (`keep`, ref `1280×360`: `0,561111`/`124` contra `0,5625`/`123,75`). O nosso
canvas vive em **metros** — fica o exacto, e há gate a **exigir que a divergência exista**.
⛔ **O `expand` não é portado**, e a razão é geométrica: nele os filhos ancorados chegam às bordas
REAIS, e quem ancora nesta casa mede contra a caixa LOCAL da moldura — fazê-los chegar exigiria
redimensionar a moldura por quadro, que é escrever no DOCUMENTO.

## §3 — Os números que se CONTAM (delta contra o `main` de que a linha nasceu: `3090cac3f`)

| contador | delta | onde |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** (`144 → 145`, degrau escrito) | `project_schema.rs` + a tripla |
| registo do `ph2d-ecs` | **+4** (`91 → 95`) | `scene/registry.rs` |
| os DOIS espelhos | **+4** cada (`92 → 96`) | `ph2d-render` · `ph2d-script` |
| `SignalVerb::ALL` | **+1** (`7 → 8`, APENDADO) | `signal_actions.rs` |
| `LIVE_SECTIONS` | **+1** (`28 → 29`) | `ids/live_sections.rs` |
| `any_live_section` | **+1** (`22 → 23`) | `paint_frame.rs` |
| `Driver`/`Driven` · `SignalOrigin` · `EditorAction` | **+1** cada, append-only | — |

⛔ **Zero contrato congelado** (§6): `Tool=12`, `NodeOp` e a superfície do vector-doc intocados.

## §4 — SETE coisas que uma leitura rápida do diff entende ao contrário

1. **O `CounterRuntime` não estar registado não é um esquecimento — é a wave.** Se ele fosse
   documento, **cada ponto marcado** seria um passo de `Ctrl+Z` e entraria no ficheiro.
2. **A pose do canvas ser reescrita por quadro não suja o undo:** ela passa pelo ledger do
   `preview_drive`, como a do solver e a do script.
3. **O `hud_label_live` não edita o documento.** Ele publica na `LiveGeometry` — *o passe publica o
   que o rótulo MOSTRA; ele não escreve o que ele É* (ADR-0153, um nível acima).
4. **Não há lei de âncora nova, e isso é o achado da §5.0:** o passe de âncoras que já existe
   **nomeia o HUD como o caso de uso dele** por escrito. Construir uma segunda teria sido a forma
   mais cara de ignorar a medição.
5. **O ramo do clique devolver `false` num `Baixo` fora de botão é load-bearing:** devolver `true`
   ali comeria todo clique de canvas durante uma corrida.
6. **O `publica` receber `&mut SimWorld` não é descuido:** o que um rótulo mostra é derivado pela
   MESMA porta que o desenho usa, e a consulta do bevy pede `&mut World`.
7. **Uma edição de um bloco ausente devolver `false` é a lei, não um erro:** um clique a meio de uma
   troca de selecção pode aterrar depois de o componente sair.

## §5 — As premissas que a MEDIÇÃO derrubou (e a foto foi quem mediu)

| eu supunha | a medição |
|---|---|
| a `origin` autorada põe o texto no sítio | o cozimento **RE-CENTRA** o caminho; a pose mora no `Transform` |
| o compound cozido entra na `LiveGeometry` | ele nasce de um `default()` — sem id, sem opacidade, sem efeitos — e a entrada viva **substitui** o documento ⇒ `replace_cooked` |
| a `LiveGeometry` está em espaço local | está em **MUNDO**: todos os produtores assam o afim (`bake_xform`) |
| pendurar o `UiLabel` chega | sem `VecShape::Text` o texto vivo **nunca corre** |
| `TimerState::default()` arranca | ele nasce **PARADO** — o doc do `timer::born` di-lo por escrito |
| o placar reage porque tem `SignalActions` | a identidade só é dada a quem tem `Transform` **ou** `ChildOf`, e o `resolve` colhe reactores com `&StableId` ⇒ **um reactor sem pose é invisível, em silêncio** |
| o nome do campo cabe no `placeholder` | um placeholder desaparece quando o campo tem valor — que é quando o nome faz falta |
| o botão é alcançável porque o corpo dele é grande | o dedo aterra no **RÓTULO** — o hit-test de objecto entrega *a forma mais ao topo que contém o ponto*, e o `+10` é desenhado por cima. Sem a subida da cadeia, o alvo maior da tela é o único inalcançável **no meio dele** |

## §6 — Gates e provas

* `ph2d-hud`: 9 gates (a tabela do oráculo **é** o gate) · **6/6 mutações sangram**
  ([`mutacao_hud_w1.sh`](../ferramentas/mutacao_hud_w1.sh), com controlo sobre o próprio filtro).
* `ph2d-ecs`: 9 + o do placar, este com **CONTROLO NEGATIVO** (sem pose ⇒ zero efeitos).
* `ph2d-app-components`: 6 da ponte + 7 do instantâneo/dreno.
* `ph2d-editor-core`: 4 do vocabulário.
* ⚠️ **O clique do botão prova-se na LEI**, não na foto: eventos sintéticos não chegam à Xwayland
  virtual e o `ydotool` move o rato REAL do dono (proibido). ⭐ O que a CENA prova de si mesma é a
  outra metade — *que a forma sob o dedo resolve para o botão* —, e ela leu `NAO` até a lei existir.
* ⛔ **O `hud_label_live` é o 10.º produtor de `LiveGeometry`, logo a política das QUINAS teve de o
  julgar** (`corner_handles.rs`): **recusado**, e já por construção — aquele passe só olha entidades
  cujo `VecShape` é `Text`, e o primeiro braço de `has_derived_verts` cobre isso. ⚠️ Mas *«já é
  verdade»* e *«é afirmado»* são coisas diferentes: o gate tem **duas** metades, e a segunda lê o
  SELECTOR do host, senão o dia em que alguém desenhar um rótulo sem `VecShape` faz a recusa
  evaporar-se em silêncio — que é exactamente como esta política falhou por quatro objectos.

## §7 — ABERTO, e de quem é cada um

* ⏳ **As quatro âncoras do `VecAnchors` não estão ligadas ao canvas** — elas medem contra a caixa
  LOCAL de uma moldura, e a raiz do HUD é conduzida por ESCALA. Ligar as duas é uma wave (e o
  `expand` do oráculo depende dela).
* ⏳ **No EDITOR, um HUD colado às bordas cai atrás dos painéis**: a vista da câmera é a da JANELA e
  o editor pinta num sub-rectângulo. Numa janela de jogo é exacto. **Decisão de produto.**
* ⏳ O `UiButton` só é alcançável por um caminho **vectorial** (`path_at`): um botão feito de
  *sprite* não é pego. Nomeado, não construído.
* ✅ **FECHOU, e a auto-conferência achou um DEFEITO a sério — não era o gate que faltava, era a
  lei.** A cena passou a perguntar-se a si mesma se o dedo alcança o botão (`hud_smoke_confere_o_dedo`,
  no último quadro da subida, porque a pose do canvas é conduzida), e ela leu **`NAO`**. ⚠️ **O
  veredito sozinho manda procurar em três camadas** (a pose, o afim, a elegibilidade) ⇒ as colunas,
  e elas resolveram-no numa corrida: `local=[0.0, 0.0]` (o afim está certo — o centro do mundo cai
  na origem local do corpo), `dist_mundo=0.556` contra `tolerancia=0.099` (o contorno está longe, e
  bem), e **`achou=Some(2)` com `esperado=3`**.
  ⛔⛔⛔ **O `2` é o RÓTULO.** O `path_at` cai no ramo do preenchimento e devolve *a forma mais ao
  topo que contém o ponto* — e o `+10` é desenhado por cima do corpo. ⇒ **carregar no meio do botão
  não o pressionava**; só a margem à volta das letras funcionaria. *O alvo maior da tela era o único
  inalcançável no meio dele.*
  ⭐⭐ **A cura são DUAS metades, e nenhuma basta:** a LEI ([`ph2d_ecs::hud::botao_de`]) sobe a cadeia
  de `ChildOf` — um clique em qualquer descendente de um botão é um clique no botão, que é o que
  toda a interface que existe faz —, e a CENA passa a pendurar o rótulo **no botão** (pose local
  `(0,0)`: a posição é herdada, e repetir o `(0,−5)` somava o deslocamento duas vezes).
  ⚠️ **É a MESMA FORMA da política das quinas**, um nível acima: *o componente mora no CONTAINER, e
  a pergunta SOBE a cadeia em vez de olhar só a própria entidade* (o `container_of` do envelope, o
  mesmo `MAX_PROFUNDIDADE`, a mesma cerca contra um ciclo de `ChildOf`).
  ⭐ **E o veredito passa agora pela PORTA DO PRODUTO** e não por uma comparação de ids: `achou=Some(2)`
  continua a ser o rótulo, e o que se afirma é `dono == a entidade do botão`. *Comparar `achou == id`
  media outro programa — reprovava a cena com o clique a funcionar, e aprovaria o dia em que o rótulo
  saísse de cima do corpo com a fiação partida.* Hoje lê **`SIM`**, com três gates (os dois da folha
  + o `disabled`/nome em branco que já lá estava).
* ⏳ O gizmo do canvas: arrastar a raiz não faz nada (a pose é conduzida) — hoje o painel **di-lo**;
  desenhar a caixa de referência no canvas é wave própria.

## §8 — O smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_HUD_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

Diagnóstico: `PH2D_HUD_LOG=1` (a vista e quantos canvas conduzidos) · `PH2D_SIGNAL_LOG=1` (os
sinais, os efeitos resolvidos — **imprime mesmo a zero**, que é o caso mudo — e o contador).

## §9 — Os QUATRO vermelhos que só o PORTÃO viu (e o quinto, que é carga)

⚠️ **Nenhum deles é da crate-folha nem da lei** — os quatro são *costura*, e três vivem em `tests/it/`
de crates que a linha não corre no laço interno (a família que o `CLAUDE.md` §2 nomeia).

1. ⛔⛔ **O verbo novo existia e o artista não lhe chegava.** `SignalVerb::ALL` foi a `8` e o
   `INSP_ACTION_VERB` ficou em `7` ⇒ *o `Add to Counter` era inalcançável pelo seletor*. O id entra
   **APENDADO**, pela mesma razão do enum: a posição **é** a tag, e um id no meio reescreveria o
   verbo de toda linha de acção já gravada, em silêncio. ⭐ E a fixtura escrita à mão do painel
   **disse que era ela a velha** — a cerca `assert_fixture_covers_the_model` fez exactamente o que o
   doc dela promete, e o sintoma sem ela seria *«a entrada 7 não foi pintada»*, que se lê como
   defeito do painel.
2. ⭐⭐ **A lei da CAIXA DE OBJECTO acordou, e a resposta é NÃO** (`ramo_botao_do_hud` é o 4.º ramo a
   receber o `on_canvas`). Escrita no gate, com mecanismo: a condição é indexada pela **ferramenta na
   mão** e o HUD não é uma — o eixo dele é *«o relógio anda?»*, e suprimir a caixa por aí tirava a
   selecção e o gizmo de toda cena a correr. E o modo de falha é de outra ordem: para quem autora a
   caixa mata o gesto **sempre**; aqui ela só ensombra o botão quando calha por cima, e o que
   acontece então é o editor a fazer o que faz — seleccionar.
3. **Tecto de LOC de FUNÇÃO** (`sections/hud_corpo.rs::corpo`, `227` contra `200`) — curado por
   **CORTE**: os quatro blocos viram `bloco_raiz`/`bloco_rotulo`/`bloco_botao`/`bloco_contador` e a
   `corpo` fica a ser o **ÍNDICE** deles, que é como ela já se lia. ⛔ Nenhuma entrada nova no
   `FN_OVERAGE_OK`.
4. **Tecto de LOC de FICHEIRO** (`app_state.rs`, `981` contra `976`) — curado por **CORTE**, e as
   duas metades são reais: a prosa do campo `components` estava escrita **duas vezes** (o irmão
   di-la em full) ⇒ uma linha com ponteiro; e uma **doc ÓRFÃ** do `PH2D_NEST_SMOKE` vivia 48 linhas
   acima do campo dela, sobre um vizinho que já tinha a sua — *duas docs contraditórias num campo e
   nenhuma noutro*. ⚠️⚠️ **Fica em `976`, exactamente no tecto**: para o INTEGRADOR, este é um
   ficheiro com **zero folga** numa rodada em que o tecto de LOC é a grandeza que soma entre linhas
   sem ninguém a contar (`CLAUDE.md` §5.0).

⚠️ **O quinto é CARGA e pede promoção à lista do §5.0:**
`measure_input_cost::the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`
([`ph2d-tool-painter`](../../../crates/ph2d-tool-painter/)) — reprovou no meio de um fan-out de
**15 009** testes e passa **3 de 3 sozinho a `load 14–22`**, com **zero linhas** do diff desta linha
naquela crate (`git diff --stat <merge-base> -- crates/ph2d-tool-painter/` vem vazio). É um gate que
compara um custo medido contra um número — a forma canónica da família.

## §10 — O report do dono: *«passo 4 não funciona»* — e o defeito NÃO era do botão

⛔⛔⛔ **O botão era DESENHADO num sítio e PICADO noutro.** A auto-conferência da §7 dizia `SIM` — e
ela mede **metade** da corrente: *que forma está sob um PONTO DO MUNDO, e de quem ela é*. O clique
do artista começa no **ECRÃ**, e era ali que a corrente estava partida.

### As caixas, medidas

A sonda nova (`PH2D_HUD_PROBE=1`) conduz o `ramo_botao_do_hud` **real** e **varre o ecrã** a
perguntar ao mesmo `path_at` do produto onde o botão é alcançável:

| | caixa de ecrã do botão |
|---|---|
| onde ele é **DESENHADO** (medido na foto) | `x 879..1050 · y 402..462` |
| onde o dedo o **ENCONTRAVA** | `x 800..1128 · y 728..848` |
| **depois da cura** | `x 872..1056 · y 392..472` |

⇒ ele era clicável **~340 px abaixo** de onde aparece — **dentro do painel da timeline**, onde
`on_canvas` é `false` (medido: `painel=true widget=true`) e o ramo **nem chegava a correr**.

### A causa, e a lei que já estava escrita

`center_split = Horizontal { t: 0,55 }` ⇒ **a cena desenha numa BANDA de `1930×556`** e a projecção
dela MUDA, enquanto `vec_world_at` mapeava o cursor contra a janela de `1930×1012`.
⭐ **A aritmética fecha antes do código:** `10` unidades de mundo em `556 px` são `55,6 px/unidade`,
e o centro do botão (`y = −2,778`) cai em **`432`** — exactamente onde a foto o mostra.

⚠️⚠️ **A lei já estava escrita e a porta não a usava:** o doc do
`ph2d_app_motion::field_gizmo::scene_window_wh` diz *«todo mapeamento mundo↔tela do chrome da cena
TEM de usar isto»*, e há um gate vizinho chamado
`the_cursor_is_mapped_through_the_scene_viewport_not_the_window`.
⇒ porta nova [`App::scene_window`](../../../shells/desktop/src/connector_gesture.rs), lida pelo
`vec_world_at` **e** pelo `vec_px_to_world` (sem ela a tolerância de captura saía `1,85×` maior).

### ⚠️ Isto é PRÉ-EXISTENTE e MAIOR que o HUD — o censo

O `vec_world_at` é a porta de **seis** gestos vectoriais (selecção, balde, trim, esqueleto,
conector, pré-visualização de UI): **todos** apontavam ao sítio errado com a timeline aberta. E o
censo da shell mede **72** chamadas de `screen_to_world` em **33** ficheiros, contra **19** sítios
que já passam pela banda ⇒ *a dívida não é do HUD, é do mapeamento do chrome*. Os nomes saem de
`grep -rn "screen_to_world(" shells/desktop/src/`; ⛔ **esta linha curou as DUAS portas do vector** e
deixa as outras nomeadas — o dono do chrome decide a rodada.

### A prova

Com a cura, a sonda conduz o gesto na posição em que o botão **aparece** (`tela=(965, 432,4)`),
`on_canvas=true`, os dois lados do clique são consumidos, e a foto seguinte lê **`Pontos: 17`**
contra os `6`–`7` que o relógio sozinho dá no mesmo instante — os `+10` do botão.
Gates: `o_cursor_e_mapeado_pela_banda_da_cena` (as duas portas · e a própria `scene_window` derivar
do split, senão a cura morre calada).
