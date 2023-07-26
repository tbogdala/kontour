import os, json, re, sys

# This is an example python script that shows how you can process the generated text from Kontour.
# In this example, we've used config-kent.toml to generate a series of random conversations and now
# we'd want to pull those out of the initial raw generation and clean them up for a dataset. Not all
# of this will be automated, but the script will print out file names with errors for the user
# to follow up on. Running the command where the preproc directores exist will cause the script to
# check the 'fixme' folders to see if the data passes the self-tests and if so, moves them to the 
# 'passed' dir.
#
# The example script is written to be somewhat flexible, but currently the notion of a 'user' tag
# and the 'bot' tag are hardcoded at the top of the script and should be edited as needed.
#
# USAGE: python <script> <data_root directory>
#
# example: python 01_basic_format_pass.py generated-kent-smalltalk/20230722142436/


# setup the string matching constants needed
user_re_search_pattern = r"(?i)user:"
user_tag = "USER:"
bot_tag = "KENT:"


# takes a conversation string and returns who is talking
# 0 = not_found, 1 = user, 2 = bot
def get_who_is_talking(conv_string):
    who_is_talking = 0  
    if conv_string.casefold().startswith(user_tag.casefold()):
        who_is_talking = 1
    elif conv_string.casefold().startswith(bot_tag.casefold()):
        who_is_talking = 2
    return who_is_talking

# process the raw files and returns a tuple of (raw json object, list of sentences that were generated in a conversation).
# this is not guaranteed to be in order. the only thing done is beginning garbage is stripped
# and empty lines are dropped.
def proc_raw_file(raw_file, output_dir):
    # open each file
    with open(raw_file) as raw_f:
        # deserialize the json
        raw_json = json.load(raw_f)
        raw_output = raw_json["generated_text_output"]
        
        # filter out anything before the first USER: prompt
        user_pattern = re.compile(user_re_search_pattern, re.IGNORECASE)
        index = user_pattern.search(raw_output).start()
        if index == -1:
            print("ERROR in " + raw_file + ": no 'user:' string found to trim to.")
            return (raw_json, [])
        first_trim_pass = raw_output[index:]
        
        # split the string up based on newlines
        split_pattern = r"\n|\r\n"
        split_raw_string = re.split(split_pattern, first_trim_pass)
        filtered_conv_fragments = []
        for conv_fragment in split_raw_string:
            who_is_talking = get_who_is_talking(conv_fragment)
            if who_is_talking > 0:
                filtered_conv_fragments.append(conv_fragment)
            else:
                stripped_fragment = conv_fragment.strip()
                # if we don't know who's talking, but it's not an empty line, then
                # we append it to the last conversation fragment.
                if len(stripped_fragment) > 0:
                    prev_fragment = filtered_conv_fragments.pop()
                    new_combined_fragment = prev_fragment + "\n" + stripped_fragment
                    filtered_conv_fragments.append(new_combined_fragment)
                    
        return (raw_json, filtered_conv_fragments)

# goes through the filtered conversation and prints out any formatting errors so that 
# they can be manually corrected. Returns bool indicating if there were errors
def test_filtered_conv(filename, filtered_conv_strings):
    if len(filtered_conv_strings) < 1:
        print("ERROR in ", filename, ": empty filtered conversations!")
        return true
        
    errors_present = False

    # first test: user must be the first one talking
    who_is_talking = get_who_is_talking(filtered_conv_strings[0])
    if who_is_talking != 1: #not user?
        print("ERROR in ", filename, ": USER isn't first one talking!")
        errors_present = True

    # second test: bot must be last one talking
    who_is_talking = get_who_is_talking(filtered_conv_strings[ len(filtered_conv_strings) -1 ])
    if who_is_talking != 2: #not user?
        print("ERROR in ", filename, ": BOT isn't last one talking!")
        errors_present = True

    # third test: user and bot should alternate
    last_talker = 2
    for c in filtered_conv_strings:
        who_is_talking = get_who_is_talking(c)
        if who_is_talking == last_talker: 
            print("ERROR in ", filename, ": There's a coversation order problem where talkers don't alternate appropriately.")
            errors_present = True
            break
        else:
            last_talker = who_is_talking
    
    # fourth test: warning: "As an AI language model" check
    bad_ai_phrase1 = "As an AI language model"
    for c in filtered_conv_strings:
        if bad_ai_phrase1 in c:
            print("WARNING in ", filename, ": There's use of '", bad_ai_phrase1, "' in the conversation. Needs review.")
            
    return errors_present


if __name__ == "__main__":
    if len(sys.argv) > 1:
        data_root_dir = sys.argv[1]
        print("Using data_root: ", data_root_dir)
    else:
        print("python ", sys.argv[0], "<data_root directory>")

    # build the directory paths to use
    generate_preproc = False
    raw_dir = os.path.join(data_root_dir,"raw")
    preproc_dir = os.path.join(data_root_dir, "preproc-passed")
    preproc_errors_dir = os.path.join(data_root_dir, "preproc-fixme")

    # if this directory doesn't exist, then we'll assume default behavior is to generate new cleaned data
    if os.path.exists(preproc_dir) == False:
        os.makedirs(preproc_dir)
        generate_preproc = True
        print("Created ", preproc_dir, " folder; generation of preprocessed data enabled.")
    else:
        print("Detected ", preproc_dir, " folder; generation of preprocessed data is disabled and will only be checked for validity.")
    
    if os.path.exists(preproc_errors_dir) == False:
        os.makedirs(preproc_errors_dir)
        print("Created ", preproc_errors_dir, " folder.")
    

    # loop over all raw files if we're generating the cleaned data
    number_of_files_with_errors = 0
    if generate_preproc:
        for file in os.listdir(raw_dir):
            filename = os.fsdecode(file)
            if filename.endswith(".json"):
                # we'll set the out_dir here based on whether or not errors were found.
                raw_json_obj, filtered_conv = proc_raw_file(os.path.join(raw_dir, file), preproc_dir)
                errors_present = test_filtered_conv(filename, filtered_conv)
                if errors_present:
                    number_of_files_with_errors += 1
                    out_dir = preproc_errors_dir
                else:
                    out_dir = preproc_dir

                # serialize the json 
                out_json = {
                    "system_message": raw_json_obj["system_message"],
                    "instruction": raw_json_obj["instruction"],
                    "conversation": filtered_conv,
                }
                out_json_str = json.dumps(out_json, indent=4)
                
                # write it out to the correct directory
                with open(os.path.join(out_dir, filename), "w") as out_f:
                    out_f.write(out_json_str)
    else:
        # we're not generating the data, so check the error folder and run those
        # through the test to see if they've been fixed up.
        for file in os.listdir(preproc_errors_dir):
            filename = os.fsdecode(file)
            if filename.endswith(".json"):
                error_filepath = os.path.join(preproc_errors_dir, file)
                with open(error_filepath) as error_f:
                    # deserialize the json
                    error_json = json.load(error_f)
                    error_conversation = error_json["conversation"]
                    
                    errors_present = test_filtered_conv(filename, error_conversation)
                    if errors_present:
                        number_of_files_with_errors += 1
                    else:
                        print("No errors detected for ", filename, "! Moving to passed folder!")
                        passed_filepath = os.path.join(preproc_dir, file)
                        assert os.path.exists(passed_filepath) == False, "File already exists in the passed folder somehow"
                        os.rename(error_filepath, passed_filepath)

        # now that the error directory has been processed, go over all the passed files
        # again to see if any of them fail.
        for file in os.listdir(preproc_dir):
            filename = os.fsdecode(file)
            if filename.endswith(".json"):
                passed_filepath = os.path.join(preproc_dir, file)
                with open(passed_filepath) as passed_f:
                    # deserialize the json
                    passed_json = json.load(passed_f)
                    passed_conversation = passed_json["conversation"]
                    
                    errors_present = test_filtered_conv(filename, passed_conversation)
                    if errors_present:
                        print("New errors detected for ", filename, "! Moving to fixme folder!")
                        error_filepath = os.path.join(preproc_errors_dir, file)
                        assert os.path.exists(error_filepath) == False, "File already exists in the fixme folder somehow"
                        os.rename(passed_filepath, error_filepath)
                        number_of_files_with_errors += 1
                    
    print("Files with errors: ", number_of_files_with_errors)
    