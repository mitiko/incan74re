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

Copyright (c) 2021 Dimitar Rusev <mitikodev@gmail.com>

## License

The mdma project is released under the GPL-3.0 License  
A build requirement is the libsais library by Ilya Grebnov licensed under Apache License 2.0

## Note

This is a port from my [BWDPerf project](https://github.com/Mitiko/BWDPerf).