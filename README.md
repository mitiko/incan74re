# MDMA - Merciful Dictionary Meta Analyzer

A dynamic scheme for global dictionary optimization.  
The name is a joke, don't do drugs.

How it works:

1. Build the SA, LCP array and an additional offsets structure for faster counting
2. Generate all matches
3. Rank matches
4. Choose the best word
5. Split the data at the locations of the word
6. Discard matches with rank < 0
7. Repeat until no more matches are left

## Perf

| File   | Runtime | Max memory usage | Matches initially |
|--------|---------|------------------|-------------------|
| book1  |   2.92s |     20'708 bytes |           735'111 |
| enwik6 |  18.16s |     53'532 bytes |         2'234'683 |
| enwik7 | 336.63s |    511'784 bytes |        22'274'031 |

I've tried to get it to a place where memory usage, speed and accuracy is almost pareto optimal but there's still some trickery to be done.

## Ranking

Unlike other simpler algorithms for optimal dictionaries, ranking is done with the understaning that post-transform we'll apply some sort fo entropy coding scheme.

## Match generation

Generating all possible matches is an O(N) operation done on the LCP and SA arrays.  
A match is represented as a range in the SA - (sa_index, sa_count) tuple and a length.

To reduce memory consumption and improve speed I've reduced the max length to 256 but it can easily be extended to 16 or 32 bits. For shorter text files without long repetitions 256 seems to be a good enough limit.

In the (hopefully near) future I plan on adding tokens for longer matches as these are already being detected in the matchfinder and can just be encoded into the dictionary to be applied by the parser.
There's also room for more complex anaysis, hence the A in MDMA.

## Patterns

This scheme allows for regex-like pattern matching, where patterns may be ranked according to their own ranking function.
The harder problem is generating relevant patterns and transmitting the relevant information when parsing.

But yeah, a lot of opportunities.
One specific idea I had for genetic data was to also match the complementary of oligonicleotides. After parsing, each such pattern is only an extra bit in the stream. And it can have it's own model..