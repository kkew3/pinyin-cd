import os
import sys
import itertools
import collections
import argparse

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


def get_first_split_pattern(path):
    """
    :param path: normalized path

    >>> get_first_split_pattern('../../hello/world')
    ('../..', ('hello', 'world'))
    >>> get_first_split_pattern('hello/world/again')
    ('.', ('hello', 'world', 'again'))
    >>> get_first_split_pattern('../hello')
    ('..', ('hello',))
    >>> get_first_split_pattern('../..')
    ('../..', ())
    >>> get_first_split_pattern('hello')
    ('.', ('hello',))
    >>> get_first_split_pattern('/')
    ('/', ())
    >>> get_first_split_pattern('/hello')
    ('/', ('hello',))
    >>> get_first_split_pattern('.')
    ('.', ())

    TODO not supporting win32 absolute path
    """
    assert path

    basedir = []
    pattern = []
    in_basedir = True
    for component in path.split(os.path.sep):
        if in_basedir:
            if not component:
                basedir.append(os.path.sep)
                in_basedir = False
            elif component == os.curdir:
                basedir.append(component)
                in_basedir = False
            elif component == os.pardir:
                basedir.append(component)
            else:
                in_basedir = False
                pattern.append(component)
        else:
            if component:
                pattern.append(component)
    if not basedir:
        basedir.append(os.curdir)
    basedir = os.path.join(*basedir)
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


def make_parser():
    parser = argparse.ArgumentParser(prog='pycd')
    parser.add_argument('-i', action='store_true', help='match first letters')
    parser.add_argument('-p', action='store_true', help='match prefix')
    parser.add_argument('pattern', nargs='?', const='')
    return parser


def main():
    args = make_parser().parse_args()
    for match in resolve(args.pattern, args.i, args.p):
        print(match)


if __name__ == '__main__':
    main()
