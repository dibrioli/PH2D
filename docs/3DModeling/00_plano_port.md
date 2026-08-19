# Plano de port — modelador NURBS/B-Rep (clone de UX do MoI) para a PH2D

**Linha:** `line/3DModeling` · **Data:** 2026-08-19 · **Estado:** plano, zero código escrito.
**Original estudado:** `/home/enio/Documentos/Recursos/MOI_Clone_2026-08-19` (TypeScript/Vite,
8.078 LOC em `src/` + `tests/`, 118 testes, marcos 0-6 fechados e 7 parcial).

> Este doc é o **roteador** do módulo: o que o original é, o que a PH2D já tem, qual stack Rust
> ganhou e **por qual medição**, e as waves. O mecanismo de cada wave vai para o handoff dela.

---

## §1 — O que o original é (estudo)

Modelador **NURBS/B-Rep** no navegador, clone do *fluxo* do [MoI 3D](https://moi3d.com) — não da
lista de comandos. O `PLANO_MVP.md` dele nomeia quatro pilares, em ordem de importância:

| Pilar | O que é | Estado no original |
|---|---|---|
| **Painel lateral** | Sem menus aninhados. Abas Desenhar/Construir/Transformar; o painel **vira o diálogo** do comando ativo | ✅ |
| **Snap "grudento"** | 11 tipos (End/Mid/Center/Quad/Int/Perp/Tan/on-curve/grid/ortho/cplane) + trava de ângulo + coordenada digitada no meio do gesto + **histerese** | ✅ |
| **Kernel NURBS real** | Booleana, fillet e offset em B-Rep de verdade, nunca em malha | ✅ (OCCT/WASM via replicad 1.0.0) |
| **Malha de saída limpa** | Tesselação com controle de desvio/ângulo | ✅ triângulos com 3 tolerâncias |

### §1.1 — A superfície completa do kernel (fonte: `src/kernel/protocol.ts`)

São **19 operações**, e esta lista é o **conjunto de aceitação** do port — não uma aproximação:

`ping` · `importStep` · `makeBox` · `tessellate(tol, angTol)` · `dispose` · `extrude` · `revolve` ·
`loft(ruled)` · `planar` · `sweep(profile, spine)` · `transform(spec)` · `serializeShape` ·
`deserializeShape` · `exportStep` · `exportStl` · `boolean(union|subtract|intersect)` ·
`fillet(onEdge, r)` · `chamfer(onEdge, d)` · `shell(onFace, t)`

Mais os perfis que viajam **paramétricos** ao kernel (`ProfileCurve`): `segments` · `circle` ·
`arc` · `ellipse` · `spline`. ⚠️ **Isto é load-bearing e está medido no original:** extrudar um
círculo paramétrico dá **exatamente 3 faces**; amostrado em polilinha daria dezenas.

### §1.2 — As 9 leis que o original pagou para descobrir (`HANDOFF.md` §2)

Estas atravessam a linguagem e valem no port **inteiras**. Quebrar qualquer uma dá bug silencioso.

1. **A thread de UI nunca vê um `TopoDS_Shape`** — só id e buffers.
2. **Toda mutação passa por um `Command`** — inclusive o arrasto de ponto de controle, que
   **reverte o movimento ao vivo antes** de executar o comando (senão a pilha vê estado já alterado).
3. **`TransformSpec` é única para curva e sólido** — se as duas contas divergirem, mover uma curva
   e um sólido juntos os separa.
4. **Duas camadas de render** — profundidade atrás (superfície, aresta, grade), traço nítido à
   frente (curva, preview, snap, cota). **As duas projeções têm de bater** (teste de meio pixel).
5. **Aresta e face se localizam por PONTO, não por índice** — os ids são ponteiros e a ordem não é
   garantida. O app manda um ponto *sobre* a aresta e o kernel acha a geometria.
6. **Unidades diferentes na tesselação, medidas e não supostas** — `faceGroups` conta índices de
   triângulo, `edgeGroups` conta pontos.
7. **Preview assíncrono: uma requisição em voo por vez** — as que chegam durante a espera são
   descartadas menos a última, e cada atualização **descarta a forma anterior** ou vaza heap.
8. **Booleana e transformação preservam as entradas** — é o que deixa o undo restaurar sem refazer.
9. **Estilo entra no undo; visibilidade não** — estilo descreve o objeto, visibilidade é estado de vista.

### §1.3 — O que o original mediu, e que não se re-descobre

Do `RESULTADOS_SPIKE.md` (OCCT/WASM, Node 22). Os números são **do outro stack** e servem de
*baseline de expectativa*, nunca de meta do nosso:

| Fato medido | Número | Consequência de desenho |
|---|---|---|
| **Tesselar custa mais que modelar** | 2.621 ms (tol 0,01) contra 3-20 ms das operações | É o que obriga o trabalho fora da thread de UI |
| Preview de extrude | 14 ms/quadro (~70 fps) | Preview ao vivo é viável |
| Preview de booleana | 63 ms/quadro (~16 fps) | Debounce **só** em booleana |
| Fillet em **todas** as arestas de uma taça | **falhou** | Fillet é sempre **por aresta selecionada**; sem botão "arredondar tudo" |
| Fillet numa aresta | 21 ms | — |
| Matriz de 12 furos booleanos | 915 ms | Precisa de progresso incremental |

### §1.4 — Achados abertos do original (`HANDOFF.md` §4) — herdados como trabalho ou como não-problema

`4.1` código morto (`TranslateObjects`) — **não porta** · `4.2` grade duplicada em 2D e 3D —
**o port não repete: uma fonte, dois consumidores** · `4.3` **sólidos não participam do snap** (é a
lacuna que mais se sente ao modelar) — **entra no escopo do port** · `4.4` export leva só o primeiro
sólido — resolvido por compound · `4.5` sem IGES · `4.6` formas órfãs no worker sem teto ·
`4.7` sem tratamento de "documento não salvo".

---

## §2 — O que a PH2D **já tem** (inventário medido, 2026-08-19)

⚠️ *Antes de construir um item de lista aberta, meça se a composição já o exprime* (CLAUDE.md §5.0).
Medido nesta worktree — **61.633 LOC de infraestrutura 3D já existem**:

| Peça | LOC | O que dá ao port |
|---|---|---|
| [`ph2d-mesh`](../../crates/ph2d-mesh/) | 20.824 | Malha residente, octree, `Ray`/`Hit`, AABB, adjacência, **`write_stl`/`write_obj`/`write_ply`** e `import_stl`/`import_obj`/`import_ply` |
| [`ph2d-sculpt3d`](../../crates/ph2d-sculpt3d/) | 30.617 | Vizinho, não dependência — mas prova o padrão drop-crate 3D (ADR-0150) |
| [`ph2d-mesh-render`](../../crates/ph2d-mesh-render/) | 6.112 | Passe wgpu 28, matcaps, SSAO, SSS, `Camera3d`, **`wire_indices`** (arestas) |
| [`ph2d-sdf`](../../crates/ph2d-sdf/) | 3.011 | Remesh por Surface Nets, AO assado |
| [`ph2d-light`](../../crates/ph2d-light/) | 1.069 | O rig de luz único do app |
| `shells/desktop/src/sculpt3d_*.rs` | 13 arquivos | **A navegação orbital já mora no shell** — e é o que mantém `Tool=12` fora do caminho |
| `ph2d-vec-*` (8 crates) + `kurbo` | — | Curva 2D, booleana viva, envelope, layout — a matemática de curva do editor |
| `ph2d-grid`, `ph2d-guides`, `ph2d-panel-grid-snap` | 7.012 | Grade e guias 2D (o snap **3D** é novo) |

**Consequências diretas:**

- A **camada de profundidade** da lei §1.2.4 já existe (`ph2d-mesh-render`), e a **camada de traço
  à frente** também (Vello, o mesmo que desenha todo o editor). O port **não constrói renderer** —
  ele constrói a **costura** entre os dois, e o gate de paridade de projeção da lei §1.2.4.
- **Export STL/OBJ/PLY já existe** em `ph2d-mesh`. Do Marco 7 do original, só **STEP** é novo.
- A **navegação orbital já existe** no shell e é o precedente que impede o contrato congelado
  `Tool=12` de ser encostado.

---

## §3 — O stack: qual kernel, e por qual medição

O original decidiu por OCCT/WASM porque *"é o único kernel B-Rep sério disponível no browser"* — uma
restrição **do browser**, que não existe aqui. Em Rust nativo a pergunta se reabre inteira.

### §3.1 — Os candidatos, e o que os elimina

| Candidato | Licença | Nativo? | Veredito |
|---|---|---|---|
| **`monstertruck` 0.4.0** | **Apache-2.0** ✅ | **Rust puro** ✅ | **ESCOLHIDO** — fork fortificado do `truck`, com `-solid` (booleanas), `-fillet` (fillet+chanfro), `-healing`, `-io` (STEP) |
| `truck` (ricosjp, upstream) | Apache-2.0 ✅ | Rust puro ✅ | **Perde para o fork**: booleana que entra em `panic` em vez de `Result`, e **sem fillet/chanfro** — as duas operações mais difíceis do conjunto §1.1 |
| `brepkit` | **AGPL-3.0** ou comercial ❌ | Rust puro | **Eliminado pela licença.** É o mais completo (fillet de raio variável, shell, offset, draft, STEP/IGES/3MF/glTF, `unsafe` proibido por lint) — e AGPL é copyleft de rede sobre um produto fechado. Só entra por compra de licença comercial: **decisão do Enio, não minha** |
| `opencascade-rs` / `occt-rs` | **LGPL-2.1** ❌ | C++ (cxx/cmake) | **Eliminado por duas portas independentes:** (a) LGPL **não está** na allowlist do [`deny.toml`](../../deny.toml), e `cargo deny` roda no `ship.sh` e no CI; (b) construir o OCCT na matriz de 3 SOs (linux+macOS+windows) é custo de build desproporcional |
| `fornjot` | — | — | **Morto.** O próprio repositório diz *"No longer in development"*, e o autor registra que os objetivos não foram alcançados |

### §3.2 — O que eu **medi** do `monstertruck` (não é leitura de README)

Sonda fora do repo (`cargo generate-lockfile` + `cargo metadata` sobre
`monstertruck 0.4 --no-default-features --features ["solid","step"]`):

| Pergunta que decide | Medição | Veredito |
|---|---|---|
| Exige toolchain C/C++? | `cargo tree -e normal,build` ⇒ **`cc` não é alcançável** | ✅ Rust puro — a matriz de 3 SOs não paga nada |
| Puxa `wgpu` (conflito com o 28 da casa)? | **Nenhum `wgpu`** no grafo — `gpu`/`render` são features **separadas** que não ligamos | ✅ Sem conflito de lockstep |
| Quantos pacotes novos no `Cargo.lock`? | 129 no grafo, **96 já estão na PH2D** ⇒ **33 novos** | ✅ Modesto |
| Passa no `cargo deny`? | 1 pacote fora da allowlist: **`xxhash-rust` (BSL-1.0)** | 🔶 Exige **uma** exceção — e o `deny.toml` **já tem duas exceções BSL-1.0** (`error-code`, `clipboard-win`): é o padrão que já existe no arquivo, não política nova |
| A feature `fillet` existe? | ❌ Não no umbrella — `solid = [meshing, -solid, -fillet, -healing, modeling/fillet]`. **A feature certa é `solid`** | ✅ (armadilha registrada) |

**API confirmada em docs.rs:** `monstertruck-solid` → `and`/`or`/`difference`/`symmetric_difference`
/`plane_cut`, todas **`Result`-shaped com `ShapeOpsError` tipado, nunca `None` silencioso`** ·
`monstertruck-fillet` → `fillet_edges`, `fillet`, `fillet_with_side`, `fillet_along_wire`,
`FilletOptions`/`FilletProfile`/`RadiusSpec`/`FilletError`, **por aresta individual** (que é
exatamente o modelo que a medição do original forçou, §1.3).

### §3.3 — ⚠️ O que eu **não** consegui confirmar, e por isso a W0 existe

`monstertruck-modeling` documenta `builder::extrude` e os traits `Sweep`/`ClosedSweep`, mas a
docs.rs **não confirma** publicamente **revolve, loft, sweep-por-trilho, shell e offset** — que o
README afirma. São **5 das 19 operações** do §1.1. Some-se a isto que a **0.4.0 foi publicada em
2026-08-17, dois dias atrás**, por um mantenedor único.

*Um plano cuja peça central está na palavra de um README é um plano com um buraco.* É por isso que a
W0 é um **spike medido com kill-criterion**, e não a primeira wave de implementação — exatamente o
que o próprio original fez com o `spike-occt/` antes de escrever uma linha do app.

---

## §4 — Arquitetura proposta

### §4.1 — As crates (drop-crate, ADR-0075)

```
ph2d-brep          A PONTE. Único consumidor de `monstertruck` no repo inteiro.
                   BrepStore (id -> Solid), as 19 ops do §1.1, tessellate -> ph2d-mesh::Mesh.
                   PURA: sem wgpu, sem UI, sem thread. Testável em nextest contra o kernel real.
ph2d-brep-job      A disciplina do assíncrono (lei §1.2.7): uma requisição em voo,
                   descarte das intermediárias, descarte da forma anterior. Folha, testável sem UI.
ph2d-brep-ecs      A ponte ECS: componentes de RECEITA (autoria), nunca o solver vivo.
ph2d-snap3d        As 11 leis de snap + histerese + trava de ângulo. Matemática pura.
ph2d-tool-model3d  As ferramentas sob o contrato Tool=12 (CONGELADO — §4.3).
ph2d-panel-model3d O painel lateral de abas, no padrão do Widget Gallery (DIRETRIZ §5.2).
```

**A costura que dá o direito de errar na escolha do kernel:** `ph2d-brep` é o **único** lugar do
repo que nomeia `monstertruck`. Se a W0 reprovar o kernel, ou se ele morrer daqui a um ano, o que
se reescreve é **uma crate** — não o módulo. Isto não é elegância, é o preço de entrar num kernel
de 2 dias de idade.

### §4.2 — O modelo de dados: a analogia é a **física**, não o documento vetorial

O original guarda `Map<ObjId, SceneObject>` com `shapeId` apontando para o worker. Traduzir isso
literalmente **quebra o undo da PH2D**, e o precedente está escrito: a `ph2d-physics-ecs` guarda
*"components de **CONFIG**, nunca estado vivo de solver — o undo ordena por bytes"*
([ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)), e a
lei irmã do shell diz que *"referência durável entre objetos é o **NOME** (`stable_name_id`), nunca
os bits"* — porque **o undo respawna tudo com bits novos**.

Um `ShapeId` cru num componente é exatamente o veneno que essa lei descreve.

**Duas rotas, e a bifurcação é a única decisão de desenho real da W1:**

| Rota | O sólido durável é | Custo | Ganho |
|---|---|---|---|
| **(a) Receita autorada** (recomendada) | A **receita** (perfil + operação + parâmetros); o sólido é **cache derivado**, chaveado por hash de conteúdo | Recook ao desfazer | É a lei **fonte ≠ cozido** que a PH2D já tem duas vezes ([ADR-0121](../architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md) Live Corners, [ADR-0132](../architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md) Live Path Effects) — e dá edição paramétrica **que o próprio MoI não tem** |
| **(b) B-Rep serializado** | Os bytes do sólido no snapshot (o `monstertruck` tem feature `rkyv`; o original usava BREP em texto) | Snapshot pesado | Fidelidade literal ao original; importado de STEP não tem receita |

⚠️ **As duas são necessárias, não alternativas:** um sólido **importado de STEP** não tem receita —
ele *é* bytes. A recomendação é **(a) como norma e (b) como caso do importado**, com a fronteira
explícita. Isto é decisão de W1 com ADR, não de agora.

### §4.3 — ⚠️ O contrato congelado que este módulo encosta

`Tool = 12 métodos` / `RasterEditTool = 5` / `CanvasPaintTool = 1` / `PanelEvent = 4`
([CLAUDE.md §6](../../CLAUDE.md), gate `architecture_tool_contract_surface`).

Ferramenta de CAD precisa de **evento de ponteiro em 3D**. A tentação é somar um método ao `Tool` —
e isso é **mudança de contrato congelado: ADR + parar e reportar ao Enio** (DIRETRIZ §4).

**Não é preciso**, e o precedente é do módulo irmão: *"a navegação orbital mora no SHELL, nunca numa
`Tool` — é isso que mantém `Tool=12` fora do caminho"* ([ADR-0150](../architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)).
O `sculpt3d` entrega ponteiro 3D pelos 13 arquivos `shells/desktop/src/sculpt3d_*.rs`, sem tocar o
contrato. **O port segue esse caminho.** Se alguma wave concluir que não dá: PARA e reporta.

### §4.4 — Render: a lei §1.2.4 traduzida

| Camada do original | Na PH2D | Gate que a lei exige |
|---|---|---|
| WebGL atrás (superfície, aresta de sólido, grade) | `ph2d-mesh-render` (wgpu 28, matcap, SSAO) + `wire_indices` | — |
| Canvas 2D à frente (curva, preview, snap, cota) | Vello — o mesmo renderer de todo o editor | — |
| *"as duas projeções precisam bater"* (`cameraSync.test.ts`, meio pixel) | `Camera3d` ⟷ transform do Vello | ⚠️ **Gate novo obrigatório.** Sem ele, todo erro de câmera vira "o snap não gruda onde eu vejo" |

---

## §5 — Waves

Cada wave fecha com gate batched + handoff. **Nenhuma wave começa sem a anterior verde.**

### W0 — Spike do kernel (**bloqueia tudo**, com kill-criterion escrito ANTES)

Espelha o `spike-occt/` do original. Mede o `monstertruck` contra o **conjunto de aceitação do
§1.1**, na fixture da definição de pronto do original (a **taça**: perfil → revolve → shell 2 mm →
fillet na borda → matriz circular de 6 recortes booleanos → export).

- **Mede:** cada uma das 19 ops (existe? tempo? erro legível?) · tesselação nas 3 tolerâncias ·
  paridade de perfil paramétrico (o círculo extrudado dá **3 faces**? — §1.1) · **determinismo**
  (HR-5: o grafo traz `chacha20`/`rand`; se alguma op amostra RNG, o replay-hash do CI morre) ·
  memória residente (HR-13 emendado pelo ADR-0117: quem declara budget possui um gate que **mede**).
- **Entrega:** `docs/3DModeling/01_resultados_spike.md`, no formato do `RESULTADOS_SPIKE.md` do
  original — tabela de medição, e **as decisões que os números forçarem**.
- ⛔ **Kill-criterion (congelado agora, antes do build):** se **booleana + fillet + shell** não
  fecharem a taça após **2 tentativas**, o kernel está **reprovado** — a linha PARA e reporta ao
  Enio com as duas saídas que sobram (comprar licença comercial do `brepkit`, ou abrir exceção de
  LGPL no `deny.toml` + assumir o build nativo do OCCT em 3 SOs). **Não há terceira tentativa**
  (regra two-strikes, DIRETIVA §5).

### W1 — Fundação: `ph2d-brep` + a ponte ECS + o ADR

ADR (próximo livre: **0160**) fixando: kernel escolhido com a tabela do §3.2 ao lado · a rota (a)/(b)
do §4.2 · a fronteira "só `ph2d-brep` nomeia o kernel" com gate · a exceção BSL-1.0 no `deny.toml`.
Mais `BrepStore`, as ops puras, e a tesselação `Solid → ph2d_mesh::Mesh`.

### W2 — Viewport e as duas camadas
Costura `ph2d-mesh-render` + Vello, **o gate de paridade de projeção** (§4.4), grade adaptativa de
**uma** fonte (o achado 4.2 do original, não repetido), layout de 4 vistas, picking de face/aresta
**por ponto** (lei §1.2.5).

### W3 — Curvas e snap
Plano de construção, as 8 ferramentas de desenho, `ph2d-snap3d` com as 11 leis + histerese + trava
de ângulo + entrada digitada. **Inclui o achado 4.3**: snap em vértice/centro-de-face de sólido,
que o original deixou aberto.

### W4 — Construir
As 11 operações de sólido, `ph2d-brep-job` com a disciplina do §1.2.7, preview ao vivo do extrude,
debounce **só** em booleana (§1.3), fillet **sempre por aresta selecionada** (§1.3), e a tabela de
tradução de erro do kernel (a lição ② do spike do original).

### W5 — Transformar e organizar
`TransformSpec` **única** para curva e sólido (lei §1.2.3, com gate de divergência), matrizes com
progresso incremental, browser de cena, estilos.

### W6 — I/O e polimento
STEP in/out (o resto do export **já existe** em `ph2d-mesh`), persistência versionada (**HR-14**,
e o `PROJECT_SCHEMA` é número que **soma entre linhas: conta-se, nunca se escolhe** — CLAUDE.md
§5.0), atalhos, tratamento de erro na UI, e o teto de formas do achado 4.6.

---

## §6 — Riscos

| Risco | Mitigação |
|---|---|
| **O kernel tem 2 dias de idade** (0.4.0, 2026-08-17, mantenedor único) | W0 mede antes de tudo; `ph2d-brep` isola o nome; Apache-2.0 permite vendorizar se o upstream morrer |
| 5 das 19 ops não confirmadas (§3.3) | É literalmente o que a W0 mede |
| **Determinismo** (HR-5): `chacha20`/`rand` no grafo | Item explícito da W0; se uma op amostrar RNG, ela não entra no caminho do replay-hash |
| Contrato `Tool=12` congelado | §4.3 — caminho do `sculpt3d`, e PARA se não der |
| ⚠️ **`line/sculpt3d` está VIVA e toca `shells/desktop/src/`** | A costura de shell deste módulo cai na mesma pasta. Anotar cada arquivo novo no handoff (§1.5.9 item 3) e preferir **arquivo irmão novo** a engordar compartilhado (DIRETRIZ §1.5.2.1) |
| Escopo (*"é um CAD, sempre infla"* — o próprio original) | O conjunto de aceitação é o §1.1, congelado. O que estiver fora vai para backlog, não para a wave |

---

## §7 — O que é decisão do Enio, não minha

1. **Se a W0 reprovar o kernel** (§5 kill-criterion): comprar licença comercial do `brepkit` ou
   abrir exceção LGPL + build nativo do OCCT. Ambas são política de produto.
2. **Fidelidade vs. ganho**: a rota (a) do §4.2 dá **edição paramétrica que o MoI não tem**. É mais
   do que "clone" — e "mais" também é um desvio do alvo. A recomendação é (a); a palavra é dele.
