.PHONY: dep
dep:
	go mod tidy
	bazel run //:gazelle -- update-repos --from_file=go.mod -to_macro=3rdparty/go_repositories.bzl%go_repositories -prune

.PHONY: gazelle
gazelle:
	bazel run //:gazelle

.PHONY: clean
clean:
	bazel clean --expunge
