# 02 — A malha sobre a imagem: o estado da arte, medido

> **A pergunta:** como é que um desenho pintado passa a obedecer a um esqueleto — de onde vem a
> **malha** e de onde vêm os **pesos**?
>
> Ordem do dono, 2026-09-09: *«faça pesquisa para criar o modo mais intuitivo e eficaz de criar e
> fazer o bind da malha»*. Veredito dele sobre a recomendação: **«vamos tentar como vc recomenda»**.
>
> Página de leitura (para o dono): <https://claude.ai/code/artifact/0efd5bdd-c103-4f64-a965-7b863c9a8f5d>

---

## §1 — Triagem de licença PRIMEIRO (§0.9), e ela pára na primeira porta aberta

| app | licença medida (`pacman -Qi`) | porta |
|---|---|---|
| **Godot 4.7.2** | **MIT** | ⭐ portar, com atribuição. Corrido: `godot --headless --doctool` |
| **OpenToonz** | **BSD-3-Clause** | ⭐ portar, com atribuição. ⛔ sem porta de consola (medido no arsenal) |
| Krita · Inkscape · Synfig | GPL | observar a saída, nunca o fonte |
| Blender | GPL-2.0-or-later | idem |
| Spine · Live2D · Moho · After Effects · Rive (editor) | proprietário | só documentação pública |

⇒ **duas portas abertas**, e uma delas correu.

## §2 — ⭐ O que o ORÁCULO devolveu (Godot, MIT, corrido nesta máquina)

O `--doctool` despejou a API; o vocabulário do editor saiu do próprio binário.

**`Polygon2D`** resolve as três coisas num nó:

| membro | o que é |
|---|---|
| `polygon: PackedVector2Array` | o **contorno** |
| `internal_vertex_count: int` | os vértices do **miolo**, apendados depois do contorno |
| `polygons: Array` | a **triangulação** (listas de índices) |
| `uv: PackedVector2Array` | uma UV **por vértice** |
| `skeleton: NodePath` | o `Skeleton2D` que ele segue |
| `add_bone(path, weights)` · `set_bone_weights` · `erase_bone` | os **pesos, um vector por osso** |

**`Bone2D`**: `rest: Transform2D` · `apply_rest` · `set_length` · `set_bone_angle` ·
`autocalculate_length_and_angle`. **`Skeleton2D`**: `set_bone_local_pose_override` + uma pilha de
modificações (a IK deles).

**O vocabulário do editor, verbatim do binário:**

```
Create Polygon · Create Polygon & UV · Create Polygon Points
Create Internal Vertex · Remove Internal Vertex
Clear UV · Scale Polygon · Grid Step
Paint Bone Weights · Sync Bones · Sync Bones to Polygon
```

⛔⛔ **A porta aberta é o CONTRA-EXEMPLO.** O Godot não tem malha automática nem pesos
automáticos: cada vértice do miolo entra por `Create Internal Vertex`, e o peso pinta-se com
`Paint Bone Weights`. **Portá-lo seria portar o trabalho** — e é a única das sete ferramentas
estudadas que oferece o modo manual como *primeiro* passo.

## §3 — As quatro maneiras de conseguir uma malha, e quem usa cada uma

| maneira | quem | veredito |
|---|---|---|
| **um a um, à mão** | Godot · Rive | o mais trabalhoso |
| **grelha sobre a caixa** | AE *Mesh Warp* · Moho *Smart Warp* | barato e desperdiçado — metade dos pontos cai no vazio, e a borda (onde o olho repara) fica amassada |
| **traçada do recorte** | Spine *Create Hull* · Live2D *Automatic Mesh Generation* · OpenToonz *Plastic* | **o padrão do campo** |
| **só alfinetes** | AE *Puppet* | o mais intuitivo — **não existe interface de malha nenhuma** |

E os **pesos**: `automático por distância` em AE, Spine, OpenToonz, Moho e Blender (*With
Automatic Weights*, difusão de calor); `pincel` como **correcção**, nunca como primeiro passo.
⛔ Só o Godot exige pintar de raiz.

⭐⭐⭐ **A lei do campo, numa frase:** *a malha deixou de ser autorada e passou a ser consequência;
o artista aponta o que se move.*

## §4 — O nosso balanço, MEDIDO

| | |
|---|---|
| ✅ **peso automático** | `ph2d_skeleton::Skeleton::weights_at` — derivado por distância, **sem tabela guardada**; custo medido `0,146 %` de um quadro |
| ✅ **a pele já é agnóstica de mídia** | `SkinBind::source` são **bytes opacos**, e o doc dela já dizia porquê: *«serve um `VecPath` hoje e uma malha raster amanhã sem uma variante nova nem um schema por mídia»* |
| ✅ **formas vetoriais deformam** | ponta a ponta, com o *Bind* no painel |
| ⏳ **a malha sobre a imagem** | nada no app traça onde a tinta acaba (o `VecContour` é o efeito de anéis do CorelDRAW, não um traçador) |
| ⏳ **desenhar imagem entortada** | `draw_image_rgba_transformed` é **um afim por imagem** — vira, gira e escala; não entorta |

⭐⭐ **E temos uma vantagem que nenhuma das sete tem:** aqui o artista já desenha vetor, e vetor já
deforma com osso. Onde o Spine precisa de **fabricar** um contorno traçando a silhueta, aqui o
contorno pode ser uma forma que ele desenhou com a caneta que já usa.

## §5 — A rota de desenho, medida antes de escolhida

| rota | o que custa | veredito |
|---|---|---|
| **(A) recorte + afim por triângulo, no Vello** | `push_clip(forma)` **já existe** (`scene.rs`), `draw_image_rgba_transformed` já existe, e o compositor põe o Vello **por cima** do passe de sprites | ⭐ **escolhida** — exacta (afim por partes), zero capacidade nova de render |
| (B) instância de sprite por célula | a instância já carrega um **basis 2×2 completo** (*«rotation + scale + skew exactly»*) e UV própria — o 9-slice já emite N pedaços assim | ⛔ um quad com afim é um **paralelogramo**; células vizinhas abrem fenda ou sobrepõem-se |
| (C) pipeline de triângulos texturados | o mais rápido | ⏸️ é a optimização **com razão medida**, se a (A) não couber no quadro |

## §6 — O desenho escolhido

**Porta principal — prender a imagem sem ver malha nenhuma.** O artista escolhe a imagem e o
esqueleto e aperta *Bind*, o mesmo botão das formas. A malha é traçada da própria tinta e **nunca
aparece na tela**. Um número só, *densidade*.

**Escape — o contorno é uma forma que ele desenha.** Quando o traço automático pega o que não
devia (uma sombra, um brilho), ele desenha a silhueta com a caneta de vetor e diz que aquela forma
é o recorte. O miolo continua deduzido. Zero interface nova.

⛔ **Fora, com motivo:** a grelha uniforme (o campo afastou-se dela) e o vértice-a-vértice do
Godot como *primeiro* passo (quando existe noutras ferramentas, existe como conserto).

## §7 — As portas, uma por pergunta

| pergunta | porta |
|---|---|
| onde a tinta acaba? | [`ph2d_poly2d::contour`](../../crates/ph2d-poly2d/src/contour.rs) |
| quantos vértices o artista paga? | [`ph2d_poly2d::simplify`](../../crates/ph2d-poly2d/src/simplify.rs) |
| em que triângulos isso se divide? | [`ph2d_poly2d::triangulate`](../../crates/ph2d-poly2d/src/triangulate.rs) |
| a malha desta imagem, numa chamada | [`ph2d_poly2d::mesh_of`](../../crates/ph2d-poly2d/src/mesh.rs) |
| que peso cada ponto tem? | `ph2d_skeleton::Skeleton::weights_at` — **já existia** |
| onde a ligação mora? | `SkinBind::source` — **já existia**, e é opaco de propósito |

⚠️ **A crate nova tem ZERO dependências**, pela mesma razão que criou a `ph2d-affine`: o
esqueleto serve vector, raster, Flip e 3D, e não pode puxar a cena vectorial — nem o `wgpu`.

⚠️⚠️ **Uma lei ficou em DOIS sítios, e a nota está nos dois:** o `triangulate` daqui tem um gémeo
em `f32`, o `ph2d_flip_render::fill::triangulate`, atrás daquela parede de `wgpu`. **A medição que
autoriza fundi-los é nomeada:** correr os goldens do Flip com a conversão `f32 → f64` posta — o
*ear-clipping* decide por produtos cruzados, e uma decisão degenerada pode virar de sinal. É de
quem possui o Flip.

## §8 — ⛔ Recusas MEDIDAS desta pesquisa

| recusa | o mecanismo |
|---|---|
| **Portar o fluxo do Godot** (a única porta MIT com esta feature) | Medido no binário dele: `Create Internal Vertex` um a um e `Paint Bone Weights` à mão, **sem nenhum automático**. É o fluxo mais trabalhoso das sete ferramentas estudadas, e nós já temos a metade que lhe falta (peso derivado). |
| **Grelha uniforme sobre a caixa da imagem** | Metade dos vértices cai onde não há tinta, e a borda — onde o olho repara — fica presa a uma grelha que não a segue. O campo inteiro afastou-se dela; sobrevive só como *warp* de propósito geral (AE *Mesh Warp*, Moho *Smart Warp*), que é outra feature. |
| **Desenhar a malha por instância de sprite** (rota B) | A instância carrega um basis 2×2, logo cada célula sai **paralelogramo**; duas células vizinhas de um warp real não partilham aresta, e a costura abre fenda. Medido lendo o `sprite.wgsl`. |
| **Guardar a UV por vértice** | É o *vector paralelo* que o esqueleto já proíbe por escrito nos pesos: a UV de um vértice de repouso **é** a posição dele sobre o tamanho da imagem. O que viaja é o tamanho (um par), não uma lista. |
| **Limiar de alfa em `128`** | Come os dois ou três pixels em que uma borda suavizada sobe de `0` a `255`: a silhueta fica **por dentro** do desenho e o artista vê a própria arte a ser aparada ao deformar. Medido no gate `a_soft_edge_is_not_shaved_off`. |
