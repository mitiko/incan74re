# Incan74re

A global dynamic dictionary optimizer.  
Unlike CM coders, this compression utility can see future data and make better global decisions - it casts not spells but weather forecasts :wink:.

How it works:

1. Build the SA, LCP array and an additional offsets structure for faster word counting
2. Generate all matches
3. Rank matches
4. Choose the best word at each iteration
5. Split the data at the locations of the word
6. Discard matches with rank < 0
7. Repeat until no more matches are left

Copyright (c) 2021 Dimitar Rusev <mitikodev@gmail.com>

## License

The incan74re (*incantare*) project is released under the GPL-3.0 License  
A build requirement is the libsais library by Ilya Grebnov licensed under Apache License 2.0

## Notes

This is a port from my [BWDPerf project](https://github.com/Mitiko/BWDPerf).

The project started as an lzw python competitor with genetic data in mind.  
Then I rewrote it in C# (BWD) for better speed and clarity (also I had made some logic mistakes the first time).  
Next I introduced a better data structure for a match finder. This went through multiple changes and optimizations.  
Finally I found out about the Suffix Array and the FM-index, for a final C# rewrite.  
Turns out I was starting to get throttled by the GC and the project was growing into more of a modular compressor rather than a singular dictonary transform, also a more functional laguage would've benefited my use case, so I took up rust (quite quickly and with little effort actually) and rewrote it in rust for a ~90x speed improvement.

Later, I tried adding MT code but the bottleneck still is memory latency and log2 computations, that I have to work on first.
