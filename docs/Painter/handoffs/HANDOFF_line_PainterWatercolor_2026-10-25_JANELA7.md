# Handoff — line/PainterWatercolor, janela 7 (25/10) — smoke do dono: OK em tudo

Commits: `4b6186562d` (R1–R4 do smoke da janela 6) e `aec9653a32` (cauda com o Continuous). Texto exato: `git log -2`.
Anterior: [`PROMPT_NOVA_JANELA_7.md`](../smoke_2026-10-24/PROMPT_NOVA_JANELA_7.md). Seguinte: [`PROMPT_NOVA_JANELA_8.md`](../smoke_2026-10-24/PROMPT_NOVA_JANELA_8.md).

## 1. R1 — o 1.º carimbo do Continuous não marca (`4b6186562d`)

Causa MEDIDA: Digital, Accumulate off, Flow<1 — o 1.º dab (sem passo) levava o Flow inteiro por baixo da 1.ª cápsula:
início +23 % sobre o miolo a Spacing 0,4, +74 % a 1,0.
Cura em `crates/ph2d-painter-brush/src/stroke/abertura.rs`: retém os dabs sem passo; a 1.ª cápsula leva a abertura como
sub-dab k=n (`Ante::fecho` / `Capsula::com_fecho`); `n()` = n + fecho, `n_do_passo()` na taxa `c_cap`; `sub_dabs` idem;
toque sem cápsula carimba no pen-up.
Gates: `o_inicio_do_continuo_nao_marca`, `um_toque_do_continuo_pinta_o_dab_inteiro`
(`crates/ph2d-tool-painter/tests/it/sonda_continuo.rs`), `o_fecho_leva_o_sub_dab_de_tras` (`capsula_tests.rs`).
Mutação 8/9 (`n()` sem o fecho só sangra no unitário).

## 2. R2–R4 — cartão Wet Paint e Tuning (`4b6186562d`)

- R2: `Knob::ExtDiffusion` no cartão Wet Paint (`KNOBS_NO_CARTAO` = 10, id `PAINTER_WETPAINT_DIFFUSION`, FIELDS 23).
  Rótulo «Diffusion» (o nome inteiro cortava no dock mínimo — gate `cada_nome_deste_painel_cabe_na_coluna_da_seccao`);
  «Pigment diffusion» na dica.
- R3: os 5 knobs do cartão (Backruns, Granulation, Diffusion, Wet bloom, Tip pickup) → `KnobGroup::Hidden`; Tuning 84→79
  linhas; censo `NOUTRA_SUPERFICIE` = 15.
- R4: dicas em Wet bloom e Tip pickup (só agem em tinta MOLHADA de outra cor); gate `as_dicas_dos_knobs_do_pincel`.

## 3. Cauda (Taper Tail) com Continuous (`aec9653a32`)

Report do dono: «Taper Tail > 0 com Continuous buga» (discos e degraus). Não era a cauda: era o PLANO do traço
(`traco_plano.rs`).
1. `mask::dab_batch_region` parava no disco da frente; agora `caixa_do_dab` inclui a cápsula até ao centro anterior (e `toca`).
2. `sem_a_strength` não escalava `ante.coverage` (traço S× mais claro).
3. A cauda adensava com dabs SOLTOS a Flow inteiro; agora encadeia em cápsula ao dab anterior DA CAUDA quando
   Accumulate off + Flow<1 ou Continuous. Accumulate ligado fica o `adensa` (a cápsula ali dá 229 sobre 72).
Gates: `o_plano_compoe_a_capsula_inteira`, `a_cauda_com_a_taxa_afila_sem_discos`, `a_cauda_nao_costura_o_grao_aleatorio`
(+Flow 0,3). Mutação 4/6; sobrevivem `toca` só com disco e encadear com o último dab do corpo (diferença abaixo de 8/255).
Sondas ignoradas (para medir): `sonda_continuo_pontas`, `sonda_cauda_continuous`.

## 4. Prova de fecho

`clippy -D warnings` e `fmt` limpos nas crates tocadas; sem suíte (ordem do dono — só antes da integração). Smoke do dono
25/10: OK. Contadores como delta: `KNOBS_NO_CARTAO` 9→10, FIELDS +1 (23), Tuning 84→79, censo NOUTRA_SUPERFICIE 15.
Um merge pode partir: censo `os_escondidos_sao_so_os_de_outra_superficie_ou_de_calibracao`
(`crates/ph2d-panel-wet-tuning/src/rows.rs`) e a contagem de FIELDS do cartão se outra linha mexer no Wet.

## 5. ABERTO

1. NOVO, pré-existente, medido: Accumulate LIGADO + passagem translúcida (Flow 0,3) sem Continuous — a cauda escurece
   pelos dabs soltos do `adensa` (79…121 sobre um corpo 56/71). A lei certa NÃO é a cápsula (com Accumulate ligado ela
   compõe cada sub-dab à opacidade da passagem).
2. Ressalto de ~+10 % na entrada da janela da cauda sem Continuous (Accumulate off), dentro do gate (≤ 15 %).
3. Wet presets não gravam Press. S-curve, Light scatter e Speed lag (o Reset para o preset põe-nos no neutro no Wet).
4. Disponibilidade do «Reset to brush preset» é um canal à parte (devia ser campo do `BrushSettings`).
5. Defeito 5 da auditoria do Pen: Tilt smooth/Rotation não esmaecem (cura no motor: não rotear um `DabTilt` neutro).
6. Dry brush (`ExtDryBrush`) é knob MORTO (`dry_gate = 0` em todo dab).
7. Decisão 2 do dono: escondidos do Digital sem linha (`pigment_espectral`, `passo_pela_pressao`, `borda_px`,
   `tecto_por_dab`, `sub_passo_fino`, `particle_max_px`) — mostrar ou fixar de propósito.
8. Da janela 4: acelerar a água do Wet (dono escolheu); Borracha da Watercolor que apaga mantendo a sessão (dono
   escolheu); família «mundo» das macros lê a sprite sem a tinta por assar; Borracha do Wet × Strength; preview da
   Borracha no Wet; Texture Eraser à Strength 1; a shell a 23 linhas do tecto.

Integração no main: só por ordem do Enio (`/pd-integracao`); a suíte completa é do integrador.
