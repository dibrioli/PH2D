# Handoff de INTEGRAÇÃO — `line/3DModeling`: O RENDER POR MALHA (2026-10-04)

> **Para o integrador** (DIRETRIZ §1.5.3–1.5.4). Branch `line/3DModeling`, HEAD = o commit deste handoff (o anterior `e31744d98`), merge-base
> **`1ad60a1ce`** (o `main` não andou durante a linha: `git rev-parse main` = merge-base). **111 commits.**
> ⛔ **DECISÃO DO DONO (04/10): o desenvolvimento dos módulos 3D (modeling) PARA.** Esta é a última entrega da
> linha; o que fica aberto (§8) não tem dono até ordem nova. Integrar continua a ser só por ordem dele.

## §1 — O que a linha entrega (o produto)

O modo **Render** do modelador deixou de ser o campo TRAÇADO e passou a ser um **desenhista de jogo por
malha** — [ADR-0176](../../architecture/decisions/0176-o-render-do-modelador-e-uma-malha-de-jogo.md): triângulos
numa passada, luz de custo fixo, nada acumulado entre quadros, limites do WebGL2 / GLES3 pedidos também no
desktop. O campo continua a fonte (a malha é extraída dele). Ondas, cada uma com o seu handoff de continuação:

| onda | handoff | smoke do dono |
|---|---|---|
| o desenhista por malha (cena 37) | [O_RENDER_POR_MALHA](HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md) || ver o handoff |
| brilho e estilo no desenhista | [O_BRILHO_E_O_ESTILO](HANDOFF_line_3DModeling_O_BRILHO_E_O_ESTILO_2026-10-02.md) || ver o handoff |
| o céu fotográfico (HDRI CC0 embutidos, cena 38) | [O_CEU_DE_VERDADE](HANDOFF_line_3DModeling_O_CEU_DE_VERDADE_2026-10-03.md) | ok 03/10 |
| o sol e a sombra (PCSS) | [O_SOL_E_A_SOMBRA](HANDOFF_line_3DModeling_O_SOL_E_A_SOMBRA_2026-10-03.md) | ok 03/10 |
| texturas triplanares (pacote CC0, schema `179`) | [AS_TEXTURAS](HANDOFF_line_3DModeling_AS_TEXTURAS_2026-10-03.md) | ok 03/10 |
| o contacto entre peças (cena 40) | [O_CONTACTO](HANDOFF_line_3DModeling_O_CONTACTO_2026-10-03.md) | ok 03/10 |
| o chão que tapa (cena 41) | [O_CHAO_QUE_TAPA](HANDOFF_line_3DModeling_O_CHAO_QUE_TAPA_2026-10-03.md) | ok 03/10 |
| **o render traçado SAI** (só o Matcap fica no traçado) | [O_RENDER_ANTIGO_SAI](HANDOFF_line_3DModeling_O_RENDER_ANTIGO_SAI_2026-10-03.md) | **pendente** |
| as capturas de reflexo (cena 42) | [AS_CAPTURAS_DE_REFLEXO](HANDOFF_line_3DModeling_AS_CAPTURAS_DE_REFLEXO_2026-10-04.md) | **pendente** |
| a junta dos reflexos (reports 3–4) | [A_JUNTA_DOS_REFLEXOS](HANDOFF_line_3DModeling_A_JUNTA_DOS_REFLEXOS_2026-10-04.md) | **pendente** |
| a base pousada (report 5) | [A_BASE_POUSADA](HANDOFF_line_3DModeling_A_BASE_POUSADA_2026-10-04.md) | **«não percebi nenhuma melhoria»** (§8) |

## §2 — Crates NOVAS (drop-crate, sem dependência externa nova)

`ph2d-mesh-forward` (o desenhista), `ph2d-sky` (céu fotográfico: atlas octaédrico, lei do sol; 8 EXR embutidos),
`ph2d-triplanar` (CPU + WGSL; texturas CC0 embutidas), `ph2d-contacto` (o céu que uma peça tapa às outras).
Todas `0.0.0`, só caminhos do workspace; o `Cargo.lock` muda só por elas.

## §3 — Foundational tocado (e por que é aditivo)

| crate / ficheiro | o quê | aditivo? |
|---|---|---|
| `ph2d-field-gpu` | **cortado**: o pintor do Render traçado (`paint*`, `ceu_tempo*`, `brilho.rs`, `sondas_na_placa.rs`, exemplos) — fica a marcha e o Matcap; `naga` sai das dev-deps, `ph2d-bloom`/`ph2d-style` das deps | remoção; único consumidor era `ph2d-app-field3d` (desta linha) |
| `ph2d-field-render` | **cortado**: sombreamento traçado da CPU (`shade_render*`, `shadow`, `occlusion`, `probes`, `bounce`, `ground_*`, `sss_shadow`, `banda`, `refine`) e os gates deles; ficam Matcap, marcha, cache de fitas, `soma_halo`, `PointLamp`/`Surfaces`, `curvatura` | remoção; o integrador confirma por `cargo check --workspace` que nenhuma outra linha importa os símbolos (§6) |
| `ph2d-field-eval` | `extract_parts`/`extract_planes`/`extract_band`/`par` (as peças que se tocam, extração por peça) | novo, append-only |
| `ph2d-field-ecs` | `FieldTexture` (componente REGISTADO; o registo do campo `8 → 9`), escrita de params | novo componente |
| `ph2d-field` | `TEXTURE_FIELDS`, `TEXTURE_FROM_FILE`, `TEXTURE_SOURCES` | consts novas |
| `ph2d-material` | `por_pixel.rs` (a lei por pixel em WGSL), `prefilter` | novo + ajuste interno |
| `ph2d-bloom` | `wgsl::COMPOE` (a composição do halo, uma porta para os dois desenhistas) | const nova |
| `ph2d-component-desc` | `ph2d::field::FieldTexture` no catálogo `Model3D` | +1 entrada de catálogo |
| `ph2d-i18n` | `model3d_sky.rs`, `model3d_texture.rs` + chaves em `app_field3d`, `component_catalog`, `model3d_inert` | chaves novas |
| `ph2d-panel-model3d` | fileiras de céu e textura no Render | aditivo |
| `ph2d-vector` | `scene.rs`: método `rgba()` (leitura dos bytes) | aditivo; fora do contrato congelado |
| `Cargo.toml` (raiz) | `[profile.dev.package.ph2d-sky]` e `exr` a `opt-level = 2` (`11,0 s → 2,9 s` na suíte, medido) | perfil, não código |
| `shells/desktop` | **`+14 / −2`**: `PROJECT_SCHEMA 178 → 179` (+ teste) e `1` linha em `fase_field3d_requests.rs` | ver §4 |

## §4 — Números que SOMAM entre linhas (recontar, nunca escolher)

- **`PROJECT_SCHEMA`: `178 → 179`** (`FieldTexture` registado, sem degrau de migração). Se outra linha também
  subiu, reconta-se: `python3 scripts/schema-recount.py`.
- **ADR `0176`** (novo) — se outra linha usou `0176`, renumera-se na integração.
- **Registo de componentes**: `+1` (`FieldTexture`, catálogo `Model3D`); registo do campo `8 → 9`.
- **Cenas de smoke do modelador**: `37, 38, 40, 41, 42` novas; a próxima é a **43**.
- Shell: `+14 / −2` linhas (o tecto dela soma entre linhas).

## §5 — Contratos congelados: NENHUM tocado

`git diff 1ad60a1ce..HEAD` sem ocorrências em `NodeOp`, `OpResolver`, `NodeManifest`, `Tool`, `RasterEditTool`,
`CanvasPaintTool`, `PanelEvent`, `VectorOp`/`Vertex`/`Segment`/`Region` (o `ph2d-vector` só ganhou o leitor
`rgba()`); os gates `architecture_*contract_surface` estão na varredura do §6.

## §6 — Gate batched (1× sobre o diff acumulado)

Agente `verificador`, 04/10, load `27–42` (outra linha na máquina), merge-base `1ad60a1ce`:

| passo | resultado |
|---|---|
| `BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` | **PASS** — `19 369` testes, `19 369` verdes (`11 374` saltados) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | **PASS** |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` (as flags do `ship.sh`) | **PASS** |
| `file_loc_caps` (4/4), `arch_safe_clamp_only`, `architecture_the_shell_only_shrinks`, `architecture_workspace_file_loc_cap`, `architecture_no_orphan_source_file` | **PASS** (dentro do `it` da shell, na varredura impactada) |
| `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` | **PASS** |
| `cargo fmt --all --check` | FAIL → **corrigido** neste commit (`ph2d-panel-model3d/tests/it/seam.rs`, 2 blocos) → PASS |
| suíte com placa `ph2d-mesh-forward` (`--ignored`, sem sondas/instrumentos) | **28/28** (04/10, antes do fecho) |

Auditoria em duas lentes: (1) correção — cada cura tem régua contra o Cycles e prova de mutação no handoff da
onda (a última: 3 de 4 vermelhas, a 4.ª era redundante e saiu); (2) costura — o produto não chega ao traçado
(gate red-first do O_RENDER_ANTIGO_SAI) e nenhum consumidor fora da linha usa os símbolos cortados (o `check`
do workspace passa).

## §7 — Smokes (comando exato)

Da raiz da worktree (ou do `main` depois de integrar):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=42 cargo run -p ph2d-host-desktop --profile smoke
```

**MODEL** → **Shading** → **Render**. Trocar o `42` por `37` (o render por malha), `38` (o céu), `40` (o
contacto), `41` (o chão que tapa). O que cada uma tem de mostrar e como saber que deu errado: o §Smoke do
handoff de cada onda (§1). O binário `--profile smoke` foi construído na worktree (2.ª corrida `0,31 s`, zero
`Compiling`).

## §8 — ABERTO (sem dono depois da decisão de 04/10)

- **Report 6 do dono (04/10, sem resposta)**: na cena 42 ampliada, a borda do reflexo da bola amarela e o
  contorno do reflexo da bola vermelha no cromo saem serrilhados — *«não percebi nenhuma melhoria»*. Pose
  reproduzida na sonda (`PH2D_SONDA_YAW=2.9 PH2D_SONDA_ALVO=0.03,0.33,-0.28 PH2D_SONDA_MEIA=0.08`), causa NÃO
  medida. Medido antes: a ondulação fina de um texel do octaedro (`LADO 512`) na ampliação extrema; faces `512`
  não a curam ([A_BASE_POUSADA](HANDOFF_line_3DModeling_A_BASE_POUSADA_2026-10-04.md) §5, §7).
- Os smokes **pendentes** do §1 (o render antigo sai; capturas; junta; base).
- Custo da última cura não medido no relógio (máquina a load `28–40`); por construção, nenhuma leitura a mais.
- Herdados: o §7 do A_BASE_POUSADA e o §6 do A_JUNTA (a luz entre peças, o limbo, os dentes da extração na
  quina côncava, o escondido do centro → 2.ª camada da captura).

## §9 — O que só o `ship.sh` apanha / avisos ao integrador

- Fixturas novas grandes (oráculos do Cycles, só leitura de testes `#[ignore]`): `oraculo_reflexo_vizinhas.csv`
  `13,9 MB`, mais `~14 MB` entre `ph2d-mesh-forward`, `ph2d-contacto`, `ph2d-sky`, `ph2d-triplanar`; texturas e céus
  CC0 embutidos `~2 MB` (licenças no handoff AS_TEXTURAS / O_CEU_DE_VERDADE).
- Os testes de placa da `ph2d-mesh-forward` são `#[ignore]` (`28/28` verdes com `PH2D_GPU=1`, 04/10). O caminho
  GLES/WebGL2 **não se verifica nesta máquina** (sem GL): `cabe_no_gles` / `as_capturas_cabem_no_gles` saltam aqui.
- O grupo `0` do desenhista usa `15` das `16` texturas por estágio do WebGL2.

## §10 — Perfil do loop do agente

Ver o §10 do [A_BASE_POUSADA](HANDOFF_line_3DModeling_A_BASE_POUSADA_2026-10-04.md) (paralelismo `1,13`,
`test:check` `2,8×`, `34 %` das edições pela ferramenta, `491 mil` de contexto por passo).
