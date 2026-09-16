#!/usr/bin/env bash
# Provas de mutação do TOP-20 #16 (`ScriptProperties` + anexar um script pela UI) — W1..W4.
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ **CONTROLO sobre o próprio FILTRO** — um filtro que casa ZERO testes sai VERDE.
# ⚠️ **Toda troca é por `muta`, que ABORTA se o texto não aparecer exactamente o número esperado de
#    vezes** — um `sed` que não casa é um no-op silencioso e imprime «SOBREVIVEU» sobre nada.
# ⚠️ **As provas do PRAZO correm com `timeout`**: sem o prazo o laço do teste nunca acaba, e isso É
#    o sangue que elas procuram.
set -u
cd "$(dirname "$0")/../../.." || exit 1

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1").$2"; }
restaura() { cp "$TMP/$(basename "$1").$2" "$1"; touch "$1"; }

muta() { # ficheiro vezes antigo novo
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys
p, n, old, new = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
s = open(p).read()
c = s.count(old)
if c != n:
    sys.exit(f"  ⛔ ANCORA: {old!r} aparece {c} vezes em {p} (esperado {n})")
open(p, "w").write(s.replace(old, new))
PY
}

prova() { # nome crate filtro [timeout_s]
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(timeout "${4:-900}" cargo test -p "$2" --all-targets -- "$3" 2>&1); rc=$?
  if [ "$rc" = 124 ]; then
    echo "  ✅ sangrou (o teste nao acabou em ${4}s — o prazo e' o que o fazia acabar)"
    return
  fi
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto mutado"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

# um bloco = guarda, muta, prova, restaura. Se a âncora falhar, restaura e conta como falha.
bloco() { # nome crate filtro ficheiro vezes antigo novo [timeout]
  guarda "$4" b
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3" "${8:-900}"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}

LEI=crates/ph2d-script/src/props.rs
MOD=crates/ph2d-script/src/module.rs
CENA=crates/ph2d-script/src/scene.rs
HOST=crates/ph2d-script/src/host.rs
PONTE=crates/ph2d-app-components/src/script_bridge.rs
INSP=crates/ph2d-app-components/src/script_inspector.rs
POP=crates/ph2d-panel-inspector/src/populate_script.rs
EVT=crates/ph2d-panel-inspector/src/event_script.rs
PINT=crates/ph2d-panel-inspector/src/sections/script.rs
SEM=crates/ph2d-panel-inspector/src/sync_script.rs
OUTBOX=shells/desktop/src/render_loop/fase_signal_outbox.rs
REB=shells/desktop/src/render_loop/fase_fabrica_e_morte.rs

# ── W1: a lei (o oráculo e as divergências) ──────────────────────────────────
bloco "D1: um valor POSTO igual ao default continua proprio" ph2d-script "d1_um_valor_posto" \
  "$LEI" 1 "Some(v) if v.kind() == d.default.kind() =>" "Some(v) if v.kind() == d.default.kind() && *v != d.default =>"
bloco "D3: o do tipo errado nao se aplica" ph2d-script "d3_um_texto" \
  "$LEI" 1 "Some(v) if v.kind() == d.default.kind() =>" "Some(v) if true =>"
bloco "Q10: «nao sei» nao e' «nada»" ph2d-script "q10_com_o_script" \
  "$LEI" 1 "let Some(decls) = decls else {" "let Some(decls) = decls.or(Some(&[])) else {"
bloco "Q7: a ordem e' a da declaracao" ph2d-script "q7_a_ordem" \
  "$LEI" 1 "    for (name, value) in own {" "    out.values.sort_by(|a, b| a.name.cmp(&b.name));
    for (name, value) in own {"
bloco "PROPS_MAX recusa com nome" ph2d-script "um_script_nao_oferece_mais" \
  "$LEI" 1 "if previous.len() >= PROPS_MAX {" "if false && previous.len() >= PROPS_MAX {"
bloco "o nome id e' da casa" ph2d-script "as_declaracoes_mal_formadas" \
  "$LEI" 1 'pub const RESERVED_NAMES: &[&str] = &["id"];' 'pub const RESERVED_NAMES: &[&str] = &[];'
bloco "UM AMBIENTE POR SCRIPT" ph2d-script "dois_scripts_nao_se_pisam" \
  "$MOD" 1 "        .set_environment(env.clone())
" ""
bloco "o balde das declaracoes SAI depois da carga" ph2d-script "declarar_dentro_de_um_gancho" \
  "$MOD" 1 "let sink = lua.remove_app_data::<DeclSink>().unwrap_or_default();" \
  "let sink = lua.app_data_mut::<DeclSink>().map(|mut s| std::mem::take(&mut *s)).unwrap_or_default();"

# ── W2a: o executor ──────────────────────────────────────────────────────────
bloco "a ordem e' a da IDENTIDADE" ph2d-script "quem_emite_primeiro" \
  "$CENA" 1 "    out.sort_by_key(|(sid, bits, _)| (*sid, *bits));" "    let _ = out.len();"
bloco "REBOBINAR e' RENASCER" ph2d-script "o_init_corre_uma_vez" \
  "$CENA" 1 "        let n = self.instances.len();
        self.instances.clear();" "        let n = self.instances.len();"
bloco "cada escrita tem DONO (campo desconhecido)" ph2d-script "um_campo_desconhecido" \
  "$CENA" 1 ".find(|w| pose_field(&w.field).is_none())" ".find(|w| false && pose_field(&w.field).is_none())"
bloco "um NaN nao chega a pose" ph2d-script "um_numero_que_nao_e_numero" \
  "$CENA" 1 ".find(|w| !w.value.is_finite())" ".find(|w| false && !w.value.is_finite())"
bloco "o PRAZO de um quadro (gancho)" ph2d-script "um_laco_sem_saida_num_gancho" \
  "$CENA" 1 "        self.arm();
        let out = f();" "        let out = f();" 90
bloco "RECARREGAR nao renasce" ph2d-script "recarregar_troca_as_funcoes" \
  "$CENA" 1 "                slot.module = Some(m);" "                slot.module = Some(m);
                self.instances.retain(|_, i| i.path != path);"
bloco "a mao do artista manda SO' onde mexeu" ph2d-script "editar_um_numero_a_meio" \
  "$CENA" 1 "if inst.applied.get(&p.name) == Some(&p.value) {" "if false && inst.applied.get(&p.name) == Some(&p.value) {"
bloco "um sinal sem nome e' erro" ph2d-script "um_sinal_sem_nome" \
  "$HOST" 1 "        if name.trim().is_empty() {" "        if false {"

# ── W2b: a ponte e o ledger ──────────────────────────────────────────────────
bloco "um script que PAROU continua a conduzir" ph2d-app-components "um_script_que_parou" \
  "$PONTE" 1 "} else if drive.still_driving(entity, Driver::ScriptPose) {" "} else if false && drive.still_driving(entity, Driver::ScriptPose) {"
bloco "o arrasto na pausa e' a OUTRA MAO" ph2d-app-components "arrastar_na_pausa" \
  "$PONTE" 1 "            drive.driven(entity, Driven::ScriptPose(now), Driven::ScriptPose(now));" "            let _ = now;"
bloco "rebobinar DEVOLVE a pose" ph2d-app-components "a_corrida_move_o_objecto" \
  "$PONTE" 1 "        drive.release_to_authored(sim, entity, Driver::ScriptPose);" "        let _ = entity;"
bloco "um passo fixo, UMA chamada" ph2d-app-components "dois_tiques_num_quadro" \
  "$PONTE" 1 "        for _ in 0..ticks {" "        for _ in 0..ticks.min(1) {"
bloco "parado, nada corre" ph2d-app-components "parado_nada_corre" \
  "$PONTE" 1 "    if playing {" "    if true {"

# ── W3: o instantâneo, o dreno e o painel ────────────────────────────────────
bloco "Reset/Remove largam o valor" ph2d-app-components "por_o_numero_do_default" \
  "$INSP" 1 "E::Forget(n) => cfg.own.contains_key(n).then(|| Mexe::Larga(n.clone()))," "E::Forget(_) => None,"
bloco "sem ficheiro diz-se NoFile" ph2d-app-components "o_estado_do_ficheiro" \
  "$INSP" 1 "_ if cfg.source.trim().is_empty() => InspectorScriptStatus::NoFile," "_ if false => InspectorScriptStatus::NoFile,"
bloco "o orfao diz o que o script quer" ph2d-app-components "os_orfaos_dizem" \
  "$INSP" 1 "OrphanWhy::WrongKind { declared } => Some(declared.label())," "OrphanWhy::WrongKind { .. } => None,"
bloco "os guardados contam-se" ph2d-app-components "com_o_script_partido" \
  "$INSP" 1 "kept: r.kept.len()," "kept: 0,"
bloco "cancelar o dialogo nao escreve" ph2d-app-components "browse_escreve_o_que_o_dialogo" \
  "$INSP" 1 "                None => continue," "                None => E::Source(String::new()),"
bloco "as caixas estao no populate" ph2d-panel-inspector "a_caixa_manda_o_contrario" \
  "$POP" 1 "    for id in crate::ids::INSP_SCRIPT_BOOL {" "    for id in crate::ids::INSP_SCRIPT_BOOL.iter().take(0).copied() {"
bloco "a caixa e' semeada do snapshot" ph2d-panel-inspector "a_caixa_manda_o_contrario" \
  "$SEM" 1 "            *value = if *on {" "            *value = if !*on {"
bloco "Reset manda o nome da SUA linha" ph2d-panel-inspector "reset_e_remove" \
  "$EVT" 1 "            info.props
                .get(i)" "            info.props
                .get(i + 1)"
bloco "cada linha pinta o controlo do tipo dela" ph2d-panel-inspector "cada_linha_pinta" \
  "$PINT" 1 "            let id = ids::INSP_SCRIPT_BOOL[i];" "            let id = ids::INSP_SCRIPT_NUM[i];"

# ── W2b/W4: a shell ──────────────────────────────────────────────────────────
bloco "o script tem cursor PROPRIO e fala antes da tabela" ph2d-host-desktop "o_script_emite_antes" \
  "$OUTBOX" 1 ".read(&mut self.signal_readers.script)" ".read(&mut self.signal_readers.toast)"
bloco "os scripts renascem no invariante" ph2d-host-desktop "os_scripts_renascem" \
  "$REB" 1 "            if let Some(host) = script.as_mut() {
                repostos +=
                    ph2d_app_components::script_bridge::rewind(host, sim, &mut self.preview_drive);
            }" "            let _ = &script;"
bloco "o painel pinta tudo o que o script aceita" ph2d-host-desktop "o_script_nao_declara_mais" \
  "$LEI" 1 "pub const PROPS_MAX: usize = 32;" "pub const PROPS_MAX: usize = 31;"

echo
echo "== $TOTAL provas, $FALHAS que NAO sangraram =="
rm -rf "$TMP"
[ "$FALHAS" = 0 ]
