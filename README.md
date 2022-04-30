# Description

`cd` to zh-hans directory using *PinYin*.


# Installation

```bash
git clone https://github.com/kkew3/pinyin-cd.git
cd pinyin-cd
python3 -m virtualenv rt
. rt/bin/activate
pip install pypinyin
deactivate
```

Then source `pinyincd.sh` by absolute path in `.bashrc` or `.zshrc`.


# Alternative Rust Installation

```bash
git clone https://github.com/kkew3/pinyin-cd.git
cd pinyin-cd/pinyincd_rust_backend
cargo build --release
```

Then source `pinyincd_rust.sh` by absolute path in `.bashrc` or `.zshrc`.

Rust version is about 10x faster than Python version.


# Usage

- `pycd DIR`: replace each zh-hans character with the pinyin without tones
- `pycd -i DIR`: replace each zh-hans character with the pinyin first letters
- `pycd [OPTION] -p DIR`: match the prefix of each component of `DIR` (like `zsh`)

`..` and `.` are supported in `DIR`.


# Example

Given directory

	./
	|- 中心/
	   |- 蛇/
	   |- 折扣sh/
	|- 威妥玛拼音/
	   |- 战略/
	   |- 你好/

`pycd -i zx/zksh` switches to `./中心/折扣sh`, since the PinYin of `中心` is `[z]hong[x]in` and that of `折扣` is `[z]he[k]ou`.
Likewise, `pycd -p weituoma/zhan` switches to `./威妥玛拼音/战略`.
