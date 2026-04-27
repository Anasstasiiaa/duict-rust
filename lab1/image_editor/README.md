## Performance comparison

Tested different Tokio runtime configurations:

1 thread:
Time: 157 ms

2 threads:
Time: 158 ms

CPU cores (4 threads):
Time: 150 ms

Conclusion:
Best performance achieved when number of threads equals CPU cores.
Too many threads may reduce performance due to context switching overhead.
