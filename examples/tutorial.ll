comment This is a tutorial of script-ll

comment The .ENTRY label is the first label that gets executed.
label .ENTRY
    comment Declaring variables.
    var age 21
    var name "John"

    comment Jumping to label `can_drive`
    jmp can_drive

    comment `TEMP` is a variable initiated by default for storing temporary values
    comment jmp_eq is a command which jumps to a label if the two values are equal, in this case, TEMP and 1
    jmp_eq TEMP 1 print_can_drive_1
    comment And here, TEMP and 0
    jmp_eq TEMP 0 print_can_drive_0
    comment This command prints a newline, it doesn't take any arguments
    print_newline

comment This label is for printing when the person cannot drive
label print_can_drive_0
    print name
    print " can't drive"

comment This label is for printing when the person can drive
label print_can_drive_1
    print name
    print " can drive!"

label can_drive
    comment `exists` checks if a variable exists of a given type
    comment In this case, we check if there is a variable `age` which has a type `NUMBER`.
    exists age NUMBER
    comment jmp_gt is a command which jumps to a label if the first value is greater than the second value.
    comment In this case, if `age` is greater than 17, then jump to `print_can_drive_1`
    jmp_gt age 17 _can_drive_1
    comment jmp_gt is a command which jumps to a label if the first value is less than the second value.
    comment In this case, if `age` is less than 17, then jump to `print_can_drive_0`
    jmp_lt age 17 _can_drive_0

label _can_drive_1
    comment Return 1 if the person can drive
    return 1

label _can_drive_0
    comment Return 0 if the person cannot drive
    return 0