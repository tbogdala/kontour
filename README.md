# Kontour

A utility program for [text-generation-webui](https://github.com/oobabooga/text-generation-webui)
that generates text responses for a configured list of instructions for each
configured model and text generation parameter set.


## Requirements

* A working [text-generation-webui](https://github.com/oobabooga/text-generation-webui)
  install with models downloaded that you want to test. Make sure to have started
  the text-generation-webui server with the `--api` flag.
* To build from source you will need a rust toolchain installed.


## Building from source

1) Clone the repo.
2) Compile with `cargo build --release`
3) Run with `cargo run --release`


## Usage

1) Start text-generation-webui with the API enabled.
2) Setup the `config.toml` file in this directory with the models, instructions
and generation parameters you'd like. The file is commented with instructions
on what the settings do. Make sure that the `api_url` in the `config.toml` config 
file points to that server instance.
3) Run the program.

Unless changed in the `config.toml`, you should see a `generated` folder get created 
next to the program. There should be another folder named with digits in that
which, if you look closely, should match the local time the program was running. Inside
that folder, you should see the `summary.md` report that was generated and a directory
named `raw` that will have all of the JSON data recorded during the generation process.


## License

The code for this program is released under the GPL v3.

KONTOUR  Copyright (C) 2023  Timothy Bogdala

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
