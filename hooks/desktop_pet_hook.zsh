#!/usr/bin/env zsh

[[ -o interactive ]] || return 0

if [[ -n ${DESKTOP_PET_HOOK_INSTALLED:-} ]]; then
  return 0
fi

typeset -g DESKTOP_PET_HOOK_INSTALLED=1
typeset -g DESKTOP_PET_HOOK_DIR=${${(%):-%N}:A:h}
typeset -g DESKTOP_PET_SOCKET=${DESKTOP_PET_SOCKET:-/tmp/desktop_pet_${USER}.sock}
typeset -g DESKTOP_PET_SENDER=${DESKTOP_PET_SENDER:-$DESKTOP_PET_HOOK_DIR/desktop_pet_send.py}
typeset -g DESKTOP_PET_PTY_RUNNER=${DESKTOP_PET_PTY_RUNNER:-$DESKTOP_PET_HOOK_DIR/desktop_pet_pty.py}
typeset -g DESKTOP_PET_CAPTURE_OUTPUT=${DESKTOP_PET_CAPTURE_OUTPUT:-smart}
typeset -g DESKTOP_PET_ACTIVE_COMMAND_ID=""
typeset -g DESKTOP_PET_ACTIVE_COMMAND=""
typeset -g DESKTOP_PET_ACTIVE_TTY="unknown"
typeset -g DESKTOP_PET_ACTIVE_CWD=""

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

__desktop_pet_capture_mode() {
  emulate -L zsh

  local mode="${DESKTOP_PET_CAPTURE_OUTPUT:-smart}"
  mode="${(L)mode}"

  case "$mode" in
    1|always|on|true|yes)
      print -r -- "always"
      ;;
    0|off|false|no)
      print -r -- "off"
      ;;
    auto|smart|"")
      print -r -- "smart"
      ;;
    *)
      print -r -- "smart"
      ;;
  esac
}

__desktop_pet_should_capture_output() {
  emulate -L zsh

  local command="$1"
  local mode="$(__desktop_pet_capture_mode)"
  local -a words
  local program
  local subcommand

  case "$mode" in
    off)
      return 1
      ;;
    always)
      return 0
      ;;
  esac

  words=(${(z)command})
  (( ${#words} )) || return 1

  program="${words[1]##*/}"
  subcommand="${words[2]:-}"

  case "$program" in
    __desktop_pet_exec|.|source|eval|exec|exit|logout|suspend|cd|pushd|popd|dirs|fg|bg|jobs|wait|disown|history|fc|alias|unalias|bindkey|zle|trap|stty|reset|clear|tput)
      return 1
      ;;
    vim|nvim|vi|view|nano|emacs|less|more|most|man|top|htop|btop|btm|watch|fzf|fzf-tmux|ssh|sftp|mosh|tmux|screen|ranger|mc|lazygit|tig|k9s|kubectl|helm)
      return 1
      ;;
    python|python3|ipython|node|ruby|irb|lua|R)
      (( ${#words} <= 2 )) && return 1
      ;;
  esac

  case "$program:$subcommand" in
    git:help|git:log|git:show|git:diff|git:blame|git:reflog)
      return 1
      ;;
  esac

  return 0
}

__desktop_pet_should_use_pty_capture() {
  emulate -L zsh

  local command="$1"
  local -a words
  local program
  local kind
  local token

  command -v python3 >/dev/null 2>&1 || return 1
  [[ -f $DESKTOP_PET_PTY_RUNNER ]] || return 1

  words=(${(z)command})
  (( ${#words} )) || return 1

  for token in "${words[@]}"; do
    case "$token" in
      ';'|'&'|'&&'|'||'|'|'|'|&'|'(' | ')' | '{' | '}')
        return 1
        ;;
    esac
  done

  program="${words[1]}"

  while [[ $program == [A-Za-z_][A-Za-z0-9_]*=* ]]; do
    shift words
    (( ${#words} )) || return 1
    program="${words[1]}"
  done

  kind=$(whence -w -- "$program" 2>/dev/null)
  kind="${kind##*: }"

  if [[ $kind == command ]]; then
    return 0
  fi

  [[ $program == */* && -x $program ]] || return 1
  return 0
}

__desktop_pet_set_last_status() {
  return "$1"
}

__desktop_pet_run_preexec_hooks() {
  emulate -L zsh

  local command="$1"
  local hook

  if (( $+functions[preexec] )); then
    preexec "$command" "$command" "$command" || true
  fi

  for hook in "${preexec_functions[@]}"; do
    [[ $hook == __desktop_pet_preexec ]] && continue
    (( $+functions[$hook] )) || continue
    "$hook" "$command" "$command" "$command" || true
  done
}

__desktop_pet_run_precmd_hooks() {
  emulate -L zsh

  local exit_status="$1"
  local hook

  if (( $+functions[precmd] )); then
    __desktop_pet_set_last_status "$exit_status"
    precmd || true
  fi

  for hook in "${precmd_functions[@]}"; do
    [[ $hook == __desktop_pet_precmd ]] && continue
    (( $+functions[$hook] )) || continue
    __desktop_pet_set_last_status "$exit_status"
    "$hook" || true
  done
}

__desktop_pet_clear_active_command() {
  emulate -L zsh

  typeset -g DESKTOP_PET_ACTIVE_COMMAND_ID=""
  typeset -g DESKTOP_PET_ACTIVE_COMMAND=""
  typeset -g DESKTOP_PET_ACTIVE_TTY="unknown"
  typeset -g DESKTOP_PET_ACTIVE_CWD=""
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
    line=${line%$'\r'}

    if [[ $stream == stderr ]]; then
      print -ru2 -- "$line"
    else
      print -r -- "$line"
    fi

    __desktop_pet_send output "$command_id" "$command" "$stream" "$line" "" "$tty_value" "$cwd_value"
  done
}

__desktop_pet_exec() {
  emulate -L zsh

  local command="$1"
  local tty_value
  local cwd_value="$PWD"
  local command_id="${EPOCHREALTIME:-$SECONDS}:$$:$RANDOM"
  local exit_status

  [[ -z ${command//[[:space:]]/} ]] && return 0

  tty_value=$(readlink "/proc/$$/fd/2" 2>/dev/null)
  [[ $tty_value == /dev/* ]] || tty_value=$(tty < /dev/tty 2>/dev/null)
  [[ -n $tty_value ]] || tty_value="unknown"

  __desktop_pet_send start "$command_id" "$command" "" "" "" "$tty_value" "$cwd_value"

  if __desktop_pet_should_use_pty_capture "$command"; then
    python3 "$DESKTOP_PET_PTY_RUNNER" \
      --shell "${SHELL:-/bin/sh}" \
      --command "$command" \
      | __desktop_pet_pipe pty "$command_id" "$command" "$tty_value" "$cwd_value"
    exit_status=${pipestatus[1]}
  else
    {
      eval "$command"
    } > >(__desktop_pet_pipe stdout "$command_id" "$command" "$tty_value" "$cwd_value") \
      2> >(__desktop_pet_pipe stderr "$command_id" "$command" "$tty_value" "$cwd_value")
    exit_status=$?
  fi

  __desktop_pet_send finish "$command_id" "$command" "" "" "$exit_status" "$tty_value" "$cwd_value"
  return "$exit_status"
}

__desktop_pet_preexec() {
  emulate -L zsh

  local command="$1"
  local tty_value
  local cwd_value="$PWD"
  local command_id="${EPOCHREALTIME:-$SECONDS}:$$:$RANDOM"

  [[ -z ${command//[[:space:]]/} ]] && return 0
  [[ $command == __desktop_pet_exec\ * ]] && return 0

  tty_value=$(readlink "/proc/$$/fd/2" 2>/dev/null)
  [[ $tty_value == /dev/* ]] || tty_value=$(tty < /dev/tty 2>/dev/null)
  [[ -n $tty_value ]] || tty_value="unknown"

  typeset -g DESKTOP_PET_ACTIVE_COMMAND_ID="$command_id"
  typeset -g DESKTOP_PET_ACTIVE_COMMAND="$command"
  typeset -g DESKTOP_PET_ACTIVE_TTY="$tty_value"
  typeset -g DESKTOP_PET_ACTIVE_CWD="$cwd_value"

  __desktop_pet_send start "$command_id" "$command" "" "" "" "$tty_value" "$cwd_value"
  return 0
}

__desktop_pet_precmd() {
  local exit_status=$?
  emulate -L zsh

  local command_id="$DESKTOP_PET_ACTIVE_COMMAND_ID"

  [[ -n $command_id ]] || return 0

  __desktop_pet_send finish \
    "$command_id" \
    "$DESKTOP_PET_ACTIVE_COMMAND" \
    "" \
    "" \
    "$exit_status" \
    "$DESKTOP_PET_ACTIVE_TTY" \
    "$DESKTOP_PET_ACTIVE_CWD"

  __desktop_pet_clear_active_command
  return 0
}

__desktop_pet_zshexit() {
  local exit_status=$?
  emulate -L zsh

  local command_id="$DESKTOP_PET_ACTIVE_COMMAND_ID"

  [[ -n $command_id ]] || return 0

  __desktop_pet_send finish \
    "$command_id" \
    "$DESKTOP_PET_ACTIVE_COMMAND" \
    "" \
    "" \
    "$exit_status" \
    "$DESKTOP_PET_ACTIVE_TTY" \
    "$DESKTOP_PET_ACTIVE_CWD"

  __desktop_pet_clear_active_command
  return 0
}

__desktop_pet_history_filter() {
  emulate -L zsh
  [[ $1 == __desktop_pet_exec\ * ]] && return 1
  return 0
}

__desktop_pet_accept_line() {
  emulate -L zsh

  local command="$BUFFER"
  local exit_status

  if [[ -z ${command//[[:space:]]/} || $command == __desktop_pet_exec\ * ]]; then
    zle .accept-line
    return
  fi

  if ! __desktop_pet_should_capture_output "$command"; then
    zle .accept-line
    return
  fi

  print -sr -- "$command"
  __desktop_pet_run_preexec_hooks "$command"
  zle -I
  BUFFER=""
  CURSOR=0
  __desktop_pet_exec "$command"
  exit_status=$?
  __desktop_pet_run_precmd_hooks "$exit_status"
  __desktop_pet_set_last_status "$exit_status"
  zle reset-prompt
  return "$exit_status"
}

# Default mode mirrors stdout/stderr for ordinary commands and falls back to
# start/finish only for interactive or terminal-heavy commands.
add-zsh-hook preexec __desktop_pet_preexec
add-zsh-hook precmd __desktop_pet_precmd
add-zsh-hook zshexit __desktop_pet_zshexit
add-zsh-hook zshaddhistory __desktop_pet_history_filter
zle -N accept-line __desktop_pet_accept_line
