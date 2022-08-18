![Crates.io](https://img.shields.io/crates/v/diff_priv)
# DiffPriv
DiffPriv is a differential privacy framework for real time data streaming written in Rust. Supporting k-anonymity,
(c,l)-diversity and ε-differential privacy. The framework is based on the [Preserving Differential Privacy and Utility of Non-stationary Data Streams](https://ieeexplore.ieee.org/document/8637412) paper, with various improvements implemented.

This project is the result of my master thesis: Differential privacy in large scale data streaming.
It has been developer during an internship at [STRM Privacy](https://strmprivacy.io/)

## How to use
it's recommended to first build the application using as it will significantly speed up the algorithm
> cargo build --release

An application.conf needs to be present in the root folder.
This will build a binary that can be run with the following command

> RUST_LOG="debug" ./target/release/diff-priv

This will use a dataset from the `datasets` folder, the supported datasets can be seen in `test/tests.rs`
`RUST_LOG` part can be removed to the users liking. This removes debugging logging when the algorithm will run.

## Where is the data exported
When `main.rs` is run, the processed datasets can be seen in the `exports` directory.

## Application parameters
Inside the `application.conf` all the different privacy parameters can be edited to the users liking.
At this moment for `buffer_size` we use `3*k` and for `k_max` we use `4*k`. This can be edited in the `environment.rs` and `tests.rs` file.
Additional parameters can be easily added through the `config.rs` file by adding it as a struct attribute and then adding it to `application.conf`.


| Parameter | Description                                                           |
|-----------|-----------------------------------------------------------------------|
| k         | The minimum k-anonymity level                                         |
| k_max     | The maximum k-anonymity level a cluster can have before being deleted |
| l         | The minimum l-diversity level                                         |
| (c,l)     | The minimum (c,l)-diversity level                                     |
| eps       | ε-differential privacy level                                          |
| diff_thrs | The maximum distance the data tuple and the cluster centroid can have |
| delta     | The maximum time in seconds a cluster can exist without the addition of new data tuples                        |
| buff_size | The maximum amount of tuples the buffers W_curr and W_prev can contain                                         |
| noise_thr | categorical noise level                                         |

# Documentation
{{readme}}


# Architecture
The architecture of the DiffPriv framework can be seen below
![Alt text](midipsa_1.png?raw=true "Title")

# Thesis related stuff in the repo
In my thesis is described tests using `knn-test.sh`. To run this you need Java 8.

License:

MIT License

Copyright (c) 2022 Maciek Mika

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.