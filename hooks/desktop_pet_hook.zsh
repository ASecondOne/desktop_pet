#!/usr/bin/env zsh

[[ -o interactive ]] || return 0

if [[ -n ${DESKTOP_PET_HOOK_INSTALLED:-} ]]; then
  return 0
fi

typeset -g DESKTOP_PET_HOOK_INSTALLED=1
typeset -g DESKTOP_PET_HOOK_DIR=${${(%):-%N}:A:h}
typeset -g DESKTOP_PET_SOCKET=${DESKTOP_PET_SOCKET:-/tmp/desktop_pet_${USER}.sock}
typeset -g DESKTOP_PET_SENDER=${DESKTOP_PET_SENDER:-$DESKTOP_PET_HOOK_DIR/desktop_pet_send.py}

autoload -Uz add-zsh-hook

__desktop_pet_send() {
  emulate -L zsh

  local kind="$1"
  local command_id="$2"
  local command="$3"
  local stream="$4"
  local text="$5"
  local exit_code="$6"
  local tty_value="$7"
  local cwd_value="$8"

  [[ -S $DESKTOP_PET_SOCKET ]] || return 0
  command -v python3 >/dev/null 2>&1 || return 0
  [[ -f $DESKTOP_PET_SENDER ]] || return 0

  local -a args
  args=(
    --socket "$DESKTOP_PET_SOCKET"
    --kind "$kind"
    --command-id "$command_id"
    --shell-pid "$$"
    --tty "$tty_value"
    --cwd "$cwd_value"
    --command "$command"
  )

  if [[ $kind == output ]]; then
    args+=(--stream "$stream" --text "$text")
  fi

  if [[ $kind == finish ]]; then
    args+=(--exit-code "$exit_code")
  fi

  python3 "$DESKTOP_PET_SENDER" "${args[@]}" >/dev/null 2>&1 || true
}

__desktop_pet_pipe() {
  emulate -L zsh

  local stream="$1"
  local command_id="$2"
  local command="$3"
  local tty_value="$4"
  local cwd_value="$5"
  local line

  while IFS= read -r line || [[ -n $line ]]; do
    if [[ $stream == stderr ]]; then
      print -ru2 -- "$line"
    else
      print -r -- "$line"
    fi

    __desktop_pet_send output "$command_id" "$command" "$stream" "$line" "" "$tty_value" "$cwd_value"
  done
}

__desktop_pet_exec() {
  local command="$1"
  local tty_value
  local cwd_value="$PWD"
  local command_id="${EPOCHREALTIME:-$SECONDS}:$$:$RANDOM"
  local exit_status

  [[ -z ${command//[[:space:]]/} ]] && return 0

  tty_value=$(tty 2>/dev/null) || tty_value="unknown"

  __desktop_pet_send start "$command_id" "$command" "" "" "" "$tty_value" "$cwd_value"

  {
    eval "$command"
  } > >(__desktop_pet_pipe stdout "$command_id" "$command" "$tty_value" "$cwd_value") \
    2> >(__desktop_pet_pipe stderr "$command_id" "$command" "$tty_value" "$cwd_value")

  exit_status=$?
  __desktop_pet_send finish "$command_id" "$command" "" "" "$exit_status" "$tty_value" "$cwd_value"
  return "$exit_status"
}

__desktop_pet_history_filter() {
  emulate -L zsh
  [[ $1 == __desktop_pet_exec\ * ]] && return 1
  return 0
}

__desktop_pet_accept_line() {
  emulate -L zsh

  local command="$BUFFER"

  if [[ -z ${command//[[:space:]]/} || $command == __desktop_pet_exec\ * ]]; then
    zle .accept-line
    return
  fi

  print -sr -- "$command"
  BUFFER="__desktop_pet_exec ${(qqq)command}"
  zle .accept-line
}

add-zsh-hook zshaddhistory __desktop_pet_history_filter
zle -N accept-line __desktop_pet_accept_line
