# Plano — o modelador **implícito**, rota corrigida

**Linha:** `line/3DModeling` · **Data:** 2026-08-19 · **Estado:** plano, zero código.
**Decisão do Enio (2026-08-19):** *"vamos ao caminho 3. Planeje, corrija a rota anteriormente
estabelecida. Vamos em busca do padrão ouro dentro das possibilidades."*

> **Este doc substitui a escolha de stack e as waves do [`00_plano_port.md`](00_plano_port.md).**
> O que continua valendo de lá: o **estudo do original** (§1, as 9 leis, as 19 operações) e o
> **inventário do que a PH2D tem** (§2). O *porquê* desta rota está em
> [`02_o_que_torna_boolean_e_fillet_extraordinarios.md`](02_o_que_torna_boolean_e_fillet_extraordinarios.md).

---

## §0 — O que "padrão ouro dentro das possibilidades" quer dizer aqui

**Padrão ouro** = o que a nTop entrega e nenhum app de malha entrega: booleana e arredondamento que
**não podem falhar**, porque não são algoritmos de topologia — são aritmética.

**Dentro das possibilidades** = três fronteiras que aceito de olhos abertos, e que estão escritas
aqui para ninguém as redescobrir como surpresa:

1. ⛔ **Sem export STEP.** Um modelo implícito não tem superfícies analíticas para escrever num
   arquivo de CAD. Sai malha (STL/OBJ/PLY — que a PH2D **já exporta**). Isto foi aceito ao escolher
   o caminho 3.
2. ⛔ **Resolução é finita.** A superfície é extraída numa densidade escolhida. Para arte e render
   isso é indiferente ou melhor; para metrologia, não serve.
3. ⛔ **Quina viva não é de graça** — exige contorno com QEF (§4). É a peça de engenharia real deste
   plano, e é conhecida, não pesquisa aberta.

---

## §1 — A tese: **o modelo é uma FUNÇÃO**, não uma malha e nem um grid

O documento não guarda triângulos nem voxels. Guarda uma **árvore de expressão**: primitivas,
transformações e operações. Perguntar *"esta forma existe no ponto p?"* é avaliar `f(p)`.

```
diferença(raio: 3mm)
├── união(raio: 8mm)
│   ├── caixa(40, 20, 20)
│   └── revolve(perfil: <curva do editor vetorial>, eixo: Y)
└── cilindro(r: 5, h: 50)
```

**O que essa escolha compra, e cada item é uma consequência direta, não uma promessa:**

| Consequência | Por quê |
|---|---|
| **Booleana não pode falhar** | União é `min(a,b)`. Não existe geometria degenerada para uma comparação de dois números |
| **Arredondamento não pode falhar** | É um operador sobre os mesmos dois números — e funciona onde **três ou mais** formas se encontram, que é o caso que quebra o rolling-ball do CAD e o Bevel do Blender |
| **O raio do fillet fica EDITÁVEL PARA SEMPRE** | Ele é um **parâmetro da operação**, não uma geometria assada. ⭐ Nem o Blender nem o MoI dão isto: lá o arredondamento é destrutivo |
| **Resolução infinita na fonte** | A malha é **derivada**. Extrai-se grosso para mexer e fino para exportar, do *mesmo* documento |
| **Undo quase de graça** | O documento é dado autorado pequeno e serializável — exatamente o que o undo por snapshot da PH2D quer (o oposto do `ShapeId` cru que envenenaria o undo, `00_plano_port.md` §4.2) |
| **É a lei que a casa já tem** | **fonte ≠ cozido**: [ADR-0121](../architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md) (Live Corners) e [ADR-0132](../architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md) (Live Path Effects). Este módulo é a terceira aplicação da mesma lei, em 3D |

---

## §2 — O motor de avaliação: **`fidget`** (medido, não lido)

[`fidget`](https://github.com/mkeeter/fidget) **0.5.0** (2026-08-03), de **Matt Keeter** — o autor do
`libfive`, que é a implementação de referência desta família inteira. Ela faz o que é difícil:

- **Tape**: converte a árvore em código de linha reta, deduplicado.
- **Aritmética de intervalo + simplificação de tape**: avalia uma *região* de uma vez e **poda** o
  que está inteiramente dentro ou fora. É isto que torna viável malhar em alta resolução — e é a
  parte que ninguém quer reescrever.
- **JIT** com assembly escrita à mão (aarch64 + x86_64).
- **Manifold Dual Contouring**: extração de malha **preservando quina viva** (§4).
- Avaliação por ponto, por intervalo, **SIMD** e **gradiente** (a normal sai de graça, exata).

### §2.1 — O que eu medi (sonda fora do repo)

| Pergunta | Medição | Veredito |
|---|---|---|
| Licença | **MPL-2.0** | ✅ já na allowlist do [`deny.toml`](../../deny.toml) |
| Exige C/C++? | **nenhum** `cc`/`cmake`/`bindgen`/`cxx` | ✅ matriz de 3 SOs não paga nada |
| Puxa `wgpu` (conflito com o 28 da casa)? | **Nenhum** com `default-features = false, features = ["mesh"]` | ✅ o backend `wgpu` dela é feature separada |
| Pacotes novos no `Cargo.lock` | 93 no grafo, **73 já estão na PH2D** ⇒ **20 novos** | ✅ mais limpo que o kernel B-Rep (33) |
| `cargo deny` | **nenhum** pacote fora da allowlist | ✅ zero exceções necessárias |
| Features default | `[bytecode, gui, jit, mesh, raster, rhai, shapes, solver, wgpu]` | ⚠️ **desligar tudo**; `gui`/`rhai`/`raster` são do app-demo dela |

### §2.2 — ⚠️ O risco, dito pelo próprio autor

O README diz, com estas palavras: *"experimental"*, *"Lego-kit-without-a-manual energy"*,
**"recomendo forkar para trabalho sério"**, e — o que mais importa aqui —
**"a extração de malha dela é muito menos testada em batalha que a do libfive"**.

**Mitigação, e é a mesma do plano anterior:** `ph2d-field-eval` é a **única** crate do repo que
nomeia `fidget`. MPL-2.0 permite vendorizar e forkar. O que a W0 mede é exatamente a parte que o
autor sinaliza como menos testada: **a qualidade da malha nas quinas** (§4).

⚠️ **O `jit` fica DESLIGADO até ser medido.** Ele é assembly escrita à mão ⇒ `unsafe` ⇒ HR-2 exige
justificativa escrita. A W0 mede intérprete contra JIT e **o número decide**, não o conforto.

---

## §3 — O arredondamento: a peça central, e a armadilha que separa produto de brinquedo

Este módulo existe por causa desta seção.

### §3.1 — ⚠️ `smooth-min` **destrói** a propriedade de distância

O operador famoso (`opSmoothUnion`) mistura dois campos e produz um resultado que **já não é uma
distância**: ele viola a condição de Eikonal (`‖∇f‖ = 1`) e a 1-Lipschitz. Consequências reais, e é
por aqui que implementações ingênuas ficam com cara de brinquedo:

- **o raio deixa de ser o raio** — ⚠️ **MEDIDO E CORRIGIDO pela W0**
  ([`01_resultados_spike.md`](01_resultados_spike.md) §3): a previsão era que **encadear** degradaria.
  **Errado.** Uma aplicação degrada exatamente o mesmo que duas (desvio 0,4132 contra 0,4142), e a
  degradação é **local**, onde duas superfícies se tocam quase **tangentes**. O operador exato
  entrega o raio pedido com **0,00 %** de erro num filete transversal. *Mecanismo certo, cura errada
  — e a cura que esta linha prescrevia (rastrear Lipschitz pela cadeia) mirava o alvo errado.*
- **offset e casca erram** (`f(p) − t` só é espessura se `f` for distância de verdade);
- a marcha de raios (§5.3) **atravessa a superfície**, porque o passo seguro deixou de ser seguro.

### §3.2 — A resposta, e ela tem referência publicada

⚠️ **DIRETIVA §1: existe algoritmo de referência publicado ⇒ porte-o.** A fonte canônica desta
família é o **Inigo Quilez** — [smin](https://iquilezles.org/articles/smin/) e
[distance functions](https://iquilezles.org/articles/distfunctions/). ⛔ Nada de constante inventada
aqui.

**Dois caracteres de arredondamento, e a diferença é de PRODUTO, não de qualidade:**

| | **Redondo exato** (`rounded union`) | **Orgânico** (`smooth min`) |
|---|---|---|
| Geometria | Raio **constante** de verdade, quando as entradas são distância honesta | Transição contínua, raio "efetivo" variável |
| Look | O do CAD: reflexo em faixa limpa, o blend termina numa borda definida | O de escultura: derrete, sem borda |
| Preserva distância? | **Sim** (é `min` deslocado, com correção exata no canto) | **Não** (§3.1) |
| Serve para | Produto, hard-surface — **o que o Enio comparou ao Plasticity** | Orgânico, junções de 3+ formas, personagem |

**Os dois entram**, e são um **knob por operação**, não um modo global. O padrão é o **exato**,
porque é ele que dá o look que motivou este módulo.

### §3.3 — Manter o campo honesto

Ordem de preferência, e é uma lei do módulo:

1. **Preferir operadores que preservam distância** (o exato do §3.2).
2. **Saber quando o campo degradou** — rastrear a cota de Lipschitz pela árvore, para que o
   arredondamento seguinte saiba com o que está lidando.
3. **Re-distanciar** só onde for necessário (fast sweeping / fast marching sobre o campo amostrado).
   ⚠️ É caro; entra **medido**, não por precaução.

---

## §4 — A quina viva: **Manifold Dual Contouring**

O Surface Nets que a `ph2d-sdf` já usa **arredonda a quina** — ótimo para o remesh de escultura,
inaceitável aqui: um cubo tem de sair com aresta.

**Manifold Dual Contouring** resolve um **QEF** (mínimos quadrados sobre as normais dos cruzamentos)
por célula de octree e coloca o vértice **na quina**, e não no centro. É *manifold*, estanque,
hierárquico e preserva feição — e a normal exata vem do **gradiente** da função, de graça (§2).

- ✅ A `fidget` **implementa** MDC — é o que se usa.
- ⚠️ É a parte que o autor marca como menos testada (§2.2) ⇒ **item nº 1 da W0**.
- ⛔ **Não** adotar a ideia nova do Keeter ([*"Please Steal my Meshing Algorithm Idea"*, 2026-07-03](https://www.mattkeeter.com/blog/2026-07-03-meshing/)):
  ela promete manifold **sem auto-intersecção** + feição fina, mas exige **tetraedralização de
  Delaunay 3D incremental robusta**, que o próprio autor nomeia como a barreira, e **não está
  implementada**. Registrada aqui para ninguém a redescobrir como novidade.

---

## §5 — Arquitetura

### §5.1 — As crates

```
ph2d-field        O DOCUMENTO: a árvore (primitivas, ops com raio, transformações).
                  Autoria pura, serializável, sem avaliador dentro. É NOSSA para sempre.
ph2d-field-eval   A PONTE: árvore -> tape da fidget, avaliação, intervalo, MDC -> ph2d_mesh::Mesh.
                  ÚNICA crate do repo que nomeia `fidget`.
ph2d-field-ecs    Componentes de RECEITA (o padrão do ADR-0131), nunca campo vivo.
ph2d-tool-model3d Ferramentas sob o contrato Tool=12 (CONGELADO — ponteiro 3D pelo SHELL, ADR-0150).
ph2d-panel-model3d Painel de abas no padrão do Widget Gallery (DIRETRIZ §5.2).
```

⚠️ **Por que `ph2d-field` e `ph2d-field-eval` são separadas:** a árvore é o documento do usuário e
tem de sobreviver a qualquer troca de motor. Se a `fidget` provar-se frágil (§2.2), o que se
reescreve é **uma** crate — e nenhum arquivo salvo quebra.

### §5.2 — Reuso: o que a PH2D **doa** a este módulo

| Já existe | Papel aqui |
|---|---|
| `ph2d-mesh` (20.824 LOC) | Recebe a malha do MDC; octree, raio, e **export STL/OBJ/PLY já pronto** |
| `ph2d-mesh-render` (6.112) | Desenha o resultado: matcap, SSAO, luz — **zero renderer novo** |
| `ph2d-sdf` (3.011) | ⭐ **malha → campo**: é a ponte que deixa **um objeto ESCULPIDO entrar na booleana**. Ninguém no mercado tem isso |
| `ph2d-vec-*` + `kurbo` | ⭐ **Os perfis 2D vêm da caneta que já existe.** Um editor vetorial completo alimentando um modelador 3D é o diferencial de fluxo desta casa |
| `shells/desktop/src/sculpt3d_*.rs` | A órbita 3D já mora no shell — e é o que mantém `Tool=12` fora do caminho |
| `ph2d-light` | O mesmo rig de luz da tinta 2D |

⚠️ **A peça que falta para os perfis:** distância 2D com sinal de um path de Bézier. É trabalho real
(distância a uma cúbica), com referência publicada, e é pré-requisito de `extrude`/`revolve`.

### §5.3 — Onde roda, e o teto que **só a medição** pode escrever

⚠️ [`CLAUDE.md §0`](../../CLAUDE.md): *o teto é o do HARDWARE, nunca o do caminho lento* — e proíbe
escrever qualquer `MAX_*` antes de medir.

### ⭐ MEDIDO na W0 — e a resposta **inverteu** o que esta seção previa

A previsão era *"malhar e desenhar a malha; a marcha de raios é a candidata seguinte, e só entra com
número"*. O número veio ([`01_resultados_spike.md`](01_resultados_spike.md) §1c) e disse o
contrário:

| Caminho | O que ele entrega | Custo medido (1 núcleo, JIT) |
|---|---|---|
| **Traçar o campo** | ⭐ **quina perfeita, filete liso, zero serrilhado** | **57 ms** / quadro a 560² |
| Malhar e rasterizar | ❌ aresta viva serrilhada (§2 do spike) | 21 ms a 128³ |

**O que o artista vê passa a ser o campo traçado. A malha é o artefato de EXPORTAÇÃO.**
Não é troca por velocidade — é por **qualidade**: a malha estava a definir o teto do que se vê, e é
o caminho pior. *Deixar a malha definir a imagem era exatamente o erro que o §0 do `CLAUDE.md`
proíbe.*

⚠️ **E o JIT entra**: 5,3× no traçado (a medição anterior estava errada — comparava `VmShape` com
`VmShape`; registro em `01_resultados_spike.md` §6). A justificativa de `unsafe` que o HR-2 exige
passa a existir e está escrita.

**Ordem de trabalho que a medição impõe:** traçado com JIT → **threads** (32 nesta máquina, e um
raio não fala com o vizinho) → GPU **só se** a medição ainda pedir. ⛔ Não o inverso.

---

## §6 — Waves

### W0 — Spike medido + **a primeira imagem** ✅ **FEITA (2026-08-19)**

> **Resultados: [`01_resultados_spike.md`](01_resultados_spike.md).** Nenhum kill-criterion disparou.
> Em uma linha: **o arredondamento exato entrega o raio pedido com 0,00 % de erro** e o vértice
> triplo fecha sem falhar (era a promessa do caminho) · **a quina viva REPROVOU** — a aresta é
> quantizada à grade, com o mecanismo já nomeado · **o JIT não se paga** (ganho −2 % a −11 %, fica
> desligado) · **64³ malha em 9,9 ms** num núcleo, então nenhuma GPU entra sem número novo.
> O código vive em [`spikes/field-spike/`](../../spikes/field-spike/) e re-roda com um comando.

Prova o mecanismo e produz os números que viram teto. **Entregável duplo: uma tabela e uma imagem.**

**A peça de teste** é a que quebra o Bevel do Blender: dois volumes se cruzando **mais um terceiro
batendo no vértice comum**, arredondados.

Mede:
1. **Quina viva** — um cubo sai com aresta reta? (o item que o autor sinaliza, §2.2/§4)
2. **Os dois caracteres de fillet** (§3.2) lado a lado, na mesma peça.
3. **A degradação do campo** (§3.1) — encadear dois arredondamentos ainda dá o raio pedido?
4. **Resolução × tempo × memória** — a curva que define os tetos (HR-13: quem declara budget **mede**).
5. **Intérprete × JIT** — o número que decide se `unsafe` entra (HR-2).
6. **Determinismo** (HR-5) — `chacha20` está no grafo; se algo amostrar RNG, não entra no replay-hash.

⛔ **Kill-criteria (congelados agora):**
- Se o MDC **não** entregar quina viva limpa em 2 tentativas ⇒ a extração de malha é reescrita por
  nós ou trocada; **não** se afrouxa a barra da quina — ela é metade do produto.
- Se o campo degradar a ponto de o raio pedido não ser o raio entregue, e o re-distanciamento (§3.3)
  não couber no orçamento ⇒ **PARA e reporta**: o arredondamento exato é o motivo do módulo existir.

### W1 — O documento + ADR
`ph2d-field` (a árvore), serialização versionada (**HR-14**), ponte ECS, e o
**ADR-0161** com a tabela de evidência das três famílias.
⚠️ **0161, não 0160** — a `line/sculpt3d` já ocupou o 0160 (medido 2026-08-19: `main` está em 0159,
e aquela linha tem 0160 não-integrado). ⛔ **Este número é uma LEITURA, não uma reserva** — número
que soma entre linhas se **conta**, nunca se escolhe (CLAUDE.md §5.0), e outra linha pode ocupar o
0161 antes de você. **Reconte no dia de escrever**, com o comando que mede as linhas vivas, não só
o `main`:
```bash
for b in $(git branch --format='%(refname:short)' | grep '^line/'); do
  git ls-tree -r --name-only "$b" -- docs/architecture/decisions/ | grep -oE '/[0-9]{4}' | tr -d /
done | sort -n | tail -1
```

### W2 — Ver a coisa
`ph2d-field-eval` → MDC → `ph2d-mesh` → `ph2d-mesh-render`. Malha grossa ao mexer, fina ao parar.

### W3 — Os perfis vêm do editor vetorial
Distância 2D com sinal de path (§5.2), `extrude` e `revolve`. **É aqui que o fluxo do MoI renasce**,
com a caneta que a casa já tem.

### W4 — As operações e o painel
União/diferença/intersecção **com raio por operação**, casca, offset, draft, padrões. Painel de abas.

### W5 — A ponte com a escultura, e o export
Malha esculpida → campo (via `ph2d-sdf`) → entra na booleana. Export pelo `ph2d-mesh`.

### W6 — Polimento
Atalhos, orçamento de quadro (HR-4), erro na UI, e os tetos escritos **com a tabela ao lado**.

---

## §7 — Riscos

| Risco | Mitigação |
|---|---|
| **`fidget` é "experimental" pelo próprio autor**, e a malha é a parte menos testada | Item nº 1 da W0. `ph2d-field-eval` isola o nome; MPL-2.0 permite forkar. O **documento** (`ph2d-field`) não depende dela |
| **Quina viva** sair arredondada | Kill-criterion da W0. ⛔ Não afrouxar |
| **Campo deixa de ser distância** (§3.1) | Operadores exatos por padrão; Lipschitz rastreado; re-distanciamento medido, não profilático |
| `unsafe` do JIT (HR-2) | Desligado até a medição justificar |
| **Sem STEP** | Aceito na decisão do caminho 3 (§0). ⚠️ Se um dia for exigido, é **outro módulo**, não um remendo aqui |
| ⚠️ `line/sculpt3d` viva, toca `shells/desktop/src/` **e já usou o ADR-0160** | Arquivo irmão novo em vez de engordar compartilhado; ADR **0161**; tudo anotado no handoff (§1.5.9 item 3) |
| Contrato `Tool=12` congelado | Ponteiro 3D pelo shell (ADR-0150). PARA e reporta se não der |

---

## §8 — O que segue sendo decisão do Enio

1. **O caráter default do arredondamento** (§3.2): exato (look de produto) ou orgânico. Recomendo
   **exato como padrão**, com o orgânico a um clique — mas isso se julga **olhando**, e a W0 entrega
   a imagem para isso.
2. **Se o preview justifica um segundo motor em GPU** (§5.3) — só depois dos números da W0.

---

## ⛔ Recusas MEDIDAS

| Recusa | Motivo medido |
|---|---|
| Kernel B-Rep (`monstertruck`) como base do módulo | Decisão do Enio pelo caminho 3; e ele não tem casca e tem 2 dias ([`00_plano_port.md`](00_plano_port.md) §3.3) |
| `wgpu`/`gui`/`rhai`/`raster` da `fidget` | Features do app-demo dela; `wgpu` ainda colidiria com o lockstep 28 da casa (§2.1) |
| `jit` da `fidget` ligado por omissão | `unsafe` sem medição — HR-2 (§2.2) |
| Ideia de malhagem do Keeter (2026-07-03) | Exige Delaunay 3D incremental robusto, **não implementada** pelo próprio autor (§4) |
| Segundo avaliador em GPU antes da W0 | *Dois motores sobre um estado é pior que um motor lento*; e o autor da `fidget` alega 2048³ em 77 ms em CPU (§5.3) |
| Escrever `MAX_*` neste plano | `CLAUDE.md §0` proíbe teto antes de medição (§5.3) |
