label .ENTRY
	is_match "Hello World This Is An Unironic Sentence" "[A-Z]"
	print TEMP
	print_newline

	count_matches "Hello World This Is An Unironic Sentence" "[A-Z]"
	print TEMP
	print_newline

	replace_all "Hello World This Is An Unironic Sentence" "[A-Z]" ""
	print TEMP
	print_newline
	
	replace_n "Hello World This Is An Unironic Sentence" "[A-Z]" "" 2
	print TEMP
	print_newline