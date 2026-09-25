# HANDOFF de INTEGRAÇÃO — `line/Vector`: o CAMPO do domínio, o bind SEM pontos novos, e a pesquisa do estado da arte (2026-09-24)

> **Para o agente INTEGRADOR, noutra janela, por ordem do dono** (*«antes de seguir, vamos integrar a
> linha ao main — escreva handoff para que outro agente em outra janela faça a integração»*).
> Leia este documento inteiro antes do primeiro comando. ⚠️ Ele descreve **só** os commits desta
> rodada; a rodada anterior (75 commits, F9/F29) já está no `main` e o documento dela é o
> [handoff de 2026-09-20](HANDOFF_INTEGRACAO_line_Vector_A_LINHA_2026-09-20.md).
>
> ⚠️ Os hashes abaixo são os da linha **rebaseada sobre `20a630f1b`**; se o integrador rebasear outra
> vez, eles mudam — procure pelo assunto do commit.

## 0. Onde está e o que fazer

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| ramo | `line/Vector` |
| base | **rebaseado sobre o `main` de 2026-09-24** (`20a630f1b`), `behind 0` quando este doc foi escrito |
| forma | **fast-forward** — sem conflito com o `main` (a única coisa que o `main` andou foi uma nota de memória em `project-memory/`, sem sobreposição) |
| ⚠️ `CARGO_TARGET_DIR` | a worktree usa o `target/` **dela**. ⛔⛔ **Nunca partilhe o target entre worktrees** (troca os `.rlib`; memória `feedback_sharing_a_target_dir_between_worktrees_corrupts_the_build`) |

**Passos, na ordem** (DIRETRIZ §1.5.3; `/pd-integracao`):

1. No primário: `cd /home/enio/Documentos/Projetos/PH2D && git status` — ⚠️ o primário tem ficheiros
   de `project-memory/` modificados e por rastrear **de outras sessões**; não são desta linha e não
   entram no merge (a linha não toca em `project-memory/`).
2. Se o `main` tiver andado desde `20a630f1b`: `git -C Worktrees/line-Vector rebase main` e **reconte**
   o mapa: `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` (⚠️ caminho
   ABSOLUTO; ⚠️ a coluna `base:` é o merge-base, não o `main` de agora).
3. `bash scripts/foundational-integrate.sh line/Vector` (gate da árvore combinada) e
   `bash scripts/censos-da-arvore-combinada.sh`.
4. `git merge --ff-only line/Vector` no `main`. **Ship/push só por ordem explícita do dono** (§0.7).

## 1. A superfície de colisão (do `collision-surface.sh`, 2026-09-24)

```
merge-base 20a630f1b · 19 commit(s) · 69 arquivo(s)   (antes do commit deste handoff)
PROJECT_SCHEMA 160 (base: 160) · tripla (160, 13, 22) · VEC_SCENE_SCHEMA 22 (base 22)
FLIP_SCHEMA 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23            — todos = base
ph2d-ecs 103 · ph2d-render (espelho) 104 · ph2d-script (espelho) 104 — todos = base
crates/ph2d-nodegraph/src/node.rs intocado · crates/ph2d-editor-core/src/tool.rs intocado
ADR: esta linha não cria ADR
Cargo.lock: nenhum '+name' novo (só a aresta interna ph2d-vec-skin → serde, que já é do workspace)
marcadores de conflito: nenhum · tectos de LOC nos ficheiros tocados: nenhum passa
```

⇒ **Zero contador partilhado, zero contrato congelado, zero ADR, zero pacote externo.**

## 2. Foundational tocado — e porque é aditivo

| ficheiro | o que muda | porque é aditivo |
|---|---|---|
| `crates/ph2d-editor-core/tests/it/architecture_no_restricted_source_citations.rs` | **+5 entradas** na tabela `ALVO_PERMISSIVO`: os quatro ficheiros do `rive-runtime` (MIT, licença lida no `LICENSE` de raiz — o nível da prova está declarado no comentário) e o `oraculo_do_campo.py` (obra própria) | só apenda linhas a uma lista; nenhuma entrada existente muda |
| `shells/desktop/src/render_loop/fase_vector_bone_overlay.rs` | nova função `pincel_de_peso_a_vista` + tipo `PincelDePeso`: com o verbo `Weight` na mão desenha o retículo BBW posado | só corre com aquele verbo na mão (custo medido `0,587 ms` em release, 4 010 caminhos); fora dele o quadro é o de sempre |
| `shells/desktop/src/vec_bone_smoke.rs` | o roteiro da cena `PH2D_VEC_BONE_SMOKE` | texto de cena, `eprintln!` |
| `shells/desktop/tests/it/o_pincel_de_peso_esta_fiado.rs` | +37 linhas de gate | gate novo |
| `CLAUDE.md` | (a) promove uma flake ao §5.0 (`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`, `ph2d-tool-painter`); (b) reescreve **a MESMA linha física** do §5 Vector | ⚠️ ver §5.1 — é o único sítio com atrito textual previsível |

As crates do módulo (`ph2d-skeleton`, `-live`, `-render`, `ph2d-vec-skin`, `ph2d-app-vec`,
`ph2d-app-skeleton`) são **da linha**.

## 3. O que a linha faz — por wave, com os números MEDIDOS

| commit | o quê | número |
|---|---|---|
| `d6e67bd04` | **o `CampoDoDominio` sobrevive ao bind** (malha BBW + pesos + régua), gravado no `SkinnedPath`; a lei da curva consulta-o em vez da recta entre nós. Nos NÓS coincide ao bit com a tabela ⇒ não move âncora, só alças. `PH2D_SKIN_CAMPO=0` bissecta | arestas que cruzam a junta: erro `0,3709`/`0,3751` na lei antiga; as que não cruzam `0,0000` (o controlo). A malha passa a cobrir a arte: estrela `19,23 % → 0,00 %` de amostras sem malha |
| `9b7456a4e` | auditoria: a tabela publicada (`8,7 %`) vinha de uma fixtura DEGENERADA (`tendon: 0`) | honesto: `0,32 %` da peça |
| `b38599357` `543fbedda` `97e1327c9` `dd28096e5` | **o RETÍCULO do peso fica à vista** (report do dono: *«não deveria aparecer o lattice?»*); o peso lido NO VÉRTICE | dentro de um triângulo o peso saltava até `0,1626` |
| `defacb960` | o lattice endurece o canto — o dono tinha razão | `+43,5°` medidos |
| `c9ca1ec64` | `MisturaDoAngulo::Desdobrado` existe, **DESLIGADA** (`PH2D_SKIN_ANGULO=1`) | bico `93° → 121°`, mas PIOR acima de `108°`; a imagem delega na CPU quando ligada |
| `e63fa479a` | painel: o knob `Segments` sozinho é no-op ao bit — só gates | `155,3°`; com `Curve Handles` `61,7°` |
| `af973f2c5` `37e706344` | régua da ONDULAÇÃO; porte C¹ (side-vertex de Nielson; oráculo = SciPy BSD-3), **DESLIGADO** (`PH2D_SKIN_C1=1`) | no campo `68 → 22` ondas, no DESENHO `12 → 12` ⇒ não compensa ligar |
| `67bbbfee1` | `DIVISOES_POR_OSSO` `3 → 5` (joelho medido) | serpentina `28×` menor. ⚠️ **hoje DORMENTE** (ver `37b5e8715`) |
| `a986e098f` `316153806` | **a `reconcilia` SAI** — lida a lei do Rive (MIT): ela não ajusta nada; medido, a conciliação ERA a serpentina | serpentina `0,016016 → 0,001727` (`9,3×`); o ajuste livre = chão do modelo às 3 casas; recook `427,3 → 335,7 µs`. Preço: quebra de tangente num nó até `13,65°` |
| `1e702a4bd` | porque o `Effects: Arc` é perfeito com poucos pontos: ele ACRESCENTA nós no caminho derivado — só sondas | elipse 4 → 4 nós e `126×` mais fiel |
| `9e645c879` | **`IndiceDoCampo`**: a consulta ao campo era varredura linear | consulta `18,8×`; recook `336 → 105 µs`; desenho idêntico (`0e0` em 229 amostras) |
| `0e1e7311b` | `refit_pelo_bake` (a ideia do dono): assar e depois ajustar — **SEM chamador de produto** | ver §6 |
| `37b5e8715` | **o `bind` deixa de acrescentar pontos** (ordem do dono). Uma linha: `bind` → `bind_com(.., false)` | barra da cena `54 → 8` nós; erro contra a verdade `0,001176 → 0,216709` (**`184×`**); ponto novo numa forma presa volta a saltar `7,5639 %` (dívida nomeada com asserção) |
| `711d51335` | a pesquisa do estado da arte (doc [`04`](../04_pesquisa_ossos_sobre_desenho_vetorial.md)), interrompida por ordem do dono | — |

## 4. O formato gravado muda — DENTRO de bytes opacos, com leitura versionada

- O `SkinnedPath` (que viaja no `SkinBind::source`, bytes opacos) **apenda** `campo: Option<CampoDoDominio>`
  ⇒ **não mexe em `PROJECT_SCHEMA`** (confirmado pelo mapa do §1).
- ⛔ O postcard é posicional: a leitura é [`skinned_mesh::le`](../../../crates/ph2d-skeleton-live/src/skinned_mesh.rs)
  e tenta **o formato novo primeiro, depois `SkinnedPathV1`** (`campo: None`). A ordem é load-bearing
  (doc-comment do `SkinnedPathV1`).
- Um bind gravado pelo `main` abre com `campo: None` ⇒ a lei volta à recta entre nós **até re-prender**;
  nenhuma migração, e o desenho é o de sempre.
- ⚠️ A feature `serde` da `ph2d-vec-skin` tem de ser ligada por **quem grava** — a `ph2d-skeleton-live`
  liga-a no `Cargo.toml`. *A feature não viaja com o código* (HOWTO §2.4).
- ⚠️ Formas gravadas **entre 19 e 20/09** vêm já subdivididas e ficam assim — a linha não as desfaz.

## 5. O que uma leitura rápida do diff entende AO CONTRÁRIO

1. ⚠️ **O `CLAUDE.md` reescreve a MESMA linha física do §5 Vector** (o parágrafo longo). Com fast-forward
   não há conflito; se outra linha tocar esse parágrafo antes desta fundir, o conflito é textual e a
   resolução é **manter as duas redacções**, nunca escolher uma.
2. **Dois gates sobre a mesma propriedade com sentidos opostos — um em cada árvore:** o `main` tem
   `dobrar_a_barra_nao_crava_uma_quina_em_no_nenhum` (exige `quina < 1e-9`, a `reconcilia`); a linha
   **remove-o** e tem `o_ajuste_compra_fidelidade_e_paga_tangente` (exige `quina > 1e-6`). Depois do ff
   só fica o da linha — ⛔ **não o «restaure»**: a medição de `a986e098f` é a razão.
3. **`DIVISOES_POR_OSSO = 5` não é o produto:** o produto não subdivide; o `5` só é alcançável por
   `bind_com(.., true)` (contrafactual dos gates + formas gravadas entre 19 e 20/09). O censo
   `nenhum_caminho_de_produto_pede_a_subdivisao` exige **exactamente 1** chamada de produto, com `false`.
4. **`refit_pelo_bake` e `refit_pela_curva` estão no `src/` e NÃO são código morto a apagar:** são as
   rotas medidas (e a primeira é a decisão pendente do dono, §6). Só testes os chamam, de propósito.
5. **As três portas de ambiente:** `PH2D_SKIN_CAMPO` nasce **LIGADA** (`=0` desliga); `PH2D_SKIN_C1` e
   `PH2D_SKIN_ANGULO` nascem **DESLIGADAS** (`=1` liga). Nenhuma é knob de produto.
6. **A malha da IMAGEM presa NÃO saiu** com a ordem do dono — a ordem é sobre pontos de controlo de uma
   forma VETORIAL (autoria); a malha de uma imagem não é autoria de ninguém.
7. ⛔ **O `01_a_fila.md` prescrevia a `reconcilia` (`:448`) e a F32 «a subdivisão nasce no bind»** — os
   dois sítios levam agora uma nota `SUPERADO` a apontar para aqui. Não reconstrua nenhum dos dois.
8. ⚠️ **O §5 do `CLAUDE.md` dizia que o vinco do cotovelo é «a LEI QUE MISTURA OS OSSOS»** — corrigido
   nesta linha pela verificação de 2026-09-23: a mistura preserva `|p−J|` por construção; o vinco é uma
   **dobra do mapa**, `det J = |1 − θ̄′·r|`, produto da meia-espessura da arte pela derivada do ângulo
   misturado (doc [`04`](../04_pesquisa_ossos_sobre_desenho_vetorial.md) §1.3).

## 6. ABERTO — com dono

- ⭐⭐⭐ **DECISÃO DO DONO — ligar o bake no desenho.** A recusa de `0e1e7311b` foi medida contra a
  subdivisão que o commit seguinte retirou. Sobre os `8` nós de hoje (tabela no doc-comment de
  [`curva_segundo_corpo.rs`](../../../crates/ph2d-vec-skin/src/curva_segundo_corpo.rs)):
  lei de hoje `53 µs`, ouro p90 `0,22281` · bake a `64` amostras/seg `303 µs`, `0,00165` ⇒ **`~135×`
  mais fiel por `5,7×` o relógio**. Os nós extra ficam no desenho, nunca no documento.
- ⭐⭐ **Memoizar o `IndiceDoCampo` no bind** — hoje é reconstruído POR QUADRO
  ([`curva.rs:397`](../../../crates/ph2d-vec-skin/src/curva.rs), chamado de `skin_live.rs:393`; e
  `curva_segundo_corpo.rs:116,233`) sobre uma malha que não muda: `58,8` de `126,6 µs` do recook
  (`46,4 %`, medido a `load 93–98` ⇒ tecto; a razão entre colunas é robusta). A obra mais barata do corpus.
- ⛔ **As réguas medem a peça errada:** `46` de `53` chamadas de `b_palco(..)` na `ph2d-skeleton-live`
  pedem `b_palco(true)` (a barra de 54 nós que o produto já não faz); a porta honesta
  `barra_da_cena_do_produto()` tem **1** chamador. Os vereditos dessa família são sobre o contrafactual.
- ⛔ **Ninguém mediu o QUADRO** com N formas presas (`PH2D_FLUID_PROFILE=1` existe e nunca correu numa
  cena de esqueleto); todo orçamento publicado é soma de bancadas.
- ⏳ O salto de `7,5639 %` de um ponto novo numa forma presa (dívida com asserção).
- ⏳ **O SMOKE desta rodada inteira não foi feito pelo dono** (os reports que a guiaram foram a meio dela).

## 7. Prova de fecho

Ver o bloco **«Prova de fecho — números»** no fim deste ficheiro (escrito depois do portão, sobre a
árvore rebaseada).

## 8. Smoke — o que o dono vê

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

1. A cena abre com uma barra presa a três ossos. **Ela tem os pontos que foram desenhados** — prender
   não acrescenta nenhum. Errado: pontos coloridos a mais ao longo da barra.
2. Escolha o verbo **Weight** do esqueleto: aparece a **grelha** colorida por cima da barra (o campo do
   peso), e ela dobra com o osso. Errado: só pontos, sem grelha.
3. Rode o osso do meio a ~90°: a barra dobra; com poucos pontos a dobra forte perde fidelidade (é o
   preço medido da ordem de 20/09). Errado: a barra a serpentear ao longo do comprimento.

## Prova de fecho — números (2026-09-24, árvore rebaseada sobre `20a630f1b`)

| portão | resultado |
|---|---|
| `bash scripts/nextest-impacted.sh` (pela porta `ph2d-run.sh`) | **`15 691 / 15 691` verdes**, `13 095` fora do impacto · `load 52` antes, `91` depois |
| `bash scripts/censos-da-arvore-combinada.sh` | **`127 / 127` verdes** · controlo do filtro: `12` de `12` censos correram |
| `cargo clippy --all-targets -D warnings` nas 5 crates do módulo **e** na `ph2d-host-desktop` | zero avisos |
| `cargo fmt --all --check` | limpo |
| `bash scripts/doc-index.sh --check` | `20` índices em dia |
| os três arch-gates que leem o `CLAUDE.md` e os docs (`architecture_docs_paths_and_smokes_resolve` · `architecture_docs_reference_live_gates` · `architecture_no_restricted_source_citations`), re-corridos DEPOIS das edições de doc | `9 / 9` verdes |
| `collision-surface.sh` | §1 — zero contador, zero contrato, zero pacote externo |

⚠️ **Auditoria (≥2 lentes):** a do DELTA `main`↔linha (dois agentes independentes, 2026-09-23, corpus em
`~/Referencias/vector-bones-2026-09-23/journal.jsonl`, resultados `1` e `39`) e as `12` verificações
adversariais da pesquisa, que corrigiram três afirmações (a mais pesada está no §5.8). ⚠️ **A prova de
mutação desta rodada é a dos commits** (`37b5e8715`: `6/6` a sangrar, com um sobrevivente curado);
não foi re-corrida no fecho, porque o código não mudou desde ela — só docs.
