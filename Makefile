.PHONY: dep
dep:
	go mod tidy
	go mod vendor
	bazel run //:gazelle -- update-repos --from_file=go.mod -to_macro=3rdparty/go_repositories.bzl%go_repositories -prune

.PHONY: gazelle
gazelle:
	bazel run //:gazelle
