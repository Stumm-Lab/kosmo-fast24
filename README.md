# Kosmo: Efficient Online Miss Ratio Curve Generation for Eviction Policy Evaluation

Welcome to the FAST'24 artifact submission for Kosmo! These instructions are intended to guide you through setting up and running our Kosmo simulations on your local machine. However, we have also provided credentials to a small cloud container which we have set up with the same code to allow for a no-install test of our tool (more information on this can be found in the [Virtual Machine Usage](#virtual-machine-usage) section. Note that this machine is not the same machine which was used in measuring the results presented in the paper and is simply an ease-of-use alternative to using your local machine.

## Abstract

In-memory caches play an important role in reducing the load on backend storage servers for many workloads. Miss ratio curves (MRCs) are an important tool for configuring these caches with respect to cache size and eviction policy. MRCs provide insight into the trade-off between cache size (and thus costs) and miss ratio for a specific eviction policy. Over the years, many MRC-generation algorithms have been developed. However, to date, only Miniature Simulations is capable of efficiently generating MRCs for popular eviction policies, such as *Least Frequently Used* (LFU), *First-In-First-Out* (FIFO), and *Least Recently/Frequently Used* (LRFU), that do not adhere to the inclusion principle. One critical downside of Miniature Simulations is that it incurs significant memory overhead, precluding its use for online cache analysis at runtime in many cases.

In this paper, we introduce Kosmo, an MRC generation algorithm that allows for the simultaneous generation of MRCs for a variety of eviction policies that do not adhere to the inclusion principle. We evaluate Kosmo using 52 publicly-accessible cache access traces with a total of roughly 126 billion accesses. Compared to Miniature Simulations, Kosmo has lower memory overhead by a factor of 5 on average, and as high as 9, and a higher throughput by a factor of 1.2 making it far more suitable for online MRC generation.

## Dependencies

1. This package was compiled and tested using `Rust v1.77.0-nightly`. Please install the latest version of Rust (including the latest version of`cargo`) [here](https://www.rust-lang.org/tools/install).

2. This package uses `gnuplot v5.4` to generate MRC plots. Please install the latest version of gnuplot [here](http://www.gnuplot.info/download.html).

## Description of Tools

This package is made up of three tools:

1. `wss`: This tool calculates the working set size of a given access trace.

2. `accurate`: This tool runs full simulations to compute the accurate MRC for a given access trace.

3. `mrc`: This tool runs Kosmo or MiniSim (or both) to generate an MRC for a given access trace.

### Access Trace

Each of three tools takes a path to an access trace as input. This access tace is stored in binary format where each access is 25 bytes and follows the following storage format (all properties are stored in little endian):

| Property  | Type                           |
| --------- | ------------------------------ |
| Timestamp | u64                            |
| Command   | u8 (0: GET/READ, 1: SET/WRITE) |
| Key       | u64                            |
| Size      | u32                            |
| TTL       | u32 (0 indicates no TTL)       |

Note that only GET/READ accesses are considered when generating an MRC.

### Eviction Policy Arguments

The `accurate` and `mrc` tools take eviction policies as arguments. The supported eviction policies are:

* `lfu`

* `fifo`

* `2q`

* `lrfu`

* `lru`

### wss

The working set size of an access trace must be computed before running any of the other two tools as its output is an input to the other tools.

#### Arguments

| Argument | Description                   | Short Tag | Long Tag |
| -------- | ----------------------------- | --------- | -------- |
| Path     | The path to the access trace. | `-p`      | `--path` |

#### Example Command

```
cargo run -r --bin wss -- -p /path/to/access/trace.bin
```

Once complete, this will compute the working set size (in bytes) of the access trace at the supplied path.

#### Help Command

```
cargo run -r --bin wss -- -h
```

### accurate

The accurate tool allows for the generation of the accurate MRCs against which the MRCs produced by Kosmo and MiniSim can be tested for accuracy.

#### Arguments

| Argument         | Description                                                                                                                                         | Short Tag | Long Tag   |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ---------- |
| Path             | The path to the access trace.                                                                                                                       | `-p`      | `--path`   |
| Working set size | The working set size of the access trace. This should be the value computed by the `wss` tool.                                                      | `-e`      | `--policy` |
| Eviction policy  | The eviction policy. Please refer to the [eviction policy arguments](#eviction-policy-arguments) section for a list of supported eviction policies. | `-w`      | `--wss`    |

#### Example Command

```
cargo run -r --bin accurate -- -p /path/to/access/trace.bin -e lfu -w 1000 -o /path/to/output.csv
```

#### Help Command

```
cargo run -r --bin accurate -- -h
```

### mrc

The mrc tool allows for the generation of an MRC using either Kosmo, MiniSim, or both.

#### Arguments

| Argument                | Description                                                                                                                                                                                                                                                                                                                                                                                                   | Short Tag | Long Tag           |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ------------------ |
| Path                    | The path to the access trace.                                                                                                                                                                                                                                                                                                                                                                                 | `-p`      | `--path`           |
| Working set size        | The working set size of the access trace. This should be the value computed by the `wss` tool.                                                                                                                                                                                                                                                                                                                | `-w`      | `--wss`            |
| SHARDS threshold        | This *optional* argument is the sampling threshold $T$ of SHARDS. The sampling ratio is $R = T/P$, where $P$ is the modulus (we use a modulus value of $P = 16777216$). For example, if you input a threshold of $1677721$, the sampling ratio would be roughly 0.1. If paired with an `S_max` value (the next argument), the threshold is the initial SHARDS threshold. If omitted, SHARDS will not be used. | `-t`      | `--shards-t`       |
| SHARDS S_max            | This *optional* argument is the `S_max` value of SHARDS. If omitted, SHARDS will run in fixed-rate mode (unless the SHARDS threshold is also omitted).                                                                                                                                                                                                                                                        | `-s`      | `--shards-s`       |
| Kosmo eviction policy   | This *optional* argument defines the eviction policy used by Kosmo. If omitted, Kosmo is not run. Please refer to the [eviction policy arguments](#eviction-policy-arguments) section for a list of supported eviction policies.                                                                                                                                                                              | `-k`      | `--kosmo-policy`   |
| MiniSim eviction policy | This *optional* argument defines the eviction policy used by MiniSim. If omitted, MiniSim is not run. Please refer to the [eviction policy arguments](#eviction-policy-arguments) section for a list of supported eviction policies.                                                                                                                                                                          | `-m`      | `--minisim-policy` |
| Output                  | The output path of the resulting MRC plot. This will be saved as a PDF file.                                                                                                                                                                                                                                                                                                                                  | `-o`      | `--output-path`    |
| Accurate path           | This *optional* argument is the path to the accurate curve. This should be the file saved by the `accurate` command. If omitted, the resulting mean absolute errors (MAEs) of Kosmo or MiniSim will not be reported.                                                                                                                                                                                          | `-a`      | `--accurate-path`  |
| Run type                | Specifies whether running to measure memory or throughput. If measuring memory, the high water mark after the entire access trace has been processed is reported. If measuring throughput, accesses are batched and processed directly from memory (without loading the progress bar during batch processing). Possible values are: `memory` or `throughput`.                                                 | `-r`      | `--run-type`       |

#### Example Command

```
cargo run -r --bin mrc -- -p /path/to/access/trace.bin -w 1000 -t 1677721 -s 2048 -k lfu -o /path/to/output.pdf -a /path/to/accurate.csv -r memory
```

#### Help Command

```
cargo run -r --bin mrc -- -h
```

## Getting Started Instructions

After installing the dependencies and cloning the repository, you may run a simple "hello world" style test using the small sample access trace we have provided. This trace can be found in the `traces` folder and is named `wdev.bin`. It is the `wdev` access trace in the `MSR` dataset. For this example, we will use the LFU eviction policy; however, any eviction policy may be substituted provided it is consistently used throughout.

First, you may find the access trace's working set size by running:

```
cargo run -r --bin wss -- -p ./traces/wdev.bin
```

This should yield 313344512 as the working set size (in bytes) for the wdev access trace.

Next, you may find the accurate curve of the trace by running the following command:

```
cargo run -r --bin accurate -- -p ./traces/wdev.bin -w 313344512 -e lfu -o ./accurate/wdev-lfu.csv
```

This will yield the accurate MRC in CSV format at the output path. You may check the file to ensure an output has been generated. There should be 100 rows (as 100 accurate points are simulated), where the format of each row is: <cache size>,<miss ratio>.

Next, you may measure the mean absolute error (MAE), throughput, and memory usage of both Kosmo and MiniSim processing the access trace. First, we will measure the MAE and throughput for Kosmo by running the following command (here, we recommend using fixed-size SHARDS with an initial sampling ratio of 0.1 (a threshold of 1677721) and an S_max value of 2048):

```
cargo run -r --bin mrc -- -p ./traces/wdev.bin -w 313344512 -t 1677721 -s 2048 -k lfu -o ./traces/mrc.pdf -a ./accurate/wdev-lfu.csv -r throughput
```

This should report the calculated MAE of Kosmo as well as the throughput (in accesses/second). It will also save the MRC as a PDF file at `traces/mrc.pdf`. Note that the progress may appear to freeze at times. This is normal and due to the progress bar not updating during batch processing.

Next, you may measure the memory usage of Kosmo processing the access trace by running the following command:

```
cargo run -r --bin mrc -- -p ./traces/wdev.bin -w 313344512 -t 1677721 -s 2048 -k lfu -o ./traces/mrc.pdf -a ./accurate/wdev-lfu.csv -r memory
```

Finally, we can repeate the previous two commands for MiniSim:

```
cargo run -r --bin mrc -- -p ./traces/wdev.bin -w 313344512 -t 1677721 -s 2048 -m lfu -o ./traces/mrc.pdf -a ./accurate/wdev-lfu.csv -r throughput
```

```
cargo run -r --bin mrc -- -p ./traces/wdev.bin -w 313344512 -t 1677721 -s 2048 -m lfu -o ./traces/mrc.pdf -a ./accurate/wdev-lfu.csv -r memory
```

## Detailed Instructions

After having completed the [Getting Started Instructions](#getting-started-instructions), you should have our testing suite installed and running on your machine. From here, to reproduce the results presented in our paper, you must simply repeat the instructions for all datasets used, substituting "wdev" for each access trace in the `~/kosmo-fast24/traces` folder. However, most of our datasets are extremely large and processing them (especially running the accurate tool) can each take several hours, or in many cases, several days. As such, we have provided a few larger access traces from our full dataset in a virtual machine (in the `~/kosmo-fast24/traces`) from which you may download and test larger access traces, or test directly on the virtual machine.

We have uploaded the following access traces:

* SEC: 2003.bin

* SEC: 2007.bin

* SEC: 2009.bin

* Twitter: cluster7.bin

* Twitter: cluster53.bin

To save time, we have also uploaded the accurate traces for various eviction policies in the `~/kosmo-fast24/accurate/` folder. You may use these instead of running the `accurate` tool and, if you choose, use the tool to verify their results.

### Virtual Machine Usage

In our experimentation, we used a local machine configured with an AMD Threadripper 3990x, 256 GB of 3200 MHz DRAM, and an 8TB M.2 Sabrent SSD. Although we are unable to make this machine publicly accessible, to improve the ease of testing for the evaluators, we created a virtual machine with the code in this repository pre-installed.

Our tests consisted of roughly 6 months of compute time on our machine with roughly 4 TiB of data to process. We understand it is not feasible for the reviewers to test all this data in a short period of time. We have therefore pre-loaded the virtual machine with 5 small to medium-sized access traces.

Unfortunately, our virtual machine has significantly fewer resources than that of the machine used to test the algorithms in the paper. Particularly, the virtual machine contains only 2 cores whereas our local machine has 64. Therefore, the throughput results obtained from the virtual machine will not match those we obtained.

Per the co-chairs's instructions, reviewers of our artifact can find the credentials for this virtual machine in our Artifact Evaluation Submission, or by sending us a message on HotCRP.

## Claims

In our evaluation we make the following claims which can be verified by this artifact:

* Kosmo has lower memory overhead than MiniSim by a factor of 5 on average, up to a factor of 9.

* Kosmo has a higher throughput than MiniSim by a factor of 1.2 on average (note: as mentioned in the [Virtual Machine Usage](#virtual-machine-usage) section, the virtual machine we have set up has significantly fewer resources than that of the machine used to obtain the results in the paper. The throughput results may be different than those presented.)

* Kosmo has a roughly equivalent MAE to MiniSim.

Please let us know on HotCRP if you have any questions regarding the usage or results of the tools.
