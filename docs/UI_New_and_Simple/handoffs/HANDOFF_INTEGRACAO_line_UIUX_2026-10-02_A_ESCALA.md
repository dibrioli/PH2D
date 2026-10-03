# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-02 — a escala da interface (e o HiDPI)

> Leitor: o agente integrador (DIRETRIZ §1.5.3–1.5.4), e só por ordem do Enio. A linha FECHOU e
> PAROU: não integrou nem fez push.
>
> **Branch `line/UIUX` · base (merge-base = `main`) `1ad60a1ce` · 15 commits sobre a base** — o
> HEAD é o commit que traz este ficheiro. `git cherry main HEAD` = 15 `+`; o `main` não andou
> durante a jornada (não houve rebase).
> O mecanismo de cada mudança está na mensagem do seu commit. A spec é
> [`../spec/05_a_escala_da_interface.md`](../spec/05_a_escala_da_interface.md), e o estado de
> antes desta janela está na [continuação de 02/10](HANDOFF_CONTINUACAO_line_UIUX_2026-10-02.md)
> (§2 = como a escala funciona).

## §0 — Para o `CLAUDE.md` §5.1 (UI/UX)

Troque o link do último handoff por este ficheiro. A natureza do módulo não mudou, por isso a frase
fica igual.

## §1 — O que a linha entrega (os 15 commits)

| commit | o quê |
|---|---|
| `0db5b4754` | A legenda de um chip é uma palavra SOLTA: o orçamento é o passo do chip. `SETTINGS` → `PREFS`. |
| `a412d9aa5` | O pulldown da fila é tão largo quanto a face mais larga que pode mostrar (`Compound.faces/row_w`, `size_row_pulldowns`). |
| `5554c0f8a` | O número que não cabe perde CASAS, arredondado (`widget::numero_que_cabe`): nunca perde dígitos nem vira `…`. |
| `a0e8e26d0` · `463ab3181` | Barra de estado apertada: só o TEXTO encolhe, os curtos ficam inteiros (`teto_comum`), e a largura VIAJA calculada. |
| `4ae8d7e05` · `99e88a4a6` | ⭐ **A escala da interface inteira**, em *Edit ▸ Preferences ▸ Interface scale*: `80·90·100·125·150·175·200 %`. |
| `6a50d1d8a` | ⭐ **HiDPI**: `s = scale_factor do winit × UiScale`. |
| `000ddc97b` | O oráculo HiDPI corrido: [`../spec/oraculos/hidpi_2026-10-02.md`](../spec/oraculos/hidpi_2026-10-02.md). |
| `0d0bd8c5a` | A coluna vertical legada (`F9`) é tão larga quanto o pulldown mais largo (escolha do dono). |
| `ad0f040e9` | O texto do chrome pousa no píxel do ECRÃ (`ui_scale::ao_pixel`) e fica nítido a 125/175 %. |
| `b59d0bff9` | Três gates reforçados: as mutações que lhes sobreviviam passam a reprovar (§4). |
| `238b52654` · `c6f5fc390` · `7231395d2` | Docs: as continuações e a memória (2 lições). |

### O que esta janela fez (os três abertos da continuação, todos fechados)

1. **HiDPI.** `UiScaleMap::no_ecra(UiScale, ecra)` (o `new` passa a ser o ecrã `1,0`) e `impl
   Default` (a identidade). `pintar_no_chrome` recebe o `UiScaleMap` (antes recebia o `UiScale`).
   `HeroScreen.escala_do_ecra` (fábrica `1,0`): `HeroScreen::escala()` = ecrã × preferência, logo
   todas as portas físicas o herdam.
   - A shell escreve o factor na `fase_hero_paint`, ANTES da pintura que constrói o índice de hit.
     O factor viaja com o índice: escrito no `ScaleFactorChanged`, as consultas entre o evento e o
     quadro seguinte usariam o factor novo contra o índice velho.
   - **Oráculos corridos:** o Blender 5.2.2 faz `efectivo = sistema × utilizador`, com a arte
     física. O `Auto` da Godot 4.7.2 é o factor do sistema, e um valor manual SUBSTITUI-o. ⇒ Fica
     a lei do Blender; o nosso `100 %` é o `Auto` da Godot.
   - **Foto** (`WINIT_X11_SCALE_FACTOR=2`, 2560×1412): o chrome sai a 2×, e as réguas marcam as
     mesmas unidades por píxel que a 1×.
2. **Coluna `F9`.** Antes tinha `57/61/65 px` fixos e o pulldown pedia `57,9`: `Global` saía `Gl…`
   e `Selected` saía `S…`/`Sel…`. Opções medidas e levadas ao dono (coluna larga · face rodada ·
   abreviar): ele escolheu a **coluna mais larga**, a mesma escolha que fez para a fila.
   - Agora: `widget::column_width_px` (a coluna a ~80 px no Medium). O `frame_layout` mede (pede
     o `TextSystem`) e o `paint_left_rail` mede o mesmo rail.
   - ⚠️ A 1.ª redacção re-derivava a largura do chip como `rect.w − recuos`, perdia um ULP e
     `Selected` saía `Select…` (42,700977 num orçamento de 42,700974). A largura agora VIAJA
     medida (`widest_pulldown`).
3. **Nitidez a 125/175 %.** A causa: o preset de fábrica `CrispHeavyPlus` não tem hint, e o
   `y.round()` da linha de base era LÓGICO. ⚠️ O Vello sozinho estaria certo: o `append`
   compõe a escala no `run.transform`.
   - A cura é `ui_scale::ao_pixel`: a grelha vale `s` só dentro do `pintar_no_chrome`, e a
     `100 %` é `round` AO BIT.
   - Medido nas fotos (largura média da transição AA nas bordas horizontais, píxeis do ecrã;
     antes → depois):

     | | 100 % | 125 % | 175 % | 200 % |
     |---|---|---|---|---|
     | subtítulo | 0,75 → 0,75 | **1,07 → 0,63** | **0,96 → 0,67** | 0,87 → 0,82 |
     | menu | 0,85 → 0,85 | **1,01 → 0,69** | **0,94 → 0,84** | 0,82 → 0,80 |

     As bordas verticais não mudaram.

## §2 — Foundational tocado (aditivo) e contratos

- `ph2d-tokens`: `UiScale`, um ficheiro novo, `ui_scale.rs`.
- `ph2d-editor-core`: o `ui_scale.rs` novo e o hero; `paint.rs`/`paint_text.rs` passam a
  arredondar por `ao_pixel`.
- `ph2d-i18n`: as chaves novas.
- `ph2d-app-field3d` e `ph2d-panel-model3d`: as faces da vista.
- **Contratos congelados: nenhum** (§6 do `CLAUDE.md`). **Contadores intocados**:
  `PROJECT_SCHEMA`, registos, ADR, `Cargo.lock` e todo `Cargo.toml` (provado por grep no diff
  acumulado).

## §3 — Superfície de colisão (para o integrador)

**Falha ALTO noutra linha (não compila):**
- `ToolRailEntry::Compound` ganhou `faces` e `row_w`; `entry_advance` ganhou `axis`.
  `bar_rail`/`bar_split`/`publish_overflow` pedem `&mut TextSystem`.
- `AreaMenu.faces`; `StatusBar::preferred_width(&mut TextSystem)`; `paint_topbar_rail_chip`
  ganhou `vao`.
- `HeroScreen` ganhou `ui_scale` e **`escala_do_ecra`**: um literal da struct noutra linha
  precisa dos dois.
- `ui_scale::pintar_no_chrome(UiScaleMap, …)`: era `UiScale`. Hoje só esta linha a chama.
- `hero::frame_layout(hero, viewport, &mut TextSystem)`: é `pub(super)`.

**Funde LIMPO e REPROVA depois:**
- O censo `a_shell_pergunta_ao_chrome_pelas_portas_fisicas` reprova uma linha que escreva
  `hero.hit_index.hit(` / `hero.store.panel_at(` / `.handle_pointer(` na shell ou num
  `ph2d-app-*`. A cura é a porta física, nunca uma isenção.
- O censo irmão `o_factor_do_ecra_chega_ao_hero_antes_da_pintura` exige UM só escritor de
  `.escala_do_ecra =` na shell.
- Um pintor de texto NOVO que arredonde com `.round()` em vez de `ui_scale::ao_pixel` volta a
  desfocar a 125 %. Nenhum censo o apanha, só a foto (lição na memória).

**Enums e ids (append-only):**
- `ContextMenuKind::SettingsScaleSubmenu`.
- `UiScale`, 7 degraus: `P80`, `P90` ANTES de `P100`, que é o `#[default]`.
- Ids `CTX_MENU_SETTINGS_SCALE` e `CTX_MENU_SCALE_{80,90,100,125,150,175,200}`.
- i18n `chrome.menu.interface_scale` e `chrome.menu.scale_*`; `chrome.topbar.pill.settings` = `PREFS`.
- `prefs.txt`: chave `ui_scale=`.

**Muda comportamento:**
- A fila de ferramentas fica mais larga (pulldowns).
- A coluna `F9` cresce ~19 px (Medium) e o canvas perde esses px.
- Um decimal que não cabe sai arredondado.
- A 125/175 % o texto pousa noutro píxel.
- Num ecrã com escala do SO, o chrome inteiro cresce por esse factor (antes saía minúsculo).

**Tectos e a shell:**
- `shells/desktop`: `+161 −91` (líquido **+70**), e a catraca `the_shell_only_shrinks` está
  verde.
- `hero.rs` **698/700** (o comentário histórico do construtor encolheu); `hero/paint.rs`
  **700/700**; `left_rail.rs` 692.
- ⚠️ Duas linhas que acrescentem campos ao hero somam contra o mesmo tecto.

**Itens partilhados com usos apagados:** nenhum. **Listas/catracas baixadas:** nenhuma.

## §4 — Fecho: gate batched, mutação, auditoria

**Gate batched (1× sobre o diff acumulado; `load` a subir de 5,7 a 18,6 por outra worktree; zero
falhas, zero flakes):**
- `nextest-impacted` (BASE = merge-base): **19 613/19 613**.
- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets`.
- clippy do `ship.sh` (`--features ph2d-spike/bevy_ecs -D warnings`) e `cargo machete`.
- `check-standalone-optional` (10/10) e `check-workflow-packages` (32/397).
- `fmt --check`, `typos`, `censos-da-arvore-combinada.sh` (12/12) e `file_loc_caps` (4/4).

**Mutação** (relatório: `target/prova/mutacao_UIUX_2026-10-02.md`, que não é versionado):
- Cada gate novo foi visto VERDE e depois com a sua cura desfeita sozinha: 9 de 11 sangraram à
  primeira, em 27 mutações.
- **3 mutações sobreviveram**, e cada uma era um caso sem régua; foram curadas em `b59d0bff9` e
  reaplicadas, e as três reprovam:

  | mutação | o que lhe faltava | gate novo |
  |---|---|---|
  | `teto_comum` com parte igual | um caso com folga dos curtos | `a_folga_dos_curtos_vai_para_os_longos` |
  | `pintar_no_chrome` sem o atalho da identidade | a cena intermédia sob `scale(1)` dá o MESMO `draw_data` | `a_identidade_pinta_directo_sem_rascunho` |
  | `ao_pixel` com viés `+1e-4` | valores perto de um empate (entram `0,49999` e `10,49995`) | — |

- Controlos vistos nesta janela:
  - a coluna devolvida a `rail_width_px` **sobreviveu** à 1.ª redacção do gate (o chip
    transbordava sobre o canvas com o texto inteiro); com a contenção, reprova 1 458;
  - `ao_pixel` = `round` lógico ⇒ «a linha de base cai em 92,5»;
  - `chrome_hit` sem o ecrã ⇒ vermelho;
  - escritor do factor apagado ou movido ⇒ vermelho.

**Auditoria — duas lentes:**
- **LENTE wiring.**
  - CLAIM: o factor do ecrã chega a toda pergunta ao chrome.
  - TRAÇO: `WinitHost.scale` (`winit_host.rs:28`, actualizado em
    `despacho_metodos_janela_e_vetor.rs:150`) → `fase_hero_paint` → `hero.escala_do_ecra` →
    `HeroScreen::escala()` (`na_escala.rs:40`) → `chrome_hit` / `handle_pointer_fisico` /
    `rect_to_physical`; e a saída é `pintar_no_chrome(hero.escala())` (hero e toasts).
  - ASSERÇÃO-VERMELHA: `no_ecra_hidpi_a_escala_multiplica_e_o_clique_cai_no_mesmo_id` e
    `o_factor_do_ecra_chega_ao_hero_antes_da_pintura`.
  - NÃO-CHECADO: o `ScaleFactorChanged` a meio de um arrasto (o factor muda no quadro seguinte).
    A macOS `cursor_pos.rs` não usa o chrome.
- **LENTE correção.**
  - CLAIM: a 100 % num ecrã de factor 1 nada se move.
  - TRAÇO: `no_ecra(P100, 1.0)` ⇒ `s = 1,0` ⇒ `is_identity` ⇒ `f(viewport, scene)` directo; e
    `ao_pixel` com `GRELHA = 1` é `round`.
  - ASSERÇÃO-VERMELHA: `a_cem_por_cento_e_o_caminho_de_sempre`,
    `a_identidade_pinta_directo_sem_rascunho` e `ao_pixel_a_cem_por_cento_e_o_round`.
  - NÃO-CHECADO pela suíte: ecrã `1,25` × `80 %` dá `s = 1,0` exacto em `f32` (conferido à
    parte), logo cai no caminho de fábrica, o que é correcto; nenhum gate o fixa.

## §5 — Premissas do briefing que a medição derrubou

- «10 commits por integrar»: eram **9** quando a janela abriu (todos `+` no cherry).
- «Como o Blender combina DPI»: o Blender multiplica, mas a Godot **substitui** (o `Auto` dela é
  o factor do sistema). A doc do `no_ecra` dizia «os dois multiplicam», e foi corrigida.
- «Foto em tela virtual: o `kwin --virtual` aceita escala?»: o `--scale` SOZINHO não é HiDPI.
  Para o nosso app basta `WINIT_X11_SCALE_FACTOR` na `fotografa_cena.sh` (lição na memória).
- «Nas fotos ficou bom, mas não foi medido»: estava **43 %** mais macio. A foto a olho não o via.

## §6 — Smoke do dono (o que ensaiar)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. Em *Edit ▸ Preferences ▸ Interface scale*, escolha **125 %** e depois **175 %**.
   - Tem de acontecer: tudo cresce e as letras ficam tão nítidas como a 100 %.
   - Deu errado se: as letras parecerem «desfocadas» só nestas duas escalas.
3. Carregue **F9**: aparece a coluna vertical à esquerda.
   - Tem de acontecer: os botões «Global» e «Selected» ficam inteiros.
   - Deu errado se: aparecer `Gl…`/`Sel…`, ou um botão sair da coluna por cima do desenho.
4. (Só com um ecrã com zoom do sistema.) Nas definições do sistema, ponha a escala do ecrã em
   150 % ou 200 % e abra o app.
   - Tem de acontecer: o app abre do tamanho certo, não minúsculo.
   - Deu errado se: os cliques caírem ao lado dos botões.

## §7 — Perfil do loop (`bash scripts/agent-loop-profile.sh`, 20 sessões)

```
  ✗ paralelismo de ferramenta              1.07/passo   alvo: >= 1,5  (5% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                321   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check              5733 : 1934   alvo: <= 1,0  razao 3.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  20%   alvo: >= 80%  (6690 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         701 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               61 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## §8 — Binário de smoke (último passo: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida)

```
SMOKE_BUILD_OUTPUT
```
