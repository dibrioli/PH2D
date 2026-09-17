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

## §6 — Gates e provas

* `ph2d-hud`: 9 gates (a tabela do oráculo **é** o gate) · **6/6 mutações sangram**
  ([`mutacao_hud_w1.sh`](../ferramentas/mutacao_hud_w1.sh), com controlo sobre o próprio filtro).
* `ph2d-ecs`: 9 + o do placar, este com **CONTROLO NEGATIVO** (sem pose ⇒ zero efeitos).
* `ph2d-app-components`: 6 da ponte + 7 do instantâneo/dreno.
* `ph2d-editor-core`: 4 do vocabulário.
* ⚠️ **O clique do botão prova-se na LEI**, não na foto: eventos sintéticos não chegam à Xwayland
  virtual e o `ydotool` move o rato REAL do dono (proibido).

## §7 — ABERTO, e de quem é cada um

* ⏳ **As quatro âncoras do `VecAnchors` não estão ligadas ao canvas** — elas medem contra a caixa
  LOCAL de uma moldura, e a raiz do HUD é conduzida por ESCALA. Ligar as duas é uma wave (e o
  `expand` do oráculo depende dela).
* ⏳ **No EDITOR, um HUD colado às bordas cai atrás dos painéis**: a vista da câmera é a da JANELA e
  o editor pinta num sub-rectângulo. Numa janela de jogo é exacto. **Decisão de produto.**
* ⏳ O `UiButton` só é alcançável por um caminho **vectorial** (`path_at`): um botão feito de
  *sprite* não é pego. Nomeado, não construído.
* ⏳ O gizmo do canvas: arrastar a raiz não faz nada (a pose é conduzida) — hoje o painel **di-lo**;
  desenhar a caixa de referência no canvas é wave própria.

## §8 — O smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_HUD_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

Diagnóstico: `PH2D_HUD_LOG=1` (a vista e quantos canvas conduzidos) · `PH2D_SIGNAL_LOG=1` (os
sinais, os efeitos resolvidos — **imprime mesmo a zero**, que é o caso mudo — e o contador).
