# ADR-0161 — A modelagem 3D é uma ÁRVORE DE CAMPO IMPLÍCITO, e o que o artista vê é o campo TRAÇADO

- **Status:** proposto — o **caráter** já foi aprovado pelo Enio no smoke de 2026-08-19
  (*"excepcional, lindo e maravilhoso"*); o registro formal aguarda o aceite.
  ⚠️ **O número foi CONTADO, não escolhido** (2026-08-19): `main` está em 0159 e a `line/sculpt3d`
  já ocupa o **0160** sem ter integrado. Número de ADR é leitura, não reserva — se outra linha
  reivindicar o 0161 na mesma janela, **renumera na integração**
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- **Data:** 2026-08-19
- **Linha:** `line/3DModeling`
- **Cofre do módulo:** [`docs/3DModeling/`](../../3DModeling/README.md)
- **Não confundir com** [ADR-0150](0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md),
  que decide a **escultura** (malha + verbos). Este decide a **modelagem** (booleana e
  arredondamento exatos). São dois módulos, e a §"Consequências" diz onde eles se encontram.

## O problema, e a força que obriga a decidir agora

O pedido do Enio não foi *"portem um modelador NURBS"* — foi, nas palavras dele:

> *"o que me atrai neste tipo de modelagem é justamente o modo eficaz de lidar com operações
> booleanas e arredondamento de arestas, coisas que não são tão fáceis com o Blender e outros apps
> de modelagem 3D que têm resultados inferiores."*

Ou seja: o alvo é o **resultado**, e a representação é meio. Isso obriga a decidir antes da primeira
linha, porque as três famílias candidatas divergem já na estrutura de dados do documento — e trocar
depois é reescrever o módulo.

E há duas forças externas, ambas **medidas** ([`02_o_que_torna_boolean_e_fillet_extraordinarios.md`](../../3DModeling/02_o_que_torna_boolean_e_fillet_extraordinarios.md)):

1. **Metade da queixa já está resolvida pela indústria.** O Blender adotou o `Manifold` como solver
   de booleana na **4.5**. Trazer booleana de malha para cá **empata**, não vence.
2. **A metade que não está resolvida é o arredondamento**, e a causa é estrutural: o `Bevel` opera
   sobre a topologia **depois** que a booleana já a bagunçou. É por isso que existe uma indústria de
   addons só para remendar bevel, e por isso o Plasticity (Parasolid) parece de outro planeta.

## Decisão

**O documento do módulo é uma ÁRVORE DE EXPRESSÃO autorada — primitivas, transformações e operações
com raio. Nada de triângulos, nada de voxels. `f(p)` responde "esta forma existe aqui?".**

Dela decorrem, e cada uma é consequência e não escolha independente:

### 1. Booleana e arredondamento não podem falhar, **por construção**

União é `min(a, b)`; não existe geometria degenerada para uma comparação de dois números. O
arredondamento é um operador sobre os mesmos dois números, e **funciona onde três ou mais formas se
encontram** — o caso que quebra o rolling-ball do CAD e o `Bevel` do Blender. Medido: o vértice
triplo fecha ([`01_resultados_spike.md`](../../3DModeling/01_resultados_spike.md) §1).

### 2. ⭐ O que o artista VÊ é o campo **traçado**; a malha é artefato de **exportação**

Medido (§1c do spike): traçando o campo ponto a ponto, a quina sai como uma navalha e o filete sai
liso, **zero serrilhado**; a mesma cena extraída em malha serrilha. A geometria estava certa e o
defeito era **inteiramente** da extração.

⚠️ **Isto inverte a ordem óbvia, e o motivo não é velocidade — é qualidade.** Deixar a malha
desenhar a tela seria deixar **o caminho pior definir o teto do que se vê**, que é literalmente o
que o [`CLAUDE.md §0`](../../../CLAUDE.md) proíbe. Custo do traçado: **46–57 ms** por quadro a 560²
num **único** núcleo (esta máquina tem 32, e um raio não fala com o vizinho).

### 3. O arredondamento é **EXATO por padrão**, e tem dois lados que são operadores diferentes

| | quina **côncava** (junção de peças) | quina **convexa** (aresta da peça) |
|---|---|---|
| Operador | `union_round(a, b, r)` | deslocamento: `f − r`, com a fonte encolhida de `r` |
| Centro do arco | **fora** do sólido | **dentro** do sólido |
| Erro medido | **0,00 %** em r = 0,05 / 0,12 / 0,25 | **0,00 %** em r = 0,04 / 0,08 / 0,20 / 0,12 |

*Não são a mesma operação com o sinal trocado* — a assimetria do centro do arco é geométrica. As
outras três (`intersection_round`, `difference_round`, `offset`) saem por **De Morgan**
(`A ∩ B = ¬(¬A ∪ ¬B)`), sem fórmula nova: duplicar a fórmula seria a segunda resposta que diverge.

O caráter **orgânico** (smooth-min) fica como knob **por operação**, nunca global.
⚠️ **Ele NÃO vai à UI com a etiqueta "raio"**: medido, entrega **exatamente 3/4** do número pedido,
em todos os raios testados — rotulá-lo "raio" mentiria 25 % ao utilizador, sempre.

### 4. O raio fica **editável para sempre**

Ele é parâmetro da operação, não geometria assada. ⭐ **Nem o Blender nem o MoI dão isto** — lá,
arredondar é destrutivo. É a mesma lei **fonte ≠ cozido** que a casa já aplica duas vezes
([ADR-0121](0121-vector-live-corners-authored-source-cooked-geometry.md) Live Corners,
[ADR-0132](0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md) Live Path
Effects), agora em 3D.

### 5. O motor de avaliação é a `fidget`, **isolada atrás de uma crate**

[`fidget`](https://github.com/mkeeter/fidget) 0.5.0 (Matt Keeter, autor do `libfive`), **MPL-2.0** —
já na allowlist do `deny.toml`. Medido: **zero** dependência C/C++, **zero** `wgpu`, **20 pacotes
novos**, **zero** exceção de licença necessária. `ph2d-field-eval` é a **única** crate do repo que a
nomeia.

**O JIT dela fica LIGADO**: medido **5,3×** no traçado (o caminho que o artista olha) e 1,6× na
malhagem. É a justificativa que o HR-2 exige para `unsafe`, e ela é forte.
⚠️ *A primeira medição dizia "ganho zero" e estava errada — comparava `VmShape` com `VmShape`.
Registro em [`01_resultados_spike.md`](../../3DModeling/01_resultados_spike.md) §6.*

### 6. ⛔ Escala **não-uniforme** é recusada

Ela destrói a propriedade de distância (‖∇f‖ = 1), que é a fundação de tudo acima: sem ela o raio
deixa de ser o raio, a casca perde a espessura e a marcha de raios atravessa a superfície. Quem
quiser um elipsoide usa uma primitiva de elipsoide, não uma esfera esticada.

## Alternativas consideradas — e o preço de cada uma

| Alternativa | Por que não |
|---|---|
| **B-Rep / NURBS** (o caminho do MoI e do Plasticity) | O padrão-ouro do arredondamento é o **Parasolid**, que **não se compra numa loja**: contrato Siemens, ~130 ISVs, sem preço público. O aberto em Rust é o `monstertruck` (Apache-2.0, 2 dias de idade quando medido) — e **não tem casca**, que é o passo 3 da definição de pronto do original. Detalhe em [`00_plano_port.md`](../../3DModeling/00_plano_port.md) §3 |
| **OCCT** (`opencascade-rs`) | Duas portas independentes: LGPL **não está** na allowlist do [`deny.toml`](../../../deny.toml), e o `cargo deny` roda no `ship.sh` e no CI; e construir OCCT na matriz de 3 SOs é custo desproporcional |
| **`brepkit`** | O mais completo — e **AGPL-3.0** sobre produto fechado. Só por licença comercial: decisão de produto, não de engenharia |
| **Booleana de malha exata** (`Manifold` / `manifold-rust`) | Robusta e rápida — e **é o que o Blender já tem desde a 4.5**. Empatar não é o alvo. E ela **não resolve arredondamento**, que é o buraco real |
| **Escrever um kernel B-Rep do zero** | O `fornjot` tentou exatamente isso em Rust e foi **arquivado em 2026-06-19** com *"its goals were never reached"*. O menor kernel existente mede **96.660 LOC**; todo o resto deste módulo estima-se em 15–25 k |
| **Assar o campo num grid de voxels como documento** | Perde a edição paramétrica, prende a resolução na autoria e infla o undo. O grid existe como **cache derivado**, não como verdade |

## O preço da escolhida (honesto)

1. ⛔ **Sem export STEP.** Um modelo implícito não tem superfícies analíticas para escrever num
   arquivo de CAD. Sai malha (STL/OBJ/PLY, que a `ph2d-mesh` **já exporta**). Aceito na decisão.
2. ⛔ **A malha de exportação é limitada pela resolução**, e o extrator atual **serrilha aresta
   viva** — medido, com o mecanismo nomeado (quantização à grade, §2 do spike). Não está no caminho
   do que se vê, mas **está** no caminho do que se exporta.
3. ⚠️ **A `fidget` é "experimental" pelo próprio autor** (*"recomendo forkar para trabalho sério"*),
   e a extração de malha é a parte que ele marca como menos testada. MPL-2.0 permite forkar; o
   documento (`ph2d-field`) não depende dela.
4. ⚠️ **`unsafe` entra** pelo JIT. Justificado por 5,3×, e desligável por feature.

## Consequências

- **Crates:** `ph2d-field` (o documento — dados autorados, sem avaliador) · `ph2d-field-eval` (a
  ponte: árvore → tape da `fidget` → traçado e malha). A fronteira existe para que trocar de motor
  seja trabalho de **uma** crate e **nenhum arquivo salvo quebre**.
- **Undo e persistência de graça:** o documento é dado autorado pequeno e serializável — exatamente
  o que o undo por snapshot da casa quer, e o oposto do `ShapeId` cru que o envenenaria
  ([ADR-0131](0131-physics-global-runtime-truth-rapier-ecs-bridge.md)).
- **Os perfis 2D vêm da caneta que já existe** (`ph2d-vec-*`): o fluxo do MoI renasce sobre o editor
  vetorial da casa. É combinação que não existe no mercado.
- **A escultura entra na booleana:** `ph2d-sdf` já converte malha → campo. Uma cabeça esculpida pode
  ser cortada por uma forma dura, com arredondamento exato.
- **`Tool = 12` não é encostado:** o ponteiro 3D chega pelo **shell**, como no `sculpt3d`
  ([ADR-0150](0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)).
- **Gates que nascem com o módulo:** os dois raios exatos a 0,00 % viram **teste**, não anedota de
  spike; e a estabilidade do formato serializado (HR-14).

## O que este ADR NÃO decide

- **Qual extrator de malha** conserta a aresta viva do export (§preço 2). A conferência decisiva é
  contra o `libfive`, e é trabalho de wave.
- **GPU.** A ordem que a medição impõe é: JIT → **threads** (32 nesta máquina) → GPU **só se** ainda
  pedir. ⛔ Não o inverso.
- **O desenho do painel e das ferramentas** — vem com a UI, sob o Widget Gallery (DIRETRIZ §5.2).
- **Se o orgânico ganha calibração ×4/3 ou um nome próprio** (§3) — é decisão de produto do Enio.
