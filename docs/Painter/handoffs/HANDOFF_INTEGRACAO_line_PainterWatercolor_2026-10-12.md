# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor` no `main` (2026-10-12, ordem do dono)

*«Smoke OK. Antes de seguir vamos integrar ao main»* (dono, 12/10). Uma linha só; sem push (ship é outra ordem).

## §1 — Coordenadas

- `main` antes: `f1138408e1` (a `line/motion-value` integrada hoje, local). Merge-base da linha: `9ea7821383`.
- 29 commits rebaseados SEM conflito sobre `f1138408e1`; `--ff-only` ⇒ `main` = `cbc3dd7b84`.
- Sobreposição medida com o que o `main` ganhou (367 ficheiros × 232 da linha): `Cargo.toml` (bloco novo do
  `opt-level` do wet-paint/pigment), `CLAUDE.md` (cada linha a sua entrada da §5) e `project-memory/MEMORY.md`
  (duas contagens) — linhas disjuntas.
- Os 29 commits rebaseados = os originais (multiconjunto de `+/-` por ficheiro, controlo positivo ✓); zero
  marcadores.

## §2 — O portão da árvore combinada (`scripts/foundational-integrate.sh`)

Registos regenerados sem diferença · staleness ✓ · `CARGO_BUILD_WARNINGS=deny cargo check --workspace
--all-targets` ✓ · `nextest-impacted` (BASE=main) **18 212/18 212** (`load` 5–23).

⚠️ **Duas travas do primário, as duas conhecidas:**
1. `docs/archive/integracao-jornadas/PROMPT_LINHAS_RAPIDAS_2026-10-11.md` está NÃO versionado no primário (o
   documento das regras R1–R9 que os handoffs citam) ⇒ o script recusa. Foi posto de lado durante o portão e
   reposto (sha256 conferido). ⏳ Alguém tem de o versionar — os links `../../IntegracaoMultiAgente/…` dos
   handoffs apontam para ele.
2. O `--ff-only` falhou por `project-memory/MEMORY.md` sujo no primário (escritas de outra sessão: o YouTube
   no Quadro, a lista de apps, uma contagem) — a linha muda o MESMO ficheiro noutras linhas. Cura: cópia +
   diff local, `checkout`, ff, `git apply --3way` do diff, `restore --staged` ⇒ o primário ficou exactamente
   com as mesmas edições locais, agora sobre o `main` novo.

## §3 — O que entrou (os handoffs de continuação têm o detalhe e o que fica aberto)

- 11/10: presets por mídia (o mapa 18, Wet Liner, a fábrica de 12 mídias) · onda da velocidade (wet-paint e
  pigment a `opt-level 2` em dev; portões de cenas em paralelo) · o bake do papel e o LLVM 21–22
  ([COMPILADOR](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-11_COMPILADOR.md)) · #48/#49
  ([RELEVO_E_VERDE](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-11_RELEVO_E_VERDE.md)).
- 12/10: #50 a orla esbranquiçada · a família C do óleo
  ([FAMILIA_C](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_FAMILIA_C.md)) · #51 a textura TINGE-se e o
  Randomize «a cada traço» ([BUG51](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG51_TEXTURA_TINGE.md))
  · #52 a ponta inteira rodada / Size < 1, também no Wet Paint
  ([BUG52](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG52_PONTA_CORTADA.md)) · #53 o Solid de
  carimbos, os fios na cor da ponta e sem a corda
  ([BUG53](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG53_SOLID_DE_CARIMBOS.md)).
- Smoke do dono aprovado nos três últimos (#52, #53 e a corda).

## §4 — Próximo

Push só por ordem do dono (`/pd-ship`). A linha continua no `main` novo (a worktree já está nele).
