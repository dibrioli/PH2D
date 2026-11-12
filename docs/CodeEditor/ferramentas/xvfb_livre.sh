# shellcheck shell=bash
# Um ecrã Xvfb LIVRE, para `source` (11/11): o próprio Xvfb escolhe o número (`-displayfd`) e só o escreve
# quando já aceita ligações — nem número fixo (`:79`, `:$((60 + CENA + PASSO))`…) a colidir com outra sessão
# ou outra corrida em paralelo, nem `sleep 1` cego à espera dele.
# Uso:   . "<raiz>/docs/CodeEditor/ferramentas/xvfb_livre.sh"
#        xvfb_livre 1600x900x24 "$T/xvfb.log" [prazo_s] || exit 2
# Depois: DSP=:N (e DISPLAY exportado), XV = pid do Xvfb (o do `timeout` quando há prazo: o TERM chega-lhe).
xvfb_livre() {
  local geo=${1:?geometria} log=${2:?log} prazo=${3:-} fifo n=
  fifo=$(mktemp -u "${TMPDIR:-/tmp}/xvfb_livre.XXXXXX")
  mkfifo "$fifo" || return 1
  if [ -n "$prazo" ]; then
    timeout "$prazo" Xvfb -displayfd 3 -screen 0 "$geo" -nolisten tcp 3>"$fifo" >"$log" 2>&1 &
  else
    Xvfb -displayfd 3 -screen 0 "$geo" -nolisten tcp 3>"$fifo" >"$log" 2>&1 &
  fi
  XV=$!
  # Se o Xvfb morre antes de escrever, o fifo fecha e o `read` volta vazio (sem esperar o prazo).
  read -r -t 30 n <"$fifo"
  rm -f "$fifo"
  [ -n "$n" ] || { echo "xvfb_livre: o Xvfb não subiu (ver $log)" >&2; return 1; }
  DSP=":$n"
  export DISPLAY="$DSP"
}
