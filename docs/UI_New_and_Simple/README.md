# UI_New_and_Simple — a base para redesenhar a UI/UX do PH2D

> **Etapa 1 (levantamento).** Esta pasta não contém spec ainda: contém a **medição do que
> temos**, o **inventário do que a subida do stack abriu**, e as **referências de terceiros
> baixadas** para estudo. A spec nasce da conversa com o Enio, e cada decisão dela tem de
> apontar para uma linha daqui.
>
> Aberta em 2026-08-30 pela `line/UIUX`, a pedido do Enio.

## Por que existe

O Enio abriu a etapa com um diagnóstico próprio (2026-08-30):

- A UI foi desenhada para **iPad/Wacom**, inspirada em apps de iPad, e *"não creio que fiz as
  melhores escolhas: não é tão fácil chegar ao resultado desejado."*
- Quer **mais simplicidade**, **menos cores na paleta**, **docks/painéis mais sólidos** e
  fáceis de manter.
- Referências: **Godot** primeiro, **Blender** depois.
- ⭐ **A exigência de método:** *"Precisamos de especificações existentes claras, bem definidas
  e documentadas, já testadas em outros apps"* — partir de espec de projeto open source, não
  inventar.
- Princípios de **design de interação** (Rogers/Sharp/Preece) como fundo teórico.
- Precisaremos de **troca de Modos e Layouts como no Blender**, porque as nossas ferramentas
  são *"apps completos aninhados"*.

E três defeitos concretos, com foto:

1. **Painéis flutuantes tapam as réguas** (foto 1).
2. **Painéis por cima das vistas 3D**; o **gizmo de navegação foge para o centro** para
   continuar acessível (foto 2).
3. **Sem menus na barra superior, os painéis incharam** e ficaram mal organizados.

## O roteador

| Você quer | Leia |
|---|---|
| ⛔ **O que o Enio já DECIDIU** (não re-litigar) | [`00_DECISOES_DO_ENIO.md`](00_DECISOES_DO_ENIO.md) |
| ⭐ **A spec — o modelo de áreas** (rascunho 1) | [`spec/01_modelo_de_areas.md`](spec/01_modelo_de_areas.md) |
| ⭐⭐ **O que falta para COMEÇAR, e em que ordem** | [`spec/02_o_que_falta_para_comecar.md`](spec/02_o_que_falta_para_comecar.md) |
| **Quanto de UI nós temos, medido** (tokens, ids, painéis, LOC) | [`medicoes/01_o_estado_medido.md`](medicoes/01_o_estado_medido.md) |
| **Quanto do canvas o chrome tapa** — a foto 1 em número | [`medicoes/02_a_area_tapada.md`](medicoes/02_a_area_tapada.md) |
| **Quantas cores precisamos mesmo** | [`medicoes/03_o_censo_de_cor.md`](medicoes/03_o_censo_de_cor.md) |
| ⭐ **O que as timelines alcançam — e por que o 3D fica de fora** | [`medicoes/04_o_alcance_das_timelines.md`](medicoes/04_o_alcance_das_timelines.md) |
| **px/metros e graus/radianos — metade já ship-a** | [`medicoes/05_as_duas_reguas.md`](medicoes/05_as_duas_reguas.md) |
| **O que a subida Vello 0.8→0.10 / wgpu 28→29 / parley 0.6→0.11 abriu** | [`pesquisa/01_o_que_a_subida_abriu.md`](pesquisa/01_o_que_a_subida_abriu.md) |
| **Que referências existem, com a licença de cada uma** | [`pesquisa/02_referencias_e_licenca.md`](pesquisa/02_referencias_e_licenca.md) |
| **O diagnóstico das 3 fotos + os princípios que o explicam** | [`pesquisa/03_diagnostico_e_principios.md`](pesquisa/03_diagnostico_e_principios.md) |
| ⭐ **Modo vs Layout vs Ferramenta — os TRÊS eixos** | [`pesquisa/04_modo_layout_e_ferramenta.md`](pesquisa/04_modo_layout_e_ferramenta.md) |
| **«Pintar sobre vetor» — metade já existe** | [`pesquisa/05_pintar_sobre_vetor.md`](pesquisa/05_pintar_sobre_vetor.md) |
| ⭐⭐ **A engine é 2.5D — e o que isso resolve** | [`pesquisa/06_a_engine_e_2_5d.md`](pesquisa/06_a_engine_e_2_5d.md) |
| ⭐⭐⭐ **Redesenhar os widgets: plano, minimalista e COMPACTO** (o orçamento de 154 px, a lei do estreito, os 44 widgets) | [`pesquisa/07_o_redesenho_dos_widgets.md`](pesquisa/07_o_redesenho_dos_widgets.md) |
| ⭐⭐⭐ **O MODELO a seguir, com código completo** — por que o app ficou com a mesma cara, os 13 candidatos com licença, e a recomendação (Godot 4.6 «Modern», MIT) | [`pesquisa/08_modelos_com_codigo_para_seguir.md`](pesquisa/08_modelos_com_codigo_para_seguir.md) |
| **Baixar as referências numa máquina nova** | `bash fetch-referencias.sh` |

## ⭐⭐⭐ ONDE ESTAMOS (auditado contra o CÓDIGO em 2026-09-04)

> ⚠️ **Esta secção é o placar, e as de baixo são história.** Ela foi escrita a medir o código, não
> a ler as outras secções — que estavam paradas em 30/08. ⛔ **Ao actualizá-la, MEÇA outra vez:** a
> auditoria de hoje achou **uma** linha de pendências que mandava construir o que já shipa
> ([`pesquisa/07 §23`](pesquisa/07_o_redesenho_dos_widgets.md)).

### Os degraus do plano de arranque ([`spec/02 §3`](spec/02_o_que_falta_para_comecar.md))

| | degrau | estado |
|---|---|---|
| **A** | modelo de ÁREAS (`Slot`, `allowed_slots`, `can_float`) | ✅ **feito**, com os 2 gates que a spec pedia |
| **B** | fundir os 16 apelidos de cor | ⛔ **construído e REVERTIDO** — equivalência re-medida (0/64 divergências), mas a pergunta é de *design system*: **veredito do Enio** |
| **C** | a barra de menus | ✅ **feito** — ⚠️ e em 04/09 apanhou o `Export SVG…`, que a substituição da superfície tinha deixado sem porta |
| **D** | régua e trilho viram REGIÕES da área | ✅ **feito** — tapada `86,8 % → 0 %` (esq.) e `29,4 % → 0 %` (topo) |
| **E** | painéis declaram onde podem viver · fuga do gizmo | ✅ a metade do chrome **docado**; ⏳ **falta a metade cara**: dar **ORIGEM à cena** (hoje o sub-rectângulo dela é ancorado em `(0,0)` por construção em toda a cadeia). Sem ela, um painel **arrastado à mão** ainda tapa a régua |
| **F** | Layouts por tarefa + cabeçalho de área | ✅ **6 dos 8** (`TaskLayout`, abas, persistência, e o `canvas` que cada layout NOMEIA); o cabeçalho é um **pulldown** que custa `0 px`. ⛔ *Código* e *Runtime* estão bloqueados por outros (não há editor de texto; `shells/game`/R1 adiado) |
| **G** | **esvaziar os painéis** | ⏳ **a maior obra aberta: 1 painel de 25 censado.** O `3D Model` perdeu `17` das `74` entradas; ⛔ **nenhum outro foi medido** — o «66 de 74» é só dele |
| **H** | separar LAYOUT de PALETA | ⛔ **a trava não existe**: `PanelLayout` não tem leitor de produção (medido) |
| **I** | cortar os temas `4 → 2` | ⏳ **veredito do Enio** (hoje: Forge · Workshop · Sunstone · Blueprint) |

⏳ **E duas que a restrição de ecrã abriu e ninguém pegou:** um **gesto de RECOLHER** as colunas
(hoje são dois itens de menu, e recolher dá `89–92 %` de tela) · a **fila de ferramentas DOBRA**
(`54 → 108 px`) no iPad 11 e no mini com o pincel em mãos.

### O redesenho dos widgets ([`pesquisa/07 §15.2`](pesquisa/07_o_redesenho_dos_widgets.md))

`1` caixa de verificação ✅ · `2` interruptor→caixa ✅ · `3` pílulas fora ✅ ·
`4` **ritmo da linha** ⏳ *decisão do Enio* · `5` scrollbar fina ⛔ **recusada por medição** (e o
que ela revelou — arrastar o CORPO para rolar — ✅ feito) · `6` coluna de animação ✅ **desenhada**
(⏳ **não põe chave**: falta o consumidor) · `7` 4.º preset de fonte ⛔ **construído e revertido**.

⭐⭐⭐ **O MODELO ESTÁ ESCOLHIDO E A WAVE 1 ESTÁ NO CÓDIGO (04/09):** Godot 4.6 «Modern» (MIT), o
cinza `#292929` e o azul `#569eff` dele, quatro presets da tabela dele (`Dark` · `Gray` · `Light` ·
`Black (OLED)`) — cada tema **derivado de cinco entradas**, nenhum slot escrito à mão; a tabela de
estados do widget (forma egui) e os quatro pintores de cromo (painel · secção · botão · campo) a
lê-la: **moldura zero, raio 4**. O redesenho abre no `Dark`; `PH2D_UI_NEW=0` devolve o clássico
intacto. ⭐ **E a wave 2 (05/09) fez da moldura uma PORTA** (`visuals::frame` / `paint::stroke_frame`):
24 pintores convertidos, os 22 que faltam nomeados numa catraca, e um gate de PIXEL a provar que
o tema moderno emite menos geometria. ⭐ **E a wave 3 (05/09) pôs a catraca a ZERO** — os 22
passaram pela porta (20 convertidos, 2 isentos por mecanismo), e o modo *Image Tools* ligado
deixou de ser um anel traçado por cima do chip (que sumiria no moderno) para ser o chip a
pintar-se activo pela matriz do rail. ⭐⭐ **E a wave 4 (05/09) levou a porta aos PAINÉIS** — onde
o artista vive: 59 ficheiros traçavam moldura à mão e nenhum conhecia a porta; hoje o censo varre
as crates de painel e a shell, 72 sítios passam por ela e 7 ficam isentos pelo mecanismo (o
contorno que É a mensagem). O vocabulário ganhou `Selected` — a selecção entre iguais sem tinta,
que o Godot Modern traça a 2 px em `mono` no nó do grafo — porque três anéis eram o único sinal
de um estado. ⚠️ **E o smoke dela devolveu dois defeitos (05/09), os dois curados**: o navegador de
**Assets** não tinha porta (era só um chip da barra legada; hoje é a linha *Assets* do menu
*Window*, com o censo de alcance a cobrir também os botões directos), e os **cartões estavam a
4/255 do painel** — a wave 1 tinha portado as regras do Godot sem a pilha de superfícies dele; a
escada reassentou-se com o painel na `base` e os cartões em `surface_high` (Dark `#292929 →
#393939`), e o texto secundário e o acento do *Light* passam a ser derivados até à lei de contraste.
Mecanismo e o que ficou de fora: [`pesquisa/08 §7`](pesquisa/08_modelos_com_codigo_para_seguir.md).

⏳ **O que sobra do estudo §5.3, medido em 04/09:** cantos dos painéis a `16 px` (o estudo diz `4`) ·
cartões com moldura · caixas de texto com moldura permanente · etiquetas e amostras ainda pílulas ·
esbatimento do rótulo e inércia da rolagem. ⛔ **As secções JÁ recolhem** (10 painéis, animado) —
a lista dizia o contrário.

## ⚠️ `referencias/` é gitignorada — e isso é a decisão, não um esquecimento

O payload (41 MB de repositórios de terceiros, com licenças alheias) **não entra no git do
PH2D**. O que entra é [`fetch-referencias.sh`](fetch-referencias.sh), que o reconstrói em
qualquer máquina, com a licença de cada alvo escrita ao lado.

É o mesmo precedente de `docs/Pixel Art/` e `docs/Tilling/` (gitignoradas por decisão do Enio),
e obedece à lei do repo — *uma ferramenta fora do repo não existe nas outras máquinas*: o
**script** é versionado exatamente para que a pasta seja reconstruível, e não é o payload que
viaja.

## ⛔ A triagem de licença vem ANTES de qualquer leitura de fonte

Detalhe em [`pesquisa/02_referencias_e_licenca.md`](pesquisa/02_referencias_e_licenca.md).
O resumo que decide o que podemos fazer:

| Alvo | Licença | O que podemos fazer |
|---|---|---|
| **Godot** (motor + editor) | **MIT** | ⭐ ler o código **e portá-lo**. É a porta permissiva. |
| **Godot docs / contributing** | CC-BY 4.0 | ler e citar |
| **Blender — HIG e manual** | **CC-BY-SA 4.0** (é *documentação*) | ⭐ ler e citar livremente |
| **Blender — código C/C++** | **GPL** | ⛔ **não é lido nesta linha.** Só comportamento observável |
| **GNOME HIG** | CC-BY-SA 4.0 | ler e citar |
| **Adobe Spectrum (design data)** | **Apache-2.0** | ⭐ ler os tokens **e usá-los** como base |
| **Apple HIG / Unity** | proprietário | ler online e resumir; ⛔ nada entra na árvore |

*O Blender entra aqui como **documento**, nunca como fonte. É por isso que a pasta baixa
`blender-developer-docs` e não `blender`.*

## Estado (2026-08-30) — ⚠️ **HISTÓRICO: o placar vivo é «ONDE ESTAMOS» acima**

✅ **Etapa 1 fechada** e **nove decisões tomadas pelo Enio**
([`00_DECISOES_DO_ENIO.md`](00_DECISOES_DO_ENIO.md)): painéis **ancorados com flutuação
declarada** · comandos em **barra global + cabeçalho por área** · **Layouts por tarefa** (e
⚠️ **Modos são per-objecto** — D3 corrigida por ele) · **encaixes fixos** · **a régua entra na
área de desenho** · a **tabela de modos por tipo de objecto** (D6, com o Flip a ganhar `Draw`
próprio) · os **8 Layouts** (D7) · **as timelines em todos os modos** (D8) · e ⭐⭐ **a engine é 2.5D**
(D9): canvas 2D, com objectos 3D desenhados **entre as camadas** dele — cena em **metros**, arte em
**pixels**, e ⭐ **as duas réguas escolhíveis** (px/m e graus/rad, D9.2).

Os dois pré-requisitos que o §6 do estado nomeava **foram medidos**:
- **51,0 % do canvas é chrome** no viewport de referência — que é o **iPad Pro 12,9"**. A régua da
  esquerda está **86,8 % tapada**, e ⚠️ **por causa do rail, não dos painéis**.
- **16 dos 83 slots de cor são apelidos puros** — todos os 16 `timeline-*`. Fundi-los é uma
  mudança de **zero pixels**.

✅ **Rascunho 1 da spec escrito** — [`spec/01_modelo_de_areas.md`](spec/01_modelo_de_areas.md):
**seis encaixes** (número *derivado*: os 12 do Godot dariam 89,6 % da largura no nosso alvo),
o que um painel **declara** (portado do `EditorDock` MIT), as **regiões** de uma área, e o
orçamento medido — **94,4 % de tela com os lados recolhidos**, contra ⛔ **49,6 % com tudo aberto,
que não é melhor do que hoje**.

⏳ **Próximo — as três que ficaram, e as três são do Enio:**
1. ⭐ **O 3D vira OBJECTO.** A D9 fixou que ele tem lugar e tamanho no canvas 2D; hoje os dois
   módulos 3D tomam a **janela inteira**. E a escultura tem de virar objecto da cena **antes**
   de qualquer coisa a alcançar.
2. **Como partir o `DrawMode`** nos dois eixos (2 modos + 12 ferramentas achatados em 14 variantes
   vivas, com gates).
3. **A ordem de migração** — 2 073 ids e 25 painéis.

⚠️ *A pergunta «pose 2D ou 3D?» que estava aqui foi **respondida pela D9** — e a medição que eu
oferecia para a decidir ficou cancelada com ela.*

## ✅ A implementação COMEÇOU (2026-08-30) — o degrau `D` do modelo de áreas

**As réguas deixaram de partilhar coordenada com o chrome.** `HeroLayout::draw_area` é o que
sobra da janela depois de o chrome **docado** tirar a sua faixa, e as duas réguas são regiões
dela: **86,8 % → 0 %** (esquerda) e **29,4 % → 0 %** (cima). Detalhe e o preço da fase seguinte:
[`spec/02 §6-bis`](spec/02_o_que_falta_para_comecar.md).

⛔ **E a wave achou um defeito de INPUT que nenhuma sonda deste repo via:** a régua não está no
`HitIndex` e o gesto dela corre antes do hit-test de chrome — os 6 px de cima de cada botão da
barra e os 3 px da esquerda de cada chip do trilho **criavam uma guia em vez de carregar no
botão**, em modo Vector. Curado pela mesma mudança.

⚠️ **Duas notas destes documentos foram REFUTADAS pela própria medição:**
1. *«o `blueprint` é o único tema que liga `PanelLayout::Sidebar`»* — **`PanelLayout` não tem um
   leitor de produção.** A trava dura nº 1 da ordem de arranque não existe
   ([`medicoes/03 §5`](medicoes/03_o_censo_de_cor.md)).
2. *«o degrau `A` não depende de nada»* — depende, em metade: mover a **cena** para dentro da
   área exige dar-lhe uma ORIGEM, e a cadeia inteira ancora o sub-rectângulo dela em `(0,0)` por
   construção. É onde o orçamento da docagem de facto está.

## O que esta etapa NÃO fez

- ⚠️ **Deixou de ser só documentação em 2026-08-30:** a **unidade de ângulo** (Settings →
  *Angle unit*) foi implementada a pedido do Enio — `DisplayAngle { Degrees, Radians }`, persistida,
  4 gates, 3 provas de mutação, `PROJECT_SCHEMA` **103 → 104**. Ver
  [`spec/02 §6`](spec/02_o_que_falta_para_comecar.md).
- Não decidiu nada sozinha: as nove decisões são do Enio, e o que fica em aberto está nomeado
  nos `⏳` de cada documento.
- ⚠️ **A spec é RASCUNHO** — o §7 dela lista o que falta decidir, e a §8 o que ⛔ não fazer.
