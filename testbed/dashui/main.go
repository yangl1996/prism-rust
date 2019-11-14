package main

import (
	"fmt"
	"os"
)

func main() {
	if len(os.Args) < 2 {
		usage()
	}

	switch os.Args[1] {
	case "dashboard":
		dashboard(os.Args[2:])
	default:
		usage()
	}
}

func usage() {
	fmt.Println("Subcommands: dashboard")
	os.Exit(1)
}
