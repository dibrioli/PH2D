# O que falta para começar a implementação da UI nova (2026-08-30)

> Pergunta do Enio, depois de fechadas as nove decisões.
>
> ⭐ **Resposta curta: nada BLOQUEIA o começo — mas a ordem não é livre**, e há duas obras que
> não são de UI e que travam partes dela. Este documento é a ordem, com o que cada degrau
> desbloqueia e o que ⛔ não se pode fazer antes de quê.

---

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

| # | obra | por que não depende de nada |
|---|---|---|
| **A** | **O modelo de ÁREAS** — `Slot` enumerado (6), `Area`, `Region`, e o que um painel **declara** (`allowed_slots`, `can_float`, `layout_key`) | ⭐ É a peça em que as nove decisões convergem, e é foundational puro. O rascunho está em [`01_modelo_de_areas.md`](01_modelo_de_areas.md); o molde é o `EditorDock` do Godot (**MIT — portável**) |
| **B** | **Fundir os 16 apelidos de cor** (`timeline-*` → os slots gerais) | **Mudança de zero pixels**, provada nos três temas ([`medicoes/03`](../medicoes/03_o_censo_de_cor.md)). Independente de tudo |
| **C** | **A barra de menus** — dar um sítio aos 148 itens que já existem | Os handlers já existem (40 ficheiros de chrome). É **realojamento**, não construção |

⭐ **A, B e C são paralelizáveis** — não se tocam.

---

## §3 — A ORDEM, e as travas que ela tem

```
    A. modelo de áreas ──┬── D. a régua e o trilho viram REGIÕES da área
                         │      (cura a foto 1 — 87,8 % tapada)
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
                                       └── I. cortar os temas 4 → 2
```

### ⛔⛔ As três travas duras

1. **`I` depois de `H`.** O tema `blueprint` é hoje o **único** que liga `PanelLayout::Sidebar`.
   Cortar os temas antes de libertar o layout da paleta **apaga o único modo ancorado do app**.
2. **`E` inclui remover a fuga do gizmo.** Ela é o remédio do sintoma; com os painéis fora da
   vista, passaria a fugir de uma moldura que já não a alcança.
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

1. **Como partir o `DrawMode`.** São **2 modos + 12 ferramentas** achatados em 14 variantes vivas,
   com gates. Hoje **não se exprime «Edit + ferramenta Fillet»**. ⚠️ É a peça que faz a D6 (modos)
   e o terceiro eixo (ferramentas) existirem de facto no vetor — sem ela a tabela de modos é
   verdade no papel.
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

## §7 — Recomendação de arranque

⭐ **Começar por `A` (o modelo de áreas), e por uma fatia fina dele:** os seis encaixes + o
descritor de painel + **um** painel convertido (o mais simples), com o gate que prova que um
painel de propriedades **não consegue** ser posto sobre a viewport.

⛔ **Não começar pelo desenho visual.** A folha de estilo é a parte fácil e a que se refaz; o que
não se refaz barato é a **derivação** — e o §1 diz que sem ela a migração de 2 076 ids não fecha.

⏳ **E fazer `B` em paralelo, num commit próprio** — é zero-pixel, provável, e tira 16 slots de cor
de circulação antes de alguém construir sobre eles.
