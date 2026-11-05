# HANDOFF de continuação — Bugs #57–#60 e pincéis aprovados (04/11)

Linha `line/PainterWatercolor`, worktree `Worktrees/line-PainterWatercolor`. Base `c006c3108b`, HEAD `79c7342768`. Sem push, sem integração. O dono viu o smoke de 04/11 e disse «tudo aprovado». Mecanismo e medições de cada bug: [BUGS_painter.md](../BUGS_painter.md) (#57 l.520, #60 l.535, #59 l.545, #58 l.555).

## Commits (`git log --oneline c006c3108b..HEAD`)

| SHA | O quê |
|---|---|
| `57018239f0` | #57 `outflow_rows` só lê `by0..=by1` |
| `4b74b57aaf` | #58 `ph2d-det-math` no lugar da libm do sistema (128 chamadas) |
| `daf42147fe` | `ph2d-tokens` volta a folha sem dependências |
| `7d8d27b741` | memória: mesmo traço, mesmo byte |
| `de2c0c2356` | Dabber 4 e Knife Stain |
| `d603e15284` | #59 borda pela área (Custom e Dots) |
| `b1abb522df` | Marker Liner em Mix, Bokeh com o aro do dono |
| `2f79e77681` | gate `a_borda_pela_area`, braço sem cura em processo filho |
| `b5694e75a6` | fmt do Dry Marker |
| `feabfc91fa` | #60 compositor da óptica envelhece a previsão na recusa |
| `38bf6ba3ff` | aprovação: grava miniaturas e `larguras_aprovadas.tsv` |
| `cf4aef1f34` | tabelas de largura de Dabber 4 e Knife Stain remedidas |
| `17f7a5fbe5` | pinos gravados de novo (hashes movidos pela libm fixada) |
| `17c8bce3fa` | fmt e clippy do fecho |
| `79c7342768` | cerca dos outros meios, opções gravadas de novo, exemplos refeitos |

## O que cada bug mudou

- **#57** fast-dry dependia do caminho (`outflow_rows` lia linhas fora do bbox com `uv`/`bt` velhos). Curado sem mudar pincel. Gate `o_fast_dry_de_raiz_e_o_do_instantaneo`.
- **#58** `libm =0.2.16` via `ph2d-det-math` (`DetMath`, `det_sin`, `det_powf`…) em wet-paint, painter-brush, tool-painter, painter-effects, color, pen-law, symmetry, arclen, affine (11 crates; `ph2d-tokens` fica de fora). Gate `o_motor_nao_chama_a_libm_do_sistema` (`crates/ph2d-det-math/tests/it/o_motor_nao_chama_a_libm.rs`). Prova `scripts/prova-libm/prova.sh`: 0 chamadas e 0 pincéis diferentes sob libm adversária — só nesta máquina; a prova noutro Linux vem da noite após integrar. Custo em traços reais (máquina sob carga): Dabber ×1,00, Wet Wash ×1,10 (×0,97 noutra rodada), Graphite HB ×1,02, Airbrush ×1,04, Knife Oily ×1,02.
- **#59** `FalloffCurve::weight_area` (`falloff_curve.rs`), `falloff_weight_passo` (`spec/queries.rs`), `dots_t` (`texture/patterns.rs`), `texture/pela_area.rs`; em `stamp_device.rs` a curva Custom sai do lote da placa. Degrau: Bokeh 0,59→0,37, Halftone 0,57→0,29. Custo: Halftone ×1,02, Bokeh r25 ×1,74 (0,73 ms no traço inteiro), r150 ×1,28, máscara Custom 1024² ×1,16. Gate `a_borda_pela_area` (`mede_sem_a_area` é global no processo, por isso o braço sem cura corre em filho). Decisão: Bokeh saiu de `IDENTIDADE` do Krita.
- **#60** tela congelada: `DoWorker::serve` (`offthread_otica.rs`) recusava cópia se `acc + copia_ms_ema > 25 ms` e a média nunca descia (1920×1080 em diante, perfil smoke). Cura: cada recusa multiplica a previsão por 0,8. Gate `o_fan_brush_pinta_na_tela_grande_o_que_o_preview_mostra` (`preview_paridade_tests.rs`, ~17 s). Efeito: pincéis de água em tela grande parecem mais molhados durante a pintura (Water Brush 128 %, Blender Soft 113 %, Blender Hard 93 %, Blender Dots 68 %, Wet Drip 59 %…). NÃO medido o quadro no app com a água visível; não sabemos a tela do dono.

## Receitas aprovadas

- Marker Liner Multiply→Mix (Marker Chisel e Dry Marker: em estudo, não mudados).
- Bokeh, curva `(0,.28) (.756,.31) (.79,.49) (.852,.62) (.92,.47) (1,0)`, copiada a olho da foto do dono.
- Dabber 4: grain_depth 1→0,9, spacing 0,3→0,2, jitter_spacing 0→0,6, jitter_scale 0→0,05, dab_flatten 0,6→0,3.
- Knife Stain: grain_depth 1→0,5, pigmentPerDab 1200→250, Constant→Smooth, hardness 1→0,8.
- Rebelle não foi aberto: só capturas existentes; não há captura de Stain. Memória: [project_brush_tweaks_owner_approved_2026_11_04](../../../project-memory/project_brush_tweaks_owner_approved_2026_11_04.md).
- Tabelas de largura remedidas (raio 26: Dabber 4 54,94→52,01; Knife Stain 49,64→51,77). Abaixo do raio 2,8 do motor o véu do Knife Stain não atinge a régua (`sem_tinta_ate`): pincel muito pequeno dele deixa traço muito fraco; a confirmar se incomoda.

## Pinos gravados de novo (com atribuição)

Verdes em `c006c3108b` e `57018239f0`, vermelhos desde `daf42147fe`; o HEAD com a det-math devolvida à libm fica verde, logo a causa é a libm fixada. `os_outros_meios_nao_mudam` (cerca do dono, 07/10) foi gravada de novo UMA vez: Digital igual, Watercolor 1 byte e Impasto 2 bytes em 2 560 000, todos de 1 nível; registado no cabeçalho do teste. Isto contraria o «nunca regravar» desse cabeçalho e foi decidido sob a aprovação do dono de 04/11 — **a janela principal tem de dizer isto ao dono**.

## Fecho

fmt/clippy verdes nas 11 crates antes do último commit (depois só hashes); gates por nome verdes, alarmes de pincel verdes. NÃO correu suíte completa (é da integração).

## Pendências técnicas

- `presets_matriz.py` quebra com `AssertionError: (137, '142')` (regex não aceita cinco knobs de `tuning/defs.rs`): `F1_presets_de_fabrica.md` defasado desde 31/10.
- `bancada_da_agua` −30 % de CPU possível trocando para instantâneos (não feito).
- `caligrafia_marcadores.rs` 706 linhas e `texturas.rs` ~1040 linhas, já acima do limite.
- A fila pesada guarda ainda a entrada antiga `corre_f0.sh`.
- Outros padrões com degrau (`Bricks`) e a pré-visualização do grão no painel ainda leem num ponto (#59).
- Fora da cura da libm: vetor, Quadro, `ph2d-grid`/`ph2d-warp-style`.

## PRÓXIMO (ordem aprovada pelo dono, smoke a cada etapa)

1. Pontas soltas da E10: E10c2, `levelingOwnStroke` sentinela (R23), Bug #56 parede entre traços.
   ⚠️ Nota: a R23 era do `textureSizeExpAcross`, não do `levelingOwnStroke` (feita em 04/11; ver [BUGS #56/#61](../BUGS_painter.md)).
2. E11: placa de vídeo lê meio e pasta.
3. E12: apagar a lei velha (23 referências a `LeiDaTinta::Velha`) depois de o portão-mãe passar inteiro; plano em [F4_2_plano_wet_paint.md](../Arquitetura/F4_2_plano_wet_paint.md) l.266–280.

Decisões do dono ainda abertas: Bug #54 (branco opaco em papel de cor; 25 miniaturas até 10 níveis) e a maquete F5 da ajuda.

## Smoke

Nada pendente desta onda: já aprovada. Para rever a tela grande (#60): abrir uma tela 4096×3072, pintar com Water Brush e ver se a água aparece enquanto pinta; deu errado se só aparece ao levantar a caneta.
