# HANDOFF DE INTEGRAÇÃO — `line/3DModeling` · **FECHO DA LINHA** (2026-09-16)

**Ordem do dono:** *«Smoke OK. Antes de seguir vamos integrar ao main. Escreva handoff.»*

⚠️ **Este é o handoff de FECHO (DIRETRIZ §1.5.9): ele é o ÍNDICE e o mapa de colisão.** O
*mecanismo* de cada wave vive nos docs e nos três handoffs de wave que a linha já entregou (§2) —
este não os repete, e um integrador que precise do *porquê* de uma decisão vai lá.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/3DModeling` |
| HEAD | a **ponta** de `line/3DModeling` — ⚠️ *este handoff é o último commit dela*, logo um hash escrito aqui nascia obsoleto; o último commit de **produto** é o `281ca4bda` (o chão) |
| merge-base com `origin/main` | `1d43da737` (2026-09-13) |
| commits | **90** (89 de produto + este) |
| ficheiros | **238** · `+45 417` / `−1 520` (sem contar este handoff) |

⭐⭐ **O `main` NÃO ANDOU: `0` commits desde o merge-base.** `git merge-base --is-ancestor
origin/main HEAD` responde **verdadeiro** ⇒ a integração é um **`--ff-only` directo**, sem rebase e
sem um único conflito possível **hoje**.

⛔⛔ **PRAZO DE VALIDADE, e ele é a coisa mais importante desta página:** isto é verdade *enquanto
esta for a primeira linha da rodada*. Se outra entrar antes, **tudo em §3 tem de ser RE-CONTADO como
DELTA** contra o valor que o `main` passar a ter — lido **no ficheiro** (`git show main:<arq>`),
nunca na coluna `base:` do mapa, que é o merge-base e **não** se actualiza (CLAUDE.md §1, linha do
integrador).

---

## §2 — O que a linha traz — o índice

| bloco | o quê | onde está o mecanismo |
|---|---|---|
| **W148 · a cache contra o CASCO** | a cache de fitas passa a guardar uma fita por **casco de folha**; servir por CAIXA só acertava `1,9`–`2,4×` | [`docs/3DModeling/12_a_cache_contra_o_casco.md`](../12_a_cache_contra_o_casco.md) · [handoff de 13/09](HANDOFF_INTEGRACAO_line_3DModeling_2026-09-13.md) |
| **O MODO RENDER** (material · luz · céu · olhar) | duas crates novas de LEI (`ph2d-material` OpenPBR, `ph2d-view-transform`), material **por objecto**, selector de cor, emissivo, verniz, o céu com FONTE, a luz como **objecto da cena com gizmo** | [`docs/Render3d/05_o_modo_render_do_modelador.md`](../../Render3d/05_o_modo_render_do_modelador.md) (§1–§43) |
| **A PLACA** | o campo vira **WGSL**, a marcha, o G-buffer, a sombra, a oclusão e o **pintor inteiro** vão para o dispositivo; várias lâmpadas; a escultura atravessa | `05` §36–§43 + [`ph2d-field-gpu`](../../../crates/ph2d-field-gpu/) (crate nova) |
| **W4 · a peça faz sombra e ganha profundidade** | sombra própria + oclusão que refina com a mão parada | `05` §30–§35 |
| **W4 · O CHÃO que só recebe** | a peça pousa num chão **invisível** | [`docs/Render3d/07_o_chao_que_so_recebe.md`](../../Render3d/07_o_chao_que_so_recebe.md) · [handoff do CHÃO](HANDOFF_INTEGRACAO_line_3DModeling_O_CHAO_2026-09-16.md) |
| **O ARCO no perfil** | a quina arredondada vira **um arco** na fita (o vaso de `77` para `26 ms`), a barra do arco é a que está escrita, o preview guarda os arcos | [handoff do ARCO](HANDOFF_INTEGRACAO_line_3DModeling_O_ARCO_2026-09-16.md) · [`docs/Render3d/06_auditoria_do_vaso.md`](../../Render3d/06_auditoria_do_vaso.md) |
| **INFRA — o tecto de recursos por LINHA** | uma linha deixa de poder tomar a máquina: CPU ≤ 50 % **por linha**, RAM sem swap, prazo, **exclusão da GPU** | [`docs/DevOps/TETOS_DE_RECURSO_POR_LINHA.md`](../../DevOps/TETOS_DE_RECURSO_POR_LINHA.md) |

---

## §3 — ⚠️ CONTADORES PARTILHADOS — a saída da ferramenta, **colada**

```
SUPERFÍCIE DE COLISÃO — line/3DModeling contra main
  merge-base 1d43da737   ·   89 commit(s)   ·   238 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        132   (base: 128)
  ⚠   └ tripla do gate               (132, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠ FIELD_DOC_VERSION                      23   (base: 22)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                               85   (base: 85)
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — último no disco: 0169   próximo livre: 0170
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 3 pacote(s) '+name' novo(s):
      "ph2d-field-gpu"  "ph2d-material"  "ph2d-view-transform"

▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⇒ **os dois degraus desta linha são `PROJECT_SCHEMA` `+4` e `FIELD_DOC_VERSION` `+1`.** ⚠️ Conte-os
como **delta**; a tripla do gate vive em ficheiro IRMÃO do da escada e os dois têm de subir juntos.

⛔⛔ **E há OUTRA linha na mesma faixa, medida hoje (16/09):** a `line/Vector` escreve
`PROJECT_SCHEMA = 133` sobre o **mesmo** `main` de `128` — ou seja `+5`. Os literais **não** colidem
(`132` contra `133`), o que é pior do que parece: *um merge textual funde os dois sem um marcador de
conflito e o degrau da segunda linha evapora*. ⇒ **quem entrar em segundo re-conta o seu DELTA sobre
o valor que o `main` passar a ter** (esta entrando primeiro, a `Vector` vai a `132 → 137`, nunca
`133`). ⚠️ Medido com `git show <ramo>:shells/desktop/src/project_schema.rs`, **não** de memória.

⚠️ **Estavam vivas SEIS linhas às 16/09** (`git worktree list`), com commits próprios sobre este
mesmo `main`: `3DModeling` (90) · `sculpt3d` (193) · `Vector` (147) · `UIUX` (94) · `motion-value`
(69) · `components` (67). *A afirmação «`main` não andou» do §1 é sobre o passado; o futuro desta
rodada depende da ORDEM que o integrador escolher.*

✅ **Os três pacotes novos do `Cargo.lock` são INTERNOS** (crates nossas, `crates/ph2d-*`) —
**nenhuma dependência externa nova** entra com esta linha, logo não há `cargo deny`/`audit` novo a
julgar.

---

## §4 — Foundational / partilhado tocado, e **porquê**

| ficheiro | | porquê | risco de mesmo-símbolo |
|---|---|---|---|
| `crates/ph2d-material/**` | **novo** | a lei OpenPBR em CPU, provada contra as fixturas MaterialX | nenhum — crate nova |
| `crates/ph2d-view-transform/**` | **novo** | exposição + *view transform* da cena, provados contra o Blender | nenhum — crate nova |
| `crates/ph2d-field-gpu/**` | **novo** | a marcha, o pintor e o chão no dispositivo | nenhum — crate nova |
| `Cargo.toml` | M | a lista `[profile.dev.package]` `opt-level = 2` ganha **`ph2d-field-render` e `ph2d-material`** (o quadro de referência corria a `opt-0` e toda comparação CPU×placa era inválida) | ⚠️ **lista partilhada** — outra linha que lhe acrescente uma crate colide no mesmo bloco |
| `.gitignore` | M | des-ignora `.claude/settings.json` e `.claude/hooks/` | ⚠️ ver a linha seguinte |
| `.claude/hooks/tecto-de-recursos{,.prova}.sh` · `.claude/settings.json` | **novos** | o guarda que recusa comando pesado fora da porta e vigia sem prazo — *uma cerca que só protege a árvore onde foi escrita não é uma cerca* | ⚠️ qualquer linha que também versione `.claude/` colide |
| `scripts/ph2d-run.sh` | M | o tecto passa a ser da **LINHA** (o por-comando **não compõe**: dois comandos com tecto próprio somam `30,05` de 32 núcleos; na fatia da linha dão `16,03`) | ⚠️ ficheiro de infra partilhado |
| `scripts/{ship,nextest-impacted,cargo-test-narrow}.sh` | M | entram na porta de recursos sozinhos | ⚠️ idem |
| `.claude/commands/pd-linha-fechar.md` | M | passa a chamar o `cargo machete` **pelo nome** (a §1.5.9 pedia-o e nenhuma linha o corria) | baixo |
| `crates/ph2d-editor-core` (6 f) · `ph2d-vec-scene` (5) · `shells/desktop` (3) · `ph2d-gpu` (2) · `ph2d-i18n` · `ph2d-component-desc` · `ph2d-panel-registry-init` · `ph2d-ui-testkit` · `ph2d-vec-edit` | M | fiação do modo Render (pulldown de área, selector de cor, gizmo da luz) e os gates que moram com o que exercitam | ⚠️ `ph2d-i18n`: **47 chaves novas**, todas em `model3d.rs` e prefixadas `field.dim.*` / `panel.model3d.*` ⇒ colisão só se outra linha editar **aquele** ficheiro |
| `project-memory/**` (12 f) | M/A | as lições da jornada | conflito textual banal |
| `CLAUDE.md` | M | **UMA** linha no §5 (o chão + o arco) e **uma correcção**: a nota que dizia que o `collision-surface.sh` não vê o `FIELD_DOC_VERSION` **envelheceu** (o `f065b17bb`, já no main, acrescentou-o) | ⚠️ toda linha edita o §5 |

⛔ **Nada em `crates/ph2d-nodegraph` nem em `ph2d-editor-core/src/tool.rs`** ⇒ **contrato congelado
intocado** (§4 da DIRETRIZ), **zero ADR**.

---

## §5 — Os portões, corridos **1× sobre o diff acumulado**

| portão | veredito |
|---|---|
| `nextest-impacted` (`BASE=HEAD`) | ✅ **13 583 / 13 583**, `0` reprovadas (`58,8 s`, CPU `91 %` ociosa) |
| `cargo clippy --all-targets` (as 3 crates do chão) | ✅ **zero** avisos |
| `cargo fmt --all` | ✅ aplicado — ⚠️ e apanhou **dois ficheiros que esta linha escreveu** e que estavam vermelhos desde `02554bfae` e `b29972e57`, numa crate **vizinha** e na shell (§6-bis do handoff do CHÃO): *um fecho que formata só as suas crates é cego ao que a linha escreveu fora delas* |
| `cargo machete` | ✅ **nenhuma dependência declarada e não usada** |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✅ **zero** — o `dead_code` que só a workspace inteira vê |
| `scripts/check-standalone-optional.sh` | ✅ as **10** crates com dependência interna opcional compilam sozinhas |
| `scripts/check-workflow-packages.sh` | ✅ os **32** nomes citados pelos workflows existem entre os `362` membros |
| `scripts/doc-index.sh --check` | ✅ **19 índices em dia** |
| bateria de **PLACA** (`--ignored`, app-field3d) | **62 / 63** — ver §6 |

⛔ **O que FICA para o `ship.sh`** (não é gate de integração e não foi corrido aqui): `typos`,
`cargo deny`/`audit` e a suíte **da workspace inteira** no perfil `ci-test`. ⚠️ As duas primeiras
têm risco baixo nesta linha — **nenhuma dependência externa nova** (§3) —, e a terceira é o que a
varredura impactada não cobre por construção.

---

## §6 — ⛔ O ÚNICO vermelho, e porque ele não é desta linha

`preview::device_tests::com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento`.

**Duas provas de que não é o chão:**

1. ele passa **`None`** no chão à `gpu_frame::paint` — o quadro de MOVIMENTO que ele mede **não
   conhece** esta wave;
2. com a **CPU a `82`–`91 % OCIOSA** ele passa **3 de 3** (`12`, `11`, `11` de `18`) — que é o
   `11 de 18` que a [`05` §43.9](../../Render3d/05_o_modo_render_do_modelador.md) lhe registou
   **antes** desta wave existir.

⛔⛔ **E o crivo NÃO é o `loadavg` — isto custou-me três corridas.** A `load 7,5` ele lê `5`, `2` e
`3` de `18` e reprova, porque aquilo é uma média de **um minuto a decair** (a de cinco lia `32` no
mesmo instante). A grandeza é a **ociosidade**, que o próprio gate imprime. *Uma régua que desmente
uma flake tem de medir a grandeza que a flake segue, não a que é fácil de ler.*

⚠️ **Vizinho já registado:** sob `cargo test` (não `nextest`) a suíte `ph2d-field-render --test it`
reprova entre `0` e `6` gates de CONTAGEM, com o conjunto a mudar entre corridas — estado
partilhado. Sob `nextest` os `93` passam. *O executor que conta é o `nextest`.*

---

## §7 — O que smokar, e **o que NÃO foi smokado**

**Aprovado pelo dono** (hoje): o chão invisível —

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

barra da área 3D → o chip **`Matcap`** → **`Render`**; a peça ganha sombra, e levantá-la com a seta
do gizmo afasta-a e amolece-a.

⏳ **NÃO smokado, e nomeado:**

- a **cena do contorno misto** no preview (arcos + tesselado juntos) — adiada pelo dono, à espera de
  placa calma;
- as **quatro pendências de placa** que ele mandou adiar, na lista viva
  [`doc 06 §13.0`](../06_resultados_cena_e_gizmo.md) (comparar A/B · os laços `0..CENAS` que medem a
  cena `1` duas vezes e nunca a última · a cena do contorno misto · re-correr a bateria de placa);
- o **`CENAS` do roteador é `32`** — conte-o em
  [`smoke_scenes.rs`](../../../crates/ph2d-app-field3d/src/smoke_scenes.rs), nunca aqui.

---

## §8 — Ordem e dependências

- os 89 commits são **lineares** e não há cherry-pick: integra-se o ramo inteiro;
- ⚠️ **o `.gitignore` e os `.claude/hooks/` têm de aterrar no MESMO merge** — o primeiro é o que faz
  os segundos existirem para o git;
- ⚠️ **`Cargo.toml` e `Cargo.lock` aterram juntos** (as três crates novas).

---

## §9 — ⏳ O que fica aberto (além do §7)

- o chão **não devolve luz** à peça — é a `W5` (GI), e o chão será o primeiro receptor dela;
- um só chão, **horizontal**, com a altura lida da cena e **sem botão de «pousar outra vez»**;
- sem lâmpada nenhuma o dispositivo **recusa** o quadro e o chão é pintado pela CPU (cerca anterior
  a esta wave);
- a lista viva do módulo é o [`doc 06 §13.0`](../06_resultados_cena_e_gizmo.md); a do render é o
  [`03_o_plano.md`](../../Render3d/03_o_plano.md), com a **`W4` fechada**.
