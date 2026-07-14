# HANDOFF de CONTINUAÇÃO — `line/Vector` (2026-07-14)

> **Para:** o **próximo implementador** da linha `line/Vector`.
> **De:** o agente que fechou o undo (ponto fixo), a lasca do Build, e construiu o **Blend**.
> **Estado:** a linha está **aberta**, verde (6986/6986), e tem **um defeito conhecido, medido e
> NÃO consertado** (§2). O Enio mandou você **começar pela auditoria do algoritmo de Blend** — ele
> ainda não é perfeito, e ele sabe disso.
>
> Leia **§0** (como se trabalha aqui) e **§2 inteiro** antes de tocar em qualquer coisa.

---

## §0 — Como se trabalha aqui (Modo L) — **não é opcional**

Você é **uma linha autônoma** numa jornada multi-agente
([GUIA_JORNADA_MODO_L.md](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) ·
[DIRETRIZ §1.5](IntegracaoMultiAgente/DIRETRIZ.md) · ADR-0106 · ADR-0107).

| | |
|---|---|
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector`, branch `line/Vector` |
| **Você commita** | `git commit --no-verify -m "..."` — local, à vontade, em blocos |
| **Você NÃO** | **integra** · **pusha** · roda **`ship.sh`**. Nunca. Por conta própria é violação de protocolo (CLAUDE.md §0.7) |
| **Foundational** | você **PODE e DEVE** tocar (ADR-0107). Ao **criar** foundational novo, projete para isolamento (módulo irmão, extensão append-only) |
| **PARE e reporte ao Enio** | só em 2 casos: **contrato congelado** (CLAUDE.md §6) ou **rebase conflitando fora dos seus arquivos** |
| **Você fecha** | escreve o **handoff de integração** (DIRETRIZ §1.5.9) e **PARA** |

### A regra mecânica que já custou caro nesta linha (leia mesmo achando óbvio)

> **O `cwd` do shell DERIVA no meio do turno.** Aconteceu comigo **duas vezes** hoje: um `cargo`
> foi parar no repo primário (o sinal é `failed to create directory .../target — Not a directory`:
> o `target/` da raiz é um symlink quebrado, de propósito).
>
> **Todo comando começa com `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector &&`.**
> **Toda mutação de arquivo usa caminho ABSOLUTO.** Para ler o `main`, use refs
> (`git show main:arquivo`), **nunca** o filesystem.

Memórias: [[feedback_sed_relative_path_hits_primary_cwd]] ·
[[feedback_perl_utf8_mojibake_use_edit_tool]] (texto acentuado **só** via ferramenta Edit) ·
[[feedback_backticks_in_commit_message_are_command_substitution]] (use `git commit -F <arquivo>`).

### Ritmo

- **Inner loop:** só `cargo check -p <crate>`.
- **Gate batched, 1× no fechamento:** `cargo nextest run --workspace --no-fail-fast` +
  `cargo clippy --workspace --all-targets` + `rustup run 1.95 cargo fmt --all -- --check` + `typos`.
- **Smoke 1× no fim**, com o comando pronto incluindo o `cd` ([[feedback_ready_to_smoke_example]]).
- **Não commite antes do smoke do Enio** quando o incremento for visual.

---

## §1 — Estado da linha

| | |
|---|---|
| **Base** | tip do `main` de hoje (a jornada de 6 linhas **integrou**; o Blend entrou **sem smoke**, e o Enio o smokou depois — ver §2) |
| **Commits não integrados** | 4 (o Blend + os 3 consertos que o smoke pediu) |
| **Gate no HEAD** | **6986/6986 verde** · clippy 0 · fmt limpo · typos limpo |
| **Tier** | `workstation` ⇒ Modo L |

**O que o Enio JÁ aprovou no smoke:** Live Corners · Shape Builder · undo/redo · a lasca do Build.

**O que ele reprovou e eu consertei hoje** (tudo commitado, com gate):

1. as intermediárias **ondulavam** (a reta do documento é a cúbica **degenerada**, e a
   parametrização dela não é uniforme);
2. **3 das 10 arestas** da estrela **viravam pontos** (o corte era decidido pela BORDA da peça, que
   é sempre uma âncora — a fronteira onde o `f64` empata);
3. a forma nova **nascia atrás** (o `sync` dava o maior `RootOrder`, e maior = FUNDO);
4. as quinas convexas casavam com os **VALES** da estrela (faltava o termo de **bending**).

---

## §2 — ⚠️ COMECE AQUI: **a auditoria do algoritmo de Blend**

**Ordem do Enio.** O algoritmo *melhorou muito* (ele disse "está melhor"), mas **não está certo**,
e há **um defeito medido e aberto**.

### 2.1 — O defeito: **o quadrado GIRA a caminho do círculo**

Blend de um quadrado para um círculo: os intermediários **rodam 45°** e voltam. O Enio: *"o porquê
da rotação?"*

**Medido** (o probe está no histórico do git, é fácil refazer):

```
quinas do quadrado:   -135°   -45°    45°   135°
âncoras do círculo:      0°    90°   180°   -90°     virada (sen,cos) = (0,1) em TODAS
casamento escolhido:  cada quina → a âncora 45° adiante   ⇒ giro de 45°
```

### 2.2 — A causa (hipótese forte, com evidência)

**Uma âncora de um contorno SUAVE não é uma feature — é artefato da parametrização.** As 4 âncoras
do círculo existem porque a elipse é cozida em 4 cúbicas; o artista nunca as autorou, e a virada
delas é `(0, 1)` (perfeitamente suave). Não há nada ali para casar.

E o motor **obriga** cada âncora da forma menor a casar com uma âncora da outra (`align` escolhe
`min(n,m)` nós, e os nós são **âncora↔âncora**). Então a quina a 45° é forçada para uma âncora a
0°, 90°, 180° ou 270° — e o melhor de quatro casamentos ruins é o giro de 45°.

> **A resposta certa não está no conjunto de candidatos.** A quina a 45° devia casar com o ponto
> do círculo a 45° — que fica **no MEIO de um segmento** (arco 0,125). Nem a DP (nós âncora↔âncora)
> nem o degradado `rotation_only` (fases âncora↔âncora) conseguem expressá-lo.

### 2.3 — O que auditar (não confie na minha hipótese: **meça**)

1. **Reproduza** o giro e confirme os números acima. O caminho mais curto é um teste em
   `crates/ph2d-vec-blend/src/tests.rs` que imprime, para cada nó, o ângulo da âncora de A e o da
   de B (eu apaguei o meu probe; reescrevê-lo é 20 linhas).
2. **Pergunte o que é uma feature.** A minha aposta: só é candidata a nó a âncora cuja **virada**
   passa de um limiar (`|sen| > ε`) — uma quina de verdade. O resto do contorno deve ser mapeado
   por arco **contínuo**, não por âncora.
3. **O caso degenerado é o mais importante:** quando **nenhuma** das duas formas tem feature (dois
   círculos, dois blobs suaves), não há nó nenhum — e aí a correspondência é uma **fase contínua**,
   que hoje é buscada **só em candidatos âncora↔âncora**. Uma varredura densa de fase (+ refino)
   provavelmente resolve o círculo→círculo E o quadrado→círculo.
4. **Não regrida o que já está gateado.** Os 15 gates de `ph2d-vec-blend` são mutation-tested; se
   um deles ficar vermelho, você quebrou algo que o Enio já aprovou. Em especial:
   `no_convex_corner_ever_marries_a_reflex_vertex` (a lição mais cara do dia) e
   `a_morph_between_two_polygons_is_a_polygon`.
5. **O gate que FALTA** (e que o Enio acabou de te dar de graça): *"um quadrado não gira a caminho
   de um círculo"*. Escreva-o **primeiro**, veja-o **vermelho**, e só então mexa no motor. O
   oráculo: a orientação da forma do meio (o ângulo do vértice mais distante do centro, ou a
   inércia principal) não pode variar monotonicamente com `t`.

### 2.4 — O que **não** é a causa (já descartei, não gaste tempo)

- **Não** é o lerp de coordenadas. O lerp está certo; é a correspondência que está errada.
- **Não** é a falta de subdivisão (isso era o defeito ANTERIOR, e está consertado: o quadrado sai
  subdividido com os pontos que a outra forma pede).
- **Não** é o termo de bending (ele está lá e funciona — foi ele que consertou o casamento
  quina↔vale). No círculo ele é **neutro**, porque todas as viradas são iguais.

### 2.5 — O horizonte, depois disso

O motor é **lerp de coordenadas**. Ele encolhe a forma no meio do caminho e pode auto-intersectar
numa rotação grande. O estado da arte é **as-rigid-as-possible** (Alexa 2000) / **trabalho mínimo**
(Sederberg & Greenwood 1992, do qual já usamos a espinha: a DP e o termo de bending). **A
correspondência era o pré-requisito dos dois** — e é ela que você está auditando.

---

## §3 — O motor, como ele está hoje (leia antes de mexer)

`crates/ph2d-vec-blend/` — **puro**, sem ECS, sem shell. `kurbo` confinada nele (como na booleana).

| Arquivo | O quê |
|---|---|
| `lib.rs` | `Outline` (o contorno como cadeia de cúbicas + arco acumulado), o **corte**, o lerp, `morph`/`steps`, `path_from` |
| `matching.rs` | **A CORRESPONDÊNCIA** — `search`/`align`/`dp_from`, o mapa monótono, as viradas |
| `tests.rs` | 15 gates |

### As 4 decisões que sustentam o motor (quebrar qualquer uma reabre um bug que o Enio já viu)

1. **A reta entra na forma CANÔNICA** (controles a ⅓ e ⅔). No documento, uma reta é a cúbica
   **degenerada** `(P0,P0,P3,P3)` — geometricamente reta, mas com **parametrização não-uniforme**.
   Cortada em posições de arco diferentes, ela devolve controles em frações diferentes de cada
   aresta, e o lerp de frações desalinhadas **entorta a reta**. Medido: 0,24 unidade numa forma de
   tamanho 2 (12%).
2. **O segmento se decide pelo MEIO da peça, nunca pela borda.** Toda borda de peça É uma âncora, e
   âncora é fronteira entre segmentos: perguntar "de quem é este ponto?" ali é perguntar de que lado
   de um empate o `f64` caiu. Quando caía errado, a peça **colapsava num ponto** e a aresta **sumia
   do pareamento** (3 das 10 arestas da estrela).
3. **O corte é na UNIÃO** (as âncoras de A + as pré-imagens das de B). É isso que **subdivide** a
   forma menor com os pontos que a maior pede — e o que preserva as quinas das DUAS.
4. **O custo da correspondência tem 3 parcelas**, todas adimensionais e da mesma ordem (peso 1,0 —
   um empate honesto, não um botão a calibrar):
   - **posição** (com cada forma normalizada: centro + escala RMS) — compara FORMA, não lugar;
   - **bending** (a virada, como o par `(sen, cos)` — **sem `atan2`**, HR-5): impede uma quina
     **convexa** de casar com um vértice **reentrante**;
   - **distorção de arco**: dois nós vizinhos cobrem frações de perímetro parecidas nas duas formas.

### Gotchas

- **O `offset` (Rotate Match) é RELATIVO ao automático**, e **não re-decide o sentido**. Quando ele
  re-decidia, uma forma simétrica devolvia o mesmo resultado físico e **o botão parecia não fazer
  nada** — um escape inerte é pior que escape nenhum.
- **`DP_BUDGET`**: acima de `n·m³` a DP cai para `rotation_only` (um nó = a rotação de antes). O
  degradado é seguro, mas é **exatamente o caminho que produz o giro do §2** — cuidado ao auditar.
- `PH2D_BLEND_LOG=1` imprime, por passo, o nº de vértices e quantos têm alça de curva.
- **Cena de smoke: `PH2D_BUILD_SMOKE=7`** (quadrado → estrela, 3 passos). Para o círculo, desenhe à
  mão (a Shape tool: elipse + retângulo, selecione as duas, seção **Blend**).

---

## §4 — A FILA (a ordem é do Enio)

1. **A auditoria do §2** — *"comece pela auditoria desse algoritmo, que ainda não é perfeito"*.
2. **Morph vivo** (o `t` animável). O desenho está pronto e é o do **conector**: uma entidade cuja
   geometria é função pura da relação, re-cozida por frame (`connector_live`). O motor **já serve**
   (`morph(t)` existe e está gateado). É o que transforma o Blend numa feature de **animação**.
3. **Envelope / puppet warp**.
4. Do Illustrator, o que ainda falta no Blend: **Replace Spine** (os passos seguem um caminho
   desenhado) e **Smooth Color** (o nº de passos sai do degradê).
5. Aberto de antes: **Live Path Effects como nós** (o multiplicador; a costura fonte≠cozido do
   ADR-0121 é o pré-requisito) · tipos de quina · texto em caminho · trim path · repeater · largura
   variável.

---

## §5 — Dívidas e minas que eu declaro

- ⚠️ **Os botões Arrange de z-order (To Front / Backward / …) estão MORTOS.** Eles chamam
  `VecScene::reorder_path`, que muta a ordem do **vetor da cena** — e a projeção de z reescreve essa
  ordem a partir da **árvore** a cada frame (ADR-0110). **Quem quer mandar no z escreve no
  `RootOrder`** (`vec_zorder::restack` é o exemplo). Não consertei: está fora do que o Enio pediu, e
  é um item de fila, não um bug do Blend.
- **A lasca das pontinhas de quina** (0,07–0,30% da área da fonte) é descartada pelo filtro do Build.
  São invisíveis, mas são geometria **real**. O Enio aprovou; o piso é **um número**
  (`SLIVER_AREA_FRACTION`) se ele mudar de ideia.
- **`vec_history` é fila MORTA** (o undo global a subsumiu; ainda é populada e não lida).
- **`ADR-0115` está duplicado no `main`** (áudio espectral × composição de clips). **Não é desta
  linha** — o gate `architecture_adr_numbers_are_unique` pina a exceção e ela é **auto-limpante**.
- **O Blend é destrutivo** (o `Make` + `Expand` do Illustrator num passo só), e isso é deliberado
  (ADR-0108: booleana e afins são *edit-time*). A **sessão** é o que dá a sensação de vivo: Steps,
  Rotate, Reverse e o checkbox **re-rodam** sem desfazer.

---

## §6 — Ao fechar

1. Gate batched (§0) + **auditoria de ≥2 lentes** sobre o diff acumulado.
2. **Smoke** e reporte ao Enio (comando pronto, com o `cd`).
3. **Handoff de integração** (DIRETRIZ §1.5.9): mapeie a superfície de colisão com as linhas vivas
   (`git worktree list`), e **conte os números que somam entre linhas** — `PROJECT_SCHEMA` e
   `VECTOR_SECTIONS` já morderam nesta linha ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
4. **PARE.** Não integre, não pushe, não faça ship.
