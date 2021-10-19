label .ENTRY
    print "Enter the name of the directory: "
    input
    var dir_name TEMP
    var command_to_execute "mkdir "
    add command_to_execute dir_name
    var command_to_execute TEMP
    cmd command_to_execute
    print "Successfully created directory `"
    print dir_name
    print "`"