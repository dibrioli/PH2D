# R3 — a velocidade de PINTAR (latência do pincel) — 31/10

Missão R3 da «superar o Rebelle» (coordenador ph2d-f2), ramo `line/PainterWatercolor-r3` desde `6ba5d06493`.
Tudo **ao bit**: cada corte tem a tela (FNV-1a de `imagem_aplicada`) igual antes e depois, e nenhum usa tolerância.

## Régua

- Sonda nova [`crates/ph2d-tool-painter/examples/mede_o_dab.rs`](../../../crates/ph2d-tool-painter/examples/mede_o_dab.rs):
  entra só pelas portas públicas do app (`set_paint_media`, `set_wet_oleo_real_ligado`, `wet_preset_aplicar`,
  `on_canvas_pointer`, `take_preview_arc` a cada 8 eventos = o ritmo do oráculo, 2 ms/evento), tela 3840×2160,
  água no relógio fixo e sem `on_tick` (o dab determinista). Modos: `raios` (Digital · Wet · Wet+Óleo Real ×
  raio 10/30/60/120, 150 movimentos de 3 px), `facas` (as 20 pinceladas do `t_facas.jsonl`: 10 facas de fábrica
  ×2, `size_px 90`), `janela` (raio 10 numa tela limpa × depois de um raio 120). Variantes no MESMO processo,
  intercaladas por ronda rodada, mínimo (mediana ao lado); filtros `MOTOR=` `MODO=` `RAIO=`.
- Build: `CARGO_PROFILE_SMOKE_STRIP=none CARGO_PROFILE_SMOKE_DEBUG=line-tables-only bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo build --profile smoke -p ph2d-tool-painter --example mede_o_dab`;
  corrido DIRECTO (`target/smoke/examples/mede_o_dab 3 facas`), fora do cgroup.
- «Antes» = o MESMO exemplo compilado sobre `git archive 6ba5d06493` (em `target/r3_base_src`, target próprio),
  corrido alternado com o «depois» (A B A B).
- ⚠️ **Carga.** A máquina teve as linhas r1/r2/PainterWatercolor a correr sondas pesadas o dia todo (load 30–90).
  As tabelas finais são de load 8–20; o trabalho puro mede-se com `RAYON_NUM_THREADS=1` (CPU de 1 fio, imune à
  disputa de fios). Sonda interna de partição: `sonda_r3_facas` (`wetpaint/sonda_r3_facas.rs`, ignorada).

## Antes → depois

**As 20 facas do oráculo, 4K** (o MESMO traço do Rebelle; alternado A B A B, load 7,7 → 19,4):

| | parede | CPU | hash |
|---|---|---|---|
| antes (`6ba5d06493`) | 17 859 · 20 048 ms | 319 · 291 s | `a5456e2d616dd358` |
| **depois** (`9c33e532f0`) | **5 448 · 4 898 ms** | **72 · 75 s** | `a5456e2d616dd358` |
| Rebelle (D_rebelle_4k.md) | span 8 s (6,4 s são o ritmo de envio) | 9,1–11,4 s | — |

Trabalho puro (1 fio) das mesmas facas, por corte: por evento 103,1 s → por quadro 34,5 → luz do óleo guardada
31,3 → normal do papel 28,0 (–73 %). ⇒ **pintamos o traço das facas mais depressa que o Rebelle** (4,9–5,4 s contra
8 s), mas com ~7× a CPU dele: a luz do Óleo Real é nossa na CPU, a dele é da placa (RealShader).

**Por motor e raio** (µs por movimento de 3 px, mínimo de 3 rondas, load 18,9 → 19,1; hashes iguais entre os dois
binários: Digital `af7261c0fcead03b`, Wet `166718b7ad4136e7`, Óleo `e9ed1a2cd7e7211e`):

| motor | raio | antes | depois | depósito µs/dab antes → depois |
|---|---|---|---|---|
| Digital | 10 / 30 / 60 / 120 | 15,7 / 32,7 / 30,0 / 30,8 | 16,0 / 38,4 / 33,2 / 30,3 | — (caminho não tocado) |
| Wet | 10 | 444 | **314** | 488 → 350 |
| Wet | 30 | 320 | **273** | 951 → 798 |
| Wet | 60 | 374 | **269** | 2 376 → 1 713 |
| Wet | 120 | 503 | 503 | 7 267 → 7 158 |
| Óleo Real | 10 | 2 398 | **582** | 686 → 440 |
| Óleo Real | 30 | 1 674 | **720** | 1 245 → 1 111 |
| Óleo Real | 60 | 1 718 | **1 157** | 2 557 → 2 134 |
| Óleo Real | 120 | 2 434 | **2 126** | 9 097 → 8 766 |

**Pincel fino depois de um largo** (`janela`, 1 fio, ×14, load 14): 1 016,8 → **250,0** µs/dab (tela limpa 262,2 →
250,1); hashes `9c636713200ba410` / `6c8b169ec3585e9c` iguais.

## Onde ia o tempo (perf, o binário com símbolos)

1. **A composição por EVENTO**: cada lote recompunha a região suja + o alcance da luz do óleo (~32 células de cada
   lado); 8–16 eventos por quadro do ecrã ⇒ a mesma área dezenas de vezes. Nas facas, ~80 % da CPU.
2. **A troca de pincel recompunha a 4K inteira**: um knob `Rebuild::Render` (o Emboss 0 da Knife Smooth 3 ↔ 0,1 das
   outras) sujava a grade inteira — 8,4 M células com a luz do óleo, **93 %** das células compostas nas 20 facas.
   Só o `render::render_region` do motor lê esses knobs; o composite do produto lê os da célula.
3. **A luz do óleo refazia por célula o que é do cartão**: `CeuDoMapa::de` (8 leituras do mapa + `atan2`), a
   direcção da luz (4 trig + `hypot` + `tan`), e com a normal do papel o `acos`/`cos`/`sin` de cada fatia.
4. **O fio do evento ficava à espera**: 2,9 s de trabalho em 13,6 s de parede — cada passe paralelo lançado de
   FORA do pool do rayon acorda os fios e adormece quem chamou; ~60 % da CPU era rayon/crossbeam a girar.
5. **A janela do rasto só crescia**: depois de um pincel largo o fino limpava a janela grande a cada transferência.

## Commits (todos ao bit)

| commit | corte | gate |
|---|---|---|
| `055c740ba7` | a tela do Wet compõe-se UMA vez por quadro, na união (`wetpaint/por_quadro.rs`), pelas portas da pilha | `por_quadro_tests` (evento × quadro ao byte, com/sem óleo, a meio do traço, Ctrl+Z); mutação → 3 007 bytes |
| `db73d4faac` | os knobs `Rebuild::Render` vão à tabela sem sujar a tela (`session.rs::reconcile_facts`) | `trocar_um_knob_de_render_entre_tracos_nao_recompoe_a_tela` (sujo ≠ Full; recomposição inteira não muda 1 byte); mutação → vermelho |
| `980a44bc1d` | luz do óleo: céu, direcção e normal do papel guardados por fio/escala | 52 `oleo*` do motor, `bit_identica`, `oleo_cache`, hash das facas |
| `e97d4f588b` | a marcha do horizonte salta a amostra que não sobe | idem |
| `98884b9909` | o lote e a composição do Wet correm dentro do pool (`no_pool`; `wet_diag::NO_POOL` = a variante de antes) | hash das três variantes das facas |
| `9c33e532f0` | a janela do rasto volta ao piso no `start_stroke`; a limpeza da ponta salta a linha na base | trail/tip/fingerprint/janela/window do motor, mistura/janela_do_rasto/brocha/reservat |

Sondas: `examples/mede_o_dab.rs`, `wetpaint/sonda_r3_facas.rs`.

## O que fica (e quem pode)

- **A luz do Óleo Real na CPU** é agora o grosso (visibilidade/rebote ~20 %, céu ~5 %, sombra directa ~6 % do
  trabalho das facas). É inerente por quadro: a região é a suja + o alcance de quem a lê. O salto que falta é
  levá-la à placa (o Rebelle faz assim) — com a lei «o mesmo bit por placa», é desenho novo, não corte.
- **Ficheiros vetados que pesam** (escrevo a necessidade, não toquei): `ph2d-painter-brush/src/footprint.rs`
  (`FootprintDeform::apply` = 19,5 % do trabalho do dab de raio 10, 9,5 % no de 120) e `spec.rs`
  (`falloff_weight`) — a silhueta por célula; `ph2d-wet-paint/src/par.rs`: com o lote dentro do pool o piso
  `PISO_NS_DEPOSITO` (35 µs) foi medido com o lançamento de FORA — o dab de raio 10 dá 250 µs a 1 fio e ~300 com
  32, logo merece re-medir os pisos com `no_pool` ligado.
- **A transferência do rasto é série por lei** (o arrasto lê células que o mesmo laço já escreveu — Gauss-Seidel):
  ~24 % do fio crítico no raio 120. Paralelizar exige outra lei (Jacobi) ⇒ muda a tinta: decisão de produto.
- **O depósito do Wet raio 120** (~7 ms/dab) é a silhueta por célula (acima) a ~45 mil células.
- **Vermelhos ANTERIORES** (iguais com os ficheiros na base `6ba5d06493`, não tocados):
  `painel_so_no_futuro::mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` («real_oil»),
  `oleo_por_traco::o_traco_seguinte_usa_o_depth_o_gloss_e_a_caixa_novos`, `as_leis_da_escala::o_relevo_nao_cresce_com_o_pincel`,
  `shape_color::a_secao_desligada_e_o_traco_de_hoje_ao_byte`, `opcoes::com_as_opcoes_desligadas_o_s_e_a_bancada_sao_os_de_hoje_ao_byte`,
  `grid::oleo_celula::o_campo_nasce_com_o_oleo…` (`size_of` 16≠12).
- Não corri a suíte inteira (só por nome, regra do dono 23/10): fica para a integração.
