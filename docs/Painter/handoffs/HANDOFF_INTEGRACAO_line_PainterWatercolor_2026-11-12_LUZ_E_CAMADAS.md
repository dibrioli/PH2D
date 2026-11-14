# Handoff de integração — line/PainterWatercolor, a luz do óleo e as camadas (12/11, n.º 2)

Leitor: o agente integrador. O [handoff DESEMPENHO_TOTAL](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_DESEMPENHO_TOTAL.md)
**CONTINUA a valer** (§2 ADR, §5 censo da árvore combinada, §7 decisões, §8 abertos, §9 riscos); aqui só o que esta janela
acrescentou: `8a15446986..HEAD` (11 commits). **Nada integrado nem enviado.**

## 1. Identidade

- Branch `line/PainterWatercolor`, HEAD `b77199380c`; merge-base com o main `bc4a6ce930`. O main andou ~593 commits (integração
  12/11b de outras linhas); a linha **NÃO foi rebaseada**.
- Tocado: `ph2d-tool-painter`, `ph2d-wet-paint`, `crates/ph2d-tool-painter/Cargo.toml` (1 linha: feature `rc` do `serde`; sem
  pacote novo, `Cargo.lock` intocado), `docs/`. **`shells/desktop` intocado.** Contratos §6 intocados.
- Commits (mecanismo e medição em cada mensagem): `c0abbaefc4` a luz · `b1ae43c7bd` foto do smoke · `2a44e584e6` censo do preview
  · `445b0fbc0c` recusa medida · `31924323c9` `1a4eb3f2ab` cerdas · `6af700c7fe` LayerStack · `3314144293` B18.1 · `5411676605`
  régua · `d46bc1225f` B18.2 · `b77199380c` clippy.

## 2. Superfície de colisão

- `LayerImage.rgba8` passou a `Arc<Vec<u8>>` (`3314144293`) e os planos de `PaintedDocument` (tela, relevo, cobertura, material,
  vidro) a `Arc` (`d46bc1225f`). Quem constrói de outra linha precisa de `.into()` / `rgba8_mut()`; **conflito provável** se outra
  linha mexer em `LayerImage`/`PaintedDocument`/`to_painted`/`from`, ou em `ph2d-app-macros` (`aplicar`).
- O Óleo Real (`oleo_*`, `Afinacao`) mudou de lei: se outra linha mexer aí, conte com regravações de impressões (§9).

## 3. Números que se CONTAM (DELTA)

- `PROJECT_SCHEMA` **+0** · ADR **+0** (a linha continua com 0202–0204) · registo de componentes 0 · pacotes externos 0.
- Formato gravado **ao bit**: `a_imagem_partilhada_grava_os_mesmos_bytes_que_o_vec`,
  `o_documento_colhido_partilha_os_planos_e_grava_os_bytes_do_vec`; `IMPRESSAO_DO_DOCUMENTO_V3` igual.

## 4. O que entra

**A luz do traço sem defeitos (`c0abbaefc4`).** Os portões que a luz única (`7359c0095b`) pôs vermelhos e o smoke do dono
(«brilho picotado, pixelado, pontilhado»). Cunha = 1/(2·fatias); fatias 2 → 4; horizonte [1,3,6,12,24] → [1,2,3,4,6,8,12,16,24].
Matriz dos 5 portões (✓ verde / ✗ vermelho):

| variante | parede | orla | 400 % | encaroca r16 | vãos |
|---|---|---|---|---|---|
| HEAD anterior (2 × 5, cunha 1/8) | ✗ | ✗ | ✗ 2,99 | ✗ 12 pretos | ✗ Clássico 0,11 % |
| 2 × 5, cunha 1/4 | ✓ | ✓ | ✗ 2,99 | ✓ | ✗ 0,11 % |
| 4 × 5, cunha 1/8 | ✗ Estúdio | ✓ | ✓ 3,02 | ✓ | ✓ |
| **4 × 9, cunha 1/8 (entra)** | ✓ | ✓ | ✓ 3,34 | ✓ | ✓ |

- O brilho picotado: o reflexo do AMBIENTE ganha o antisserrilhado da lâmpada (`Afinacao::antisserrilhado_ambiente` 0 → 1).
  Sonda `sonda_brilho_picotado` (Thick Impasto, pontinhos soltos): sem brilho 123 · 0 → 553 · 0,25 → 318 · 0,5 → 218 · 1 → 173.
  A luz única não causou os grãos (5 variantes: 549–557). Portão novo `o_brilho_do_oleo_nao_e_picotado`.
- `oleo_vista`: o antisserrilhado mede a curvatura no passo do píxel da vista (a s = 1 ao bit).
- **Custo:** `sonda_agua_oleo_4k` (`PH2D_SONDA_OLEO=1`): 1 014,6 → 1 160,3 G instruções (**+14 %**); quadro do traço
  41,9/48,4 → 45,9/52,0 ms (≈ +10 %, carga ~3).
- Regravados com nota (mudança de lei): `a_fenda_neutra_e_bit_identica_a_de_antes`, `o_material_neutro_e_bit_identico_a_de_antes`,
  o hash do Clear glass, e as folgas do modo Blender contra o Cycles (0,5: L +8,6 %, sat −8,5 %; 1: L +27,6 %, sat −5,7 %;
  nota em `medida_transparencia.rs`).

**Camadas sem cópia.** `3314144293` (B18.1, cherry-pick de `1b0d80f584`), `d46bc1225f` (B18.2, de `017e853b82`). Sondas ABAB no
mesmo processo: trocar camada (mín/med) 8,61/9,49 → 1,29/1,47 ms; colher o documento 17,95/19,49 → 0,005/0,006 ms; duplicar 0,02 ms.

**Curas de vermelhos herdados:** `2a44e584e6` (censo do preview: 3 campos de estado dizem porque não vão ao preview) ·
`6af700c7fe` (2 testes do LayerStack sobre a base fixa, vermelhos desde `5651d764d4`) · os 5 portões da luz · Clear glass.

**Cerdas (`31924323c9`, `1a4eb3f2ab`) — SEM ganho medido.** Com `CARGO_INCREMENTAL=0` (ruído ~0,1 %) `b1ae43c7bd` 375,12 G ×
`1a4eb3f2ab` 375,17 G: os −2,9 % vistos antes eram o incremental do perfil smoke. Ficam por serem ao bit e inofensivos.
Hashes do dab iguais (`5e697cf298bc63ee`, `ee9ea4585b78243d`, `a4c20a624c456e06`, `7537d55b4001a80e`); o do Flat 240 mudou de
`fe1a4c3dc2e7193a` por `c0abbaefc4` (a luz).

**Recusas MEDIDAS (não reconstruir):** memo da última mistura das cerdas (395,17 G, pior) · saltar o `cos` em `bristles_relevo::entre`
(393,39 G, sem ganho) · 3 cortes α5 (`&sil` no `sil_base`, `map` eixo a eixo em `penteia…::alvo`, `oleo_material::espalha`) ao bit e
sem ganho · poda da raiz do horizonte por produtos (+3 % ciclos, 329,5 G × 340–341 G, 3 blocos) · cache do campo por linhas na
água (`445b0fbc0c`) · memcmp por tile em `PlanoEmTiles::de` (ruído) · fechos #15 (`sqrt` do `falloff_t`, `footprint.rs:556`),
#34 (`dyn CellFn`), #29 (divisão `bristles_fios.rs:308`): sem corte ao bit.
Régua: `docs/DevOps/MEDIR_VELOCIDADE.md` passo 6 — A×B entre duas compilações usa `CARGO_INCREMENTAL=0`.

**Matriz final do app** (base `4381ee6d4a` × HEAD `d46bc1225f`, mesmo target, 2 blocos, 40 corridas, carga 0,9–31,5, mediana 9,8;
cada célula = mín/mediana base → head; ms, n = 2+2):

| cena | toque | traço p95 | soltar | Ctrl+Z | água Hz |
|---|---|---|---|---|---|
| Wet 4K | 64.3 → 50.2/50.9 | 14.1/14.6 → 14.1 | 20.9/21.4 → 21.8/22.4 | 45.3/53.7 → 33.9/34.5 | 39.7 → 39.6 |
| Filbert 4K * | 77.4/78.7 → 65.8/196.8 | 22.8/23.0 → 24.9/36.5 | 35.8/39.4 → 23.1/23.6 | 63.0/64.5 → 52.1/53.9 | 39.7 → 39.6 |
| Impasto 4K | 11.8/15.9 → 11.8/19.3 | 11.5/15.2 → 11.4/17.1 | 18.2/20.0 → 18.9/23.9 | 14.0/17.7 → 14.1/21.5 | 0 |
| Filbert 1080p | 27.0/37.5 → 24.7/27.9 | 22.0/40.0 → 20.8/38.0 | 22.9/38.0 → 20.0/35.5 | 39.9/45.5 → 28.1/33.5 | 39.8 |
| Wet + Óleo Real 4K | 68.6/73.6 → 57.6/71.5 | 14.1/17.9 → 14.1/25.6 | 32.2/32.5 → 22.8/33.4 | 62.2/71.6 → 46.4/51.9 | 39.9 → 39.2 |
| W5 Flat Bristle `]`×10 | 43.7/52.8 → 32.0/35.6 | 22.2/316.7 → 15.8/256.3 | 0/167.6 → 0/71.6 | 158.5/251.7 → 47.8/56.4 | 38.2 → 39.1 |
| W6 Wet `]`×10 | 64.0/70.2 → 51.7/68.3 | 13.9/19.4 → 13.9/19.5 | 22.1/25.1 → 22.9/24.0 | 50.0/63.5 → 34.0/44.6 | 38.7 → 35.1 |
| W9 Bristle 16·4000 | 55.3/74.6 → 49.8/64.3 | 22.9/38.1 → 21.9/38.0 | 43.1/59.2 → 27.2/38.6 | 67.2/79.2 → 52.1/71.5 | 38.8 → 39.8 |
| D1 Round Mixing `]`×10 | 23.1/28.1 → 23.0/27.0 | 12.1/17.9 → 12.9/18.2 | 12.0/13.4 → 11.8/17.9 | 15.9/18.9 → 15.9/18.1 | 0 |
| Wet 4K + camadas | 63.7/67.3 → 48.6/51.7 | 13.1/17.2 → 13.1/13.6 | 21.0/23.5 → 22.0/22.4 | 44.9/52.8 → 32.9/34.5 | 39.9 → 40.0 |

\* Filbert 4K: um bloco do head teve toque 196,8 e traço p95 36,5 (ruído de carga; o outro bloco 65,8). CPU parado depois e a
rodada de controlo (≈ 260 em todas) sem mudança relevante.

Camadas (ms, mín/mediana, base → head): aba 11.9/12.4 → 11.2/13.1 · apaga 49.6/56.0 → 34.4/36.0 · duplica 64.3/73.8 → 48.2/48.5 ·
grupo 45.2/51.2 → 41.2/42.0 · mais 104.1/109.8 → 79.8/111.3 · troca 87.3/91.0 → 59.9/63.3 · volta 90.0/97.1 → 81.5/108.8.

## 5. Prova de fecho (12/11)

- `cargo fmt --check` (2 crates) ✓ · clippy `PH2D_SUITE=fecho` `-D warnings --all-targets` ✓ (depois de `b77199380c`).
- Tectos: `workspace_src_files_under_loc_cap` ✓ · `overage_allowlist_has_no_stale_entries` ✓ · `the_shell_only_shrinks` ✓ ·
  `architecture_claude_md_cabe_no_orcamento` ✓.
- Testes por nome, todos ✓ salvo os vermelhos por decisão (§7): wet-paint `oleo` · `historia_` · `a_troca_da_o_bit` ·
  `o_restauro_por_plano`; tool-painter `oleo` · `o_brilho_do_oleo_nao_e_picotado` · `o_impasto_nao_encaroca_em_tamanho_nenhum` ·
  `cada_campo_do_pincel_vai_para_o_preview_ou_diz_porque_nao` · `layers::tests::` · `copias_de_plano` · `camadas_integridade` ·
  `duplicat` · `troca_figura` · `persist` · `painted` · `macro_track` · `undo_vivo` · `undo_tinta_viva` · `cerdas` · `pousa_o_dab` ·
  `ponta_das_cerdas` · `sil_memo`; panel-painter-layers `ler_a_pilha_publicada`; app-macros `aplicar`.
- `cargo check --tests` de `ph2d-app-painter`, `ph2d-app-macros`, `ph2d-host-desktop` ✓.
- Alarmes de pincel: largura = Oil Wash, Smear Wet · miniatura = 74 (a mesma contagem do fecho anterior); nada regravado.
- `agent-loop-profile` (20 sessões): paralelismo 1,19 ✗ · test:check 3,6× ✗ · respostas/sessão 174 ✓ · edições sem conferir 1,9 % ✓
  · contexto 238 mil ✓ · início 68 mil ✓.
- Limpo: worktrees `.ph2d-ag-*` e targets desta janela, `.ph2d-mat-*`, `.ph2d-perf-target-pw`.
- **Não corrido (integração):** suíte completa e o censo da árvore combinada.

## 6. Smoke do dono

Binário: `Worktrees/line-PainterWatercolor/target/smoke/ph2d-host-desktop`.

Compilado 2× depois de `rm -rf target/*/incremental` (23 GB) sobre `b77199380c`; a 1.ª recompilou só o
`ph2d-host-desktop` (24,95 s), a 2.ª: `Finished \`smoke\` profile [optimized] target(s) in 0.25s` (zero `Compiling`).

Comando base (cada passo acrescenta só o que diz): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PILHA=0 PH2D_COMPOSITE_PAINEL=wet PH2D_COMPOSITE_TELA=3840x2160 PH2D_COMPOSITE_FUNDO=papel PH2D_COMPOSITE_PAPEL=1 ./target/smoke/ph2d-host-desktop`

1. **Brilho do óleo.** O comando base com `PH2D_COMPOSITE_PRESET="Thick Impasto"` na frente → no cartão **Wet Paint**, a caixa
   **Real Oil** marcada (marque-a se não estiver) → pinte um traço grosso de óleo.
   Certo: o brilho é liso, sem grãos nem pontinhos soltos (veja a referência
   [antes × depois](Worktrees/line-PainterWatercolor/docs/Painter/smoke_2026-11-12/brilho_antes_depois.png)). Errado: pontinhos
   claros espalhados no brilho. O custo: o traço com óleo fica ~10 % mais pesado.
2. **Camadas.** No comando base: aba **Layers** → **+**, clique na camada de baixo e na de cima, duplique e apague. Certo: troca
   e duplicar quase instantâneos, a pintura não pisca. Errado: engasgo a cada clique ou imagem diferente depois de apagar.
3. **Ctrl+Z do Wet.** No comando base pinte um traço e aperte Ctrl+Z. Certo: o traço some quase na hora. Errado: atraso visível.
4. **Filbert Dry (decisão do dono).** Com `PH2D_COMPOSITE_PRESET="Filbert Dry"`: pinte azul e amarelo por cima. Certo: verde suave.
   Errado: faixas/degraus de cor na mistura.

## 7. Decisões que são DO DONO

- `a_vista_a_400_por_cento_tem_o_detalhe_que_a_ampliacao_nao_tem` **fica vermelho (NOVO)**: 2,02× com o antisserrilhado do ambiente
  (0: 3,34× · 0,25: 2,51× · sem brilho 1,39×). O alvo 3× era cumprido pelos grãos que o dono reprovou; a A.1 (`oleo_vista`) não
  está ligada ao produto — decide-se quando entrar. Não afrouxado.
- Custo da luz corrigida (+14 % instruções, ~+10 % por quadro com óleo a 4K). Se quiser mais barata, 2 × 5 com cunha 1/4 deixa
  2 defeitos (vão preto; 400 %).
- O brilho do ambiente com antisserrilhado: o produto do dono decidiu contra o oráculo Cycles (folgas do modo Blender subiram).
- `o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde` (Filbert Dry) · `impasto_shine_glints…` · `wax_bleeds…` (B29.3, de antes).
- B21.1 Mailbox: muda o app inteiro — da linha da UI/plataforma. B42.3/B17.9 (orçamentos de undo) = produto. O resto do handoff anterior.

## 8. ABERTO, por impacto

1. **W5 depósito fora do quadro** (dono: muda QUANDO a tinta aparece). Cortes restantes: `FiosNoDab::fio` (perlin por fio, 1,38 %)
   guardável por dab; flag «linha limpa» do `tip_rows` (≤ 0,3 %); `sqrt` do `falloff_t`, `dyn CellFn`, divisão: sem corte ao bit.
2. **Ctrl+Z:** a troca do motor (Wet 32 ms, Óleo 55 ms; sonda `sonda_as_fases_do_ctrl_z_wet_a_4k`) está presa à memória (a foto lê
   ~6×33 MB) → chunk esparso como tile 64×64 (adenda 2 do ADR-0204) e marcar os tiles escritos pelo passo da água
   (`solver.rs`, `flow_jacobi.rs`; `ladeira_do_oleo.rs:424` troca o plano inteiro).
3. **Custo da luz:** `visibilidade_e_ceu` 10 %, rebote 6 %; a poda por produtos foi recusada.
4. B19.10; a pista GPU recompõe sempre na troca de camada.
5. O «pixelado» com zoom = ligar a A.1 (`oleo_vista`) ao produto (§7).
6. §0.11 fora do Painter: `PaintedDocumentSemPapel` nas migrações congeladas da shell — candidato a sair (dono da shell).
7. O achado transversal da UI do handoff anterior (§8 n.º 5) continua.

## 9. Riscos para o integrador

- **Medições sob carga:** a régua é instruções (`perf stat`) ou A×B no mesmo binário; entre compilações, `CARGO_INCREMENTAL=0`
  (`docs/DevOps/MEDIR_VELOCIDADE.md` passo 6). O relógio absoluto vale pouco.
- `LayerImage.rgba8` e `PaintedDocument` em `Arc` (§2): construções de outras linhas pedem `.into()` / `rgba8_mut()`.
- Regravações de impressões do Óleo Real (§4): se outra linha mexer em `oleo_*`, reconte-as; a lei nova da luz é a do `c0abbaefc4`.
