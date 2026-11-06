# HANDOFF DE INTEGRAÇÃO — linha Pixel Art (conta-gotas do seletor e zoom ao sair) — 2026-11-06

> Leitor: o agente integrador. A linha **fecha e PARA**: não integra, não faz push.
> Antecessor: [P8d + P9](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-05_P9.md), já no `main`; este fecha os dois «abertos do dono» dele.

## 1. Identidade

- Branch `line/PixelArt`; worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt`.
- Merge-base `bbfefaf393` (o `main` no dia, 0 commits atrás). Código em `4b83b84fd7`; o commit seguinte só leva este handoff e a linha do §5.
- `git diff --stat bbfefaf393..4b83b84fd7`: 11 ficheiros, +246 / −2.
- **Smoke do dono: `Smoke OK` em 06/11** («Zoom ok · color picker ok»). Integrar não é aprovar.

## 2. O que entrega

**Conta-gotas do seletor partilhado em Draw.** `ramo_pixel` vem antes de `forward_to_hero` (`input_dispatch.rs`), que é quem serve o `EyedropperPick`. Com o seletor armado, o clique ia para `input::press`: pintava, e o seletor ficava armado. Agora:

| Clique armado | Resultado |
|---|---|
| Sobre o sprite | `Pointer::Amostra(rgba)`: a cor autorada (`pick`, `Sample::AllLayers`), sem o desvio da leitura do ecrã; `conta_gotas::entrega` escreve no seletor e desarma; não pinta |
| Célula vazia | Desarma sem trocar a cor (alfa zero pintaria nada) |
| Fora do sprite | `Pointer::NotMine`: o caminho da casa lê o ecrã; medido: segue em Draw e desarma |
| Sobre painel ou menu | `free = false`, nada muda |

**Zoom ao sair do Pixel.** `input_roda::set_zoom` escreve `height_world` sem clamp, de propósito (a escada inteira até 96 px por célula). `Camera2d::zoomed` cravava esse valor na faixa: o 1.º entalhe de APROXIMAR **afastava** para `0,5`. Agora, fora da faixa, o gesto anda de volta e empurrar para mais longe dela não faz nada. Dentro da faixa, o resultado é o de antes.

## 3. Superfície de colisão

| Onde | O quê | Por que é aditivo |
|---|---|---|
| `ph2d-render/src/camera.rs` (**foundational**) | `zoomed` ganha 2 ramos, só com a câmera FORA de `[ZOOM_MIN, ZOOM_MAX]` | dentro da faixa é o mesmo `clamp_height`; único chamador em produto: `canvas_zoom.rs:93`, e `Camera2d::zoom` delega |
| `ph2d-app-pixel/src/compose.rs` | campo `View.conta_gotas`, variante `Pointer::Amostra`, 4 linhas em `pointer` | ⚠️ `View` é montada à mão em testes: **linha nova com `View { … }` não compila sem o campo**, cura de uma linha |
| `ph2d-app-pixel/src/conta_gotas.rs` | NOVO | ficheiro novo, `pub mod` em `lib.rs` |
| `ph2d-app-pixel/src/input.rs` | `cell` passa a `pub(crate)` | visibilidade |
| `shells/desktop/.../despacho_pixel.rs` | braço `Pointer::Amostra`, +5 linhas | o único `match` sobre `Pointer` |
| `shells/pixel-scenes/tests/it/` | `conta_gotas.rs` NOVO, `main.rs` +1, `edit.rs` +1, `fecho.rs` +3 | testes |

⭐ **`line/GameEngineFuncional`** tem `828051653b` (`Tab` só muda o modo no Pixel, timeline em `F4`), ainda fora do `main`. Medido: **nenhum ficheiro em comum** com os que esta linha toca em `ph2d-app-pixel`, `ph2d-render` e testes, e aquele diff não acrescenta `View {`. O dono relatou no smoke o `Tab` a abrir a timeline e **não foi corrigido aqui, de propósito**, para não duplicar: quem integra confere que aquele commit entrou.

Ids, consts, novos registos, `Cargo.toml`, `Cargo.lock`: nada.

## 4. Contratos e schemas: NÃO tocados

`git diff --name-only bbfefaf393..HEAD | grep -E 'nodegraph|ph2d-tool-|ph2d-vec|codec|PROJECT_SCHEMA|schema'` não devolve nada. Contratos do §6 intactos; formato gravado igual.

`ZOOM_MIN_HEIGHT_WORLD` **continua `0,5`**: está preso a `CAMERA_MIN_HEIGHT_WORLD` (a câmera de jogo) por um gate, e às réguas de memória e zoom do `fx_live`.

## 5. Números medidos

Sonda em processo no binário `smoke`, cena `PH2D_PIXEL_SMOKE=19`, janela 1930×1040 em tela virtual, roda e clique pelos métodos reais do `App` (`on_wheel_event`, `on_mouse_input`). A sonda não foi commitada.

| Saindo do Draw a `h = 0,1054` | Antes | Depois |
|---|---|---|
| 1.º entalhe de aproximar | salta para `0,4998` (afasta) | `0,1054` (parado) |
| Entalhes seguintes de aproximar | `0,5000`, preso | `0,1054` |
| Entalhes de afastar | a partir de `0,5` | `0,1171`, `0,1301`, `0,1446`, `0,1607`… |

| Conta-gotas armado, clique na camisa/fundo | Antes | Depois |
|---|---|---|
| Seletor | continua armado | desarma |
| Cor da frente, semeada `[40,20,80]` | não recebe a amostra | `[255,255,255,255]`, o branco do fundo sob o ponto |
| Fora do sprite | — | cinza do ecrã `[86,93,109]`, segue em Draw |

- `ph2d-app-pixel`: 11 387 linhas (`cat | wc -l`) contra o tecto 11 415; `the_shell_only_shrinks` verde. ⚠️ O tecto soma entre linhas: se reprovar, `python3 scripts/recontar.py`.

## 6. Prova de fecho

- **Mutações, 5 de 5 sangraram:** pintar armado (`if false`), alfa zero entrar no seletor, não desarmar, saltar perto da faixa, saltar longe dela. Código restaurado e verde depois.
- **Verdes por nome:** `conta_gotas` 2 · `a_camera_left_past_the_limit_never_jumps` · `ph2d-render zoom` 11 · `ph2d-host-desktop canvas_zoom` 8 (1 ignorado) · `fecho::` 4 · `enter_in_a_focused_field` 1 · `the_pixel_export_answers_a_real_pointer` 4 · `file_loc_caps` 4 · `the_shell_only_shrinks` 2 · `arch_safe_clamp_only` 2.
- **Estáticos:** `cargo fmt --check` limpo · clippy `-D warnings --all-targets` em `ph2d-app-pixel`, `ph2d-render`, `ph2d-host-pixel-scenes`, `ph2d-host-desktop` · `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh`.
- ⚠️ **NÃO correram:** `nextest-impacted`, a suíte completa e `censos-da-arvore-combinada.sh`. O modelo do comando de fecho ainda pede o `nextest-impacted`; prevaleceu a regra do dono de 23/10 (CLAUDE.md §2): são da integração. ⚠️ O clippy não foi o de `--workspace`: só as 4 crates acima.
- ⚠️ Prova de ponteiro: não há teste automático no `App` inteiro (o despacho real só correu na sonda e no smoke do dono). `shells/pixel-scenes/tests/it/conta_gotas.rs` entra pela porta `PixelApp::pointer`.

## 7. Auditoria de 2 lentes

**Correção.** `zoomed`: `f32::clamp` só entra em pânico com limite NaN ou invertido. No ramo perto, `h < MIN ≤ MAX`; no longe, `MIN ≤ MAX < h`; `h` NaN falha as duas comparações e cai no `clamp_height` de antes. `camera.rs` está fora do escopo do `arch_safe_clamp_only`, que passou. Traço: roda → `canvas_zoom.wheel` → `zoomed` → `tick` → `camera.height_world`; asserção vermelha: mutações 4 e 5.

**Costura.** Clique → `ramo_pixel` → `View::of` lê `eyedropper_pending` → `pointer` → `amostra` → `entrega` → `set_blender_value` → `paint.rs:420` espelha em `widget_colors` → `sync_swatch` → `app.fg`. O retorno antes de `self.pressed = …` deixa o `Up` em `NotMine`. Fora do Draw, `Canvas.editing = None` dá `NotMine`. Asserção vermelha: mutações 1 a 3.

**Deixado de propósito:**
- Armado, o botão direito também amostra (o `button` só filtra o do meio), e Espaço + clique amostra em vez de mover a vista: o próximo clique é do conta-gotas.
- O limite de aproximar fora do Draw continua `0,5`: depois de afastar além dele, a roda não volta ao zoom do Draw. O dono aprovou o zoom no smoke.
- A sonda 1 não reproduziu «até escolher outro objeto»: o tecto foi o mesmo com o sprite selecionado e sem nada selecionado. Não houve teste com outro objeto de fato (a cena tem um só). O dono deu `Zoom ok`, sem esse sintoma de volta.

## 8. Smoke do dono (já feito: `Smoke OK`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=19 ./target/smoke/ph2d-host-desktop`
2. Painel **Pixel** ▸ **Colors** ▸ quadrado de **Foreground** ▸ ícone do conta-gotas no seletor ▸ clique na camisa azul: **Foreground** fica azul e o boneco não muda.
3. Roda para aproximar até **Zoom 4800 %**; sair para **Object**; roda para aproximar: parado; roda para afastar: aos poucos, sem pulo. ⚠️ O `Tab` só sai do modo depois de `828051653b` entrar.

## 9. ABERTO

- Conferir na integração que o `Tab` do `828051653b` entrou (§3).
- `curve_a11y`/`range_a11y`: lacuna do app ([P9a](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-05_P9a.md)).
- Os «deixado de propósito» da [P9 §7](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-05_P9.md).
- Próxima etapa do módulo: **P10 Tilemaps**, em [02_plano_modulo.md](../02_plano_modulo.md).

## 10. Perfil do loop (`scripts/agent-loop-profile.sh`, 20 sessões)

```
✓ paralelismo de ferramenta              1.84/passo   alvo: >= 1,5
✓ respostas por sessao (mediana)                 58   alvo: <= 800
✗ cargo test : cargo check                 243 : 41   alvo: <= 1,0
✓ edicoes SEM conferir que casaram             1.4%   alvo: <= 2%
✗ contexto relido por passo (media)         306 mil   alvo: <= 250 mil
✓ contexto no inicio da sessao               66 mil   alvo: <= 80 mil
```
