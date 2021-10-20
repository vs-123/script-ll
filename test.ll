label .ENTRY
    print "Give me a filename and I will print its content: "

    input
    var filename TEMP

    jmp_eq filename "" .ENTRY

    read_file filename
    add TEMP "\n"
    print TEMP

    jmp .ENTRY