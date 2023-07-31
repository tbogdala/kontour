import os, json, sys

# This script takes a series of directories that should contain the cleaned up conversation
# JSON files. Each of these JSON files will have their conversation condensed into one text
# string which will then form the row in the merged JSONL file. This is the data format
# expected for the qloara project for the 'oasst1' format.
#
# NOTE: The system message is still hardcoded in here now and will need to be adjusted manually.
#
# USAGE: python <script> <output merged jsonl> <filtered json directory>+
#
# example: python 02_build_condensed_dataset.py kent-merged-dataset.jsonl \
#   generated-kent-arthistory/20230720235010/preproc-passed/ \
#   generated-kent-philosophy/20230721063108/preproc-passed/ \
#   generated-kent-smalltalk/20230722142436/preproc-passed/

# setup the string matching constants needed
user_tag = "### Human:"
bot_tag = "### Assistant:"

# If a system message is desired before each conversation it can be placed here in this variable.
system_message = ""

###################################################################

def process_data_directory(output_f, dir_path):
    # run through all the json files in the directory
    for file in os.listdir(dir_path):
        filename = os.fsdecode(file)
        if filename.endswith(".json"):
            # deserialize the json
            with open(os.path.join(dir_path, file)) as json_f:
                json_object = json.load(json_f)
                conversation = json_object["conversation"]
                
                # time to build the jsonl fragment string
                jsonl = system_message

                # loop through the conversation. our checks in the previous script should
                # ensure that the USER starts first and BOT finishes, so we can naively
                # iterate over the array.
                turn_tracker = True   # true == user ; false == bot
                for turn in conversation:
                    colon_index = turn.find(":")
                    if turn_tracker == True:
                        jsonl += user_tag + turn[colon_index+1:] + "</s> "
                        turn_tracker = False
                    else:
                        jsonl += bot_tag + turn[colon_index+1:] + "</s> "
                        turn_tracker = True
                
                assert turn_tracker == True, "The bot should be the last one to speak so turn tracker should sit on user."
                
                # write the generated string out to the output file
                json_obj = {
                    "text": jsonl
                }
                output_f.write(json.dumps(json_obj))
                output_f.write("\n")
           


if __name__ == "__main__":
    outfile_name = ""
    data_directories = []
    if len(sys.argv) > 2:
        outfile_name = sys.argv[1]
        for parameter in sys.argv[2:]:
            data_directories.append(parameter)
        print("Using these directories: ", data_directories)
    else:
        print("python ", sys.argv[0], "<output file> <processed json directory>...")

    # the goal will be to build a jsonl out of all the json files in these directories.
    # qlora expects jsonl items to look like this:
    #   {"text": "### Human: Question?###Assistant: Answer."}

    # open up that output file and lets get to work
    with open(outfile_name, "w") as output_f:
        for data_dir in data_directories:
            print("Processing directory: ", data_dir)
            process_data_directory(output_f, data_dir)

