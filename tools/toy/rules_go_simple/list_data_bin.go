package main

import (
	"fmt"
	"os"

	"github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/list_data_lib"
)

func main() {
	files, err := list_data_lib.ListData()
	if err != nil {
		_, _ = fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	for _, f := range files {
		fmt.Println(f)
	}
}
