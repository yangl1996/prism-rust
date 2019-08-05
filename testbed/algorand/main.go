package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/algorand/go-algorand-sdk/client/algod"
	"github.com/algorand/go-algorand-sdk/client/kmd"
	"github.com/algorand/go-algorand-sdk/transaction"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Subcommands: gentx")
		os.Exit(1)
	}

	switch os.Args[1] {
	case "gentx":
		gentx(os.Args[2:])
	default:
		fmt.Println("Subcommands: log, plot, check")
		os.Exit(1)
	}
}

func gentx(args []string) {
	gentxCommand := flag.NewFlagSet("gentx", flag.ExitOnError)
	rate := gentxCommand.Uint("rate", 1, "Sets the rate (txn/s) for generating transactions")
	node := gentxCommand.String("node", "", "Sets the name of the node to generate transactions")

	gentxCommand.Parse(args)

	if *node == "" {
		fmt.Println("Missing option 'node'")
		os.Exit(1)
	}

	fmt.Println(*rate)
	fmt.Println(*node)
}
