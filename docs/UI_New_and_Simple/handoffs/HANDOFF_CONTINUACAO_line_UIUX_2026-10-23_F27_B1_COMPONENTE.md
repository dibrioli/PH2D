# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 23/10 — F27 B1: a família COMPONENTE e a Câmera de ponta a ponta

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `972f411a6f`. Anterior:
  [F27 Bloco A](HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_BLOCO_A.md); integração a fechar:
  [F18–F26](HANDOFF_INTEGRACAO_line_UIUX_2026-10-22_F18_A_F26.md).
- Fonte do desenho e das ondas: [`../F27_bloco_B_plano.md`](../F27_bloco_B_plano.md) (censo, decisão da
  família genérica, B1/B2/B3, Bloco C). **Não está copiado aqui** — leia-o antes da B2.
- Esta janela: Bloco A fechado (smoke do dono OK em 23/10, depois do reparo do «Object» invisível, §7 do
  handoff do Bloco A) + a onda **B1**. A próxima LLM abre **janela nova** para a **B2**.
- Decisão do dono, 23/10: `ph2d-panel-painter-layers` (Pintura) fica **FORA** do Bloco B — a linha
  PainterWatercolor volta a ser trabalhada. Não tocar.

## 1 — Commits desta janela

| commit | o quê |
|---|---|
| `dd3680dd77` | Bloco A: lado B da junta + motor com faixa (cena `=27`) |
| `b29bbe562e` | Bloco A: handoff de continuação + índice |
| `359b6881ec` | Bloco A, smoke do dono: o «Object» recusado arma uma escolha que SE VÊ |
| `e0bd7d8248` | Bloco A: §7 — o reparo do smoke do dono |
| `a1b7353b77` | **B1**: 17 ficheiros, +1054/−3 (código, 3 pares de testes, cena `=28`, plano) |
| (este) | este handoff + regeneração de `README.md` desta pasta |

## 2 — Como foi construído (B1)

### 2.1 `ph2d-ecs::canal_componente` — a família `COMPONENTE = 6` (append em `canal.rs`)

- **Chave** = `campo = field_id << 2 | faixa` (faixa 0–3 = x/y/z/w de Vec2/Vec4/Color), `i:j` = os 32 bits
  altos do `stable_type_id` (o id travado dos prefabs, `stable_type_ids_are_locked_in`).
- `info()`/`membro()` resolvem pelo catálogo (`ph2d-component-desc`). `grupo` = índice denso
  `(componente, campo)`, **só em sessão** (nunca gravado); `por_i = false`.
- `ler_valor`/`escrever_valor` = a FORMA do valor guardado: real, faixa de array/objecto, inteiro com
  sinal, booleano, opção de `View::Options`/`Toggle`.
- **Fora** (`faixas = 0`, sem canal): refs, `View::Named`/`Not`, Text, Curve, Gradient, Palette, Marker,
  Enum sem opções.
- Gates (`canal_componente_tests.rs`, 170 linhas): nenhum componente colide nos 32 bits (>100 no
  catálogo); `field_id` cabe (< 16384) e é único; ida e volta de toda chave (>300 canais); a forma do valor.

### 2.2 `ph2d-app-components::canais_componente`

`campos/ler/escrever/publicar/aplicar` sobre `slot_in` + `property_laws::sanitize` (a mesma porta dos
scripts). Pré-visualização pelo `FieldDrive` (mesmo ledger dos scripts); eco por `CanaisLidos`.
Ligado em `fase_timeline_drain` (`aplicar`) e `fase_autokey` (`publicar`).

### 2.3 `marcas_inspector`: tabela `COMPONENTES`

Controlo → `(componente, field_id, faixa)`, listas derivadas uma vez (`OnceLock`). Cobre Câmera,
Seguimento, Limites e os 32 bits da máscara. **É a tabela que a B2 substitui por derivação.**

### 2.4 Gate `architecture_every_editable_field_is_a_channel`

`sections/camera.rs` entra no ESCOPO (`camera`/`follow`/`limits_body` em `POR_FUNCAO`, `Canal`).
Texto i18n: 2 chaves em `ph2d-i18n/src/canais.rs`.

### 2.5 Cena `PH2D_AUTOKEY_SMOKE=28` (`crates/ph2d-app-vec/src/autokey_smoke_camera.rs`)

Auto Key grava altura, offset e dolly nos quadros 12 e 24; no q18 a curva conduz a câmera
(altura 8,0). Fotografada (1930×1040): Inspector «Game Camera» com losango animado em Height, Offset,
Dolly e ponto livre nos demais; dope-sheet com «Camera · Height / Offset / Dolly». Log:
`q18 (a meio): altura 8.00 offset [1.00, 0.50] dolly 0.25 … OK`. Trilhas: `camera_fields.0` 1×2ch ·
`.1` 2×2ch · `.5` 1×2ch.

### 2.6 Achado na foto (para a B2)

**«Look Through»** (`INSP_CAMERA_PREVIEW`) é estado de VISTA, não documento, e pinta a coluna do
losango sem canal — um controlo que mente. Cura na B2: classificar `NaoAnimavel` e não pintar decorador.

## 3 — Superfície de colisão (um merge pode partir)

- **`canal.rs` (ph2d-ecs):** `familia::COMPONENTE = 6` + dois braços de `match` (`membro`, `info`).
  Outra linha que acrescente família com id 6 colide; **conte no disco**, nunca escolha.
- **`lib.rs`** de `ph2d-ecs`, `ph2d-app-components`, `ph2d-app-vec`: um `mod` cada (append).
- **Shell (+7 linhas):** `fase_autokey.rs` (+2), `fase_timeline_drain.rs` (+5). A catraca
  `the_shell_only_shrinks` ficou verde.
- **`marcas_inspector.rs`:** tabela nova; outra linha que mexa nesse ficheiro conflita na tabela.
- **Gate `architecture_every_editable_field_is_a_channel`:** ESCOPO ganhou `camera.rs` (lista). Outra
  linha que também edite a lista faz conflito textual trivial.
- **`autokey_smoke.rs`:** despacho da cena 28 (o número 28 é o próximo livre **nesta** linha — recontar
  na integração).
- **Nada foundational além do id 6:** o commit não toca `PROJECT_SCHEMA`/`DOC_VERSION`/`AUTOKEYED`;
  nenhum contrato §6.
- **Contadores como delta:** módulos de teste novos 3 (`canal_componente_tests` em ph2d-ecs,
  `canais_componente_tests` e extensão de `marcas_inspector_tests` em app-components); gate
  `every_editable_field` +26 linhas (6 testes no total); famílias de canal +1 (`COMPONENTE`); cenas de
  smoke +1 (`=28`); chaves i18n +2 linhas em `canais.rs`.

## 4 — Smoke do dono (em linguagem simples)

1. Rode `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=28 cargo run -p ph2d-host-desktop --profile smoke`.
2. Espere uns 2 segundos: a cena grava sozinha e pára no quadro 18 com «Camera» seleccionada. No
   terminal aparece a linha `q18 … OK`.
3. No Inspector, secção «Game Camera»: Height, Offset e Dolly têm o losango; na linha do tempo há três
   faixas «Camera · …».
4. Arraste a agulha do tempo entre 0 e 1 s: o número de Height muda (de 6 a 10).
5. Clique o losango ao lado de «Priority»: tem de nascer uma faixa «Camera · Priority».
6. Deu errado se: não há losangos, faltam faixas, ou o terminal diz «PARADA».

Estado: **smoke do dono ainda não feito** (B1).

## 5 — Verificação

- `clippy -D warnings --all-targets` limpo: `ph2d-ecs`, `ph2d-app-components`, `ph2d-app-vec`,
  `ph2d-i18n`, `ph2d-host-desktop`, `ph2d-editor-core`.
- Testes por nome: `ph2d-ecs` canal 20/20 · `canais_componente` 4/4 · `marcas_inspector` 8/8 ·
  `architecture_every_editable_field_is_a_channel` 6/6 · `ph2d-i18n` 7/7 · `the_shell_only_shrinks` 1/1.
- Cena `=28` fotografada e lida (§2.5).
- **Não feito:** prova de mutação da B1; suíte `nextest-impacted` (corre no fecho da linha).

## 6 — ABERTO, por ordem

1. **B2 (janela nova) — as outras ~67 secções do Inspector num lote.** Censo (explorador, 23/10): 68
   ficheiros de `sections/` fora do ESCOPO, 108 chamadas PORTA + 19 `SEM_COLUNA`, **0 com canal**.
   - Tabela `COMPONENTES` passa a derivada do catálogo por convenção (ou continua tabela, decidir).
   - Helpers locais passam pelos de `rows.rs`: `player_rows.rs` (64 ids, 0 chamadas visíveis ao gate),
     `particles.rs` (`num_row` próprio).
   - ESCOPO do Inspector = a crate inteira; a lista de isenções só diminui.
   - «Look Through» → `NaoAnimavel` sem decorador (§2.6).
   - Por decidir: textos (`FieldKind::Text`) como canal → `TextosDosCanais`.
2. **B3 —** os quatro painéis sem lado (hierarchy, skeleton, timeline, flip-frames: só `SEM_COLUNA`; a
   regra `toda_crate_que_pinta_campos_escolheu_o_seu_lado` só olha PORTAS — passa a contar `SEM_COLUNA`);
   `easing_rows` está `NaoAnimavel` com razão contraditória; `TEXTO_CONTEUDO` (`canal_vetor` 199) sem
   controlo no painel.
3. **Bloco C —** medir `marcas()` e `linhas::layout` com N canais (a família genérica multiplica as
   linhas do dope-sheet). Tabela da medição no plano, antes de qualquer tecto.
4. **Bloco D —** menu do botão direito na linha de grupo do dope-sheet; ADR «o canal de qualquer
   propriedade» (número contado no disco).
5. **Item 9 do F18–F26:** chaves `canal.junta.*` só vistas no locale base.
6. **FORA desta linha:** `ph2d-panel-painter-layers`, gravidade animável, física a meio da simulação,
   `IkBendSide`, Rope Length da Pulley, áudio do mixer.
7. Próxima cena de smoke: `PH2D_AUTOKEY_SMOKE=29`.
