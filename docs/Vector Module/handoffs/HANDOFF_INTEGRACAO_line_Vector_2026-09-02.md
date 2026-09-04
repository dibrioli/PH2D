# Handoff de integração — `line/Vector`, 2026-09-02

> **51 commits · 224 ficheiros · +25 259 / −1 727** contra a `merge-base 066b4f92e`.
>
> ⚠️⚠️ **ESTE HANDOFF COBRE OS COMMITS 27–51.** Os 1–26 (estampa, pincel de contorno, grupos) já
> têm o seu, em [`…_line_Vector_2026-08-31.md`](HANDOFF_INTEGRACAO_line_Vector_2026-08-31.md) — e
> **ele nunca foi integrado**, então a linha entrega os dois de uma vez. Este acrescenta
> **101 ficheiros · +9 866 / −412** sobre o `74dfb6066`.
>
> ⚠️ **É um ROTEADOR.** O mecanismo de cada wave está na **mensagem de commit**, densa de propósito;
> o endereço é o hash. Aqui ficam: o que entrou, as leis que a linha pagou, o que está **aberto**
> (auditado contra o código) e o que uma leitura rápida do diff entende ao contrário.

## §1 — O que entrou (commits 27–51), por assunto

### 1.1 TRIM — a ferramenta de aparar (`38caa318e`, `b4fe6c6f3`, `439af190d`)

O motor **já existia dentro do `fx_knot`** e ninguém o alcançava. O gesto apara o pedaço sob o
cursor até à fronteira seguinte, e uma ponta que **termina** sobre outra curva passa a ser fronteira
dela. ⚠️ Ele escreve **no lugar** (`path_mut`): o id, a fatia de z e a entidade ECS sobrevivem ao
corte — é a lei que a solda veio a copiar.

### 1.2 SOLDAR (plano 39) — `3c98c26df`, `ec406ef1b`, `a79a121a5`, `6454ce9c3`, `4e144b8f6`, `20691d845`

Linhas cruzadas partem-se em **arcos que partilham o nó**. Quatro reports do Enio moldaram-na:

| report | o que mudou |
|---|---|
| *"weld dividiu e não soldou"* | as pontas fundem-se numa coordenada só; arrastar leva todos os arcos |
| *"as linhas não compartilham o mesmo nó"* | cruzar não é a única forma de se encontrar — pontas que se **tocam** ligam-se; e o nó partilhado **vê-se** (anel verde) |
| *"o smoke não tinha nada do que vc falou"* | a cena `=81` armava a ferramenta errada e o botão **não era pintado** |
| *"deveria criar apenas 1 [path]"* | a rede é **UM caminho composto**, escrito no lugar do participante mais ao fundo |
| *"com stroke muito largo, o stroke se quebra"* | **tampa REDONDA** na rede — cada arco é um sub-caminho, e a kurbo põe *tampa*, nunca *junta*, na ponta de cada um |

### 1.3 O BALDE (plano 40) — de `91cc20b83` a `20691d845`

A região que o clique aponta vira forma, **com a fronteira em arcos de verdade** (nunca uma
poligonal). Cinco reports com fotos, e o quinto trocou o **modelo**:

| report | o que mudou |
|---|---|
| *"para de funcionar nas áreas não coloridas"* · *"não acompanha o nó"* · *"acima do stroke"* | o preenchimento é vivo, sai da rede como parede, e nasce **por baixo** |
| *"nascendo deslocado para fora do stroke"* | a área re-cozida **desce ao espaço do caminho** |
| *"o preenchimento some"* (2×) | o **penhasco mudo** dos cruzamentos (acima do tecto a rede **RECUSA** em vez de responder zero), a fusão global de travessias, e o toque em «T» |
| *"os preenchimentos se quebram"* | herança por região, depois por **âncora** |
| *"várias inconsistências"* · *"ainda sem solução"* | ⭐ **o MODELO trocado** (`12dbef1f8`): a tinta agarra-se às **LINHAS**, não a uma coordenada |

⭐⭐⭐ **A lei que fica** (`vec_bucket_claim.rs`): *uma região é o **lado** de um conjunto de pedaços de
linha*. No clique gravam-se as **âncoras** da face — de que contorno de que caminho cada arco é um
pedaço, em que fracção, de que lado. **É stateless**: cada quadro resolve-se do documento sozinho, e
*o mesmo desenho dá sempre as mesmas cores*. Partir, fundir, crescer e «área nova» caem de graça.

### 1.4 EXPORT SVG (plano 41) — `cb659a656`

O app exportava **onze** coisas e **nenhuma levava uma curva**. `File > Export SVG…` escreve a
geometria **cozida, em mundo**, pela **mesma porta** que o renderer usa, com `data-ph2d-id` e
`data-ph2d-fill` — as marcas que deixam separar a LINHA da TINTA sem adivinhar pela cor. Foi o
instrumento que resolveu os três reports seguintes: os ficheiros do Enio são hoje **fixtura**
(`shells/desktop/tests/fixtures/drawing*.svg`, 5 ficheiros) e reproduzem a cena dele arco a arco.

## §2 — As leis que esta linha pagou (o que não se re-aprende)

1. ⛔⛔ **Uma recusa vale mais que uma resposta errada.** Acima do tecto de amostragem, devolver
   *"zero cruzamentos"* fazia **toda forma voltar a ser um anel inteiro**, em silêncio. Medido: 64
   círculos → a lente mede `2 235`; 65 → **`7 844`**. Hoje a rede sai `recusada` e o app **diz**.
2. ⛔ **Uma cura sem fixtura que a distinga é uma afirmação, não um resultado.** Duas mutações
   SOBREVIVERAM nesta linha por isso: a regra da semente (o voto já concordava com ela nos ficheiros
   reais) e a cobertura por fracção (o corte ao meio faz «cobre» e «meio mais próximo» empatarem).
3. ⛔ **Um gate textual afirma sobre a REDACÇÃO.** Dois falharam no mesmo dia: um nomeava **uma
   grafia** do defeito (`filter_map`, e a mutação usou `filter`+`map`); o outro usava uma **agulha
   que casa em dois sítios**. As duas leis saíram para funções que se podem chamar.
4. ⚠️ **A régua mente antes do código — quatro vezes.** A identidade de uma face pela **área** (duas
   metades empatam); um vencedor afirmado com margem de **1%** contra uma resolução de **7%**; a
   amostragem **cega a formas magras** (`0` amostras numa lasca de `1,3%` da caixa); e a semente
   posta na **fronteira** em vez do miolo, que partia o caso de identidade.
5. ⚠️ **A mesma decisão muda de sinal quando o que a justificava deixa de ser verdade.** *Congelar*
   quem perdeu a região era certo quando a forma **era** a receita; com âncoras deixou de ser — e
   ainda assim **esconder ficou pior** (§3).
6. ⛔ **O `insert_path(0, …)` não é o fundo** — quem manda no desenho é o `RootOrder` da entidade.
7. ⚠️ **Um `VecPath` é uma entidade ECS**: N arcos = N linhas na Hierarquia, N poses e N gizmos — e
   uma rede que se pode rasgar com o dedo.

## §3 — ⏳ ABERTO (auditado contra o CÓDIGO em 2026-09-02)

- ⏳⏳ **O report vivo: sete preenchimentos para SEIS faces.** Medido nas fixturas `drawing03`/
  `drawing04`: seis batem com a face deles a `0,0000` de área e o **sétimo tem o miolo fora de toda
  a face** — cor onde já não há região. ⛔ **A cura foi construída e REVERTIDA por ordem do Enio**
  (*"piorou o problema do preenchimento"*): esconder faz **desaparecer** área pintada, e uma
  ausência lê-se como trabalho perdido. **A terceira saída está nomeada e não construída**: nem
  apagar nem congelar — **avisar**, e o artista decide.
- ⏳ **A lâmina (`ph2d_vec_boolean::cut_open`) recusa um composto** — recusa pré-existente em que a
  saída da solda passou a cair. O verbo irmão que faz o trabalho é o **Trim**, já contour-indexed.
- ⏳ **A caneta só continua a partir do ARCO 0** de uma rede (o cabeçote é `verts.last()`).
- ⏳ **Marcadores e alças de largura vivem no arco 0** (`marker::end_tangent`, `width_handles`).
- ⏳ **As pontas SOLTAS de uma rede ficam redondas** — um `StrokeSpec` tem **uma** tampa para o
  caminho todo; dizer *"junta no nó, a do artista na ponta livre"* pede um segundo caminho.
- ⏳ **Importar SVG** continua a devolver documento vazio (`ph2d-imageio-svg` faz o parse com `usvg`
  e deita fora o resultado). A exportação não o cura.
- ⏳ **Cada área do balde é o seu próprio item na Hierarquia**, e o gizmo dela é **inerte** (o
  re-cozimento divide a pose fora). Decisão de produto.
- ⏳ Os dois do handoff de 31/08 continuam abertos: as **rotas que desenham sem arte de pincel** e a
  segunda selecção invisível do `vec_blend_picks`.

## §4 — O que uma leitura rápida do diff entende ao contrário

1. **`ph2d-vec-fill` é uma crate NOVA** (aparece no `Cargo.lock` como `+ph2d-vec-fill`) — a lei do
   balde, pura, sem cena e sem ponteiro. ⛔ Não é um rename de nada.
2. **`Rede::interior_point`/`interior_samples`/`adjacencias` foram APAGADAS**, não esquecidas:
   perderam o consumidor de produto quando a votação por área saiu, e *API pública sem consumidor
   mente sobre o que o produto faz*. O gate que media a re-semeadura passou a medir a lei que a
   substituiu, **sobre a mesma fixtura**.
3. **A solda escreve NO LUGAR** do participante mais ao fundo (`path_mut`) — o id sobrevive. Um
   `insert_path` daria um objecto novo com nome de fábrica.
4. **`VecBucketFill` deixou de ser `Copy`** (ganhou `Vec<FillAnchor>`), e isso é o que move o
   `PROJECT_SCHEMA`.
5. **`build_contours` deixou de ser `pub(crate)`** no `ph2d-vec-render` — é a porta única de que o
   exportador SVG precisa, para o ficheiro não discordar do ecrã.
6. **A cena `=82` do balde e a `=81` da solda mudaram de TEXTO** várias vezes: o texto é o que o
   Enio lê, e uma cena que ensina o contrário do que acontece é pior que uma cena ausente.
7. **O `toast` passou de `pub(super)` a `pub(crate)`** — o *Export SVG…* precisa da mesma porta de
   aviso.

## §5 — Superfície de colisão (saída de `collision-surface.sh`, 2026-09-02)

⚠️ **REFERÊNCIA, nunca evidência** (DIRETRIZ §1.5.9 item 3): mede esta linha contra o `main` **de
hoje**. O integrador **RE-RODA** o script em cada worktree imediatamente antes de fundir.

```
SUPERFÍCIE DE COLISÃO — line/Vector contra main
  merge-base 066b4f92e   ·   51 commit(s)   ·   224 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        108   (base: 103)
  ⚠   └ tripla do gate               (108, 13, 18)   (base: (103, 13, 17))
  ⚠ VEC_SCENE_SCHEMA                       18   (base: 17)
    FLIP_SCHEMA                            13   ·   DOC_VERSION (timeline)   18
▸ REGISTRO DE COMPONENTES
  ⚠ ph2d-render (espelho)                  80   (base: 79)
  ⚠ ph2d-script (espelho)                  80   (base: 79)
▸ CONTRATO CONGELADO (§6)                  intocado (nodegraph e tool)
▸ ADR                                      esta linha não cria ADR
▸ Cargo.lock                               1 pacote novo: "ph2d-vec-fill"
▸ MARCADORES DE CONFLITO                   nenhum
▸ TETOS DE LOC                             nenhum ficheiro da linha passa do teto
```

**O que isto quer dizer para o integrador:**

- ⚠️⚠️ **`PROJECT_SCHEMA` é `108` sobre uma base de `103` — cinco degraus.** Os números **contam-se**,
  nunca se escolhem, e a escada vive em **três** sítios (`project_schema.rs`, a escada de
  `project_schema_history.rs` e a **tripla** em `project_schema_tests.rs`). ⛔ Se outra linha subiu o
  seu, a fusão é **muda**: o git não sabe o que o número significa.
- ⚠️ **`VEC_SCENE_SCHEMA` 17 → 18** (o `StrokePaint::Brush`, do bloco 1–26).
- ⚠️ **O registo de componentes sobe 79 → 80** nos **dois** espelhos (`ph2d-render`, `ph2d-script`),
  pelo `ph2d::ecs::VecBucketFill`. O catálogo do `ph2d-component-desc` é **ORDENADO** — uma entrada
  no sítio errado acusa dois componentes inocentes.
- **Ids novos:** `ids::VECTOR_PATH_WELD`, `VECTOR_TRIM_*` (`chrome/vector_cut.rs`),
  `ids::CTX_MENU_EXPORT_SVG` (`ids/menus.rs`), `DrawMode::{Trim, Bucket}` (`params_mode.rs`).
- **Contrato congelado: NENHUM tocado** ⇒ não exige ADR (§4 da DIRETRIZ).

## §6 — A linha para o `CLAUDE.md` §5 (Vector)

Já aplicada nesta worktree, em **uma linha** (DIRETRIZ §1.5.9 item 8). O parágrafo de jornada é
**este** documento.

## §7 — O que só o `ship.sh` apanha (o gate de integração NÃO roda)

- **`typos`** e **`cargo machete`**: a linha não acrescentou dependência externa nenhuma
  (`ph2d-vec-fill` é interna), mas nenhum dos dois correu aqui.
- **`cargo deny` / `RUSTSEC`**: idem.
- **`fmt` pré-fork** e **clippy latente** em crates que a linha não tocou.
- **A matriz 3-OS e o `physics_ecs_c9`**: o risco real é os três OS **discordarem**, e só o CI o mede.

## §8 — Ordem, dependências e o que SMOKAR

**Ordem:** os 51 commits são lineares e cada um compila; ⛔ não há reordenação segura. As waves
dependem umas das outras nesta ordem: `arclen` → quinas → estampa/pincel → grupos → Trim → Soldar →
Balde → Export SVG. O **modelo do balde muda em `12dbef1f8`** — um `cherry-pick` parcial que pare
antes dele traz a heurística que os quatro reports condenaram.

**Smokado pelo Enio (com foto, e validado por ele):** `=80` (Trim) · `=81` (Soldar) · `=82` (o
Balde, cinco rondas) · `File > Export SVG…` (produziu as cinco fixturas).

**⛔ NÃO smokado:**

- a **tampa redonda** da rede soldada (`20691d845`) — entrou depois do último report dele;
- o **`Export SVG…`** sobre um documento com **gradiente**, **padrão** ou **pincel de contorno** (o
  cabeçalho nomeia o que aproximou, mas ninguém o abriu num editor externo);
- o **carregar um `.ph2dproj` v103–107** contra o v108 — a recusa por versão é a esperada, e o
  Enio decidiu em 26/08 que **não há degrau de migração** por não haver projetos gravados.

## §9 — Estado de fecho

| passo | resultado |
|---|---|
| gate batched (`nextest-impacted.sh`, `BASE=main`) | ✅ **13 256 testes, 13 256 passaram**, 1 311 saltados |
| clippy `--all-targets -D warnings` nas **18** crates do diff | ✅ limpo |
| `cargo fmt --all --check` | ✅ limpo |
| `doc-index.sh --check` | ✅ 14 índices em dia |
| `collision-surface.sh` | §5 |
| `incremental/` reclamado | ✅ **22 GB** (`target/debug/incremental`) |
| binário de smoke compilado | ✅ 2.ª corrida `Finished` em **0,18 s**, **zero** `Compiling` |

```
$ cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
$ cargo build -p ph2d-host-desktop --release
    Finished `release` profile [optimized] target(s) in 0.36s
$ cargo build -p ph2d-host-desktop --release
    Finished `release` profile [optimized] target(s) in 0.18s
```

⚠️ **A linha PARA aqui.** Não integra e não pusha ([`CLAUDE.md §0.7`](../../../CLAUDE.md)): a
integração e o ship só acontecem por ordem explícita do Enio, por um agente integrador dedicado.
