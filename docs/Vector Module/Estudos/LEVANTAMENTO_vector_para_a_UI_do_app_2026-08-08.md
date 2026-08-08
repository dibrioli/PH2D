# Levantamento — o que falta ao Vector para desenhar **a UI deste app**

> **Pedido do Enio (2026-08-08):** *"tornar o nosso módulo vector pronto para criar a UI deste nosso
> APP. Veja o que falta para capacitá-lo integralmente para Isso."*
>
> Documento de **MEDIÇÃO**, não de plano. Todo número aqui foi contado nesta worktree em
> 2026-08-08, com o comando ao lado. O plano-mãe é
> [`PLANO_UI_UX_padrao_figma.md`](PLANO_UI_UX_padrao_figma.md); ele diz *o que construir*, este diz
> **onde estamos**.

---

## §0 — A resposta em três linhas

O laço **já fecha**: uma moldura autorada com filhos vestidos vira uma `PanelSpec`, o codegen
escreve o código, e o `ph2d-panel-authored` é um painel **vivo** — pinta, responde ao ponteiro,
rola, e a row **mexe na arte**.

O que falta para ele desenhar *qualquer* painel deste app são **três coisas de naturezas
diferentes**, e a maior delas é a mais barata:

| # | falta | tamanho | natureza |
|---|---|---|---|
| **1** | **quatro widgets que a lei já cobre** e ninguém acrescentou | **18,3%** da UI real | **OMISSÃO** — fiação |
| **2** | a família cuja aparência é função de uma **LISTA** | **14,6%** | **ESTRUTURAL** — pede filhos autorados |
| **3** | **um** painel autorado por build · colapso de seção · o valor não é salvo | — | escopo declarado |

---

## §1 — A medição: de que os painéis deste app são FEITOS

> `grep -rhoE "\bX::new\(|paint_x\(" crates/ph2d-panel-*/src/` — **construções reais**, não menções.
> 23 painéis, **438** construções de widget.

| widget | construções | vestível hoje? | por quê |
|---|---:|:---:|---|
| `Button` | **156** | ✅ | |
| `SectionHeader` | 38 | ✅ | |
| **`ColorSwatch`** | **36** | ❌ | ⚠️ `(id, label, rgba, state, size)` — **nenhuma lista** |
| `Checkbox` | 32 | ✅ | |
| `Slider` | 28 | ✅ | |
| **`SegmentedAdaptive`** | 28 | ❌ | `options: Vec<SegmentedOption>` |
| **`Dropdown`** | 26 | ❌ | lista |
| **`NumberInput`** | **21** | ❌ | ⚠️ `(id, label, value, step, min, max, state)` — **nenhuma lista** |
| **`IconButton`** | 21 | ❌ | ⚠️ `(rect, ícone, estado)` — **nenhuma lista** |
| `Card` | 16 | ✅ | |
| `Toggle` | 11 | ✅ | |
| `TextInput` | 11 | ✅ | |
| **`Tabs`** | 9 | ❌ | lista |
| `Tag` | 2 | ✅ | |
| **`LevelMeter`** | 2 | ❌ | ⚠️ `(id, label, rms, peak_hold, clipped)` — **nenhuma lista** |
| **`RadioGroup`** | 1 | ❌ | lista |
| `ProgressBar` · `Spinner` · `ListItem` · `Divider` | 0 | ✅ | vestíveis e **não usados** |

**Cobertura hoje: 294 / 438 = 67,1%.**

---

## §2 — ⭐ O achado: o buraco tem DUAS naturezas, e misturá-las custaria a wave errada

O `skin.rs` declara a fronteira como **estrutural**, e a frase está certa:

> *"Um widget cuja aparência é função de (retângulo, rótulo, estado) é vestível **hoje**. Um widget
> cuja aparência é função de uma **LISTA** (Tabs, TreeView, RadioGroup, Dropdown, Combobox) precisa
> de filhos autorados."*

⚠️ **Mas quatro dos que faltam NÃO são dessa família** — eles satisfazem a lei e simplesmente nunca
foram acrescentados. Conferido campo a campo (§1): nenhum tem `Vec`.

| natureza | widgets | % da UI real | o que custa |
|---|---|---:|---|
| **OMISSÃO** | `ColorSwatch` · `NumberInput` · `IconButton` · `LevelMeter` | **18,3%** | 4 entradas em `WidgetKind::ALL` + 4 códigos novos + 4 peles + 4 braços de `populate` |
| **ESTRUTURAL** | `SegmentedAdaptive` · `Dropdown` · `Tabs` · `RadioGroup` | **14,6%** | *a lista vem dos FILHOS* — wave própria |

⇒ **Fechar só a omissão leva a cobertura de 67,1% para 85,4%**, e é fiação no molde que a W6.2 já
estabeleceu (código explícito, nunca a ordem do enum; `from_code` devolve `None` para o
desconhecido).

⚠️ **E a segunda metade não é "mais do mesmo".** *Filhos autorados* é a resposta que o próprio
`skin.rs` nomeia, e ela muda o modelo: hoje um filho vestido é uma **row**; ali um filho passa a
poder ser uma **opção** de um controle irmão. Quem a construir tem de decidir *como uma sub-árvore
diz "estas quatro formas são as minhas opções"* — e isso é desenho, não fiação.

---

## §3 — Os limites que NÃO são de widget

### 3.1 **UM** painel autorado por build

`ph2d-panel-authored` é uma crate única com um `generated/panel.rs`, e o `Panel::ID` é o literal
`"authored"`.

⚠️ **Derivar o `ID` do desenho foi construído e DESFEITO**, e a razão está escrita no `lib.rs`: o
`ID` é a chave do mapa de visibilidade e o literal que toda lista de painéis do shell carrega —
derivá-lo faria renomear a moldura deixar uma entrada órfã e o app pensar *"é outro painel"*.

O `generated::PANEL_ID` (hoje `"color"`) continua emitido e lido pelo gate: **ele é o slug, e é a
identidade que a fatia multi-painel vai usar para nomear a crate.** A decisão está tomada; o que
falta é a fatia.

### 3.2 O que a row **não** faz

| | estado |
|---|---|
| rola | ✅ (`AUTHORED_SCROLLBAR_ID`, `scrollbar_is_needed`) |
| fecha pelo X | ✅ (e escreve a MESMA visibilidade que o abridor lê) |
| **colapsa seção** | ❌ — **9 dos 23** painéis reais usam colapso; o `SectionHeader` vestível é só o cabeçalho |
| **o valor sobrevive ao arquivo** | ❌ — ele vive no `WidgetStore`, que é de runtime. Nomeado no `vec_widget_drive.rs`: *"reabrir o projeto devolve os controles ao default"* |

### 3.3 ⚠️ O canal de intent **não tem consumidor**, e o doc dele MENTE

`AuthoredIntent{Value,Flag,Text,Fired}` é empurrado a cada gesto. O doc do `drain_intents` diz
**"a shell chama uma vez por frame"** — e a varredura do repo inteiro diz que **ninguém fora dos
testes da própria crate o chama**.

⚠️ **A ausência era uma cerca de Chesterton correta** (o `state.rs` a declara: *"quem escuta ainda
não existe … é a W4b/W8a"*), **mas ela envelheceu**: a **W8b.3 ligou a row à arte** — por outra
rota, o `WidgetStore` → `VecViewState`. Ou seja, uma das três metades que a cerca escalonava **foi
feita por fora**, e a nota sobreviveu ao fato.

**A consequência prática, e ela não é só higiene:** uma fila que só é empurrada **cresce sem teto**
enquanto o painel autorado está aberto — um arrasto de slider emite um `AuthoredIntent` com duas
`String` por quadro.

### 3.4 W8a — o runtime

⛔ **Bloqueado por ausência:** `ph2d-runtime` não existe. Não é adiamento, é pré-requisito — e é o
próximo item da fila que o Enio deu.

---

## §4 — A ordem recomendada, com o preço

| ordem | wave | ganho medido | custo |
|---|---|---|---|
| **1** | **os quatro por OMISSÃO** | 67,1% → **85,4%** de cobertura | fiação; o molde da W6.2 já existe |
| **2** | **colapso de seção** | 9 dos 23 painéis reais | comportamento, não tipo novo |
| **3** | a família da **LISTA** (filhos autorados) | 85,4% → **~100%** | desenho novo: *como uma sub-árvore diz "estas são as minhas opções"* |
| **4** | multi-painel | N painéis por build | a decisão está tomada (§3.1) |
| **5** | persistir o valor de uma row | o painel lembra | W4b/W8a |

⚠️ **A ordem não é por tamanho, é por RAZÃO ganho/custo**, e o (1) é o único item deste levantamento
onde os dois lados já estão medidos.

---

## §5 — O que este levantamento NÃO diz

- **Não diz que 85,4% é "pronto".** Um painel deste app não é só uma lista de rows — a §3.2 mede
  duas coisas que ele tem e o gerado não.
- **Não mede o Inspector**, que é o painel com mais widgets do app e cuja estrutura (seções por
  domínio, rows condicionais ao tipo do objeto selecionado) é de outra classe: um painel autorado
  descreve uma lista FIXA, e o Inspector é uma lista **função da seleção**.
- **Não trata da UI dos JOGOS** (o outro consumidor que o pedido de 2026-08-01 nomeia). Aquele é o
  W8a, e a fronteira dele é o runtime.
