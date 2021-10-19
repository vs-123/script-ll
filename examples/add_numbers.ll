label .ENTRY
    print "Enter number 1: "
    input
    var n1 TEMP
    to_number n1
    var n1 TEMP

    print "Enter number 2: "
    input
    var n2 TEMP
    to_number n2
    var n2 TEMP

    add n1 n2
    var sum TEMP
    to_string sum

    print "The sum of "
    print n1
    print " and "
    print n2
    print " is "
    print sum