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
	"path"
	tm "github.com/buger/goterm"
)

type Snapshot struct {
	Generated_transactions   int
	Confirmed_transactions   int
	Deconfirmed_transactions int
	Incoming_message_queue   int
	Mined_proposer_blocks    int
	Mined_voter_blocks       int
	Mined_transaction_blocks int
	Total_proposer_block_delay int
	Total_voter_block_delay int
	Total_transaction_block_delay int
	Received_proposer_blocks int
	Received_voter_blocks int
	Received_transaction_blocks int
}

type Report struct {
	Node string
	Data Snapshot
}

func main() {
	logCommand := flag.NewFlagSet("log", flag.ExitOnError)
	intervalFlag := logCommand.Uint("interval", 1, "Sets the interval between data points")
	durationFlag := logCommand.Uint("duration", 3600, "Sets the duration of the log")
	nodeListFlag := logCommand.String("nodelist", "nodes.txt", "Sets the path to the node list file")
	dataDirFlag := logCommand.String("datadir", "data", "Sets the path to the directory to hold data")

	plotCommand := flag.NewFlagSet("plot", flag.ExitOnError)
	plotNodeListFlag := plotCommand.String("nodelist", "nodes.txt", "Sets the path to the node list file")
	plotDataDirFlag := plotCommand.String("datadir", "data", "Sets the path to the directory holding RRD files")
	plotContentFlag := plotCommand.String("content", "txrate", "Sets the content to plot, possible values are txrate, blockdelay, messagequeue, mining")
	plotNodeFlag := plotCommand.String("node", "node_0", "Sets the node to plot")
	plotStepFlag := plotCommand.Uint("step", 1, "Sets the step of the plot")
	plotOutputFlag := plotCommand.String("output", "output.png", "Sets the output path")

	if len(os.Args) < 2 {
		fmt.Println("Subcommands: log, plot")
		os.Exit(1)
	}

	switch os.Args[1] {
	case "log":
		logCommand.Parse(os.Args[2:])
		log(*intervalFlag, *durationFlag, *nodeListFlag, *dataDirFlag)
	case "plot":
		plotCommand.Parse(os.Args[2:])
		plot(*plotNodeListFlag, *plotDataDirFlag, *plotContentFlag, *plotNodeFlag, *plotOutputFlag, *plotStepFlag)
	default:
		fmt.Println("Subcommands: log")
		os.Exit(1)
	}
}

func plot(nodesFile, dataDir, content, node, output string, step uint) {
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
		p := path.Clean(dataDir + "/" + name + ".rrd")
		nodes[name] = p
	}
	if err := scanner.Err(); err != nil {
		fmt.Println("Error reading node list:", err)
		os.Exit(1)
	}
	if _, ok := nodes[node]; !ok {
		fmt.Println("Node", node, "does not exist")
		os.Exit(1)
	}
	g := rrd.NewGrapher()
	g.SetSize(800, 300)
	switch content {
	case "txrate":
		// create def of generated tx for each node, and create a cdef to sum them up
		genSum := ""
		for n, p := range nodes {
			g.Def(n + "_gen", p, "generated_tx", "AVERAGE", fmt.Sprintf("step=%v", step))
			if genSum == "" {
				genSum = n + "_gen"
			} else {
				genSum += "," + n + "_gen,+"
			}
		}
		g.CDef("gen_sum", genSum)
		// create def of confirmed tx for the node we care about
		g.Def(node + "_confirm", nodes[node], "confirmed_tx", "AVERAGE", fmt.Sprintf("step=%v", step))
		// plot the lines
		g.Line(1.0, "gen_sum", "00FF00")
		g.Line(1.0, node + "_confirm", "FF0000")
	case "blockdelay":
	case "messagequeue":
	case "mining":
	default:
		fmt.Println("Plot content options: txrate, blockdelay, messagequeue, mining")
		os.Exit(1)
	}
	_, e := g.SaveGraph(output, time.Now().Add(-time.Duration(600) * time.Second), time.Now())
	if e != nil {
		fmt.Println("Error plotting data:", e)
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
		n := path.Clean(dataDir + "/" + k + ".rrd")
		c := rrd.NewCreator(n, time.Now(), interval)
		c.DS("confirmed_tx", "COUNTER", interval * 2, 0, "U")
		c.DS("deconfirmed_tx", "COUNTER", interval * 2, 0, "U")
		c.DS("generated_tx", "COUNTER", interval * 2, 0, "U")
		c.DS("queue_length", "GAUGE", interval * 2, 0, "U")
		c.DS("mined_proposer", "COUNTER", interval * 2, 0, "U")
		c.DS("mined_voter", "COUNTER", interval * 2, 0, "U")
		c.DS("mined_transaction", "COUNTER", interval * 2, 0, "U")
		c.DS("proposer_delay_sum", "COUNTER", interval * 2, 0, "U")
		c.DS("voter_delay_sum", "COUNTER", interval * 2, 0, "U")
		c.DS("tx_delay_sum", "COUNTER", interval * 2, 0, "U")
		c.DS("received_proposer", "COUNTER", interval * 2, 0, "U")
		c.DS("received_voter", "COUNTER", interval * 2, 0, "U")
		c.DS("received_tx", "COUNTER", interval * 2, 0, "U")
		c.DS("proposer_delay_mean", "COMPUTE", "proposer_delay_sum,received_proposer,/")
		c.DS("voter_delay_mean", "COMPUTE", "voter_delay_sum,received_voter,/")
		c.DS("tx_delay_mean", "COMPUTE", "tx_delay_sum,received_tx,/")
		c.RRA("LAST", 0, 1, duration / interval)
		err = c.Create(true)
		if err != nil {
			fmt.Println("Error creating round-robin database:", err)
			os.Exit(1)
		}
	}

	fmt.Println("Start logging data")
	c := make(chan Report)
	for k, v := range nodes {
		monitor(k, v, interval, c)
	}

	// start displaying data
	t := time.NewTicker(time.Duration(interval) * time.Second)
	prev := make(map[string]Snapshot)
	curr := make(map[string]Snapshot)
	start := time.Now()
	go func() {
		for {
			select {
			case r := <-c:
				cv, cvp := curr[r.Node]
				if cvp {
					prev[r.Node] = cv
				}
				curr[r.Node] = r.Data
			case now := <-t.C:
				if len(curr) == 0 || len(prev) == 0 {
					continue
				}
				// calculate average among nodes
				ctot := Snapshot{}
				for _, v := range curr {
					ctot.Generated_transactions += v.Generated_transactions
					ctot.Confirmed_transactions += v.Confirmed_transactions
					ctot.Deconfirmed_transactions += v.Deconfirmed_transactions
					ctot.Incoming_message_queue += v.Incoming_message_queue
					ctot.Mined_proposer_blocks += v.Mined_proposer_blocks
					ctot.Mined_voter_blocks += v.Mined_voter_blocks
					ctot.Mined_transaction_blocks += v.Mined_transaction_blocks
					ctot.Total_proposer_block_delay += v.Total_proposer_block_delay
					ctot.Total_voter_block_delay += v.Total_voter_block_delay
					ctot.Total_transaction_block_delay += v.Total_transaction_block_delay
					ctot.Received_proposer_blocks += v.Received_proposer_blocks
					ctot.Received_voter_blocks += v.Received_voter_blocks
					ctot.Received_transaction_blocks += v.Received_transaction_blocks
				}
				cavg := Snapshot {
					Generated_transactions: ctot.Generated_transactions,
					Confirmed_transactions: ctot.Confirmed_transactions / len(curr),
					Deconfirmed_transactions: ctot.Deconfirmed_transactions / len(curr),
					Incoming_message_queue: ctot.Incoming_message_queue / len(curr),
					Mined_proposer_blocks: ctot.Mined_proposer_blocks,
					Mined_voter_blocks: ctot.Mined_voter_blocks,
					Mined_transaction_blocks: ctot.Mined_transaction_blocks,
					Total_proposer_block_delay: ctot.Total_proposer_block_delay,
					Total_voter_block_delay: ctot.Total_voter_block_delay,
					Total_transaction_block_delay: ctot.Total_transaction_block_delay,
					Received_proposer_blocks: ctot.Received_proposer_blocks,
					Received_voter_blocks: ctot.Received_voter_blocks,
					Received_transaction_blocks: ctot.Received_transaction_blocks,
				}
				ptot := Snapshot{}
				for _, v := range prev {
					ptot.Generated_transactions += v.Generated_transactions
					ptot.Confirmed_transactions += v.Confirmed_transactions
					ptot.Deconfirmed_transactions += v.Deconfirmed_transactions
					ptot.Incoming_message_queue += v.Incoming_message_queue
					ptot.Mined_proposer_blocks += v.Mined_proposer_blocks
					ptot.Mined_voter_blocks += v.Mined_voter_blocks
					ptot.Mined_transaction_blocks += v.Mined_transaction_blocks
					ptot.Total_proposer_block_delay += v.Total_proposer_block_delay
					ptot.Total_voter_block_delay += v.Total_voter_block_delay
					ptot.Total_transaction_block_delay += v.Total_transaction_block_delay
					ptot.Received_proposer_blocks += v.Received_proposer_blocks
					ptot.Received_voter_blocks += v.Received_voter_blocks
					ptot.Received_transaction_blocks += v.Received_transaction_blocks
				}
				pavg := Snapshot {
					Generated_transactions: ptot.Generated_transactions,
					Confirmed_transactions: ptot.Confirmed_transactions / len(prev),
					Deconfirmed_transactions: ptot.Deconfirmed_transactions / len(prev),
					Incoming_message_queue: ptot.Incoming_message_queue / len(prev),
					Mined_proposer_blocks: ptot.Mined_proposer_blocks,
					Mined_voter_blocks: ptot.Mined_voter_blocks,
					Mined_transaction_blocks: ptot.Mined_transaction_blocks,
					Total_proposer_block_delay: ptot.Total_proposer_block_delay,
					Total_voter_block_delay: ptot.Total_voter_block_delay,
					Total_transaction_block_delay: ptot.Total_transaction_block_delay,
					Received_proposer_blocks: ptot.Received_proposer_blocks,
					Received_voter_blocks: ptot.Received_voter_blocks,
					Received_transaction_blocks: ptot.Received_transaction_blocks,
				}
				// display the values
				tm.Clear()
				tm.MoveCursor(1, 1)
				dur := int(now.Sub(start).Seconds())
				tm.Printf("Experiment duration - %v sec\n", dur)
				tm.Printf("                            %8v  %8v\n", "Overall", "Window")
				tm.Printf("  Generated Transactions    %8v  %8v\n", cavg.Generated_transactions / dur, (cavg.Generated_transactions - pavg.Generated_transactions) / int(interval))
				tm.Printf("  Confirmed Transactions    %8v  %8v\n", cavg.Confirmed_transactions / dur, (cavg.Confirmed_transactions - pavg.Confirmed_transactions) / int(interval))
				tm.Printf("Deconfirmed Transactions    %8v  %8v\n", cavg.Deconfirmed_transactions / dur, (cavg.Deconfirmed_transactions - pavg.Deconfirmed_transactions) / int(interval))
				tm.Printf("            Queue Length    %8v  %8v\n", cavg.Incoming_message_queue, (cavg.Incoming_message_queue - pavg.Incoming_message_queue) / int(interval))
				tm.Printf("    Mining -    Proposer    %8.3g  %8.3g\n", float64(cavg.Mined_proposer_blocks) / float64(dur), float64(cavg.Mined_proposer_blocks - pavg.Mined_proposer_blocks) / float64(interval))
				tm.Printf("    Mining -       Voter    %8.3g  %8.3g\n", float64(cavg.Mined_voter_blocks) / float64(dur), float64(cavg.Mined_voter_blocks - pavg.Mined_voter_blocks) / float64(interval))
				tm.Printf("    Mining - Transaction    %8.3g  %8.3g\n", float64(cavg.Mined_transaction_blocks) / float64(dur), float64(cavg.Mined_transaction_blocks - pavg.Mined_transaction_blocks) / float64(interval))
				tm.Printf("     Delay -    Proposer    %8.3g  %8.3g\n", float64(cavg.Total_proposer_block_delay) / float64(cavg.Received_proposer_blocks), float64(cavg.Total_proposer_block_delay - pavg.Total_proposer_block_delay) / float64(cavg.Received_proposer_blocks - pavg.Received_proposer_blocks))
				tm.Printf("     Delay -       Voter    %8.3g  %8.3g\n", float64(cavg.Total_voter_block_delay) / float64(cavg.Received_voter_blocks), float64(cavg.Total_voter_block_delay - pavg.Total_voter_block_delay) / float64(cavg.Received_voter_blocks - pavg.Received_voter_blocks))
				tm.Printf("     Delay - Transaction    %8.3g  %8.3g\n", float64(cavg.Total_transaction_block_delay) / float64(cavg.Received_transaction_blocks), float64(cavg.Total_transaction_block_delay - pavg.Total_transaction_block_delay) / float64(cavg.Received_transaction_blocks - pavg.Received_transaction_blocks))
				tm.Flush()
			}
		}
	}()

	select {}
}

func monitor(node string, url string, interval uint, datachan chan Report) {
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
			err = updater.Update(time.Now(), snapshot.Confirmed_transactions, snapshot.Deconfirmed_transactions, snapshot.Generated_transactions, snapshot.Incoming_message_queue, snapshot.Mined_proposer_blocks, snapshot.Mined_voter_blocks, snapshot.Mined_transaction_blocks, snapshot.Total_proposer_block_delay, snapshot.Total_voter_block_delay, snapshot.Total_transaction_block_delay, snapshot.Received_proposer_blocks, snapshot.Received_voter_blocks, snapshot.Received_transaction_blocks)
			if err != nil {
				// sometimes we get error if interval is set to 1 and the timer goes a bit faster
				continue
			}
			datachan <- Report{Node: node, Data: snapshot}
		}
	}()
}
