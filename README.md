# MDMA - Macro Dynamic Meta Analyzer

A pre-compression step that analyzes the input and computes a static dictionary-like structure for tokenization that feeds directly into the entropy coding stage.

"Dynamic" here is a bit fooling, in reality it's closer to blockwise-static - check out [this thread](https://encode.su/threads/3586-Towards-optimal-dictionary-transforms?p=69222&viewfull=1#post69222). Of course, the name is modified because I like the acronym.

## Advantages

Static-blockwise dictionary methods are hella fast decoding-wise and can provide a much stronger optimality claim than LZ77 and LZ78 can (at the expense of encoding time and memory).

The ability to process a whole block at once means the analyzer can find all sorts of interesting patterns across the block and it's not constrained to a buffer of processed data.
