# BUGS do módulo 3D / Sculpt — os que a CAUSA enganava

> Irmão do [`BUGS_physics.md`](../Physics/BUGS_physics.md) e do
> [`BUGS_painter.md`](../Painter/BUGS_painter.md): aqui só entra o defeito cuja
> **causa apontava para o lugar errado**, ou cujo gate estava **VERDE sobre
> ele**. O log cronológico das waves é o
> [`21_plano_modos_e_ferramentas.md`](21_plano_modos_e_ferramentas.md) §7 — este
> arquivo existe para a próxima LLM não repetir a investigação, não para
> duplicá-la.

---

## #1 — "Modo L: o Falloff parece ter borda dura" (2026-08-13)

**Sintoma (screenshot do Enio):** uma escada ao longo de um arco que cruza o anel
do cursor e segue pela esfera, no modo `L` do Grab.

**O que enganou, em ordem:**

1. **A grandeza errada.** Eu li o `rigid_profile` em `r/ε = 3` — **0,00011** — e
   declarei a hipótese refutada em voz alta. O `rigid_profile` é só o ESCALAR do
   kernel; o que o artista vê é `|grab|`, que inclui o termo anisotrópico
   `(r·f)r`, e ele vale **0,03472**. Os dois diferem **300×** na borda. *A
   tabela da §7.10 estava certa e eu tinha medido outra coisa.*
2. **O gate que certificava o defeito.**
   `the_rim_residual_is_what_chose_the_scale_family` mede exatamente esse
   0,0347 e afirma `< 0,036` — ele não estava cego, ele **aprovava** o resíduo,
   com uma mensagem (*"o Tri é o que torna a borda do CURSOR honesta"*) que era
   verdadeira enquanto `ε = raio/3` e falsa desde a §7.11.
3. **A cura óbvia é a errada.** Esticar `KELVINLET_REACH` mede 4 → 1,19 % ·
   5 → 0,48 % · 6 → 0,215 % — **nunca zero**, com vértices a crescer como `r²`.
   Um kernel regularizado tem cauda infinita por construção.

**A causa real:** a curva que o `stroke` entrega a um verbo de campo era a
**indicadora do suporte** (`dist <= query_r`, um corte C0), e o corte caía onde o
campo ainda carrega 3,47 % do bico. O degrau sempre existiu — a **§7.11 mudou-o
de LUGAR**, do anel do cursor (10 vértices, onde se lê como *a borda do pincel*)
para 3× o anel (114 vértices, onde nada o explica). É a §0 mordendo a wave
anterior da própria linha.

**A cura:** `kelvinlet::rim_landing` — uma janela C¹ no **CONSUMIDOR**, com o
kernel do paper intacto. Detalhe, números e as três mutações no §7.13 do plano.

**A lição que sobrevive ao fix:** *um gate pode estar verde porque CERTIFICA o
número, e o veredito dele é calibrado para uma colocação que outra wave pode
mudar.* Quem move `ε`, `REACH` ou o raio da consulta reconfere este gate.

---

## #2 — "Pinch em B e S são idênticos" — o chip `B` vestia a lei de OUTRA ferramenta (2026-08-15)

**Report:** *"Pinch em B e S bons mas idênticos ou quase idênticos."*

**Por que a causa enganava:** o `B` do Pinch carregava `LateralPull::Tangential`,
e o doc dessa variante afirmava, em letras garrafais, que ela **não** era a lei do
Blender — logo o chip parecia legítimo por construção (*"três leis, não duas"*) e
o defeito parecia ser de calibração. Medido, o que separava os dois em força
`1,00` era **0,0125 r, 9 % do pico**: dois apertos radiais separados por um
arredondamento.

⚠️ **A causa raiz é uma leitura de fonte alheia feita pelo COMENTÁRIO e não pelo
código.** A nota descrevia o `pinch.cc` a partir do comentário dele — *"Project
the displacement into the X vector (aligned to the stroke)"* — e essa frase é
**falsa no próprio Blender**: o código monta `X = cross(area_no, grab_delta)`, que
é **perpendicular** ao traço. O erro inverteu o mapa inteiro e propagou-se para
**três** docs (a variante, o gate do verbo, o corpo do `lateral_pull`), cada um a
citar o anterior.

**A verdade, lida do `cross`:** o `crease.cc:112` faz **exatamente** a nossa
projeção tangencial (*"pinched towards a **line** instead of a single point"*), e
o `pinch.cc` remove a componente **ao longo do traço**. Nós coincidíamos com um e
faltava-nos o outro.

**A lição:** *quando a fonte tem um comentário e um `cross`, o `cross` é a fonte.*

⚠️ **E a nota que declarava isto bloqueado tinha ENVELHECIDO:** ela dizia *"fechar
a dele pede o frame do traço dentro do `Dab` — wave própria"*, e o `Dab::path`
chegou na wave da FAIXA sem ninguém reconferir.

---

## #3 — "Pinch/Blob em L são ruins" — o gate afirmava a coisa certa sobre o lugar errado (2026-08-15)

**Report:** *"Blob modo L ruim … em L Pinch ruim."*

**Por que a causa enganava:** havia um gate,
`the_elastic_pinch_gives_back_along_the_normal_what_it_takes_from_the_plane`,
verde, cujo nome afirmava exatamente a propriedade que justificava o `l-mode`
existir — e ele media **0,5043 contra 0,1515** do modo que já shipava. A leitura
natural era *"a lei está certa, o problema é afinação"*.

⚠️ **Ele somava o deslocamento normal sobre a ESFERA INTEIRA.** Decomposto por
banda, a espirrada que ele media vive **toda fora do anel do cursor**; dentro
dele a normal é **negativa** (−0,00078 na banda 0,5-0,75 r contra um lateral de
+0,00761). *Uma soma global disse o contrário do que acontece sob o cursor* — a
mesma doença que o Painter 2D pagou ao medir a ondulação no EIXO do traço em vez
do ombro.

**A causa real, e ela é geometria:** o traço zero da `F` reparte `+s` na normal e
`−s/2` no plano, mas os vértices de uma **malha** vivem na superfície
(`r · n ≈ 0`), então o termo normal é ~zero. **Uma casca não tem material fora do
plano para receber o que sai de lado.** Consequência medida: o campo removia
**4,8× mais** volume que o modo sem ele — o oposto exato do que existia para
fazer —, com **62,4 %** do gesto fora do anel.

⚠️ **E os outros dois gates da família EXIGIAM o defeito.** Com o campo já
removido, a mensagem de falha de um deles foi: *"o empurrão elástico não alcança
além do anel — sem isso o `l-mode` do Blob é um domo mais fraco e nada mais"*.
Ele estava certo sobre o mecanismo e errado sobre o veredito.

**A lição:** *um gate pode pinar exatamente aquilo que o artista vai reprovar — e
o nome dele soa como uma virtude enquanto isso.*

**Fechamento:** o `Field::Pinch` saiu. A razão final é de REFERÊNCIA, não de
número: o `elastic_deform.cc` do Blender porta o mesmo paper e declara cinco
famílias (`GRAB`, `GRAB_BISCALE`, `GRAB_TRISCALE`, `SCALE`, `TWIST`) — **nenhuma é
o pinch**. Detalhe inteiro na §7.24 do plano 21.
