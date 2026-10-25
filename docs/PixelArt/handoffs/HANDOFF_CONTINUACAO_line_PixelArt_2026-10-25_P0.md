# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P0 (Fundação) — 2026-10-25

> Leitor: a próxima LLM que assume a linha numa janela nova. A onda P0 está **fechada**; a **linha
> continua** (a bateria `/pd-linha-fechar` NÃO foi corrida). Nada integrado, nada pushado.
> Antes de ler: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current`
> (tem de dizer `line/PixelArt`). Lei do módulo: [ADR-0192](../../architecture/decisions/0192-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) ·
> [plano](../02_plano_modulo.md) (§5 ondas, §6 ordem, §7 licença) · [estudo do Aseprite](../01_estudo_aseprite.md) · [README](../README.md).

## 1. Identidade

- Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, merge-base `c3662cd8e4`.
- 18 commits, do `f58df7d684` ao `d523ad759f` (`git log --oneline c3662cd8e4..HEAD`).
- Última correção: `d523ad759f` (seis gates da shell que a onda acordou). Antes: `c2ca97bd69` (clippy), `eabb3c93e9` (fmt).

## 2. O que existe (ADR-0192)

**`ph2d-pixel-core`** — sem dependências (gate `the_pixel_core_declares_no_dependency` + censo de
relógio/SO/hash, em `tests/it/pure_core.rs`).
- `Sprite` RGBA/Gray/Indexed, paleta DB32, Background layer, cels esparsos.
- `apply(op) -> inverso` com `Op::Patch` (o inverso devolve a caixa do cel ao byte).
- `Stroke`: linha canonicalizada (AB == BA), `begin_replacing`.
- `History` com tecto 64 MiB por sprite (o do MVP).
- `compose_rgba` (só Normal; P3 traz os 19 modos) · `pick`.
- Formato `PH2DPIXEL` 1.0: registos etiquetados, salta etiqueta desconhecida.
- Fixtura de paridade: `crates/ph2d-pixel-core/tests/fixtures/mvp_line_cells.json`, gerada por
  `docs/PixelArt/ferramentas/extrai_linhas_mvp.mjs` (correr do **primário**: o MVP mora em
  `docs/Pixel Art/`, não versionado).

**`ph2d-pixel-render`** — `SpriteImage` (StableImage, nearest), xadrez de 16 células, grelha, borda,
cursor, `cell_at`, escada de zoom inteiro `1…96` px/célula (`zoom::{step,nearest,fit}`).

**`ph2d-pixel-io`** — só `frame_rgba` (P9 enche).

**`ph2d-app-pixel`**
- `object_add`: Add ▸ Pixel Sprite abre o diálogo New Sprite; `object_add::add` sem diálogo (testes/smoke).
- `mode::Family` (PixelSprite ▸ Edit).
- `input`: lápis `B`; borracha `E` (botão direito troca frente→fundo); conta-gotas `I`/Alt; mão `H`/Espaço;
  `X` troca cores; Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y no histórico do sprite; roda = zoom inteiro à volta
  do cursor, arestas em píxeis inteiros.
- `compose`: a lógica de entrada que a shell chama numa linha por evento.
- `draw`: overlay do viewport; publica `pick_at`/`half_extents` (clique e caixa do gizmo).
- `persist`: `to_bytes` só dos vivos; read/install.
- `frame`: smoke e reconciliação (cópia de objecto ganha cópia do documento; o mais velho por
  StableId guarda o original; enquadra ao entrar em Edit).
- `panel`, `smoke` (`PH2D_PIXEL_SMOKE=1`: sprite 32×32, cara em 4 traços = 4 passos de desfazer, já em Edit).

**`ph2d-panel-pixel-tools`** — painel id `pixel_tools` (`ph2d_editor_core::screens::hero::PANEL_PIXEL`,
ícone `LayoutGrid`): ferramentas uma por linha, frente/fundo com hex, Swap (X), paleta (esquerdo =
frente, direito = fundo, via `HitIndex::register_with_secondary`). Abre sozinho ao entrar em Edit e
fecha ao sair; Window → Pixel.

**Diálogo New Sprite** (`ContextMenuKind::NewSpriteDialog`): Width/Height 1..=65535, Color Mode,
Background, OK/Cancel, Enter/Esc, lembra as últimas escolhas.

## 3. Superfície de colisão (foundational/partilhado tocado)

Rodar `collision-surface.sh` pelo caminho ABSOLUTO em cada worktree; a coluna `base:` é o merge-base.

- `ph2d-ecs`: `PixelSprite(u32)` registado. **DELTA: ECS 108→109, espelhos render/script 109→110** —
  CONTE no merge, não escolha.
- `ph2d-component-desc`: `ObjectKind::PixelSprite`, fora do DRAWABLE; `ObjectKind` mudou-se para `object_kind.rs`.
- `ph2d-i18n`: `pixel.rs`; `TextKey` movido para `text_key.rs`.
- `ph2d-editor-core`: vocabulário `pixel_panel.rs`; `EditorAction::Pixel(PixelRequest)`; ids em
  `ids/chrome/pixel.rs`; `PIXEL_SCROLLBAR_ID = NodeId(852)` **contado à mão** (reconferir no merge);
  handlers de chrome `pixel_toggle` z=74 e `new_sprite` z=176; `HitIndex::register_with_secondary`;
  menu Window MODULE_TRUTHS 19→20.
- `ph2d-ui-testkit`: `right_click_at`.
- `ph2d-app-vec`: `is_empty_object` exclui o PixelSprite. `ph2d-app-components`: `kind_of`.
- `ph2d-app-registry-init`: sync + D6 com (PixelSprite, Edit).
- `ph2d-app-audio` recebeu os pintores de costuras/espectrograma da shell; `ph2d-app-physics` recebeu 4
  testes de dispatch da shell (cura da catraca, não feature).
- Shell: `ProjectFile.pixel_sprites` é o **último campo**; **`PROJECT_SCHEMA` 193→194** (braços: 193 +0;
  192/191 e 190/189 +0,0; blob ilegível recusa o projecto). Reconte com `python3 scripts/schema-recount.py`.
  Novos: `input_dispatch/despacho_pixel.rs`, `render_loop/fase_pixel_panel.rs`.
- Catracas DESCERAM: campos da App 172→169, `main.rs` 1061→1058, `App::new` 211→208. Shell contra o
  merge-base: 40 ficheiros, +400/−562 linhas (`the_shell_only_shrinks`: 188 895 contra 189 041 na
  medição do agente). Outra linha que suba esses números colide aqui.

## 4. O que um merge pode partir

- Contadores somados: ECS, registos de espelho, `PROJECT_SCHEMA`, MODULE_TRUTHS, NodeId 852, z=74/176.
- Catracas da shell (campos da App, `main.rs`, `App::new`): outra linha pode ter MOVIDO o mesmo número.
- `ProjectFile` com campo novo de outra linha também «último»: conflito de ordem, decida pela contagem.
- Movimentos de `ObjectKind` e `TextKey` para ficheiros próprios: conflito textual em quem editou os originais.

## 5. Prova de fecho (medida)

- clippy `-D warnings` nas 13 crates tocadas: verde (2 avisos consertados).
- Testes da onda ✓: pixel-core 20 · pixel-render 3 · app-pixel 17 · panel-pixel-tools 9 ·
  `the_new_sprite_dialog_is_alive` 4 · gates de shell/LOC/ramos do canvas · registos · `kind_of` ·
  schema · `::pixel::` 4.
- `nextest-impacted`: 19 621 testes; 6 vermelhos desta linha, consertados em `d523ad759f`; 4 de carga
  (loadavg ~92) que passam sozinhos: `a_luz_recua_quando_a_placa_recusa_a_memoria`,
  `a_cena_3_conduzida_passo_a_passo`, `a_terceira_camada_da_mesma_tinta_nao_clareia`,
  `as_bordas_nao_esbranquicam_depois_de_varios_cruzamentos` ([FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)).
- `a_altura_de_abertura_de_um_painel_so_encolhe` já é vermelho no merge-base quando corre com os
  vizinhos do módulo (isolamento entre testes, não desta linha).
- Build `--profile smoke`, 2.ª corrida: `Finished \`smoke\` profile [optimized] target(s) in 0.26s`.
- Foto da cena 1 (`fotografa_cena.sh`): sprite com a cara em Edit + painel Pixel com ferramentas e 32 cores.

## 6. ABERTO e premissas derrubadas

Aberto:
- O desenho é overlay por cima dos sprites do mundo (ordem de desenho do mundo = P14).
- Clicar nas amostras de frente/fundo não abre seletor (P2).
- Conversão RGB→Gray Rec.601 é provisória (P2).
- Sem teste de shell do caminho inteiro pick→OK→objecto (coberto no hero e na família).
- **Mutação só provada no Enter do diálogo.** Falta prova de mutação das leis do núcleo (inverso do
  Patch, canonicalização da linha, um traço = um passo) no fecho da linha.
- Pergunta ao dono sobre a licença do Aseprite (plano §7) continua aberta.
- A casa não tem marca de «projecto por gravar»: nada a ligar.
- Falta no fecho da linha: `/pd-linha-fechar`, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`.

Premissas derrubadas pela medição:
- O briefing/plano supunha `PROJECT_SCHEMA` escolhido: foi **contado** (193→194).
- O plano dizia «Hand/Slice sem ícone visível»: a Mão entrou como ferramenta H.

## 7. Smoke do dono (para o Enio, sem jargão)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. Abre já um «Pixel Sprite» com uma cara desenhada, em modo Edit, e o painel «Pixel» à direita.
3. Clique numa cor da paleta (esquerdo = cor principal, direito = cor de fundo) e desenhe arrastando no
   sprite. `E` borracha, `I` conta-gotas (ou segure Alt), `X` troca as cores, roda do rato aproxima/afasta
   (o pixel fica sempre nítido), segurar Espaço e arrastar move a vista.
4. `Ctrl+Z` quatro vezes desfaz a cara traço a traço; `Ctrl+Shift+Z` refaz.
5. File ▸ Save, fechar, abrir o mesmo ficheiro: o desenho volta igual.
6. `+` da Hierarquia (ou Shift+A) ▸ Pixel Sprite: aparece o diálogo New Sprite; escolha 64 × 48,
   Indexed, White ▸ OK: nasce um sprite branco desse tamanho, já em Edit.

Deu errado se: o desenho sai borrado, um traço precisa de vários Ctrl+Z, o diálogo cria algo diferente
do pedido, ou o desenho não volta depois de reabrir.

## 8. Próximo item

Plano §6: **P1** (Desenho: tamanho/forma do pincel, pixel-perfect, Line/Curve/Rect/Ellipse/Contour/
Polygon/Spray/Bucket, simetria, Tiled Mode) ∥ **P2** (cor, paleta, inks, Indexed completo) ∥ **P3**
(camadas, 19 blend modes). **O dono escolhe** — pergunte-lhe antes de começar.
O 1.º passo de cada onda é extrair os vetores de teste do MVP (`docs/Pixel Art/app/tests/*.test.js`)
para JSON, como o `extrai_linhas_mvp.mjs` fez (do primário), e só depois escrever o código.
