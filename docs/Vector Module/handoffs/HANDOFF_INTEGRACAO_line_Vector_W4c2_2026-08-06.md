# HANDOFF DE INTEGRAÇÃO — `line/Vector`, **a escala fica VIVA** (W4c.2, 2026-08-06)

**Status:** FECHADO 2026-08-06 · no `main` em `6a5aa61b9` (o commit que trouxe este arquivo).

> **Branch:** `line/Vector` · **HEAD:** `01bbe2485` · **base:** `main` (rebased, `--ff-only` limpo)
> **Commits desta wave:** 2 (`4204b330b` o motor, `01bbe2485` o roteiro de smoke)
> **⚠️ PENDENTE DE SMOKE.** A W4c.1 (`af5ab7cec`) foi aprovada e está na mesma branch.
> Plano-mãe: [`PLANO_UI_UX_padrao_figma.md`](../Estudos/PLANO_UI_UX_padrao_figma.md) · estado e fila:
> [`HANDOFF_line_Vector_tokens_2026-08-06.md`](HANDOFF_line_Vector_tokens_2026-08-06.md)

---

## 1. O que entra

A escala do design system deixa de ser assada na compilação. `Spacing::Md.px()`,
`Radius::Lg.px()` e `StrokeToken::Default.px()` passam a devolver **o número que o artista
autorou no modo vigente** — e o app inteiro re-espaça, re-arredonda e re-engrossa na hora.

**Como, em três linhas:**

1. **`ph2d-tokens::num_runtime`** (novo) guarda uma **tabela plana de 21 `f32`** e a bandeira
   *"ela está cheia?"*.
2. **`publish(theme)`** resolve o grafo de autoria (a camada + os aliases da W4c.1) para essa
   tabela, **uma vez por quadro**, no fim de `tokens_bridge::dispatch`.
3. **`Spacing::px()` lê a tabela**; **`Spacing::factory_px()`** (novo, `const fn`) ficou com a
   tabela gerada do `tokens.json`.

---

## 2. ⚠️ A MEDIÇÃO que virou o desenho ao contrário — leia isto antes do diff

O handoff da linha prescrevia: *"`const` item → `px_live(theme)` no ponto de uso, **um a um**"*, e
estimava **15 sítios**. Medido:

| | contagem |
|---|---|
| `const` items que lêem a escala | **13** (a varredura do handoff perdia os que vivem **dentro** de um `const fn` — o compilador achou-os) |
| sítios de **LEITURA** da escala | **~1200** (`Spacing::Sm.px()` e irmãos, `crates` + `shells`) |
| modos activos ao mesmo tempo | **1** (`AppGfx.theme` → `HeroScreen`; a tecla `M` e o menu escrevem no mesmo campo) |

⇒ Enfiar `theme` em cada leitura seria **mil e duzentas edições** para responder, mil e duzentas
vezes, uma pergunta que o app responde **uma** vez por quadro. Com a tabela, esses ~1187 sítios
ficam vivos **sem serem tocados**, e os 13 `const` **quebram na compilação** — o compilador
enumera-os, em vez de uma lista à mão que envelhece.

E é literalmente a arquitectura que o plano enuncia (§(b), Vol. 2 §4): *a tabela achatada por modo
é a forma de RUNTIME; o grafo de autoria vive no editor*.

**O `px_live(theme)` da W4c.1 MORREU.** Com `px()` vivo ele seria a **terceira** porta para duas
perguntas, e a terceira é a que alguém chama por engano.

---

## 3. ⚠️ O TETO que a W4c.1 deixou como dívida: MEDIDO, e não há teto a escrever

A dívida era: *o painel de Tokens desenha-se a si mesmo com estes tokens, então um valor absurdo
pode empurrar para fora da tela o botão que o desfaria*. Medição
(`crates/ph2d-panel-tokens/tests/scale_ceiling.rs`, sonda `-- --ignored --nocapture`):

```
spacing.* (px) |  y (scroll 0) | rolagem que o alcanca |   y ja' rolado
          8.0 |         174.0 |                   0.0 |          174.0
        256.0 |         670.0 |                 220.0 |          450.0
       1024.0 |        2206.0 |                1756.0 |          450.0
       4096.0 |        8350.0 |                7900.0 |          450.0
      65536.0 |      131230.0 |              130780.0 |          450.0
```

**A rolagem alcança o desfazer em toda escala testada.** O escape não é um número — é o corpo
rolável que o painel já tem. E um cap **constante** seria um palpite: o penhasco de posição é
`y ≈ 158 + 2·px`, **função da altura da janela**, então qualquer literal estaria errado para
metade dos monitores. A porta continua a recusar só o que não é um comprimento.

⚠️ **Três afirmações minhas que a medição derrubou**, escritas no cabeçalho do gate porque a
próxima pessoa vai ter as mesmas três:

1. *"o `Reset This Mode` vive num cabeçalho que não rola"* — **falso**, ele é pintado no corpo
   rolável;
2. *"então basta rolar até ao fim"* — **falso**, a rolagem máxima passa **por cima** dele
   (`y = −108244`);
3. *"o controle é a escala de fábrica"* — **falso**, o botão só é pintado com algo autorado.

⚠️ E a 1ª versão do probe reportou `NENHUMA` em `65536 px` — era a **resolução da varredura**
(passo ~3440 px contra uma janela de 900), não o produto. Hoje a rolagem é **resolvida** (a posição
é afim na rolagem) em vez de amostrada.

---

## 4. Foundational tocado — tudo aditivo, exceto UM rename deliberado

| Crate | O quê |
|---|---|
| `ph2d-tokens` | **módulo novo `num_runtime`** · `NumToken::index()` · `num_overrides::any_authored()` · **`Spacing`/`Radius`/`StrokeToken`: `px()` → `factory_px()` (`const fn`) + `px()` novo, não-const** · `px_live` removido |
| `ph2d-editor-core` | 10 `const` → `fn` (ver §5) · `TOOL_RAIL_WIDTH_PX` → `tool_rail_width_px()` · **os dois `RAIL_W` → `rail_w()`** |
| `ph2d-panel-grid-snap` | `PAD`/`ROW_GAP` → `pad()`/`row_gap()` |
| `ph2d-panel-hierarchy` | `INDENT_PX` (const local de fn) → `let` |
| `ph2d-ui-testkit` | **`set_panel_scroll(panel, y)`** — nomeado em vez de um `store_mut()` genérico, que seria porta aberta para um gate semear o que depois "prova" |
| `shells/desktop` | `tokens_bridge::dispatch` publica no **fim** · `RAIL_W` → `rail_w()` |

⚠️ **O rename `px()` → `factory_px()` é a única mudança não-aditiva**, e é deliberada: manter
`px()` como fábrica obrigaria os 1187 sítios de leitura a mudar de nome. Quem quiser a fábrica em
contexto `const` chama `factory_px()`, e o compilador diz onde.

---

## 5. Os treze `const` que o compilador enumerou

`context_menu_overlay::PAD_Y` · `tool_rail::TOOL_RAIL_WIDTH_PX` · `card::HEADER_H` ·
`color_swatch::CHECKER_CELL_PX` · **`color_swatch::SwatchSize::px` (era `const fn` — 3 dos 13
vivem aqui dentro)** · `dropdown::POPOVER_GAP` · `showcase::notes::{NOTE_TEXT_PAD_X, _Y}` ·
`showcase::{ROW_GAP, FIELD_H, SEPARATOR_PAD_Y}` · `grid_snap::layout::{PAD, ROW_GAP}` ·
`hierarchy::paint::INDENT_PX`.

**Cascata:** `TOOL_RAIL_WIDTH_PX` → dois `pub const RAIL_W` → `fn rail_w()` (17 sítios de uso,
`editor-core` + `panel-motion-graph` + `shells/desktop`), e `showcase::FIELD_H` → `W6_FIELD_H`.
Exactamente o que o handoff da linha avisou.

---

## 6. Colisões de integração

| Eixo | Estado |
|---|---|
| **`PROJECT_SCHEMA`** | **INTOCADO por esta wave.** ⚠️ A branch carrega o **57→58** da W4c.1, que é **PROVISÓRIO** — o valor se **CONTA** contra o `main` do dia. E ⚠️ **a colisão dele é MUDA**: se outra linha escrever 58, o `project.rs` **não conflita** (mesmo literal dos dois lados) e um dos bumps evapora com a suíte verde; quem denuncia é o `project_schema_tests.rs` ao lado |
| **ADR** | nenhum |
| **`Cargo.toml`** | **zero** — nenhuma dep, nenhuma crate nova |
| **ids / tokens / i18n** | **zero novos** |
| **Contrato congelado** | **3/3 + 4/4 + 11/11 verdes**, rodados |
| **`ph2d-ecs` registry** | intocado |

---

## 7. O que só o gate batched pega — e o que foi rodado

| Gate | Resultado |
|---|---|
| `scripts/nextest-impacted.sh` (BASE=main, `--no-fail-fast`) | **8846/8847** · a única falha é `ph2d-timeline::motion_path_perf::the_cost_of_sampling_a_path_is_flat_in_its_anchors` |
| clippy `--all-targets` (7 crates tocadas) | limpo |
| `cargo fmt --all --check` | limpo |
| `cargo check --workspace --all-targets` | limpo |
| `design_token_sync` | verde |
| `architecture_workspace_file_loc_cap` · `file_loc_caps` (shell) | verdes |
| `no_magic_numeric` · `no_tofu_glyphs` · `node_id_collisions` · `architecture_panel_wiring_parity` | verdes |

⚠️ **A falha é um gate de RAZÃO sensível a carga, e não é desta linha:** ele passa **isolado**, e
esta wave toca **zero** arquivos em `ph2d-timeline`. É a mesma família do
`the_cost_of_depth_is_linear_not_explosive` que o CLAUDE.md §5 já documenta — **re-rode sozinho
antes de suspeitar do merge**.

---

## 8. Gates e mutações

**7** no `num_runtime` (`crates/ph2d-tokens/src/num_runtime_tests.rs`) · **3** de teto/alcance no
painel (`scale_ceiling.rs`) · **3** no arch-gate da shell
(`shells/desktop/tests/the_scale_is_published_before_the_paint.rs`).

**6 mutações, 6 sangram:**

| Mutação | Sangra |
|---|---|
| `publish` nunca enche a tabela | 5 gates de `num_runtime` |
| `live()` devolve sempre `None` | 4 |
| `index()` devolve sempre `0` | 5 |
| a ponte **não publica** | o arch-gate |
| a ponte publica **antes** do laço de intents | o arch-gate |
| `Spacing::px()` volta a ser a fábrica | `the_panel_moves_when_the_scale_moves` |

⚠️ **O arch-gate é o que impede o defeito silencioso da wave:** apague a linha de publicação e
**toda a workspace fica verde** com a escala morta — as suítes de unidade publicam elas próprias.
Ele também recusa que a ponte passe a ser chamada sob `is_panel_visible` (a escala congelaria para
quem fechasse o painel).

---

## 9. O que smoke-testar

```
env PH2D_BUILD_SMOKE=59 cargo run -p ph2d-host-desktop --release
```

A cena imprime `21 de escala (px)` — **se essa linha não aparecer, pare**.

O roteiro (passo 9, reescrito nesta wave) pede a demonstração:

1. **Suba `spacing.lg` de 12 para ~40** — o app tem de **re-espaçar na hora** (painel, cards,
   rail). Baixe para 2 e ele aperta.
   ⚠️ **Se a janela não se mexer, PARE:** a tabela não está a ser publicada e o resto não diz nada.
2. `stroke.default` engrossa as linhas; `radius.*` arredonda as quinas.
3. **Reset devolve na hora**; o **elo** (`radius.md` seguindo `spacing.md`) move os dois.
4. **O absurdo**: ponha `spacing.md` em 1000 e **role** — o *Reset This Mode* tem de continuar
   alcançável (é o gate `the_panel_survives_an_absurd_scale`, medido até 65536).
5. **O modo**: autore no Forge, aperte `M` — o Workshop tem de voltar à fábrica (a escala de
   fábrica é uma só; o override é do par `(modo, token)`).
6. **O arquivo**: Ctrl+S, feche, reabra, Ctrl+O — a escala volta.

---

## 10. Aberto, com preço

- **`Density` e `Motion` ficam de fora** da família numérica, cada um com o motivo escrito no
  `num.rs` (uma é escolha do artista, a outra mede-se em **ms**). `chrome.*` não tem identidade de
  token — dar-lhe uma é wave própria.
- **A cor não passou pela mesma reforma.** `ColorToken::resolve(theme)` continua a receber o modo,
  e ali isso é **correcto**: a tabela de fábrica **é** por-modo. Uma tabela plana de cor seria uma
  optimização (o §2 do handoff da linha mediu que ela já faz uma varredura linear de strings por
  chamada), não uma mudança de semântica — e não é desta wave.
- **W4c.3 (math / `TokenValue::Expr`) é o próximo passo**, e agora ele tem onde pousar: um variant
  novo no `NumValue`, resolvido dentro do `publish`.
