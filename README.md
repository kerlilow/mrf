# {[m]atch=[r]eplace:[f]ormat}

[![Tests Status](https://github.com/kerlilow/mrf/workflows/tests/badge.svg)](https://github.com/kerlilow/mrf/actions)

Rename files by pattern matching.

## Demo

## Installation

### From source

Currently, only installation from source using Rust's package manager [Cargo](https://github.com/rust-lang/cargo) is supported.

```sh
$ cargo install mrf
```

## How It Works

### Tokenization

The input string is split into tokens. There are 4 types of tokens:
1. Number - A contiguous string of ASCII digits (0-9).
2. Whitespace - A contiguous string of ASCII whitespaces.
3. Punctuation - A contiguous string of ASCII punctuations.
4. Text - A contiguous string of characters that are none of the above.

For example, the string `example-001` will be tokenized as `[example][-][001]`.

### Matching

Each matcher matches one or more tokens. There are 2 types of matchers:
1. Any - Match any type of tokens, the default matcher. Example: `{}`.
2. Number - Match a Number token, specified with `n`. Example: `{n}`.

Note: A matcher matches the minimum number of tokens required.

For example, the string `example-001` with the replacer string `{}{n}` will be
matched as:
```
[example-][001]
 ^^^^^^^^  ^^^
   Any     Number
```

Note that the Any matcher matches 2 tokens here.

On the other hand, for the same string, if the replacer string `{}{}` will be
matched as:
```
[example][-001]
 ^^^^^^^  ^^^^
   Any     Any
```

Note that this time, the first Any matcher matched the 1 token, while the second
Any matcher matched 2 tokens.

### Replacing

A replacement string may be specified to replace the matched substring with an
equal sign (`=`) in the specifier. Example: `{=replaced}`.

### Formatting

A format specifier may be specified to format the matched substring (or the
replacement, if specified).

The following format specifiers are supported:
1. Padding (aligned to the right) - Specify the desired width. Example: `{:3}`.
2. Zero-padding (aligned to the right) - Specify `0`, followed by the desired
width. Example: `{:03}`.

## Usage

### Rename/move files with `mrf mv`

```
mrf mv <item>... <replacer>
```

#### Examples

##### Replace hyphen with underscore

```sh
$ mrf mv * '{}{=_}{}'
Moving 1 out of 1 items:
    image-001.jpg -> image_001.jpg
```

##### Rename while keeping numbering

```sh
$ mrf mv * '{=photo}{}'
Moving 1 out of 1 items:
    image-001.jpg -> photo-001.jpg
```

##### Add zero padding

```sh
$ mrf mv * '{}{n:03}{}'
Moving 1 out of 1 items:
    image-1.jpg -> image-001.jpg
```

### Execute commands with `mrf exec`

```
mrf exec <command> <item>... <replacer>
```

#### Examples

##### Make directory

```sh
$ mrf exec -r 'mkdir -p' * '{3}{=}'
Matched 1 out of 1 items:
    image-2020-01-01.jpg -> 2020
```

##### Copy files

```sh
$ mrf exec cp * '{}{=_}{}'
Matched 1 out of 1 items:
    image-001.jpg -> image_001.jpg
```

### Map strings with `mrf map` (useful for testing and understanding)

```
mrf map [FLAGS] <item>... <replacer>
```

#### Examples

##### Replace hyphen with underscore

```sh
$ mrf map example-001 '{}{=_}{}'
example-001 -> example_001
```

##### Pipe to cp (consider using the "exec" subcommand instead)

```sh
$ mrf map * '{}{=-}{}' | xargs -0 -n2 cp
```

## Roadmap

- [ ] Nicer error reports
- [ ] Match highlighting
- [ ] More installation methods
- [ ] Exact, prefix, suffix matcher
- [ ] Regex matcher

## License

This project is licensed under the terms of the MIT license.

See the [LICENSE.md](LICENSE.md) file in this repository for more information.
