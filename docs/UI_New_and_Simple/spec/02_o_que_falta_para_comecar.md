# O que falta para começar a implementação da UI nova (2026-08-30)

> Pergunta do Enio, depois de fechadas as nove decisões.
>
> ⭐ **Resposta curta: nada BLOQUEIA o começo — mas a ordem não é livre**, e há duas obras que
> não são de UI e que travam partes dela. Este documento é a ordem, com o que cada degrau
> desbloqueia e o que ⛔ não se pode fazer antes de quê.

---

> ⚠️⚠️ **AUDITADA CONTRA O CÓDIGO em 2026-09-10** (`line/UIUX`). Quatro linhas desta página
> descreviam trabalho **já pago** ou uma decisão **já tomada**, e uma quinta estava **invertida**.
> Estão corrigidas em cima, cada uma com a medição dentro. ⛔ *O que se perde ao não reconferir não
> é tempo, é construir o que já existe* (`CLAUDE.md` §5.0) — e uma delas mandava construir
> exactamente o gesto que o dono ordenou **retirar**.
>
> | linha | era | é |
> |---|---|---|
> | §3 — degrau `E` (`allowed_slots` + a fuga do gizmo) | «inclui remover a fuga» | ✅ `26/26` declaram; a fuga ficou **inerte por construção** e FICA, pelas janelas flutuantes |
> | §3 — degrau `I` (temas `4 → 2`) | «cortar 4 → 2» | ⛔ **INVERTIDO**: são **8** hoje, e o menu do topo mostra **uma família por aparência** |
> | §5.1 — partir o `DrawMode` | «14 variantes vivas» | ⏳ aberta, e são **17** — o achatamento cresce sozinho |
> | §8 — um gesto de RECOLHER | «não existe … decisão de produto» | ✅ **decidida CONTRA** pelo dono (2026-09-09); a w49 removeu o gesto |
> | §8 — a fila de ferramentas dobra | «`54 → 108 px`» | ✅ **curada** na entrega 32 (`tool_bar::bar_split`, o `⋯`) |

## §1 — A restrição que molda tudo, e ela é um número

**2 076 ids de widget · 25 painéis · 53 widgets primitivos · 67 gates**
(recontado 2026-08-30 — eram 2 073 antes desta linha acrescentar três).

⛔ **Uma superfície desse tamanho não se redesenha à mão.** Toda proposta que exija tocar cada id
morre na aritmética. ⇒ **a UI nova tem de NASCER DE UMA TABELA**, e isto não é preferência: é o
único achado do repo com placar limpo — *"o único painel **42/42 limpo** na caça aos knobs mortos
é o gerado por TABELA — um painel derivado de uma tabela não tem onde esconder um knob morto"*
(`CLAUDE.md` §5.0).

⚠️ **Consequência prática:** o primeiro código da UI nova não é um painel bonito. É o **descritor**
de que os painéis passam a ser derivados.

---

## §2 — O que pode começar HOJE, sem esperar por nada

| # | obra | por que não depende de nada | estado |
|---|---|---|---|
| **A** | **O modelo de ÁREAS** — `Slot` enumerado (6), `Area`, `Region`, e o que um painel **declara** (`allowed_slots`, `can_float`, `layout_key`) | ⭐ É a peça em que as nove decisões convergem, e é foundational puro | ✅ **FEITO** |
| **B** | **Fundir os 16 apelidos de cor** (`timeline-*` → os slots gerais) | **Mudança de zero pixels** | ✅ **FEITO (2026-09-07)** — e a ordem partiu-se em duas: os **valores** fundiram, os **nomes** ficaram |
| **C** | **A barra de menus** — dar um sítio aos 148 itens que já existem | Os handlers já existem. É **realojamento**, não construção | ✅ **FEITO** |

### ⛔⛔ AUDITORIA (2026-09-01) — duas destas linhas mandavam construir o que JÁ EXISTE

⚠️ **A `A` está feita, com os dois gates que a spec §3 pedia.** `Slot` (6 variantes) + `SlotSet` +
`Panel::ALLOWED_SLOTS` / `DEFAULT_SLOT` / `CAN_FLOAT` / `TITLE` vivem em
[`screens/slot.rs`](../../../crates/ph2d-editor-core/src/screens/slot.rs) e
[`panel/panel_trait.rs`](../../../crates/ph2d-editor-core/src/panel/panel_trait.rs), e os gates são
`every_panel_is_born_inside_its_own_allowed_slots` e `the_slot_a_panel_declares_is_where_it_paints`.
⭐ O `DEFAULT_SLOT` chegou a ter default e ele foi **retirado por medição**: 20 dos 21 painéis
herdavam-no e **três mentiam**.

✅ **E a `B` FOI FEITA em 2026-09-07, por ordem do dono** (*«Vamos fundir os apelidos de cor»*) —
mas a medição feita **antes** de executar partiu a ordem em duas metades que a nota não separava:

| metade | veredito | porquê |
|---|---|---|
| os **VALORES** (57 escritos à mão: 16 `forge` · 16 `sunstone` · 16 `blueprint` · 9 `workshop`) | ✅ **fundidos** | duplicação a sério, mantida em sincronia à mão; um deles podia divergir sem ninguém ver |
| os **NOMES** (16 variantes de `ColorToken`) | ⛔ **ficam** | cada um é uma linha **regulável** no painel *Tokens* (`ColorToken::ALL`) e um destino de vínculo no editor vetorial — apagá-los tirava ao artista 16 controlos que ele tem hoje |

⭐ **E o estado da arte, que o dono mandou consultar, diz o mesmo.** O manual do Blender (CC-BY-SA):
*«The colors for each editor can be set separately by simply selecting the editor you wish to
change»* — uma secção de cor por editor. O `theme_modern.cpp` do Godot (MIT) escreve cor por
CONTROLO (`font_color` de `Tree`, de `Button`, …). *As duas referências mantêm tokens de componente
e expõem-nos; o que elas não fazem é escrever o valor deles à mão.*

⚠️⚠️ **E metade da ordem já estava feita sem ninguém saber:** a família moderna deriva tudo desde a
wave 1, logo ali os 16 nunca tiveram valor próprio. *A nota de 30/08 descrevia um mundo que a wave
1 já tinha mudado* — §0.0: quem move o número que tornava algo inalcançável tem de reconferir a
nota. O que sobrava de duplicação real era só a família clássica.

Hoje um apelido **declara pai** ([`ColorToken::alias_parent`](../../../crates/ph2d-tokens/src/color.rs))
e a fábrica resolve através dele: não há valor a escrever, logo não há valor que possa divergir.
Prova de zero-pixel contra o ficheiro ANTIGO: **64 pares, 0 divergências**. Gate:
[`an_alias_has_no_value_it_has_a_parent`](../../../crates/ph2d-tokens/tests/an_alias_has_no_value_it_has_a_parent.rs).

⭐⭐ **A lei que isto deixa:** *quando uma linha de plano e o GATE que a mede discordam, manda o
gate* — ele foi escrito por quem mediu, e a linha do plano é o resumo que envelhece.

### ⛔ E a `E` também já está resolvida

A metade *«remover a fuga do gizmo»* está **inerte por construção** desde que a área de desenho
deixou de ser a janela (degrau `D`): uma coluna docada já não lhe toca a aresta. ⚠️ **A fuga FICA de
propósito** — o que ainda a alcança são as janelas que declaram `CAN_FLOAT`, e apagá-la poria o
gizmo por baixo de uma delas. O mecanismo está escrito no sítio
([`render_loop/mod.rs`](../../../shells/desktop/src/render_loop/mod.rs), a montagem dos obstáculos).

---

## §3 — A ORDEM, e as travas que ela tem

```
    A. modelo de áreas ──┬── D. a régua e o trilho viram REGIÕES da área
                         │      (cura a foto 1 — 86,8 % tapada)
                         │
                         ├── E. painéis declaram allowed_slots
                         │      (cura a foto 2 — e ⚠️ REMOVE a fuga do gizmo
                         │       no MESMO trabalho, senão vira remédio duplo)
                         │
                         └── F. Layouts (as 8 abas) + o cabeçalho por área
                                │
    C. barra de menus ──────────┴── G. esvaziar os painéis
                                       (cura a foto 3 — 66 de 74 entradas
                                        do painel medido têm outro dono)

    B. fundir os 16 apelidos ─── H. separar LAYOUT de PALETA
                                       │
                                       └── I. cortar os temas 4 → 2   ⛔ INVERTIDO (topo)
```

### ⛔⛔ As travas duras — eram três, **são duas**

1. ~~**`I` depois de `H`.** O tema `blueprint` é hoje o **único** que liga
   `PanelLayout::Sidebar`.~~ ⛔ **REFUTADA em 2026-08-30, pela medição que devia tê-la sustentado
   desde o início:** `PanelLayout` **não tem um leitor de produção** — só a declaração, o
   produtor, o re-export e o `#[test]` do próprio ficheiro; `screens/layout.rs` nem importa o
   tipo. Não há modo ancorado a perder. Mecanismo e a lição
   ([`medicoes/03 §5`](../medicoes/03_o_censo_de_cor.md)): *confirmar que um valor é PRODUZIDO
   não é medir que ele é CONSUMIDO.* ⇒ **`I` não depende de `H`**; depende do veredito do Enio
   sobre quantos temas quer.

   ⛔⛔ **E o degrau `I`, tal como está escrito, está INVERTIDO** (medido 2026-09-10): são **`8`**
   temas hoje (`Forge` · `Workshop` · `Sunstone` · `Blueprint` · `Dark` · `Gray` · `Light` ·
   `Oled`), não `4`. O redesenho **acrescentou** os quatro presets do Godot 4.6 em vez de cortar os
   quatro de sempre. ⚠️ **E a pergunta mudou de forma com isso:** o menu da barra do topo mostra
   **uma família por aparência** (`screens/hero/menu_rows.rs`), logo o artista nunca vê `8` — vê
   `4`. ⇒ *«cortar 4 → 2» respondia a um ecrã que já não existe*; a pergunta viva é se os quatro
   **clássicos** sobrevivem ao dia em que o interruptor `PH2D_UI_NEW` sair, e essa é do dono.
2. ~~**`E` inclui remover a fuga do gizmo.**~~ ✅ **FECHADA, e por CONSTRUÇÃO** (medido
   2026-09-10). O `E` está inteiro: **26 de 26** painéis declaram `ALLOWED_SLOTS`. E a fuga
   deixou de alcançar uma coluna docada no dia em que a área de referência passou a ser a
   `HeroLayout::draw_area` em vez da janela — *sem uma linha da lei mudar*. ⚠️ **Ela FICA, e o
   motivo está escrito ao lado dela** (`render_loop/mod.rs`): o que ainda a alcança são as janelas
   que declaram **flutuar** (Grid Snap, galeria), e apagá-la deixaria o gizmo por baixo de uma
   delas. ⇒ *não era «remover», era «tirar do caminho» — e a diferença é quem sobra a alcançá-la.*
3. **`G` é onde a área se ganha, não em `E`.** Medido: ancorar dá **49,6 %** de canvas com tudo
   aberto — contra os 49 % de hoje. O que devolve tela é esvaziar os painéis (e recolher, que
   passa a valer: **94,4 %**).

---

## §4 — ⛔ As DUAS obras que não são de UI e que travam partes dela

Nenhuma delas é desta linha, e as duas já estão nomeadas com endereço.

| obra | trava o quê | estado |
|---|---|---|
| **A escultura virar ENTIDADE** | tudo o que é per-objecto no 3D: modos, timeline, undo por componente, instâncias, persistência | ⛔ `grep -rn 'Sculpt' crates/ph2d-ecs/` devolve **zero**. ⭐ Molde pronto: o `PaintedDoc(u32)` |
| **O 3D renderizar para TEXTURA** | o z-index da **D9.1** (3D *entre* camadas) | ⭐⭐ **É o mesmo primitivo do *W-Saída* do Flip** — *"é UM buraco, não três"*. Três consumidores: Flip · 16 exportadores · 3D-entre-camadas |

⚠️ **Elas não bloqueiam A, B, C, D, E, F, G, H nem I** — só a parte 3D da tabela de modos (D6) e o
compositor da D9. *A UI nova pode ser construída inteira antes de qualquer uma delas existir.*

---

## §5 — ⏳ As três decisões que ainda são suas

1. **Como partir o `DrawMode`.** ⏳ **ABERTA, e a contagem PIOROU: são `17` variantes vivas**
   (medido 2026-09-10 em `params_mode.rs`; eram `14` quando esta linha foi escrita — entraram
   `Trim`, `Bucket` e `Bone`). Continua a não se exprimir *«Edit + ferramenta Fillet»*. ⚠️ É a peça
   que faz a D6 (modos) e o terceiro eixo (ferramentas) existirem de facto no vetor — sem ela a
   tabela de modos é verdade no papel. ⛔ **E o achatamento cresce sozinho**: cada ferramenta nova
   do vetor acrescenta uma variante, logo esperar torna a decisão mais cara, nunca mais barata.
2. **Adoptamos o campo `Mode` do Workspace?** (*"switch to this Mode when activating"* — o atalho
   do Blender que liga Layout e Modo sem os acoplar.) Uma linha de modelo, e resolve *"o layout
   Escultura põe-me em modo Sculpt"*.
3. **Os 9 toggles de módulo → Layout.** Hoje são interruptores **independentes** (2⁹ combinações);
   um Layout é *um-de-N*. Converter um no outro é uma decisão de produto sobre o que acontece às
   combinações que ninguém desenhou.

---

## §6 — O que esta linha entregou hoje, e que já é parte da UI nova

✅ **A unidade de ângulo** (Settings → *Angle unit*), a irmã da de comprimento —
`DisplayAngle { Degrees, Radians }`, persistida, com 4 gates e 3 provas de mutação.
`PROJECT_SCHEMA` **103 → 104**.

⭐ **Ela é pequena de propósito, e serve de molde para as próximas:** mostrou que o padrão de um
knob novo neste app tem **oito** pontas (id · registo · linha de menu · variante de
`ContextMenuKind` · handler · dispatch gerado · campo de runtime · campo de ficheiro), que os
gates de LOC cobram o corte quando ele cresce, e que **o censo de obsolescência desce a catraca
sozinho** quando o corte acontece.

⚠️ **E cobrou duas correcções minhas** — uma afirmação de precisão que o gate refutou (delegar o
estreito ao largo custava **1 ULP**), e dois tetos de LOC que eu estourei e curei por corte, não
por excepção.

---

---

## §6-bis — ✅ O degrau `D` FECHOU (2026-08-30): as réguas são regiões

O primeiro degrau do modelo de áreas está no `main` da linha. `HeroLayout::draw_area` é o que
sobra da janela depois de o chrome **docado** tirar a sua faixa, e as duas réguas passaram a ser
regiões dela.

| | antes | depois |
|---|---:|---:|
| régua da esquerda tapada | **86,8 %** | **0,0 %** |
| régua de cima tapada | **29,4 %** | **0,0 %** |

⭐ **Custou menos do que o §3 previa, e a razão é medível:** `ruler::top_band`/`left_band`/`hit`
já recebiam um rect por argumento, `ruler::in_band` já filtrava os traços fora da faixa, e a
**projecção deriva de `window_w`/`window_h`, nunca do rect** (`grid::world_bounds`). ⇒ a régua
mudou-se **sem levar a projecção consigo** — um traço marcado em 100 continua a cair no mesmo
pixel. *Uma régua que se mudasse e apontasse para outro sítio seria pior do que uma tapada.*

### ⛔⛔ E a wave achou um SEGUNDO defeito, de INPUT, que nenhuma sonda deste repo via

A régua **não está no `HitIndex`**. O gesto da guia é geométrico (`ruler::hit`) e corre em
`input_dispatch.rs` **antes** do hit-test de chrome, com um `return` quando acerta. Enquanto o
hospedeiro foi a viewport inteira:

- um press nos **6 px de cima de qualquer botão da barra** (banda `y ∈ [0,20]`, barra em `y=14`),
- ou nos **3 px da esquerda de qualquer chip do trilho** (banda `x ∈ [0,20]`, chip em `x=17`),

**nascia uma guia em vez de carregar no botão** — só em modo Vector (`rulers_live` exige
`panel_visible("vector")`). ⚠️ Os gates de costura do repo perguntam todos ao `HitIndex`, onde a
régua não está: por isso 15 gates verdes conviviam com isto. *Uma superfície que faz hit-test
FORA do índice é invisível a toda a família de portões que o repo tem.*

### ⭐ O preço da `E`/`A2` ficou MEDIDO, e é maior do que o §3 dizia

O §3 tratava `A` como uma peça só. Ela são duas, e a segunda é cara:

- **As regiões de CHROME** (feito) — rects, barato.
- **A CENA mudar-se para dentro da região** — o sub-rectângulo da cena é ancorado em `(0,0)`
  **por construção em toda a cadeia**: `CenterSplit::scene_viewport` devolve `[0, 0, w, h·t]` e
  **todo** consumidor lê só as DIMS (`field_gizmo::scene_window_wh` → `scene_camera_window` →
  `pan_scene_camera`, mais `uniform_for_subrect`/`set_viewport` no render). ⇒ **dar origem à cena
  é uma mudança na porta única do mundo↔tela**, não um rect. É a obra da docagem, e é onde o
  orçamento da `E` de facto está.

⚠️ Enquanto isso não acontecer, um painel **arrastado à mão** continua a poder tapar a régua: o
que a wave cura é o chrome **docado**. A cerca que o proíbe é a `allowed_slots`/`can_float` da D1
(a `E`), não esta.

---

## §7 — Recomendação de arranque

⭐ **Começar por `A` (o modelo de áreas), e por uma fatia fina dele:** os seis encaixes + o
descritor de painel + **um** painel convertido (o mais simples), com o gate que prova que um
painel de propriedades **não consegue** ser posto sobre a viewport.

⛔ **Não começar pelo desenho visual.** A folha de estilo é a parte fácil e a que se refaz; o que
não se refaz barato é a **derivação** — e o §1 diz que sem ela a migração de 2 076 ids não fecha.

⏳ **E fazer `B` em paralelo, num commit próprio** — é zero-pixel, provável, e tira 16 slots de cor
de circulação antes de alguém construir sobre eles.


---

## §8 — ⛔⛔ A RESTRIÇÃO DE ECRÃ (2026-08-31), e o que ela fecha

> Enio: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*

Medido nos três tablets ([`medicoes/06`](../medicoes/06_o_orcamento_de_ecra_em_tablet.md)):

| alvo | colunas abertas | a pintar | colunas fechadas |
|---|---:|---:|---:|
| iPad 12.9 | 50,8 % | 50,8 % | 92,0 % |
| iPad 11 | 44,0 % | **40,8 %** | 90,2 % |
| iPad mini | 40,9 % | **37,6 %** | 89,0 % |

### O que isto muda na ordem deste documento

| degrau | estado |
|---|---|
| **`F` — cabeçalho por área** | ✅ **RESOLVIDO na entrega 33, sem faixa nova**: o inquilino é um **pulldown** no fim da fila de ferramentas, cuja face é a leitura do estado. A faixa (entrega 30, revertida na 31) custava `28 px` permanentes; esta custa `0`. ⛔ **UM** chip e não nove — com os nove crus a fila vai a **2 linhas até no iPad 12,9"** (medido por mutação) |
| **`G` — esvaziar os painéis** | ⭐⭐⭐ **o `3D Model` FECHOU** (entregas 33 + 35): perdeu **17 das 74** entradas — as vistas e a câmera no 1.º pulldown (*View*), os verbos do gizmo e o referencial nos chips **`MOVE`/`ROT`/`SCALE`/`SPACE` que já existiam** (⛔ eles estavam **mortos**, não ausentes — Enio, 01/09), e a exportação no menu global **File**. ⚠️ O `add.*` (20 na tabela da D2) **já estava fechado** pela paleta de formas da W100 — a tabela é que não sabia. ⏳ Sobram as **operações booleanas** e as **acções**, sem destino decidido; o `verb`/`kind` **ficam no painel** (são propriedade do objecto, pelo critério da própria D2). ⛔⛔ **Nenhum outro painel foi censado, e a razão é um INSTRUMENTO que falta, não desleixo** (medido 2026-09-10): o censo classifica as **entradas declaradas** de um painel, e `19` de `26` não declaram **nenhuma** — os rótulos deles são literais dentro do pintor. Entre os cegos estão `hierarchy`, `inspector`, `painter_layers` e `motion_graph`, que o artista tem abertos o dia inteiro. ⇒ **este degrau e o buraco do HR-15 são o MESMO item**, e a ordem de cura dos literais passa a ser «o mais aberto primeiro» — [`medicoes/07 §3-bis`](../medicoes/07_o_buraco_do_hr15_o_texto_PINTADO.md). O `66 de 74` é do `3D Model`. Recolher as duas colunas dá `89–92 %` |
| ✅ **um gesto de RECOLHER** | ⛔⛔ **DECIDIDO pelo dono em 2026-09-09, e a decisão foi CONTRA:** *«Vamos retirar a opção de colapsar arrastando. Deixa o colapsar apenas no menu da barra superior»* — depois de o gesto existir (arrastar a borda para dentro) e de ele **prender o app por um minuto** ao ser combinado com o menu. A w49 removeu-o e unificou o piso da largura. ⇒ recolher é, por ordem, **dois itens de menu** — e esta linha não se reabre sem ele. Ver [`medicoes/09 §4`](../medicoes/09_o_penhasco_com_todos_os_paineis_abertos.md) |
| ✅ **a fila de ferramentas** | **CURADA na entrega 32, pela cura que esta linha já escolhia:** a faixa é **sempre uma linha** e o que não cabe vive atrás do `⋯` (`tool_bar::bar_split`). Medido: `+3,2` pontos de área no iPad 11 e `+3,3` no mini, e a coluna «com pincel» deixou de ser o pior caso. ⚠️ **Foi o TECTO de obsolescência do gate que obrigou a actualizar os números** (`40,8 → 44,0` disparou-o) — [`the_chrome_never_eats_more_of_a_tablet_than_this`](../../../crates/ph2d-editor-core/tests/the_chrome_never_eats_more_of_a_tablet_than_this.rs) |

### ⛔ E a largura do chrome não escala — o que agrava, não alivia

As duas colunas são `612 px` **absolutos**: `44,8 %` da largura no 12,9" e **`54,0 %`** no mini.
O [`spec/01 §6`](01_modelo_de_areas.md) declara que a largura do encaixe **não escala com o alvo de
toque** — e não dizia nada sobre ecrãs mais pequenos, onde o mesmo número custa mais.

⚠️ **Toda proposta de UI nova passa a nomear o que custa em ALTURA e em LARGURA, nos três alvos.**
O gate que o exige é `the_chrome_never_eats_more_of_a_tablet_than_this`.
