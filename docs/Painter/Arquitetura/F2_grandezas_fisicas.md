# F2 — Literatura → grandezas físicas, e a LEI ÚNICA de camada (agente F2, 31/10)

> Leitor: a próxima LLM (F3 = ADR da arquitetura nova). Fonte do pedido: [`PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md`](../49_realshader/PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md) §F2.
> Mapa do código: [`F1_mapa_do_modulo_e_os_4_modos.md`](F1_mapa_do_modulo_e_os_4_modos.md); censo dos 144 knobs: [`F1_censo_wet_paint.md`](F1_censo_wet_paint.md) (não duplicado aqui — §5 é só a leitura FÍSICA).
> ⛔ **Dono 31/10: o Rebelle deixa de ser referência.** A régua é a física, a consistência interna (as provas do §3.3) e o olho do dono.
> Modelo de referência executável: [`ferramentas/lei_unica.py`](ferramentas/lei_unica.py) (numpy + matplotlib; `python3 docs/Painter/Arquitetura/ferramentas/lei_unica.py` regera as figuras e corre as provas como `assert`).

## 0. O defeito-motor, lido contra a física

[`dono_6.png`](../49_realshader/smoke_31_10_oleo/dono_6.png): Thick Impasto, pigmento ↓, água ↑, amarelo sobre azul assentado →
(1) azul MAIS ESCURO que o azul puro, (2) ferrugem na orla, (3) degraus por passada.

Nenhum dos três é alcançável por uma lei de pigmento conservativo (§3.3). O mecanismo plausível no código (hipótese a
medir pela régua de varredura da F0, não medido aqui):

- `assenta()` (`crates/ph2d-wet-paint/src/assenta.rs:439`) não TRANSFERE pigmento: RESOLVE a cor nova do assentado pela
  **inversa** da lei do ecrã ([`resolve`](../../../crates/ph2d-wet-paint/src/assenta.rs) `:393`, Newton por canal,
  passo `±48`, `clamp(0,255)` em sRGB), e o próprio doc-comment diz «**sem a caixa das tintas**». A lei do ecrã
  `Ecra::Cobre` não é K-M: tapa com `h = 1 − (1 − a_f)^esconde` (`:224`), soma um «tapar extra» pesado por
  `diferenca_de_pigmento`, `molhada`, `vela`, `oleo`, `opaca`. Inverter uma lei não-monótona com prisão só em `[0,255]`
  pode exigir uma cor de assentado FORA do envelope das duas tintas — azul mais escuro que o azul (1) e, na orla onde
  `a_f` é pequeno e a derivada quase cega (`CEGO = 0.1`), um palpite que deriva para um tom que nenhuma tinta tem (2).
- A cobertura vem de uma tabela de massa (`opacity.rs:66`, `alpha(m) = 1 − 0.998^m`) que ignora o espalhamento e a
  espessura; diluir (água ↑) não muda o pigmento, muda a «massa → alfa» por outro caminho (`WaterPerDab`, `Opaque`,
  `DryCover`, `WetCover`…) — cada passada soma por uma lei diferente da do assentar ⇒ degraus (3).

A cura não é mais um termo: é trocar «cor resolvida para preservar um ecrã» por «**pigmento conservado + uma óptica
monótona**». Aí «assentar não muda o que se vê» (regra do dono 10/10) vale por CONSTRUÇÃO, sem Newton (§3.2).

## 1. Fontes

| # | Fonte | O que dá |
|---|---|---|
| KM31 | P. Kubelka, F. Munk (1931), «Ein Beitrag zur Optik der Farbanstriche», *Z. Tech. Phys.* 12:593–601 | Duas fluxos (K absorção, S espalhamento); camada infinita $R_\infty$. |
| K48 | P. Kubelka (1948), «New contributions to the optics of intensely light-scattering materials. Part I», *JOSA* 38(5):448–457, doi:10.1364/JOSA.38.000448 | Solução hiperbólica de camada FINITA sobre fundo $R_g$ (eq. 2 abaixo). |
| D40 | D. R. Duncan (1940), «The colour of pigment mixtures», *Proc. Phys. Soc.* 52:390 | Mistura: $K$ e $S$ somam ponderados pela concentração (volume) de cada pigmento. |
| S42 | J. L. Saunderson (1942), *JOSA* 32:727 | Correcção de superfície (interface ar/veículo) — o «brilho» e o escurecer do molhado. |
| HM92 | C. Haase, G. Meyer (1992), «Modeling pigmented materials for realistic image synthesis», *ACM TOG* 11(4):305–335 | K-M multibanda em síntese de imagem; pigmento = par (K,S) espectral. |
| C97 | C. Curtis, S. Anderson, J. Seims, K. Fleischer, D. Salesin (1997), «Computer-Generated Watercolor», SIGGRAPH '97, 421–430, doi:10.1145/258734.258896 — <https://grail.cs.washington.edu/projects/watercolor/> | Três camadas (água móvel · pigmento em suspensão · depositado), adsorção/dessorção, granulação, edge darkening por evaporação na orla, back-runs; glazes compostos por K-M. |
| B01 | W. Baxter, V. Scheib, M. Lin, D. Manocha (2001), «DAB: Interactive Haptic Painting with 3D Virtual Brushes», SIGGRAPH 2001, doi:10.1145/383259.383313 | Transferência BIDIRECCIONAL pincel↔tela (deposita e apanha), secagem por camada. |
| B04 | W. Baxter, J. Wendt, M. Lin (2004), «IMPaSTo: A Realistic, Interactive Model for Paint», NPAR 2004, doi:10.1145/987657.987665 — <https://gamma.cs.unc.edu/IMPaSTo/> | Óleo: volume conservado, advecção do volume, pick-up/deposit, K-M interactivo com 8 bandas. ⚠️ errata dos autores: eq. 8 `tanh`→`coth`, eq. 9 `sinh`→`csch`. |
| VLR05 | T. Van Laerhoven, F. Van Reeth (2005), «Real-time simulation of watery paint», *Comp. Anim. Virtual Worlds* 16:429–439 | UM modelo para aquarela, guache e tinta-da-china: os meios como parâmetros de um só fluido. |
| C15 | Z. Chen, B. Kim, D. Ito, H. Wang (2015), «Wetbrush: GPU-based 3D painting simulation at the bristle level», *ACM TOG* 34(6) (SIGGRAPH Asia) | Tinta como fluido viscoso (partículas perto do pincel, densidade longe); óleo e meios por viscosidade. |
| SJ21 | Š. Sochorová, O. Jamriška (2021), «Practical pigment mixing for digital painting», *ACM TOG* 40(6), doi:10.1145/3478513.3480549 | K-M de constante única em RGB com pigmentos latentes (Mixbox) — confirma K-M em 3 bandas como aceitável. |
| KD59 | I. Krieger, T. Dougherty (1959), *Trans. Soc. Rheol.* 3:137–152 | Viscosidade da suspensão $\eta = \eta_v(1-\phi/\phi_m)^{-[\eta]\phi_m}$, $[\eta]=2{,}5$. |
| AV49 | W. Asbeck, M. Van Loo (1949), «Critical pigment volume relationships», *Ind. Eng. Chem.* 41:1470 | CPVC: acima de $\phi_c$ falta ligante ⇒ filme poroso/«giz» (aquarela seca ≈ acima do CPVC; óleo abaixo). |
| O62 | S. E. Orchard (1962), «On surface levelling in viscous liquids and gels», *Appl. Sci. Res. A* 11:451 | Nivelamento: taxa $\propto \sigma h^3/(\eta\lambda^4)$; com tensão de cedência $\tau_y$ o relevo fica (empasto). |
| W21 | E. W. Washburn (1921), «The dynamics of capillary flow», *Phys. Rev.* 17:273 | Absorção do suporte: profundidade $\ell=\sqrt{\gamma r\cos\theta\, t/(2\eta)}$. |

(Fontes C97 e B04 conferidas por pesquisa web nesta sessão; as restantes são citação bibliográfica padrão — os DOI de
D40/KD59/O62 não foram abertos.)

## 2. Tabela das grandezas

Área: **T** Tinta · **P** Pincel · **S** Superfície · **Te** Tempo · **L** Luz.

| Grandeza | Símbolo · unidade | Equação | Fonte | Área |
|---|---|---|---|---|
| Fracção volúmica de pigmento | $\phi_i$ · adim. $[0, \phi_c]$ | $\phi_i = V_i/(V_{pig}+V_{veic})$ | D40, AV49 | T |
| Espessura (volume por área) | $h$ · µm | $h = V/A$; pigmento por área $v_i = \phi_i h$ | B04, K48 | T/S |
| Fracção do veículo: água↔óleo (o MEIO) | $m$ · adim. $[0,1]$ | $m = V_{óleo}/(V_{água}+V_{óleo})$ — UMA escala | VLR05, C15 | T |
| Absorção e espalhamento do pigmento | $K_{i,\lambda},\ S_{i,\lambda}$ · µm⁻¹ por banda | $K/S = (1-R_\infty)^2/2R_\infty$ | KM31, HM92 | T/L |
| Mistura | $\mathcal K_\lambda,\ \mathcal S_\lambda$ · adim. (espessura óptica) | $\mathcal K=\sum_i v_i K_i,\ \mathcal S=s(n_v)\sum_i v_i S_i$ | D40 | T |
| Contraste de índice (molhado/seco, água/óleo) | $s(n_v)$ · adim. | $s\propto\left(\frac{n_p-n_v}{n_p+n_v}\right)^2$; $n_v$: água 1,33 · linhaça 1,48 · goma+ar ~1,25 | S42 | L/T |
| Reflectância da camada | $R_\lambda$ · adim. | K48 (eq. 2) | K48 | L |
| Cobertura (poder de esconder) | $C$ · adim. | $C = 1-(R|_{R_g=1}-R|_{R_g=0})$ — DERIVADA | K48 | L |
| Viscosidade | $\eta$ · Pa·s | $\log\eta_v=(1-m)\log\eta_{água}+m\log\eta_{óleo}$; × KD59 em $\phi$ | KD59, C15 | T |
| Tensão de cedência (empasto que fica) | $\tau_y$ · Pa | escoa só se $\rho g h$ ou o gradiente de pressão $>\tau_y$; cresce com $\phi$ e $m$ | O62 | T/S |
| Nivelamento | taxa · s⁻¹ | $\dot a/a = -\sigma h^3 (2\pi/\lambda)^4/(3\eta)$ | O62 | Te/S |
| Absorção do suporte | $\ell(t)$ · µm; porosidade $p_s$ | $\ell=\sqrt{\gamma r\cos\theta\,t/2\eta}$ | W21, C97 | S |
| Relevo do suporte | $z_s(x)$ · µm | campo de altura do papel/tela | C97 | S |
| Evaporação (água) | $E$ · µm/s | $\dot h_{água} = -E\,(1-m)$; maior na orla (fluxo para fora ⇒ edge darkening) | C97 | Te |
| Cura do óleo | $\tau_{cura}$ · s (escala artística) | $\dot o = -o/\tau(m)$ | B04 | Te |
| Abertura (quanto o assentado re-mistura) | $o$ · adim. $[0,1]$ | $o(t)=e^{-t/\tau(m)}$, $\tau(0)$ ≈ segundos, $\tau(1)$ ≈ horas | B01, B04 | Te |
| Adsorção/dessorção (suspenso ↔ depositado) | $\gamma_{ads},\gamma_{des}$ · s⁻¹ | C97 §4.4; densidade do pigmento ⇒ granulação | C97 | T/S |
| Carga do pincel | $V_b$ · µm·área; $\phi_b, m_b$ | reservatório conservado | B01, B04 | P |
| Depósito | $\Delta V_{dep}$ | $\Delta V = k_d\,A_c\,(V_b/V_{cap})$, pressão → $A_c$ | B01, B04 | P |
| Pick-up | $\Delta v_{up}$ | $\Delta v_i = k_u\,o\,A_c\, v_i^{tela}$ (só o ABERTO) | B01, B04 | P/Te |
| Pressão → área de contacto | $A_c(p)$ | cerdas como molas (B01) | B01 | P |
| Relevo seco | $h_{seco}$ · µm | $h_{seco} = \sum v_i + m\,(h-\sum v_i)\,k_{ligante}$ — a água evapora, o óleo fica | B04, AV49 | S/L |
| Brilho | rugosidade · adim. | da rugosidade do filme (nivelado ⇒ liso) e de $m$ (óleo ⇒ veículo à superfície) | S42 | L |

## 3. A LEI ÚNICA de camada (proposta para a F3)

### 3.1 Estado por célula

Três fatias, todas pigmento CONSERVADO (volumes $v_i \ge 0$), nunca cor:
- $R_g$ — o **fechado**: reflectância por banda de tudo o que já curou (papel incluído). É só um fundo.
- **aberto**: pigmentos $v_i^{a}$ assentados que ainda re-misturam (abertura $o$).
- **filme**: pigmentos $v_i^{f}$ em suspensão + veículo $w$ com meio $m$ (móvel; o solver advecta-o).

### 3.2 As equações

$$\textbf{(1)}\quad \mathcal K_\lambda=\sum_i (v_i^{f}+v_i^{a})\,K_{i,\lambda},\qquad \mathcal S_\lambda=s(n_v(m,\text{molhado}))\sum_i (v_i^{f}+v_i^{a})\,S_{i,\lambda}$$

$$\textbf{(2)}\quad R_\lambda=\frac{1-R_g\,(a-b\coth b\mathcal S)}{a-R_g+b\coth b\mathcal S},\quad a=1+\tfrac{\mathcal K}{\mathcal S},\ b=\sqrt{a^2-1};\qquad \mathcal S\to0:\ R=R_g\,e^{-2\mathcal K}$$

$$\textbf{(3)}\quad C=1-\big(R|_{R_g=1}-R|_{R_g=0}\big)\quad\text{(cobertura derivada de }\phi\times h\text{, nunca um knob)}$$

$$\textbf{(4)}\quad \text{transferências conservam: } \Delta v^{pincel}_i+\Delta v^{f}_i+\Delta v^{a}_i=0;\quad \text{pick-up }\Delta v_i^{pincel}=k_u\,o\,A_c\,v_i^{a,f};\quad \text{fecho: } R_g\leftarrow R(\mathcal K^a,\mathcal S^a,R_g),\ v^a\leftarrow0 \text{ quando } o\to0$$

$$\textbf{(5)}\quad \text{meio } m\in[0,1]:\ n_v(m),\ \eta(m,\phi),\ \tau(m),\ E\,(1-m),\ h_{seco}(m)\quad\text{— o meio escolhe PARÂMETROS, nunca a lei}$$

Pontos da escala: aquarela $m\approx0$ ($\phi$ pequeno, $h$ grande de água que evapora, $\tau$ curto, $\eta$ baixa ⇒ flui,
orla escura, sem relevo); guache $m\approx0$ com $\phi$ perto do CPVC (cobre pela espessura, seca clara: ar ⇒ $s$↑);
óleo $m\approx1$ ($\eta,\tau_y$ altos ⇒ empasto fica, $\tau$ longo ⇒ re-mistura minutos, $n_v$ igual molhado/seco ⇒ não
clareia ao secar). Nenhum sentinela (`PINCEL_DE_OLEO`, `com_opaca`, `com_oleo`, `vela`) — são todos casos de (1)–(5).

**«Assentar não muda o que se vê» por construção:** filme e aberto entram JUNTOS em (1) — mover pigmento entre eles
não muda $\mathcal K,\mathcal S$, logo não muda $R$. A única mudança no secar é física: $s(n_v)$ molhado→seco (a aquarela
clareia, o óleo não). O fecho (aberto → $R_g$) também é invariante: (2) aplicado ao aberto sobre $R_g$ é exactamente o
que (2) dava antes. Zero Newton, zero inversa.

### 3.3 Monotonia e conservação — porquê o azul NÃO pode ficar mais escuro que o azul puro

**Lema 1 (mistura, mediante).** Por banda, $\dfrac{\mathcal K}{\mathcal S}=\dfrac{\sum v_iK_i}{s\sum v_iS_i}\in\Big[\min_i\dfrac{K_i}{sS_i},\ \max_i\dfrac{K_i}{sS_i}\Big]$ para $v_i\ge0$. Como $R_\infty(K/S)=1+q-\sqrt{q^2+2q}$ é decrescente em $q$, $R_{\infty,mix}\in[\min_i R_{\infty,i},\max_i R_{\infty,i}]$ por banda.

**Lema 2 (camada finita).** Para $K/S$ fixo, (2) é uma função monótona da espessura óptica $\mathcal S$ que vai de $R_g$ ($\mathcal S=0$) a $R_\infty$ ($\mathcal S\to\infty$) sem a ultrapassar: $R\in[\min(R_g,R_\infty),\max(R_g,R_\infty)]$ (K48).

**Lema 3 (diluir).** Água/óleo têm $K=S=0$: diluir multiplica todos os $v_i$ pelo mesmo $\alpha<1$ ⇒ $K/S$ inalterado, $\mathcal S$ menor ⇒ por Lema 2, $R$ aproxima-se de $R_g$. **Diluir nunca escurece além do fundo nem além de $R_\infty$.**

**Teorema.** Com pigmento conservado (4) e $v_i\ge0$, em cada banda $R\ \ge\ \min\big(R_g,\ \min_i R_{\infty,i}(s)\big)$.
Amarelo sobre azul: se o amarelo é mais claro que o azul numa banda (R e G), o resultado nessa banda é ≥ $\min(R_g,R_{\infty,azul})$ ≥ $R_{\infty,azul}$ (o azul assentado é ele próprio uma camada de azul sobre papel, ≥ o masstone). ⇒ **o resultado nunca é mais escuro que o azul puro nas três bandas.** Condições: (i) comparar no MESMO estado de veículo ($s$ molhado < $s$ seco: o molhado escurece por física — mas uniforme para todos os pigmentos e limitado pelo masstone MOLHADO); (ii) $v_i\ge0$ (sem «cor negativa» que uma inversa pode produzir); (iii) nenhuma transferência cria volume (assert no modelo). A ferrugem: a mistura não produz $R_\lambda$ fora de $[\min,\max]$ dos ingredientes por banda; medido na grelha (a): $\max\big((R-G)-(R-G)_{amarelo}\big)=-0{,}18<0$ — nenhum tom mais «quente» que o amarelo.

Degraus por passada: com (1) a $N$-ésima passada só soma $v$; $R(N)$ é monótona e suave em $N$ (Lema 2); um degrau visível só vem de uma lei que muda de ramo entre passadas — o que a lei única não tem.

### 3.4 As provas no modelo (`lei_unica.py`, saída de 31/10)

```
provas: 4000 casos — R ∈ [fundo, R∞_mistura] e diluir é monótono: OK
(a) min sobre a grelha de max_banda(R − R_azul_puro) = +0.0025 (>0 ⇒ nenhuma célula mais escura que o azul)
(a) max (R−G) − (R−G)_amarelo_puro = -0.1755 (≤0 ⇒ sem ferrugem)
```
O `+0,0025` é o canto φ baixo × água alta: 30 µm de azul já é quase o masstone — o resultado TOCA o azul puro e não
passa (o comportamento correcto, não uma folga).

## 4. Figuras

| | |
|---|---|
| [`a_grelha_amarelo_sobre_azul.png`](fotos/f2_lei_unica/a_grelha_amarelo_sobre_azul.png) | (a) A grelha do painel: x = pigmento $\phi$ no tubo (0,05–0,6), y = água/diluente acrescentado (0–0,9); 1, 3 e 6 passadas de amarelo sobre 30 µm de azul (metade aberto), óleo $m=0{,}85$. Do azul (canto φ↓ água↑) ao verde-amarelo, liso, sem escuro nem ferrugem. |
| [`b_cobertura_vs_espessura.png`](fotos/f2_lei_unica/b_cobertura_vs_espessura.png) | (b) $C(h)$ (luminância) para $\phi$ = 0,05 · 0,20 · 0,45, azul e amarelo, x em log. Monótona, satura. O azul escuro esconde cedo (por ABSORÇÃO, $K$ grande); o amarelo precisa de 10× a espessura (esconde por espalhamento) — a física que um knob «Opaque» único não exprime. |
| [`c_escala_de_meio.png`](fotos/f2_lei_unica/c_escala_de_meio.png) | (c) O mesmo volume de azul, $m$ = 0 → 1 (9 linhas), seco. Água: espalha, orla escura (C97), clara (ar) e sem relevo; óleo: aresta nítida, saturada, com relevo (veículo fica). Em baixo, o relevo seco $h$ [µm] por $m$. A orla conserva o pigmento (assert). |

## 5. Controlos SEM grandeza física (candidatos a sair ou fundir)

Leitura física apenas; quem escreve/lê cada um está no [censo](F1_censo_wet_paint.md). «→» = a grandeza que o substitui.

**Leis de ecrã paralelas (o núcleo do defeito):**
- `Ecra::Cobre{filme, esconde, molhada, vela, oleo, opaca}` (`assenta.rs:20-72`) e os knobs que os alimentam: `Glaze`, `WetCover`, `DryCover`, `Opaque`, o «tapar extra» — → $C$ derivada (3) + $s$ molhado/seco; o glaze é só camada fina de pigmento transparente ($S$ pequeno).
- `PINCEL_DE_OLEO = -1`, rampa `OLEO_PLENO`, `com_oleo`, a mistura RGB/RYB da aguada (`ryb_mix`) — → $m$.
- `GateSaturation`, `PaperGate` (saturações de alfa) — → K48 já satura.
- `Intensity`, `PigmentPerDab` duplicam $\phi\cdot h$ — fundir em **concentração** + **carga**.

**Sem grandeza, artísticos (manter como «efeito» explícito só se o dono os quiser, fora da lei de cor):**
`BristleNoiseHue/Sat/Light` (jitter de cor por cerda — legítimo como variação de PIGMENTO no reservatório, não como cor), `BristleStreak*` (riscas claras/escuras), `ExtGrainDither`, `ExtEdgeTint`, `ExtRakeLight`, `ExtWetSheen` (→ Luz), `VisualGrain`, `PaperVisibility`, `PaperContrast` (→ $z_s$ + luz), `Dust/DustDrop`, `TipClean`.

**Têm grandeza mas estão partidos em vários knobs (fundir numa):**
- Água/tempo: `WaterPerDab`, `WaterCap`, `Evaporation`, `BaseEvaporation`, `Retention`, `Dryer*` → $w$, $E$, $\tau(m)$.
- Re-mistura: `Rewet`, `WetLift`, `Dissolve (rewetPaint)`, `Solubility`, `RewetDrag`, `Pickup`, `BlendForce`, `TipReservoir` → **abertura $o$** + $k_u$.
- Escoamento: `Viscosity`, `Drag`, `Brake`, `BrakeReach`, `MaxVelocity`, `Projection`, `LevelClamp`, `Leveling`, `LevelingOwnStroke`, `KnifeLevel`, `DryerLevel` → $\eta(m,\phi)$, $\tau_y$, nivelamento O62. (`MaxVelocity`/`Projection`/`LevelClamp` são do solver, não do artista.)
- Orla: `EdgeDarkening`, `EdgeSoftMass`, `ExtBackrun`, `WetOpen*` → evaporação na orla e back-run de C97 (consequências de $E$ e $\eta$, não knobs).
- Papel: `Felt`, `Capillary`, `CapillaryGate`, `PaperRelief`, `PaperPress`, `Emboss*`, `ExtGranulation`, `ExtValleyGran`, `ExtStaining` → $p_s$ (W21), $z_s$, densidade do pigmento ($\gamma_{ads}$).
- Relevo: `BristleHeight`, `KnifeRidge`, `OilDepth` → $h_{seco}(m)$ + $\tau_y$ (o relevo é tinta que ficou, não um ganho).
- Material: `Metallic`, `IorLevel`, `Transmission*`, `Subsurface*`, `Coat*`, `Sheen` — legítimos em Luz, mas `IorLevel`/`Transparency`/`Transmission` duplicam $n_v$ e $S$; `OilGloss`/`Roughness` → rugosidade do filme nivelado.

**Do pincel/gesto (fora desta lei; ficam no Pincel):** pressão/inclinação (`Pressure*`, `Tilt*`), forma de cerdas (`Bristle*` geométricos, `TuftArc`), espaçamento (`Spacing`, `Capsule*`, `PigmentSpacing`) — grandezas geométricas, não de tinta.

## 6. O que a F3 tem de decidir com isto

1. Pigmento como par $(K,S)$ por banda, derivado da cor escolhida ($R_\infty$) + um atributo **transparência do pigmento** ($S$) — o único «opaco» que sobra é do PIGMENTO, não do pincel.
2. Estado em volumes (filme · aberto · $R_g$) em vez de cores RGB + alfa; o render é (2). A migração de presets mapeia `Opaque/DryCover/WetCover/Glaze` → $S$ do pigmento e $\phi$; `Pigment/Water` do topo → $\phi$ e diluição.
3. A régua da F0 testa os Lemas 1–3 sobre o APP (não sobre o modelo): `R ≥ min(fundo, masstones)` por banda e monotonia em $N$ passadas.
4. Bandas: 3 bastam para o teorema; 8 (B04) ou pigmentos latentes (SJ21) só se o olho do dono pedir verdes mais limpos — medir antes.
