#!/usr/bin/env bash
# Prova de mutacao da METADE VISIVEL da tinta fina (§16 do doc 27): a porta que
# e' dona do plano, a voz do pen-down, a fileira do painel e a cena.
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO em QUATRO pontos, porque cada um deles ja'
# mentiu nesta casa:
#   (a) a ancora tem de casar EXACTAMENTE UMA vez — casar zero le-se como
#       «sobreviveu» e casar duas aplica a mutacao no sitio errado;
#   (b) a mutacao tem de COMPILAR — um erro de compilacao le-se como sangrar;
#   (c) a corrida tem de correr N > 0 testes, e a populacao e' `passed + failed`
#       — nunca so' `passed` (o `running N tests` CONTA os `#[ignore]`).
#   (d) a corrida LIMPA tem de estar VERDE — com a arvore ja' vermelha, TODA
#       mutacao le-se como SANGRA e o placar sai perfeito e fabricado.
#
# ⛔⛔⛔ **ELE JA' NAO CABE NO PRAZO DE 30 MIN DA FATIA, e isso e' MEDIDO**
# (2026-09-21): a corrida de `nextest` custa ~50 s e a rede tem 35 mutacoes ⇒
# ~30 min de relogio, e a corrida que as levou todas foi MORTA na M25 com a
# arvore inteira — o risco de MUTACAO CONGELADA que o §8-bis ja' registou.
# ⇒ `MUTA_FILTRO=<regex>` corre so' as mutacoes cujo NOME casa (ERE, contra o
# nome inteiro). O sumario DIZ o filtro, porque *um placar parcial lido como
# completo e' a forma mais barata de um arnes mentir*.
#   exemplo:  MUTA_FILTRO='^M3[0-5] ' bash docs/3D/ferramentas/muta_a_metade_visivel.sh
set -u
FILTRO="${MUTA_FILTRO:-}"
# ⭐⭐⭐⭐ **PRE-VOO DAS ANCORAS (`MUTA_SO_ANCORAS=1`)** — confere que cada
# ancora casa EXACTAMENTE uma vez, sem correr um unico teste.
# ⛔⛔ Ele existe porque `cargo fmt` (ou um corte de ficheiro) reescreve a
# indentacao de uma ancora, ela passa a casar ZERO, e **isso le-se exactamente
# como uma mutacao que SOBREVIVEU** — com o custo de uma corrida inteira para
# descobrir. O pre-voo custa segundos e corre-se DEPOIS de todo `fmt`.
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
APP=crates/ph2d-app-sculpt3d/src
PAN=crates/ph2d-panel-sculpt3d/src
# ⚠️ **O MOTOR entra na rede a partir de 21/09.** As mutacoes M27-M29 vivem na
# `ph2d-sculpt3d` (as duas cercas do dab e a lei da folha na amostra), e uma
# arvore que so' guarda `$APP`/`$PAN` deixaria uma mutacao CONGELADA ali se o
# arnes fosse morto — o incidente do §8-bis, numa crate que ninguem restaura.
SC=crates/ph2d-sculpt3d/src
BK=$(mktemp -d)
cp -r "$APP" "$BK/app"; cp -r "$PAN" "$BK/pan"; cp -r "$SC" "$BK/sc"
restore() {
  rm -rf "$APP" "$PAN" "$SC"
  cp -r "$BK/app" "$APP"; cp -r "$BK/pan" "$PAN"; cp -r "$BK/sc" "$SC"
  find "$APP" "$PAN" "$SC" -name '*.rs' -exec touch {} +
}
trap restore EXIT

# ⛔⛔⛔ **UM PROCESSO POR TESTE, e a razão está MEDIDA (2026-09-20).**
# A 1.ª corrida deste arnes deu CINCO abortos «zero testes correram», e o ramo
# que passou a imprimir a prova mostrou o que eles eram: `SIGSEGV` no binario
# de `--lib` da `ph2d-app-sculpt3d` — o defeito NOMEADO no CLAUDE.md §5, que
# mata a corrida INTEIRA e leva o `test result:` com ele. O `nextest` corre um
# processo por teste e le `229/229` em 3 de 3 na mesma arvore.
# ⚠️ *Um arnes que morre a meio nao reporta uma mutacao: ele reporta o proprio
# acidente, e o numero final le-se igual nas duas leituras.*
corrida() { cargo "nextest" run -p ph2d-app-sculpt3d -p ph2d-panel-sculpt3d --lib 2>&1; }

# A populacao honesta e a que o `nextest` diz ter CORRIDO — nunca a que ele
# listou (os `#[ignore]` entram na listagem e nao correm).
populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

# ⭐⭐⭐⭐ **O 4.o CONTROLO DO ARNES: a corrida limpa tem de estar VERDE.**
# ⛔⛔ Ate' 2026-09-21 so' se contava que ela CORREU testes (`> 0`), e o `|`
# deitava fora o codigo de saida. Com a arvore vermelha ANTES de mutar, TODA
# mutacao le-se como SANGRA e o placar sai perfeito e fabricado — e o erro e'
# para o lado que nao se nota, porque um placar cheio nao faz ninguem olhar.
if [ -z "$SO_ANCORAS" ]; then
limpa=$(corrida); rc_limpo=$?
verde=$(printf '%s' "$limpa" | populacao)
echo "VERDE antes: $verde testes correram"
[ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa nao correu teste nenhum"; exit 2; }
if [ "$rc_limpo" -ne 0 ]; then
  echo "ABORTO: a corrida limpa esta' VERMELHA -- um placar tirado daqui e' fabricado."
  printf '%s' "$limpa" | grep -E '^ *(FAIL|test result:|Summary)' | tail -8 | sed 's/^/      | /'
  exit 2
fi
fi

sangram=0; total=0
muta() { # ficheiro  ancora  substituto  nome
  local f="$1" agulha="$2" subst="$3" nome="$4"
  # ⚠️ O filtro corta ANTES do contador: o `total` tem de descrever a POPULACAO
  # que de facto correu, senao o placar diz «N de 35» sobre seis corridas.
  if [ -n "$FILTRO" ] && ! printf '%s' "$nome" | grep -Eq "$FILTRO"; then return; fi
  if [ -n "$SO_ANCORAS" ]; then
    total=$((total+1))
    local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
    if [ "$n" -ne 1 ]; then echo "  ✗ ANCORA [$nome]: casou $n vezes (esperado 1)"; else sangram=$((sangram+1)); fi
    return
  fi
  total=$((total+1))
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a ancora casou $n vezes (esperado 1)"; return; fi
  python3 -c '
import sys
p,a,b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p,"w").write(s.replace(a, b, 1))
' "$f" "$agulha" "$subst"
  touch "$f"
  local out rc corridos
  out=$(corrida); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutacao nao compila"
  else
    corridos=$(printf '%s' "$out" | populacao)
    if [ "$corridos" -eq 0 ]; then
      # ⛔⛔ **Um aborto MUDO lê-se exactamente como uma mutação que não entrou**,
      #    e foi isso que este arnês fez em 2026-09-20: CINCO abortos, um deles
      #    sobre o próprio CONTROLO (uma linha em branco, que não pode abortar).
      #    Corridas à mão provaram que as cinco de facto SANGRAVAM. ⇒ o aborto
      #    passa a IMPRIMIR a prova, porque *um instrumento que se declara
      #    inconclusivo sem dizer porquê não é mais honesto que um que mente*.
      echo "  ABORTO [$nome]: zero testes correram — as ultimas linhas foram:"
      printf '%s' "$out" | tail -6 | sed 's/^/      | /'
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── A PORTA que e' dona do plano ─────────────────────────────────────────
muta "$APP/tinta_da_peca.rs" \
  'Some(c) => Tinta::semeada(c, faces(), k),' \
  'Some(_) => Tinta::nova(mesh.vert_count(), faces(), k),' \
  'M1 garante: o plano novo nasce BRANCO em vez de semeado'

muta "$APP/tinta_da_peca.rs" \
  '        && t.nivel() == k
        && concorda_com(t, mesh)' \
  '        && concorda_com(t, mesh)' \
  'M2 garante: trocar de nivel deixa de reconstruir'

# ⚠️⚠️ **A CONTA MUDOU DE SITIO em 2026-09-21** — ela e' hoje a
# `ph2d_mesh_colors::Topologia::descreve`, porque o panico do dono provou que a
# pergunta tem um segundo leitor (a porta do device) que esta crate nao
# alcanca. O que sobra aqui e' a TRADUCAO de `Tinta`+`Mesh` para as duas
# contagens, e e' ela que estas duas mutacoes medem; a LEI tem rede propria no
# `muta_a_cerca_do_plano.sh` (N5/N6).
# ⛔ As duas ancoras de antes casavam ZERO vezes depois da delegacao, e o
# aborto do arnes foi quem o disse — *uma rede sem controlo de ancora teria
# lido as duas como SOBREVIVENTES*.
muta "$APP/tinta_da_peca.rs" \
  '.descreve(mesh.vert_count(), mesh.faces().len())' \
  '.descreve(mesh.vert_count(), t.topologia().faces())' \
  'M3 concorda_com: a metade das FACES passa a concordar sempre'

muta "$APP/tinta_da_peca.rs" \
  '.descreve(mesh.vert_count(), mesh.faces().len())' \
  '.descreve(t.topologia().verts(), mesh.faces().len())' \
  'M4 concorda_com: a metade dos VERTICES passa a concordar sempre'

muta "$APP/tinta_da_peca.rs" \
  '        let por_vertice = t.plano_por_vertice().to_vec();
        mesh.colors_mut().copy_from_slice(&por_vertice);' \
  '        let _ = &mesh;' \
  'M5 devolve: o canal por vertice deixa de ser reescrito'

muta "$APP/tinta_da_peca.rs" \
  '    if concorda_com(&t, mesh) {' \
  '    if true {' \
  'M6 devolve: um plano desactualizado escreve na mesma'

muta "$APP/tinta_da_peca.rs" \
  '    let k = k.min(NIVEL_MAX);' \
  '    let k = k;' \
  'M7 garante: o tecto deixa de cortar'

# ── A ROTA: quem segura o plano ──────────────────────────────────────────
muta "$APP/tinta_da_peca.rs" \
  '    if e_a_activa && o_traco_segura {' \
  '    if o_traco_segura {' \
  'M8 rota: um traco NOUTRA peca empresta o plano a esta'

muta "$APP/tinta_da_peca.rs" \
  '        pedir: if e_a_activa || a_peca_tem {' \
  '        pedir: if true {' \
  'M9 rota: TODA peca da cena ganha um plano novo'

# ── A VOZ do pen-down ────────────────────────────────────────────────────
# ⚠️⚠️ **A LENTE DA VOZ mudou de agulha em 2026-09-21** — ela era
# `dyntopo_armado && o_gesto_muda_a_topologia(...)` e o consumidor perguntava
# `(interruptor || corre_sem_o_interruptor) && niveis == 1 && ...`; hoje os
# dois leem a `o_passe_corre_no_pen_down`. A ancora de antes casava ZERO vezes.
muta "$APP/recusa.rs" \
  '            && crate::tinta_da_peca::o_passe_corre_no_pen_down(
                verbo,
                self.dyntopo_armado,
                self.niveis,
                self.tinta_fina_armada,
            )' \
  '            && self.dyntopo_armado' \
  'M10 recusa: a lente volta a ser o INTERRUPTOR e nao a PORTA'

# ── A PORTA DO PEN-DOWN, metade a metade ─────────────────────────────────
muta "$APP/tinta_da_peca.rs" \
  '    (dyntopo_armado || verbo.corre_sem_o_interruptor())' \
  '    dyntopo_armado' \
  'M38 porta: o Density deixa de contar (o falso NEGATIVO da voz)'

muta "$APP/tinta_da_peca.rs" \
  '        && niveis == 1
        && o_gesto_muda_a_topologia(verbo, tinta_fina_armada)' \
  '        && o_gesto_muda_a_topologia(verbo, tinta_fina_armada)' \
  'M39 porta: a pilha montada deixa de contar (o falso POSITIVO da voz)'

muta "$APP/history_dyntopo.rs" \
  '        if !self.o_passe_de_topologia_corre_no_pen_down() {' \
  '        if false {' \
  'M40 pen-down: a foto e a triangulacao voltam a correr para todo gesto'

# ── O ATALHO DO UPLOAD ───────────────────────────────────────────────────
muta "$APP/tinta_da_peca.rs" \
  '    emprestado && !mexeu && !tinta_suja && !malha_por_subir' \
  '    emprestado && !mexeu && !tinta_suja' \
  'M41 atalho: a QUARTA cerca desaparece (a malha por subir deixa de contar)'

muta "$APP/slots.rs" \
  '                    matches!(line.job, SlotJob::Full),' \
  '                    false,' \
  'M42 atalho: a quarta cerca fica certa e a CHAMADA passa-lhe sempre false'

muta "$APP/recusa.rs" \
  '            tinta_fina_armada: self.tinta_fina_armada(),' \
  '            tinta_fina_armada: false,' \
  'M11 recusa: a voz nunca arma no produto'

# ── A CENA ───────────────────────────────────────────────────────────────
muta "$APP/scenes_tinta_fina.rs" \
  '        cena.wireframe = true;' \
  '        cena.wireframe = false;' \
  'M12 cena: o arame nasce desligado e o CONTROLO some'

muta "$APP/scenes_tinta_fina.rs" \
  'pub(crate) const LATITUDES: usize = 24;' \
  'pub(crate) const LATITUDES: usize = 96;' \
  'M13 cena: a peca deixa de ser grossa e o degrau some'

# ── A FILEIRA do painel ──────────────────────────────────────────────────
muta "$PAN/state_modes.rs" \
  '            Self::Quatro => Some(2),' \
  '            Self::Quatro => Some(1),' \
  'M14 DetalheDaTinta: dois chips colapsam no mesmo nivel'

muta "$PAN/state_modes.rs" \
  '            Self::Dezasseis => Some(4),' \
  '            Self::Dezasseis => Some(3),' \
  'M14b DetalheDaTinta: o degrau NOVO colapsa no anterior'

muta "$PAN/paint/brush.rs" \
  '    let y = brush_cor::paint_detalhe_da_tinta(ctx, snap, x, w, y);' \
  '    let y = brush_cor::paint_detalhe_da_tinta(ctx, snap, x, w, y);
' \
  'M15 CONTROLO: uma mutacao INERTE (uma linha em branco) nao pode sangrar'

# ── O PESO da peça ───────────────────────────────────────────────────────
muta "$APP/objects.rs" \
  '            + self
                .tinta
                .as_ref()
                .map_or(0, ph2d_mesh_colors::Tinta::footprint_bytes)' \
  '' \
  'M16 footprint: o plano deixa de contar no peso da peca'


# ── ⭐⭐⭐⭐ A CURA DE 2026-09-21: o plano sobrevive a um traco de COR ──────
muta "$APP/tinta_da_peca.rs" \
  '    if verbo.paints_color() && tinta_fina_armada {
        return false;
    }' \
  '' \
  'M17 porta: um pincel de COR volta a refinar com o plano armado'

muta "$APP/tinta_da_peca.rs" \
  '    verbo.refina_no_dyntopo() || verbo.colapsa_no_dyntopo()
}' \
  '    false
}' \
  'M18 porta: NINGUEM mexe na topologia (o CONTROLO do verbo de forma)'

muta "$APP/tinta_da_peca.rs" \
  '        self.stroke.tinta_fina.is_some()
            || self' \
  '        false
            || self' \
  'M19 porta: a tinta EMPRESTADA deixa de contar — e e a que conta no gesto'

muta "$APP/dyntopo.rs" \
  '        if !self.o_gesto_em_maos_muda_a_topologia(verbo) {' \
  '        if !verbo.refina_no_dyntopo() && !verbo.colapsa_no_dyntopo() {' \
  'M20 refine_for_dab: volta a perguntar so ao verbo'

muta "$APP/dyntopo.rs" \
  '        if self.tinta_fina_armada() {
            return (true, 0);
        }' \
  '' \
  'M21 toggle: ligar o interruptor volta a triangular com o plano armado'

# ⚠️ A M22 media a TERCEIRA metade sozinha; hoje ela vive dentro da porta e a
# mutacao que a apaga e' a M39 acima. O que sobra aqui e' a mesma pergunta pela
# porta INTEIRA, que e' o que o pen-down de facto le.
muta "$APP/history_dyntopo.rs" \
  '        if !self.o_passe_de_topologia_corre_no_pen_down() {
            self.dyn_before = None;
            return;
        }' \
  '' \
  'M22 pen-down: a triangulacao volta a correr para quem nao mexe na topologia'

muta "$APP/scenes_tinta_fina.rs" \
  'pub(crate) const DEGRAU_DA_LICAO: ph2d_panel_sculpt3d::state::DetalheDaTinta =
    ph2d_panel_sculpt3d::state::DetalheDaTinta::Oito;' \
  'pub(crate) const DEGRAU_DA_LICAO: ph2d_panel_sculpt3d::state::DetalheDaTinta =
    ph2d_panel_sculpt3d::state::DetalheDaTinta::Quatro;' \
  'M23 cena: a const da licao separa-se do roteiro'

muta "$APP/tinta_da_peca.rs" \
  'pub(crate) const NIVEL_MAX: u8 = 4;' \
  'pub(crate) const NIVEL_MAX: u8 = 3;' \
  'M24 tecto: o motor deixa de alocar o degrau que o painel oferece'

# ── ⭐⭐⭐⭐ O GESTO QUE ERRA A PECA (o que sobrou do report de 21/09) ─────
# ⚠️ A prova de comportamento dela e' `#[ignore]` + placa, logo o ELO tem de
# estar no censo de texto — senao esta mutacao SOBREVIVE, como as M19-M22.
muta "$APP/input_down.rs" \
  '                scene.close_stroke();
                scene.drag = Some(Drag::Orbit);' \
  '                scene.drag = Some(Drag::Orbit);' \
  'M25 o gesto que erra a peca volta a morrer com o plano dentro'

# ---- ⭐ A ORDEM DO DONO DE 21/09: «permita pintar mesmo se [nao] tocar um vertex»
# ⚠️ Mesmo caso da M25: a prova de comportamento
# (`um_traco_de_cor_que_comeca_fora_da_peca_pinta`) e' `#[ignore]` + placa, logo
# quem a mata aqui e' o ELO no censo de texto. Sem ele esta mutacao SOBREVIVE.
muta "$APP/input_down.rs" \
  '            if took || scene.brush.verb.paints_color() {' \
  '            if took {' \
  'M26 um traco de cor que comeca fora da peca volta a virar orbita'

# ---- ⭐ O 3.º REPORT DE 21/09: «a tinta so' e' depositada se o pincel esta'
#      sobre um vertex». As duas cercas do dab contam VERTICES e a unidade que
#      a tinta fina escreve e' a AMOSTRA; a terceira e' a lei da folha na
#      unidade certa. ⚠️ As tres tem gate de comportamento, e os tres sao
#      `#[ignore]` + placa — logo quem as mata aqui e' o elo no censo de texto.
muta "$SC/stroke_dab_core.rs" \
  '        if pegada_ja_vazia && !so_amostras {' \
  '        if pegada_ja_vazia {' \
  'M27 a pegada VAZIA volta a matar o dab de cor fina'

muta "$SC/stroke_dab_core.rs" \
  '            if self.footprint.is_empty() && !(so_amostras && pegada_ja_vazia) {' \
  '            if self.footprint.is_empty() {' \
  'M28 a mascara volta a decidir sem um vertice sobre que julgar'

muta "$SC/tinta_fina.rs" \
  '            if corta_a_folha && dot_olho(a.nrm) > crate::dab_alcance::NORMAL_LIMIAR {' \
  '            if false && dot_olho(a.nrm) > crate::dab_alcance::NORMAL_LIMIAR {' \
  'M29 a lei da folha deixa de valer para a AMOSTRA'

# ---- ⭐ O QUARTO CANAL DO DESFAZER (21/09, depois do smoke aprovado). As tres
#      primeiras sao ELOS: a lei tem gates PUROS (que correm aqui e sangram),
#      mas o produto pode deixar de a CHAMAR sem que nenhum deles veja — a
#      prova de comportamento ate' a' tecla e' `#[ignore]` + placa. As duas
#      ultimas matam-se nos gates puros da propria lei.
muta "$APP/history.rs" \
  '            let janela = JanelaFina::do_traco(&do_traco);' \
  '            let janela: Option<JanelaFina> = None;' \
  'M30 o close_stroke deixa de colher a janela do plano emprestado'

muta "$APP/history.rs" \
  '        if self.stroke.touched().is_empty() && finas.is_none() {' \
  '        if self.stroke.touched().is_empty() {' \
  'M31 o portao do close_stroke volta a contar so VERTICES'

# ⚠️ A agulha e' a 3.a linha do bloco: ela quebra a do censo (que leva as TRES
# juntas) E neutraliza a aplicacao — as duas metades na mesma mutacao.
muta "$APP/undo.rs" \
  '                    let inversa = j.troca(obj.tinta.as_mut())?;' \
  '                    let inversa = { drop(j); None }?;' \
  'M32 o quarto canal deixa de ser aplicado no desfazer'

muta "$APP/history_tinta_fina.rs" \
  '        if IdDoPlano::de(t) != self.plano {' \
  '        if false {' \
  'M33 a janela de OUTRO plano deixa de ser largada'

# ⛔ A 1.a redacao desta mutacao apagava um campo da IDENTIDADE e SOBREVIVEU —
# a fixtura do gate sobrescrevia o proprio campo que testava. Hoje ela apaga a
# cerca dos INDICES, que e' uma pergunta diferente e tem gate proprio.
muta "$APP/history_tinta_fina.rs" \
  '        if self.amostras.iter().any(|&i| i as usize >= n) {' \
  '        if false {' \
  'M34 a cerca dos INDICES desaparece e o desfazer volta a poder ESTOURAR'

muta "$APP/history_tinta_fina.rs" \
  '        if t.tocadas().is_empty() {' \
  '        if false {' \
  'M35 um traco que nao tocou uma amostra passa a deixar janela'

# ---- ⭐ O EMPRESTIMO POR DONO (§10.5, a latente da auditoria de 21/09). Os
#      gates da lei chamam a porta DIRECTAMENTE e nenhum gate de produto ve' a
#      troca, porque hoje nenhum gesto muda a peca activa a meio de um traco
#      ⇒ quem mata estas duas e' o ELO no censo de texto.
muta "$APP/history.rs" \
  '            crate::tinta_da_peca::devolve_ao_dono(&mut self.objects, do_traco);' \
  '            let i = self.active;
            let crate::objects::SceneObject { stack, tinta, tinta_suja, .. } = &mut self.objects[i];
            crate::tinta_da_peca::devolve(stack.mesh_mut(), tinta, Some(do_traco));
            *tinta_suja = true;' \
  'M36 o close_stroke volta a devolver o plano a peca ACTIVA'

muta "$APP/input_down.rs" \
  '            let dono = scene.objects[scene.active].id;' \
  '            let dono = crate::objects::ObjectId(u32::MAX);' \
  'M37 o emprestimo deixa de carregar quem o emprestou'

echo
if [ -n "$SO_ANCORAS" ]; then
  # ⚠️ **O sumario tem de dizer o que ele MEDIU.** Aqui nenhum teste correu:
  # dizer «sangram» sobre uma corrida de ancoras seria um instrumento a
  # descrever-se mal, que e' o defeito que este arnes inteiro existe para nao ter.
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ] || exit 1
  exit 0
fi
if [ -n "$FILTRO" ]; then
  echo "MUTACAO: $sangram de $total sangram  ⚠️ SUBCONJUNTO (MUTA_FILTRO='$FILTRO')"
else
  echo "MUTACAO: $sangram de $total sangram"
fi
