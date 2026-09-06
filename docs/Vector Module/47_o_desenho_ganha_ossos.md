# 47 — O desenho ganha OSSOS (estudo 42, item 5)

> **A lacuna, na palavra do estudo:** *"Sem eles um personagem só anima como recorte de papel."*
> ([42 §3](42_o_que_falta_ao_vetor.md), tamanho **G**, e *"é a aposta inteira do Rive"*.)
>
> ⚠️ **E o §6 do mesmo estudo já mediu a armadilha:** as seis crates `ph2d-node-rig-*` **parecem**
> ossos no vetor e não são — elas deformam a **nuvem de pontos do grafo** de Motion. O
> `ph2d-vec-scene/src/lib.rs:12` promete *"Rig/bones… entram na Fase 1"* desde a ADR-0108, e a Fase 1
> nunca os trouxe. *Um nome parecido em seis crates é a razão pela qual esta lacuna passou dois meses
> a parecer fechada.*

---

## §1 — O estado da arte (o que existe, e o que foi ABANDONADO)

| Ferramenta | O modelo | O que interessa aqui |
|---|---|---|
| **Rive** | Osso **na hierarquia** (`RootBone`/`Bone`); ligação por `Skin` + um `Tendon` por osso (a matriz de repouso invertida) e um `Weight` por vértice — com **`CubicWeight`**, que dá pesos PRÓPRIOS às duas alças | É o modelo que copiamos: um osso é um nó da mesma árvore, e as três metades de um vértice skinam-se **em separado** |
| **Spine** | Ossos + slots + malhas; pesos por vértice de malha | Deforma **malha**, não curva. O contorno vira triângulos e deixa de ser editável |
| **Moho** (Anime Studio) | O referencial de rig **vetorial**: cada osso tem uma **região de influência** (*Bone Strength*), e três ligações — *region* (automática) · *point* (rígida ao osso mais perto) · *flexi* (pesos) | A lei de pesos daqui: raio por osso, derivado do comprimento. E o *point binding* é exactamente o nosso caso degenerado |
| **Blender** | *Armature modifier*: envelope (raio) ou *bone heat* (Laplace sobre a MALHA) | O *bone heat* exige malha — não há malha numa curva. Fica de fora por falta de substrato, não por preço |
| **Illustrator** | **Não tem ossos.** Só *Puppet Warp* (pinos) | O nosso Envelope-Pins (ADR-0129) já é isso, e é outra coisa |
| **After Effects** | *Puppet* (pinos sobre malha); rigs de osso só por expressão (DUIK, RubberHose) | idem |

⛔ **O que foi TENTADO e perdido:** o Flare (Rive 1) tinha **jelly bones** — ossos que dobravam a
forma numa curva em vez de a rodar em torno de uma junta — e a reescrita para o Rive 2 **perdeu-os**;
nunca voltaram ([42 §4.2](42_o_que_falta_ao_vetor.md)). *Não os construa nesta wave sem ler aquela
linha: eles não são um refinamento da LBS, são outro deformador.*

---

## §2 — O desenho, e a PORTA ÚNICA de cada pergunta

### §2.1 — *Onde vive um osso?* → **é uma ENTIDADE, como tudo o resto**

`VecBone { length, strength }` pendurado numa entidade com `Transform`. A **hierarquia da cena é o
esqueleto**.

⭐⭐⭐ **A cinemática direta não se escreve: ela já corre.** `propagate_transforms` compõe a pose de
um filho com a do pai — que é a definição de FK. A alternativa (uma árvore de ossos DENTRO de um
componente) seria uma segunda hierarquia, e a ADR-0110 rejeita-a pelo nome: *uma hierarquia*.

Vem de graça, sem uma linha: **undo · save · o olho · o cadeado · renomear · reparentar · o gizmo de
sprite a rodar o osso · e a TIMELINE a animar o `Transform` dele.** ⚠️ *Este último é o item 1 do
estudo 42 a pagar o item 5* — a timeline já anima a pose de um `VecPath`, e um osso é uma pose.

⛔ **Um osso NÃO é um `VecPath`.** Ele não entra na cena vectorial, não exporta para SVG, não tem
tinta. O que se vê é **overlay** (a mesma família da gaiola do Envelope).

### §2.2 — *Onde vive a ligação?* → **na FORMA**, um componente por forma

`VecSkin { source, bones }` na entidade do caminho. Espelha o `VecEnvelope` em tudo o que é o padrão
da casa (a fonte autorada viaja em **bytes postcard** para o `ph2d-ecs` não depender do
`ph2d-vec-scene`; o recook reescreve o caminho da cena a cada quadro).

⛔ **E NÃO é um container**, ao contrário do Envelope — a diferença é medida: o Envelope precisa de
um container porque a **gaiola não tem outra casa** (não é entidade). Aqui o esqueleto **já são
entidades**, então a pergunta *"quem é o dono do que é partilhado?"* não existe. Uma forma presa
fica onde o artista a pôs.

### §2.3 — *Onde vivem os PESOS?* → **em lado nenhum: eles são DERIVADOS**

⚠️⚠️ **Esta é a decisão que mais código apaga, e ela vem de um aviso que já estava escrito no
repo.** O doc do `VecVertex::corner_radius` diz, sobre porque o raio mora dentro do vértice:

> *"e não num vetor paralelo ao lado dos `verts`, de propósito: dezenas de operações inserem, apagam,
> invertem e soldam vértices, e cada uma delas teria de lembrar de mexer no vetor paralelo também."*

Uma tabela de pesos indexada por ordem de varredura **é exactamente esse vector paralelo**. O Rive
paga-o (e por isso re-liga quando se edita a forma).

⇒ **Guardamos o BIND, nunca o peso.** O bind é `(fonte autorada, matriz de repouso de cada osso)`, e
o peso é função pura de `(ponto, eixo de repouso, raio)`. Consequências:

- editar a forma (acrescentar um vértice, soldar, inverter) **re-pesa sozinho** — não há o que
  dessincronizar;
- mexer o esqueleto no repouso re-pesa sozinho (é o *region binding* do Moho);
- e a pintura de pesos, quando vier, é uma camada de **excepções** por cima — a mesma forma que as
  excepções de instância da `line/components`.

⭐ **E derivar é barato, MEDIDO:** uma peça de **200 vértices sobre 12 ossos** (600 pontos, cada um
pesado contra cada osso) custa **24,31 µs por quadro — `0,146 %` de 16,7 ms** (`cargo test -p
ph2d-vec-skin --release measure_the_price_of_deriving_the_weights_every_frame -- --ignored
--nocapture`, load 4,79). ⇒ **não há memo**, e não é preguiça: um resumo guardado seria estado
derivado a envenenar o undo, exactamente como o `FieldProfileSource` do 3D Modeling decidiu pelo
mesmo preço.

### §2.4 — A LEI dos pesos

Para o ponto `p` e o osso `j`, com `d_j` = distância de `p` ao **segmento** de repouso do osso e
`r_j = strength_j · length_j` o raio de influência:

```text
w_j = (1 − (d_j/r_j)²)²     se d_j < r_j        (bump C¹, zero na borda)
    = 0                     senão
p'  = Σ ŵ_j · (M_j · p)     com ŵ = w normalizado         ← Linear Blend Skinning
```

⛔ **A alternativa GLOBAL (`1/d²`, sem raio) foi rejeitada com o mecanismo:** ela nunca deixa um
ponto órfão, mas um ponto **longe de tudo** recebe pesos quase iguais de todos os ossos e passa a
seguir a **média do esqueleto** — a aba de um chapéu 200 unidades acima de um esqueleto de 100 dá
`0,44 / 0,28 / 0,28` para cabeça / tronco / perna, e a aba **atrasa-se** em vez de seguir a cabeça.

⇒ **O órfão resolve-se por FALLBACK, não por suporte infinito:** um ponto fora de todos os raios
prende-se **rigidamente ao osso mais próximo** — que é literalmente o *point binding* do Moho. Nunca
fica para trás, nunca segue uma média.

⭐⭐ **E é o suporte finito que torna o botão *Bind* SEM CERIMÓNIA honesto:** quando o artista não
aponta um osso, ele prende a forma a **todos** os ossos da cena — e isso dá o **mesmo desenho** que
prender ao esqueleto certo, porque um esqueleto longe fica fora do raio de todo ponto (peso `0`, e a
normalização devolve os mesmos números) e um ponto órfão prende-se ao mais **próximo**, que é do
esqueleto certo. Gate: `binding_to_the_whole_scene_draws_the_same_as_binding_to_the_right_skeleton`.
⛔ Com a lei global isto seria **falso** — mais uma razão pela qual ela caiu.

⚠️ **O raio é DERIVADO** (`strength = 1` ⇒ o osso alcança **um comprimento dele** a partir do próprio
eixo), nunca um número escolhido: um esqueleto grande e um pequeno pesam igual porque a lei é
adimensional em `d/r`.

### §2.5 — A pose de repouso é a IDENTIDADE, e isso é o gate

`rest_j` = o afim `osso → forma` no instante do bind. A cada quadro:

```text
M_j = S_agora⁻¹ ∘ B_j_agora ∘ rest_j⁻¹
```

Parado no repouso, `M_j = I` **para todo osso** ⇒ `Σ ŵ_j · p = p`, seja qual for o peso. *É a lei da
casa (todo motor novo é no-op no ponto neutro), e aqui ela cai de graça da álgebra* — não é uma
guarda escrita à mão.

### §2.6 — O gesto

Modo **Bone** (o 17.º). Arrastar no vazio faz um osso: a origem no press, o comprimento e o ângulo no
arrasto. **O pai é o osso SELECCIONADO**, e o osso novo fica seleccionado ⇒ arrasto-arrasto-arrasto é
uma cadeia. Clicar num osso existente selecciona-o (é assim que se ramifica). A origem **encaixa na
ponta do pai** dentro de 12 px.

⛔⛔ **E POSAR não passa pelo gizmo de sprite — isso foi medido durante a implementação.** O gizmo
dimensiona-se pela caixa da GEOMETRIA (`vec_gizmo_view::anchor_half` exige um `VecPathRef`), e um
osso não tem geometria nenhuma: a caixa sai `0 × 0` e as alças colapsam num ponto. ⇒ o osso posa-se
**agarrando o osso**, que é o gesto do Spine e do Moho:

| onde a mão pega | o que acontece |
|---|---|
| o **CORPO** do osso | ele **gira** — a origem fica, a ponta segue o ponteiro |
| a **BOLINHA** da junta | ele **desloca** |

*Duas coisas diferentes pedem dois gestos*: sem o segundo, um esqueleto inteiro nunca sai de onde
nasceu, e a única saída seria o painel de Transform. ⚠️ O raio da bolinha é **um número só**
(`ph2d_vec_render::BONE_JOINT_R_PX`), lido pelo desenho **e** pelo hit-test — dois fariam o dedo
girar quando queria deslocar.

### §2.7 — O que uma forma PRESA deixa de aceitar, e por quê

⛔ **A alça de raio (Fillet/Chamfer) RECUSA uma forma presa** — ela junta-se à lista do
`corner_handles::has_derived_verts`, ao lado da forma viva, do conector, do morph e do filho de
envelope. A razão é a mesma dos quatro: o `recook` reescreve os `verts` **todo quadro e sem
condição**, então um raio autorado na forma viva desapareceria no quadro seguinte, **em silêncio** —
a ferramenta pareceria funcionar.

⚠️ **A saída é barata e é a que o modelo já oferece:** *Release* → arredondar → *Bind* outra vez (o
bind é sempre um *re-bind na pose actual*). ⛔ E **não** se resolve "como o blend", que é o único
permitido nessa lista: ali a escrita PARA quando o artista assume o spine; aqui nada a pára.

*Quem descobriu isto foi um gate que já existia* (`every_live_host_that_rewrites_verts_is_named_by_
the_radius_handle_policy`), e ele não pergunta se a escrita é boa — pergunta se alguém **decidiu**.

---

## §3 — Onde isto encosta em contrato congelado ou schema

| Superfície | Encosta? | Prova |
|---|---|---|
| §6 **Nodes** (`NodeOp`/`OpResolver`/`NodeManifest`) | **não** | nada desta wave é um nó; as `ph2d-node-rig-*` não são tocadas |
| §6 **Tools** (`Tool=12`/`PanelEvent=4`) | **não** | o modo novo é um variante de `DrawMode`, que não é contrato congelado; nenhum método de trait novo |
| §6 **Vector data-model** (`ph2d-vector-doc`) | **não** | o gate `architecture_vector_contract_surface` varre só `ph2d-vector-doc` + `-traits`, e esta wave não lhes toca |
| `VEC_SCENE_SCHEMA_VERSION` | **não** | o `VecPath` **não ganha campo**: os pesos são derivados e o bind vive num componente ECS |
| `PROJECT_SCHEMA` | **não** | o `ComponentBlob` é chaveado por `blake3(nome canónico)` — a lei está escrita no [`registry.rs`](../../crates/ph2d-ecs/src/scene/registry.rs) ao lado do `VecMorphMachine`: um ficheiro antigo simplesmente **não tem** o blob, e a entidade volta sem esqueleto |

---

## §4 — A UI, pelas QUATRO condições independentes

1. **existe** — o pill *Bone* na fileira TOOL; a seção **SKELETON** com `Bind` · `Release` ·
   `Length` · `Strength`.
2. **é pintado e registado** — `populate_*` + `paint_*`, e o `hit_indexed_ids_are_registered` cobre
   os ids literais.
3. **o clique chega ao barramento** — o `is_button`/`forwards_plain_click` da família (a lição do
   [bug #29](BUGS_vector.md): três rotas mortas ao mesmo tempo, e o gate de registo estava verde).
4. **a sequência leva a algum lado** — gate de COSTURA com o gesto REAL (Down+Up sobre o rect
   pintado), oráculo = `EditorAction`, nunca `WidgetEvent`.

---

## §5 — Aberto, e nomeado (⛔ não são esquecimentos)

- **IK** — a matemática existe e está gateada em `ph2d-node-rig-{ik_2bone,fabrik,rubber_hose}`, mas
  atrás do `NodeOp`; alcançá-la daqui é extrair a lei para uma folha. Wave própria.
- **Pintura de pesos** — a camada de excepções do §2.3.
- **Smart Bones** (Moho) — um ângulo de osso a conduzir uma acção inteira. É o `VecMorphMachine`
  conduzido por um osso; o substrato já existe.
- **Reset Pose** — precisa da pose local de repouso no osso; hoje o undo cobre.
- **Jelly bones** — ⛔ leia o §1 antes.
