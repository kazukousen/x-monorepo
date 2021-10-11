package list_data_lib

import (
	"io/fs"
	"path/filepath"
	"strings"
)

func ListData() ([]string, error) {
	var files []string
	err := filepath.Walk(".", func(path string, info fs.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if !info.IsDir() && strings.HasSuffix(path, ".txt") {
			files = append(files, path)
		}
		return nil
	})
	if err != nil {
		return nil, err
	}

	return files, nil
}
