package foo

import (
	"fmt"
	"github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/bar"
	"github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/baz"
)

func Foo() {
	fmt.Println("foo")
	bar.Bar()
	baz.Baz()
}