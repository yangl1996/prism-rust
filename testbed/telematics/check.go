package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"strings"
	"sync"
)

type WalletBalance struct {
	Balance uint
}

type UTXOSnapshot struct {
	Checksum string
}

type BlockchainSnapshot struct {
	Leaders []string
}

func check(nodesFile string, verbose bool) {
	nodes := make(map[string]string)
	node_list := make([]string, 0)
	file, err := os.Open(nodesFile)
	if err != nil {
		fmt.Println("Error opening node list:", err)
		os.Exit(1)
	}
	defer file.Close()
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		s := strings.Split(scanner.Text(), ",")
		name := s[0]
		ip := s[2]
		port := s[5]
		url := fmt.Sprintf("http://%v:%v", ip, port)
		nodes[name] = url
		node_list = append(node_list, name)
	}
	if err := scanner.Err(); err != nil {
		fmt.Println("Error reading node list:", err)
		os.Exit(1)
	}

	// check leader sequence
	leaders := make(map[string][]string)
	failed := false
	var m0 sync.Mutex
	var wg0 sync.WaitGroup
	for k, v := range nodes {
		url := v + "/blockchain/snapshot"
		node := k
		wg0.Add(1)
		go func(node, url string) {
			defer wg0.Done()
			resp, err := http.Get(url)
			if err != nil {
				m0.Lock()
				failed = true
				m0.Unlock()
				return // the node is not running yet
			}
			defer resp.Body.Close()

			data := BlockchainSnapshot{}
			err = json.NewDecoder(resp.Body).Decode(&data)
			if err != nil {
				m0.Lock()
				failed = true
				m0.Unlock()
				return
			}
			m0.Lock()
			leaders[node] = data.Leaders
			m0.Unlock()
		}(node, url)
	}
	wg0.Wait()
	if !failed {
		min_ledger_tip := ^uint(0)
		max_ledger_tip := uint(0)
		for _, l := range leaders {
			levels := uint(len(l))
			if levels < min_ledger_tip {
				min_ledger_tip = levels
			}
			if levels > max_ledger_tip {
				max_ledger_tip = levels
			}
		}
		if min_ledger_tip == max_ledger_tip {
			fmt.Printf("Ledger depth %v is consistent across nodes\n", min_ledger_tip)
		} else {
			fmt.Printf("Lowest ledger tip: %v, highest ledger tip: %v\n", min_ledger_tip, max_ledger_tip)
			if verbose {
				for idx := range node_list {
					n := node_list[idx]
					fmt.Printf("%10v: %v\n", n, len(leaders[n]))
				}
			}
		}

		// check if the leader blocks are consistent for each level
		for i := uint(0); i < min_ledger_tip; i++ {
			inited := false
			base := ""
			for _, v := range leaders {
				if !inited {
					base = v[i]
					inited = true
				} else {
					if v[i] != base {
						fmt.Printf("Proposer leader differs among nodes at level %v\n", i)

						if verbose {
							for idx := range node_list {
								n := node_list[idx]
								fmt.Printf("%10v: %v\n", n, leaders[n][i])
							}
						}
						return
					}
				}
			}
		}
		fmt.Printf("Proposer leaders are consistent until level %v\n", min_ledger_tip)
	} else {
		fmt.Println("Failed to query some of the nodes")
		return
	}

	// check balance
	balance := make(map[string]uint)
	failed = false
	var m1 sync.Mutex
	var wg1 sync.WaitGroup
	for k, v := range nodes {
		url := v + "/wallet/balance"
		node := k
		wg1.Add(1)
		go func(node, url string) {
			defer wg1.Done()
			resp, err := http.Get(url)
			if err != nil {
				m1.Lock()
				failed = true
				m1.Unlock()
				return // the node is not running yet
			}
			defer resp.Body.Close()

			data := WalletBalance{}
			err = json.NewDecoder(resp.Body).Decode(&data)
			if err != nil {
				m1.Lock()
				failed = true
				m1.Unlock()
				return
			}
			m1.Lock()
			balance[node] = data.Balance
			m1.Unlock()
		}(node, url)
	}
	wg1.Wait()
	if !failed {
		min := ^uint(0)
		max := uint(0)
		for _, v := range balance {
			if v < min {
				min = v
			}
			if v > max {
				max = v
			}
		}
		if min == max {
			fmt.Println("All wallets have the same balance", min)
		} else {
			fmt.Println("Wallets have different balances ranging between", min, "and", max)
			if verbose {
				for idx := range node_list {
					n := node_list[idx]
					fmt.Printf("%10v: %v\n", n, balance[n])
				}
			}
			return
		}
	} else {
		fmt.Println("Failed to query some of the nodes")
		return
	}

	// check utxodb
	utxohash := make(map[string]string)
	failed = false
	var m2 sync.Mutex
	var wg2 sync.WaitGroup
	for k, v := range nodes {
		url := v + "/utxo/snapshot"
		node := k
		wg2.Add(1)
		go func(node, url string) {
			defer wg2.Done()
			resp, err := http.Get(url)
			if err != nil {
				m2.Lock()
				failed = true
				m2.Unlock()
				return // the node is not running yet
			}
			defer resp.Body.Close()

			data := UTXOSnapshot{}
			err = json.NewDecoder(resp.Body).Decode(&data)
			if err != nil {
				m2.Lock()
				failed = true
				m2.Unlock()
				return
			}
			m2.Lock()
			utxohash[node] = data.Checksum
			m2.Unlock()
		}(node, url)
	}
	wg2.Wait()
	if !failed {
		inited := false
		base := ""
		for _, v := range utxohash {
			if !inited {
				base = v
				inited = true
			} else {
				if v != base {
					fmt.Println("UTXO checksum differs among nodes")

					if verbose {
						for idx := range node_list {
							n := node_list[idx]
							fmt.Printf("%10v: %v...\n", n, utxohash[n][0:16])
						}
					}
					return
				}
			}
		}
		fmt.Println("UTXO checksum " + base[0:16] + "... is consistent across nodes")
	} else {
		fmt.Println("Failed to query some of the nodes")
	}
}
