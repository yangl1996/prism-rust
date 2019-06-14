package main

import (
	"github.com/ziutek/rrd"
	"fmt"
	"time"
	"os"
	"bufio"
	"strings"
)

func main() {
	fmt.Println("Parsing node list")
	nodes := make(map[string]string)
	file, err := os.Open("nodes.txt")
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
		url := fmt.Sprintf("http://%v:%v/telematics/snapshot", ip, port)
		nodes[name] = url
	}
	if err := scanner.Err(); err != nil {
		fmt.Println("Error reading node list:", err)
		os.Exit(1)
	}

	fmt.Println("Creating round-robin database")
	err = os.MkdirAll("data", os.ModeDir)
	if err != nil {
		fmt.Println("Error creating data directory", err)
		os.Exit(1)
	}
	for k, _ := range nodes {
		n := "data/" + k + ".rrd"
		c := rrd.NewCreator(n, time.Now(), 10)
		c.DS("confirmed_tx", "COUNTER", 20, 0, "U")
		c.DS("deconfirmed_tx", "COUNTER", 20, 0, "U")
		c.DS("generated_tx", "COUNTER", 20, 0, "U")
		c.RRA("LAST", 0, 1, 4320)	// collect 4320 datapoints (12 hours)
		err = c.Create(true)
		if err != nil {
			fmt.Println("Error creating round-robin database:", err)
			os.Exit(1)
		}
	}

}
