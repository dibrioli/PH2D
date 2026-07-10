# HANDOFF de integração — linha `line/Painter` (aquarela: junções + sessão molhada) — 2026-07-09

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter`.

## 1. Identidade

- **Branch:** `line/Painter` · **HEAD:** ver `git log -1` no worktree (último commit = fechamento:
  BUGS #8 + este handoff + remoção do diag TEMP).
- **Base do fork (merge-base com main):** `1e956ae6` (*ci(spike): install AVIF build tools*).
  O main avançou só `ec5a031e` (*docs(memory)* — só `project-memory/`) desde o fork ⇒ **zero
  interseção de arquivos** com o que a linha tocou.
- **Commits da linha:** 19 (18 de implementação + 1 de fechamento). Lineares, sem dependência de
  outra linha.
- **Gates no fechamento:** `cargo test -p ph2d-tool-painter --lib` = **501 passed / 0 failed /
  16 ignored** · `clippy --all-targets` = 0 warnings · LOC caps ok (render **699/700** — vide §6)
  · fmt rodado com o toolchain pinado do worktree.

## 2. Foundational/compartilhado tocado — **NENHUM**

Verificação (rode você mesmo):

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
git diff --name-only $(git merge-base main line/Painter)..line/Painter
```

**Todo o diff vive em `crates/ph2d-tool-painter/**` (o crate da própria linha) + `docs/Painter/*`
+ este handoff.** Nada em `shells/*`, `ph2d-editor*`, `ph2d-core`, `ph2d-painter-brush`, tokens,
ids, sync/codegen ou scripts. Durante a investigação houve tracers TEMP em
`trait_impls_raster.rs` e `layers/undo.rs` (do próprio crate) — **removidos no fechamento**;
esses 2 arquivos terminam byte-idênticos ao main (conferir: devem sumir do diff acima).

⇒ Integração esperada: `--ff-only` trivial (ou merge textual limpo), sem
`foundational-integrate.sh` de árvore combinada com outra linha por causa DESTA linha.

## 3. Símbolos que podem colidir com outra linha — **nenhum global**

- **Zero** `NodeId`/`IconId`/token/entrada em lista ordenada/chave de i18n novos.
- **Zero** dependência nova (nenhum `Cargo.toml` tocado).
- Símbolos novos são todos **internos ao crate** (`pub(super)` ou menor):
  `build_wet_field`/`sample_wet_field`/`WET_FIELD_BLUR_PX = 8` (`watercolor_rewet_px.rs`),
  helpers/consts de watercolor (`watercolor_field.rs`/`_render.rs`/`_mixer.rs`), testes novos em
  `paint/tests.rs`. Outra linha só colide se tiver editado **os mesmos arquivos** deste crate
  (mesmo-símbolo, DIRETRIZ §1.5.5) — grep sugerido:
  `git log main..line/<outra> --name-only | grep ph2d-tool-painter`.

## 4. Contratos congelados encostados — **nenhum**

`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` intocados (implementações internas do
crate mudaram; superfícies/assinaturas não). Nodes/Vector n/a.

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **typos:** docs novos e comentários extensos em **pt-BR** (BUGS #8, doc 12 takes 7-10, este
  handoff, comentários em `watercolor_*`). O allowlist pt-BR já existe (`23d43bdd`), mas palavra
  nova pode escapar — se o `typos` acusar, é allowlist, não conteúdo.
- **fmt skew:** o fork é pós-`style_edition 2024` e o fmt do fechamento usou o toolchain pinado —
  risco baixo; ainda assim o `ship.sh` confirma.
- **machete/deny/RUSTSEC:** sem deps novas; advisory-db local envelhece — RUSTSEC novo é risco
  genérico do dia do ship, não desta linha.
- **bindgen --check:** fora do escopo desta linha (nada de FFI tocado).

## 6. Ordem/dependências + o que smoke-testar + avisos operacionais

- **Sem ordem interna** (commits lineares) e **sem dependência de outra linha**.
- **Smoke:** o resultado final foi smoke-aprovado pelo Enio HOJE (2026-07-09, cruz com Charge<1,
  com e sem Rewet — "smoke OK! Está bom!"). Nada pendente de smoke nesta linha. Os smoke-gates
  EDGE-3/EDGE-4 citados em handoffs anteriores foram REVERTIDOS na árvore (não existem aqui;
  histórico no doc 12).
- **Testes `#[ignore]` intencionais (3):** `watercolor_app_params_incremental_matches_full_
  {diluted,mixer_on}` = repro executável de um resíduo Δ2 CONHECIDO de staleness incremental
  (sub-visível; tolerância do gate é ≤1) — viram gate quando o resíduo for atacado;
  `watercolor_app_params_diff_spatial_map` e `watercolor_junction_transition_profile` = diags
  exploratórios (`--ignored --nocapture`). **Não** os "conserte" nem os promova na integração.
- **LOC no teto:** `watercolor_render.rs` = **699/700**. Se o Mergiraf/merge inserir QUALQUER
  linha aí, o cap estoura — resolva extraindo pro sibling `watercolor_rewet_px.rs` (207/700,
  padrão já estabelecido), nunca por allowlist.
- **Cuidado operacional (lição da linha):** o cwd do Bash reseta pro repo MAIN a cada turno —
  toda mutação de arquivo por caminho absoluto do worktree
  ([[feedback_sed_relative_path_hits_primary_cwd]]).

## 7. O que a linha entrega (contexto de 1 parágrafo pro integrador)

Aquarela do Painter: (a) **fix das junções** — cruzamento de traços com Charge<1/Rewet clareava
com fronteira dura/serrilhada; agora o clareamento (look ratificado pelo Enio) tem transição
orgânica — lerp proporcional no blend do pigmento + campo de molhado mascarado por posse
(BUGS_painter.md **#8**, doc 12 takes 7-10); (b) **sessão molhada estável** — composite = função
pura do estado da sessão, janela de secagem 60s, água só interage com tinta seca (retângulo do
Dilution morto), params por-traço via tabela de estilos + mapa de dono; (c) **guards novos**:
`watercolor_junction_lightening_is_soft_and_preserved`,
`watercolor_wet_session_survives_charge_slider_change`, parity/byte-exact guards da sessão.

— *Linha `Painter` pronta. Aguardo ordem de integração (não integro nem pusho por conta própria).*
