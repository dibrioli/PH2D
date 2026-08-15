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
