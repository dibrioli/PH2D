# Handoff de integração — linha PainterWatercolor · 13/11 (onda n.º 7): desempenho 4K — sim a 6 traços, vidro em tiles (ADR-0204), «+» sem diff, gestos de camada

> A linha NÃO está integrada; quem o dono nomear integra. Leitor: o integrador e a próxima LLM da linha.
> Anteriores (continuam a valer): [n.º 6 PRIMEIRO_TOQUE_4K](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_PRIMEIRO_TOQUE_4K.md) ·
> [n.º 5 ONDA5_VIDRO_E_MEDIA](HANDOFF_CONTINUACAO_2026-11-12_ONDA5_VIDRO_E_MEDIA.md) ·
> [n.º 4 SELECAO_CAMADAS_E_RITMO](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_SELECAO_CAMADAS_E_RITMO.md) ·
> [MISTURA_E_ENVIO](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_MISTURA_E_ENVIO.md) ·
> [LUZ_E_CAMADAS](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_LUZ_E_CAMADAS.md) ·
> [DESEMPENHO_TOTAL](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_DESEMPENHO_TOTAL.md).
> Notas da onda (a verdade dos números): [`N7_ITEM1_sim6.md`](../missao_4k/medicoes/N7_ITEM1_sim6.md) ·
> [`n7_assenta/`](../missao_4k/medicoes/n7_assenta/README.md) · [`N7_ITEM2_vidro_tiles.md`](../missao_4k/medicoes/N7_ITEM2_vidro_tiles.md) ·
> [ADR-0204](../../architecture/decisions/0204-o-vidro-entra-no-journal-por-tile-e-o-desfazer-do-traco-deixa-de-segurar-o-plano.md) ·
> [`N7_ITEM3_mais.md`](../missao_4k/medicoes/N7_ITEM3_mais.md) · [`N7_CAMADAS.md`](../missao_4k/medicoes/N7_CAMADAS.md) ·
> [`N7_FECHO.md`](../missao_4k/medicoes/N7_FECHO.md). Backlog: [`../missao_4k/BACKLOG.md`](../missao_4k/BACKLOG.md) (B13.12, B18.3–B18.12).

## 0. Superfície de colisão

- Base: `main` = `4f46a3573c` = merge-base; `git log HEAD..main` vazio em 13/11. **Não rebaseie** salvo se o `main` andar.
- Esta onda = `da83348ee0..6d13348bb0`, 45 commits (41 de trabalho + 4 de fecho). Código: **70 ficheiros, +5 484/−255**, em
  3 crates: `ph2d-tool-painter` (57), `ph2d-wet-paint` (11), `ph2d-app-painter` (2). **Zero** em `shells/`, `Cargo.*`,
  `project-memory/`, schema; `CLAUDE.md` só a linha do §5.1.
- Contadores (DELTA da onda): `PROJECT_SCHEMA` 0 · ADR **+1** (0204; a linha inteira cria 0201–0204 — reconte contra o `main` do
  dia, o próximo livre no disco é 0205) · cenas de smoke 0 · tripla 0 · gates novos **35** (§9) · Cargo.lock sem pacote externo novo.
  O `collision-surface.sh` da linha inteira mostra `PROJECT_SCHEMA` 239 (base 237): vem das ondas anteriores, não desta.
- ⛔ **Tecto de LOC (`workspace_src_files_under_loc_cap`, 700) — VERMELHO na árvore da linha:**

  | ficheiro | início da onda | HEAD | `main` |
  |---|---|---|---|
  | `ph2d-tool-painter/src/undo_planes.rs` | 668 | **764** | 520 |
  | `ph2d-tool-painter/src/tool/paint/watercolor_vidro_journal_tests.rs` (novo; `*_tests.rs` em `src/` NÃO é excluído) | — | **781** | — |
  | `ph2d-tool-painter/src/tool/paint/watercolor_backdrop.rs` | 725 (já vermelho desde a n.º 6) | **737** | 575 |

  A cura é CORTAR por responsabilidade (irmão `*_tests.rs` / módulo), nunca subir `FILE_OVERAGE_OK`. É o 1.º item do ABERTO (§6).
- O que um merge pode partir (desenho mudado, não só linhas):
  - `undo_delta.rs` (`StoredPlane::Nova`), `undo_delta_aparece.rs` (`StoredEntry::Aparece`), `undo_planes.rs` (`vidro_map`,
    `tira_a_tela_nova`/`poe_a_tela_nova`, `MAIS_SEM_DIFF`), `undo_window.rs` + `undo_window_vidro.rs` (`VidroJournal`),
    `undo_elide.rs`/`undo_record.rs` (o vidro é a 4.ª família de `ElidedRelief`): quem mais escrever entradas de undo por baixo.
  - `tool/layers/cache.rs` (`a_troca_mantem_a_figura`/`_com`/`_salvo_a_tinta`, `alcance_da_tinta_de`, `mede_a_estrutura`,
    `take_estrutura_confinada`) e `layers/mutate.rs` (`set_active_layer`, `tela_transparente` com `PISO_DO_MMAP`).
  - `ph2d-app-painter/src/painter_gpu_preview.rs::regiao_suja` aceita a lista nova quando a ferramenta entrega `(antes, depois)`.
  - `compositor/abaixo.rs` (`composite_below_cortavel`, `Corte`): `composite_below_com` passa a ir por ela.
  - `ph2d-wet-paint/src/solver/advect_*` (`COMMIT_NO_BLOCO`, `advect_lanes.rs`) e `assenta/newton.rs` (o Newton saiu de `assenta.rs`).
- Testes que viram chaves globais: os novos tomam a trava das chaves por fio (as chaves desta onda são `thread_local`/por fio). Um
  teste novo que vire `TROCA_RECOMPOE_COMO_ANTES`, `VIDRO_PELO_JOURNAL` ou `MAIS_SEM_DIFF` sem o molde por fio volta a ser flaky.
- Aviso herdado do `main`: `shells/desktop/src/main.rs:941` (braço `WindowEvent::CursorLeft` duplicado). Para o integrador.

## 1. As 3 decisões do dono de 13/11 (verbatim: «1- tente · 2- pode · 3- sim») e o que se fez

| # | decisão | o que se fez | estado |
|---|---|---|---|
| 1 | sim a 6 traços em tempo real (mudar a aritmética da `assenta`) | o produto ao bit foi de **0,77 → ~0,91 sim-s/s** (média, 4K, 6 traços); a aproximação fica atrás da chave `PH2D_ASSENTA_PALPITE=1` (0,97), **DESLIGADA**, à espera do olho do dono | parcial: o alvo 1,0 não sai sem `unsafe` (AVX2) ou mudar a física |
| 2 | desfazer em janela/tiles do vidro (e do canvas) por traço | [ADR-0204](../../architecture/decisions/0204-o-vidro-entra-no-journal-por-tile-e-o-desfazer-do-traco-deixa-de-segurar-o-plano.md) escrito e **implementado** (passos 0–8), chave `VIDRO_PELO_JOURNAL` ligada | feito; aberto o `fork_par_raw` da tela (§6) |
| 3 | o «+» a 4K com o desfazer a lembrar menos passos | `StoredPlane::Nova`: o «+» sem diff, chave `MAIS_SEM_DIFF` ligada | feito |

## 2. Item 1 — a sim do Wet a 6 traços (`2aa76c2a4e`, `787ef3bf42`, `6c1a3527b8`, `2c3a2185b0`)

Régua: `crates/ph2d-wet-paint/examples/sonda_estagios_4k.rs` (3840×2160, 6 faixas `Lei::Cobre`, 16 fios, `smoke`), rotas
intercaladas no MESMO processo, a régua é a MÉDIA. Hash do produto `a9bac64ac5b857c8` igual em todas as curas.

| rota | média ms | sim-s/s | nota |
|---|---|---|---|
| antes da onda | 32,4 | 0,77 | load 4,8–7,5 |
| tecto `z` (a `assenta` de graça, resultado errado de propósito) | 27,1 | 0,92 | nenhuma aritmética da `assenta` chega a 1,0 |
| `COMMIT_NO_BLOCO` (o complemento da troca dentro dos blocos do núcleo) | 30,7 | 0,82 | −1,7 ms do advect em todo passo |
| `TRANSPORTE_LANES` (blocos de 8 lanes, cor e divisão só onde alguma lane troca) | 27,8–28,2 | 0,89 | −0,3 ms do núcleo (2–3 %) |
| **produto exacto no fim (Newton com as voltas em execução)** | **27,4** | **0,91** | `2c3a2185b0`; mediana ~23 ms = 1,0 |
| `PH2D_ASSENTA_PALPITE=1` (Newton com 0 voltas) | 25,8 | 0,97 | erro no ecrã: pior 20,4 níveis, médio 0,91 |

- A média é dos PICOS (p90 ~68 ms: secagem 1/`dry_every`, campo de fluxo 1/4 e project 1/3 a coincidir). Os picos têm
  dependência de dados em série: adiantá-los muda o bit.
- **O palpite a olho** ([`n7_assenta/README.md`](../missao_4k/medicoes/n7_assenta/README.md)): alarmes 77 → **80** (largura de
  **Smear Soft, Smear Spikey, Water Brush** a mais); **15 miniaturas** pintam diferente (Blender Dots/Hard/Soft, Ink Brush, Broad Nib,
  Dip Pen, Ink Bullet, Marker Bullet/Chisel, Smear Flat/Soft/Spikey/Wet, Water Brush, Wet Dry Brush); **Wet Round e Watercolor Basic
  iguais ao bit**. 16 pranchas lado a lado (esquerda exato, direita palpite) na mesma pasta, por ex.
  [Smear Wet](../missao_4k/medicoes/n7_assenta/Smear_Wet_exato_vs_palpite.png),
  [Water Brush](../missao_4k/medicoes/n7_assenta/Water_Brush_exato_vs_palpite.png). Só o dono liga a chave por omissão.
- **Tecto sem `unsafe`: ~0,92–1,0.** O próximo degrau do núcleo é AVX2 com gather por `core::arch`, barrado por
  `unsafe_code = "forbid"` na workspace inteira: pede a decisão de política de abrir `unsafe` num módulo isolado (dono/ADR).
- Gates: `o_complemento_nos_blocos_da_o_commit_a_parte_ao_bit`, `o_transporte_em_lanes_da_o_escalar_ao_bit`,
  `o_palpite_do_assentar_fica_perto_da_conta_exacta` (tecto 21 níveis; controlo 107).

## 3. Item 2 — o vidro no journal por tile, ADR-0204 (`552e79d66c` … `80559a3331`, `e409ad95e2`)

Passos 0–8 em [`N7_ITEM2_vidro_tiles.md`](../missao_4k/medicoes/N7_ITEM2_vidro_tiles.md). Chave `VIDRO_PELO_JOURNAL` (`true`; `false` =
o oráculo dos portões).

| medida | antes | depois |
|---|---|---|
| cópias do plano do vidro, traços 2.º..8.º (WB) | `[1;7]` | **`[0;7]`** |
| donos do plano em repouso depois do 1.º / 2.º traço | 3 / 2 | **1 / 1** (só o mapa) |
| 1.º traço na base depois do «+» | 1 cópia | **0** (asserido) |
| 2.º pen-down WB a 4K, `ab` intercalado (load 5–8, indicativo) | mín 14,0 · média 17,2 ms | **mín 11,8 · média 13,6 ms** |
| gesto aberto 520×390: passo · sessão · plano | — | 688 128 B · 917 504 B · 1 419 600 B; a pilha retém 4 = 4 do oráculo |

- D1 `VidroJournal` (molde do `ReliefJournals`) e a porta `plane_fork::fork_vidro`; D2 elisão do `before`/cursor + `StoredEntry::Aparece`
  (o plano que NASCE no passo guarda só a janela); D3 `BaseCongelada = (Weak, camada, SessaoDoVidro)`, o `vbase` é uma janela densa;
  D5 o journal da sessão conta no tecto (`bytes_do_instantaneo`); D7 censo dos `fork_canvas`: nada a curar.
- Curas achadas pelo caminho: a sessão molhada atravessava para a CÓPIA da camada (`GuardaDaSessao { tela, camada }`); restauro e poda
  largam a sessão; a crate não compilava sem `debug_assertions` (`TileJournal::get` só de debug chamado da produção).
- Gates (17): ver a lista do §9; mutações vermelhas por passo na nota. Duas sobrevivem por desenho (registadas na nota, passo 6).

## 4. Item 3 — o «+» a 4K e o fundo do toque (`daece51c64` … `d218f859f4`)

| cura | chave (em execução) | antes | depois |
|---|---|---|---|
| `StoredPlane::Nova`: o «+» guarda a tela que saiu, sem diff; o refazer recria o neutro | `MAIS_SEM_DIFF` | `UndoEntry::split` 14 ms; o 1.º traço na nova copiava 33 MB | sem split; 0 donos extra no 1.º pen-down |
| a tela nova vai SEMPRE ao `mmap` (pede 32 MiB + 4 KiB e `truncate`) | `TELA_NOVA_PELO_MMAP` | `tela_zero` 18–29 ms (o calloc saía do heap com `memset`) | **< 0,3 ms** |
| a compressão da adormecida espera 300 ms de silêncio de caneta (tecto 3 s; acordar cancela) | `ENCOLHE_NO_SILENCIO` | «+» Wet 33,6–39,0 ms; `adormeceu_muda` 1,6–14,9; 1.º preview até 29,7 | **6,0–9,8 ms**; 1,6–1,7; ≤ 5,9 |
| janela da tinta por palavras `u64` (a paralela SAIU: pior sob contenção) | `JANELA_DA_TINTA_POR_PALAVRAS` | 12,2 ms sob contenção · 3,0 calmo | 5,7 · 2,0 ms |
| o fundo do toque aquece na troca de camada e no «+» | `FUNDO_AQUECIDO_NA_TROCA` | 1.º pen-down WB na nova 32,7–33,6 ms | 12,0–17,6 ms (mas o clique subia a 16–22) |
| … num fio, com pool rayon próprio e cancelável entre faixas | `FUNDO_AQUECIDO_EM_FUNDO` | clique do «+» 40–171 ms (2→33 camadas) | **0,06–0,18 ms**; toque igual |
| … também na troca de pincel (`preset_aplicar`); o fio descartado pára entre faixas | (as mesmas) | — | 5/512 faixas depois do corte, sai em 0,59 ms |

- A contabilidade (decisão 3): o «+» pesa 1 plano `Nova` + 1 `OnlyAfter` = 66,4 MB a 4K. Sobre camada opaca lembra MAIS (−33,2 MB);
  sobre camada rala perde até ≈ 1,5 traço por «+» no histórico.
- O controlo de `o_ctrl_z_de_um_traco_no_papel_e_confinado_em_cada_meio` acompanha o ADR-0204 (`v=<janela>` com a chave; a rota antiga
  no mesmo processo continua `v=WHOLE`) — `b8be38f548`.
- **Item 5 (espera do pen-down pelo motor, 3,8 ms + `abre_o_passo` 2 ms): documentado, NÃO curado.** O `bring_home` espera ≤ um estágio
  do worker; abrir o passo antes do `recv` fotografaria um estado a meio do estágio. Não re-medido.

## 5. Gestos de camada, ondas 1–5 (`305651f43d`, `50e6660aad`, `79d7c4768c`, `88c047b67d`, `234ef11b64`)

Notas: [`N7_CAMADAS.md`](../missao_4k/medicoes/N7_CAMADAS.md). Chaves A×B: `PH2D_TROCA_RECOMPOE=1` / `TROCA_RECOMPOE_COMO_ANTES` (troca,
tinta viva, sujo pendente, Wet dup/apagar/grupo) e `PH2D_ESTRUTURA_RECOMPOE=1` (placa compõe a tela inteira).

| onda | mecanismo | cura |
|---|---|---|
| 1 | a cura B18.3 nunca pegava no app: o papel liga o relevo (`!impasto_visible()`) e a pista da placa drena com `composited = None` | `a_troca_mantem_a_figura` aceita relevo sem traço aberto e o slot da placa em dia; drenagem CPU 60,9 → 0 ms, dobra da placa 5,1 → 0 |
| 2 | WB: a troca caía em `!papel_atravessa_vidro()`; apagar/duplicar/grupo = `invalidate_composite` | a troca com aguada no papel de cor não recompõe; apagar/duplicar/grupo marcam só a janela da tinta (`alcance_da_tinta_de`) |
| 3 | Wet: a tinta viva recusava toda troca | marca só a janela da que adormece e da que acorda; troca + quadro 45–64 → 8–17 ms |
| 4 | no APP a sim viva deixa um sujo PENDENTE no clique (os testes de relógio fixo não o vêem) | um sujo pendente confinado não impede manter a figura; drenagem Wet 41,7 → 6,5 ms, digital 45,8 → 2,0 |
| 5 | o piso ~24 ms era a PLACA: `regiao_suja` exigia `ops` iguais | a ferramenta entrega `(antes, depois)` da lista achatada e a placa compõe só o sujo; Wet dup/apagar/grupo marcam a janela na CPU |

### Matriz do app (4K, Textured Paper, 15 pincéis; `mede_pen_up_4k.sh matriz 3840x2160 papel`, ms)

Fontes: [`DEPOIS_N6.md`](../missao_4k/medicoes/DEPOIS_N6.md) (fim da n.º 6, load 2,7 → 6–8) · `N7_MATRIZ_resumo.txt` / `N7_MATRIZ_NOVA_resumo.txt`
(depois dos itens 1–3, antes das ondas de camada, load 0,4) · **`N7_MATRIZ_FIM2_resumo.txt` / `N7_MATRIZ_FIM2_NOVA_resumo.txt`** (final, HEAD
`234ef11b64`, load 3,7 / 0,9). Comparação indicativa (corridas diferentes); o A×B fino está nas notas.

Camada de base (trocar · voltar · duplicar · apagar · grupo · «+»):

| pincel | n.º 6 | antes das ondas de camada | **final** |
|---|---|---|---|
| Chalk (digital, igual nos outros secos) | 33,8 · 33,5 · 41,9 · 34,8 · 41,3 · 14,0 | 33,2 · 33,1 · 38,3 · 34,2 · 41,0 · 12,9 | **11,9 · 11,9 · 11,9 · 11,0 · 11,7 · 13,4** |
| Watercolor Basic | 70,8 · 70,9 · 78,6 · 69,8 · 69,3 · 14,0 | 73,8 · 71,1 · 71,1 · 66,9 · 69,5 · 14,0 | **13,8 · 13,2 · 12,1 · 12,1 · 11,9 · 14,2** |
| Express Oil | 64,4 · 79,6 · 42,2 · 35,5 · 43,7 · 116,8 | 64,7 · 36,8 · 39,3 · 35,5 · 41,2 · 22,6 | **22,5 · 28,2 · 10,9 · 11,7 · 11,1 · 28,5** |
| Filbert | 123,2 · 289,6 · 46,7 · 36,4 · 42,4 · 30,4 | 65,9 · 36,0 · 39,0 · 33,9 · 41,2 · 25,1 | **22,7 · 22,2 · 11,8 · 11,9 · 12,1 · 26,9** |
| Wet Liner | 62,0 · 90,7 · 38,8 · 32,5 · 41,3 · 105,8 | 60,3 · 36,8 · 37,3 · 34,3 · 41,0 · 26,6 | **21,9 · 20,1 · 11,1 · 11,8 · 11,8 · 24,8** |
| omissão impasto | 36,7 · 39,9 · 41,8 · 35,3 · 42,6 · 17,6 | 34,7 · 34,3 · 40,0 · 33,5 · 41,2 · 19,2 | **12,9 · 12,9 · 12,0 · 11,9 · 11,9 · 20,8** |

Camada NOVA (`NOVA=1`; trocar · voltar · duplicar · apagar · grupo):

| pincel | antes das ondas de camada | **final** |
|---|---|---|
| Chalk (digital) | 34,1 · 34,2 · 39,8 · 32,6 · 41,4 | **13,0 · 13,0 · 18,2 · 13,8 · 13,8** |
| Watercolor Basic | 80,1 · 72,6 · 75,5 · 73,6 · 80,2 | 14,8 · 13,1 · **89,7 · 72,4 · 78,4** ✗ |
| omissão water | 79,2 · 68,9 · 70,8 · 63,6 · 69,7 | 14,1 · 13,2 · **80,5 · 71,0 · 76,1** ✗ |
| Express Oil | 235,0 · 82,0 · 40,8 · 77,3 · 40,9 | **38,8 · 34,2** · 28,5 · **45,1** · 12,0 |
| Filbert | 239,2 · 96,3 · 39,3 · 92,7 · 40,9 | **59,4 · 61,0 · 35,9 · 61,1** · 12,1 |
| Wet Liner | 87,7 · 90,7 · 53,2 · 72,6 · 41,7 | **50,3 · 64,6 · 34,0 · 35,9** · 14,0 |
| omissão wet | 74,5 · 78,4 · 54,7 · 65,9 · 46,1 | **50,0 · 35,4** · 32,2 · 21,5 · 13,2 |
| omissão impasto | 12,0 · 10,9 · 40,9 · 36,7 · 41,3 | 10,7 · 10,9 · **46,2 · 37,4 · 41,1** ✗ |

Pen-up do 2.º traço na nova: Chalk 33,8 → 12,9 · WB 68,3 → 13,9 · Wet Liner 74,1 → 47,5 · omissão wet 78,3 → 50,7 · impasto 36,0 → 40,4.
«Camada de baixo, 1.º» do omissão wet: 198,0 → 51,1.

Traço (n.º 6 → final): Composite Brush toque 41,6 → 37,2 · Filbert toque 43,6 → 42,7, Ctrl+Z 53,6 → 50,7 · Express Oil Ctrl+Z 53,5 → 51,4 ·
Wet Liner Ctrl+Z 48,9 → 49,4 · omissão wet Ctrl+Z 37,1 → 34,9 · **Finger Tip Ctrl+Z 47,6 → 35,0** · WB toque 27,1 → 26,0. CPU parado
52–68 % → **50–55 %** (ainda ✗). Wet img/s 39,5–40,0 → 39,7–40,2.

## 6. ABERTO, por ordem (números da matriz final)

1. ⛔ **Tecto de LOC (§0):** `undo_planes.rs` 764, `watercolor_vidro_journal_tests.rs` 781, `watercolor_backdrop.rs` 737 > 700. Cortar por
   responsabilidade antes da integração; a suíte do integrador reprova sem isto.
2. **WB duplicar/apagar/grupo na camada nova: 72–90 ms** (89,7 · 72,4 · 78,4; omissão water 80,5 · 71,0 · 76,1). Pista CPU,
   `FULL-composite`: `alcance_da_tinta_de` → `None` pela guarda `papel_atravessa_vidro`. Falta a prova «a figura só muda em tinta ∪ vidro»
   com o vidro de cada camada na janela.
3. **Impasto na camada nova: 37–46 ms** (duplicar 46,2 · apagar 37,4 · grupo 41,1; pen-up do 2.º traço 40,4). Não investigado.
4. **Wet a trocar na camada nova: 35–60 ms** (Express Oil 38,8/34,2, apagar 45,1; Filbert 59,4/61,0/apagar 61,1; Wet Liner 50,3/64,6;
   omissão wet 50,0). Na base caiu a ~22 ms; a nova ainda paga. Hipótese: o fio da compressão (um traço 300–760 ms depois do «+» ainda o
   encontra) ou o 1.º aquecimento; ler o `[quadro-lento]` dos logs `target/mede-pen-up/matriz/*.log`.
5. **Ctrl+Z do Wet: 46–51 ms** (Express Oil 51,4 · Filbert 50,7 · Wet Liner 49,4; omissão wet 34,9).
6. **Finger Tip Ctrl+Z: 35 ms.**
7. **Toque do Filbert 42,7 e do Composite Brush 37,2 ms** (34–43 na nova; Filbert traço p95 20,9).
8. **CPU parado 50–54 %** (B23.2: o laço do Painter já dorme, falta o resto da shell — outro módulo).
9. **B19.10** (faixas/ruído), **B20.6** (anel da placa), a **espera activa ~10 % do rayon** (`crossbeam_epoch`).
10. **A espera do pen-down pelo motor** (item 5, §4): documentada; adiar o pen-down inteiro é arriscado, meça antes.
11. **`fork_par_raw` (aberto do ADR-0204):** copia a tela inteira (33 MB a 4K) quando outro dono forte segura o `Arc`; o mesmo em
    `undo_vivo/casco.rs`. Não aparece no pen-down WB (< 0,3 ms); confirmar o `strong` da tela no Wet antes de construir o fork por tile.
12. **Os 7 vermelhos antigos e os 77 alarmes** ([`N7_FECHO.md`](../missao_4k/medicoes/N7_FECHO.md)): decisão do dono; só ele regrava
    (`PH2D_PINCEIS_APROVA=1`). E a chave `PH2D_ASSENTA_PALPITE` (§2): o olho do dono.
13. Menores, registados nas notas: a troca de meio pelo chip (`set_paint_media`) e a troca de ferramenta ainda não aquecem o fundo; o
    «+» síncrono cresce com o número de camadas (40 → 171 ms de 2 a 33; escondido do clique, não do toque colado); `mark_dirty` da activa
    no «+» do Wet (não mexer sem gate da pista da placa); o Wet não é bit-determinístico entre duas ferramentas no «+» (anterior); a
    1.ª drenagem da sessão da placa nasce sem lista guardada; o `stamp` do 1.º traço (6–13 ms) e o `zera_sessao` que às vezes toca 33 MB.

## 7. ⛔ Recusas MEDIDAS (não reconstruir)

| # | ideia | medição | porquê |
|---|---|---|---|
| a | quantizar `sc`/`uc` do memo da `assenta` (¼ ou 1 nível) | 33,4 → 33,0 ms | os misses do memo não dominam |
| b | Newton com tolerância 0,25 de nível; e as duas juntas | 33,4 → 32,4; 30,5 → 29,8 ms | ruído / 2 %, muda o pincel sem comprar o alvo |
| c | qualquer aritmética da `assenta` | tecto 27,1 ms = 0,92 sim-s/s | a `assenta` grátis não chega a 1,0 |
| d | `#[inline(always)]` no `flow_at_point` do momentum | 30,0 → 30,3 ms | já embutido de facto |
| e | `with_min_len(4)` / `(16)` nos andadores por linha | 30,0 → 30,0 / 43,4 ms | o pin do rayon é ociosidade, não sobrecarga; pedaços grandes desequilibram |
| f | `RAYON_NUM_THREADS` 12 / 14 | 33,4 / 31,4 contra 30,0 ms | menos fios = menos parede |
| g | lanes que recolhem os 4 cantos × 6 planos sempre | núcleo 13,21 → 14,16 ms | a escalar salta cor e divisão com `want = 0` |
| h | janela da tinta em paralelo (rayon) | 14–46 ms contra 9–12 da série | o pool está ocupado logo depois do «+»; saiu, com o gate dela |
| i | AVX2 por `core::arch` | não construído | `unsafe_code = "forbid"` (política, não da linha) |

Herdadas da n.º 6 §4 e da n.º 5 §1 continuam a valer.

## 8. Alarmes de pincel

Por nome, sem `PH2D_ASSENTA_PALPITE`, nunca `PH2D_PINCEIS_APROVA` ([`N7_FECHO.md`](../missao_4k/medicoes/N7_FECHO.md)): **77 = 77**, a
mesma lista do início da onda (miniatura 75, os mesmos nomes; largura 2: Oil Wash, Smear Wet). **Nenhuma regressão da onda.** Os 7
vermelhos por nome (2 contagens de presets de fábrica, a costura do papel em `tiling`, Filbert Dry no portão-mãe do óleo, o censo do
preview, `o_classic_e_o_paper_edge_zero…`, `a_primeira_marca_do_wet_refaz…`) são IDÊNTICOS em `da83348ee0`. O censo do preview ganhou
`PaintState.aquecimento` nesta onda: classificado `CACHE` (`9a6b40f0ad`), a mensagem voltou à da base.

## 9. Prova de fecho

- **Gates novos, por nome, na HEAD `6d13348bb0` (depois das curas do clippy):** `ph2d-tool-painter` lib **31/31**
  (`bash scripts/cargo-test-narrow.sh ph2d-tool-painter --lib -- <os 31>`, 5,9 s; lista em `N7_FECHO.md`) · `ph2d-wet-paint` **3/3** ·
  `ph2d-app-painter` **1/1** (`apagar_duplicar_e_agrupar_compoem_na_placa_so_o_sujo_ao_bit`, placa × placa). A sonda
  `sonda_gestos_de_camada_na_cena_do_app_a_4k` é `#[ignore]`, não é gate.
- Mutações por item (vermelhas) nas notas: ADR-0204 passos 1–7, `MAIS_SEM_DIFF` M1/M2, `TELA_NOVA_PELO_MMAP`, `ENCOLHE_NO_SILENCIO` M1/M3,
  janela por palavras, fundo aquecido M1–M3 (3e) e M1–M4 (3f), troca de pincel/corte M1/M2, camadas M1–M7 e onda 5. Equivalentes
  registadas: `ENCOLHE_NO_SILENCIO` M2, camadas M5/M6, ADR-0204 passo 6 (2).
- `fmt` (`rustfmt --edition 2024 --check` nos 70 `.rs` do diff) e `clippy -D warnings --all-targets` nas 3 crates (`PH2D_SUITE=fecho`):
  limpos (`58db5f05dc`, `08c3e5101e`; os 10 erros eram todos da onda).
- `cargo build -p ph2d-host-desktop --profile smoke` na HEAD, 2×: 25,6 s e depois `Finished … in 0.21s`, zero `Compiling`; o único
  aviso é o do `main` (`main.rs:941`, §0).
- `scripts/agent-loop-profile.sh` no `N7_FECHO.md`; `rm -rf target/*/incremental` feito.
- A suíte completa e o `nextest-impacted` são do integrador (`scripts/integrar-rodada.sh`).

## 10. Regras aprendidas (novas nesta onda)

- **O limiar dinâmico do `mmap` do glibc tem tecto de 32 MiB:** uma tela 4K (31,6 MiB) depois de um `free` sai do heap com `memset`
  (18–29 ms). Peça `32 MiB + 4 KiB` e faça `truncate(n)`: vai sempre ao `mmap`, sem `unsafe`.
- **Um fio de fundo com `into_par_iter` rouba o pool rayon do fio principal:** a contenção aparece em quem corre a seguir (o
  `adormeceu_muda` 2 → 14 ms). Espere o silêncio da caneta antes de lançar trabalho de fundo, ou dê-lhe um pool próprio.
- **Teste com relógio fixo não vê a marca suja PENDENTE da sim viva do app** (~40 imagens/s): no clique há quase sempre um sujo por
  drenar. Só a matriz do app mostrou a onda 4.
- **O subagente não corre `--ignored` nem a fila pesada:** as sondas pesadas são do coordenador, num script.
- **A matriz normal e a `NOVA=1` escrevem no MESMO directório** (`target/mede-pen-up/matriz/`): mova os logs entre as corridas.
- **`unsafe_code = "forbid"` na workspace barra AVX2 (`core::arch`):** o tecto do núcleo escalar é o da política.
- **Código só de debug (`cfg(any(test, debug_assertions))`) chamado da produção parte o build `smoke`/`release`:** confira com
  `cargo check --profile smoke`.
- **Num zsh, `$N` sem aspas não se parte em palavras:** os nomes chegam ao libtest como UM filtro e o script diz «0 testes» (exit 4). Use
  `${=N}` ou um array.
- **`*_tests.rs` dentro de `src/` conta no tecto de LOC** (só `/tests/` e `/tests.rs` são excluídos).
- Herdadas: a régua do tempo real é a MÉDIA; «MB novos» de `/proc/self/stat` contam o PROCESSO inteiro (use `/proc/thread-self/stat`
  para o fio); chave A×B global em teste precisa de trava; `cargo fmt --all` formata ficheiros de outras linhas.

## 11. Smoke do dono (UI de cartões, `PH2D_CARDS` por omissão)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`
2. No Painter, crie uma imagem de 3840×2160 com o papel «Textured Paper». Pinte um traço com «Chalk», clique em «+» em Layers, pinte
   na camada nova e clique nas duas linhas de camada, uma e outra. Depois use, na barra de Layers, duplicar, apagar e agrupar. Repita
   com «Wet Liner» e com «Watercolor Basic».
3. O que tem de acontecer: trocar de camada, duplicar, apagar e agrupar respondem logo, sem travar, com o Chalk e com a aguada na
   camada de baixo. O Wet troca de camada sem a travada de antes. O «+» é instantâneo. Ctrl+Z tira exatamente um passo.
4. Deu errado se: a figura muda de aspeto depois de trocar, duplicar, apagar ou agrupar; se o Ctrl+Z tira mais ou menos que um passo;
   ou se a tinta debaixo muda depois do «+». Ainda lentos e conhecidos (não é falha nova): duplicar, apagar ou agrupar uma camada
   NOVA com aguada (Watercolor Basic), e o Ctrl+Z do Wet.
5. Para decidir a aproximação do Wet (opcional): abra as 16 pranchas em
   [n7_assenta](Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/medicoes/n7_assenta/README.md) (à esquerda como é hoje, à direita
   com a aproximação: a simulação fica cerca de 7 % mais rápida e 15 pincéis mudam um pouco). Para pintar com ela, corra o passo 1 com
   `PH2D_ASSENTA_PALPITE=1 ` antes de `./target/smoke/…`.
