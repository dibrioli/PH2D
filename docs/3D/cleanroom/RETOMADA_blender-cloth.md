═══════════════════════════════════════════════════════════════════
CLEAN-ROOM · RETOMADA — a linha JÁ EXISTE  (PH2D · SKILL_Cleanroom)
═══════════════════════════════════════════════════════════════════
Alvo: blender-cloth · Licença: GPLv2 (degrau T2) · Módulo: 3D/Sculpt · Linha: line/sculpt3d
Worktree: /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d
Espec: docs/3D/cleanroom/SPEC_cloth_brush.md
Cabeçalho da espec: TODAS as emendas atestadas (a de 07/09 pelo R-pré atrasado em
  `1cea3d701`; a Q23 de 09/09 em `e0275bba1`) — o censo do cabeçalho sai em silêncio.
Parou no passo: o pincel e o filtro de tecido SHIPAM; o report do dono de 09/09 está
  curado, medido e comitado (seis commits locais, ⛔ nem integrados nem pushados).
Motivo da troca: **INCIDENTE §6 — exposição SUBSTANCIAL, classificada pelo R**
  (ledger `55d7c5243` + adenda `3af98d4a4`).

Você assume a linha E o papel de Implementador. A janela anterior
PAROU — se ela ainda responder, ignore-a: a linha é sua, e NUNCA há
duas janelas na mesma linha. Sua janela está limpa e TEM de
continuar limpa: você NUNCA abre o fonte do alvo (§3.I); E e R são
subagentes.

Leia INTEIRA: docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md
Depois, EM ORDEM:
1. FASE 0 do docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md
   (cd/pwd/branch — você começa na RAIZ, que é a árvore ERRADA;
   árvore suja = trabalho do anterior: commite, não descarte). As
   regras A–H do MODELO_ABERTURA_LINHA valem para você.
2. PASSO 0 DO I (§3.I): confira/crie o deny config · declare seu
   session-id no INBOX (append cego).
3. Motivo = incidente §6? O código escrito APÓS a exposição está em
   QUARENTENA (§6.3): não o toque até um R-pós compará-lo.
   ⭐ **JÁ FOI COMPARADO E SAIU LIMPO** — o R conferiu expressão (não comportamento)
   sobre as ~850 linhas de produto escritas depois da exposição, em 17 ficheiros:
   sweep verde e busca dirigida a zero. **As seis regiões da jornada de 09/09 fundem
   como estão, e nada delas se reescreve.**
4. Retome o BLOCO-LINHA (§10) no passo em que a anterior parou.

───────────────────────────────────────────────────────────────────
O QUE ESTA JANELA DEIXA POR FAZER (a primeira tarefa é a nº 1)
───────────────────────────────────────────────────────────────────

**1. ⛔ REESCREVER DUAS REGIÕES PRÉ-EXISTENTES (2026-09-07), e é por isto que a
   janela anterior queimou.** Ficheiros:
   - `crates/ph2d-sculpt3d/src/cloth_filter_kind.rs`
   - `shells/desktop/src/sculpt3d_filter.rs`
   São **doc-comments**, não lei: dois deles repetem uma oração de FINALIDADE que tem
   de passar a dizer só o **comportamento**, com a fixture que o mede ao lado. Outros
   três sítios repetem apenas o FACTO e ficam (facto não é protegível).
   ⚠️ **Você não vai ver o texto exposto** — o R viu-o e nomeou as regiões. Reescreva
   pelo comportamento medido, e mande um R conferir.
   ⚠️ **A mesma oração já saltou para o produto DUAS vezes neste módulo** — foi essa
   propagação medida, e não o tamanho, que decidiu a queima. Ao reescrever, procure a
   terceira.

**2. ⏳ O `Expand` continua o mais violento dos cinco.** Ele é o único tipo que mexe no
   comprimento de repouso (`τ`), logo o **tecto de esticão não o mede** — o tecto
   compara contra um comprimento que a própria lei cresce. Medido depois da cura do
   arrasto: um arrasto de ecrã inteiro leva o volume a `2,4×`.
   ⛔ O `plano_filtro_expandir_3invocacoes` está nos ABERTOS da bancada **de propósito**:
   é o mesmo defeito do irmão de uma invocação, composto — curar aquele cura este.
   São um item, não dois.

**3. ⏳ O tamanho da ruga segue a densidade da malha** (doc 11 §5) — decoupá-la é um
   solver hierárquico (Müller 2008).

**4. ⏳ A resposta do filtro é QUADRÁTICA no arrasto** (`Σ_j (k−j+1)·S_j·Δt`, espec §7).
   É a lei do alvo, e ela torna o fim da faixa difícil de dosear. O *Filter Strength*
   (novo em 09/09) dá um multiplicador linear por cima, mas não muda a curva.

**5. ⚠️ Uma lição de PROCESSO que o R nomeou:** uma emenda shipou com *«aguarda R-pré»*
   escrita dentro de si e atravessou **dois dias**; quem a apanhou foi o R-pré da
   emenda seguinte. ⇒ **o portão dos atestados confere-se na ABERTURA da jornada, não
   no fim.** Faça-o no passo 2.

───────────────────────────────────────────────────────────────────
O QUE ESTÁ FEITO E VERDE (não refaça)
───────────────────────────────────────────────────────────────────
- `1e7495666` o material segue a malha quando o TIPO muda o tamanho (`muda_o_material`)
- `2fa571890` o filtro mede o ARRASTO e não o relógio (`PASSO_DE_ARRASTO`)
- `aa12f36a2` as dez corridas de invocações repetidas viraram gates (corpus `27`)
- `f4fd3d215` o *Filter Strength* — o oitavo controlo do alvo
- `9e7b26263` o incidente §6 registado no canal cego
- Portão: **188 suítes de crate + 228 do shell, zero falhas**; `86 + 27` corridas do
  oráculo verdes; `fmt` e índices em dia; binário `--release` construído e com os
  textos novos dentro.
- O registo da jornada: `docs/3D/cloth/12_o_material_e_o_relogio.md`
- O smoke do dono: `PH2D_SCULPT3D_SMOKE=37`, passos **(12)** e **(13)** novos.
  ⏳ **O dono ainda NÃO reportou o veredito deste smoke** — é o próximo facto a chegar.
═══════════════════════════════════════════════════════════════════
