package main

import (
	"flag"
	"fmt"
	"os"
	"strings"
	"io/ioutil"
	"net/http"
	"time"
	"math/rand"
	"regexp"
	"strconv"

	tm "github.com/buger/goterm"
	"github.com/algorand/go-algorand-sdk/client/algod"
	"github.com/algorand/go-algorand-sdk/client/kmd"
	"github.com/algorand/go-algorand-sdk/transaction"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Subcommands: gentx perf block")
		os.Exit(1)
	}

	switch os.Args[1] {
	case "gentx":
		gentx(os.Args[2:])
	case "perf":
		perf(os.Args[2:])
	case "block":
		block(os.Args[2:])
	default:
		fmt.Println("Subcommands: gentx perf block")
		os.Exit(1)
	}
}

func block(args []string) {
	blockCommand := flag.NewFlagSet("block", flag.ExitOnError)
	node := blockCommand.String("node", "", "Sets the name of the node to request data")
	round := blockCommand.Uint64("round", 0, "Sets the round to request")

	blockCommand.Parse(args)

	if *node == "" {
		fmt.Println("Missing option 'node'")
		os.Exit(1)
	}

	// get algod API address and token
	algodAddrBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/algod.net")
	algodAddr := strings.TrimSpace(string(algodAddrBytes))
	algodAddr = "http://" + algodAddr
	if err != nil {
		fmt.Printf("Failed to read algod listening address: %v\n", err)
		os.Exit(1)
	}
	algodTokenBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/algod.token")
	algodToken := strings.TrimSpace(string(algodTokenBytes))
	if err != nil {
		fmt.Printf("Failed to read algod token: %v\n", err)
		os.Exit(1)
	}

	// get the block we are requesting
	algodClient, err := algod.MakeClient(algodAddr, algodToken)
	if err != nil {
		fmt.Printf("Failed to initialize algod client: %v\n", err)
		os.Exit(1)
	}
	block, err := algodClient.Block(*round)
	if err != nil {
		fmt.Printf("Failed to request block data: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("Block of round %v\n", block.Round)
	fmt.Printf("Transactions:  %v\n", len(block.Transactions.Transactions))
	fmt.Printf("Timestamp:     %v\n", block.Timestamp)
}

func perf(args []string) {
	perfCommand := flag.NewFlagSet("perf", flag.ExitOnError)
	node := perfCommand.String("node", "", "Sets the name of the node to measure performance")
	interval := perfCommand.Uint("interval", 500, "Sets the interval (in ms) between queries")

	perfCommand.Parse(args)

	if *node == "" {
		fmt.Println("Missing option 'node'")
		os.Exit(1)
	}

	// get algod API address and token
	algodAddrBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/algod.net")
	algodAddr := strings.TrimSpace(string(algodAddrBytes))
	algodAddr = "http://" + algodAddr
	if err != nil {
		fmt.Printf("Failed to read algod listening address: %v\n", err)
		os.Exit(1)
	}
	algodTokenBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/algod.token")
	algodToken := strings.TrimSpace(string(algodTokenBytes))
	if err != nil {
		fmt.Printf("Failed to read algod token: %v\n", err)
		os.Exit(1)
	}

	// compile regular expressions
	totalTxRe := regexp.MustCompile(`algod_ledger_transactions_total\{\} (\d+)`)
	roundRe := regexp.MustCompile(`algod_ledger_round\{\} (\d+)`)
	txPoolRe := regexp.MustCompile(`algod_tx_pool_count\{\} (\d+)`)

	// start querying metrics
	client := &http.Client{}
	ticker := time.NewTicker(time.Duration(*interval * 1000) * time.Microsecond)
	start := time.Now()
	for now := range ticker.C {
		req, err := http.NewRequest("GET", algodAddr + "/metrics", nil)
		if err != nil {
			fmt.Printf("Failed to create HTTP request: %v\n", err)
		}
		req.Header.Add("X-Algo-API-Token", algodToken)
		resp, err := client.Do(req)
		if err != nil {
			fmt.Printf("Failed to request metrics: %v\n", err)
		}
		body, err := ioutil.ReadAll(resp.Body)
		if err != nil {
			fmt.Printf("Failed to read algod response: %v\n", err)
		}
		resp.Body.Close()
		d := string(body)

		totalTxMatch := totalTxRe.FindStringSubmatch(d)
		if totalTxMatch == nil {
			continue
		}
		totalTx, _ := strconv.Atoi(totalTxMatch[1])
		roundMatch := roundRe.FindStringSubmatch(d)
		if roundMatch == nil {
			continue
		}
		round, _ := strconv.Atoi(roundMatch[1])
		txPoolMatch := txPoolRe.FindStringSubmatch(d)
		if txPoolMatch == nil {
			continue
		}
		txPool, _ := strconv.Atoi(txPoolMatch[1])

		tm.Clear()
		tm.MoveCursor(1, 1)
		duration := int(now.Sub(start).Seconds())
		tm.Printf("Time since start - %v sec\n", duration)
		tm.Printf("Total Tx         - %v\n", totalTx)
		tm.Printf("Round            - %v\n", round)
		tm.Printf("Memory Pool      - %v\n", txPool)
		tm.Flush()
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
		fmt.Printf("Failed to read kmd listening address: %v\n", err)
		os.Exit(1)
	}
	kmdTokenBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/kmd-v0.5/kmd.token")
	kmdToken := strings.TrimSpace(string(kmdTokenBytes))
	if err != nil {
		fmt.Printf("Failed to read kmd token: %v\n", err)
		os.Exit(1)
	}
	algodAddrBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/algod.net")
	algodAddr := strings.TrimSpace(string(algodAddrBytes))
	algodAddr = "http://" + algodAddr
	if err != nil {
		fmt.Printf("Failed to read algod listening address: %v\n", err)
		os.Exit(1)
	}
	algodTokenBytes, err := ioutil.ReadFile("/tmp/prism/" + *node + "/algod.token")
	algodToken := strings.TrimSpace(string(algodTokenBytes))
	if err != nil {
		fmt.Printf("Failed to read algod token: %v\n", err)
		os.Exit(1)
	}

	// get kmd and algod clients
	kmdClient, err := kmd.MakeClient(kmdAddr, kmdToken)
	if err != nil {
		fmt.Printf("Failed to initialize kmd client: %v\n", err)
		os.Exit(1)
	}
	algodClient, err := algod.MakeClient(algodAddr, algodToken)
	if err != nil {
		fmt.Printf("Failed to initialize algod client: %v\n", err)
		os.Exit(1)
	}


	// get the wallet
	wallets, err := kmdClient.ListWallets()
	walletID := ""
	if err != nil {
		fmt.Printf("Failed to list wallets: %v\n", err)
		os.Exit(1)
	}
	for _, wallet := range wallets.Wallets {
		if wallet.Name == "unencrypted-default-wallet" {
			walletID = wallet.ID
		}
	}
	if walletID == "" {
		fmt.Printf("Unable to find the test wallet\n")
		os.Exit(1)
	}
	walletHandleResp, err := kmdClient.InitWalletHandle(walletID, "")
	if err != nil {
		fmt.Printf("Failed to initialize wallet handle: %v\n", err)
		os.Exit(1)
	}
	wallet := walletHandleResp.WalletHandleToken

	// renew the handle every 10 seconds. why on earth does the handle auto-expire
	go func() {
		ticker := time.NewTicker(time.Duration(10) * time.Second)
		for range ticker.C {
			_, err := kmdClient.RenewWalletHandle(wallet)
			if err != nil {
				fmt.Printf("Failed to renew wallet handle: %v\n", err)
			}
		}
	}()

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

	// increase the number of keepalive connections
	// algorand's benchmark does this, so we might as well do it here
	http.DefaultTransport.(*http.Transport).MaxIdleConnsPerHost = 100

	// get suggested transaction parameters
	txParams, err := algodClient.SuggestedParams()
	if err != nil {
		fmt.Printf("Failed to retrieve suggested transaction parameters: %v\n", err)
		os.Exit(1)
	}

	interval := 1000000 / *rate
	ticker := time.NewTicker(time.Duration(interval) * time.Microsecond)
	fmt.Printf("Start sending payments from %v to itself at rate %v\n", addr, *rate)
	for range ticker.C {
		go func() {
			randNote := make([]byte, 32)
			rand.Read(randNote)
			tx, err := transaction.MakePaymentTxn(addr, addr, 2000, 100000, txParams.LastRound, txParams.LastRound + 1000, randNote, "", txParams.GenesisID, txParams.GenesisHash)
			if err != nil {
				fmt.Printf("Error generating transaction: %v\n", err)
				return
			}
			signed, err := kmdClient.SignTransaction(wallet, "", tx)
			if err != nil {
				fmt.Printf("Error signing transaction: %v\n", err)
				return
			}
			sendResp, err := algodClient.SendRawTransaction(signed.SignedTransaction)
			if err != nil {
				fmt.Printf("Error sending transaction: %v\n", err)
				return
			}
			fmt.Printf("Queued transaction %v\n", sendResp.TxID)
		}()
	}
}
