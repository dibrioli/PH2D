# Handoff de continuação — Luz 2D, W6d (FECHO): a borda contínua e o canto convexo (2026-10-18)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `ad5f8fe39c`, por cima da [W6c](HANDOFF_CONTINUACAO_W6c_2026-10-17_O_PISCAR_DO_RENASCIMENTO.md); sem rebase (é do
> integrador). A linha CONTINUA. Mecanismo, rodadas e kill-criteria (§0–§24):
> [W6d — o disco em zoom](HANDOFF_CONTINUACAO_W6d_2026-10-18_O_DISCO_EM_ZOOM.md) — este doc não os repete.
> SUÍTE: `PH2D_SUITE=onda bash scripts/nextest-impacted.sh` corria no fecho (o log em `target/w6d/suite.log` — a próxima janela LÊ-O PRIMEIRO; um ✗ dele é da W6d até prova em contrário) · BUILD: por fazer — o smoke compila-se (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2×) depois do último commit da próxima onda

## 1. Superfície de colisão e commits

Commits da onda: `739fb6cfb1` (o campo + o fora) e `6223a97ab8` (janela anterior) · nesta janela `886f86f16f` (a
união — depois retirada) · `773e50ec5b` (**a lei final**: retas próprias) · `eb7ad4c266` (tolerância do G7) ·
`d7a5b6e711` (fotos no app) · `c062e990a5` (fmt/clippy). Tocam `ph2d-light2d-gpu` (`wgsl/c3_borda.wgsl`, `c3_direta`)
e `ph2d-app-light2d` (`quadro_ambiente.rs`, `tests/it/*`). Sem contrato congelado, sem schema, sem shell. Um merge
parte: `TOL_REBOTE_OUTROS` (`tests/it/o_rebote_do_desenho.rs`, agora 0,0033) se outra linha mexer no rebote.

## 2. A lei final (`Config::borda_continua` + `borda_fora`, ligadas por omissão)

- «Dentro» = a bilinear da distância com sinal que o `c3_borda` grava no 4.º canal.
- O cheio/vazio da faixa tira a distância da reta PRÓPRIA de cada parcial vizinho (o Sobel dele, 5×5 só na faixa):
  cheio = o menor ≥ 0, vazio = o maior < 0 (`pelas_retas_proprias` em `c3_borda.wgsl`).
- O texel rente que a borda põe fora e o iso-½ dentro marcham pela borda exata (o «fora» só no rente, `c3_direta`).
- Interruptor: `PH2D_LIGHT2D_BORDA=0` (`quadro_ambiente.rs`, as duas flags) volta à borda de antes — serve para comparar.
- **O canto (§14):** o empréstimo com a normal do PRÓPRIO Sobel (diagonal no canto) chanfrava o canto convexo ~1,3
  texel da captura; o texel do eixo medial remapeava para o chanfro (f2 = 0, sem normal) e saía no ambiente.

## 3. Números (DELTA, rodada 12: antes da W6d → lei final)

| | antes | depois |
|---|---|---|
| (a) pretos rentes no lado aceso | 139 (pior −0,964) | **1** (−0,176) |
| (b) penumbra dos tangentes | 40 | 37 (outro mecanismo, W5g; não tocado) |
| anéis (polígono zoom 0,8) | 10 errados | 0 |
| bits fora dos corpos | — | IGUAIS (0 em 24 configurações) |
| custo C3 cena 5 | 3,384 ms | 3,047 (−10 %) |
| custo C3 cena 6 | 2,191 ms | 2,094 (−4 %) |
| F3 do vértice girado (`c3_grade_fina::o_interior_de_um_opaco_e_o_exterior_mais_proximo`) | 0,0563 | 0,0531 |

`o_interior_do_opaco_nao_depende_da_posicao_na_grade` (a caixa) verde.

**Correção ao handoff anterior:** o G7 dos outros (`o_rebote_do_desenho::os_outros_devolvem_a_luz_como_a_imagem`) NÃO
era anterior à W6d: o `LuzDoQuadro` liga a borda pelos interruptores do app. Com a lei final mede 0,30 % (disco
pequeno); `TOL_REBOTE_OUTROS` 0,003 → **0,0033**, com a M12 da W5e remedida a 0,37 % (separa-se por 0,04; na W5e 0,09).

## 4. Prova de fecho

- App: 11/11 portões (A a L) · `ph2d-light2d-gpu` `--lib` 6/6 e `it` 42/42 · `clippy -D warnings` verde nas duas crates.
- Prova de mutação 5/5: M1 ramo do campo · M2 o rente · M3 sinal do cheio (**só o F3 a vê — lacuna: o app não
  observa**, aberto 2) · M4 caminho rápido · M7 a normal do próprio texel (o chanfro de volta — só o portão da caixa a vê).
- Fotos no app: `fotos_W6d/app_cena5.png` e `app_cena6.png` limpas; `app_cena3.png` sai toda clara (não prova o canto).
- Suíte inteira: recusada/por fazer pela cadência — ver o marcador SUÍTE no topo.

## ⛔ Recusas MEDIDAS (não reconstruir; detalhe nos §15–§21 da W6d)

| Recusa | Porquê |
|---|---|
| A UNIÃO campo ∪ reta por texel | Cura a caixa mas quebra o F3 no vértice girado (0,1129 > 0,062; o envelope de fora das duas leis) e custa +4 % (o código dela embutido na marcha custa o mesmo correndo ou não). |
| A união filtrada pela TORÇÃO da bilinear (τ 0,05–0,30) | Mesmo custo, perde qualidade (τ 0,30 devolve o canto preto). |
| A união só na geometria (`meio_geo`) | O mesmo F3 vermelho. |

## 5. ABERTO (por ordem)

1. **O texel EXATAMENTE no canto convexo da caixa** (1,8516; 0,9141 na sonda `medida_luz2d_w6d_canto`; fora do gabarito
   do portão B por estar a < 1 texel das faces) sai **0,012 contra 0,575** de antes. Mecanismo: o PARCIAL do vértice
   guarda a sua reta diagonal (−0,207 no centro com o vértice lá); o iso-0 corta o canto ~0,45 texel da captura.
   Cura candidata (POR MEDIR; escreva o kill-criterion antes): o parcial cuja normal discorda de todos os vizinhos
   toma a distância pela interseção (convexo mínimo / côncavo máximo; teste d₂(c_q₁)+d₁(c_q₂) > 0) das retas dos
   parciais a DOIS texels (as adjacentes saem inclinadas ~16°), 7×7 só nesses texels. Régua: a sonda do canto + um
   gabarito de canto no portão B.
2. A M3 só tem o F3 como observador — um portão do app que a veja.
3. A família (b) (penumbra dos tangentes, 37) é da W5g — fora desta onda.

## 6. Smoke do Enio

1. No terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=6 cargo run -p ph2d-host-desktop --profile smoke`
   (os discos). Depois o mesmo com `PH2D_LIGHT2D_SMOKE=5` (as formas).
2. Na janela, aproxime o zoom de um disco no lado aceso (roda do rato), de perto e de longe.
3. Tem de acontecer: a borda acesa do disco fica lisa, sem pontinhos escuros colados a ela, em qualquer zoom.
4. Deu errado se: aparecem pontinhos escuros na borda do lado da lâmpada, ou um risco escuro. Para comparar, o mesmo
   comando com `PH2D_LIGHT2D_BORDA=0` à frente mostra os pontinhos de antes.
5. Já se sabe: o canto de uma caixa ainda pode mostrar UM ponto escuro (o aberto 1) — não é defeito novo.

## 7. Para o integrador

Sem contrato congelado, sem schema. CLAUDE.md §5.1 Luz 2D: trocar o «Último:» para este handoff quando integrar
(a linha tem de caber nos 700 bytes). A coluna `base:` da `collision-surface.sh` é o merge-base `ad5f8fe39c`.
