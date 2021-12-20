# A Parallel Mandelbrot Set Plotter

This program plots the Mandelbrot set and writes it out as a PNG file. It uses Rust's concurrency primitives to distribute the work across eight threads.

The book shows several different versions of this program: single- and multi-threaded versions in the "Tour of Rust" chapter, and a final version based on the Rayon crate in the "Concurrency" chapter, which makes more effective use of parallelism.

Each of these versions appears on a different branch in this repository:

*   [Branch `single-threaded`](https://github.com/ProgrammingRust/mandelbrot/tree/single-threaded)
    is the base version of the program. It does all the work on the main
    thread. This is the first version shown in the "Tour of Rust" chapter.

*   [Branch `bands`](https://github.com/ProgrammingRust/mandelbrot/tree/bands)
    splits the plotting area up into eight bands, and assigns one thread to
    each. This often makes poor use of the threads, because some bands take
    significantly longer than others to complete: once a fast thread completes
    its band, its CPU goes idle while its less fortunate brethren are still hard
    at work. This is the final version shown in the Tour.

*   [Branch `task-queue`](https://github.com/ProgrammingRust/mandelbrot/tree/task-queue)
    gets an almost perfect linear speedup from its threads. It splits
    the plotting area up into many more bands, and then has threads draw
    bands from a common pool until the pool is empty. When a thread
    finishes one band, it goes back for more work. Since the bands still
    take different amounts of time to render, the problem cited above
    still occurs, but on a much smaller scale. This version is not shown in the book.

*   [Branch `lockfree`](https://github.com/ProgrammingRust/mandelbrot/tree/lockfree)
    uses Rust's atomic types to implement a lock-free iterator type, and
    uses that to dole out bands from the pool instead of a
    mutex-protected count. On Linux, this is no faster than the
    mutex-based version, which isn't too surprising: on Linux, locking
    and unlocking an uncontended mutex *is* simply a pair of atomic
    operations. This version is also not shown in the book.

*   [Branch `rayon`](https://github.com/ProgrammingRust/mandelbrot/tree/rayon)
    uses the Rayon library instead of Crossbeam. Rayon provides a *parallel
    iterator* API that makes our code much simpler. It looks a lot like Rust
    code that uses plain old iterators. This is the final version shown in the
    Concurrency chapter.

## License

The example code in this directory and its subdirectories is licensed under the
terms of the MIT license. See [LICENSE-MIT](LICENSE-MIT) for details.
