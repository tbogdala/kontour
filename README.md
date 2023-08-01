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
3) Run `kontour`. It should automatically use the `config.toml` next to the binary.

Unless changed in the `config.toml`, you should see a `generated` folder get created 
next to the program. There should be another folder named with digits in that
which, if you look closely, should match the local time the program was running. Inside
that folder, you should see the `summary.md` report that was generated and a directory
named `raw` that will have all of the JSON data recorded during the generation process.


## Sample Survey Output

The screenshot below is from the report in the `samples` directory, which an be 
[viewed right here](/samples/summary.md).

![Screenshot of markdown summary report](/samples/Screenshot_summary_md.jpg?raw=true "Markdown summary report sample")


## Advanced Usage: Dataset Generation

Instruction groups can be set up to replace a tag in an instruction (e.g. "{TONE}") with a random
choice from the substitutes list. For example, in this given instruction, "{TONE}" will get substituted
with a random string from the `substitutes` array of that `instruction_groups` entry, like "Neutral" maybe:

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

With `instruction_groups` defined and the name of the instruction group used in an instruction,
Kontour will choose a random substitute. This can be used to create multiple variations of a prompt.

### Controlling the Amount of Data to Generate

Wth the `-r <number>` command line parameter, Kontour can generate a set number of generations,
which means that if you have instruction groups set up, you can use this to create a dataset.

In this way, you can look at the `config-kent.toml` file as an example of how to generate an conversation set
based on generic instructions that can be randomly assembled. When creating Kent's dataset
to train on, I would comment out all topics but one so that, if I needed to, I could
limit the amount of conversations per topic and just change the `output_folder` variable.

An example terminal command to generate 50 conversations for Kent would look like this:

```bash
kontour -f config-kent.toml --unsorted-report -r 50
```

Why `unsorted-report`? Because normally, the report is sorted to group together instructions in a way that makes it
easier to compare generations across different parameter sets and models. That's unnecessary when generating
datasets and can even be slightly confusing, so I just disable it.

Further processing of this dataset used the scripts in the `data-cleaning` folder. These take the
raw json jobs from kontour, clean them up a bit and put them into a more flexible conversation
format. From there, they can be assmebled into whatever dataset format is requied by the training software.
Currently the example provided uses the qlora project and the 'oasst1' format. 

### Example Steps for Dataset Generation

An example of how I'd run kontour to generate a dataset of 500 conversations would look like this:

```bash
kontour -f config-kent.toml --unsorted-report -r 500

# clean up the conversation data from the generated text and run some tests.
# Conversations are then split into `preproc-passed` and `preproc-failed` with
# the log.txt file giving helpful output on what to edit.
cd data-cleaning
python 01_basic_format_pass.py generated-kent/20230730145223/ > log.txt

# after manually editing, run the command again to check all the files once more.
# if they pass the tests, they will be moved to `preproc-passed`.
python 01_basic_format_pass.py generated-kent/20230730145223/ > log.txt

# now assemble them all into one jsonl file for the qlora project... multiple folders 
# can be specified on the end of this command to assemble conversations split across directories
python 02_build_condensed_dataset.py merged-dataset.jsonl generated-kent/20230730145223/preproc-passed/ 


# at this point, I switch into the qlora project's folder (https://github.com/artidoro/qlora),
# which for this example will be in my home directory, just like kontour (again, just for ease of writing here).
# this also assumes qlora has all necessary dependencies setup and can run it's own scripts fine.
cd ~/qlora

# I link in the merged jsonl dataset; if your OS doesn't support this, just copy the file into the qlora folder instead.
ln -s ~/kontour/data-cleaning/merged-dataset.jsonl merged-dataset.jsonl

# finally, fine-tune a 13B qlora based on open_llama_13b (https://huggingface.co/openlm-research/open_llama_13b)
~/kontour/data-cleaning/03_qlora_finetune_13b.sh
```


## Current Bugs and Limitations

* It seems that switching models with the API calls in text-generation-webui can be unreliable
  at times. If you run into problems with this, try creating separate configuration files for
  each model.

* Currently the random instruction group substitutions aren't tracked explicitly. So in the saved
  json for the job, currently there's only the finished instruction and system message ... without
  anything to identify what random instruction_groups substitutions happened. If it seems
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
