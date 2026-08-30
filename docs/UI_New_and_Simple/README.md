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
| **Quanto de UI nós temos, medido** (tokens, ids, painéis, LOC) | [`medicoes/01_o_estado_medido.md`](medicoes/01_o_estado_medido.md) |
| **O que a subida Vello 0.8→0.10 / wgpu 28→29 / parley 0.6→0.11 abriu** | [`pesquisa/01_o_que_a_subida_abriu.md`](pesquisa/01_o_que_a_subida_abriu.md) |
| **Que referências existem, com a licença de cada uma** | [`pesquisa/02_referencias_e_licenca.md`](pesquisa/02_referencias_e_licenca.md) |
| **O diagnóstico das 3 fotos + os princípios que o explicam** | [`pesquisa/03_diagnostico_e_principios.md`](pesquisa/03_diagnostico_e_principios.md) |
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

## O que esta etapa NÃO fez

- Não escreveu spec. Não tocou uma linha de `crates/`.
- Não decidiu nada: as perguntas abertas estão no fim de
  [`pesquisa/03_diagnostico_e_principios.md`](pesquisa/03_diagnostico_e_principios.md), e são
  do Enio.
