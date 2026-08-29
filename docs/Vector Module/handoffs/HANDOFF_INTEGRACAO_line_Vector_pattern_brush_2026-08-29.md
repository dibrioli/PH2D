# Handoff de integração — `line/Vector` (Texture Pattern + Padrão no traço + **Pincel de contorno**)

> **2026-08-29** · DIRETRIZ §1.5.9 · ⛔ **A linha NÃO integrou e NÃO pushou.** Fecha aqui e espera
> ordem explícita do Enio ([`CLAUDE.md §0.7`](../../../CLAUDE.md)).
>
> ⚠️⚠️ **ESTE SUPERSEDE o [handoff de 27/08](HANDOFF_INTEGRACAO_line_Vector_texture_pattern_2026-08-27.md)**,
> escrito quando a linha tinha **20** commits e só o plano 33. Aquele continua a ser a melhor
> leitura do *mecanismo* do Texture Pattern — ⛔ **mas todo número de integração que ele cita está
> morto**: ele diz `PROJECT_SCHEMA 99→100` (hoje são **três** degraus, e o `100` foi tomado pelo
> `main`), `20 commits` e `85 ficheiros`. *Um handoff descreve o mundo do dia em que foi escrito, e
> não reclama quando envelhece — é por isso que este re-mediu tudo contra o `main` de HOJE.*

---

## 1 — Identidade

| | |
|---|---|
| Branch | `line/Vector` |
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| HEAD | `d256ac82e` |
| Merge-base com `main` | `330582deb` |
| Commits | **37** · **157 ficheiros** · +15 386 / −534 |
| `main` no dia do fecho | **`f41a257e4`** (≈120 commits à frente da merge-base) |

⚠️⚠️ **O `main` ANDOU MUITO desde o fork** — a `line/components`, a `line/3DModeling` e a
`line/motion-value` foram integradas no meio. Isto é exactamente o *prazo de validade* que a
§1.5.9 avisa, e **mudou a resposta**: ver §3.

---

## 2 — ⭐⭐⭐ O CONFLITO REAL, MEDIDO CONTRA O `main` DE HOJE (não contra a merge-base)

```
git merge-tree --write-tree --name-only main HEAD
```

⇒ **DOIS ficheiros conflituam, e são os dois do schema:**

```
shells/desktop/src/project_schema.rs
shells/desktop/src/project_schema_tests.rs
```

**Tudo o resto funde sozinho** — incluindo `render_loop/mod.rs`, `input_dispatch.rs`,
`app_state.rs`, `main.rs`, `Cargo.lock`, `shells/desktop/Cargo.toml` e
`crates/ph2d-editor-core/tests/hr12_widgets_a11y.rs` (todos `Auto-merging`, sem `CONFLICT`).

⚠️ **Fundir sozinho não é fundir certo** — o allowlist do `hr12_widgets_a11y` e o `render_loop`
são listas cujo texto funde e cuja semântica pode não. Quem responde por isso é o
`scripts/foundational-integrate.sh` sobre a árvore combinada, não este documento.

---

## 3 — ⛔⛔ A COLISÃO DE NÚMERO: `PROJECT_SCHEMA` — as duas linhas escreveram o literal `100`

> *«Número que soma entre linhas se CONTA, nunca se escolhe — e a colisão passa MUDA quando duas
> linhas escrevem o MESMO literal»* (`CLAUDE.md` §5.0). Aqui ela **não** passou muda, porque o git
> viu texto diferente à volta; mas o número está errado dos dois lados se alguém escolher em vez
> de contar.

| | merge-base (`330582deb`) | `main` HOJE (`f41a257e4`) | `line/Vector` | ⇒ **depois de fundir** |
|---|---|---|---|---|
| `PROJECT_SCHEMA` | 99 | **100** (F5.3, órfãos do `ObjectInstance`) | 102 | **103** |
| tripla do gate | (99, 13, 14) | **(100, 13, 14)** | (102, 13, 17) | **(103, 13, 17)** |
| `VEC_SCENE_SCHEMA_VERSION` | 14 | 14 (intocado) | **17** | **17** |
| `FLIP_SCHEMA` | 13 | 13 | 13 | 13 |
| `DOC_VERSION` (timeline) | 18 | 18 | 18 | 18 |

⭐ **A `line/Vector` traz TRÊS degraus** (não um), e os três têm de deslizar um lugar:

| degrau desta linha | o que é | passa a ser |
|---|---|---|
| `99 → 100` + `VEC_SCENE 14 → 15` | **Texture Pattern** (plano 33 W3): o `Paint` ganhou a 5.ª variante `Pattern`. Aditivo. | `100 → 101` |
| `100 → 101` + `VEC_SCENE 15 → 16` | **Padrão no traço** (plano 35 wave A): o `StrokeSpec` trocou `color: Rgba8` por `paint: StrokePaint`. ⛔ **DESTRUTIVO nos dois sentidos** — um campo mudou de TIPO no meio da estrutura, e o postcard é posicional. | `101 → 102` |
| `101 → 102` + `VEC_SCENE 16 → 17` | **Pincel de contorno** (plano 36 W1): o `StrokePaint` ganhou `Brush(Box<BrushStroke>)`. Variante APENDADA, do lado aditivo. | `102 → 103` |

**Resolução (os TRÊS sítios, `CLAUDE.md` §5.0):**

1. `shells/desktop/src/project_schema.rs` — manter o degrau `100` do `main` (os órfãos) **e**
   renumerar os três blocos `/// # 100/101/102` desta linha para `101/102/103`;
   `pub(crate) const PROJECT_SCHEMA: u32 = 103;`
2. `shells/desktop/src/project_schema_tests.rs` — manter o comentário e a tupla `(100, 13, 14)` do
   `main`, e acrescentar os três desta linha renumerados, terminando em **`(103, 13, 17)`**.
3. ⚠️ **`VEC_SCENE_SCHEMA_VERSION` NÃO se mexe** — o `main` não lhe tocou, e os passos `15/16/17`
   ficam onde estão. *Renumerar por simpatia é o erro simétrico ao de não renumerar.*

⛔ **Nenhum dos três degraus desta linha tem migração**, e está certo: é a decisão do Enio de 26/08
(*"não há projetos gravados"*) — o número sobe para o load **recusar em voz alta** em vez de ler
lixo bem-formado em silêncio. É a mesma escolha que o `main` fez no degrau dele.

---

## 4 — Foundational / partilhado tocado, e porquê

### 4.1 — Crates NOVAS (duas folhas, ambas ZERO-dep de domínio)

| Crate | Porquê | Colisão |
|---|---|---|
| **`crates/ph2d-vec-pattern/`** | A casa única do vocabulário de ladrilho (`TileKind`/`TileLaw`/`PatternMode` + o assador). Folha porque tem **dois donos que não se veem**: a `ph2d-vec-scene` (pura, sem vello/kurbo) e a `ph2d-vec-render` (que alcança a stack Linebender). | nome livre no `main` |
| **`crates/ph2d-asset-id/`** | O `AssetId` **saiu** da `ph2d-asset`: o `Paint::Pattern` guarda *qual* imagem, e a `ph2d-vec-scene` é pura — a `ph2d-asset` puxava descodificadores de imagem para dentro do modelo de documento. ⭐ **É invisível:** `ph2d-asset/src/id.rs` passou a ser um `pub use`, e **nenhum dos 78 ficheiros** que usam `ph2d_asset::AssetId` muda uma linha. | nome livre no `main` |

⚠️ As duas declaram `license.workspace = true` — o ✗ que só o ship vê (o commit `330582deb` do
`main` é literalmente sobre isso) **não se repete aqui**; `cargo deny` corrido e verde (§6).
A membership é por **glob** (`crates/*`), então o `Cargo.toml` da raiz **não** foi tocado.

### 4.2 — Foundational editado

| Ficheiro | O que | Aditivo? |
|---|---|---|
| `ph2d-editor-core/src/ids/chrome/vector.rs` | +39 linhas: os chips de tinta do traço e a caixa *Stroke* | **sim** |
| `ph2d-editor-core/src/ids/chrome/vector_sections.rs` | +6: `VECTOR_SECTION_TEXPAT`, `…_TEXPAT_STROKE`, `VECTOR_SECTION_BRUSH` | **sim** |
| `ph2d-editor-core/src/ids/chrome/vector_texture_pattern.rs` | **ficheiro novo** (+163): os ids do padrão, **derivados por slot** | **sim** |
| `ph2d-editor-core/src/ids/chrome/mod.rs` | +2 (`mod` + `pub use`) | **sim** |
| `ph2d-editor-core/tests/hr12_widgets_a11y.rs` | +4: uma entrada de allowlist para `paint_brush.rs`, com a justificação | **sim** |
| `ph2d-i18n/src/vector.rs` | 6 chaves novas (lista em §5) | **sim** |
| `ph2d-ui-state/src/transition.rs` | mecânico: o `StrokeSpec` deixou de ser `Copy` ⇒ `.clone()` / `clone_from` | não-aditivo, **trivial** |
| `ph2d-tool-vector/src/tool_adopt.rs` | mecânico: `s.color` -> `s.color()` | idem |
| `ph2d-vector/src/scene.rs` + `lib.rs` | `fill_path_image` (o ladrilho nativo do Vello) + re-export de `Extend` | **sim** |
| `ph2d-vec-boolean` (9 fich.) · `ph2d-vec-edit` (2) · `ph2d-vec-blend` (2) | mecânico: `StrokeSpec` sem `Copy` e `color` -> `color()` | **trivial** |
| `shells/desktop/src/` (48 fich.) | 17 ficheiros novos (pattern + brush + smokes) + o dreno no `render_loop` | ver §5 |

⭐ **A mudança de tipo mais invasiva é `StrokeSpec::color: Rgba8` -> `paint: StrokePaint`**, e ela
falha **ALTO** (erro de compilação) em todo leitor que não passe por `color()`. **Medido contra o
`main` de hoje: ele acrescentou UM único uso de `Paint::*` desde o fork**
(`input_dispatch.rs:646`, uma construção `Paint::Solid`, não um `match` exaustivo) ⇒ **o risco de
compilação da fusão é praticamente nulo**.

---

## 5 — Símbolos que podem COLIDIR (o que grepar)

### 5.1 — `bash scripts/collision-surface.sh`, colado verbatim

⚠️ **Esta tabela mede contra a MERGE-BASE (`330582deb`), não contra o `main` de hoje** — é
referência, nunca evidência. A coluna «base» já envelheceu: ver §3, onde a mesma pergunta feita ao
`main` actual dá **outra** resposta para o `PROJECT_SCHEMA`.

```
SUPERFÍCIE DE COLISÃO — line/Vector contra main
  merge-base 330582deb   ·   37 commit(s)   ·   157 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        102   (base: 99)
  ⚠   └ tripla do gate               (102, 13, 17)   (base: (99, 13, 14))
  ⚠ VEC_SCENE_SCHEMA                       17   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  78   (base: 78)
    ph2d-script (espelho)                  78   (base: 78)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0167   próximo livre: 0168
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 2 pacote(s) '+name' novo(s):
      "ph2d-asset-id"
      "ph2d-vec-pattern"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
     1785 / 600   shells/desktop/src/app_state.rs  (tem marcador/allowlist — confira o valor congelado)
     6641 / 600   shells/desktop/src/input_dispatch.rs  (tem marcador/allowlist — confira o valor congelado)
     1379 / 600   shells/desktop/src/main.rs  (tem marcador/allowlist — confira o valor congelado)
    11601 / 600   shells/desktop/src/render_loop/mod.rs  (tem marcador/allowlist — confira o valor congelado)
    nenhum arquivo da linha passa do teto
```

⚠️ Os «2 pacotes novos» são as **duas crates internas** desta linha (§4.1) — **zero dependências
externas novas**.

### 5.2 — Ids de chrome novos (todos por **hash de string**, não literais numéricos)

```
VECTOR_FILL_KIND_PATTERN          "vector.fill_kind.pattern"
VECTOR_STROKE_PRESENT             "vector.stroke.present"
VECTOR_STROKE_KIND_SOLID          "vector.stroke.kind.solid"
VECTOR_STROKE_KIND_PATTERN        "vector.stroke.kind.pattern"
VECTOR_STROKE_KIND_BRUSH          "vector.stroke.kind.brush"
VECTOR_SECTION_TEXPAT             "vector.section.texpat"
VECTOR_SECTION_TEXPAT_STROKE      "vector.section.texpat.stroke"
VECTOR_SECTION_BRUSH              "vector.section.brush"
VECTOR_BRUSH_PICK_SHAPE           "vector.brush.pick"
VECTOR_BRUSH_{SPACING,SCALE,OFFSET,ROTATION}[_NUM]   "vector.brush.…"
VECTOR_BRUSH_FLIP                 "vector.brush.flip"
```

⭐ **Colidir exige duas linhas escolherem a MESMA STRING** — improvável, e um choque de hash é
apanhado pelo gate de unicidade de ids. ⚠️ Os ids do padrão são **derivados**
(`texpat_id(slot, knob)` sobre `TEXPAT_SLOTS = 2` e `TexPatKnob::ALL` com **24** variantes), não
constantes soltas: quem acrescentar um knob acrescenta uma variante, não um id.

### 5.3 — Números CONTADOS que outra linha pode mover

| O quê | Valor | Onde | Nota |
|---|---|---|---|
| Secções do painel vetorial | **40** | `crates/ph2d-panel-vector/tests/seam.rs:654` | ⚠️ **CONTADO** (o gate imprime `left: 40`), nunca escolhido — subiu `38 -> 39 -> 40` nesta linha. Se outra linha acrescentar uma secção, **re-conte**, não some. |
| `TEXPAT_SLOTS` | 2 | `ids/chrome/vector_texture_pattern.rs` | preenchimento + traço |
| Cena de smoke `PH2D_BUILD_SMOKE` | **76** e **77** | `shells/desktop/src/build_smoke_router.rs` | ⚠️ o número da próxima cena **CONTA-SE lendo o roteador**; `77` era o primeiro livre no dia do fecho |

### 5.4 — Chaves de i18n novas

```
panel.vector.section.texpat · panel.vector.section.texpat_stroke · panel.vector.section.brush
panel.vector.brush.flip · panel.vector.stroke.present · panel.vector.texpat.lock
```

---

## 6 — Contratos congelados encostados: **NENHUM**

A sonda confirma `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` **intocados**.
⇒ nada exige ADR, e esta linha **não cria ADR nenhum** (fora de toda disputa pelo `0168`).

⚠️ O contrato congelado do **modelo vetorial ANTIGO** (`ph2d-vector-doc` + `-traits`, gate
`architecture_vector_contract_surface`) também está intocado — esta linha vive toda no motor
**novo** (`ph2d-vec-*`), cujo contrato ainda não foi congelado (`CLAUDE.md` §6, follow-up).

---

## 7 — O que só o `ship.sh` pega — **corrido aqui, e verde**

| Check | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✓ |
| `cargo clippy` (vec-scene · vec-render · host-desktop, `--all-targets`) | ✓ zero avisos |
| `cargo machete` | ✓ nenhuma dep por usar |
| `cargo deny --all-features check` | ✓ *advisories ok, bans ok, licenses ok, sources ok* |
| `typos` | ✓ nada |
| `bash scripts/doc-index.sh --check` | ✓ 14 índices em dia |
| `scripts/nextest-impacted.sh --no-fail-fast` | ✓ **12 646 / 12 646**, zero vermelhos |

⚠️ **O que fica por medir, e é do ship:** clippy `--workspace --all-targets` com **todas as
features** (aqui só as três crates do diff + as suas dependentes), e o `nextest` no perfil
`ci-test` sobre a **árvore combinada** — que é precisamente o que o
`scripts/foundational-integrate.sh` faz. ⚠️ A varredura acima correu contra o `main` da
merge-base, **não** contra a árvore fundida.

⚠️ **RUSTSEC:** zero dependências externas novas ⇒ a deriva de advisory desta linha é a do
calendário, não a do diff.

---

## 8 — Ordem, dependências e o que smokar

### 8.1 — Ordem dos commits: **estritamente sequencial, não reordenável**

Três planos encadeados, cada um a construir sobre o anterior:

1. **Plano 33 — Texture Pattern** (`437eb70a1` … `fdf8c6c50`): o assador, a porta de render, o
   dado, a UI, as alças de canvas, o save, a cena `=76`.
2. **Plano 35 — o padrão no TRAÇO** (`90c0a80e7` … `9f0d80bc5`): a tinta do traço vira enum
   (`StrokePaint`), o traço desenha com o ladrilho, e as **duas secções** *Pattern* (wave F).
3. **Plano 36 — o PINCEL** (`a853c0b4d` … `d256ac82e`): modelo, motor, desenho, UI e o
   **tracejado** (W3-bis) + a cena `=77`.

⛔ **O degrau de schema de cada plano está no commit dele** — rebasear fora de ordem parte a escada.

### 8.2 — ✅ Smokado pelo Enio, e APROVADO

| Cena | Veredito |
|---|---|
| `PH2D_BUILD_SMOKE=76` (Texture Pattern) | aprovado, depois de **quatro** reports curados nesta linha (o `Clamp` em branco · «filters anula pattern» · «em column o pattern some» · «pattern anula stroke», que era **a cena**) |
| `PH2D_BUILD_SMOKE=77` (o Pincel) | **«smoke ok»** (2026-08-29) |

O comando, inteiro:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_BUILD_SMOKE=77 cargo run -p ph2d-host-desktop --release
```

### 8.3 — ⏳ O que **NÃO** foi smokado (o integrador tem de saber)

- **A persistência do pincel**: há gate (`the_brush_survives_the_save`), mas ninguém gravou e
  reabriu um `.ph2dproj` com um traço-pincel à mão.
- **A recusa de um ficheiro antigo**: com `PROJECT_SCHEMA` a saltar 99 -> 103 na árvore fundida,
  qualquer `.ph2dproj` gravado antes tem de ser **recusado em voz alta**. Nenhum ficheiro velho foi
  aberto — não há projetos gravados (decisão do Enio, 26/08).
- **As QUINAS do pincel** (plano 36 W5): não existem. Um contorno com quina viva mostra as cópias a
  saltar no canto — é por isso que a cena `=77` é feita de curvas suaves, e a nota está escrita na
  mensagem do smoke.
- **O painel do pincel sob `reduced_motion=1`** (`~/.ph2d/prefs.txt`).

### 8.4 — Dívida NOMEADA (não é regressão, é declarado)

- `paint_bind::fade` esmaece a `fallback` de um pincel e **não** as cópias.
- O slider *Opacity* do preenchimento é **inerte** sobre um `Paint::Pattern` — **pré-existente**
  (plano 33), e curá-lo exige decidir o que a swatch do Fill significa num padrão (plano 35 §7.4).
  Pergunta já devolvida ao Enio.

---

## 9 — A LINHA para o `CLAUDE.md §5` (colar na integração, **no primário**)

> ⚠️ DIRETRIZ §1.5.9 item 8 + §1.5 tabela: o §5 edita-se **na integração**, uma linha de trabalho
> por vez. A narrativa vive **neste** handoff. Segue o texto pronto para a lista **Aberto** do
> módulo *Vector*, a substituir a cláusula `⏳ **(3)** … Texture pattern … SEM PLANO`:

```
✅ **(3) O TEXTURE PATTERN FECHOU, e virou DOIS modelos** (planos
[33](docs/Vector%20Module/33_plano_texture_pattern.md) · [35](docs/Vector%20Module/35_plano_padrao_no_traco.md) ·
[36](docs/Vector%20Module/36_plano_pincel_de_contorno.md), [handoff](docs/Vector%20Module/handoffs/HANDOFF_INTEGRACAO_line_Vector_pattern_brush_2026-08-29.md)):
o `Paint` ganhou a 5.ª variante e o `StrokeSpec` trocou uma COR por uma TINTA (`StrokePaint`).
⭐⭐ **São DOIS modelos, e todo aplicativo sério entrega os dois** — *"a coisa precisa funcionar sem
limitações"* (Enio, 28/08): **`Pattern`** é a TINTA que o contorno revela (normativo em SVG 2, e
por isso um tracejado são **buracos** no papel de parede — ⛔ não é defeito), e **`Brush`** é a ARTE
que PERCORRE a linha (o *Pattern Brush* do Illustrator), que escala com a largura e **reinicia em
cada traço**. ⚠️ A arte de um pincel é uma **FORMA do documento** (gesto de duas mãos, ⛔ sem
diálogo de ficheiro), e o motor já estava pago desde o plano 23 — faltava **endereçá-lo como
propriedade do traço**. ⚠️ Tetos MEDIDOS: `MAX_DASHES = 4096` (o joelho está entre 4 103 traços a
6,32 ms e 8 205 a 12,08, contra o *kill* de 8). ⏳ Falta a **W5, as QUINAS** — os 4 modos
automáticos do Illustrator medidos lado a lado antes de escolher o nosso; hoje um contorno com
quina viva mostra as cópias a saltar no canto, e as cenas de smoke são de curvas suaves por isso.
Cenas **`=76`** (a estampa) e **`=77`** (o pincel).
```

---

## 10 — ⚠️ Três leis que esta linha pagou, e que o integrador lê em 30 segundos

1. **A ficha da ferramenta possui uma COR, nunca a TINTA.** Ajustar o *Width* apagava o padrão do
   traço porque o `StrokeStyle::onto` declarava, por doc-comment, que *"nada do spec antigo
   sobrevive"* — verdade enquanto um traço tinha uma tinta só.
2. **Perante um report com foto, o primeiro passo é LER O ESTADO DO DOCUMENTO.** Quatro rondas de
   caça ao *"inverte"* / *"não resolveu"* fecharam quando o log passou a **nomear a tinta e a ficha
   inteira**: o contorno estava **TRACEJADO**, e o padrão desenhava certo o tempo todo. *Ler uma
   foto é gerar hipóteses; ler o documento é medir.*
3. **Um workaround oferecido no lugar de uma pesquisa é uma limitação transformada em política.**
   Eu respondi *"diminua o Width da estampa"*; o Enio devolveu *"sem limitações, qual o estado da
   arte?"*. A pesquisa mostrou que o buraco não era um knob — era o **segundo modelo**, cujo motor
   estava nesta árvore, pago e medido, há um mês.

---

## 11 — Estado da worktree

- `incremental/` **reclamado** (`rm -rf target/*/incremental`) — DIRETRIZ §1.5.9 item 7.
- ⭐ O binário do smoke fica **compilado** (`target/release/`) — item 9 do processo (`24f4a8276`).
- `git status` limpo; nada por commitar.

**Resumo:** *Linha `Vector` pronta (HEAD `d256ac82e`, 37 commits). Handoff de integração escrito.
UM conflito real, e é o `PROJECT_SCHEMA` — as duas linhas escreveram o literal `100`; a resposta é
**103**, contada. Aguardo ordem de integração.*
