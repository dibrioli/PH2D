# HANDOFF — refatoração do Painter, janela 4 (31/10 noite → 01/11)

Ramo `line/PainterWatercolor`, de `46cc03493a` a `0598b869d4` (+ este). Nada integrado nem empurrado. Anterior:
[HANDOFF_31_10_janela3.md](HANDOFF_31_10_janela3.md). Smoke do dono feito em 01/11 (maquete + 9 perguntas): ver §3.

## 1. Feito

| commit | o quê |
|---|---|
| `48a60e9214` | F3 §7 — revisão do coordenador (R1–R9) |
| `787663c0e3` | **E9b**: o Thick Impasto `mix_p99 9,5` era a cor POR CERDA do preset (`bristleNoiseSat 0,3`), não a mistura ([E9b](E9b_ponta_thick_impasto.md)). Curas: `wp/bristles_cor.rs` `com_ruido` escala o `ds` pela folga `min(s,100−s)/50` (o amarelo saturado só podia perder saturação: B médio +14) — teste `o_ruido_de_saturacao_tem_media_zero` com prova de mutação; a régua `mix_*` corre sem `bristleNoise*` (`PH2D_SONDA_COR_DO_FIO=1` repõe). Thick Impasto **3,0/3,4**; **`portao_mae_f0_lei_pigmento_minimo` verde (30 s) e sem `#[ignore]`**. `corre_f0.sh` congela o binário em `$OUT/sonda.bin`. F3b medições + maquete |
| `b919276263` | F3 R8 — «Pigment mix» = slider 0..1, não seletor |
| `28c5db8bd0` | **E10b** — a lei Pigment mix pura em `ph2d-paint-tinta/src/pigmento.rs`: `Espectro` lateral 56 B (`Σv·log₂R` 10 bandas, `Σv·δ` resíduo da ida-volta ⇒ identidade exacta sem atalho, `Σv·p`); `p=0` igual ao bit; azul+amarelo `p=1` = `101,148,113` ([E10b](E10b_pigment_mix.md)) |
| `be475f3a46` | **E10c** — o lado espectral ligado ao motor (`Grid::pigmento` preguiçoso: 0 B com `p=0`, +112 B/célula depois); todas as portas da tela movem o `Espectro` pela fracção da `Soma`; `Params::pigment_mix` sem painel. Conservação 1,2e-7 em 50 passos ([E10b §6](E10b_pigment_mix.md)) |
| `b6ed757a59` | F3b 2.ª rodada: Body (o meio basta nos 12 óleos; a parede binária empilha 12–39 µm nas 3 águas = **Bug #56**), Absorbency (o `paperGate` não tem parte de papel → vai todo ao Hardness) |
| `286176cd21` | `advect_jacobi_bits_*` regravados: a cena passa pelo depósito do produto e a E4 (`7a4bc540b7`) mudou-o de propósito; com o depósito antigo o HEAD dá os hashes velhos ao bit |
| `0598b869d4` | **F3 §8 — o smoke do dono** (a forma do painel reprovada; respostas Q2–Q9) |

Testes por nome verdes: `ph2d-wet-paint tinta` (67), `bristles` (22), `advect_jacobi_` (4), `o_invariante_soma_massa`,
os 7 `pigmento_*` de `ph2d-paint-tinta`, os 4 `pigment_mix_motor_*`, `portao_mae_f0_lei_pigmento_minimo`.

## 2. A correr quando esta janela fechou

A grelha E9 inteira (régua congelada, binário de 01/11 09:15 com todo o código acima) pela fila:
`/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/5007c508-75cd-4b63-816e-9d4c1b01715c/scratchpad/e9full2/`
(log `…/scratchpad/e9full2.log`). Duas corridas anteriores perderam-se (uma morta às 23:19 sem causa no registo; a
outra com a pasta temporária ao fechar a sessão). Confira: `pgrep -fc sonda.bin` (só o próprio pgrep = acabou) e
`wc -l …/e9full2/varredura.tsv` (3 241 = completa). Se a pasta sumiu outra vez: `bash scripts/fila-pesada.sh correr`
(a entrada continua na fila; a régua não mudou) — ou mude o `$OUT` para uma pasta que dure.
Resumo: `python3 docs/Painter/Arquitetura/fotos/f0_varredura/resumo_f0.py <tsv>`; mosaicos `mosaico_f0.py`; entra no
E9 como §8 (fotos em `fotos/e9/`).

## 3. O smoke do dono (01/11) — LEIA o F3 §8 antes de tocar no painel

A maquete foi reprovada na FORMA, a lei aprovada. Inegociáveis novos (memória
`project_painter_panel_e10_owner_decisions_2026_11_01`): tools na barra vertical; secções comuns aos 4 modos (Paper,
Grain, Shape…) ficam; padrão do Inspector; **todo knob vivo fica** (avançado); Tilt e Spacing «Continuous» intocados.
Respostas: Dilution único ✓ · Dry speed por tela · Paper press 1 (escolha técnica) · meios ✓ · **Shadow fica** ·
cerdas na secção Shape · camada fina presa ao matiz (escolha técnica: menos artefactos) · **Body fica**.

## 4. Aberto (por ordem)

1. Grelha (§2) → E9 §8.
2. **E10 conforme o F3 §8** (no painel de HOJE, `tp/wetpaint_settings.rs` e vizinhos): `Medium`, `Dilution`
   (Pigment+Water fundidos, migração exacta), `Volume` em µm (0..1000, F3b §4), `Pigment mix` (liga o `Params::pigment_mix`),
   `Body` (tensão de cedência que substitui `levelingOwnStroke` e cura o #56), `Hardness`, `Absorbency` (tela), `Shadow`;
   Dry speed passa a por-tela; os 5 knobs MORTOS saem; os presets de fábrica por função (`migra_para_controlos`,
   `controlosE10`); os testes do F3 §4 (o «31» passa a ser a contagem real dos vivos). Fotos antes × depois das
   miniaturas que mudam (só a janela principal regera).
3. **Camada fina presa ao matiz** (Q8) — lei nova na óptica da camada (E9 §7.3 #1), com a borda_fina antes × depois.
4. E10c aberto: a PONTA não leva o `Espectro` (`ponta_tinta` em `trail.rs`, 699 linhas — par num ficheiro novo); o
   azul+amarelo pelo `Engine` inteiro quando o controlo existir; custo da `cura` em série a 4096².
5. Depois: E11 GPU (o Espectro: 56 B/fatia, `log2`/`exp2` ao bit com o `libm`) · E12 · E13 · os outros 3 modos · ajuda
   ilustrada (C) · sondas para a crate dev-only · crateras da costura.
6. Avisos de compilação alheios (`wetpaint_preset/preview.rs:487`, `sonda_relevo_oleo.rs:15`) — limpar no fecho.
7. Lição medida desta janela: bisectar trocando SÓ `crates/<x>` por `git checkout <c> -- crates/<x>` deu hashes
   falsamente iguais (a medição do agente com um `git worktree` inteiro por commit achou o commit certo) — bisecção
   séria é árvore inteira por commit, com `CARGO_TARGET_DIR` próprio.
