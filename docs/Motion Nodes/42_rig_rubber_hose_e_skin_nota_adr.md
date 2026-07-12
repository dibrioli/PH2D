# 42 — Rig: `rig.rubber_hose` + `rig.skin_deformer` — **M4 Rig FECHADO** — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** **M4 — Rig** (fechamento)
**Status:** implementado, testado (4 mutantes provados), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** nenhum

---

## 1. `rig.rubber_hose` — o membro SEM cotovelo

O look mais antigo da animação: os braços e pernas de macarrão dos anos 1930 (Fleischer, Disney inicial),
revividos pelo *Cuphead* e, em motion graphics, pela ferramenta de referência — o **`RubberHose` da Battle
Axe** (After Effects). **A graça do estilo é que o membro NÃO TEM junta**: é uma mangueira. Um solver de IK
te dá um cotovelo; este se recusa.

### O algoritmo: curvatura constante, achada por bisseção

Uma cadeia em que **toda junta gira o MESMO ângulo `α`** traça um **arco de círculo** — que é exatamente o
que "sem cotovelo" significa: uma curvatura constante ao longo do membro **não tem onde ser uma quina**. Então
o solve inteiro tem **uma incógnita**:

```
ache α tal que |endpoint(α) − raiz| = |alvo − raiz|
depois aponte o arco inteiro pro alvo
```

`|endpoint(α)|` cai **monotonicamente** da extensão total (`α = 0`, membro reto) até zero (`α = 360°/ossos`, a
cadeia fechada num círculo) — então uma **bisseção** acha o α (24 divisões, determinística, e **correta para
ossos de comprimentos DIFERENTES** também: o endpoint é *caminhado*, não assumido como a corda de um polígono
regular). Depois:

```
heading do 1º osso = heading(alvo − raiz) − heading(endpoint(α))
```

**HR-5:** o leaf parabólico `cos/sin` + bisseção. **Sem `asin`** — e sem nenhuma forma fechada que precisasse
de um. **Fora de alcance → reto**, apontado pro alvo (uma mangueira não estica).

## 2. `rig.skin_deformer` — a carne nos ossos

**Linear Blend Skinning** (*skeleton subspace deformation*; o "smooth skin" do Maya, o modifier Armature do
Blender, o vertex skinning de todo engine). É uma linha:

```
p' = Σ_j  w_j · ( R_j · (p − rest_origin_j) + posed_origin_j )
```

### Três inputs, porque uma pele precisa de uma BIND POSE

`in` (os pontos) · **`rest`** (o esqueleto como autorado) · **`posed`** (o mesmo esqueleto depois dos solvers).
Uma pele **é a DIFERENÇA** entre os dois — então o grafo **diz isso em voz alta**:

```text
skeleton ─┬───────────────────────> skin.rest
          └─> fabrik(target) ─────> skin.posed
grid ───────────────────────────────> skin.in
```

**A bind pose é um FIO** — não um snapshot tirado às suas costas num momento que você tem que lembrar. Dá pra
ver, e dá pra cortar.

### Os pesos: envelopes (o auto-bind do Blender), não bone heat

`w_j ∝ 1 / (distância do ponto ao SEGMENTO do osso j em repouso)^falloff`, normalizado. **Ao segmento, não à
junta** — senão um ponto ao lado do MEIO de um osso longo seria capturado por qualquer das pontas que estivesse
mais perto, e a pele **vincaria** ali. (A outra opção do Blender, *bone heat*, resolve um Laplaciano **sobre a
superfície da malha**; aqui não há malha, só pontos — não há superfície pra difundir.)

**Identidade quando nada se moveu:** `posed == rest` → toda mudança de frame é a identidade e os pontos saem
**exatamente** onde entraram (a regra do doc 39). É o bug que arruína todo bind, e é uma guarda.

**Artefato conhecido do LBS** — o colapso *candy-wrapper* em torções grandes — é inerente ao blend **linear**
de rotações (é por isso que cinema usa dual quaternions). Em 2D, nos ângulos que uma mangueira ou um membro
realmente atingem, não aparece. Documentado, não escondido.

## 3. As guardas — 4 mutantes provados VERMELHOS

| # | Mutante | Guarda |
|---|---|---|
| 1 | toda a dobra em UMA junta (= um cotovelo) | `the_tip_reaches_the_goal_and_the_curvature_is_constant` — os giros deixam de ser iguais |
| 2 | LBS **esquecendo o rest origin** (blend em direção ao osso) | `a_rigid_turn_of_the_skeleton_turns_the_skin_rigidly` **e** `a_rig_at_rest_moves_no_point` |
| 3 | `skin.posed` ligado no **rest** (a diferença vira zero) | `the_flesh_follows_the_bones_it_is_skinned_to` → *"the far end of the flesh swings (0)"* |
| 4 | (bug do oráculo, meu) `turns()` sem **wrap** de ±180° | acusava um cotovelo **que não existia**: `−318°` é `+42°` |

O #4 merece registro: o **teste** estava errado, não o código. `heading` vive num círculo, e a diferença de dois
headings que cruzam ±180° volta como `−318°` onde o giro é claramente `+42°`. Um oráculo que não modela o
domínio do que mede **acusa o inocente**.

E o teste da carne tem uma guarda barata e decisiva contra o modo de falha mais feio do skinning: **a pele não
rasga** — pontos vizinhos da tira continuam vizinhos, em todo tick.

## 4. Superfície nova (pro integrador)

| Item | Valor |
|---|---|
| Crates novas | `ph2d-node-rig-rubber-hose` · `ph2d-node-rig-skin-deformer` |
| Node ids | `rig.rubber_hose` (param `flip`) · `rig.skin_deformer` (param `falloff`; **3 inputs**: `in`/`rest`/`posed`) |
| Leaves | `fk.rs` e `pose.rs` seguem **byte-idênticos** entre as crates `rig.*` (`#[allow(dead_code)]` onde uma delas não usa tudo — a cópia **não pode divergir**, é o contrato dela) |
| Codegen | registry-init regenerado — **79** crates-nó |
| Contrato | **intacto** (8/2/1) · **zero** foundational |

## 5. **M4 Rig está FECHADO**

`rig.skeleton` · `rig.fk` · `rig.ik_2bone` · `rig.fabrik` · `rig.rubber_hose` · `rig.skin_deformer` — os seis
nós do plano, **sem `Domain::Rig`, sem descongelar contrato, sem ADR** (a decisão M4.N3, doc 40).

**Aberto no M4:** só os **FX de PASSE** (`glow`/`bloom`/`blur`/`vignette`/`levels`/`hue_shift`), que exigem o
**compositor HDR** — cross-module, **decisão do Enio**, não fan-out (o handoff manda PARAR e reportar).
