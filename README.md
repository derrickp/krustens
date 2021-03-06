# Krustens

Get statistics from your Spotify history.

## Usage
**Required:** At least one file downloaded from Spotify of your listening history. It can be downloaded from the `Account` -> `Privacy` page after requesting an export of your data. In the resulting export it will be a file named similar to `StreamingHistory0.json`. This program can use any number of them.

### Generating Listen Events
`Krustens` reads the streaming history files and generates events it can read back later to generate the statistics. It does this to attempt to ensure that it does not double-count any duplicate track plays between the spotify history files. `Krustens` right now also counts any song played for less than 10s as "skipped".

To start clone the repo and run `cargo run -- -h` to see the defaults for the input and outputs. If you'd like to change those values, provide different options to `-o` or `-i`. The default for input is `./data/spotify_play_history` and output is `./output`.

When `krustens` starts it will read its configuration from this file. Once this is done run
```bash
cargo run -- -m process
```
to generate the events. `-m` denotes what mode to run in (`process` or `generate`). A sqlite database (named `krustens.sqlite`) is generated when processing that will contain all of the events and snapshots of the data. Anytime the listens are processed this database will be used, check if the listen has been tracked already.

Once the events have been generated, you can generate statistics by just running the program with no extra parameter. (The default mode is `generate`)
```bash
cargo run
```
The `output` folder will be generated and it will then contain a number of stats files. Right now the only real values that can be changed is the year to generate statistics for (e.g. 2020 or 2021) and how many _Top_ songs or artists to include in the general stats. For a full list of options run `cargo run -- -h`.
