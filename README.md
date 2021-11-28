# Krustens

Get statistics from your Spotify history.

## Usage
**Required:** At least one file downloaded from Spotify of your listening history. It can be downloaded from the `Account` -> `Privacy` page after requesting an export of your data. In the resulting export it will be a file named similar to `StreamingHistory0.json`. This program can use any number of them.

### Generating Listen Events
`Krustens` reads the streaming history files and generates events it can read back later to generate the statistics. It does this to attempt to ensure that it does not double-count any duplicate track plays between the spotify history files. `Krustens` right now also counts any song played for less than a minute as "skipped".

To start clone the repo and then fill in the config file `resources/config.json`, or ensure that your data locations match.
```javascript
{
    "history_folder": "./data/spotify_play_history", // The location that `krustens` will read the history files from. It will read all JSON files in this directory
    "output_folder": "./output", // Where the output statistics should go. It will place a `stats` folder in this location.
    "count_general_stats_to_compile": 25 // For the general stats, this is the `Top` number to grab (e.g. Top 25 artists)
}
```
When `krustens` starts it will read its configuration from this file. Once this is done run
```bash
cargo run process_listens
```
to generate the events. An `app_data` folder will be generated alongside the application which will contain a file of the events and a snapshot of what has been seen.

Once the events have been generated, you can generate statistics by just running the program with no extra parameter.
```bash
cargo run
```
The `stats` folder will be generated and it will then contains a number of stats files. Right now all statistics generated are hard-coded, but more options will likely be added in the future.
