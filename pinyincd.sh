__pycd_usage() {
	echo "usage: pycd [OPTIONS..] [PATTERN]"
	echo
	echo "OPTIONS"
	echo "  --       mark the end of OPTIONS"
	echo "  -h       show this help message and exit"
	echo "  -i       match first letters"
	echo "  -p       match prefix"
	echo "  -l       don't cd, list matches only"
}

pycd() {
	local args_i
	local args_p
	local args_h
	local args_l
	local parsing_opts=1
	local args_pattern

	while [ -n "$1" ]; do
		if [ -n "$parsing_opts" ]; then
			case "$1" in
				--)
					parsing_opts=
					;;
				-*)
					case "$1" in -*h*) args_h=1; ;; esac
					case "$1" in -*i*) args_i=1; ;; esac
					case "$1" in -*p*) args_p=1; ;; esac
					case "$1" in -*l*) args_l=1; ;; esac
					;;
				*)
					args_pattern="$1"
					;;
			esac
		else
			args_pattern="$1"
		fi
		shift
	done
	if [ -n "$args_h" ]; then
		__pycd_usage
		return 0
	fi


    # reference: https://stackoverflow.com/a/54755784/7881370
    local pycd_basedir="$(dirname "${BASH_SOURCE[0]:-${(%):-%x}}")"
	if [ -n "$args_l" ]; then
		"$pycd_basedir/rt/bin/python" "$pycd_basedir/pinyincd.py" "$args_i" "$args_p" "$args_pattern"
	else
		selected="$("$pycd_basedir/rt/bin/python" "$pycd_basedir/pinyincd.py" "$args_i" "$args_p" "$args_pattern" \
			| fzf --exit-0 --select-1)"
		if [ -z "$selected" ]; then
			return 1
		else
			cd "$selected"
		fi
	fi
}
