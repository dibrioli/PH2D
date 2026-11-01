# J8 — os vermelhos antigos e as medições da fila do dono (janela 8, 01/11)

Ordem: o §4 do [handoff da janela 7](HANDOFF_01_11_janela7.md). Estado de partida (rodada única, `target/j8/vermelhos.sh`):
8 vermelhos listados; `cada_preset_de_fabrica_e_a_sua_receita` e `os_candidatos_da_w2_sao_as_suas_receitas` já
estavam VERDES (curados em `5dcf477f73`, a lista estava velha).

## 1. Curados

| teste | causa medida | cura | commit |
|---|---|---|---|
| wet-tuning `every_key_of_this_panel_exists_on_both_sides` · painter-layers `every_painter_key_exists_on_both_sides` (38 chaves «órfãs») | o hook `.claude/hooks/prazo-do-agente.sh` gravava `$(git -C $CLAUDE_PROJECT_DIR rev-parse --git-common-dir)/ph2d-agentes` — no checkout principal isso é `.git` RELATIVO, resolvido no cwd do agente: nasceram `crates/ph2d-tool-painter/src/tool/paint/.git/` e `docs/Painter/Arquitetura/.git/`; o censo (`ph2d-label-census/src/keys.rs`) pula pasta com `.git` e deixou de ver o `tool/paint/` inteiro | `--path-format=absolute` no hook; o censo só pula `.git` FICHEIRO ou pasta com `HEAD` (`outra_checkout`); as pastas-lixo apagadas | `f20ed68601` |
| wp `o_water_do_smear_no_meio_e_a_lei_de_antes` | impressão digital da grade inteira gravada a 25/10; cada mudança no depósito da faixa (E4…E10) a movia sem tocar no Smear | a lei prova-se pela conta: `tools/smear.rs::agua_do_smear(0,5) = (1, 0)` exactos (teste unitário com o mesmo nome) | `f20ed68601` |
| tp it `cada_preset_da_agua_cai_na_foto_do_seu_rebelle` | Fundo do Smear Soft/Spikey 91 contra a foto do Rebelle 119 ± 25 | régua redecidida (o Rebelle deixou de ser referência): `cada_preset_da_agua_tem_a_sua_cara` — Blender/Shattered resto ≥ 0,95 e fundo ≤ 20; Smear Soft/Spikey/Flat fundo ≥ 60 e ≥ 3× o do Blender; Smear Wet dissolve (−0,2). A foto só se imprime | `f20ed68601` |
| aviso `set_wet_lei_da_tinta`/`wet_lei_da_tinta` sem uso | só sondas e testes as chamam | `#[cfg(test)]` até a E12 decidir a lei | `f20ed68601` |
| tp it `a_guache_opaca_cobre_o_azul_seco` | nasceu em `e8f35b5710` (E10: a parede `levelingOwnStroke` → Body 0,16). A/B no mesmo binário, branco: Body 0 → 11,2, Body 0,16 → 19,0; foto: o branco cobre, com riscas de cerda finas | tecto absoluto 13 → 21 (medido com o Body: 19,0 · 10,8 · 18,5 · 9,5); a razão contra a lei desligada fica < 0,6 em todas (0,40–0,55) | `4fb0c114b1` |

O vermelho `ph2d-label-census` `a_regua_le_cada_ficheiro_uma_vez_e_nao_uma_vez_por_irmao` aparece só com a crate
inteira em paralelo (29 leituras): sozinho passa, com e sem a mudança do censo — contador partilhado entre testes.

## 2. Ainda vermelhos — a família da água sobre o óleo (a fila do dono §3.1)

Bisseção (árvore inteira por commit, `target/j8/bisseca.sh`, `bis_out/`):

| teste | verde em | vermelho desde | porquê |
|---|---|---|---|
| tp `a_agua_nao_sobe_a_ladeira_do_oleo` (a 4 px: 54 477, 179 células; régua < 34 000 e < 50) | `3570226c1e` | `4279ba26eb` (a cápsula pesa cada sub-dab pelo sub-passo natural) | o Dabber da cena ficou com a espessura CERTA (antes era engrossado pelo arredondamento) — a parede baixou. `FILME_POR_MASSA = 1/K_W` já está em unidades físicas. |
| tp `a_queda_cresce_com_a_diluicao` | `286176cd21` | `e8f35b5710` (régua) e a E10g (física) | ver abaixo |

**A régua da queda** (reescrita na J8): (1) somava a tinta que o pincel POUSA fora do óleo ao soltar (584 → 4 426 com
a E10) — passa a ser a massa fora aos 240 tiques − ao soltar; (2) escalava `volumes.water` do ficheiro, que a E10 já
não lê — passa por `agua_vezes` (Dilution e Volume juntos, o MESMO pigmento). Queda (pouca · média · muita):

| árvore | queda | 
|---|---|
| J11 `847b472b3d` | 234 · 68 977 · 119 325 |
| Bingham `c5e6daa0a9` = `286176cd21` | 4 · 78 · 49 691 |
| E10 `e8f35b5710` | 29 · 380 · 60 640 |
| HEAD (E10g + J7) | **2 433 · 2 978 · 4 862** — vermelho |

**O mecanismo (sonda `queda_na_ladeira_tests::diag_o_que_trava_a_queda`, interruptores no mesmo processo):**
produto 4 862 · sem Bingham 5 012 · sem Body 4 862 · sem ladeira 2 940 · ladeira de 30/10 12 012 · **`PAPEL_ETA = 1`
(sem a viscosidade do óleo) 55 411** · ETA 1 e sem endurecer 62 445. ⇒ **o meio (`OleoCelula::meio`) é do LÍQUIDO
mas vive na CÉLULA e não anda com ele**: a água (`m = 0`) que escorre para uma célula de óleo seco herda `m = 1` —
mobilidade `8⁻¹` (`flow_jacobi.rs`, `meio::mobilidade`) e evaporação `×0,1` (`drying.rs`). A E10g só mistura o meio
no POUSO (`oleo_celula::pousa`, pelo volume molhado); a advecção não o transporta.

**A sonda do contorno** (`escorre_contorna_tests::diag_o_escorrer_contorna_o_relevo`, 12 linhas × ladeira on/off,
fotos `target/j8/sonda_contorna/`): (a) óleo ainda MOLHADO (Dabber d0,5, fracção assente 0,12–0,19): a ladeira não
pesa (números iguais com e sem) — ela só vê o `sett`; (b) tinta sem Real Oil: sem campo `oleo`, sem parede; (c) óleo
seco: a parede impede atravessar (abaixo 0) mas só tira 23 % do líquido que sobe para cima do relevo (910k → 701k).
O Wet Drip de fábrica tem o Real Oil desligado: a fotografia de antes (`oleo_antes`) nunca é usada nessa cena.

### ⛔ Recusa MEDIDA — o dique

Lei candidata: cortar o componente do fluxo que entra num vizinho cujo chão de óleo ≥ superfície daqui (molhado/seco
das águas rasas), interruptor `DIQUE` em `flow_jacobi.rs` (removido). Medido: subida a 4 px 54 477 → 53 856 (1 %); a
queda da crista −24 a −50 %; derreter neutro; contorno inalterado. **A água que sobe não entra pelo campo de fluxo**
— candidatas por medir: a advecção do suspenso, o espalhar da mancha, a pegada do próprio pincel.

## 3. A lentidão do óleo diluído (fila do dono §3.2) — medida, não curada

Sonda `desempenho_oleo_tests::diag_desempenho_do_oleo_diluido` (intercalada, mínimo, 240×160, 120 s simulados):

- **82–88 % do CPU do óleo diluído é a COMPOSIÇÃO da tela**, não a simulação: por tique 1,9 ms (Wet Drip Oil) e 6,0–9,5
  ms (Filbert Oil d0,9) contra 0,14 ms da água na mesma área. Em 120 s: composição 17,3 s / 40,8 s contra simulação
  3,8 s / 5,7 s.
- O passo da simulação custa 0,9–1,2× o da água por célula; só a secagem (`assenta`) do Filbert pesa 4,6×/passo.
- A simulação fica ACORDADA: a água seca em 21–37 s; o Wet Drip Oil tem 2 000 células vivas aos 120 s, o Filbert Oil
  d0,9 ~14 000 (não encolhe) — o mesmo defeito do §2 (o meio na célula, `ENDURECE`).
- Cronómetros que faltam: a divisão interna de `wetpaint_composite_veiled` (`wetpaint/composite.rs:30`, ramo do óleo
  `composite_oleo.rs`).

## 4. Sondas desta janela (todas `#[ignore]`)

`escorre_contorna_tests.rs` (novo) · `desempenho_oleo_tests.rs` (novo) · `queda_na_ladeira_tests::diag_o_que_trava_a_queda`.
