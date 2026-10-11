# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, «presets de todas as mídias» (2026-10-11, prompt 17)

> **Para o AGENTE INTEGRADOR** (só por ordem EXPLÍCITA do dono). Cobre a janela do [prompt 17](../NewFeaturesWetPaint/17_PROMPT_janela_presets_todas_as_midias.md)
> sobre a base **`8f20c60de`** (= `main` no arranque; o `main` não andou). Continua o
> [handoff anterior](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-11.md) (já no `main`). O *mecanismo*, as tabelas e as recusas
> vivem em [`05_plano_da_linha_e_o_antes.md` §4b «Presets de todas as mídias»](../NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md)
> e no [`18_mapa_das_midias.md`](../NewFeaturesWetPaint/18_mapa_das_midias.md); aqui só o que decide a integração. Aberto: §8.
>
> ⚠️ **Estado no momento de escrever:** o trabalho está na árvore **por comitar** (ver §1); o commit leva o hash que o integrador lê em `git log`.

---

## §1 — Coordenadas

| | |
|---|---|
| ramo · worktree | `line/PainterWatercolor` · `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` |
| base | `8f20c60de` (= `main`); HEAD no arranque `eb953b36a` (o prompt 17) |
| diff (árvore, por comitar) | 30 ficheiros alterados (`+909 / −57`, a maior parte `cabecalhos.json` e `presets_de_fabrica.rs`) + ~25 novos (5 de código, 12 fotos do Rebelle, a fixture dos 281, a prancha, 2 ferramentas, o 18, o PNG do Wet Liner) |
| crate nova · ADR · contrato congelado | **nenhum · nenhum · nenhum** (`ph2d-tool-painter`, `ph2d-app-painter`, `ph2d-i18n`) |
| `PROJECT_SCHEMA` · registos | **não mudam** (a linha não toca `shells/desktop/src/project_schema*.rs`) |

## §2 — Superfície de colisão

A colar de `collision-surface.sh` (caminho ABSOLUTO) **pelo integrador** no momento da integração: o `main` não andou nesta janela, logo
não há ficheiros tocados pelos dois lados nem rebase; os deltas esperados são só os de baixo.

- **Ficheiros foundational tocados:** `crates/ph2d-i18n/src/app_painter.rs` (+4 linhas: duas chaves `app.painter.presets.folder.{watercolor,fluid}`, nos dois idiomas). Nada em `node.rs`/`tool.rs`.
- **Cargo.lock:** sem pacote novo. **CLAUDE.md §5.1:** só a entrada **Painter** (o link «Último» e três palavras).
- **Binários/dados novos:** 12 fotos `docs/Painter/NewFeaturesWetPaint/fixtures/rebelle/*.png` (25 no total; `oilpastel` → `oilpastel2`) e o preset `assets/brushes/WetPainter/Watercolor/Fluid/Wet Liner.ph2d-brush.png`; os outros quatro `*.ph2d-brush.png` de fábrica **mudam de bytes** (regerados).

## §3 — O que a janela entrega (condensado; detalhe no 05 §4b)

| fase | entrega | portão / prova |
|---|---|---|
| **A — o mapa** | `fixtures/rebelle_presets_281.json` (os 281 presets do Rebelle em texto, com cabeçalho) · 13 fotos novas do oráculo (25 em `fixtures/rebelle/`) · a sonda `tests/it/sonda_midias.rs` (~90 receitas, JSON sem recompilar) · a régua `ferramentas/midias_regua.py` · o [18](../NewFeaturesWetPaint/18_mapa_das_midias.md) | **Veredito:** só a família **aquarela** é pintada hoje. Caneta, nanquim e marcador passam as 7 métricas do traço e **tapam no cruzamento** (nosso 1,00 · Rebelle 0,016–0,24 — a métrica nova). Decisões pendentes do dono: 18 §5 |
| **B — a W2, a largura prometida** | **`crates/ph2d-tool-painter/src/tool/paint/wetpaint/largura.rs`** (novo: `LeiDaLargura`, `LARGURA_DO_WET_ROUND`, `f64`) · o remapeamento do raio em `dab_spec.rs::authored_spec` (ramo WetPaint) · `WetPaintState.largura` (`wetpaint.rs`) com `pub fn set_wet_largura_crua(escala)` e `wet_largura_tabela()` · `wet_preset_aplicar` fixa a lei a partir da `width_table` (sem tabela: a do Wet Round) · `wet_preset_capturar` escreve a lei activa · `preview.rs` (`CenaDoPreview`/`ChaveDoPreview`) leva-a · a ponte `ph2d-app-painter` **já não apaga** a `width_table` | `a_largura_e_a_prometida` · `a_lei_de_omissao_e_a_do_wet_round` · teste unitário em `largura.rs` |
| **B — o gerador** | 64 raios do motor de 1,5 a 1 100 · 480 quadros de água · tela direita · acima de 512 pela lei crua × 2 · **régua = a largura a ¼ do máximo da coluna** (`sonda_gestos.rs::largura_que_se_ve`) | fábrica regerada, **byte-idêntica em duas regerações** |
| **B — Strength = 1** | `authored_spec` fixa a Strength do Wet Paint em 1 (a divisão `cobertura ÷ strength` não volta exacta; `o_pincel_guardado_pinta_o_mesmo_traco` diferia 1 px, 3 de 3). Quem pintava a 1 não muda ao bit | o mesmo portão |
| **B — a física passa à lei crua** | **21 pontos de inserção em 8 ficheiros**: `grid_ratio_tests`, `mistura_tests`, `mistura_camadas_tests`, `opcoes_tests`, `mistura_wet_drip_tests`, `sonda_achatamento`, `sonda_gota`, `sonda_presets` (env `PH2D_SONDA_CRUA` / `PH2D_SONDA_TAMANHOS`) — chamam `set_wet_largura_crua(1.0)` depois de aplicar o preset. Sem isso 9 reprovavam (a cena estava desenhada para o carimbo antigo) | os portões de física, 148/0 |
| **C — o 1.º lote** | **5 presets de fábrica** (o **Wet Liner** é novo, `Watercolor/Fluid`); os cinco regerados (miniaturas + tabelas) | `cada_preset_parece_a_sua_midia`: cobertura **0,716** (Rebelle 0,653 · Round 0,184), textura **3,9 %** (1,7 · 8,5), com o Round como controlo |
| **HR-15** | `rotulo_da_pasta` em `painter_bridge_presets.rs` + chaves `app.painter.presets.folder.{watercolor,fluid}` em `ph2d-i18n/src/app_painter.rs`; o teste da ponte conta a fábrica | `cada_pasta_de_fabrica_tem_rotulo` |

## §4 — Crates e ficheiros tocados (fora de `docs/` e `assets/`)

| crate | ficheiros | natureza |
|---|---|---|
| `ph2d-tool-painter` | `wetpaint/largura.rs` (NOVO) · `wetpaint.rs` · `dab_spec.rs` · `wetpaint_preset.rs` · `wetpaint_preset/preview.rs` · 7 de testes/sondas em `wetpaint/` · `tests/it/{main,presets_de_fabrica,sonda_gestos}.rs` · NOVOS `tests/it/{midias_de_fabrica,regua_midia,sonda_midias}.rs` | a lei da largura, o preset, os portões |
| `ph2d-app-painter` | `painter_bridge_presets.rs` · `painter_bridge_presets_tests.rs` | rótulo da pasta; a ponte guarda a tabela |
| `ph2d-i18n` | `app_painter.rs` | 2 chaves, 2 idiomas |

Ferramentas (`docs/Painter/NewFeaturesWetPaint/ferramentas/`): `midias_regua.py` (nova), `prancha_midias.sh` (nova), `extrai_pinceis.py`, `fixtures_rebelle.py`, `gera_eventos.py` (alteradas).

## §5 — Portões novos, cercas e o que um merge pode partir

- **Portões novos:** `presets_de_fabrica::a_largura_e_a_prometida` (5 presets × 2, 3,5, 7, 13, 26, 55, 120, 300 px) — **antes VERMELHO** (tabelas antigas: Round raio 26 → 94 px, 120 → 343; controlo dentro: a lei crua do Round reprova), **depois VERDE** (até 100 px ± 1 px; acima ≤ 1,8 %: Round 605/600, Dry 589/600, Drip 243/240) · `a_lei_de_omissao_e_a_do_wet_round` · `midias_de_fabrica::cada_preset_parece_a_sua_midia` (a régua do Python portada em `regua_midia.rs`, conferida na mesma imagem: 0,713/4,07 contra 0,71/4,1) · `cada_pasta_de_fabrica_tem_rotulo` · teste unitário em `largura.rs`.
- **Sondas `#[ignore]` (à mão):** `sonda_ruido_da_largura` · `sonda_midias::{as_midias_com_os_knobs_de_hoje, a_lei_densa_dos_presets_de_fabrica}`.
- **Custos (correm na suíte normal):** `a_largura_e_a_prometida` **~80–100 s** · `cada_preset_parece_a_sua_midia` **~60 s**. ⚠️ **Para o integrador:** pode ser preciso um override lento em `.config/nextest.toml`, como os portões de cena anteriores da linha — **não foi adicionado**.
- **Bateria de fecho (depois do código):** cerca VERDE (0 ficheiros protegidos) · `fmt` ok · `clippy -D warnings` ok (`ph2d-tool-painter`, `ph2d-i18n`, `ph2d-app-painter`) · `ph2d-wet-paint` 148/0 · `ph2d-tool-painter` lib `wetpaint` 165 + 1, onde o 1 é **flake de carga confirmado** (`a_wet_move_costs…` 3/3 sozinho a load 11; antes `the_water_runs_at_a_usable_rate…` 2/2 sozinho) · `tests/it` 9/0 · censo do registo 134/0 · censo das camadas 237/0 · `ph2d-app-painter` 103/0 · `ph2d-i18n` 24/0 · regeração da fábrica byte-idêntica.
- **Um merge pode partir:**
  1. **As quatro miniaturas/tabelas de fábrica** e a cerca `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`: se outra linha mexer no motor do Wet Paint, **regere com `PH2D_REGERA_PRESETS=1`** e reconte, não copie bytes.
  2. **Os 21 pontos de `set_wet_largura_crua(1.0)`**: um teste de física novo vindo do `main` que meça uma lei num carimbo de tamanho fixo precisa do mesmo.
  3. **`ph2d-i18n/src/app_painter.rs`**: dois braços novos; chaves de pasta devem ficar nos dois idiomas.
  4. **Contagem de fábrica = 5** (`the_factory_presets_are_counted`, teste da ponte): reconte se o `main` acrescentar presets.
  5. **Um projecto/pincel guardado antes** leva a `width_table` antiga; a fábrica não, porque foi regerada.

### §5.1 — Mutações

| # | mutação | portão que deve sangrar | resultado |
|---|---|---|---|
| M1 | apagar o remapeamento da largura em `authored_spec` | `a_largura_e_a_prometida` | 🔴 sangrou (Drip raio 7 → 10 px, 13 → 17, 26 → 36) |
| M2 | saltar o `pigmentPerDab` ao aplicar o preset | `cada_preset_parece_a_sua_midia` | 🔴 sangrou (Liner textura 5,43 fora do vale de 1,71; controlo 8,49) |
| M3 | alterar a contagem de fábrica | `the_factory_presets_are_counted` | 🔴 sangrou (5 contra 4; asserção com `- 1`, o `[Receita; N]` não compila com a const mexida) |
| M4 | preview sem a lei da largura | o portão da miniatura de fábrica | 🔴 sangrou (Wet Drip: 8 929 px mudaram) |
| M5 | a lei crua a vazar a de omissão | o portão de bytes de `opcoes_tests` | 🔴 sangrou (S anel None/4/8 e a bancada com outro hash) |

resultado: **5 de 5 vermelhas**, cada portão visto verde antes na árvore limpa; restauro por cópia + `touch`, os 5 ficheiros iguais ao byte depois (`cmp`).

## §6 — O que só o `ship.sh` pega

O gate batched (`nextest-impacted` sobre o diff acumulado, `ship.sh`) **não foi corrido** nesta janela; a bateria do §5 é dirigida. O integrador corre 1× na árvore combinada: `foundational-integrate.sh` → `censos-da-arvore-combinada.sh` → `./scripts/ship.sh` (só por ordem do dono). Atenção aos dois portões lentos do §5 e a `typos` (texto novo em PT nos docs e nos nomes das fotos).

## §7 — Smokes

Compilar (2.ª corrida sem «Compiling»: `Finished \`smoke\` profile [optimized] target(s) in 0.27s`):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop
```

1. `Ctrl+N` → fundo **White** → **Create** → **Painter** → **Wet Paint**. No navegador do topo há **cinco** pincéis no grupo **Watercolor · Fluid**: Wet Round · Wet Wash · Wet Dry Brush · Wet Drip · **Wet Liner** (novo).
2. **A largura é o número.** Com o Wet Round, Size px **40**: o traço, depois de a água assentar, tem a largura do anel do cursor. Errado: o traço visivelmente mais fino que o anel (era ~metade antes).
3. **Wet Liner:** abre a 29 px, traço azul carregado, borda limpa, quase sem grão de papel; ao lado do Wet Round (pálido, granulado) a diferença é evidente. Errado: igual ao Wet Round.
4. Os outros três continuam com a mesma carga de tinta; o Dry Brush passou a mostrar a nuvem de fios até à largura do anel.

## §8 — ABERTO

1. **Decisões do dono** ([18 §5](../NewFeaturesWetPaint/18_mapa_das_midias.md)): (a) a **ordem** dos blocos **A** (a lei da tinta por preset: caneta, nanquim, marcador) · **B** (o caminho sem água + o papel: lápis, carvão, pastéis, aerógrafo) · **C** (os fios + a altura: óleo, acrílica) · (b) a **tinta**: ou o preset de tinta liga a mistura de pigmentos da OBRA (imediato, muda a obra inteira) ou a lei vai para o pincel (a W5 reduzida, padrão-ouro) · (c) o **Staining com K–M** do [handoff anterior §8.1](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-11.md) segue por decidir.
2. **As duas bancadas do Rebelle discordam do Round** (tinta 54 nas gotas, cobertura 0,78 nos gestos; 18 §2).
3. **O Dry Brush longe do Rebelle** (orla 0,23 contra 1,00; estrias partidas onde o dele é estriado e cheio) — os fios com física própria.
4. **A W2 é prometida a ± 2 px até 100 px e ± 2 % acima:** a lei ondula de raio para raio (franja das cerdas); 64 e 128 pontos erram o mesmo, e cinco traços ao mesmo raio variam ± 1–3 px (`sonda_ruido_da_largura`). A cura é a franja determinística dos fios.
5. **BUGS_painter [#47](../BUGS_painter.md)** (Shape Noise esconde o traço) — bloqueia o bloco B.
6. **Do handoff anterior §8, itens 4–7, ainda abertos:** fio único fino e longo (O3) · Mistura II (a)–(d) · calibração (sangria no molhado, Wash sem a linha escura, custo da Live Water, Smooth Edges) · as gotas recusadas.
7. **Nomes que NÃO existem de propósito:** nenhum preset «guache», «Flat», «caneta», «nanquim», «marcador» — a tabela do 18 §3 é o que impede um preset de mentir.
