package bar

import (
	"fmt"
	"github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/baz"
)

func Bar() {
	fmt.Println("bar")
	baz.Baz()
}
