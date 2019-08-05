package main

import (
	"flag"
	"fmt"
	"os"
	"strings"
	"io/ioutil"
	"net/http"

	"github.com/algorand/go-algorand-sdk/client/algod"
	"github.com/algorand/go-algorand-sdk/client/kmd"
	//"github.com/algorand/go-algorand-sdk/transaction"
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

	// get token and listening address of algod and kmd
	kmdAddrBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/kmd-v0.5/kmd.net")
	kmdAddr := strings.TrimSpace(string(kmdAddrBytes))
	kmdAddr = "http://" + kmdAddr
	if err != nil {
		fmt.Printf("Failed to read kmd listening address: %v", err)
		os.Exit(1)
	}
	kmdTokenBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/kmd-v0.5/kmd.token")
	kmdToken := strings.TrimSpace(string(kmdTokenBytes))
	if err != nil {
		fmt.Printf("Failed to read kmd token: %v", err)
		os.Exit(1)
	}
	algodAddrBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/algod.net")
	algodAddr := strings.TrimSpace(string(algodAddrBytes))
	algodAddr = "http://" + algodAddr
	if err != nil {
		fmt.Printf("Failed to read algod listening address: %v", err)
		os.Exit(1)
	}
	algodTokenBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/algod.token")
	algodToken := strings.TrimSpace(string(algodTokenBytes))
	if err != nil {
		fmt.Printf("Failed to read algod token: %v", err)
		os.Exit(1)
	}

	// get kmd and algod clients
	kmdClient, err := kmd.MakeClient(kmdAddr, kmdToken)
	if err != nil {
		fmt.Printf("Failed to initialize kmd client: %v", err)
		os.Exit(1)
	}
	algodClient, err := algod.MakeClient(algodAddr, algodToken)
	if err != nil {
		fmt.Printf("Failed to initialize algod client: %v", err)
		os.Exit(1)
	}


	// get the wallet
	wallets, err := kmdClient.ListWallets()
	walletID := ""
	if err != nil {
		fmt.Printf("Failed to list wallets: %v", err)
		os.Exit(1)
	}
	for _, wallet := range wallets.Wallets {
		if wallet.Name == "unencrypted-default-wallet" {
			walletID = wallet.ID
		}
	}
	if walletID == "" {
		fmt.Printf("Unable to find the test wallet")
		os.Exit(1)
	}
	walletHandleResp, err := kmdClient.InitWalletHandle(walletID, "")
	if err != nil {
		fmt.Printf("Failed to initialize wallet handle: %v", err)
		os.Exit(1)
	}
	wallet := walletHandleResp.WalletHandleToken

	// get the address
	listKeysResp, err := kmdClient.ListKeys(wallet)
	if err != nil {
		fmt.Printf("Failed to list addresses: %v\n", err)
		os.Exit(1)
	}
	if len(listKeysResp.Addresses) != 1 {
		fmt.Println("Number of addresses is not 1")
		os.Exit(1)
	}
	addr := listKeysResp.Addresses[0]

	fmt.Printf("Start sending payments from %v to itself\n", addr)
	fmt.Println(algodClient)

	// increase the number of keepalive connections
	// algorand's benchmark does this, so we might as well do it here
	http.DefaultTransport.(*http.Transport).MaxIdleConnsPerHost = 100



	fmt.Println(*rate)
}
