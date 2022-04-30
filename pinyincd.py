import os
import sys
import itertools
import collections
import pathlib
import posixpath
import ntpath

import pypinyin


if sys.platform == 'win32':
    print('win32 not supported currently', file=sys.stderr)
    sys.exit(1)


def to_pinyin(string, pinyin_style: pypinyin.Style):
    """
    >>> list(to_pinyin('hello', pypinyin.Style.NORMAL))
    ['hello']
    """
    return map(''.join, itertools.product(
        *pypinyin.pinyin(list(string), style=pinyin_style, heteronym=True)))


# Reference: https://www.oreilly.com/library/view/python-cookbook/0596001673/ch04s16.html
def _path_components(path, _pathmodule=None):
    """
    :param path: normalized path

    >>> import posixpath
    >>> _path_components('../../hello/world', posixpath)
    ['..', '..', 'hello', 'world']
    >>> _path_components('hello/world/again', posixpath)
    ['hello', 'world', 'again']
    >>> _path_components('../hello', posixpath)
    ['..', 'hello']
    >>> _path_components('../..', posixpath)
    ['..', '..']
    >>> _path_components('hello', posixpath)
    ['hello']
    >>> _path_components('/', posixpath)
    ['/']
    >>> _path_components('/hello', posixpath)
    ['/', 'hello']
    >>> _path_components('.', posixpath)
    ['.']
    >>> _path_components('./hello', posixpath)
    ['.', 'hello']
    >>> import ntpath
    >>> _path_components('.', ntpath)
    ['.']
    >>> _path_components('C:', ntpath)
    ['C:']
    >>> _path_components('C:\\\\', ntpath)
    ['C:\\\\']
    >>> _path_components('C:\\\\hello', ntpath)
    ['C:\\\\', 'hello']
    >>> _path_components('hello\\\\world', ntpath)
    ['hello', 'world']
    """
    pathmodule = _pathmodule or os.path
    # os.path is either posixpath or ntpath
    if pathmodule not in (posixpath, ntpath):
        raise ValueError('invalid pathmodule {!r}'.format(pathmodule))

    components = []
    while True:
        parts = pathmodule.split(path)
        # sentinel for absolute paths
        if parts[0] == path:
            components.append(parts[0])
            break
        # sentinel for relative paths
        if parts[1] == path:
            components.append(parts[1])
            break
        path = parts[0]
        components.append(parts[1])
    components.reverse()
    return components


def get_first_split_pattern(path, _pathmodule=None):
    """
    :param path: normalized path

    >>> import posixpath
    >>> get_first_split_pattern('../../hello/world', posixpath)
    ('../..', ('hello', 'world'))
    >>> get_first_split_pattern('hello/world/again', posixpath)
    ('.', ('hello', 'world', 'again'))
    >>> get_first_split_pattern('../hello', posixpath)
    ('..', ('hello',))
    >>> get_first_split_pattern('../..', posixpath)
    ('../..', ())
    >>> get_first_split_pattern('hello', posixpath)
    ('.', ('hello',))
    >>> get_first_split_pattern('/', posixpath)
    ('/', ())
    >>> get_first_split_pattern('/hello', posixpath)
    ('/', ('hello',))
    >>> get_first_split_pattern('.', posixpath)
    ('.', ())
    >>> import ntpath
    >>> get_first_split_pattern('.', ntpath)
    ('.', ())
    >>> get_first_split_pattern('C:', ntpath)
    ('C:', ())
    >>> get_first_split_pattern('C:\\\\', ntpath)
    ('C:\\\\', ())
    >>> get_first_split_pattern('C:hello', ntpath)
    ('C:', ('hello',))
    >>> get_first_split_pattern('C:\\\\hello', ntpath)
    ('C:\\\\', ('hello',))
    >>> get_first_split_pattern('C:\\\\hello\\\\world', ntpath)
    ('C:\\\\', ('hello', 'world'))
    >>> get_first_split_pattern('hello', ntpath)
    ('.', ('hello',))
    >>> get_first_split_pattern('hello\\\\world', ntpath)
    ('.', ('hello', 'world'))
    """
    if not path:
        raise ValueError('path must not be empty')

    pathmodule = _pathmodule or os.path
    # os.path is either posixpath or ntpath
    if pathmodule not in (posixpath, ntpath):
        raise ValueError('invalid pathmodule {!r}'.format(pathmodule))

    anchor = {
        posixpath: pathlib.PurePosixPath,
        ntpath: pathlib.PureWindowsPath,
    }[pathmodule](path).anchor

    basedir = []
    pattern = []
    in_basedir = True
    for component in _path_components(path, _pathmodule):
        if in_basedir:
            if component in (anchor, os.pardir):
                basedir.append(component)
            elif component == os.curdir:
                basedir.append(component)
                in_basedir = False
            else:
                in_basedir = False
                pattern.append(component)
        else:
            pattern.append(component)
    if not basedir:
        basedir.append(os.curdir)
    basedir = pathmodule.join(*basedir)
    return basedir, tuple(pattern)


def resolve(path, pinyin_firstletter: bool, prefix: bool):
    """
    Returns the target directory.
    """
    if not path:
        return [os.path.expanduser('~')]
    path = os.path.normpath(path)

    pinyin_style = (pypinyin.Style.FIRST_LETTER if pinyin_firstletter
                    else pypinyin.Style.NORMAL)
    basedir, pattern = get_first_split_pattern(path)
    matched_directories = []
    queue = collections.deque([(basedir, pattern)])
    while queue:
        basedir, pattern = queue.popleft()
        if not pattern:
            matched_directories.append(basedir)
        else:
            cur_pattern = pattern[0]
            if not prefix:
                with os.scandir(basedir) as it:
                    for entry in it:
                        if entry.is_dir():
                            if cur_pattern in to_pinyin(entry.name,
                                                        pinyin_style):
                                queue.append((
                                    os.path.join(basedir, entry.name),
                                    pattern[1:],
                                ))
            else:
                with os.scandir(basedir) as it:
                    for entry in it:
                        if entry.is_dir():
                            for py in to_pinyin(entry.name, pinyin_style):
                                if py.startswith(cur_pattern):
                                    queue.append((
                                        os.path.join(basedir, entry.name),
                                        pattern[1:],
                                    ))
                                    break
    return matched_directories


class Args:
    def __init__(self):
        self.i = False
        self.p = False
        self.pattern = ''


def parse_args():
    args = Args()
    args.i = bool(sys.argv[1])
    args.p = bool(sys.argv[2])
    args.pattern = sys.argv[3]
    return args


def main():
    args = parse_args()
    for match in resolve(args.pattern, args.i, args.p):
        print(match)


if __name__ == '__main__':
    main()
