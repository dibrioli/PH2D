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
| **Baixar as referências numa máquina nova** | `bash fetch-referencias.sh` |

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

## Estado (2026-08-30)

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
  esquerda está **87,8 % tapada**, e ⚠️ **por causa do rail, não dos painéis**.
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

## O que esta etapa NÃO fez

- ⚠️ **Deixou de ser só documentação em 2026-08-30:** a **unidade de ângulo** (Settings →
  *Angle unit*) foi implementada a pedido do Enio — `DisplayAngle { Degrees, Radians }`, persistida,
  4 gates, 3 provas de mutação, `PROJECT_SCHEMA` **103 → 104**. Ver
  [`spec/02 §6`](spec/02_o_que_falta_para_comecar.md).
- Não decidiu nada sozinha: as nove decisões são do Enio, e o que fica em aberto está nomeado
  nos `⏳` de cada documento.
- ⚠️ **A spec é RASCUNHO** — o §7 dela lista o que falta decidir, e a §8 o que ⛔ não fazer.
