# Append parsing

This project implements what I have been referring to as "append parsing".

Expression parsing is often implemented with a combination of recursion and loops, which parse with right associativity and left associativity, respectively. A combination of the two allows one to implement a function that can parse a list of tokens into a syntax tree that contains each operator in a correct position relative to the other operators in the syntax tree, according to precedence rules.

I present an algorithm that allows one to build a correct syntax tree by appending each item consecutively to an existing tree. This has several advantages, including allowing partial parsing of a given token list, as more common parsers store the entire parsing state inside the stack, making it more difficult to manage the state of the current parse.

This is only possible because of a few key observations as to where a token can be appended.

First, if syntax is represented as a binary tree, tokens can only be appended (applied to an existing node) on the rightmost nodes in a tree.

In parsing the expression `2 * 1 + 3`, consider `2 * 1` to have already been parsed. This represents in the expression `((* 2) 1)` (Lisp-style), which can be visually represented as the following binary tree:

```
      _
     / \
    _   1
   / \
  *   2
```

(binary tree braches, here, do not store any data; this is denoted with an underscore)

Thus, the following places any given operator could be placed in the tree are as follows, using `+` as an example:
- `(((* 2) 1) +)`
- `((* 2) (1 +))`

Keep in mind that, in the second item, `1` and `+` would be switched, because of the unique behaviour of infix operators to "take" an already existing node as its child. This means that any applications (e.g. `+ 1`) cannot have the `1` interacted with by any new nodes, regardless of their precedence. This does not have a significant effect on the algorithm.

