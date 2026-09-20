# HANDOFF DE INTEGRAÇÃO — `line/Vector` (o ESQUELETO), a linha inteira · 2026-09-20

> **Para o agente INTEGRADOR.** Este documento não é a narrativa da jornada — ela vive na
> [fila do módulo](../01_a_fila.md), uma secção por wave (F9 … F36). O que está aqui é o que evita
> **conflito e regressão** ao trazer esta linha para o `main`.
>
> ⚠️ **Consulta-se por §, não se lê de ponta a ponta.** O §2 é a superfície de colisão, o §9 é o que
> uma leitura rápida do diff entende ao contrário, e o §11 é o que fica aberto e de quem é.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/Vector` |
| HEAD | **o commit deste handoff** — o tip de `line/Vector` (⚠️ um documento não pode nomear o próprio sha; `git rev-parse --short line/Vector` responde) |
| merge-base com `main` | `6d2d9db6f` — **igual ao tip do `main`**: o rebase já foi feito (§1.5.2 item 3) |
| commits | **75** (os `73` medidos pelo script do §2 + a F9 e este) |
| superfície | **286** ficheiros de código e doc, mais este handoff |

⚠️ **A linha é LONGA e isso é anti-padrão declarado** (DIRETRIZ §1.5.2 item 5: *«peça integração a
cada 1–2 jornadas»*). Ela acumulou porque o dono foi encadeando reports e waves sobre o mesmo
módulo, e só hoje mandou integrar.

⭐ **O rebase já está feito e é limpo.** Os únicos conflitos foram em **quatro ficheiros de
`project-memory/`** onde as duas árvores acrescentaram à **mesma lista** (o `main` trouxe as memórias
da sessão do Cascadeur); a resolução foi ficar com **os dois lados**, o do `main` primeiro.
⛔ **Zero conflitos em código.**

⚠️⚠️ **DUAS waves entraram DEPOIS do rebase, por ordem do dono de 2026-09-20** (*«decidir o que
fazer quando um ponto tem um osso só … O caminho da placa gráfica: implemente se esse é o padrão
ouro»*): a **F29** (`0029f2fc8`, os dois modos de peso, `PROJECT_SCHEMA` **+1**) e a **F9**
(`f53d48138`, a pele no dispositivo). ⇒ *o `PROJECT_SCHEMA` desta linha é hoje `148`, delta **+4***
contra o merge-base, e a §2 abaixo é a corrida NOVA do script.

---

## §2 — Superfície de colisão (saída do script, **colada**, não escrita de memória)

```

SUPERFÍCIE DE COLISÃO — line/Vector contra main
  merge-base 6d2d9db6f   ·   73 commit(s)   ·   286 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        148   (base: 144)
  ⚠   └ tripla do gate               (148, 13, 22)   (base: (144, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      23   (base: 23)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                               91   (base: 91)
    ph2d-render (espelho)                  92   (base: 92)
    ph2d-script (espelho)                  92   (base: 92)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0170   próximo livre: 0171
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

⚠️⚠️ **PRAZO DE VALIDADE (§1.5.9 item 3):** a tabela acima mede a linha contra o `main` de
**2026-09-20**. Se outra linha aterrar antes desta, **ela não reclama**, e re-rodar o script **não**
actualiza a coluna `base:` (ela é o merge-base). ⇒ leia o valor no ficheiro
(`git show main:shells/desktop/src/project_schema.rs`) e conte o degrau como **DELTA**.

### §2.1 — O DELTA que esta linha pede

| contador | base | linha | **delta** |
|---|---|---|---|
| `PROJECT_SCHEMA` | `144` | `147` | **`+3`** |
| a tripla do gate | `(144, 13, 22)` | `(147, 13, 22)` | **só o 1.º número** |
| `VEC_SCENE_SCHEMA` | `22` | `22` | **0** |
| registo `ph2d-ecs` · os dois espelhos | `91 · 92 · 92` | idem | **0** |

⭐ **Os três degraus, em ordem, e o que cada um carrega:**

1. `70c71b4e7` — **a ESCOLHA da lei de deformação, por desenho** (`SkinBind` ganhou `law: SkinLaw`);
2. `897e62677` — **o *Look At*** (um osso aponta para um alvo);
3. `77a3b7cd2` — **as correcções de peso à mão** (`SkinBind` ganhou `correcoes`).

⛔ **Os três são do MESMO componente (`SkinBind`) e o postcard é POSICIONAL.** Se renumerar, renumere
**a escada, a tripla e o degrau de migração** — os três sítios do `CLAUDE.md` §5.0 —, e
`python3 scripts/schema-recount.py` é quem os conta (com `assert` em cada passo).

⭐ **Nota que poupa tempo:** o `VEC_SCENE_SCHEMA` **não se mexe** apesar de a pele ter mudado três
vezes de forma — tudo o que a pele guarda viaja nos **bytes opacos** do `SkinBind::source`, que é do
`WorldSnapshot` e não da `VecScene`.

### §2.2 — Símbolos com valor LITERAL que podem colidir com outra linha

| símbolo | valor | onde | risco |
|---|---|---|---|
| `PropKind::BoneBendInX/InY/OutX/OutY` | **`13`–`16`** | `ph2d-timeline/src/prop.rs` | ⚠️⚠️ **discriminante EXPLÍCITO.** Outra linha que acrescente um `PropKind` pega o mesmo número e o git **funde limpo** — é o caso canónico do §1.5.5. |
| `PropKind::IkBendSide` | **`17`** | idem | idem |
| `PropKind::AUTOKEYED` | array de **`11`** | idem | ⚠️ lista ORDENADA com a **contagem no tipo** — duas linhas a acrescentar escrevem `11` as duas, e nenhuma está certa. |
| ids de chrome do osso | **11** consts | `ph2d-editor-core/src/ids/chrome/vector_bone.rs` | ficheiro **novo e só desta linha** ⇒ risco baixo |
| ids da ferramenta | **5** consts | `ph2d-tool-vector/src/ids/vector_bone.rs` | idem |
| chaves `ph2d-i18n` | **15** em `vector.rs` · **10** em `shell.rs` | `ph2d-i18n` | aditivo; colide só se outra linha escolher a **mesma chave** |

⛔ **Zero pacote EXTERNO novo no `Cargo.lock`** — o script confirma (*«nenhum '+name' novo»*). A única
mexida é uma **aresta interna** (dev-dependency de uma sonda).

---

## §3 — Foundational e partilhado tocado, e porquê

A linha é do **esqueleto**, que por desenho atravessa vários subsistemas
([ADR-0169](../../architecture/decisions/0169-the-skeleton-is-its-own-module-and-each-medium-answers-only-what-a-point-is.md):
*cada mídia responde só o que um ponto é*). O que ela tocou fora de `crates/ph2d-skeleton*`,
`ph2d-vec-*`, `ph2d-app-vec` e `ph2d-app-skeleton`:

| árvore | o quê | aditivo? |
|---|---|---|
| `ph2d-timeline` | **5 canais novos** (as quatro alças do osso · o lado da dobra) + o autokey deles | **sim** — variantes no FIM do enum |
| `ph2d-render` | `sprite/source_cells.rs` (a folha de quadros cortada por rectângulo) e o 9-slice a deformar | **sim** — ficheiros novos |
| `ph2d-render` (**F9, 2026-09-20**) | ⚠️ **o passe de sprites**: `SpriteMesh` ganha `skin`, `MeshFrame` ganha 4 tabelas, o `sprite.wgsl` ganha um `@group(2)` de **quatro** buffers, o `skin_bgl` vai de `3` para `4` entradas, e o `renderer_draw`/`clip_pass` ligam-no em **quatro** sítios | ⚠️ ver abaixo |
| `ph2d-poly2d` | `clip_rect`, `refine_adaptive_arestas` | **sim** — ficheiros novos |
| `ph2d-ecs` | **uma linha**: `pub use bevy_ecs::query::{Has, With, Without}` | **sim** |
| `ph2d-i18n` | 25 chaves | **sim** |
| `ph2d-component-desc` | `catalog/skeleton.rs` | **sim** |
| `ph2d-preview-drive` | o ledger passa a conhecer a pose de osso | aditivo |
| `ph2d-editor-core` | os ids do chrome do osso + **3** arch-gates | aditivo |
| `shells/desktop` | **12** fases novas do quadro (`render_loop/fase_*`) + **5** do `input_dispatch` | ⚠️ ver abaixo |

⛔⛔ **E há um SEGUNDO atrito, novo desde 2026-09-20: o `ph2d-render` é território disputado.**
O `CLAUDE.md` §5 diz por escrito que *«o `gpu-cook` e o `renderer_draw` estão a ser reescritos por
duas linhas vivas»* (a nota do `ParticleEmitter`). ⇒ ao fundir, confira **nominalmente**:

| ficheiro | o que esta linha lhe fez |
|---|---|
| `src/shaders/sprite.wgsl` | `@group(2)` com **4** bindings e o `posa_pela_pele` reescrito (a lei da JUNTA) |
| `src/pipeline.rs` | **uma** constante: `from_fn::<_, 3, _>` → `from_fn::<_, 4, _>` no `skin_bgl` |
| `src/renderer_draw.rs` | o `skin_buffers.upload(…)` ganha o 4.º argumento |
| `src/sprite_mesh.rs` | `MeshFrame` ganha `juntas`, e o `clear`/`push` truncam-na |
| `src/picking.rs` | `DrawnMesh` passa a guardar um `Cow` ⇒ **perdeu o `Copy`** |

⭐ **`SpriteMesh::skin` é `Option` e nasce `None`**, logo toda malha que não é uma pele desenha
**byte-idêntica por construção** — o `SEM_PELE` decide **antes** de qualquer multiplicação, e não
por multiplicar por uma identidade (`inf · 0` é `NaN`). ⇒ *uma linha que só desenhe sprites não é
tocada por isto*; quem colide é quem editar os cinco ficheiros acima.

⚠️⚠️ **O atrito real do resto da linha é a ORDEM DAS FASES do quadro, não os ficheiros.** As fases novas
(`fase_bone_ik_and_limits`, `fase_bone_smart_and_knobs`, `fase_skeleton_verbs`,
`fase_timeline_key_insert`, `fase_selection_mirror_*`, `fase_vector_bone_overlay`,
`despacho_peso_do_osso`…) entram num `render_loop/mod.rs` que é **um índice de fases** — duas linhas
a acrescentar fases **fundem limpo** e a ordem resultante pode não ser nenhuma das duas.

⭐ **A rede existe e reprova alto:** os gates de ordem da shell
(`a_hierarchy_drag_leaves_the_capture_a_fixed_point` e irmãos) medem *todo escritor da árvore corre
antes de ela ser lida, e a leitura antes da captura* — e o portão do integrador corre-os. ⛔ Eles
vivem em `shells/desktop/tests/it/`, que um `cargo check --all-targets` **não** compila.

---

## §4 — Contratos congelados (§6)

**NENHUM encostado.** O script confirma `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs`
**intocados** ⇒ **nenhum ADR é exigido**, e a linha não cria ADR nenhum (último no disco `0170`,
próximo livre `0171`) ⇒ **fora de toda disputa de número**.

---

## §5 — O que só o `ship.sh` apanha — **já corrido nesta árvore**

| verificação | resultado |
|---|---|
| `cargo fmt --all -- --check` | **0 diffs** |
| `typos` (config do repo) | **0** |
| `cargo machete` | *«didn't find any unused dependencies»* |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | **zero** |
| `scripts/check-standalone-optional.sh` | **10** crates com dependência interna opcional, todas compilam sozinhas |
| `scripts/check-workflow-packages.sh` | **32** nomes citados por workflow, todos existem |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **zero** |

⚠️ **O `fmt` e o `typos` eram dívida DESTA linha, não pré-fork.** 17 ficheiros da própria linha nunca
tinham passado pelo `cargo fmt` (as sessões correram só `cargo check -p`, que não o exige) e o
`typos` acusava 7 em dois deles. Curados em `679c3ea8a`: a abreviatura `anc_*` (de *âncora*) virou
`ancora_*` em 12 sítios, e uma mensagem de gate passou a nomear o caracter em vez da palavra.
*Quem cura é quem escreveu* (§1.5.9 item 5).

---

## §6 — O que só a ÁRVORE COMBINADA apanha (§1.5.9 item 5-bis) — **corrido DEPOIS do rebase**

```
bash scripts/censos-da-arvore-combinada.sh
  → 90 tests run: 90 passed  ·  controlo do filtro: 8 de 8 censos correram ✓
```

⇒ **censo de texto (HR-15) e tecto de LOC verdes sobre a soma.** ⛔ Isto **não substitui** o portão
do integrador (`--ff-only` + `foundational-integrate.sh`): o que muda é que estas duas famílias
deixam de ser **descobertas** lá, em série.

---

## §7 — Ordem e dependências entre os commits

⭐ **Não há ordem a preservar além da do próprio histórico** — a linha é uma cadeia linear já
rebaseada, e cada wave assenta na anterior.

⛔ **Não faça cherry-pick parcial.** Vários pares são *lei + o report do dono que a corrigiu horas
depois*, e o primeiro de cada par shipa um comportamento que o segundo **substitui**:

1. a lei da curva (F30) → **o gesto que a alcança** (a mancha no contorno, F31);
2. o refit que sai e a correcção das alças que entra (F33) → **as duas alças a rodar juntas** (F35);
3. o pincel de peso (F26) → as **três** waves de report que se lhe seguem.

---

## §8 — O que SMOKAR, e o que **não** foi smokado

**O smoke desta linha, byte a byte** (o binário fica quente — ver §13):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

✅ **Aprovado pelo dono em 2026-09-20** (*«Muito bom. Parabéns. Smoke OK»*) nas duas curas de fecho:
a **espessura do contorno** que sobrevive ao quadro, e o **contorno sem bicos** ao dobrar a barra.

⏳ **O que NÃO foi smokado, e o integrador deve saber:**

- as cenas `=2` e `=3` do mesmo smoke (as três mídias · o braço a animar) foram smokadas nas
  **jornadas delas**, não depois do rebase;
- ⛔⛔ **o caminho de GPU da pele (F9) SHIPA LIGADO e NÃO foi smokado pelo dono** — ele fechou
  depois da aprovação de 2026-09-20. ⚠️ Ele é o caminho de **OMISSÃO** de toda arte presa, logo o
  re-smoke do `PH2D_VEC_BONE_SMOKE=1` (e das cenas `=2`/`=3`) é o **primeiro** pedido desta linha ao
  dono. A bissecção é `PH2D_SKIN_GPU=0`, que devolve a lei à CPU **sem recompilar**:
  ```
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_SKIN_GPU=0 PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
  ```
  ⭐ **O que o substitui enquanto não há smoke** é um gate de **PIXEL** sobre um adaptador real
  (`sprite_mesh_gpu::a_placa_desenha_o_que_a_cpu_posa`), que lê **`0 px`** de diferença entre as
  duas imagens. ⚠️ **Ele é `#[ignore]`, logo o CI nunca o corre** — corra-o à mão com
  `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-render --test it -- --ignored --exact sprite_mesh_gpu::a_placa_desenha_o_que_a_cpu_posa`;
- ⛔ **um clique não é provável na sessão virtual da fotografia** (o XTest da Xwayland é ignorado e o
  `ydotool` mexe no rato REAL do dono) ⇒ toda costura de clique desta linha prova-se em **gate**
  (`seam_*` / `MockPanelHost`), nunca por foto. O instrumento da foto é
  [`fotografa_cena.sh`](../../Components/ferramentas/fotografa_cena.sh), e ele **recusa** correr com
  `DISPLAY=:0`.

---

## §9 — Nove coisas que uma leitura rápida do diff entende ao contrário

1. **`VecPath::replace_geometry` não é um `replace_cooked` mais fraco.** São **duas perguntas**, e
   quem as separa é a FONTE: *parâmetros vivos* produzem estilo, uma *fotografia congelada* produz só
   posições. As duas destruturam a struct de forma **exaustiva** de propósito — um campo novo obriga
   a responder as duas no commit em que nasce.
2. **A `reconcilia` não é uma cerca sobre a `correccao_das_alcas`.** A correcção continua **livre no
   plano**; o passe põe as **duas alças de um nó** a rodar pelo mesmo ângulo. ⛔ *Prender cada alça à
   direcção dela foi construído, medido e recusado: dá quebra zero e **mata a F30*** (numa aresta
   recta o segmento deixa de arquear).
3. **`Skin::blend_linear` não é código morto.** Ele é a lei de antes, é o **CONTROLO** de todo gate
   que mede a cura do entalhe, e é usado *dentro* da lei nova para a translação do centro.
4. **A subdivisão do bind não é uma opção de qualidade:** é ela que torna o pincel de peso alcançável
   entre os nós. Vários gates pedem a barra **GROSSA** pelo nome (`barra_da_cena_com(false)`) porque é
   ela que contém o fenómeno de um ficheiro **gravado antes** daquela wave.
5. **Os 5 `PropKind` novos não são «mais canais»:** eles fazem a timeline animar a **curvatura** de um
   osso e o **lado da dobra** da IK — sem eles o *Onion* mostrava o quad de repouso.
6. **`pior_desvio` e `pior_desvio_do_desenho` medem coisas diferentes** (a REPRESENTAÇÃO contra o
   DESENHO), e a linha usa os dois de propósito: o primeiro **não afirma nada** quando a contagem de
   vértices muda.
7. ⛔⛔ **O `skin_gpu` não foi «substituído»: ele foi APAGADO por ser ÓRFÃO.** Ele era o payload de
   2026-09-17 (empacotamento + lei de referência), **sem um único consumidor de produto**, e com a
   lei **LINEAR** — a antiga — lá dentro, defendida por um gate que AFIRMAVA que as duas leis
   diferem. A F9 construiu a outra metade e o seu **próprio** payload. ⇒ manter os dois era deixar
   uma lei errada viva onde o próximo agente a pode ligar. ⭐ As duas propriedades dele que
   sobrevivem passaram a medir-se contra o payload VIVO, e a primeira ficou **mais forte**.
8. ⛔ **A `Especie` da F29 não é «um enum onde havia um bool».** Ela **CARREGA o número**
   (`Soma(f64)` / `Alvo(f64)`), e é isso que torna impossível ler um `delta` como se fosse um
   `alvo`: *um campo cujo significado depende de um modo guardado ao lado é um defeito à espera*.
   ⇒ o `PROJECT_SCHEMA` **tem** de subir, porque o campo foi **TROCADO** e não apendado — um
   ficheiro do schema anterior leria o primeiro byte do `f64` como o discriminante, em silêncio.
9. **Os `*_tests.rs` partidos em irmãos** (`curva_alcas_tests`, `ponto_novo_salto_tests`,
   `skin_live_traco_tests`) não são reorganização: foram **tectos de LOC curados por CORTE**, e
   **nenhuma** entrada nova entrou no `FILE_OVERAGE_OK`.

---

## §10 — As premissas que a medição derrubou (para não voltarem)

1. *«a 2.ª mídia está bloqueada: falta malha sobre a imagem»* — **duas das quatro peças já existiam**,
   e o doc de uma delas dizia-o por escrito.
2. *«o `smoothstep` no peso cura a quina»* — corta o p50 a meio (`9,05° → 3,98°`) e **não** toca no
   máximo (`28,62° → 26,98°`).
3. *«partir um segmento não move o desenho, por construção»* — verdade com o ajuste desacoplado,
   **falsa** desde a conciliação: `0,0201 %` da peça, com a barra posta no vale contra os `11,11 %`
   da compensação da F28.
4. *«a causa da espessura que não muda é o painel»* — o valor **chegava** ao documento; quem o
   desfazia era o re-cozimento.
5. *«o dual quaternion / os centros de rotação optimizados curam a dobra»* — os dois **refutados com
   número**, e a causa é do MEIO: uma folha plana com os ossos no plano dela.
6. *«uma alça degenerada precisa de uma cascata de tangente»* — construída e **removida**: nenhuma
   mutação a conseguia matar.
7. ⛔⛔ *«a lei da pele é uma mistura LINEAR de afins»* — **verdade até 2026-09-19 e falsa desde
   então**, e a F9 foi desenhada em cima dela. O portão de paridade leu `2,315e-3 m`. *A dívida
   estava NOMEADA no repo e eu não a li antes de desenhar.*
8. *«a F9 compra `~11 %` de um quadro»* — a decomposição lê **`35,9 %`** a 8 imagens, `4,5 ×` a
   nota que parou a fila, e **`96 %` disso é a lei por vértice**. ⚠️ A minha hipótese era a CÓPIA
   (`0,2 %`): *uma sonda que não parte o relógio acusa o suspeito errado.*
9. *«uma fixtura de dois ossos mede a lei do centro»* — com **um** par só, `Σ wᵢwⱼ·junta / Σ wᵢwⱼ`
   devolve a junta seja qual for o peso; a mutação que apagava a ponderação **sobreviveu**.
10. *«o caso degenerado da F29 é observável na faixa»* — ele só se vê em **`v = 0`**: com
    `Σoutros = 0` a renormalização final devolve `1` para todo valor positivo.

---

## §11 — O que fica ABERTO, e de quem é cada item

| item | de quem |
|---|---|
| ⛔ **O SMOKE da F9** — a pele no dispositivo é o caminho de OMISSÃO e **não foi visto pelo dono** (ver §8) | **DONO** |
| O **memo do payload** da pele: os `12,5 %` que sobram **não são deformação**, são a tabela de pesos derivada por vértice a cada construção. ⭐ Ela é grandeza do **BIND** (a quota sai do REPOUSO) | linha |
| A truncagem a `K = 4` passou a tocar o **CENTRO** e não só os pesos — exacta com `≤ 4` ossos por vértice (a arte do dono tem `3`), aproximação **declarada** acima disso | declarado |
| A **150°** a face de dentro do cotovelo ainda se dobra sobre si mesma | geometria, **declarado** |
| Um caminho com **EFFECTS** não é subdividido no bind | linha |
| A mídia **imagem** ancora a mancha no vértice mais perto (a vectorial já usa o contorno) | linha |
| **F12** — o *Frame All* enquadra a janela e os painéis tapam-lhe as bordas | linha |
| O `ph2d-panel-skeleton` pinta a barra de rolagem com o `VECTOR_SCROLLBAR_ID` ⇒ com o painel de vetor fechado o arrasto do polegar nunca arma — **achado da `line/components`, não curado** | linha |

---

## §12 — A UMA LINHA do `CLAUDE.md` §5 (§1.5.9 item 8)

A narrativa vive **aqui** e na [fila](../01_a_fila.md). O `CLAUDE.md` §5 recebe **uma linha** — a de
estado/aberto do módulo, com o ponteiro para este handoff. ⛔ **Não acrescente um parágrafo de
jornada:** foi assim que o §5 chegou a 917 KB, e ele é injectado por inteiro em todo agente.

---

## §12-bis — ⚠️ Uma flake de carga NOVA, para PROMOVER à lista do `CLAUDE.md` §5.0

**`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`** (`ph2d-tool-painter`,
`tool::paint::tests::measure_input_cost`) — reprovou no meio do fan-out de **16 654** testes e passa
**3 de 3 sozinho a `load 5,55`**, com **ZERO linhas de diff desta linha** naquela crate
(`git diff --stat <merge-base>..HEAD -- crates/ph2d-tool-painter` devolve vazio). ⇒ as três
assinaturas da família: gate de RELÓGIO · zero diff · verde isolado com a carga impressa ao lado.

⭐ *A linha pede, o integrador escreve* — é o protocolo que as rodadas de 09/09 e 09/17 já usaram.

---

## §13 — Prova de fecho

⛔⛔ **A varredura impactada corrida DEPOIS da F9 deu TRÊS vermelhos, e dois eram reais.** Os dois
vivem em `ph2d-editor-core/tests/it/`, que é **a família que o `CLAUDE.md` §2 nomeia**: *uma corrida
por-crate é cega a um gate que mede a minha crate e vive noutra.* Ficam registados porque a cura de
cada um é uma lei:

| gate | o que era, e a cura |
|---|---|
| `architecture_who_reads_the_posed_skin_mesh` | os **três** ficheiros novos da F9 apareceram como *leitores fora do censo* — e eles **não** são costuras que precisam de resposta quando a placa posar: eles **SÃO** a placa a posar. ⇒ `MOTOR` de `7` para `10`. ⭐ E a **PREMISSA da tabela morreu**: a «W3» que ela orçava **aconteceu**, logo ela deixou de medir *«o que cada costura vai precisar»* e passa a medir *«quem paga o `posado()` por PERGUNTA»* — a morte está escrita no diff |
| `the_protection_tint_rides_the_sprite_pass_with_the_art_mesh` | a agulha pedia `out.push(inst, malha)` e a porta passou a devolver um `Cow` ⇒ a chamada é `malha.as_deref()`, **sem uma linha de comportamento mudar**. ⛔ **SEXTA vez** que este gate paga a mesma forma (a 1.ª foi um `pub`): *uma agulha nomeia a LEI, nunca a GRAFIA de um argumento nem a VISIBILIDADE de uma função* |
| `the_pen_down_is_still_a_canvas_copy…` | **flake de CARGA** — ver §12-bis |

```
git rebase main                        → 70/70 · conflitos SÓ em 4 .md de project-memory (listas)
scripts/nextest-impacted.sh            → 16 654 tests run: 16 654 passed        (PÓS-F9, 2.ª corrida)
scripts/censos-da-arvore-combinada.sh  → 90 passed · 8 de 8 censos correram     (PÓS-F9)
cargo clippy --workspace --all-targets --all-features -- -D warnings → zero
cargo fmt --all -- --check             → 0 diffs
typos                                  → 0
cargo machete                          → nenhuma dependência por usar
CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets → zero
scripts/check-standalone-optional.sh   → 10/10
scripts/check-workflow-packages.sh     → 32/32

rm -rf target/*/incremental            → 30,8 GB reclamados (28 do debug + 2,8 do smoke)   (PÓS-F9)
build do smoke (1.ª corrida)           → Finished in 30,23s
build do smoke (2.ª corrida)           → Finished in 0,27s, ZERO linhas "Compiling"   ← a PROVA
```

⚠️ **O binário do smoke fica quente NESTA worktree** (`target/smoke/ph2d-host-desktop`, 84,8 MB) — é
o `cd` do §8 que o alcança, e só ele.

**Instrumentos que este handoff cita vivem versionados:** `scripts/collision-surface.sh`,
`scripts/censos-da-arvore-combinada.sh`, `scripts/nextest-impacted.sh`,
`scripts/schema-recount.py` e
[`docs/Components/ferramentas/fotografa_cena.sh`](../../Components/ferramentas/fotografa_cena.sh).
