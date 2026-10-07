# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, «o papel é do documento e a aguada é um filtro» (2026-10-05 → 2026-10-07)

> **Este documento é para o AGENTE INTEGRADOR** (que só funde por ordem explícita do dono). Diz o que a
> linha toca, onde um merge pode doer, a prova de fecho e os smokes. Cobre **todos** os commits desde o
> merge-base `a46c4c200` — nenhum foi integrado. O *mecanismo* de cada passo vive nos docs:
>
> * os **BUGS** [#35–#46](../BUGS_painter.md) (uma linha por defeito, com mecanismo, medida e mutações);
> * o doc [47](../47_o_papel_no_rebelle_estado_da_arte.md) (o papel nunca é tinta — o Rebelle medido) e o
>   doc [48](../48_a_aguada_e_um_filtro_o_estado_da_arte.md) (a aguada é um filtro, Kubelka–Munk);
> * o plano [46](../46_plano_cada_controlo_em_cada_meio.md) (itens 8–9, Wet Paint);
> * o handoff anterior desta linha, [04/10](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-04.md)
>   (integrado no `main` em `da62cfdfd`).

---

## §1 — Coordenadas

| | |
|---|---|
| ramo | `line/PainterWatercolor` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` |
| base | merge-base **`a46c4c200`** (o `main` de então); HEAD no fecho `2283a1b87` (+ o commit deste handoff) |
| `main` hoje | andou **10 commits** desde o fork (integração da `line/UIUX` de 07/10: `331699d48` … `1ca435ca5`) ⇒ **há rebase** |
| commits | **31**, TODOS por integrar (`git cherry main HEAD` = 31 `+`) — `git log --oneline $(git merge-base main HEAD)..HEAD` |
| diff | `127` ficheiros, `+10 911 / −813` (antes deste handoff) |
| crates novas · pacotes EXTERNOS novos · ADR novo | **nenhum · nenhum · nenhum** (`Cargo.lock` intocado pela linha) |
| última integração desta linha | a rodada de 04/10 |

**O que a linha fez, por assunto** (cada linha = uma entrada dos BUGS; o resto está lá):

| período | commits (assunto) | o quê |
|---|---|---|
| 05/10 — aquarela e Wet Paint | `12fc8a036` · `c4c997876` · `3ff4f3600` · `88d964ed5` · `1eb219df8` · `206b19dfe` | **#35** a mancha do Solid na aguada deixa de piscar (o composite corre depois de o evento fechar) · **#36** a mancha refaz-se só onde mudou (21 → 5,8 ms/quadro num laço de 1000 px) · **#37** `then_some(Region)` curado em 5 sítios · **#38** o composite é o mesmo em qualquer janela (soma na TELA, exacta em ponto fixo) · Solid e fios na água do Wet Paint (doc 46 item 9; o Composite Brush recusado, item 8) · o censo: a procura completa cabe |
| 06/10 — o papel | `690bc83f4` … `0a66ce2e6` | **#39** a cor do PAPEL é do documento e vale nos quatro meios (`PainterTool::papel`) · **#40/#41** o miolo do Solid obedece ao tecto do traço; a Strength entra UMA vez (era ao quadrado) · **#42** a orla de um traço pintado antes do papel · **#43** a GPU compõe sobre o papel (WGSL `sobre_o_papel`) |
| 06/10 — Solid com papel | `6e49cfa69` · `61b1e3163` · `08ba3aea6` | **#44** o `over` da mancha num destino translúcido + os buracos do rabisco (`solid_cercado.rs`, nos quatro meios) |
| 06–07/10 — o papel nunca é tinta | `a212da7f6` · `77a9d25f2` · `b277d4a6f` | **#45** (doc 47) uma sprite branca nasce papel branco + camada vazia; a aguada deixa de ter o papel no chão |
| 07/10 — a aguada é um filtro | `d39bf7af5` · `2f3f27645` · `c917fb752` · `89c32a9d9` · `c5da1ee9b` · `80bb45119` · `8dae0106a` · `3b6267a9b` · `2283a1b87` | **#46** (doc 48) o plano `vidros`, a aguada que o escreve, o contorno claro (2.ª volta), a auditoria do fecho |

---

## §2 — Superfície de colisão (colada do `collision-surface.sh`, 2026-10-07)

⚠️ **Tirada ANTES do último commit `2283a1b87`**, que colapsou os degraus do schema da linha: a tripla e
o degrau abaixo (`185`) já não valem — a linha tem **UM** degrau, `183 → 184` (papel + vidro no
`PaintedDocument`), tripla `(184, 13, 22)`.

```
SUPERFÍCIE DE COLISÃO — line/PainterWatercolor contra main
  merge-base a46c4c200   ·   30 commit(s)   ·   127 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        185   (base: 183)      ← antes do colapso; hoje 184
  ⚠   └ tripla do gate               (185, 13, 22)   (base: (183, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ✗ FIELD_DOC_VERSION                     —   (base: —)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS

▸ REGISTRO DE COMPONENTES
    ph2d-ecs 106 · ph2d-render (espelho) 107 · ph2d-script (espelho) 107   (iguais à base)

▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — último no disco 0179 · próximo livre 0180 · a linha não cria ADR
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
  ✗ SONDA CEGA em: FIELD_DOC_VERSION
```

- ⚠️ **O `main` TAMBÉM foi a `PROJECT_SCHEMA = 184`** (degrau próprio, da integração UIUX/Vec). Dois
  degraus no mesmo número fundem LIMPO e evaporam um: o degrau da linha **reconta-se para `184 → 185`**,
  dentro do conflito — `python3 scripts/schema-recount.py 185` (três sítios: o doc da escada e a const
  em `shells/desktop/src/project_schema.rs`; a tripla em `project_schema_tests.rs`).
- **Ficheiros tocados pelos DOIS lados desde o merge-base: só `shells/desktop/src/project_schema.rs` e
  `project_schema_tests.rs`.** Nenhum outro conflito esperado.
- A sonda `FIELD_DOC_VERSION` lê «—» (cega): o `main` já a removeu (`69b28daa5`) — não é colisão.
- Contratos congelados (`node.rs`, `tool.rs`) intocados; nenhum ADR, nenhum pacote externo, nenhum
  marcador de conflito, nenhum tecto de LOC passado.

---

## §3 — O que está fora da pasta do módulo (e porquê) — tudo ADITIVO

| onde | o quê | porquê |
|---|---|---|
| `crates/ph2d-tool-painter` — `compositor` | `composite_into` ganha o argumento opcional `vidro: Option<(&Vidros, &mut [[f32;3]])>` (`None` em todo chamador antigo ⇒ byte-idêntico; a função é privada) · módulo novo `compositor::vidro` (`Vidro = [u8; 7]`, `Vidros`, `sela`, `alfas`, `cobre`, `composite_region_sobre_o_papel` `pub(crate)`) | o papel de cor atravessa cada canal da aguada (BUGS #46, doc 48 §4) |
| `crates/ph2d-tool-painter` — desfazer | `PlaneDeltas` ganha o **20.º plano** `vidros: StoredMap<Vidro>` (`split`/`side`/`confine`/`heap_bytes`/`divergences`; o round-trip cobre-o) · `undo_planes.rs` → os instrumentos de inspeção vão para o filho `undo_planes_inspect.rs` (**mudança pura**, 697 → 488 linhas, para caber no tecto de LOC) · `undo_window::WriteWindow::declarada()` (`const fn` nova) | o vidro viaja com o desfazer |
| `crates/ph2d-tool-painter` — documento | `ModelSnapshot::{papel, vidros}` · `StashedDoc::vidros` · `PaintedDocument::{papel, vidros}` (posicionais, **no fim**; as migrações congeladas lêem `PaintedDocumentSemPapel` e dão papel/vidros vazios) | #39 / #46 |
| `crates/ph2d-app-painter` | `gpu_eligible` recusa (a CPU produz) quando `papel_atravessa_vidro()` (papel de cor + um plano de vidro) — o mesmo padrão da sobreposição de proteção; o gate de paridade da GPU segue verde | a GPU não conhece o vidro |
| `crates/ph2d-painter-brush` | `solid_cercado.rs` (BUGS #44) + os avisos do clippy do #44 | o que o gesto cerca |
| `crates/ph2d-render` | WGSL `sobre_o_papel` nas três entradas que gravam (`cs_flat`, `cs_grouped`, `cs_encode`); `GpuGlobals` 32 → 48 B, `EncodeGlobals` 16 → 32 B | BUGS #43 (início da linha — detalhe lá) |
| `shells/desktop` | **só** o degrau do schema (§2) | — |

**Lista/catraca BAIXADA:** nenhuma. **Item partilhado cujos usos se apagaram:** nenhum.

---

## §4 — Contratos congelados e schemas

- **Contratos congelados (`CLAUDE.md` §6): nenhum encostado** (`collision-surface`: `node.rs`/`tool.rs`
  intocados). `Tool=12` · `RasterEditTool=5` · `CanvasPaintTool=1` · `PanelEvent=4` inalterados.
- **`PROJECT_SCHEMA`: UM degrau da linha, `183 → 184`** (`PaintedDocument::papel` + `::vidros`, ambos
  acrescentados no fim). Era dois (`183 → 184` no #39 e `184 → 185` no #46); a auditoria colapsou-os
  (`2283a1b87`). **Reconta-se contra o `main` (§2):** `184 → 185`. Os outros schemas: iguais à base.

---

## §5 — O que só o `ship.sh` pega — corrido no fecho (07/10)

Corridos 1× sobre o diff acumulado:

| régua | resultado |
|---|---|
| `nextest-impacted` | **17 206 / 17 206** verde |
| clippy `--all-targets -D warnings` | limpo em `ph2d-painter-brush`, `ph2d-tool-painter`, `ph2d-app-painter`, `ph2d-host-desktop`, `ph2d-i18n`, `ph2d-panel-painter-layers`, `ph2d-render`, `ph2d-wet-paint`; e `--all-features` em `tool-painter` (`gpu`) e `app-painter` (`panel-painter-layers`) |
| `cargo fmt` | limpo |
| `file_loc_caps::shell_files_respect_hr18_loc_cap` · `architecture_workspace_file_loc_cap` · `arch_safe_clamp_only` | verdes |
| filtros `architecture_` | editor-core `104` passed · shell `4/4` |
| `censos-da-arvore-combinada.sh` | **114 / 114** (árvore combinada) |
| GPU, `PH2D_GPU=1` | `the_gpu_producer_shows_what_the_cpu_producer_shows` e `ph2d-render o_papel_compoe_se_sob_as_tres_saidas` verdes |

**Depois das correções da auditoria (`2283a1b87`), nova corrida:** suíte do painter **1 523 passed** +
2 flakes de carga catalogados (`the_mask_stroke_cost_does_not_follow_the_canvas`,
`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`, ambos em
[`FLAKES_DE_CARGA.md`](../../DevOps/FLAKES_DE_CARGA.md); sozinhos, `3/3` verdes a `load 6,4`) ·
`ph2d-app-painter` **60** · testes de projecto da shell **96** · gates de LOC/arquitectura verdes ·
clippy e fmt limpos.

---

## §6 — Prova de fecho

- O binário de smoke compilado duas vezes (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop
  --profile smoke`), a 2.ª corrida sem um único `Compiling`:

O binário do smoke, compilado 2× depois do último commit de código (`2283a1b87`); a 2.ª saída:

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
▸ linha line_painterwatercolor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.20s
```

Zero «Compiling»: o Enio não espera build. ⚠️ Qualquer edição de código depois disto invalida-o em silêncio.

**Provas de mutação desta sessão** (o detalhe no corpo dos commits e nos BUGS):

| prova | resultado |
|---|---|
| #45 | **7/7** sangram |
| #46 (1.ª volta) | **13/13** — a V5 só sangrou depois de um gate novo (a base do vidro fora da mancha) |
| #46, família do contorno (X) | **9/9** — a X6 só sangrou depois de o oráculo da camada própria deixar de escrever com a lei sob teste |
| auditoria (Y) | **4/5** sangram, **1 declarada** (R2, abaixo) |

⚠️ **Lição da família X:** um oráculo que escreve com a MESMA lei que testa é cego (a mutação desloca os
dois lados por igual) — porta contra porta, nunca porta contra si própria.

**Auditoria (DIRETIVA §3), duas lentes — correção/costura e determinismo/perf — e a disposição:**

| achado | disposição |
|---|---|
| **F1** — um ajuste espacial (desfoque) por cima da aguada escalava a TRANSPARÊNCIA do papel acima de 1 | **curado**: escala a ABSORÇÃO (`um_desfoque_por_cima_da_aguada_nao_leva_o_papel_acima_de_um`, raio 3 — o de fábrica, raio 0, não o reproduzia) |
| **R2** — base congelada do vidro por meio (com a camada na chave) | feito; a mutação «um lugar só» **sobrevive por ser INALCANÇÁVEL** — medido: mudar de meio TERMINA a sessão do Wet Paint, aguada e fluido nunca alternam quadros |
| **R3** — alpha-lock (e o `keep` da selecção) selam com os alfas da base/misturados, na aguada e no Wet Paint | **curado** (`a_selecao_tambem_guarda_o_vidro`, `com_o_alfa_preso_a_aguada_deixa_o_vidro`, `com_o_alfa_preso_o_wet_paint_deixa_o_vidro`) |
| **R4** — dois degraus do schema | **colapsados em um** (§4) |
| **N3 · N4** | curados |
| **R5 · N1 · N2** + a borracha sobre a orla de alfa 0 | **ABERTOS** — §9 |

---

## §7 — Smokes

**Aprovados pelo dono em 07/10** (*«OK»*), no binário desta árvore:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop
```

1. **O papel nasce.** `Ctrl+N` → fundo **White** → **Create** → módulo **Painter**. Pinte com Digital
   (Strength 0,5), Watercolor, Impasto e Wet Paint. Na secção **Paper** escolha **Color** castanho-escuro.
   Tem de acontecer: *todo traço escurece junto com o papel*, e **não** aparecem pontos brancos em volta
   do Wet Paint. Voltando ao branco, tudo volta ao que era. Deu errado se um traço fica claro sobre o
   papel escuro.
2. **A aguada é um filtro.** Watercolor, vermelho puro, sobre o castanho: sai um vermelho-acastanhado
   escuro, **nunca mais claro que o papel**. Com **Body 0** é o filtro puro. Deu errado se parecer
   fluorescente (o antigo `220,97,86`).
3. **O contorno claro.** Uma mancha de Watercolor vermelha, seca; depois Wet Paint, Digital e Impasto a
   cruzá-la sobre papel escuro: **sem contorno claro** em volta. Com papel branco, igual a antes.

**Os reports anteriores do dono que abriram os itens de 05–06/10** (registados nas linhas dos BUGS): #35
*«o preenchimento pisca»*; #36 a lentidão numa mancha de 1000 px; #39 *«a cor do papel funciona mal»*
(e o botão que saiu: o seletor aplica); #40 o miolo do Solid mais escuro (Digital/Impasto); #44 duas
fotos — Solid com papel escuro e os buracos do rabisco. ⚠️ Esta sessão **não** tem o registo de cada
uma dessas curas ter sido aprovada no smoke; o que tem «OK» explícito do dono em 07/10 são os três
smokes acima (#45, #46 e o contorno). Pergunte ao dono antes de dar os anteriores por aprovados (§9).

---

## §8 — O que uma leitura rápida do diff entende ao contrário

1. **Os pixels da camada NÃO mudaram de lei** — continuam a ser a aparência sobre o BRANCO. O filtro do
   papel vive só no plano lateral `vidros` + no `t` por canal do compositor. Sem papel de cor, ou sem
   vidro, **tudo é byte-idêntico** ao de antes (`None` em todo chamador antigo).
2. **O selo não é uma cache.** Um selo partido CONTINUA o vidro (inferência de cobrir/apagar), não volta
   à lei de um alfa — voltar É o defeito do contorno claro (BUGS #46, 2.ª volta).
3. **`abre_a_sprite` (a porta de `bind_document`) converte uma sprite toda branca em papel branco +
   camada transparente; o `set_source` cru NÃO converte** (é a porta da arte; mover a regra para lá
   moveu 181 testes do motor — recusado, medido).
4. **O chão óptico da aguada é o branco de referência com QUALQUER papel** (`cor_do_chao`): o papel já
   não entra na tinta.
5. **O Wet Paint só escreve o vidro numa camada que já o tem** («o plano não nasce no Wet Paint»).
6. **A GPU recusa-se com papel de cor + vidro** (a CPU produz) — não é regressão, é a regra de
   elegibilidade (padrão da proteção); com papel sem vidro a GPU compõe (#43).
7. **`undo_planes_inspect.rs` é mudança pura** — o diff mostra 209 linhas movidas, nenhuma alterada.

---

## §9 — Aberto

Do BUGS #46 (cada um medido lá):

- **Deformar / transformar / liquefazer não movem o vidro**: o selo parte e a leitura infere (exacto
  onde o alfa de baixo é `< 255`).
- **A borracha sobre a orla invisível da aguada** (alfa 0) não a apaga do vidro — `≤ 4` níveis sobre o
  papel de cor; no branco, invisível.
- **O escorrido do Wet Paint sobre uma camada com vidro** entra no `split` do desfazer sem janela
  (compara o plano inteiro no pen-down seguinte).
- **O caminho defensivo sem `wet_backdrop`** reconstrói o chão preto a cada quadro.
- **Custo no pen-down: `+6–8` ms numa camada com vidro** (a cópia do plano partilhado com o desfazer;
  um diário de ladrilhos eliminá-lo-ia).
- **Molhado sobre molhado e o `Pigment`** decidem o peso da mistura sobre o branco de referência:
  `8` níveis contra o oráculo do chão preto.
- **Com Body 0 só um pigmento de canais 0/255 é filtro puro** (a óptica devolve `pigmento·(1 − T)`).
- **Watercolor e Wet Paint ignoram a Strength** (anterior à linha, registado no #41).

Fora de ciclo, da abertura da linha:

- **Velocidade do Solid na Aguada** — `diag_a_mancha_de_1000px_na_aguada` (#36) e
  `diag_o_preco_do_cercado` (`+0,15–0,3` ms por evento, #44).
- **O vazio do rabisco simétrico** (129 px, #44).

---

## §10 — Para o integrador

1. **Ordem:** rebase da linha sobre o `main` → no conflito de `shells/desktop/src/project_schema.rs` e
   `project_schema_tests.rs`, **reconte o degrau** (`python3 scripts/schema-recount.py 185`; escada, const
   e tripla — os três sítios, §2) → corra
   `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor/scripts/collision-surface.sh`
   pelo caminho ABSOLUTO (a coluna `base:` é o merge-base; leia o `main` no ficheiro) →
   `foundational-integrate.sh` → `censos-da-arvore-combinada.sh` na árvore combinada.
2. **`Cargo.lock`:** a linha não o altera — qualquer diferença vem do `main`.
3. **`CLAUDE.md` §5.1, Painter:** esta linha trocou o link do último handoff para este e a frase do que o
   módulo é (papel do documento + aguada como filtro). Se o `main` mexeu na entrada, junte à mão, ≤ 700
   bytes.
4. **Flakes:** nenhuma nova a promover; os dois do §5 já estão catalogados.
5. **Memória:** a linha não escreveu em `project-memory/`. ⚠️ O checkout da RAIZ tem ficheiros por
   comitar em `project-memory/` (`MEMORY.md` e quatro `feedback_*`/`reference_*`/`project_*` novos) que
   **não são desta linha** — não os arraste.
6. **Smoke é do dono:** integrar não é aprovar (§7 diz o que ele aprovou nesta árvore; depois da fusão
   é o `main` que se smoka).

### O perfil do agente (`agent-loop-profile.sh`, 20 sessões, no fecho de 07/10)

```
✗ paralelismo de ferramenta              1.11/passo   alvo: >= 1,5  (7% dos passos com 2+ chamadas)
✓ respostas por sessao (mediana)                262   alvo: <= 800
✗ cargo test : cargo check                690 : 210   alvo: <= 1,0  razao 3.3x (baseline: 4,3x)
✗ edicoes pela ferramenta Edit                  32%   alvo: >= 80%
✗ contexto relido por passo (media)         514 mil   alvo: <= 250 mil
✓ contexto no inicio da sessao               61 mil   alvo: <= 80 mil
```

## §11 — A integração (2026-10-07, ordem do dono: *«integre sua linha ao main… confira se Motion nodes já entrou»*)

- **Pré-condição:** a `line/motion-value` já estava no `main` (`211a4b541` = tip dela). Única linha
  desta rodada ⇒ a ordem não se mede.
- **Rebase** em `integ/PainterWatercolor-2026-10-07` sobre `main@211a4b541`, com `merge=text` nos
  ficheiros da escada, da tripla, do `project_migrate.rs` e do `CLAUDE.md` enquanto durou (tirado no
  fim). Sobreposição textual: só esses quatro ficheiros; o `project_migrate.rs` fundiu limpo.
- ⚠️ **O `main` já estava em `185`, não em `184`** (o `184` do objecto vetorial e o `185` dos quadros da
  `line/MiroClone`) — o §2/§10 acima envelheceu. Três conflitos de schema, resolvidos por estágio:
  - commit do papel: `python3 scripts/schema-recount.py 186` (`183→184` ⇒ `185→186`);
  - commit `vidro (1/3)`: `schema-recount.py 187` (`184→185` ⇒ `186→187`);
  - commit do colapso (`fecho(auditoria)`): escada e tripla do `main` + **um** degrau `185 → 186`
    (papel + vidro), escrito sobre `git show main:` — o resultado final é o que a linha queria, um
    número acima.
  O «um vNNN é recusado» da prosa foi corrigido à mão nos três (o script não o toca).
- **`CLAUDE.md` §5.1:** a entrada do Painter vem da linha; a do Vector, do `main` («cada forma é um
  objecto»).
- **Prova do rebase** (`cmp.sh`, multiconjunto `+/-` por ficheiro, controlo positivo = pares
  trocados diferem): **29 de 32** commits iguais ao original; os 3 diferentes diferem SÓ nas linhas
  do degrau recontado.
- **Árvore combinada:** clippy `-D warnings --all-targets` nas 7 crates da linha + `ph2d-host-desktop`
  limpo; `cargo fmt --check` limpo; depois `foundational-integrate.sh` (check do workspace com
  avisos negados + `nextest-impacted` contra o `main`) antes do `--ff-only`.
