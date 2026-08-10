# Handoff de integração — `line/sculpt3d`, **MESTRE** (W8.3 → W9.1)

> **Data:** 2026-08-04 · **Branch:** `line/sculpt3d` · **Base:** `main` em `dc0587cbe`
> **Commits:** **28** (`0ae9950f3` … `e397540f1`) · 88 arquivos, +12.886 / −1.210
> ✅ **TODOS OS SMOKES APROVADOS pelo Enio** — as cinco waves W8.3..W8.7 num smoke só
> (2026-08-04), depois a W8.8, depois a W9.1 em **duas rodadas** (a 1ª reprovou o aspecto).
>
> ⚠️ **Este documento SUPERSEDE o `..._W8.7_2026-08-04.md`** para efeito de integração: aquele
> descreve **uma** wave e ficou como detalhe. O que está no `main` hoje (W1..W8.2) é o
> `..._W4-W8_2026-08-02.md`.

---

## 1. O que a linha entrega, numa frase por wave

| wave | entrega | smoke |
|---|---|---|
| **W8.3** | **A escultura sobrevive a fechar o app** — blob `sculpt` no `ProjectFile`, versionado por dentro | `=8` |
| **W8.4** | A **porta de entrada**: import STL/PLY/OBJ pelo seletor de arquivo | `=9` |
| **W8.5** | A **porta de saída**: export, com o *round-trip* como oráculo | `=10` |
| **W8.6** | O **OBJETO MISTO** — a forma acende um SPRITE da cena | `=11` |
| **W8.7** | Os **canais assados viajam no documento** (rota A: a malha some do build, o objeto continua reluminável) | `=12` |
| **W8.8** | **FUNDIR** e **ISOLAR**, os dois verbos que faltavam à lista de peças | `=13` |
| **W9.1** | **A TOPOLOGIA DINÂMICA** — o traço adensa onde o pincel toca | `=14` |

Detalhe por wave: `docs/3D/handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_W8.7_2026-08-04.md` (a W8.7 no corpo, e
as §8.bis / §8.ter / §9.bis para W8.8, W9.1 e a 2ª rodada dela).

---

## 2. A tabela de colisão — **medida hoje, contra o `main` em `dc0587cbe`**

| item | `main` | linha | nota |
|---|---|---|---|
| `PROJECT_SCHEMA` | **50** | **52** | ⚠️ **DOIS degraus, e os dois PROVISÓRIOS** — v51 (W8.3, `sculpt`) · v52 (W8.7, `baked_forms`) |
| `FLIP_SCHEMA` / `VEC_SCENE` | 13 / 14 | **intocados** | |
| registro `ph2d-ecs` | 46 | **47** | `ph2d::ecs::BakedForm` |
| espelho `ph2d-render` | 47 | **48** | ⚠️ **o contador é TRÊS** |
| espelho `ph2d-script` | 47 | **48** | idem |
| contrato congelado | — | **intocado** | `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` — gate **4/4**, rodado |
| ADR | — | **nenhum novo** | tudo sob o **ADR-0150** |
| crates novas | — | **nenhuma** | |
| deps EXTERNAS novas | — | **nenhuma** | |
| cenas de smoke | `1..7` | **`1..14`** | |

⚠️ **`PROJECT_SCHEMA` se CONTA contra o `main` do dia, não se escolhe.** Se outra linha bumpar na
mesma janela, o valor certo não está em nenhum dos dois lados
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). E confira o
**`project_schema_tests.rs`**: em 2026-08-01 o `project.rs` **não conflitou** porque duas linhas
escreveram o mesmo literal e o git não tem opinião sobre o que o número significa — quem denunciou
foi a tripla ao lado.

**`Cargo.toml` tocados: DOIS**, e nenhum traz dep de fora do workspace:

- `crates/ph2d-mesh/Cargo.toml` — `serde` (a crate **serializa a si mesma**: `levels`/`details` são
  privados de propósito, e um serializador fora dela seria uma segunda casa que sabe do que uma
  pilha é feita) + `postcard` em **`[dev-dependencies]`**, só para o gate que pina a forma do
  arquivo ⇒ **machete-safe**, o padrão das crates-nó da `ph2d-gpu-cook`.
- `crates/ph2d-light/Cargo.toml` — `serde`, no **DONO** do rig e não no shell (o doc do `LightRig`
  sempre disse *"o rig inteiro, como o documento o guarda"*).

`Cargo.lock`: **+3 linhas**, só arestas internas de path.

---

## 3. ⚠️ O PONTO DE MERGE SENSÍVEL: `project.rs` foi PARTIDO

`shells/desktop/src/project.rs` perdeu **165 linhas** e ganhou 62. O que saiu foi o **load inteiro**
(`project_load` + `project_load_from`), para o irmão novo **`shells/desktop/src/project_load.rs`** —
um split de LOC, feito porque os dois campos novos não cabiam.

**Isto é o item de maior risco da integração**, e não é o schema: `project.rs` é dos arquivos que
mais linhas tocam, e uma linha que edite o corpo do `project_load_from` funde **limpo** contra um
arquivo de onde a função saiu — o resultado é uma árvore que funde e **não compila**, ou pior, que
compila com a edição alheia num corpo morto ([[feedback_clean_text_merge_can_be_semantically_broken]]).

**Se houver conflito ali, resolva pelos ESTÁGIOS do índice**, não pelos marcadores
([[feedback_resolve_conflicts_from_index_stages_not_markers]]), e confira depois que
`project_load_from` existe **uma vez só** em toda a shell.

Outros arquivos partidos por LOC nesta linha (todos por assunto, todos com re-export ou `#[path]`,
nenhum muda caminho de chamada): `stroke.rs` → `stroke_growth.rs` · `dyntopo.rs` → `dyntopo_flip.rs`
· `input_dispatch/keyboard.rs` → `keyboard_files.rs`.

---

## 4. Onde o código mora

**25 arquivos novos na shell**, todos com prefixo `sculpt3d_*`/`baked_form*`/`project_*` — a
promessa de removibilidade do `docs/3D/02.3` continua verificável: as quatro crates do módulo caem
com a feature `sculpt3d`, e desligá-la não toca em nada do 2D.

⚠️ **As três exceções, que ficam FORA do `cfg`, e cada uma tem o seu motivo escrito:**

1. **`ProjectFile.sculpt`** é `Vec<u8>` opaco **sem `cfg`** — postcard é posicional, e um campo
   condicional daria DUAS formas de arquivo com o mesmo número de schema. Um binário sem escultura
   **carrega os bytes adiante** em vez de os triturar.
2. **`ProjectFile.baked_forms`** é campo de **SPRITE** e não parte do blob acima, embora aquele já
   guarde as malhas: o parser da escultura é `#[cfg(feature = "sculpt3d")]`, e guardar os canais lá
   os tornaria legíveis só com o módulo 3D no build — o oposto exato do que a **rota A** promete.
3. **`ph2d-light`** deixou de estar vazia e virou o dono do rig (já registrado na integração de
   2026-08-01): ela é **não-removível**, a única exceção, e isso já está no `02.3`.

---

## 5. O que a 2ª rodada da W9.1 ensinou, e que vale para além dela

O smoke reprovou o aspecto (*"funciona mas o aspecto da escultura depois o P fica horrível"*) e a
causa **não era a que eu consertei primeiro**.

⚠️ **NENHUMA MÉTRICA DE POSIÇÃO VIA O DEFEITO.** Pelo desvio de vértice (distância à média dos
vizinhos, por aresta local) o produto media **0,7131** e a correção da herança do `pre` — que está
certa — deixava **0,7158**. Igual. Só medindo **ÂNGULO DE TRIÂNGULO** ele apareceu: **21,21° no
controle contra 1,53°, com 15% da malha abaixo de 10°**. *Uma lasca não desloca vértice nenhum* —
ela dá uma normal por-vértice que não aponta para lado nenhum, e a **luz** desenha isso como agulha.

Três peças, todas **ablacionadas** (nenhuma shipou sem o próprio número):

| | pior ângulo mínimo | abaixo de 10° |
|---|---|---|
| escolha por FACE (era por ARESTA) | 0,59° | 48,0% |
| + fecho de aresta mais longa (LEPP de Rivara) | 2,43° | 1,5% |
| + **flip de aresta** (`dyntopo_flip.rs`) | **16,85°** | **0,0%** |

⛔ **MEDIDO E REJEITADO, não refaça:** promover a vizinha a 1→4 também dá qualidade perfeita
(20,47°) e **cascateia pela malha inteira** — 57 → **846** vértices no hemisfério que o artista
nunca tocou. *Qualidade é global; a pegada não pode ser.*

⚠️ **O alcance do refino passou a ultrapassar o pincel**, e o gate diz o número medido em vez de uma
barra escolhida: **1,66× · 1,31× · 1,38× · 0,93×** o raio em quatro densidades de esfera — ele
**encolhe** quando a malha já é fina.

E a segunda metade: um vértice nascido no meio do traço **herda o `pre` dos pais** (`Birth`,
parâmetro obrigatório de `refine_in_sphere`). ⚠️ **A minha fixture não continha o fenômeno duas
vezes** — ela não triangulava (o produto triangula ao armar, então o refino devolvia `NotTriangles`
e a sonda media duas cenas **idênticas**) e usava `Draw`, que estica as arestas em poucos por cento.
São **quatro gestos**, cada um vendo um erro que os outros não veem: **puxar** (a agulha, 0,053 ×
0,720) · **varrer** (a cratera do `accum` herdado em zero, 0,108 × 0,446) · **afinar o detalhe com
`U`** · **sob máscara** (0,108 × **0,867**).

⚠️ E a primeira barra que escrevi foi **0,45** com a cratera medindo **0,446**: teria passado por um
triz. *Uma barra escolhida antes de ver os dois lados da mutação é um palpite com casas decimais.*

**E a `=14` abria em CANVAS BRANCO** por um motivo que nenhum gate via: `smoke_armed()` era uma
**enumeração** (`"1" | … | "13"`) e a cena nova não estava nela — cada peça da cena existia e estava
certa, e o módulo simplesmente **não armava**. Virou um **parse**; dois gates novos, e a mutação
(a enumeração de volta) sangra.

---

## 6. O gate de fechamento — rodado, não auto-relatado

| verificação | resultado |
|---|---|
| `cargo fmt --all --check` | ✅ |
| `cargo clippy --workspace --all-targets` | ✅ **0 warnings** |
| `cargo machete` | ✅ nenhuma dep sem uso |
| `cargo deny check` | ✅ advisories · bans · licenses · sources |
| `typos` | ✅ |
| contrato congelado | ✅ **4/4** e **3/3** |
| suíte `--release` | ✅ workspace |
| suíte **DEBUG** das crates do módulo | ✅ **em série** — *a linha tem precedente registrado: o `ph2d-flip-colorize` panicava só em debug* |

⚠️ **FLAKE DE CARGA CONHECIDA, e ela NÃO é desta linha.** Rodando as cinco crates do módulo em
paralelo em **debug**, dois gates de `crates/ph2d-mesh/tests/measure_normals.rs` reprovam —
`measure_normals_parallel_speedup` e `the_region_refresh_is_bound_by_the_footprint_not_by_the_mesh`.
Os dois são gates de **RAZÃO** com relógio, e **passam isolados** (verificado: 3/3). A linha **não
toca aquele arquivo** (conferido por `git diff --name-only`), e é a mesma família do
`the_cost_of_depth_is_linear_not_explosive` da `ph2d-timeline`: *re-rode sozinho antes de suspeitar
de um merge*.

⚠️ **Gates de GPU: `#[ignore]`, 25 no `ph2d-mesh-render`.** Sem adapter eles fazem *skip gracioso*,
**que não é verde** — rode-os na RTX:

```
cargo test -p ph2d-mesh-render --release -- --ignored
```

---

## 7. Os smokes, e o que julgar em cada um

```
env PH2D_SCULPT3D_SMOKE=<n> cargo run -p ph2d-host-desktop --release
```

| `n` | wave | o que julgar |
|---|---|---|
| `8` | W8.3 | esculpa, **feche e reabra** — a pilha de níveis volta |
| `9` | W8.4 | import STL/PLY/OBJ pelo seletor |
| `10` | W8.5 | export, e o arquivo re-importa igual |
| `11` | W8.6 | a forma acende um **sprite da cena** |
| `12` | W8.7 | reabra: as sombras **ANDAM**; e o objeto acende **sem cena 3D nenhuma** |
| `13` | W8.8 | **FUNDIR** e **ISOLAR** |
| `14` | W9.1 | **o `P`** — o passo 1 é o CONTROLE (facetado *antes* de armar); o passo 3 é o que reprovou na 1ª rodada (esfregar **por cima do mesmo lugar** várias vezes) |

⚠️ **Três cenas imprimem o número que as torna válidas** (quantas arestas de beira · quanto mede a
maior aresta · quantas peças abriu). **Se a linha não aparecer, pare** — o resto do smoke não diz
nada. E **rode uma vez SEM a env var**: é a metade que prova a inércia (o frame 2D byte-idêntico).

---

## 8. Aberto, e NOMEADO — nada disto bloqueia a integração

- **W9.2 — a estrutura mutável.** Cada passe do refino termina num `Mesh::rebuild` inteiro (anéis,
  octree, normais). Medido: **0,59 ms @6k · 1,55 @24k · 5,50 @98k**, e o mesmo dab toca **0,33% das
  faces** ⇒ o rebuild faz ~300× o trabalho que a mudança pede. Ele **cabe hoje** e deixa de caber
  perto de ~100k vértices — que é exatamente o que a topologia dinâmica existe para fazer. **O
  gatilho é um número medido, não um palpite.**
- **W9.3 — o COLLAPSE.** A metade que falta do quarteto de remalhamento incremental (*split ·
  collapse · flip · smooth*): hoje o refino só ACRESCENTA, então um traço que desfaz um volume
  deixa a densidade que ele criou. Histerese prescrita: `alvo/2,05`.
- **O documento não guarda a topologia dinâmica de forma incremental** — ele guarda a malha
  resultante, que é o certo, mas um arquivo grande cresce com o detalhe.
- Herdado das waves anteriores: import/export não carrega **cor, material nem a MÁSCARA** · o
  **marching cubes** (o remesh usa Surface Nets, escolhido de propósito) · o remesh **RECUSA** com a
  pilha de multires montada, e a recusa é **nomeada no log** (a alternativa seria achatá-la em
  silêncio) · a resolução do remesh não é autorável (o botão usa o default 150).
- ⚠️ **Uma mutação NÃO sangra, de propósito e documentada:** a herança do `base_nrm` é COERÊNCIA e
  não correção — o único verbo que lê a normal congelada é o `Inflate`, e um vértice que nasce
  **sobre** a superfície tem normal viva praticamente igual à média das congeladas dos pais. Ela
  fica porque um registro em que quatro canais vêm dos pais e um vem da malha viva é o tipo de
  assimetria que a próxima pessoa "corrige" para o lado errado.

---

## 9. Ordem de integração

Toca `ph2d-ecs` (registro), `ph2d-light` (`serde`), `ph2d-mesh` (`serde` + `postcard` de dev),
`ph2d-mesh-render`, `ph2d-render`/`ph2d-script` (**só o número**) e a shell.

⚠️ **Colisões prováveis, por risco:**

1. **`project.rs` partido** (§3) — o maior risco, e o único que funde limpo estando errado.
2. **`PROJECT_SCHEMA` 52** — provisório; **conte** contra o `main` do dia, e olhe o
   `project_schema_tests.rs` ao lado.
3. **O contador do registro é TRÊS**, e cada um só roda na suíte da própria crate — a família que já
   ficou vermelho-latente **três vezes** neste repo.
4. **`input_dispatch/keyboard.rs`** ganhou `+4` linhas dentro do `#[cfg(feature = "sculpt3d")]` que
   já existia, e cedeu o bloco de arquivos para `keyboard_files.rs` — o mesmo arquivo que a
   `line/anim` e a `line/physics` já fizeram cruzar o teto de LOC por soma.
