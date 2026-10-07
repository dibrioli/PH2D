# 02 — O estudo: como os apps vetoriais amados distribuem a UI

> 2026-10-07. Síntese das quatro pesquisas em [`referencias/`](referencias/) — cada afirmação daqui
> tem a fonte lá, com URL. **R1** [Affinity](referencias/R1_affinity.md) (⭐ o estilo preferido do
> Enio) · **R2** [Figma · Sketch · Penpot · Graphite](referencias/R2_figma_sketch_penpot_graphite.md) ·
> **R3** [Illustrator · Linearity Curve · Rive · Inkscape](referencias/R3_illustrator_curve_rive_inkscape.md) ·
> **R4** [apps de iPad](referencias/R4_apps_de_ipad.md) (pedido do Enio: *«apps para iPad costumam ser
> bons nisso»*).
>
> ⚠️ Método: isto é **leitura** de manuais, notas de versão e fóruns — basta para decidir ONDE vai um
> controlo. Para uma LEI (o que um gesto faz à geometria) a casa exige oráculo CORRIDO (`CLAUDE.md`
> §0.9), e nada aqui substitui isso.

## §1 — A resposta curta: todos convergem para CINCO lugares

Ignorando o nome que cada um dá, **todo** editor vetorial bem avaliado põe a UI nos mesmos cinco
sítios, e cada sítio responde a uma pergunta diferente. A R2 resume-o a partir do Graphite, que é
o que o declara com mais nitidez: *ferramentas = verbos · opções = como o verbo actua ·
propriedades = os substantivos · rodapé = o que as teclas fazem agora.*

| lugar | pergunta que responde | Affinity | Figma | Illustrator | Graphite | Rive | Affinity iPad |
|---|---|---|---|---|---|---|---|
| **① coluna ESQUERDA** | *que gesto tenho na mão?* | Tools panel (flyouts, 1–N colunas, cor fill/stroke em baixo) | toolbar (na BASE, flutuante) | Toolbar (Basic/Advanced + gaveta «…») | tool shelf em 3 grupos de cor | (no topo) | toolbar esquerda |
| **② faixa de CIMA** | *como o gesto actua?* | **Context Toolbar** | barra secundária do Vector Edit | Control panel | **Control bar** (opções à esq., vista à dir.) | toolbar | (na BASE) |
| **③ coluna DIREITA** | *o que é a coisa seleccionada?* | Studio (Layers, Color, Stroke, Transform, Appearance…) | Properties | Properties | Properties + Layers | Inspector | faixa de ícones que abre painéis |
| **④ PERTO da seleção** | *o que faço a seguir com isto?* | — (só no iPad: Quick Menu) | selection actions | **Contextual Task Bar** | — | — | Quick Menu (3 dedos) |
| **⑤ RODAPÉ** | *que teclas valem agora?* | **status bar com dica + modificadores** | — | status/help bar | **status bar com input hints** | timeline (em Animate) | Zoom/Snap/Preview |

⇒ **O PH2D tem hoje ① ② ④ ⑤ VAZIOS para o vector** e tudo em ③ ([`01_o_estado_atual.md`](01_o_estado_atual.md)).
O trabalho não é inventar uma UI: é **pôr cada secção no lugar cuja pergunta ela responde** — e o
nosso `section_scope.rs` já faz essa pergunta (GESTO / SELEÇÃO / DOCUMENTO).

## §2 — Por que o Affinity é o modelo (e onde ele NÃO é)

O Enio prefere o Affinity ao Inkscape. As fontes dão razões concretas para isso, e são todas de
**densidade e lugar**, não de beleza (R1 §5):

- **Campos numéricos compactos sem botões `+/−`.** Arrasta-se o rótulo para mudar o valor e a roda
  do rato muda ×1 / ×10 / ×0,1. Os campos aceitam unidades e expressões. O Inkscape, por contraste,
  tem campos *«immense with large minus and plus buttons»*.
- **Ferramentas dedicadas onde o Inkscape tem efeitos.** O Corner Tool e o Stroke Width Tool são
  ferramentas na mão, e no Inkscape a mesma coisa é um *Live Path Effect* num diálogo.
- **Um sítio para cada coisa.** O Fill & Stroke do Inkscape tem *«tabs stacked on top of tabs»*. O
  próprio Inkscape admitiu na 1.4 que as opções da toolbar *«were hard to discover»* (R3 §D).

**Os sete traços do Affinity que este plano copia** (R1 §1–3):

1. **Context Toolbar.** O conteúdo depende da ferramenta e do tipo de seleção (Node numa forma
   paramétrica ≠ Node numa curva). Leva **as amostras Fill/Stroke, que abrem a cor ali mesmo, e a
   largura do traço**. O avançado fica num botão **Settings** dentro da própria barra.
2. **Tools panel com flyouts.** Um triângulo no canto marca a família, e clique longo abre-a. Há
   **dicas ricas** (nome + uma frase), as amostras **Fill/Stroke por baixo** e `X`/`Shift+X`/`D`
   para trocar/inverter/repor.
3. **Status bar que diz os modificadores da ferramenta actual** (*«Alt: canto · Shift: 45°»*).
4. **Teclas de uma letra para as ferramentas**, iguais às do Illustrator e do Figma no núcleo
   (`V A P N T`). Segurar a tecla dá a ferramenta **temporária** e soltar volta à anterior.
5. **Nós Sharp / Smooth / Smart.** Duplo-clique alterna o tipo, Alt+clique faz canto, e arrastar o
   **segmento** muda a curvatura.
6. **Snapping com feedback.** Guias coloridas por eixo, **rótulo de distância**, uma lista de
   candidatos com halo, e Alt suspende o encaixe.
7. **Studio = layout do modo**, gravado sozinho, com três níveis de reposição. É a nossa **D7**
   (layout *Vetor*), que já existe.

**Onde o Affinity tropeçou — e não copiamos** (R1 §4, armadilhas):

- **Ícones monocromáticos e iguais entre si** (V3). Foi chamado *«retrograde step»*. ⇒ Forma
  distinta por ferramenta e cor por **família** (o Graphite faz isto: geral/vetor/raster).
- **Opção usada todo dia escondida no overflow** da Context Toolbar, que não se pode reordenar.
  ⇒ A barra tem **prioridade declarada** e o overflow é o `⋯` que já temos.
- **Barra flutuante a tapar painéis** (bug da V3). ⇒ Posições **âncora**, nunca soltas (é a nossa D1).
- **Sem escala de UI** (ilegível em 4K). ⇒ A nossa escala da interface (spec/05) fica.

## §3 — O que os APPS DE iPAD sabem e o desktop não (R4)

O Enio tem razão: os apps de iPad resolvem o mesmo problema com **menos espaço e sem teclado**, e
isso força soluções melhores. Para o PH2D, que tem o iPad como alvo
([`medicoes/06`](../medicoes/06_o_orcamento_de_ecra_em_tablet.md)), quatro lições mudam o desenho:

1. **A barra contextual pode morar EMBAIXO.** No Affinity Designer 2 de iPad a Context Toolbar fica
   *«below the canvas»* e **rola** quando não cabe. Em cima ficam só as Personas e os menus. ⇒ A
   escolha topo/base é uma **posição âncora**, e a V3 desktop deixa pô-la em qualquer das duas.
2. **Os modificadores viram um objecto na tela ou um dedo extra.** O Command Controller (Affinity) e
   o Touch Shortcut (Illustrator/Fresco) têm dois estados, **segurar / travar**, e o estado travado
   **fica visível**. A alternativa sem chrome nenhum: **+1 dedo = Shift, +2 = Shift+Alt** (Linearity,
   Graphic, Affinity).
3. **Gesto no lugar do chrome.**
   - **Desenhar e segurar = forma perfeita** (QuickShape do Procreate, Concepts). +1 dedo torna-a
     regular, e a forma nasce **com nós editáveis**.
   - **2 dedos = desfazer, 3 = refazer, 4 = foco** é universal entre todos os apps pesquisados.
   - **Tocar e segurar = menu de contexto** (é o clique direito do dedo).
4. **A UI sai do caminho durante o traço** (Magic Canvas do Linearity) e volta ao soltar.

E dois avisos de iPad que valem também no desktop:
- **Não ponha controlo no centro do topo nem no centro da base.** Lá vivem o `⋯` de janela do
  iPadOS e o indicador de Home.
- **Todo gesto precisa de uma porta VISÍVEL ao lado**, um botão ou um menu. *«Gesture-heavy sem
  tutorial»* é a crítica nº 1 do Procreate Dreams. E **impor um só idioma divide os usuários**: a
  roda do Concepts foi amada por metade e odiada pela outra, e o app teve de oferecer os dois.

## §4 — As armadilhas que TODOS pagaram (convergência das quatro pesquisas)

| armadilha | quem pagou | a nossa defesa |
|---|---|---|
| **Barra flutuante que tapa a arte** e reaparece sozinha | Illustrator CTB (*«IN THE CENTER of the screen»*, a maioria desliga-a) · Figma toolbar da base · Figma Draw (*«let me move the toolbar»*) | posições **âncora** (D1), **fixar** (pin) e **desligar**, e a posição gravada no layout |
| **Painéis «flutuantes» que não se movem** | Figma UI3 beta, **revertido** (*«speed is a feature»*) | ancorados (D1, já decidido) |
| **Controlo frequente num canto minúsculo** | booleanas e constraints no Figma UI3 (teve de lançar um toggle) | Fill/Stroke/largura **na barra de cima**, nunca num canto |
| **Painel que «salta»** (secções mudam de sítio com a seleção) | Figma UI3 | a **ordem é uma tabela** (`paint_body_plan`), e a secção que não serve **some**, sem reordenar as outras |
| **Barra contextual sem customização** | Sketch Copenhagen (*«relearn my tool every few months»*) | posições **estáveis** por ferramenta e paleta `Ctrl+K` com o nome antigo |
| **Overflow numa setinha** sem tooltip nem atalho | Inkscape (admitido na 1.4) | o `⋯` já existente com o rótulo e a tecla (gate de elisão, `medicoes/21`) |
| **Duas barras contextuais concorrentes** | Illustrator (Control + Properties + CTB) | **um sítio por comando** (D2) — e o mesmo id |
| **Mexer em atalho consagrado** | Figma (`⌘K`) | §5: o nosso `U/I/D/X` colidia com o idioma Illustrator ⇒ as booleanas mudam de tecla |
| **Keys criadas sem querer em modo de animação** | Rive (aviso oficial) | a timeline já é um modo explícito (D8) — fica nomeado para quando o Animate tocar no vetor |

## §5 — O teclado: o padrão do ILLUSTRATOR (decidido 07/10)

> Enio, 2026-10-07: *«qual mais aceito layout de teclas? Affinity, Inkscape ou Illustrator? Vamos com
> o mais bem aceito.»* ⇒ **Illustrator.** A prova é medida, não opinião:
>
> - ⭐ **O próprio concorrente o adopta.** O Inkscape 1.4.4 instalado nesta máquina traz em
>   `/usr/share/inkscape/keys/` perfis de **Illustrator** (`adobe-illustrator-cs2.xml`, cabeçalho:
>   *«in the style of Adobe Illustrator CC 2020»*), CorelDRAW, FreeHand, Xara e Zoner — e **nenhum do
>   Affinity**. Em abril de 2025 a comunidade dele testou um perfil novo, *«that mimics that of
>   Illustrator CC 2024»* ([pixls.us](https://discuss.pixls.us/t/help-test-illustrator-shortcuts-in-inkscape/49318)).
> - **O Affinity e o Figma copiam o núcleo** (`V A P N T I` — Affinity: [R1 §6](referencias/R1_affinity.md) e
>   [affinity.help](https://affinity.help/designer2/en-US.lproj/pages/Workspace/shortcuts.html); Figma: `V P T R L`, R2 §A).
>   Quem vem de qualquer um dos três encontra as mesmas letras.
> - O Inkscape nativo (`S`, `N`=nós, `F1`…) é o único fora do idioma — e é o que o Enio gosta menos.

As teclas de ferramenta abaixo vêm do perfil instalado (`grep tool-switch
adobe-illustrator-cs2.xml`: `v a p t m l b n i k`) e, para as que ele não cobre, de
[hongkiat](https://www.hongkiat.com/blog/adobe-illustrator-keyboard-shortcuts/) (`Shift+M`, `Shift+W`,
`C`, `W`):

| o nosso `DrawMode` | tecla (Illustrator) | a ferramenta do Illustrator | nota |
|---|---|---|---|
| `Select` | `V` | Selection | |
| `Node` | `A` | Direct Selection | |
| `Pen` | `P` | Pen | |
| `Pencil` | `N` | Pencil | |
| `Shape` | `M` (rectângulo) · `L` (elipse) | Rectangle · Ellipse | as duas abrem o mesmo `Shape` com a forma do catálogo já escolhida |
| `Text` | `T` | Type | ✅ já existe |
| `Build` | `Shift+M` | Shape Builder | |
| `Width` | `Shift+W` | Width | |
| `Bucket` | `K` | Live Paint Bucket | |
| `PickBlend` | `W` | Blend | |
| `Cut` | `C` | Scissors | ⚠️ a tesoura do Illustrator corta num PONTO; o nosso `Cut` desenha uma LÂMINA (é a Knife, que no Illustrator não tem tecla). A tecla é da mesma família de verbo |
| `Frame` | `Shift+O` | Artboard | o contêiner mais próximo |
| conta-gotas | `I` | Eyedropper | |
| `Fillet` · `Chamfer` · `Trim` · `Connect` | — | (o Illustrator faz cantos pelos *Live Corners* do Direct Selection, sem tecla) | ficam na barra ①, no radial e no `Ctrl+K` — ⛔ **não se inventa tecla** fora do idioma |

E as do traço e da cor, do mesmo idioma: `X` alterna Fill/Stroke activo · `Shift+X` troca as cores ·
`D` repõe o padrão (as amostras em baixo da barra ①).

⛔ **O conflito, resolvido pela decisão:** hoje `U` `I` `D` `X` são as **booleanas**
(`keyboard_cadeia.rs:343-346`), e no Illustrator `I` é o conta-gotas, `D` o padrão de cores e `X` a
troca Fill/Stroke. ⇒ as booleanas mudam para **`Ctrl+Alt+U/I/D/X`** (o idioma do Penpot; o
Illustrator não tem tecla de fábrica para o Pathfinder) e ficam também na barra ④.

⚠️ **Duas divergências que FICAM, por decisão anterior do dono:** `Tab` é o MODO (Object ↔ Edit,
05/10) — no Illustrator esconde os painéis; e `Espaço` = mão, `Z` = zoom já coincidem.

## §6 — O que cada lugar recebe no PH2D (a ponte para o plano)

| lugar | conteúdo | origem no painel de hoje |
|---|---|---|
| ① **barra de ferramentas** (esquerda) | 16 ferramentas por família, com ícone, flyout, dica rica, Fill/Stroke em baixo | secção `Tool` (os 16 botões de texto) |
| ② **barra contextual** (a fila que já existe em cima) | opções do GESTO + Fill/Stroke/largura da seleção + encaixe à direita | `Shape` (catálogo → flyout), `Pencil`, `Symmetry`, as fileiras por modo da `Tool` (marquee, corte), `Snap` |
| ③ **propriedades** (direita) | o que a SELEÇÃO é: Transform, Aparência, Fill, Stroke, específicos do tipo, Efeitos, e o resto recolhido | as outras ~30 secções |
| ④ **ações da seleção** (perto dela) | booleanas, agrupar, duplicar, apagar, converter, alinhar | `Boolean`, `Arrange`, `Path`, `Align` |
| ⑤ **rodapé** (a pílula que já existe) | dica da ferramenta + modificadores + coordenadas + encaixe + zoom | hoje: estatística de motor |

O detalhe de cada mudança está no plano.
