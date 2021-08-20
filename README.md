# Odra

This may or may not become a forth-inspired language. Possibly targeting V8 / compiled to JS.

## Ideas
- stack based (as pretty much the only src of mutability?)
- focused on parallelism? `: dup fork { return } join`? 
  `return` would terminate current fiber with the top of the stack as the result;
  `join` awaits the fiber handle and puts the result on the stack
- pure FP? with IO type? (no IO type I think - erlang style?)