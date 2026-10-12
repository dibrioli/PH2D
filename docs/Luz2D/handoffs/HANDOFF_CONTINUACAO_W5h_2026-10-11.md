# Handoff de continuação — Luz 2D, W5h: o miolo dos opacos pela DIFUSÃO (2026-10-11)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D`, base `main`
> `8f20c60de`; W5d–W5h por integrar. Esta janela: `2cc265fb5` (`wip`, seis portões antigos VERMELHOS — §3).
> A ordem do dono que esta onda cumpre: [W5g §2b](HANDOFF_CONTINUACAO_W5g_2026-10-11.md).

## 1. O que existe agora

- **Gabarito** (`ph2d-light2d-ref/src/miolo.rs`): Δu − u/ℓ² = 0 na UNIÃO dos opacos, u = a luz de fora na
  borda; diferenças finitas de Gibou 2002 (a borda corta a aresta no ponto exato, matriz simétrica) + CG
  com Jacobi. `NOS_NO_MENOR` 320 (MEDIDO contra Poisson: 160 → 5,2e-3 em 0,12 s · 320 → 7,6e-4 em 0,95 s).
  `Gabarito::{novo, com_miolo}` resolve uma vez; `fluencia(cena, p)` guarda as 32 últimas cenas (os ~28
  chamadores ponto a ponto ficaram corretos sem mudar). `Rebote::com_miolo`. `suavidade = 0` = a lei
  antiga (o `o_interior_de_um_opaco` do gabarito corre nela).
- **Placa** (`c3_miolo.wgsl`, `solver_miolo.rs`, `Config::miolo`, `MIOLO_PADRAO` = s 1, ℓ ∞, relaxa 4 → 32):
  a `c3_fina` marca o interior no alfa (override `MIOLO`); `classifica` → nível 0 do conhecido (1 = anel, 0 =
  miolo fundo, −1 = exterior ou fora da região); `puxa` (peso = fração do anel); do mais grosso ao nível 1
  `inicia/desce` + Jacobi em **f32**; `final` = UMA relaxação na fina sobre o nível 1 + `mix(exterior mais
  próximo, difusão, s)`. Texturas `miolo_fino` (formato da fluência) + `miolo_conhecido` (f16) +
  `miolo_a/b` (f32), as três de mip a partir do nível 1 (`Textura::mips`). Carimbo `C3_MIOLO` (`TemposMs::c3_miolo`, no `fixo`).
- **App**: `PH2D_LIGHT2D_MIOLO=0` (comparar) · `PH2D_LIGHT2D_SUAVIDADE` (0–1) · `PH2D_LIGHT2D_PENETRACAO` (m).
- **Defeito antigo curado**: `captura_do_quadro` passava de 8 com a célula engrossada (W5e) → «captura tem de
  ser potência de 2 até 8» → luz apagada com o aviso «this GPU cannot run it» (cena 8, foto). Agora `.min(CAPTURA_MAX)`;
  portão `a_celula_grossa_nao_passa_a_captura_do_limite` (com controlo).

## 2. As rodadas (RTX 5060 Ti, `smoke`/`dev`)

Portões novos verdes: `o_miolo_do_disco` + `a_penetracao_e_a_de_bessel` (ref) · `o_miolo_da_placa_e_o_do_gabarito`
(pior 0,039 em 0,1–0,7 R · rms 0,022; controlo 0,60) · `a_penetracao_da_placa_e_a_do_gabarito` (0,045 · 0,021) ·
`a_suavidade_da_placa_mistura_as_duas_leis` (½ = a média, 6e-4).

`sonda_relaxacoes_do_miolo` (o mínimo de 3; pior · rms · ms do C3_MIOLO):

| relaxa (nível 1 → máx) | 256² finos | 1920² finos (≈ 1080p) |
|---|---|---|
| 32 → 32 | 0,025 · 0,011 · 0,37 | 0,037 · 0,021 · 2,42 |
| 4 → 32 (padrão) | 0,039 · 0,022 · 0,26 | 0,042 · 0,024 · 1,82 |
| 2 → 32 | 0,059 · 0,032 · 0,21 | 0,052 · 0,028 · 1,73 |

### ⛔ Recusas MEDIDAS (não reconstruir)

| tentativa | medido | por quê |
|---|---|---|
| exterior com peso 1 na pirâmide | rms 0,055 → 0,016 ao tirá-lo | a média do exterior (sombra atrás, claro junto à lâmpada) numa célula grossa puxa a solução |
| Jacobi na fina (f16), muitas iterações | 1024 iter: rms 0,032, PIOR que 8 (0,016) | o f16 estagna e deriva; em f32 converge (0,011) |
| pirâmide toda em f32 a partir do nível 0 | ~207 MB a 1080p | por isso a fina só guarda o conhecido e faz 1 relaxação |
| amostra de `g` a 1e-9 fora (teste) | 247 s, metade das amostras DENTRO | vértices `f32`: use 1e-6 |

O anel (a condição de fronteira = o pé da W5g) contra o gabarito: viés médio −0,7 % de g_max; o pior texel
−27 % nos tangentes (a penumbra) — é a W5g, não o miolo (`sonda_anel_do_miolo`).

## 3. ⚠️ ABERTO, na ordem

1. **O miolo só vê a REGIÃO da janela** — `o_interior_do_opaco_nao_depende_da_posicao_na_grade` reprova a
   zoom 2,5 (774–2 058 texels > 0,10; a caixa de 2,2 m sai da janela: Neumann na borda da região ≠ a caixa
   inteira do gabarito). Cura desenhada: um passe `c3_fina_anel` na grade INTEIRA que calcula só os texels
   do ANEL fora da região (o predicado `meio` é barato; o anel é perímetro) e marca o resto do interior;
   o miolo corre na grade inteira. Medir o custo.
2. **Velocidade** (§0.0: 1,8 ms a 1080p é caro): (a) várias relaxações por despacho na memória do grupo
   (bloco 16² + halo k — corta os ~250 despachos dependentes, que dominam os níveis grossos); (b) só os
   ladrilhos com interior (lista compacta + despacho indireto). E/ou um V-ciclo de correção (a cascata
   perde precisão com a escala: rms 0,011 → 0,021 de 256² para 1920²).
3. **Reescrever contra a lei nova** (os vermelhos de propósito): `as_formas_redondas` ×3 (os controlos da
   W5c não reprovam a lei nova; o TV e os cruzamentos medem-se contra o `Gabarito`), `o_miolo_das_formas`
   (o controlo passa a ser `miolo: None`), `o_rebote_nao_depende_da_oitava`. A F3 (`a_face_acesa_da_parede`) passou.
4. **Suavidade e Penetração por OBJETO** (proposta W5g §2b, a confirmar no smoke): um canal no capture (o
   dono do texel escreve s e ℓ) → o `classifica` e o `relaxa` leem-no por texel; o gabarito lê-o do
   `Oclusor`; os sete sítios da DIRETIVA §2 no cartão de quem bloqueia, `ph2d-i18n`, portão de costura que
   dirige o slider e lê a luz no miolo. O padrão sai da foto que o dono aprovar.
5. Pergunta ao dono (pendente): o vão FECHADO da pilha (cena 8) conta como exterior? Com a difusão o buraco
   preto duro virou uma mancha suave (foto).
6. Herdados: o portão lento `o_miolo_das_formas` (R9), o serrilhado de 1–2 px no terminador, a abertura lenta.

## W5i — a ESTUFA das pipelines (o «app demora a abrir», W5f §3, e o nível novo a frio, W5e ABERTO 1)

`ph2d-app-light2d/src/forno.rs`: as pipelines dependem só da configuração (nunca da grade — o portão do
redimensionar), então um solver de grade 32² montado noutro fio sobre a MESMA `Fabrica` deixa-as prontas.
`LuzDoQuadro::com_montagem_em_fundo()` (a shell liga-a; os portões ficam síncronos): configuração fria →
à estufa, o quadro sai sem luz (`aquecendo()`), a janela segue; ao abrir adianta os três níveis e o
entorno. O entorno frio: a vista acende sem ele até ficar pronto. `config_do_solver` é agora a ÚNICA porta
da configuração (o quadro e a estufa; antes duplicava a do `teto::config_do_quadro`). A `Fabrica` compila
FORA do cadeado (consulta → compila → guarda; o fio que chega 2.º fica com a do 1.º).
Portão `a_luz_nasce_na_estufa_e_a_troca_de_nivel_nao_compila`: controlo síncrono 309 ms / 26 pipelines no
1.º quadro (cache do driver quente; a frio no app eram 3,5 s); com a estufa 1.º quadro 0 ms, acesa em 21
quadros, o pior 11 ms; com a estufa ociosa, Baixa/Média/Alta montam no quadro com 0 pipelines. Foto da
cena 7 no app: 62 qps (as da W5h, 4–5 qps). ⚠️ Fica: a pipeline dos sprites por texel nasce no 1.º quadro
que os tem (no fio do quadro; não medida) e a variante «célula engrossada» acima de 1080p não é adiantada.

## Fotos

[`fotos_W5h/cena7_antes_esquerda_difusao_direita.png`](fotos_W5h/cena7_antes_esquerda_difusao_direita.png) — a
pizza vira gradiente. [`fotos_W5h/cena8_antes_esquerda_difusao_direita.png`](fotos_W5h/cena8_antes_esquerda_difusao_direita.png)
— o «X» por bloco some (as duas fotos em momentos diferentes da simulação).

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. Olhe as quatro bolas em volta da lâmpada.
3. Cada bola: clara do lado da lâmpada, escurecendo suave até o lado de trás — sem fatia de pizza nem aresta dura.
4. Errado: uma aresta reta do centro à borda. Comparar com o antigo: `PH2D_LIGHT2D_MIOLO=0` antes do `cargo`;
   experimentar `PH2D_LIGHT2D_SUAVIDADE=0.5` ou `PH2D_LIGHT2D_PENETRACAO=0.3`.
