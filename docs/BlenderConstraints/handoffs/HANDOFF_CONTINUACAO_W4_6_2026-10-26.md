# Continuação — `line/BlenderConstraints`, onda W4 (CAD no vetor), 26/10/2026 — W4.5 FEITA, abre a W4.6 (a tool «Rules»)

> Anterior: [`W4.5`](HANDOFF_CONTINUACAO_W4_5_2026-10-26.md). Doc da onda: [`04_W4_cad.md`](../04_W4_cad.md) — **§3.5** (o
> que a W4.5 fez, medido), **§3.6** (o veredito do dono no smoke e o desenho da W4.6), §4 (gates ✓). **Não duplicado aqui.**

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4`. Commits da
W4.5 (depois do handoff W4.5): dois WIP · `88dbe19673` sonda · `65f220af64` secção do Inspector · `7bc4cd616c` doc ·
`f0bf204615` chão preso · `66ef09238c` ordem da secção · `b8ca424171` unidades · o commit deste handoff.
`PROJECT_SCHEMA` = 195 (reconta-se na integração), `VEC_SCENE_SCHEMA_VERSION = 24` (`RegraCad.id` + `next_regra` no
MESMO degrau). Família de canal nova `CAD = 9` (`ph2d_ecs::canal_cad`).

## Feito na W4.5 (não refazer)

§3.5: `RegraId` durável (`assenta_regras`) · canal `CAD = 9` (não a 8: o `app-constraints` drena-a inteira) ·
`ph2d_vec_entities::medida` (`ConducaoCad`, resolve SEMPRE do autorado) · `ph2d_vec_constraints::violacao` ·
cena 6 `smoke_regua.rs` · secção `DRAWING RULES` (`cad_inspector.rs` + `ph2d-panel-inspector` `*_cad.rs`).

## Veredito do dono (smoke 26/10) — DECIDIDO, não perguntar

O motor passa; a superfície é recusada (§3.6, memória `project_drawing_rules_are_a_tool_with_explicit_gizmos_2026_10_26`):
**regras = TOOL NOVA na barra lateral esquerda, gizmos explícitos e bem desenhados como os das Shapes; nunca um painel
no Inspector.**

## Próximo passo — W4.6 (§3.6 itens 1–5)

1. **Ler antes de desenhar** (delegar ao `explorador`): como uma tool do Vetor entra na barra esquerda e como as Shapes
   desenham os gizmos e a barra flutuante (`ph2d-app-vec/src/vector_mode*.rs`, `quickshape_frame.rs`, o registo de
   tool/painel — gate de registo é `motivo:` válido). Escrever no §3.6 o desenho (onde mora, o que a shell paga — o
   tecto `the_shell_only_shrinks` tem ~1 linha de folga) ANTES do código.
2. Criar à mão: seleccionar 1–2 pontos/lados → só as regras possíveis; porta `junta_regra` (`ph2d-vec-edit/src/cad.rs`).
3. Gizmos: selos por regra, cota com número clicável (valor + chave do Auto Key pelo canal `CAD`), hover, escolher,
   Delete, cor da forma. Tokens/i18n (HR-15). Gate de seam no canvas = teste na crate (o testkit não conduz o canvas).
4. Tirar a lista do Inspector (ou reduzi-la a resumo) — o painel não é a superfície.
5. Cena de smoke nova, poucas regras visíveis (caixa com tampa por ângulo animado); fotografar ANTES (`import -window`,
   nunca `spectacle` em `kwin --virtual`); smoke ao dono em PT-BR, passos numerados, explicando o que cada coisa na
   tela É.

## Portões por nome (verde no fim da W4.5)

Os de §4 marcados W4.5, `the_shell_only_shrinks`, `workspace_src_files_under_loc_cap`, `no_dependency_climbs_a_layer`,
fiação de painel, i18n, hex, magic-numeric, `seam_cad::*`. ⚠️ Vermelhos HERDADOS (do integrador): ver handoff W4.5.

## Armadilhas

- `git commit -- <paths>` não leva ficheiro por rastrear: `git add` antes, `git show --stat HEAD` depois.
- `rustfmt --check` no ficheiro da shell antes de comitar; um teste por chamada no `cargo-test-narrow`.
- O hook recusa comandos cujo texto nomeia programas de placa (`kwin_wayland --virtual`): `ps -eo pid,comm | grep`.
- A entidade do path tem de existir antes de gravar chaves (o `step` da cena espera-a).
