# EXAME J6 — Digital (+ 1 Watercolor): os 8 do censo «só por valores» (28/10)

> Método: [`BRIEFING_FAMILIA.md`](../BRIEFING_FAMILIA.md) + [`REGRAS_DE_CUSTO.md`](../REGRAS_DE_CUSTO.md) (≤ 2 rondas, ≤ 4
> variantes). Receitas: `EM_ESTUDO_J6` em `crates/ph2d-tool-painter/tests/it/familias/lapis.rs` (fora da fábrica; o gerador
> de estudos `escreve_os_estudos_da_aguarela` lê-a com `"receita_meio"`). Presets de estudo (byte-iguais ao gerador):
> [`pranchas/digital_j6/presets/`](../pranchas/digital_j6/presets/); réguas: [`pranchas/digital_j6/reguas/`](../pranchas/digital_j6/reguas/).
> Biblioteca da matriz: os **65** de fábrica pintados de novo nesta corrida (`/tmp/claude-1000/j6_digital/biblioteca`,
> FIXA no HEAD `708597183c`) — a biblioteca `j6_wet` só tinha `meta.json` em 28 dos 66 e a matriz ignorava todo o Digital
> (o 1.º Studio Pen saiu a d 0,827 do Wet Liner; com a biblioteca inteira, 0,283 do Hard Round). Revisão cega: NÃO feita
> (é do coordenador). Custo: não medido (load 7–50 durante a corrida; o relógio não vale).

## Resumo

| # | pincel | estado | régua | vizinho (d) | prancha |
|---|---|---|---|---|---|
| B-7 | Rough Sketch | **pronto p/ cega** | 382/454 | Soft Pastel 1,205 | `J6d_Rough_Sketch{,_cega}.jpg` + `_chave.txt` |
| I-12 | Optical Wash | **pronto p/ cega** | 210/272 | Glaze Round 2,411 | `J6d_Optical_Wash{,_cega}.jpg` + `_chave.txt` |
| K-1 | Clouds | curado, não aprovado (2 rondas) | 364/454 | Graphite HB 1,071 | `J6d_Clouds.jpg` |
| C-4 | Studio Pen | **funde com Hard Round** | 446/454 | Hard Round **0,283** | `J6d_Studio_Pen.jpg` |
| C-6 | Technical Pen | **funde com Pixel 1px** | 426/454 | Pixel 1px **0,330** | `J6d_Technical_Pen.jpg` |
| B-6 | Tilted Shade | motor em falta | 115/176 deitada (J3: 157) | Square Hard 1,407 | `J6d_Tilted_Shade{,_deitado}.jpg` |
| L-5 | Blur | motor em falta | 68/80 (só «sobre») | — (matriz `nan`) | `J6d_Blur.jpg` |
| M-7 | Sketchy Web | sem oráculo útil | — | — | — |

## Por pincel

**B-7 Rough Sketch** — oráculo MyPaint `ramon/Sketch_1`. Receita Digital: dureza 0,8, jitter 0,2 pela pressão
(`pressureJitter` 1), `jitter_scale` 0,85 (o `radius_by_random` 0,88 log do .myb), passo 0,14 (3,5 dabs/raio), Accumulate
fluxo 0,6, tinta pela pressão. R1 (jitter 0,5, fluxo 0,34): 284 — bolhas soltas; R2: **382** (variante jitter 0,3 passo
0,1 dureza 0,9: 358). Razão de uso: o esboço solto de manchas que se sobrepõem — a linha que «procura» a forma, que nenhum
outro da biblioteca faz (todos os lápis são contínuos). **Ainda vejo**: o MyPaint tem um HALO de discos grandes e muito
TÉNUES à volta do núcleo opaco (o raio sorteado leva a opacidade junto); os nossos discos grandes são tão opacos como os
pequenos e mais nítidos. Isso é motor (opacidade que desce com o raio sorteado) — fila.

**I-12 Optical Wash** — oráculo Krita `j) Waterpaint Soft Edges`. Receita Watercolor: `fill` 0, `edge_gain` 1,5,
`edge_spread` 5, `warp` 1. R1 (`fill` 0,03, spread 3, warp 2): 168, miolo tingido e orla rasgada; R2: **210** (orla 2,0 × 8:
172). Razão de uso: o contorno de aguarela sem corpo — o véu que só deixa a orla, para esboçar formas em tela grande. **Ainda
vejo**: a 200 px a orla do Krita é escura, rasgada e esbate-se para DENTRO ~6 px; a nossa é clara e fina; as pontas dele
são rectas, as nossas redondas. Peculiaridade do oráculo (não é lei a copiar): no Krita os traços VERTICAIS e os que
cruzam tinta quase não pintam (só um pingo no fim) — o motor dele é um borrador de cor.

**K-1 Clouds** — oráculo MyPaint `ramon/Clouds`. Receita: dureza 0,2, jitter 0,25, `jitter_scale` 0,4, passo 0,1, fluxo
0,35. R1 (jitter 0,6, passo 0,35): 296 — bolhas espalhadas; R2: **364** (variante dureza 0, passo 0,08: 362). **Não
aprovado**: as massas do MyPaint crescem com a VELOCIDADE (`radius_logarithmic` 2,57 pelas `speed1/speed2`) e enchem a
tela a 40 e 200 px; o nosso é uma linha macia granulada. A próxima ronda (proibida pela regra 1) seria só valores: a linha
«Speed size» (`speedSize`) já existe.

**C-4 Studio Pen / C-6 Technical Pen** — as receitas da J3 (`canetas.md`) medidas de novo contra a biblioteca inteira:
a G-pen (Krita `Ink-3`, S da pressão) a **0,283** do Hard Round e o fineliner (Krita `Ink-2`) a **0,330** do Pixel 1px
(limiar 0,591). A régua casa (446, 426) porque o oráculo É um redondo duro. ⇒ **fundem**: a G-pen é o Hard Round com a
linha «Pressure S-curve»; o fineliner é o Pixel 1px/Hard Round com «Speed size». Saem da lista de estudo.

**B-6 Tilted Shade** — medido: o Rebelle `Pencil/Digital/Tilted Hard` com o rato é um redondo DURO sem grão (na bateria
padrão ele é um Hard Round; a identidade só aparece com a caneta), e o Krita `Pencil-5 Tilted` com o rato é um fio de 2 px
— os dois oráculos não concordam fora da inclinação. Na bateria deitada (`exames/lapis/bateria_deitado.json`, fotos
`fixtures/krita_deitado/`) alargar a faixa por VALORES (diâmetro 5,2 → 7, achatado 0,6 → 0,7; `tiltLength` já no tecto 5)
PIOROU a régua: **157 → 115/176** (a faixa sai ~45 px contra 9–17 do Krita; ao longo engorda no início). A receita da J3
fica (`lapis::EM_ESTUDO`). **Motor (fila, o diagnóstico da J3 confirma-se)**: a rampa da ponta dentro do tecto do traço
(com a acumulação sai um degrau, não o esbatido) e a faixa deslocada para o LADO da mina.

**L-5 Blur** — o único oráculo em que o efeito se vê é o Krita `k) Blender Blur` (o `x) Filter Blur` não se vê na
bateria; o Rebelle `Wet/Default/Blurred` é um véu ciano, outro pincel). Medido na prancha: o Krita PUXA o vermelho numa
língua macia de ~50 px abaixo da faixa e CLAREIA a faixa numa largura de ~24 px — é um borrador que apanha o papel. R1:
a ferramenta Blur (macia, força 1) é simétrica e quase invisível. R2: o Smear macio (24 px, força 0,7) e o Smear Basic de
fábrica puxam uma língua DURA de ~10 px sem clarear; o Blur a 24 px igual ao R1 (a cena «sobre» não muda com o tamanho do
preset). **Motor (fila)**: o borrador que apanha o PAPEL e esbate a língua (o «dulling» do Krita). A matriz dá `nan` às
ferramentas que não pintam no papel (a biblioteca pinta-as sem faixas) — a distinção não se mede assim.

**M-7 Sketchy Web** — **sem oráculo útil**: nas fotos da bateria o Krita `Sketching-1 Chrome Thin` é uma linha lisa de
1 px e o `Sketching-2 Chrome Large` uma linha lisa larga (a teia só aparece quando a caneta volta perto dos próprios
pontos, rabiscando; a espiral não chega). O `Experimental Webs` foi o oráculo da J4 (reprovado, `efeitos.md`). Precisa de
uma foto NOVA do Krita com um gesto de rabisco (vez do Krita) — não corrida aqui.

## Fila (para o coordenador)

1. Motor: opacidade que desce com o raio sorteado (o halo do Sketch_1) — B-7.
2. Motor: borrador que apanha o papel e esbate a língua (Krita Blender Blur) — L-5.
3. Motor (J3): rampa da ponta dentro do tecto + faixa do lado da mina — B-6.
4. Valores: K-1 com `speedSize` (3.ª ronda, fora da regra de custo).
5. Oráculo: foto do Krita Sketching com rabisco — M-7.
