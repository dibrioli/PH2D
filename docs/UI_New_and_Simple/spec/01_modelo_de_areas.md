# Spec — o modelo de ÁREAS (rascunho 1, 2026-08-30)

> ⚠️ **RASCUNHO.** Descende directamente das cinco decisões do Enio
> ([`00_DECISOES_DO_ENIO.md`](../00_DECISOES_DO_ENIO.md)) e das medições. Cada número aqui é
> **derivado** de uma medição ou copiado de uma referência com licença permissiva — nenhum é
> escolhido. Onde algo é proposta minha e não dedução, está marcado **⏳ proposta**.
>
> É a peça em que as cinco decisões convergem. Nada mais desta linha pode ser desenhado antes.

---

## §1 — O que é uma Área

Três níveis de **espaço**, o vocabulário do Blender (`paradigms.md`), com os nomes que vamos usar:

| nível | é | exemplo |
|---|---|---|
| **Layout** | a janela inteira, arrumada para uma **tarefa** (D3) | *Editor 2D*, *Editor de Texto*, *Runtime* |
| **Área** | um rectângulo que hospeda **um editor** | o canvas 2D, a linha do tempo, o grafo de nós |
| **Região** | uma faixa **dentro** de uma área | cabeçalho, ferramentas, régua, conteúdo |

⚠️ **E há dois eixos de ESTADO que não são espaço, e não se confundem com estes**
([`pesquisa/04`](../pesquisa/04_modo_layout_e_ferramenta.md)):

| eixo | quem decide | onde o selector vive |
|---|---|---|
| **Modo** | ⭐ o **tipo do objecto** seleccionado | cabeçalho da **área** |
| **Ferramenta** | o utilizador, dentro do modo | **toolbar** da área |

⭐⭐ **É por isso que o cabeçalho e a toolbar são regiões obrigatórias do `CENTER`** (§4): são os
donos declarados desses dois selectores. Um modelo de áreas que não lhes desse sítio empurrá-los-ia
de volta para a barra de cima — que é como chegámos aos 29 pills.

⭐ **A lei que faz isto funcionar (D5): regiões são IRMÃS numa fila, nunca camadas empilhadas.**
Uma área reparte a sua largura entre as regiões; elas não se sobrepõem porque **não partilham
coordenada**. É por isso que o defeito das réguas desaparece por construção, e não por uma
verificação — hoje ele existe porque régua, rail e barra ancoram todos em `canvas = (0,0,w,h)`
e a ordem de pintura decide o vencedor (`hero/paint.rs` 265 → 420 → 542).

---

## §2 — Os encaixes (D4): **seis**, e o número é DERIVADO

O Godot tem 12 (`editor_dock.h:53`): quatro por lado (duas colunas × duas metades), mais três em
baixo, mais o principal. ⛔ **Não copiamos os 12**, e a razão é aritmética:

| colunas por lado | largura | de 1366 (iPad Pro) |
|---|---:|---:|
| **1** (308 + 304) | 612 px | **44,8 %** — cabe |
| 2 (o modelo do Godot) | 1224 px | **89,6 %** — ⛔ impossível |

⇒ **uma coluna por lado.** Os 12 do Godot pressupõem um monitor de desktop largo; o nosso alvo
declarado é 1366 pontos.

```
┌──────────────────────────────────────────────────────┐
│  BARRA GLOBAL   (Arquivo · Editar · Ver · Ajuda)     │   D2
├───────────────┬──────────────────────┬───────────────┤
│  LEFT_TOP     │                      │  RIGHT_TOP    │
│               │                      │               │
├───────────────┤       CENTER         ├───────────────┤
│               │                      │               │
│  LEFT_BOTTOM  │                      │  RIGHT_BOTTOM │
├───────────────┴──────────────────────┴───────────────┤
│                     BOTTOM                            │
├──────────────────────────────────────────────────────┤
│  BARRA DE ESTADO  (o HUD de hoje)                     │
└──────────────────────────────────────────────────────┘
```

**Seis encaixes:** `LEFT_TOP` · `LEFT_BOTTOM` · `RIGHT_TOP` · `RIGHT_BOTTOM` · `BOTTOM` ·
`CENTER`.

⏳ **Proposta:** `BOTTOM_LEFT`/`BOTTOM_RIGHT` (o Godot tem-nos) ficam **de fora até alguém os
pedir** — são um corte da faixa de baixo e podem ser acrescentados sem mudar nada do resto.

**Regras:**
1. Um encaixe hospeda **0..n** painéis. Com `n > 1` são **abas**. *É assim que um encaixe absorve
   crescimento sem crescer.*
2. Um encaixe com `0` painéis **tem largura zero** — não fica um vazio a ocupar espaço.
3. As divisórias arrastam-se; as posições fazem parte do Layout gravado.
4. **`CENTER` nunca está vazio e nunca é uma aba de outro encaixe.**

---

## §3 — O que um Painel DECLARA

Portado do `EditorDock` do Godot (`editor_dock.h:76-91`, **MIT — podemos portar**), com as
diferenças nomeadas.

| campo | tipo | papel |
|---|---|---|
| `title` | texto i18n | o nome que aparece na aba |
| `icon` | `IconId` | o ícone da aba |
| `layout_key` | chave estável | ⭐ com que nome o layout o grava/restaura |
| `shortcut` | atalho | abrir/focar |
| `default_slot` | `Slot` | onde nasce |
| `allowed_slots` | conjunto de `Slot` | ⭐⭐ **D1: onde ele PODE estar** |
| `can_float` | bool | ⭐⭐ **D1: se pode sair para janela própria** |
| `closable` | bool | se pode ser fechado |
| `transient` | bool | some quando deixa de fazer sentido |

⭐⭐ **`allowed_slots` + `can_float` são a D1 inteira.** Um painel de propriedades declara
`allowed_slots = {RIGHT_TOP, RIGHT_BOTTOM}` e `can_float = false` — e **nunca chega perto de uma
viewport ou de uma régua**, porque não há valor que o exprima.

⚠️ **Divergência deliberada do Godot:** eles têm `available_layouts` (`VERTICAL | HORIZONTAL |
FLOATING`), que descreve a **forma** que o dock aceita. Nós usamos `allowed_slots`, que descreve
os **sítios**. Com seis encaixes fixos o sítio já implica a forma, e um conjunto de sítios é
directamente verificável por um portão.

⭐ **E isto é gateável:** um portão que percorre o registo de painéis e falha se algum declara um
`default_slot` fora dos próprios `allowed_slots`, ou se um painel com `can_float = false` tem
código que o desenha fora de um encaixe.

---

## §4 — As regiões de uma Área (D5)

Cada área reparte-se assim, **de fora para dentro**:

```
┌────────────────────────────────────────────────┐
│ CABEÇALHO   [editor ▾][modo ▾] menus ⋯ opções ▾│  ← D2
├──────┬─────────────────────────────────────────┤
│ FER- │ ▓▓▓▓▓▓▓ RÉGUA (topo) ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │
│ RA-  ├─┬───────────────────────────────────────┤
│ MEN- │R│                                       │
│ TAS  │É│           CONTEÚDO                    │
│      │G│                                       │
│      │U│                                       │
│      │A│                                       │
└──────┴─┴───────────────────────────────────────┘
```

A ordem do cabeçalho é a do HIG do Blender (`editors.md`, «What Goes Where?»), que já é uma
espec pronta:

1. selector de editor · 2. **mode toggle** · 3. **pulldowns do editor** ·
4. *(centro)* selector do dado principal / busca · 5. *(direita)* opções de exibição, em popovers

⭐⭐ **O trilho de ferramentas e a régua são REGIÕES do `CENTER`, não faixas da janela.** É a
generalização de D5, e é o que apaga o defeito das duas réguas de uma vez:

- Hoje: `rail.x = 0` e `left_band.x = 0` ⇒ 86,8 % tapada.
- No modelo: a área dá `[0..57]` ao trilho e `[57..77]` à régua. **Não há sobreposição porque não
  há coordenada partilhada.**

⚠️ **Consequência que tem de entrar no mesmo trabalho:** a barra global (§2) deixa de flutuar
sobre o conteúdo e passa a **subtrair** altura, como o trilho subtrai largura. A régua de cima
fica dentro da área, abaixo dela. ⛔ *Uma barra que continue a flutuar reproduz o defeito de
cima, num modelo novo.*

---

## §5 — O orçamento, medido

No alvo 1366 × 1024. Trilho 57, régua 20, painel esquerdo 308, direito 304.

| estado | `CENTER` | área de desenho | % da janela |
|---|---:|---:|---:|
| ambos os lados abertos | 754 | 677 | **49,6 %** |
| só a direita | 1062 | 985 | **72,1 %** |
| **ambos recolhidos** | 1366 | 1289 | **⭐ 94,4 %** |

**Contra hoje:** 51,0 % de chrome, régua esquerda 86,8 % tapada — e ⛔ **recolher os painéis hoje
não devolve a régua**, porque o trilho continua em `x = 0`.

⭐⭐ **Repare no que este modelo compra, e no que NÃO compra.**
- ⛔ **Não compra área** com tudo aberto: 49,6 % contra os 49 % de hoje. *Um dock ocupa o mesmo
  que um flutuante* — já estava dito na D1 e a aritmética confirma.
- ⭐ **Compra duas coisas:** (a) **nada fica escondido** — as réguas ficam 100 % visíveis em todos
  os estados, por construção; e (b) **recolher passa a valer**: 94,4 % de tela limpa, com a régua
  a funcionar.

⇒ **A área ganha-se na D2** (mover 66 das 74 entradas do painel medido para os seus donos), não
na D4.

---

## §6 — A escala de hardware, e a restrição que ela impõe

Do Spectrum ([`pesquisa/02 §4`](../pesquisa/02_referencias_e_licenca.md)): no modo toque o **alvo
cresce 1,25×** e o **padding interno encolhe ~0,77×**.

⛔⛔ **Restrição declarada agora, não descoberta depois: a LARGURA DOS ENCAIXES NÃO ESCALA com o
alvo de toque.** Se escalasse, os 612 px dos dois lados passariam a 765 = **56 % da largura** com
os painéis apenas abertos.

⇒ **o que escala é o conteúdo DENTRO do encaixe** (altura de linha, alvo de toque), e o encaixe
responde **rolando**, não alargando. ⚠️ Isto contradiz o instinto e é precisamente por isso que
está escrito.

⚠️ **E são dois eixos, não um** — Godot e Spectrum discordam e a discordância é real:
`EDSCALE` (um float global) responde *«o ecrã é fino?»*; o `scale-set` por-token responde *«o
dedo é gordo?»*. ⏳ **Proposta:** dois números independentes, `pixel_scale` (do SO) e
`touch_scale` (do input), e ⛔ nunca um só. Não medido.

---

## §7 — O que fica por decidir

✅ **FECHADAS pelo Enio em 2026-08-30** (ver [`00_DECISOES_DO_ENIO.md`](../00_DECISOES_DO_ENIO.md)):
a **lista de Layouts** (D7 — oito) · os **modos por tipo de objecto** (D6) · e **as timelines em
todos os modos** (D8, que no modelo é uma linha: a Timeline é uma área do `BOTTOM` em qualquer
Layout, ligada à **selecção**).

Ficam:

1. **⏳⏳ Pose 2D ou 3D — a maior das três.** O `PropKind` da Timeline tem 13 variantes e nenhuma
   tem Z, porque o `ph2d_ecs::Transform` é `Vec2` + um `f32`. As duas saídas e os seus preços
   estão em [`medicoes/04 §6`](../medicoes/04_o_alcance_das_timelines.md); ⛔ **quantos sítios leem
   `Transform` não foi medido**, e sem esse número a escolha é gosto.
2. **⏳ A escultura tem de virar ENTIDADE** antes de qualquer timeline a alcançar — hoje é um campo
   do estado do app, inalcançável por tudo. Molde pronto: o `PaintedDoc`.
3. **⏳ Como partir o `DrawMode`** nos dois eixos. São **2 modos + 12 ferramentas** achatados num
   enum de 14 variantes vivas, com gates. ⚠️ Hoje **não se exprime «Edit + ferramenta Fillet»**.
4. **⏳ Adoptamos o campo `Mode` do Workspace?** (*"switch to this Mode when activating"* — o
   atalho que liga os dois eixos sem os acoplar.)
5. **⏳ Os 9 toggles de módulo** (`vector_toggle`, `motion_toggle`, …) são interruptores
   independentes — 2⁹ combinações. Um Layout é *um-de-N*. Como se converte um no outro?
6. **⏳ Migração.** São **2 073 ids** e **25 painéis**. A ordem de conversão não está desenhada, e
   ⚠️ o §1 do estado medido avisa: *uma superfície desse tamanho não se redesenha à mão.*

---

## §8 — ⛔ O que NÃO fazer, com o motivo ao lado

| ⛔ | porquê |
|---|---|
| Cortar os temas de 4 → 2 antes de separar layout de paleta | o `blueprint` é o único que liga `PanelLayout::Sidebar` — apaga o único modo ancorado ([`medicoes/03 §5`](../medicoes/03_o_censo_de_cor.md)) |
| Manter a fuga do gizmo depois de ancorar | remédio duplo: passa a fugir de uma moldura que já não o alcança ([D1](../00_DECISOES_DO_ENIO.md)) |
| Copiar os 12 encaixes do Godot | 89,6 % da largura no nosso alvo (§2) |
| Escalar a largura dos encaixes com o toque | 56 % da largura só com os painéis abertos (§6) |
| Deixar a barra global a flutuar | reproduz o defeito da régua de cima num modelo novo (§4) |
| Cortar os 34 slots de cor dos nós por analogia com o Timeline | não são apelidos — são valores distintos, e a pergunta é de produto ([`medicoes/03 §4`](../medicoes/03_o_censo_de_cor.md)) |
