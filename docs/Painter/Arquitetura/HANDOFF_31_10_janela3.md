# HANDOFF — refatoração do Painter, janela 3 (31/10)

Ramo `line/PainterWatercolor`, de `c8c40f241f` a `d9db91ec81` (+ este). Nada integrado nem empurrado. Anterior:
[HANDOFF_31_10_janela2.md](HANDOFF_31_10_janela2.md). Sem smoke do dono: a lei nova continua atrás de `Sim::tinta`.

## 1. Feito (detalhe e números: [E9_portao_mae.md](E9_portao_mae.md) §7)

| commit | o quê |
|---|---|
| `8d6ec43322` | o MEIO viaja nas somas (`Soma::m = Σv·m`, `CelulaTinta::meio()`, 72 B); o depósito pousa o meio do pincel (= `Params::oleo`, o Real Oil, até a E10); o ecrã decompõe em **sRGB** (`ph2d_paint_tinta::decompoe` — a tela compõe nos valores codificados: manchas_azul 9–29 % → **0**); a régua plana lê SECO (`film ← 0`) e «coberto» = cobertura física da célula (`grid::tinta::cobertura`); `mix_*` só com `C ≥ 0,99` |
| `cc8a222a43` | coluna `orla_p99_plano` (camada fina) na varredura; régua partida para `sonda_varredura_regua.rs` (tecto de LOC) |
| `4bcf13769e` | E9 §7 com as fotos `fotos/e9/{antes_srgb_plano,azul_sozinho_antes_srgb,depois_srgb_plano,borda_fina}.png` |
| `d9db91ec81` | [F3_controlos.md](F3_controlos.md) — RASCUNHO da E10 por um agente (31 controlos: PAINT 7 · BRUSH 9 · PAPER 4 · TIME 4 · LIGHT 7; migração de 188 linhas, 60 «por medir»; presets por função; testes por nome; maquete). **O coordenador ainda NÃO o reviu.** |

Testes por nome verdes: `ph2d-wet-paint tinta` (62), `o_invariante_soma_massa`, `ph2d-paint-tinta` inteira (14; `lemas::sobre_branco…` passou a sRGB).
Célula E9 (plano, Pigmento): **Wet Round PASSA tudo** (mix 3,0/3,4); **Thick Impasto mix_p99 9,5 > 8** (o resto passa).
`portao_mae_f0_lei_pigmento_minimo` continua `#[ignore]` e vermelho só pelo Thick Impasto.

## 2. A correr quando esta janela fechou

A grelha E9 inteira (15 pincéis × 54 × 2 leis × 2 papéis, ~3 h), pela fila (`fila-pesada.sh correr`, régua
congelada nos 5 ficheiros da sonda): saída em
`/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/f32e4ca9-6cf5-4a99-8fcf-308732ccfc1c/scratchpad/e9full/`
(`varredura.tsv`, `g*.log`, `celulas/`), log da fila em `…/scratchpad/e9full.log`. ⚠️ Enquanto corre, NÃO recompile os
testes de `ph2d-tool-painter` em `--profile smoke` (o `corre_f0.sh` usa o binário mais novo de `target/smoke/deps`).
Confira se acabou: `pgrep -f corre_f0.sh` vazio e `wc -l …/e9full/varredura.tsv` (3 241 linhas = completo).
Resumo: `python3 docs/Painter/Arquitetura/fotos/f0_varredura/resumo_f0.py <tsv>`; mosaicos: `mosaico_f0.py`.
O resultado entra no E9 como §8 (copie as fotos para `fotos/e9/`).

## 3. Aberto (por ordem)

1. **Ler a grelha** (§2) → E9 §8. Se a Pigmento passar perto em todo o papel padrão, tirar o `#[ignore]` do
   `portao_mae_f0_lei_pigmento_minimo` só quando o Thick Impasto passar.
2. **Thick Impasto `mix_p99 9,5`**: no MOTOR puro 3 passadas do mesmo azul derivam ΔOklab **0,0000** (medido com um
   teste de diagnóstico, não guardado) — o desvio (`36,90,223` na célula contra `40,90,220`) entra pela FERRAMENTA. Suspeitas:
   o depósito pousa `x.ponta` = `tip_*` sRGB que o bico velho também escreve (`trail/bico.rs` `linha`, `transfer.rs:192`
   fórmula da `sujidade` com `tinta`/`limpo`), não a `ponta_tinta`; ou a cor que a ferramenta entrega ao motor.
   Medir na ferramenta (sonda com `PH2D_SONDA_MIX_PIOR=1` imprime os piores píxeis com a cobertura).
3. **Camada fina muda de matiz** (`orla_p99` 25–31; borda do azul ciano, do amarelo limão): física das 3 bandas na
   CAMADA. Decisão do dono (mostrar `fotos/e9/borda_fina.png` no smoke): manter, ou prender o matiz da camada fina à
   cor escolhida (o vidrado sobre fundo curado continua por banda).
4. **E10**: rever o F3 (o agente marcou riscos: Body talvez redundante; os presets de óleo a `m = 1` mudam as
   miniaturas — o mesmo dab cobre 3,98× menos, ver `M_UM` por meio; `paperGate` partido entre papel e pincel; 14 knobs
   de cerda sem leitor). Perguntas SÓ do dono, a levar com a maquete: (a) Dilution num slider ou Pigment + Water como
   hoje; (b) Dry speed da tela ou do pincel; (c) Paper press por omissão 0, ½ ou 1; (d) meio do Acrylic 0,5, Express
   Oil 1, pastel de óleo/cera 1. Depois: implementar o painel (inglês), tabela de migração, presets por função.
5. Depois, como no prompt: «Pigment mix» (WGM) · E11 GPU · E12 · E13 · os outros 3 modos · ajuda ilustrada (C).
6. Avisos de compilação ALHEIOS a esta janela (2 `unused import` em `wetpaint_preset/preview.rs:487` e
   `sonda_relevo_oleo.rs:15`) — limpar no fecho.
