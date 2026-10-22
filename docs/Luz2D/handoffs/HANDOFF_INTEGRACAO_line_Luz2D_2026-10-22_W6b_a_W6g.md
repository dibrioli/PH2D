# Handoff de integração — `line/Luz2D` W6b…W6g para o `main` (fecho 2026-10-22)

> DIRETRIZ §1.5.9. A linha FECHOU e PAROU: não integrou nem fez push. O detalhe de cada onda está nos handoffs de
> continuação ([W6b](HANDOFF_CONTINUACAO_W6b_2026-10-17_A_VISTA_RENASCE.md) ·
> [W6c](HANDOFF_CONTINUACAO_W6c_2026-10-17_O_PISCAR_DO_RENASCIMENTO.md) ·
> [W6d](HANDOFF_CONTINUACAO_W6d_2026-10-18_O_DISCO_EM_ZOOM.md) + [fecho](HANDOFF_CONTINUACAO_W6d_FECHO_2026-10-18_A_BORDA_E_O_CANTO.md) ·
> [W6e](HANDOFF_CONTINUACAO_W6e_2026-10-18_O_CANTO.md) · [W6f](HANDOFF_CONTINUACAO_W6f_2026-10-18_A_MISTURA_DAS_FACES.md) ·
> [W6g §1–§56](HANDOFF_CONTINUACAO_W6g_2026-10-19_O_CAMPO_QUE_GUARDA_O_VERTICE.md)).

## 1. Identidade

- Branch `line/Luz2D`, worktree `Worktrees/line-Luz2D/`, HEAD = o commit deste handoff (o último de código: `a024c29d77`).
- Base do fork (merge-base): `ad5f8fe39c` (W5x–W6a integradas em 17/10). **78 commits**, 76 ficheiros. O `main` andou 3
  commits desde a base (`451c3ad14a`, `bdc7a38639`, `100d34c8e0`: processo, a vez da placa — fora das crates da luz).
- Sem rebase (é do integrador).

## 2. Fora das crates da luz (tudo ADITIVO)

| Ficheiro | O quê | Porquê aditivo |
|---|---|---|
| `crates/ph2d-editor-core/src/luz_edits.rs` | variante nova `AvisoDaLuz::CantosArredondados { blocos: u32 }` | só acrescenta; `git grep AvisoDaLuz` não acha outro `match` exaustivo no workspace |
| `crates/ph2d-i18n/src/luz2d.rs` | chave `panel.inspector.luz.corners_rounded_n` | a tabela única de locale; o `Teste` deriva dela |
| `crates/ph2d-panel-inspector/src/sections/luz.rs` (+ `tests/it/seam_luz.rs`) | o aviso pintado no cartão LUZ | `tr_with`; sem hex nem `f32` de UI novos |
| `shells/desktop/src/render_loop/present_luz.rs` | +3 linhas: `luz2d.soltar_se_renasce(…)` → `flip_compose.receber_luz(gpu, None)` | composição (o renascimento solta os receptores antes, W6c); a catraca `the_shell_only_shrinks` passou na suíte |

O resto: `crates/ph2d-light2d-gpu` (27 ficheiros), `crates/ph2d-app-light2d` (20), `docs/Luz2D` (23).

## 3. Superfície de colisão (`bash scripts/collision-surface.sh`, 22/10 — REFERÊNCIA, o `main` do dia lê-se no ficheiro)

```
  merge-base ad5f8fe39c   ·   77 commit(s)   ·   75 arquivo(s)
▸ SCHEMAS            PROJECT_SCHEMA 191 (base: 191) · tripla (191, 13, 23) (base igual) · VEC_SCENE 23 · FLIP 13 · DOC_VERSION 20 — todos = base
▸ REGISTRO           ph2d-ecs 108 · ph2d-render 109 · ph2d-script 109 — todos = base
▸ CONTRATO (§6)      crates/ph2d-nodegraph/src/node.rs intocado · crates/ph2d-editor-core/src/tool.rs intocado
▸ ADR                esta linha não cria ADR (último 0188)
▸ Cargo.lock         nenhum '+name' novo
▸ MARCADORES         nenhum
▸ TETOS DE LOC       nenhum arquivo da linha passa do teto
```

Símbolos novos que podem colidir: a variante `AvisoDaLuz::CantosArredondados`; a chave i18n acima; as variáveis de
ambiente novas `PH2D_LIGHT2D_RENASCE_PAGINAS_NOVAS`, `PH2D_LIGHT2D_T_NULO`, `PH2D_LIGHT2D_RECORTE`, `PH2D_LIGHT2D_JUNTOS`,
`PH2D_LIGHT2D_BORDA` (todas `=0` compara; nenhuma é partilhada com outro módulo).

## 4. Contratos congelados

**Nenhum.** Grep do diff: zero ocorrências de `NodeOp`, `OpResolver`, `NodeManifest`, `RasterEditTool`, `CanvasPaintTool`,
`PanelEvent`, `ColorProfile`, `PROJECT_SCHEMA`; nenhum caminho de crates de node, tool, imageio ou do modelo vetorial.

## 5. O que mudou no produto (números MEDIDOS; RTX 5060 Ti)

| Onda | O quê | Medido |
|---|---|---|
| W6b–W6c | a vista RENASCE quando a placa a degrada; os receptores soltam-se antes — nenhum quadro sem luz | o vigia da `c2`, ≤ 2× por processo |
| W6d | a borda sem pontos pretos (o campo contínuo) e o canto convexo sem chanfro | 17 zooms 0,6–4: ×0 cruzamentos a mais |
| W6e–W6g r9 | os CANTOS VIVOS no padrão (o campo que guarda o vértice + a mistura das faces, a `V`/`V2` esparsas) | memória +0,3…+3,4 % sobre a lei W6d (era +21…+62 %); C3 +2,4…+4,8 % na vista de fábrica (§55) |
| bug 6 (§33) | a luz de FORA entra pela borda (o R do entorno no lugar do céu) | — |
| §35 | o laranja do refletor de cima curado | bissectado |
| §37 | o entorno da Baixa a 4× (Média/Alta 8×), por ordem do dono | erro da luz de fora −4,0…−4,7 % → −1,2…−1,4 % |
| §40.2 | a 2.ª passagem do entorno recortada | −3 % (1080p Baixa) … −27 % (2160p Alta); o mesmo bit |
| §44 | os 4 quadrantes num despacho, só até 589 824 texels | o mesmo bit (gate); a vista ganha até 1120×512 |
| §46–§47 | o nível 0 da descida por UM cone | o mesmo bit |
| §48–§49 | a vista nascida em páginas novas do driver (≥ 3 GiB) renasce uma vez | 2160p Alta `c2` 63,4 → 55,1 ms |
| §52–§53 | a descida não lê o R de cima onde T = 0 | 2160p Alta `c2` 51,7 → 50,9 ms; a vista ganha nos 9 tetos; o mesmo bit |

## 6. O portão do fecho (sobre o diff acumulado, 1×)

- `nextest-impacted` (base = merge-base, `target/bug6/suite_52.log`): **17 799 / 17 801**. As 2 vermelhas são razões de
  relógio do `ph2d-tool-painter` (que a linha não toca), com `load` 108–215:
  `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (verde ao repetir) e
  `a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs` (1 vermelha, 2 verdes ao repetir) —
  candidatas a flakes de carga, por confirmar com a máquina calma. Incluiu `file_loc_caps` (shell),
  `arch_safe_clamp_only` e `the_shell_only_shrinks`: verdes.
- `clippy --all-targets --all-features -D warnings` nas 6 crates tocadas (`ph2d-light2d-gpu`, `ph2d-app-light2d`,
  `ph2d-panel-inspector`, `ph2d-i18n`, `ph2d-editor-core`, `ph2d-host-desktop`): **verde** depois de `a024c29d77`.
- `cargo fmt --check` (as duas crates da luz): limpo. `cargo machete` nas 6: nenhuma dependência sobrando.
- Auditoria, 2 lentes. **Corrigido:** `PH2D_LIGHT2D_T_NULO=0` só chegava ao entorno (agora também à vista).
  **Fica, dito:** `Solver::diag_cones_um` (`#[doc(hidden)] pub`) é a porta de comparação do gate do §47; o `eprintln!` do
  renascimento (`quadro_renasce.rs`) é a linha de terminal da W6b; com o renascimento pedido e a grade ausente os
  receptores soltam-se a cada quadro até ela voltar (plausível, sem efeito na tela — o quadro já sai sem luz).
- ⚠️ **Os gates de placa saltam sem GPU** (`eprintln!("sem GPU — saltado")` e passam): num CI sem placa o «0 texels» não
  é prova — o integrador corre-os nesta máquina.

## 7. O que só o `ship.sh` pega

Nada sabido: fmt e clippy das crates tocadas verdes no fecho, machete limpo, sem dependência externa nova.

## 8. Smokes do dono (depois de integrado, a partir da raiz do repo)

1. `cd /home/enio/Documentos/Projetos/PH2D && PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke` — os
   cantos vivos das formas e a luz que entra pela borda da vista; `=6` a segunda cena dos cantos.
2. A Baixa com e sem os quadrantes juntos: o mesmo, com `PH2D_LIGHT2D_JUNTOS=0` — a luz tem de ser igual (aprovado na
   linha em 22/10).
3. Os outros: `PH2D_LIGHT2D_SMOKE=1..8` (7 = traço do Flip, 8 = formas do Motion: `PH2D_MOTION_OBJETO_SMOKE=1
   PH2D_LIGHT2D_SMOKE=8`).

## 9. Aberto (próxima onda da linha; [W6g §56](HANDOFF_CONTINUACAO_W6g_2026-10-19_O_CAMPO_QUE_GUARDA_O_VERTICE.md))

1. **Decidido (22/10):** os cantos vivos ficam e o alvo «C3 ≤ +5 % sobre a lei W6d» vale para TODA câmera das cenas 5 e
   6 — hoje +5,4…+13,2 % fora da vista de fábrica (§55): a rodada do custo dos cantos (fina e borda).
2. A borda nos níveis 11–12 a 2160p Alta: +3,62 ms, sem outra cura de mesmo bit à vista.
3. A anomalia da r5b (§12); a família (b) (§37–38).
4. Um gate que prove o elo contador → `AvisoDaLuz::CantosArredondados` na app (o contador da placa já tem
   `o_teto_dos_cantos_recusa_e_diz`).
