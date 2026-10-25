# Handoff — line/PainterWatercolor, janela 10 (25/10) — a queda de FPS do dono medida e curada; Composite só no Digital; sai o «Basic brush»

Commits (sobre `ab26424e7e`): `9dddc903e3`, `4ce1155a83`, `9d8514727b`. Texto exato: `git show <hash>`. Todos aprovados no smoke do dono («Smoke OK» / «ok»).
Anterior: [`HANDOFF_..._JANELA9.md`](HANDOFF_line_PainterWatercolor_2026-10-25_JANELA9.md). Seguinte: [`PROMPT_NOVA_JANELA_11.md`](../smoke_2026-10-24/PROMPT_NOVA_JANELA_11.md).

## 1. A queda de FPS ao mesclar (`9dddc903e3`)

Cena do dono reproduzida sem janela (a abertura §2 da janela 9: tela 2048², Wet Flat, Grid 1, Flow 1, Size 122, Wet bloom 0,50, Tip pickup 0,46, amarelo sobre azul molhado).
Sonda `sonda_fps_cena_do_dono` em `crates/ph2d-tool-painter/src/tool/paint/wetpaint/molhado_tests.rs` (`#[ignore]`; imprime o HASH da tela por cena):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib wetpaint::molhado_tests::sonda_fps_cena_do_dono -- --ignored --nocapture --test-threads=1
```

Por `Move` ao mesclar (antes): esticão `molhado_estica` (scatter) 5,1 ms (serial, 81 ns/célula × 13,6 k células de janela × 4,1 janelas/`Move`) · levantar 0,6 · tip (reservatório do Tip pickup) 0,6 ·
pouso da janela 3,7 (2,1 sem a lei; janela esticada maior).

**Cura:** `molhado_estica` em duas fases. Fase 1: destinos por célula (`Fonte`/`alcance_y`) por `walk_rows2` em `Rows::Medida(PISO_NS_AGUA)`. Fase 2: pouso paralelo por faixas, na ordem das
células, **idêntico ao bit**; cópia de volta por `walk_rows5`. Piso novo `par::PISO_ALVOS_ESTICA` (`AtomicUsize`, 8192 destinos, tabela medida ao lado em `crates/ph2d-wet-paint/src/par.rs`):
piso por contagem de células estava errado, o custo por célula varia ~6× com a abertura.

**Medido** (carga 4–10, corrida limitada a CPU, mínimo): `Move` ao mesclar 11,0 → ~6–7,6 ms; pior 16,2 → ~8,8–11,4 ms.

**Gates:** `o_esticao_pousa_o_grid_do_codigo_em_serie_de_antes` (FNV `0x1659f3d6060362cb` = o grid do código serial antigo) + `o_levantar_do_molhado_e_o_mesmo_nas_duas_rotas` (existente);
varredura `crates/ph2d-wet-paint/tests/it/measure_estica_rows.rs` (`#[ignore]`). Mutação: 3 de 5 sangram; M3 (zerar a janela velha) e M4 (cor no ramo `dv≤0`) são mutantes EQUIVALENTES
(a caixa esticada contém a janela; a cor do pig-0 nunca é copiada e a 1.ª parcela real a substitui com w=1).

**Preset do Wet (mesmo commit).** A auditoria achou que o preset do Wet só gravava `SpeedSize`/`SpeedOpacity`: `PressureSCurve`, `PressureJitter` e `SpeedLag` (lidos pelo Wet em `mao_do_wet`)
perdiam-se ao salvar e vazavam do pincel anterior ao aplicar. Cura: `mao::linhas_da_dinamica_do_wet()` (não-knob ∧ `da_pressao` ∧ `lida_por(WetPaint)`) usada por `wet_preset_capturar`/`aplicar`
(`wetpaint_preset.rs`). Gate `o_preset_do_wet_leva_a_mao_sem_knob` (a lista antiga sangra); `cada_preset_de_fabrica_e_a_sua_receita` verde.

## 2. Composite Brush é só do Digital (`4ce1155a83`)

Ordem do dono. O cartão aparecia no Wet Paint e no Impasto e `composite_active()` ignorava o meio. Porta única `PaintMedia::offers_composite` (= Digital), lida por `composite_active`,
pelo cartão do painel (`BrushSettings::composite_offered`) e pela linha Strength (`composite_on`). Gates `o_composite_brush_so_aparece_no_digital` (painel) e `o_composite_so_manda_no_digital`
(ferramenta); o código antigo sangra os dois.

## 3. Sai o «Basic brush» do painel (`9d8514727b`)

Ordem do dono: o menu «Basic brush» (Digital Basic / Watercolor Basic, `PAINTER_BRUSH_PRESET`) duplicava o navegador de presets. Removidos: a linha, o popover, o decodificador, a rota de opção,
o estado pendente, a entrada de `seccoes` e 3 chaves i18n; o teste de costura saiu e `seam_wet_presets` passou a exigir a linha AUSENTE. A ferramenta mantém o id e `apply_brush_preset`
(fixture de muitos gates e porta interna das macros) — **possível rota órfã a avaliar**.

Censo `o_censo_dos_controlos_so_encolhe` perdeu duas linhas: `WetPaint × Composite` e `WetPaint × Wet bloom`. A razão da 2.ª («o censo pinta uma cor só») era falsa: `risca` cruza azul molhado com
amarelo; verificado que falhava igual com o esticão antigo, logo NÃO é efeito de `9dddc903e3`. O interruptor `Composite off` passou a ser só do Digital.
nextest painel + i18n (motivo: registo de painel) 321/323; os 2 vermelhos eram esses censos, hoje verdes por nome; clippy limpo.

## 4. Auditoria dos controlos do Wet Paint (ordem do dono; tabela entregue a ele em palavras simples)

Os controlos principais leem e DECIDEM. Suspeitas REFUTADAS: «Press não é gravado nos presets do Wet» (é); «Speed lag não é lido» (é, no caminho Stroke, armado por Speed size/opacity/hardness);
interruptor On (política de preset) funciona de ponta a ponta (`estado.lembrar` + `aplicar`). Escondido-e-lido: `ExtDryBrush` com `dry_gate=0` nos pincéis de fábrica (não aparece).
NÃO verificados no Wet: Color, Solid, Type, «On empty» (esmaecido por `VazioSemModoDeCor` sob Mix).

## 5. Perguntas do dono respondidas (sem código)

- Tempo: a simulação é fixa a 40 Hz (`SIM_HZ`, `sim.rs`) e a água pausa com a caneta em baixo, salvo Live Water ⇒ baixar a taxa não ajuda a mesclar. A física conta passos (menos passos = câmera lenta;
  dt maior = recalibrar contra o Rebelle + estabilidade) ⇒ não recomendado.
- Espaço: Grid 1 = 1 px por célula; Grid 2/4 engrossa também o pigmento (composite bilinear suave, silhueta do dab super-amostrada).
- «Pigmento fino + água grossa»: **NO-GO.** O Flow Grid já é essa divisão para velocidade/pressão; ~98 % de um passo é trabalho na resolução do pigmento (advect + `drying_pass`, tabela do doc 28 arquivado),
  logo a água grossa ganha ≤ ~2 %, e o filme grosso traz de volta a borda em degrau recusada em [`30_plano_multiresolucao.md`](../30_plano_multiresolucao.md) §0/§2.4.
  Medição decisiva e barata se reabrir: separar o custo de advect/`drying_pass` por campo (água × pigmento) em `measure_stage_cost.rs` a 2048².

### ⛔ Recusas MEDIDAS (não reconstruir)

| Recusado | Medido |
|---|---|
| Piso do esticão por contagem de células | custo/célula varia ~6× com a abertura; hoje por destinos (8192) |
| Baixar a taxa temporal da sim para mesclar | água pausada com a caneta em baixo; só vira câmera lenta |
| Água grossa + pigmento fino | ≤ ~2 % de ganho; degrau de borda (doc 30 §2.4) |

## 6. Prova de fecho

Só testes por nome (o dono proibiu a bateria): os gates acima, `wetpaint::molhado_tests`, painel `o_censo_so_encolhe`/`seam_*`, i18n. clippy `-D warnings` e `fmt` limpos nas crates tocadas
(`ph2d-wet-paint`, `ph2d-tool-painter`, `ph2d-panel-painter-layers`, `ph2d-i18n`). Suíte completa NÃO corrida (é do integrador). Contadores: nenhum registo/FIELDS/Tuning mexeu (delta 0);
3 chaves i18n a menos; o censo de controlos encolheu 2 linhas.

**Um merge pode partir:** a assinatura de `molhado_estica`; `par::PISO_ALVOS_ESTICA`; o novo `mod measure_estica_rows` em `tests/it/main.rs` do `ph2d-wet-paint`; qualquer linha que reintroduza o
«Basic brush» ou o `composite_active` sem o meio; as duas linhas do censo (a lista é catraca); outra linha que mexa em `trail/molhado.rs`.

Sondas (ignoradas, comando no cabeçalho): `sonda_fps_cena_do_dono`, `sonda_fps_mesclar`, `sonda_retangulos`. Nome atual do filtro da Borracha por meio: módulo `tool::paint::tests::borracha_por_meio_tests`,
testes `o_apagado_na_aguada_molhada_fica_apagado`, `o_follow_sem_lado_nao_muda_a_agua`, sonda `sonda_borracha_por_meio`.

## 7. ABERTO

1. Decisão do dono PENDENTE — célula do Rebelle em forma fechada (Newton; muda bits) para encurtar `a_largura_e_a_prometida`.
2. Decisão do dono PENDENTE — Borracha da Watercolor que mantém a sessão: (a) buraco limpo ou (b) levantar com mata-borrão (mapa em [`HANDOFF_..._JANELA8.md`](HANDOFF_line_PainterWatercolor_2026-10-25_JANELA8.md) §5.3).
3. NOVO — pouso da janela (passos 3–5 do transfer) segue SERIAL: ~3,7 ms/`Move` ao mesclar a 2048². O arrasto de pintura lê células escritas no mesmo passe; a única cura é um arrasto que lê um
   instantâneo (muda bits ⇒ decisão do dono). O dono pode usar Grid Size 2 (assentar 6,2 → 2,5 ms/quadro).
4. NOVO — rota órfã `PAINTER_BRUSH_PRESET` / `apply_brush_preset` na ferramenta (§3): avaliar se só resta a porta das macros.
5. Da janela 7: Accumulate ON + passagem translúcida (Flow 0,3) sem Continuous escurece a cauda; ressalto ~+10 % na entrada da janela da cauda sem Continuous; S-curve; Light scatter (não achado no
   código com esse nome); canal do «Reset to brush preset»; defeito 5 do Pen (Tilt smooth/Rotation); `ExtDryBrush` (escondido e lido); decisão 2 do dono (escondidos do Digital sem linha).
6. Da janela 4: família «mundo» das macros lê a sprite sem a tinta por assar; Borracha do Wet × Strength; preview da Borracha no Wet; Texture Eraser à Strength 1; a shell a 23 linhas do tecto.
7. Não verificados no Wet: Color, Solid, Type, «On empty» (§4).

Retirados da lista: «Wet presets não gravam Press» (refutado), «Speed lag» (agora gravado e lido), «filtro `borracha_por_meio`» (nome achado, §6).
Integração no main: só por ordem do Enio (`/pd-integracao`); a suíte completa é do integrador.
