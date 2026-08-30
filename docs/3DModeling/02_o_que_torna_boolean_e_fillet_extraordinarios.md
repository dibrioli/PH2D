# O que torna booleana e arredondamento **extraordinários** — pesquisa

**Pergunta do Enio (2026-08-19):** *"o que me atrai neste tipo de modelagem é justamente o modo
eficaz de lidar com operações booleanas e arredondamento de arestas, coisas que não são tão fáceis
com o Blender e outros apps de modelagem 3D que têm resultados inferiores. Procure alternativas
eficazes e modernas com resultados extraordinários."*

> ⚠️ Esta pesquisa **reformula o alvo** do módulo. O plano [`00_plano_port.md`](00_plano_port.md)
> assumia *"portar um modelador NURBS"*. O que o Enio quer é o **resultado** (booleana que nunca
> falha + arredondamento bonito), e NURBS é **um** dos caminhos até ele — não o único, e talvez
> nem o melhor para esta casa.

---

## §1 — O diagnóstico: **metade da queixa já foi resolvida, e não foi por NURBS**

Medido, não suposto:

### §1.1 — A booleana do Blender **já é robusta desde a 4.5**

O Blender adotou o **[Manifold](https://github.com/elalish/manifold)** como solver de booleana na
**4.5** ([PR #136902](https://projects.blender.org/blender/blender/pulls/136902)) — a biblioteca do
Emmett Lalish (ex-Google, hoje Wētā FX) cuja tese é *"um algoritmo de booleana de malha com
**manifold garantido**, que acredito ser o primeiro do gênero"*, e cujo README diz: *"se a booleana
aqui algum dia falhar com você, por favor abra uma issue"*. Adotado também por OpenSCAD, Godot,
Babylon.js e mais 20 projetos, com relatos de **~1000× mais rápido** que as alternativas.

⚠️ **Consequência dura:** trazer o Manifold (ou um port dele) para a PH2D **não entrega nada acima
do Blender** nessa metade. Seria empatar, não vencer.

### §1.2 — O arredondamento **não** foi resolvido, e a razão é ESTRUTURAL

O `Bevel` do Blender opera sobre **topologia de malha DEPOIS** que a booleana já produziu a
bagunça: n-gons, triângulos finos, arestas quase-coincidentes. Daí o sintoma que todo mundo conhece
— *"bevel auto-intersecta em geometria complexa"* —, e a indústria de addons que existe só para
remendar isso (MESHmachine, Soft Bevel, Boolean Bevel).

O CAD arredonda **a definição da superfície**, não a malha: uma bola de raio *r* rola no vale entre
duas superfícies e a superfície de blend é o rastro dela. O resultado é **exato em qualquer zoom** e
tesselável na densidade que se quiser, depois.

> **É por isso que o [Plasticity](https://www.plasticity.xyz/) parece extraordinário e o Blender
> não.** Ele é "CAD para artistas" sobre o kernel **Parasolid**, e a frase que vendem é literalmente
> *"os algoritmos de booleana do Parasolid têm tolerâncias mais exatas e lidam melhor com geometria
> tangente e coincidente do que qualquer outro kernel"*. É o mesmo kernel do SolidWorks e do Shapr3D.

**Portanto: o diferencial que você quer está no ARREDONDAMENTO e na LIMPEZA do resultado, não na
booleana.** Qualquer plano que gaste o esforço na booleana está mirando na metade já resolvida.

---

## §2 — As três famílias que entregam, e o que cada uma custa

| | **A. B-Rep exato** | **B. Booleana de malha exata** | **C. Implícito / SDF** |
|---|---|---|---|
| **Booleana** | Exata; falha em geometria difícil | **Nunca falha** (manifold garantido) | **Não pode falhar** — é `min`/`max` de dois números |
| **Arredondamento** | ⭐ Exato, rolling-ball, lindo em qualquer zoom | ❌ **Não resolve** — é só booleana | ⭐ **Nunca falha** — é `smooth-min`, um operador só, em qualquer geometria |
| **Qualidade de superfície** | ⭐ Perfeita (analítica) | Herda a malha de entrada | Limitada pela **resolução** do campo |
| **Aresta viva** | Nativa | Nativa | ⚠️ **O ponto fraco** — precisa de contorno que preserve quina |
| **Export STEP / fabricação** | ⭐ Sim | Não | Não |
| **Custo p/ a PH2D** | Kernel de 2 dias (§3 do plano) ou contrato comercial | Baixo — mas empata com o Blender | **Baixo: 60% já existe** (§3) |
| **Quem prova em produção** | Plasticity, Shapr3D, SolidWorks (Parasolid) | Blender 4.5+, OpenSCAD, Godot | **nTop**, Altair, Womp |

### §2.1 — O que existe, com nome e link

**Família A (B-Rep):**
- **Parasolid** ([Siemens](https://plm.sw.siemens.com/en-US/plm-components/parasolid/)) — o padrão-ouro
  do arredondamento. ⚠️ **Não é `cargo add`**: licenciado a ~130-200 ISVs via Siemens/Tech Soft 3D,
  sem preço público, contrato comercial. É o que o Plasticity paga.
- **`monstertruck`** ([GitHub](https://github.com/virtualritz/monstertruck)) — Apache-2.0, Rust puro,
  rolling-ball fillet. **Não tem casca** e tem 2 dias de idade (§3.3 do plano).
- **OCCT** — LGPL + C++ em 3 SOs. Barrado pela política do repo.

**Família B (booleana de malha exata):**
- **`manifold-rust`** ([GitHub](https://github.com/larsbrubaker/manifold-rust)) — **port PURO em
  Rust** do Manifold 3.5.0, Apache-2.0, **686 testes passando**, paridade validada traço-a-traço
  contra o C++, patrocinado pela MatterHackers. **Sem C++.**
- **`boolmesh`** ([crates.io](https://crates.io/crates/boolmesh)) — Rust puro, MPL-2.0, só `glam` +
  `rayon`. Exige entrada manifold.
- **Estado da arte acadêmico:** [Interactive and Robust Mesh Booleans](https://arxiv.org/abs/2205.14151)
  (Cherchi/Pellacini/Attene/Livesu, SIGGRAPH Asia 2022) — predicados exatos a taxa interativa até
  200k triângulos; e [EMBER](https://dl.acm.org/doi/abs/10.1145/3528223.3530181) (SIGGRAPH 2022).

**Família C (implícito/SDF) — a resposta industrial moderna:**
- **[nTop](https://www.ntop.com/resources/blog/understanding-the-basics-of-b-reps-and-implicits/)** é
  a prova de produção, e a tese deles é exatamente a sua queixa: *"operações de B-rep como
  arredondamentos, offsets e booleanas frequentemente falham; modelos implícitos são baseados em
  matemática que **nunca falha**"*.
- O ponto fraco (aresta viva) tem trabalho **de 2026**:
  [Dual Contouring of Signed Distance Data](https://dl.acm.org/doi/10.1145/3799902.3811116)
  (SIGGRAPH 2026) — recupera quina a partir de amostras discretas de SDF, resolvendo um QEF por
  célula; há implementação em GPU com solver SVD.

---

## §3 — ⚠️ O que muda a conta: a PH2D já tem a família C pela metade

| Peça que a família C exige | Estado na PH2D |
|---|---|
| Malha → campo de distância | ✅ [`ph2d-sdf/src/field.rs`](../../crates/ph2d-sdf/src/field.rs) |
| Campo → malha | ✅ `surface_nets.rs` + `remesh.rs` (portados do SculptGL) |
| Espessura / offset de campo | ✅ `thickness.rs` |
| Malha residente + octree + raio | ✅ `ph2d-mesh` (20.824 LOC) |
| Render sombreado, matcap, SSAO | ✅ `ph2d-mesh-render` (6.112 LOC) |
| GPU de propósito geral | ✅ wgpu 29 em toda a casa |
| **Booleana e `smooth-min` no campo** | ❌ **não existe — e é o trabalho** |
| **Contorno que preserva quina** | ❌ o Surface Nets atual **arredonda** a quina |

**Ou seja:** o que falta da família C são **duas** peças, e ambas são código nosso, sem kernel de
terceiro, sem licença, sem C++. Compare com a família A, onde a peça central é um kernel de 2 dias
de idade que **não tem casca**.

⚠️ **E há um argumento de casa:** o [CLAUDE.md §0](../../CLAUDE.md) diz que *o teto é o do
HARDWARE, nunca o do caminho lento*. Avaliar um campo implícito é o trabalho mais paralelizável que
existe — um valor por voxel, independente dos vizinhos, sem ramo. É **exatamente** a forma que a GPU
desta máquina come. O B-Rep, ao contrário, é sequencial e cheio de ramo por natureza.

---

## §4 — Recomendação

**Família C (implícito/SDF, residente na GPU) como aposta principal.** As razões, em ordem:

1. **Ela responde às DUAS metades da sua queixa com UM mecanismo.** Booleana `min`/`max` e
   arredondamento `smooth-min` são o mesmo tipo de operação sobre o mesmo dado. Não há caso
   patológico: não existe "geometria difícil" para um campo escalar.
2. **É onde a PH2D já está** (§3) — e é onde a máquina é mais forte.
3. **É diferenciação real:** os apps de malha não fazem isso, e os que fazem (nTop) custam preço de
   empresa. Copiar a família B seria empatar com o Blender (§1.1).

⚠️ **E o preço, dito antes de você perguntar:** o resultado da família C é **limitado pela
resolução** do campo, e a quina viva exige o contorno de 2026 (§2.1). Onde o B-Rep entrega uma
superfície analítica perfeita, o implícito entrega uma malha muito boa numa densidade escolhida.
Para **arte e render** isso é indiferente ou superior; para **fabricação e STEP**, não é.

⛔ **Uma família só, não duas.** A memória desta casa registra que *dois motores sobre um estado é
pior que um motor lento* — assumir o laço inteiro ou nada. Se a escolha for C, o B-Rep não entra
"para os casos exatos" depois; ele seria um segundo dono da mesma pergunta.

---

## §5 — O que decide, e é do Enio: **uma imagem, não um argumento**

A diferença entre as duas famílias é de **caráter do arredondamento**, e isso se julga com o olho,
não com tabela:

- **rolling-ball (B-Rep):** raio constante, quina viva onde o blend acaba — o look "produto
  industrial", reflexo em faixa limpa.
- **smooth-min (implícito):** transição contínua, um pouco mais "orgânica", que continua funcionando
  onde três ou mais formas se encontram — caso em que o rolling-ball costuma falhar.

**Proposta:** antes de escolher, a W0 vira um **teste comparativo visual** — a mesma peça (dois
volumes se cruzando + um terceiro no vértice, o caso que quebra o Bevel do Blender), arredondada
pelos dois caminhos, lado a lado na tela. Você olha e decide. É barato: o lado implícito é curto
sobre o `ph2d-sdf` que já existe, e o lado B-Rep é o spike que o plano já previa.

---

## ⛔ Recusas MEDIDAS

| Recusa | Motivo medido |
|---|---|
| Adotar Manifold/`manifold-rust` como *o* diferencial | O **Blender 4.5 já o adotou** (§1.1) — empata, não vence |
| Gastar o esforço do módulo na booleana | Metade já resolvida pela indústria; o buraco é o arredondamento (§1.2) |
| Parasolid | Contrato comercial Siemens, sem preço público, ~130 ISVs (§2.1) |
| OCCT / `opencascade-rs` | LGPL fora da allowlist + build C++ em 3 SOs ([`00_plano_port.md`](00_plano_port.md) §3.1) |
| `brepkit` | AGPL-3.0 sobre produto fechado (idem) |
| Escrever kernel B-Rep do zero | fornjot arquivado com *"goals were never reached"* (idem §7) |
| Rodar as famílias A **e** C juntas | *Dois motores sobre um estado é pior que um motor lento* (§4) |
