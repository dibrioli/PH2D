# HANDOFF de integração — `line/InputMap`, ONDA 2, 10/11 (teclado de notebook inteiro como na foto do dono + camada de polimento pro)

Leitor: o agente integrador. Branch `line/InputMap`, código até `c1688d3a24` (+ este handoff). A onda 2 são **48 commits** (`git log --oneline 8f5c9d81b3..HEAD`; `git diff --shortstat 8f5c9d81b3..HEAD`: 150 files, +8 515 −974) sobre o handoff da onda 1; a linha inteira (ondas 1+2) são **136 commits**, 298 files, `+25 109 −3 061` (`git diff --stat main...HEAD`), sobre `bc4a6ce930` = merge-base = ponta do `main` (**o `main` não andou: sem rebase, `--ff-only` serve**). Nada integrado, nenhum push. **Suíte completa, `nextest-impacted` e `ship.sh` NÃO correram** (regra do dono 23/10): são do integrador. **Este handoff SUBSTITUI o da onda 1 como «último» do módulo e o COMPLETA** — a superfície da onda 1 (crates `ph2d-input-map-editor`, `ph2d-app-input-map`, `ph2d-player-controls`, `ph2d-input-map-scene`, tokens, schema 232, `hidden_in_game`, `play_map`, F1) vale integralmente: [HANDOFF onda 1](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md). Contratos: [01_plano.md §13](../01_plano.md) (o código manda). Prompt: [PROMPT_MISSAO_InputMap_ONDA2](PROMPT_MISSAO_InputMap_ONDA2_2026-11-10.md) (§2 = a ordem do dono, verbatim).

## 1. Ordem do dono (10/11, depois do smoke da onda 1) — cumprida

(1) O teclado desenhado das versões B e C passa a ser COMPLETO, no layout compacto de notebook da foto dele (`docs/InputMap/maquetes/06_teclado_notebook_dono.png`): fila F, números, pontuação, modificadores, setas em T, numérico com Num Lock. (2) Polimento pro em cada aspecto: hover, mouse down, foco nos campos numéricos, todos os acabamentos. Verbatim no prompt §2.

## 2. O que cada onda entregou

**W1 — códigos e nomes** (plano §13.1). `NAMED` 21 → **50** teclas nomeadas; numérico com códigos próprios `0xF740–0xF74F` (**Numpad Enter `0xF74F` ≠ Enter**); tabela à parte das teclas do sistema (Windows ×2, Fn, PrtSc: rótulo, sem nome de script, `Key::system_owned()` → porquê i18n `input_map_keyboard.system.{fn,windows,print_screen}`). API nova em `Key`: `cap`, `cap_short`, `cap_lines`, `is_enter_like`, `ENTER`, `NUMPAD_ENTER`. **Medido nesta máquina:** Fn ausente das caps evdev do teclado; o KDE liga Print→Spectacle e Meta→lançador; winit 0.30.13 não dá estado de Num Lock ⇒ **sem luz** de Num Lock. **Crate folha nova `crates/ph2d-input-winit`** (a tradução winit→`Key` saiu da shell: **189 034 → 188 940**; Super/Fn/PrintScreen ficam `None`). Luau `KeyName` com **101 nomes** (`kp7`; `api_table` + `ph2d.d.luau`; gate `os_nomes_de_tecla_sao_os_do_teclado`). `editor_keys.rs` em `ph2d-input` = o que nunca liga (F5, acordes; `raw_keys` reexporta). A escuta partilhada de `ph2d-editor-core` (`input_map_ops.rs`) recusa teclas do sistema e F5/Ctrl; **a recusada segue** (F5 sob a escuta ainda comanda o Play). O ficheiro do jogador lê `key:kp.`/`kp-`/`kp+`.

**W2 — o teclado desenhado** (plano §13.2). Tabela de dados de **101 teclas**; `Arrangement::{Wide,Narrow}` escolhido por função pura medida (`keys_fit.rs`, `keys_fit`); legenda por **GRUPO** (fila F, borda do principal, setas, Home/End/PgUp/PgDn, numérico): palavra a 11 → palavra a 10 (`TypeToken::Xxs`) → curta → símbolo ISO/IEC 9995-7. Ícones novos `kbd-{end,enter,home,page-down,page-up,print-screen,num-lock}` (**Num Lock = ⇭, nunca cadeado**); PrtSc em duas linhas «Prt»/«Sc» a 1600 e **o ícone a 1280** (duas linhas não cabem em 22 px). **B a 1280 empilha**: teclado Wide na largura inteira em cima, lista + cartão da aberta rolam como UMA região por baixo (o teclado nunca rola). Medido: B1600 lado a lado Wide passo 39,3 (kb 215 px); B1280 empilhada Wide 31,9 (kb 198); C1600 Wide 44,8; C1280 Wide ~32. Numérico em colunas de 1 u (a foto tem ~0,82 u: **desvio consciente**). Teclas do sistema esmaecidas, sem ligar, porquê no hover. Cena **19 «B5 · numpad»**; `CENAS` 18 → 19.

**W3–W5 — polimento** (plano §13.3). `interact.rs`: **registo único de controlos** (`doc.control`: alvo + premido nativo — Down captura, Up dentro dispara, fora cancela — + Tab/Shift+Tab/Enter/Espaço/Esc + nós a11y + cursor via `look_at`). **Hover INSTANTÂNEO por decisão D32** (`docs/CartoesUI/01_decisoes_do_dono.md`): o exemplo «fade do hover» do prompt está SUPERADO. Defeito curado: o `WidgetStore` da aba nunca alimentava `hover_live` (default `SETTLED = 1.0` ⇒ botões do cartão sempre pintados em hover). Polegares com fade (`Role::Fade`); pulso da escuta `Role::Decoration` (ausente em Discrete/reduced motion); caret pisca pelo novo `UiMotion::elapsed()` (fixo em reduced motion). `field.rs` = campo numérico partilhado (A e B): seleciona tudo ao focar no Down, ↑/↓ 0,01, Shift ×10, Alt ×0,1, Enter = UM passo de undo, Esc devolve, Tab aplica e segue, **clamp ao limite que o modelo guarda** com frase (Dead: `MAX_DEAD_ZONE` 0,99, agora `pub`). Campos de texto: foco no Down com caret e arrasto que seleciona; duplo clique seleciona tudo. Diálogo da receita: armadilha de foco que nasce no Cancel (Enter = Replace, Esc = Cancel). **Esc cancela a escuta do «Opens with»** (lei mudada: na onda 1 era «recusado»; F5 continua recusado). Tela do jogador: linha escolhida, Reset como botão, recuo igual; Remove/Add desligados com o porquê.

**W6 — acabamento.** 3 passagens de fotos + 2 revisões visuais independentes (27 + 22 itens, triados). **NÃO feito, por decisão:** a frase do hover da B continua a substituir o cabeçalho (R7b da onda 1); sem ◆ em Dead/Press; fila F em meia altura (as barras pedem altura de tecla); lupa de busca do app inteiro; Inspector. Auditoria de correção: 4 defeitos curados vermelho→verde (Tab em linha recortada → o Tab rola/revela; Numpad Enter na busca vazia; aviso de clamp nomeava 1,00 com 0,99 guardado; foco morto).

## 3. Foundational/compartilhado tocado nesta onda — por quê

| Onde | O que | Natureza |
|---|---|---|
| `crates/ph2d-input` | `Key` 50 nomeadas + numérico + sistema; `cap*`, `is_enter_like`, `system_owned`; `editor_keys.rs`; `MAX_DEAD_ZONE` pub; ficheiro do jogador `kp*` | aditivo |
| `crates/ph2d-input-winit` | **crate folha NOVA** (`Cargo.lock` +1 interno) | nova |
| `shells/desktop` | tradução winit→Key sai (−94 linhas); o resto é composição | **encolhe** |
| `ph2d-editor-core` | escuta partilhada recusa sistema/F5/Ctrl (`input_map_ops.rs`); visibilidade do caret aditiva no pintor de texto (resto do app byte-idêntico: por omissão aceso); `paint_text_elided_weighted` e os pintores de texto cortado → `paint_text_elided.rs` reexportado do mesmo caminho (`paint_text.rs` 701 → 526); `UiMotion::elapsed()`; `WidgetStore::caret_visible`, `TextInput::caret_visible`; 7 ícones `kbd-*` + SVGs | aditivo + movimento |
| `ph2d-script` | `KeyName` 101 nomes (`api_table`, `ph2d.d.luau`) | aditivo |
| `ph2d-i18n` | frases novas (porquês das teclas do sistema, limites, Remove/Add desligados) nos ficheiros `input_map_*` | aditivo |
| `ph2d-app-game` | `CENAS` 18 → 19 | contagem |

**Mesmo-símbolo a grepar numa fusão:** `CENAS` (`ph2d-app-game/src/smoke.rs`), a lista de ícones e `enum_order_matches_svgs`, `paint_text.rs`/`paint_text_elided.rs` e seus `mod`/`pub use`, o `NAMED`/`script_names` e `KeyName`, o `mod` de `ph2d-i18n/src/lib.rs`.

## 4. Superfície de colisão (colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap/scripts/collision-surface.sh`, 10/11, linha inteira)

```
SUPERFÍCIE DE COLISÃO — line/InputMap contra main
  merge-base bc4a6ce930   ·   136 commit(s)   ·   298 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        232   (base: 231)
  ⚠   └ tripla do gate               (232, 13, 29)   (base: (231, 13, 29))
    VEC_SCENE_SCHEMA                       29   (base: 29)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 25   (base: 25)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              121   (base: 121)
  ✗ ph2d-render (espelho = ecs + K)       —   (base: —)
  ✗ ph2d-script (espelho = ecs + K)       —   (base: —)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0200   próximo livre: 0201
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 5 pacote(s) '+name' novo(s):
      "ph2d-app-input-map"
      "ph2d-input-map-editor"
      "ph2d-input-map-scene"
      "ph2d-input-winit"
      "ph2d-player-controls"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
  ✗  1030 / 600   shells/desktop/src/main.rs
───────────────────────────────────────────────────────────────────────────────
  ✗ SONDA CEGA em: ph2d-render (espelho = ecs + K), ph2d-script (espelho = ecs + K) — a const mudou de ficheiro e nenhum candidato a tem.
    Corrija o caminho NESTE script: um «—» dos dois lados lia-se «ninguém mexeu» (13/09).
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

**Contadores como DELTA:** `PROJECT_SCHEMA` — **a onda 2 não o mexeu** (o +1, 231 → 232, é da onda 1; reconte pelo `main` do dia com `python3 scripts/schema-recount.py`; se outra linha subiu, esta vira `main + 1`, nos três sítios). Componentes DELTA 0 (`ph2d-ecs` 121; sonda cega do render/script: a linha não toca o registo). §6 intocado. ADR: nenhum. `Cargo.lock`: **+1** interno nesta onda (`ph2d-input-winit`), nenhum externo. `CENAS` **+1** (18 → 19). Shell **−94** (189 034 → 188 940; tecto da catraca 189 041 da onda 1: **re-meça** contra o `main` do dia). `main.rs` 1 030/600 é pré-existente (a linha não o fez crescer).

## 5. Valores literais que colidem

Nenhum token novo nesta onda (os 5 + `categorical-*` são da onda 1). Smoke: `PH2D_GAME_SMOKE` **6..19** (19 = B5 · numpad; 18 = P1 com `PH2D_GAME_SMOKE_DRIVE=editor`). **Se outra linha tomou cenas após 5, renumere pelo `main` do dia** (conte, não escolha).

## 6. Decisões técnicas e o porquê

- **Legenda por GRUPO, não por tecla**: um grupo inteiro troca de nível junto (senão a fila F mistura «Esc» a 11 e «F10» a 9); o símbolo ISO só se todo o grupo o tem.
- **Wide/Narrow é função pura medida** (`keys_fit`), não ramo por largura: o teste e o desenho decidem pelo mesmo número.
- **Numpad Enter = código próprio**, nunca `Enter`; `is_enter_like` faz os dois confirmarem (busca, diálogo).
- **Sem luz de Num Lock** (winit 0.30.13 não a dá) — mentir estado seria pior que não mostrar.
- **Teclas do sistema nunca ligam**: `AddBinding` delas recusado com `tracing::warn!`, nunca no-op calado.
- **Hover instantâneo (D32)**; só o polegar faz fade e o pulso da escuta é decoração.
- **Clamp ao limite do modelo com frase** (nomeia o valor guardado, 0,99).
- **Esc cancela a escuta do «Opens with»**: inverte a lei da onda 1; o gate antigo foi substituído.
- **Aviso do wrap**: parágrafos com quebra equilibrada (bissecção sobre a gulosa), sem palavra viúva.

## 7. O que um merge pode partir

1. **Schema/tripla** (§4) — conte contra o `main` do dia.
2. **Catraca da shell** (`the_shell_only_shrinks`): linha que cresça a shell soma sobre os −94; mover, nunca subir. Re-meça.
3. **`paint_text.rs` (526) / `paint_text_elided.rs`**: linha que edite `paint_text_elided*` em `paint_text.rs` conflita (o código mudou de ficheiro).
4. **`ph2d-input`**: `NAMED`/`script_names` e o gate `os_nomes_de_tecla_sao_os_do_teclado` (101 nomes) — linha que acrescente tecla soma na lista; `Key` e o ficheiro do jogador.
5. **`ph2d-app-input-map` está em 449/449 exatos**: nenhuma folga; linha que acrescente aí corta antes.
6. **Ícones**: ordem de `enum_order_matches_svgs` (7 `kbd-*` novos) e a lista de ícones de `ph2d-editor-core`.
7. **i18n**: `lib.rs` e as chaves `input_map_*`; `a_tabela_inglesa_fala_ingles` (app só em inglês).
8. `CENAS` 19 e `.config/testes-lentos.tsv` (gerado).
9. **Escuta partilhada**: outra linha que use `capture_binding`/`take_captured_binding` agora vê F5/Ctrl/sistema passarem (recusados, não ligados).

## 8. Prova de fecho (feita na linha)

- Bateria verde: `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings`, `cargo machete`, `check-standalone-optional.sh`, `check-workflow-packages.sh`; crate nova `ph2d-input-winit` inteira; testes por nome: `input_map::` **193 passed**, `ph2d-player-controls` 39, `ph2d-input` `keyboard` 13, gates do `ph2d-editor-core` (`file_loc_cap`, `the_shell_only_shrinks`, `enum_order_matches_svgs`, `hr12_widgets_a11y`, `no_magic_numeric`), `ph2d-i18n` `a_tabela_inglesa_fala_ingles`, `ph2d-host-desktop` `the_two_normalizers_never_disagree`. **Censos combinados 133/133 e 12 de 12.** Nenhum `#[cfg(target_os` escrito ou movido.
- **Provas de mutação: 40 mutações, 37 sangraram na 1.ª rodada; as sobreviventes mortas** (`mutacao_w2.rs`, `doc_tests.rs`); as **4 leis da onda 1 continuam a sangrar**; o **buraco do F5** fechado (F5 em «press one» nunca liga; sob a escuta do ladrilho Listen o F5 liga-se/segue ao Play). **L4d equivalente por desenho** (o Esc larga a escrita); **L6f inalcançável pelas portas públicas** — ambas assumidas, não escondidas. ⚠️ `mut.py` restaura com `git checkout HEAD -- <file>`: comite antes de mutar.
- Auditoria de correção: §2/W6.

### Binário do smoke (2.ª corrida, zero `Compiling`)

```

▸ linha line_inputmap · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.33s
```

### Perfil do loop do agente (`bash scripts/agent-loop-profile.sh`, 20 sessões)

```
  ✗ paralelismo de ferramenta              1.29/passo   alvo: >= 1,5  (15% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                101   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                 151 : 49   alvo: <= 1,0  razao 3.1x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             1.6%   alvo: <= 2%  (7 de 429; o guarda R6 recusa-as desde 14/10)
  ✓ contexto relido por passo (media)         240 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               67 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## 9. Smoke (do dono)

Fotos: `docs/InputMap/fotos/` (14 cenas a 1600×1000 e 1280×720; 13 montagens nossa|maquete, `montagem_teclado_dono` — nosso teclado sobre a foto do notebook —, `montagem_b5`). O build é o da corrida acima.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && PH2D_GAME_SMOKE=19 bash scripts/ph2d-run.sh cargo run -p ph2d-host-desktop --profile smoke` (cenas 6..19; a 18 junta `PH2D_GAME_SMOKE_DRIVE=editor`).
2. Na aba Input Map, versão **Keyboard**: o teclado é o da foto (fila F, pontuação, setas em T, numérico). Passe o mouse por uma tecla: acende na hora, sem fade. Clique no Numpad 0: liga à ação aberta; Numpad Enter ≠ Enter. Windows/Fn/Print Screen aparecem apagadas e dizem porquê.
3. Reduza a janela para 1280×720: a Keyboard empilha (teclado em cima, lista rolando embaixo).
4. Campo Dead zone: clique (seleciona tudo), ↑/↓, Shift, Enter, Esc. Digite 5: a frase diz que ficou em 0.99.
5. Errado se: tecla do sistema liga, hover com atraso/sempre aceso, Tab pousa em linha escondida, Num Lock vira cadeado.

**Não smokado por humano:** clique real e gamepad físico; a única fotografia confirma desenho, não gesto.

## 10. ABERTO e achados FORA da linha

- **Acordes com Alt ligam sob a escuta partilhada** (o store não guarda estado de Alt): fica para a linha que dona o store.
- Segundas legendas do numérico (End/PgDn/setas sob o 1/3/…) não desenhadas: o binding é físico.
- Tela «Controls» do jogo **sem hover**: não há caminho de ponteiro em `host.rs` (custo de shell).
- Campos numéricos da C (se vierem a existir) arrastam ao premir; hoje a C não os tem.
- Fora da linha, herdado da onda 1: aba de região/Animated Sprite, Planner e física sem `chrome_over`; `CrispHeavyPlus` perde pingos de i/j a ~13 px negrito (app inteiro); Hierarquia trunca «Collision Shap…».
- Marcadores de conflito: nenhum. Nada visual em aberto, salvo os recusados do §2/W6.

## 11. Proposta para o `CLAUDE.md §5.1` (só na integração — DIRETRIZ §1.5.9 item 8; a linha NÃO a escreve)

Trocar o link do «Último» e o intervalo do smoke (≤ 700 B):

> - **Input Map** — aba central com três vistas (List · Keyboard · Try it), teclado de notebook inteiro (101 teclas), mapa no undo do projeto, tela «Controls» do jogador (F1 no Play), Luau pela ação (`ph2d-input-map-editor`, `ph2d-app-input-map`, `ph2d-player-controls`). Smoke `PH2D_GAME_SMOKE=6..19`. Último: [onda 2, 10/11](docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA2.md) · [plano](docs/InputMap/01_plano.md)

**Memória:** `project_inputmap_three_versions_2026_11_10.md` ganhou o parágrafo «Onda 2 entregue 10/11»; a linha do `MEMORY.md` não muda.
