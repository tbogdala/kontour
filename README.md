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


## Model and Parameter Surveys

On a basic level, Kontour will look for `config.toml` and just do text inference
on all combinations of instruction, model and parameter set. This allows for a way
to test generation across different options and then compare and contrast the output.

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


## Sample Survey Output

The screenshot below is from the report in the `samples` directory, which an be 
[viewed right here](/samples/summary.md).

![Screenshot of markdown summary report](/samples/Screenshot_summary_md.jpg?raw=true "Markdown summary report sample")


## Dataset Generation

Instruction Groups can be set up to replace a tag in an instruction (e.g. "{TONE}") with a random
choice from the substitutes list.

```toml
# there can be multiple tags inside each instruction, though the same tag gets replaced
# with the same text.
instructions = [
  "The overall emotional tone of the conversation should be {TONE}."
]

# there can be multiple groups
[[instruction_groups]]
name = "{TONE}"
substitutes = ["Calm", "Tense", "Hopeful", "Discouraged", "Anxious", "Upbeat", "Neutral", "Motivational", "Sad", "Depressed", "Happy", "Excited", "Loving"]
```

With instruction groups defined and the name of the instruction group used in an instruction,
Kontour will choose a random substitute. This can be used to create multiple variations of a prompt.

Also with the `-r <number>` command line parameter, Kontour can generate a set number of generations,
which means that if you have instruction groups set up, you can use this to create a dataset.

In this way, you can look at the `config-kent.toml` file as an example of how to generate an conversation set
based on generic instructions that can be randomly assembled. When creating Kent's dataset
to train on, I would comment out all topics but one so that, if I needed to, I could
limit the amount of conversations per topic.

Further processing of this dataset used the scripts in the `data-cleaning` folder. These take the
raw json jobs from kontour, clean them up a bit and put them into a more flexible conversation
format. From there, they can be assmebled into whatever dataset format is requied by the training software.
Currently the example provided uses the qlora project and the 'oasst1' format. 


## Miscellaneous Fetures

It is possible to create multiple different configuration files to generate content for
several projects by using the `-f` argument to specify the configuration to use.



## Current Bugs and Limitations

* It seems that switching models with the API calls in text-generation-webui can be unreliable
  at times. If you run into problems with this, try creating separate configuration files for
  each model.

* Currently the random instruction group substitutions aren't tracked explicitly. If it seems
  like it might be useful, it would be possible to log those in the job json and carry that
  forward into the dataset fragments so that greater flexibility in topic composition could
  be handled in a crontrolled way.


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
