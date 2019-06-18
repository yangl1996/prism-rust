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
	Hash string
}

func check(nodesFile string, verbose bool) {
	nodes := make(map[string]string)
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
	}
	if err := scanner.Err(); err != nil {
		fmt.Println("Error reading node list:", err)
		os.Exit(1)
	}

	// check balance
	balance := make(map[string]uint)
	failed := false
	var m sync.Mutex
	var wg sync.WaitGroup
	for k, v := range nodes {
		url := v + "/wallet/balance"
		node := k
		wg.Add(1)
		go func(node, url string) {
			defer wg.Done()
			resp, err := http.Get(url)
			if err != nil {
				m.Lock()
				failed = true
				m.Unlock()
				return // the node is not running yet
			}
			defer resp.Body.Close()

			data := WalletBalance{}
			err = json.NewDecoder(resp.Body).Decode(&data)
			if err != nil {
				m.Lock()
				failed = true
				m.Unlock()
				return
			}
			m.Lock()
			balance[node] = data.Balance
			m.Unlock()
		}(node, url)
	}
	wg.Wait()
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
				for k, v := range balance {
					fmt.Printf("%v: %v\n", k, v)
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
			utxohash[node] = data.Hash
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
					fmt.Println("UTXO hash differs among nodes")

					if verbose {
						for k, v := range utxohash {
							fmt.Printf("%v: %v\n", k, v)
						}
					}
					return
				}
			}
		}
		fmt.Println("UTXO hash " + base + " is consistent across nodes")
	} else {
		fmt.Println("Failed to query some of the nodes")
	}
}

