package foo

import (
	"fmt"
	"github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/internal/bar"
	"github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/internal/baz"
)

func Foo() {
	fmt.Println("foo")
	bar.Bar()
	baz.Baz()
}
