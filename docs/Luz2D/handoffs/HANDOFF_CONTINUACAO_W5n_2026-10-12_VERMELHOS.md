# Handoff de continuação — Luz 2D, W5n: os 13 vermelhos herdados (2026-10-12)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre `main`
> `780540ecde`. Commits: `b6c4d2fc3d` (o produto) · `4bbcc80ece` (os portões, a faixa lenta do nextest) · este.
> Ordem do dono: «antes do ship resolva todos os problemas vermelhos». Nada integrado, nada enviado.

## 1. O mecanismo de cada um

| vermelho | mecanismo | cura |
|---|---|---|
| `o_interior_do_opaco_nao_depende_da_posicao_na_grade` (zoom 2,5: 837 · 774 · 2 058) | ⚠️ NÃO era a região (W5h §3.1): com a grade inteira, o mesmo bit. A cascata pull-push da W5h **não converge** — o erro de média frequência fica (`relaxa` 32→512: 0 errados) | **produto**: V-ciclos de correção (`Miolo::ciclos` 3, `suaviza` 2) |
| `o_rebote_nao_depende_da_oitava` (+17 % · +49 %) | o mesmo; e a régua lia o ponto de DENTRO contra o exterior (a lei antiga) | o mesmo; a série (que já resolve o miolo) no MESMO ponto |
| `so_a_regiao_…_mesmo_bit` (texel 37,101) | o miolo só via a REGIÃO (Neumann na borda dela) | **produto**: `c3_anel.wgsl` (o anel e o conhecido de fora da região; a moldura reclassificada; os níveis ≥ 1 inteiros) |
| `as_formas_redondas` ×3, `o_miolo_das_formas` | o controlo (flags W5c/W5f) dava o número do produto: a difusão filtra a borda | controlo = a lei antiga (`miolo: None`) + as flags; o desvio do meio-tom só no anel de FORA (dentro é gradiente) |
| cenas 5/6/7 «sem faces» | `CONTRASTE_DAS_FACES` 10 era a pizza (18,5× o menor) | 1,65 medido, com controlo SEM BLOQUEAR |
| `o_traco_do_flip_recebe…` (3,12e-3) | onde aterra ≠ o que causa: o filtro da placa tem 8 bits de fração; a difusão pôs gradiente dentro dos corpos (0,95/texel) e o teto era um máximo AMOSTRADO | folga por pixel: 3 meios ulps f16 + `(Δx+Δy)·2⁻⁸`; controlo: meio texel ao lado |
| `o_interior_de_um_opaco_e_o_exterior_mais_proximo` | mede a lei antiga contra o gabarito da difusão | `miolo: None` + gabarito `suavidade 0` (helper `gabarito_por`) |
| `g5_…_declara` (`[16,16,32,32,16]`) | o `gpu-allocator` soma ao tamanho o salto de ALINHAMENTO do sítio: o mesmo buffer de 4 096 B deu +16 e +32; as pirâmides têm cauda de mips +1 280–1 792 | teto 3 KiB, variação entre grades < 1 KiB; controlo: as pirâmides sem a cauda reprovam |

O V-ciclo (`c3_miolo.wgsl`): defeito `d = T(u) − u` no nível 1 → desce ×σ (`(16+κ)/(4+κ)`) → erro resolvido no mais
grosso → sobe (bilinear + 2 pares de relaxações amortecidas a 0,8) → `u += e` + 1 relaxação. ⛔ Texel grosso com
QUALQUER filho conhecido prende o erro a zero (com o `w` fracionário o operador grosso não tem fronteira).

## 2. Medido (RTX 5060 Ti, `dev`, load ≤ 1,1)

Rodada `sonda_ciclos_do_miolo` (ignorada; reproduz-se):

| | caixa 2,5 · 0/8 | 4/8 | face rebote 1,6 | 2,5 | `c3_miolo` (cenas 1–6) |
|---|---|---|---|---|---|
| W5h (0 ciclos, sem anel) | 837 (0,136) | 2 058 (0,195) | +17,2 % | +49,3 % | 0,47 ms |
| 1 ciclo | 0 | 1 137 | −5,8 % | −1,1 % | 0,74–0,77 |
| 2 ciclos | 0 | 0 (0,096) | −3,2 % | +3,3 % | 0,93–0,97 |
| **3 ciclos (padrão)** | 0 (0,049) | 0 (0,071) | −3,6 % | −0,4 % | 1,12–1,16 (caixa 2,5: 1,29) |
| 6 ciclos | 0 (0,049) | 0 (0,033) | −3,4 % | −0,7 % | 1,70–1,76 |

O anel: `c3_fina` +0,22 ms (o despacho na grade inteira). Total +0,87 ms na vista de fábrica. Os −3,4 % da oitava
grossa são a discretização do nível 1 (convergido), daí o teto 4 % só do ponto de dentro. Os 2 texels do zoom 2,5 ·
3/8 no canto da caixa ficam iguais com 3, 6, 12 e 24 ciclos (teto 2 na difusão). Ablação da F3 penumbra
(`miolo: None`): pé+face 0,001436 · só pé 0,000337 · só face 0,001498 · nenhum 0,000292 · W5c 0,001281.
Cenas, faces (frente ÷ trás): cena 5 de ×2,10 (a cruz) a ×7,39, sem bloquear ×1,11–1,33; cena 6 ×3,43–8,12 (×1,34–1,49);
cena 7 ×7,24–7,93 (sem o Flip ×1,26–1,50). Flip: o maior uso da folga 0,889; meio texel ao lado 66,8.

### ⛔ Recusas MEDIDAS

| tentativa | medido |
|---|---|
| o anel sozinho (W5h §3.1) como cura do interior | grade inteira = região: os mesmos 837 texels |
| mais relaxações na cascata | 32→512 cura, mas ~4 000 despachos |
| V-ciclo com o `w` fracionário no erro | diverge (×10⁴ em 2 ciclos); prender só com `w > ½` também |
| folga do Flip com 2 meios ulps | 1,32 da folga em texels sem gradiente (o resto 1,33e-3 > 9,8e-4) |

## 3. A corrida final (série, `--nocapture`, load 0,6–1,1)

GPU: `4 passed` (`c3` ×2 — «sem o anel, 2 156 texels de dentro mudam» —, `g5`, a holográfica junto à face).
App: os 10 alvos verdes (`interior … ok` sozinho em 513 s); nextest das 3 crates (paralelo, `ci-test`): 151/155.

## 4. ⚠️ ABERTO

1. `a_nitidez_da_luz::com_o_automatico_desligado_o_custo_e_o_do_nivel_escolhido` reprova **com os `src/` da base**
   (`780540ecde`, sozinho, load 0,7): Low 8,87 · High 22,32 ms (a lei pede ×4). Não é da W5n (com
   `PH2D_LIGHT2D_MIOLO=0`: 9,27 · 25,40). O `c2` da holográfica no High oscila 18,5–24,7 ms entre corridas: medir o
   Low por passe antes de mexer.
2. Velocidade (W5h §3.2): o miolo custa agora ~1,1 ms + 0,22 do anel. Caminhos: o anel só nos ladrilhos com corpo;
   a relaxação do mais grosso (64 despachos por ciclo num nível de 5×3) num despacho só, na memória do grupo.
3. A `vis_na_face` (W5g) piora a penumbra 4× contra o gabarito (tabela acima) e o anel do miolo herda-a.
4. Os três portões de gabarito pesado entraram na faixa lenta do nextest (12 × 60 s).
5. Os da W5h §3 (4–6) e W5m §4 seguem.

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. Olhe os corpos (disco, elipse, losango, barra, L, cruz) e depois dê zoom (roda do rato) até um deles encher a tela.
3. Tem de acontecer: cada corpo claro do lado da lâmpada e escurecendo suave para trás; com o zoom, a luz dentro
   do corpo NÃO muda.
4. Errado: ao dar zoom, o miolo de um corpo grande clareia ou muda de forma.
