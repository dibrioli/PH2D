# HANDOFF DE INTEGRAÇÃO — linha Pixel Art (P7: pincéis e dinâmica) — 2026-11-02

> Leitor: o agente integrador (lote, `scripts/integrar-rodada.sh`). A linha **fecha e PARA**: não integra, não faz push.
> Plano: [P7_plano.md](../P7_plano.md). Leis medidas e decisões do dono: [P7a](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-02_P7a.md) (§3–§4, §6) ·
> núcleo/app/lista: [P7b](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-02_P7b.md) (§2 = o que existe). Anterior de integração:
> [P6h–P6l](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-01_P6h_a_P6l.md) (já no main).

## 1. Identidade

- Branch `line/PixelArt`; worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt`.
- Merge-base `3df5894489`; commits: `git log --oneline 3df5894489..HEAD` (P7 inteira, de `48106f005a` a este handoff). P7b.4–5 nesta janela: `567356cec8` corte do `seam.rs` · `037c3bb17a` UI do pincel e da Dynamics · `9b9ae5b82e` cena 18 · `5881704ea1` fundo da cena e pega dentro da trilha (fotos) · `288d03792f` hook · `eb127bca16` testes para `pixel-scenes`.
- **Smoke do dono: PENDENTE** (cena 18, §5). Integrar não é aprovar.

## 2. O que a linha entrega

- **Núcleo** (`ph2d-pixel-core`): `pattern.rs` (`PatternBrush`, Source/Destination/Paint, `permuted`, `to_rgba/from_rgba`),
  `dither.rs` (Bayer 2/4/8 preso ao canvas), `dynamics.rs` (UMA rampa para pressão e velocidade; filtro da velocidade no
  VECTOR, τ = 50 ms; pega f = `66·f − 1` px/evento), carimbo por ponto no `Stroke`. Gates `brush_pattern`,
  `dynamics_oracle` (80 arrastos do Trial, erro 0,146 / tecto 0,16), `dynamics_laws`, `dither_laws`.
- **Crate NOVA `ph2d-pen-law`**: a corda (`pulled_string_step`) sai do `ph2d-painter-brush` (que passa a depender dela).
- **`ph2d-pixel-io::brush_file`**: `brushes.txt` canónico (`ph2d-pixel-brushes 1`), pincéis no COMPUTADOR
  (`PH2D_PIXEL_BRUSHES_DIR` › `$XDG_DATA_HOME/ph2d/pixel_brushes` › `~/.local/share/ph2d/pixel_brushes`).
- **App** (`ph2d-app-pixel`): Ctrl+B (com selecção / arrasto, direito corta, UM passo), escolher a frente pinta o pincel
  inteiro (dono §6.1), a mão do traço livre (`mao.rs`: porta da caneta, corda 8 px de ecrã/unidade), espaços Alt+1..9,
  `aplica_pincel` (`PixelRequest::Brush`), **espaço activo destrancado acompanha os PARÂMETROS do pincel** (forma, tamanho,
  ângulo, padrão, tinta, pixel-perfect) e **nunca as cores** (o Trial devolve as cores da imagem — `rb2b`); trancado nunca
  muda. ⚠️ A semântica do cadeado é desenho NOSSO (não medida no Trial além do `rb2b`): ver §6.
- **Painel** (`ph2d-panel-pixel-tools`): `seam.rs` cortado (584 → 500, `seam_opcoes.rs`); `ids_pincel.rs`,
  `paint_pincel.rs`, `seam_pincel.rs`: alinhamento do padrão (Fixed / Per stroke / Follows, com dica), Flip H/V,
  Rotate Left/Right (dono §6.2), pincéis guardados (miniatura ≤ 16², recuperar, cadeado, apagar), *Save Brush Here* e
  *Dynamics* numa fila (compacto ACIMA da paleta — a dobra de 01/11), a secção Dynamics (Stabilizer + Distance 1..64 — a
  foto 55 do Trial mostra 16 a ¼ da barra; Size/Angle/Color from Off·Pressure·Speed; Min size/angle; Blend Smooth/2×2/4×4/
  8×8; barras de duas pegas por sensor EM USO). *New Brush* no menu Edit. Size/Angle escondidos com o padrão na mão.
- **Widget NOVO** `ph2d-editor-core/src/widget/range_slider.rs` (+ `range_slider/tests.rs`, o gate dele): sem estado;
  `range_handle_at` (empate → o lado do ponteiro, e sobre as pegas juntas a que se pode mexer), `range_drag` (nunca cruzam,
  encostam), pegas presas DENTRO da trilha. O arrasto no painel segue o idioma do seletor (`active_id` + `last_pointer`).
- **i18n**: `crates/ph2d-i18n/src/pixel_pincel.rs` — inclui as chaves `panel.pixel.pattern.*` e
  `panel.pixel.dynamics.dither_*` que o editor-core já pedia SEM tradução (o censo do painel não as via).
- **Hook** `.claude/hooks/prazo-do-agente.sh`: `--path-format=absolute` (ver §6).
- **Cena 18** (`smoke_pincel.rs`, `max_level` 17 → 18) + sonda `tests/it/cena18.rs`.

## 3. Superfície de colisão

| Onde | Aqui | Nota |
|---|---|---|
| Crate NOVA `crates/ph2d-pen-law` | workspace por glob `crates/*`; `Cargo.lock` (+1 nome) | `ph2d-painter-brush` e `ph2d-app-pixel` dependem dela |
| `crates/ph2d-app-pixel/Cargo.toml` | +`ph2d-pen`, +`ph2d-pen-law` | |
| `Cargo.lock` | `pulldown-cmark` 0.13.6 (RUSTSEC-2026-0433, `48106f005a`) + `ph2d-pen-law` | regenere se outra linha mexeu |
| `ph2d-editor-core/src/pixel_panel/mod.rs` | `PixelRequest::Brush`, `PixelPanelInfo.{pattern, brushes}`, `mod brushes` | append-only; matches exaustivos fora da linha: o `check --workspace` passou (§4) |
| `ph2d-editor-core/src/widget/mod.rs` | `mod range_slider` + re-exports; `TRACK_MAX_PX` passa a público | |
| `ph2d-i18n/src/lib.rs` | `mod pixel_pincel` + `.or_else(pixel_pincel::tr)` | colisão textual provável com outras linhas que juntam tabelas — resolva mantendo as duas |
| `EDIT_ACTIONS` (`ids_selecao.rs`) | 17 → 18 (`pixel.edit.new_brush`) | só desta linha |
| Smoke `PH2D_PIXEL_SMOKE` | `max_level` **18** | |
| `CLAUDE.md §5` Pixel Art | trocado (smoke 1..18, este handoff) | |
| `.claude/hooks/prazo-do-agente.sh` | 1 linha + comentário | partilhado por todas as linhas |
| Catraca `the_shell_only_shrinks` (`ph2d-app-pixel` tecto 13 424) | **12 562** medido (era 14 385 antes do movimento) | SOMA entre linhas; reconte na árvore combinada |
| `shells/pixel-scenes` (`Cargo.toml` + `tests/it/main.rs`) | +6 módulos de teste; dev-deps +`ph2d-pen`, +`ph2d-pixel-io`, +`ph2d-pixel-render` | `Cargo.lock` +3 entradas na lista dela |
| `CLAUDE.md §5` (tecto 14 000 B) | **13 998** (o main tinha 13 989: só 11 B de folga) | outra linha que cresça o §5 vai reprovar — encurte a entrada DELA |

## 4. Gates (fecho, 1×)

**Mutações — 13/13 SANGRARAM** (âncoras pré-voadas, controlo verde antes):

| # | Mutação | Apanhada por |
|---|---|---|
| M1 | âncora ⌈w/2⌉ em vez de ⌊w/2⌋ | `brush_pattern` (3 testes) |
| M2 | `escolhe_fg` sem `pincel_imagem=false` | `pincel::choosing_a_colour_paints_the_whole_brush…` |
| M3 | sem o −1 da barra | `dynamics_oracle` (⚠️ só ele — `dynamics_laws` deixa passar) |
| M4 | o 1.º evento a contar | `dynamics_laws::the_first_event_only_marks_the_start` + oracle |
| M5 | filtro escalar | `dynamics_laws::reversing_the_direction…` |
| M6 | corda 6 px/unidade | `dinamica::the_stabilizer_string…` |
| M7 | parâmetro desligado aplicado | `pincel::each_ctrl_b_is_a_new_slot…` |
| M8 | botão direito não corta | `pincel::ctrl_b_without_a_selection…` |
| M9 | cadeado ignorado | `pincel::an_unlocked_shape_slot…`, `the_brush_menu…` |
| M10 | o espaço acompanha a *Image Color* | `pincel::each_ctrl_b_is_a_new_slot…` |
| M11 | pegas cruzam | `range_slider::as_pegas_nunca_se_cruzam`, `seam_pixel_pincel::dragging…` |
| M12 | a forma não larga o padrão | `pincel::the_brush_menu…` |
| M13 | o arrasto agarra sempre a pega de cima | `seam_pixel_pincel::dragging…` |

**Mutações:** os testes `pincel::`, `dinamica::`, `cena18::` (e `selecao`, `pontas`, `fecho`) moram AGORA em
`shells/pixel-scenes/tests/it/` (`eb127bca16`, a catraca — §3); as mutações correram antes do movimento, sobre os mesmos
testes sem alteração além do caminho `crate::edit` → `crate::comum`.

**Verificador (1×, árvore em `5881704ea1` + o movimento):** fmt `--check` e clippy `--all-targets -D warnings` das 8 crates
tocadas verdes · `cargo test -p ph2d-app-pixel` (motivo: deps) e `-p ph2d-host-pixel-scenes` (motivo: código movido)
verdes: 64 + 109 · `ph2d-pen-law` sem testes próprios (a lei é provada pelo `ph2d-painter-brush pulled_string`, verde) ·
`seam_pixel*` (36), `every_`, `seletor`, `brush_pattern`, `dynamics*`, `dither`, `stroke`, `brush_file` (3), `range_slider`
verdes · portões `architecture_panel_loc_cap`, `_widget_loc_cap`, `_panel_wiring_parity`,
`_interactive_crate_has_behavioral_test`, `the_shell_only_shrinks` (depois do movimento), `o_claude_md_cabe_no_orcamento`
(depois de encurtar a entrada) verdes · `cargo check --workspace --all-targets` verde (82 s, zero avisos).

## 5. Smoke (para o dono) e fotos

**Smoke da cena 18 APROVADO pelo dono em 02/11** («smoke OK»). A próxima onda é a P8 (`02_plano_modulo.md` §5).

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=18 ./target/smoke/ph2d-host-desktop`.
Sonda (`cena18`): Ctrl+B carimba as 6 cores; Flip H inverte a fila; com as pegas da cena (0/300 ‰) e células de 5 px de
ecrã, 2 px/evento → 3 de altura, 24 px/evento → chega a 16. Fotos `smoke_P7b_cena18_*_2026-11-02.png` (fábrica, layout do
dono, Dynamics aberta — esta com um build temporário NÃO comitado que abria a secção). `~/.ph2d/layout.txt` do dono:
sha256 `794f6484…` igual antes/depois.

Binário do smoke já compilado na worktree (`--profile smoke`), 2.ª corrida:
```
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

## 6. Não medido / aberto

- **O cadeado**: no Trial só se mediu que escolher uma cor NÃO muda o espaço (`rb2b`). Que um espaço destrancado acompanhe
  forma/tamanho/ângulo/virar é desenho nosso (sem ele o cadeado seria controlo morto). Se o dono quiser outra lei, é
  `pinceis.rs::acompanha_espaco`/`funde`.
- Os parâmetros por espaço (as caixas Brush/Color/Ink/Extras do Aseprite, foto 66) NÃO têm UI: `NEW_BRUSH` e `SAVE_HERE`
  são fixos.
- Angle ← Velocity (Max = ângulo da barra por analogia), Gradient até saturar, pressão real só com a Wacom do dono.
- O `dynamics_laws` não apanha o −1 da barra (M3); só o oráculo. Aceitável (o oráculo corre na mesma suíte), registado.
- **Hook `prazo-do-agente`**: `git rev-parse --git-common-dir` sai RELATIVO ao `-C`; com a janela em `crates/` nasceu
  `crates/.git/ph2d-agentes/<id>`, e o glob `crates/*` partiu TODO o cargo (o 1.º lote de mutações não correu).
  Corrigido aqui; até integrar, o hook do main continua com o defeito — não chame agentes com o cwd num subdirectório.
