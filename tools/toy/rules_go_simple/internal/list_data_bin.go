package main

import (
	"fmt"
	"github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/internal/list_data_lib"
	"os"
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
