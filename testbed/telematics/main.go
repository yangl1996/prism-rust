package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"github.com/ziutek/rrd"
	"net/http"
	"os"
	"strings"
	"time"
	"flag"
)

type Snapshot struct {
	Generated_transactions   int
	Confirmed_transactions   int
	Deconfirmed_transactions int
	Incoming_message_queue   int
}

func main() {
	logCommand := flag.NewFlagSet("log", flag.ExitOnError)
	intervalFlag := logCommand.Uint("interval", 1, "Sets the interval between data points")
	durationFlag := logCommand.Uint("duration", 3600, "Sets the duration of the log")
	nodeListFlag := logCommand.String("nodes", "nodes.txt", "Sets the path to the node list")
	dataDirFlag := logCommand.String("datadir", "data", "Sets the path to the directory to hold data")

	if len(os.Args) < 2 {
		fmt.Println("Subcommands: log")
		os.Exit(1)
	}

	switch os.Args[1] {
	case "log":
		logCommand.Parse(os.Args[2:])
		log(*intervalFlag, *durationFlag, *nodeListFlag, *dataDirFlag)
	default:
		fmt.Println("Subcommands: log")
		os.Exit(1)
	}
}

func log(interval, duration uint, nodesFile, dataDir string) {
	fmt.Println("Parsing node list")
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
		url := fmt.Sprintf("http://%v:%v/telematics/snapshot", ip, port)
		nodes[name] = url
	}
	if err := scanner.Err(); err != nil {
		fmt.Println("Error reading node list:", err)
		os.Exit(1)
	}

	fmt.Println("Creating round-robin database")
	err = os.MkdirAll(dataDir, os.ModeDir | os.FileMode(0755))
	if err != nil {
		fmt.Println("Error creating data directory", err)
		os.Exit(1)
	}
	for k, _ := range nodes {
		n := "data/" + k + ".rrd"
		c := rrd.NewCreator(n, time.Now(), interval)
		c.DS("confirmed_tx", "COUNTER", interval * 2, 0, "U")
		c.DS("deconfirmed_tx", "COUNTER", interval * 2, 0, "U")
		c.DS("generated_tx", "COUNTER", interval * 2, 0, "U")
		c.DS("queue_length", "GAUGE", interval * 2, 0, "U")
		c.RRA("LAST", 0, interval, duration / interval) // collect 3600 data points
		err = c.Create(true)
		if err != nil {
			fmt.Println("Error creating round-robin database:", err)
			os.Exit(1)
		}
	}

	fmt.Println("Start logging data")
	for k, v := range nodes {
		monitor(k, v, interval)
	}

	select {}
}

func monitor(node string, url string, interval uint) {
	ticker := time.NewTicker(time.Duration(interval) * time.Second)
	updater := rrd.NewUpdater("data/" + node + ".rrd")
	go func() {
		for range ticker.C {
			resp, err := http.Get(url)
			if err != nil {
				continue // the node is not running yet
			}
			defer resp.Body.Close()

			snapshot := Snapshot{}
			err = json.NewDecoder(resp.Body).Decode(&snapshot)
			if err != nil {
				continue
			}
			err = updater.Update(time.Now(), snapshot.Confirmed_transactions, snapshot.Deconfirmed_transactions, snapshot.Generated_transactions, snapshot.Incoming_message_queue)
			if err != nil {
				fmt.Println("Error updating round-robin database:", err)
			}
		}
	}()
}
