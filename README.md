# script-ll

![script-ll](https://github.com/vs-123/script-ll/blob/main/images/LL-icon.png)


## Introduction:
**script-ll** is a lined-language that was designed for writing scripts.

## Quick Start:
You could go to the examples folder and take a look at the contents of [tutorial.ll](https://github.com/vs-123/script-ll/blob/main/examples/tutorial.ll)

## Commands
For now (since 18 October 2021), there are 16 (sixteen) commands:
 - `exists <VARIABLE_NAME> <TYPE> info: Checks for the existence of a variable called <IDENTIFIER> of type <TYPE>.`
 - `var <VARIABLE_NAME> <VARIABLE_VALUE> info: Creates a variable called <VARIABLE_NAME> with the value <VARIABLE_VALUE>.`
 - `print <VARIABLE_NAME/STRING/NUMBER> info: Prints the given <VARIABLE_NAME/STRING/NUMBER>.`
 - `print_newline info: Prints a newline.`
 - `add <NUMBER_1> <NUMBER_2> info: Adds <NUMBER_1> and <NUMBER_2> and stores it to variable 'TEMP'.`
 - `sub <NUMBER_1> <NUMBER_2> info: Subtracts <NUMBER_2> from <NUMBER_1> and stores it to variable 'TEMP'.`
 - `mul <NUMBER_1> <NUMBER_2> info: Multiplies <NUMBER_1> by <NUMBER_2> and stores it to variable 'TEMP'.`
 - `div <NUMBER_1> <NUMBER_2> info: Divides <NUMBER_1> by <NUMBER_2> and stores it to variable 'TEMP'.`
 - `mod <NUMBER_1> <NUMBER_2> info: Gets the remainder of <NUMBER_1> and <NUMBER_2> and stores it to variable 'TEMP'.`
 - `label <LABEL_NAME> info: Creates a label with the name <LABEL_NAME>.`
 - `jmp <LABEL_NAME> info: Jumps to label <LABEL_NAME>.`
 - `jmp_gt <NUMBER_1> <NUMBER_2> <LABEL_NAME> info: Jumps to label <LABEL_NAME> if <NUMBER_1> is greater than <NUMBER_2>.`
 - `jmp_lt <NUMBER_1> <NUMBER_2> <LABEL_NAME> info: Jumps to label <LABEL_NAME> if <NUMBER_1> is less than <NUMBER_2>.`
 - `jmp_eq <NUMBER_1/STRING_1> <NUMBER_2/STRING_2> <LABEL_NAME> info: Jumps to label <LABEL_NAME> if <NUMBER_1/STRING_1> is equal to <NUMBER_2/STRING_2>.`
 - `return <NUMBER/STRING> info: Stores <NUMBER/STRING> to variable 'TEMP'`
 - `comment <ANYTHING> info: Does not do anything`
