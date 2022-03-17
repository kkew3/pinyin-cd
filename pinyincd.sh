pycd() {
    # reference: https://stackoverflow.com/a/54755784/7881370
    local pycd_basedir="$(dirname "${BASH_SOURCE[0]:-${(%):-%x}}")"

    if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
        "$pycd_basedir/rt/bin/python" "$pycd_basedir/pinyincd.py" --help
    else
        selected="$("$pycd_basedir/rt/bin/python" "$pycd_basedir/pinyincd.py" "$@" | fzf --exit-0 --select-1)"
        if [ -z "$selected" ]; then
			return 1
		else
			cd "$selected"
		fi
	fi

}
