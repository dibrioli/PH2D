# O OSSO QUE DOBRA — os *bendy bones*, da lei ao painel (`line/Vector`, 2026-09-15)

> ⚠️ Este doc descreve o mundo **no dia em que foi escrito**. O estado vivo é o `CLAUDE.md` §5.

**F8 da [fila do módulo](../01_a_fila.md), pedido do dono em 2026-09-14.** Quatro waves, quatro
commits, e a lei da pele **não mudou uma linha**.

---

## §1 — Em uma linha

Um osso passou a poder **arquear**: ele parte-se em `N` sub-ossos ao longo de uma Bézier, o
desenho e o dedo seguem a curva, e o artista alcança os dois controlos no painel.

---

## §2 — O que o 1.º passo mediu, e que mudou o plano

A célula da F8 dizia duas coisas que a medição derrubou:

1. ⛔ **«o B-Bone ataca na ORIGEM as *arestas retas ao dobrar*»** — **refutado** por uma recusa
   medida um bloco abaixo dela na mesma página: subdividir com a população de amostras constante
   **piora** (`2,61 % → 4,94 %` a 24 sub-ossos) e, com o alcance já certo, não cura nada. ⇒ *o
   B-Bone é uma feature de AUTORIA — um rabo em S, um membro flexível —, não a cura da dobra.*
2. ⭐ **A lei da pele já é de `N` ossos.** O [`Skin`](../../../crates/ph2d-skeleton/src/lib.rs)
   mistura poses **rígidas** por peso, que é exactamente o que um osso curvo é (o Blender diz
   `Segments = N` e fabrica `N` ossos virtuais). ⇒ o que muda é **quem produz**, e é **um** sítio.

E dos «quatro consumidores» que a célula temia, só **dois** precisavam da curva: quem **desenha** e
quem **agarra**. A pele lê as juntas, a IK lê o losango — as duas querem raiz e ponta, que um osso
curvo continua a ter.

---

## §3 — As quatro waves

| # | O quê | Onde |
|---|---|---|
| W1 | A **lei pura**: `Bend`, `BoneSpec`, `frame`, `share`, `SkinBone::bent` | `ph2d-skeleton/src/bend.rs` |
| W2 | O **schema** (`Bone.segments` + `Bone.curve`) e o **produtor** | `ph2d-skeleton-ecs` · `skin_live::resolve_with` |
| W3 | O **desenho** e o **dedo** aprendem a polilinha | `ph2d-skeleton-render/src/body.rs` · `bone_pick` |
| W4 | O **painel**: *Segments* + as quatro alças | `ph2d-panel-skeleton` · `ph2d-app-skeleton/src/knobs.rs` |

`PROJECT_SCHEMA` **128 → 129** (conte o DELTA: `+1`). Degrau obrigatório pelo **postcard**, que é
posicional — um ficheiro de dois campos seria lido com quatro em silêncio.

---

## §4 — O ponto neutro é exacto POR CONSTRUÇÃO, não por tolerância

Esta é a decisão que atravessa as quatro waves, e ela tem **duas** metades que se encontram:

1. **A fábrica COLAPSA.** `SkinBone::bent` devolve UM osso quando `segments <= 1` **ou** a
   curvatura é recta, sem passar pelos frames. *Uma recta não precisa de `N` ossos para a desenhar.*
2. **E mesmo que não colapsasse, o resultado seria a identidade ao bit.** O ponto da curva está
   escrito como *«a recta MAIS a correcção das alças»* — com alças a zero a correcção é `0.0` e
   somar `0.0` a um finito é exacto —, e o frame sai da razão `(c₁−c₀)/(x₁−x₀)`, cujo numerador e
   denominador saem da **mesma expressão** quando a curva é recta ⇒ `1.0` exacto.

⇒ todo rig já autorado deforma-se e desenha-se **ao bit** como antes. As referências dos gates não
são goldens gravados (isso mediria a minha aritmética) — são a **porta antiga** chamada ao lado da
nova.

---

## §5 — Seis decisões que a medição tomou

| A decisão | Porquê, com o número |
|---|---|
| A escala do frame é **axial**, nunca uniforme | Com uma semelhança a arte fica **`+87 %`** mais gorda nas pontas de um arco forte — e a régua da dobra, que mede inversão, não vê nada |
| A curvatura **arqueia o corpo e não mexe a ponta** | A Bézier acaba na ponta **do osso**; se a movesse, a corrente abria fenda em cada junta ao dobrar. ⚠️ A minha 1.ª redacção do gate exigia o contrário |
| Os sub-ossos **partilham o eixo de repouso** | Com eixos próprios o `bump` deles somaria e partir um osso passaria a **engordá-lo** — um braço de 8 segmentos ganharia ~8× de influência contra o vizinho |
| A curvatura é uma **fracção do comprimento** | A mesma lei da `strength`: com unidades absolutas, escalar um personagem **endireitaria** todos os ossos dele em silêncio |
| `MAX_SEGMENTS = 32` | Medido: `1 → 2,0 %` de um quadro, `8 → 5,4 %`, `16 → 9,3 %`, **`32 → 17,9 %`**; a `64` um osso curvo já pede `~35 %` e um par come o quadro. A tabela vive no doc da const |
| A curvatura só é **pintada** com `segments > 1` | Num osso rígido ela é *provadamente* inerte, e há gate a dizê-lo pelo nome. Pintá-la seriam quatro números que gravam no documento e não mudam um pixel |

---

## §6 — O que as provas de mutação acharam

**22 mutações, 22 matam.** Três sobreviveram à primeira redacção, e cada uma foi um achado:

1. ⭐ **A metade dos SEGMENTOS do `is_rigid` é EQUIVALENTE** — com `n = 1` a fábrica devolve a
   identidade por construção. ⇒ aquela metade é **poupança de custo, não correcção**, e a
   equivalência ganhou gate próprio (provado por mutação, para não ser vácuo).
2. ⭐ **A normal de uma junta usar só UMA corda deixava a suíte verde** — o contorno continuava com
   o número certo de pontos e a subir com o arco. *Uma régua que conta pontos não vê para onde eles
   apontam.* ⇒ gate que mede `|n·entra| == |n·sai|`, com controlo a exigir que a fixtura dobre.
3. ⭐ **O `clamp` antes do `as u8` era redundante** — em Rust o cast satura e o `segments_of` fecha
   os dois lados a seguir. *Duas respostas à mesma pergunta divergem no dia em que alguém mexe numa
   delas* ⇒ o clamp saiu.

⚠️ **E o arnês da prova mentiu TRÊS vezes antes de dizer a verdade**, sempre no filtro: o regex não
casava o formato `nome --- FAILED` (quatro «SOBREVIVEU» falsos), uma âncora casava dois sítios, e
um `assert` de contagem estava errado. ⇒ **o arnês passou a ter controlo no próprio filtro** (a
árvore sã tem de ler zero reprovadas **e** a suíte tem de ter corrido) e cada mutação declara
quantos sítios a âncora deve casar.

---

## §7 — Seis coisas que uma leitura rápida do diff entende ao contrário

1. **`bone_segments` não foi substituída — ela foi DERIVADA.** Ela continua a existir e a devolver
   os mesmos bytes; hoje tira o primeiro e o último nó da polilinha. Quem quer as duas extremidades
   (a cinemática, o losango da IK, o gesto da ponta) continua a chamá-la, e está certo.
2. **`bend::frame` NÃO é chamado no caminho de um osso recto.** Ele existe para o osso curvo; a
   exactidão do neutro vem do colapso, e o gate sobre o `frame` recto guarda a *equivalência*, não
   o produto.
3. **A `share` multiplica um peso que já existia** — ela não é um peso novo. Num osso recto vale
   `1.0` ao bit, e `peso * 1.0 == peso`.
4. **O `sub: (u8, u8)` não é um índice de ordenação.** Ele diz *qual sub-osso de quantos*, e o
   desempate do órfão usa-o para achar o GRUPO por aritmética de índice — é por isso que os
   sub-ossos têm de sair consecutivos, e há gate.
5. **O painel esconder a curvatura não é um `ParamGate` genérico** — é uma condição escrita no
   `campos_do_osso`, derivada da mesma lei que a fábrica usa (`is_rigid`).
6. **O `PROJECT_SCHEMA` não sobe duas vezes.** A W4 mudou o **significado** das alças (de distância
   para fracção) e não a forma dos bytes, e o único valor no mundo é zero — o campo nasceu dois
   commits antes, nesta mesma linha por integrar.

---

## §8 — O que ficou ABERTO, com o mecanismo nomeado

- ⏳ **O esticão VARIA ao longo do osso.** Os nós saem do PARÂMETRO e não do comprimento de arco:
  medido, `1,129` nas pontas contra `1,003` no meio com as alças a `0,2 L` (**13 %**), e `87 %` a
  `0,6 L`. A cura publicada é a **equalização por comprimento de arco** (o `equalize_cubic_bezier`
  da referência), e ela custa exactamente a exactidão do ponto neutro deste ficheiro — um somatório
  de cordas não devolve `L` ao bit. ⇒ **fica por medir num smoke, não por escrever.**
- ⏳ **As alças não têm gesto de CANVAS.** Hoje elas autoram-se pelo painel. Arrastá-las sobre o
  desenho é o gesto natural (o mesmo da caneta), e o substrato já existe: a polilinha é pintada e
  o `bone_pick` já sabe medi-la.
- ⏳ **As tangentes AUTOMÁTICAS dos vizinhos** (o *Handle Type: Auto* do Blender) não existem. Elas
  são lei sobre a HIERARQUIA, e a hierarquia não vive na crate da lei de propósito — quem as
  construir escreve um produtor de `Bend`, não uma lei nova.
- ⏳ **O `[hero] unhandled event: Click(NodeId(…))`** do log do dono continua **nomeado e não
  medido** (ver o §7-quater do handoff do conta-gotas).

---

## §9 — Como se smoka

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

O tentáculo da cena tem **6 ossos já presos**. Clicar num osso enche a secção *Skeleton* com os
números dele; `Segments` acima de `1` faz aparecer *Curve In* e *Curve Out*.
